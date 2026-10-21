// files : navigateur de fichiers tactile pour lapp (écran portrait 272x480), rendu OpenGL ES.
//
// - toucher un dossier pour l'ouvrir, un fichier pour voir ses infos + un aperçu texte
// - glisser verticalement pour défiler (avec inertie), "‹ Retour" pour remonter
// - "Cachés" affiche / masque les fichiers commençant par un point
// - quitter : glisser vers le haut depuis le bord bas de l'écran, ou Echap
//
// Dossier de départ : --path <dir>, sinon $LAPP_START, sinon /host (permission "files"), sinon /data, sinon /.
// Rendu : GPU (EGL / OpenGL ES 2) avec repli CPU (softbuffer) ; LAPP_CPU=1 force le CPU.
// Tests hors écran : files --screenshot out.png [--path DIR] [--detail NOM] [--hidden] [--size 272x480]
//                    files --icon icon.png

mod gfx;

use std::fs;
use std::io::Read;
use std::num::NonZeroU32;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;
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
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent};
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
        eprintln!("files : aucune police trouvée, texte désactivé");
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

// ------------------------------------------------------------------ données

#[derive(Clone)]
struct Entry {
    name: String,
    is_dir: bool,
    is_link: bool,
    size: u64,
    mtime: i64,
    mode: u32,
}

enum Preview {
    None,
    Binary,
    Text(Vec<String>),
    Err(String),
}

struct Detail {
    entry: Entry,
    link: Option<String>,
    preview: Preview,
}

fn read_entries(path: &Path, hidden: bool) -> Result<Vec<Entry>, String> {
    let rd = fs::read_dir(path).map_err(|e| format!("{} : {}", path.display(), err_fr(&e)))?;
    let mut v = Vec::new();
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if !hidden && name.starts_with('.') {
            continue;
        }
        let Ok(lm) = e.metadata() else { continue };
        let is_link = lm.file_type().is_symlink();
        let m = if is_link { fs::metadata(e.path()).unwrap_or_else(|_| lm.clone()) } else { lm };
        v.push(Entry { name, is_dir: m.is_dir(), is_link, size: m.len(), mtime: m.mtime(), mode: m.mode() });
    }
    v.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(v)
}

fn err_fr(e: &std::io::Error) -> String {
    match e.kind() {
        std::io::ErrorKind::PermissionDenied => "accès refusé".into(),
        std::io::ErrorKind::NotFound => "introuvable".into(),
        _ => e.to_string(),
    }
}

fn load_detail(dir: &Path, e: &Entry) -> Detail {
    let p = dir.join(&e.name);
    let link = if e.is_link { fs::read_link(&p).ok().map(|t| t.to_string_lossy().into_owned()) } else { None };
    let preview = match fs::metadata(&p) {
        Ok(m) if m.is_file() && m.len() > 0 => match fs::File::open(&p) {
            Ok(f) => {
                let mut buf = Vec::new();
                match f.take(8192).read_to_end(&mut buf) {
                    Ok(_) if buf.contains(&0) => Preview::Binary,
                    Ok(_) => {
                        let text = String::from_utf8_lossy(&buf);
                        let lines = text
                            .lines()
                            .take(60)
                            .map(|l| l.replace('\t', "    ").chars().filter(|c| !c.is_control()).take(200).collect::<String>())
                            .collect();
                        Preview::Text(lines)
                    }
                    Err(er) => Preview::Err(err_fr(&er)),
                }
            }
            Err(er) => Preview::Err(err_fr(&er)),
        },
        _ => Preview::None,
    };
    Detail { entry: e.clone(), link, preview }
}

fn fmt_size(n: u64) -> String {
    const U: [&str; 4] = ["Ko", "Mo", "Go", "To"];
    if n < 1000 {
        return format!("{n} o");
    }
    let mut v = n as f64 / 1024.0;
    let mut i = 0;
    while v >= 1000.0 && i < U.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if v < 10.0 {
        format!("{v:.1} {}", U[i])
    } else {
        format!("{v:.0} {}", U[i])
    }
}

/// Date UTC "AAAA-MM-JJ HH:MM" (pas de fuseau horaire dans le bac à sable).
fn fmt_date(secs: i64) -> String {
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let (hh, mm) = (rem / 3600, rem % 3600 / 60);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + (m <= 2) as i64;
    format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}")
}

