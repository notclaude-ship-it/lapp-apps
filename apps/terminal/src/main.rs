// Terminal : un « Termux » pour lapp — un shell bash dans le bac à sable, avec émulateur de terminal
// (couleurs, écran alternatif pour vim/less/htop…), historique défilable au doigt, rangée de touches
// supplémentaires (ESC, TAB, CTRL, flèches…) au-dessus du clavier virtuel de lapp-home.
//
// Quitter : glisser vers le haut depuis le bord bas de l'écran (ou `exit` puis toucher l'écran).
// Sur PC : cargo run -- --screenshot out.png [--type 'ls --color\n'] [--size 272x480]
//          LAPP_DATA=/tmp/x cargo run          (fenêtre sur le bureau, vrai clavier)

mod pty;
mod term;

use std::time::{Duration, Instant};

use lapp_sdk::prelude::*;
use lapp_sdk::{arg_flag, arg_value, data_dir};

use pty::Pty;
use term::*;

const SIZES: [f32; 7] = [8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 16.0];
const DEFAULT_SIZE: usize = 3;

const BG: u32 = 0x0b0d10;
const FG: u32 = 0xd6d9de;
const CURSOR: u32 = 0xd6d9de;
const PANEL: u32 = 0x15181c;
const KEY: u32 = 0x2a2e35;
const KEY_ON: u32 = 0x2f6fed;

const DEFAULT_RC: &str = r#"# ~/.bashrc : Terminal (lapp). Ce fichier est à vous, modifiez-le librement.
export PATH="$HOME/bin:$PATH"
HISTFILE="$HOME/.bash_history"
HISTSIZE=5000
HISTCONTROL=ignoredups
PS1='\[\e[1;32m\]lapp\[\e[0m\]:\[\e[1;34m\]\w\[\e[0m\]\$ '
alias ls='ls --color=auto'
alias ll='ls -lah'
alias la='ls -A'
alias grep='grep --color=auto'
alias ..='cd ..'
"#;

const WELCOME: &str = "\x1b[1mTerminal\x1b[0m (lapp) : bash en bac à sable\r\n\x1b[90mDonnées : /data (persistantes)\r\nQuitter : glisser du bas vers le haut\x1b[0m\r\n\r\n";

