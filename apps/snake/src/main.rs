use std::time::Instant;
use lapp_sdk::prelude::*;

const GRID_SIZE: i32 = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    fn is_opposite(self, other: Dir) -> bool {
        matches!(
            (self, other),
            (Dir::Up, Dir::Down)
                | (Dir::Down, Dir::Up)
                | (Dir::Left, Dir::Right)
                | (Dir::Right, Dir::Left)
        )
    }
}

struct Rng(u64);

impl Rng {
    fn new() -> Self {
        let seed = Instant::now().elapsed().as_nanos() as u64 ^ 0x9E3779B97F4A7C15;
        Rng(if seed == 0 { 0x853c49e6748fea9b } else { seed })
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.0 >> 32) as u32
    }

    fn gen_range(&mut self, max: i32) -> i32 {
        if max <= 0 {
            0
        } else {
            (self.next_u32() % (max as u32)) as i32
        }
    }
}

struct SnakeGame {
    body: Vec<(i32, i32)>,
    dir: Dir,
    next_dir: Dir,
    food: (i32, i32),
    dead: bool,
    score: u32,
    high_score: u32,
    step_timer: f32,
    rng: Rng,
    store: Store,
    btn_restart_pressed: bool,
    btn_dir_pressed: Option<Dir>,
}

impl SnakeGame {
    fn new() -> Self {
        let store = Store::open("snake");
        let high_score = store.get_i64("best").unwrap_or(0).max(0) as u32;
        let mut game = Self {
            body: Vec::new(),
            dir: Dir::Right,
            next_dir: Dir::Right,
            food: (0, 0),
            dead: false,
            score: 0,
            high_score,
            step_timer: 0.0,
            rng: Rng::new(),
            store,
            btn_restart_pressed: false,
            btn_dir_pressed: None,
        };
        game.reset();
        game
    }

    fn reset(&mut self) {
        self.body.clear();
        let cx = GRID_SIZE / 2;
        let cy = GRID_SIZE / 2;
        self.body.push((cx, cy));
        self.body.push((cx - 1, cy));
        self.body.push((cx - 2, cy));
        self.dir = Dir::Right;
        self.next_dir = Dir::Right;
        self.dead = false;
        self.score = 0;
        self.step_timer = 0.0;
        self.spawn_food();
    }

    fn spawn_food(&mut self) {
        for _ in 0..100 {
            let x = self.rng.gen_range(GRID_SIZE);
            let y = self.rng.gen_range(GRID_SIZE);
            if !self.body.contains(&(x, y)) {
                self.food = (x, y);
                return;
            }
        }
        self.food = (0, 0);
    }

    fn change_dir(&mut self, new_dir: Dir) {
        if !self.dead && !self.dir.is_opposite(new_dir) {
            self.next_dir = new_dir;
        }
    }

    fn step(&mut self) {
        if self.dead {
            return;
        }
        self.dir = self.next_dir;
        let (hx, hy) = self.body[0];
        let (nx, ny) = match self.dir {
            Dir::Up => (hx, hy - 1),
            Dir::Down => (hx, hy + 1),
            Dir::Left => (hx - 1, hy),
            Dir::Right => (hx + 1, hy),
        };

        // Mur
        if nx < 0 || nx >= GRID_SIZE || ny < 0 || ny >= GRID_SIZE {
            self.kill_player();
            return;
        }

        // Collision corps
        if self.body.contains(&(nx, ny)) {
            self.kill_player();
            return;
        }

        self.body.insert(0, (nx, ny));
        if (nx, ny) == self.food {
            self.score += 1;
            if self.score > self.high_score {
                self.high_score = self.score;
                let _ = self.store.set("best", self.high_score);
            }
            self.spawn_food();
        } else {
            self.body.pop();
        }
    }

    fn kill_player(&mut self) {
        self.dead = true;
    }
}

