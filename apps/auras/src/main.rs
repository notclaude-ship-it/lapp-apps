// Auras : application lapp (écran portrait 272x480, rendu GPU avec repli CPU).
//
// Appuyer sur « Invoquer » donne toujours une aura, tirée au hasard selon sa rareté.
// 54 auras, chacune d'un type (Plante, Feu, Cosmos…) ; l'aura équipée donne un bonus de chance
// sur les auras de son type (ex. Sakura : ×2 sur les auras Plante).
// Chaque nouvelle aura déclenche une cinématique ; l'Index permet d'équiper une aura et de revoir
// son animation.
//
// Quitter : glisser vers le haut depuis le bord bas de l'écran, ou Echap.
// Sur PC : cargo run -- --screenshot out.png   (rendu hors écran)
//          cargo run -- --icon icon.png         (icône)
//          LAPP_DATA=/tmp/auras cargo run       (fenêtre)
// Captures de test : --equip <id> | --page index [--detail N] | --cs <id> [--t secondes]

use lapp_sdk::prelude::*;
use std::f32::consts::{PI, TAU};

// ---------------------------------------------------------------- données

struct Aura {
    id: &'static str,
    name: &'static str,
    n: u64,
    col: u32,
    col2: u32,
    typ: &'static str,
    fx: &'static str,
    k: f32,
    hue: bool,
    rainbow: bool,
    bt: &'static str,
    bm: f32,
}

const fn au(id: &'static str, name: &'static str, n: u64, col: u32, col2: u32, typ: &'static str, fx: &'static str) -> Aura {
    Aura { id, name, n, col, col2, typ, fx, k: 1.0, hue: false, rainbow: false, bt: "", bm: 0.0 }
}
impl Aura {
    const fn k(mut self, k: f32) -> Aura {
        self.k = k;
        self
    }
    const fn bonus(mut self, bt: &'static str, bm: f32) -> Aura {
        self.bt = bt;
        self.bm = bm;
        self
    }
    const fn hue(mut self) -> Aura {
        self.hue = true;
        self
    }
    const fn rainbow(mut self) -> Aura {
        self.rainbow = true;
        self
    }
    fn tier(&self) -> usize {
        (((self.n as f64).log2() / 3.0).floor() as usize).min(9)
    }
    fn boost_type(&self) -> &'static str {
        if self.bt.is_empty() { self.typ } else { self.bt }
    }
    fn boost(&self) -> f32 {
        if self.bm > 0.0 { self.bm } else { TIER_BONUS[self.tier()] }
    }
    fn perk(&self) -> String {
        if self.boost() <= 1.0 {
            return "Aucun bonus".into();
        }
        if self.boost_type() == "Tous" {
            format!("Bonus {} sur toutes les auras", fmt_x(self.boost()))
        } else {
            format!("Bonus {} sur les auras {}", fmt_x(self.boost()), self.boost_type())
        }
    }
}

const TIER_BONUS: [f32; 10] = [1.2, 1.5, 1.8, 2.0, 3.0, 4.0, 5.0, 7.0, 9.0, 12.0];

static AURAS: [Aura; 54] = [
    au("commun", "Poussière", 2, 0xC9C9D6, 0x8E8EA3, "Neutre", "dust").bonus("Neutre", 1.0),
    au("feuillage", "Feuillage", 5, 0x9BE37A, 0x3F9A2E, "Plante", "leaves"),
    au("bulle", "Bulle", 10, 0x8FD8FF, 0x2F7FD6, "Eau", "bubbles"),
    au("etincelle", "Étincelle", 15, 0xFFC46B, 0xFF7A1A, "Feu", "sparkle"),
    au("brise", "Brise", 20, 0xC8FFF0, 0x5FD6B6, "Vent", "wind"),
    au("caillou", "Caillou", 30, 0xC2A98A, 0x6E5638, "Terre", "rocks"),
    au("saphir", "Saphir", 45, 0x5AA8FF, 0x1F4FD1, "Eau", "sparkle"),
    au("braise", "Braise", 60, 0xFF8A2B, 0xFF2D1F, "Feu", "fire"),
    au("bourgeon", "Bourgeon", 80, 0xD6FF8A, 0x56B82A, "Plante", "petals"),
    au("pluie", "Pluie", 100, 0x9CC8FF, 0x3A64C8, "Eau", "rain"),
    au("givre", "Givre", 128, 0x9FEAFF, 0x3CB6E6, "Glace", "frost"),
    au("rafale", "Rafale", 160, 0xE6FBFF, 0x7AA6C9, "Vent", "wind").k(1.3),
    au("orage", "Orage", 200, 0xFFE74A, 0x8B5CFF, "Foudre", "storm"),
    au("quartz", "Quartz", 250, 0xFFD0F0, 0xC06BD6, "Terre", "rocks"),
    au("abysse", "Abysse", 300, 0x34E0D0, 0x1257A8, "Eau", "waves"),
    au("venin", "Venin", 400, 0xA6FF3D, 0x3F8F12, "Poison", "bubbles"),
    au("ronce", "Ronce", 500, 0xFF6A7A, 0x2F6B2A, "Plante", "leaves").k(1.3),
    au("sakura", "Sakura", 640, 0xFFB3D1, 0xFF5A9C, "Plante", "petals"),
    au("magma", "Magma", 800, 0xFF5A1F, 0x8A1205, "Feu", "fire").k(1.3),
    au("blizzard", "Blizzard", 1_000, 0xFFFFFF, 0x7FB8FF, "Glace", "frost").k(1.8),
    au("solaire", "Solaire", 1_200, 0xFFD34D, 0xFF8A00, "Lumière", "sun"),
    au("tempete", "Tempête", 1_500, 0xC9B8FF, 0x4A3AB8, "Vent", "wind").k(1.6),
    au("aurore", "Aurore", 2_000, 0x6BFFB8, 0x8A5CFF, "Lumière", "aurora"),
    au("lune", "Lune d'argent", 2_500, 0xDFE8FF, 0x6F86D6, "Cosmos", "moon"),
    au("peste", "Peste", 3_000, 0xC27BFF, 0x4A1A6E, "Poison", "bubbles").k(1.5),
    au("volt", "Volt", 4_000, 0x7FE8FF, 0x1A5CFF, "Foudre", "storm").k(1.5),
    au("neant", "Néant", 5_000, 0xB06BFF, 0x3B0F6E, "Ombre", "void"),
    au("amethyste", "Améthyste", 6_500, 0xD59BFF, 0x6A2FB5, "Terre", "rocks").k(1.4),
    au("ombre", "Ombre rouge", 8_000, 0xFF4A5A, 0x5A0A1A, "Ombre", "void"),
    au("celeste", "Céleste", 12_000, 0xFFF1B8, 0xFFC23D, "Lumière", "angel"),
    au("comete", "Comète", 20_000, 0x9FE0FF, 0x4A7AFF, "Cosmos", "comet"),
    au("seisme", "Séisme", 25_000, 0xFFB36B, 0x7A3A12, "Terre", "rocks").k(1.6),
    au("atome", "Atome", 30_000, 0x7AFFEA, 0x2A8CFF, "Cosmos", "atom"),
    au("galaxie", "Galaxie", 50_000, 0xC48BFF, 0xFF6BD5, "Cosmos", "galaxy"),
    au("phenix", "Phénix", 75_000, 0xFFB02E, 0xFF3A1A, "Feu", "angel"),
    au("glacier", "Glacier éternel", 90_000, 0xC9F6FF, 0x2A9CD6, "Glace", "frost").k(2.0),
    au("chronos", "Chronos", 120_000, 0xE8C27A, 0x9A6A2A, "Temps", "clock"),
    au("sablier", "Sablier", 150_000, 0xFFE0A0, 0xC88A3A, "Temps", "clock").k(1.5),
    au("typhon", "Typhon", 200_000, 0x7AFFD8, 0x0A5A6E, "Vent", "wind").k(2.0),
    au("glitch", "Glitch", 300_000, 0x39FFF2, 0xFF2BD6, "Spécial", "glitch").bonus("Tous", 1.5),
    au("jugement", "Jugement", 400_000, 0xFFF27A, 0xFFB800, "Foudre", "storm").k(2.2),
    au("foret", "Forêt ancestrale", 500_000, 0xFFE27A, 0x2A8A3A, "Plante", "leaves").k(2.0),
    au("supernova", "Supernova", 750_000, 0xFFD0F5, 0xFF3AA8, "Cosmos", "sun").k(1.4),
    au("divin", "Divin", 1_000_000, 0xFFFFFF, 0xFFD86B, "Lumière", "divine"),
    au("abime", "Abîme", 1_500_000, 0x5A7AFF, 0x0A0A3A, "Ombre", "void").k(1.5),
    au("arcenciel", "Arc-en-ciel", 2_000_000, 0xFFFFFF, 0xFF9AD8, "Lumière", "aurora").rainbow(),
    au("paradoxe", "Paradoxe", 3_000_000, 0x7AFFFF, 0x2A3AFF, "Temps", "clock").k(1.8),
    au("nebuleuse", "Nébuleuse", 5_000_000, 0x8AD8FF, 0x3A5AFF, "Cosmos", "galaxy").hue().k(1.4),
    au("kraken", "Kraken", 7_500_000, 0x2AFFC8, 0x0A2A5A, "Eau", "waves").k(1.6),
    au("dragon", "Dragon", 10_000_000, 0xFFD23A, 0x6A0AFF, "Feu", "fire").k(2.0),
    au("apocalypse", "Apocalypse", 25_000_000, 0xFF2A2A, 0x2A0000, "Ombre", "void").k(2.0),
    au("eternite", "Éternité", 50_000_000, 0xFFFFFF, 0xA0C8FF, "Temps", "clock").k(2.0),
    au("createur", "Créateur", 100_000_000, 0xFFF6D6, 0xFFB8FF, "Spécial", "divine").bonus("Tous", 3.0).k(1.5),
    au("infini", "Infini", 1_000_000_000, 0xFFFFFF, 0xB48AFF, "Spécial", "infinity").bonus("Tous", 5.0),
];

const BG: u32 = 0x120D1D;
const STAGE: u32 = 0x0D0918;
const PANEL: u32 = 0x1D1530;
const LINE: u32 = 0x3A2D57;
const INK: u32 = 0xEFE9FF;
const MUTED: u32 = 0xA79CC4;
const GOLD: u32 = 0xF3C969;
const GOLD_INK: u32 = 0x2A1A05;

