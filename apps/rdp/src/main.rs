// RDP : client Bureau à distance Microsoft (RDP, NLA) pour lapp.
//
// - Formulaire de connexion (hôte[:port], utilisateur, mot de passe, domaine), résolution, disposition du clavier distant
// - Session : bureau distant affiché avec zoom / déplacement ; toucher = clic, appui long = clic droit
// - Modes de glissement : Pan (déplacer la vue), Souris (glisser-déposer), Défil. (molette)
// - Clavier virtuel de lapp-home + panneau de touches spéciales (Ctrl, Alt, Maj, Win, Échap, Tab, flèches, Suppr)
// - Le certificat du serveur est mémorisé à la première connexion (empreinte) ; un changement est signalé
//
// Sur PC : cargo run -- --screenshot out.png [--page session]   (rendu hors écran, la page « session » est factice)
//          cargo run -- --icon icon.png

mod rdp;

use std::time::{Duration, Instant};

use lapp_sdk::prelude::*;
use lapp_sdk::{arg_value, tiny_skia};
use rdp::*;

const RES: &[(u16, u16)] = &[(1024, 768), (800, 600), (1280, 720), (1366, 768)];
/// (libellé, zoom) ; 0.0 = ajusté à l'écran
const ZOOMS: &[(&str, f32)] = &[("Ajusté", 0.0), ("50 %", 0.5), ("75 %", 0.75), ("100 %", 1.0), ("150 %", 1.5), ("200 %", 2.0)];
const MODES: &[&str] = &["Pan", "Souris", "Défil."];
const KEYROWS: [[&str; 6]; 2] = [["Ctrl", "Alt", "Maj", "Win", "Échap", "Tab"], ["Suppr", "C-A-Fin", "←", "↑", "↓", "→"]];
const LABELS: [&str; 4] = ["PC (adresse ou adresse:port)", "Utilisateur", "Mot de passe", "Domaine (facultatif)"];
const LONG_PRESS: f32 = 0.6;

#[derive(PartialEq, Clone, Copy)]
enum Screen {
    Form,
    Session,
}

struct Touch {
    x0: f32,
    y0: f32,
    t0: Instant,
    moved: bool,
    dragging: bool,
}

struct FormLayout {
    fields: [Rect; 4],
    res: Rect,
    kbd: Rect,
    nla: Rect,
    keep: Rect,
    notice_y: f32,
    notice: Vec<String>,
    connect: Rect,
    content_h: f32,
}

struct Rdp {
    store: Store,
    screen: Screen,
    f: [String; 4],
    focus: Option<usize>,
    res_idx: usize,
    azerty: bool,
    nla: bool,
    keep_pass: bool,
    notice: String,
    scroll: Scroller,
    head_h: f32,
    focus_dirty: bool,
    last_kbd: f32,
    // session
    sess: Option<Session>,
    conn_key: String,
    fp_saved: bool,
    zoom_idx: usize,
    mode: usize,
    right_once: bool,
    show_keys: bool,
    mods: [bool; 4], // Ctrl, Alt, Maj, Win (collées : valables pour la prochaine touche)
    vx: f32,
    vy: f32,
    z: f32,
    dims: (f32, f32),
    area: Rect,
    view: Option<Pixmap>,
    view_dirty: bool,
    seen: u64,
    last_change: Instant,
    touch: Option<Touch>,
    long_fired: bool,
    wheel_acc: f32,
    pressed: Option<usize>,
}

impl Rdp {
    fn new() -> Rdp {
        let store = Store::open("state.conf");
        let keep_pass = store.get_bool("keep_pass").unwrap_or(false);
        let f = [
            store.get_or("host", ""),
            store.get_or("user", ""),
            if keep_pass { store.get_or("pass", "") } else { String::new() },
            store.get_or("domain", ""),
        ];
        Rdp {
            res_idx: (store.get_i64("res").unwrap_or(0).max(0) as usize).min(RES.len() - 1),
            azerty: store.get_bool("azerty").unwrap_or(true),
            nla: store.get_bool("nla").unwrap_or(true),
            keep_pass,
            store,
            screen: Screen::Form,
            f,
            focus: None,
            notice: String::new(),
            scroll: Scroller::new(),
            head_h: 44.0,
            focus_dirty: false,
            last_kbd: 0.0,
            sess: None,
            conn_key: String::new(),
            fp_saved: false,
            zoom_idx: 0,
            mode: 0,
            right_once: false,
            show_keys: false,
            mods: [false; 4],
            vx: 0.0,
            vy: 0.0,
            z: 1.0,
            dims: (1.0, 1.0),
            area: Rect::new(0.0, 0.0, 1.0, 1.0),
            view: None,
            view_dirty: true,
            seen: 0,
            last_change: Instant::now(),
            touch: None,
            long_fired: false,
            wheel_acc: 0.0,
            pressed: None,
        }
    }

    // ------------------------------------------------------------------------------------------------
    // Formulaire
    // ------------------------------------------------------------------------------------------------

