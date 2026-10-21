// settings : réglages tactiles pour lapp (écran portrait 272x480), rendu OpenGL ES (repli CPU).
//
// - Luminosité : /sys/class/backlight/* ; Volume : PipeWire (wpctl), repli ALSA (amixer) ; Informations : appareil, noyau, mémoire, stockage, batterie, IP
// - quitter : glisser vers le haut depuis le bord bas de l'écran, ou Echap
// Permissions : display, system (/sys en écriture + /dev/snd), network (adresse IP), storage.
// Tests hors écran : settings --screenshot out.png [--page info] ; settings --icon icon.png

mod gfx;

use std::fs;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use ab_glyph::{point, Font, FontVec, PxScale, ScaleFont};
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Rect, Transform};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, KeyEvent, MouseButton, TouchPhase, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::raw_window_handle::HasWindowHandle;
use winit::window::{Window, WindowId};

use gfx::Gfx;

type Rgb = [f32; 3];

const BG: Rgb = [0.97, 0.97, 0.985];
const BAR: Rgb = [0.985, 0.985, 0.995];
const WHITE: Rgb = [1.0, 1.0, 1.0];
const BLUE: Rgb = [0.0, 0.478, 1.0];
const DARK: Rgb = [0.11, 0.11, 0.13];
const GREY: Rgb = [0.55, 0.55, 0.6];
const LINE: Rgb = [0.80, 0.80, 0.83];
const FOLDER: Rgb = [0.33, 0.67, 0.96];
const FOLDER_TAB: Rgb = [0.22, 0.56, 0.90];
const PRESS: Rgb = [0.84, 0.88, 0.95];

const HEADER: f32 = 44.0;
const PATHBAR: f32 = 20.0;
const ROW: f32 = 54.0;
const PAD_BOTTOM: f32 = 18.0;

// ------------------------------------------------------------------ dessin (pixmaps)

fn col(c: Rgb, a: f32) -> Color {
    Color::from_rgba(c[0], c[1], c[2], a.clamp(0.0, 1.0)).unwrap()
}

fn fill_rect(pm: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, c: Color) {
    if let Some(r) = Rect::from_xywh(x, y, w.max(0.01), h.max(0.01)) {
        let mut p = Paint::default();
        p.set_color(c);
        pm.fill_rect(r, &p, Transform::identity(), None);
    }
}

fn fill_rrect(pm: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) {
    let r = r.min(w / 2.0).min(h / 2.0);
    let k = r * 0.552_284_7;
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    pb.close();
    if let Some(p) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(c);
        paint.anti_alias = true;
        pm.fill_path(&p, &paint, FillRule::Winding, Transform::identity(), None);
    }
}

fn fill_poly(pm: &mut Pixmap, pts: &[(f32, f32)], c: Color) {
    let mut pb = PathBuilder::new();
    for (i, (x, y)) in pts.iter().enumerate() {
        if i == 0 {
            pb.move_to(*x, *y);
        } else {
            pb.line_to(*x, *y);
        }
    }
    pb.close();
    if let Some(p) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(c);
        paint.anti_alias = true;
        pm.fill_path(&p, &paint, FillRule::Winding, Transform::identity(), None);
    }
}

fn draw_folder(pm: &mut Pixmap, x: f32, y: f32, sz: f32) {
    fill_rrect(pm, x, y + sz * 0.12, sz * 0.46, sz * 0.3, sz * 0.09, col(FOLDER_TAB, 1.0));
    fill_rrect(pm, x, y + sz * 0.26, sz, sz * 0.64, sz * 0.1, col(FOLDER, 1.0));
    fill_rrect(pm, x, y + sz * 0.26, sz, sz * 0.08, sz * 0.04, col(WHITE, 0.25));
}

fn draw_file(pm: &mut Pixmap, x: f32, y: f32, sz: f32, tint: Rgb) {
    let (w, h) = (sz * 0.76, sz);
    let x = x + (sz - w) / 2.0;
    let fold = w * 0.32;
    fill_rrect(pm, x, y, w, h, sz * 0.07, col(LINE, 1.0));
    let b = sz * 0.05;
    fill_rrect(pm, x + b, y + b, w - 2.0 * b, h - 2.0 * b, sz * 0.05, col(WHITE, 1.0));
    // coin plié
    fill_poly(pm, &[(x + w - fold, y), (x + w, y + fold), (x + w - fold, y + fold)], col(LINE, 1.0));
    fill_poly(pm, &[(x + w - fold, y), (x + w, y), (x + w, y + fold)], col(BG, 1.0));
    // lignes de texte
    for i in 0..3 {
        let ly = y + h * (0.46 + 0.16 * i as f32);
        fill_rrect(pm, x + w * 0.18, ly, w * (0.64 - if i == 2 { 0.14 } else { 0.0 }), h * 0.05, h * 0.025, col(tint, 0.75));
    }
}

fn folder_pm(sz: f32) -> Pixmap {
    let n = sz.ceil() as u32 + 1;
    let mut pm = Pixmap::new(n, n).unwrap();
    draw_folder(&mut pm, 0.0, 0.0, sz);
    pm
}

