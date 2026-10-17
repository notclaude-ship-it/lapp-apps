// Conquis Cliqueur : jeu cliqueur infini pour lapp (écran portrait 272x480).
//
// - Échap (ou bouton ≡ en haut à droite) : menu (Reprendre / Entrer un code / Quitter)
// - Code BUG  : vidéo plein écran, impossible à fermer avant la fin.
// - Code TUNE : interface secrète qui demande un 2e code
//     INFINI -> château en or      (clics et production ×4 pour toujours)
//     DIAMEN -> château en diamant (clics et production ×1000 pour toujours)
// - Onglet Magie : sorts (mana) et améliorations magiques à débloquer.
// - Boutique infinie : 8 unités puis des « Dimensions » sans fin.
//
// Sur PC : cargo run
//          cargo run -- --screenshot out.png [--page menu|code|tune|video|magic] [--vt 8] [--skin 1|2] [--demo]

mod audio;

use lapp_sdk::prelude::*;
use lapp_sdk::{arg_flag, arg_value, load_png};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const VIDEO_FPS: f32 = 6.0;
const VIDEO_FRAMES: usize = 232;
const GROWTH: f64 = 1.15;
const MAX_CODE: usize = 16;
const OFFLINE_CAP: f64 = 8.0 * 3600.0;
const TIERS: usize = 60;

#[derive(Clone)]
struct Tier {
    name: String,
    base: f64,
    rate: f64,
}

struct MagicDef {
    id: &'static str,
    name: &'static str,
    spell: bool,
    cost: f64,
    req: f64,
    mana: f32,
    dur: f32,
    desc: &'static str,
}

static MAGIC: [MagicDef; 8] = [
    MagicDef { id: "frenesie", name: "Frénésie", spell: true, cost: 500.0, req: 250.0, mana: 40.0, dur: 15.0, desc: "Clics ×7 pendant 15 s" },
    MagicDef { id: "pluie", name: "Pluie d’or", spell: true, cost: 5_000.0, req: 2_500.0, mana: 30.0, dur: 0.0, desc: "+60 s de production" },
    MagicDef { id: "res1", name: "Réservoir de mana I", spell: false, cost: 2_000.0, req: 1_000.0, mana: 0.0, dur: 0.0, desc: "Mana max +50" },
    MagicDef { id: "source", name: "Source arcanique", spell: false, cost: 20_000.0, req: 10_000.0, mana: 0.0, dur: 0.0, desc: "Le mana revient 2× plus vite" },
    MagicDef { id: "benediction", name: "Bénédiction", spell: true, cost: 50_000.0, req: 25_000.0, mana: 60.0, dur: 20.0, desc: "Production ×3 pendant 20 s" },
    MagicDef { id: "res2", name: "Réservoir de mana II", spell: false, cost: 200_000.0, req: 100_000.0, mana: 0.0, dur: 0.0, desc: "Mana max +100" },
    MagicDef { id: "main", name: "Main de mage", spell: false, cost: 1_000_000.0, req: 500_000.0, mana: 0.0, dur: 0.0, desc: "Clic +10 % de la prod/s" },
    MagicDef { id: "grimoire", name: "Grimoire ancien", spell: false, cost: 50_000_000.0, req: 25_000_000.0, mana: 0.0, dur: 0.0, desc: "Toute la production ×2" },
];

fn make_tiers() -> Vec<Tier> {
    let fixed: [(&str, f64, f64); 8] = [
        ("Éclaireur", 15.0, 0.1),
        ("Soldat", 100.0, 1.0),
        ("Archer", 1_100.0, 8.0),
        ("Cavalier", 12_000.0, 47.0),
        ("Catapulte", 130_000.0, 260.0),
        ("Château", 1_400_000.0, 1_400.0),
        ("Armée", 20_000_000.0, 7_800.0),
        ("Empire", 330_000_000.0, 44_000.0),
    ];
    let mut v: Vec<Tier> = fixed.iter().map(|(n, c, r)| Tier { name: n.to_string(), base: *c, rate: *r }).collect();
    let mut i = 1;
    while v.len() < TIERS {
        let last = v.last().unwrap().clone();
        v.push(Tier { name: format!("Dimension {i}"), base: last.base * 13.0, rate: last.rate * 6.5 });
        i += 1;
    }
    v
}

fn fmt(n: f64) -> String {
    if !n.is_finite() {
        return "∞".into();
    }
    if n < 1000.0 {
        return format!("{:.0}", n.floor());
    }
    const SUF: [&str; 12] = ["", "k", "M", "B", "T", "Qa", "Qi", "Sx", "Sp", "Oc", "No", "Dc"];
    let e = (n.log10() / 3.0).floor() as usize;
    if e < SUF.len() {
        format!("{:.2}{}", n / 1000f64.powi(e as i32), SUF[e])
    } else {
        format!("{:.2e}", n)
    }
}

