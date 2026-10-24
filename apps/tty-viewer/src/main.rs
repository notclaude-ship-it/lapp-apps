use lapp_sdk::prelude::*;

mod hostcmd;
mod sudo;

struct TtyViewer {
    selected: u8,
    content: String,
    list_scroll: Scroller,
    content_scroll: Scroller,
    /// `Some(refus)` : sudo demande un mot de passe (`refus` = celui qu'on avait été refusé)
    need_pw: Option<bool>,
    pw: String,
    show_pw: bool,
    /// le clavier virtuel a été demandé pour la page du mot de passe
    kb_asked: bool,
}

impl TtyViewer {
    fn new() -> Self {
        let mut app = Self {
            selected: 0,
            content: String::new(),
            list_scroll: Scroller::new(),
            content_scroll: Scroller::new(),
            need_pw: None,
            pw: String::new(),
            show_pw: false,
            kb_asked: false,
        };
        app.load();
        app
    }

    fn load(&mut self) {
        // /dev/vcs<N> = copie texte de l'écran de la console N (/dev/vcs = console active, tty0).
        // Le bac à sable n'y a pas accès : la lecture passe par le courtier de commandes de l'hôte.
        let path = if self.selected == 0 {
            "/dev/vcs".to_string()
        } else {
            format!("/dev/vcs{}", self.selected)
        };
        // `cat` direct d'abord (marche si l'utilisateur est dans le groupe tty) ; sinon `sudo cat`.
        let res = match hostcmd::run(&["cat", &path], "") {
            Ok(o) if o.code == 0 => Ok(o),
            _ => sudo::run(&["cat", &path]),
        };
        match res {
            Ok(o) if o.code == 0 => {
                self.need_pw = None;
                let s: String = o.out.lines().map(|l| l.trim_end()).collect::<Vec<_>>().join("\n");
                let s = s.trim_end();
                self.content = if s.is_empty() { format!("(tty{} est vide)", self.selected) } else { s.to_string() };
            }
            Ok(o) => {
                self.need_pw = None;
                self.content = format!("Impossible de lire {}: {}", path, o.err.trim());
            }
            Err(e) if e == sudo::NEEDED || e == sudo::WRONG => {
                // mot de passe sudo à saisir : la page de saisie s'affiche
                self.need_pw = Some(e == sudo::WRONG);
                self.pw.clear();
                self.kb_asked = false;
                self.content.clear();
            }
            Err(e) => {
                self.need_pw = None;
                self.content = format!("Impossible de lire {}: {}", path, e);
            }
        }
        self.content_scroll.jump_to(0.0);
    }

    /// Zones de la page du mot de passe : (champ, bouton Afficher/Masquer, bouton Valider).
    fn pw_rects(&self, ctx: &Ctx) -> (Rect, Rect, Rect) {
        let s = ctx.scale();
        let (w, _) = ctx.size();
        let w = w as f32;
        let top = 44.0 * s;
        let m = 16.0 * s;
        let field = Rect::new(m, top + 78.0 * s, w - 2.0 * m, 42.0 * s);
        let bw = (w - 2.0 * m - 10.0 * s) / 2.0;
        let y = field.y + field.h + 14.0 * s;
        (field, Rect::new(m, y, bw, 42.0 * s), Rect::new(m + bw + 10.0 * s, y, bw, 42.0 * s))
    }

    fn submit_pw(&mut self, ctx: &Ctx) {
        if self.pw.is_empty() {
            return;
        }
        sudo::set(std::mem::take(&mut self.pw));
        self.load();
        if self.need_pw.is_none() {
            ctx.keyboard(0);
        } else {
            ctx.keyboard(1);
            self.kb_asked = true;
        }
        ctx.animate();
    }

    fn render_pw(&mut self, ctx: &Ctx, g: &mut Gfx, refused: bool) {
        let s = ctx.scale();
        let (w, _) = ctx.size();
        let w = w as f32;
        let top = ui::header(ctx, g, "TTY Viewer", None).height;
        if !self.kb_asked {
            self.kb_asked = true;
            ctx.keyboard(1);
        }
        let (title, tc) = if refused {
            ("Mot de passe incorrect", ctx.theme.danger)
        } else {
            ("Droits administrateur", ctx.theme.text)
        };
        let t1 = Text::new(18.0 * s).bold().color(tc).center().max_width(w - 24.0 * s);
        ctx.text_mid(g, title, w / 2.0, top + 24.0 * s, &t1);
        let t2 = Text::new(12.0 * s).color(ctx.theme.subtext).center().max_width(w - 24.0 * s);
        ctx.text_mid(g, "Lire les consoles de l'hôte demande sudo.", w / 2.0, top + 46.0 * s, &t2);
        ctx.text_mid(g, "Entre le mot de passe sudo de l'appareil.", w / 2.0, top + 62.0 * s, &t2);

        let (field, show, ok) = self.pw_rects(ctx);
        g.rrect(field.x, field.y, field.w, field.h, 8.0 * s, ctx.theme.surface);
        g.rrect(field.x, field.y, field.w, 1.0, 0.0, ctx.theme.line);
        g.rrect(field.x, field.y + field.h - 1.0, field.w, 1.0, 0.0, ctx.theme.line);
        let (cx, cy) = (field.x + 14.0 * s, field.y + field.h / 2.0);
        if self.show_pw {
            let st = Text::new(16.0 * s).mono().color(ctx.theme.text).left().max_width(field.w - 28.0 * s).ellipsis_left();
            ctx.text_mid(g, &self.pw, cx, cy, &st);
        } else {
            let r = 4.0 * s;
            let step = 14.0 * s;
            let max = ((field.w - 28.0 * s) / step) as usize;
            let n = self.pw.chars().count();
            for i in 0..n.min(max) {
                g.circle(cx + r + i as f32 * step, cy, r, ctx.theme.text);
            }
        }
        ui::button(ctx, g, show, if self.show_pw { "Masquer" } else { "Afficher" }, ui::ButtonStyle::Secondary, false);
        ui::button(ctx, g, ok, "Valider", ui::ButtonStyle::Primary, false);
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
        if let Some(refused) = self.need_pw {
            self.render_pw(ctx, g, refused);
            return;
        }
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
        if self.need_pw.is_some() {
            match ev {
                Event::Key(Key::Char(c)) => {
                    self.pw.push(c);
                    ctx.animate();
                }
                Event::Key(Key::Space) => {
                    self.pw.push(' ');
                    ctx.animate();
                }
                Event::Key(Key::Back) => {
                    self.pw.pop();
                    ctx.animate();
                }
                Event::Key(Key::Enter) => self.submit_pw(ctx),
                Event::Tap { x, y } => {
                    let (field, show, ok) = self.pw_rects(ctx);
                    if ok.contains(x, y) {
                        self.submit_pw(ctx);
                    } else if show.contains(x, y) {
                        self.show_pw = !self.show_pw;
                        ctx.animate();
                    } else if field.contains(x, y) {
                        ctx.keyboard(1); // réaffiche le clavier s'il a été masqué
                    }
                }
                _ => {}
            }
            return;
        }
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