#[derive(Clone, Copy, PartialEq, Debug)]
enum XKey {
    Esc,
    Tab,
    Ctrl,
    Alt,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PgUp,
    PgDn,
    Ch(char),
    Page,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Action {
    FontDown,
    FontUp,
    Kbd,
    Key(XKey),
}

fn xkeys(page2: bool) -> [(XKey, &'static str); 8] {
    if page2 {
        [
            (XKey::Home, "HOME"),
            (XKey::End, "END"),
            (XKey::PgUp, "PGUP"),
            (XKey::PgDn, "PGDN"),
            (XKey::Alt, "ALT"),
            (XKey::Ch('|'), "|"),
            (XKey::Ch('~'), "~"),
            (XKey::Page, "«"),
        ]
    } else {
        [
            (XKey::Esc, "ESC"),
            (XKey::Tab, "TAB"),
            (XKey::Ctrl, "CTRL"),
            (XKey::Left, "←"),
            (XKey::Down, "↓"),
            (XKey::Up, "↑"),
            (XKey::Right, "→"),
            (XKey::Page, "»"),
        ]
    }
}

#[derive(Clone, Copy, Default)]
struct Layout {
    s: f32,
    w: f32,
    h: f32,
    px: f32,
    cw: f32,
    lh: f32,
    bar_h: f32,
    x0: f32,
    y0: f32,
    cols: usize,
    rows: usize,
    extra: Rect,
    kbd_h: f32,
}

// ------------------------------------------------------------------ couleurs

fn ansi(i: u8) -> Color {
    const P: [u32; 16] = [
        0x000000, 0xcd3131, 0x0dbc79, 0xe5e510, 0x2472c8, 0xbc3fbc, 0x11a8cd, 0xe5e5e5, 0x666666, 0xf14c4c, 0x23d18b, 0xf5f543,
        0x3b8eea, 0xd670d6, 0x29b8db, 0xffffff,
    ];
    match i {
        0..=15 => rgb(P[i as usize]),
        16..=231 => {
            let n = i - 16;
            let lv = |v: u8| if v == 0 { 0 } else { 55 + 40 * v };
            Color::from_rgba8(lv(n / 36), lv(n / 6 % 6), lv(n % 6), 255)
        }
        _ => {
            let v = 8 + 10 * (i - 232);
            Color::from_rgba8(v, v, v, 255)
        }
    }
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let k = |x: f32, y: f32| x + (y - x) * t;
    Color::from_rgba(k(a.red(), b.red()), k(a.green(), b.green()), k(a.blue(), b.blue()), 1.0).unwrap_or(a)
}

/// (premier plan, arrière-plan) d'une cellule : gras = couleurs vives, inverse, atténué.
fn cell_colors(c: &Cell) -> (Color, Color) {
    let bold = c.attr & BOLD != 0;
    let mut fg = match c.fg {
        Col::Default => {
            if bold {
                rgb(0xffffff)
            } else {
                rgb(FG)
            }
        }
        Col::Idx(i) => ansi(if bold && i < 8 { i + 8 } else { i }),
        Col::Rgb(r, g, b) => Color::from_rgba8(r, g, b, 255),
    };
    let mut bg = match c.bg {
        Col::Default => rgb(BG),
        Col::Idx(i) => ansi(i),
        Col::Rgb(r, g, b) => Color::from_rgba8(r, g, b, 255),
    };
    if c.attr & INVERSE != 0 {
        std::mem::swap(&mut fg, &mut bg);
    }
    if c.attr & DIM != 0 {
        fg = mix(fg, bg, 0.5);
    }
    (fg, bg)
}

/// Traits et blocs dessinés à la main (jointures nettes entre cellules). Renvoie false si le glyphe est à la police.
fn draw_special(g: &mut Gfx, ch: char, x: f32, y: f32, cw: f32, lh: f32, fg: Color) -> bool {
    let u = ch as u32;
    if !(0x2500..=0x259F).contains(&u) {
        return false;
    }
    // L=1 R=2 U=4 D=8
    let seg: u8 = match u {
        0x2500 | 0x2501 | 0x2550 => 3,
        0x2502 | 0x2503 | 0x2551 => 12,
        0x250C | 0x2554 | 0x256D => 2 | 8,
        0x2510 | 0x2557 | 0x256E => 1 | 8,
        0x2514 | 0x255A | 0x2570 => 2 | 4,
        0x2518 | 0x255D | 0x256F => 1 | 4,
        0x251C | 0x2560 => 4 | 8 | 2,
        0x2524 | 0x2563 => 4 | 8 | 1,
        0x252C | 0x2566 => 1 | 2 | 8,
        0x2534 | 0x2569 => 1 | 2 | 4,
        0x253C | 0x256C => 15,
        _ => 0,
    };
    if seg != 0 {
        let t = (lh / 12.0).round().max(1.0);
        let cx = (x + cw / 2.0).floor();
        let cy = (y + lh / 2.0).floor();
        if seg & 1 != 0 {
            g.rect(x, cy, cx - x + t, t, fg);
        }
        if seg & 2 != 0 {
            g.rect(cx, cy, x + cw - cx, t, fg);
        }
        if seg & 4 != 0 {
            g.rect(cx, y, t, cy - y + t, fg);
        }
        if seg & 8 != 0 {
            g.rect(cx, cy, t, y + lh - cy, fg);
        }
        return true;
    }
    let (hh, hw) = ((lh / 2.0).round(), (cw / 2.0).round());
    match u {
        0x2588 => g.rect(x, y, cw, lh, fg),
        0x2580 => g.rect(x, y, cw, hh, fg),
        0x2584 => g.rect(x, y + lh - hh, cw, hh, fg),
        0x258C => g.rect(x, y, hw, lh, fg),
        0x2590 => g.rect(x + cw - hw, y, hw, lh, fg),
        0x2591 => g.rect(x, y, cw, lh, with_alpha(fg, 0.25)),
        0x2592 => g.rect(x, y, cw, lh, with_alpha(fg, 0.5)),
        0x2593 => g.rect(x, y, cw, lh, with_alpha(fg, 0.75)),
        _ => return false,
    }
    true
}

// ------------------------------------------------------------------ application

struct TermApp {
    store: Store,
    term: Term,
    pty: Option<Pty>,
    dead: Option<i32>,
    welcomed: bool,
    scroll: Scroller,
    follow: bool,
    font_idx: usize,
    ctrl: bool,
    alt: bool,
    page2: bool,
    kbd_asked: bool,
    born: Instant,
    activity: Instant,
    lay: Layout,
    pressed: Option<Action>,
    drag_acc: f32,
}

impl TermApp {
    fn new() -> TermApp {
        let store = Store::open("state.conf");
        let font_idx = store.get_i64("font").map(|v| (v.max(0) as usize).min(SIZES.len() - 1)).unwrap_or(DEFAULT_SIZE);
        TermApp {
            store,
            term: Term::new(10, 5),
            pty: None,
            dead: None,
            welcomed: false,
            scroll: Scroller::new(),
            follow: true,
            font_idx,
            ctrl: false,
            alt: false,
            page2: false,
            kbd_asked: false,
            born: Instant::now(),
            activity: Instant::now(),
            lay: Layout::default(),
            pressed: None,
            drag_acc: 0.0,
        }
    }

    // -------------------------------------------------------------- mise en page

    fn compute_layout(&self, ctx: &Ctx) -> Layout {
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let px = SIZES[self.font_idx] * s;
        let st = Text::new(px).mono();
        let cw = ctx.measure("M", &st).round().max(1.0);
        let lh = ctx.line_height(&st).round().max(1.0);
        let bar_h = (22.0 * s).round();
        let kbd_h = ctx.keyboard_height();
        let extra_h = if kbd_h > 0.0 { (30.0 * s).round() } else { 0.0 };
        let bottom = h - kbd_h - extra_h;
        let cols = (((w - 4.0 * s) / cw).floor() as usize).max(8);
        let rows = (((bottom - bar_h) / lh).floor() as usize).max(3);
        let x0 = ((w - cols as f32 * cw) / 2.0).floor().max(0.0);
        Layout { s, w, h, px, cw, lh, bar_h, x0, y0: bar_h, cols, rows, extra: Rect::new(0.0, bottom, w, extra_h), kbd_h }
    }

    fn hits(&self) -> Vec<(Rect, Action)> {
        let l = self.lay;
        let mut v = Vec::new();
        let bw = 28.0 * l.s;
        v.push((Rect::new(l.w - bw, 0.0, bw, l.bar_h), Action::Kbd));
        v.push((Rect::new(l.w - 2.0 * bw, 0.0, bw, l.bar_h), Action::FontUp));
        v.push((Rect::new(l.w - 3.0 * bw, 0.0, bw, l.bar_h), Action::FontDown));
        if l.extra.h > 0.0 {
            let kw = l.w / 8.0;
            for (i, (k, _)) in xkeys(self.page2).iter().enumerate() {
                v.push((Rect::new(i as f32 * kw, l.extra.y, kw, l.extra.h), Action::Key(*k)));
            }
        }
        v
    }

    fn hit(&self, x: f32, y: f32) -> Option<Action> {
        self.hits().into_iter().find(|(r, _)| r.contains(x, y)).map(|(_, a)| a)
    }

    fn sync_scroll(&mut self) {
        let l = self.lay;
        if l.lh <= 0.0 {
            return;
        }
        let total = (self.term.visible_sb() + self.term.rows) as f32 * l.lh;
        self.scroll.set_content(total, self.term.rows as f32 * l.lh);
        if self.follow {
            self.scroll.pos = self.scroll.max;
        } else if self.scroll.pos >= self.scroll.max - 1.0 {
            self.follow = true;
        }
    }

    // -------------------------------------------------------------- session

    fn start(&mut self) {
        let data = data_dir();
        let _ = std::fs::create_dir_all(data.join("bin"));
        let rc = data.join(".bashrc");
        if !rc.exists() {
            let _ = std::fs::write(&rc, DEFAULT_RC);
        }
        let datas = data.to_string_lossy().to_string();
        let shell = ["/bin/bash", "/usr/bin/bash", "/bin/sh"].into_iter().find(|p| std::path::Path::new(p).exists()).unwrap_or("/bin/sh");
        let path = format!("{datas}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin");
        let envs = [
            ("TERM", "xterm-256color"),
            ("COLORTERM", "truecolor"),
            ("LANG", "C.UTF-8"),
            ("HOME", datas.as_str()),
            ("SHELL", shell),
            ("PATH", path.as_str()),
            ("USER", "lapp"),
            ("LOGNAME", "lapp"),
            ("TMPDIR", "/tmp"),
        ];
        if !self.welcomed {
            self.welcomed = true;
            self.term.feed(WELCOME.as_bytes());
        }
        match Pty::spawn(shell, &[], &envs, &datas, self.term.cols as u16, self.term.rows as u16) {
            Ok(p) => {
                if let Some(t) = arg_value("--type") {
                    p.write(t.replace("\\n", "\r").as_bytes());
                }
                self.pty = Some(p);
                if arg_flag("--screenshot") {
                    self.pump_blocking();
                }
            }
            Err(e) => {
                self.dead = Some(127);
                self.term.feed(format!("\x1b[31mImpossible de lancer {shell} : {e}\x1b[0m\r\n").as_bytes());
            }
        }
    }

    /// Hors écran (--screenshot) : laisse au shell le temps de répondre avant de dessiner.
    fn pump_blocking(&mut self) {
        let t0 = Instant::now();
        let mut last = Instant::now();
        let mut got = false;
        while t0.elapsed() < Duration::from_millis(4000) {
            std::thread::sleep(Duration::from_millis(40));
            let n = self.pump();
            if n > 0 {
                got = true;
                last = Instant::now();
            }
            if got && last.elapsed() > Duration::from_millis(500) {
                break;
            }
            if self.pty.is_none() {
                break;
            }
        }
    }

    /// Lit la sortie du shell et l'interprète ; renvoie le nombre d'octets traités.
    fn pump(&mut self) -> usize {
        let mut out = Vec::new();
        let mut ended = None;
        if let Some(p) = self.pty.as_mut() {
            if let Err(code) = p.poll(&mut out) {
                ended = Some(code);
            }
        }
        let n = out.len();
        if n > 0 {
            let before = self.term.sb.len();
            let alt_before = self.term.alt_active;
            self.term.feed(&out);
            if !self.term.replies.is_empty() {
                let r = std::mem::take(&mut self.term.replies);
                if let Some(p) = self.pty.as_ref() {
                    p.write(&r);
                }
            }
            // défilement figé : on garde les mêmes lignes à l'écran quand l'historique grandit
            if !self.follow && !alt_before && !self.term.alt_active {
                let added = self.term.sb.len() as isize - before as isize;
                if added > 0 {
                    self.scroll.pos += added as f32 * self.lay.lh;
                }
            }
            self.activity = Instant::now();
        }
        if let Some(code) = ended {
            self.pty = None;
            self.dead = Some(code);
            self.term.feed(format!("\r\n\x1b[90m[processus terminé : {code}]\r\nToucher l'écran ou Entrée pour relancer\x1b[0m\r\n").as_bytes());
            self.follow = true;
        }
        n
    }

    fn restart(&mut self) {
        self.dead = None;
        self.term.feed(b"\r\n");
        self.follow = true;
        self.start();
    }

    // -------------------------------------------------------------- entrées

    fn send(&mut self, b: &[u8]) {
        if let Some(p) = self.pty.as_ref() {
            p.write(b);
        }
        self.follow = true;
        self.activity = Instant::now();
    }

    fn send_char(&mut self, c: char) {
        let mut out: Vec<u8> = Vec::new();
        if self.alt {
            out.push(0x1b);
        }
        let mut buf = [0u8; 4];
        if self.ctrl {
            let b = match c {
                'a'..='z' => Some(c as u8 - b'a' + 1),
                'A'..='Z' => Some(c as u8 - b'A' + 1),
                '@' | ' ' => Some(0),
                '[' => Some(27),
                '\\' => Some(28),
                ']' => Some(29),
                '^' => Some(30),
                '_' | '/' => Some(31),
                '?' => Some(127),
                _ => None,
            };
            match b {
                Some(b) => out.push(b),
                None => out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes()),
            }
        } else {
            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
        }
        self.ctrl = false;
        self.alt = false;
        self.send(&out);
    }

    fn send_xkey(&mut self, k: XKey) {
        match k {
            XKey::Ctrl => self.ctrl = !self.ctrl,
            XKey::Alt => self.alt = !self.alt,
            XKey::Page => self.page2 = !self.page2,
            XKey::Esc => self.send(b"\x1b"),
            XKey::Tab => self.send(b"\t"),
            XKey::Up => {
                let b = self.term.arrow('A');
                self.send(&b)
            }
            XKey::Down => {
                let b = self.term.arrow('B');
                self.send(&b)
            }
            XKey::Right => {
                let b = self.term.arrow('C');
                self.send(&b)
            }
            XKey::Left => {
                let b = self.term.arrow('D');
                self.send(&b)
            }
            XKey::Home => {
                let b: &[u8] = if self.term.app_cursor { b"\x1bOH" } else { b"\x1b[H" };
                self.send(b)
            }
            XKey::End => {
                let b: &[u8] = if self.term.app_cursor { b"\x1bOF" } else { b"\x1b[F" };
                self.send(b)
            }
            XKey::PgUp => self.send(b"\x1b[5~"),
            XKey::PgDn => self.send(b"\x1b[6~"),
            XKey::Ch(c) => self.send_char(c),
        }
    }

    fn scroll_by(&mut self, dy: f32) {
        let lh = self.lay.lh.max(1.0);
        if self.term.alt_active {
            // vim, less, htop… : le glissement devient des flèches (comme Termux)
            self.drag_acc += dy;
            while self.drag_acc >= lh {
                let b = self.term.arrow('A');
                self.send(&b);
                self.drag_acc -= lh;
            }
            while self.drag_acc <= -lh {
                let b = self.term.arrow('B');
                self.send(&b);
                self.drag_acc += lh;
            }
        } else {
            self.scroll.drag(dy);
            self.follow = false;
            self.sync_scroll();
        }
    }

    fn tap(&mut self, ctx: &Ctx, x: f32, y: f32) {
        match self.hit(x, y) {
            Some(Action::Kbd) => {
                if ctx.keyboard_height() > 0.0 {
                    ctx.keyboard(0);
                } else {
                    ctx.keyboard(1);
                }
            }
            Some(Action::FontDown) => self.set_font(self.font_idx.saturating_sub(1)),
            Some(Action::FontUp) => self.set_font((self.font_idx + 1).min(SIZES.len() - 1)),
            Some(Action::Key(k)) => self.send_xkey(k),
            None => {
                if self.dead.is_some() && self.pty.is_none() {
                    self.restart();
                } else if !self.follow {
                    self.follow = true;
                } else if ctx.keyboard_height() <= 0.0 {
                    ctx.keyboard(1);
                }
            }
        }
    }

    fn set_font(&mut self, idx: usize) {
        if idx != self.font_idx {
            self.font_idx = idx;
            let _ = self.store.set("font", idx);
        }
    }

    // -------------------------------------------------------------- dessin

    fn draw_bar(&self, ctx: &Ctx, g: &mut Gfx) {
        let l = self.lay;
        g.rect(0.0, 0.0, l.w, l.bar_h, rgb(PANEL));
        g.rect(0.0, l.bar_h - 1.0, l.w, 1.0, rgb(0x262a30));
        let title = if self.term.title.is_empty() { "Terminal" } else { self.term.title.as_str() };
        let st = Text::new(11.0 * l.s).color(rgb(0x8d95a1)).left().max_width(l.w - 96.0 * l.s);
        ctx.text_mid(g, title, 8.0 * l.s, l.bar_h / 2.0, &st);
        let bst = Text::new(11.0 * l.s).bold().color(rgb(0xc2c8d0)).center();
        for (r, a) in self.hits() {
            let label = match a {
                Action::Kbd => "KB",
                Action::FontUp => "A+",
                Action::FontDown => "A-",
                Action::Key(_) => continue,
            };
            if self.pressed == Some(a) {
                g.rect(r.x, r.y, r.w, r.h - 1.0, rgb(KEY));
            }
            let (cx, cy) = r.center();
            ctx.text_mid(g, label, cx, cy, &bst);
        }
    }

    fn draw_extra(&self, ctx: &Ctx, g: &mut Gfx) {
        let l = self.lay;
        g.rect(l.extra.x, l.extra.y, l.extra.w, l.extra.h, rgb(PANEL));
        let st = Text::new(11.0 * l.s).bold().color(rgb(FG)).center();
        let names = xkeys(self.page2);
        for (r, a) in self.hits() {
            let Action::Key(k) = a else { continue };
            let label = names.iter().find(|(x, _)| *x == k).map(|(_, n)| *n).unwrap_or("?");
            let armed = (k == XKey::Ctrl && self.ctrl) || (k == XKey::Alt && self.alt);
            let fill = if armed {
                rgb(KEY_ON)
            } else if self.pressed == Some(a) {
                rgb(0x3a3f48)
            } else {
                rgb(KEY)
            };
            let b = r.inset(1.5 * l.s);
            g.rrect(b.x, b.y, b.w, b.h, 4.0 * l.s, fill);
            let (cx, cy) = b.center();
            ctx.text_mid(g, label, cx, cy, &st);
        }
    }
}

impl App for TermApp {
    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        let new = self.compute_layout(ctx);
        let changed = new.cols != self.lay.cols || new.rows != self.lay.rows;
        self.lay = new;
        if changed && (self.term.cols != new.cols || self.term.rows != new.rows) {
            self.term.resize(new.cols, new.rows);
            if let Some(p) = self.pty.as_ref() {
                p.resize(new.cols as u16, new.rows as u16);
            }
        }
        if !self.kbd_asked {
            self.kbd_asked = true;
            ctx.keyboard(1);
        }
        // on attend un instant que la hauteur du clavier soit connue avant de lancer le shell
        if self.pty.is_none() && self.dead.is_none() {
            let ready = new.kbd_h > 0.0 || self.born.elapsed() > Duration::from_millis(350) || arg_flag("--screenshot");
            if ready {
                self.start();
            }
        }
        self.pump();
        if self.scroll.update(dt) {
            ctx.animate();
        }
        self.sync_scroll();
        let busy = self.activity.elapsed() < Duration::from_secs(2) || self.pty.is_none();
        ctx.redraw_in(Duration::from_millis(if busy { 30 } else { 200 }));
    }

    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let l = self.lay;
        let bg0 = rgb(BG);
        g.clear(bg0);
        let st = Text::new(l.px).mono();

