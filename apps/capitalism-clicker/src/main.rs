//! Capitalism Supremacy Clicker
//! Game made by Ziyad.K for the phone with Alexis.R
use lapp_sdk::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

// ===== Adaptateurs SDK : si une signature differe, ne corrigez QUE ces fonctions =====
fn rc(x: f32, y: f32, w: f32, h: f32) -> Rect { Rect::new(x, y, w, h) }
fn hit(r: &Rect, x: f32, y: f32) -> bool { r.contains(x, y) }
fn fill(g: &mut Gfx, r: Rect, c: Color) { g.rect(r.x, r.y, r.w, r.h, c) }
fn rfill(g: &mut Gfx, r: Rect, rad: f32, c: Color) { g.rrect(r.x, r.y, r.w, r.h, rad, c) }
fn disc(g: &mut Gfx, x: f32, y: f32, rad: f32, c: Color) { g.circle(x, y, rad, c) }
fn gsize(g: &Gfx) -> (f32, f32) { let (w, h) = g.size(); (w as f32, h as f32) }
fn img(p: &str) -> Option<Pixmap> { lapp_sdk::load_png(p) }
fn new_scroller() -> Scroller { Scroller::new() }
fn sget(st: &Store, k: &str) -> Option<String> { st.get(k).map(|s| s.to_string()) }
fn sset(st: &mut Store, k: &str, v: &str) { let _ = st.set(k, v); }
fn sdel(st: &mut Store, k: &str) { let _ = st.remove(k); }
// =====================================================================================

// Nombre d'ameliorations
const N: usize = 11;
// (nom, cout de base, gain par seconde)
const UP: [(&str, f64, f64); N] = [
    ("Free labour", 15.0, 0.2), ("Salarié", 120.0, 1.5), ("Scam", 400.0, 4.0),
    ("Entreprise", 1100.0, 10.0), ("Méga entreprise", 12e3, 80.0), ("Trading", 50e3, 300.0),
    ("Big Corp", 130e3, 700.0), ("Giga Corp", 1.4e6, 6.5e3), ("Téra Corp", 20e6, 60e3),
    ("Planet Inc.", 330e6, 600e3), ("Galaxy Ltd.", 5.1e9, 6e6),
];
// Anciennes sauvegardes (9 ameliorations) -> nouvelle position de chaque amelioration
const OLD_MAP: [usize; 9] = [0, 1, 3, 4, 6, 7, 8, 9, 10];
const STAGES: [(f64, &str, u32); 8] = [
    (0.0, "Pièce", 0xFFC800), (1e2, "Billet", 0x3FA34D), (1e4, "Liasse", 0x2E7D32),
    (1e6, "Mallette", 0x8D5A2B), (1e8, "Coffre", 0x707B85), (1e10, "Compte", 0x1E6FD9),
    (1e13, "Stocks", 0x8E44AD), (1e16, "Empire", 0xD4A017),
];
// Images du code "random" (la derniere, le drapeau de l'URSS, sert aussi a Stalin / Lenin)
const PICS: [&str; 8] = [
    "assets/rand0.png", "assets/rand1.png", "assets/rand2.png", "assets/rand3.png",
    "assets/rand4.png", "assets/rand5.png", "assets/rand6.png", "assets/ussr.png",
];
const FLAG: usize = 7;
// Clavier AZERTY ('<' = effacer)
const KEYS: [&str; 3] = ["AZERTYUIOP", "QSDFGHJKLM", "WXCVBN<"];

fn key_list() -> Vec<(char, usize, usize)> {
    let mut v = vec![];
    for (r, row) in KEYS.iter().enumerate() {
        for (c, ch) in row.chars().enumerate() { v.push((ch, r, c)); }
    }
    v
}

fn fmt(v: f64) -> String {
    const S: [&str; 12] = ["", "K", "M", "B", "T", "Qa", "Qi", "Sx", "Sp", "Oc", "No", "Dc"];
    if v < 1000.0 {
        return if v < 10.0 && v.fract() != 0.0 { format!("{:.1}", v) } else { format!("{:.0}", v) };
    }
    let i = ((v.log10() / 3.0) as usize).min(11);
    format!("{:.2}{}", v / 1000f64.powi(i as i32), S[i])
}
fn t(s: f32, c: Color) -> Text { Text::new(s).bold().color(c).center() }

