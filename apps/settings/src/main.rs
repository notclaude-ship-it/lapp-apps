// settings : réglages tactiles pour lapp (écran portrait 272x480), rendu OpenGL ES (repli CPU).
//
// - Luminosité : /sys/class/backlight/* ; Volume : PipeWire (wpctl), repli ALSA (amixer) ; Informations : appareil, noyau, mémoire, stockage, batterie, IP
// - Wi-Fi : activer/désactiver, lister les réseaux, se connecter (mot de passe au clavier virtuel de lapp-home), déconnecter, oublier
//   (NetworkManager : `sudo nmcli` est lancé sur l'hôte via le courtier de commandes, voir wifi.rs ; le mot de passe sudo
//   est demandé s'il est inconnu, puis enregistré, voir sudo.rs)
// - Bluetooth : activer/désactiver, rechercher, jumeler, connecter, déconnecter, oublier
//   (BlueZ : bluetoothctl est lancé sur l'hôte via le courtier de commandes, voir bluetooth.rs)
// - quitter : glisser vers le haut depuis le bord bas de l'écran, ou Echap
// Permissions : display, system (/sys en écriture + /dev/snd), network (adresse IP), storage, commands (sudo pour le Wi-Fi, bluetoothctl, rfkill).
// Tests hors écran : settings --screenshot out.png [--page info|wifi|wifinet|wifipass|sudo|sudowrong|bt|btdev] [--demo] ; settings --icon icon.png

mod bluetooth;
mod gfx;
mod hostcmd;
mod kbd;
mod sudo;
mod wifi;

use std::fs;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
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
const GREEN: Rgb = [0.20, 0.78, 0.35];
const RED: Rgb = [0.86, 0.20, 0.20];

// page Wi-Fi : liste des réseaux (pixels de conception)
const LIST_Y: f32 = 128.0;
const NET_ROW: f32 = 52.0;
// page d'un réseau
const NET_INFO_Y: f32 = HEADER + 10.0;
const NET_INFO_H: f32 = 84.0;
const NET_NEXT_Y: f32 = NET_INFO_Y + NET_INFO_H + 10.0;
const PASS_H: f32 = 46.0;
const BTN_H: f32 = 44.0;
/// page du mot de passe sudo : champ et bouton
const SUDO_PASS_Y: f32 = HEADER + 78.0;
const SUDO_BTN_Y: f32 = SUDO_PASS_Y + PASS_H + 10.0;

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
    Wifi,
    WifiNet,
    Sudo,
    Bt,
    BtDev,
}

