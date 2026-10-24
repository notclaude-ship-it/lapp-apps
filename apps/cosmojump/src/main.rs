mod draw;
mod game;
mod hud;

use game::*;
use lapp_sdk::{arg_flag, arg_value};
use lapp_sdk::prelude::*;

struct Cosmo {
    g: Game,
    demo: Option<u32>,
    force_over: bool,
}

impl App for Cosmo {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        draw::world(&self.g, ctx, g);
        hud::draw(&self.g, ctx, g);
    }

    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        // mode capture : simule N images avec un pilote automatique
        if let Some(n) = self.demo.take() {
            self.g.start();
            for _ in 0..n {
                self.g.autopilot();
                self.g.update(ctx, 1.0 / 60.0);
                if self.g.st == St::Over {
                    break;
                }
            }
            if self.force_over && self.g.st == St::Play {
                self.g.py = self.g.cam - 100.0;
                for _ in 0..40 {
                    self.g.update(ctx, 1.0 / 60.0);
                }
            }
        }
        self.g.update(ctx, dt);
        ctx.animate();
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        let s = ctx.scale();
        let vw = ctx.width() / s;
        match ev {
            Event::Down { x, .. } | Event::Move { x, .. } => {
                if self.g.st == St::Play {
                    self.g.target = (x / s).clamp(0.0, vw);
                }
            }
            Event::Tap { x, y } => match self.g.st {
                St::Title => self.g.start(),
                St::Over => {
                    if self.g.over_t > 0.5 && (hud::replay_rect(ctx).contains(x, y) || self.g.over_t > 1.5) {
                        self.g.reset();
                        self.g.start();
                    }
                }
                St::Play => {}
            },
            Event::Key(k) => match (k, self.g.st) {
                (Key::Left, St::Play) => self.g.target = (self.g.target - 45.0).max(0.0),
                (Key::Right, St::Play) => self.g.target = (self.g.target + 45.0).min(vw),
                (Key::Space | Key::Enter | Key::Up, St::Title) => self.g.start(),
                (Key::Space | Key::Enter, St::Over) if self.g.over_t > 0.5 => {
                    self.g.reset();
                    self.g.start();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn icon(&mut self, _ctx: &Ctx, g: &mut Gfx, size: u32) {
        let f = size as f32;
        g.rrect(0.0, 0.0, f, f, f * 0.22, rgb(0x1D2166));
        g.rrect(0.0, f * 0.5, f, f * 0.5, f * 0.22, rgb(0x7A45A8));
        g.rect(0.0, f * 0.5, f, f * 0.2, rgb(0x7A45A8));
        g.circle(f * 0.2, f * 0.2, f * 0.012, rgb(0xFFFFFF));
        g.circle(f * 0.82, f * 0.14, f * 0.015, rgb(0xFFFFFF));
        g.circle(f * 0.72, f * 0.35, f * 0.01, rgb(0xFFFFFF));
        g.rrect(f * 0.18, f * 0.8, f * 0.64, f * 0.09, f * 0.045, rgb(0x3FAE52));
        g.rrect(f * 0.18, f * 0.8, f * 0.64, f * 0.05, f * 0.045, rgb(0x7BE07F));
        draw::alien(g, f * 0.5, f * 0.8, f / 60.0, 0.3, 120.0, 0.0, false, 0.0);
    }
}

fn main() {
    let demo = arg_value("--demo").and_then(|v| v.parse::<u32>().ok());
    let force_over = arg_flag("--over");
    run(Config::new("Cosmo Jump"), Cosmo { g: Game::new(), demo, force_over });
}
