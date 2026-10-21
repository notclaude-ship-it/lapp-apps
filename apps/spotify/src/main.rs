// Spotify : client lapp pour spotifyd (Spotify Connect), via MPRIS sur le bus D-Bus filtré de lapp.
//
// Affiche le morceau en cours (pochette, titre, artiste, album, progression) et pilote le lecteur :
// lecture/pause, précédent/suivant, positionnement dans le morceau, aléatoire, répétition.
//
// spotifyd n'expose MPRIS que lorsqu'il est le lecteur actif : tant qu'aucun client Spotify ne l'a
// sélectionné, l'app affiche un écran d'attente.
//
// Quitter : glisser vers le haut depuis le bord bas de l'écran, ou Echap.
// Sur PC : cargo run -- --screenshot out.png --demo   (rendu hors écran avec un morceau fictif)
//          cargo run -- --icon icon.png               (icône)
// Diagnostic : lapp run com.exemple.spotify --probe   (lit l'état MPRIS, envoie quelques commandes, affiche le résultat)
//              ... --screenshot out.png --settle 4000 (laisse 4 s au fil D-Bus et aux pochettes avant la capture)

use std::time::{Duration, Instant};

use lapp_sdk::prelude::*;
use lapp_sdk::tiny_skia::{FillRule, LineCap, LineJoin, Paint, PathBuilder, Stroke, Transform};
use lapp_sdk::{arg_flag, arg_value};

mod art;
#[allow(dead_code)]
mod dbus;
mod player;

use art::{cover_pixmap, Art, Raw};
use player::{Cmd, LoopMode, Player, Snapshot, Status};

// ------------------------------------------------------------------ palette

fn bg() -> Color {
    rgb(0x121212)
}
fn surface() -> Color {
    rgb(0x1E1E1E)
}
fn line() -> Color {
    rgb(0x2A2A2A)
}
fn white() -> Color {
    rgb(0xFFFFFF)
}
fn sub() -> Color {
    rgb(0xB3B3B3)
}
fn dim() -> Color {
    rgb(0x4D4D4D)
}
fn green() -> Color {
    rgb(0x1DB954)
}

const COVER_COLORS: [u32; 8] = [0x2D46B9, 0x8D67AB, 0xE8115B, 0x1E3264, 0x477D95, 0xBA5D07, 0x509BF5, 0x148A08];

fn hash(s: &str) -> usize {
    s.bytes().fold(0xcbf29ce4u32, |h, b| (h ^ b as u32).wrapping_mul(0x01000193)) as usize
}

fn fmt_time(us: i64) -> String {
    let s = (us.max(0) / 1_000_000) as u64;
    format!("{}:{:02}", s / 60, s % 60)
}

// ------------------------------------------------------------------ icônes (anticrénelées, mises en cache GPU)

#[derive(Clone, Copy, Debug)]
enum Icon {
    Play,
    Pause,
    Prev,
    Next,
    Shuffle,
    Repeat,
}

fn path(f: f32, pts: &[(f32, f32)], close: bool) -> Option<lapp_sdk::tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    pb.move_to(pts[0].0 * f, pts[0].1 * f);
    for p in &pts[1..] {
        pb.line_to(p.0 * f, p.1 * f);
    }
    if close {
        pb.close();
    }
    pb.finish()
}