    fn form_layout(&self, ctx: &Ctx) -> FormLayout {
        let s = ctx.scale();
        let w = ctx.width();
        let pad = 12.0 * s;
        let mut y = self.head_h - self.scroll.pos + 8.0 * s;
        let top = y;
        let mut fields = [Rect::new(0.0, 0.0, 0.0, 0.0); 4];
        for fl in fields.iter_mut() {
            *fl = Rect::new(pad, y + 17.0 * s, w - 2.0 * pad, 34.0 * s);
            y += 56.0 * s;
        }
        y += 2.0 * s;
        let st = Text::new(13.0 * s);
        let notice = if self.notice.is_empty() { Vec::new() } else { wrap(ctx, &self.notice, &st, w - 2.0 * pad) };
        let notice_y = y;
        if !notice.is_empty() {
            y += notice.len().min(6) as f32 * 17.0 * s + 8.0 * s;
        }
        let connect = Rect::new(pad, y, w - 2.0 * pad, 44.0 * s);
        y += 44.0 * s + 12.0 * s;
        let row = |y: &mut f32| {
            let r = Rect::new(0.0, *y, w, 40.0 * s);
            *y += 40.0 * s;
            r
        };
        let res = row(&mut y);
        let kbd = row(&mut y);
        let nla = row(&mut y);
        let keep = row(&mut y);
        y += 8.0 * s;
        FormLayout { fields, res, kbd, nla, keep, notice_y, notice, connect, content_h: y - top + 8.0 * s }
    }

    fn render_form(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = ctx.theme;
        let s = ctx.scale();
        let h = ctx.height();
        g.clear(t.bg);
        let kbd_h = ctx.keyboard_height();
        if (kbd_h - self.last_kbd).abs() > 0.5 {
            self.last_kbd = kbd_h;
            self.focus_dirty = true;
        }
        let view_h = (h - self.head_h - kbd_h).max(60.0 * s);
        let l = self.form_layout(ctx);
        self.scroll.set_content(l.content_h, view_h);
        if self.focus_dirty {
            self.focus_dirty = false;
            if let Some(i) = self.focus {
                let rel_top = l.fields[i].y - 17.0 * s - self.head_h + self.scroll.pos;
                let rel_bot = l.fields[i].bottom() - self.head_h + self.scroll.pos;
                if rel_top < self.scroll.pos {
                    self.scroll.pos = (rel_top - 6.0 * s).max(0.0);
                } else if rel_bot > self.scroll.pos + view_h {
                    self.scroll.pos = (rel_bot - view_h + 8.0 * s).min(self.scroll.max);
                }
            }
        }
        let l = self.form_layout(ctx);

        // champs
        let lab = Text::new(11.5 * s).color(t.subtext).left();
        for i in 0..4 {
            let r = l.fields[i];
            ctx.text_mid(g, LABELS[i], r.x + 2.0 * s, r.y - 8.0 * s, &lab);
            let focused = self.focus == Some(i);
            g.rrect(r.x, r.y, r.w, r.h, 9.0 * s, if focused { t.accent } else { t.line });
            g.rrect(r.x + 1.5, r.y + 1.5, r.w - 3.0, r.h - 3.0, 8.0 * s, t.surface);
            let max_w = r.w - 20.0 * s;
            if self.f[i].is_empty() {
                let ph = ["192.168.1.20", "Administrateur", "", ""][i];
                let st = Text::new(15.0 * s).color(rgba(0x000000, 0.28)).left().max_width(max_w);
                ctx.text_mid(g, ph, r.x + 10.0 * s, r.y + r.h / 2.0, &st);
            }
            let shown = if i == 2 { "•".repeat(self.f[i].chars().count()) } else { self.f[i].clone() };
            let st = Text::new(15.0 * s).color(t.text).left().ellipsis_left().max_width(max_w);
            if !shown.is_empty() {
                ctx.text_mid(g, &shown, r.x + 10.0 * s, r.y + r.h / 2.0, &st);
            }
            if focused {
                let tw = ctx.measure(&shown, &st).min(max_w);
                g.rect(r.x + 10.0 * s + tw + 1.0, r.y + 8.0 * s, 1.5, r.h - 16.0 * s, t.accent);
            }
        }

        // options
        let (rw, rh) = RES[self.res_idx];
        ui::list_row(ctx, g, l.res, "Résolution", Some(&format!("{rw}×{rh}")), false);
        ui::list_row(ctx, g, l.kbd, "Clavier du PC distant", Some(if self.azerty { "AZERTY" } else { "QWERTY" }), false);
        for (r, label, on) in [(l.nla, "Exiger NLA (sécurisé)", self.nla), (l.keep, "Retenir le mot de passe", self.keep_pass)] {
            ui::list_row(ctx, g, r, label, None, false);
            ui::toggle(ctx, g, Rect::new(r.right() - 12.0 * s - 46.0 * s, r.y + (r.h - 26.0 * s) / 2.0, 46.0 * s, 26.0 * s), on);
        }

        // message
        if !l.notice.is_empty() {
            let st = Text::new(13.0 * s).color(t.danger).left();
            for (i, line) in l.notice.iter().take(6).enumerate() {
                ctx.text(g, line, 12.0 * s, l.notice_y + i as f32 * 17.0 * s, &st);
            }
        }
        ui::button(ctx, g, l.connect, "Se connecter", ButtonStyle::Primary, false);

        let head = ui::header(ctx, g, "Bureau à distance", None);
        self.head_h = head.height;
    }