fn file_pm(sz: f32, tint: Rgb) -> Pixmap {
    let n = sz.ceil() as u32 + 1;
    let mut pm = Pixmap::new(n, n).unwrap();
    draw_file(&mut pm, 0.0, 0.0, sz, tint);
    pm
}

struct Fonts {
    reg: Option<FontVec>,
    bold: Option<FontVec>,
    mono: Option<FontVec>,
}

#[derive(Clone, Copy)]
enum Kind {
    Reg,
    Bold,
    Mono,
}

#[derive(Clone, Copy, PartialEq)]
enum Align {
    Left,
    Center,
    Right,
}

fn first_font(paths: &[&str]) -> Option<FontVec> {
    paths.iter().find_map(|p| FontVec::try_from_vec(fs::read(p).ok()?).ok())
}

fn load_fonts() -> Fonts {
    let mut reg = std::env::var("LAPP_FONT").ok().and_then(|p| first_font(&[p.as_str()]));
    if reg.is_none() {
        reg = first_font(&[
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/liberation2/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
        ]);
    }
    let bold = first_font(&[
        "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
        "/usr/share/fonts/truetype/liberation2/LiberationSans-Bold.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
    ]);
    let mono = first_font(&[
        "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
        "/usr/share/fonts/truetype/liberation2/LiberationMono-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
        "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
    ]);
    if reg.is_none() {
        eprintln!("settings : aucune police trouvée, texte désactivé");
    }
    Fonts { reg, bold, mono }
}

impl Fonts {
    fn get(&self, k: Kind) -> Option<&FontVec> {
        match k {
            Kind::Reg => self.reg.as_ref(),
            Kind::Bold => self.bold.as_ref().or(self.reg.as_ref()),
            Kind::Mono => self.mono.as_ref().or(self.reg.as_ref()),
        }
    }
}

fn em_scale(font: &FontVec, px: f32) -> PxScale {
    PxScale::from(px * font.height_unscaled() / font.units_per_em().unwrap_or(1000.0))
}

fn text_width(font: &FontVec, scale: PxScale, s: &str) -> f32 {
    let sf = font.as_scaled(scale);
    let mut w = 0.0;
    let mut prev = None;
    for ch in s.chars() {
        let id = font.glyph_id(ch);
        if let Some(p) = prev {
            w += sf.kern(p, id);
        }
        w += sf.h_advance(id);
        prev = Some(id);
    }
    w
}

/// Raccourcit `s` avec une ellipse pour tenir dans `max` pixels (ellipse au début si `left`).
fn fit(font: &FontVec, scale: PxScale, s: &str, max: f32, left: bool) -> String {
    if text_width(font, scale, s) <= max {
        return s.to_string();
    }
    let mut chars: Vec<char> = s.chars().collect();
    while !chars.is_empty() {
        if left {
            chars.remove(0);
        } else {
            chars.pop();
        }
        let body: String = chars.iter().collect();
        let t = if left { format!("…{body}") } else { format!("{body}…") };
        if text_width(font, scale, &t) <= max {
            return t;
        }
    }
    String::from("…")
}

/// Rastérise `s` dans une petite image transparente (marge de 1 px, prémultipliée).
fn text_pm(fonts: &Fonts, s: &str, px: f32, kind: Kind, rgb: Rgb, max_w: Option<f32>, ell_left: bool) -> Pixmap {
    let Some(font) = fonts.get(kind) else { return Pixmap::new(1, 1).unwrap() };
    let scale = em_scale(font, px);
    let owned;
    let s = match max_w {
        Some(m) => {
            owned = fit(font, scale, s, m, ell_left);
            owned.as_str()
        }
        None => s,
    };
    let sf = font.as_scaled(scale);
    let tw = text_width(font, scale, s);
    let w = (tw.ceil() as u32 + 3).clamp(1, 4096);
    let h = ((sf.ascent() - sf.descent()).ceil() as u32 + 3).clamp(1, 512);
    let mut pm = Pixmap::new(w, h).unwrap();
    let (pw, ph) = (w as i32, h as i32);
    let baseline = 1.0 + sf.ascent();
    let data = pm.data_mut();
    let mut x = 1.0;
    let mut prev = None;
    for ch in s.chars() {
        let id = font.glyph_id(ch);
        if let Some(p) = prev {
            x += sf.kern(p, id);
        }
        let g = id.with_scale_and_position(scale, point(x, baseline));
        x += sf.h_advance(id);
        prev = Some(id);
        if let Some(og) = font.outline_glyph(g) {
            let b = og.px_bounds();
            let (bx, by) = (b.min.x as i32, b.min.y as i32);
            og.draw(|gx, gy, cov| {
                let (px_, py_) = (bx + gx as i32, by + gy as i32);
                if px_ < 0 || py_ < 0 || px_ >= pw || py_ >= ph {
                    return;
                }
                let a = cov.clamp(0.0, 1.0);
                let i = ((py_ * pw + px_) * 4) as usize;
                for c in 0..3 {
                    data[i + c] = (data[i + c] as f32 * (1.0 - a) + rgb[c] * 255.0 * a) as u8;
                }
                data[i + 3] = (data[i + 3] as f32 * (1.0 - a) + 255.0 * a) as u8;
            });
        }
    }
    pm
}