const ROLL_DELAYS: [f32; 11] = [0.045, 0.05, 0.055, 0.06, 0.07, 0.08, 0.095, 0.115, 0.14, 0.175, 0.22];
const INTRO: [f32; 10] = [0.35, 0.6, 1.0, 1.4, 1.9, 2.4, 2.9, 3.4, 3.9, 4.5];

fn fmt(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(' ');
        }
        out.push(ch);
    }
    out
}
fn fmt_x(m: f32) -> String {
    if m.fract() == 0.0 { format!("×{}", m as u32) } else { format!("×{:.1}", m).replace('.', ",") }
}

/// Minuscules, sans accents ni apostrophes.
fn norm(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars().flat_map(|c| c.to_lowercase()) {
        let r = match ch {
            'à' | 'â' | 'ä' | 'á' => "a",
            'é' | 'è' | 'ê' | 'ë' => "e",
            'î' | 'ï' | 'í' => "i",
            'ô' | 'ö' | 'ó' => "o",
            'ù' | 'û' | 'ü' | 'ú' => "u",
            'ç' => "c",
            'œ' => "oe",
            '\'' | '’' | '-' | '`' => " ",
            _ => {
                out.push(ch);
                continue;
            }
        };
        out.push_str(r);
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn dz(v: &[u8]) -> String {
    let mut s: u32 = 0x5A17_C3E9;
    let b: Vec<u8> = v
        .iter()
        .map(|&x| {
            s = s.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            x ^ (s >> 24) as u8
        })
        .collect();
    String::from_utf8(b).unwrap_or_default()
}
const Z_TITLE: [u8; 11] = [4, 251, 152, 164, 108, 189, 54, 199, 247, 198, 9];
const Z_FIELD: [u8; 4] = [10, 241, 146, 180];
const Z_OK: [u8; 7] = [31, 255, 154, 184, 40, 187, 37];
const Z_CLOSE: [u8; 10] = [171, 30, 79, 241, 10, 187, 37, 201, 250, 119];
const Z_BAD: [u8; 13] = [10, 241, 146, 180, 108, 183, 57, 199, 240, 107, 206, 89, 144];
const Z_MISS: [u8; 17] = [8, 235, 132, 176, 108, 183, 57, 208, 237, 106, 213, 90, 223, 191, 35, 246, 132];
const Z_GOT: [u8; 12] = [8, 235, 132, 176, 108, 177, 53, 208, 250, 107, 213, 73];
const Z_OLD: [u8; 10] = [40, 235, 132, 176, 63, 240, 52, 203, 241, 99];
const Z_BK: [u8; 8] = [47, 230, 216, 178, 45, 189, 63, 193];
const Z_NONE: [u8; 18] = [27, 247, 147, 191, 108, 29, 247, 132, 237, 96, 211, 88, 223, 168, 61, 246, 216, 0];
const Z_YES: [u8; 24] = [25, 236, 153, 182, 62, 187, 36, 215, 246, 106, 206, 12, 204, 184, 60, 231, 203, 91, 109, 129, 12, 204, 172, 29];
const ZR: u64 = 0x9694f117e2688b39;

fn zx(v: &[u8]) -> Vec<u8> {
    let mut s: u32 = 0x6C1F_2A7B;
    v.iter()
        .map(|&x| {
            s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            x ^ (s >> 24) as u8
        })
        .collect()
}
const ZK: [u64; 3] = [0x2ea2ada917d8411e, 0xaec617f5f4ebf590, 0x41cdc6a5f6ced737];

fn kh(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let pre: Vec<u8> = "Δ".bytes().chain(s.bytes()).collect();
    for r in 0..200_000u64 {
        for b in pre.iter().copied().chain(r.to_le_bytes()) {
            h ^= b as u64;
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        h ^= h >> 29;
        h = h.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    }
    h
}

// ---------------------------------------------------------------- hasard

struct Rng(u64);
impl Rng {
    fn new() -> Rng {
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
        Rng(t | 1)
    }
    fn f(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}
// ---------------------------------------------------------------- outils de dessin

fn hsh(i: f32, s: f32) -> f32 {
    let x = ((i * 12.9898 + s * 78.233) % 6283.0).sin() * 43758.547;
    x - x.floor()
}
fn cyc(i: usize, t: f32, speed: f32) -> (f32, f32) {
    let p = t * speed + hsh(i as f32, 9.0);
    let k = p.floor();
    (i as f32 * 1.01 + k * 7.3, p - k)
}
fn c(hex: u32, a: f32) -> Color {
    rgba(hex, a)
}
fn glow(g: &mut Gfx, x: f32, y: f32, r: f32, hex: u32, a: f32) {
    if r <= 0.5 {
        return;
    }
    let steps = 7;
    for i in 0..steps {
        let f = 1.0 - i as f32 / steps as f32;
        g.circle(x, y, r * f, c(hex, a * 0.2));
    }
}
fn line(g: &mut Gfx, x1: f32, y1: f32, x2: f32, y2: f32, w: f32, col: Color) {
    let d = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    let n = (d / (w * 0.7).max(0.5)).ceil().max(1.0) as usize;
    for j in 0..=n {
        let u = j as f32 / n as f32;
        g.circle(x1 + (x2 - x1) * u, y1 + (y2 - y1) * u, w / 2.0, col);
    }
}
fn tline(g: &mut Gfx, x1: f32, y1: f32, x2: f32, y2: f32, w1: f32, w2: f32, col: Color) {
    let d = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    let n = (d / (w2.min(w1) * 0.7).max(0.5)).ceil().clamp(1.0, 80.0) as usize;
    for j in 0..=n {
        let u = j as f32 / n as f32;
        g.circle(x1 + (x2 - x1) * u, y1 + (y2 - y1) * u, (w1 + (w2 - w1) * u) / 2.0, col);
    }
}
fn ring(g: &mut Gfx, x: f32, y: f32, r: f32, sy: f32, w: f32, col: Color) {
    let n = ((TAU * r) / (w * 0.8).max(0.6)).ceil().clamp(12.0, 260.0) as usize;
    for j in 0..n {
        let a = j as f32 / n as f32 * TAU;
        g.circle(x + a.cos() * r, y + a.sin() * r * sy, w / 2.0, col);
    }
}
fn star4(g: &mut Gfx, x: f32, y: f32, r: f32, col: Color) {
    if r < 0.4 {
        return;
    }
    let th = (r * 0.28).max(0.8);
    g.rrect(x - r, y - th / 2.0, r * 2.0, th, th / 2.0, col);
    g.rrect(x - th / 2.0, y - r, th, r * 2.0, th / 2.0, col);
    g.circle(x, y, r * 0.3, col);
}
fn star5(g: &mut Gfx, x: f32, y: f32, r: f32, col: Color) {
    if r < 0.5 {
        return;
    }
    for i in 0..5 {
        let a = -PI / 2.0 + i as f32 * TAU / 5.0;
        tline(g, x, y, x + a.cos() * r, y + a.sin() * r, r * 0.55, r * 0.08, col);
    }
    g.circle(x, y, r * 0.38, col);
}

// ---------------------------------------------------------------- effets d'aura

fn fx(g: &mut Gfx, idx: usize, t: f32, x: f32, y: f32, r: f32, k: f32) {
    let a = &AURAS[idx];
    let k = k * a.k;
    let (c1, c2) = (a.col, a.col2);
    let nk = |n: f32| (n * k) as usize;
    match a.fx {
        "dust" => {
            glow(g, x, y, r * 1.7 * (1.0 + 0.06 * (t * 2.0).sin()), c1, 0.3);
            for i in 0..nk(14.0) {
                let fi = i as f32;
                let dir = if hsh(fi, 2.0) > 0.5 { 1.0 } else { -1.0 };
                let an = hsh(fi, 1.0) * TAU + t * 0.35 * dir;
                let rr = r * (0.9 + hsh(fi, 3.0) * 0.8);
                g.circle(x + an.cos() * rr, y + an.sin() * rr * 0.7, r * 0.035 + 0.5, c(c1, 0.4 + 0.4 * (t * 2.0 + fi).sin()));
            }
        }
        "leaves" => {
            glow(g, x, y, r * 1.7, c2, 0.3);
            for i in 0..nk(18.0) {
                let fi = i as f32;
                let an = hsh(fi, 1.0) * TAU + t * (0.7 + hsh(fi, 2.0) * 0.8);
                let rr = r * (0.85 + 0.7 * hsh(fi, 3.0));
                let (px, py) = (x + an.cos() * rr, y + an.sin() * rr * 0.55 - (t + fi).sin() * r * 0.15);
                let rot = an + t * 2.0;
                let col = c(if i % 3 != 0 { c1 } else { c2 }, 0.95);
                tline(g, px - rot.cos() * r * 0.1, py - rot.sin() * r * 0.1, px + rot.cos() * r * 0.1, py + rot.sin() * r * 0.1, r * 0.03, r * 0.09, col);
            }
        }
        "sparkle" => {
            glow(g, x, y, r * 1.8, c2, 0.45);
            glow(g, x, y, r * 1.1, c1, 0.25);
            for i in 0..nk(24.0) {
                let (sd, f) = cyc(i, t, 0.7);
                let an = hsh(sd, 1.0) * TAU;
                let rr = r * (0.4 + hsh(sd, 2.0) * 1.3);
                star4(g, x + an.cos() * rr, y + an.sin() * rr, r * 0.13 * (f * PI).sin(), c(if i % 2 == 1 { 0xFFFFFF } else { c1 }, 1.0));
            }
        }
        "fire" => {
            glow(g, x, y + r * 0.2, r * 1.8, c2, 0.38);
            for i in 0..nk(40.0) {
                let (sd, f) = cyc(i, t, 1.1);
                let px = x + (hsh(sd, 1.0) - 0.5) * r * 1.6 + (f * 6.0 + sd).sin() * r * 0.12;
                let py = y + r * 0.9 - f * r * 2.5;
                let col = if f < 0.3 { 0xFFF2A8 } else if f < 0.6 { c1 } else { c2 };
                glow(g, px, py, r * 0.28 * (1.0 - f) * (0.5 + hsh(sd, 2.0)), col, 0.9 - f * 0.6);
            }
        }
        "frost" => {
            glow(g, x, y, r * 1.8, c2, 0.35);
            for i in 0..10 {
                let an = i as f32 / 10.0 * TAU + t * 0.4;
                let (px, py) = (x + an.cos() * r * 1.35, y + an.sin() * r * 1.35 * 0.75);
                let (dx, dy) = ((an + PI / 2.0).cos(), (an + PI / 2.0).sin());
                tline(g, px - dx * r * 0.2, py - dy * r * 0.2, px + dx * r * 0.12, py + dy * r * 0.12, r * 0.02, r * 0.11, c(c1, 0.85));
            }
            for i in 0..nk(20.0) {
                let (sd, f) = cyc(i, t, 0.35);
                let px = x + (hsh(sd, 1.0) - 0.5) * r * 3.4 + (f * 5.0 + sd).sin() * r * 0.15;
                let py = y - r * 1.7 + f * r * 3.4;
                let s = r * (0.05 + hsh(sd, 2.0) * 0.07);
                let col = c(0xFFFFFF, 0.85 * (f * PI).sin());
                for j in 0..3 {
                    let an = f * 3.0 + j as f32 * PI / 3.0;
                    line(g, px - an.cos() * s, py - an.sin() * s, px + an.cos() * s, py + an.sin() * s, (r * 0.02).max(1.0), col);
                }
            }
        }
        "storm" => {
            let fl = if (t * 23.0).sin() > 0.6 { 0.55 } else { 0.3 };
            glow(g, x, y, r * 1.9, c2, fl);
            for b in 0..nk(4.0) {
                let (sd, f) = cyc(b, t, 2.6);
                if f > 0.4 {
                    continue;
                }
                let an = hsh(sd, 1.0) * TAU;
                let (mut px, mut py) = (x + an.cos() * r * 0.5, y + an.sin() * r * 0.5);
                for j in 1..=7 {
                    let rr = r * (0.5 + j as f32 * 0.22);
                    let off = (hsh(sd, j as f32 + 2.0) - 0.5) * 0.5;
                    let (nx, ny) = (x + (an + off).cos() * rr, y + (an + off).sin() * rr);
                    line(g, px, py, nx, ny, r * 0.05, c(c1, 1.0 - f * 2.0));
                    line(g, px, py, nx, ny, (r * 0.018).max(1.0), c(0xFFFFFF, 0.9));
                    px = nx;
                    py = ny;
                }
            }
        }
        "waves" => {
            glow(g, x, y, r * 1.9, c2, 0.45);
            for i in 0..5 {
                let f = (t * 0.3 + i as f32 / 5.0) % 1.0;
                let r0 = r * (0.6 + f * 1.7);
                let n = 70;
                for s in 0..n {
                    let an = s as f32 / n as f32 * TAU;
                    let rr = r0 + (an * 8.0 + t * 3.0 + i as f32).sin() * r * 0.06;
                    g.circle(x + an.cos() * rr, y + an.sin() * rr * 0.8, r * 0.022 + 0.4, c(c1, (1.0 - f) * 0.9));
                }
            }
            for i in 0..nk(12.0) {
                let (sd, f) = cyc(i, t, 0.5);
                g.circle(x + (hsh(sd, 1.0) - 0.5) * r * 2.4, y + r - f * r * 2.2, r * 0.03 + 0.5, c(0xFFFFFF, 0.6 * (1.0 - f)));
            }
        }
        "bubbles" => {
            glow(g, x, y, r * 1.8, c2, 0.5);
            for i in 0..nk(24.0) {
                let (sd, f) = cyc(i, t, 0.55);
                let px = x + (hsh(sd, 1.0) - 0.5) * r * 2.0 + (f * 8.0 + sd).sin() * r * 0.1;
                let py = y + r * 1.1 - f * r * 2.6;
                let mut rr = r * 0.1 * (0.5 + hsh(sd, 2.0));
                let mut al = 1.0;
                if f > 0.88 {
                    rr *= 1.0 + (f - 0.88) * 12.0;
                    al = (1.0 - f) * 8.0;
                }
                g.circle(px, py, rr, c(c1, 0.75 * al));
                g.circle(px, py, rr * 0.75, c(c2, 0.55 * al));
                g.circle(px - rr * 0.35, py - rr * 0.35, rr * 0.25, c(0xFFFFFF, 0.8 * al));
            }
        }
        "petals" => {
            glow(g, x, y, r * 1.8, c2, 0.35);
            for i in 0..nk(28.0) {
                let (sd, f) = cyc(i, t, 0.22);
                let px = x - r * 2.2 + hsh(sd, 1.0) * r * 4.4 + f * r + (f * 9.0 + sd).sin() * r * 0.25;
                let py = y - r * 2.0 + f * r * 4.0;
                let rot = f * 7.0 + sd;
                let s = r * (0.07 + hsh(sd, 2.0) * 0.06);
                let al = ((f * PI).sin() * 1.6).min(1.0);
                tline(g, px - rot.cos() * s, py - rot.sin() * s, px + rot.cos() * s, py + rot.sin() * s, s * 0.3, s * 0.9, c(if i % 3 != 0 { c1 } else { c2 }, al));
            }
        }
        "sun" => {
            for i in 0..18 {
                let an = i as f32 / 18.0 * TAU + t * 0.25;
                let l = r * (1.6 + 0.4 * (t * 2.0 + i as f32 * 1.7).sin()) * if k > 1.0 { 1.3 } else { 1.0 };
                tline(g, x + an.cos() * r * 0.5, y + an.sin() * r * 0.5, x + an.cos() * l, y + an.sin() * l, r * 0.16, r * 0.02, c(c1, 0.45));
            }
            glow(g, x, y, r * 1.6, c2, 0.55);
            glow(g, x, y, r * 0.9, 0xFFF6C9, 0.6);
        }
        "moon" => {
            glow(g, x, y, r * 1.9, c2, 0.4);
            for i in 0..nk(28.0) {
                let fi = i as f32;
                let an = hsh(fi, 1.0) * TAU;
                let rr = r * (0.6 + hsh(fi, 2.0) * 1.6);
                star4(g, x + an.cos() * rr, y + an.sin() * rr, r * 0.07 * (0.4 + 0.6 * (t * 1.5 + fi).sin().abs()), c(0xFFFFFF, 1.0));
            }
            for m in 0..2 {
                let an = t * 0.6 + m as f32 * PI;
                let (mx, my) = (x + an.cos() * r * 1.45, y + an.sin() * r * 0.55);
                let mr = r * (0.24 + 0.06 * an.sin());
                glow(g, mx, my, mr * 2.2, c1, 0.5);
                g.circle(mx, my, mr, c(c1, 1.0));
                g.circle(mx + mr * 0.45, my - mr * 0.2, mr * 0.85, c(STAGE, 0.92));
            }
        }
        "void" => {
            glow(g, x, y, r * 2.2, c1, 0.4);
            for i in 0..nk(60.0) {
                let (sd, f) = cyc(i, t, 0.45);
                let rr = r * 2.4 * (1.0 - f) + r * 0.5;
                let an = hsh(sd, 1.0) * TAU + f * 5.0;
                g.circle(x + an.cos() * rr, y + an.sin() * rr * 0.6, r * 0.03 * (0.5 + f) + 0.4, c(if i % 4 != 0 { c1 } else { 0xFFFFFF }, f));
            }
            for i in 0..90 {
                let an = i as f32 / 90.0 * TAU + t * 1.6;
                let rr = r * (1.05 + 0.05 * (i as f32 * 3.1).sin());
                g.circle(x + an.cos() * rr * 1.25, y + an.sin() * rr * 0.35, r * 0.05, c(if i % 3 != 0 { c1 } else { 0xFF9CF2 }, 0.55 + 0.45 * an.sin()));
            }
            g.circle(x, y, r * 0.58, c(c1, 0.9));
            g.circle(x, y, r * 0.55, c(0x000000, 1.0));
        }
        "angel" => {
            glow(g, x, y, r * 2.0, c2, 0.35);
            let flap = (t * 2.2).sin() * 0.14;
            for side in [-1.0f32, 1.0] {
                for j in 0..8 {
                    let ang = -1.05 + j as f32 * 0.17 + flap;
                    let l = r * (1.5 - j as f32 * 0.09);
                    let (ox, oy) = (x + side * r * 0.25, y - r * 0.15);
                    let (ex, ey) = (ox + side * ang.cos() * l, oy + ang.sin() * l);
                    tline(g, ox, oy, ex, ey, r * 0.15, r * 0.04, c(if j % 2 == 0 { c2 } else { 0xFFFFFF }, 0.55));
                }
            }
            ring(g, x, y - r * 1.05, r * 0.38, 0.3, r * 0.06, c(c1, 0.95));
            for i in 0..nk(18.0) {
                let (sd, f) = cyc(i, t, 0.4);
                star4(g, x + (hsh(sd, 1.0) - 0.5) * r * 3.0, y + r - f * r * 2.6, r * 0.07 * (f * PI).sin(), c(0xFFFFFF, 1.0));
            }
        }
        "galaxy" => {
            glow(g, x, y, r * 2.3, 0x5B2BB5, 0.45);
            let n = (70.0 * k) as usize;
            let pal = if a.hue { [0x6BE0FF, 0x5A8CFF, 0x7BFFE0, 0xB8F0FF, 0x3A5AFF] } else { [0xFF6BD5, 0xC48BFF, 0x6BB8FF, 0x7BFFE0, 0xFFE38B] };
            for arm in 0..3 {
                for j in 0..n {
                    let u = j as f32 / n as f32;
                    let an = u * TAU * 1.3 + arm as f32 * TAU / 3.0 + t * 0.45;
                    let rr = r * (0.15 + u * 2.1);
                    let jx = (hsh(j as f32, arm as f32) - 0.5) * r * 0.25 * u;
                    let jy = (hsh(j as f32, arm as f32 + 5.0) - 0.5) * r * 0.25 * u;
                    let col = pal[((u * 5.0 + t * 0.4 + arm as f32) as usize) % pal.len()];
                    g.circle(x + an.cos() * rr + jx, y + an.sin() * rr * 0.62 + jy, r * 0.035 * (1.3 - u) + 0.4, c(col, 1.0 - u * 0.8));
                }
            }
            glow(g, x, y, r * 0.8, 0xFFFFFF, 0.7);
        }
        "clock" => {
            glow(g, x, y, r * 1.9, c2, 0.4);
            let rings = [(1.15f32, 0.5f32, 12usize), (1.55, -0.3, 24), (1.95, 0.15, 60)];
            for (idx2, (rr, sp, ticks)) in rings.iter().enumerate() {
                let col = c(c1, 0.75 - idx2 as f32 * 0.15);
                ring(g, x, y, r * rr, 1.0, r * 0.025 + 0.4, col);
                for i in 0..*ticks {
                    let an = i as f32 / *ticks as f32 * TAU + t * sp;
                    let big = i % (ticks / 12) == 0;
                    let l = if big { r * 0.16 } else { r * 0.07 };
                    let w = if big { r * 0.03 } else { r * 0.012 };
                    line(g, x + an.cos() * r * rr, y + an.sin() * r * rr, x + an.cos() * (r * rr - l), y + an.sin() * (r * rr - l), w.max(0.8), col);
                }
            }
            for (an, l, w) in [(t * 2.5, 1.0f32, 0.04f32), (t * 0.4, 0.7, 0.06)] {
                line(g, x, y, x + an.cos() * r * l, y + an.sin() * r * l, r * w, c(0xFFF4D6, 1.0));
            }
            for i in 0..nk(20.0) {
                let (sd, f) = cyc(i, t, 0.3);
                let an = hsh(sd, 1.0) * TAU + f * 2.0;
                g.circle(x + an.cos() * r * 2.1 * f, y + an.sin() * r * 2.1 * f, r * 0.025 + 0.4, c(c1, 1.0 - f));
            }
        }
        "glitch" => {
            let step = (t * 12.0).floor();
            glow(g, x, y, r * 1.8, 0x3A1AFF, 0.4);
            for (col, o) in [(0xFF2B5Eu32, -1.0f32), (0x2BFF88, 0.0), (0x2BC8FF, 1.0)] {
                let dx = (hsh(step, o + 3.0) - 0.5) * r * 0.25 + o * r * 0.05;
                ring(g, x + dx, y, r * 1.2, 1.0, r * 0.05, c(col, 0.75));
                ring(g, x - dx, y, r * 0.75, 1.0, r * 0.05, c(col, 0.75));
            }
            for i in 0..nk(16.0) {
                let fi = i as f32;
                if hsh(step * 31.0 + fi, 2.0) > 0.6 {
                    continue;
                }
                let bx = x + (hsh(step + fi, 3.0) - 0.5) * r * 4.0;
                let by = y + (hsh(step + fi, 4.0) - 0.5) * r * 3.2;
                g.rect(bx, by, r * (0.1 + hsh(fi, step) * 0.7), r * (0.03 + hsh(step, fi) * 0.12), c(if i % 2 == 1 { c1 } else { c2 }, 0.75));
            }
            let mut sy = y - r * 2.0 + (t * 40.0) % (r * 0.08);
            while sy < y + r * 2.0 {
                g.rect(x - r * 2.2, sy, r * 4.4, (r * 0.02).max(1.0), c(0xFFFFFF, 0.07));
                sy += r * 0.08;
            }
        }
        "wind" => {
            glow(g, x, y, r * 1.8, c2, 0.3);
            for i in 0..nk(16.0) {
                let fi = i as f32;
                let rr = r * (0.7 + hsh(fi, 1.0) * 1.4);
                let a0 = hsh(fi, 3.0) * TAU + t * (1.5 + hsh(fi, 2.0) * 1.6);
                let len = 0.6 + hsh(fi, 4.0) * 0.9;
                let w = r * (0.02 + 0.035 * hsh(fi, 5.0)) + 0.6;
                let col = c(if i % 3 != 0 { c1 } else { c2 }, 0.75);
                let n = ((len * rr) / (w * 0.8)).ceil().clamp(4.0, 90.0) as usize;
                for j in 0..=n {
                    let an = a0 + len * j as f32 / n as f32;
                    g.circle(x + an.cos() * rr, y + an.sin() * rr * 0.55, w / 2.0, col);
                }
            }
            for i in 0..nk(12.0) {
                let (sd, f) = cyc(i, t, 0.8);
                let an = hsh(sd, 1.0) * TAU + f * 4.0;
                let rr = r * (0.6 + f * 1.5);
                g.circle(x + an.cos() * rr, y + an.sin() * rr * 0.55, r * 0.025 + 0.4, c(0xFFFFFF, 1.0 - f));
            }
        }
        "rocks" => {
            glow(g, x, y, r * 1.8, c2, 0.35);
            let n = (9.0 * k.min(1.6)) as usize;
            for i in 0..n {
                let fi = i as f32;
                let an = fi / n as f32 * TAU + t * 0.5;
                let rr = r * (1.25 + 0.15 * (t + fi).sin());
                let px = x + an.cos() * rr;
                let py = y + an.sin() * rr * 0.6 + (t * 2.0 + fi).sin() * r * 0.08;
                let sz = r * (0.12 + hsh(fi, 2.0) * 0.1);
                let rot = t * (0.6 + hsh(fi, 3.0)) + fi;
                g.circle(px, py, sz * 1.05, c(c1, 0.9));
                for j in 0..3 {
                    let aj = rot + j as f32 * TAU / 3.0;
                    g.circle(px + aj.cos() * sz * 0.35, py + aj.sin() * sz * 0.35, sz * 0.7, c(c2, 1.0));
                }
                g.circle(px - sz * 0.3, py - sz * 0.3, sz * 0.25, c(c1, 0.8));
            }
            for i in 0..nk(14.0) {
                let (sd, f) = cyc(i, t, 0.6);
                g.circle(x + (hsh(sd, 1.0) - 0.5) * r * 2.6, y - r * 0.2 + f * r * 1.6, r * 0.025 + 0.4, c(c1, 1.0 - f));
            }
        }
        "rain" => {
            glow(g, x, y, r * 1.8, c2, 0.32);
            for i in 0..nk(36.0) {
                let (sd, f) = cyc(i, t, 1.4);
                let px = x + (hsh(sd, 1.0) - 0.5) * r * 3.6 - f * r * 0.3;
                let py = y - r * 1.9 + f * r * 3.4;
                line(g, px, py, px - r * 0.05, py + r * 0.24, (r * 0.022).max(1.0), c(c1, 0.75));
            }
            for i in 0..4 {
                let p = t * 0.6 + i as f32 / 4.0;
                let f = p % 1.0;
                let px = x + (hsh(i as f32 + p.floor(), 2.0) - 0.5) * r * 2.4;
                ring(g, px, y + r * 1.55, (r * 0.5 * f).max(1.0), 0.25, 1.2, c(c1, 1.0 - f));
            }
        }
        "aurora" => {
            glow(g, x, y, r * 1.8, c2, 0.25);
            let rb = [0xFF7A7A, 0xFFD27A, 0x8AFF8A, 0x7AD8FF, 0xB88AFF, 0xFF8AE0];
            for b in 0..4 {
                for s in 0..=36 {
                    let u = s as f32 / 36.0;
                    let px = x - r * 2.2 + u * r * 4.4;
                    let py = y - r * 1.35 + (u * 5.0 + t * 1.2 + b as f32).sin() * r * 0.3 + b as f32 * r * 0.14;
                    let l = r * (0.9 + 0.35 * (u * 9.0 + t * 2.0 + b as f32).sin());
                    let col = if a.rainbow { rb[((u * 6.0 + t * 0.8 + b as f32) as usize) % 6] } else if b % 2 == 1 { c1 } else { c2 };
                    for q in 0..4 {
                        let f = q as f32 / 4.0;
                        g.rect(px - r * 0.06, py + l * f, r * 0.12, l / 4.0, c(col, 0.45 * (1.0 - f)));
                    }
                }
            }
            for i in 0..nk(18.0) {
                let fi = i as f32;
                star4(g, x + (hsh(fi, 1.0) - 0.5) * r * 4.0, y + (hsh(fi, 2.0) - 0.5) * r * 3.4, r * 0.05 * (t * 1.4 + fi).sin().abs(), c(0xFFFFFF, 1.0));
            }
        }
        "comet" => {
            glow(g, x, y, r * 1.8, c2, 0.3);
            for m in 0..3 {
                let fm = m as f32;
                let an = t * (0.9 + fm * 0.35) + fm * 2.1;
                let rr = r * (1.15 + fm * 0.35);
                let sy = 0.45 + fm * 0.1;
                for j in (0..=28).rev() {
                    let aj = an - j as f32 * 0.055;
                    let q = 1.0 - j as f32 / 28.0;
                    g.circle(x + aj.cos() * rr, y + aj.sin() * rr * sy, r * 0.08 * q + 0.5, c(if j < 3 { 0xFFFFFF } else { c1 }, q * 0.9));
                }
                glow(g, x + an.cos() * rr, y + an.sin() * rr * sy, r * 0.35, c1, 0.8);
            }
            for i in 0..nk(16.0) {
                let fi = i as f32;
                star4(g, x + (hsh(fi, 1.0) - 0.5) * r * 4.0, y + (hsh(fi, 2.0) - 0.5) * r * 3.0, r * 0.05 * (t + fi).sin().abs(), c(0xFFFFFF, 1.0));
            }
        }
        "atom" => {
            glow(g, x, y, r * 1.1, c1, 0.6);
            for o in 0..3 {
                let rot = o as f32 * PI / 3.0 + t * 0.2;
                let (cr, sr) = (rot.cos(), rot.sin());
                for s in 0..90 {
                    let e = s as f32 / 90.0 * TAU;
                    let (ex, ey) = (e.cos() * r * 1.6, e.sin() * r * 0.5);
                    g.circle(x + ex * cr - ey * sr, y + ex * sr + ey * cr, r * 0.015 + 0.5, c(c1, 0.5));
                }
                for el in 0..2 {
                    let e = t * (2.4 + o as f32 * 0.4) + el as f32 * PI;
                    let (ex, ey) = (e.cos() * r * 1.6, e.sin() * r * 0.5);
                    let (px, py) = (x + ex * cr - ey * sr, y + ex * sr + ey * cr);
                    glow(g, px, py, r * 0.3, c2, 0.9);
                    g.circle(px, py, r * 0.07, rgb(0xFFFFFF));
                }
            }
            for i in 0..7 {
                let fi = i as f32;
                g.circle(x + (fi * 2.4 + t).cos() * r * 0.12, y + (fi * 2.4 + t).sin() * r * 0.12, r * 0.1, rgb(if i % 2 == 1 { c1 } else { c2 }));
            }
        }
        "infinity" => {
            glow(g, x, y, r * 2.5, c2, 0.45);
            let rb = [0xFF7A7A, 0xFFD27A, 0x8AFF8A, 0x7AD8FF, 0xB88AFF, 0xFF8AE0];
            let p = |s: f32| {
                let d = 1.0 + s.sin().powi(2);
                (x + r * 2.0 * s.cos() / d, y + r * 2.0 * s.sin() * s.cos() / d)
            };
            for s in 0..160 {
                let (px, py) = p(s as f32 / 160.0 * TAU);
                g.circle(px, py, r * 0.03 + 0.5, c(rb[((s as f32 / 8.0 + t * 3.0) as usize) % 6], 0.85));
            }
            for i in 0..nk(40.0) {
                let (px, py) = p(i as f32 / (40.0 * k) * TAU + t * 1.3);
                glow(g, px, py, r * 0.14, 0xFFFFFF, 0.7);
            }
            for i in 0..4 {
                let f = (t * 0.4 + i as f32 / 4.0) % 1.0;
                ring(g, x, y, r * (0.4 + f * 2.4), 1.0, r * 0.03 + 0.5, c(c2, 1.0 - f));
            }
            for i in 0..nk(26.0) {
                let (sd, f) = cyc(i, t, 0.5);
                let an = hsh(sd, 1.0) * TAU;
                let rr = r * (0.4 + f * 2.2);
                star4(g, x + an.cos() * rr, y + an.sin() * rr, r * 0.1 * (f * PI).sin(), c(rb[(hsh(sd, 3.0) * 6.0) as usize % 6], 1.0));
            }
        }
        _ => {
            // divin
            let pal = [0xFF9A9A, 0xFFE08A, 0x9AFFB0, 0x8AD8FF, 0xC59AFF, 0xFF9AE6];
            for i in 0..24 {
                let dir = if i % 2 == 1 { 1.0 } else { -1.0 };
                let an = i as f32 / 24.0 * TAU + t * 0.15 * dir;
                let l = r * (2.2 + 0.5 * (t * 1.7 + i as f32).sin());
                tline(g, x, y, x + an.cos() * l, y + an.sin() * l, r * 0.12, r * 0.02, c(pal[(i + (t * 2.0) as usize) % 6], 0.3));
            }
            for i in 0..4 {
                let f = (t * 0.35 + i as f32 / 4.0) % 1.0;
                ring(g, x, y, r * (0.6 + f * 2.0), 1.0, (r * 0.04 * (1.0 - f)).max(0.6), c(c2, 1.0 - f));
            }
            glow(g, x, y, r * 2.4, c2, 0.4);
            glow(g, x, y, r * 1.2, 0xFFFFFF, 0.9);
            for i in 0..nk(26.0) {
                let (sd, f) = cyc(i, t, 0.5);
                let an = hsh(sd, 1.0) * TAU;
                let rr = r * (0.5 + f * 2.0);
                star4(g, x + an.cos() * rr, y + an.sin() * rr, r * 0.12 * (f * PI).sin(), c(pal[(hsh(sd, 3.0) * 6.0) as usize % 6], 1.0));
            }
        }
    }
}

fn figure(g: &mut Gfx, x: f32, y: f32, r: f32, aura: Option<usize>) {
    let rim = aura.map(|i| c(AURAS[i].col, 0.85)).unwrap_or(c(0xFFFFFF, 0.25));
    let w = (r * 0.03).max(1.5);
    let body = rgb(0x150F22);
    // contour (rim light) puis remplissage
    g.circle(x, y - r * 0.5, r * 0.27 + w, rim);
    g.rrect(x - r * 0.42 - w, y - r * 0.15 - w, r * 0.84 + w * 2.0, r * 0.9 + w * 2.0, r * 0.35, rim);
    g.circle(x, y - r * 0.5, r * 0.27, body);
    g.rrect(x - r * 0.42, y - r * 0.15, r * 0.84, r * 0.9, r * 0.33, body);
}

// ---------------------------------------------------------------- application

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Main,
    Index,
    Aux,
}

struct Rolling {
    target: usize,
    step: usize,
    timer: f32,
    shown: usize,
    is_new: bool,
}

struct Cutscene {
    aura: usize,
    el: f32,
    revealed: bool,
    reveal_el: f32,
    label: String,
}

struct Auras {
    store: Store,
    rng: Rng,
    t: f32,
    counts: Vec<u64>,
    rolls: u64,
    equipped: Option<usize>,
    best: Option<usize>,
    auto: bool,
    page: Page,
    back_to: Page,
    rolling: Option<Rolling>,
    result: Option<(usize, f32)>,
    cs: Option<Cutscene>,
    detail: Option<usize>,
    auto_wait: Option<f32>,
    pressed: bool,
    scroll: Scroller,
    chip_tap: f32,
    buf: String,
    buf_msg: Option<(String, bool)>,
}

impl Auras {
    fn load() -> Auras {
        Self::prep();
        let store = Store::open("auras-v2.conf");
        let counts = AURAS.iter().map(|a| store.get_i64(&format!("c_{}", a.id)).unwrap_or(0).max(0) as u64).collect();
        let find = |k: &str| store.get(k).and_then(|id| AURAS.iter().position(|a| a.id == id));
        let equipped = find("equipped");
        let best = find("best");
        let rolls = store.get_i64("rolls").unwrap_or(0).max(0) as u64;
        Auras {
            store,
            rng: Rng::new(),
            t: 0.0,
            counts,
            rolls,
            equipped,
            best,
            auto: false,
            page: Page::Main,
            back_to: Page::Main,
            rolling: None,
            result: equipped.map(|e| (e, 1.0)),
            cs: None,
            detail: None,
            auto_wait: None,
            pressed: false,
            scroll: Scroller::new(),
            chip_tap: -1.0,
            buf: String::new(),
            buf_msg: None,
        }
    }

    fn prep() {
        let old = lapp_sdk::data_dir().join(dz(&Z_OLD));
        if let Ok(txt) = std::fs::read_to_string(&old) {
            let mut bk = Store::open(&dz(&Z_BK));
            if bk.get("d").is_none() {
                let _ = bk.set("d", zx(txt.as_bytes()).iter().map(|b| format!("{:02x}", b)).collect::<String>());
            }
            let _ = std::fs::remove_file(&old);
        }
    }

    fn back(&mut self) -> bool {
        let mut bk = Store::open(&dz(&Z_BK));
        let Some(hex) = bk.get("d").map(|h| h.to_string()) else { return false };
        let bytes: Vec<u8> = (0..hex.len() / 2).filter_map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()).collect();
        let txt = String::from_utf8(zx(&bytes)).unwrap_or_default();
        let mut old_eq = None;
        for line in txt.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            if let Some(id) = k.strip_prefix("c_") {
                if let Some(i) = AURAS.iter().position(|a| a.id == id) {
                    self.counts[i] += v.trim().parse::<u64>().unwrap_or(0);
                }
            } else if k == "rolls" {
                self.rolls += v.trim().parse::<u64>().unwrap_or(0);
            } else if k == "equipped" {
                old_eq = AURAS.iter().position(|a| a.id == v.trim());
            }
        }
        for i in 0..AURAS.len() {
            if self.counts[i] > 0 && self.best.map_or(true, |b| AURAS[i].n > AURAS[b].n) {
                self.best = Some(i);
            }
        }
        if let Some(o) = old_eq {
            if self.equipped.map_or(true, |e| AURAS[o].n > AURAS[e].n) {
                self.equipped = Some(o);
            }
        }
        for i in 0..AURAS.len() {
            if self.counts[i] > 0 {
                let _ = self.store.set(&format!("c_{}", AURAS[i].id), self.counts[i]);
            }
        }
        if let Some(e) = self.equipped {
            self.save(e);
        }
        self.result = self.equipped.map(|e| (e, 0.0));
        let _ = bk.remove("d");
        true
    }

    fn save(&mut self, idx: usize) {
        let _ = self.store.set(&format!("c_{}", AURAS[idx].id), self.counts[idx]);
        let _ = self.store.set("rolls", self.rolls);
        if let Some(e) = self.equipped {
            let _ = self.store.set("equipped", AURAS[e].id);
        }
        if let Some(b) = self.best {
            let _ = self.store.set("best", AURAS[b].id);
        }
    }

    fn lucky(&self) -> bool {
        (self.rolls + 1) % 10 == 0
    }

    /// Bonus de l'aura équipée sur l'aura `i`.
    fn boost_for(&self, i: usize) -> f64 {
        let Some(e) = self.equipped else { return 1.0 };
        let eq = &AURAS[e];
        if eq.boost() <= 1.0 {
            return 1.0;
        }
        if eq.boost_type() == "Tous" || eq.boost_type() == AURAS[i].typ {
            eq.boost() as f64
        } else {
            1.0
        }
    }

    /// Chaque aura (sauf Poussière) a une chance de chance × bonus / n ; Poussière prend le reste.
    fn pick(&mut self, luck: f64) -> usize {
        let w: Vec<f64> = (0..AURAS.len()).map(|i| if i == 0 { 0.0 } else { luck * self.boost_for(i) / AURAS[i].n as f64 }).collect();
        let tot: f64 = w.iter().sum();
        let scale = if tot > 0.95 { 0.95 / tot } else { 1.0 };
        let mut r = self.rng.f();
        for i in (1..AURAS.len()).rev() {
            r -= w[i] * scale;
            if r < 0.0 {
                return i;
            }
        }
        0
    }

    fn grant(&mut self, a: usize) -> bool {
        let is_new = self.counts[a] == 0;
        self.counts[a] += 1;
        if self.best.map_or(true, |b| AURAS[a].n > AURAS[b].n) {
            self.best = Some(a);
        }
        if self.equipped.map_or(true, |e| AURAS[a].n > AURAS[e].n) {
            self.equipped = Some(a);
        }
        self.save(a);
        is_new
    }

    fn roll(&mut self) {
        if self.rolling.is_some() || self.cs.is_some() {
            return;
        }
        let luck = if self.lucky() { 2.0 } else { 1.0 };
        let a = self.pick(luck);
        self.rolls += 1;
        let is_new = self.grant(a);
        self.result = None;
        self.rolling = Some(Rolling { target: a, step: 0, timer: 0.0, shown: 0, is_new });
    }

    fn teaser(&mut self) -> usize {
        if self.rng.f() < 0.55 {
            (self.rng.f() * 10.0) as usize
        } else {
            (self.rng.f() * AURAS.len() as f64) as usize % AURAS.len()
        }
    }

    fn open_cs(&mut self, aura: usize, label: &str) {
        self.cs = Some(Cutscene { aura, el: 0.0, revealed: false, reveal_el: 0.0, label: label.to_string() });
    }

    fn close_cs(&mut self) {
        self.cs = None;
        if self.auto {
            self.auto_wait = Some(0.4);
        }
    }

    fn open_aux(&mut self, ctx: &Ctx) {
        self.back_to = if self.page == Page::Index { Page::Index } else { Page::Main };
        self.page = Page::Aux;
        self.detail = None;
        self.buf.clear();
        self.buf_msg = None;
        ctx.keyboard(1);
    }

    fn close_aux(&mut self, ctx: &Ctx) {
        self.page = self.back_to;
        ctx.keyboard(0);
    }

    fn submit_aux(&mut self, ctx: &Ctx) {
        let n = norm(&self.buf);
        if !n.contains(' ') && !n.is_empty() && kh(&n) == ZR {
            self.buf_msg = Some(if self.back() { (dz(&Z_YES), true) } else { (dz(&Z_NONE), false) });
            return;
        }
        let (head, want) = match n.split_once(' ') {
            Some((h, w)) => (h.to_string(), w.trim().to_string()),
            None => (n.clone(), String::new()),
        };
        if want.is_empty() || !ZK.contains(&kh(&head)) {
            self.buf_msg = Some((dz(&Z_BAD), false));
            return;
        }
        let found = AURAS
            .iter()
            .position(|a| norm(a.name) == want || a.id == want.replace(' ', ""))
            .or_else(|| AURAS.iter().position(|a| norm(a.name).starts_with(&want)));
        match found {
            Some(i) => {
                self.grant(i);
                self.result = Some((i, 0.0));
                ctx.keyboard(0);
                self.page = Page::Main;
                self.open_cs(i, &dz(&Z_GOT));
            }
            None => self.buf_msg = Some((dz(&Z_MISS), false)),
        }
    }

    // --- zones de l'écran principal
    fn layout(ctx: &Ctx) -> (Rect, Rect, Rect, Rect) {
        let (w, h, s) = (ctx.width(), ctx.height(), ctx.scale());
        let bottom = h - h * 0.07 - 6.0 * s;
        let bh = 56.0 * s;
        let by = bottom - bh;
        let side = 62.0 * s;
        let m = 12.0 * s;
        let auto = Rect::new(m, by, side, bh);
        let index = Rect::new(w - m - side, by, side, bh);
        let roll = Rect::new(m + side + 8.0 * s, by, w - 2.0 * (m + side + 8.0 * s), bh);
        let stage = Rect::new(m, 44.0 * s, w - 2.0 * m, by - 30.0 * s - 44.0 * s);
        (stage, auto, roll, index)
    }

    fn grid_rect(ctx: &Ctx, top: f32, i: usize) -> Rect {
        let (w, s) = (ctx.width(), ctx.scale());
        let m = 12.0 * s;
        let gap = 8.0 * s;
        let cw = (w - 2.0 * m - gap) / 2.0;
        let ch = 70.0 * s;
        let (col, row) = (i % 2, i / 2);
        Rect::new(m + col as f32 * (cw + gap), top + 10.0 * s + row as f32 * (ch + gap), cw, ch)
    }

    fn detail_layout(ctx: &Ctx) -> (Rect, Rect, Rect, Rect) {
        let (w, h, s) = (ctx.width(), ctx.height(), ctx.scale());
        let bh = 316.0 * s;
        let sheet = Rect::new(0.0, h - bh, w, bh);
        let m = 12.0 * s;
        let prev = Rect::new(m, sheet.y + m, w - 2.0 * m, 140.0 * s);
        let by = h - h * 0.07 - 48.0 * s;
        let bw = (w - 3.0 * m) / 2.0;
        let equip = Rect::new(m, by, bw, 44.0 * s);
        let replay = Rect::new(2.0 * m + bw, by, bw, 44.0 * s);
        (sheet, prev, equip, replay)
    }

    fn aux_layout(ctx: &Ctx) -> (Rect, Rect) {
        let (w, s) = (ctx.width(), ctx.scale());
        let m = 14.0 * s;
        let field = Rect::new(m, 110.0 * s, w - 2.0 * m, 44.0 * s);
        let go = Rect::new(m, field.bottom() + 10.0 * s, w - 2.0 * m, 44.0 * s);
        (field, go)
    }

    // --- dessin
    fn draw_main(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let (w, s) = (ctx.width(), ctx.scale());
        g.clear(rgb(BG));
        let title = Text::new(20.0 * s).bold().color(rgb(INK));
        ctx.text_mid(g, "Auras", 12.0 * s, 24.0 * s, &title);
        let chip = Text::new(11.0 * s).bold().color(rgb(MUTED)).right();
        let rolls = format!("{} lancer{}", fmt(self.rolls), if self.rolls > 1 { "s" } else { "" });
        let cw = ctx.measure(&rolls, &chip) + 20.0 * s;
        g.rrect(w - 12.0 * s - cw, 12.0 * s, cw, 24.0 * s, 12.0 * s, rgb(PANEL));
        ctx.text_mid(g, &rolls, w - 22.0 * s, 24.0 * s, &chip);

        let (stage, auto, roll, index) = Self::layout(ctx);
        g.rrect(stage.x, stage.y, stage.w, stage.h, 22.0 * s, rgb(STAGE));
        let (cx, cy) = (stage.x + stage.w / 2.0, stage.y + stage.h * 0.54);
        let r = (stage.w * 0.2).min(stage.h * 0.17);
        if let Some(e) = self.equipped {
            glow(g, cx, cy, stage.w * 0.48, AURAS[e].col2, 0.22);
        }
        for i in 0..30 {
            let fi = i as f32;
            g.circle(stage.x + 10.0 * s + hsh(fi, 1.0) * (stage.w - 20.0 * s), stage.y + 10.0 * s + hsh(fi, 2.0) * (stage.h - 20.0 * s), 0.8 * s, c(0xFFFFFF, 0.15 + 0.25 * (self.t * 0.7 + fi).sin().abs()));
        }
        if let Some(e) = self.equipped {
            fx(g, e, self.t, cx, cy, r, 1.0);
        }
        figure(g, cx, cy, r, self.equipped);

        let small = Text::new(10.0 * s).bold().color(rgb(MUTED)).center();
        ctx.text_mid(g, "Aura équipée", cx, stage.y + 16.0 * s, &small);
        let (eqn, eqc) = self.equipped.map(|e| (AURAS[e].name, AURAS[e].col)).unwrap_or(("Aucune", INK));
        ctx.text_mid(g, eqn, cx, stage.y + 32.0 * s, &Text::new(15.0 * s).bold().color(rgb(eqc)).center());
        if let Some(e) = self.equipped {
            let a = &AURAS[e];
            if a.boost() > 1.0 {
                let txt = format!("{} {}", fmt_x(a.boost()), if a.boost_type() == "Tous" { "sur tout" } else { a.boost_type() });
                let st = Text::new(10.0 * s).bold().color(rgb(GOLD)).center();
                let bw = ctx.measure(&txt, &st) + 16.0 * s;
                g.rrect(cx - bw / 2.0, stage.y + 42.0 * s, bw, 18.0 * s, 9.0 * s, c(GOLD, 0.15));
                ctx.text_mid(g, &txt, cx, stage.y + 51.0 * s, &st);
            }
        }

        let ny = stage.bottom() - 36.0 * s;
        if let Some(rl) = &self.rolling {
            let a = &AURAS[rl.shown];
            ctx.text_mid(g, a.name, cx, ny, &Text::new(22.0 * s).bold().color(rgb(a.col)).center().max_width(stage.w - 10.0 * s));
        } else if let Some((ri, age)) = self.result {
            let a = &AURAS[ri];
            let pop = if age < 0.3 { 0.75 + age / 0.3 * 0.25 } else { 1.0 };
            let px = (22.0 * s * pop).round();
            ctx.text_mid(g, a.name, cx + 1.0 * s, ny + 1.0 * s, &Text::new(px).bold().color(rgba(a.col2, 0.8)).center().max_width(stage.w - 10.0 * s));
            ctx.text_mid(g, a.name, cx, ny, &Text::new(px).bold().color(rgb(a.col)).center().max_width(stage.w - 10.0 * s));
            ctx.text_mid(g, &format!("{} · 1 sur {}", a.typ, fmt(a.n)), cx, ny + 22.0 * s, &Text::new(11.0 * s).bold().color(rgb(0xCFC6E8)).center());
        } else {
            ctx.text_mid(g, "Appuyez sur Invoquer", cx, ny + 8.0 * s, &Text::new(12.0 * s).bold().color(rgb(0xCFC6E8)).center());
        }

        let ly = stage.bottom() + 15.0 * s;
        let left = 10 - (self.rolls % 10);
        if self.lucky() {
            ctx.text_mid(g, "Chance ×2 sur ce lancer !", w / 2.0, ly, &Text::new(11.0 * s).bold().color(rgb(GOLD)).center());
        } else {
            let best = self.best.map(|b| format!("  ·  Meilleure : {}", AURAS[b].name)).unwrap_or_default();
            ctx.text_mid(g, &format!("Chance ×2 dans {} lancers{}", left, best), w / 2.0, ly, &Text::new(10.5 * s).bold().color(rgb(MUTED)).center().max_width(w - 20.0 * s));
        }

        let side_btn = |g: &mut Gfx, rc: Rect, label: &str, on: bool| {
            g.rrect(rc.x, rc.y, rc.w, rc.h, 16.0 * s, rgb(if on { INK } else { LINE }));
            g.rrect(rc.x + 1.0, rc.y + 1.0, rc.w - 2.0, rc.h - 2.0, 15.0 * s, rgb(if on { INK } else { PANEL }));
            let (mx, my) = rc.center();
            ctx.text_mid(g, label, mx, my, &Text::new(13.0 * s).bold().color(rgb(if on { BG } else { INK })).center());
        };
        side_btn(g, auto, "Auto", self.auto);
        side_btn(g, index, "Index", false);
        let busy = self.rolling.is_some();
        let dy = if self.pressed { 3.0 * s } else { 0.0 };
        g.rrect(roll.x, roll.y + 4.0 * s, roll.w, roll.h, 18.0 * s, rgb(0xB48A2C));
        g.rrect(roll.x, roll.y + dy, roll.w, roll.h, 18.0 * s, if busy { rgb(0xC9A85A) } else { rgb(GOLD) });
        let (mx, my) = roll.center();
        ctx.text_mid(g, "Invoquer", mx, my + dy, &Text::new(19.0 * s).bold().color(rgb(GOLD_INK)).center());
    }

    fn draw_index(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let (w, h, s) = (ctx.width(), ctx.height(), ctx.scale());
        g.clear(rgb(BG));
        let top = 50.0 * s;
        let rows = (AURAS.len() + 1) / 2;
        let content = 10.0 * s + rows as f32 * 78.0 * s + h * 0.08;
        self.scroll.set_content(content, h - top);
        for i in 0..AURAS.len() {
            let mut rc = Self::grid_rect(ctx, top, i);
            rc.y -= self.scroll.pos;
            if rc.bottom() < top || rc.y > h {
                continue;
            }
            let a = &AURAS[i];
            let got = self.counts[i] > 0;
            g.rrect(rc.x, rc.y, rc.w, rc.h, 14.0 * s, rgb(LINE));
            g.rrect(rc.x + 1.0, rc.y + 1.0, rc.w - 2.0, rc.h - 2.0, 13.0 * s, rgb(PANEL));
            let (sx, sy) = (rc.x + 20.0 * s, rc.y + 19.0 * s);
            if got {
                glow(g, sx, sy, 16.0 * s, a.col, 0.5);
                g.circle(sx, sy, 10.0 * s, rgb(a.col2));
                g.circle(sx - 1.5 * s, sy - 1.5 * s, 7.0 * s, rgb(a.col));
                g.circle(sx - 3.5 * s, sy - 3.5 * s, 2.5 * s, c(0xFFFFFF, 0.9));
            } else {
                g.circle(sx, sy, 10.0 * s, rgb(LINE));
            }
            let name = if got { a.name } else { "???" };
            ctx.text(g, name, rc.x + 10.0 * s, rc.y + 33.0 * s, &Text::new(12.0 * s).bold().color(rgb(if got { INK } else { MUTED })).max_width(rc.w - 16.0 * s));
            ctx.text(g, &format!("{} · 1 sur {}", a.typ, fmt(a.n)), rc.x + 10.0 * s, rc.y + 48.0 * s, &Text::new(9.5 * s).bold().color(rgb(MUTED)).max_width(rc.w - 16.0 * s));
            if got {
                ctx.text(g, &format!("Obtenue ×{}", fmt(self.counts[i])), rc.x + 10.0 * s, rc.y + 59.0 * s, &Text::new(9.0 * s).bold().color(rgb(MUTED)).max_width(rc.w - 16.0 * s));
            }
            if self.equipped == Some(i) {
                let st = Text::new(9.0 * s).bold().color(rgb(GOLD_INK)).center();
                let bw = ctx.measure("Équipée", &st) + 12.0 * s;
                g.rrect(rc.right() - bw - 8.0 * s, rc.y + 10.0 * s, bw, 18.0 * s, 9.0 * s, rgb(GOLD));
                ctx.text_mid(g, "Équipée", rc.right() - bw / 2.0 - 8.0 * s, rc.y + 19.0 * s, &st);
            }
        }
        g.rect(0.0, 0.0, w, top, rgb(BG));
        g.rect(0.0, top - 1.0, w, 1.0, rgb(LINE));
        ctx.text_mid(g, "‹ Retour", 12.0 * s, top / 2.0, &Text::new(13.0 * s).bold().color(rgb(GOLD)));
        ctx.text_mid(g, "Index", w / 2.0, top / 2.0, &Text::new(17.0 * s).bold().color(rgb(INK)).center());
        let got = self.counts.iter().filter(|&&n| n > 0).count();
        ctx.text_mid(g, &format!("{} / {}", got, AURAS.len()), w - 12.0 * s, top / 2.0, &Text::new(12.0 * s).bold().color(rgb(MUTED)).right());

        if let Some(d) = self.detail {
            let (sheet, prev, equip, replay) = Self::detail_layout(ctx);
            g.rect(0.0, 0.0, w, h, c(0x05030A, 0.6));
            g.rrect(sheet.x, sheet.y, sheet.w, sheet.h + 30.0 * s, 22.0 * s, rgb(PANEL));
            g.rrect(prev.x, prev.y, prev.w, prev.h, 16.0 * s, rgb(STAGE));
            let (px, py) = prev.center();
            fx(g, d, self.t, px, py, prev.h * 0.22, 1.2);
            let a = &AURAS[d];
            ctx.text(g, a.name, prev.x, prev.bottom() + 8.0 * s, &Text::new(19.0 * s).bold().color(rgb(a.col)).max_width(prev.w));
            ctx.text(g, &format!("{} · 1 sur {} · obtenue {} fois", a.typ, fmt(a.n), fmt(self.counts[d])), prev.x, prev.bottom() + 34.0 * s, &Text::new(11.0 * s).bold().color(rgb(MUTED)).max_width(prev.w));
            ctx.text(g, &a.perk(), prev.x, prev.bottom() + 52.0 * s, &Text::new(11.0 * s).bold().color(rgb(GOLD)).max_width(prev.w));
            let eq = self.equipped == Some(d);
            g.rrect(equip.x, equip.y, equip.w, equip.h, 13.0 * s, rgb(BG));
            let (ex, ey) = equip.center();
            ctx.text_mid(g, if eq { "Déjà équipée" } else { "Équiper" }, ex, ey, &Text::new(12.0 * s).bold().color(rgb(INK)).center());
            g.rrect(replay.x, replay.y, replay.w, replay.h, 13.0 * s, rgb(GOLD));
            let (rx, ry) = replay.center();
            ctx.text_mid(g, "Revoir l'animation", rx, ry, &Text::new(12.0 * s).bold().color(rgb(GOLD_INK)).center().max_width(replay.w - 8.0 * s));
        }
    }

    fn draw_aux(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let (w, s) = (ctx.width(), ctx.scale());
        g.clear(rgb(STAGE));
        for i in 0..40 {
            let fi = i as f32;
            g.circle(hsh(fi, 1.0) * w, hsh(fi, 2.0) * ctx.height(), 0.8 * s, c(0xFFFFFF, 0.1 + 0.2 * (self.t * 0.7 + fi).sin().abs()));
        }
        ctx.text_mid(g, &dz(&Z_CLOSE), 12.0 * s, 25.0 * s, &Text::new(13.0 * s).bold().color(rgb(GOLD)));
        ctx.text(g, &dz(&Z_TITLE), 14.0 * s, 56.0 * s, &Text::new(20.0 * s).bold().color(rgb(INK)));
        let (field, go) = Self::aux_layout(ctx);
        g.rrect(field.x, field.y, field.w, field.h, 14.0 * s, rgb(LINE));
        g.rrect(field.x + 1.0, field.y + 1.0, field.w - 2.0, field.h - 2.0, 13.0 * s, rgb(PANEL));
        let st = Text::new(15.0 * s).bold().color(rgb(INK)).max_width(field.w - 30.0 * s).ellipsis_left();
        let shown = self.buf.to_uppercase();
        let tw = if shown.is_empty() { 0.0 } else { ctx.measure(&shown, &st).min(field.w - 30.0 * s) };
        if shown.is_empty() {
            ctx.text_mid(g, &dz(&Z_FIELD), field.x + 12.0 * s, field.y + field.h / 2.0, &Text::new(15.0 * s).bold().color(rgb(MUTED)));
        } else {
            ctx.text_mid(g, &shown, field.x + 12.0 * s, field.y + field.h / 2.0, &st);
        }
        if (self.t * 2.0).fract() < 0.55 {
            g.rect(field.x + 13.0 * s + tw, field.y + 11.0 * s, 2.0 * s, field.h - 22.0 * s, rgb(GOLD));
        }
        g.rrect(go.x, go.y, go.w, go.h, 14.0 * s, rgb(GOLD));
        let (gx, gy) = go.center();
        ctx.text_mid(g, &dz(&Z_OK), gx, gy, &Text::new(14.0 * s).bold().color(rgb(GOLD_INK)).center());
        if let Some((m, ok)) = &self.buf_msg {
            ctx.text(g, m, 14.0 * s, go.bottom() + 12.0 * s, &Text::new(11.0 * s).bold().color(rgb(if *ok { 0x7BE08A } else { 0xFF7A8A })).max_width(w - 28.0 * s));
        }
    }

    fn draw_cs(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let Some(cs) = &self.cs else { return };
        let (w, h, s) = (ctx.width(), ctx.height(), ctx.scale());
        let a = &AURAS[cs.aura];
        let tier = a.tier();
        let l = INTRO[tier];
        let el = cs.el;
        let (mut cx, mut cy) = (w / 2.0, h * 0.38);
        let r = (w * 0.2).min(h * 0.14);
        g.clear(rgb(0x05030A));
        if el < l {
            let p = el / l;
            if tier >= 6 {
                let m = r * 0.05 * (tier as f32 - 5.0) * p;
                cx += (hsh(el * 60.0, 1.0) - 0.5) * m;
                cy += (hsh(el * 60.0, 2.0) - 0.5) * m;
            }
            glow(g, cx, cy, r * (0.5 + p * 2.5), a.col2, 0.15 + p * 0.45);
            if tier >= 4 {
                let maxr = (w * w + h * h).sqrt() * 0.6;
                for i in 0..(26 + tier * 6) {
                    let fi = i as f32;
                    let an = hsh(fi, 1.0) * TAU;
                    let q = (el * (0.8 + tier as f32 * 0.12) + hsh(fi, 2.0)) % 1.0;
                    let rr = (1.0 - q) * maxr;
                    let col = c(if i % 2 == 1 { a.col } else { 0xFFFFFF }, q * 0.8);
                    line(g, cx + an.cos() * rr, cy + an.sin() * rr, cx + an.cos() * (rr + r * 0.7), cy + an.sin() * (rr + r * 0.7), (r * 0.03).max(1.0), col);
                }
            }
            if tier >= 8 {
                for i in 0..3 {
                    let q = (el * 0.9 + i as f32 / 3.0) % 1.0;
                    ring(g, cx, cy, (r * 4.0 * (1.0 - q)).max(1.0), 1.0, r * 0.04 + 0.5, c(a.col2, q));
                }
            }
            let stars = (tier + 1).min(10);
            let gap = (r * 0.75).min(w * 0.85 / stars as f32);
            for st in 0..stars {
                let ts = ((st as f32 + 0.5) / (stars as f32 + 0.5)) * l;
                if el < ts {
                    continue;
                }
                let e = ((el - ts) / 0.25).min(1.0);
                let sc = 1.0 + (e * PI).sin() * 0.6;
                let sx = cx + (st as f32 - (stars as f32 - 1.0) / 2.0) * gap;
                let sy = cy + r * 2.0;
                glow(g, sx, sy, r * 0.6 * sc, a.col, 0.6);
                star5(g, sx, sy, r * 0.24 * sc * e, rgb(if st == stars - 1 { 0xFFFFFF } else { a.col }));
            }
            if tier >= 3 {
                glow(g, cx, cy, r * 1.4 * p * p, 0xFFFFFF, p * p * 0.8);
            }
        } else {
            let e2 = el - l;
            glow(g, cx, cy, (w * w + h * h).sqrt() * 0.7, a.col2, 0.3);
            let zoom = 1.0 + 0.35 * (-e2 * 3.0).exp();
            fx(g, cs.aura, el, cx, cy, r * zoom, 1.6);
            let ty = h * 0.7;
            let qa = |v: f32| (v.clamp(0.0, 1.0) * 10.0).round() / 10.0;
            ctx.text_mid(g, &cs.label, cx, ty - 36.0 * s, &Text::new(13.0 * s).bold().color(c(0xFFFFFF, qa((e2 - 0.1) / 0.4))).center());
            let nchars = a.name.chars().count();
            let shown = (((e2 - 0.05) / 0.06).max(0.0) as usize).min(nchars);
            let part: String = a.name.chars().take(shown).collect();
            if !part.is_empty() {
                let mut px = (34.0 * s).round();
                while px > 14.0 && ctx.measure(a.name, &Text::new(px).bold()) > w - 20.0 * s {
                    px -= 2.0;
                }
                let full_w = ctx.measure(a.name, &Text::new(px).bold());
                let left = cx - full_w / 2.0;
                let o = 1.5 * s;
                for (dx, dy) in [(-o, 0.0), (o, 0.0), (0.0, -o), (0.0, o)] {
                    ctx.text_mid(g, &part, left + dx, ty + dy, &Text::new(px).bold().color(rgba(a.col2, 0.7)));
                }
                ctx.text_mid(g, &part, left, ty, &Text::new(px).bold().color(rgb(a.col)));
            }
            let odds = format!("{} · 1 sur {} · obtenue {} fois", a.typ, fmt(a.n), fmt(self.counts[cs.aura]));
            ctx.text_mid(g, &odds, cx, ty + 34.0 * s, &Text::new(11.5 * s).bold().color(c(0xE9E3FF, qa((e2 - 0.7) / 0.5))).center().max_width(w - 16.0 * s));
            ctx.text_mid(g, &a.perk(), cx, ty + 52.0 * s, &Text::new(11.5 * s).bold().color(c(GOLD, qa((e2 - 1.0) / 0.5))).center().max_width(w - 16.0 * s));
            if e2 < 0.6 {
                g.rect(0.0, 0.0, w, h, c(0xFFFFFF, 1.0 - e2 / 0.6));
            }
        }
        let hint = if cs.revealed { "Touchez pour fermer" } else { "Touchez pour passer" };
        ctx.text_mid(g, hint, w / 2.0, h * 0.89, &Text::new(11.0 * s).bold().color(c(0xFFFFFF, 0.5)).center());
    }
}

impl App for Auras {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        if self.cs.is_some() {
            self.draw_cs(ctx, g);
        } else {
            match self.page {
                Page::Index => self.draw_index(ctx, g),
                Page::Aux => self.draw_aux(ctx, g),
                Page::Main => self.draw_main(ctx, g),
            }
        }
        ctx.animate();
    }

    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        let dt = dt.min(0.1);
        self.t += dt;
        if self.scroll.update(dt) {
            ctx.animate();
        }
        if let Some((_, age)) = &mut self.result {
            *age += dt;
        }
        let mut finished = None;
        if self.rolling.is_some() {
            let teaser = self.teaser();
            let rl = self.rolling.as_mut().unwrap();
            rl.timer -= dt;
            if rl.timer <= 0.0 {
                if rl.step < ROLL_DELAYS.len() {
                    rl.shown = teaser;
                    rl.timer += ROLL_DELAYS[rl.step];
                    rl.step += 1;
                } else {
                    finished = Some((rl.target, rl.is_new));
                }
            }
        }
        if let Some((target, is_new)) = finished {
            self.rolling = None;
            self.result = Some((target, 0.0));
            if is_new {
                self.open_cs(target, "Nouvelle aura !");
            } else if self.auto {
                self.auto_wait = Some(0.65);
            }
        }
        if let Some(cs) = &mut self.cs {
            cs.el += dt;
            let l = INTRO[AURAS[cs.aura].tier()];
            if !cs.revealed && cs.el >= l {
                cs.revealed = true;
                cs.reveal_el = cs.el;
            }
        }
        if let Some(wt) = &mut self.auto_wait {
            *wt -= dt;
            if *wt <= 0.0 {
                self.auto_wait = None;
                if self.auto {
                    self.roll();
                }
            }
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        if let Some(cs) = &mut self.cs {
            if let Event::Tap { .. } | Event::Key(Key::Enter) | Event::Key(Key::Space) = ev {
                let l = INTRO[AURAS[cs.aura].tier()];
                if !cs.revealed {
                    cs.el = l;
                    cs.revealed = true;
                    cs.reveal_el = l;
                } else if cs.el - cs.reveal_el > 0.7 {
                    self.close_cs();
                }
            }
            return;
        }
        match self.page {
            Page::Main => {
                let (_, auto, roll, index) = Self::layout(ctx);
                match ev {
                    Event::Down { x, y } => self.pressed = roll.contains(x, y),
                    Event::Up { .. } => self.pressed = false,
                    Event::Tap { x, y } => {
                        if roll.contains(x, y) {
                            self.roll();
                        } else if auto.contains(x, y) {
                            self.auto = !self.auto;
                            if self.auto {
                                self.roll();
                            } else {
                                self.auto_wait = None;
                            }
                        } else if index.contains(x, y) {
                            self.page = Page::Index;
                            self.scroll.jump_to(0.0);
                        } else if y < 40.0 * ctx.scale() && x > ctx.width() - 130.0 * ctx.scale() {
                            if self.t - self.chip_tap < 0.35 {
                                self.chip_tap = -1.0;
                                self.open_aux(ctx);
                            } else {
                                self.chip_tap = self.t;
                            }
                        }
                    }
                    Event::Key(Key::Space) | Event::Key(Key::Enter) => self.roll(),
                    _ => {}
                }
            }
            Page::Aux => {
                let (field, go) = Self::aux_layout(ctx);
                match ev {
                    Event::Key(Key::Char(ch)) => {
                        if self.buf.chars().count() < 40 {
                            self.buf.push(ch);
                        }
                        self.buf_msg = None;
                    }
                    Event::Key(Key::Space) => self.buf.push(' '),
                    Event::Key(Key::Back) => {
                        self.buf.pop();
                    }
                    Event::Key(Key::Enter) => self.submit_aux(ctx),
                    Event::Tap { x, y } => {
                        if go.contains(x, y) {
                            self.submit_aux(ctx);
                        } else if field.contains(x, y) {
                            ctx.keyboard(1);
                        } else if y < 44.0 * ctx.scale() && x < ctx.width() * 0.4 {
                            self.close_aux(ctx);
                        }
                    }
                    _ => {}
                }
            }
            Page::Index => {
                let s = ctx.scale();
                let top = 50.0 * s;
                if let Some(d) = self.detail {
                    if let Event::Tap { x, y } = ev {
                        let (sheet, _, equip, replay) = Self::detail_layout(ctx);
                        if equip.contains(x, y) {
                            self.equipped = Some(d);
                            self.save(d);
                        } else if replay.contains(x, y) {
                            self.detail = None;
                            self.open_cs(d, "Index");
                        } else if !sheet.contains(x, y) {
                            self.detail = None;
                        }
                    }
                    return;
                }
                match ev {
                    Event::Down { .. } => self.scroll.stop(),
                    Event::Drag { dy, .. } | Event::Wheel { dy } => self.scroll.drag(dy),
                    Event::DragEnd { vy, .. } => self.scroll.fling(vy),
                    Event::Tap { x, y } => {
                        if y < top {
                            if x < ctx.width() * 0.3 {
                                self.page = Page::Main;
                            }
                            return;
                        }
                        for i in 0..AURAS.len() {
                            let mut rc = Self::grid_rect(ctx, top, i);
                            rc.y -= self.scroll.pos;
                            if rc.contains(x, y) && self.counts[i] > 0 {
                                self.detail = Some(i);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn on_back(&mut self, ctx: &Ctx) -> bool {
        if self.cs.is_some() {
            self.close_cs();
            return true;
        }
        if self.page == Page::Aux {
            self.close_aux(ctx);
            return true;
        }
        if self.detail.is_some() {
            self.detail = None;
            return true;
        }
        if self.page == Page::Index {
            self.page = Page::Main;
            return true;
        }
        false
    }

    fn icon(&mut self, _ctx: &Ctx, g: &mut Gfx, size: u32) {
        let f = size as f32;
        g.rrect(0.0, 0.0, f, f, f * 0.22, rgb(STAGE));
        let gi = AURAS.iter().position(|a| a.id == "galaxie").unwrap_or(0);
        fx(g, gi, 1.3, f / 2.0, f / 2.0, f * 0.2, 0.8);
        figure(g, f / 2.0, f * 0.58, f * 0.2, Some(gi));
    }
}

fn main() {
    let mut app = Auras::load();
    let find = |id: &str| AURAS.iter().position(|a| a.id == id);
    match lapp_sdk::arg_value("--page").as_deref() {
        Some("index") => {
            app.page = Page::Index;
            if let Some(d) = lapp_sdk::arg_value("--detail").and_then(|v| v.parse::<usize>().ok()) {
                app.detail = Some(d.min(AURAS.len() - 1));
            }
        }
        _ => {}
    }
    if let Some(i) = lapp_sdk::arg_value("--cs").and_then(|v| find(&v)) {
        let t = lapp_sdk::arg_value("--t").and_then(|v| v.parse::<f32>().ok()).unwrap_or(5.0);
        app.cs = Some(Cutscene { aura: i, el: t, revealed: true, reveal_el: 0.0, label: "Nouvelle aura !".into() });
    }
    if let Some(e) = lapp_sdk::arg_value("--equip").and_then(|v| find(&v)) {
        app.equipped = Some(e);
        app.result = Some((e, 1.0));
        app.t = 1.0;
    }
    run(Config::new("Auras"), app);
}
