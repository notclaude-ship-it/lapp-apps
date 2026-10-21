// Navigateur : navigateur web léger pour lapp (texte, liens, images JPEG/PNG), écran portrait 272x480.
//
// Barre du haut : précédent, suivant, recharger/arrêter, champ d'adresse (une saisie qui n'est pas une adresse
// lance une recherche DuckDuckGo). Quitter : glisser vers le haut depuis le bord bas, ou Echap (revient
// d'abord en arrière dans l'historique).
//
// Sur PC : cargo run -- --screenshot out.png [--html page.html] [--edit]
//          cargo run -- --icon icon.png
//          cargo run                            (fenêtre, avec LAPP_DATA=/tmp/x pour les données)

mod html;

use html::{Doc, Item};
use lapp_sdk::prelude::*;
use lapp_sdk::{arg_flag, arg_value};
use lapp_sdk::tiny_skia::IntSize;
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

const UA: &str = "Mozilla/5.0 (X11; Linux aarch64; rv:120.0) Gecko/20100101 Firefox/120.0 Lapp-Navigateur/1.0";
const LOCAL_BASE: &str = "https://local.invalid/";
const START: &[(&str, &str)] = &[
    ("Wikipédia", "https://fr.m.wikipedia.org/"),
    ("Hacker News", "https://news.ycombinator.com/"),
    ("Lite CNN", "https://lite.cnn.com/"),
    ("NPR (texte)", "https://text.npr.org/"),
    ("Lobsters", "https://lobste.rs/"),
    ("example.com", "https://example.com/"),
];

/// Génération de la page courante : les threads d'images s'arrêtent quand elle change.
static CUR_GEN: AtomicU64 = AtomicU64::new(0);

enum Msg {
    Page { id: u64, url: String, ct: String, body: String },
    Fail { id: u64, err: String },
    Image { gen: u64, idx: usize, pix: Option<(u32, u32, Vec<u8>)> },
}

// ---------------------------------------------------------------- réseau

fn fetch(url: &str, limit: u64) -> Result<(String, String, Vec<u8>), String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .redirects(6)
        .user_agent(UA)
        .build();
    let resp = agent
        .get(url)
        .set("Accept-Language", "fr,en;q=0.8")
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(c, _) => format!("Erreur HTTP {c}"),
            other => format!("{other}"),
        })?;
    let ct = resp.content_type().to_string();
    let final_url = resp.get_url().to_string();
    let mut buf = Vec::new();
    resp.into_reader().take(limit).read_to_end(&mut buf).map_err(|e| e.to_string())?;
    Ok((final_url, ct, buf))
}

fn decode_body(buf: Vec<u8>) -> String {
    match String::from_utf8(buf) {
        Ok(s) => s,
        // pas de l'UTF-8 : on suppose du latin-1 / windows-1252
        Err(e) => e.into_bytes().iter().map(|&b| b as char).collect(),
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn resize_box(src: &[u8], sw: u32, sh: u32, dw: u32, dh: u32) -> Vec<u8> {
    let mut out = vec![0u8; (dw * dh * 4) as usize];
    for dy in 0..dh {
        let y0 = (dy as u64 * sh as u64 / dh as u64) as u32;
        let y1 = (((dy + 1) as u64 * sh as u64 / dh as u64) as u32).max(y0 + 1).min(sh);
        for dx in 0..dw {
            let x0 = (dx as u64 * sw as u64 / dw as u64) as u32;
            let x1 = (((dx + 1) as u64 * sw as u64 / dw as u64) as u32).max(x0 + 1).min(sw);
            let mut acc = [0u32; 4];
            let mut n = 0u32;
            for yy in y0..y1 {
                for xx in x0..x1 {
                    let i = ((yy * sw + xx) * 4) as usize;
                    for k in 0..4 {
                        acc[k] += src[i + k] as u32;
                    }
                    n += 1;
                }
            }
            let o = ((dy * dw + dx) * 4) as usize;
            for k in 0..4 {
                out[o + k] = (acc[k] / n.max(1)) as u8;
            }
        }
    }
    out
}

fn decode_jpeg(bytes: &[u8], max_w: u32) -> Option<(u32, u32, Vec<u8>)> {
    let mut d = jpeg_decoder::Decoder::new(Cursor::new(bytes));
    d.read_info().ok()?;
    // décodage à échelle réduite (1/2, 1/4, 1/8) : bien plus rapide sur de grosses photos
    let _ = d.scale(max_w.min(60000) as u16, (max_w / 2).max(1) as u16);
    let px = d.decode().ok()?;
    let info = d.info()?;
    let (w, h) = (info.width as u32, info.height as u32);
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => {
            for c in px.chunks_exact(3) {
                out.extend_from_slice(&[c[0], c[1], c[2], 255]);
            }
        }
        jpeg_decoder::PixelFormat::L8 => {
            for &v in &px {
                out.extend_from_slice(&[v, v, v, 255]);
            }
        }
        _ => return None,
    }
    Some((w, h, out))
}

/// Télécharge et décode une image (JPEG ou PNG), réduite à `max_w` de large au plus.
fn load_image(url: &str, max_w: u32) -> Option<(u32, u32, Vec<u8>)> {
    let (_, _, bytes) = fetch(url, 2_500_000).ok()?;
    let (w, h, rgba) = if bytes.starts_with(&[0xFF, 0xD8]) {
        decode_jpeg(&bytes, max_w)?
    } else if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        let pm = Pixmap::decode_png(&bytes).ok()?;
        (pm.width(), pm.height(), pm.take())
    } else {
        return None;
    };
    if w < 20 || h < 20 {
        return None;
    }
    let scale = (max_w as f32 / w as f32).min((max_w * 3) as f32 / h as f32).min(1.0);
    if scale < 1.0 {
        let dw = ((w as f32 * scale).round() as u32).max(1);
        let dh = ((h as f32 * scale).round() as u32).max(1);
        Some((dw, dh, resize_box(&rgba, w, h, dw, dh)))
    } else {
        Some((w, h, rgba))
    }
}