// ------------------------------------------------------------------ système

use std::ffi::CString;
use std::process::Command;

fn read_trim(p: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(p).ok().map(|s| s.trim().to_string())
}

fn ioerr(e: &std::io::Error) -> String {
    match e.kind() {
        std::io::ErrorKind::PermissionDenied => "accès refusé".into(),
        std::io::ErrorKind::NotFound => "introuvable".into(),
        _ => e.to_string(),
    }
}

/// Rétroéclairage (/sys/class/backlight/*).
struct Backlight {
    dir: PathBuf,
    max: u32,
}

fn find_backlight() -> Option<Backlight> {
    let mut v: Vec<_> = fs::read_dir("/sys/class/backlight").ok()?.flatten().collect();
    v.sort_by_key(|e| e.file_name());
    for e in v {
        let d = e.path();
        let max = read_trim(d.join("max_brightness")).and_then(|s| s.parse::<u32>().ok());
        if let Some(max) = max {
            if max > 0 && d.join("brightness").exists() {
                return Some(Backlight { dir: d, max });
            }
        }
    }
    None
}

impl Backlight {
    fn get(&self) -> Option<f32> {
        let b: u32 = read_trim(self.dir.join("brightness"))?.parse().ok()?;
        Some(b as f32 * 100.0 / self.max as f32)
    }

    fn set(&self, pct: f32) -> Result<(), String> {
        let v = ((pct.clamp(3.0, 100.0) / 100.0) * self.max as f32).round() as u32;
        fs::write(self.dir.join("brightness"), v.to_string()).map_err(|e| format!("Luminosité : {}", ioerr(&e)))
    }
}

/// Volume via ALSA (`amixer`), premier contrôle qui répond.
fn amixer(args: &[&str]) -> Option<String> {
    let o = Command::new("amixer").args(args).output().ok()?;
    if !o.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&o.stdout).into_owned())
}

fn parse_pct(s: &str) -> Option<f32> {
    for part in s.split('[') {
        if let Some(e) = part.find("%]") {
            if let Ok(v) = part[..e].parse::<f32>() {
                return Some(v);
            }
        }
    }
    None
}

fn wpctl(args: &[&str]) -> Option<String> {
    let o = Command::new("wpctl").args(args).output().ok()?;
    if !o.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&o.stdout).into_owned())
}

/// Volume réel du système (PipeWire, le même que celui des touches volume) : "Volume: 0.45 [MUTED]".
fn wpctl_get() -> Option<f32> {
    let out = wpctl(&["get-volume", "@DEFAULT_AUDIO_SINK@"])?;
    let v: f32 = out.split_whitespace().nth(1)?.parse().ok()?;
    Some((v * 100.0).clamp(0.0, 100.0))
}

fn vol_get() -> Option<(String, f32)> {
    if let Some(p) = wpctl_get() {
        return Some(("wpctl".to_string(), p));
    }
    for c in ["Master", "PCM", "Speaker", "Headphone", "Playback"] {
        if let Some(out) = amixer(&["sget", c]) {
            if let Some(p) = parse_pct(&out) {
                return Some((c.to_string(), p));
            }
        }
    }
    None
}

fn vol_set(ctrl: &str, pct: f32) -> Result<(), String> {
    if ctrl == "wpctl" {
        let v = format!("{:.2}", (pct / 100.0).clamp(0.0, 1.0));
        wpctl(&["set-volume", "-l", "1.0", "@DEFAULT_AUDIO_SINK@", &v]).ok_or_else(|| "Volume : réglage impossible".to_string())?;
        if pct > 0.0 {
            let _ = wpctl(&["set-mute", "@DEFAULT_AUDIO_SINK@", "0"]);
        }
        return Ok(());
    }
    amixer(&["-q", "sset", ctrl, &format!("{}%", pct.round() as i32)]).map(|_| ()).ok_or_else(|| "Volume : réglage impossible".to_string())
}

// ------------------------------------------------------------------ informations

fn fmt_bytes(n: u64) -> String {
    let g = n as f64 / 1_073_741_824.0;
    if g >= 1.0 {
        format!("{g:.1} Go")
    } else {
        format!("{:.0} Mo", n as f64 / 1_048_576.0)
    }
}

fn fmt_uptime(s: u64) -> String {
    let (d, h, m) = (s / 86400, s % 86400 / 3600, s % 3600 / 60);
    if d > 0 {
        format!("{d} j {h} h {m:02} min")
    } else if h > 0 {
        format!("{h} h {m:02} min")
    } else {
        format!("{m} min")
    }
}

fn mem_info() -> Option<String> {
    let t = fs::read_to_string("/proc/meminfo").ok()?;
    let get = |k: &str| t.lines().find(|l| l.starts_with(k)).and_then(|l| l.split_whitespace().nth(1)).and_then(|v| v.parse::<u64>().ok());
    let (tot, av) = (get("MemTotal:")?, get("MemAvailable:")?);
    Some(format!("{} / {}", fmt_bytes((tot - av.min(tot)) * 1024), fmt_bytes(tot * 1024)))
}