#[derive(PartialEq, Clone, Copy)]
enum Scr { Title, Saves, Game, Menu, Code, Random }

#[derive(Clone)]
struct Game { money: f64, total: f64, mult: f64, used: u8, lv: [u32; N] }
impl Game {
    fn new() -> Game { Game { money: 0.0, total: 0.0, mult: 1.0, used: 0, lv: [0; N] } }
    fn cps(&self) -> f64 { (0..N).map(|i| self.lv[i] as f64 * UP[i].2).sum() }
    fn click(&self) -> f64 { (1.0 + 0.05 * self.cps()) * self.mult }
    fn cost(&self, i: usize) -> f64 { UP[i].1 * 1.15f64.powi(self.lv[i] as i32) }
    fn ser(&self) -> String {
        let l: Vec<String> = self.lv.iter().map(|x| x.to_string()).collect();
        format!("{};{};{};{};{}", self.money, self.total, self.mult, self.used, l.join(","))
    }
    fn de(s: &str) -> Option<Game> {
        let p: Vec<&str> = s.split(';').collect();
        if p.len() < 5 { return None; }
        let mut g = Game::new();
        g.money = p[0].parse().ok()?; g.total = p[1].parse().ok()?;
        g.mult = p[2].parse().ok()?; g.used = p[3].parse().ok()?;
        let mut raw: Vec<u32> = vec![];
        for v in p[4].split(',').take(N) { raw.push(v.parse().ok()?); }
        if raw.len() == OLD_MAP.len() {
            // ancienne sauvegarde : on replace les niveaux (Scam et Trading restent a 0)
            for (o, &n) in OLD_MAP.iter().enumerate() { g.lv[n] = raw[o]; }
        } else {
            for (i, v) in raw.iter().enumerate() { g.lv[i] = *v; }
        }
        Some(g)
    }
}

struct Capi {
    scr: Scr, g: Game, slot: u32, next_ms: f64, w: f32, h: f32, fps: f32, rng: u64, save_t: f32,
    scroll: Scroller, back: Option<Rect>, stonks: Option<Pixmap>, usa: Option<Pixmap>,
    ms_t: f32, usa_t: f32, elon_t: f32, pulse: f32, floats: Vec<(f32, f32, f32, String)>,
    code: String, rand_t: f32, rand_list: Vec<String>, ids: Vec<u32>, store: Store,
    // codes random / stalin / lenin / cheat
    pics: Vec<Option<Pixmap>>, pic_i: usize, pic_t: f32, ussr_t: f32, cheat_t: f32, anticheat_t: f32,
    // taxe toutes les 10 secondes
    tax_clock: f32, tax_t: f32, tax_lost: f64,
}

