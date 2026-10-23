use lapp_sdk::prelude::*;
use std::path::PathBuf;
use std::process::Command;

struct Line {
    text: String,
    is_cmd: bool,
    is_err: bool,
}

struct TerminalApp {
    history: Vec<String>,
    history_idx: Option<usize>,
    lines: Vec<Line>,
    input: String,
    cwd: PathBuf,
    scroller: Scroller,
    kbd_mode: u32,
}

impl TerminalApp {
    fn new() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/data"));
        let mut app = Self {
            history: Vec::new(),
            history_idx: None,
            lines: Vec::new(),
            input: String::new(),
            cwd,
            scroller: Scroller::new(),
            kbd_mode: 1,
        };
        app.lines.push(Line {
            text: "Lapp Terminal v1.0.0".to_string(),
            is_cmd: false,
            is_err: false,
        });
        app.lines.push(Line {
            text: "Tapez 'help' pour obtenir de l'aide.".to_string(),
            is_cmd: false,
            is_err: false,
        });
        app
    }

    fn execute_cmd(&mut self) {
        let cmd_str = self.input.trim().to_string();
        if cmd_str.is_empty() {
            return;
        }

        self.history.push(cmd_str.clone());
        self.history_idx = None;

        let prompt_symbol = if self.cwd.to_string_lossy() == "/" { "/" } else { self.cwd.file_name().and_then(|n| n.to_str()).unwrap_or("~") };
        let prompt_line = format!("$ {} {}", prompt_symbol, cmd_str);
        self.lines.push(Line {
            text: prompt_line,
            is_cmd: true,
            is_err: false,
        });
        self.input.clear();

        if cmd_str == "clear" {
            self.lines.clear();
            return;
        }

        if cmd_str == "help" {
            self.lines.push(Line {
                text: "Commandes intégrées : cd, pwd, clear, help".to_string(),
                is_cmd: false,
                is_err: false,
            });
            self.lines.push(Line {
                text: "Toutes les commandes shell /bin/sh sont exécutables.".to_string(),
                is_cmd: false,
                is_err: false,
            });
            return;
        }

        if cmd_str == "pwd" {
            self.lines.push(Line {
                text: self.cwd.display().to_string(),
                is_cmd: false,
                is_err: false,
            });
            return;
        }

        if cmd_str.starts_with("cd ") || cmd_str == "cd" {
            let target = if cmd_str == "cd" {
                std::env::var("HOME").unwrap_or_else(|_| "/data".to_string())
            } else {
                cmd_str[3..].trim().to_string()
            };
            let new_path = if target.starts_with('/') {
                PathBuf::from(&target)
            } else {
                self.cwd.join(&target)
            };
            if new_path.is_dir() {
                if let Ok(canonical) = new_path.canonicalize() {
                    self.cwd = canonical;
                } else {
                    self.cwd = new_path;
                }
                let _ = std::env::set_current_dir(&self.cwd);
            } else {
                self.lines.push(Line {
                    text: format!("cd: {}: Aucun dossier de ce nom", target),
                    is_cmd: false,
                    is_err: true,
                });
            }
            return;
        }

        let output = Command::new("/bin/sh")
            .arg("-c")
            .arg(&cmd_str)
            .current_dir(&self.cwd)
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    self.lines.push(Line {
                        text: line.to_string(),
                        is_cmd: false,
                        is_err: false,
                    });
                }
                let stderr = String::from_utf8_lossy(&out.stderr);
                for line in stderr.lines() {
                    self.lines.push(Line {
                        text: line.to_string(),
                        is_cmd: false,
                        is_err: true,
                    });
                }
            }
            Err(e) => {
                self.lines.push(Line {
                    text: format!("Erreur d'exécution: {}", e),
                    is_cmd: false,
                    is_err: true,
                });
            }
        }
    }

    fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let idx = match self.history_idx {
            None => self.history.len().saturating_sub(1),
            Some(i) => i.saturating_sub(1),
        };
        self.history_idx = Some(idx);
        if let Some(cmd) = self.history.get(idx) {
            self.input = cmd.clone();
        }
    }

    fn history_next(&mut self) {
        if let Some(idx) = self.history_idx {
            if idx + 1 < self.history.len() {
                let new_idx = idx + 1;
                self.history_idx = Some(new_idx);
                self.input = self.history[new_idx].clone();
            } else {
                self.history_idx = None;
                self.input.clear();
            }
        }
    }
}