impl App for SnakeGame {
    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        if !self.dead {
            ctx.animate();
            self.step_timer += dt;
            let speed = (0.16 - (self.score as f32 * 0.003)).max(0.08);
            if self.step_timer >= speed {
                self.step_timer = 0.0;
                self.step();
            }
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        match ev {
            Event::Key(Key::Up) | Event::Key(Key::Char('z')) | Event::Key(Key::Char('w')) => {
                self.change_dir(Dir::Up);
            }
            Event::Key(Key::Down) | Event::Key(Key::Char('s')) => {
                self.change_dir(Dir::Down);
            }
            Event::Key(Key::Left) | Event::Key(Key::Char('q')) | Event::Key(Key::Char('a')) => {
                self.change_dir(Dir::Left);
            }
            Event::Key(Key::Right) | Event::Key(Key::Char('d')) => {
                self.change_dir(Dir::Right);
            }
            Event::Key(Key::Space) | Event::Key(Key::Enter) => {
                if self.dead {
                    self.reset();
                }
            }
            Event::DragEnd { vx, vy, .. } => {
                if vx.abs() > vy.abs() {
                    if vx > 80.0 {
                        self.change_dir(Dir::Right);
                    } else if vx < -80.0 {
                        self.change_dir(Dir::Left);
                    }
                } else {
                    if vy > 80.0 {
                        self.change_dir(Dir::Down);
                    } else if vy < -80.0 {
                        self.change_dir(Dir::Up);
                    }
                }
            }
            Event::Down { x, y } => {
                let w = ctx.width();
                let s = ctx.scale();
                let pad_cy = 390.0 * s;
                let pad_cx = w / 2.0;
                let btn_sz = 46.0 * s;

                if Rect::new(pad_cx - btn_sz / 2.0, pad_cy - btn_sz - 4.0 * s, btn_sz, btn_sz).contains(x, y) {
                    self.btn_dir_pressed = Some(Dir::Up);
                    self.change_dir(Dir::Up);
                } else if Rect::new(pad_cx - btn_sz / 2.0, pad_cy + 4.0 * s, btn_sz, btn_sz).contains(x, y) {
                    self.btn_dir_pressed = Some(Dir::Down);
                    self.change_dir(Dir::Down);
                } else if Rect::new(pad_cx - btn_sz * 1.5 - 4.0 * s, pad_cy - btn_sz / 2.0, btn_sz, btn_sz).contains(x, y) {
                    self.btn_dir_pressed = Some(Dir::Left);
                    self.change_dir(Dir::Left);
                } else if Rect::new(pad_cx + btn_sz * 0.5 + 4.0 * s, pad_cy - btn_sz / 2.0, btn_sz, btn_sz).contains(x, y) {
                    self.btn_dir_pressed = Some(Dir::Right);
                    self.change_dir(Dir::Right);
                }

                if self.dead {
                    let btn_r = Rect::new(w * 0.15, 230.0 * s, w * 0.7, 44.0 * s);
                    if btn_r.contains(x, y) {
                        self.btn_restart_pressed = true;
                    }
                }
            }
            Event::Up { .. } => {
                self.btn_dir_pressed = None;
                self.btn_restart_pressed = false;
            }
            Event::Tap { x, y } => {
                if self.dead {
                    let w = ctx.width();
                    let s = ctx.scale();
                    let btn_r = Rect::new(w * 0.15, 230.0 * s, w * 0.7, 44.0 * s);
                    if btn_r.contains(x, y) {
                        self.reset();
                    }
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let w = ctx.width();
        let h = ctx.height();
        let s = ctx.scale();

        g.clear(rgb(0x181824));

        // Barre supérieure avec scores
        let header_h = 44.0 * s;
        g.rect(0.0, 0.0, w, header_h, rgb(0x222232));
        let st_score = Text::new(14.0 * s).bold().color(rgb(0xFFFFFF)).left();
        let st_best = Text::new(14.0 * s).color(rgb(0xFFA500)).right();
        ctx.text_mid(g, &format!("Score: {}", self.score), 12.0 * s, header_h / 2.0, &st_score);
        ctx.text_mid(g, &format!("Best: {}", self.high_score), w - 12.0 * s, header_h / 2.0, &st_best);

        // Grille de jeu
        let grid_px = (GRID_SIZE as f32 * 15.0 * s).min(w - 16.0 * s);
        let cell_px = grid_px / (GRID_SIZE as f32);
        let grid_x = (w - grid_px) / 2.0;
        let grid_y = header_h + 12.0 * s;

        // Fond du plateau
        g.rrect(grid_x - 3.0 * s, grid_y - 3.0 * s, grid_px + 6.0 * s, grid_px + 6.0 * s, 8.0 * s, rgb(0x2E2E40));
        g.rrect(grid_x, grid_y, grid_px, grid_px, 6.0 * s, rgb(0x101018));

        // Pomme
        let (fx, fy) = self.food;
        let apple_cx = grid_x + (fx as f32 + 0.5) * cell_px;
        let apple_cy = grid_y + (fy as f32 + 0.5) * cell_px;
        g.circle(apple_cx, apple_cy, cell_px * 0.42, rgb(0xEE4444));
        g.circle(apple_cx - cell_px * 0.12, apple_cy - cell_px * 0.12, cell_px * 0.14, rgb(0xFFAAAA));

        // Serpent
        for (i, &(bx, by)) in self.body.iter().enumerate() {
            let px = grid_x + bx as f32 * cell_px;
            let py = grid_y + by as f32 * cell_px;
            let rad = cell_px * 0.25;
            let color = if i == 0 {
                if self.dead { rgb(0xDD3333) } else { rgb(0x44DD66) }
            } else {
                let shade_f = (1.0 - (i as f32 * 0.02)).max(0.6);
                shade(rgb(0x22AA44), shade_f)
            };
            g.rrect(px + 1.0, py + 1.0, cell_px - 2.0, cell_px - 2.0, rad, color);
        }

        // D-Pad tactile en bas
        let pad_cy = 390.0 * s;
        let pad_cx = w / 2.0;
        let btn_sz = 46.0 * s;
        let pad_col = rgb(0x2A2A3E);
        let pad_act = rgb(0x404060);

        let draw_btn = |g: &mut Gfx, r: Rect, sym: &str, active: bool| {
            let col = if active { pad_act } else { pad_col };
            g.rrect(r.x, r.y, r.w, r.h, 10.0 * s, col);
            let st = Text::new(20.0 * s).bold().color(rgb(0xFFFFFF)).center();
            ctx.text_mid(g, sym, r.x + r.w / 2.0, r.y + r.h / 2.0, &st);
        };

        draw_btn(g, Rect::new(pad_cx - btn_sz / 2.0, pad_cy - btn_sz - 4.0 * s, btn_sz, btn_sz), "▲", self.btn_dir_pressed == Some(Dir::Up));
        draw_btn(g, Rect::new(pad_cx - btn_sz / 2.0, pad_cy + 4.0 * s, btn_sz, btn_sz), "▼", self.btn_dir_pressed == Some(Dir::Down));
        draw_btn(g, Rect::new(pad_cx - btn_sz * 1.5 - 4.0 * s, pad_cy - btn_sz / 2.0, btn_sz, btn_sz), "◀", self.btn_dir_pressed == Some(Dir::Left));
        draw_btn(g, Rect::new(pad_cx + btn_sz * 0.5 + 4.0 * s, pad_cy - btn_sz / 2.0, btn_sz, btn_sz), "▶", self.btn_dir_pressed == Some(Dir::Right));

        // Écran de mort
        if self.dead {
            g.rect(0.0, header_h, w, h - header_h, rgba(0x000000, 0.75));
            let st_over = Text::new(28.0 * s).bold().color(rgb(0xFF4444)).center();
            ctx.text_mid(g, "GAME OVER", w / 2.0, 150.0 * s, &st_over);

            let st_res = Text::new(16.0 * s).color(rgb(0xFFFFFF)).center();
            ctx.text_mid(g, &format!("Score: {}", self.score), w / 2.0, 185.0 * s, &st_res);

            let btn_r = Rect::new(w * 0.15, 230.0 * s, w * 0.7, 44.0 * s);
            ui::button(ctx, g, btn_r, "Play Again", ButtonStyle::Primary, self.btn_restart_pressed);
        }
    }
}

fn main() {
    run(Config::new("Snake").size(272, 480), SnakeGame::new());
}