fn fmt_rate(n: f64) -> String {
    if n < 100.0 {
        format!("{:.1}", n)
    } else {
        fmt(n)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Game,
    Menu,
    Code,
    Tune,
    Video,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Shop,
    Magic,
}

struct Floater {
    x: f32,
    y: f32,
    t: f32,
    txt: String,
}

#[derive(Clone, Copy, PartialEq)]
enum Kb {
    Ch(char),
    Del,
}

struct Conquis {
    store: Store,
    tiers: Vec<Tier>,
    owned: Vec<u32>,
    pts: f64,
    total: f64,
    click_lvl: u32,
    clicks: u64,
    skin: u8, // 0 pierre, 1 or (×4), 2 diamant (×1000)
    mana: f32,
    mg: Vec<bool>,
    frenzy_left: f32,
    bless_left: f32,
    tab: Tab,
    screen: Screen,
    code: String,
    vt: f32,
    last: Instant,
    since_save: f32,
    scroll: Scroller,
    floats: Vec<Floater>,
    pop: f32,
    pending_toast: Option<String>,
    sound: Option<audio::Sound>,
    sound_wait: f32,
}

struct Layout {
    head_h: f32,
    shop_top: f32,
    tab_h: f32,
    list_top: f32,
    row_h: f32,
    cx: f32,
    cy: f32,
    r: f32,
    menu_btn: Rect,
}

impl Conquis {
    fn new(store: Store) -> Conquis {
        let tiers = make_tiers();
        let mut app = Conquis {
            store,
            owned: vec![0; tiers.len()],
            tiers,
            pts: 0.0,
            total: 0.0,
            click_lvl: 0,
            clicks: 0,
            skin: 0,
            mana: 0.0,
            mg: vec![false; MAGIC.len()],
            frenzy_left: 0.0,
            bless_left: 0.0,
            tab: Tab::Shop,
            screen: Screen::Game,
            code: String::new(),
            vt: 0.0,
            last: Instant::now(),
            since_save: 0.0,
            scroll: Scroller::new(),
            floats: Vec::new(),
            pop: 0.0,
            pending_toast: None,
            sound: None,
            sound_wait: 0.0,
        };
        app.load();
        app
    }

    fn load(&mut self) {
        let Some(s) = self.store.get("state") else { return };
        let s = s.to_string();
        let mut it = s.split(';');
        let mut next = || it.next().unwrap_or("");
        self.pts = next().parse().unwrap_or(0.0);
        self.total = next().parse().unwrap_or(0.0);
        self.click_lvl = next().parse().unwrap_or(0);
        self.clicks = next().parse().unwrap_or(0);
        let saved_at: f64 = next().parse().unwrap_or(0.0);
        for (i, n) in next().split(',').enumerate() {
            if i < self.owned.len() {
                self.owned[i] = n.parse().unwrap_or(0);
            }
        }
        self.skin = next().parse::<u8>().unwrap_or(0).min(2);
        self.mana = next().parse().unwrap_or(0.0);
        for (i, c) in next().chars().enumerate() {
            if i < self.mg.len() {
                self.mg[i] = c == '1';
            }
        }
        let away = (now_secs() - saved_at).clamp(0.0, OFFLINE_CAP);
        let gain = self.rate() * away;
        if gain >= 1.0 && away > 30.0 {
            self.pts += gain;
            self.total += gain;
            self.pending_toast = Some(format!("Hors-ligne : +{} territoires", fmt(gain)));
        }
    }

    fn save(&mut self) {
        let owned = self.owned.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",");
        let mg: String = self.mg.iter().map(|b| if *b { '1' } else { '0' }).collect();
        let s = format!(
            "{};{};{};{};{};{};{};{};{}",
            self.pts, self.total, self.click_lvl, self.clicks, now_secs(), owned, self.skin, self.mana, mg
        );
        let _ = self.store.set("state", s);
    }

    // ---- règles du jeu ------------------------------------------------------------------
    fn has(&self, id: &str) -> bool {
        MAGIC.iter().position(|m| m.id == id).map_or(false, |i| self.mg[i])
    }
    fn perm_mult(&self) -> f64 {
        match self.skin {
            2 => 1000.0,
            1 => 4.0,
            _ => 1.0,
        }
    }
    fn prod_mult(&self) -> f64 {
        let mut m = self.perm_mult();
        if self.has("grimoire") {
            m *= 2.0;
        }
        if self.bless_left > 0.0 {
            m *= 3.0;
        }
        m
    }
    fn max_mana(&self) -> f32 {
        100.0 + if self.has("res1") { 50.0 } else { 0.0 } + if self.has("res2") { 100.0 } else { 0.0 }
    }
    fn mana_regen(&self) -> f32 {
        if self.has("source") {
            2.0
        } else {
            1.0
        }
    }
    fn rate(&self) -> f64 {
        let base: f64 = self.tiers.iter().zip(&self.owned).map(|(t, n)| t.rate * *n as f64).sum();
        base * self.prod_mult()
    }
    fn click_power(&self) -> f64 {
        let mut p = 2f64.powi(self.click_lvl as i32) * self.perm_mult();
        if self.frenzy_left > 0.0 {
            p *= 7.0;
        }
        if self.has("main") {
            p += self.rate() * 0.1;
        }
        p
    }
    fn click_cost(&self) -> f64 {
        50.0 * 4f64.powi(self.click_lvl as i32)
    }
    fn tier_cost(&self, i: usize) -> f64 {
        (self.tiers[i].base * GROWTH.powi(self.owned[i] as i32)).ceil()
    }

    fn layout(&self, ctx: &Ctx) -> Layout {
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let head_h = 44.0 * s;
        let shop_top = (h * 0.50).round();
        let tab_h = 32.0 * s;
        let list_top = shop_top + tab_h + if self.tab == Tab::Magic { 26.0 * s } else { 0.0 };
        let zone_top = head_h + 74.0 * s;
        let zone_h = shop_top - zone_top;
        let r = (zone_h / 2.0 - 6.0 * s).min(62.0 * s).max(20.0 * s);
        Layout {
            head_h,
            shop_top,
            tab_h,
            list_top,
            row_h: 54.0 * s,
            cx: w / 2.0,
            cy: zone_top + zone_h / 2.0,
            r,
            menu_btn: Rect::new(w - 48.0 * s, 0.0, 48.0 * s, head_h),
        }
    }

    fn do_click(&mut self, ctx: &Ctx) {
        let l = self.layout(ctx);
        let s = ctx.scale();
        let gain = self.click_power();
        self.pts += gain;
        self.total += gain;
        self.clicks += 1;
        self.pop = 1.0;
        let off = ((self.clicks % 7) as f32 - 3.0) * 9.0 * s;
        if self.floats.len() < 10 {
            self.floats.push(Floater { x: l.cx + off, y: l.cy - l.r * 0.5, t: 0.0, txt: format!("+{}", fmt(gain)) });
        }
        ctx.animate();
    }

    fn buy(&mut self, ctx: &Ctx, row: usize) {
        if row == 0 {
            let c = self.click_cost();
            if self.pts >= c {
                self.pts -= c;
                self.click_lvl += 1;
                self.save();
            } else {
                ctx.toast("Pas assez de territoires");
            }
        } else if row - 1 < self.tiers.len() {
            let i = row - 1;
            let c = self.tier_cost(i);
            if self.pts >= c {
                self.pts -= c;
                self.owned[i] += 1;
                self.save();
            } else {
                ctx.toast("Pas assez de territoires");
            }
        }
    }

    fn magic_tap(&mut self, ctx: &Ctx, k: usize) {
        if k >= MAGIC.len() {
            return;
        }
        let m = &MAGIC[k];
        if !self.mg[k] {
            if self.total < m.req {
                ctx.toast(format!("Verrouillé : {} territoires conquis", fmt(m.req)));
            } else if self.pts < m.cost {
                ctx.toast("Pas assez de territoires");
            } else {
                self.pts -= m.cost;
                self.mg[k] = true;
                self.save();
                ctx.toast(format!("Débloqué : {}", m.name));
            }
            return;
        }
        if !m.spell {
            ctx.toast("Déjà actif");
            return;
        }
        if self.mana < m.mana {
            ctx.toast("Pas assez de mana");
            return;
        }
        match m.id {
            "frenesie" => {
                if self.frenzy_left > 0.0 {
                    ctx.toast("Déjà en cours");
                    return;
                }
                self.mana -= m.mana;
                self.frenzy_left = m.dur;
                ctx.toast("Frénésie ! Clics ×7");
            }
            "benediction" => {
                if self.bless_left > 0.0 {
                    ctx.toast("Déjà en cours");
                    return;
                }
                self.mana -= m.mana;
                self.bless_left = m.dur;
                ctx.toast("Bénédiction ! Production ×3");
            }
            "pluie" => {
                self.mana -= m.mana;
                let g = (self.rate() * 60.0).max(self.click_power() * 50.0);
                self.pts += g;
                self.total += g;
                ctx.toast(format!("Pluie d’or : +{}", fmt(g)));
            }
            _ => {}
        }
    }

    fn submit_code(&mut self, ctx: &Ctx) {
        let code = self.code.trim().to_uppercase();
        self.code.clear();
        match self.screen {
            Screen::Code => {
                if code == "BUG" {
                    self.save();
                    self.screen = Screen::Video;
                    self.vt = 0.0;
                    self.sound_wait = 0.0;
                    self.sound = Some(audio::Sound::start_video());
                    self.last = Instant::now();
                } else if code == "TUNE" {
                    self.screen = Screen::Tune;
                } else if code.is_empty() {
                    ctx.toast("Entre un code");
                } else {
                    ctx.toast("Code invalide");
                }
            }
            Screen::Tune => {
                if code == "INFINI" {
                    if self.skin >= 1 {
                        ctx.toast(if self.skin == 2 { "Tu as déjà le diamant (×1000)" } else { "Le château est déjà en or" });
                    } else {
                        self.skin = 1;
                        self.save();
                        self.screen = Screen::Game;
                        ctx.toast("Château en or ! Clics et production ×4 pour toujours");
                    }
                } else if code == "DIAMEN" {
                    if self.skin == 2 {
                        ctx.toast("Le château est déjà en diamant");
                    } else {
                        self.skin = 2;
                        self.save();
                        self.screen = Screen::Game;
                        ctx.toast("Château en diamant ! ×1000 pour toujours");
                    }
                } else if code.is_empty() {
                    ctx.toast("Entre le 2e code");
                } else {
                    ctx.toast("Code invalide");
                }
            }
            _ => {}
        }
    }

    // ---- clavier à l'écran -------------------------------------------------------------
    fn kb_keys(&self, ctx: &Ctx) -> Vec<(Rect, Kb)> {
        let s = ctx.scale();
        let w = ctx.width();
        let head_h = 44.0 * s;
        let gap = 3.0 * s;
        let kw = (w - 12.0 * s - gap * 9.0) / 10.0;
        let kh = 42.0 * s;
        let top = head_h + 86.0 * s;
        let rows = ["1234567890", "AZERTYUIOP", "QSDFGHJKLM", "WXCVBN"];
        let mut v = Vec::new();
        for (ri, row) in rows.iter().enumerate() {
            let y = top + ri as f32 * (kh + gap);
            for (ci, c) in row.chars().enumerate() {
                v.push((Rect::new(6.0 * s + ci as f32 * (kw + gap), y, kw, kh), Kb::Ch(c)));
            }
        }
        let y = top + 3.0 * (kh + gap);
        v.push((Rect::new(6.0 * s + 6.0 * (kw + gap), y, kw * 2.0 + gap, kh), Kb::Del));
        v
    }
    fn submit_rect(&self, ctx: &Ctx) -> Rect {
        let s = ctx.scale();
        let y = 44.0 * s + 86.0 * s + 4.0 * (42.0 * s + 3.0 * s) + 10.0 * s;
        Rect::new(16.0 * s, y, ctx.width() - 32.0 * s, 46.0 * s)
    }

    // ---- menu ----------------------------------------------------------------------------
    fn menu_buttons(&self, ctx: &Ctx) -> (Rect, [Rect; 3]) {
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let pw = w * 0.82;
        let bh = 46.0 * s;
        let gap = 12.0 * s;
        let ph = 56.0 * s + 3.0 * bh + 2.0 * gap + 16.0 * s;
        let px = (w - pw) / 2.0;
        let py = (h - ph) / 2.0;
        let bx = px + 16.0 * s;
        let bw = pw - 32.0 * s;
        let y0 = py + 56.0 * s;
        let b = [
            Rect::new(bx, y0, bw, bh),
            Rect::new(bx, y0 + bh + gap, bw, bh),
            Rect::new(bx, y0 + 2.0 * (bh + gap), bw, bh),
        ];
        (Rect::new(px, py, pw, ph), b)
    }

    // ---- lignes de liste -----------------------------------------------------------------
    /// (titre, détail, texte de droite, mis en avant, verrouillé)
    fn shop_row(&self, row: usize) -> (String, String, String, bool, bool) {
        if row == 0 {
            let c = self.click_cost();
            ("Épée affûtée".into(), format!("Clic ×2 · coût {}", fmt(c)), format!("Niv {}", self.click_lvl), self.pts >= c, false)
        } else {
            let i = row - 1;
            let c = self.tier_cost(i);
            (
                self.tiers[i].name.clone(),
                format!("+{}/s · coût {}", fmt_rate(self.tiers[i].rate * self.prod_mult()), fmt(c)),
                self.owned[i].to_string(),
                self.pts >= c,
                false,
            )
        }
    }

    fn magic_row(&self, k: usize) -> (String, String, String, bool, bool) {
        let m = &MAGIC[k];
        let own = self.mg[k];
        if !own && self.total < m.req {
            return (m.name.into(), format!("Verrouillé · {} territoires", fmt(m.req)), "🔒".into(), false, true);
        }
        if !own {
            return (m.name.into(), format!("{} · coût {}", m.desc, fmt(m.cost)), "Débloquer".into(), self.pts >= m.cost, false);
        }
        if m.spell {
            let rem = match m.id {
                "frenesie" => self.frenzy_left,
                "benediction" => self.bless_left,
                _ => 0.0,
            };
            let (right, ok) = if rem > 0.0 { (format!("{} s", rem.ceil()), true) } else { ("Lancer".into(), self.mana >= m.mana) };
            return (m.name.into(), format!("{} · {} mana", m.desc, m.mana), right, ok, false);
        }
        (m.name.into(), m.desc.into(), "OK".into(), true, false)
    }

    fn skin_colors(&self) -> (Color, Color, Color, Color) {
        match self.skin {
            2 => (rgb(0x2A9FC0), rgb(0xB9F2FF), rgb(0x17607A), rgb(0xE9FBFF)),
            1 => (rgb(0xB8860B), rgb(0xF2C94C), rgb(0x6B4F0A), rgb(0xF2C94C)),
            _ => (rgb(0x5B5F68), rgb(0xA3A7B0), rgb(0x454850), rgb(0xC5C8CF)),
        }
    }

    // ---- dessin --------------------------------------------------------------------------
    fn draw_game(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = ctx.theme;
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let l = self.layout(ctx);
        g.clear(t.bg);

        // liste (dessinée d'abord : le haut de l'écran la recouvre)
        let rows = if self.tab == Tab::Shop { self.tiers.len() + 1 } else { MAGIC.len() };
        self.scroll.set_content(rows as f32 * l.row_h + 8.0 * s, h - l.list_top);
        for row in 0..rows {
            let y = l.list_top + row as f32 * l.row_h - self.scroll.pos;
            if y + l.row_h < l.list_top || y > h {
                continue;
            }
            let (title, detail, right, ok, locked) = if self.tab == Tab::Shop { self.shop_row(row) } else { self.magic_row(row) };
            let r = Rect::new(8.0 * s, y + 3.0 * s, w - 16.0 * s, l.row_h - 6.0 * s);
            g.rrect(r.x, r.y, r.w, r.h, 10.0 * s, if ok { t.surface } else { shade(t.surface, 0.96) });
            let tc = if ok { t.text } else { t.subtext };
            ctx.text_mid(g, &title, r.x + 12.0 * s, r.y + r.h * 0.32, &Text::new(15.0 * s).bold().color(tc).left().max_width(r.w * 0.62));
            let dc = if ok { t.accent } else { t.subtext };
            ctx.text_mid(g, &detail, r.x + 12.0 * s, r.y + r.h * 0.70, &Text::new(11.5 * s).color(dc).left().max_width(r.w * 0.74));
            let big = right.chars().count() <= 3 && !locked;
            let rs = Text::new(if big { 20.0 * s } else { 13.0 * s }).bold().color(if ok && !big { t.accent } else { t.subtext }).right().max_width(r.w * 0.26);
            ctx.text_mid(g, &right, r.right() - 12.0 * s, r.y + r.h / 2.0, &rs);
        }

        // zone du haut
        g.rect(0.0, 0.0, w, l.list_top, t.bg);
        let head = ui::header(ctx, g, "Conquis Cliqueur", None);
        for k in 0..3 {
            g.rrect(l.menu_btn.x + 14.0 * s, 14.0 * s + k as f32 * 7.0 * s, 20.0 * s, 3.0 * s, 1.5 * s, t.text);
        }
        let big = Text::new(30.0 * s).bold().color(t.text).center().max_width(w - 24.0 * s);
        ctx.text_mid(g, &fmt(self.pts), w / 2.0, head.height + 20.0 * s, &big);
        ctx.text_mid(g, "territoires conquis", w / 2.0, head.height + 41.0 * s, &Text::new(12.0 * s).color(t.subtext).center());
        let mult = if self.perm_mult() > 1.0 { format!(" · ×{}", self.perm_mult()) } else { String::new() };
        ctx.text_mid(
            g,
            &format!("{} par seconde{}", fmt_rate(self.rate()), mult),
            w / 2.0,
            head.height + 58.0 * s,
            &Text::new(13.0 * s).bold().color(t.accent).center(),
        );

        // gros bouton
        let k = 1.0 - 0.07 * self.pop;
        let r = l.r * k;
        let (outer, inner, body, door) = self.skin_colors();
        g.circle(l.cx, l.cy + 3.0 * s, r, rgba(0x000000, 0.15));
        g.circle(l.cx, l.cy, r, outer);
        g.circle(l.cx, l.cy, r * 0.9, inner);
        draw_castle(g, l.cx, l.cy, r * 1.2, body, door);
        if self.skin == 2 {
            g.circle(l.cx - r * 0.45, l.cy - r * 0.5, r * 0.12, rgba(0xFFFFFF, 0.8));
            g.circle(l.cx + r * 0.5, l.cy - r * 0.2, r * 0.07, rgba(0xFFFFFF, 0.7));
        }

        // onglets
        g.rect(0.0, l.shop_top, w, l.tab_h, t.surface);
        g.rect(0.0, l.shop_top, w, 1.0, t.line);
        for (i, (name, tab)) in [("Boutique", Tab::Shop), ("Magie", Tab::Magic)].iter().enumerate() {
            let on = self.tab == *tab;
            let cx = w / 4.0 + i as f32 * w / 2.0;
            ctx.text_mid(g, name, cx, l.shop_top + l.tab_h / 2.0 - 1.0 * s, &Text::new(14.0 * s).bold().color(if on { t.accent } else { t.subtext }).center());
            if on {
                g.rect(i as f32 * w / 2.0 + 12.0 * s, l.shop_top + l.tab_h - 3.0 * s, w / 2.0 - 24.0 * s, 3.0 * s, t.accent);
            }
        }
        if self.tab == Tab::Magic {
            let y = l.shop_top + l.tab_h;
            g.rect(0.0, y, w, 26.0 * s, t.bg);
            let mm = self.max_mana();
            let bw = w - 24.0 * s - 92.0 * s;
            g.rrect(12.0 * s, y + 8.0 * s, bw, 10.0 * s, 5.0 * s, t.line);
            let fill = (self.mana / mm).clamp(0.0, 1.0) * bw;
            if fill > 1.0 {
                g.rrect(12.0 * s, y + 8.0 * s, fill.max(10.0 * s), 10.0 * s, 5.0 * s, rgb(0x7C4DFF));
            }
            ctx.text_mid(g, &format!("Mana {}/{}", self.mana.floor(), mm), w - 12.0 * s, y + 13.0 * s, &Text::new(12.0 * s).bold().color(rgb(0x7C4DFF)).right());
        }

        // "+N" flottants
        let fs = Text::new(16.0 * s).bold().color(rgb(0x9A6B00)).center();
        for f in &self.floats {
            ctx.text_mid(g, &f.txt, f.x, f.y, &fs);
        }
    }

    fn draw_code(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = ctx.theme;
        let s = ctx.scale();
        let w = ctx.width();
        g.clear(t.bg);
        let title = if self.screen == Screen::Tune { "Interface secrète" } else { "Entrer un code" };
        ui::header(ctx, g, title, Some("Retour"));
        let box_r = Rect::new(16.0 * s, 44.0 * s + 16.0 * s, w - 32.0 * s, 40.0 * s);
        g.rrect(box_r.x, box_r.y, box_r.w, box_r.h, 10.0 * s, t.surface);
        let (cx, cy) = box_r.center();
        if self.code.is_empty() {
            let ph = if self.screen == Screen::Tune { "2E CODE" } else { "CODE" };
            ctx.text_mid(g, ph, cx, cy, &Text::new(22.0 * s).bold().color(t.line).center());
        } else {
            ctx.text_mid(g, &self.code, cx, cy, &Text::new(22.0 * s).mono().bold().color(t.text).center().max_width(box_r.w - 16.0 * s));
        }
        for (r, k) in self.kb_keys(ctx) {
            g.rrect(r.x, r.y + 1.0 * s, r.w, r.h, 6.0 * s, rgba(0x000000, 0.12));
            g.rrect(r.x, r.y, r.w, r.h, 6.0 * s, t.surface);
            let (label, sz) = match k {
                Kb::Ch(c) => (c.to_string(), 16.0),
                Kb::Del => ("Effacer".to_string(), 13.0),
            };
            let (x, y) = r.center();
            ctx.text_mid(g, &label, x, y, &Text::new(sz * s).bold().color(t.text).center());
        }
        ui::button(ctx, g, self.submit_rect(ctx), "Valider", ButtonStyle::Primary, false);
    }

    fn draw_menu(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = ctx.theme;
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        self.draw_game(ctx, g);
        g.rect(0.0, 0.0, w, h, rgba(0x000000, 0.55));
        let (panel, b) = self.menu_buttons(ctx);
        g.rrect(panel.x, panel.y, panel.w, panel.h, 16.0 * s, t.bg);
        ctx.text_mid(g, "Menu", w / 2.0, panel.y + 28.0 * s, &Text::new(20.0 * s).bold().color(t.text).center());
        ui::button(ctx, g, b[0], "Reprendre", ButtonStyle::Primary, false);
        ui::button(ctx, g, b[1], "Entrer un code", ButtonStyle::Secondary, false);
        ui::button(ctx, g, b[2], "Quitter", ButtonStyle::Danger, false);
    }

    fn draw_video(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        g.clear(rgb(0x000000));
        let idx = ((self.vt * VIDEO_FPS) as usize).min(VIDEO_FRAMES - 1);
        let key = format!("vid{idx}");
        let (pw, ph) = g.img(&key, &|| {
            load_png(&format!("assets/frames/f{:04}.png", idx + 1)).unwrap_or_else(|| Pixmap::new(2, 2).unwrap())
        });
        g.blit(&key, (w - pw as f32) / 2.0, (h - ph as f32) / 2.0, 1.0);
        let p = (self.vt * VIDEO_FPS / VIDEO_FRAMES as f32).clamp(0.0, 1.0);
        g.rect(0.0, h - 4.0 * s, w, 4.0 * s, rgba(0xFFFFFF, 0.15));
        g.rect(0.0, h - 4.0 * s, w * p, 4.0 * s, rgb(0xFF3B30));
        ctx.animate();
    }
}