        let first = (self.scroll.pos / l.lh).floor().max(0.0) as usize;
        let off = first as f32 * l.lh - self.scroll.pos;
        let cur_line = self.term.visible_sb() + self.term.cy;
        let show_cur = self.term.cursor_visible && self.follow && self.dead.is_none();

        for k in 0..=l.rows {
            let li = first + k;
            let Some(line) = self.term.vline(li) else { break };
            let y = l.y0 + off + k as f32 * l.lh;

            // arrière-plans, par plages de même couleur
            let mut run_start = 0usize;
            let mut run_col = bg0;
            for col in 0..=l.cols {
                let col_bg = if col < l.cols {
                    let c = line.get(col).copied().unwrap_or(BLANK);
                    let (_, mut b) = cell_colors(&c);
                    if show_cur && li == cur_line && col == self.term.cx {
                        b = rgb(CURSOR);
                    }
                    b
                } else {
                    bg0
                };
                if col_bg != run_col {
                    if run_col != bg0 {
                        g.rect(l.x0 + run_start as f32 * l.cw, y, (col - run_start) as f32 * l.cw, l.lh, run_col);
                    }
                    run_start = col;
                    run_col = col_bg;
                }
            }

            // glyphes
            for col in 0..l.cols {
                let c = line.get(col).copied().unwrap_or(BLANK);
                if c.ch == '\0' {
                    continue;
                }
                let is_cur = show_cur && li == cur_line && col == self.term.cx;
                let (mut fg, _) = cell_colors(&c);
                if is_cur {
                    fg = bg0;
                }
                let x = l.x0 + col as f32 * l.cw;
                let cells = if line.get(col + 1).map(|n| n.ch == '\0').unwrap_or(false) { 2.0 } else { 1.0 };
                if c.ch != ' ' && c.attr & HIDDEN == 0 && !draw_special(g, c.ch, x, y, l.cw, l.lh, fg) {
                    let mut b = [0u8; 4];
                    ctx.text(g, c.ch.encode_utf8(&mut b), x, y, &st.color(fg));
                }
                if c.attr & UNDER != 0 {
                    let t = (l.lh / 14.0).round().max(1.0);
                    g.rect(x, y + l.lh - t - 1.0, cells * l.cw, t, fg);
                }
                if c.attr & STRIKE != 0 {
                    g.rect(x, (y + l.lh / 2.0).floor(), cells * l.cw, 1.0, fg);
                }
            }
        }