#[derive(Clone, Copy, PartialEq)]
enum Btn {
    Connect,
    Disconnect,
    Forget,
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
fn cards() -> [(f32, f32); 5] {
    let y0 = HEADER + 14.0;
    [(y0, 68.0), (y0 + 78.0, 68.0), (y0 + 156.0, 48.0), (y0 + 214.0, 48.0), (y0 + 272.0, 48.0)]
}

/// Identifiant X11 de la fenêtre de premier niveau (pour le clavier virtuel de lapp-home).
fn x11_window(w: &Window) -> Option<u32> {
    use winit::raw_window_handle::RawWindowHandle;
    match w.window_handle().ok()?.as_raw() {
        RawWindowHandle::Xlib(h) => Some(h.window as u32),
        RawWindowHandle::Xcb(h) => Some(h.window.get()),
        _ => None,
    }
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
    // Wi-Fi
    wifi: Option<wifi::State>,
    wifi_err: Option<String>,
    sel: Option<wifi::Net>,
    pass: String,
    show_pass: bool,
    scroll: f32,
    last_y: f32,
    jobs: usize,
    busy: Option<String>,
    inbox: Arc<Mutex<Vec<wifi::Done>>>,
    kbd: Option<kbd::KbdClient>,
    // mot de passe sudo (demandé quand une commande Wi-Fi en a besoin)
    sudo_pw: String,
    sudo_show: bool,
    sudo_wrong: bool,
    sudo_retry: Option<(wifi::Action, bool)>,
    sudo_from: Page,
    // Bluetooth
    bt: Option<bluetooth::State>,
    bt_err: Option<String>,
    bt_sel: Option<bluetooth::Dev>,
    bt_inbox: Arc<Mutex<Vec<bluetooth::Done>>>,
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
            wifi: None,
            wifi_err: None,
            sel: None,
            pass: String::new(),
            show_pass: false,
            scroll: 0.0,
            last_y: 0.0,
            jobs: 0,
            busy: None,
            inbox: Arc::new(Mutex::new(Vec::new())),
            kbd: None,
            sudo_pw: String::new(),
            sudo_show: false,
            sudo_wrong: false,
            sudo_retry: None,
            sudo_from: Page::Wifi,
            bt: None,
            bt_err: None,
            bt_sel: None,
            bt_inbox: Arc::new(Mutex::new(Vec::new())),
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

    // -------------------------------------------------------------- Wi-Fi (tâches et clavier)

    fn kbd_show(&mut self) {
        let Some(win) = self.window.as_ref().and_then(|w| x11_window(w)) else { return };
        if self.kbd.is_none() {
            self.kbd = kbd::KbdClient::connect();
        }
        if let Some(k) = &self.kbd {
            k.set(win, 1);
        }
    }

    fn kbd_hide(&mut self) {
        let Some(win) = self.window.as_ref().and_then(|w| x11_window(w)) else { return };
        if let Some(k) = &self.kbd {
            k.set(win, 0);
        }
    }

    /// Lance une action Wi-Fi dans un thread (nmcli peut mettre plusieurs secondes).
    fn job(&mut self, label: &str, act: wifi::Action, rescan: bool) {
        self.busy = Some(label.to_string());
        self.jobs += 1;
        let inbox = self.inbox.clone();
        std::thread::spawn(move || {
            let d = wifi::run(act, rescan);
            if let Ok(mut q) = inbox.lock() {
                q.push(d);
            }
        });
        self.redraw_later();
    }

    /// Récupère les résultats des tâches terminées ; `true` s'il y en avait.
    fn drain(&mut self) -> bool {
        let done: Vec<wifi::Done> = match self.inbox.lock() {
            Ok(mut q) => q.drain(..).collect(),
            Err(_) => return false,
        };
        if done.is_empty() {
            return false;
        }
        for d in done {
            self.jobs = self.jobs.saturating_sub(1);
            match d.state {
                Ok(st) => {
                    self.wifi_err = None;
                    if let Some(sel) = self.sel.clone() {
                        let fresh = st.nets.iter().find(|n| n.ssid == sel.ssid).cloned();
                        self.sel = fresh.or(Some(sel));
                    }
                    let max = ((st.nets.len() as f32 * NET_ROW) - (self.size.1 as f32 / self.scale() - LIST_Y - PAD_BOTTOM)).max(0.0);
                    self.scroll = self.scroll.min(max);
                    self.wifi = Some(st);
                }
                Err(e) => self.wifi_err = Some(e),
            }
            match d.note {
                Some(Ok(m)) if !m.is_empty() => self.set_toast(m),
                Some(Err(e)) => self.set_toast(e),
                _ => {}
            }
            if d.back && self.page == Page::WifiNet {
                self.leave_net();
            }
            // pas de question au démarrage (page principale) : seulement quand l'utilisateur est dans le Wi-Fi
            if let Some(su) = d.sudo {
                if matches!(self.page, Page::Wifi | Page::WifiNet) {
                    self.ask_sudo(su);
                }
            }
        }
        if self.jobs == 0 {
            self.busy = None;
        }
        true
    }

    /// Affiche la page du mot de passe sudo ; l'action `su.retry` sera relancée une fois le mot de passe saisi.
    fn ask_sudo(&mut self, su: wifi::Sudo) {
        self.sudo_from = self.page;
        self.sudo_retry = Some((su.retry, su.rescan));
        self.sudo_wrong = su.wrong;
        self.sudo_pw.clear();
        self.sudo_show = false;
        self.page = Page::Sudo;
        self.kbd_show();
    }

    fn submit_sudo(&mut self) {
        if self.sudo_pw.is_empty() {
            self.set_toast("Saisis le mot de passe sudo".into());
            return;
        }
        // enregistré par sudo.rs dès que sudo l'accepte, oublié s'il est refusé
        sudo::set(std::mem::take(&mut self.sudo_pw));
        self.sudo_show = false;
        self.kbd_hide();
        self.page = self.sudo_from;
        if let Some((act, rescan)) = self.sudo_retry.take() {
            self.job("Authentification…", act, rescan);
        }
    }

    fn cancel_sudo(&mut self) {
        self.kbd_hide();
        self.sudo_pw.clear();
        self.sudo_show = false;
        self.sudo_retry = None;
        self.page = self.sudo_from;
    }

    fn open_wifi(&mut self) {
        self.page = Page::Wifi;
        self.scroll = 0.0;
        let on = self.wifi.as_ref().map(|s| s.enabled).unwrap_or(true);
        self.job("Recherche des réseaux…", wifi::Action::None, on);
    }

    fn leave_net(&mut self) {
        self.kbd_hide();
        self.page = Page::Wifi;
        self.sel = None;
        self.pass.clear();
        self.show_pass = false;
    }

    fn back(&mut self) {
        match self.page {
            Page::Info | Page::Wifi | Page::Bt => self.page = Page::Main,
            Page::WifiNet => self.leave_net(),
            Page::Sudo => self.cancel_sudo(),
            Page::BtDev => self.leave_dev(),
            Page::Main => {}
        }
        self.redraw_later();
    }

    fn max_scroll(&self) -> f32 {
        let n = if matches!(self.page, Page::Bt | Page::BtDev) {
            self.bt.as_ref().map(|b| b.devs.len()).unwrap_or(0)
        } else {
            self.wifi.as_ref().map(|w| w.nets.len()).unwrap_or(0)
        } as f32;
        let view = self.size.1 as f32 / self.scale() - LIST_Y - PAD_BOTTOM;
        (n * NET_ROW - view).max(0.0)
    }

    fn connect_sel(&mut self) {
        let Some(n) = self.sel.clone() else { return };
        let pw = if n.needs_pass() { self.pass.clone() } else { String::new() };
        self.kbd_hide();
        let label = format!("Connexion à {}…", n.ssid);
        self.job(&label, wifi::Action::Connect(n, pw), false);
    }

    /// Boutons de la page d'un réseau : (bouton, y, libellé).
    fn net_buttons(&self, n: &wifi::Net) -> Vec<(Btn, f32, &'static str)> {
        let mut y = NET_NEXT_Y + if n.needs_pass() { PASS_H + 10.0 } else { 0.0 };
        let mut v = vec![];
        if n.active {
            v.push((Btn::Disconnect, y, "Déconnecter"));
        } else {
            v.push((Btn::Connect, y, "Se connecter"));
        }
        y += BTN_H + 10.0;
        if n.profile.is_some() {
            v.push((Btn::Forget, y, "Oublier ce réseau"));
        }
        v
    }

    // -------------------------------------------------------------- Bluetooth

    /// Lance une action Bluetooth dans un thread (bluetoothctl peut mettre plusieurs secondes).
    fn bt_job(&mut self, label: &str, act: bluetooth::Action) {
        self.busy = Some(label.to_string());
        self.jobs += 1;
        let inbox = self.bt_inbox.clone();
        std::thread::spawn(move || {
            let d = bluetooth::run(act);
            if let Ok(mut q) = inbox.lock() {
                q.push(d);
            }
        });
        self.redraw_later();
    }

    /// Récupère les résultats Bluetooth terminés ; `true` s'il y en avait.
    fn drain_bt(&mut self) -> bool {
        let done: Vec<bluetooth::Done> = match self.bt_inbox.lock() {
            Ok(mut q) => q.drain(..).collect(),
            Err(_) => return false,
        };
        if done.is_empty() {
            return false;
        }
        for d in done {
            self.jobs = self.jobs.saturating_sub(1);
            match d.state {
                Ok(st) => {
                    self.bt_err = None;
                    if let Some(sel) = self.bt_sel.clone() {
                        let fresh = st.devs.iter().find(|n| n.mac == sel.mac).cloned();
                        self.bt_sel = fresh.or(Some(sel));
                    }
                    let max = ((st.devs.len() as f32 * NET_ROW) - (self.size.1 as f32 / self.scale() - LIST_Y - PAD_BOTTOM)).max(0.0);
                    self.scroll = self.scroll.min(max);
                    self.bt = Some(st);
                }
                Err(e) => self.bt_err = Some(e),
            }
            match d.note {
                Some(Ok(m)) if !m.is_empty() => self.set_toast(m),
                Some(Err(e)) => self.set_toast(e),
                _ => {}
            }
            if d.back && self.page == Page::BtDev {
                self.leave_dev();
            }
        }
        if self.jobs == 0 {
            self.busy = None;
        }
        true
    }

    fn open_bt(&mut self) {
        self.page = Page::Bt;
        self.scroll = 0.0;
        // la liste connue s'affiche tout de suite, la recherche (quelques secondes) la complète ensuite
        self.bt_job("Lecture de l'état…", bluetooth::Action::None);
        if self.bt.as_ref().map(|b| b.powered).unwrap_or(false) {
            self.bt_job("Recherche des appareils…", bluetooth::Action::Scan);
        }
    }

    fn leave_dev(&mut self) {
        self.page = Page::Bt;
        self.bt_sel = None;
    }

    fn bt_label(&self) -> String {
        match (&self.bt, &self.bt_err) {
            (Some(b), _) if !b.present => "Indisponible".into(),
            (Some(b), _) if !b.powered => "Désactivé".into(),
            (Some(b), _) => {
                let n = b.devs.iter().filter(|d| d.connected).count();
                match n {
                    0 => "Activé".into(),
                    1 => b.devs.iter().find(|d| d.connected).map(|d| d.name.clone()).unwrap_or_default(),
                    n => format!("{n} appareils"),
                }
            }
            (None, Some(_)) => "Indisponible".into(),
            (None, None) => "…".into(),
        }
    }

    /// Boutons de la page d'un appareil : (bouton, y, libellé).
    fn dev_buttons(&self, d: &bluetooth::Dev) -> Vec<(Btn, f32, &'static str)> {
        let mut y = NET_NEXT_Y;
        let mut v = vec![];
        if d.connected {
            v.push((Btn::Disconnect, y, "Déconnecter"));
        } else if d.paired {
            v.push((Btn::Connect, y, "Se connecter"));
        } else {
            v.push((Btn::Connect, y, "Jumeler et connecter"));
        }
        y += BTN_H + 10.0;
        if d.paired {
            v.push((Btn::Forget, y, "Oublier cet appareil"));
        }
        v
    }

    fn tap_bt(&mut self, y: f32) {
        let s = self.scale();
        let Some(st) = self.bt.clone() else { return };
        let (ty, th) = (HEADER + 10.0, 48.0);
        if y >= ty * s && y < (ty + th) * s {
            if self.jobs == 0 && st.present {
                let on = !st.powered;
                self.bt_job(if on { "Activation…" } else { "Désactivation…" }, bluetooth::Action::Toggle(on));
            }
            return;
        }
        if !st.powered || y < LIST_Y * s {
            return;
        }
        let idx = (y / s - LIST_Y + self.scroll) / NET_ROW;
        if idx < 0.0 {
            return;
        }
        if let Some(d) = st.devs.get(idx as usize) {
            self.bt_sel = Some(d.clone());
            self.page = Page::BtDev;
        }
    }

    fn tap_dev(&mut self, x: f32, y: f32) {
        let s = self.scale();
        let w = self.size.0 as f32;
        let Some(d) = self.bt_sel.clone() else { return };
        if self.jobs > 0 || x < CARD_X * s || x > w - CARD_X * s {
            return;
        }
        for (b, by, _) in self.dev_buttons(&d) {
            if y >= by * s && y < (by + BTN_H) * s {
                match b {
                    Btn::Connect => {
                        let l = if d.paired { format!("Connexion à {}…", d.name) } else { format!("Jumelage avec {}…", d.name) };
                        self.bt_job(&l, bluetooth::Action::Connect(d.clone()));
                    }
                    Btn::Disconnect => self.bt_job("Déconnexion…", bluetooth::Action::Disconnect(d.clone())),
                    Btn::Forget => self.bt_job("Suppression…", bluetooth::Action::Forget(d.clone())),
                }
                return;
            }
        }
    }

    // -------------------------------------------------------------- interaction

    fn tap(&mut self, x: f32, y: f32) {
        let s = self.scale();
        let w = self.size.0 as f32;
        if y < HEADER * s {
            if x < 84.0 * s {
                self.back();
            } else if self.page == Page::Wifi && x > w - 96.0 * s && self.jobs == 0 {
                self.job("Recherche des réseaux…", wifi::Action::None, true);
            } else if self.page == Page::Bt && x > w - 110.0 * s && self.jobs == 0 && self.bt.as_ref().map(|b| b.powered).unwrap_or(false) {
                self.bt_job("Recherche des appareils…", bluetooth::Action::Scan);
            }
            return;
        }
        match self.page {
            Page::Main => {
                for (i, (cy, ch)) in cards().iter().enumerate().skip(2) {
                    if y >= cy * s && y < (cy + ch) * s {
                        if i == 2 {
                            self.open_wifi();
                        } else if i == 3 {
                            self.open_bt();
                        } else {
                            self.info = info_rows();
                            self.page = Page::Info;
                        }
                    }
                }
            }
            Page::Wifi => self.tap_wifi(y),
            Page::WifiNet => self.tap_net(x, y),
            Page::Sudo => self.tap_sudo(x, y),
            Page::Bt => self.tap_bt(y),
            Page::BtDev => self.tap_dev(x, y),
            Page::Info => {}
        }
    }

    fn tap_wifi(&mut self, y: f32) {
        let s = self.scale();
        let Some(st) = self.wifi.clone() else { return };
        let (ty, th) = (HEADER + 10.0, 48.0);
        if y >= ty * s && y < (ty + th) * s {
            if self.jobs == 0 {
                let on = !st.enabled;
                self.job(if on { "Activation…" } else { "Désactivation…" }, wifi::Action::Toggle(on), on);
            }
            return;
        }
        if !st.enabled || y < LIST_Y * s {
            return;
        }
        let idx = (y / s - LIST_Y + self.scroll) / NET_ROW;
        if idx < 0.0 {
            return;
        }
        if let Some(n) = st.nets.get(idx as usize) {
            self.sel = Some(n.clone());
            self.pass.clear();
            self.show_pass = false;
            self.page = Page::WifiNet;
            if n.needs_pass() {
                self.kbd_show();
            }
        }
    }

    fn tap_sudo(&mut self, x: f32, y: f32) {
        let s = self.scale();
        let w = self.size.0 as f32;
        if self.jobs > 0 {
            return;
        }
        if y >= SUDO_PASS_Y * s && y < (SUDO_PASS_Y + PASS_H) * s {
            if x > w - 84.0 * s {
                self.sudo_show = !self.sudo_show;
            } else {
                self.kbd_show();
            }
            return;
        }
        if x >= CARD_X * s && x <= w - CARD_X * s && y >= SUDO_BTN_Y * s && y < (SUDO_BTN_Y + BTN_H) * s {
            self.submit_sudo();
        }
    }

    fn tap_net(&mut self, x: f32, y: f32) {
        let s = self.scale();
        let w = self.size.0 as f32;
        let Some(n) = self.sel.clone() else { return };
        if self.jobs > 0 {
            return;
        }
        if n.needs_pass() && y >= NET_NEXT_Y * s && y < (NET_NEXT_Y + PASS_H) * s {
            if x > w - 84.0 * s {
                self.show_pass = !self.show_pass;
            } else {
                self.kbd_show();
            }
            return;
        }
        if x < CARD_X * s || x > w - CARD_X * s {
            return;
        }
        for (b, by, _) in self.net_buttons(&n) {
            if y >= by * s && y < (by + BTN_H) * s {
                match b {
                    Btn::Connect => {
                        if n.needs_pass() && self.pass.chars().count() < 8 {
                            self.set_toast("Mot de passe : 8 caractères minimum".into());
                        } else {
                            self.connect_sel();
                        }
                    }
                    Btn::Disconnect => self.job("Déconnexion…", wifi::Action::Disconnect(n.clone()), false),
                    Btn::Forget => self.job("Suppression…", wifi::Action::Forget(n.clone()), false),
                }
                return;
            }
        }
    }

    /// Touches du clavier (physique, ou virtuel de lapp-home via XTEST).
    fn key(&mut self, el: &ActiveEventLoop, ev: &KeyEvent) {
        if self.page == Page::Sudo {
            match &ev.logical_key {
                Key::Named(NamedKey::Backspace) => {
                    self.sudo_pw.pop();
                }
                Key::Named(NamedKey::Enter) => {
                    if self.jobs == 0 {
                        self.submit_sudo();
                    }
                }
                Key::Named(NamedKey::Escape) => self.back(),
                _ => {
                    if let Some(t) = &ev.text {
                        for c in t.chars().filter(|c| !c.is_control()) {
                            if self.sudo_pw.chars().count() < 128 {
                                self.sudo_pw.push(c);
                            }
                        }
                    }
                }
            }
            self.redraw_later();
            return;
        }
        if self.page == Page::WifiNet && self.sel.as_ref().map(|n| n.needs_pass()).unwrap_or(false) {
            match &ev.logical_key {
                Key::Named(NamedKey::Backspace) => {
                    self.pass.pop();
                }
                Key::Named(NamedKey::Enter) => {
                    if self.jobs == 0 && self.pass.chars().count() >= 8 {
                        self.connect_sel();
                    }
                }
                Key::Named(NamedKey::Escape) => self.back(),
                _ => {
                    if let Some(t) = &ev.text {
                        for c in t.chars().filter(|c| !c.is_control()) {
                            if self.pass.chars().count() < 63 {
                                self.pass.push(c);
                            }
                        }
                    }
                }
            }
            self.redraw_later();
            return;
        }
        match &ev.logical_key {
            Key::Named(NamedKey::Escape) => {
                if self.page == Page::Main {
                    el.exit();
                } else {
                    self.back();
                }
            }
            Key::Named(NamedKey::Backspace) => self.back(),
            _ => {}
        }
    }

    fn press(&mut self, x: f32, y: f32) {
        let s = self.scale();
        self.press = Some((x, y, false));
        self.last_y = y;
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
        if matches!(self.page, Page::Wifi | Page::Bt) && self.drag.is_none() && self.press.map(|p| p.2).unwrap_or(false) {
            let max = self.max_scroll();
            self.scroll = (self.scroll - (y - self.last_y) / s).clamp(0.0, max);
        }
        self.last_y = y;
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
            Page::Wifi => self.render_wifi(g, s),
            Page::WifiNet => self.render_net(g, s),
            Page::Sudo => self.render_sudo(g, s),
            Page::Bt => self.render_bt(g, s),
            Page::BtDev => self.render_dev(g, s),
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
        self.txt(g, "Wi-Fi", 18.0 * s, (y + (h - 14.0 * 1.17) / 2.0) * s, 14.0 * s, Kind::Reg, DARK, Align::Left, None, false);
        self.txt(g, &self.wifi_label(), w - 36.0 * s, (y + (h - 13.0 * 1.17) / 2.0) * s, 13.0 * s, Kind::Reg, GREY, Align::Right, Some(w - 36.0 * s - 90.0 * s), false);
        self.txt(g, "›", w - 20.0 * s, (y + (h - 20.0 * 1.17) / 2.0) * s, 20.0 * s, Kind::Reg, LINE, Align::Right, None, false);
        let (y, h) = cards()[3];
        self.card(g, s, y, h);
        self.txt(g, "Bluetooth", 18.0 * s, (y + (h - 14.0 * 1.17) / 2.0) * s, 14.0 * s, Kind::Reg, DARK, Align::Left, None, false);
        self.txt(g, &self.bt_label(), w - 36.0 * s, (y + (h - 13.0 * 1.17) / 2.0) * s, 13.0 * s, Kind::Reg, GREY, Align::Right, Some(w - 36.0 * s - 90.0 * s), false);
        self.txt(g, "›", w - 20.0 * s, (y + (h - 20.0 * 1.17) / 2.0) * s, 20.0 * s, Kind::Reg, LINE, Align::Right, None, false);
        let (y, h) = cards()[4];
        self.card(g, s, y, h);
        self.txt(g, "Informations", 18.0 * s, (y + (h - 14.0 * 1.17) / 2.0) * s, 14.0 * s, Kind::Reg, DARK, Align::Left, None, false);
        self.txt(g, "›", w - 20.0 * s, (y + (h - 20.0 * 1.17) / 2.0) * s, 20.0 * s, Kind::Reg, LINE, Align::Right, None, false);
        self.header(g, s, "Réglages", false);
    }

    fn wifi_label(&self) -> String {
        match (&self.wifi, &self.wifi_err) {
            (Some(st), _) if !st.enabled => "Désactivé".into(),
            (Some(st), _) => st.nets.iter().find(|n| n.active).map(|n| n.ssid.clone()).unwrap_or_else(|| "Non connecté".into()),
            (None, Some(e)) if e == sudo::NEEDED || e == sudo::WRONG => "Droits requis".into(),
            (None, Some(_)) => "Indisponible".into(),
            (None, None) => "…".into(),
        }
    }

    fn switch(&self, g: &mut Gfx, s: f32, x: f32, y: f32, on: bool) {
        g.rrect(x, y, 46.0 * s, 28.0 * s, 14.0 * s, col(if on { GREEN } else { LINE }, 1.0));
        let kx = if on { x + 20.0 * s } else { x + 2.0 * s };
        g.rrect(kx, y + 2.0 * s, 24.0 * s, 24.0 * s, 12.0 * s, col(WHITE, 1.0));
    }

    fn net_row(&self, g: &mut Gfx, s: f32, y: f32, n: &wifi::Net) {
        let w = self.size.0 as f32;
        g.rect(0.0, y, w, NET_ROW * s, col(WHITE, 1.0));
        g.rect(14.0 * s, y + NET_ROW * s - 1.0, w - 14.0 * s, 1.0, col(LINE, 0.6));
        self.txt(g, &n.ssid, 14.0 * s, y + 9.0 * s, 14.0 * s, Kind::Reg, DARK, Align::Left, Some(w - 100.0 * s), false);
        let (sub, c) = if n.active {
            ("Connecté".to_string(), BLUE)
        } else if n.profile.is_some() {
            ("Enregistré".to_string(), GREY)
        } else if n.secured() {
            (n.security.clone(), GREY)
        } else {
            ("Ouvert".to_string(), GREY)
        };
        self.txt(g, &sub, 14.0 * s, y + 29.0 * s, 11.0 * s, Kind::Reg, c, Align::Left, Some(w - 100.0 * s), false);
        // barres de signal
        let base = y + (NET_ROW / 2.0 + 8.0) * s;
        for (i, th) in [15u8, 35, 55, 75].iter().enumerate() {
            let bh = (5.0 + 3.5 * i as f32) * s;
            let on = n.signal >= *th;
            g.rrect(w - 66.0 * s + i as f32 * 8.0 * s, base - bh, 5.0 * s, bh, 1.5 * s, col(if on { DARK } else { LINE }, 1.0));
        }
        self.txt(g, "›", w - 16.0 * s, y + (NET_ROW - 20.0 * 1.17) / 2.0 * s, 20.0 * s, Kind::Reg, LINE, Align::Right, None, false);
    }

    fn render_wifi(&self, g: &mut Gfx, s: f32) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let enabled = self.wifi.as_ref().map(|st| st.enabled).unwrap_or(false);
        let nets: &[wifi::Net] = self.wifi.as_ref().map(|st| &st.nets[..]).unwrap_or(&[]);
        // lignes défilantes d'abord ; le haut de page les recouvre ensuite
        if enabled {
            let mut y = (LIST_Y - self.scroll) * s;
            for n in nets {
                if y + NET_ROW * s > 0.0 && y < h {
                    self.net_row(g, s, y, n);
                }
                y += NET_ROW * s;
            }
        }
        g.rect(0.0, 0.0, w, LIST_Y * s, col(BG, 1.0));
        // carte de l'interrupteur
        let (ty, th) = (HEADER + 10.0, 48.0);
        self.card(g, s, ty, th);
        self.txt(g, "Wi-Fi", 18.0 * s, (ty + (th - 14.0 * 1.17) / 2.0) * s, 14.0 * s, Kind::Reg, DARK, Align::Left, None, false);
        if self.wifi.is_some() {
            self.switch(g, s, w - 18.0 * s - 46.0 * s, (ty + 10.0) * s, enabled);
        }
        // ligne d'état
        let status = if let Some(b) = &self.busy {
            b.clone()
        } else if let Some(e) = &self.wifi_err {
            e.clone()
        } else if self.wifi.is_none() {
            String::new()
        } else if !enabled {
            "Wi-Fi désactivé".to_string()
        } else if nets.is_empty() {
            "Aucun réseau trouvé".to_string()
        } else {
            "Réseaux disponibles".to_string()
        };
        let err = self.busy.is_none() && self.wifi_err.is_some();
        self.txt(g, &status, 18.0 * s, (HEADER + 10.0 + 48.0 + 9.0) * s, 11.0 * s, Kind::Reg, if err { RED } else { GREY }, Align::Left, Some(w - 36.0 * s), false);
        self.header(g, s, "Wi-Fi", true);
        self.txt(g, "Actualiser", w - 12.0 * s, (HEADER * s - 13.0 * s * 1.17) / 2.0, 13.0 * s, Kind::Reg, BLUE, Align::Right, None, false);
    }