fn disk_info(path: &str) -> Option<String> {
    let c = CString::new(path).ok()?;
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c.as_ptr(), &mut st) } != 0 {
        return None;
    }
    let bs = st.f_frsize as u64;
    let (tot, free) = (st.f_blocks as u64 * bs, st.f_bavail as u64 * bs);
    if tot == 0 {
        return None;
    }
    Some(format!("{} libres / {}", fmt_bytes(free), fmt_bytes(tot)))
}

fn battery_info() -> Option<String> {
    for e in fs::read_dir("/sys/class/power_supply").ok()?.flatten() {
        if !e.file_name().to_string_lossy().starts_with("BAT") {
            continue;
        }
        let p = e.path();
        let c = read_trim(p.join("capacity"))?;
        let st = read_trim(p.join("status")).unwrap_or_default();
        let fr = match st.as_str() {
            "Charging" => "en charge",
            "Discharging" => "sur batterie",
            "Full" => "pleine",
            _ => "",
        };
        return Some(if fr.is_empty() { format!("{c} %") } else { format!("{c} % · {fr}") });
    }
    None
}

fn local_ips() -> Vec<String> {
    let Ok(t) = fs::read_to_string("/proc/net/fib_trie") else { return vec![] };
    let mut last = String::new();
    let mut out: Vec<String> = vec![];
    for l in t.lines() {
        let l = l.trim();
        if let Some(ip) = l.strip_prefix("|-- ") {
            last = ip.to_string();
        } else if l.contains("/32 host LOCAL") && !last.starts_with("127.") && !out.contains(&last) {
            out.push(last.clone());
        }
    }
    out
}

fn info_rows() -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = vec![];
    let mut add = |k: &str, val: Option<String>| {
        if let Some(x) = val {
            if !x.is_empty() {
                v.push((k.to_string(), x));
            }
        }
    };
    add("Appareil", read_trim("/proc/device-tree/model").map(|s| s.trim_matches('\0').to_string()));
    add("Nom", read_trim("/proc/sys/kernel/hostname"));
    add("Noyau", read_trim("/proc/sys/kernel/osrelease"));
    add("Architecture", Some(std::env::consts::ARCH.to_string()));
    add("Allumé depuis", read_trim("/proc/uptime").and_then(|s| s.split('.').next().and_then(|x| x.parse::<u64>().ok())).map(fmt_uptime));
    add("Mémoire", mem_info());
    add("Stockage", disk_info("/data"));
    add("Batterie", battery_info());
    add("Température", read_trim("/sys/class/thermal/thermal_zone0/temp").and_then(|s| s.parse::<f32>().ok()).map(|t| format!("{:.0} °C", t / 1000.0)));
    let ips = local_ips();
    add("Adresse IP", if ips.is_empty() { None } else { Some(ips.join(", ")) });
    v
}

// ------------------------------------------------------------------ état

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Main,
    Info,
}

#[derive(Clone, Copy, PartialEq)]
enum Drag {
    Bright,
    Vol,
}

struct GlState {
    surface: Surface<WindowSurface>,
    ctx: PossiblyCurrentContext,
    gfx: Gfx,
}

const CARD_X: f32 = 10.0;
const SL_X0: f32 = 24.0;
const SL_X1: f32 = 248.0;

/// (y, hauteur) des trois cartes de la page principale, en pixels de conception (272 de large).
fn cards() -> [(f32, f32); 3] {
    let y0 = HEADER + 14.0;
    [(y0, 68.0), (y0 + 78.0, 68.0), (y0 + 156.0, 48.0)]
}

struct App {
    window: Option<Rc<Window>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    gl: Option<GlState>,
    cpu: Option<Gfx>,
    surf_size: (u32, u32),
    fonts: Fonts,
    size: (u32, u32),
    cursor: (f32, f32),
    press: Option<(f32, f32, bool)>, // départ x, y, a bougé
    drag: Option<Drag>,
    page: Page,
    bl: Option<Backlight>,
    bright: Option<f32>,
    vol: Option<(String, f32)>,
    info: Vec<(String, String)>,
    toast: Option<(String, Instant)>,
    last_apply: Instant,
}

impl App {
    fn new(size: (u32, u32)) -> App {
        let bl = find_backlight();
        let bright = bl.as_ref().and_then(|b| b.get());
        App {
            window: None,
            surface: None,
            gl: None,
            cpu: None,
            surf_size: (0, 0),
            fonts: load_fonts(),
            size,
            cursor: (0.0, 0.0),
            press: None,
            drag: None,
            page: Page::Main,
            bl,
            bright,
            vol: vol_get(),
            info: Vec::new(),
            toast: None,
            last_apply: Instant::now(),
        }
    }

    fn scale(&self) -> f32 {
        if let Some(s) = scale_override() {
            return s;
        }
        self.size.0.min(self.size.1) as f32 / 272.0
    }

    fn set_toast(&mut self, msg: String) {
        self.toast = Some((msg, Instant::now() + Duration::from_millis(2800)));
    }

