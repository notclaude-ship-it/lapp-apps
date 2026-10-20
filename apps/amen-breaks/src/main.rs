use lapp_sdk::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Child;

struct AmenApp {
    files: Vec<String>,
    scroll: Scroller,
    playing_file: Option<String>,
    child: Option<Child>,
}

impl AmenApp {
    fn new() -> Self {
        let mut files = Vec::new();

        if let Ok(entries) = fs::read_dir(lapp_sdk::app_dir()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        let ext_lower = ext.to_lowercase();
                        if ext_lower == "wav" || ext_lower == "mp3" {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                files.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        files.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        AmenApp {
            files,
            scroll: Scroller::new(),
            playing_file: None,
            child: None,
        }
    }

    fn play(&mut self, name: &str) {
        if let Some(mut old) = self.child.take() {
            let _ = old.kill();
        }
        let full_path = lapp_sdk::app_path(name);
        self.child = play_audio(&full_path);
        self.playing_file = Some(name.to_string());
    }
}

impl App for AmenApp {
    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        if self.scroll.update(dt) {
            ctx.animate();
        }
        if let Some(ref mut child) = self.child {
            if let Ok(Some(_)) = child.try_wait() {
                self.child = None;
                self.playing_file = None;
            }
        }
    }

    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        g.clear(ctx.theme.bg);

        let head = ui::header(ctx, g, "Amen Breaks", None);
        let s = ctx.scale();
        let row_h = 48.0 * s;
        let view_h = (ctx.height() - head.height).max(1.0);
        let content_h = self.files.len() as f32 * row_h;

        self.scroll.set_content(content_h, view_h);

        let start_y = head.height;
        let scroll_pos = self.scroll.pos;

        let first_idx = (scroll_pos / row_h).floor() as usize;
        let visible_count = (view_h / row_h).ceil() as usize + 2;
        let last_idx = (first_idx + visible_count).min(self.files.len());

        for i in first_idx..last_idx {
            let y = start_y + (i as f32 * row_h) - scroll_pos;
            let r = Rect::new(0.0, y, ctx.width(), row_h);
            let name = &self.files[i];
            let is_playing = self.playing_file.as_deref() == Some(name.as_str());

            let detail = if is_playing { "▶ Lecture" } else { "" };
            ui::list_row(ctx, g, r, name, Some(detail), is_playing);
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        let s = ctx.scale();
        let head_h = 44.0 * s;

        match ev {
            Event::Down { .. } => self.scroll.stop(),
            Event::Drag { dy, .. } | Event::Wheel { dy } => self.scroll.drag(dy),
            Event::DragEnd { vy, .. } => self.scroll.fling(vy),
            Event::Tap { x: _, y } => {
                if y > head_h {
                    let row_h = 48.0 * s;
                    let idx = ((y - head_h + self.scroll.pos) / row_h).floor() as usize;
                    if idx < self.files.len() {
                        let name = self.files[idx].clone();
                        self.play(&name);
                        ctx.toast(format!("▶ {}", name));
                    }
                }
            }
            _ => {}
        }
    }
}

fn play_audio(path: &Path) -> Option<Child> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let players: &[(&str, &[&str])] = if ext == "wav" {
        &[
            ("aplay", &["-q"]),
            ("pw-play", &[]),
            ("paplay", &[]),
            ("mpv", &["--no-video", "--really-quiet"]),
            ("play", &["-q"]),
            ("ffplay", &["-nodisp", "-autoexit", "-loglevel", "quiet"]),
        ]
    } else {
        &[
            ("pw-play", &[]),
            ("paplay", &[]),
            ("mpv", &["--no-video", "--really-quiet"]),
            ("play", &["-q"]),
            ("ffplay", &["-nodisp", "-autoexit", "-loglevel", "quiet"]),
            ("aplay", &["-q"]),
        ]
    };

    for &(cmd, args) in players {
        let mut c = std::process::Command::new(cmd);
        c.args(args).arg(path);
        if let Ok(child) = c.spawn() {
            return Some(child);
        }
    }
    None
}

impl Drop for AmenApp {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
    }
}

fn main() {
    run(Config::new("Amen Breaks"), AmenApp::new());
}