    /// Champ de mot de passe (points, ou texte si `show`) avec le bouton Afficher/Masquer.
    fn pass_field(&self, g: &mut Gfx, s: f32, y: f32, pw: &str, show: bool) {
        let w = self.size.0 as f32;
        self.card(g, s, y, PASS_H);
        let ty = (y + (PASS_H - 14.0 * 1.17) / 2.0) * s;
        let (x0, x1) = (18.0 * s, w - 96.0 * s);
        if pw.is_empty() {
            self.txt(g, "Mot de passe", x0, ty, 14.0 * s, Kind::Reg, GREY, Align::Left, None, false);
        } else if show {
            self.txt(g, pw, x0, ty, 14.0 * s, Kind::Reg, DARK, Align::Left, Some(x1 - x0), true);
        } else {
            let step = 13.0 * s;
            let max = ((x1 - x0) / step) as usize;
            let count = pw.chars().count();
            let skip = count.saturating_sub(max);
            let cy = (y + PASS_H / 2.0) * s;
            for i in 0..count - skip {
                g.rrect(x0 + i as f32 * step, cy - 4.0 * s, 8.0 * s, 8.0 * s, 4.0 * s, col(DARK, 1.0));
            }
        }
        let lbl = if show { "Masquer" } else { "Afficher" };
        self.txt(g, lbl, w - 18.0 * s, (y + (PASS_H - 12.0 * 1.17) / 2.0) * s, 12.0 * s, Kind::Reg, BLUE, Align::Right, None, false);
    }