    fn redraw_later(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    // -------------------------------------------------------------- interaction

    fn slide_to(&mut self, x: f32, fin: bool) {
        let s = self.scale();
        let pct = ((x - SL_X0 * s) / ((SL_X1 - SL_X0) * s) * 100.0).clamp(0.0, 100.0);
        let ready = fin || self.last_apply.elapsed() > Duration::from_millis(70);
        match self.drag {
            Some(Drag::Bright) => {
                let p = pct.max(3.0);
                self.bright = Some(p);
                if ready {
                    self.last_apply = Instant::now();
                    let r = self.bl.as_ref().map(|b| b.set(p));
                    if let Some(Err(e)) = r {
                        self.set_toast(e);
                    }
                }
            }
            Some(Drag::Vol) => {
                if let Some((c, v)) = self.vol.as_mut() {
                    *v = pct;
                    if fin || self.last_apply.elapsed() > Duration::from_millis(220) {
                        self.last_apply = Instant::now();
                        let (c, p) = (c.clone(), pct);
                        if let Err(e) = vol_set(&c, p) {
                            self.set_toast(e);
                        }
                    }
                }
            }
            None => {}
        }
    }

    fn tap(&mut self, x: f32, y: f32) {
        let s = self.scale();
        if y < HEADER * s {
            if x < 84.0 * s && self.page == Page::Info {
                self.page = Page::Main;
            }
            return;
        }
        if self.page == Page::Main {
            let (cy, ch) = cards()[2];
            if y >= cy * s && y < (cy + ch) * s {
                self.info = info_rows();
                self.page = Page::Info;
            }
        }
    }

    fn press(&mut self, x: f32, y: f32) {
        let s = self.scale();
        self.press = Some((x, y, false));
        self.drag = None;
        if self.page == Page::Main && x >= CARD_X * s && x <= self.size.0 as f32 - CARD_X * s {
            let cs = cards();
            for (i, (cy, ch)) in cs.iter().enumerate().take(2) {
                if y >= (cy + 24.0) * s && y < (cy + ch) * s {
                    let avail = if i == 0 { self.bl.is_some() && self.bright.is_some() } else { self.vol.is_some() };
                    if avail {
                        self.drag = Some(if i == 0 { Drag::Bright } else { Drag::Vol });
                        self.slide_to(x, false);
                    }
                }
            }
        }
        self.redraw_later();
    }

    fn moved(&mut self, x: f32, y: f32) {
        let s = self.scale();
        if self.drag.is_some() {
            self.slide_to(x, false);
        }
        if let Some(p) = self.press.as_mut() {
            if (x - p.0).abs() > 9.0 * s || (y - p.1).abs() > 9.0 * s {
                p.2 = true;
            }
        }
        self.redraw_later();
    }

    fn release(&mut self, x: f32, y: f32, el: Option<&ActiveEventLoop>) {
        let Some(p) = self.press.take() else { return };
        if self.drag.is_some() {
            self.slide_to(x, true);
            self.drag = None;
            self.redraw_later();
            return;
        }
        let h = self.size.1 as f32;
        if p.1 > h * 0.94 && p.1 - y > h * 0.08 {
            if let Some(el) = el {
                el.exit(); // glisser vers le haut depuis le bas de l'écran
            }
        } else if !p.2 {
            self.tap(x, y);
        }
        self.redraw_later();
    }

    // -------------------------------------------------------------- rendu

    fn render(&self, g: &mut Gfx) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let s = self.scale();
        g.rect(0.0, 0.0, w, h, col(BG, 1.0));
        match self.page {
            Page::Main => self.render_main(g, s),
            Page::Info => self.render_info(g, s),
        }
        g.rect(0.0, h - PAD_BOTTOM * s, w, PAD_BOTTOM * s, col(BG, 1.0));
        g.rrect(w * 0.35, h - w * 0.035, w * 0.3, w * 0.012, w * 0.006, col(LINE, 1.0));
        if let Some((msg, _)) = &self.toast {
            let bw = w - 28.0 * s;
            let bh = 34.0 * s;
            let by = h - PAD_BOTTOM * s - bh - 8.0 * s;
            g.rrect(14.0 * s, by, bw, bh, 9.0 * s, col([0.1, 0.1, 0.12], 0.92));
            self.txt(g, msg, w / 2.0, by + (bh - 12.0 * s * 1.17) / 2.0, 12.0 * s, Kind::Reg, WHITE, Align::Center, Some(bw - 16.0 * s), false);
        }
    }

    fn header(&self, g: &mut Gfx, s: f32, title: &str, back: bool) {
        let w = self.size.0 as f32;
        g.rect(0.0, 0.0, w, HEADER * s, col(BAR, 1.0));
        g.rect(0.0, HEADER * s - 1.0, w, 1.0, col(LINE, 1.0));
        if back {
            self.txt(g, "‹ Retour", 10.0 * s, (HEADER * s - 15.0 * s * 1.17) / 2.0, 15.0 * s, Kind::Reg, BLUE, Align::Left, None, false);
        }
        self.txt(g, title, w / 2.0, (HEADER * s - 16.0 * s * 1.17) / 2.0, 16.0 * s, Kind::Bold, DARK, Align::Center, Some(w - 168.0 * s), false);
    }