fn make_icon(kind: Icon, n: u32, c: Color) -> Pixmap {
    let mut pm = Pixmap::new(n, n).expect("pixmap");
    let f = n as f32;
    let mut paint = Paint::default();
    paint.set_color(c);
    paint.anti_alias = true;
    let mut round = Stroke::default();
    round.width = 0.06 * f;
    round.line_cap = LineCap::Round;
    round.line_join = LineJoin::Round;
    let mut thick = round.clone();
    thick.width = 0.085 * f;

    // polygone plein aux angles légèrement arrondis
    let fill = |pm: &mut Pixmap, pts: &[(f32, f32)]| {
        if let Some(p) = path(f, pts, true) {
            pm.fill_path(&p, &paint, FillRule::Winding, Transform::identity(), None);
            pm.stroke_path(&p, &paint, &round, Transform::identity(), None);
        }
    };
    let line = |pm: &mut Pixmap, pts: &[(f32, f32)]| {
        if let Some(p) = path(f, pts, false) {
            pm.stroke_path(&p, &paint, &thick, Transform::identity(), None);
        }
    };

    match kind {
        Icon::Play => fill(&mut pm, &[(0.32, 0.20), (0.32, 0.80), (0.82, 0.50)]),
        Icon::Pause => {
            fill(&mut pm, &[(0.28, 0.22), (0.42, 0.22), (0.42, 0.78), (0.28, 0.78)]);
            fill(&mut pm, &[(0.58, 0.22), (0.72, 0.22), (0.72, 0.78), (0.58, 0.78)]);
        }
        Icon::Prev => {
            fill(&mut pm, &[(0.22, 0.22), (0.31, 0.22), (0.31, 0.78), (0.22, 0.78)]);
            fill(&mut pm, &[(0.80, 0.22), (0.80, 0.78), (0.38, 0.50)]);
        }
        Icon::Next => {
            fill(&mut pm, &[(0.20, 0.22), (0.20, 0.78), (0.62, 0.50)]);
            fill(&mut pm, &[(0.69, 0.22), (0.78, 0.22), (0.78, 0.78), (0.69, 0.78)]);
        }
        Icon::Shuffle => {
            line(&mut pm, &[(0.10, 0.32), (0.36, 0.32), (0.62, 0.68), (0.74, 0.68)]);
            line(&mut pm, &[(0.10, 0.68), (0.36, 0.68), (0.62, 0.32), (0.74, 0.32)]);
            fill(&mut pm, &[(0.72, 0.55), (0.72, 0.81), (0.90, 0.68)]);
            fill(&mut pm, &[(0.72, 0.19), (0.72, 0.45), (0.90, 0.32)]);
        }
        Icon::Repeat => {
            line(&mut pm, &[(0.14, 0.52), (0.14, 0.34), (0.64, 0.34)]);
            fill(&mut pm, &[(0.62, 0.20), (0.62, 0.48), (0.84, 0.34)]);
            line(&mut pm, &[(0.86, 0.48), (0.86, 0.66), (0.36, 0.66)]);
            fill(&mut pm, &[(0.38, 0.52), (0.38, 0.80), (0.16, 0.66)]);
        }
    }
    pm
}

fn draw_icon(g: &mut Gfx, kind: Icon, size: f32, c: Color, cx: f32, cy: f32) {
    let n = size.round().max(6.0) as u32;
    let u = c.to_color_u8();
    let key = format!("ic:{:?}:{}:{:02x}{:02x}{:02x}{:02x}", kind, n, u.red(), u.green(), u.blue(), u.alpha());
    g.img(&key, &|| make_icon(kind, n, c));
    g.blit(&key, cx - n as f32 / 2.0, cy - n as f32 / 2.0, 1.0);
}

// ------------------------------------------------------------------ mise en page

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Btn {
    Shuffle,
    Prev,
    Play,
    Next,
    Repeat,
}

struct Layout {
    s: f32,
    w: f32,
    head: f32,
    cover: Rect,
    title_cy: f32,
    artist_cy: f32,
    album_cy: f32,
    bar: Rect,
    bar_hit: Rect,
    time_cy: f32,
    ctrl_cy: f32,
    foot_cy: f32,
    /// (bouton, centre x, rayon du bouton, rayon de toucher)
    btns: [(Btn, f32, f32, f32); 5],
}