impl Capi {
    fn new() -> Capi {
        let store = Store::open("saves.conf");
        let ids = sget(&store, "ids").map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or_default();
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(7) | 1;
        Capi {
            scr: Scr::Title, g: Game::new(), slot: 0, next_ms: 1.0, w: 272.0, h: 480.0, fps: 60.0, rng: seed, save_t: 0.0,
            scroll: new_scroller(), back: None, stonks: img("assets/stonks.png"), usa: img("assets/usa.png"),
            ms_t: 0.0, usa_t: 0.0, elon_t: 0.0, pulse: 0.0, floats: vec![],
            code: String::new(), rand_t: 0.0, rand_list: vec![], ids, store,
            pics: PICS.iter().map(|p| img(p)).collect(), pic_i: 0, pic_t: 0.0, ussr_t: 0.0, cheat_t: 0.0, anticheat_t: 0.0,
            tax_clock: 0.0, tax_t: 0.0, tax_lost: 0.0,
        }
    }
    fn rnd(&mut self) -> u64 { let mut x = self.rng; x ^= x << 13; x ^= x >> 7; x ^= x << 17; self.rng = x; x }
    fn rf(&mut self) -> f32 { (self.rnd() % 10000) as f32 / 10000.0 }
    fn save_ids(&mut self) {
        let s: Vec<String> = self.ids.iter().map(|i| i.to_string()).collect();
        sset(&mut self.store, "ids", &s.join(","));
    }
    fn save(&mut self) {
        if self.slot > 0 { let d = self.g.ser(); sset(&mut self.store, &format!("g{}", self.slot), &d); }
    }
    fn start(&mut self, g: Game, slot: u32) {
        self.next_ms = 1.0;
        while self.next_ms <= g.total { self.next_ms *= 10.0; }
        self.g = g; self.slot = slot; self.scr = Scr::Game; self.floats.clear();
        self.clear_pop(); self.tax_clock = 0.0; self.tax_t = 0.0;
        self.scroll = new_scroller(); self.save();
    }
    fn clear_pop(&mut self) {
        self.usa_t = 0.0; self.pic_t = 0.0; self.ussr_t = 0.0; self.cheat_t = 0.0; self.anticheat_t = 0.0;
    }
    fn earn(&mut self, v: f64) {
        self.g.money += v; self.g.total += v;
        if self.g.total >= self.next_ms {
            self.ms_t = 3.0;
            while self.next_ms <= self.g.total { self.next_ms *= 10.0; }
        }
    }
    fn gen_num(&mut self) -> String {
        let n = 1 + (self.rnd() % 24) as usize;
        (0..n).map(|i| { let d = (self.rnd() % 10) as u8; (b'0' + if i == 0 && n > 1 && d == 0 { 1 } else { d }) as char }).collect()
    }
    fn apply_code(&mut self, ctx: &Ctx) {
        let c = self.code.to_lowercase();
        self.code.clear();
        match c.as_str() {
            "elon" => { self.elon_t = 5.0; ctx.toast("ELON prend le contrôle !"); }
            "stonks" => {
                if self.g.used & 1 == 0 { self.g.used |= 1; self.g.mult *= 2.0; ctx.toast("STONKS ! Clic +100%"); }
                else { ctx.toast("Code déjà utilisé"); return; }
            }
            "freedom" => { self.clear_pop(); self.usa_t = 5.0; ctx.toast("FREEDOM !"); }
            // une des images au hasard pendant 5 s
            "random" => {
                self.clear_pop();
                let ok: Vec<usize> = (0..self.pics.len()).filter(|&i| self.pics[i].is_some()).collect();
                if !ok.is_empty() { let k = (self.rnd() % ok.len() as u64) as usize; self.pic_i = ok[k]; }
                self.pic_t = 5.0;
            }
            // drapeau de l'URSS pendant 5 s + perte de 50% de l'argent
            "stalin" | "lenin" => {
                self.clear_pop();
                self.g.money *= 0.5;
                self.ussr_t = 5.0;
            }
            // texte en grand + perte de 100% de l'argent
            "cheat" => {
                self.clear_pop();
                self.g.money = 0.0;
                self.cheat_t = 5.0;
            }
            "anticheat" => {
                self.clear_pop();
                self.g.money = 999999.0;
                self.anticheat_t = 5.0;
            }
            _ => { ctx.toast("Code invalide"); return; }
        }
        self.scr = Scr::Game;
    }

