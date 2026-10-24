//! Logique de Cosmo Jump : physique, génération infinie, collisions. Unités = points (échelle 272 de large).
use lapp_sdk::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub const GRAV: f32 = 1000.0;
pub const JUMP: f32 = 540.0;
pub const SPRING: f32 = 920.0;
pub const ROCKET_V: f32 = 780.0;
pub const PW: f32 = 14.0;
pub const PH: f32 = 30.0;

pub struct Rng(u64);
impl Rng {
    pub fn new() -> Rng {
        let n = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
        Rng((n ^ 0x9E37_79B9_7F4A_7C15) | 1)
    }
    pub fn u(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    pub fn f(&mut self) -> f32 {
        (self.u() >> 40) as f32 / 16_777_216.0
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Normal,
    Moving,
    Crumble,
}

pub struct Plat {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub kind: Kind,
    pub dir: f32,
    pub speed: f32,
    pub crack: f32, // < 0 : intacte ; sinon temps depuis le contact
    pub fall: f32,
    pub spring: bool,
    pub spring_t: f32,
    pub rocket: bool,
}
pub struct Coin {
    pub x: f32,
    pub y: f32,
    pub got: bool,
    pub ph: f32,
}
pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub base: f32,
    pub ph: f32,
    pub dead: bool,
}
pub struct Part {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max: f32,
    pub size: f32,
    pub col: u32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum St {
    Title,
    Play,
    Over,
}

pub struct Game {
    pub st: St,
    pub px: f32,
    pub py: f32,
    pub pvx: f32,
    pub pvy: f32,
    pub face: f32,
    pub target: f32,
    pub cam: f32,
    pub maxy: f32,
    pub coins_n: u32,
    pub stomps: u32,
    pub best: u32,
    pub new_record: bool,
    pub plats: Vec<Plat>,
    pub coins: Vec<Coin>,
    pub enemies: Vec<Enemy>,
    pub parts: Vec<Part>,
    pub top_y: f32,
    pub rocket: f32,
    pub t: f32,
    pub over_t: f32,
    pub squash: f32,
    pub shake: f32,
    pub vw: f32,
    pub vh: f32,
    pub last_ms: u32,
    pub last_enemy: f32,
    rng: Rng,
    store: Store,
}

impl Game {
    pub fn new() -> Game {
        let store = Store::open("cosmojump");
        let best = store.get_i64("best").unwrap_or(0).max(0) as u32;
        let mut g = Game {
            st: St::Title, px: 136.0, py: 40.0, pvx: 0.0, pvy: 0.0, face: 0.0, target: 136.0, cam: 0.0, maxy: 40.0,
            coins_n: 0, stomps: 0, best, new_record: false, plats: vec![], coins: vec![], enemies: vec![], parts: vec![],
            top_y: 40.0, rocket: 0.0, t: 0.0, over_t: 0.0, squash: 0.0, shake: 0.0, vw: 272.0, vh: 480.0, last_ms: 0,
            last_enemy: 0.0, rng: Rng::new(), store,
        };
        g.reset();
        g
    }

    pub fn reset(&mut self) {
        self.st = St::Title;
        self.px = self.vw / 2.0;
        self.py = 40.0;
        self.pvx = 0.0;
        self.pvy = 0.0;
        self.target = self.px;
        self.cam = 0.0;
        self.maxy = 40.0;
        self.coins_n = 0;
        self.stomps = 0;
        self.new_record = false;
        self.rocket = 0.0;
        self.last_ms = 0;
        self.last_enemy = 0.0;
        self.coins.clear();
        self.enemies.clear();
        self.parts.clear();
        self.plats.clear();
        self.plats.push(Plat { x: -20.0, y: 40.0, w: self.vw + 40.0, kind: Kind::Normal, dir: 0.0, speed: 0.0, crack: -1.0, fall: 0.0, spring: false, spring_t: 0.0, rocket: false });
        self.top_y = 40.0;
    }

    pub fn start(&mut self) {
        self.st = St::Play;
        self.pvy = JUMP * 1.15;
        self.gen();
    }

    pub fn score(&self) -> u32 {
        ((self.maxy - 40.0).max(0.0) / 10.0) as u32 + self.coins_n * 10 + self.stomps * 50
    }

    pub fn burst(&mut self, x: f32, y: f32, n: u32, col: u32, sp: f32) {
        for _ in 0..n {
            let a = self.rng.f() * 6.2832;
            let v = self.rng.f() * sp;
            let life = 0.35 + self.rng.f() * 0.45;
            let size = 2.0 + self.rng.f() * 3.0;
            self.parts.push(Part { x, y, vx: a.cos() * v, vy: a.sin() * v, life, max: life, size, col });
        }
    }

    fn die(&mut self) {
        self.st = St::Over;
        self.over_t = 0.0;
        self.pvy = 320.0;
        let sc = self.score();
        if sc > self.best {
            self.best = sc;
            self.new_record = sc > 0;
            let _ = self.store.set("best", sc);
        }
        let (x, y) = (self.px, self.py + PH / 2.0);
        self.burst(x, y, 26, 0x7CFFB2, 220.0);
    }