impl Layout {
    fn new(ctx: &Ctx) -> Layout {
        let s = ctx.scale();
        let w = ctx.width();
        let head = 44.0 * s;
        let cs = (w - 76.0 * s).max(40.0);
        let cover = Rect::new((w - cs) / 2.0, head + 16.0 * s, cs, cs);
        let title_cy = cover.bottom() + 24.0 * s;
        let artist_cy = title_cy + 24.0 * s;
        let album_cy = artist_cy + 19.0 * s;
        let bar_y = album_cy + 34.0 * s;
        let bar = Rect::new(26.0 * s, bar_y, w - 52.0 * s, 4.0 * s);
        let bar_hit = Rect::new(bar.x - 10.0 * s, bar_y - 14.0 * s, bar.w + 20.0 * s, 32.0 * s);
        let time_cy = bar_y + 22.0 * s;
        let ctrl_cy = time_cy + 40.0 * s;
        let foot_cy = ctrl_cy + 45.0 * s;
        let btns = [
            (Btn::Shuffle, 32.0 * s, 13.0 * s, 24.0 * s),
            (Btn::Prev, 86.0 * s, 17.0 * s, 28.0 * s),
            (Btn::Play, w / 2.0, 29.0 * s, 34.0 * s),
            (Btn::Next, w - 86.0 * s, 17.0 * s, 28.0 * s),
            (Btn::Repeat, w - 32.0 * s, 13.0 * s, 24.0 * s),
        ];
        Layout { s, w, head, cover, title_cy, artist_cy, album_cy, bar, bar_hit, time_cy, ctrl_cy, foot_cy, btns }
    }

    fn hit_button(&self, x: f32, y: f32) -> Option<Btn> {
        self.btns.iter().find(|(_, cx, _, hit)| (x - cx).powi(2) + (y - self.ctrl_cy).powi(2) <= hit * hit).map(|b| b.0)
    }

    fn frac(&self, x: f32) -> f32 {
        ((x - self.bar.x) / self.bar.w).clamp(0.0, 1.0)
    }
}

// ------------------------------------------------------------------ application

struct SpotifyApp {
    player: Player,
    art: Art,
    snap: Snapshot,
    raw: Option<std::sync::Arc<Raw>>,
    seen_err: u64,
    pressed: Option<Btn>,
    /// fraction de la piste pendant un glissement sur la barre de progression
    seek: Option<f32>,
    /// position demandée (µs) affichée en attendant que le lecteur la confirme
    pos_override: Option<(i64, Instant)>,
}

impl SpotifyApp {
    fn new(player: Player, art: Art) -> SpotifyApp {
        let snap = player.snapshot();
        let raw = art.want(&snap.track.art_url);
        SpotifyApp { player, art, snap, raw, seen_err: 0, pressed: None, seek: None, pos_override: None }
    }

    fn shown_pos(&self) -> i64 {
        let s = &self.snap;
        let len = s.track.length_us.max(0);
        if let Some(f) = self.seek {
            return (f as f64 * len as f64) as i64;
        }
        let elapsed = if s.playing() { s.sampled.elapsed().as_micros() as i64 } else { 0 };
        let p = match self.pos_override {
            Some((us, at)) if at.elapsed() < Duration::from_millis(1500) => us + if s.playing() { at.elapsed().as_micros() as i64 } else { 0 },
            _ => s.position_us + elapsed,
        };
        if len > 0 {
            p.clamp(0, len)
        } else {
            p.max(0)
        }
    }

    fn press(&mut self, b: Btn) {
        let s = &self.snap;
        match b {
            Btn::Play => self.player.send(Cmd::PlayPause),
            Btn::Next if s.can_next => self.player.send(Cmd::Next),
            Btn::Prev if s.can_prev => self.player.send(Cmd::Prev),
            Btn::Shuffle => self.player.send(Cmd::Shuffle),
            Btn::Repeat => self.player.send(Cmd::Loop),
            _ => {}
        }
    }

    fn commit_seek(&mut self, frac: f32) {
        let len = self.snap.track.length_us;
        if len > 0 {
            let us = (frac as f64 * len as f64) as i64;
            self.pos_override = Some((us, Instant::now()));
            self.player.send(Cmd::Seek(us));
        }
    }

    // -------------------------------------------------------------- écrans