    // ---------- mises en page (points x ctx.scale()) ----------
    fn tbtn(&self, s: f32, i: usize) -> Rect { rc((self.w - 200.0 * s) / 2.0, (190.0 + i as f32 * 58.0) * s, 200.0 * s, 46.0 * s) }
    fn mbtn(&self, s: f32, i: usize) -> Rect { rc((self.w - 200.0 * s) / 2.0, (90.0 + i as f32 * 58.0) * s, 200.0 * s, 46.0 * s) }
    fn menu_btn(&self, s: f32) -> Rect { rc(self.w - 76.0 * s, 4.0 * s, 70.0 * s, 26.0 * s) }
    fn ok_btn(&self, s: f32) -> Rect { rc(36.0 * s, 270.0 * s, self.w - 72.0 * s, 40.0 * s) }
    fn key_rect(&self, s: f32, r: usize, c: usize) -> Rect {
        let kw = (self.w - 12.0 * s) / 10.0;
        let off = (10 - KEYS[r].chars().count()) as f32 * kw / 2.0;
        rc(6.0 * s + off + c as f32 * kw, 150.0 * s + r as f32 * 38.0 * s, kw - 3.0 * s, 34.0 * s)
    }
    fn shown(&self) -> Vec<u32> { self.ids.iter().rev().take(7).cloned().collect() }
    fn row_y(&self, s: f32, k: usize) -> f32 { 70.0 * s + k as f32 * 48.0 * s }