    fn render_sudo(&self, g: &mut Gfx, s: f32) {
        let w = self.size.0 as f32;
        let (title, tc) = if self.sudo_wrong { ("Mot de passe incorrect", RED) } else { ("Droits administrateur", DARK) };
        self.txt(g, title, w / 2.0, (HEADER + 14.0) * s, 14.0 * s, Kind::Bold, tc, Align::Center, Some(w - 36.0 * s), false);
        self.txt(g, "Le Wi-Fi utilise sudo. Le mot de passe", w / 2.0, (HEADER + 38.0) * s, 12.0 * s, Kind::Reg, GREY, Align::Center, Some(w - 24.0 * s), false);
        self.txt(g, "est enregistré dès qu'il est accepté.", w / 2.0, (HEADER + 54.0) * s, 12.0 * s, Kind::Reg, GREY, Align::Center, Some(w - 24.0 * s), false);
        self.pass_field(g, s, SUDO_PASS_Y, &self.sudo_pw, self.sudo_show);
        let (bx, bw) = (CARD_X * s, w - 2.0 * CARD_X * s);
        let bg = if self.sudo_pw.is_empty() { [0.62, 0.76, 0.96] } else { BLUE };
        g.rrect(bx, SUDO_BTN_Y * s, bw, BTN_H * s, 10.0 * s, col(bg, 1.0));
        self.txt(g, "Valider", w / 2.0, (SUDO_BTN_Y + (BTN_H - 15.0 * 1.17) / 2.0) * s, 15.0 * s, Kind::Bold, WHITE, Align::Center, Some(bw - 20.0 * s), false);
        if let Some(b) = &self.busy {
            self.txt(g, b, w / 2.0, (SUDO_BTN_Y + BTN_H + 12.0) * s, 12.0 * s, Kind::Reg, GREY, Align::Center, Some(w - 36.0 * s), false);
        }
        self.header(g, s, "Administrateur", true);
    }

