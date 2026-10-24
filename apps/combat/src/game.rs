//! Logique du jeu de combat : combattants, IA, manches, particules.
//! Unités : points (l'écran de l'appareil fait 272 x 480 points). `h` = hauteur au-dessus du sol.

pub const MIN_X: f32 = 24.0;
pub const ROUND_TIME: f32 = 45.0;
const WALK: f32 = 92.0;
const WALK_BACK: f32 = 70.0;
const JUMP_V: f32 = 400.0;
const GRAV: f32 = 1150.0;
const SPACING: f32 = 26.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Act {
    Idle,
    Walk,
    Punch,
    Kick,
    Block,
    Hurt,
    Ko,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Input {
    pub dir: f32,
    pub block: bool,
    pub jump: bool,
    pub punch: bool,
    pub kick: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Fighter {
    pub x: f32,
    pub h: f32,
    pub vy: f32,
    pub airvx: f32,
    pub face: f32,
    pub hp: f32,
    pub act: Act,
    pub t: f32,
    pub walk: f32,
    pub kb: f32,
    pub flash: f32,
    hit_done: bool,
}

impl Fighter {
    pub fn new(x: f32, face: f32) -> Fighter {
        Fighter { x, h: 0.0, vy: 0.0, airvx: 0.0, face, hp: 100.0, act: Act::Idle, t: 0.0, walk: 0.0, kb: 0.0, flash: 0.0, hit_done: false }
    }
    pub fn grounded(&self) -> bool {
        self.h <= 0.0 && self.vy <= 0.0
    }
    pub fn attacking(&self) -> bool {
        matches!(self.act, Act::Punch | Act::Kick)
    }
    fn free(&self) -> bool {
        matches!(self.act, Act::Idle | Act::Walk | Act::Block)
    }
    /// Fenêtre active de l'attaque en cours (début, fin) en secondes.
    fn window(&self) -> (f32, f32) {
        match self.act {
            Act::Punch => (0.07, 0.16),
            _ => (0.16, 0.30),
        }
    }
    fn duration(&self) -> f32 {
        match self.act {
            Act::Punch => 0.30,
            _ => 0.52,
        }
    }
    /// Progression 0..1 de l'extension du membre (pour le dessin).
    pub fn strike(&self) -> f32 {
        if !self.attacking() {
            return 0.0;
        }
        let (a, b) = self.window();
        if self.t < a {
            -(1.0 - self.t / a) * 0.5
        } else if self.t < b {
            1.0
        } else {
            (1.0 - (self.t - b) / (self.duration() - b)).max(0.0)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HitFx {
    pub x: f32,
    pub h: f32,
    pub blocked: bool,
    pub heavy: bool,
    pub dir: f32,
}

pub struct Particle {
    pub x: f32,
    pub h: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max: f32,
    pub col: u32,
    pub size: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum St {
    Title,
    Intro,
    Fight,
    RoundEnd,
    MatchEnd,
}

struct Rng(u32);
impl Rng {
    fn f(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 >> 8) as f32 / 16_777_216.0
    }
}

pub struct Ai {
    pub lvl: u32,
    tmr: f32,
    dir: f32,
    block_t: f32,
    seen: bool,
    rng: Rng,
}

impl Ai {
    pub fn new(lvl: u32) -> Ai {
        Ai { lvl, tmr: 0.4, dir: 0.0, block_t: 0.0, seen: false, rng: Rng(0x9E37_79B9 ^ lvl.wrapping_mul(2654435761)) }
    }

    pub fn think(&mut self, me: &Fighter, foe: &Fighter, dt: f32) -> Input {
        let mut inp = Input::default();
        if foe.act == Act::Ko || me.act == Act::Ko {
            return inp;
        }
        let lv = self.lvl.min(12) as f32;
        let dist = (foe.x - me.x).abs();
        let toward = if foe.x >= me.x { 1.0 } else { -1.0 };

        // réaction à une attaque adverse
        if foe.attacking() && dist < 78.0 {
            if !self.seen {
                self.seen = true;
                if self.rng.f() < 0.22 + 0.06 * lv {
                    self.block_t = 0.34;
                }
            }
        } else {
            self.seen = false;
        }
        self.block_t = (self.block_t - dt).max(0.0);
        inp.block = self.block_t > 0.0;

        self.tmr -= dt;
        if self.tmr <= 0.0 {
            self.tmr = (0.34 - 0.022 * lv).max(0.09) + self.rng.f() * 0.14;
            let r = self.rng.f();
            self.dir = 0.0;
            if dist > 64.0 {
                self.dir = if r < 0.88 { toward } else { 0.0 };
            } else {
                let atk = 0.42 + 0.04 * lv;
                if r < atk {
                    if dist > 46.0 || self.rng.f() < 0.35 {
                        inp.kick = true;
                    } else {
                        inp.punch = true;
                    }
                } else if r < atk + 0.22 {
                    self.dir = -toward;
                } else if r < atk + 0.27 {
                    inp.jump = true;
                    self.dir = toward;
                } else if r < atk + 0.40 {
                    self.block_t = 0.25;
                }
            }
        }
        if dist < 40.0 && self.dir == toward {
            self.dir = 0.0;
        }
        inp.dir = self.dir;
        inp
    }
}

pub struct Game {
    pub st: St,
    pub t: f32,
    pub st_t: f32,
    pub vw: f32,
    pub p: [Fighter; 2],
    pub ai: Ai,
    pub wins: [u32; 2],
    pub round: u32,
    pub timer: f32,
    pub level: u32,
    pub best: u32,
    pub parts: Vec<Particle>,
    pub shake: f32,
    pub hitstop: f32,
    pub flash: f32,
    pub round_winner: i32, // 0 joueur, 1 IA, -1 égalité
    pub combo: u32,
    pub combo_t: f32,
    pub demo_ai: Option<Ai>,
    rng: Rng,
    acc: f32,
    pe: Input,
}

impl Game {
    pub fn new() -> Game {
        let vw = 272.0;
        Game {
            st: St::Title,
            t: 0.0,
            st_t: 0.0,
            vw,
            p: [Fighter::new(vw * 0.3, 1.0), Fighter::new(vw * 0.7, -1.0)],
            ai: Ai::new(1),
            wins: [0, 0],
            round: 1,
            timer: ROUND_TIME,
            level: 1,
            best: 1,
            parts: Vec::new(),
            shake: 0.0,
            hitstop: 0.0,
            flash: 0.0,
            round_winner: -1,
            combo: 0,
            combo_t: 0.0,
            demo_ai: None,
            rng: Rng(0xC0FF_EE11),
            acc: 0.0,
            pe: Input::default(),
        }
    }

    pub fn max_x(&self) -> f32 {
        self.vw - MIN_X
    }

    pub fn start_match(&mut self) {
        self.wins = [0, 0];
        self.round = 1;
        self.ai = Ai::new(self.level);
        self.start_round();
    }

    pub fn start_round(&mut self) {
        self.p = [Fighter::new(self.vw * 0.3, 1.0), Fighter::new(self.vw * 0.7, -1.0)];
        self.timer = ROUND_TIME;
        self.parts.clear();
        self.combo = 0;
        self.set(St::Intro);
    }

    fn set(&mut self, st: St) {
        self.st = st;
        self.st_t = 0.0;
    }

    fn burst(&mut self, x: f32, h: f32, n: usize, cols: &[u32], speed: f32, dir: f32) {
        for i in 0..n {
            let a = self.rng.f() * std::f32::consts::TAU;
            let sp = speed * (0.3 + self.rng.f());
            self.parts.push(Particle {
                x,
                h,
                vx: a.cos() * sp + dir * speed * 0.5,
                vy: a.sin().abs() * sp * 0.9,
                life: 0.3 + self.rng.f() * 0.3,
                max: 0.6,
                col: cols[i % cols.len()],
                size: 1.5 + self.rng.f() * 2.5,
            });
        }
    }

    pub fn update(&mut self, dt: f32, in0: Input) {
        // pas fixe 1/120 s ; les appuis ponctuels attendent le prochain pas
        self.acc += dt.min(0.05);
        self.pe.jump |= in0.jump;
        self.pe.punch |= in0.punch;
        self.pe.kick |= in0.kick;
        while self.acc >= 1.0 / 120.0 {
            self.acc -= 1.0 / 120.0;
            let inp = Input { dir: in0.dir, block: in0.block, jump: self.pe.jump, punch: self.pe.punch, kick: self.pe.kick };
            self.pe = Input::default();
            self.step(1.0 / 120.0, inp);
        }
    }

    fn step(&mut self, dt: f32, in0: Input) {
        self.t += dt;
        self.st_t += dt;
        self.flash = (self.flash - dt * 4.0).max(0.0);
        self.shake *= (1.0 - dt * 9.0).max(0.0);
        if self.shake < 0.1 {
            self.shake = 0.0;
        }
        self.combo_t -= dt;
        if self.combo_t <= 0.0 {
            self.combo = 0;
        }
        for q in &mut self.parts {
            q.x += q.vx * dt;
            q.h += q.vy * dt;
            q.vy -= 600.0 * dt;
            q.life -= dt;
            if q.h < 0.0 {
                q.h = 0.0;
                q.vy = 0.0;
                q.vx *= 0.8;
            }
        }
        self.parts.retain(|q| q.life > 0.0);

        match self.st {
            St::Title | St::MatchEnd => {
                // les combattants restent figés : on ne simule rien
                return;
            }
            St::Intro => {
                for f in &mut self.p {
                    f.flash = 0.0;
                }
                if self.st_t > 1.5 {
                    self.set(St::Fight);
                }
                return;
            }
            _ => {}
        }

        if self.hitstop > 0.0 {
            self.hitstop -= dt;
            return;
        }

        let fighting = self.st == St::Fight;
        let (i0, i1) = if fighting {
            let i1 = self.ai.think(&self.p[1], &self.p[0], dt);
            let i0 = match self.demo_ai.as_mut() {
                Some(a) => a.think(&self.p[0], &self.p[1], dt),
                None => in0,
            };
            (i0, i1)
        } else {
            (Input::default(), Input::default())
        };

        let mut fx: Vec<HitFx> = Vec::new();
        for k in 0..2 {
            let foe = self.p[1 - k];
            let inp = if k == 0 { i0 } else { i1 };
            let maxx = self.max_x();
            step_fighter(&mut self.p[k], &foe, inp, dt, maxx);
        }
        // résolution des coups
        for k in 0..2 {
            let foe = self.p[1 - k];
            let me = self.p[k];
            if let Some(res) = resolve_hit(&me, &foe) {
                let foe_m = &mut self.p[1 - k];
                let dmg_scale = if k == 1 { 0.75 + 0.05 * self.ai.lvl.min(8) as f32 } else { 1.0 };
                let (dmg, heavy) = res.dmg;
                let blocked = foe_m.act == Act::Block && foe_m.grounded() && foe_m.face == -me.face;
                if blocked {
                    foe_m.hp -= dmg * 0.08 * dmg_scale;
                    foe_m.kb = me.face * 120.0;
                    foe_m.flash = 0.0;
                } else {
                    foe_m.hp -= dmg * dmg_scale;
                    foe_m.act = Act::Hurt;
                    foe_m.t = 0.0;
                    foe_m.hit_done = true;
                    foe_m.kb = me.face * if heavy { 190.0 } else { 100.0 };
                    foe_m.flash = 0.15;
                    if foe_m.h > 0.0 {
                        foe_m.vy = foe_m.vy.max(120.0);
                    }
                }
                if foe_m.hp <= 0.0 {
                    foe_m.hp = 0.0;
                    foe_m.act = Act::Ko;
                    foe_m.t = 0.0;
                    foe_m.kb = me.face * 230.0;
                    foe_m.vy = 260.0;
                }
                self.p[k].hit_done = true;
                fx.push(HitFx { x: foe.x - me.face * 10.0, h: foe.h + 38.0, blocked, heavy, dir: me.face });
                if k == 0 && !blocked {
                    self.combo += 1;
                    self.combo_t = 1.2;
                }
            }
        }
        for e in fx {
            if e.blocked {
                self.burst(e.x, e.h, 5, &[0x7FC8FF, 0xFFFFFF], 90.0, e.dir);
                self.hitstop = self.hitstop.max(0.03);
                self.shake = self.shake.max(1.5);
            } else {
                self.burst(e.x, e.h, if e.heavy { 16 } else { 10 }, &[0xFFE066, 0xFFFFFF, 0xFF8A3C], if e.heavy { 190.0 } else { 140.0 }, e.dir);
                self.hitstop = self.hitstop.max(if e.heavy { 0.09 } else { 0.055 });
                self.shake = self.shake.max(if e.heavy { 7.0 } else { 3.5 });
                self.flash = self.flash.max(if e.heavy { 0.25 } else { 0.0 });
            }
        }

        // séparation des combattants
        let dx = self.p[1].x - self.p[0].x;
        let dh = (self.p[1].h - self.p[0].h).abs();
        if dx.abs() < SPACING && dh < 40.0 && self.p[0].act != Act::Ko && self.p[1].act != Act::Ko {
            let push = (SPACING - dx.abs()) / 2.0;
            let s = if dx >= 0.0 { 1.0 } else { -1.0 };
            let (mn, mx) = (MIN_X, self.max_x());
            self.p[0].x = (self.p[0].x - s * push).clamp(mn, mx);
            self.p[1].x = (self.p[1].x + s * push).clamp(mn, mx);
        }

        if fighting {
            self.timer -= dt;
            let (a, b) = (self.p[0].hp, self.p[1].hp);
            let ko0 = self.p[0].act == Act::Ko;
            let ko1 = self.p[1].act == Act::Ko;
            if ko0 || ko1 || self.timer <= 0.0 {
                self.timer = self.timer.max(0.0);
                self.round_winner = if ko0 && ko1 {
                    -1
                } else if ko1 {
                    0
                } else if ko0 {
                    1
                } else if (a - b).abs() < 0.5 {
                    -1
                } else if a > b {
                    0
                } else {
                    1
                };
                match self.round_winner {
                    0 => self.wins[0] += 1,
                    1 => self.wins[1] += 1,
                    _ => {
                        self.wins[0] += 1;
                        self.wins[1] += 1;
                    }
                }
                if ko0 || ko1 {
                    self.hitstop = 0.0;
                    self.flash = 0.35;
                    self.shake = 9.0;
                }
                self.set(St::RoundEnd);
            }
        } else if self.st == St::RoundEnd && self.st_t > 2.4 {
            if self.wins[0] >= 2 || self.wins[1] >= 2 {
                let won = self.wins[0] >= 2 && self.wins[0] >= self.wins[1];
                if won {
                    self.best = self.best.max(self.level + 1);
                }
                self.set(St::MatchEnd);
            } else {
                self.round += 1;
                self.start_round();
            }
        }
    }

    pub fn player_won(&self) -> bool {
        self.wins[0] >= 2 && self.wins[0] >= self.wins[1]
    }
}

struct Hit {
    dmg: (f32, bool),
}

fn resolve_hit(me: &Fighter, foe: &Fighter) -> Option<Hit> {
    if !me.attacking() || me.hit_done || foe.act == Act::Ko {
        return None;
    }
    let (a, b) = me.window();
    if me.t < a || me.t > b {
        return None;
    }
    let (range, dmg, heavy) = if me.act == Act::Punch { (46.0, 7.0, false) } else { (64.0, 12.0, true) };
    let dist = (foe.x - me.x) * me.face;
    if dist < -6.0 || dist > range {
        return None;
    }
    if (me.h - foe.h).abs() > 42.0 {
        return None;
    }
    // une garde levée à la bonne hauteur encaisse ; un adversaire accroupi en l'air est touché normalement
    Some(Hit { dmg: (dmg, heavy) })
}

fn step_fighter(f: &mut Fighter, foe: &Fighter, inp: Input, dt: f32, maxx: f32) {
    f.flash = (f.flash - dt).max(0.0);
    f.t += dt;

    // physique verticale
    if f.h > 0.0 || f.vy > 0.0 {
        f.h += f.vy * dt;
        f.vy -= GRAV * dt;
        if f.h <= 0.0 {
            f.h = 0.0;
            f.vy = 0.0;
            f.airvx = 0.0;
        }
    }

    match f.act {
        Act::Ko => {
            f.x += f.kb * dt;
            f.kb *= (1.0 - 3.0 * dt).max(0.0);
        }
        Act::Hurt => {
            f.x += f.kb * dt;
            f.kb *= (1.0 - 6.0 * dt).max(0.0);
            if f.t >= 0.30 {
                f.act = Act::Idle;
                f.t = 0.0;
            }
        }
        Act::Punch | Act::Kick => {
            if f.t >= f.duration() {
                f.act = Act::Idle;
                f.t = 0.0;
            }
            f.x += f.airvx * dt;
        }
        _ => {}
    }
    if !matches!(f.act, Act::Ko | Act::Hurt) {
        f.x += f.kb * dt;
        f.kb *= (1.0 - 8.0 * dt).max(0.0);
    }

    if f.free() {
        if f.grounded() {
            f.face = if foe.x >= f.x { 1.0 } else { -1.0 };
        }
        let grounded = f.grounded();
        if inp.jump && grounded {
            f.vy = JUMP_V;
            f.airvx = inp.dir * WALK;
            f.h = 0.01;
            f.act = Act::Idle;
        }
        if inp.punch {
            f.act = Act::Punch;
            f.t = 0.0;
            f.hit_done = false;
        } else if inp.kick {
            f.act = Act::Kick;
            f.t = 0.0;
            f.hit_done = false;
        } else if grounded {
            if inp.block {
                if f.act != Act::Block {
                    f.t = 0.0;
                }
                f.act = Act::Block;
            } else if inp.dir != 0.0 {
                let back = inp.dir != f.face;
                let sp = if back { WALK_BACK } else { WALK };
                f.x += inp.dir * sp * dt;
                f.walk += dt * 9.0 * if back { -1.0 } else { 1.0 };
                f.act = Act::Walk;
            } else {
                f.act = Act::Idle;
            }
        } else {
            f.x += f.airvx * dt;
            f.act = Act::Idle;
        }
    }
    f.x = f.x.clamp(MIN_X, maxx);
}