fn now_secs() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}

fn draw_castle(g: &mut Gfx, cx: f32, cy: f32, u: f32, c: Color, door: Color) {
    g.rect(cx - 0.4 * u, cy - 0.1 * u, 0.8 * u, 0.5 * u, c);
    for dx in [-0.4f32, -0.1, 0.2] {
        g.rect(cx + dx * u, cy - 0.28 * u, 0.2 * u, 0.2 * u, c);
    }
    g.rrect(cx - 0.1 * u, cy + 0.14 * u, 0.2 * u, 0.26 * u, 0.1 * u, door);
}

impl App for Conquis {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        if let Some(m) = self.pending_toast.take() {
            ctx.toast(m);
        }
        match self.screen {
            Screen::Game => self.draw_game(ctx, g),
            Screen::Menu => self.draw_menu(ctx, g),
            Screen::Code | Screen::Tune => self.draw_code(ctx, g),
            Screen::Video => self.draw_video(ctx, g),
        }
    }

    fn tick(&mut self, ctx: &Ctx, _dt: f32) {
        let now = Instant::now();
        let real = (now - self.last).as_secs_f32().min(3600.0);
        self.last = now;
        if self.screen == Screen::Video {
            ctx.animate();
            // l'image attend que le son soit prêt (1,5 s au plus) pour partir avec lui
            let waiting = self.sound.as_ref().map_or(false, |s| s.state() == audio::PENDING);
            if waiting && self.sound_wait < 1.5 {
                self.sound_wait += real;
                return;
            }
            self.vt += real;
            if self.vt * VIDEO_FPS >= VIDEO_FRAMES as f32 {
                self.screen = Screen::Game;
                self.vt = 0.0;
                if let Some(s) = self.sound.take() {
                    s.detach(); // laisse finir les dernières millisecondes du son
                }
            }
            return;
        }
        let gain = self.rate() * real as f64;
        self.pts += gain;
        self.total += gain;
        self.mana = (self.mana + self.mana_regen() * real).min(self.max_mana());
        self.frenzy_left = (self.frenzy_left - real).max(0.0);
        self.bless_left = (self.bless_left - real).max(0.0);
        self.since_save += real;
        if self.since_save > 5.0 {
            self.since_save = 0.0;
            self.save();
        }
        if self.pop > 0.0 {
            self.pop = (self.pop - real * 6.0).max(0.0);
            ctx.animate();
        }
        let s = ctx.scale();
        for f in &mut self.floats {
            f.t += real;
            f.y -= 70.0 * s * real;
        }
        self.floats.retain(|f| f.t < 0.8);
        if !self.floats.is_empty() {
            ctx.animate();
        }
        if self.scroll.update(real) {
            ctx.animate();
        }
        if self.screen == Screen::Game {
            ctx.redraw_in(Duration::from_millis(100));
        }
    }

    fn on_back(&mut self, _ctx: &Ctx) -> bool {
        match self.screen {
            Screen::Game => {
                self.scroll.stop();
                self.screen = Screen::Menu;
            }
            Screen::Menu => self.screen = Screen::Game,
            Screen::Code => self.screen = Screen::Menu,
            Screen::Tune => {
                self.code.clear();
                self.screen = Screen::Code;
            }
            Screen::Video => {} // impossible de fermer la vidéo
        }
        true
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        match self.screen {
            Screen::Video => {}
            Screen::Game => {
                let l = self.layout(ctx);
                match ev {
                    Event::Down { .. } => self.scroll.stop(),
                    Event::Drag { y, dy, .. } if y >= l.list_top => self.scroll.drag(dy),
                    Event::Wheel { dy } => self.scroll.drag(dy),
                    Event::DragEnd { vy, .. } => self.scroll.fling(vy),
                    Event::Key(Key::Space) | Event::Key(Key::Enter) => self.do_click(ctx),
                    Event::Tap { x, y } => {
                        let (dx, dy) = (x - l.cx, y - l.cy);
                        if l.menu_btn.contains(x, y) {
                            self.screen = Screen::Menu;
                        } else if y >= l.shop_top && y < l.shop_top + l.tab_h {
                            let new = if x < ctx.width() / 2.0 { Tab::Shop } else { Tab::Magic };
                            if new != self.tab {
                                self.tab = new;
                                self.scroll.stop();
                                self.scroll.jump_to(0.0);
                            }
                        } else if y >= l.list_top {
                            let rel = y - l.list_top + self.scroll.pos;
                            if rel >= 0.0 {
                                let row = (rel / l.row_h) as usize;
                                if self.tab == Tab::Shop {
                                    self.buy(ctx, row);
                                } else {
                                    self.magic_tap(ctx, row);
                                }
                            }
                        } else if y < l.shop_top && dx * dx + dy * dy <= l.r * l.r * 1.2 {
                            self.do_click(ctx);
                        }
                    }
                    _ => {}
                }
            }
            Screen::Menu => {
                if let Event::Tap { x, y } = ev {
                    let (_, b) = self.menu_buttons(ctx);
                    if b[0].contains(x, y) {
                        self.screen = Screen::Game;
                    } else if b[1].contains(x, y) {
                        self.code.clear();
                        self.screen = Screen::Code;
                    } else if b[2].contains(x, y) {
                        self.save();
                        ctx.exit();
                    }
                }
            }
            Screen::Code | Screen::Tune => match ev {
                Event::Tap { x, y } => {
                    if y < 44.0 * ctx.scale() && x < ctx.width() * 0.32 {
                        self.on_back(ctx);
                        return;
                    }
                    for (r, k) in self.kb_keys(ctx) {
                        if r.contains(x, y) {
                            match k {
                                Kb::Ch(c) if self.code.len() < MAX_CODE => self.code.push(c),
                                Kb::Del => {
                                    self.code.pop();
                                }
                                _ => {}
                            }
                            return;
                        }
                    }
                    if self.submit_rect(ctx).contains(x, y) {
                        self.submit_code(ctx);
                    }
                }
                Event::Key(Key::Char(c)) if c.is_ascii_alphanumeric() && self.code.len() < MAX_CODE => {
                    self.code.push(c.to_ascii_uppercase());
                }
                Event::Key(Key::Back) => {
                    self.code.pop();
                }
                Event::Key(Key::Enter) => self.submit_code(ctx),
                _ => {}
            },
        }
    }

    fn icon(&mut self, _ctx: &Ctx, g: &mut Gfx, size: u32) {
        let u = size as f32;
        g.clear(rgb(0xB8860B));
        g.circle(u / 2.0, u / 2.0, u * 0.42, rgb(0xF2C94C));
        draw_castle(g, u / 2.0, u / 2.0, u * 0.6, rgb(0x6B4F0A), rgb(0xF2C94C));
    }
}

fn main() {
    let store = Store::open("state.conf");
    let mut app = Conquis::new(store);
    if arg_flag("--demo") {
        app.pts = 123_456.0;
        app.total = 600_000.0;
        app.mana = 62.0;
        app.mg[0] = true;
        app.mg[2] = true;
        for i in 0..4 {
            app.owned[i] = 3 + i as u32;
        }
    }
    if let Some(k) = arg_value("--skin").and_then(|v| v.parse::<u8>().ok()) {
        app.skin = k.min(2);
    }
    match arg_value("--page").as_deref() {
        Some("menu") => app.screen = Screen::Menu,
        Some("magic") => app.tab = Tab::Magic,
        Some("code") => {
            app.screen = Screen::Code;
            app.code = "BU".into();
        }
        Some("tune") => {
            app.screen = Screen::Tune;
            app.code = "INF".into();
        }
        Some("video") => {
            app.screen = Screen::Video;
            app.vt = arg_value("--vt").and_then(|v| v.parse().ok()).unwrap_or(0.0);
        }
        _ => {}
    }
    // edge_exit(false) : pas de glissement de sortie, sinon on pourrait quitter pendant la vidéo.
    run(Config::new("Conquis Cliqueur").edge_exit(false), app);
}