// ---------------------------------------------------------------- adresses

/// Adresse absolue d'un lien (None pour les liens inutilisables). Déballe les redirections DuckDuckGo.
fn resolve(base: &str, href: &str) -> Option<String> {
    let h = href.trim();
    let l = h.to_ascii_lowercase();
    if h.is_empty() || h.starts_with('#') || l.starts_with("javascript:") || l.starts_with("mailto:") || l.starts_with("tel:") || l.starts_with("data:") {
        return None;
    }
    let b = url::Url::parse(base).ok()?;
    let mut u = b.join(h).ok()?;
    if u.scheme() != "http" && u.scheme() != "https" {
        return None;
    }
    u.set_fragment(None);
    if u.host_str().map_or(false, |x| x.ends_with("duckduckgo.com")) && u.path() == "/l/" {
        if let Some((_, v)) = u.query_pairs().find(|(k, _)| k == "uddg") {
            return Some(v.into_owned());
        }
    }
    Some(u.to_string())
}

/// Saisie de la barre d'adresse -> adresse à charger (ou recherche).
fn normalize(input: &str) -> String {
    let s = input.trim();
    if s.is_empty() {
        return String::new();
    }
    let l = s.to_ascii_lowercase();
    if l.starts_with("http://") || l.starts_with("https://") {
        return s.to_string();
    }
    if !s.contains(char::is_whitespace) && (s.contains('.') || l.starts_with("localhost")) && !s.starts_with('.') && !s.ends_with('.') {
        return format!("https://{s}");
    }
    let q: String = url::form_urlencoded::byte_serialize(s.as_bytes()).collect();
    format!("https://html.duckduckgo.com/html/?q={q}")
}

fn short_url(u: &str) -> String {
    let u = u.trim_start_matches("https://").trim_start_matches("http://").trim_start_matches("www.");
    u.trim_end_matches('/').to_string()
}

// ---------------------------------------------------------------- mise en page

#[derive(Clone)]
struct Run {
    x: f32,
    w: f32,
    text: String,
    st: Text,
    size: f32,
    bold: bool,
    mono: bool,
    dim: bool,
    link: Option<usize>,
}

enum RowKind {
    Runs(Vec<Run>),
    Image { idx: usize, x: f32, w: f32, link: Option<usize> },
    Rule,
}

struct Row {
    y: f32,
    h: f32,
    kind: RowKind,
}