        // ce qui dépasse sous la zone du terminal est recouvert
        let term_bottom = l.y0 + l.rows as f32 * l.lh;
        g.rect(0.0, term_bottom, l.w, (l.h - term_bottom).max(0.0), bg0);

        // indicateur de position dans l'historique
        if !self.term.alt_active && self.scroll.max > 0.0 && !self.follow {
            let area = l.rows as f32 * l.lh;
            let total = area + self.scroll.max;
            let th = (area * area / total).max(16.0 * l.s);
            let ty = l.y0 + (area - th) * (self.scroll.pos / self.scroll.max);
            g.rrect(l.w - 3.0 * l.s, ty, 2.0 * l.s, th, l.s, rgba(0xffffff, 0.35));
        }

        if l.extra.h > 0.0 {
            self.draw_extra(ctx, g);
        }
        self.draw_bar(ctx, g);
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        match ev {
            Event::Down { x, y } => {
                self.scroll.stop();
                self.drag_acc = 0.0;
                self.pressed = self.hit(x, y);
            }
            Event::Up { .. } => self.pressed = None,
            Event::Tap { x, y } => self.tap(ctx, x, y),
            Event::Drag { dy, .. } | Event::Wheel { dy } => {
                if self.pressed.is_none() {
                    self.scroll_by(dy);
                }
            }
            Event::DragEnd { vy, .. } => {
                if !self.term.alt_active && self.pressed.is_none() {
                    self.scroll.fling(vy);
                }
            }
            Event::Key(k) => {
                if self.pty.is_none() {
                    if k == Key::Enter && self.dead.is_some() {
                        self.restart();
                    }
                    return;
                }
                match k {
                    Key::Char(c) => self.send_char(c),
                    Key::Space => self.send_char(' '),
                    Key::Enter => self.send(b"\r"),
                    Key::Back => self.send(b"\x7f"),
                    Key::Up => self.send_xkey(XKey::Up),
                    Key::Down => self.send_xkey(XKey::Down),
                    Key::Left => self.send_xkey(XKey::Left),
                    Key::Right => self.send_xkey(XKey::Right),
                }
            }
            _ => {}
        }
    }

    /// Echap : envoyé au shell (vim…). Une fois le shell terminé, Echap quitte l'app.
    fn on_back(&mut self, _ctx: &Ctx) -> bool {
        if self.pty.is_some() {
            self.send(b"\x1b");
            true
        } else {
            false
        }
    }

    fn icon(&mut self, ctx: &Ctx, g: &mut Gfx, size: u32) {
        let f = size as f32;
        g.rrect(0.0, 0.0, f, f, f * 0.22, rgb(0x0f1318));
        g.rrect(f * 0.04, f * 0.04, f * 0.92, f * 0.92, f * 0.19, rgb(0x1b2129));
        let st = Text::new(f * 0.46).mono().color(rgb(0x4ade80)).center();
        ctx.text_mid(g, ">_", f / 2.0, f * 0.5, &st);
    }
}

fn main() {
    run(Config::new("Terminal"), TermApp::new());
}