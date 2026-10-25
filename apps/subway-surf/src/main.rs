// Subway Surf : course infinie sur trois voies (écran portrait 272x480).
//
// Glisser ← → pour changer de voie, ↑ pour sauter, ↓ pour rouler. Esquivez les rames et les barrières,
// ramassez les pièces, le jetpack et l'aimant. Une première collision fait venir le gardien, la suivante vous attrape.
// Clavier (PC) : flèches (ou q d z s) / Espace saut / Bas ou S roulade.
// Quitter : glisser vers le haut depuis le bord bas de l'écran, ou Echap.
// Sur PC : cargo run -- --screenshot out.png [--demo 600]   (rendu hors écran ; --demo = N images jouées par une IA)
//          cargo run -- --sim [secondes]                     (simulation de l'IA, sans affichage)
//          cargo run -- --icon icon.png                      (icône)
mod draw;
mod game;

use draw::*;
use game::*;
use lapp_sdk::prelude::*;
use lapp_sdk::{arg_value, Store};

struct Surf {
    g: Game,
    store: Store,
    anchor: Option<(f32, f32)>,
    pend: Input,
    demo: Option<u32>,
    best_shown: u32,
}

impl Surf {
    fn begin(&mut self) {
        match self.g.st {
            St::Title => self.g.start(),
            St::Over if self.g.st_t > 0.6 => self.g.start(),
            _ => {}
        }
    }

    fn save(&mut self) {
        let _ = self.store.set("best", self.g.best);
        let total = self.store.get_i64("coins").unwrap_or(0) + self.g.coins as i64;
        let _ = self.store.set("coins", total);
    }
}

impl App for Surf {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let s = ctx.scale();
        let (vw, vh) = (ctx.width() / s, ctx.height() / s);
        let gm = &self.g;
        let cam = Cam::new(vw, vh, gm);
        let sh = gm.shake * gm.shake * 7.0;
        let (ox, oy) = if sh > 0.0 { ((gm.t * 91.0).sin() * sh * s, (gm.t * 73.0).cos() * sh * 0.6 * s) } else { (0.0, 0.0) };
        g.clear(rgb(0xC9E6F4));
        {
            let mut p = P { g: &mut *g, s, ox, oy, cw: vw, ch: vh };
            scene(&mut p, &cam, gm);
        }
        let mut p = P { g: &mut *g, s, ox: 0.0, oy: 0.0, cw: vw, ch: vh };
        if gm.st != St::Title {
            hud(ctx, &mut p, vw, gm, self.best_shown);
        }
        overlay(ctx, &mut p, vw, vh, gm);
    }

    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        if let Some(n) = self.demo.take() {
            // mode capture : l'IA joue
            self.g.start();
            for _ in 0..n {
                let inp = self.g.autopilot();
                self.g.update(1.0 / 60.0, inp);
            }
        }
        let dt = dt.min(0.05);
        let inp = self.pend;
        self.pend = Input::default();
        let before = self.g.st;
        self.g.update(dt, inp);
        if before == St::Play && self.g.st == St::Over {
            self.save();
        }
        ctx.animate();
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        let thr = 22.0 * ctx.scale();
        match ev {
            Event::Down { x, y } => self.anchor = Some((x, y)),
            Event::Move { x, y } => {
                if let Some((ax, ay)) = self.anchor {
                    let (dx, dy) = (x - ax, y - ay);
                    if dx.abs().max(dy.abs()) > thr {
                        if dx.abs() > dy.abs() {
                            if dx < 0.0 {
                                self.pend.left = true;
                            } else {
                                self.pend.right = true;
                            }
                        } else if dy < 0.0 {
                            self.pend.jump = true;
                        } else {
                            self.pend.roll = true;
                        }
                        self.anchor = Some((x, y));
                    }
                }
            }
            Event::Up { .. } | Event::DragEnd { .. } => self.anchor = None,
            Event::Tap { .. } => {
                self.anchor = None;
                self.begin();
            }
            Event::Key(k) => match k {
                Key::Enter | Key::Space if self.g.st != St::Play => self.begin(),
                Key::Left => self.pend.left = true,
                Key::Right => self.pend.right = true,
                Key::Up | Key::Space => self.pend.jump = true,
                Key::Down => self.pend.roll = true,
                Key::Char(c) => match c.to_ascii_lowercase() {
                    'q' | 'a' => self.pend.left = true,
                    'd' => self.pend.right = true,
                    'z' | 'w' => self.pend.jump = true,
                    's' => self.pend.roll = true,
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
    }

    fn icon(&mut self, _ctx: &Ctx, g: &mut Gfx, size: u32) {
        draw::icon(g, size);
    }
}

fn main() {
    if lapp_sdk::arg_flag("--sim") {
        let secs = arg_value("--sim").and_then(|v| v.parse::<f32>().ok()).unwrap_or(120.0);
        for seed in 1..=8u64 {
            let (t, score, coins, died) = sim(seed, secs);
            println!("graine {seed}: {:.1}s score {score} pièces {coins} {}", t, if died { "PERDU" } else { "ok" });
        }
        return;
    }
    let demo = arg_value("--demo").and_then(|v| v.parse::<u32>().ok());
    let store = Store::open("state.conf");
    let best = store.get_i64("best").unwrap_or(0).max(0) as u32;
    let mut g = Game::new();
    g.best = best;
    run(Config::new("Subway Surf"), Surf { g, store, anchor: None, pend: Input::default(), demo, best_shown: best });
}