    fn card(&self, g: &mut Gfx, s: f32, y: f32, h: f32) {
        let w = self.size.0 as f32;
        g.rrect(CARD_X * s, y * s, w - 2.0 * CARD_X * s, h * s, 10.0 * s, col(LINE, 1.0));
        g.rrect(CARD_X * s + 1.0, y * s + 1.0, w - 2.0 * CARD_X * s - 2.0, h * s - 2.0, 9.0 * s, col(WHITE, 1.0));
    }

    fn slider(&self, g: &mut Gfx, s: f32, idx: usize, title: &str, val: Option<f32>) {
        let w = self.size.0 as f32;
        let (y, h) = cards()[idx];
        self.card(g, s, y, h);
        self.txt(g, title, 18.0 * s, (y + 11.0) * s, 14.0 * s, Kind::Reg, DARK, Align::Left, None, false);
        match val {
            Some(v) => {
                self.txt(g, &format!("{:.0} %", v), w - 18.0 * s, (y + 12.0) * s, 13.0 * s, Kind::Reg, GREY, Align::Right, None, false);
                let cy = (y + 48.0) * s;
                let (x0, x1) = (SL_X0 * s, SL_X1 * s);
                g.rrect(x0, cy - 2.0 * s, x1 - x0, 4.0 * s, 2.0 * s, col(LINE, 1.0));
                let kx = x0 + (x1 - x0) * v / 100.0;
                g.rrect(x0, cy - 2.0 * s, (kx - x0).max(0.0), 4.0 * s, 2.0 * s, col(BLUE, 1.0));
                g.rrect(kx - 11.0 * s, cy - 11.0 * s, 22.0 * s, 22.0 * s, 11.0 * s, col(LINE, 1.0));
                g.rrect(kx - 10.0 * s, cy - 10.0 * s, 20.0 * s, 20.0 * s, 10.0 * s, col(WHITE, 1.0));
            }
            None => {
                self.txt(g, "Indisponible", w - 18.0 * s, (y + 12.0) * s, 13.0 * s, Kind::Reg, GREY, Align::Right, None, false);
                g.rrect(SL_X0 * s, (y + 46.0) * s, (SL_X1 - SL_X0) * s, 4.0 * s, 2.0 * s, col(LINE, 0.6));
            }
        }
    }

    fn render_main(&self, g: &mut Gfx, s: f32) {
        let w = self.size.0 as f32;
        self.slider(g, s, 0, "Luminosité", self.bl.as_ref().and(self.bright));
        self.slider(g, s, 1, "Volume", self.vol.as_ref().map(|v| v.1));
        let (y, h) = cards()[2];
        self.card(g, s, y, h);
        self.txt(g, "Informations", 18.0 * s, (y + (h - 14.0 * 1.17) / 2.0) * s, 14.0 * s, Kind::Reg, DARK, Align::Left, None, false);
        self.txt(g, "›", w - 20.0 * s, (y + (h - 20.0 * 1.17) / 2.0) * s, 20.0 * s, Kind::Reg, LINE, Align::Right, None, false);
        self.header(g, s, "Réglages", false);
    }

    fn render_info(&self, g: &mut Gfx, s: f32) {
        let w = self.size.0 as f32;
        let mut y = (HEADER + 6.0) * s;
        for (k, v) in &self.info {
            self.txt(g, k, 14.0 * s, y + 9.0 * s, 12.0 * s, Kind::Reg, GREY, Align::Left, None, false);
            self.txt(g, v, w - 14.0 * s, y + 9.0 * s, 12.0 * s, Kind::Reg, DARK, Align::Right, Some(w - 28.0 * s - 92.0 * s), true);
            y += 34.0 * s;
            g.rect(14.0 * s, y - 1.0, w - 28.0 * s, 1.0, col(LINE, 0.6));
        }
        if self.info.is_empty() {
            self.txt(g, "Aucune information", w / 2.0, y + 40.0 * s, 13.0 * s, Kind::Reg, GREY, Align::Center, None, false);
        }
        self.header(g, s, "Informations", true);
    }

    fn redraw(&mut self) {
        let Some(window) = self.window.clone() else { return };
        let sz = window.inner_size();
        let (Some(nw), Some(nh)) = (NonZeroU32::new(sz.width), NonZeroU32::new(sz.height)) else { return };
        self.size = (sz.width, sz.height);
        if let Some(mut gs) = self.gl.take() {
            if self.surf_size != self.size {
                gs.surface.resize(&gs.ctx, nw, nh);
                self.surf_size = self.size;
            }
            gs.gfx.begin(sz.width, sz.height);
            self.render(&mut gs.gfx);
            gs.gfx.end_frame();
            let _ = gs.surface.swap_buffers(&gs.ctx);
            self.gl = Some(gs);
        } else {
            let mut g = self.cpu.take().unwrap_or_else(|| Gfx::new_cpu(sz.width, sz.height));
            g.begin(sz.width, sz.height);
            self.render(&mut g);
            if let (Some(pm), Some(surface)) = (g.canvas(), self.surface.as_mut()) {
                if surface.resize(nw, nh).is_ok() {
                    if let Ok(mut buf) = surface.buffer_mut() {
                        for (dst, px) in buf.iter_mut().zip(pm.data().chunks_exact(4)) {
                            *dst = ((px[0] as u32) << 16) | ((px[1] as u32) << 8) | px[2] as u32;
                        }
                        let _ = buf.present();
                    }
                }
            }
            self.cpu = Some(g);
        }
    }