    fn render_net(&self, g: &mut Gfx, s: f32) {
        let w = self.size.0 as f32;
        let Some(n) = &self.sel else {
            self.header(g, s, "Wi-Fi", true);
            return;
        };
        // informations
        self.card(g, s, NET_INFO_Y, NET_INFO_H);
        let etat = if n.active {
            "Connecté"
        } else if n.profile.is_some() {
            "Enregistré"
        } else {
            "Non connecté"
        };
        let rows = [
            ("Signal", format!("{} %", n.signal)),
            ("Sécurité", if n.secured() { n.security.clone() } else { "Ouvert".to_string() }),
            ("État", etat.to_string()),
        ];
        for (i, (k, v)) in rows.iter().enumerate() {
            let ry = NET_INFO_Y + 4.0 + i as f32 * 26.0;
            self.txt(g, k, 18.0 * s, (ry + 6.0) * s, 12.0 * s, Kind::Reg, GREY, Align::Left, None, false);
            self.txt(g, v, w - 18.0 * s, (ry + 6.0) * s, 12.0 * s, Kind::Reg, if i == 2 && n.active { BLUE } else { DARK }, Align::Right, Some(w - 36.0 * s - 80.0 * s), false);
            if i < 2 {
                g.rect(18.0 * s, (ry + 26.0) * s - 1.0, w - 36.0 * s, 1.0, col(LINE, 0.6));
            }
        }
        // mot de passe
        if n.needs_pass() {
            self.pass_field(g, s, NET_NEXT_Y, &self.pass, self.show_pass);
        }
        // boutons
        let mut last = NET_NEXT_Y;
        for (b, by, label) in self.net_buttons(n) {
            let (bx, bw) = (CARD_X * s, w - 2.0 * CARD_X * s);
            let (bg, fg) = match b {
                Btn::Connect if n.needs_pass() && self.pass.chars().count() < 8 => ([0.62, 0.76, 0.96], WHITE),
                Btn::Connect => (BLUE, WHITE),
                Btn::Disconnect => (WHITE, DARK),
                Btn::Forget => (WHITE, RED),
            };
            if bg == WHITE {
                g.rrect(bx, by * s, bw, BTN_H * s, 10.0 * s, col(LINE, 1.0));
                g.rrect(bx + 1.0, by * s + 1.0, bw - 2.0, BTN_H * s - 2.0, 9.0 * s, col(WHITE, 1.0));
            } else {
                g.rrect(bx, by * s, bw, BTN_H * s, 10.0 * s, col(bg, 1.0));
            }
            self.txt(g, label, w / 2.0, (by + (BTN_H - 15.0 * 1.17) / 2.0) * s, 15.0 * s, Kind::Bold, fg, Align::Center, Some(bw - 20.0 * s), false);
            last = by + BTN_H;
        }
        if let Some(b) = &self.busy {
            self.txt(g, b, w / 2.0, (last + 12.0) * s, 12.0 * s, Kind::Reg, GREY, Align::Center, Some(w - 36.0 * s), false);
        }
        self.header(g, s, &n.ssid, true);
    }

