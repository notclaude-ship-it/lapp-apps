// Voiture : course infinie en vue de dessus (écran portrait 272x480).
//
// Esquivez la circulation, ramassez les pièces, la vitesse augmente avec le temps.
// Commandes : toucher la moitié gauche / droite de l'écran (ou ← → / q d) pour changer de voie.
// Quitter : glisser vers le haut depuis le bord bas de l'écran, ou Echap.
//
// Sur PC : cargo run -- --screenshot out.png [--demo]   (rendu hors écran)
//          cargo run -- --icon icon.png                 (icône)

use std::time::{SystemTime, UNIX_EPOCH};

use lapp_sdk::arg_flag;
use lapp_sdk::prelude::*;

const LANES: usize = 4;
const PLAYER_Y: f32 = 0.80;
const ENEMY_COLORS: [u32; 6] = [0x1E88E5, 0xFDD835, 0x8E24AA, 0xFB8C00, 0x00ACC1, 0xB0BEC5];

struct Rng(u64);

impl Rng {
    fn new() -> Self {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1);
        Rng((n ^ 0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }

    /// Réel dans [0, 1[
    fn f(&mut self) -> f32 {
        (self.next() >> 8) as f32 / 16_777_216.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() as usize) % n.max(1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Ready,
    Playing,
    Dead,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Car,
    Coin,
}

struct Obj {
    kind: Kind,
    lane: usize,
    /// Centre vertical, en fraction de la hauteur de l'écran.
    y: f32,
    color: u32,
}

struct Game {
    store: Store,
    rng: Rng,
    state: State,

    objs: Vec<Obj>,
    lane: usize,
    /// Position latérale animée, en numéro de voie (fractionnaire).
    px: f32,
    speed: f32,
    scroll: f32,
    time: f32,
    dist: f32,
    coins: u32,
    spawn_t: f32,

    best: u32,
    new_record: bool,
    dead_t: f32,
    crash_x: f32,
    crash_y: f32,
    btn_pressed: bool,
}

impl Game {
    fn new() -> Self {
        let store = Store::open("voiture");
        let best = store.get_i64("best").unwrap_or(0).max(0) as u32;
        let mut g = Game {
            store,
            rng: Rng::new(),
            state: State::Ready,
            objs: Vec::new(),
            lane: 1,
            px: 1.0,
            speed: 0.5,
            scroll: 0.0,
            time: 0.0,
            dist: 0.0,
            coins: 0,
            spawn_t: 0.6,
            best,
            new_record: false,
            dead_t: 0.0,
            crash_x: 0.0,
            crash_y: 0.0,
            btn_pressed: false,
        };
        if arg_flag("--demo") {
            g.start();
            g.time = 25.0;
            g.speed = 0.8;
            g.dist = 310.0;
            g.coins = 7;
            g.lane = 2;
            g.px = 2.0;
            g.objs.push(Obj { kind: Kind::Car, lane: 0, y: 0.18, color: ENEMY_COLORS[0] });
            g.objs.push(Obj { kind: Kind::Car, lane: 2, y: 0.18, color: ENEMY_COLORS[1] });
            g.objs.push(Obj { kind: Kind::Coin, lane: 3, y: 0.18, color: 0 });
            g.objs.push(Obj { kind: Kind::Car, lane: 1, y: 0.42, color: ENEMY_COLORS[3] });
            g.objs.push(Obj { kind: Kind::Coin, lane: 3, y: 0.58, color: 0 });
            g.objs.push(Obj { kind: Kind::Car, lane: 3, y: 0.62, color: ENEMY_COLORS[2] });
        }
        g
    }

    fn start(&mut self) {
        self.objs.clear();
        self.state = State::Playing;
        self.lane = 1 + self.rng.below(2);
        self.px = self.lane as f32;
        self.speed = 0.5;
        self.time = 0.0;
        self.dist = 0.0;
        self.coins = 0;
        self.spawn_t = 0.5;
        self.new_record = false;
        self.dead_t = 0.0;
        self.btn_pressed = false;
    }

    fn score(&self) -> u32 {
        self.dist as u32 + self.coins * 25
    }

    fn steer(&mut self, dir: i32) {
        if self.state != State::Playing {
            return;
        }
        let n = (self.lane as i32 + dir).clamp(0, LANES as i32 - 1);
        self.lane = n as usize;
    }

    fn spawn_row(&mut self) {
        // 1 ou 2 voitures, jamais plus : il reste toujours au moins 2 voies libres.
        let n = if self.rng.f() < 0.35 && self.time > 8.0 { 2 } else { 1 };
        let mut free: Vec<usize> = (0..LANES).collect();
        for _ in 0..n {
            let i = self.rng.below(free.len());
            let lane = free.remove(i);
            let color = ENEMY_COLORS[self.rng.below(ENEMY_COLORS.len())];
            self.objs.push(Obj { kind: Kind::Car, lane, y: -0.12, color });
        }
        if self.rng.f() < 0.45 && !free.is_empty() {
            let i = self.rng.below(free.len());
            let lane = free[i];
            self.objs.push(Obj { kind: Kind::Coin, lane, y: -0.12, color: 0 });
        }
    }

    fn crash(&mut self, cx: f32, cy: f32) {
        self.state = State::Dead;
        self.dead_t = 0.0;
        self.crash_x = cx;
        self.crash_y = cy;
        let sc = self.score();
        if sc > self.best {
            self.best = sc;
            self.new_record = true;
            let _ = self.store.set("best", self.best as i64);
        }
    }

    fn button_rect(&self, ctx: &Ctx) -> Rect {
        let (w, h, s) = (ctx.width(), ctx.height(), ctx.scale());
        Rect::new(w * 0.15, h * 0.58, w * 0.7, 46.0 * s)
    }

    // ---- géométrie ---------------------------------------------------------------------------

    /// (x gauche de la route, largeur d'une voie)
    fn road(w: f32) -> (f32, f32) {
        let road_w = w * 0.76;
        ((w - road_w) / 2.0, road_w / LANES as f32)
    }

    fn lane_x(w: f32, lane: f32) -> f32 {
        let (x0, lw) = Self::road(w);
        x0 + (lane + 0.5) * lw
    }

    fn car_size(w: f32) -> (f32, f32) {
        let (_, lw) = Self::road(w);
        let cw = lw * 0.6;
        (cw, cw * 1.8)
    }

    fn render_icon(&self, g: &mut Gfx, w: f32, h: f32) {
        g.clear(rgb(0x2E7D32));
        let rw = w * 0.62;
        let rx = (w - rw) / 2.0;
        g.rect(rx, 0.0, rw, h, rgb(0x37393F));
        for k in 0..8 {
            let col = if k % 2 == 0 { rgb(0xE53935) } else { rgb(0xF5F5F5) };
            let y = k as f32 * h / 6.0 - h * 0.05;
            g.rect(rx - w * 0.04, y, w * 0.04, h / 6.0, col);
            g.rect(rx + rw, y, w * 0.04, h / 6.0, col);
        }
        for k in 0..4 {
            g.rect(w / 2.0 - w * 0.012, k as f32 * h / 3.0 + h * 0.03, w * 0.024, h * 0.17, rgba(0xFFFFFF, 0.8));
        }
        let cw = w * 0.3;
        draw_car(g, w / 2.0 - rw * 0.2, h * 0.55, cw, cw * 1.8, 0xE53935, true);
        draw_car(g, w / 2.0 + rw * 0.2, h * 0.2, cw * 0.8, cw * 1.44, 0x1E88E5, false);
    }
}

fn draw_car(g: &mut Gfx, cx: f32, cy: f32, cw: f32, ch: f32, color: u32, player: bool) {
    let c = rgb(color);
    // ombre
    g.rrect(cx - cw / 2.0 + cw * 0.08, cy - ch / 2.0 + ch * 0.04, cw, ch, cw * 0.3, rgba(0x000000, 0.28));
    // roues
    let wheel = rgb(0x151515);
    let wx = cw * 0.16;
    let wh = ch * 0.2;
    for &fy in &[0.12f32, 0.68] {
        let y = cy - ch / 2.0 + ch * fy;
        g.rrect(cx - cw / 2.0 - cw * 0.07, y, wx, wh, cw * 0.05, wheel);
        g.rrect(cx + cw / 2.0 - wx + cw * 0.07, y, wx, wh, cw * 0.05, wheel);
    }
    // carrosserie
    g.rrect(cx - cw / 2.0, cy - ch / 2.0, cw, ch, cw * 0.3, c);
    if player {
        g.rect(cx - cw * 0.07, cy - ch / 2.0 + ch * 0.03, cw * 0.14, ch * 0.94, rgba(0xFFFFFF, 0.9));
    }
    // vitres et toit
    let glass = rgb(0x16222e);
    g.rrect(cx - cw * 0.36, cy - ch * 0.22, cw * 0.72, ch * 0.2, cw * 0.1, glass);
    g.rect(cx - cw * 0.36, cy - ch * 0.02, cw * 0.72, ch * 0.16, shade(c, 0.82));
    g.rrect(cx - cw * 0.34, cy + ch * 0.14, cw * 0.68, ch * 0.13, cw * 0.1, glass);
    // phares avant et feux arrière
    let lamp = rgb(0xFFF59D);
    g.circle(cx - cw * 0.27, cy - ch * 0.455, cw * 0.09, lamp);
    g.circle(cx + cw * 0.27, cy - ch * 0.455, cw * 0.09, lamp);
    let tail = if player { rgb(0xFF1744) } else { rgb(0xB71C1C) };
    g.rect(cx - cw * 0.42, cy + ch * 0.44, cw * 0.22, ch * 0.04, tail);
    g.rect(cx + cw * 0.2, cy + ch * 0.44, cw * 0.22, ch * 0.04, tail);
}

fn draw_coin(g: &mut Gfx, cx: f32, cy: f32, r: f32) {
    g.circle(cx, cy, r, rgb(0xB8860B));
    g.circle(cx, cy, r * 0.85, rgb(0xFFC107));
    g.circle(cx, cy, r * 0.55, rgb(0xFFE082));
    g.rect(cx - r * 0.1, cy - r * 0.35, r * 0.2, r * 0.7, rgb(0xFFA000));
}

impl App for Game {
    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        let dt = dt.min(0.05);
        let (w, h) = (ctx.width(), ctx.height());
        match self.state {
            State::Ready => {
                self.scroll += 0.3 * dt;
                ctx.animate();
            }
            State::Dead => {
                self.dead_t += dt;
                if self.dead_t < 1.0 {
                    ctx.animate();
                }
            }
            State::Playing => {
                ctx.animate();
                self.time += dt;
                self.speed = (0.5 + self.time * 0.012).min(1.3);
                self.scroll += self.speed * dt;
                self.dist += self.speed * dt * 12.0;

                // déplacement latéral fluide
                let target = self.lane as f32;
                self.px += (target - self.px) * (dt * 16.0).min(1.0);

                // apparition de la circulation
                self.spawn_t -= dt;
                if self.spawn_t <= 0.0 {
                    self.spawn_row();
                    let base = (1.05 - self.time * 0.006).max(0.66);
                    self.spawn_t = base * (1.0 + 0.3 * self.rng.f());
                }

                // la circulation avance moins vite que nous
                let rel = self.speed * 0.55;
                for o in &mut self.objs {
                    o.y += rel * dt;
                }
                self.objs.retain(|o| o.y < 1.2);

                // collisions
                let (cw, ch) = Game::car_size(w);
                let (_, lw) = Game::road(w);
                let pcx = Game::lane_x(w, self.px);
                let pcy = PLAYER_Y * h;
                let mut hit: Option<(f32, f32)> = None;
                let mut gained = 0u32;
                self.objs.retain(|o| {
                    let ox = Game::lane_x(w, o.lane as f32);
                    let oy = o.y * h;
                    let dx = (ox - pcx).abs();
                    let dy = (oy - pcy).abs();
                    match o.kind {
                        Kind::Car => {
                            if dx < cw * 0.85 && dy < ch * 0.9 {
                                hit = Some(((ox + pcx) / 2.0, (oy + pcy) / 2.0));
                            }
                            true
                        }
                        Kind::Coin => {
                            if dx < lw * 0.45 && dy < ch * 0.6 {
                                gained += 1;
                                false
                            } else {
                                true
                            }
                        }
                    }
                });
                self.coins += gained;
                if let Some((x, y)) = hit {
                    self.crash(x, y);
                }
            }
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        let w = ctx.width();
        match ev {
            Event::Key(Key::Left) | Event::Key(Key::Char('q')) | Event::Key(Key::Char('a')) => self.steer(-1),
            Event::Key(Key::Right) | Event::Key(Key::Char('d')) => self.steer(1),
            Event::Key(Key::Space) | Event::Key(Key::Enter) => match self.state {
                State::Ready => self.start(),
                State::Dead if self.dead_t > 0.5 => self.start(),
                _ => {}
            },
            Event::Down { x, y } => match self.state {
                State::Ready => self.start(),
                State::Playing => self.steer(if x < w / 2.0 { -1 } else { 1 }),
                State::Dead => {
                    if self.dead_t > 0.5 && self.button_rect(ctx).contains(x, y) {
                        self.btn_pressed = true;
                    }
                }
            },
            Event::Up { .. } => self.btn_pressed = false,
            Event::Tap { x, y } => {
                if self.state == State::Dead && self.dead_t > 0.5 && self.button_rect(ctx).contains(x, y) {
                    self.start();
                }
            }
            _ => {}
        }
    }

    fn icon(&mut self, _ctx: &Ctx, g: &mut Gfx, size: u32) {
        self.render_icon(g, size as f32, size as f32);
    }

    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let (w, h, s) = (ctx.width(), ctx.height(), ctx.scale());
        let (x0, lw) = Game::road(w);
        let road_w = lw * LANES as f32;
        let (cw, ch) = Game::car_size(w);

        // herbe, bandes qui défilent
        g.clear(rgb(0x2E7D32));
        for k in 0..6 {
            if k % 2 == 0 {
                let y = (k as f32 * 0.25 + self.scroll).rem_euclid(1.5) - 0.25;
                g.rect(0.0, y * h, w, 0.25 * h, rgb(0x388E3C));
            }
        }

        // route + bordures rouges/blanches
        g.rect(x0, 0.0, road_w, h, rgb(0x37393F));
        for k in 0..16 {
            let y = (k as f32 * 0.08 + self.scroll).rem_euclid(1.28) - 0.08;
            let col = if k % 2 == 0 { rgb(0xE53935) } else { rgb(0xF5F5F5) };
            g.rect(x0 - 5.0 * s, y * h, 5.0 * s, 0.08 * h + 1.0, col);
            g.rect(x0 + road_w, y * h, 5.0 * s, 0.08 * h + 1.0, col);
        }

        // arbres sur les bas-côtés
        for i in 0..8 {
            let y = (i as f32 * 0.15 + self.scroll).rem_euclid(1.2) - 0.1;
            let jit = ((i * 37) % 5) as f32 - 2.0;
            let r = x0 * 0.28;
            for &left in &[true, false] {
                let base = if left { x0 * 0.5 } else { w - x0 * 0.5 };
                let off = if left { jit } else { -jit } * x0 * 0.07;
                let cx = base + off;
                let cy = y * h + if left { 0.0 } else { 0.07 * h };
                g.circle(cx + r * 0.2, cy + r * 0.25, r, rgba(0x000000, 0.25));
                g.circle(cx, cy, r, rgb(0x1B5E20));
                g.circle(cx - r * 0.25, cy - r * 0.25, r * 0.55, rgb(0x2E8B34));
            }
        }

        // lignes de voies pointillées
        for l in 1..LANES {
            let lx = x0 + l as f32 * lw;
            for k in 0..8 {
                let y = (k as f32 * 0.16 + self.scroll).rem_euclid(1.28) - 0.16;
                g.rect(lx - 1.5 * s, y * h, 3.0 * s, 0.08 * h, rgba(0xFFFFFF, 0.75));
            }
        }

        // circulation et pièces
        for o in &self.objs {
            let cx = Game::lane_x(w, o.lane as f32);
            let cy = o.y * h;
            match o.kind {
                Kind::Car => draw_car(g, cx, cy, cw, ch, o.color, false),
                Kind::Coin => draw_coin(g, cx, cy, lw * 0.22),
            }
        }

        // voiture du joueur
        let pcx = Game::lane_x(w, self.px);
        if self.state != State::Dead {
            draw_car(g, pcx, PLAYER_Y * h, cw, ch, 0xE53935, true);
        } else {
            draw_car(g, pcx, PLAYER_Y * h, cw, ch, 0x555555, true);
            let t = (self.dead_t / 0.6).min(1.0);
            let r = cw * (0.6 + 1.2 * t);
            let a = 1.0 - 0.5 * t;
            g.circle(self.crash_x, self.crash_y, r, with_alpha(rgb(0xFF6D00), a));
            g.circle(self.crash_x, self.crash_y, r * 0.65, with_alpha(rgb(0xFFC107), a));
            g.circle(self.crash_x, self.crash_y, r * 0.3, with_alpha(rgb(0xFFFFFF), a));
            if self.dead_t < 0.5 {
                g.rect(0.0, 0.0, w, h, rgba(0xFF3300, 0.35 * (1.0 - self.dead_t / 0.5)));
            }
        }

        // bandeau d'infos
        let bar = 40.0 * s;
        g.rect(0.0, 0.0, w, bar, rgba(0x000000, 0.55));
        let st_l = Text::new(14.0 * s).bold().color(rgb(0xFFFFFF)).left();
        let st_r = Text::new(12.0 * s).color(rgb(0xFFC107)).right();
        let st_c = Text::new(11.0 * s).color(rgb(0xB0BEC5)).center();
        ctx.text_mid(g, &format!("{}", self.score()), 12.0 * s, bar / 2.0, &st_l);
        ctx.text_mid(g, &format!("Record {}", self.best), w - 12.0 * s, bar / 2.0 - 7.0 * s, &st_r);
        ctx.text_mid(g, &format!("{} pièces", self.coins), w - 12.0 * s, bar / 2.0 + 8.0 * s, &st_r);
        ctx.text_mid(g, &format!("{} km/h", (self.speed * 180.0) as u32), w / 2.0, bar / 2.0, &st_c);

        match self.state {
            State::Ready => {
                g.rect(0.0, 0.0, w, h, rgba(0x000000, 0.55));
                let title = Text::new(34.0 * s).bold().color(rgb(0xFFFFFF)).center();
                let sub = Text::new(14.0 * s).color(rgb(0xFFC107)).center();
                let hint = Text::new(12.0 * s).color(rgb(0xCFD8DC)).center();
                ctx.text_mid(g, "VOITURE", w / 2.0, h * 0.30, &title);
                ctx.text_mid(g, "Touchez pour démarrer", w / 2.0, h * 0.42, &sub);
                ctx.text_mid(g, "Gauche / droite : changer de voie", w / 2.0, h * 0.50, &hint);
                ctx.text_mid(g, "Évitez les voitures, prenez les pièces", w / 2.0, h * 0.55, &hint);
                if self.best > 0 {
                    ctx.text_mid(g, &format!("Record : {}", self.best), w / 2.0, h * 0.63, &sub);
                }
            }
            State::Dead if self.dead_t > 0.35 => {
                g.rect(0.0, bar, w, h - bar, rgba(0x000000, 0.7));
                let over = Text::new(28.0 * s).bold().color(rgb(0xFF5252)).center();
                let res = Text::new(18.0 * s).bold().color(rgb(0xFFFFFF)).center();
                let det = Text::new(13.0 * s).color(rgb(0xCFD8DC)).center();
                ctx.text_mid(g, "ACCIDENT !", w / 2.0, h * 0.26, &over);
                ctx.text_mid(g, &format!("Score : {}", self.score()), w / 2.0, h * 0.36, &res);
                ctx.text_mid(g, &format!("{} pièces", self.coins), w / 2.0, h * 0.42, &det);
                if self.new_record {
                    let rec = Text::new(15.0 * s).bold().color(rgb(0xFFC107)).center();
                    ctx.text_mid(g, "Nouveau record !", w / 2.0, h * 0.49, &rec);
                } else {
                    ctx.text_mid(g, &format!("Record : {}", self.best), w / 2.0, h * 0.49, &det);
                }
                ui::button(ctx, g, self.button_rect(ctx), "Rejouer", ButtonStyle::Primary, self.btn_pressed);
            }
            _ => {}
        }
    }
}

fn main() {
    run(Config::new("Voiture").size(272, 480), Game::new());
}