    /// Texte (mis en cache sous forme d'image). `y` = haut de ligne ; `x` = bord gauche / centre / bord droit.
    #[allow(clippy::too_many_arguments)]
    fn txt(&self, g: &mut Gfx, s: &str, x: f32, y: f32, px: f32, kind: Kind, rgb: Rgb, al: Align, max_w: Option<f32>, ell_left: bool) {
        if s.is_empty() {
            return;
        }
        let key = format!(
            "t{}|{:.1}|{:02x}{:02x}{:02x}|{}|{}|{}",
            kind as u8,
            px,
            (rgb[0] * 255.0) as u8,
            (rgb[1] * 255.0) as u8,
            (rgb[2] * 255.0) as u8,
            max_w.map(|m| m as i32).unwrap_or(-1),
            ell_left as u8,
            s
        );
        let (w, _) = g.img(&key, &|| text_pm(&self.fonts, s, px, kind, rgb, max_w, ell_left));
        let tw = w as f32 - 3.0;
        let left = match al {
            Align::Left => x,
            Align::Center => x - tw / 2.0,
            Align::Right => x - tw,
        };
        g.blit(&key, left - 1.0, y - 1.0, 1.0);
    }

    fn try_gl(&mut self, el: &ActiveEventLoop, attrs: winit::window::WindowAttributes) -> Option<Rc<Window>> {
        if std::env::var("LAPP_CPU").is_ok() {
            return None;
        }
        let built = DisplayBuilder::new().with_window_attributes(Some(attrs)).build(
            el,
            ConfigTemplateBuilder::new(),
            |c| c.reduce(|a, b| if b.num_samples() < a.num_samples() { b } else { a }).unwrap(),
        );
        let (window, config) = match built {
            Ok(v) => v,
            Err(e) => {
                eprintln!("settings : EGL indisponible, repli CPU : {e}");
                return None;
            }
        };
        let window = window?;
        let raw = window.window_handle().ok()?.as_raw();
        let display = config.display();
        let ca = ContextAttributesBuilder::new().with_context_api(ContextApi::Gles(Some(Version::new(2, 0)))).build(Some(raw));
        let nc = match unsafe { display.create_context(&config, &ca) } {
            Ok(c) => c,
            Err(e) => {
                eprintln!("settings : contexte GLES impossible, repli CPU : {e}");
                return None;
            }
        };
        let sa = window.build_surface_attributes(Default::default()).ok()?;
        let surface = match unsafe { display.create_window_surface(&config, &sa) } {
            Ok(s) => s,
            Err(e) => {
                eprintln!("settings : surface EGL impossible, repli CPU : {e}");
                return None;
            }
        };
        let ctx = nc.make_current(&surface).ok()?;
        let _ = surface.set_swap_interval(&ctx, SwapInterval::Wait(NonZeroU32::new(1).unwrap()));
        let gl = unsafe { glow::Context::from_loader_function_cstr(|s| display.get_proc_address(s)) };
        let g = match gfx::GlGfx::new(gl) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("settings : shaders GLES refusés, repli CPU : {e}");
                return None;
            }
        };
        let rn = g.renderer();
        let low = rn.to_lowercase();
        if low.contains("llvmpipe") || low.contains("softpipe") || low.contains("swrast") {
            // rendu logiciel de Mesa : plus lent que le CPU maison et inutilisable sans mémoire partagée X
            eprintln!("settings : EGL sans GPU réel ({rn}), repli CPU");
            return None;
        }
        eprintln!("settings : rendu GPU ({rn})");
        self.gl = Some(GlState { surface, ctx, gfx: Gfx::Gl(Box::new(g)) });
        Some(Rc::new(window))
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        // pas de gestionnaire de fenêtres : on prend toute la taille de l'écran
        let (w, h) = el
            .primary_monitor()
            .or_else(|| el.available_monitors().next())
            .map(|m| (m.size().width, m.size().height))
            .filter(|(w, h)| *w > 0 && *h > 0)
            .unwrap_or((272, 480));
        let attrs = Window::default_attributes()
            .with_title("Réglages")
            .with_decorations(false)
            .with_resizable(false)
            .with_inner_size(PhysicalSize::new(w, h))
            .with_position(PhysicalPosition::new(0, 0));
        let window = match self.try_gl(el, attrs.clone()) {
            Some(w) => w,
            None => {
                let window = match el.create_window(attrs) {
                    Ok(win) => Rc::new(win),
                    Err(e) => {
                        eprintln!("settings : fenêtre impossible : {e}");
                        el.exit();
                        return;
                    }
                };
                let ctx = match softbuffer::Context::new(window.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("settings : contexte graphique : {e}");
                        el.exit();
                        return;
                    }
                };
                match softbuffer::Surface::new(&ctx, window.clone()) {
                    Ok(s) => self.surface = Some(s),
                    Err(e) => {
                        eprintln!("settings : surface graphique : {e}");
                        el.exit();
                        return;
                    }
                }
                eprintln!("settings : rendu CPU");
                window
            }
        };
        self.size = (w, h);
        window.request_redraw();
        self.window = Some(window);
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        match &self.toast {
            Some((_, until)) if Instant::now() >= *until => {
                self.toast = None;
                self.redraw_later();
                el.set_control_flow(ControlFlow::Wait);
            }
            Some((_, until)) => el.set_control_flow(ControlFlow::WaitUntil(*until)),
            None => el.set_control_flow(ControlFlow::Wait),
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, ev: WindowEvent) {
        match ev {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(_) => self.redraw_later(),
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::KeyboardInput { event: KeyEvent { logical_key: Key::Named(k), state: ElementState::Pressed, .. }, .. } => {
                if k == NamedKey::Escape || k == NamedKey::Backspace {
                    if self.page == Page::Info {
                        self.page = Page::Main;
                        self.redraw_later();
                    } else if k == NamedKey::Escape {
                        el.exit();
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x as f32, position.y as f32);
                self.moved(self.cursor.0, self.cursor.1);
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => match state {
                ElementState::Pressed => self.press(self.cursor.0, self.cursor.1),
                ElementState::Released => self.release(self.cursor.0, self.cursor.1, Some(el)),
            },
            WindowEvent::Touch(t) => {
                let (x, y) = (t.location.x as f32, t.location.y as f32);
                self.cursor = (x, y);
                match t.phase {
                    TouchPhase::Started => self.press(x, y),
                    TouchPhase::Moved => self.moved(x, y),
                    TouchPhase::Ended | TouchPhase::Cancelled => self.release(x, y, Some(el)),
                }
            }
            _ => {}
        }
    }
}

// ------------------------------------------------------------------ main

fn make_icon(path: &str) {
    let n = 128u32;
    let mut pm = Pixmap::new(n, n).unwrap();
    let f = n as f32;
    fill_rrect(&mut pm, 0.0, 0.0, f, f, f * 0.22, col([0.36, 0.37, 0.42], 1.0));
    fill_rrect(&mut pm, 0.0, f * 0.5, f, f * 0.5, f * 0.22, col([0.27, 0.28, 0.33], 1.0));
    fill_rect(&mut pm, 0.0, f * 0.5, f, f * 0.2, col([0.27, 0.28, 0.33], 1.0));
    for (i, k) in [0.30f32, 0.62, 0.45].iter().enumerate() {
        let y = f * (0.27 + 0.23 * i as f32);
        fill_rrect(&mut pm, f * 0.18, y, f * 0.64, f * 0.05, f * 0.025, col(WHITE, 0.85));
        fill_rrect(&mut pm, f * (0.18 + 0.64 * k) - f * 0.07, y - f * 0.045, f * 0.14, f * 0.14, f * 0.07, col(WHITE, 1.0));
    }
    if let Err(e) = pm.save_png(path) {
        eprintln!("settings : icône : {e}");
        std::process::exit(1);
    }
}

/// Echelle d'interface commune (dpi) : `--scale X`, sinon `LAPP_SCALE` (exportee par lapp-home).
fn scale_override() -> Option<f32> {
    static OVR: std::sync::OnceLock<Option<f32>> = std::sync::OnceLock::new();
    *OVR.get_or_init(|| {
        let parse = |s: &str| s.trim().parse::<f32>().ok().filter(|v| v.is_finite() && *v > 0.0).map(|v| v.clamp(0.5, 8.0));
        let args: Vec<String> = std::env::args().skip(1).collect();
        args.iter()
            .position(|a| a == "--scale")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| parse(s))
            .or_else(|| std::env::var("LAPP_SCALE").ok().as_deref().and_then(parse))
    })
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let val = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();

    if let Some(p) = val("--icon") {
        make_icon(&p);
        return;
    }
    let size = val("--size")
        .and_then(|s| {
            let (a, b) = s.split_once('x')?;
            Some((a.parse().ok()?, b.parse().ok()?))
        })
        .unwrap_or((272, 480));

    // rendu hors écran (CPU) pour les tests
    if let Some(out) = val("--screenshot") {
        let mut app = App::new(size);
        if val("--page").as_deref() == Some("info") {
            app.info = info_rows();
            app.page = Page::Info;
        }
        let mut g = Gfx::new_cpu(size.0, size.1);
        g.begin(size.0, size.1);
        app.render(&mut g);
        if let Err(e) = g.canvas().unwrap().save_png(&out) {
            eprintln!("settings : {e}");
            std::process::exit(1);
        }
        return;
    }

    let el = match EventLoop::new() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("settings : pas d'affichage disponible : {e}");
            std::process::exit(1);
        }
    };
    el.set_control_flow(ControlFlow::Wait);
    let mut app = App::new(size);
    if let Err(e) = el.run_app(&mut app) {
        eprintln!("settings : {e}");
        std::process::exit(1);
    }
}