struct Entry {
    url: String,
    scroll: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum Press {
    None,
    Link(usize),
    Btn(u8),
    Start(usize),
}

struct StartRow {
    y: f32,
    h: f32,
    label: String,
    url: String,
    header: bool,
}

struct Browser {
    store: Store,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
    doc: Option<Doc>,
    base: String,
    title: String,
    rows: Vec<Row>,
    content_h: f32,
    laid_w: f32,
    dirty: bool,
    imgs: HashMap<usize, Pixmap>,
    pending_imgs: usize,
    gen: u64,
    scroll: Scroller,
    restore: Option<f32>,
    hist: Vec<Entry>,
    pos: usize,
    next_id: u64,
    loading: Option<u64>,
    error: Option<String>,
    editing: bool,
    input: String,
    fresh: bool,
    kb_asked: bool,
    pressed: Press,
    recent: Vec<String>,
}

impl Browser {
    fn new(store: Store, recent: Vec<String>) -> Browser {
        let (tx, rx) = channel();
        Browser {
            store,
            tx,
            rx,
            doc: None,
            base: String::new(),
            title: String::new(),
            rows: Vec::new(),
            content_h: 0.0,
            laid_w: 0.0,
            dirty: true,
            imgs: HashMap::new(),
            pending_imgs: 0,
            gen: 0,
            scroll: Scroller::new(),
            restore: None,
            hist: Vec::new(),
            pos: 0,
            next_id: 1,
            loading: None,
            error: None,
            editing: false,
            input: String::new(),
            fresh: false,
            kb_asked: false,
            pressed: Press::None,
            recent,
        }
    }

    // ------------------------------------------------------------ géométrie

    fn tb_h(ctx: &Ctx) -> f32 {
        40.0 * ctx.scale()
    }
    fn top(ctx: &Ctx) -> f32 {
        Self::tb_h(ctx) + 18.0 * ctx.scale()
    }
    fn btn_rect(ctx: &Ctx, i: usize) -> Rect {
        let s = ctx.scale();
        Rect::new(4.0 * s + i as f32 * 30.0 * s, 4.0 * s, 28.0 * s, 32.0 * s)
    }
    fn url_rect(ctx: &Ctx) -> Rect {
        let s = ctx.scale();
        let x = 4.0 * s + 3.0 * 30.0 * s + 2.0 * s;
        Rect::new(x, 5.0 * s, ctx.width() - x - 6.0 * s, 30.0 * s)
    }
    fn cur_url(&self) -> Option<String> {
        self.hist.get(self.pos).map(|e| e.url.clone())
    }

    // ------------------------------------------------------------ navigation

    fn save_scroll(&mut self) {
        let p = self.scroll.pos;
        if let Some(e) = self.hist.get_mut(self.pos) {
            e.scroll = p;
        }
    }

    fn go(&mut self, url: String, push: bool) {
        self.save_scroll();
        if push {
            self.hist.truncate(self.pos + 1);
            self.hist.push(Entry { url: url.clone(), scroll: 0.0 });
            self.pos = self.hist.len() - 1;
            self.restore = None;
        }
        self.start_fetch(url);
    }

    fn go_back(&mut self) {
        if self.pos > 0 && !self.hist.is_empty() {
            self.save_scroll();
            self.pos -= 1;
            self.restore = Some(self.hist[self.pos].scroll);
            let u = self.hist[self.pos].url.clone();
            self.start_fetch(u);
        }
    }

    fn go_forward(&mut self) {
        if self.pos + 1 < self.hist.len() {
            self.save_scroll();
            self.pos += 1;
            self.restore = Some(self.hist[self.pos].scroll);
            let u = self.hist[self.pos].url.clone();
            self.start_fetch(u);
        }
    }