fn fmt_mode(mode: u32) -> String {
    let mut s = String::new();
    for (i, c) in "rwxrwxrwx".chars().enumerate() {
        s.push(if mode & (1 << (8 - i)) != 0 { c } else { '-' });
    }
    format!("{s} ({:03o})", mode & 0o777)
}

fn kind_label(e: &Entry) -> &'static str {
    if e.is_dir {
        if e.is_link { "Lien vers un dossier" } else { "Dossier" }
    } else if e.is_link {
        "Lien vers un fichier"
    } else {
        "Fichier"
    }
}

/// Teinte de la vignette de fichier selon l'extension.
fn tint(name: &str) -> Rgb {
    let ext = name.rsplit_once('.').map(|(_, e)| e.to_lowercase()).unwrap_or_default();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" => [0.2, 0.7, 0.4],
        "rs" | "py" | "c" | "h" | "sh" | "js" | "toml" | "json" | "cpp" => [0.95, 0.5, 0.1],
        "mp3" | "wav" | "ogg" | "flac" | "mp4" | "mkv" => [0.8, 0.25, 0.5],
        "zip" | "gz" | "xz" | "zst" | "tar" | "squashfs" | "lapp" => [0.55, 0.35, 0.8],
        "txt" | "md" | "log" | "conf" | "cfg" => [0.35, 0.45, 0.6],
        _ => [0.5, 0.5, 0.55],
    }
}

// ------------------------------------------------------------------ état

/// Binaire lapp visible dans le bac à sable (permission "install") ; LAPP_BIN pour les tests.
fn lapp_bin() -> String {
    std::env::var("LAPP_BIN").unwrap_or_else(|_| "/lapp".into())
}

fn first_line(b: &[u8]) -> String {
    String::from_utf8_lossy(b).lines().map(|l| l.trim()).find(|l| !l.is_empty()).unwrap_or("").trim_start_matches("Error:").trim().to_string()
}

/// Lignes de `lapp info` ; Err = message à afficher.
fn lapp_info(path: &Path) -> Result<Vec<String>, String> {
    let o = Command::new(lapp_bin()).arg("info").arg(path).output().map_err(|_| "Installation indisponible (permission install)".to_string())?;
    if !o.status.success() {
        let e = first_line(&o.stderr);
        return Err(if e.is_empty() { "Fichier .lapp invalide".into() } else { e });
    }
    Ok(String::from_utf8_lossy(&o.stdout).lines().map(|l| l.to_string()).collect())
}

fn lapp_install(path: &Path, untrusted: bool) -> Result<String, String> {
    let mut c = Command::new(lapp_bin());
    c.arg("install").arg(path);
    if untrusted {
        c.arg("--allow-untrusted");
    }
    let o = c.output().map_err(|_| "Installation indisponible (permission install)".to_string())?;
    if !o.status.success() {
        return Err(first_line(&o.stderr));
    }
    Ok(first_line(&o.stdout))
}

#[derive(Clone)]
struct Modal {
    title: String,
    lines: Vec<String>,
    ok: String,
    path: PathBuf,
    untrusted: bool,
}

struct Press {
    start: (f32, f32),
    last: (f32, f32),
    last_t: Instant,
    vel: f32, // px/s, sens du doigt
    dragging: bool,
}

struct GlState {
    surface: Surface<WindowSurface>,
    ctx: PossiblyCurrentContext,
    gfx: Gfx,
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
    press: Option<Press>,
    cwd: PathBuf,
    entries: Vec<Entry>,
    scroll: f32,
    scroll_v: f32,
    last_frame: Instant,
    stack: Vec<f32>,
    show_hidden: bool,
    detail: Option<Detail>,
    toast: Option<(String, Instant)>,
    modal: Option<Modal>,
    start: PathBuf,
}

impl App {
    fn new(start: PathBuf, size: (u32, u32)) -> App {
        let mut app = App {
            window: None,
            surface: None,
            gl: None,
            cpu: None,
            surf_size: (0, 0),
            fonts: load_fonts(),
            size,
            cursor: (0.0, 0.0),
            press: None,
            cwd: PathBuf::from("/"),
            entries: Vec::new(),
            scroll: 0.0,
            scroll_v: 0.0,
            last_frame: Instant::now(),
            stack: Vec::new(),
            show_hidden: false,
            detail: None,
            toast: None,
            modal: None,
            start: start.clone(),
        };
        app.goto_start();
        app
    }

