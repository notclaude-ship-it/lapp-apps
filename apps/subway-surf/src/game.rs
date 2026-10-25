//! Logique du jeu : course infinie sur trois voies (esquiver, sauter, rouler, ramasser).
use std::time::{SystemTime, UNIX_EPOCH};

/// Distance de la caméra (perspective).
pub const D: f32 = 6.0;
/// Distance de visibilité.
pub const FAR: f32 = 92.0;
pub const GRAV: f32 = 28.0;
pub const JUMP_V: f32 = 10.0;
pub const ROLL_T: f32 = 0.7;
pub const PH_RUN: f32 = 1.3;
pub const PH_ROLL: f32 = 0.55;
pub const JET_Y: f32 = 3.4;
pub const TRAIN_H: f32 = 2.0;
pub const V0: f32 = 11.0;
pub const VMAX: f32 = 26.0;

pub struct Rng(u64);

impl Rng {
    pub fn new() -> Self {
        let n = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
        Rng((n ^ 0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn seeded(n: u64) -> Self {
        Rng(n.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }

    /// Réel dans [0, 1[
    pub fn f(&mut self) -> f32 {
        (self.next() >> 8) as f32 / 16_777_216.0
    }

    pub fn below(&mut self, n: usize) -> usize {
        (self.next() as usize) % n.max(1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum St {
    Title,
    Play,
    Over,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// Rame de métro : infranchissable (changer de voie).
    Train,
    /// Barrière basse : sauter.
    Low,
    /// Barre en hauteur : rouler.
    Over,
    Coin,
    Jet,
    Magnet,
}

pub struct Obj {
    pub kind: Kind,
    /// Position latérale (-1, 0, 1 = voies).
    pub x: f32,
    /// Position le long de la voie (repère du monde).
    pub z: f32,
    pub len: f32,
    /// Hauteur (pièces).
    pub h: f32,
    pub color: usize,
    pub taken: bool,
}

pub struct Part {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub life: f32,
    pub max: f32,
    pub col: u32,
    pub size: f32,
}

#[derive(Default, Clone, Copy)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub roll: bool,
}

pub struct Game {
    pub rng: Rng,
    pub st: St,
    pub st_t: f32,
    pub t: f32,
    pub objs: Vec<Obj>,
    pub parts: Vec<Part>,
    pub next_z: f32,
    pub dist: f32,
    pub speed: f32,
    pub play_t: f32,
    pub lane: i32,
    pub px: f32,
    pub py: f32,
    pub vy: f32,
    pub roll_t: f32,
    pub run: f32,
    pub jet_t: f32,
    pub mag_t: f32,
    pub inv_t: f32,
    pub guard_t: f32,
    pub flash: f32,
    pub shake: f32,
    pub coins: u32,
    pub best: u32,
    pub new_best: bool,
    pub caught: bool,
}

impl Game {
    pub fn new() -> Game {
        Game {
            rng: Rng::new(),
            st: St::Title,
            st_t: 0.0,
            t: 0.0,
            objs: Vec::new(),
            parts: Vec::new(),
            next_z: 40.0,
            dist: 0.0,
            speed: 8.0,
            play_t: 0.0,
            lane: 1,
            px: 0.0,
            py: 0.0,
            vy: 0.0,
            roll_t: 0.0,
            run: 0.0,
            jet_t: 0.0,
            mag_t: 0.0,
            inv_t: 0.0,
            guard_t: 0.0,
            flash: 0.0,
            shake: 0.0,
            coins: 0,
            best: 0,
            new_best: false,
            caught: false,
        }
    }

    pub fn start(&mut self) {
        let best = self.best;
        let rng = Rng(self.rng.0 | 1);
        *self = Game::new();
        self.rng = rng;
        self.best = best;
        self.st = St::Play;
        self.speed = V0;
        self.next_z = 45.0;
    }

    pub fn score(&self) -> u32 {
        self.dist as u32 + self.coins * 5
    }

    pub fn airborne(&self) -> bool {
        self.py > 0.08 && self.jet_t <= 0.0
    }

    pub fn update(&mut self, dt: f32, inp: Input) {
        self.t += dt;
        self.st_t += dt;
        self.flash = (self.flash - dt).max(0.0);
        self.shake = (self.shake - dt * 2.2).max(0.0);
        self.update_parts(dt);
        match self.st {
            St::Title => {
                self.dist += 6.0 * dt;
                self.run += dt * 11.0;
                return;
            }
            St::Over => return,
            St::Play => {}
        }

        self.play_t += dt;
        let target = (V0 + self.play_t * 0.17).min(VMAX);
        self.speed += (target - self.speed) * (dt * 1.2).min(1.0);
        self.dist += self.speed * dt;
        self.run += dt * (7.0 + self.speed * 0.5);
        self.jet_t = (self.jet_t - dt).max(0.0);
        self.mag_t = (self.mag_t - dt).max(0.0);
        self.inv_t = (self.inv_t - dt).max(0.0);
        self.guard_t = (self.guard_t - dt).max(0.0);
        self.roll_t = (self.roll_t - dt).max(0.0);

        if inp.left {
            self.try_lane(-1);
        }
        if inp.right {
            self.try_lane(1);
        }
        if inp.jump && self.py < 0.05 && self.jet_t <= 0.0 {
            self.vy = JUMP_V;
            self.roll_t = 0.0;
        }
        if inp.roll {
            self.roll_t = ROLL_T;
            if self.airborne() {
                self.vy = -20.0;
            }
        }

        let tx = (self.lane - 1) as f32;
        self.px += (tx - self.px) * (1.0 - (-dt * 15.0).exp());

        if self.jet_t > 0.0 {
            self.py += (JET_Y - self.py) * (1.0 - (-dt * 4.0).exp());
            self.vy = 0.0;
            if self.jet_t <= dt {
                self.inv_t = 2.2;
            }
        } else {
            self.vy -= GRAV * dt;
            self.py += self.vy * dt;
            if self.py <= 0.0 {
                self.py = 0.0;
                self.vy = 0.0;
            }
        }

        while self.next_z < self.dist + FAR {
            self.spawn_row();
        }

        // aimant : les pièces proches viennent vers le joueur
        if self.mag_t > 0.0 {
            let a = 1.0 - (-dt * 7.0).exp();
            let (dist, px, ty) = (self.dist, self.px, self.py + 0.7);
            for o in self.objs.iter_mut().filter(|o| o.kind == Kind::Coin && !o.taken) {
                let rel = o.z - dist;
                if rel > -1.0 && rel < 11.0 {
                    o.x += (px - o.x) * a;
                    o.h += (ty - o.h) * a;
                }
            }
        }

        self.collide();
        let d = self.dist;
        self.objs.retain(|o| o.z + o.len - d > -8.0);
    }

    fn blocked(&self, x: f32) -> bool {
        self.objs.iter().any(|o| {
            o.kind == Kind::Train && (o.x - x).abs() < 0.5 && {
                let rel = o.z - self.dist;
                rel < 0.4 && rel + o.len > -0.5
            }
        })
    }

    fn try_lane(&mut self, d: i32) {
        let nl = self.lane + d;
        if !(0..=2).contains(&nl) {
            return;
        }
        if self.jet_t <= 0.0 && self.inv_t <= 0.0 && self.py < TRAIN_H - 0.15 && self.blocked((nl - 1) as f32) {
            // choc latéral contre une rame : on reste sur la voie
            self.stumble();
            return;
        }
        self.lane = nl;
    }

    fn stumble(&mut self) {
        if self.st != St::Play {
            return;
        }
        if self.guard_t > 0.0 {
            self.die();
            return;
        }
        self.guard_t = 5.0;
        self.inv_t = self.inv_t.max(0.7);
        self.flash = 0.35;
        self.shake = 0.7;
        self.speed *= 0.65;
        let (x, y, z) = (self.px, self.py + 0.6, self.dist);
        self.burst(x, y, z, 10, 0xFFFFFF, 0.9);
    }

    pub fn die(&mut self) {
        if self.st != St::Play {
            return;
        }
        self.caught = self.guard_t > 0.0;
        self.st = St::Over;
        self.st_t = 0.0;
        self.flash = 0.5;
        self.shake = 1.0;
        let sc = self.score();
        self.new_best = sc > self.best;
        if self.new_best {
            self.best = sc;
        }
        let (x, y, z) = (self.px, self.py + 0.6, self.dist);
        self.burst(x, y, z, 26, 0xFF8A3C, 1.6);
    }

    fn burst(&mut self, x: f32, y: f32, z: f32, n: usize, col: u32, power: f32) {
        for _ in 0..n {
            let a = self.rng.f() * std::f32::consts::TAU;
            let v = (0.4 + self.rng.f()) * 3.0 * power;
            self.parts.push(Part {
                x,
                y,
                z,
                vx: a.cos() * v,
                vy: 2.0 + self.rng.f() * 4.0 * power,
                vz: a.sin() * v * 0.6,
                life: 0.7,
                max: 0.7,
                col,
                size: 0.05 + self.rng.f() * 0.06,
            });
        }
    }

    fn update_parts(&mut self, dt: f32) {
        for q in self.parts.iter_mut() {
            q.life -= dt;
            q.x += q.vx * dt;
            q.y += q.vy * dt;
            q.z += q.vz * dt;
            q.vy -= 14.0 * dt;
            if q.y < 0.0 {
                q.y = 0.0;
                q.vy = -q.vy * 0.4;
            }
        }
        self.parts.retain(|q| q.life > 0.0);
    }

    fn collide(&mut self) {
        let safe = self.jet_t > 0.0 || self.inv_t > 0.0;
        let ph = if self.roll_t > 0.0 { PH_ROLL } else { PH_RUN };
        let (dist, px, py) = (self.dist, self.px, self.py);
        let mut crash = false;
        let mut hit = false;
        let mut gained = 0;
        let mut jet = false;
        let mut mag = false;
        let mut sparks: Vec<(f32, f32, f32)> = Vec::new();
        for o in self.objs.iter_mut() {
            if o.taken {
                continue;
            }
            let rel = o.z - dist;
            let dx = (o.x - px).abs();
            match o.kind {
                Kind::Coin => {
                    if rel > -0.8 && rel < 0.8 && dx < 0.55 && o.h > py - 0.25 && o.h < py + ph + 0.25 {
                        o.taken = true;
                        gained += 1;
                        if sparks.len() < 4 {
                            sparks.push((o.x, o.h, o.z));
                        }
                    }
                }
                Kind::Jet | Kind::Magnet => {
                    if rel > -1.0 && rel < 1.0 && dx < 0.65 && py < 2.2 {
                        o.taken = true;
                        if o.kind == Kind::Jet {
                            jet = true;
                        } else {
                            mag = true;
                        }
                    }
                }
                Kind::Train => {
                    if !safe && py < TRAIN_H - 0.15 && dx < 0.6 && rel < 0.45 && rel + o.len > -0.45 {
                        crash = true;
                    }
                }
                Kind::Low => {
                    if !safe && rel > -0.3 && rel < 0.5 && dx < 0.5 && py < 0.75 {
                        o.taken = true;
                        hit = true;
                    }
                }
                Kind::Over => {
                    if !safe && rel > -0.3 && rel < 0.45 && dx < 0.5 && py + ph > 0.95 && py < 1.55 {
                        o.taken = true;
                        hit = true;
                    }
                }
            }
        }
        self.coins += gained;
        for (x, y, z) in sparks {
            self.burst(x, y, z, 5, 0xFFD23C, 0.4);
        }
        if mag {
            self.mag_t = 10.0;
        }
        if jet {
            self.jet_t = 7.5;
            self.sky_coins();
        }
        if crash {
            self.die();
        } else if hit {
            self.stumble();
        }
    }

    /// Une longue traînée de pièces en altitude pour la durée du jetpack.
    fn sky_coins(&mut self) {
        let mut lane = self.rng.below(3);
        for i in 0..52 {
            if i % 13 == 0 {
                lane = self.rng.below(3);
            }
            self.objs.push(Obj {
                kind: Kind::Coin,
                x: lane as f32 - 1.0,
                z: self.dist + 16.0 + i as f32 * 2.2,
                len: 0.0,
                h: JET_Y + 0.5,
                color: 0,
                taken: false,
            });
        }
    }

    fn coin(&mut self, x: f32, z: f32, h: f32) {
        self.objs.push(Obj { kind: Kind::Coin, x, z, len: 0.0, h, color: 0, taken: false });
    }

    fn obj(&mut self, kind: Kind, x: f32, z: f32, len: f32, color: usize) {
        self.objs.push(Obj { kind, x, z, len, h: 0.0, color, taken: false });
    }

    fn spawn_row(&mut self) {
        let z = self.next_z;
        let diff = ((self.speed - V0) / (VMAX - V0)).clamp(0.0, 1.0);
        let free = self.rng.below(3);
        let mut others: Vec<usize> = (0..3).filter(|&l| l != free).collect();
        if self.rng.below(2) == 0 {
            others.swap(0, 1);
        }
        let lx = |l: usize| l as f32 - 1.0;
        let r = self.rng.below(100);
        let used;
        match r {
            0..=17 => {
                let l = self.rng.below(3);
                let n = 8 + self.rng.below(5);
                for i in 0..n {
                    self.coin(lx(l), z + i as f32 * 1.7, 0.75);
                }
                used = n as f32 * 1.7;
            }
            18..=37 => {
                let nb = if diff > 0.25 && self.rng.below(2) == 0 { 2 } else { 1 };
                for &l in others.iter().take(nb) {
                    self.obj(Kind::Low, lx(l), z, 0.3, 0);
                }
                if nb == 1 {
                    let l = others[0];
                    for i in 0..7 {
                        let u = (i as f32 - 3.0) / 3.0;
                        self.coin(lx(l), z - 3.0 + i as f32, 0.8 + 1.0 * (1.0 - u * u));
                    }
                }
                for i in 0..6 {
                    self.coin(lx(free), z - 2.0 + i as f32 * 1.7, 0.75);
                }
                used = 6.0;
            }
            38..=57 => {
                let nt = if diff > 0.3 && self.rng.below(3) == 0 { 2 } else { 1 };
                let len = 7.0 + self.rng.f() * 7.0;
                let col = self.rng.below(4);
                for &l in others.iter().take(nt) {
                    self.obj(Kind::Train, lx(l), z, len, col);
                }
                let n = (len / 1.7) as usize;
                for i in 0..n {
                    self.coin(lx(free), z + i as f32 * 1.7, 0.75);
                }
                used = len;
            }
            58..=72 => {
                let nb = if diff > 0.25 && self.rng.below(2) == 0 { 2 } else { 1 };
                for &l in others.iter().take(nb) {
                    self.obj(Kind::Over, lx(l), z, 0.2, 0);
                }
                for i in 0..6 {
                    self.coin(lx(free), z - 2.0 + i as f32 * 1.7, 0.75);
                }
                used = 6.0;
            }
            73..=88 => {
                let len = 8.0 + self.rng.f() * 5.0;
                let col = self.rng.below(4);
                self.obj(Kind::Train, lx(others[0]), z, len, col);
                if self.rng.below(2) == 0 {
                    self.obj(Kind::Low, lx(others[1]), z + 1.0, 0.3, 0);
                } else {
                    self.obj(Kind::Over, lx(others[1]), z + 1.0, 0.2, 0);
                }
                let n = (len / 1.7) as usize;
                for i in 0..n {
                    self.coin(lx(free), z + i as f32 * 1.7, 0.75);
                }
                used = len;
            }
            89..=95 => {
                let kind = if self.rng.below(2) == 0 { Kind::Jet } else { Kind::Magnet };
                let l = self.rng.below(3);
                self.objs.push(Obj { kind, x: lx(l), z, len: 0.0, h: 0.9, color: 0, taken: false });
                let o = (l + 1 + self.rng.below(2)) % 3;
                for i in 0..8 {
                    self.coin(lx(o), z + i as f32 * 1.7, 0.75);
                }
                used = 8.0 * 1.7;
            }
            _ => {
                let l = self.rng.below(3);
                for i in 0..10 {
                    self.coin(lx(l), z + i as f32 * 1.7, 0.75);
                }
                used = 17.0;
            }
        }
        let gap = (self.speed * 0.95).max(13.0) + self.rng.f() * 8.0 * (1.0 - diff * 0.5);
        self.next_z = z + used + gap;
    }

    /// Pilote automatique (captures d'écran, simulation).
    pub fn autopilot(&mut self) -> Input {
        let mut inp = Input::default();
        if self.st != St::Play {
            return inp;
        }
        let look = (self.speed * 0.9).max(12.0);
        let mut danger = [0.0f32; 3];
        let mut low = [f32::INFINITY; 3];
        let mut over = [f32::INFINITY; 3];
        for o in &self.objs {
            if o.taken {
                continue;
            }
            let rel = o.z - self.dist;
            let l = (o.x.round() as i32 + 1).clamp(0, 2) as usize;
            match o.kind {
                Kind::Train => {
                    if rel < look * 1.8 && rel + o.len > -0.5 {
                        danger[l] += 1000.0;
                    }
                }
                Kind::Low => {
                    if rel > -0.5 && rel < look {
                        low[l] = low[l].min(rel);
                        danger[l] += 40.0;
                    }
                }
                Kind::Over => {
                    if rel > -0.5 && rel < look {
                        over[l] = over[l].min(rel);
                        danger[l] += 40.0;
                    }
                }
                Kind::Coin => {
                    if rel > 0.0 && rel < 30.0 && o.h < 2.0 {
                        danger[l] -= 1.0;
                    }
                }
                Kind::Jet | Kind::Magnet => {
                    if rel > 0.0 && rel < 30.0 {
                        danger[l] -= 8.0;
                    }
                }
            }
        }
        let mut best = self.lane as usize;
        let mut bs = f32::INFINITY;
        for l in 0..3 {
            let s = danger[l] + (l as i32 - self.lane).abs() as f32 * 6.0;
            if s < bs {
                bs = s;
                best = l;
            }
        }
        if best as i32 > self.lane {
            inp.right = true;
        } else if (best as i32) < self.lane {
            inp.left = true;
        }
        let cl = (if inp.right { self.lane + 1 } else if inp.left { self.lane - 1 } else { self.lane }) as usize;
        if low[cl] > 0.0 && low[cl] < self.speed * 0.3 {
            inp.jump = true;
        }
        if over[cl] > 0.0 && over[cl] < self.speed * 0.25 {
            inp.roll = true;
        }
        inp
    }
}

/// Simulation sans affichage : (secondes tenues, score, pièces, a perdu).
pub fn sim(seed: u64, max_s: f32) -> (f32, u32, u32, bool) {
    let mut g = Game::new();
    g.rng = Rng::seeded(seed);
    g.start();
    let dt = 1.0 / 60.0;
    let mut t = 0.0;
    while t < max_s && g.st == St::Play {
        let inp = g.autopilot();
        g.update(dt, inp);
        t += dt;
    }
    (t, g.score(), g.coins, g.st == St::Over)
}