    fn start_fetch(&mut self, url: String) {
        let id = self.next_id;
        self.next_id += 1;
        self.loading = Some(id);
        self.error = None;
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let m = match fetch(&url, 3_000_000) {
                Ok((final_url, ct, buf)) => Msg::Page { id, url: final_url, ct, body: decode_body(buf) },
                Err(err) => Msg::Fail { id, err },
            };
            let _ = tx.send(m);
        });
    }

    fn remember(&mut self, url: &str) {
        self.recent.retain(|u| u != url);
        self.recent.insert(0, url.to_string());
        self.recent.truncate(8);
        let _ = self.store.set("recent", self.recent.join(" "));
    }

    fn set_doc(&mut self, ctx: &Ctx, url: String, doc: Doc, gen: u64) {
        self.title = doc.title.clone();
        self.base = url.clone();
        self.imgs.clear();
        self.gen = gen;
        CUR_GEN.store(gen, Ordering::SeqCst);
        let max_w = (ctx.width() - 20.0 * ctx.scale()).max(40.0) as u32;
        let jobs: Vec<(usize, String)> = doc
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, it)| match it {
                Item::Image { src, .. } => resolve(&url, src).map(|u| (i, u)),
                _ => None,
            })
            .take(14)
            .collect();
        self.pending_imgs = jobs.len();
        self.doc = Some(doc);
        self.dirty = true;
        if !jobs.is_empty() {
            let tx = self.tx.clone();
            std::thread::spawn(move || {
                for (idx, u) in jobs {
                    if CUR_GEN.load(Ordering::SeqCst) != gen {
                        return;
                    }
                    let pix = load_image(&u, max_w);
                    if tx.send(Msg::Image { gen, idx, pix }).is_err() {
                        return;
                    }
                }
            });
        }
    }

    fn load_local(&mut self, doc: Doc) {
        self.title = doc.title.clone();
        self.base = LOCAL_BASE.to_string();
        self.doc = Some(doc);
        self.dirty = true;
        self.hist.push(Entry { url: LOCAL_BASE.to_string(), scroll: 0.0 });
    }

    fn poll(&mut self, ctx: &Ctx) -> bool {
        let mut changed = false;
        while let Ok(m) = self.rx.try_recv() {
            changed = true;
            match m {
                Msg::Page { id, url, ct, body } => {
                    if self.loading != Some(id) {
                        continue;
                    }
                    self.loading = None;
                    let lower = ct.to_ascii_lowercase();
                    let src = if lower.is_empty() || lower.contains("html") || lower.contains("xml") {
                        body
                    } else if lower.starts_with("text/") || lower.contains("json") {
                        format!("<pre>{}</pre>", escape(&body))
                    } else {
                        self.error = Some(format!("Type non pris en charge : {ct}"));
                        continue;
                    };
                    if let Some(e) = self.hist.get_mut(self.pos) {
                        e.url = url.clone();
                    }
                    let doc = html::parse(&src);
                    self.remember(&url);
                    if self.restore.is_none() {
                        self.scroll.jump_to(0.0);
                    }
                    self.set_doc(ctx, url, doc, id);
                }
                Msg::Fail { id, err } => {
                    if self.loading == Some(id) {
                        self.loading = None;
                        self.error = Some(err);
                    }
                }
                Msg::Image { gen, idx, pix } => {
                    if gen != self.gen {
                        continue;
                    }
                    self.pending_imgs = self.pending_imgs.saturating_sub(1);
                    if let Some((w, h, data)) = pix {
                        if let Some(size) = IntSize::from_wh(w, h) {
                            if let Some(pm) = Pixmap::from_vec(data, size) {
                                self.imgs.insert(idx, pm);
                                self.dirty = true;
                            }
                        }
                    }
                }
            }
        }
        changed
    }

    // ------------------------------------------------------------ mise en page de la page

    fn ensure_layout(&mut self, ctx: &Ctx) {
        if self.dirty || (ctx.width() - self.laid_w).abs() > 0.5 {
            self.layout(ctx);
            self.dirty = false;
            self.laid_w = ctx.width();
        }
    }

    fn layout(&mut self, ctx: &Ctx) {
        let Some(doc) = self.doc.as_ref() else {
            self.rows.clear();
            self.content_h = 0.0;
            return;
        };
        let s = ctx.scale();
        let t = ctx.theme;
        let margin = 10.0 * s;
        let avail = ctx.width() - 2.0 * margin;
        let mut rows: Vec<Row> = Vec::new();
        let mut y = 6.0 * s;
        let mut cur: Vec<Run> = Vec::new();
        let mut cx = 0.0f32;
        let mut lh = 0.0f32;
        let mut gap = true;
        macro_rules! flush {
            () => {
                if !cur.is_empty() {
                    rows.push(Row { y, h: lh, kind: RowKind::Runs(std::mem::take(&mut cur)) });
                    y += lh + 2.0 * s;
                    cx = 0.0;
                    lh = 0.0;
                    gap = false;
                }
            };
        }
        let mut alt_s = String::new();
        for (i, it) in doc.items.iter().enumerate() {
            let w: Option<(&str, bool, bool, u8, Option<usize>, bool)> = match it {
                Item::Word { text, bold, mono, head, link } => Some((text.as_str(), *bold, *mono, *head, *link, false)),
                Item::Br => {
                    flush!();
                    None
                }
                Item::Para => {
                    flush!();
                    if !gap {
                        y += 6.0 * s;
                        gap = true;
                    }
                    None
                }
                Item::Rule => {
                    flush!();
                    rows.push(Row { y: y + 3.0 * s, h: 1.0, kind: RowKind::Rule });
                    y += 9.0 * s;
                    gap = true;
                    None
                }
                Item::Image { alt, link, .. } => {
                    if let Some(pm) = self.imgs.get(&i) {
                        flush!();
                        let (pw, ph) = (pm.width() as f32, pm.height() as f32);
                        let x = ((avail - pw) / 2.0).max(0.0);
                        rows.push(Row { y, h: ph, kind: RowKind::Image { idx: i, x, w: pw, link: *link } });
                        y += ph + 6.0 * s;
                        gap = false;
                        None
                    } else if !alt.is_empty() {
                        alt_s = format!("[{alt}]");
                        Some((alt_s.as_str(), false, false, 0, *link, true))
                    } else {
                        None
                    }
                }
            };
            let Some((text, bold, mono, head, link, dim)) = w else { continue };
            let base = 15.0 * s;
            let mut size = match head {
                1 => base * 1.5,
                2 => base * 1.3,
                3 => base * 1.15,
                4..=6 => base * 1.05,
                _ => base,
            };
            if mono {
                size *= 0.88;
            }
            let color = if dim {
                t.subtext
            } else if link.is_some() {
                t.accent
            } else {
                t.text
            };
            let mut st = Text::new(size).color(color).left();
            if bold {
                st = st.bold();
            }
            if mono {
                st = st.mono();
            }
            let mut ww = ctx.measure(text, &st);
            let sp = size * 0.3;
            if !cur.is_empty() && cx + sp + ww > avail {
                flush!();
            }
            if ww > avail {
                st = st.max_width(avail);
                ww = avail;
            }
            lh = lh.max(ctx.line_height(&st));
            if let Some(last) = cur
                .last_mut()
                .filter(|r| r.link == link && r.bold == bold && r.mono == mono && r.dim == dim && (r.size - size).abs() < 0.01)
            {
                last.text.push(' ');
                last.text.push_str(text);
                last.w += sp + ww;
                cx = last.x + last.w;
            } else {
                let x = if cur.is_empty() { 0.0 } else { cx + sp };
                cx = x + ww;
                cur.push(Run { x, w: ww, text: text.to_string(), st, size, bold, mono, dim, link });
            }
        }
        flush!();
        let _ = gap;
        self.content_h = y + 24.0 * s;
        self.rows = rows;
    }

    // ------------------------------------------------------------ page d'accueil

    fn start_rows(&self, ctx: &Ctx) -> (Vec<StartRow>, f32) {
        let s = ctx.scale();
        let mut out = Vec::new();
        let mut y = 8.0 * s + 62.0 * s;
        let mut add = |out: &mut Vec<StartRow>, y: &mut f32, label: String, url: String, header: bool| {
            let h = if header { 28.0 * s } else { 46.0 * s };
            out.push(StartRow { y: *y, h, label, url, header });
            *y += h;
        };
        add(&mut out, &mut y, "Accueil rapide".into(), String::new(), true);
        for (name, url) in START {
            add(&mut out, &mut y, name.to_string(), url.to_string(), false);
        }
        if !self.recent.is_empty() {
            add(&mut out, &mut y, "Récents".into(), String::new(), true);
            for u in &self.recent {
                add(&mut out, &mut y, short_url(u), u.clone(), false);
            }
        }
        (out, y + 24.0 * s)
    }

    fn start_at(&self, ctx: &Ctx, y: f32) -> Option<usize> {
        let py = y - Self::top(ctx) + self.scroll.pos;
        let (rows, _) = self.start_rows(ctx);
        rows.iter().position(|r| !r.header && py >= r.y && py < r.y + r.h)
    }

    fn link_at(&self, ctx: &Ctx, x: f32, y: f32) -> Option<usize> {
        let s = ctx.scale();
        let margin = 10.0 * s;
        let py = y - Self::top(ctx) + self.scroll.pos;
        for row in &self.rows {
            if py < row.y - 1.0 * s || py >= row.y + row.h + 2.0 * s {
                continue;
            }
            match &row.kind {
                RowKind::Runs(runs) => {
                    for r in runs {
                        if r.link.is_some() && x >= margin + r.x - 3.0 * s && x <= margin + r.x + r.w + 3.0 * s {
                            return r.link;
                        }
                    }
                }
                RowKind::Image { x: ix, w, link, .. } => {
                    if link.is_some() && x >= margin + ix && x <= margin + ix + w {
                        return *link;
                    }
                }
                RowKind::Rule => {}
            }
        }
        None
    }

    // ------------------------------------------------------------ dessin

    fn draw_page(&self, ctx: &Ctx, g: &mut Gfx, top: f32, pos: f32) {
        let t = ctx.theme;
        let s = ctx.scale();
        let h = ctx.height();
        let margin = 10.0 * s;
        for row in &self.rows {
            let ry = top + row.y - pos;
            if ry > h || ry + row.h < top {
                continue;
            }
            match &row.kind {
                RowKind::Runs(runs) => {
                    for r in runs {
                        let rx = margin + r.x;
                        if let Some(l) = r.link {
                            if self.pressed == Press::Link(l) {
                                g.rrect(rx - 2.0 * s, ry - 1.0 * s, r.w + 4.0 * s, row.h + 2.0 * s, 3.0 * s, t.press);
                            }
                        }
                        ctx.text(g, &r.text, rx, ry, &r.st);
                        if r.link.is_some() {
                            g.rect(rx, ry + row.h - 1.0 * s, r.w, 1.0, rgba(0x007AFF, 0.45));
                        }
                    }
                }
                RowKind::Image { idx, x, link, w } => {
                    if let Some(pm) = self.imgs.get(idx) {
                        g.image(&format!("nav-{}-{}", self.gen, idx), pm, margin + x, ry, 1.0);
                        if let Some(l) = link {
                            if self.pressed == Press::Link(*l) {
                                g.rect(margin + x, ry, *w, row.h, rgba(0x007AFF, 0.18));
                            }
                        }
                    }
                }
                RowKind::Rule => g.rect(margin, ry, ctx.width() - 2.0 * margin, 1.0, t.line),
            }
        }
    }

    fn draw_start(&self, ctx: &Ctx, g: &mut Gfx, top: f32, pos: f32) {
        let t = ctx.theme;
        let s = ctx.scale();
        let w = ctx.width();
        let (rows, _) = self.start_rows(ctx);
        let hy = top + 8.0 * s - pos;
        let big = Text::new(26.0 * s).bold().color(t.text).left();
        ctx.text(g, "Navigateur", 14.0 * s, hy, &big);
        let small = Text::new(12.5 * s).color(t.subtext).left().max_width(w - 28.0 * s);
        ctx.text(g, "Saisissez une adresse ou une recherche", 14.0 * s, hy + 34.0 * s, &small);
        for (i, r) in rows.iter().enumerate() {
            let ry = top + r.y - pos;
            if ry > ctx.height() || ry + r.h < top {
                continue;
            }
            if r.header {
                let st = Text::new(12.0 * s).bold().color(t.subtext).left();
                ctx.text_mid(g, &r.label.to_uppercase(), 14.0 * s, ry + r.h * 0.62, &st);
            } else {
                ui::list_row(ctx, g, Rect::new(0.0, ry, w, r.h), &r.label, None, self.pressed == Press::Start(i));
            }
        }
    }

    fn draw_bars(&self, ctx: &Ctx, g: &mut Gfx) {
        let t = ctx.theme;
        let s = ctx.scale();
        let w = ctx.width();
        let top = Self::top(ctx);
        g.rect(0.0, 0.0, w, top, t.surface);
        g.rect(0.0, top - 1.0, w, 1.0, t.line);
        let busy = self.loading.is_some();
        let can_back = self.pos > 0 && !self.hist.is_empty();
        let can_fwd = self.pos + 1 < self.hist.len();
        let can_reload = busy || !self.hist.is_empty();
        let btns: [(&str, bool); 3] = [("‹", can_back), ("›", can_fwd), (if busy { "×" } else { "" }, can_reload)];
        for (i, (lbl, on)) in btns.iter().enumerate() {
            let r = Self::btn_rect(ctx, i);
            if self.pressed == Press::Btn(i as u8) && *on {
                g.rrect(r.x, r.y, r.w, r.h, 8.0 * s, t.press);
            }
            let col = if *on { t.accent } else { t.line };
            let (cx, cy) = r.center();
            if lbl.is_empty() {
                // icône « recharger » : anneau ouvert + pointe
                g.circle(cx, cy, 8.5 * s, col);
                g.circle(cx, cy, 6.0 * s, t.surface);
                g.rect(cx, cy - 10.0 * s, 10.0 * s, 6.0 * s, t.surface);
                g.rect(cx + 2.0 * s, cy - 10.5 * s, 7.0 * s, 2.5 * s, col);
                g.rect(cx + 6.5 * s, cy - 10.5 * s, 2.5 * s, 7.0 * s, col);
            } else {
                let st = Text::new(26.0 * s).color(col).center();
                ctx.text_mid(g, lbl, cx, cy - 1.0 * s, &st);
            }
        }
        // champ d'adresse
        let r = Self::url_rect(ctx);
        g.rrect(r.x, r.y, r.w, r.h, r.h / 2.0, if self.editing { t.accent } else { t.line });
        let inner = r.inset(1.5 * s);
        g.rrect(inner.x, inner.y, inner.w, inner.h, inner.h / 2.0, if self.editing { t.surface } else { t.bg });
        let st = Text::new(14.0 * s).color(t.text).left().ellipsis_left().max_width(r.w - 22.0 * s);
        let (_, cy) = r.center();
        if self.editing {
            let txt = format!("{}|", self.input);
            let st = if self.fresh { st.color(t.accent) } else { st };
            ctx.text_mid(g, &txt, r.x + 11.0 * s, cy, &st);
        } else if let Some(u) = self.cur_url().filter(|u| u != LOCAL_BASE) {
            ctx.text_mid(g, &short_url(&u), r.x + 11.0 * s, cy, &st);
        } else {
            let ph = Text::new(14.0 * s).color(t.subtext).left().max_width(r.w - 22.0 * s);
            ctx.text_mid(g, "Rechercher ou saisir une adresse", r.x + 11.0 * s, cy, &ph);
        }
        // bande d'état
        let iy = Self::tb_h(ctx) + 9.0 * s;
        let (msg, col) = if busy {
            ("Chargement…".to_string(), t.subtext)
        } else if let Some(e) = &self.error {
            (e.clone(), t.danger)
        } else if self.pending_imgs > 0 {
            (format!("{} · images : {}", self.title, self.pending_imgs), t.subtext)
        } else {
            (self.title.clone(), t.subtext)
        };
        if !msg.trim().is_empty() {
            let st = Text::new(11.5 * s).color(col).center().max_width(w - 20.0 * s);
            ctx.text_mid(g, &msg, w / 2.0, iy, &st);
        }
    }

    // ------------------------------------------------------------ interactions

    fn press_at(&self, ctx: &Ctx, x: f32, y: f32) -> Press {
        let top = Self::top(ctx);
        if y < Self::tb_h(ctx) {
            for i in 0..3 {
                if Self::btn_rect(ctx, i).contains(x, y) {
                    return Press::Btn(i as u8);
                }
            }
            return Press::None;
        }
        if y < top {
            return Press::None;
        }
        if self.doc.is_some() {
            if let Some(l) = self.link_at(ctx, x, y) {
                return Press::Link(l);
            }
        } else if let Some(i) = self.start_at(ctx, y) {
            return Press::Start(i);
        }
        Press::None
    }

    fn tap(&mut self, ctx: &Ctx, x: f32, y: f32) {
        self.pressed = Press::None;
        if y < Self::tb_h(ctx) {
            if Self::btn_rect(ctx, 0).contains(x, y) {
                self.go_back();
            } else if Self::btn_rect(ctx, 1).contains(x, y) {
                self.go_forward();
            } else if Self::btn_rect(ctx, 2).contains(x, y) {
                if self.loading.is_some() {
                    self.loading = None;
                } else if let Some(u) = self.cur_url() {
                    if u != LOCAL_BASE {
                        self.restore = Some(self.scroll.pos);
                        self.start_fetch(u);
                    }
                }
            } else if Self::url_rect(ctx).contains(x, y) {
                self.editing = true;
                self.input = self.cur_url().filter(|u| u != LOCAL_BASE).unwrap_or_default();
                self.fresh = true;
            }
            return;
        }
        if self.editing {
            self.editing = false;
        }
        if y < Self::top(ctx) {
            return;
        }
        if self.doc.is_some() {
            if let Some(l) = self.link_at(ctx, x, y) {
                let href = self.doc.as_ref().and_then(|d| d.links.get(l).cloned());
                if let Some(u) = href.and_then(|h| resolve(&self.base, &h)) {
                    self.go(u, true);
                } else {
                    ctx.toast("Lien non pris en charge");
                }
            }
        } else if let Some(i) = self.start_at(ctx, y) {
            let (rows, _) = self.start_rows(ctx);
            let u = rows[i].url.clone();
            self.go(u, true);
        }
    }

    fn commit(&mut self) {
        let u = normalize(&self.input);
        self.editing = false;
        if !u.is_empty() {
            self.go(u, true);
        }
    }

    fn key(&mut self, ctx: &Ctx, k: Key) {
        if self.editing {
            match k {
                Key::Char(c) if !c.is_control() => {
                    if self.fresh {
                        self.input.clear();
                        self.fresh = false;
                    }
                    self.input.push(c);
                }
                Key::Space => {
                    if self.fresh {
                        self.input.clear();
                        self.fresh = false;
                    }
                    self.input.push(' ');
                }
                Key::Back => {
                    if self.fresh {
                        self.input.clear();
                        self.fresh = false;
                    } else {
                        self.input.pop();
                    }
                }
                Key::Enter => self.commit(),
                _ => {}
            }
            return;
        }
        let s = ctx.scale();
        match k {
            Key::Up => self.scroll.drag(60.0 * s),
            Key::Down => self.scroll.drag(-60.0 * s),
            Key::Space => self.scroll.drag(-ctx.height() * 0.8),
            Key::Left => self.go_back(),
            Key::Right => self.go_forward(),
            _ => {}
        }
    }
}