    fn goto_start(&mut self) {
        for c in [self.start.clone(), PathBuf::from("/")] {
            let c = fs::canonicalize(&c).unwrap_or(c);
            if let Ok(v) = read_entries(&c, self.show_hidden) {
                self.cwd = c;
                self.entries = v;
                return;
            }
        }
    }

    fn scale(&self) -> f32 {
        if let Some(s) = scale_override() {
            return s;
        }
        self.size.0.min(self.size.1) as f32 / 272.0
    }

    fn list_top(&self) -> f32 {
        (HEADER + PATHBAR) * self.scale()
    }

    fn list_bottom(&self) -> f32 {
        self.size.1 as f32 - PAD_BOTTOM * self.scale()
    }

    fn max_scroll(&self) -> f32 {
        let content = self.entries.len() as f32 * ROW * self.scale();
        (content - (self.list_bottom() - self.list_top())).max(0.0)
    }

    fn clamp_scroll(&mut self) {
        self.scroll = self.scroll.clamp(0.0, self.max_scroll());
    }

    fn set_toast(&mut self, msg: String) {
        self.toast = Some((msg, Instant::now() + Duration::from_millis(2800)));
    }

    fn navigate(&mut self, path: PathBuf, back: bool) {
        match read_entries(&path, self.show_hidden) {
            Ok(v) => {
                if back {
                    self.scroll = self.stack.pop().unwrap_or(0.0);
                } else {
                    self.stack.push(self.scroll);
                    self.scroll = 0.0;
                }
                self.scroll_v = 0.0;
                self.cwd = path;
                self.entries = v;
                self.clamp_scroll();
            }
            Err(e) => self.set_toast(e),
        }
    }

    fn up(&mut self) {
        if let Some(p) = self.cwd.parent().map(Path::to_path_buf) {
            self.navigate(p, true);
        }
    }

    fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        match read_entries(&self.cwd, self.show_hidden) {
            Ok(v) => {
                self.entries = v;
                self.clamp_scroll();
            }
            Err(e) => self.set_toast(e),
        }
    }

    fn row_at(&self, y: f32) -> Option<usize> {
        let (top, bottom) = (self.list_top(), self.list_bottom());
        if y < top || y >= bottom {
            return None;
        }
        let i = ((y - top + self.scroll) / (ROW * self.scale())) as usize;
        (i < self.entries.len()).then_some(i)
    }

    fn modal_geom(&self, m: &Modal) -> (f32, f32, f32, f32) {
        let (w, h, s) = (self.size.0 as f32, self.size.1 as f32, self.scale());
        let cw = w - 36.0 * s;
        let ch = (16.0 + 20.0 + 10.0 + m.lines.len() as f32 * 18.0 + 14.0 + 40.0 + 14.0) * s;
        (18.0 * s, ((h - ch) / 2.0 - 20.0 * s).max(10.0 * s), cw, ch)
    }

    fn ask_install(&mut self, path: PathBuf) {
        match lapp_info(&path) {
            Ok(l) => {
                let mut lines = vec![l.first().cloned().unwrap_or_default()];
                if let Some(p) = l.iter().find(|x| x.starts_with("permissions")) {
                    lines.push(format!("Permissions :{}", p.split_once(':').map(|v| v.1).unwrap_or("")));
                }
                self.modal = Some(Modal { title: "Installer l'application ?".into(), lines, ok: "Installer".into(), path, untrusted: false });
            }
            Err(e) => self.set_toast(e),
        }
    }

    fn modal_tap(&mut self, x: f32, y: f32) {
        let Some(m) = self.modal.clone() else { return };
        let s = self.scale();
        let (cx, cy, cw, ch) = self.modal_geom(&m);
        let inside = x >= cx && x < cx + cw && y >= cy && y < cy + ch;
        let by = cy + ch - 54.0 * s;
        let bw = (cw - 30.0 * s) / 2.0;
        let on_btn = y >= by && y < by + 40.0 * s;
        if inside && on_btn && x >= cx + 20.0 * s + bw {
            match lapp_install(&m.path, m.untrusted) {
                Ok(msg) => {
                    self.modal = None;
                    self.set_toast(if msg.is_empty() { "Installé".into() } else { msg });
                }
                Err(e) if e.contains("non approuvé") && !m.untrusted => {
                    self.modal = Some(Modal {
                        title: "Signataire non approuvé".into(),
                        lines: vec!["Cette app est signée par une clé inconnue.".into(), "L'installer quand même ?".into()],
                        ok: "Installer".into(),
                        path: m.path,
                        untrusted: true,
                    });
                }
                Err(e) => {
                    self.modal = None;
                    self.set_toast(if e.is_empty() { "Échec de l'installation".into() } else { e });
                }
            }
        } else if !inside || (on_btn && x < cx + 10.0 * s + bw + 10.0 * s) {
            self.modal = None; // Annuler ou toucher en dehors
        }
    }

    fn render_modal(&self, g: &mut Gfx, s: f32, m: &Modal) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        g.rect(0.0, 0.0, w, h, col([0.0, 0.0, 0.0], 0.45));
        let (cx, cy, cw, ch) = self.modal_geom(m);
        g.rrect(cx, cy, cw, ch, 14.0 * s, col(LINE, 1.0));
        g.rrect(cx + 1.0, cy + 1.0, cw - 2.0, ch - 2.0, 13.0 * s, col(WHITE, 1.0));
        self.txt(g, &m.title, cx + cw / 2.0, cy + 16.0 * s, 15.0 * s, Kind::Bold, DARK, Align::Center, Some(cw - 24.0 * s), false);
        for (i, l) in m.lines.iter().enumerate() {
            self.txt(g, l, cx + cw / 2.0, cy + (16.0 + 20.0 + 10.0 + i as f32 * 18.0) * s, 12.0 * s, Kind::Reg, GREY, Align::Center, Some(cw - 24.0 * s), false);
        }
        let by = cy + ch - 54.0 * s;
        let bw = (cw - 30.0 * s) / 2.0;
        g.rrect(cx + 10.0 * s, by, bw, 40.0 * s, 9.0 * s, col(LINE, 1.0));
        g.rrect(cx + 10.0 * s + 1.0, by + 1.0, bw - 2.0, 40.0 * s - 2.0, 8.0 * s, col(BG, 1.0));
        self.txt(g, "Annuler", cx + 10.0 * s + bw / 2.0, by + (40.0 * s - 15.0 * s * 1.17) / 2.0, 15.0 * s, Kind::Reg, DARK, Align::Center, None, false);
        g.rrect(cx + 20.0 * s + bw, by, bw, 40.0 * s, 9.0 * s, col(BLUE, 1.0));
        self.txt(g, &m.ok, cx + 20.0 * s + bw + bw / 2.0, by + (40.0 * s - 15.0 * s * 1.17) / 2.0, 15.0 * s, Kind::Bold, WHITE, Align::Center, None, false);
    }

    fn tap(&mut self, x: f32, y: f32) {
        if self.modal.is_some() {
            self.modal_tap(x, y);
            return;
        }
        let (w, s) = (self.size.0 as f32, self.scale());
        if y < HEADER * s {
            if x < 84.0 * s {
                if self.detail.is_some() {
                    self.detail = None;
                } else {
                    self.up();
                }
            } else if x > w - 84.0 * s && self.detail.is_none() {
                self.toggle_hidden();
            }
            return;
        }
        if self.detail.is_some() {
            return;
        }
        if let Some(i) = self.row_at(y) {
            let e = self.entries[i].clone();
            if e.is_dir {
                let p = self.cwd.join(&e.name);
                self.navigate(p, false);
            } else if e.name.to_lowercase().ends_with(".lapp") {
                let p = self.cwd.join(&e.name);
                self.ask_install(p);
            } else {
                self.detail = Some(load_detail(&self.cwd, &e));
            }
        }
    }

    fn press(&mut self, x: f32, y: f32) {
        self.scroll_v = 0.0;
        self.press = Some(Press { start: (x, y), last: (x, y), last_t: Instant::now(), vel: 0.0, dragging: false });
        self.redraw_later();
    }

    fn moved(&mut self, x: f32, y: f32) {
        let s = self.scale();
        let h = self.size.1 as f32;
        let list = self.detail.is_none() && self.modal.is_none();
        let Some(p) = self.press.as_mut() else { return };
        if !p.dragging && ((x - p.start.0).abs() > 9.0 * s || (y - p.start.1).abs() > 9.0 * s) {
            p.dragging = true;
        }
        let now = Instant::now();
        let dt = (now - p.last_t).as_secs_f32().max(0.001);
        let dy = y - p.last.1;
        p.vel = 0.65 * (dy / dt) + 0.35 * p.vel;
        p.last = (x, y);
        p.last_t = now;
        if p.dragging && list && p.start.1 < h * 0.94 {
            self.scroll -= dy;
            self.clamp_scroll();
        }
        self.redraw_later();
    }

    fn release(&mut self, x: f32, y: f32, el: Option<&ActiveEventLoop>) {
        let Some(p) = self.press.take() else { return };
        let h = self.size.1 as f32;
        if p.start.1 > h * 0.94 && p.start.1 - y > h * 0.08 {
            if let Some(el) = el {
                el.exit(); // glisser vers le haut depuis le bas de l'écran
            }
        } else if !p.dragging {
            self.tap(x, y);
        } else if self.detail.is_none() && p.last_t.elapsed() < Duration::from_millis(90) && p.vel.abs() > 120.0 {
            self.scroll_v = -p.vel; // inertie
            self.last_frame = Instant::now();
        }
        self.redraw_later();
    }

    fn redraw_later(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    // -------------------------------------------------------------- rendu

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

    fn render(&self, g: &mut Gfx) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let s = self.scale();
        g.rect(0.0, 0.0, w, h, col(BG, 1.0));
        match &self.detail {
            None => self.render_list(g, s),
            Some(d) => self.render_detail(g, s, d),
        }
        // indicateur de sortie (glisser vers le haut)
        g.rect(0.0, h - PAD_BOTTOM * s, w, PAD_BOTTOM * s, col(BG, 1.0));
        g.rrect(w * 0.35, h - w * 0.035, w * 0.3, w * 0.012, w * 0.006, col(LINE, 1.0));

        if let Some(m) = &self.modal {
            self.render_modal(g, s, m);
        }
        if let Some((msg, _)) = &self.toast {
            let bw = w - 28.0 * s;
            let bh = 34.0 * s;
            let by = h - PAD_BOTTOM * s - bh - 8.0 * s;
            g.rrect(14.0 * s, by, bw, bh, 9.0 * s, col([0.1, 0.1, 0.12], 0.92));
            self.txt(g, msg, w / 2.0, by + (bh - 12.0 * s * 1.17) / 2.0, 12.0 * s, Kind::Reg, WHITE, Align::Center, Some(bw - 16.0 * s), false);
        }
    }

    fn icon_dir(&self, g: &mut Gfx, x: f32, y: f32, sz: f32) {
        let key = format!("d|{}", sz as u32);
        g.img(&key, &|| folder_pm(sz.floor()));
        g.blit(&key, x, y, 1.0);
    }

    fn icon_file(&self, g: &mut Gfx, x: f32, y: f32, sz: f32, t: Rgb) {
        let key = format!("f|{}|{:02x}{:02x}{:02x}", sz as u32, (t[0] * 255.0) as u8, (t[1] * 255.0) as u8, (t[2] * 255.0) as u8);
        g.img(&key, &|| file_pm(sz.floor(), t));
        g.blit(&key, x, y, 1.0);
    }

    fn header(&self, g: &mut Gfx, s: f32, title: &str, back: Option<&str>, hidden_btn: bool) {
        let w = self.size.0 as f32;
        g.rect(0.0, 0.0, w, HEADER * s, col(BAR, 1.0));
        g.rect(0.0, HEADER * s - 1.0, w, 1.0, col(LINE, 1.0));
        let ty = (HEADER * s - 16.0 * s * 1.17) / 2.0;
        if let Some(b) = back {
            self.txt(g, b, 10.0 * s, (HEADER * s - 15.0 * s * 1.17) / 2.0, 15.0 * s, Kind::Reg, BLUE, Align::Left, None, false);
        }
        if hidden_btn {
            let (bx, by, bw, bh) = (w - 62.0 * s, 10.0 * s, 54.0 * s, 24.0 * s);
            g.rrect(bx, by, bw, bh, 6.0 * s, col(BLUE, 1.0));
            let ink = if self.show_hidden {
                WHITE
            } else {
                g.rrect(bx + 1.2 * s, by + 1.2 * s, bw - 2.4 * s, bh - 2.4 * s, 5.0 * s, col(BAR, 1.0));
                BLUE
            };
            self.txt(g, "Cachés", bx + bw / 2.0, by + (bh - 12.0 * s * 1.17) / 2.0, 12.0 * s, Kind::Bold, ink, Align::Center, None, false);
        }
        self.txt(g, title, w / 2.0, ty, 16.0 * s, Kind::Bold, DARK, Align::Center, Some(w - 2.0 * 84.0 * s), false);
    }

    fn render_list(&self, g: &mut Gfx, s: f32) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let (top, bottom) = (self.list_top(), self.list_bottom());
        let row_h = ROW * s;

        if self.entries.is_empty() {
            self.txt(g, "Dossier vide", w / 2.0, top + 60.0 * s, 14.0 * s, Kind::Reg, GREY, Align::Center, None, false);
        }
        let pressed = self.press.as_ref().filter(|p| !p.dragging).and_then(|p| self.row_at(p.start.1));
        let first = (self.scroll / row_h).floor().max(0.0) as usize;
        for i in first..self.entries.len() {
            let y = top + i as f32 * row_h - self.scroll;
            if y >= bottom {
                break;
            }
            let e = &self.entries[i];
            if pressed == Some(i) {
                g.rect(0.0, y, w, row_h, col(PRESS, 1.0));
            }
            let isz = 32.0 * s;
            let (ix, iy) = (14.0 * s, y + (row_h - isz) / 2.0);
            if e.is_dir {
                self.icon_dir(g, ix, iy, isz);
            } else {
                self.icon_file(g, ix, iy, isz, tint(&e.name));
            }
            let tx = 58.0 * s;
            let maxw = w - tx - 26.0 * s;
            self.txt(g, &e.name, tx, y + 9.0 * s, 15.0 * s, Kind::Reg, DARK, Align::Left, Some(maxw), false);
            let sub = if e.is_dir { fmt_date(e.mtime) } else { format!("{} · {}", fmt_size(e.size), fmt_date(e.mtime)) };
            let sub = if e.is_link { format!("↪ {sub}") } else { sub };
            self.txt(g, &sub, tx, y + 29.0 * s, 11.0 * s, Kind::Reg, GREY, Align::Left, Some(maxw), false);
            if e.is_dir {
                self.txt(g, "›", w - 12.0 * s, y + (row_h - 20.0 * s * 1.17) / 2.0, 20.0 * s, Kind::Reg, LINE, Align::Right, None, false);
            }
            g.rect(tx, y + row_h - 1.0, w - tx, 1.0, col(LINE, 0.6));
        }

        // barres par-dessus le défilement
        g.rect(0.0, bottom, w, h - bottom, col(BG, 1.0));
        let title = if self.cwd == Path::new("/") {
            "/".to_string()
        } else {
            self.cwd.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "/".into())
        };
        self.header(g, s, &title, self.cwd.parent().map(|_| "‹ Retour"), true);
        g.rect(0.0, HEADER * s, w, PATHBAR * s, col(BG, 1.0));
        g.rect(0.0, top - 1.0, w, 1.0, col(LINE, 0.7));
        let p = self.cwd.to_string_lossy();
        self.txt(g, &p, 10.0 * s, HEADER * s + (PATHBAR * s - 10.0 * s * 1.17) / 2.0, 10.0 * s, Kind::Mono, GREY, Align::Left, Some(w - 20.0 * s), true);

        // barre de défilement
        let ms = self.max_scroll();
        if ms > 0.0 {
            let track = bottom - top;
            let content = track + ms;
            let bh = (track * track / content).max(24.0 * s);
            let by = top + (track - bh) * (self.scroll / ms);
            g.rrect(w - 4.0 * s, by, 2.5 * s, bh, 1.2 * s, col([0.0, 0.0, 0.0], 0.28));
        }
    }

    fn render_detail(&self, g: &mut Gfx, s: f32, d: &Detail) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let e = &d.entry;
        let top = HEADER * s;
        let isz = 56.0 * s;
        self.icon_file(g, (w - isz) / 2.0, top + 14.0 * s, isz, tint(&e.name));
        self.txt(g, &e.name, w / 2.0, top + 78.0 * s, 15.0 * s, Kind::Bold, DARK, Align::Center, Some(w - 24.0 * s), false);
        self.txt(g, kind_label(e), w / 2.0, top + 98.0 * s, 11.0 * s, Kind::Reg, GREY, Align::Center, None, false);

        let mut rows: Vec<(&str, String)> = vec![
            ("Taille", format!("{} ({} o)", fmt_size(e.size), e.size)),
            ("Modifié", format!("{} UTC", fmt_date(e.mtime))),
            ("Droits", fmt_mode(e.mode)),
        ];
        if let Some(l) = &d.link {
            rows.push(("Lien vers", l.clone()));
        }
        let mut y = top + 118.0 * s;
        for (k, v) in &rows {
            g.rect(14.0 * s, y, w - 28.0 * s, 1.0, col(LINE, 0.7));
            self.txt(g, k, 14.0 * s, y + 6.0 * s, 12.0 * s, Kind::Reg, GREY, Align::Left, None, false);
            self.txt(g, v, w - 14.0 * s, y + 6.0 * s, 12.0 * s, Kind::Reg, DARK, Align::Right, Some(w - 28.0 * s - 70.0 * s), true);
            y += 25.0 * s;
        }
        g.rect(14.0 * s, y, w - 28.0 * s, 1.0, col(LINE, 0.7));
        y += 10.0 * s;

        // aperçu
        let bottom = h - PAD_BOTTOM * s - 6.0 * s;
        let bh = bottom - y;
        if bh >= 40.0 * s {
            g.rrect(10.0 * s, y, w - 20.0 * s, bh, 8.0 * s, col(LINE, 1.0));
            g.rrect(11.0 * s, y + 1.0 * s, w - 22.0 * s, bh - 2.0 * s, 7.0 * s, col(WHITE, 1.0));
            let (px, lh) = (10.0 * s, 12.5 * s);
            let inner_w = w - 40.0 * s;
            let note: Option<&str> = match &d.preview {
                Preview::None => Some("Pas d'aperçu"),
                Preview::Binary => Some("Fichier binaire"),
                Preview::Err(m) => Some(m.as_str()),
                Preview::Text(_) => None,
            };
            if let Some(m) = note {
                self.txt(g, m, w / 2.0, y + bh / 2.0 - 8.0 * s, 12.0 * s, Kind::Reg, GREY, Align::Center, Some(inner_w), false);
            }
            if let Preview::Text(lines) = &d.preview {
                let n = (((bh - 12.0 * s) / lh) as usize).max(1);
                for (i, l) in lines.iter().take(n).enumerate() {
                    self.txt(g, l, 18.0 * s, y + 6.0 * s + i as f32 * lh, px, Kind::Mono, DARK, Align::Left, Some(inner_w), false);
                }
            }
        }
        self.header(g, s, "Détails", Some("‹ Liste"), false);
    }

    fn redraw(&mut self) {
        // inertie du défilement
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32().min(0.05);
        self.last_frame = now;
        if self.press.is_none() && self.scroll_v.abs() > 15.0 && self.detail.is_none() {
            let before = self.scroll;
            self.scroll += self.scroll_v * dt;
            self.clamp_scroll();
            if (self.scroll - before).abs() < 0.01 {
                self.scroll_v = 0.0;
            } else {
                self.scroll_v *= (-dt * 3.2).exp();
            }
        } else if self.press.is_none() {
            self.scroll_v = 0.0;
        }

        let Some(window) = self.window.clone() else { return };
        let sz = window.inner_size();
        let (Some(nw), Some(nh)) = (NonZeroU32::new(sz.width), NonZeroU32::new(sz.height)) else { return };
        self.size = (sz.width, sz.height);
        self.clamp_scroll();

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
        if self.scroll_v.abs() > 15.0 {
            window.request_redraw();
        }
    }

    /// Crée la fenêtre et un contexte OpenGL ES (EGL) ; None si le GPU n'est pas utilisable (repli CPU).
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
                eprintln!("files : EGL indisponible, repli CPU : {e}");
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
                eprintln!("files : contexte GLES impossible, repli CPU : {e}");
                return None;
            }
        };
        let sa = window.build_surface_attributes(Default::default()).ok()?;
        let surface = match unsafe { display.create_window_surface(&config, &sa) } {
            Ok(s) => s,
            Err(e) => {
                eprintln!("files : surface EGL impossible, repli CPU : {e}");
                return None;
            }
        };
        let ctx = nc.make_current(&surface).ok()?;
        let _ = surface.set_swap_interval(&ctx, SwapInterval::Wait(NonZeroU32::new(1).unwrap()));
        let gl = unsafe { glow::Context::from_loader_function_cstr(|s| display.get_proc_address(s)) };
        let g = match gfx::GlGfx::new(gl) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("files : shaders GLES refusés, repli CPU : {e}");
                return None;
            }
        };
        let rn = g.renderer();
        let low = rn.to_lowercase();
        if low.contains("llvmpipe") || low.contains("softpipe") || low.contains("swrast") {
            // rendu logiciel de Mesa : plus lent que le CPU maison et inutilisable sans mémoire partagée X
            eprintln!("files : EGL sans GPU réel ({rn}), repli CPU");
            return None;
        }
        eprintln!("files : rendu GPU ({rn})");
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
            .with_title("Fichiers")
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
                        eprintln!("files : fenêtre impossible : {e}");
                        el.exit();
                        return;
                    }
                };
                let ctx = match softbuffer::Context::new(window.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("files : contexte graphique : {e}");
                        el.exit();
                        return;
                    }
                };
                match softbuffer::Surface::new(&ctx, window.clone()) {
                    Ok(s) => self.surface = Some(s),
                    Err(e) => {
                        eprintln!("files : surface graphique : {e}");
                        el.exit();
                        return;
                    }
                }
                eprintln!("files : rendu CPU");
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
            WindowEvent::KeyboardInput { event: KeyEvent { logical_key: Key::Named(k), state: ElementState::Pressed, .. }, .. } => match k {
                NamedKey::Escape => {
                    if self.modal.is_some() {
                        self.modal = None;
                        self.redraw_later();
                    } else if self.detail.is_some() {
                        self.detail = None;
                        self.redraw_later();
                    } else {
                        el.exit();
                    }
                }
                NamedKey::Backspace => {
                    if self.detail.is_some() {
                        self.detail = None;
                    } else {
                        self.up();
                    }
                    self.redraw_later();
                }
                _ => {}
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y * ROW * self.scale(),
                    MouseScrollDelta::PixelDelta(p) => p.y as f32,
                };
                if self.detail.is_none() {
                    self.scroll -= dy;
                    self.clamp_scroll();
                    self.redraw_later();
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
    fill_rrect(&mut pm, 0.0, 0.0, f, f, f * 0.22, col([0.0, 0.40, 0.92], 1.0));
    fill_rrect(&mut pm, 0.0, f * 0.5, f, f * 0.5, f * 0.22, col([0.0, 0.52, 1.0], 1.0));
    fill_rect(&mut pm, 0.0, f * 0.5, f, f * 0.2, col([0.0, 0.52, 1.0], 1.0));
    draw_folder(&mut pm, f * 0.2, f * 0.22, f * 0.6);
    fill_rrect(&mut pm, f * 0.2, f * 0.22 + f * 0.6 * 0.46, f * 0.6, f * 0.6 * 0.44, f * 0.06, col(WHITE, 0.30));
    if let Err(e) = pm.save_png(path) {
        eprintln!("files : icône : {e}");
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
    let flag = |k: &str| args.iter().any(|a| a == k);

    if let Some(p) = val("--icon") {
        make_icon(&p);
        return;
    }

    let start = val("--path").or_else(|| std::env::var("LAPP_START").ok()).map(PathBuf::from).unwrap_or_else(|| {
        // permission "files" : le système hôte est visible sur /host
        if Path::new("/host/data").is_dir() { PathBuf::from("/host") } else { PathBuf::from("/data") }
    });
    let size = val("--size")
        .and_then(|s| {
            let (a, b) = s.split_once('x')?;
            Some((a.parse().ok()?, b.parse().ok()?))
        })
        .unwrap_or((272, 480));

    // rendu hors écran (CPU) pour les tests
    if let Some(out) = val("--screenshot") {
        let mut app = App::new(start, size);
        if flag("--hidden") {
            app.toggle_hidden();
        }
        if let Some(n) = val("--scroll").and_then(|v| v.parse::<f32>().ok()) {
            app.scroll = n;
            app.clamp_scroll();
        }
        if let Some(f) = val("--ask") {
            app.ask_install(PathBuf::from(f));
            if let Some((m, _)) = app.toast.clone() {
                eprintln!("files : {m}");
            }
        }
        if let Some(name) = val("--detail") {
            if let Some(e) = app.entries.iter().find(|e| e.name == name).cloned() {
                app.detail = Some(load_detail(&app.cwd, &e));
            } else {
                eprintln!("files : {name} introuvable dans {}", app.cwd.display());
            }
        }
        let mut g = Gfx::new_cpu(size.0, size.1);
        g.begin(size.0, size.1);
        app.render(&mut g);
        if let Err(e) = g.canvas().unwrap().save_png(&out) {
            eprintln!("files : {e}");
            std::process::exit(1);
        }
        return;
    }

    let el = match EventLoop::new() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("files : pas d'affichage disponible : {e}");
            std::process::exit(1);
        }
    };
    el.set_control_flow(ControlFlow::Wait);
    let mut app = App::new(start, size);
    if let Err(e) = el.run_app(&mut app) {
        eprintln!("files : {e}");
        std::process::exit(1);
    }
}