    fn gen(&mut self) {
        let vw = self.vw;
        while self.top_y < self.cam + self.vh + 140.0 {
            let h = self.top_y;
            let d = (h / 4500.0).min(1.0);
            let y = h + 50.0 + self.rng.f() * (30.0 + 42.0 * d);
            let w = 70.0 - 24.0 * d + self.rng.f() * 10.0;
            let x = self.rng.f() * (vw - w);
            let roll = self.rng.f();
            let prev_crumble = self.plats.last().map(|p| p.kind == Kind::Crumble).unwrap_or(false);
            let p_move = 0.08 + 0.22 * d;
            let p_crumb = 0.07 + 0.2 * d;
            let kind = if roll < p_move { Kind::Moving } else if roll < p_move + p_crumb && !prev_crumble { Kind::Crumble } else { Kind::Normal };
            let normal = kind == Kind::Normal;
            let spring = normal && self.rng.f() < 0.07;
            let rocket = normal && !spring && h > 700.0 && self.rng.f() < 0.018;
            let dir = if self.rng.f() < 0.5 { 1.0 } else { -1.0 };
            let speed = 35.0 + self.rng.f() * 40.0 + 40.0 * d;
            self.plats.push(Plat { x, y, w, kind, dir, speed, crack: -1.0, fall: 0.0, spring, spring_t: 0.0, rocket });
            let r = self.rng.f();
            if r < 0.2 {
                self.coins.push(Coin { x: x + w / 2.0, y: y + 32.0, got: false, ph: 0.0 });
            } else if r < 0.27 {
                for k in 0..3 {
                    self.coins.push(Coin { x: x + w / 2.0, y: y + 30.0 + 24.0 * k as f32, got: false, ph: k as f32 });
                }
            }
            if h > 800.0 && y - self.last_enemy > 260.0 && self.rng.f() < 0.05 + 0.08 * d {
                let ex = 30.0 + self.rng.f() * (vw - 60.0);
                self.enemies.push(Enemy { x: ex, y: y + 70.0, base: ex, ph: self.rng.f() * 6.0, dead: false });
                self.last_enemy = y;
            }
            self.top_y = y;
        }
    }