    fn dev_row(&self, g: &mut Gfx, s: f32, y: f32, d: &bluetooth::Dev) {
        let w = self.size.0 as f32;
        g.rect(0.0, y, w, NET_ROW * s, col(WHITE, 1.0));
        g.rect(14.0 * s, y + NET_ROW * s - 1.0, w - 14.0 * s, 1.0, col(LINE, 0.6));
        self.txt(g, &d.name, 14.0 * s, y + 9.0 * s, 14.0 * s, Kind::Reg, DARK, Align::Left, Some(w - 50.0 * s), false);
        let (sub, c) = if d.connected {
            ("Connecté", BLUE)
        } else if d.paired {
            ("Jumelé", GREY)
        } else {
            ("Appuyer pour jumeler", GREY)
        };
        self.txt(g, sub, 14.0 * s, y + 29.0 * s, 11.0 * s, Kind::Reg, c, Align::Left, Some(w - 50.0 * s), false);
        self.txt(g, "›", w - 16.0 * s, y + (NET_ROW - 20.0 * 1.17) / 2.0 * s, 20.0 * s, Kind::Reg, LINE, Align::Right, None, false);
    }

    fn render_bt(&self, g: &mut Gfx, s: f32) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let present = self.bt.as_ref().map(|b| b.present).unwrap_or(false);
        let powered = self.bt.as_ref().map(|b| b.powered).unwrap_or(false);
        let devs: &[bluetooth::Dev] = self.bt.as_ref().map(|b| &b.devs[..]).unwrap_or(&[]);
        if powered {
            let mut y = (LIST_Y - self.scroll) * s;
            for d in devs {
                if y + NET_ROW * s > 0.0 && y < h {
                    self.dev_row(g, s, y, d);
                }
                y += NET_ROW * s;
            }
        }
        g.rect(0.0, 0.0, w, LIST_Y * s, col(BG, 1.0));
        let (ty, th) = (HEADER + 10.0, 48.0);
        self.card(g, s, ty, th);
        self.txt(g, "Bluetooth", 18.0 * s, (ty + (th - 14.0 * 1.17) / 2.0) * s, 14.0 * s, Kind::Reg, DARK, Align::Left, None, false);
        if self.bt.is_some() && present {
            self.switch(g, s, w - 18.0 * s - 46.0 * s, (ty + 10.0) * s, powered);
        }
        let status = if let Some(b) = &self.busy {
            b.clone()
        } else if let Some(e) = &self.bt_err {
            e.clone()
        } else if self.bt.is_none() {
            String::new()
        } else if !present {
            "Aucun contrôleur Bluetooth détecté".to_string()
        } else if !powered {
            "Bluetooth désactivé".to_string()
        } else if devs.is_empty() {
            "Aucun appareil trouvé".to_string()
        } else {
            "Appareils".to_string()
        };
        let err = self.busy.is_none() && (self.bt_err.is_some() || (self.bt.is_some() && !present));
        self.txt(g, &status, 18.0 * s, (HEADER + 10.0 + 48.0 + 9.0) * s, 11.0 * s, Kind::Reg, if err { RED } else { GREY }, Align::Left, Some(w - 36.0 * s), false);
        self.header(g, s, "Bluetooth", true);
        if powered {
            self.txt(g, "Rechercher", w - 12.0 * s, (HEADER * s - 13.0 * s * 1.17) / 2.0, 13.0 * s, Kind::Reg, BLUE, Align::Right, None, false);
        }
    }

    fn render_dev(&self, g: &mut Gfx, s: f32) {
        let w = self.size.0 as f32;
        let Some(d) = &self.bt_sel else {
            self.header(g, s, "Bluetooth", true);
            return;
        };
        self.card(g, s, NET_INFO_Y, NET_INFO_H);
        let etat = if d.connected {
            "Connecté"
        } else if d.paired {
            "Jumelé"
        } else {
            "Non jumelé"
        };
        let rows = [
            ("Adresse", d.mac.clone()),
            ("Confiance", if d.trusted { "Oui".to_string() } else { "Non".to_string() }),
            ("État", etat.to_string()),
        ];
        for (i, (k, v)) in rows.iter().enumerate() {
            let ry = NET_INFO_Y + 4.0 + i as f32 * 26.0;
            self.txt(g, k, 18.0 * s, (ry + 6.0) * s, 12.0 * s, Kind::Reg, GREY, Align::Left, None, false);
            self.txt(g, v, w - 18.0 * s, (ry + 6.0) * s, 12.0 * s, Kind::Reg, if i == 2 && d.connected { BLUE } else { DARK }, Align::Right, Some(w - 36.0 * s - 80.0 * s), false);
            if i < 2 {
                g.rect(18.0 * s, (ry + 26.0) * s - 1.0, w - 36.0 * s, 1.0, col(LINE, 0.6));
            }
        }
        let mut last = NET_NEXT_Y;
        for (b, by, label) in self.dev_buttons(d) {
            let (bx, bw) = (CARD_X * s, w - 2.0 * CARD_X * s);
            let (bg, fg) = match b {
                Btn::Connect => (BLUE, WHITE),
                Btn::Disconnect => (WHITE, DARK),
                Btn::Forget => (WHITE, RED),
            };
            if bg == WHITE {
                g.rrect(bx, by * s, bw, BTN_H * s, 10.0 * s, col(LINE, 1.0));
                g.rrect(bx + 1.0, by * s + 1.0, bw - 2.0, BTN_H * s - 2.0, 9.0 * s, col(WHITE, 1.0));
            } else {
                g.rrect(bx, by * s, bw, BTN_H * s, 10.0 * s, col(bg, 1.0));
            }
            self.txt(g, label, w / 2.0, (by + (BTN_H - 15.0 * 1.17) / 2.0) * s, 15.0 * s, Kind::Bold, fg, Align::Center, Some(bw - 20.0 * s), false);
            last = by + BTN_H;
        }
        if let Some(b) = &self.busy {
            self.txt(g, b, w / 2.0, (last + 12.0) * s, 12.0 * s, Kind::Reg, GREY, Align::Center, Some(w - 36.0 * s), false);
        }
        self.header(g, s, &d.name, true);
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
        let (a, b) = (self.drain(), self.drain_bt());
        if a || b {
            self.redraw_later();
        }
        let mut until: Option<Instant> = None;
        if let Some((_, t)) = &self.toast {
            if Instant::now() >= *t {
                self.toast = None;
                self.redraw_later();
            } else {
                until = Some(*t);
            }
        }
        if self.jobs > 0 {
            let n = Instant::now() + Duration::from_millis(150);
            until = Some(until.map_or(n, |u| u.min(n)));
        }
        match until {
            Some(u) => el.set_control_flow(ControlFlow::WaitUntil(u)),
            None => el.set_control_flow(ControlFlow::Wait),
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, ev: WindowEvent) {
        match ev {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(_) => self.redraw_later(),
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => self.key(el, &event),
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
        if args.iter().any(|a| a == "--demo") {
            app.wifi = Some(wifi::demo());
            app.bt = Some(bluetooth::demo());
        }
        match val("--page").as_deref() {
            Some("info") => {
                app.info = info_rows();
                app.page = Page::Info;
            }
            Some("wifi") => app.page = Page::Wifi,
            Some(p @ ("sudo" | "sudowrong")) => {
                app.sudo_from = Page::Wifi;
                app.sudo_wrong = p == "sudowrong";
                app.sudo_pw = "motdepasse".into();
                app.page = Page::Sudo;
            }
            Some("bt") => app.page = Page::Bt,
            Some("btdev") => {
                app.bt_sel = app.bt.as_ref().and_then(|st| st.devs.get(3).cloned());
                app.page = Page::BtDev;
            }
            Some(p @ ("wifinet" | "wifipass")) => {
                let i = if p == "wifinet" { 1 } else { 3 };
                app.sel = app.wifi.as_ref().and_then(|st| st.nets.get(i).cloned());
                if p == "wifipass" {
                    app.pass = "hunter2hunter2".into();
                }
                app.page = Page::WifiNet;
            }
            _ => {}
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
    app.job("", wifi::Action::None, false); // état du Wi-Fi pour la carte de la page principale
    app.bt_job("", bluetooth::Action::None); // état du Bluetooth pour la carte de la page principale
    if let Err(e) = el.run_app(&mut app) {
        eprintln!("settings : {e}");
        std::process::exit(1);
    }
}