impl App for Browser {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        if self.editing != self.kb_asked {
            ctx.keyboard(if self.editing { 1 } else { 0 });
            self.kb_asked = self.editing;
        }
        let t = ctx.theme;
        let s = ctx.scale();
        g.clear(t.bg);
        self.ensure_layout(ctx);
        let top = Self::top(ctx);
        let view_h = (ctx.height() - top - ctx.keyboard_height()).max(60.0 * s);
        let content_h = if self.doc.is_some() { self.content_h } else { self.start_rows(ctx).1 };
        self.scroll.set_content(content_h, view_h);
        if let Some(p) = self.restore.take() {
            self.scroll.jump_to(p);
        }
        let pos = self.scroll.pos;
        if self.doc.is_some() {
            self.draw_page(ctx, g, top, pos);
        } else {
            self.draw_start(ctx, g, top, pos);
        }
        self.draw_bars(ctx, g);
        if self.loading.is_some() || self.pending_imgs > 0 {
            ctx.redraw_in(Duration::from_millis(150));
        }
    }

    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        if self.poll(ctx) {
            ctx.animate();
        }
        if self.scroll.update(dt) {
            ctx.animate();
        }
        if self.loading.is_some() || self.pending_imgs > 0 {
            ctx.redraw_in(Duration::from_millis(150));
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        match ev {
            Event::Down { x, y } => {
                self.scroll.stop();
                self.pressed = self.press_at(ctx, x, y);
            }
            Event::Up { .. } => self.pressed = Press::None,
            Event::Drag { dy, .. } | Event::Wheel { dy } => {
                self.pressed = Press::None;
                self.scroll.drag(dy);
            }
            Event::DragEnd { vy, .. } => {
                self.pressed = Press::None;
                self.scroll.fling(vy);
            }
            Event::Tap { x, y } => self.tap(ctx, x, y),
            Event::Key(k) => self.key(ctx, k),
            _ => {}
        }
    }

    fn on_back(&mut self, _ctx: &Ctx) -> bool {
        if self.editing {
            self.editing = false;
            true
        } else if self.loading.is_some() {
            self.loading = None;
            true
        } else if self.pos > 0 && !self.hist.is_empty() {
            self.go_back();
            true
        } else if self.doc.is_some() {
            // retour à la page d'accueil
            self.doc = None;
            self.rows.clear();
            self.hist.clear();
            self.pos = 0;
            self.title.clear();
            self.scroll.jump_to(0.0);
            true
        } else {
            false
        }
    }

    fn icon(&mut self, _ctx: &Ctx, g: &mut Gfx, size: u32) {
        let f = size as f32;
        let blue = rgb(0x1E6FEB);
        let white = rgb(0xFFFFFF);
        g.rrect(0.0, 0.0, f, f, f * 0.22, blue);
        let (cx, cy) = (f / 2.0, f / 2.0);
        let r = f * 0.30;
        g.circle(cx, cy, r, white);
        g.circle(cx, cy, r * 0.86, blue);
        g.rect(cx - f * 0.012, cy - r, f * 0.024, 2.0 * r, white);
        g.rect(cx - r, cy - f * 0.012, 2.0 * r, f * 0.024, white);
        g.circle(cx, cy, r * 0.5, white);
        g.circle(cx, cy, r * 0.5 - f * 0.024, blue);
    }
}

fn main() {
    let store = Store::open("state.conf");
    let recent: Vec<String> = store.get("recent").map(|s| s.split_whitespace().map(String::from).collect()).unwrap_or_default();
    let mut app = Browser::new(store, recent);
    if let Some(p) = arg_value("--html") {
        if let Ok(src) = std::fs::read_to_string(&p) {
            app.load_local(html::parse(&src));
        }
    }
    if arg_flag("--edit") {
        app.editing = true;
        app.input = "wikipedia.org".into();
    }
    run(Config::new("Navigateur"), app);
}