    /// Pilote automatique (captures d'écran de test).
    pub fn autopilot(&mut self) {
        let (px, py) = (self.px, self.py);
        let mut best: Option<(f32, f32)> = None;
        for p in &self.plats {
            if p.crack < 0.0 && p.y > py + 5.0 && p.y < py + 125.0 {
                if best.map(|b| p.y < b.0).unwrap_or(true) {
                    best = Some((p.y, p.x + p.w / 2.0));
                }
            }
        }
        self.target = best.map(|b| b.1).unwrap_or(px);
    }
}

impl Game {
    pub fn update(&mut self, ctx: &Ctx, dt: f32) {
        let s = ctx.scale().max(0.1);
        self.vw = ctx.width() / s;
        self.vh = ctx.height() / s;
        let vw = self.vw;
        let dt = dt.min(1.0 / 30.0);
        self.t += dt;
        self.squash = (self.squash - dt * 5.0).max(0.0);
        self.shake = (self.shake - dt * 3.0).max(0.0);

        for p in &mut self.parts {
            p.life -= dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vy -= 300.0 * dt;
        }
        self.parts.retain(|p| p.life > 0.0);
        for p in &mut self.plats {
            if p.spring_t > 0.0 {
                p.spring_t -= dt;
            }
            if p.kind == Kind::Moving {
                p.x += p.dir * p.speed * dt;
                if p.x < 0.0 {
                    p.x = 0.0;
                    p.dir = 1.0;
                } else if p.x + p.w > vw {
                    p.x = vw - p.w;
                    p.dir = -1.0;
                }
            }
            if p.crack >= 0.0 {
                p.crack += dt;
                if p.crack > 0.2 {
                    p.fall += GRAV * dt;
                    p.y -= p.fall * dt;
                }
            }
        }
        for c in &mut self.coins {
            c.ph += dt * 6.0;
        }
        for e in &mut self.enemies {
            e.ph += dt;
            e.x = (e.base + (e.ph * 1.4).sin() * 45.0).clamp(22.0, vw - 22.0);
        }

        if self.st == St::Over {
            self.over_t += dt;
            self.pvy -= GRAV * dt;
            self.py += self.pvy * dt;
            ctx.animate();
            return;
        }

        // horizontal : le joueur suit le doigt
        let dx = self.target - self.px;
        self.pvx = (dx * 14.0).clamp(-420.0, 420.0);
        self.px = (self.px + self.pvx * dt).clamp(PW, vw - PW);
        let tf = (self.pvx / 200.0).clamp(-1.0, 1.0);
        self.face += (tf - self.face) * (10.0 * dt).min(1.0);

        // fusée ramassée ?
        let (cx, cy) = (self.px, self.py + PH / 2.0);
        let mut took = false;
        for p in &mut self.plats {
            if p.rocket {
                let (rx, ry) = (p.x + p.w / 2.0, p.y + 18.0);
                if (rx - cx).powi(2) + (ry - cy).powi(2) < 28.0 * 28.0 {
                    p.rocket = false;
                    took = true;
                }
            }
        }
        if took {
            self.rocket = 2.2;
            ctx.toast("Fusée !");
            let (x, y) = (self.px, self.py);
            self.burst(x, y, 18, 0xFFC93C, 200.0);
        }

        // vertical
        let prev = self.py;
        if self.rocket > 0.0 {
            self.rocket -= dt;
            self.pvy = ROCKET_V;
            self.py += self.pvy * dt;
            let (x, y) = (self.px, self.py);
            if self.rng.f() < 0.9 {
                self.parts.push(Part { x: x + (self.rng.f() - 0.5) * 8.0, y, vx: 0.0, vy: -140.0, life: 0.35, max: 0.35, size: 4.0, col: 0xFF9A3C });
            }
            if self.rocket <= 0.0 {
                self.pvy = 300.0;
            }
        } else {
            self.pvy -= GRAV * dt;
            self.py += self.pvy * dt;
            if self.pvy < 0.0 {
                let mut hit = None;
                for (i, p) in self.plats.iter().enumerate() {
                    if p.crack >= 0.0 {
                        continue;
                    }
                    if prev >= p.y - 1.0 && self.py <= p.y && self.px + PW * 0.7 > p.x && self.px - PW * 0.7 < p.x + p.w {
                        hit = Some(i);
                        break;
                    }
                }
                if let Some(i) = hit {
                    let (ty, center, spring, kind) = {
                        let p = &self.plats[i];
                        (p.y, p.x + p.w / 2.0, p.spring, p.kind)
                    };
                    let mut v = JUMP;
                    let boosted = spring && (self.px - center).abs() < 22.0;
                    if boosted {
                        v = SPRING;
                        self.plats[i].spring_t = 0.35;
                        self.shake = 0.5;
                    }
                    if kind == Kind::Crumble {
                        self.plats[i].crack = 0.0;
                    }
                    self.py = ty;
                    self.pvy = v;
                    self.squash = 1.0;
                    let x = self.px;
                    self.burst(x, ty, if boosted { 14 } else { 5 }, 0xFFFFFF, if boosted { 160.0 } else { 70.0 });
                }
            }
        }

        // pièces
        let (cx, cy) = (self.px, self.py + PH / 2.0);
        let mut got = 0;
        for c in &mut self.coins {
            if !c.got && (c.x - cx).powi(2) + (c.y - cy).powi(2) < 24.0 * 24.0 {
                c.got = true;
                got += 1;
            }
        }
        if got > 0 {
            self.coins_n += got;
            self.burst(cx, cy, 8, 0xFFD23C, 130.0);
        }

        // ennemis
        let mut hit_die = false;
        let mut killed: Vec<(f32, f32, bool)> = vec![];
        for e in &mut self.enemies {
            if e.dead {
                continue;
            }
            if (e.x - cx).powi(2) + (e.y - cy).powi(2) < 26.0 * 26.0 {
                if self.rocket > 0.0 {
                    e.dead = true;
                    killed.push((e.x, e.y, false));
                } else if self.pvy < 0.0 && self.py + PH * 0.5 > e.y {
                    e.dead = true;
                    killed.push((e.x, e.y, true));
                } else {
                    hit_die = true;
                }
            }
        }
        for (x, y, stomp) in killed {
            if stomp {
                self.stomps += 1;
                self.pvy = JUMP * 1.1;
                self.squash = 1.0;
                ctx.toast("+50");
            }
            self.burst(x, y, 16, 0xB8C4FF, 200.0);
        }

        // caméra, record, génération
        if self.py > self.cam + self.vh * 0.55 {
            self.cam = self.py - self.vh * 0.55;
        }
        if self.py > self.maxy {
            self.maxy = self.py;
        }
        let ms = ((self.maxy - 40.0).max(0.0) / 10.0) as u32 / 500;
        if ms > self.last_ms {
            self.last_ms = ms;
            ctx.toast(format!("{} m !", ms * 500));
        }
        if self.st == St::Play {
            self.gen();
        }
        let cam = self.cam;
        self.plats.retain(|p| p.y > cam - 140.0);
        self.coins.retain(|c| !c.got && c.y > cam - 60.0);
        self.enemies.retain(|e| !e.dead && e.y > cam - 80.0);

        if self.st == St::Play && (hit_die || self.py < self.cam - 30.0) {
            self.die();
        }
        if self.st == St::Title && self.py < 40.0 {
            self.py = 40.0;
            self.pvy = JUMP * 0.8;
        }
    }
}