impl App for TerminalApp {
    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        if self.scroller.update(dt) {
            ctx.animate();
        }
    }

    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        g.clear(rgb(0x121214));
        let s = ctx.scale();

        let hdr = ui::header(ctx, g, "Terminal", None);
        let header_h = hdr.height;

        let kbd_h = ctx.keyboard_height();
        let bar_h = 36.0 * s;
        let view_y = header_h;
        let view_h = (ctx.height() - header_h - bar_h - kbd_h).max(50.0);

        g.rect(0.0, view_y, ctx.width(), view_h, rgb(0x0C0C0E));

        let font_size = 11.0 * s;
        let st_normal = Text::new(font_size).mono().color(rgb(0xE5E5EA));
        let st_cmd = Text::new(font_size).mono().bold().color(rgb(0x64D2FF));
        let st_err = Text::new(font_size).mono().color(rgb(0xFF453A));
        let st_prompt = Text::new(font_size).mono().bold().color(rgb(0x30D158));

        let lh = ctx.line_height(&st_normal);

        let total_lines = self.lines.len() + 1;
        let content_h = total_lines as f32 * lh + 10.0 * s;

        self.scroller.set_content(content_h, view_h);

        let start_y = view_y + 5.0 * s - self.scroller.pos;

        for (i, line) in self.lines.iter().enumerate() {
            let y = start_y + i as f32 * lh;
            if y + lh >= view_y && y <= view_y + view_h {
                let st = if line.is_cmd {
                    &st_cmd
                } else if line.is_err {
                    &st_err
                } else {
                    &st_normal
                };
                ctx.text(g, &line.text, 8.0 * s, y, st);
            }
        }

        let prompt_y = start_y + self.lines.len() as f32 * lh;
        if prompt_y + lh >= view_y && prompt_y <= view_y + view_h {
            let prompt_symbol = if self.cwd.to_string_lossy() == "/" { "/" } else { self.cwd.file_name().and_then(|n| n.to_str()).unwrap_or("~") };
            let prompt = format!("{}> ", prompt_symbol);
            let pw = ctx.measure(&prompt, &st_prompt);
            ctx.text(g, &prompt, 8.0 * s, prompt_y, &st_prompt);

            let input_display = format!("{}_", self.input);
            ctx.text(g, &input_display, 8.0 * s + pw, prompt_y, &st_cmd);
        }

        let bar_y = ctx.height() - kbd_h - bar_h;
        g.rect(0.0, bar_y, ctx.width(), bar_h, rgb(0x222226));
        g.rect(0.0, bar_y, ctx.width(), 1.0, rgb(0x3A3A3C));

        let btn_keys = ["↑", "↓", "Tab", "Ctrl+C", "/", "-", "|", "~", "Clv"];
        let btn_w = (ctx.width() - 8.0 * s) / btn_keys.len() as f32;

        for (i, &key_label) in btn_keys.iter().enumerate() {
            let rx = 4.0 * s + i as f32 * btn_w;
            let r = ui::Rect::new(rx + 1.0, bar_y + 3.0 * s, btn_w - 2.0, bar_h - 6.0 * s);
            let st_btn = Text::new(11.0 * s).mono().color(rgb(0xFFFFFF)).center();
            g.rrect(r.x, r.y, r.w, r.h, 4.0 * s, rgb(0x3A3A3C));
            let (cx, cy) = r.center();
            ctx.text_mid(g, key_label, cx, cy, &st_btn);
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        let s = ctx.scale();
        let kbd_h = ctx.keyboard_height();
        let bar_h = 36.0 * s;
        let bar_y = ctx.height() - kbd_h - bar_h;

        match ev {
            Event::Tap { x, y } => {
                if y >= bar_y && y < bar_y + bar_h {
                    let btn_keys = ["↑", "↓", "Tab", "Ctrl+C", "/", "-", "|", "~", "Clv"];
                    let btn_w = (ctx.width() - 8.0 * s) / btn_keys.len() as f32;
                    let idx = ((x - 4.0 * s) / btn_w) as usize;
                    if idx < btn_keys.len() {
                        match btn_keys[idx] {
                            "↑" => self.history_prev(),
                            "↓" => self.history_next(),
                            "Tab" => self.input.push_str("  "),
                            "Ctrl+C" => {
                                self.lines.push(Line { text: "^C".to_string(), is_cmd: true, is_err: false });
                                self.input.clear();
                            }
                            "/" => self.input.push('/'),
                            "-" => self.input.push('-'),
                            "|" => self.input.push('|'),
                            "~" => self.input.push('~'),
                            "Clv" => {
                                if self.kbd_mode == 0 {
                                    self.kbd_mode = 1;
                                    ctx.keyboard(1);
                                } else {
                                    self.kbd_mode = 0;
                                    ctx.keyboard(0);
                                }
                            }
                            _ => {}
                        }
                    }
                } else if y < bar_y {
                    ctx.keyboard(self.kbd_mode);
                }
            }
            Event::Drag { dy, .. } | Event::Wheel { dy } => {
                self.scroller.drag(dy);
            }
            Event::DragEnd { vy, .. } => {
                self.scroller.fling(vy);
            }
            Event::Down { .. } => {
                self.scroller.stop();
            }
            Event::Key(key) => {
                match key {
                    Key::Char(ch) => {
                        self.input.push(ch);
                    }
                    Key::Back => {
                        self.input.pop();
                    }
                    Key::Enter => {
                        self.execute_cmd();
                        self.scroller.jump_to(self.scroller.max);
                    }
                    Key::Space => {
                        self.input.push(' ');
                    }
                    Key::Up => self.history_prev(),
                    Key::Down => self.history_next(),
                    _ => {}
                }
            }
        }
    }
}

fn main() {
    let cfg = Config::new("Terminal");
    run(cfg, TerminalApp::new());
}