    fn draw_waiting(&self, ctx: &Ctx, g: &mut Gfx, l: &Layout, title: &str, lines: &[&str]) {
        let s = l.s;
        let cx = l.w / 2.0;
        let cy = l.head + 120.0 * s;
        g.circle(cx, cy, 46.0 * s, green());
        // trois barres façon égaliseur
        let (bw, gap) = (9.0 * s, 6.0 * s);
        let hs = [26.0 * s, 44.0 * s, 20.0 * s];
        let x0 = cx - (3.0 * bw + 2.0 * gap) / 2.0;
        for (i, h) in hs.iter().enumerate() {
            g.rrect(x0 + i as f32 * (bw + gap), cy - h / 2.0, bw, *h, bw / 2.0, rgb(0x121212));
        }
        let t = Text::new(19.0 * s).bold().color(white()).center().max_width(l.w - 32.0 * s);
        ctx.text_mid(g, title, cx, cy + 82.0 * s, &t);
        let st = Text::new(13.0 * s).color(sub()).center().max_width(l.w - 32.0 * s);
        for (i, line) in lines.iter().enumerate() {
            ctx.text_mid(g, line, cx, cy + 114.0 * s + i as f32 * 20.0 * s, &st);
        }
    }

    fn draw_cover(&self, g: &mut Gfx, ctx: &Ctx, l: &Layout, snap: &Snapshot) {
        let s = l.s;
        let c = l.cover;
        let radius = 10.0 * s;
        g.rrect(c.x, c.y + 6.0 * s, c.w, c.h, radius, rgba(0x000000, 0.35));
        match &self.raw {
            Some(raw) => {
                let size = c.w.round() as u32;
                let key = format!("cover:{}:{}", snap.track.art_url, size);
                g.img(&key, &|| cover_pixmap(raw, size, radius));
                g.blit(&key, c.x, c.y, 1.0);
            }
            None => {
                let seed = format!("{}{}", snap.track.album, snap.track.title);
                g.rrect(c.x, c.y, c.w, c.h, radius, rgb(COVER_COLORS[hash(&seed) % COVER_COLORS.len()]));
                let letter: String = snap.track.title.chars().next().map(|ch| ch.to_uppercase().to_string()).unwrap_or_default();
                let st = Text::new(84.0 * s).bold().color(rgba(0xFFFFFF, 0.85)).center();
                let (mx, my) = c.center();
                ctx.text_mid(g, &letter, mx, my, &st);
            }
        }
    }

