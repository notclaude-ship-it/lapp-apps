// Combat : jeu de combat 1 joueur contre IA pour lapp (écran portrait 272x480).
//
// Commandes tactiles : ‹ › pour bouger (maintenir), SAUT, GARDE (maintenir), POING, PIED.
// Clavier (PC) : flèches / Espace saut / Bas ou C garde / Z poing / X pied.
// Quitter : glisser vers le haut depuis le bord bas de l'écran, ou Echap.
// Sur PC : cargo run -- --screenshot out.png [--demo 300]   (rendu hors écran ; --demo = N images jouées par une IA)
mod draw;
mod game;

use draw::*;
use game::*;
use lapp_sdk::prelude::*;
use lapp_sdk::{arg_value, Store};

struct Combat {
    g: Game,
    store: Store,
    held: Option<Ctl>,
    pend: Input,
    key_dir: f32,
    key_dir_t: f32,
    key_block_t: f32,
    demo: Option<u32>,
}

impl Combat {
    fn set_held(&mut self, c: Option<Ctl>) {
        if c == self.held {
            return;
        }
        self.held = c;
        match c {
            Some(Ctl::Jump) => self.pend.jump = true,
            Some(Ctl::Punch) => self.pend.punch = true,
            Some(Ctl::Kick) => self.pend.kick = true,
            _ => {}
        }
    }

    fn begin(&mut self) {
        match self.g.st {
            St::Title => self.g.start_match(),
            St::MatchEnd => {
                if self.g.player_won() {
                    self.g.level += 1;
                    let _ = self.store.set("best", self.g.best);
                }
                self.g.start_match();
            }
            _ => {}
        }
    }
}

impl App for Combat {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let l = layout(ctx);
        let gm = &self.g;
        let sh = gm.shake * l.s;
        let (ox, oy) = if sh > 0.0 {
            (((gm.t * 91.0).sin()) * sh, ((gm.t * 73.0).cos()) * sh * 0.6)
        } else {
            (0.0, 0.0)
        };
        g.clear(rgb(0x1A0F3C));
        background(gm, g, &l, ox, oy);
        // le combattant qui frappe passe devant
        let order = if gm.p[0].attacking() || gm.p[0].act == Act::Ko { [1, 0] } else { [0, 1] };
        for &i in &order {
            let (gi, trim) = if i == 0 { (0x3D8BFF, 0xFFD23C) } else { (0xD23B3B, 0x1B1B1B) };
            fighter(g, &l, &gm.p[i], gm.t, gi, trim, ox, oy, false);
        }
        particles(gm, g, &l, ox, oy);
        if gm.st != St::Title {
            hud(gm, ctx, g, &l);
        }
        pad(ctx, g, &l, if gm.st == St::Fight { self.held } else { None });
        overlay(gm, ctx, g, &l, self.store.get_i64("best").unwrap_or(1).max(1) as u32);
    }

    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        let l = layout(ctx);
        self.g.vw = l.vw;
        if let Some(n) = self.demo.take() {
            // mode capture : l'IA joue aussi le joueur
            self.g.demo_ai = Some(Ai::new(self.g.level + 2));
            self.g.start_match();
            for _ in 0..n {
                self.g.update(1.0 / 60.0, Input::default());
            }
            self.g.demo_ai = None;
        }
        self.key_dir_t -= dt;
        self.key_block_t -= dt;
        let dir = match self.held {
            Some(Ctl::Left) => -1.0,
            Some(Ctl::Right) => 1.0,
            _ => {
                if self.key_dir_t > 0.0 {
                    self.key_dir
                } else {
                    0.0
                }
            }
        };
        let inp = Input {
            dir,
            block: self.held == Some(Ctl::Block) || self.key_block_t > 0.0,
            jump: self.pend.jump,
            punch: self.pend.punch,
            kick: self.pend.kick,
        };
        self.pend = Input::default();
        let before = self.g.best;
        self.g.update(dt, inp);
        if self.g.best > before {
            let _ = self.store.set("best", self.g.best);
        }
        ctx.animate();
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        let l = layout(ctx);
        match ev {
            Event::Down { x, y } | Event::Move { x, y } => {
                if self.g.st == St::Fight {
                    let c = hit_test(&l, x, y);
                    self.set_held(c);
                }
            }
            Event::Up { .. } | Event::DragEnd { .. } => self.set_held(None),
            Event::Tap { .. } => {
                self.set_held(None);
                if self.g.st == St::Title || (self.g.st == St::MatchEnd && self.g.st_t > 0.5) {
                    self.begin();
                }
            }
            Event::Key(k) => match k {
                Key::Enter | Key::Space if self.g.st == St::Title || (self.g.st == St::MatchEnd && self.g.st_t > 0.5) => self.begin(),
                Key::Left => {
                    self.key_dir = -1.0;
                    self.key_dir_t = 0.16;
                }
                Key::Right => {
                    self.key_dir = 1.0;
                    self.key_dir_t = 0.16;
                }
                Key::Up | Key::Space => self.pend.jump = true,
                Key::Down => self.key_block_t = 0.3,
                Key::Char(c) => match c.to_ascii_lowercase() {
                    'z' | 'f' => self.pend.punch = true,
                    'x' | 'g' => self.pend.kick = true,
                    'c' => self.key_block_t = 0.3,
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
    }

    fn icon(&mut self, _ctx: &Ctx, g: &mut Gfx, size: u32) {
        let f = size as f32;
        g.rrect(0.0, 0.0, f, f, f * 0.22, rgb(0x2A1450));
        g.rrect(0.0, f * 0.48, f, f * 0.52, f * 0.22, rgb(0xE8643C));
        g.rect(0.0, f * 0.48, f, f * 0.2, rgb(0xE8643C));
        g.rect(0.0, f * 0.78, f, f * 0.22, rgb(0x2B1D3F));
        // deux poings qui se rencontrent + étincelle
        g.circle(f * 0.36, f * 0.52, f * 0.15, rgb(0x3D8BFF));
        g.circle(f * 0.64, f * 0.52, f * 0.15, rgb(0xD23B3B));
        g.circle(f * 0.5, f * 0.52, f * 0.10, rgb(0xFFE066));
        g.circle(f * 0.5, f * 0.52, f * 0.05, rgb(0xFFFFFF));
        g.circle(f * 0.5, f * 0.3, f * 0.025, rgb(0xFFE066));
        g.circle(f * 0.3, f * 0.3, f * 0.018, rgb(0xFFFFFF));
        g.circle(f * 0.7, f * 0.3, f * 0.018, rgb(0xFFFFFF));
    }
}

fn main() {
    let demo = arg_value("--demo").and_then(|v| v.parse::<u32>().ok());
    let store = Store::open("state.conf");
    let best = store.get_i64("best").unwrap_or(1).max(1) as u32;
    let mut g = Game::new();
    g.level = best;
    g.best = best;
    if let Some(l) = arg_value("--level").and_then(|v| v.parse::<u32>().ok()) {
        g.level = l;
    }
    run(
        Config::new("Combat"),
        Combat { g, store, held: None, pend: Input::default(), key_dir: 0.0, key_dir_t: 0.0, key_block_t: 0.0, demo },
    );
}