    // ---------- rendu ----------
    // Fond sombre + image centree (optionnelle) + lignes de texte (texte, taille, couleur) au-dessus
    fn overlay(&self, ctx: &Ctx, g: &mut Gfx, name: &str, pm: Option<&Pixmap>, lines: &[(&str, f32, u32)], left: f32, total: f32) {
        let s = ctx.scale();
        let a = (left / 0.5).min(1.0).min((total - left) / 0.3 + 0.01).max(0.0);
        fill(g, rc(0.0, 0.0, self.w, self.h), rgba(0x000000, 0.65 * a));
        let mut y = self.h / 2.0;
        if let Some(p) = pm {
            let (pw, ph) = (p.width() as f32, p.height() as f32);
            y = self.h / 2.0 - ph / 2.0;
            g.image(name, p, (self.w - pw) / 2.0, y, a);
        }
        let tot: f32 = lines.iter().map(|l| l.1 * 1.15 * s).sum();
        let mut cy = if pm.is_some() { y - 14.0 * s - tot } else { self.h / 2.0 - tot / 2.0 };
        for &(txt, sz, col) in lines.iter() {
            let lh = sz * 1.15 * s;
            ctx.text_mid(g, txt, self.w / 2.0, cy + lh / 2.0, &t(sz * s, rgba(col, a)));
            cy += lh;
        }
    }
    // Texte "tax time" (visible en jeu, dans le menu et dans l'ecran des codes)
    fn tax_banner(&self, ctx: &Ctx, g: &mut Gfx) {
        if self.tax_t <= 0.0 { return; }
        let s = ctx.scale();
        let a = (self.tax_t / 0.4).min(1.0);
        let y = self.h * 0.78;
        rfill(g, rc(16.0 * s, y - 36.0 * s, self.w - 32.0 * s, 72.0 * s), 10.0 * s, rgba(0x000000, 0.7 * a));
        ctx.text_mid(g, "tax time", self.w / 2.0, y - 12.0 * s, &t(28.0 * s, rgba(0xFF4040, a)));
        ctx.text_mid(g, &format!("-5%  (-${})", fmt(self.tax_lost)), self.w / 2.0, y + 18.0 * s, &t(14.0 * s, rgba(0xFFFFFF, a)));
    }
    fn r_title(&self, ctx: &Ctx, g: &mut Gfx) {
        let s = ctx.scale();
        for (i, l) in ["CAPITALISM", "SUPREMACY", "CLICKER"].iter().enumerate() {
            ctx.text_mid(g, l, self.w / 2.0, (56.0 + i as f32 * 36.0) * s, &t(30.0 * s, ctx.theme.accent));
        }
        for (i, l) in ["Jouer", "Anciennes parties", "Random numbers"].iter().enumerate() {
            ui::button(ctx, g, self.tbtn(s, i), l, ButtonStyle::Primary, false);
        }
        let st = Text::new(11.0 * s).color(ctx.theme.subtext).center();
        ctx.text_mid(g, "Game made by Ziyad.K", self.w / 2.0, self.h - 40.0 * s, &st);
        ctx.text_mid(g, "for the phone with Alexis.R", self.w / 2.0, self.h - 24.0 * s, &st);
    }
    fn r_saves(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let s = ctx.scale();
        let hd = ui::header(ctx, g, "Anciennes parties", Some("Retour"));
        self.back = hd.back;
        let list = self.shown();
        if list.is_empty() { ctx.text_mid(g, "Aucune sauvegarde", self.w / 2.0, 120.0 * s, &t(14.0 * s, ctx.theme.subtext)); }
        for (k, id) in list.iter().enumerate() {
            let y = self.row_y(s, k);
            let m = sget(&self.store, &format!("g{}", id)).and_then(|d| Game::de(&d)).map(|x| x.money).unwrap_or(0.0);
            let d = format!("${}", fmt(m));
            ui::list_row(ctx, g, rc(6.0 * s, y, self.w - 62.0 * s, 44.0 * s), &format!("Partie #{}", id), Some(d.as_str()), false);
            let tr = rc(self.w - 52.0 * s, y, 46.0 * s, 44.0 * s);
            ui::button(ctx, g, tr, "", ButtonStyle::Danger, false);
            let (cx, cy, w) = (self.w - 29.0 * s, y + 22.0 * s, rgb(0xFFFFFF));
            fill(g, rc(cx - 8.0 * s, cy - 9.0 * s, 16.0 * s, 3.0 * s), w);
            fill(g, rc(cx - 3.0 * s, cy - 12.0 * s, 6.0 * s, 3.0 * s), w);
            rfill(g, rc(cx - 6.0 * s, cy - 5.0 * s, 12.0 * s, 15.0 * s), 2.0 * s, w);
        }
    }
    fn r_random(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let s = ctx.scale();
        let hd = ui::header(ctx, g, "Random numbers", Some("Retour"));
        self.back = hd.back;
        for (i, n) in self.rand_list.iter().enumerate() {
            let y = (70.0 + i as f32 * 32.0) * s;
            let a = 1.0 - i as f32 * 0.07;
            let c = if i == 0 { ctx.theme.accent } else { with_alpha(ctx.theme.text, a) };
            ctx.text_mid(g, n, self.w / 2.0, y, &Text::new(13.0 * s).mono().color(c).center());
            ctx.text_mid(g, &format!("{} chiffre(s)", n.len()), self.w / 2.0, y + 13.0 * s, &Text::new(8.0 * s).color(ctx.theme.subtext).center());
        }
    }
    // Menu en jeu : bouton "Code", Save, Menu principal
    fn r_menu(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let s = ctx.scale();
        let hd = ui::header(ctx, g, "Menu", Some("Retour"));
        self.back = hd.back;
        for (i, l) in ["Code", "Save", "Menu principal"].iter().enumerate() {
            let st = if i == 2 { ButtonStyle::Secondary } else { ButtonStyle::Primary };
            ui::button(ctx, g, self.mbtn(s, i), l, st, false);
        }
        self.tax_banner(ctx, g);
    }
    // Ecran de saisie du code : clavier AZERTY a l'ecran
    fn r_code(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let s = ctx.scale();
        let hd = ui::header(ctx, g, "Code", Some("Retour"));
        self.back = hd.back;
        rfill(g, rc(16.0 * s, 72.0 * s, self.w - 32.0 * s, 44.0 * s), 8.0 * s, ctx.theme.surface);
        let txt = if self.code.is_empty() { "Entre un code".to_string() } else { self.code.clone() };
        let c = if self.code.is_empty() { ctx.theme.subtext } else { ctx.theme.text };
        ctx.text_mid(g, &txt, self.w / 2.0, 94.0 * s, &t(18.0 * s, c));
        for (ch, r, k) in key_list() {
            ui::button(ctx, g, self.key_rect(s, r, k), &ch.to_string(), ButtonStyle::Secondary, false);
        }
        ui::button(ctx, g, self.ok_btn(s), "Valider", ButtonStyle::Primary, false);
        self.tax_banner(ctx, g);
    }
    fn r_game(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let s = ctx.scale();
        let (top, rh) = (215.0 * s, 46.0 * s);
        self.scroll.set_content(N as f32 * rh + 8.0 * s, self.h - top);
        for i in 0..N {
            let y = top + i as f32 * rh - self.scroll.pos;
            let d = format!("x{}   ${}   +{}/s", self.g.lv[i], fmt(self.g.cost(i)), fmt(UP[i].2));
            ui::list_row(ctx, g, rc(6.0 * s, y, self.w - 12.0 * s, rh - 4.0 * s), UP[i].0, Some(d.as_str()), false);
        }
        fill(g, rc(0.0, 0.0, self.w, top - 3.0 * s), ctx.theme.bg);
        let st = Text::new(11.0 * s).mono().color(ctx.theme.subtext).left();
        ctx.text_mid(g, &format!("FPS {:.0}", self.fps), 8.0 * s, 17.0 * s, &st);
        ui::button(ctx, g, self.menu_btn(s), "Menu", ButtonStyle::Secondary, false);
        let (mut sx, mut sy) = (0.0, 0.0);
        if self.elon_t > 0.0 { sx = (self.rf() - 0.5) * 16.0 * s; sy = (self.rf() - 0.5) * 16.0 * s; }
        ctx.text_mid(g, &format!("${}", fmt(self.g.money)), self.w / 2.0 + sx, 46.0 * s + sy, &t(26.0 * s, ctx.theme.text));
        ctx.text_mid(g, &format!("{} / seconde", fmt(self.g.cps())), self.w / 2.0, 68.0 * s, &t(11.0 * s, ctx.theme.subtext));
        // la "cookie" : evolue avec l'argent total
        let si = (0..8).rev().find(|&i| self.g.total >= STAGES[i].0).unwrap_or(0);
        let col = rgb(STAGES[si].2);
        let (cx, cy, r) = (self.w / 2.0 + sx, 138.0 * s + sy, 50.0 * s * (1.0 + 0.1 * self.pulse));
        if si == 0 {
            disc(g, cx, cy, r, shade(col, 0.8)); disc(g, cx, cy, r * 0.85, col);
        } else {
            for l in (0..si.min(3)).rev() {
                let o = l as f32 * 5.0 * s;
                rfill(g, rc(cx - r * 1.1 + o, cy - r * 0.7 - o, r * 2.2, r * 1.4), 8.0 * s, shade(col, 1.0 - l as f32 * 0.12));
            }
        }
        ctx.text_mid(g, "$", cx, cy, &t(r * 0.9, rgba(0x000000, 0.35)));
        ctx.text_mid(g, STAGES[si].1, self.w / 2.0, 200.0 * s, &t(13.0 * s, col));
        for f in &self.floats {
            ctx.text_mid(g, &f.3, f.0, f.1 - f.2 * 50.0 * s, &t(16.0 * s, rgba(0xFFD700, 1.0 - f.2)));
        }
        if self.elon_t > 0.0 {
            for _ in 0..14 {
                let (x, y, w, h, c) = (self.rf() * self.w, self.rf() * self.h, self.rf() * 90.0 * s, self.rf() * 90.0 * s, self.rnd() as u32 & 0xFFFFFF);
                fill(g, rc(x, y, w, h), rgba(c, 0.5));
            }
            for l in ["ELON", "TO THE MOON", "X", "DOGE", "ELON"] {
                let (x, y) = (self.rf() * self.w, self.rf() * self.h);
                ctx.text_mid(g, l, x, y, &t(30.0 * s, rgb(0xFFFFFF)));
            }
        }
        if self.ms_t > 0.0 {
            self.overlay(ctx, g, "stonks", self.stonks.as_ref(), &[("STONKS", 34.0, 0xFFD700), ("CAPITALIZM !", 30.0, 0xFFFFFF)], self.ms_t, 3.0);
        }
        if self.usa_t > 0.0 {
            self.overlay(ctx, g, "usa", self.usa.as_ref(), &[("FREEDOM", 34.0, 0xFFD700), ("USA !", 30.0, 0xFFFFFF)], self.usa_t, 5.0);
        }
        // code "random" : une image au hasard pendant 5 s
        if self.pic_t > 0.0 {
            let p = self.pics.get(self.pic_i).and_then(|x| x.as_ref());
            self.overlay(ctx, g, &format!("p{}", self.pic_i), p, &[], self.pic_t, 5.0);
        }
        // codes "stalin" / "lenin" : drapeau de l'URSS pendant 5 s avec "our money" au-dessus
        if self.ussr_t > 0.0 {
            let p = self.pics.get(FLAG).and_then(|x| x.as_ref());
            self.overlay(ctx, g, &format!("p{}", FLAG), p, &[("our money", 34.0, 0xFFD700)], self.ussr_t, 5.0);
        }
        // code "cheat" : gros texte
        if self.cheat_t > 0.0 {
            self.overlay(ctx, g, "cheat", None, &[("uncle sam", 34.0, 0xFF3030), ("is", 34.0, 0xFF3030), ("desapointed", 34.0, 0xFF3030)], self.cheat_t, 5.0);
        }
        if self.anticheat_t > 0.0 {
            self.overlay(ctx, g, "cheat", None, &[("uncle sam", 34.0, 0xFF3030), ("is", 34.0, 0xFF3030), ("proud", 34.0, 0xFF3030)], self.anticheat_t, 5.0);
        }
        self.tax_banner(ctx, g);
    }
}