    fn draw_player(&self, ctx: &Ctx, g: &mut Gfx, l: &Layout, snap: &Snapshot) {
        let s = l.s;
        let w = l.w;
        self.draw_cover(g, ctx, l, snap);

        let tr = &snap.track;
        let (title, artist) = if tr.title.is_empty() {
            ("Aucun titre", "Lancez la lecture depuis Spotify")
        } else {
            (tr.title.as_str(), tr.artist.as_str())
        };
        let maxw = w - 36.0 * s;
        ctx.text_mid(g, title, w / 2.0, l.title_cy, &Text::new(19.0 * s).bold().color(white()).center().max_width(maxw));
        ctx.text_mid(g, artist, w / 2.0, l.artist_cy, &Text::new(14.0 * s).color(sub()).center().max_width(maxw));
        ctx.text_mid(g, &tr.album, w / 2.0, l.album_cy, &Text::new(12.0 * s).color(rgb(0x808080)).center().max_width(maxw));

        // progression
        let len = tr.length_us.max(0);
        let pos = self.shown_pos();
        let frac = if len > 0 { (pos as f32 / len as f32).clamp(0.0, 1.0) } else { 0.0 };
        let b = l.bar;
        g.rrect(b.x, b.y, b.w, b.h, b.h / 2.0, dim());
        let fw = b.w * frac;
        if fw > 0.0 {
            g.rrect(b.x, b.y, fw.max(b.h), b.h, b.h / 2.0, green());
        }
        let kr = if self.seek.is_some() { 8.0 * s } else { 5.5 * s };
        g.circle(b.x + fw, b.y + b.h / 2.0, kr, white());
        let tt = Text::new(11.0 * s).color(sub());
        ctx.text_mid(g, &fmt_time(pos), b.x, l.time_cy, &tt.clone().left());
        let total = if len > 0 { fmt_time(len) } else { "–:––".to_string() };
        ctx.text_mid(g, &total, b.right(), l.time_cy, &tt.right());

        // commandes
        let tone = |on: bool, enabled: bool, pressed: bool| {
            let c = if !enabled {
                dim()
            } else if on {
                green()
            } else {
                sub()
            };
            if pressed {
                shade(c, 0.7)
            } else {
                c
            }
        };
        for (btn, cx, r, _) in l.btns {
            let pressed = self.pressed == Some(btn);
            let cy = l.ctrl_cy;
            match btn {
                Btn::Shuffle => draw_icon(g, Icon::Shuffle, 26.0 * s, tone(snap.shuffle, true, pressed), cx, cy),
                Btn::Repeat => {
                    draw_icon(g, Icon::Repeat, 26.0 * s, tone(snap.loop_mode != LoopMode::Off, true, pressed), cx, cy);
                    if snap.loop_mode == LoopMode::Track {
                        g.circle(cx + 11.0 * s, cy - 10.0 * s, 6.0 * s, green());
                        ctx.text_mid(g, "1", cx + 11.0 * s, cy - 10.0 * s, &Text::new(9.0 * s).bold().color(bg()).center());
                    }
                }
                Btn::Prev => draw_icon(g, Icon::Prev, 34.0 * s, nav_tone(snap.can_prev, pressed), cx, cy),
                Btn::Next => draw_icon(g, Icon::Next, 34.0 * s, nav_tone(snap.can_next, pressed), cx, cy),
                Btn::Play => {
                    g.circle(cx, cy, r, if pressed { shade(white(), 0.85) } else { white() });
                    let kind = if snap.playing() { Icon::Pause } else { Icon::Play };
                    draw_icon(g, kind, 30.0 * s, bg(), cx, cy);
                }
            }
        }

        // pied de page : état du lecteur et volume
        let label = snap
            .player
            .as_deref()
            .and_then(|n| n.strip_prefix("org.mpris.MediaPlayer2."))
            .map(|n| n.split(".instance").next().unwrap_or(n))
            .unwrap_or("spotifyd");
        let state = match snap.status {
            Status::Playing => "Lecture",
            Status::Paused => "En pause",
            Status::Stopped => "Arrêté",
        };
        let fy = l.foot_cy;
        g.circle(26.0 * s + 3.0 * s, fy, 3.0 * s, if snap.playing() { green() } else { dim() });
        ctx.text_mid(g, &format!("{state} · {label}"), 38.0 * s, fy, &Text::new(11.0 * s).color(sub()).left().max_width(w * 0.6));
        if let Some(v) = snap.volume {
            ctx.text_mid(g, &format!("Vol. {} %", (v * 100.0).round() as i32), w - 26.0 * s, fy, &Text::new(11.0 * s).color(sub()).right());
        }
    }
}

fn nav_tone(enabled: bool, pressed: bool) -> Color {
    match (enabled, pressed) {
        (false, _) => dim(),
        (true, true) => shade(white(), 0.7),
        (true, false) => white(),
    }
}

impl App for SpotifyApp {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let l = Layout::new(ctx);
        let s = l.s;
        let snap = self.snap.clone();
        g.clear(bg());

        g.rect(0.0, 0.0, l.w, l.head, surface());
        g.rect(0.0, l.head - 1.0, l.w, 1.0, line());
        ctx.text_mid(g, "Spotify", l.w / 2.0, l.head / 2.0, &Text::new(17.0 * s).bold().color(white()).center());