    fn form_event(&mut self, ctx: &Ctx, ev: Event) {
        match ev {
            Event::Down { .. } => self.scroll.stop(),
            Event::Drag { dy, .. } | Event::Wheel { dy } => self.scroll.drag(dy),
            Event::DragEnd { vy, .. } => self.scroll.fling(vy),
            Event::Tap { x, y } => {
                let l = self.form_layout(ctx);
                if y < self.head_h {
                    return;
                }
                for i in 0..4 {
                    if l.fields[i].inset(-6.0).contains(x, y) {
                        self.focus = Some(i);
                        self.focus_dirty = true;
                        ctx.keyboard(1);
                        return;
                    }
                }
                if l.res.contains(x, y) {
                    self.res_idx = (self.res_idx + 1) % RES.len();
                } else if l.kbd.contains(x, y) {
                    self.azerty = !self.azerty;
                } else if l.nla.contains(x, y) {
                    self.nla = !self.nla;
                } else if l.keep.contains(x, y) {
                    self.keep_pass = !self.keep_pass;
                } else if l.connect.contains(x, y) {
                    self.focus = None;
                    ctx.keyboard(0);
                    self.connect(ctx, None);
                    return;
                } else {
                    self.focus = None;
                    ctx.keyboard(0);
                }
                self.save_options();
            }
            Event::Key(k) => {
                let Some(i) = self.focus else { return };
                match k {
                    Key::Char(c) if !c.is_control() => self.f[i].push(c),
                    Key::Space => self.f[i].push(' '),
                    Key::Back => {
                        self.f[i].pop();
                    }
                    Key::Enter => {
                        if i < 2 {
                            self.focus = Some(i + 1);
                            self.focus_dirty = true;
                        } else {
                            self.focus = None;
                            ctx.keyboard(0);
                            self.connect(ctx, None);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn save_options(&mut self) {
        let _ = self.store.set("res", self.res_idx);
        let _ = self.store.set("azerty", self.azerty);
        let _ = self.store.set("nla", self.nla);
        let _ = self.store.set("keep_pass", self.keep_pass);
        if !self.keep_pass {
            let _ = self.store.remove("pass");
        }
    }

    /// Lance la connexion. `trust_fp` : nouvelle empreinte à mémoriser avant de se connecter (certificat changé accepté).
    fn connect(&mut self, ctx: &Ctx, trust_fp: Option<String>) {
        let raw = self.f[0].trim().to_string();
        let mut user = self.f[1].trim().to_string();
        let mut domain = self.f[3].trim().to_string();
        if raw.is_empty() {
            self.notice = "Saisissez l'adresse du PC.".into();
            return;
        }
        if user.is_empty() {
            self.notice = "Saisissez le nom d'utilisateur.".into();
            return;
        }
        if domain.is_empty() {
            if let Some((d, u)) = user.split_once('\\') {
                domain = d.to_string();
                user = u.to_string();
            }
        }
        let (host, port) = split_host(&raw);
        let _ = self.store.set("host", &raw);
        let _ = self.store.set("user", &self.f[1]);
        let _ = self.store.set("domain", &self.f[3]);
        if self.keep_pass {
            let _ = self.store.set("pass", &self.f[2]);
        }
        self.save_options();
        self.conn_key = format!("fp:{}:{}", host.to_lowercase(), port);
        if let Some(fp) = trust_fp {
            let _ = self.store.set(&self.conn_key, fp);
        }
        let known_fp = self.store.get(&self.conn_key).map(|s| s.to_string());
        let (width, height) = RES[self.res_idx];
        self.sess = Some(Session::start(Params {
            host,
            port,
            user,
            pass: self.f[2].clone(),
            domain,
            width,
            height,
            azerty: self.azerty,
            nla_only: self.nla,
            known_fp,
        }));
        self.fp_saved = false;
        self.notice.clear();
        self.enter_session(ctx);
    }

    fn enter_session(&mut self, ctx: &Ctx) {
        self.screen = Screen::Session;
        self.zoom_idx = 0;
        self.mode = 0;
        self.right_once = false;
        self.mods = [false; 4];
        self.touch = None;
        self.seen = 0;
        self.view_dirty = true;
        self.view = None;
        self.long_fired = false;
        ctx.keyboard(0);
    }

    fn leave_session(&mut self, ctx: &Ctx) {
        self.sess = None; // Drop : déconnexion propre
        self.screen = Screen::Form;
        self.show_keys = false;
        ctx.keyboard(0);
    }

    // ------------------------------------------------------------------------------------------------
    // Session
    // ------------------------------------------------------------------------------------------------

    fn tb_h(&self, ctx: &Ctx) -> f32 {
        34.0 * ctx.scale()
    }

    fn keys_h(&self, ctx: &Ctx) -> f32 {
        if self.show_keys {
            62.0 * ctx.scale()
        } else {
            0.0
        }
    }

    fn cur_zoom(&self) -> f32 {
        let z = ZOOMS[self.zoom_idx].1;
        if z <= 0.0 {
            (self.area.w / self.dims.0).min(self.area.h / self.dims.1).max(0.05)
        } else {
            z
        }
    }

    fn clamp_view(&mut self) {
        let (dw, dh) = self.dims;
        let (vw, vh) = (self.area.w / self.z, self.area.h / self.z);
        self.vx = if dw <= vw { -(vw - dw) / 2.0 } else { self.vx.clamp(0.0, dw - vw) };
        self.vy = if dh <= vh { -(vh - dh) / 2.0 } else { self.vy.clamp(0.0, dh - vh) };
    }

    fn to_desktop(&self, x: f32, y: f32) -> (u16, u16) {
        let dx = self.vx + (x - self.area.x) / self.z;
        let dy = self.vy + (y - self.area.y) / self.z;
        (dx.clamp(0.0, self.dims.0 - 1.0) as u16, dy.clamp(0.0, self.dims.1 - 1.0) as u16)
    }

    fn send(&self, ops: Vec<Operation>) {
        if let Some(s) = &self.sess {
            s.send(ops);
        }
    }

    fn click_at(&mut self, x: f32, y: f32, right: bool) {
        let (dx, dy) = self.to_desktop(x, y);
        let b = if right { MouseButton::Right } else { MouseButton::Left };
        self.send(vec![
            Operation::MouseMove(MousePosition { x: dx, y: dy }),
            Operation::MouseButtonPressed(b),
            Operation::MouseButtonReleased(b),
        ]);
    }

    fn active_mods(&self) -> Vec<u16> {
        let codes = [SC_CTRL, SC_ALT, SC_SHIFT, SC_WIN];
        (0..4).filter(|&i| self.mods[i]).map(|i| codes[i]).collect()
    }

    /// Appuie / relâche une touche par son code de balayage, entourée des modificateurs collés.
    fn tap_scancode(&mut self, code: u16) {
        let mods = self.active_mods();
        let mut ops = Vec::new();
        for m in &mods {
            ops.push(Operation::KeyPressed(sc(*m)));
        }
        ops.push(Operation::KeyPressed(sc(code)));
        ops.push(Operation::KeyReleased(sc(code)));
        for m in mods.iter().rev() {
            ops.push(Operation::KeyReleased(sc(*m)));
        }
        self.mods = [false; 4];
        self.send(ops);
    }

    fn session_key(&mut self, k: Key) {
        match k {
            Key::Enter => self.tap_scancode(SC_ENTER),
            Key::Back => self.tap_scancode(SC_BACKSPACE),
            Key::Up => self.tap_scancode(SC_UP),
            Key::Down => self.tap_scancode(SC_DOWN),
            Key::Left => self.tap_scancode(SC_LEFT),
            Key::Right => self.tap_scancode(SC_RIGHT),
            Key::Space => {
                if self.mods.iter().any(|m| *m) {
                    self.tap_scancode(SC_SPACE)
                } else {
                    self.send(vec![Operation::UnicodeKeyPressed(' '), Operation::UnicodeKeyReleased(' ')]);
                }
            }
            Key::Char('\t') => self.tap_scancode(SC_TAB),
            Key::Char(c) if !c.is_control() => {
                let shortcut = self.mods[0] || self.mods[1] || self.mods[3];
                if shortcut {
                    if let Some(code) = char_scancode(c, self.azerty) {
                        return self.tap_scancode(code);
                    }
                }
                let c = if self.mods[2] { c.to_uppercase().next().unwrap_or(c) } else { c };
                self.mods = [false; 4];
                self.send(vec![Operation::UnicodeKeyPressed(c), Operation::UnicodeKeyReleased(c)]);
            }
            _ => {}
        }
    }

    fn toolbar_tap(&mut self, ctx: &Ctx, i: usize) {
        match i {
            0 => ctx.keyboard(if ctx.keyboard_height() > 0.0 { 0 } else { 1 }),
            1 => {
                self.show_keys = !self.show_keys;
                self.view_dirty = true;
            }
            2 => {
                self.mode = (self.mode + 1) % MODES.len();
                ctx.toast(match self.mode {
                    0 => "Pan : glisser déplace la vue",
                    1 => "Souris : glisser = cliquer-glisser",
                    _ => "Défil. : glisser fait défiler",
                });
            }
            3 => {
                let (cx, cy) = (self.vx + self.area.w / self.z / 2.0, self.vy + self.area.h / self.z / 2.0);
                self.zoom_idx = (self.zoom_idx + 1) % ZOOMS.len();
                self.z = self.cur_zoom();
                self.vx = cx - self.area.w / self.z / 2.0;
                self.vy = cy - self.area.h / self.z / 2.0;
                self.clamp_view();
                self.view_dirty = true;
            }
            4 => {
                self.right_once = !self.right_once;
                if self.right_once {
                    ctx.toast("Prochain toucher : clic droit");
                }
            }
            _ => self.leave_session(ctx),
        }
    }

    fn keys_tap(&mut self, row: usize, col: usize) {
        match (row, col) {
            (0, c @ 0..=3) => self.mods[c] = !self.mods[c],
            (0, 4) => self.tap_scancode(SC_ESC),
            (0, 5) => self.tap_scancode(SC_TAB),
            (1, 0) => self.tap_scancode(SC_DEL),
            (1, 1) => {
                // Ctrl+Alt+Suppr est intercepté par le client ; avec RDP, Ctrl+Alt+Fin joue le même rôle.
                self.mods[0] = true;
                self.mods[1] = true;
                self.tap_scancode(SC_END);
            }
            (1, 2) => self.tap_scancode(SC_LEFT),
            (1, 3) => self.tap_scancode(SC_UP),
            (1, 4) => self.tap_scancode(SC_DOWN),
            (1, 5) => self.tap_scancode(SC_RIGHT),
            _ => {}
        }
    }

    fn session_event(&mut self, ctx: &Ctx, ev: Event) {
        let s = ctx.scale();
        let w = ctx.width();
        let state = self.sess.as_ref().map(|x| x.state());
        let connected = matches!(state, Some(State::Connected));
        let tb_h = self.tb_h(ctx);
        match ev {
            Event::Down { x, y } => {
                self.long_fired = false;
                self.pressed = None;
                if y < tb_h {
                    self.pressed = Some(((x / (w / 6.0)) as usize).min(5));
                } else if connected && self.area.contains(x, y) {
                    self.touch = Some(Touch { x0: x, y0: y, t0: Instant::now(), moved: false, dragging: false });
                }
            }
            Event::Move { x, y } => {
                if let Some(t) = &mut self.touch {
                    if (x - t.x0).abs() > 9.0 * s || (y - t.y0).abs() > 9.0 * s {
                        t.moved = true;
                    }
                }
            }
            Event::Drag { x, y, dx, dy } => {
                if !connected || self.long_fired {
                    return;
                }
                let mode = self.mode;
                let (x0, y0, first) = match &mut self.touch {
                    Some(t) => {
                        t.moved = true;
                        let first = !t.dragging;
                        if mode == 1 {
                            t.dragging = true;
                        }
                        (t.x0, t.y0, first)
                    }
                    None => return,
                };
                match mode {
                    0 => {
                        self.vx -= dx / self.z;
                        self.vy -= dy / self.z;
                        self.clamp_view();
                        self.view_dirty = true;
                    }
                    1 => {
                        let mut ops = Vec::new();
                        if first {
                            let (px, py) = self.to_desktop(x0, y0);
                            ops.push(Operation::MouseMove(MousePosition { x: px, y: py }));
                            ops.push(Operation::MouseButtonPressed(MouseButton::Left));
                        }
                        let (px, py) = self.to_desktop(x, y);
                        ops.push(Operation::MouseMove(MousePosition { x: px, y: py }));
                        self.send(ops);
                    }
                    _ => self.wheel(dy),
                }
            }
            Event::Up { x, y } => {
                self.pressed = None;
                if let Some(t) = &self.touch {
                    if t.dragging {
                        let (px, py) = self.to_desktop(x, y);
                        self.send(vec![
                            Operation::MouseMove(MousePosition { x: px, y: py }),
                            Operation::MouseButtonReleased(MouseButton::Left),
                        ]);
                    }
                }
                self.touch = None;
            }
            Event::Wheel { dy } => {
                if connected {
                    self.wheel(dy);
                }
            }
            Event::Tap { x, y } => {
                self.pressed = None;
                if self.long_fired {
                    self.long_fired = false;
                    return;
                }
                if y < tb_h {
                    self.toolbar_tap(ctx, ((x / (w / 6.0)) as usize).min(5));
                } else if self.show_keys && y < tb_h + self.keys_h(ctx) {
                    let row = (((y - tb_h) / (self.keys_h(ctx) / 2.0)) as usize).min(1);
                    let col = ((x / (w / 6.0)) as usize).min(5);
                    if connected {
                        self.keys_tap(row, col);
                    }
                } else if connected && self.area.contains(x, y) {
                    let right = self.right_once;
                    self.right_once = false;
                    self.click_at(x, y, right);
                } else if let Some(st) = state {
                    self.overlay_tap(ctx, st, x, y);
                }
            }
            Event::Key(k) => {
                if connected {
                    self.session_key(k);
                }
            }
            _ => {}
        }
    }

    fn wheel(&mut self, dy: f32) {
        self.wheel_acc += dy;
        let step = 22.0;
        let notches = (self.wheel_acc / step).trunc();
        if notches != 0.0 {
            self.wheel_acc -= notches * step;
            let units = (notches as i32 * 120).clamp(-1200, 1200) as i16;
            self.send(vec![Operation::WheelRotations(WheelRotations { is_vertical: true, rotation_units: units })]);
        }
    }

    fn overlay_buttons(&self, ctx: &Ctx, st: &State) -> Vec<(Rect, &'static str, u8)> {
        let s = ctx.scale();
        let a = self.area;
        let mut labels: Vec<(&'static str, u8)> = Vec::new();
        match st {
            State::Failed { new_fp: Some(_), .. } => labels.push(("Faire confiance et reconnecter", 2)),
            State::Failed { .. } | State::Ended(_) => labels.push(("Reconnecter", 1)),
            _ => {}
        }
        labels.push(("Retour", 0));
        let n = labels.len() as f32;
        let bh = 42.0 * s;
        let y0 = a.bottom() - 16.0 * s - n * (bh + 8.0 * s);
        labels
            .into_iter()
            .enumerate()
            .map(|(i, (l, id))| (Rect::new(a.x + 18.0 * s, y0 + i as f32 * (bh + 8.0 * s), a.w - 36.0 * s, bh), l, id))
            .collect()
    }

    fn overlay_tap(&mut self, ctx: &Ctx, st: State, x: f32, y: f32) {
        for (r, _, id) in self.overlay_buttons(ctx, &st) {
            if r.contains(x, y) {
                match id {
                    0 => self.leave_session(ctx),
                    1 => {
                        self.sess = None;
                        self.connect(ctx, None);
                    }
                    _ => {
                        let fp = if let State::Failed { new_fp: Some(fp), .. } = &st { Some(fp.clone()) } else { None };
                        self.sess = None;
                        self.connect(ctx, fp);
                    }
                }
                return;
            }
        }
    }

    fn render_session(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        g.clear(rgb(0x16161a));
        let tb_h = self.tb_h(ctx);
        let keys_h = self.keys_h(ctx);
        let kbd = ctx.keyboard_height();
        self.area = Rect::new(0.0, tb_h + keys_h, w, (h - tb_h - keys_h - kbd).max(40.0 * s));
        let area = self.area;

        let (state, frame) = match &self.sess {
            Some(x) => (x.state(), x.frame()),
            None => (State::Ended(String::new()), 0),
        };

        // mémorise l'empreinte du certificat à la première connexion réussie
        if matches!(state, State::Connected) && !self.fp_saved {
            self.fp_saved = true;
            let fp = self.sess.as_ref().and_then(|x| lock(&x.shared).fp.clone());
            if let Some(fp) = fp {
                if self.store.get(&self.conn_key).is_none() {
                    let _ = self.store.set(&self.conn_key, fp);
                    ctx.toast("Certificat du serveur mémorisé");
                }
            }
        }

        let mut drawn = false;
        if matches!(state, State::Connected) {
            let dims = self.sess.as_ref().and_then(|x| x.with_frame(|_, w, h, _| (w as f32, h as f32)));
            if let Some((dw, dh)) = dims {
                if (dw, dh) != self.dims {
                    self.dims = (dw, dh);
                    self.view_dirty = true;
                    self.z = self.cur_zoom();
                    self.vx = 0.0;
                    self.vy = 0.0;
                }
                let z = self.cur_zoom();
                if (z - self.z).abs() > 1e-4 {
                    self.z = z;
                    self.view_dirty = true;
                }
                self.clamp_view();
                if frame != self.seen {
                    self.seen = frame;
                    self.view_dirty = true;
                    self.last_change = Instant::now();
                }
                let (aw, ah) = (area.w.round() as u32, area.h.round() as u32);
                let stale = self.view.as_ref().map_or(true, |p| p.width() != aw || p.height() != ah);
                if stale {
                    self.view = tiny_skia::Pixmap::new(aw.max(1), ah.max(1));
                    self.view_dirty = true;
                }
                if self.view_dirty {
                    if let (Some(pm), Some(sess)) = (self.view.as_mut(), self.sess.as_ref()) {
                        pm.fill(tiny_skia::Color::from_rgba8(0x10, 0x10, 0x14, 255));
                        let (vx, vy, z) = (self.vx, self.vy, self.z);
                        sess.with_frame(|px, sw, sh, stride| compose(px, sw, sh, stride, pm, vx, vy, z));
                    }
                    if let Some(pm) = &self.view {
                        g.replace_image("rdp:view", pm, area.x, area.y, 1.0);
                        drawn = true;
                    }
                    self.view_dirty = false;
                } else if self.view.is_some() {
                    g.blit("rdp:view", area.x, area.y, 1.0);
                    drawn = true;
                }
            }
        }
        if !drawn {
            self.overlay(ctx, g, &state);
        }

        // barres d'outils
        g.rect(0.0, 0.0, w, tb_h + keys_h, rgb(0x232328));
        let labels = [
            "Clavier".to_string(),
            "Touches".to_string(),
            MODES[self.mode].to_string(),
            ZOOMS[self.zoom_idx].0.to_string(),
            "Clic D".to_string(),
            "Fin".to_string(),
        ];
        let active = [kbd > 0.0, self.show_keys, false, false, self.right_once, false];
        let bw = w / 6.0;
        for i in 0..6 {
            let r = Rect::new(i as f32 * bw, 0.0, bw, tb_h);
            btn(ctx, g, r, &labels[i], active[i], self.pressed == Some(i), i == 5);
        }
        if self.show_keys {
            let rh = keys_h / 2.0;
            for row in 0..2 {
                for col in 0..6 {
                    let r = Rect::new(col as f32 * bw, tb_h + row as f32 * rh, bw, rh);
                    let on = row == 0 && col < 4 && self.mods[col];
                    btn(ctx, g, r, KEYROWS[row][col], on, false, false);
                }
            }
        }

        // cadence : plus rapide juste après une mise à jour de l'image ou pendant un toucher
        let busy = self.last_change.elapsed() < Duration::from_millis(400) || self.touch.is_some();
        let connecting = !matches!(state, State::Connected);
        ctx.redraw_in(Duration::from_millis(if busy { 25 } else if connecting { 120 } else { 70 }));
    }

    fn overlay(&mut self, ctx: &Ctx, g: &mut Gfx, state: &State) {
        let s = ctx.scale();
        let a = self.area;
        let (title, body, color) = match state {
            State::Connecting => ("Connexion…".to_string(), self.f[0].trim().to_string(), rgb(0xffffff)),
            State::Connected => ("Chargement de l'image…".to_string(), String::new(), rgb(0xffffff)),
            State::Ended(m) => ("Session terminée".to_string(), m.clone(), rgb(0xffffff)),
            State::Failed { msg, .. } => ("Connexion impossible".to_string(), msg.clone(), rgb(0xff8a80)),
        };
        let tst = Text::new(18.0 * s).bold().color(color).center();
        ctx.text_mid(g, &title, a.x + a.w / 2.0, a.y + 40.0 * s, &tst);
        let bst = Text::new(13.0 * s).color(rgb(0xd0d0d8)).center();
        for (i, line) in wrap(ctx, &body, &bst, a.w - 32.0 * s).iter().take(11).enumerate() {
            ctx.text_mid(g, line, a.x + a.w / 2.0, a.y + 72.0 * s + i as f32 * 17.0 * s, &bst);
        }
        for (r, label, id) in self.overlay_buttons(ctx, state) {
            let style = if id == 0 { ButtonStyle::Secondary } else { ButtonStyle::Primary };
            ui::button(ctx, g, r, label, style, false);
        }
    }
}

fn btn(ctx: &Ctx, g: &mut Gfx, r: Rect, label: &str, active: bool, pressed: bool, danger: bool) {
    let s = ctx.scale();
    let bg = if active {
        ctx.theme.accent
    } else if pressed {
        rgb(0x505058)
    } else if danger {
        rgb(0x5a2a2e)
    } else {
        rgb(0x35353c)
    };
    g.rrect(r.x + 2.0 * s, r.y + 3.0 * s, r.w - 4.0 * s, r.h - 6.0 * s, 6.0 * s, bg);
    let st = Text::new(11.5 * s).bold().color(rgb(0xffffff)).center().max_width(r.w - 6.0 * s);
    let (cx, cy) = r.center();
    ctx.text_mid(g, label, cx, cy, &st);
}

/// Coupe `text` en lignes de largeur ≤ `max_w` (retours à la ligne aux espaces ; un mot trop long est coupé).
fn wrap(ctx: &Ctx, text: &str, st: &Text, max_w: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        let mut word = word.to_string();
        loop {
            let cand = if cur.is_empty() { word.clone() } else { format!("{cur} {word}") };
            if ctx.measure(&cand, st) <= max_w {
                cur = cand;
                break;
            }
            if !cur.is_empty() {
                lines.push(std::mem::take(&mut cur));
                continue;
            }
            // mot seul trop long : on le coupe
            let mut cut = String::new();
            let mut rest = String::new();
            for (i, ch) in word.chars().enumerate() {
                if rest.is_empty() && ctx.measure(&format!("{cut}{ch}"), st) <= max_w {
                    cut.push(ch);
                } else {
                    if rest.is_empty() && i == 0 {
                        cut.push(ch);
                        continue;
                    }
                    rest.push(ch);
                }
            }
            lines.push(cut);
            if rest.is_empty() {
                break;
            }
            word = rest;
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

fn split_host(raw: &str) -> (String, u16) {
    if let Some(rest) = raw.strip_prefix('[') {
        if let Some((h, tail)) = rest.split_once(']') {
            let port = tail.strip_prefix(':').and_then(|p| p.parse().ok()).unwrap_or(3389);
            return (h.to_string(), port);
        }
    }
    if raw.matches(':').count() == 1 {
        if let Some((h, p)) = raw.split_once(':') {
            if let Ok(p) = p.trim().parse() {
                return (h.trim().to_string(), p);
            }
        }
    }
    (raw.to_string(), 3389)
}

/// Dessine la zone visible du bureau (origine `vx`,`vy` en pixels du bureau, zoom `z`) dans `out`.
/// Réduction : moyenne de n×n échantillons par pixel (texte lisible) ; agrandissement : plus proche voisin.
fn compose(src: &[u8], sw: usize, sh: usize, stride: usize, out: &mut tiny_skia::Pixmap, vx: f32, vy: f32, z: f32) {
    let ow = out.width() as usize;
    let oh = out.height() as usize;
    let step = 1.0 / z;
    let n: usize = if step > 1.0 { (step.ceil() as usize).clamp(2, 4) } else { 1 };
    let mut xs: Vec<i32> = Vec::with_capacity(ow * n);
    for px in 0..ow {
        for j in 0..n {
            xs.push((vx + (px as f32 + (j as f32 + 0.5) / n as f32) * step).floor() as i32);
        }
    }
    let o = out.data_mut();
    for py in 0..oh {
        let mut ys = [0i32; 4];
        for (i, y) in ys.iter_mut().enumerate().take(n) {
            *y = (vy + (py as f32 + (i as f32 + 0.5) / n as f32) * step).floor() as i32;
        }
        if ys[..n].iter().all(|&y| y < 0 || y as usize >= sh) {
            continue;
        }
        let row = &mut o[py * ow * 4..(py + 1) * ow * 4];
        for px in 0..ow {
            let (mut r, mut gg, mut b, mut c) = (0u32, 0u32, 0u32, 0u32);
            for &y in &ys[..n] {
                if y < 0 || y as usize >= sh {
                    continue;
                }
                let base = y as usize * stride;
                for &x in &xs[px * n..px * n + n] {
                    if x < 0 || x as usize >= sw {
                        continue;
                    }
                    let p = base + x as usize * 4;
                    r += src[p] as u32;
                    gg += src[p + 1] as u32;
                    b += src[p + 2] as u32;
                    c += 1;
                }
            }
            if c > 0 {
                let d = &mut row[px * 4..px * 4 + 4];
                d[0] = (r / c) as u8;
                d[1] = (gg / c) as u8;
                d[2] = (b / c) as u8;
                d[3] = 255;
            }
        }
    }
}

impl App for Rdp {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        match self.screen {
            Screen::Form => self.render_form(ctx, g),
            Screen::Session => self.render_session(ctx, g),
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        match self.screen {
            Screen::Form => self.form_event(ctx, ev),
            Screen::Session => self.session_event(ctx, ev),
        }
    }

    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        match self.screen {
            Screen::Form => {
                if self.scroll.update(dt) {
                    ctx.animate();
                }
            }
            Screen::Session => {
                // appui long immobile = clic droit
                let fire = match &self.touch {
                    Some(t) => !t.moved && !t.dragging && !self.long_fired && t.t0.elapsed().as_secs_f32() > LONG_PRESS,
                    None => false,
                };
                if fire {
                    if let Some(t) = &self.touch {
                        let (x, y) = (t.x0, t.y0);
                        self.long_fired = true;
                        self.click_at(x, y, true);
                        ctx.toast("Clic droit");
                    }
                }
            }
        }
    }

    fn on_back(&mut self, ctx: &Ctx) -> bool {
        if self.screen == Screen::Session {
            if self.show_keys {
                self.show_keys = false;
                self.view_dirty = true;
            } else if matches!(self.sess.as_ref().map(|s| s.state()), Some(State::Connected)) {
                self.tap_scancode(SC_ESC);
            } else {
                self.leave_session(ctx);
            }
            return true;
        }
        false
    }

    fn icon(&mut self, ctx: &Ctx, g: &mut Gfx, size: u32) {
        let f = size as f32;
        g.rrect(0.0, 0.0, f, f, f * 0.22, rgb(0x0f6cbd));
        // écran
        g.rrect(f * 0.16, f * 0.22, f * 0.68, f * 0.46, f * 0.05, rgb(0xffffff));
        g.rrect(f * 0.20, f * 0.26, f * 0.60, f * 0.38, f * 0.03, rgb(0x2b88d8));
        // pied
        g.rect(f * 0.45, f * 0.68, f * 0.10, f * 0.08, rgb(0xffffff));
        g.rrect(f * 0.33, f * 0.76, f * 0.34, f * 0.05, f * 0.02, rgb(0xffffff));
        // flèche de curseur
        let st = Text::new(f * 0.30).bold().color(rgb(0xffffff)).center();
        ctx.text_mid(g, "»", f * 0.5, f * 0.45, &st);
    }
}

fn main() {
    let mut app = Rdp::new();
    if arg_value("--page").as_deref() == Some("session") {
        app.sess = Some(Session::demo(1024, 768));
        app.screen = Screen::Session;
        if let Some(z) = arg_value("--zoom").and_then(|z| z.parse::<usize>().ok()) {
            app.zoom_idx = z.min(ZOOMS.len() - 1);
        }
        app.show_keys = lapp_sdk::arg_flag("--keys");
    }
    run(Config::new("RDP"), app);
}