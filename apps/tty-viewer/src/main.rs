use lapp_sdk::prelude::*;
use std::fs;

// ---- Lecture des consoles de l'hôte via la permission "commands" -------------------------------------
// Le bac à sable monte /host en `nodev` et son /dev est minimal : aucun /dev/vcs<N> n'y est ouvrable.
// On demande donc au courtier de l'hôte d'exécuter `cat /dev/vcs<N>` (puis `sudo -n cat` si l'utilisateur
// de la session n'a pas le droit de lecture). Protocole (u32 big-endian) :
//   requête : argc, (len, octets) x argc, len_stdin, stdin
//   réponse : code i32, len_out, out, len_err, err

fn broker_run(args: &[&str]) -> Result<(i32, Vec<u8>, Vec<u8>), String> {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;

    let sock = std::env::var("LAPP_COMMANDS_SOCKET").unwrap_or_else(|_| "/run/lapp/commands.sock".into());
    if !std::path::Path::new(&sock).exists() {
        // développement sur PC (hors bac à sable) : on lance la commande directement
        let o = std::process::Command::new(args[0])
            .args(&args[1..])
            .output()
            .map_err(|e| format!("{}: {}", args[0], e))?;
        return Ok((o.status.code().unwrap_or(-1), o.stdout, o.stderr));
    }
    let mut st = UnixStream::connect(&sock).map_err(|e| format!("courtier injoignable ({}): {}", sock, e))?;
    st.set_read_timeout(Some(std::time::Duration::from_secs(100))).ok();
    let mut req = Vec::new();
    req.extend_from_slice(&(args.len() as u32).to_be_bytes());
    for a in args {
        req.extend_from_slice(&(a.len() as u32).to_be_bytes());
        req.extend_from_slice(a.as_bytes());
    }
    req.extend_from_slice(&0u32.to_be_bytes());
    st.write_all(&req).map_err(|e| e.to_string())?;

    let mut buf = Vec::new();
    st.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    let u32at = |o: usize| -> Result<usize, String> {
        buf.get(o..o + 4)
            .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize)
            .ok_or_else(|| "réponse du courtier tronquée".to_string())
    };
    let code = u32at(0)? as u32 as i32;
    let lo = u32at(4)?;
    let out = buf.get(8..8 + lo).ok_or("réponse du courtier tronquée")?.to_vec();
    let le = u32at(8 + lo)?;
    let err = buf.get(12 + lo..12 + lo + le).ok_or("réponse du courtier tronquée")?.to_vec();
    Ok((code, out, err))
}

fn read_console(path: &str) -> Result<Vec<u8>, String> {
    let (code, out, err) = broker_run(&["cat", path])?;
    if code == 0 {
        return Ok(out);
    }
    let (code2, out2, err2) = broker_run(&["sudo", "-n", "cat", path])?;
    if code2 == 0 {
        return Ok(out2);
    }
    let msg = |e: &[u8]| String::from_utf8_lossy(e).trim().to_string();
    Err(format!("cat: {} / sudo: {}", msg(&err), msg(&err2)))
}


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
        // /dev/vcs<N> = copie texte de l'écran de la console N (/dev/vcs = console active, tty0).
        let path = if self.selected == 0 {
            "/dev/vcs".to_string()
        } else {
            format!("/dev/vcs{}", self.selected)
        };
        match read_console(&path) {
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
