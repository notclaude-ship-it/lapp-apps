use lapp_sdk::prelude::*;
use std::fs;

struct TtyViewer {
    selected: u8,
    content: String,
    list_scroll: Scroller,
    content_scroll: Scroller,
}

impl TtyViewer {
    fn new() -> Self {
        let mut app = Self {
            selected: 0,
            content: String::new(),
            list_scroll: Scroller::new(),
            content_scroll: Scroller::new(),
        };
        app.load();
        app
    }

    fn load(&mut self) {
        // Le contenu d'une console texte de l'hôte se lit via /dev/vcs<N> (copie de l'écran),
        // pas via /dev/tty<N> (le périphérique du terminal). /dev/vcs = console active (tty0).
        let path = if self.selected == 0 {
            "/host/dev/vcs".to_string()
        } else {
            format!("/host/dev/vcs{}", self.selected)
        };
        match fs::read(&path) {
            Ok(b) => {
                let s = String::from_utf8_lossy(&b);
                let s = s.lines().map(|l| l.trim_end()).collect::<Vec<_>>().join("\n");
                let s = s.trim_end();
                if s.is_empty() {
                    self.content = format!("(tty{} est vide)", self.selected);
                } else {
                    self.content = s.to_string();
                }
            }
            Err(e) => self.content = format!("Impossible de lire {}: {}", path, e),
        }
        self.content_scroll.jump_to(0.0);
    }
}

impl App for TtyViewer {
    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        let a = self.list_scroll.update(dt);
        let b = self.content_scroll.update(dt);
        if a || b {
            ctx.animate();
        }
    }

    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        g.clear(ctx.theme.bg);
        let s = ctx.scale();
        let (w, h) = ctx.size();
        let w = w as f32;
        let h = h as f32;

        let title = format!("TTY Viewer - tty{}", self.selected);
        let hdr = ui::header(ctx, g, &title, None);
        let top = hdr.height;
        let body_h = h - top;

        let list_w = 75.0 * s;
        let item_h = 36.0 * s;

        self.list_scroll.set_content(65.0 * item_h, body_h);

        g.rect(0.0, top, list_w, body_h, ctx.theme.surface);
        g.rect(list_w - 1.0, top, 1.0, body_h, ctx.theme.line);

        for i in 0..=64u8 {
            let item_y = top + i as f32 * item_h - self.list_scroll.pos;
            if item_y + item_h > top && item_y < h {
                let r = Rect::new(0.0, item_y, list_w, item_h);
                let label = format!("tty{}", i);
                ui::list_row(ctx, g, r, &label, None, self.selected == i);
            }
        }

        let content_x = list_w + 8.0 * s;
        let content_w = w - list_w - 16.0 * s;

        let txt_style = Text::new(12.0 * s).mono().color(ctx.theme.text).max_width(content_w);
        let line_h = ctx.line_height(&txt_style);
        let lines: Vec<&str> = self.content.lines().collect();
        let total_content_h = (lines.len().max(1) as f32) * line_h + 20.0 * s;

        self.content_scroll.set_content(total_content_h, body_h);

        let mut cy = top + 8.0 * s - self.content_scroll.pos;
        for line in &lines {
            if cy + line_h > top && cy < h {
                ctx.text(g, line, content_x, cy, &txt_style);
            }
            cy += line_h;
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        let s = ctx.scale();
        let list_w = 75.0 * s;
        let top = 44.0 * s;

        match ev {
            Event::Down { .. } => {
                self.list_scroll.stop();
                self.content_scroll.stop();
            }
            Event::Drag { x, dy, .. } => {
                if x < list_w {
                    self.list_scroll.drag(dy);
                } else {
                    self.content_scroll.drag(dy);
                }
                ctx.animate();
            }
            Event::DragEnd { x, vy, .. } => {
                if x < list_w {
                    self.list_scroll.fling(vy);
                } else {
                    self.content_scroll.fling(vy);
                }
                ctx.animate();
            }
            Event::Tap { x, y } => {
                if x < list_w && y > top {
                    let item_h = 36.0 * s;
                    let idx = ((y - top + self.list_scroll.pos) / item_h) as i32;
                    if idx >= 0 && idx <= 64 {
                        self.selected = idx as u8;
                        self.load();
                        ctx.animate();
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() {
    run(Config::new("TTY Viewer"), TtyViewer::new());
}