impl App for Capi {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let (w, h) = gsize(g);
        self.w = w; self.h = h;
        g.clear(ctx.theme.bg);
        self.back = None;
        match self.scr {
            Scr::Title => self.r_title(ctx, g),
            Scr::Saves => self.r_saves(ctx, g),
            Scr::Game => self.r_game(ctx, g),
            Scr::Menu => self.r_menu(ctx, g),
            Scr::Code => self.r_code(ctx, g),
            Scr::Random => self.r_random(ctx, g),
        }
    }

    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        if dt > 0.0 { self.fps = self.fps * 0.92 + (1.0 / dt) * 0.08; }
        match self.scr {
            Scr::Game | Scr::Menu | Scr::Code => {
                let c = self.g.cps();
                self.earn(c * dt as f64);
                self.ms_t = (self.ms_t - dt).max(0.0);
                self.usa_t = (self.usa_t - dt).max(0.0);
                self.elon_t = (self.elon_t - dt).max(0.0);
                self.pic_t = (self.pic_t - dt).max(0.0);
                self.ussr_t = (self.ussr_t - dt).max(0.0);
                self.cheat_t = (self.cheat_t - dt).max(0.0);
                self.anticheat_t = (self.anticheat_t - dt).max(0.0);
                self.tax_t = (self.tax_t - dt).max(0.0);
                self.pulse = (self.pulse - dt * 6.0).max(0.0);
                for f in self.floats.iter_mut() { f.2 += dt; }
                self.floats.retain(|f| f.2 < 1.0);
                // "tax time" : toutes les 10 secondes, -5% de la fortune
                self.tax_clock += dt;
                if self.tax_clock >= 10.0 {
                    self.tax_clock -= 10.0;
                    self.tax_lost = self.g.money * 0.05;
                    self.g.money -= self.tax_lost;
                    self.tax_t = 2.5;
                }
                self.save_t += dt;
                if self.save_t > 5.0 { self.save_t = 0.0; self.save(); }
                self.scroll.update(dt);
                ctx.animate();
            }
            Scr::Random => {
                self.rand_t += dt;
                if self.rand_list.is_empty() || self.rand_t >= 2.0 {
                    self.rand_t = 0.0;
                    let n = self.gen_num();
                    self.rand_list.insert(0, n);
                    self.rand_list.truncate(12);
                }
                ctx.animate();
            }
            _ => {}
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        let s = ctx.scale();
        if self.scr == Scr::Game {
            match ev {
                Event::Down { .. } => self.scroll.stop(),
                Event::Drag { dy, .. } | Event::Wheel { dy } => self.scroll.drag(dy),
                Event::DragEnd { vy, .. } => self.scroll.fling(vy),
                Event::Tap { x, y } => {
                    let (top, rh) = (215.0 * s, 46.0 * s);
                    let (cx, cy) = (self.w / 2.0, 138.0 * s);
                    if hit(&self.menu_btn(s), x, y) { self.save(); self.code.clear(); self.scr = Scr::Menu; }
                    else if (x - cx).powi(2) + (y - cy).powi(2) < (62.0 * s).powi(2) {
                        let v = self.g.click();
                        self.earn(v);
                        self.pulse = 1.0;
                        self.floats.push((x, y, 0.0, format!("+{}", fmt(v))));
                    } else if y > top {
                        let i = ((y - top + self.scroll.pos) / rh) as usize;
                        if i < N {
                            let c = self.g.cost(i);
                            if self.g.money >= c { self.g.money -= c; self.g.lv[i] += 1; }
                            else { ctx.toast("Pas assez d'argent !"); }
                        }
                    }
                }
                _ => {}
            }
            return;
        }
        match (self.scr, ev) {
            (Scr::Title, Event::Tap { x, y }) => {
                if hit(&self.tbtn(s, 0), x, y) {
                    let id = self.ids.iter().max().map_or(1, |m| m + 1);
                    self.ids.push(id);
                    self.save_ids();
                    self.start(Game::new(), id);
                } else if hit(&self.tbtn(s, 1), x, y) { self.scr = Scr::Saves; }
                else if hit(&self.tbtn(s, 2), x, y) { self.rand_list.clear(); self.scr = Scr::Random; }
            }
            (Scr::Saves, Event::Tap { x, y }) => {
                if self.back.map_or(false, |b| hit(&b, x, y)) { self.scr = Scr::Title; return; }
                for (k, id) in self.shown().into_iter().enumerate() {
                    let ry = self.row_y(s, k);
                    if hit(&rc(self.w - 52.0 * s, ry, 46.0 * s, 44.0 * s), x, y) {
                        sdel(&mut self.store, &format!("g{}", id));
                        self.ids.retain(|&i| i != id);
                        self.save_ids();
                        break;
                    } else if hit(&rc(6.0 * s, ry, self.w - 62.0 * s, 44.0 * s), x, y) {
                        if let Some(g) = sget(&self.store, &format!("g{}", id)).and_then(|d| Game::de(&d)) { self.start(g, id); }
                        break;
                    }
                }
            }
            (Scr::Random, Event::Tap { x, y }) => {
                if self.back.map_or(false, |b| hit(&b, x, y)) { self.scr = Scr::Title; }
            }
            (Scr::Menu, Event::Tap { x, y }) => {
                if self.back.map_or(false, |b| hit(&b, x, y)) { self.scr = Scr::Game; return; }
                if hit(&self.mbtn(s, 0), x, y) { self.code.clear(); self.scr = Scr::Code; }
                else if hit(&self.mbtn(s, 1), x, y) { self.save(); ctx.toast("Partie sauvegardée !"); }
                else if hit(&self.mbtn(s, 2), x, y) { self.save(); self.scr = Scr::Title; }
            }
            // Saisie du code : uniquement en appuyant sur les touches AZERTY a l'ecran
            (Scr::Code, Event::Tap { x, y }) => {
                if self.back.map_or(false, |b| hit(&b, x, y)) { self.scr = Scr::Menu; return; }
                for (ch, r, k) in key_list() {
                    if hit(&self.key_rect(s, r, k), x, y) {
                        if ch == '<' { self.code.pop(); } else if self.code.len() < 12 { self.code.push(ch); }
                    }
                }
                if hit(&self.ok_btn(s), x, y) { self.apply_code(ctx); }
            }
            _ => {}
        }
    }

    fn on_back(&mut self, _ctx: &Ctx) -> bool {
        match self.scr {
            Scr::Title => false,
            Scr::Game => { self.save(); self.scr = Scr::Title; true }
            Scr::Menu => { self.scr = Scr::Game; true }
            Scr::Code => { self.scr = Scr::Menu; true }
            _ => { self.scr = Scr::Title; true }
        }
    }
}

fn main() {
    run(Config::new("Capitalism Supremacy Clicker"), Capi::new());
}