        if let Some(err) = &snap.bus_error {
            self.draw_waiting(ctx, g, &l, "D-Bus indisponible", &["Le bus MPRIS n'est pas accessible.", err.as_str()]);
        } else if snap.player.is_none() {
            self.draw_waiting(
                ctx,
                g,
                &l,
                "En attente de Spotify",
                &["Ouvrez Spotify, puis choisissez cet", "appareil dans Spotify Connect."],
            );
        } else {
            self.draw_player(ctx, g, &l, &snap);
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        let l = Layout::new(ctx);
        let active = self.snap.player.is_some() && self.snap.bus_error.is_none();
        match ev {
            Event::Down { x, y } if active => {
                if l.bar_hit.contains(x, y) && self.snap.can_seek && self.snap.track.length_us > 0 {
                    self.seek = Some(l.frac(x));
                } else {
                    self.pressed = l.hit_button(x, y);
                }
            }
            Event::Move { x, .. } | Event::Drag { x, .. } => {
                if self.seek.is_some() {
                    self.seek = Some(l.frac(x));
                }
            }
            Event::Up { x, .. } => {
                if self.seek.take().is_some() {
                    self.commit_seek(l.frac(x));
                }
                self.pressed = None;
            }
            Event::Tap { x, y } if active => {
                if let Some(b) = l.hit_button(x, y) {
                    self.press(b);
                }
            }
            Event::Key(Key::Space) | Event::Key(Key::Enter) if active => self.player.send(Cmd::PlayPause),
            Event::Key(Key::Right) if active => self.player.send(Cmd::Next),
            Event::Key(Key::Left) if active => self.player.send(Cmd::Prev),
            _ => {}
        }
    }

    fn tick(&mut self, ctx: &Ctx, _dt: f32) {
        self.snap = self.player.snapshot();
        if let Some((n, msg)) = &self.snap.error {
            if *n != self.seen_err {
                self.seen_err = *n;
                ctx.toast(msg.replace('\n', " "));
            }
        }
        self.raw = self.art.want(&self.snap.track.art_url);
        if let Some((_, at)) = self.pos_override {
            if at.elapsed() > Duration::from_millis(1500) {
                self.pos_override = None;
            }
        }
        ctx.redraw_in(Duration::from_millis(if self.snap.playing() { 250 } else { 600 }));
    }

    fn icon(&mut self, _ctx: &Ctx, g: &mut Gfx, size: u32) {
        let f = size as f32;
        g.rrect(0.0, 0.0, f, f, f * 0.22, green());
        let (bw, gap) = (f * 0.12, f * 0.07);
        let hs = [f * 0.34, f * 0.54, f * 0.26];
        let x0 = (f - (3.0 * bw + 2.0 * gap)) / 2.0;
        for (i, h) in hs.iter().enumerate() {
            g.rrect(x0 + i as f32 * (bw + gap), f * 0.5 - h / 2.0, bw, *h, bw / 2.0, rgb(0x191414));
        }
    }
}

/// Diagnostic sans interface : `--probe` affiche l'état lu via MPRIS puis émet chaque commande l'une après l'autre.
fn probe() {
    let p = Player::spawn();
    let show = |p: &Player, label: &str| {
        let s = p.snapshot();
        println!(
            "[{label}] bus_error={:?} player={:?} status={:?} track={:?} pos={}us shuffle={} loop={:?} volume={:?} can_seek={} error={:?}",
            s.bus_error, s.player, s.status, s.track, s.position_us, s.shuffle, s.loop_mode, s.volume, s.can_seek, s.error
        );
    };
    std::thread::sleep(Duration::from_millis(1500));
    show(&p, "initial");
    for (label, cmd) in [
        ("PlayPause", Cmd::PlayPause),
        ("Shuffle", Cmd::Shuffle),
        ("Loop", Cmd::Loop),
        ("Seek 120 s", Cmd::Seek(120_000_000)),
        ("Next", Cmd::Next),
    ] {
        p.send(cmd);
        std::thread::sleep(Duration::from_millis(1200));
        show(&p, label);
    }
}

fn main() {
    if arg_flag("--probe") {
        probe();
        return;
    }
    let demo = arg_flag("--demo");
    let player = if demo { Player::demo() } else { Player::spawn() };
    let art = Art::new();
    if demo {
        // capture hors écran : la pochette doit être prête dès la première image
        art.preload_blocking(&player.snapshot().track.art_url);
    } else if let Some(ms) = arg_value("--settle").and_then(|v| v.parse::<u64>().ok()) {
        std::thread::sleep(Duration::from_millis(ms));
        let url = player.snapshot().track.art_url;
        if !url.is_empty() {
            art.preload_blocking(&url);
        }
    }
    run(Config::new("Spotify"), SpotifyApp::new(player, art));
}
