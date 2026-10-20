// Conquis Cliqueur : jeu cliqueur infini pour lapp (écran portrait 272x480).
//
// - Échap (ou bouton ≡ en haut à droite) : menu (Reprendre / Entrer un code / Quitter)
// - Code BUG  : vidéo plein écran, impossible à fermer avant la fin.
// - Code TUNE : interface secrète qui demande un 2e code
//     INFINI -> or ×4 · DIAMEN -> diamant ×1000 · NETHERITE -> noir ×10000 (pour toujours)
//     MAX -> 3e code : GALAXI -> château violet à galaxie animée ×100000 · MAXGALAXI -> galaxie MAX ×1000000
//                      TROU NOIR -> château trou noir ×10000000 (anneau de lumière, clics aspirés)
//                      ALIEN -> château alien ×100000000 (soucoupe, faisceau tracteur, radar)
//                      un code caché ouvre un 4e code (caché lui aussi : seules les empreintes K3 / K4 sont ici)
//                      -> château Éclipse ×100 milliards, 2 orbes d'énergie, cliqueurs ×8
// - Onglet Châteaux : on peut aussi acheter le château en or et en diamant.
// - Paramètres : compteur de FPS + test de puissance (processeur puis graphismes).
// - Après la Dimension 52 : « Le Big Bang » (remet tout à zéro, double tous les multiplicateurs).
// - Onglet Châteaux : achète des châteaux avec tes territoires, chacun donne un multiplicateur en plus.
// - Onglet Magie : sorts (mana) et améliorations magiques à débloquer.
// - Boutique infinie : 8 unités puis des « Dimensions » sans fin.
//
// Sur PC : cargo run
//          cargo run -- --screenshot out.png [--page menu|code|tune|video|magic] [--vt 8] [--skin 1|2] [--demo]

use lapp_sdk::prelude::*;
use lapp_sdk::{arg_flag, arg_value, load_png};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const VIDEO_FPS: f32 = 6.0;
const VIDEO_FRAMES: usize = 232;
const GROWTH: f64 = 1.15;
const MAX_CODE: usize = 16;
const OFFLINE_CAP: f64 = 8.0 * 3600.0;
const TIERS: usize = 60;
// Codes secrets : seules des empreintes PBKDF2-SHA256 (1 million de tours) sont stockées, pas le texte.
// Le 4e code est lié au 3e : son empreinte porte sur « 3e code + saut de ligne + 4e code ».
const K3: &str = "21b382f76e2d3353ea6755be270f9491140d1e71d97833b0a96c76ce5a443b83";
const K4: &str = "55555ea2bba92b442a72890adb87bcaff5157cc6f0a8c72f1bbe1b736a581b38";
/// 4e code (bis) : château Éclipse ultime ; code caché du 1er niveau : Extension du territoire infinie.
const K5: &str = "111a8c1f076d71f32985015780327e3528e59e6cf16e5a463ba62c1432f73d4b";
const K6: &str = "14f8c07c1f79dc76e04528b28898e3c3ccffc57c0d1907de712d7f336c3ba68c";

// SHA-256 + HMAC + PBKDF2, sans dépendance.
const K: [u32; 64] = [
    0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
    0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
    0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
    0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
    0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
    0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
    0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
    0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2];
fn sha256(msg: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19];
    let ml = (msg.len() as u64) * 8;
    let mut data = msg.to_vec();
    data.push(0x80);
    while data.len() % 64 != 56 { data.push(0); }
    data.extend_from_slice(&ml.to_be_bytes());
    for chunk in data.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 { w[i] = u32::from_be_bytes([chunk[i*4],chunk[i*4+1],chunk[i*4+2],chunk[i*4+3]]); }
        for i in 16..64 {
            let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
            let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }
        let mut v = h;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ ((!v[4]) & v[6]);
            let t1 = v[7].wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [t1.wrapping_add(t2),v[0],v[1],v[2],v[3].wrapping_add(t1),v[4],v[5],v[6]];
        }
        for i in 0..8 { h[i] = h[i].wrapping_add(v[i]); }
    }
    let mut out = [0u8; 32];
    for i in 0..8 { out[i*4..i*4+4].copy_from_slice(&h[i].to_be_bytes()); }
    out
}
fn hmac(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut k = if key.len() > 64 { sha256(key).to_vec() } else { key.to_vec() };
    k.resize(64, 0);
    let ipad: Vec<u8> = k.iter().map(|b| b ^ 0x36).collect();
    let opad: Vec<u8> = k.iter().map(|b| b ^ 0x5c).collect();
    let mut inner = ipad; inner.extend_from_slice(msg);
    let ih = sha256(&inner);
    let mut outer = opad; outer.extend_from_slice(&ih);
    sha256(&outer)
}
fn pbkdf2(pw: &[u8], salt: &[u8], iters: u32) -> String {
    let mut block = salt.to_vec();
    block.extend_from_slice(&1u32.to_be_bytes());
    let mut u = hmac(pw, &block);
    let mut out = u;
    for _ in 1..iters {
        u = hmac(pw, &u);
        for i in 0..32 { out[i] ^= u[i]; }
    }
    out.iter().map(|b| format!("{:02x}", b)).collect()
}

fn code_hash(pw: &str, salt: &str) -> String {
    pbkdf2(pw.as_bytes(), salt.as_bytes(), 1_000_000)
}

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

static MAGIC: [MagicDef; 15] = [
    MagicDef { id: "frenesie", name: "Frénésie", spell: true, cost: 500.0, req: 250.0, mana: 40.0, dur: 15.0, desc: "Clics ×7 pendant 15 s" },
    MagicDef { id: "pluie", name: "Pluie d’or", spell: true, cost: 5_000.0, req: 2_500.0, mana: 30.0, dur: 0.0, desc: "+60 s de production" },
    MagicDef { id: "res1", name: "Réservoir de mana I", spell: false, cost: 2_000.0, req: 1_000.0, mana: 0.0, dur: 0.0, desc: "Mana max +50" },
    MagicDef { id: "source", name: "Source arcanique", spell: false, cost: 20_000.0, req: 10_000.0, mana: 0.0, dur: 0.0, desc: "Le mana revient 2× plus vite" },
    MagicDef { id: "benediction", name: "Bénédiction", spell: true, cost: 50_000.0, req: 25_000.0, mana: 60.0, dur: 20.0, desc: "Production ×3 pendant 20 s" },
    MagicDef { id: "res2", name: "Réservoir de mana II", spell: false, cost: 200_000.0, req: 100_000.0, mana: 0.0, dur: 0.0, desc: "Mana max +100" },
    MagicDef { id: "main", name: "Main de mage", spell: false, cost: 1_000_000.0, req: 500_000.0, mana: 0.0, dur: 0.0, desc: "Clic +10 % de la prod/s" },
    MagicDef { id: "grimoire", name: "Grimoire ancien", spell: false, cost: 50_000_000.0, req: 25_000_000.0, mana: 0.0, dur: 0.0, desc: "Toute la production ×2" },
    MagicDef { id: "clone", name: "Clone magique", spell: true, cost: 500_000.0, req: 250_000.0, mana: 50.0, dur: 20.0, desc: "Cliqueurs ×3 vitesse 20 s" },
    MagicDef { id: "pacte", name: "Pacte d’or", spell: false, cost: 5e6, req: 2.5e6, mana: 0.0, dur: 0.0, desc: "Pluie d’or ×3" },
    MagicDef { id: "cristal", name: "Cristal de mana", spell: false, cost: 1e8, req: 5e7, mana: 0.0, dur: 0.0, desc: "Mana max +150" },
    MagicDef { id: "sablier", name: "Sablier éternel", spell: false, cost: 2e9, req: 1e9, mana: 0.0, dur: 0.0, desc: "Sorts 2× plus longs" },
    MagicDef { id: "distorsion", name: "Distorsion temporelle", spell: true, cost: 1e10, req: 5e9, mana: 120.0, dur: 0.0, desc: "+10 min de production" },
    MagicDef { id: "ascension", name: "Ascension arcanique", spell: true, cost: 1e12, req: 5e11, mana: 220.0, dur: 10.0, desc: "Tout ×10 pendant 10 s" },
    MagicDef { id: "archimage", name: "Archimage", spell: false, cost: 1e15, req: 5e14, mana: 0.0, dur: 0.0, desc: "Clics et prod ×3 toujours" },
];

/// Châteaux des codes qu'on peut aussi acheter : (nom, château, coût)
static SKBUY: [(&str, u8, f64); 3] = [("Château en or", 1, 1e6), ("Château en diamant", 2, 1e11), ("Château en netherite", 3, 1e16)];

// (nom, territoires conquis requis, multiplicateur) : s'ajoute au multiplicateur du château
static GRADES: [(&str, f64, f64); 12] = [
    ("Paysan", 0.0, 1.0),
    ("Écuyer", 1e3, 1.1),
    ("Chevalier", 1e5, 1.25),
    ("Baron", 1e7, 1.5),
    ("Comte", 1e9, 2.0),
    ("Duc", 1e11, 3.0),
    ("Prince", 1e13, 5.0),
    ("Roi", 1e15, 8.0),
    ("Empereur", 1e18, 15.0),
    ("Demi-dieu", 1e21, 30.0),
    ("Dieu", 1e25, 75.0),
    ("Éternel", 1e30, 200.0),
];

// Châteaux à acheter : (nom, coût en territoires, multiplicateur en plus de tous les autres)
static CASTLES: [(&str, f64, f64); 16] = [
    ("Tour de guet", 5e3, 1.2),
    ("Fort en bois", 1e5, 1.5),
    ("Donjon de pierre", 5e6, 2.0),
    ("Citadelle", 5e8, 3.0),
    ("Forteresse royale", 1e11, 5.0),
    ("Palais impérial", 5e13, 10.0),
    ("Château céleste", 1e17, 25.0),
    ("Château éternel", 1e21, 100.0),
    ("Tour de cristal", 1e25, 250.0),
    ("Bastion des tempêtes", 1e30, 500.0),
    ("Citadelle des anciens", 1e36, 1000.0),
    ("Château des nuages", 1e42, 2500.0),
    ("Forteresse stellaire", 1e50, 5000.0),
    ("Palais du temps", 1e60, 1e4),
    ("Château de l’infini", 1e75, 1e5),
    ("Château du multivers", 1e100, 1e6),
];

/// Capacités des châteaux : (nom, château requis, durée s, recharge s, [inutilisé], description courte)
static AB: [(&str, u8, f64, f64, u8, &str); 5] = [
    ("Le Vide", 6, 180.0, 480.0, 0, "Château ×100 · 3 min · recharge 8 min"),
    ("Extension du territoire", 8, 300.0, 300.0, 0, "Tout ×1 milliard · 5 min · recharge 5 min"),
    ("Avancement temporel", 9, 0.0, 600.0, 0, "5 min de production d’un coup · recharge 10 min"),
    ("Orbe violette", 8, 0.0, 60.0, 0, "Détruit le boss d’un coup · recharge 1 min"),
    ("Éclat du soleil", 11, 180.0, 600.0, 0, "Château ×1 million · 3 min · recharge 10 min"),
];

static SKIN_NAMES: [&str; 12] = ["Pierre", "Or", "Diamant", "Netherite", "Galaxie", "Galaxie MAX", "Trou noir", "Alien", "Éclipse", "Temps", "Éclipse ultime", "Solaire"];

/// Multiplicateur de base de chaque château.
fn skin_base(k: u8) -> f64 {
    [1.0, 4.0, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e11, 1e9, 1e13, 1e10][(k as usize).min(11)]
}

static BOSS_NAMES: [&str; 12] = ["Golem de roche", "Gardien doré", "Colosse de cristal", "Brute de netherite", "Titan stellaire", "Titan cosmique", "Dévoreur d’horizon", "Envahisseur", "Roi des malédictions", "Gardien du temps", "Roi des malédictions", "Seigneur solaire"];
/// Gardes de l'onglet Défense : un skin par château.
static GNAME: [&str; 12] = ["Chevalier de pierre", "Paladin doré", "Sentinelle de cristal", "Brute de netherite", "Mage stellaire", "Mage cosmique", "Spectre du vide", "Soldat alien", "Chevalier de l’éclipse", "Automate du temps", "Maître de l’Infini", "Prêtre solaire"];
static GCOL: [(u32, u32); 12] = [(0xD6D8DE, 0x8C8F99), (0xFFE27A, 0xF2C94C), (0xC8F6FF, 0x5FD3F0), (0xFFB347, 0xFF5A14), (0xE2C6FF, 0x9B5AFF), (0xFFF0A0, 0xFF7AE0), (0xFFB070, 0x1A0A05), (0x9DFFC4, 0x39FF88), (0xFF3B5C, 0x4D8DFF), (0xFFE3A0, 0x3FE0C8), (0xF4E6FF, 0xA64DFF), (0xFFF3B0, 0xFF9A1C)];
struct Mon {
    id: u64,
    x: f32,
    y: f32,
    hp: f64,
    max: f64,
    big: bool,
    fl: f32,
    ph: f32,
    sp: f32,
}
struct Shot {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    tg: u64,
    sk: u8,
    alt: bool,
    tr: Vec<(f32, f32)>,
    v: f32,
}

/// Succès : (nom, statistique suivie, objectif, unité). Chaque succès donne +2 % à tout.
static ACH: [(&str, u8, f64, &str); 22] = [
    ("Doigts agiles", 0, 100.0, "clics"),
    ("Cliqueur fou", 0, 1e4, "clics"),
    ("Machine à conquérir", 0, 1e6, "clics"),
    ("Petit seigneur", 1, 1e6, "territoires"),
    ("Grand conquérant", 1, 1e12, "territoires"),
    ("Maître des mondes", 1, 1e30, "territoires"),
    ("Acheteur compulsif", 2, 50.0, "achats"),
    ("Bâtisseur d’empire", 2, 500.0, "achats"),
    ("Apprenti mage", 3, 10.0, "sorts"),
    ("Archimage confirmé", 3, 100.0, "sorts"),
    ("Premier sang", 4, 1.0, "boss"),
    ("Chasseur de boss", 4, 10.0, "boss"),
    ("Fléau des monstres", 4, 50.0, "boss"),
    ("Collectionneur", 5, 3.0, "châteaux"),
    ("Grand collectionneur", 5, 6.0, "châteaux"),
    ("Tous les conquis", 5, 10.0, "châteaux"),
    ("Big Bang !", 6, 1.0, "Big Bang"),
    ("Cycle cosmique", 6, 5.0, "Big Bang"),
    ("Habitué", 7, 3.0, "quêtes"),
    ("Aventurier du quotidien", 7, 30.0, "quêtes"),
    ("Pouvoir éveillé", 8, 1.0, "capacités"),
    ("Maître des capacités", 8, 10.0, "capacités"),
];

/// Quêtes du jour : (statistique, objectif, texte). Statistiques : 0 clics, 1 sorts, 2 achats, 3 boss, 4 capacités.
static QP: [(usize, u64, &str); 5] = [
    (0, 400, "Fais 400 clics"),
    (1, 3, "Lance 3 sorts"),
    (2, 30, "Fais 30 achats"),
    (3, 1, "Tue 1 boss"),
    (4, 1, "Utilise 1 capacité de château"),
];

/// Talents : (nom, description, niveau max, coût de base). Les points viennent des Big Bang (+3) et des coffres (+1).
static TAL: [(&str, &str, u32, u32); 6] = [
    ("Poigne de fer", "Clics ×1,25 par niveau", 5, 1),
    ("Économie de guerre", "Production ×1,25 par niveau", 5, 1),
    ("Maître du mana", "Mana max +50 et regain +20 % par niveau", 3, 2),
    ("Chasseur de boss", "Boss −10 % de PV et butin +50 % par niveau", 4, 1),
    ("Horloger", "Recharges des capacités −10 % par niveau", 3, 2),
    ("Fortune", "Pluie d’or et Avancement temporel ×1,5 par niveau", 3, 2),
];

/// Combos de capacités : (nom, capacité A, capacité B, multiplicateur). 0 Frénésie, 1 Bénédiction, 2 Clone, 3 Ascension, 4 Vide, 5 Extension.
static CMB: [(&str, u8, u8, f64); 5] = [
    ("Singularité", 4, 0, 5.0),
    ("Horizon d’événements", 4, 1, 3.0),
    ("Territoire parfait", 5, 0, 10.0),
    ("Domaine béni", 5, 1, 5.0),
    ("Surcharge arcanique", 2, 3, 4.0),
];

#[derive(Clone)]
struct Boss {
    hp: u32,
    max: u32,
    skin: u8, // château au moment de l'apparition : chaque château a son boss
    lvl: u32,
    side: u8,
    fl: f32,
    kb: f32,
    age: f32,
    test: bool, // boss de test (code TESTEBOSS) : aucun butin, ne compte pour rien
}

/// Petit « pinceau » pour dessiner les boss avec des cercles : coordonnées relatives au centre, mises à l'échelle par k.
struct BP {
    cx: f32,
    cy: f32,
    k: f32,
    al: f32,
}

impl BP {
    fn c(&self, g: &mut Gfx, x: f32, y: f32, r: f32, col: u32, a: f32) {
        g.circle(self.cx + x * self.k, self.cy + y * self.k, (r * self.k).max(0.5), rgba(col, (a * self.al).clamp(0.0, 1.0)));
    }
    /// Ellipse (cercles empilés le long du grand axe).
    fn o(&self, g: &mut Gfx, x: f32, y: f32, rx: f32, ry: f32, col: u32, a: f32) {
        let (big, small, horiz) = if rx >= ry { (rx, ry, true) } else { (ry, rx, false) };
        let n = (((big - small) / (small * 0.4).max(1.0)).ceil() as i32).max(1);
        for i in 0..=n {
            let off = (i as f32 / n as f32 * 2.0 - 1.0) * (big - small);
            if horiz {
                self.c(g, x + off, y, small, col, a);
            } else {
                self.c(g, x, y + off, small, col, a);
            }
        }
    }
    fn l(&self, g: &mut Gfx, x0: f32, y0: f32, x1: f32, y1: f32, w: f32, col: u32, a: f32) {
        let (ax, ay, bx, by) = (self.cx + x0 * self.k, self.cy + y0 * self.k, self.cx + x1 * self.k, self.cy + y1 * self.k);
        let rad = (w * self.k * 0.7).max(0.9);
        let n = (((bx - ax).powi(2) + (by - ay).powi(2)).sqrt() / (rad * 1.1)).ceil().max(1.0) as u32;
        for i in 0..=n {
            let f = i as f32 / n as f32;
            g.circle(ax + (bx - ax) * f, ay + (by - ay) * f, rad, rgba(col, (a * self.al).clamp(0.0, 1.0)));
        }
    }
    /// Pointe effilée (corne, griffe, cristal, flamme, tentacule).
    fn s(&self, g: &mut Gfx, x0: f32, y0: f32, x1: f32, y1: f32, r0: f32, col: u32, a: f32) {
        for i in 0..=10u32 {
            let f = i as f32 / 10.0;
            self.c(g, x0 + (x1 - x0) * f, y0 + (y1 - y0) * f, r0 * (1.0 - f) + 0.6, col, a);
        }
    }
    fn r(&self, g: &mut Gfx, x: f32, y: f32, w: f32, h: f32, rad: f32, col: u32, a: f32) {
        g.rrect(self.cx + (x - w / 2.0) * self.k, self.cy + (y - h / 2.0) * self.k, w * self.k, h * self.k, rad * self.k, rgba(col, (a * self.al).clamp(0.0, 1.0)));
    }
    /// Œil qui brille.
    fn e(&self, g: &mut Gfx, x: f32, y: f32, r: f32, col: u32) {
        self.c(g, x, y, r * 3.0, col, 0.22);
        self.c(g, x, y, r * 1.8, col, 0.4);
        self.c(g, x, y, r, col, 1.0);
        self.c(g, x, y, r * 0.4, 0xFFFFFF, 1.0);
    }
}

struct PRow {
    title: String,
    detail: String,
    right: String,
    frac: f32,
    done: bool,
    act: u8,
    idx: usize,
}

fn asset_path(rel: &str) -> std::path::PathBuf {
    let base = std::env::var("LAPP_APP").map(std::path::PathBuf::from).unwrap_or_else(|_| {
        if std::path::Path::new("/app").is_dir() {
            std::path::PathBuf::from("/app")
        } else {
            std::env::current_dir().unwrap_or_default()
        }
    });
    base.join(rel)
}

fn fmt_t(secs: f64) -> String {
    let s = secs.max(0.0).ceil() as u64;
    format!("{}:{:02}", s / 60, s % 60)
}

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

/// Comme fmt mais sans zéros inutiles : 10.00M -> 10M
fn fmt_short(n: f64) -> String {
    let s = fmt(n);
    let split = s.find(|c: char| c.is_alphabetic()).unwrap_or(s.len());
    let (num, suf) = s.split_at(split);
    if num.contains('.') {
        format!("{}{}", num.trim_end_matches('0').trim_end_matches('.'), suf)
    } else {
        s
    }
}

/// Éclair en zigzag de (x0, y0) à (x1, y1), dessiné avec des points (le SDK n'a pas de lignes).
fn draw_bolt(g: &mut Gfx, x0: f32, y0: f32, x1: f32, y1: f32, seed: u32, segs: u32, jit: f32, col: u32, al: f32, w: f32) {
    let (dx, dy) = (x1 - x0, y1 - y0);
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let (nx, ny) = (-dy / len, dx / len);
    let mut pts = vec![(x0, y0)];
    for i in 1..segs {
        let f = i as f32 / segs as f32;
        let o = (hash01(seed.wrapping_add(i), 77) - 0.5) * jit;
        pts.push((x0 + dx * f + nx * o, y0 + dy * f + ny * o));
    }
    pts.push((x1, y1));
    let w = w.max(0.6);
    for p in pts.windows(2) {
        let ((ax, ay), (bx, by)) = (p[0], p[1]);
        let l = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
        let n = (l / (w * 0.9)).ceil().max(1.0) as u32;
        for k in 0..=n {
            let f = k as f32 / n as f32;
            let (x, y) = (ax + (bx - ax) * f, ay + (by - ay) * f);
            g.circle(x, y, w * 2.4, rgba(col, al * 0.15));
            g.circle(x, y, w, rgba(col, al));
        }
    }
}

fn ease_out_back(p: f32) -> f32 {
    let c1 = 1.70158;
    1.0 + (c1 + 1.0) * (p - 1.0).powi(3) + c1 * (p - 1.0).powi(2)
}

fn ez(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn fx_line(g: &mut Gfx, x0: f32, y0: f32, x1: f32, y1: f32, w: f32, col: u32, al: f32) {
    if al <= 0.0 {
        return;
    }
    let rad = (w * 0.5).max(0.8);
    let n = (((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt() / (rad * 1.5)).ceil().clamp(1.0, 160.0) as u32;
    for i in 0..=n {
        let f = i as f32 / n as f32;
        g.circle(x0 + (x1 - x0) * f, y0 + (y1 - y0) * f, rad, rgba(col, al.clamp(0.0, 1.0)));
    }
}

fn fx_ring(g: &mut Gfx, cx: f32, cy: f32, r: f32, n: u32, sz: f32, col: u32, al: f32) {
    if al <= 0.0 || r <= 0.5 {
        return;
    }
    for i in 0..n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        g.circle(cx + a.cos() * r, cy + a.sin() * r, sz.max(0.6), rgba(col, al.clamp(0.0, 1.0)));
    }
}

fn fx_glow(g: &mut Gfx, x: f32, y: f32, r: f32, col: u32, al: f32) {
    if r <= 0.5 || al <= 0.0 {
        return;
    }
    for (f, a) in [(1.0f32, 0.18f32), (0.7, 0.3), (0.45, 0.5), (0.2, 0.8)] {
        g.circle(x, y, r * f, rgba(col, (al * a).clamp(0.0, 1.0)));
    }
}

fn fx_taper(g: &mut Gfx, x0: f32, y0: f32, x1: f32, y1: f32, r0: f32, col: u32, al: f32) {
    for i in 0..=10u32 {
        let f = i as f32 / 10.0;
        g.circle(x0 + (x1 - x0) * f, y0 + (y1 - y0) * f, (r0 * (1.0 - f) + 0.6).max(0.5), rgba(col, al.clamp(0.0, 1.0)));
    }
}

fn hsv_hex(h: f32, s: f32, v: f32) -> u32 {
    let h = (h.rem_euclid(360.0)) / 60.0;
    let c = v * s;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    (((r + m) * 255.0) as u32) << 16 | (((g + m) * 255.0) as u32) << 8 | ((b + m) * 255.0) as u32
}

/// Sprite de souris d'ordinateur (flèche blanche bordée de noir) dont la pointe est au centre
/// de l'image et regarde dans la direction `k` (0..16, par pas de 22,5°).
fn make_cursor(k: u32, size: u32, s: f32, fill_col: u32, border_col: u32) -> Pixmap {
    use std::f32::consts::{PI, TAU};
    let mut pm = Pixmap::new(size, size).unwrap();
    let c = size as f32 / 2.0;
    let rot = k as f32 * TAU / 16.0 + 3.0 * PI / 4.0;
    let (cr, sr) = (rot.cos(), rot.sin());
    let base = [(0.0f32, 0.0f32), (0.0, 17.0), (4.2, 13.4), (7.2, 20.0), (10.0, 18.6), (7.1, 12.2), (12.0, 12.0)];
    let pts: Vec<(f32, f32)> = base.iter().map(|(x, y)| (c + x * s * cr - y * s * sr, c + x * s * sr + y * s * cr)).collect();
    let n = pts.len();
    let inside = |px: f32, py: f32| -> bool {
        let mut ins = false;
        let mut j = n - 1;
        for i in 0..n {
            let (xi, yi) = pts[i];
            let (xj, yj) = pts[j];
            if (yi > py) != (yj > py) && px < (xj - xi) * (py - yi) / (yj - yi) + xi {
                ins = !ins;
            }
            j = i;
        }
        ins
    };
    let dist = |px: f32, py: f32| -> f32 {
        let mut best = f32::MAX;
        for i in 0..n {
            let (x1, y1) = pts[i];
            let (x2, y2) = pts[(i + 1) % n];
            let (dx, dy) = (x2 - x1, y2 - y1);
            let t = (((px - x1) * dx + (py - y1) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
            let (qx, qy) = (x1 + t * dx, y1 + t * dy);
            best = best.min(((px - qx).powi(2) + (py - qy).powi(2)).sqrt());
        }
        best
    };
    let border = 1.2 * s.max(1.0);
    let data = pm.data_mut();
    for y in 0..size {
        for x in 0..size {
            let (mut fill, mut ol) = (0.0f32, 0.0f32);
            for sy in 0..3 {
                for sx in 0..3 {
                    let px = x as f32 + (sx as f32 + 0.5) / 3.0;
                    let py = y as f32 + (sy as f32 + 0.5) / 3.0;
                    if inside(px, py) {
                        fill += 1.0;
                        ol += 1.0;
                    } else if dist(px, py) <= border {
                        ol += 1.0;
                    }
                }
            }
            if ol > 0.0 {
                let (fill, ol) = (fill / 9.0, ol / 9.0);
                let t = fill / ol;
                let ch = |sh: u32| {
                    let b = ((border_col >> sh) & 0xFF) as f32;
                    let f = ((fill_col >> sh) & 0xFF) as f32;
                    b + (f - b) * t
                };
                let idx = ((y * size + x) * 4) as usize;
                data[idx] = (ch(16) * ol) as u8;
                data[idx + 1] = (ch(8) * ol) as u8;
                data[idx + 2] = (ch(0) * ol) as u8;
                data[idx + 3] = (ol * 255.0) as u8;
            }
        }
    }
    pm
}

fn hash01(i: u32, k: u32) -> f32 {
    let h = ((i as f32 + 1.0) * 12.9898 + k as f32 * 78.233).sin() * 43758.547;
    h - h.floor()
}

fn mix_color(a: u32, b: u32, f: f32) -> u32 {
    let ch = |s: u32| -> u32 {
        let x = ((a >> s) & 0xFF) as f32;
        let y = ((b >> s) & 0xFF) as f32;
        (x + (y - x) * f).round() as u32
    };
    (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

/// Couleur d'une étoile de la galaxie selon son éloignement du centre (0 = cœur, 1 = bord).
fn galaxy_color(u: f32) -> u32 {
    let stops: [(f32, u32); 4] = [(0.0, 0xFFF1C9), (0.22, 0xFF7AD9), (0.55, 0xA566FF), (1.0, 0x5C8BFF)];
    for w in stops.windows(2) {
        if u <= w[1].0 {
            let f = (u - w[0].0) / (w[1].0 - w[0].0);
            return mix_color(w[0].1, w[1].1, f.clamp(0.0, 1.0));
        }
    }
    stops[3].1
}

fn fmt_mult(n: f64) -> String {
    if n < 1000.0 {
        format!("{}", (n * 100.0).round() / 100.0)
    } else {
        fmt(n)
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
    Tune3,
    Tune4,
    Intro,
    Settings,
    Bench,
    BigBang,
    DomainFx,
    VoidFx,
    TimeFx,
    Skins,
    Progress,
    OrbFx,
    BossIntro,
    EyeFx,
    SunFx,
    SolarFx,
    Video,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Shop,
    Magic,
    Castles,
    Cursors,
    Defense,
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    l: f32,
    m: f32,
    s: f32,
    col: u32,
    kind: u8, // 0 rond, 1 étincelle, 2 éclat, 3 trait
    rot: f32,
    vr: f32,
    g: f32,
    sw: f32,
    tx: f32,
    ty: f32,
    kill: f32, // > 0 : particule aspirée vers (tx, ty) (trou noir)
    acc: f32,
}

struct Ring {
    ticks: bool,
    x: f32,
    y: f32,
    r0: f32,
    r1: f32,
    l: f32,
    m: f32,
    col: u32, // 0 = arc-en-ciel
    w: f32,
}

struct FxCfg {
    n: u32,
    kinds: &'static [u8],
    sp: (f32, f32),
    life: (f32, f32),
    g: f32,
    sz: (f32, f32),
    swirl: f32,
    suck: bool,
}

/// Animation de clic par château : pierre, or, diamant, netherite, galaxie, galaxie MAX, trou noir (aspiration), alien.
static FX: [FxCfg; 10] = [
    FxCfg { n: 8, kinds: &[0], sp: (50.0, 120.0), life: (0.3, 0.55), g: 240.0, sz: (2.0, 4.0), swirl: 0.0, suck: false },
    FxCfg { n: 14, kinds: &[0, 1], sp: (70.0, 160.0), life: (0.4, 0.7), g: 300.0, sz: (2.0, 4.0), swirl: 0.0, suck: false },
    FxCfg { n: 18, kinds: &[2, 2, 1], sp: (90.0, 200.0), life: (0.4, 0.8), g: 80.0, sz: (3.0, 6.0), swirl: 0.0, suck: false },
    FxCfg { n: 24, kinds: &[3, 3, 0], sp: (80.0, 200.0), life: (0.5, 0.9), g: -130.0, sz: (1.5, 3.5), swirl: 0.0, suck: false },
    FxCfg { n: 32, kinds: &[1, 0, 0], sp: (60.0, 180.0), life: (0.6, 1.1), g: 0.0, sz: (2.0, 5.0), swirl: 3.0, suck: false },
    FxCfg { n: 40, kinds: &[1, 1, 3], sp: (80.0, 240.0), life: (0.7, 1.2), g: 0.0, sz: (2.0, 4.5), swirl: 3.5, suck: false },
    FxCfg { n: 30, kinds: &[3, 3, 0], sp: (40.0, 90.0), life: (0.7, 1.1), g: 0.0, sz: (1.5, 3.5), swirl: 0.0, suck: true },
    FxCfg { n: 30, kinds: &[4, 4, 0], sp: (60.0, 170.0), life: (0.6, 1.1), g: -160.0, sz: (2.0, 4.5), swirl: 0.0, suck: false },
    FxCfg { n: 36, kinds: &[3, 3, 1], sp: (120.0, 300.0), life: (0.45, 0.9), g: 0.0, sz: (1.5, 3.5), swirl: 0.0, suck: false },
    FxCfg { n: 30, kinds: &[5, 5, 3, 0, 0], sp: (70.0, 190.0), life: (0.5, 1.0), g: 70.0, sz: (2.0, 4.5), swirl: 0.0, suck: false },
];

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
    Space,
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
    cs: Vec<bool>,
    anim_t: f32,
    cur_n: u32,
    cur_spd: u32,
    auto_acc: f64,
    cur_t: f64,
    parts: Vec<Particle>,
    rings: Vec<Ring>,
    shake: f32,
    rng: u32,
    demo_burst: bool,
    bb: u32,
    bb_confirm: f32,
    bench_phase: u8,
    bench_t: f32,
    bench_work: f64,
    bench_ops: u64,
    bench_x: f64,
    bench_frames: u32,
    bench_cpu: f64,
    bench_fps: f32,
    fps_cycle: f32,
    bbfx_t: f32,
    clone_left: f32,
    asc_left: f32,
    beam_l: f32,
    beam_x: f32,
    beam_y: f32,
    bolt_l: f32,
    bolt_seed: u32,
    orb_pulse: f32,
    intro_t: f32,
    code3_ok: String,
    own: Vec<bool>,
    temps_cd: f64,
    time_t: f32,
    time_gain: f64,
    skin_scroll: Scroller,
    st: [u64; 6],
    ach: Vec<bool>,
    lt_max: f64,
    tp: u32,
    tl: [u32; 6],
    q_day: i64,
    q: Vec<(usize, u64, bool)>,
    chest: u8,
    boss: Option<Boss>,
    boss_next: f64,
    orb_cd: f64,
    orb_t: f32,
    orb_target: (f32, f32),
    music_on: bool,
    boss_dying: Option<(Boss, f32)>,
    boss_ban: f32,
    boss_intro_t: f32,
    music: Option<std::process::Child>,
    music_fail: u8,
    last_combo: String,
    slow_t: f32,
    prog_tab: u8,
    prog_scroll: Scroller,
    ext_until: f64,
    inf_dom: bool,
    eye_t: f32,
    sun_until: f64,
    sun_cd: f64,
    sun_t: f32,
    solar_t: f32,
    def_g: u32,
    def_pw: u32,
    def_sp: u32,
    def_wl: u32,
    def_wave: u32,
    def_next: f64,
    def_kills: u64,
    mons: Vec<Mon>,
    mon_q: Vec<(f64, f64, bool)>,
    shots: Vec<Shot>,
    poofs: Vec<(f32, f32, f32, bool, bool)>,
    gcd: Vec<f32>,
    gfire: Vec<f32>,
    mon_id: u64,
    ext_cd: f64,
    vide_until: f64,
    vide_cd: f64,
    clock: f64,
    dom_t: f32,
    void_t: f32,
    verify: Option<std::sync::mpsc::Receiver<(u8, bool, String)>>,
    fps_on: bool,
    fps_frames: u32,
    fps_t: Instant,
    fps_val: f32,
    last_grade: usize,
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
            cs: vec![false; CASTLES.len()],
            anim_t: 0.0,
            cur_n: 0,
            cur_spd: 0,
            auto_acc: 0.0,
            cur_t: 0.0,
            parts: Vec::new(),
            rings: Vec::new(),
            shake: 0.0,
            rng: 0x2545F491,
            demo_burst: false,
            bb: 0,
            bb_confirm: 0.0,
            bench_phase: 0,
            bench_t: 0.0,
            bench_work: 0.0,
            bench_ops: 0,
            bench_x: 0.5,
            bench_frames: 0,
            bench_cpu: 0.0,
            bench_fps: 0.0,
            fps_cycle: 0.0,
            bbfx_t: 0.0,
            clone_left: 0.0,
            asc_left: 0.0,
            beam_l: 0.0,
            beam_x: 0.0,
            beam_y: 0.0,
            bolt_l: 0.0,
            bolt_seed: 0,
            orb_pulse: 0.0,
            intro_t: 0.0,
            code3_ok: String::new(),
            own: { let mut v = vec![false; 12]; v[0] = true; v },
            temps_cd: 0.0,
            time_t: 0.0,
            time_gain: 0.0,
            skin_scroll: Scroller::new(),
            st: [0; 6],
            ach: vec![false; ACH.len()],
            lt_max: 0.0,
            tp: 0,
            tl: [0; 6],
            q_day: -1,
            q: Vec::new(),
            chest: 0,
            boss: None,
            boss_next: 0.0,
            orb_cd: 0.0,
            orb_t: 0.0,
            orb_target: (0.0, 0.0),
            music_on: true,
            boss_dying: None,
            boss_ban: 0.0,
            boss_intro_t: 0.0,
            music: None,
            music_fail: 0,
            last_combo: String::new(),
            slow_t: 0.0,
            prog_tab: 0,
            prog_scroll: Scroller::new(),
            ext_until: 0.0,
            inf_dom: false,
            eye_t: 0.0,
            sun_until: 0.0,
            sun_cd: 0.0,
            sun_t: 0.0,
            solar_t: 0.0,
            def_g: 0,
            def_pw: 0,
            def_sp: 0,
            def_wl: 0,
            def_wave: 0,
            def_next: 0.0,
            def_kills: 0,
            mons: Vec::new(),
            mon_q: Vec::new(),
            shots: Vec::new(),
            poofs: Vec::new(),
            gcd: Vec::new(),
            gfire: Vec::new(),
            mon_id: 0,
            ext_cd: 0.0,
            vide_until: 0.0,
            vide_cd: 0.0,
            clock: now_secs(),
            dom_t: 0.0,
            void_t: 0.0,
            verify: None,
            fps_on: false,
            fps_frames: 0,
            fps_t: Instant::now(),
            fps_val: 0.0,
            last_grade: 0,
        };
        app.load();
        app.last_grade = app.grade_idx();
        if app.boss_next <= 0.0 {
            app.boss_next = now_secs() + 600.0;
        }
        if app.q.is_empty() {
            app.gen_quests();
        }
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
        self.skin = next().parse::<u8>().unwrap_or(0).min(11);
        self.mana = next().parse().unwrap_or(0.0);
        for (i, c) in next().chars().enumerate() {
            if i < self.mg.len() {
                self.mg[i] = c == '1';
            }
        }
        self.fps_on = next() == "1";
        for (i, c) in next().chars().enumerate() {
            if i < self.cs.len() {
                self.cs[i] = c == '1';
            }
        }
        self.bb = next().parse().unwrap_or(0);
        self.cur_n = next().parse().unwrap_or(0);
        self.cur_spd = next().parse().unwrap_or(0);
        self.ext_until = next().parse().unwrap_or(0.0);
        self.ext_cd = next().parse().unwrap_or(0.0);
        self.vide_until = next().parse().unwrap_or(0.0);
        self.vide_cd = next().parse().unwrap_or(0.0);
        self.temps_cd = next().parse().unwrap_or(0.0);
        let ownbits = next().to_string();
        if ownbits.is_empty() {
            // ancienne sauvegarde : tous les châteaux jusqu'au château actuel sont considérés comme débloqués
            for i in 0..=(self.skin as usize).min(11) {
                self.own[i] = true;
            }
        } else {
            for (i, c) in ownbits.chars().enumerate() {
                if i < self.own.len() {
                    self.own[i] = c == '1';
                }
            }
        }
        self.own[0] = true;
        self.own[self.skin as usize] = true;
        for (i, n) in next().split(',').enumerate() {
            if i < 6 {
                self.st[i] = n.parse().unwrap_or(0);
            }
        }
        for (i, c) in next().chars().enumerate() {
            if i < self.ach.len() {
                self.ach[i] = c == '1';
            }
        }
        self.lt_max = next().parse().unwrap_or(0.0);
        self.tp = next().parse().unwrap_or(0);
        for (i, n) in next().split(',').enumerate() {
            if i < 6 {
                self.tl[i] = n.parse().unwrap_or(0);
            }
        }
        self.q_day = next().parse().unwrap_or(-1);
        self.q.clear();
        for part in next().split(',') {
            let f: Vec<&str> = part.split(':').collect();
            if f.len() == 3 {
                if let (Ok(a), Ok(b)) = (f[0].parse::<usize>(), f[1].parse::<u64>()) {
                    if a < QP.len() {
                        self.q.push((a, b, f[2] == "1"));
                    }
                }
            }
        }
        self.chest = next().parse().unwrap_or(0);
        let bs = next().to_string();
        let bf: Vec<&str> = bs.split(':').collect();
        if bf.len() == 5 || bf.len() == 6 {
            if let (Ok(hp), Ok(max), Ok(skin), Ok(lvl), Ok(side)) = (bf[0].parse::<u32>(), bf[1].parse::<u32>(), bf[2].parse::<u8>(), bf[3].parse::<u32>(), bf[4].parse::<u8>()) {
                self.boss = Some(Boss { hp, max, skin: skin.min(11), lvl, side: side % 2, fl: 0.0, kb: 0.0, age: 5.0, test: bf.get(5) == Some(&"1") });
            }
        }
        self.boss_next = next().parse().unwrap_or(0.0);
        self.orb_cd = next().parse().unwrap_or(0.0);
        self.music_on = next() != "0";
        self.inf_dom = next() == "1";
        let dfs: Vec<String> = next().split(':').map(|x| x.to_string()).collect();
        if dfs.len() >= 9 {
            self.def_g = dfs[0].parse().unwrap_or(0);
            self.def_pw = dfs[1].parse().unwrap_or(0);
            self.def_sp = dfs[2].parse().unwrap_or(0);
            self.def_wl = dfs[3].parse().unwrap_or(0);
            self.def_wave = dfs[4].parse().unwrap_or(0);
            self.def_next = dfs[5].parse().unwrap_or(0.0);
            self.def_kills = dfs[6].parse().unwrap_or(0);
            self.sun_until = dfs[7].parse().unwrap_or(0.0);
            self.sun_cd = dfs[8].parse().unwrap_or(0.0);
        }
        let away = (now_secs() - saved_at).clamp(0.0, OFFLINE_CAP);
        // gain hors-ligne : la capacité compte seulement pendant la part de l'absence où elle était active
        let act = (self.ext_until.max(self.vide_until).max(self.sun_until) - saved_at).clamp(0.0, away);
        self.clock = saved_at + 1.0;
        let ra = self.rate();
        self.clock = 1e18;
        let rn = self.rate();
        self.clock = now_secs();
        let gain = ra * act + rn * (away - act);
        if gain >= 1.0 && away > 30.0 {
            self.pts += gain;
            self.total += gain;
            self.pending_toast = Some(format!("Hors-ligne : +{} territoires", fmt(gain)));
        }
    }

    fn save(&mut self) {
        let owned = self.owned.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",");
        let mg: String = self.mg.iter().map(|b| if *b { '1' } else { '0' }).collect();
        let cs: String = self.cs.iter().map(|b| if *b { '1' } else { '0' }).collect();
        let s = format!(
            "{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{}",
            self.pts, self.total, self.click_lvl, self.clicks, now_secs(), owned, self.skin, self.mana, mg, if self.fps_on { 1 } else { 0 }, cs, self.bb, self.cur_n, self.cur_spd, self.ext_until, self.ext_cd, self.vide_until, self.vide_cd, self.temps_cd, self.own.iter().map(|b| if *b { '1' } else { '0' }).collect::<String>()
        );
        let extra = format!(
            "{};{};{};{};{};{};{};{};{};{};{};{};{};{}",
            self.st.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(","),
            self.ach.iter().map(|b| if *b { '1' } else { '0' }).collect::<String>(),
            self.lt_max,
            self.tp,
            self.tl.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(","),
            self.q_day,
            self.q.iter().map(|q| format!("{}:{}:{}", q.0, q.1, if q.2 { 1 } else { 0 })).collect::<Vec<_>>().join(","),
            self.chest,
            self.boss.as_ref().map_or(String::new(), |b| format!("{}:{}:{}:{}:{}:{}", b.hp, b.max, b.skin, b.lvl, b.side, if b.test { 1 } else { 0 })),
            self.boss_next,
            self.orb_cd,
            if self.music_on { 1 } else { 0 },
            if self.inf_dom { 1 } else { 0 },
            format!("{}:{}:{}:{}:{}:{}:{}:{}:{}", self.def_g, self.def_pw, self.def_sp, self.def_wl, self.def_wave, self.def_next, self.def_kills, self.sun_until, self.sun_cd)
        );
        let _ = self.store.set("state", format!("{};{}", s, extra));
    }

    // ---- règles du jeu ------------------------------------------------------------------
    fn has(&self, id: &str) -> bool {
        MAGIC.iter().position(|m| m.id == id).map_or(false, |i| self.mg[i])
    }
    /// Multiplicateur du château : pierre ×1, or ×4, diamant ×1000, netherite ×10000.
    fn skin_mult(&self) -> f64 {
        let base = match self.skin {
            11 => 1e10,
            10 => 1e13,
            9 => 1_000_000_000.0,
            8 => 100_000_000_000.0,
            7 => 100_000_000.0,
            6 => 10_000_000.0,
            5 => 1_000_000.0,
            4 => 100_000.0,
            3 => 10_000.0,
            2 => 1000.0,
            1 => 4.0,
            _ => 1.0,
        };
        let base = if self.sun_on() { base * 1e6 } else { base };
        if self.skin == 6 && self.clock < self.vide_until {
            base * 100.0
        } else {
            base
        }
    }
    /// Grade actuel d'après le total de territoires conquis.
    fn grade_idx(&self) -> usize {
        GRADES.iter().rposition(|g| self.total >= g.1).unwrap_or(0)
    }
    /// Multiplicateur des châteaux achetés (cumulatif).
    fn castle_mult(&self) -> f64 {
        CASTLES.iter().zip(&self.cs).filter(|(_, own)| **own).map(|(c, _)| c.2).product()
    }
    /// Multiplicateur total permanent : château (code) × grade × châteaux achetés.
    fn perm_mult(&self) -> f64 {
        self.skin_mult()
            * GRADES[self.grade_idx()].2
            * self.castle_mult()
            * self.bb_mult()
            * if self.asc_left > 0.0 { 10.0 } else { 1.0 }
            * if self.has("archimage") { 3.0 } else { 1.0 }
            * if self.ext_on() { if self.skin == 10 { 1e10 } else { 1e9 } } else { 1.0 }
            * self.ach_mult()
            * self.combo_mult()
    }
    /// Chaque Big Bang double tous les multiplicateurs.
    fn bb_mult(&self) -> f64 {
        2f64.powi(self.bb as i32)
    }
    /// Coût du Big Bang : 10 × le prix de la première Dimension 52, ×10 à chaque Big Bang.
    fn bb_cost(&self) -> f64 {
        self.tiers.last().map_or(1e68, |t| t.base) * 10.0 * 10f64.powi(self.bb as i32)
    }
    fn prod_mult(&self) -> f64 {
        let mut m = self.perm_mult() * 1.25f64.powi(self.tl[1] as i32);
        if self.has("grimoire") {
            m *= 2.0;
        }
        if self.bless_left > 0.0 {
            m *= 3.0;
        }
        m
    }
    fn max_mana(&self) -> f32 {
        100.0 + 50.0 * self.tl[2] as f32 + if self.has("res1") { 50.0 } else { 0.0 } + if self.has("res2") { 100.0 } else { 0.0 } + if self.has("cristal") { 150.0 } else { 0.0 }
    }
    fn mana_regen(&self) -> f32 {
        (if self.has("source") { 2.0 } else { 1.0 }) * (1.0 + 0.2 * self.tl[2] as f32)
    }
    fn rate(&self) -> f64 {
        let base: f64 = self.tiers.iter().zip(&self.owned).map(|(t, n)| t.rate * *n as f64).sum();
        base * self.prod_mult()
    }
    fn click_power(&self) -> f64 {
        let mut p = 2f64.powi(self.click_lvl as i32) * self.perm_mult() * 1.25f64.powi(self.tl[0] as i32);
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
        let shop_top = (h * 0.54).round();
        let tab_h = 32.0 * s;
        let list_top = shop_top + tab_h + if self.tab != Tab::Shop { 26.0 * s } else { 0.0 };
        let zone_top = head_h + 84.0 * s;
        let zone_h = shop_top - zone_top;
        let r = ((zone_h / 2.0 - 16.0 * s) / 1.1).min(62.0 * s).max(20.0 * s);
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

    fn do_click(&mut self, ctx: &Ctx, at: Option<(f32, f32)>) {
        let l = self.layout(ctx);
        let s = ctx.scale();
        let gain = self.click_power();
        self.pts += gain;
        self.total += gain;
        self.clicks += 1;
        self.bump(ctx, 0, 1);
        self.pop = 1.0;
        let (bx, by) = at.unwrap_or((l.cx, l.cy - l.r * 0.4));
        self.burst(ctx, bx, by, 1.0);
        let off = ((self.clicks % 7) as f32 - 3.0) * 9.0 * s;
        if self.floats.len() < 10 {
            self.floats.push(Floater { x: l.cx + off, y: l.cy - l.r * 0.5, t: 0.0, txt: format!("+{}", fmt(gain)) });
        }
        ctx.animate();
    }

    fn buy(&mut self, ctx: &Ctx, row: usize) {
        if row == self.tiers.len() + 1 {
            self.big_bang(ctx);
            return;
        }
        if row == 0 {
            let c = self.click_cost();
            if self.pts >= c {
                self.pts -= c;
                self.click_lvl += 1;
                self.bump(ctx, 2, 1);
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
                self.bump(ctx, 2, 1);
                self.save();
            } else {
                ctx.toast("Pas assez de territoires");
            }
        }
    }

    fn big_bang(&mut self, ctx: &Ctx) {
        let c = self.bb_cost();
        if self.pts < c {
            ctx.toast(format!("Il faut {} territoires pour le Big Bang", fmt(c)));
            return;
        }
        if self.bb_confirm <= 0.0 {
            self.bb_confirm = 6.0;
            ctx.toast("Le Big Bang remet TOUT à zéro (sauf le château des codes). Appuie encore pour confirmer");
            return;
        }
        self.bb_confirm = 0.0;
        self.pts = 0.0;
        self.total = 0.0;
        self.click_lvl = 0;
        self.clicks = 0;
        self.mana = 0.0;
        self.frenzy_left = 0.0;
        self.bless_left = 0.0;
        self.clone_left = 0.0;
        self.asc_left = 0.0;
        self.last_grade = 0;
        self.cur_n = 0;
        self.cur_spd = 0;
        self.auto_acc = 0.0;
        self.owned.iter_mut().for_each(|n| *n = 0);
        self.mg.iter_mut().for_each(|b| *b = false);
        self.cs.iter_mut().for_each(|b| *b = false);
        self.bb += 1;
        self.tp += 3;
        self.save();
        self.bbfx_t = 0.0;
        self.screen = Screen::BigBang;
    }

    fn start_bench(&mut self) {
        self.bench_phase = 0;
        self.bench_t = 0.0;
        self.bench_work = 0.0;
        self.bench_ops = 0;
        self.bench_x = 0.5;
        self.bench_frames = 0;
        self.screen = Screen::Bench;
    }

    fn cur_cps(&self) -> f64 {
        0.5 * 1.3f64.powi(self.cur_spd as i32) * if self.clone_left > 0.0 { 3.0 } else { 1.0 }
    }
    /// Cliqueurs personnalisés du château Éclipse : ×8 sur chacun de leurs clics.
    /// Mode Éclipse : château Éclipse ou Éclipse ultime.
    fn is_ec(&self) -> bool {
        self.skin == 8 || self.skin == 10
    }
    /// Extension du territoire active (pendant sa durée, ou pour toujours avec le code caché).
    fn ext_on(&self) -> bool {
        self.is_ec() && (self.inf_dom || self.clock < self.ext_until)
    }
    fn cursor_bonus(&self) -> f64 {
        if self.is_ec() {
            8.0
        } else {
            1.0
        }
    }
    fn cur_cost(&self) -> f64 {
        (100.0 * 1.5f64.powi(self.cur_n as i32)).ceil()
    }
    fn spd_cost(&self) -> f64 {
        (250.0 * 3f64.powi(self.cur_spd as i32)).ceil()
    }

    fn cursor_row(&self, k: usize) -> (String, String, String, bool, bool) {
        if k == 0 {
            let c = self.cur_cost();
            ("Cliqueur souris".into(), format!("+1 souris · {} clic/s · coût {}", fmt_mult(self.cur_cps()), fmt(c)), self.cur_n.to_string(), self.pts >= c, false)
        } else {
            let c = self.spd_cost();
            ("Vitesse de clic".into(), format!("×1.3 de vitesse · coût {}", fmt(c)), format!("Niv {}", self.cur_spd), self.pts >= c, false)
        }
    }

    fn cursor_tap(&mut self, ctx: &Ctx, k: usize) {
        let c = match k {
            0 => self.cur_cost(),
            1 => self.spd_cost(),
            _ => return,
        };
        if self.pts < c {
            ctx.toast("Pas assez de territoires");
            return;
        }
        self.pts -= c;
        if k == 0 {
            self.cur_n += 1;
        } else {
            self.cur_spd += 1;
        }
        self.bump(ctx, 2, 1);
        self.save();
    }

    /// (x, y, angle vers le château) du cliqueur i parmi m, avec son petit « coup » vers le château.
    fn cursor_pos(&self, i: u32, m: u32, l: &Layout) -> (f32, f32, f32) {
        use std::f32::consts::PI;
        let ang = self.cur_t as f32 * 0.7 + i as f32 * std::f32::consts::TAU / m as f32;
        let vc = self.cur_cps().min(3.0);
        let ph = (self.cur_t * vc + i as f64 / m as f64).rem_euclid(1.0) as f32;
        let lunge = if ph < 0.25 { -(ph / 0.25 * PI).sin() * l.r * 0.2 } else { 0.0 };
        let ro = l.r * 1.12 + lunge;
        let (x, y) = (l.cx + ang.cos() * ro, l.cy + ang.sin() * ro);
        (x, y, (l.cy - y).atan2(l.cx - x))
    }

    fn rnd(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x as f32 / u32::MAX as f32
    }

    /// Gerbe de particules + anneaux, dont le style dépend du château.
    fn burst(&mut self, ctx: &Ctx, x: f32, y: f32, sc: f32) {
        if self.parts.len() > 260 {
            return;
        }
        let s = ctx.scale();
        let l = self.layout(ctx);
        let cfg = &FX[if self.skin == 10 { 8 } else if self.skin == 11 { 3 } else { (self.skin as usize).min(9) }];
        let n = (cfg.n as f32 * sc).ceil() as u32;
        for _ in 0..n {
            let a = self.rnd() * std::f32::consts::TAU;
            let sp = (cfg.sp.0 + self.rnd() * (cfg.sp.1 - cfg.sp.0)) * s;
            let life = cfg.life.0 + self.rnd() * (cfg.life.1 - cfg.life.0);
            let size = (cfg.sz.0 + self.rnd() * (cfg.sz.1 - cfg.sz.0)) * s;
            let kind = cfg.kinds[(self.rnd() * cfg.kinds.len() as f32) as usize % cfg.kinds.len()];
            let col = self.fx_color();
            let rot = self.rnd() * std::f32::consts::TAU;
            let vr = (self.rnd() - 0.5) * 12.0;
            // trou noir : les particules partent d'un cercle autour du château et sont aspirées en spirale
            let (px, py, vx, vy, kill) = if cfg.suck {
                let d = l.r * (1.4 + self.rnd() * 0.8);
                (l.cx + a.cos() * d, l.cy + a.sin() * d, -a.sin() * sp * 1.6, a.cos() * sp * 1.6, l.r * 0.32)
            } else {
                (x, y, a.cos() * sp, a.sin() * sp, 0.0)
            };
            self.parts.push(Particle { x: px, y: py, vx, vy, l: life, m: life, s: size, col, kind, rot, vr, g: cfg.g * s, sw: cfg.swirl, tx: l.cx, ty: l.cy, kill, acc: 520.0 * s });
        }
        if sc >= 1.0 {
            // (couleur, épaisseur, rayon final, durée, rayon de départ) en rayons du château
            let rings: &[(u32, f32, f32, f32, f32)] = match self.skin {
                0 => &[],
                1 => &[(0xF2C94C, 2.0, 1.5, 0.35, 0.95)],
                2 => &[(0x8FF0FF, 2.5, 1.7, 0.4, 0.95)],
                3 => &[(0x9B5AFF, 3.0, 1.6, 0.42, 0.95), (0xFF8A3D, 2.0, 1.3, 0.3, 0.95)],
                4 => &[(0xC77DFF, 3.0, 1.8, 0.5, 0.95), (0xFF7AE0, 2.0, 1.4, 0.4, 0.95)],
                5 => &[(0xFFD24D, 4.0, 2.0, 0.55, 0.95), (0, 3.0, 1.5, 0.45, 0.95)],
                6 => &[(0xFFB347, 3.0, 0.4, 0.5, 2.2), (0xFFF1D6, 2.0, 1.3, 0.3, 0.4)],
                7 => &[(0x39FF88, 3.0, 1.7, 0.45, 0.95), (0x00E5FF, 2.0, 1.3, 0.35, 0.95)],
                9 => &[(0xF0C060, 3.0, 1.7, 0.5, 0.95), (0x3FE0C8, 2.0, 1.3, 0.4, 0.95)],
                11 => &[(0xFFD84A, 3.0, 1.7, 0.45, 0.95), (0xFF7A1C, 2.0, 1.3, 0.35, 0.95)],
                _ => &[(0xFF3B5C, 4.0, 2.1, 0.5, 0.95), (0x4D8DFF, 4.0, 1.8, 0.45, 0.95), (0xFFFFFF, 2.0, 1.2, 0.25, 0.95)],
            };
            for (ri, (col, w, rmax, life, from)) in rings.iter().enumerate() {
                self.rings.push(Ring { ticks: self.skin == 9 && ri == 0, x: l.cx, y: l.cy, r0: l.r * from, r1: l.r * rmax, l: *life, m: *life, col: *col, w: *w });
            }
        }
        if self.is_ec() {
            if sc >= 1.0 {
                self.bolt_l = 0.22;
                self.bolt_seed = (self.rnd() * 9000.0) as u32;
                self.orb_pulse = 1.0;
            } else {
                self.orb_pulse = self.orb_pulse.max(0.45);
            }
        }
        if self.skin == 7 && sc >= 1.0 {
            self.beam_l = 0.35;
            self.beam_x = l.cx;
            self.beam_y = l.cy;
        }
        if self.skin >= 3 && sc >= 0.9 {
            self.shake = 0.16;
        }
    }

    fn fx_color(&mut self) -> u32 {
        let r = self.rnd();
        let pick = |list: &[u32], r: f32| list[(r * list.len() as f32) as usize % list.len()];
        match self.skin {
            0 => pick(&[0xC5C8CF, 0x8C8F99, 0xE3E5EA], r),
            1 => pick(&[0xFFD24D, 0xF2C94C, 0xFFF3B0], r),
            2 => pick(&[0xB9F2FF, 0xFFFFFF, 0x5FD3F0], r),
            3 => pick(&[0xFF8A3D, 0xFFB347, 0x9B5AFF, 0x6A3BD1], r),
            4 => pick(&[0xC77DFF, 0xFF7AE0, 0x9B6BFF, 0x7A8BFF], r),
            5 => hsv_hex((self.anim_t * 80.0 + r * 140.0) % 360.0, 0.85, 1.0),
            6 => pick(&[0xFFE2B0, 0xFFB347, 0xFF7A2F, 0xFFFFFF], r),
            7 => pick(&[0x39FF88, 0xA6FFCB, 0x00E5FF, 0xD4FF3A], r),
            11 => pick(&[0xFFF3B0, 0xFFD84A, 0xFF9A1C, 0xFF5A14], r),
            9 => pick(&[0xF0C060, 0xFFE3A0, 0x3FE0C8, 0xFFFFFF], r),
            _ => pick(&[0xFFFFFF, 0xFF3B5C, 0x4D8DFF, 0xCFE3FF], r),
        }
    }

    fn update_fx(&mut self, dt: f32) {
        self.beam_l = (self.beam_l - dt).max(0.0);
        self.bolt_l = (self.bolt_l - dt).max(0.0);
        self.orb_pulse = (self.orb_pulse - dt * 3.0).max(0.0);
        for q in &mut self.parts {
            q.l -= dt;
            if q.sw != 0.0 {
                let (c, s) = ((q.sw * dt).cos(), (q.sw * dt).sin());
                let vx = q.vx * c - q.vy * s;
                q.vy = q.vx * s + q.vy * c;
                q.vx = vx;
            }
            if q.kill > 0.0 {
                let (dx, dy) = (q.tx - q.x, q.ty - q.y);
                let d = (dx * dx + dy * dy).sqrt().max(0.001);
                if d < q.kill {
                    q.l = 0.0;
                }
                q.vx += dx / d * q.acc * dt;
                q.vy += dy / d * q.acc * dt;
            }
            q.vy += q.g * dt;
            q.x += q.vx * dt;
            q.y += q.vy * dt;
            q.rot += q.vr * dt;
        }
        self.parts.retain(|q| q.l > 0.0);
        for r in &mut self.rings {
            r.l -= dt;
        }
        self.rings.retain(|r| r.l > 0.0);
    }

    fn draw_fx(&self, g: &mut Gfx, l: &Layout, s: f32) {
        use std::f32::consts::{FRAC_PI_2, TAU};
        if self.bolt_l > 0.0 {
            let a = self.bolt_l / 0.22;
            for k in 0..5u32 {
                let seed = self.bolt_seed + k * 17;
                let an = hash01(seed, 81) * TAU;
                let col = if k % 2 == 0 { 0xFF6E82 } else { 0x8CBEFF };
                draw_bolt(g, l.cx, l.cy, l.cx + an.cos() * l.r * 2.3, l.cy + an.sin() * l.r * 2.3, seed, 8, l.r * 0.35, col, a, 1.3 * s);
            }
        }
        if self.beam_l > 0.0 {
            let a = self.beam_l / 0.35;
            let bw = l.r * 0.55 * (0.5 + 0.5 * a);
            g.rect(self.beam_x - bw / 2.0, 0.0, bw, self.beam_y, rgba(0x78FFAA, 0.35 * a));
            g.rect(self.beam_x - bw * 0.12, 0.0, bw * 0.24, self.beam_y, rgba(0xE6FFF0, 0.6 * a));
        }
        for rg in &self.rings {
            let p = 1.0 - rg.l / rg.m;
            let rad = rg.r0 + (rg.r1 - rg.r0) * p;
            let col = if rg.col == 0 { hsv_hex((self.anim_t * 60.0) % 360.0, 0.9, 1.0) } else { rg.col };
            if rg.ticks {
                for k in 0..12u32 {
                    let a = k as f32 * std::f32::consts::PI / 6.0;
                    for j in 0..5u32 {
                        let rr = rad + j as f32 * 3.0 * s;
                        g.circle(rg.x + a.cos() * rr, rg.y + a.sin() * rr, rg.w * s * 0.5, rgba(col, ((1.0 - p) * 0.9).max(0.0)));
                    }
                }
                continue;
            }
            for k in 0..36u32 {
                let an = k as f32 / 36.0 * TAU;
                g.circle(rg.x + an.cos() * rad, rg.y + an.sin() * rad, rg.w * s * 0.55 * (1.0 - p * 0.5), rgba(col, ((1.0 - p) * 0.9).max(0.0)));
            }
        }
        for q in &self.parts {
            if q.y > l.shop_top - 2.0 {
                continue;
            }
            let a = (q.l / q.m).clamp(0.0, 1.0);
            let al = (a * 1.4).min(1.0);
            match q.kind {
                0 => g.circle(q.x, q.y, q.s * (0.5 + 0.5 * a), rgba(q.col, al)),
                1 => {
                    g.circle(q.x, q.y, q.s * 0.8, rgba(q.col, al));
                    for k in 0..4u32 {
                        let an = q.rot + k as f32 * FRAC_PI_2;
                        g.circle(q.x + an.cos() * q.s * 1.7, q.y + an.sin() * q.s * 1.7, q.s * 0.45, rgba(q.col, al));
                    }
                }
                5 => {
                    let (c, sn) = (q.rot.cos(), q.rot.sin());
                    for k in 0..5u32 {
                        g.circle(q.x + c * q.s * 0.8 * k as f32, q.y + sn * q.s * 0.8 * k as f32, q.s * 0.5, rgba(q.col, al));
                    }
                }
                4 => {
                    let h = (q.s * (0.5 + 0.5 * a)).max(1.0);
                    g.rect(q.x - h, q.y - h, h * 2.0, h * 2.0, rgba(q.col, al))
                }
                2 => {
                    let (c, sn) = (q.rot.cos(), q.rot.sin());
                    g.circle(q.x, q.y, q.s * 0.9, rgba(q.col, al));
                    g.circle(q.x + c * q.s * 1.5, q.y + sn * q.s * 1.5, q.s * 0.55, rgba(q.col, al));
                    g.circle(q.x - c * q.s * 1.5, q.y - sn * q.s * 1.5, q.s * 0.55, rgba(q.col, al));
                }
                _ => {
                    for k in 0..4u32 {
                        let f = k as f32 * 0.03;
                        g.circle(q.x - q.vx * f, q.y - q.vy * f, q.s * (0.8 - 0.15 * k as f32), rgba(q.col, al * (1.0 - 0.22 * k as f32)));
                    }
                }
            }
        }
    }

    fn draw_cursors(&self, g: &mut Gfx, l: &Layout, s: f32) {
        use std::f32::consts::TAU;
        let m = self.cur_n.min(24);
        if m == 0 {
            return;
        }
        let size = (34.0 * s).ceil() as u32;
        for i in 0..m {
            let (x, y, theta) = self.cursor_pos(i, m, l);
            let k = (((theta + TAU) % TAU) / (TAU / 16.0)).round() as u32 % 16;
            // souris normales, ou souris d'énergie rouges / bleues qui brillent (château Éclipse)
            let (fill, border) = if self.is_ec() { (if i % 2 == 0 { 0xFF3B5C } else { 0x4D8DFF }, 0xFFFFFF) } else { (0xFFFFFF, 0x1B1B1F) };
            if self.is_ec() {
                g.circle(x, y, 9.0 * s, rgba(fill, 0.25));
                g.circle(x, y, 5.0 * s, rgba(fill, 0.35));
            }
            let key = format!("cur{}_{}_{:06x}", k, size, fill);
            let (pw, ph) = g.img(&key, &|| make_cursor(k, size, s * 0.72, fill, border));
            g.blit(&key, x - pw as f32 / 2.0, y - ph as f32 / 2.0, 1.0);
        }
    }

    fn castle_tap(&mut self, ctx: &Ctx, k: usize) {
        if k < SKBUY.len() {
            let (name, target, cost) = SKBUY[k];
            if self.own[target as usize] {
                ctx.toast("Tu as déjà ce château (équipe-le dans Menu › Conquis)");
            } else if self.pts < cost {
                ctx.toast("Pas assez de territoires");
            } else {
                self.pts -= cost;
                self.bump(ctx, 2, 1);
                let msg = format!("{} acheté ! ×{} pour toujours", name, fmt_mult(skin_base(target)));
                self.unlock_skin(ctx, target, &msg);
                self.screen = Screen::Game;
            }
            return;
        }
        let k = k - SKBUY.len();
        if k >= CASTLES.len() {
            return;
        }
        let c = &CASTLES[k];
        if self.cs[k] {
            ctx.toast("Tu possèdes déjà ce château");
        } else if self.pts < c.1 {
            ctx.toast("Pas assez de territoires");
        } else {
            self.pts -= c.1;
            self.cs[k] = true;
            self.bump(ctx, 2, 1);
            self.save();
            ctx.toast(format!("{} acheté ! ×{} en plus", c.0, fmt_mult(c.2)));
        }
    }

    fn magic_tap(&mut self, ctx: &Ctx, k: usize) {
        if k >= MAGIC.len() {
            let j = k - MAGIC.len();
            if j < self.abilities().len() {
                self.activate_ability(ctx, j);
            }
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
                self.bump(ctx, 2, 1);
                self.save();
                ctx.toast(format!("Débloqué : {}", m.name));
            }
            return;
        }
        if !m.spell {
            ctx.toast("Déjà actif");
            return;
        }
        if self.mana < m.mana && self.skin != 10 {
            ctx.toast(if m.mana > self.max_mana() { format!("Il te faut plus de mana max ({})", m.mana) } else { "Pas assez de mana".to_string() });
            return;
        }
        let dm = if self.has("sablier") { 2.0 } else { 1.0 };
        let left = match m.id {
            "frenesie" => self.frenzy_left,
            "benediction" => self.bless_left,
            "clone" => self.clone_left,
            "ascension" => self.asc_left,
            _ => 0.0,
        };
        if left > 0.0 {
            ctx.toast("Déjà en cours");
            return;
        }
        if self.skin != 10 {
            self.mana -= m.mana;
        }
        self.bump(ctx, 1, 1);
        match m.id {
            "frenesie" => {
                self.frenzy_left = m.dur * dm;
                ctx.toast("Frénésie ! Clics ×7");
            }
            "benediction" => {
                self.bless_left = m.dur * dm;
                ctx.toast("Bénédiction ! Production ×3");
            }
            "clone" => {
                self.clone_left = m.dur * dm;
                ctx.toast("Clone magique ! Cliqueurs 3× plus rapides");
            }
            "ascension" => {
                self.asc_left = m.dur * dm;
                ctx.toast("ASCENSION ARCANIQUE ! Tout ×10");
            }
            "pluie" => {
                let g = (self.rate() * 60.0).max(self.click_power() * 50.0) * if self.has("pacte") { 3.0 } else { 1.0 } * 1.5f64.powi(self.tl[5] as i32);
                self.pts += g;
                self.total += g;
                ctx.toast(format!("Pluie d’or : +{}", fmt(g)));
            }
            "distorsion" => {
                let g = (self.rate() * 600.0).max(self.click_power() * 200.0);
                self.pts += g;
                self.total += g;
                ctx.toast(format!("Distorsion temporelle : +{}", fmt(g)));
            }
            _ => {}
        }
    }

    fn submit_code(&mut self, ctx: &Ctx) {
        let code = self.code.split_whitespace().collect::<Vec<_>>().join(" ").to_uppercase();
        self.code.clear();
        match self.screen {
            Screen::Code => {
                if code == "BUG" {
                    self.save();
                    self.screen = Screen::Video;
                    self.vt = 0.0;
                    self.last = Instant::now();
                } else if code == "TUNE" {
                    self.screen = Screen::Tune;
                } else if code == "TESTEBOSS" {
                    if self.boss.is_some() {
                        ctx.toast("Un boss est déjà présent");
                    } else {
                        self.spawn_boss(ctx, true);
                    }
                } else if code.is_empty() {
                    ctx.toast("Entre un code");
                } else if self.verify.is_some() {
                    ctx.toast("Vérification en cours…");
                } else {
                    // peut-être le code caché → vérification lente (empreinte) dans un thread
                    let (tx, rx) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        let ok = code_hash(&code, "cq-6:") == K6;
                        let _ = tx.send((6u8, ok, String::new()));
                    });
                    self.verify = Some(rx);
                    ctx.toast("Vérification…");
                }
            }
            Screen::Tune => {
                if code == "MAX" {
                    self.screen = Screen::Tune3;
                    return;
                }
                let target = match code.as_str() {
                    "INFINI" => Some(1u8),
                    "DIAMEN" => Some(2),
                    "NETHERITE" => Some(3),
                    _ => None,
                };
                if let Some(k) = target {
                    if self.own[k as usize] {
                        ctx.toast("Déjà débloqué : équipe-le dans Menu › Conquis");
                    } else {
                        let msg = match k {
                            1 => "Château en or ! ×4 pour toujours",
                            2 => "Château en diamant ! ×1000 pour toujours",
                            _ => "Château en netherite ! ×10000 pour toujours",
                        };
                        self.unlock_skin(ctx, k, msg);
                    }
                } else if code.is_empty() {
                    ctx.toast("Entre le 2e code");
                } else {
                    ctx.toast("Code invalide");
                }
            }
            Screen::Tune4 => {
                if code.is_empty() {
                    ctx.toast("Entre le 4e code");
                } else if self.verify.is_some() {
                    ctx.toast("Vérification en cours…");
                } else {
                    let (tx, rx) = std::sync::mpsc::channel();
                    let pw = format!("{}\n{}", self.code3_ok, code);
                    std::thread::spawn(move || {
                        let h = code_hash(&pw, "cq-4:");
                        let _ = tx.send(if h == K5 { (5u8, true, String::new()) } else { (4u8, h == K4, String::new()) });
                    });
                    self.verify = Some(rx);
                    ctx.toast("Vérification…");
                }
            }
            Screen::Tune3 => {
                let target = match code.as_str() {
                    "GALAXI" => Some(4u8),
                    "MAXGALAXI" => Some(5),
                    "TROU NOIR" | "TROUNOIR" => Some(6),
                    "ALIEN" => Some(7),
                    "TEMPS" => Some(9),
                    "SOLAIRE" => Some(11),
                    _ => None,
                };
                if let Some(k) = target {
                    if k == 5 && !self.own[4] {
                        ctx.toast("Il faut d’abord le château galaxie (GALAXI)");
                    } else if self.own[k as usize] {
                        ctx.toast("Déjà débloqué : équipe-le dans Menu › Conquis");
                    } else {
                        let msg = match k {
                            4 => "Château galaxie ! ×100000 pour toujours",
                            5 => "Galaxie MAX ! ×1000000 pour toujours",
                            6 => "TROU NOIR ! ×10000000 pour toujours",
                            7 => "Mode ALIEN ! ×100000000 pour toujours",
                            11 => "CHÂTEAU SOLAIRE ! Clics et production ×10 milliards pour toujours",
                            _ => "Château du TEMPS ! ×1 milliard pour toujours",
                        };
                        self.unlock_skin(ctx, k, msg);
                        if k == 11 && self.skin == 11 {
                            self.solar_t = 0.0;
                            self.screen = Screen::SolarFx;
                        }
                    }
                } else if code.is_empty() {
                    ctx.toast("Entre le 3e code");
                } else if self.verify.is_some() {
                    ctx.toast("Vérification en cours…");
                } else {
                    // code inconnu : peut-être le code secret → vérification lente dans un thread
                    let (tx, rx) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        let ok = code_hash(&code, "cq-3:") == K3;
                        let _ = tx.send((3u8, ok, code));
                    });
                    self.verify = Some(rx);
                    ctx.toast("Vérification…");
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
        let rows = ["1234567890", "AZERTYUIOP", "QSDFGHJKLM", "WXCVBN?"];
        let mut v = Vec::new();
        for (ri, row) in rows.iter().enumerate() {
            let y = top + ri as f32 * (kh + gap);
            for (ci, c) in row.chars().enumerate() {
                v.push((Rect::new(6.0 * s + ci as f32 * (kw + gap), y, kw, kh), Kb::Ch(c)));
            }
        }
        let y = top + 3.0 * (kh + gap);
        v.push((Rect::new(6.0 * s + 7.0 * (kw + gap), y, kw * 1.5 + gap * 0.5, kh), Kb::Del));
        v.push((Rect::new(6.0 * s + 8.5 * (kw + gap), y, kw * 1.5 + gap * 0.5, kh), Kb::Space));
        v
    }
    fn submit_rect(&self, ctx: &Ctx) -> Rect {
        let s = ctx.scale();
        let y = 44.0 * s + 86.0 * s + 4.0 * (42.0 * s + 3.0 * s) + 10.0 * s;
        Rect::new(16.0 * s, y, ctx.width() - 32.0 * s, 46.0 * s)
    }

    // ---- menu ----------------------------------------------------------------------------
    fn menu_buttons(&self, ctx: &Ctx) -> (Rect, [Rect; 6]) {
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let pw = w * 0.82;
        let bh = 36.0 * s;
        let gap = 8.0 * s;
        let ph = 56.0 * s + 6.0 * bh + 5.0 * gap + 16.0 * s;
        let px = (w - pw) / 2.0;
        let py = (h - ph) / 2.0;
        let bx = px + 16.0 * s;
        let bw = pw - 32.0 * s;
        let y0 = py + 56.0 * s;
        let mut b = [Rect::new(0.0, 0.0, 0.0, 0.0); 6];
        for (i, r) in b.iter_mut().enumerate() {
            *r = Rect::new(bx, y0 + i as f32 * (bh + gap), bw, bh);
        }
        (Rect::new(px, py, pw, ph), b)
    }

    fn bench_row(&self, ctx: &Ctx) -> Rect {
        let s = ctx.scale();
        Rect::new(0.0, 44.0 * s + 16.0 * s + 64.0 * s, ctx.width(), 60.0 * s)
    }

    fn music_row(&self, ctx: &Ctx) -> Rect {
        let s = ctx.scale();
        Rect::new(0.0, 44.0 * s + 16.0 * s + 128.0 * s, ctx.width(), 60.0 * s)
    }

    fn settings_row(&self, ctx: &Ctx) -> Rect {
        let s = ctx.scale();
        Rect::new(0.0, 44.0 * s + 16.0 * s, ctx.width(), 60.0 * s)
    }

    // ---- lignes de liste -----------------------------------------------------------------
    /// (titre, détail, texte de droite, mis en avant, verrouillé)
    fn shop_row(&self, row: usize) -> (String, String, String, bool, bool) {
        if row == 0 {
            let c = self.click_cost();
            ("Épée affûtée".into(), format!("Clic ×2 · coût {}", fmt(c)), format!("Niv {}", self.click_lvl), self.pts >= c, false)
        } else if row == self.tiers.len() + 1 {
            let c = self.bb_cost();
            (
                "Le Big Bang".into(),
                format!("Remet tout à zéro · ×2 aux multiplicateurs · coût {}", fmt_mult(c)),
                format!("×{}", fmt_mult(self.bb_mult())),
                self.pts >= c,
                false,
            )
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
        if k >= MAGIC.len() {
            return self.ability_row(k - MAGIC.len());
        }
        let m = &MAGIC[k];
        let own = self.mg[k];
        if !own && self.total < m.req {
            return (m.name.into(), format!("Verrouillé · {} territoires", fmt(m.req)), String::new(), false, true);
        }
        if !own {
            return (m.name.into(), format!("{} · coût {}", m.desc, fmt(m.cost)), "Débloquer".into(), self.pts >= m.cost, false);
        }
        if m.spell {
            let rem = match m.id {
                "frenesie" => self.frenzy_left,
                "benediction" => self.bless_left,
                "clone" => self.clone_left,
                "ascension" => self.asc_left,
                _ => 0.0,
            };
            let (right, ok) = if rem > 0.0 { (format!("{} s", rem.ceil()), true) } else { ("Lancer".into(), self.mana >= m.mana) };
            return (m.name.into(), format!("{} · {} mana", m.desc, m.mana), right, ok, false);
        }
        (m.name.into(), m.desc.into(), "OK".into(), true, false)
    }

    fn castle_row(&self, k: usize) -> (String, String, String, bool, bool) {
        if k < SKBUY.len() {
            let (name, target, cost) = SKBUY[k];
            let m = skin_base(target);
            return if self.own[target as usize] {
                (name.into(), format!("×{} · acquis", m), "OK".into(), true, false)
            } else {
                (name.into(), format!("×{} pour toujours · coût {}", m, fmt_short(cost)), "Acheter".into(), self.pts >= cost, false)
            };
        }
        let k = k - SKBUY.len();
        let c = &CASTLES[k];
        if self.cs[k] {
            (c.0.into(), format!("×{} · acquis", fmt_mult(c.2)), "OK".into(), true, false)
        } else {
            (c.0.into(), format!("×{} · coût {}", fmt_mult(c.2), fmt_short(c.1)), "Acheter".into(), self.pts >= c.1, false)
        }
    }

    fn skin_colors(&self) -> (Color, Color, Color, Color) {
        match self.skin {
            9 => (rgb(0xF0C060), rgb(0x04141A), rgb(0xF0C060), rgb(0x3FE0C8)),
            11 => (rgb(0xFFD84A), rgb(0x120300), rgb(0x3A0F00), rgb(0xFFD84A)),
            8 | 10 => (rgb(0xE9EEFF), rgb(0x020208), rgb(0xF4F6FF), rgb(0x0A0A12)),
            7 => (rgb(0x39FF88), rgb(0x04140C), rgba(0x0F3D22, 0.95), rgb(0x7DFF9A)),
            6 => (rgb(0xFF9A3C), rgb(0x000000), rgba(0x1E1E1E, 0.95), rgb(0xFFB347)),
            5 => (rgb(0xE0A800), rgb(0x1B0540), rgba(0x0A0120, 0.38), rgb(0xFFE08A)),
            4 => (rgb(0x6A1BB8), rgb(0x1B0540), rgba(0x0A0120, 0.38), rgb(0xFF8AF0)),
            3 => (rgb(0x0B0B0E), rgb(0x1C1C22), rgb(0x5A5A66), rgb(0xD1A54B)),
            2 => (rgb(0x2A9FC0), rgb(0xB9F2FF), rgb(0x17607A), rgb(0xE9FBFF)),
            1 => (rgb(0xB8860B), rgb(0xF2C94C), rgb(0x6B4F0A), rgb(0xF2C94C)),
            _ => (rgb(0x5B5F68), rgb(0xA3A7B0), rgb(0x454850), rgb(0xC5C8CF)),
        }
    }

    fn income_ps(&self) -> f64 {
        self.rate() + self.cur_n as f64 * self.cur_cps() * self.click_power() * self.cursor_bonus()
    }

    /// Débloque un château ; il est équipé tout de suite seulement s'il est plus fort que l'actuel.
    fn unlock_skin(&mut self, ctx: &Ctx, k: u8, msg: &str) {
        self.own[k as usize] = true;
        if skin_base(k) > skin_base(self.skin) {
            self.skin = k;
            ctx.toast(msg);
        } else {
            ctx.toast(format!("{} débloqué ! Équipe-le dans Menu › Conquis", SKIN_NAMES[k as usize]));
        }
        self.screen = Screen::Game;
        self.save();
    }

    fn dot_line(&self, g: &mut Gfx, x0: f32, y0: f32, x1: f32, y1: f32, size: f32, col: u32, al: f32) {
        let l = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
        let n = (l / (size * 0.9)).ceil().max(1.0) as u32;
        for k in 0..=n {
            let f = k as f32 / n as f32;
            g.circle(x0 + (x1 - x0) * f, y0 + (y1 - y0) * f, size, rgba(col, al));
        }
    }

    /// Château du Temps : cadran avec 60 repères dorés, engrenage, sable qui tombe, 3 aiguilles
    /// (la trotteuse « tique »), anneau de repères autour avec un arc turquoise qui tourne.
    fn draw_time(&self, g: &mut Gfx, cx: f32, cy: f32, r: f32, front: bool) {
        use std::f32::consts::TAU;
        let t = self.anim_t;
        let s = (r / 48.0).max(0.6);
        if !front {
            g.circle(cx, cy, r * 0.9, rgb(0x06202A));
            g.circle(cx, cy, r * 0.62, rgba(0x0F3A44, 0.85));
            for k in 0..16u32 {
                let a = t * 0.15 + k as f32 * TAU / 16.0;
                g.circle(cx + a.cos() * r * 0.78, cy + a.sin() * r * 0.78, r * 0.05, rgba(0xF0C060, 0.14));
            }
            for k in 0..40u32 {
                let a = k as f32 / 40.0 * TAU + t * 0.15;
                g.circle(cx + a.cos() * r * 0.7, cy + a.sin() * r * 0.7, r * 0.012, rgba(0xF0C060, 0.18));
            }
            for k in 0..60u32 {
                let a = k as f32 / 60.0 * TAU;
                let big = k % 5 == 0;
                let (r1, r2) = (r * if big { 0.8 } else { 0.86 }, r * 0.92);
                self.dot_line(g, cx + a.sin() * r1, cy - a.cos() * r1, cx + a.sin() * r2, cy - a.cos() * r2, if big { 1.6 * s } else { 0.8 * s }, 0xF0C060, if big { 0.95 } else { 0.45 });
            }
            for k in 0..14u32 {
                let f = (t * 0.25 + hash01(k, 141)) % 1.0;
                g.circle(cx + r * (0.5 + hash01(k, 142) * 0.4), cy - r * 0.8 + r * 1.5 * f, (0.8 + hash01(k, 143) * 1.2) * s, rgba(0xFFDC8C, 0.7 * (1.0 - f)));
            }
        } else {
            let tk = (t * 6.0).floor();
            let fr = ((t * 6.0 - tk) * 3.2).min(1.0);
            let sa = (tk + fr + (fr * std::f32::consts::PI).sin() * 0.15) * (TAU / 20.0);
            for (a, l, w, col, al) in [(t * 0.1, 0.46f32, 2.4f32, 0xF0C060u32, 0.95f32), (t * 0.8, 0.68, 1.6, 0xFFECBE, 0.95), (sa, 0.78, 0.9, 0x3FE0C8, 0.95)] {
                self.dot_line(g, cx - a.sin() * r * l * 0.18, cy + a.cos() * r * l * 0.18, cx + a.sin() * r * l, cy - a.cos() * r * l, w * s, col, al);
            }
            g.circle(cx, cy, r * 0.07, rgb(0xF0C060));
            g.circle(cx, cy, r * 0.03, rgb(0x04141A));
            // halo : anneau de repères autour du château + arc turquoise qui tourne
            for k in 0..60u32 {
                let a = k as f32 / 60.0 * TAU - TAU / 4.0;
                let big = k % 5 == 0;
                let (r1, r2) = (r * 1.12, r * 1.12 + if big { 9.0 * s } else { 4.5 * s });
                self.dot_line(g, cx + a.cos() * r1, cy + a.sin() * r1, cx + a.cos() * r2, cy + a.sin() * r2, if big { 1.4 * s } else { 0.8 * s }, 0xF0C060, if big { 0.9 } else { 0.4 });
            }
            let sa0 = ((t * 0.5) % 1.0) * TAU - TAU / 4.0;
            for k in 0..28u32 {
                let a = sa0 - 0.8 * (1.0 - k as f32 / 27.0);
                g.circle(cx + a.cos() * (r * 1.12 - 3.0 * s), cy + a.sin() * (r * 1.12 - 3.0 * s), 1.7 * s, rgba(0x3FE0C8, 0.35 + 0.6 * k as f32 / 27.0));
            }
        }
    }

    /// Avancement temporel (≈ 4 s) : un cadran géant dont les aiguilles s'emballent (avec traînées),
    /// puis se calent sur midi : flash, titre et gain qui défile.
    fn draw_timefx(&mut self, ctx: &Ctx, g: &mut Gfx) {
        use std::f32::consts::TAU;
        let t = self.time_t;
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let (cx, cy) = (w / 2.0, h * 0.46);
        let m = w.min(h);
        let r = m * 0.36;
        let fade = if t < 0.3 { 1.0 - t / 0.3 } else if t > 3.3 { ((t - 3.3) / 0.7).min(1.0) } else { 0.0 };
        if fade > 0.0 {
            self.draw_game(ctx, g);
            g.rect(0.0, 0.0, w, h, rgba(0x02080C, 1.0 - fade));
        } else {
            g.clear(rgb(0x02080C));
        }
        g.circle(cx, cy, m * 0.8, rgba(0x0F3A44, 0.35));
        g.circle(cx, cy, m * 0.5, rgba(0x0F3A44, 0.4));
        let ang = |tt: f32| -> f32 {
            if tt < 2.2 {
                40.0 * tt * tt * tt / (3.0 * 4.84)
            } else {
                let base = 40.0 * 2.2f32.powi(3) / (3.0 * 4.84);
                let tgt = (base / TAU).ceil() * TAU;
                let u = ((tt - 2.2) / 0.6).min(1.0);
                base + (tgt - base) * (1.0 - (1.0 - u).powi(3))
            }
        };
        let spd = if t < 2.2 { t * t / 4.84 } else { (1.0 - (t - 2.2) / 0.6).max(0.0) };
        for i in 0..40u32 {
            let a = i as f32 / 40.0 * TAU + t * 0.3;
            let f = (t * (0.6 + spd * 2.0) + i as f32 * 0.173) % 1.0;
            let r1 = r * (1.05 + f * 1.6);
            let r2 = r1 + r * 0.25 * (0.3 + spd);
            self.dot_line(g, cx + a.cos() * r1, cy + a.sin() * r1, cx + a.cos() * r2, cy + a.sin() * r2, 1.2 * s, 0xF0C060, 0.35 * (1.0 - f) * (0.3 + spd));
        }
        for i in 0..3u32 {
            let rf = (t * 0.9 + i as f32 / 3.0) % 1.0;
            for k in 0..80u32 {
                let a = k as f32 / 80.0 * TAU;
                g.circle(cx + a.cos() * r * (1.0 + rf * 1.4), cy + a.sin() * r * (1.0 + rf * 1.4), 2.0 * s, rgba(0x3FE0C8, 0.4 * (1.0 - rf)));
            }
        }
        for k in 0..90u32 {
            let a = k as f32 / 90.0 * TAU;
            g.circle(cx + a.cos() * r, cy + a.sin() * r, r * 0.022, rgba(0xF0C060, 0.85));
        }
        for i in 0..60u32 {
            let ta = i as f32 / 60.0 * TAU;
            let big = i % 5 == 0;
            let (r1, r2) = (r * if big { 0.82 } else { 0.88 }, r * 0.95);
            self.dot_line(g, cx + ta.sin() * r1, cy - ta.cos() * r1, cx + ta.sin() * r2, cy - ta.cos() * r2, if big { 2.0 * s } else { 0.9 * s }, if big { 0xFFE6AA } else { 0xF0C060 }, if big { 0.95 } else { 0.5 });
        }
        for j in (0..=14u32).rev() {
            let tt = (t - j as f32 * 0.013).max(0.0);
            let al = if j == 0 { 1.0 } else { 0.5 * (1.0 - j as f32 / 15.0) };
            let aa = ang(tt);
            self.dot_line(g, cx, cy, cx + aa.sin() * r * 0.85, cy - aa.cos() * r * 0.85, if j == 0 { 2.4 * s } else { 1.5 * s }, 0x3FE0C8, al);
            let ha = aa / 12.0;
            self.dot_line(g, cx, cy, cx + ha.sin() * r * 0.55, cy - ha.cos() * r * 0.55, if j == 0 { 3.6 * s } else { 2.0 * s }, 0xF0C060, al);
        }
        g.circle(cx, cy, r * 0.07, rgb(0xF0C060));
        if t > 2.7 {
            let tp = ((t - 2.7) / 0.4).min(1.0);
            let fs = (w * 0.1).min(44.0 * s) * (0.8 + 0.2 * tp);
            let cnt = ((t - 2.7) / 0.7).min(1.0) as f64;
            let al = (tp * 1.5).min(1.0) * if t > 3.5 { (1.0 - (t - 3.5) / 0.5).max(0.0) } else { 1.0 };
            self.text_outline(ctx, g, "AVANCEMENT", cx, cy - r * 1.25, fs, 0xFFE3A0, al, w - 10.0);
            self.text_outline(ctx, g, "TEMPOREL", cx, cy - r * 1.25 + fs * 0.95, fs, 0xFFE3A0, al, w - 10.0);
            let f2 = (fs * 0.42).max(13.0 * s);
            self.text_outline(ctx, g, &format!("+{} territoires", fmt(self.time_gain * cnt)), cx, cy + r * 1.3, f2, 0x7FF3E0, al, w - 10.0);
            self.text_outline(ctx, g, "5 minutes de production en un instant", cx, cy + r * 1.3 + f2 * 1.1, (fs * 0.26).max(11.0 * s), 0xFFFFFF, al, w - 12.0);
        }
        let fl = (1.0 - (t - 2.75).abs() / 0.14).max(0.0);
        if fl > 0.0 {
            g.rect(0.0, 0.0, w, h, rgba(0xC8FFF5, fl));
        }
    }

    /// Menu « Conquis » : liste des châteaux (débloqués ou non) pour équiper un château plus ou moins fort.
    fn draw_skins(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = self.theme_for(ctx);
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        g.clear(t.bg);
        let top = 44.0 * s + 22.0 * s;
        let row_h = 58.0 * s;
        let n = SKIN_NAMES.len();
        self.skin_scroll.set_content(n as f32 * row_h + 8.0 * s, h - top);
        for i in 0..n {
            let y = top + i as f32 * row_h - self.skin_scroll.pos;
            if y + row_h < top - 2.0 || y > h {
                continue;
            }
            let own = self.own[i];
            let eq = self.skin as usize == i;
            let r = Rect::new(8.0 * s, y + 3.0 * s, w - 16.0 * s, row_h - 6.0 * s);
            g.rrect(r.x, r.y, r.w, r.h, 10.0 * s, if eq { t.accent } else { t.line });
            g.rrect(r.x + 1.5 * s, r.y + 1.5 * s, r.w - 3.0 * s, r.h - 3.0 * s, 9.0 * s, if own { t.surface } else { shade(t.surface, 0.85) });
            let tc = if own { t.text } else { t.subtext };
            ctx.text_mid(g, if own { SKIN_NAMES[i] } else { "???" }, r.x + 14.0 * s, r.y + r.h * 0.33, &Text::new(16.0 * s).bold().color(tc).left().max_width(r.w * 0.55));
            let d = if own { format!("Clics et production ×{}", fmt_mult(skin_base(i as u8))) } else { "Verrouillé".to_string() };
            ctx.text_mid(g, &d, r.x + 14.0 * s, r.y + r.h * 0.7, &Text::new(11.5 * s).color(if own { t.accent } else { t.subtext }).left().max_width(r.w * 0.7));
            let (label, col) = if eq { ("Équipé", t.accent) } else if own { ("Équiper", t.accent) } else { ("", t.subtext) };
            ctx.text_mid(g, label, r.right() - 12.0 * s, r.y + r.h / 2.0, &Text::new(13.0 * s).bold().color(col).right().max_width(r.w * 0.3));
        }
        let head = self.hdr(ctx, g, "Conquis", Some("Retour"));
        let n_own = self.own.iter().filter(|b| **b).count();
        g.rect(0.0, head.height, w, 22.0 * s, t.bg);
        ctx.text_mid(g, &format!("{} / {} châteaux débloqués · touche pour équiper", n_own, n), w / 2.0, head.height + 11.0 * s, &Text::new(11.0 * s).color(t.subtext).center().max_width(w - 8.0 * s));
    }

    // ---- v12 : succès, quêtes, talents, boss, combos, Orbe violette, musique ---------------
    fn ach_mult(&self) -> f64 {
        1.02f64.powi(self.ach.iter().filter(|b| **b).count() as i32)
    }
    fn combo_active(&self, k: u8) -> bool {
        match k {
            0 => self.frenzy_left > 0.0,
            1 => self.bless_left > 0.0,
            2 => self.clone_left > 0.0,
            3 => self.asc_left > 0.0,
            4 => self.skin == 6 && self.clock < self.vide_until,
            _ => self.ext_on(),
        }
    }
    fn combo_list(&self) -> Vec<usize> {
        (0..CMB.len()).filter(|&i| self.combo_active(CMB[i].1) && self.combo_active(CMB[i].2)).collect()
    }
    fn combo_mult(&self) -> f64 {
        self.combo_list().iter().map(|&i| CMB[i].3).product()
    }
    fn ach_cur(&self, kind: u8) -> f64 {
        match kind {
            0 => self.st[0] as f64,
            1 => self.total.max(self.lt_max),
            2 => self.st[2] as f64,
            3 => self.st[1] as f64,
            4 => self.st[3] as f64,
            5 => self.own.iter().filter(|b| **b).count() as f64,
            6 => self.bb as f64,
            7 => self.st[5] as f64,
            _ => self.st[4] as f64,
        }
    }
    fn check_ach(&mut self, ctx: &Ctx) {
        for i in 0..ACH.len() {
            if !self.ach[i] && self.ach_cur(ACH[i].1) >= ACH[i].2 {
                self.ach[i] = true;
                ctx.toast(format!("Succès : {} (+2 % à tout)", ACH[i].0));
                self.save();
            }
        }
    }
    fn bump(&mut self, ctx: &Ctx, stat: usize, n: u64) {
        self.st[stat] += n;
        self.quest_add(ctx, stat, n);
    }
    fn quest_add(&mut self, ctx: &Ctx, stat: usize, n: u64) {
        let mut done_now = false;
        let gain = self.income_ps() * 120.0;
        for q in self.q.iter_mut() {
            let (st_k, target, txt) = (QP[q.0].0, QP[q.0].1, QP[q.0].2);
            if st_k == stat && !q.2 {
                q.1 = (q.1 + n).min(target);
                if q.1 >= target {
                    q.2 = true;
                    done_now = true;
                    self.st[5] += 1;
                    self.pts += gain;
                    self.total += gain;
                    ctx.toast(format!("Quête terminée : {} · +{}", txt, fmt(gain)));
                }
            }
        }
        if done_now && !self.q.is_empty() && self.q.iter().all(|q| q.2) && self.chest == 0 {
            self.chest = 1;
            ctx.toast("Coffre du jour prêt ! (Menu › Progrès › Quêtes)");
        }
    }
    fn gen_quests(&mut self) {
        let day = (self.clock / 86400.0) as i64;
        self.q_day = day;
        let mut pool: Vec<usize> = (0..QP.len()).filter(|&i| QP[i].0 != 4 || matches!(self.skin, 6 | 8 | 9 | 10 | 11)).collect();
        self.q.clear();
        for pick in 0..3u32 {
            if pool.is_empty() {
                break;
            }
            let j = (hash01((day as u32).wrapping_mul(31).wrapping_add(pick), 171) * pool.len() as f32) as usize % pool.len();
            self.q.push((pool.remove(j), 0, false));
        }
        self.chest = 0;
    }
    fn open_chest(&mut self, ctx: &Ctx) {
        if self.chest != 1 {
            return;
        }
        let g = self.income_ps() * 1200.0;
        self.pts += g;
        self.total += g;
        self.tp += 1;
        self.chest = 2;
        self.save();
        ctx.toast(format!("Coffre ouvert : +{} et 1 point de talent", fmt(g)));
    }
    fn buy_talent(&mut self, ctx: &Ctx, i: usize) {
        let (max, base) = (TAL[i].2, TAL[i].3);
        let l = self.tl[i];
        let c = base + l;
        if l >= max {
            ctx.toast("Niveau maximum");
        } else if self.tp < c {
            ctx.toast(format!("Il te faut {} point(s) de talent", c));
        } else {
            self.tp -= c;
            self.tl[i] = l + 1;
            self.save();
        }
    }

    fn boss_hp(&self) -> u32 {
        let base = 20 + (self.st[3] * 2).min(40) as u32;
        ((base as f32 * (1.0 - 0.1 * self.tl[3] as f32)).round() as u32).max(5)
    }
    fn boss_loot(&self) -> f64 {
        (self.income_ps() * 600.0).max(self.click_power() * 100.0) * (1.0 + 0.5 * self.tl[3] as f64) * if self.skin == 10 { 5.0 } else { 1.0 }
    }
    fn spawn_boss(&mut self, ctx: &Ctx, test: bool) {
        let k = self.st[3] as usize;
        let hp = self.boss_hp();
        // age négatif : le boss reste invisible pendant l'animation d'apparition, puis surgit à t = 1,7 s
        self.boss = Some(Boss { hp, max: hp, skin: if self.skin == 10 { 8 } else { self.skin.min(11) }, lvl: k as u32 + 1, side: (k % 2) as u8, fl: 0.0, kb: 0.0, age: -1.7, test });
        self.boss_intro_t = 0.0;
        self.screen = Screen::BossIntro;
        let _ = ctx;
        self.save();
    }
    fn kill_boss(&mut self, ctx: &Ctx) {
        let Some(b) = self.boss.take() else { return };
        let name = BOSS_NAMES[(b.skin as usize).min(11)];
        if b.test {
            ctx.toast(format!("{} (test) vaincu : aucun butin", name));
        } else {
            let g = self.boss_loot();
            self.pts += g;
            self.total += g;
            self.boss_next = self.clock + 600.0;
            self.bump(ctx, 3, 1);
            ctx.toast(format!("{} vaincu ! Butin : +{} (10 min de production)", name, fmt(g)));
        }
        self.save();
        self.boss_dying = Some((b, 0.0));
    }
    fn boss_pos(&self, ctx: &Ctx) -> (f32, f32, f32) {
        let side = self.boss.as_ref().map_or(0, |b| b.side);
        self.boss_pos_side(ctx, side)
    }
    fn boss_pos_side(&self, ctx: &Ctx, side: u8) -> (f32, f32, f32) {
        let l = self.layout(ctx);
        let s = ctx.scale();
        (ctx.width() * if side == 0 { 0.17 } else { 0.83 }, l.cy - 12.0 * s, 36.0 * s)
    }
    fn orb_btn_rect(&self, ctx: &Ctx) -> Rect {
        let (bx, by, br) = self.boss_pos(ctx);
        let s = ctx.scale();
        Rect::new(bx - 34.0 * s, by + br + 20.0 * s, 68.0 * s, 18.0 * s)
    }
    fn hit_boss(&mut self, ctx: &Ctx) {
        let (bx, by, _) = self.boss_pos(ctx);
        let eclipse = self.is_ec();
        let Some(b) = self.boss.as_mut() else { return };
        b.fl = 1.0;
        b.kb = 1.0;
        if eclipse {
            ctx.toast("Insensible aux coups : utilise l’Orbe violette");
            return;
        }
        b.hp = b.hp.saturating_sub(1);
        let dead = b.hp == 0;
        if self.floats.len() < 8 {
            let jx = (self.rnd() - 0.5) * 30.0;
            self.floats.push(Floater { x: bx + jx, y: by - 20.0, t: 0.0, txt: "-1".into() });
        }
        if dead {
            self.kill_boss(ctx);
        }
    }
    /// Retourne vrai si le toucher a été utilisé par le boss (ou son bouton Orbe).
    fn boss_tap(&mut self, ctx: &Ctx, x: f32, y: f32) -> bool {
        if self.boss.is_none() {
            return false;
        }
        let (bx, by, br) = self.boss_pos(ctx);
        if self.is_ec() && self.orb_btn_rect(ctx).contains(x, y) {
            self.activate_ability(ctx, 1);
            return true;
        }
        if (x - bx).powi(2) + (y - by).powi(2) <= (br * 1.25).powi(2) {
            self.hit_boss(ctx);
            return true;
        }
        false
    }
    /// Dessine le boss du château `skin` (10 designs) autour de (cx, cy) ; `k` = taille d'un « point » du dessin.
    fn draw_boss_figure(&self, g: &mut Gfx, skin: u8, cx: f32, cy: f32, k: f32, al: f32) {
        use std::f32::consts::TAU;
        let t = self.anim_t;
        let p = BP { cx, cy, k, al };
        match skin {
            0 => {
                // golem de roche aux fissures de lave
                p.c(g, 0.0, 0.0, 100.0, 0xFF8C28, 0.14);
                for i in 0..4u32 {
                    let a = t * 0.8 + i as f32 * 1.57;
                    p.c(g, a.cos() * 82.0, -10.0 + a.sin() * 24.0, 7.0 + (i % 2) as f32 * 3.0, 0x5D5D66, 1.0);
                }
                p.o(g, -26.0, 46.0, 18.0, 24.0, 0x4D4D55, 1.0);
                p.o(g, 26.0, 46.0, 18.0, 24.0, 0x4D4D55, 1.0);
                p.o(g, 0.0, 4.0, 46.0, 44.0, 0x6B6B75, 1.0);
                p.o(g, -58.0, -8.0, 22.0, 24.0, 0x6B6B75, 1.0);
                p.o(g, 58.0, -8.0, 22.0, 24.0, 0x6B6B75, 1.0);
                p.o(g, -66.0, 30.0, 16.0, 22.0, 0x5D5D66, 1.0);
                p.o(g, 66.0, 30.0, 16.0, 22.0, 0x5D5D66, 1.0);
                p.o(g, 0.0, -44.0, 26.0, 22.0, 0x7A7A85, 1.0);
                let pl = 0.7 + 0.3 * (t * 3.0).sin();
                for (x0, y0, x1, y1) in [(-20.0f32, -12.0f32, -6.0f32, 6.0f32), (-6.0, 6.0, -16.0, 22.0), (18.0, -14.0, 8.0, 4.0), (8.0, 4.0, 20.0, 24.0), (-60.0, -16.0, -54.0, 2.0), (58.0, -18.0, 62.0, 0.0)] {
                    p.l(g, x0, y0, x1, y1, 2.2, 0xFF9628, pl);
                }
                p.e(g, -10.0, -46.0, 4.5, 0xFFAA3C);
                p.e(g, 10.0, -46.0, 4.5, 0xFFAA3C);
                for i in 0..6u32 {
                    p.c(g, -30.0 + i as f32 * 12.0, -26.0 + (i % 2) as f32 * 4.0 + ((i * 7) % 9) as f32, 3.0, 0x4D7A3A, 1.0);
                }
            }
            1 => {
                // gardien doré : armure, plumet, bouclier, lames
                p.c(g, 0.0, 0.0, 100.0, 0xFFD250, 0.16);
                p.l(g, -52.0, -64.0, -56.0, 6.0, 4.0, 0xFFF6C8, 1.0);
                p.l(g, 66.0, -92.0, 66.0, -8.0, 4.0, 0xFFF6C8, 1.0);
                p.r(g, 0.0, 26.0, 78.0, 62.0, 8.0, 0xE2B73C, 1.0);
                p.r(g, -11.0, 70.0, 16.0, 32.0, 4.0, 0xC9992B, 1.0);
                p.r(g, 13.0, 70.0, 16.0, 32.0, 4.0, 0xC9992B, 1.0);
                p.c(g, -46.0, -4.0, 20.0, 0xF2C94C, 1.0);
                p.c(g, 46.0, -4.0, 20.0, 0xF2C94C, 1.0);
                p.o(g, -64.0, 26.0, 22.0, 32.0, 0xF2C94C, 1.0);
                p.c(g, -64.0, 26.0, 6.0, 0xC0392B, 1.0);
                p.c(g, 0.0, -36.0, 25.0, 0xF2C94C, 1.0);
                p.s(g, -22.0, -50.0, 0.0, -78.0, 9.0, 0xD62839, 1.0);
                p.s(g, 22.0, -50.0, 0.0, -78.0, 9.0, 0xD62839, 1.0);
                p.r(g, 0.0, -36.0, 38.0, 9.0, 3.0, 0x150D00, 1.0);
                p.e(g, -8.0, -36.0, 3.0, 0xFFFFC8);
                p.e(g, 8.0, -36.0, 3.0, 0xFFFFC8);
                for i in 0..7u32 {
                    let f = (t * 0.6 + i as f32 * 0.15) % 1.0;
                    p.c(g, -50.0 + ((i * 37) % 100) as f32, -60.0 + ((i * 53) % 120) as f32, 1.8, 0xFFFFFF, (f * std::f32::consts::PI).sin());
                }
            }
            2 => {
                // colosse de cristal
                p.c(g, 0.0, 0.0, 100.0, 0x78E6FF, 0.18);
                p.s(g, -22.0, 16.0, -20.0, 72.0, 11.0, 0x7FDCF5, 1.0);
                p.s(g, 22.0, 16.0, 20.0, 72.0, 11.0, 0x7FDCF5, 1.0);
                for i in 0..9u32 {
                    let a = -1.57 + (i as f32 - 4.0) * 0.42;
                    let len = 58.0 + ((i * 37) % 24) as f32 + if i == 4 { 20.0 } else { 0.0 };
                    p.s(g, 0.0, -6.0, a.cos() * len, -6.0 + a.sin() * len * 1.05, 11.0 + (i % 3) as f32 * 3.0, if i % 2 == 0 { 0xBFEFFF } else { 0x6CC8F0 }, 1.0);
                }
                p.o(g, 0.0, -2.0, 30.0, 44.0, 0x9BE7FF, 1.0);
                p.c(g, 0.0, -8.0, 22.0, 0xE6FBFF, 0.8);
                p.e(g, -9.0, -10.0, 4.0, 0xFFFFFF);
                p.e(g, 9.0, -10.0, 4.0, 0xFFFFFF);
                for i in 0..8u32 {
                    let f = (t * 0.5 + i as f32 * 0.13) % 1.0;
                    let (sx, sy) = (-70.0 + ((i * 61) % 140) as f32, -50.0 + ((i * 29) % 100) as f32);
                    let a = (f * std::f32::consts::PI).sin();
                    p.l(g, sx - 5.0, sy, sx + 5.0, sy, 1.2, 0xFFFFFF, a);
                    p.l(g, sx, sy - 5.0, sx, sy + 5.0, 1.2, 0xFFFFFF, a);
                }
            }
            3 => {
                // brute de netherite : armure noire, cornes, coutures de lave
                p.c(g, 0.0, 0.0, 100.0, 0xFF5A14, 0.18);
                p.r(g, 0.0, 26.0, 78.0, 62.0, 8.0, 0x1B1622, 1.0);
                p.r(g, -13.0, 70.0, 18.0, 30.0, 4.0, 0x15111B, 1.0);
                p.r(g, 13.0, 70.0, 18.0, 30.0, 4.0, 0x15111B, 1.0);
                p.c(g, -50.0, -8.0, 21.0, 0x241C2E, 1.0);
                p.c(g, 50.0, -8.0, 21.0, 0x241C2E, 1.0);
                p.s(g, -60.0, -20.0, -54.0, -56.0, 11.0, 0x241C2E, 1.0);
                p.s(g, 60.0, -20.0, 54.0, -56.0, 11.0, 0x241C2E, 1.0);
                p.o(g, -68.0, 32.0, 15.0, 22.0, 0x241C2E, 1.0);
                p.o(g, 68.0, 32.0, 15.0, 22.0, 0x241C2E, 1.0);
                p.s(g, -14.0, -52.0, -54.0, -60.0, 8.0, 0x2A2136, 1.0);
                p.s(g, -54.0, -60.0, -50.0, -104.0, 8.0, 0x2A2136, 1.0);
                p.s(g, 14.0, -52.0, 54.0, -60.0, 8.0, 0x2A2136, 1.0);
                p.s(g, 54.0, -60.0, 50.0, -104.0, 8.0, 0x2A2136, 1.0);
                p.c(g, 0.0, -38.0, 25.0, 0x2A2136, 1.0);
                let pl = 0.65 + 0.35 * (t * 3.0).sin();
                for (x0, y0, x1, y1) in [(-22.0f32, 6.0f32, -8.0f32, 20.0f32), (-8.0, 20.0, -24.0, 44.0), (20.0, 4.0, 6.0, 22.0), (6.0, 22.0, 22.0, 46.0), (0.0, -4.0, 0.0, 30.0)] {
                    p.l(g, x0, y0, x1, y1, 2.2, 0xFF6E1E, pl);
                }
                p.e(g, -9.0, -38.0, 4.0, 0xFF821E);
                p.e(g, 9.0, -38.0, 4.0, 0xFF821E);
                for i in 0..14u32 {
                    let f = (t * 0.35 + i as f32 * 0.173) % 1.0;
                    p.c(g, -55.0 + ((i * 47) % 110) as f32, 60.0 - f * 150.0, 1.5 + (i % 3) as f32, 0xFF8C28, 1.0 - f);
                }
            }
            4 | 5 => {
                // titan stellaire (5 = cosmique doré, avec anneau et couronne de rayons)
                let gd = skin == 5;
                let (c1, c2) = if gd { (0xFFC850, 0xFF5AC8) } else { (0xAA6EFF, 0x5A96FF) };
                p.c(g, 0.0, 0.0, 100.0, c1, 0.2);
                p.o(g, -56.0, 8.0, 16.0, 40.0, c2, 0.8);
                p.o(g, 56.0, 8.0, 16.0, 40.0, c2, 0.8);
                p.o(g, 0.0, 16.0, 46.0, 58.0, c2, 0.95);
                p.o(g, 0.0, 22.0, 30.0, 40.0, c1, 0.5);
                p.c(g, 0.0, -40.0, 28.0, c2, 1.0);
                p.c(g, 0.0, -40.0, 20.0, 0x1E0A46, 0.7);
                for i in 0..22u32 {
                    let sx = -40.0 + ((i * 43) % 80) as f32;
                    let sy = -60.0 + ((i * 29) % 110) as f32;
                    p.c(g, sx, sy, 1.4 + (i % 3) as f32 * 0.5, 0xFFFFFF, 0.5 + 0.5 * (t * 2.0 + i as f32).sin());
                }
                if gd {
                    for i in 0..44u32 {
                        let a = i as f32 / 44.0 * TAU;
                        let (ex, ey) = (a.cos() * 84.0, a.sin() * 14.0);
                        p.c(g, ex * 0.94 + ey * 0.34, 14.0 + ey * 0.94 - ex * 0.34, 3.0, 0xFFD76E, 0.9);
                    }
                    for i in 0..9u32 {
                        let a = -2.6 + i as f32 * 0.33;
                        p.s(g, a.cos() * 34.0, -40.0 + a.sin() * 34.0, a.cos() * 56.0, -40.0 + a.sin() * 56.0, 3.0, 0xFFE182, 0.9);
                    }
                    for (x, y, r) in [(-13.0f32, -46.0f32, 4.0f32), (0.0, -52.0, 4.5), (13.0, -46.0, 4.0), (-7.0, -38.0, 2.5), (7.0, -38.0, 2.5)] {
                        p.e(g, x, y, r, 0xFFF0A0);
                    }
                } else {
                    for (x, y, r) in [(-12.0f32, -44.0f32, 4.0f32), (12.0, -44.0, 4.0), (0.0, -34.0, 3.0)] {
                        p.e(g, x, y, r, 0xC8E6FF);
                    }
                }
            }
            6 => {
                // dévoreur d'horizon : sphère noire, disque d'accrétion, tentacules, œil
                p.c(g, 0.0, 0.0, 100.0, 0xFF8C32, 0.18);
                for i in 0..8u32 {
                    let a = i as f32 / 8.0 * TAU + t * 0.3;
                    let len = 72.0 + 8.0 * (t * 2.0 + i as f32).sin();
                    let a2 = a + 0.2 + (t * 2.0 + i as f32).sin() * 0.3;
                    let (mx, my) = ((a + 0.5).cos() * len * 0.8, (a + 0.5).sin() * len * 0.8);
                    p.s(g, a.cos() * 40.0, a.sin() * 40.0, mx, my, 7.0, 0x120807, 1.0);
                    p.s(g, mx, my, a2.cos() * len, a2.sin() * len, 5.0, 0x120807, 1.0);
                }
                for i in 0..50u32 {
                    let a = i as f32 / 50.0 * TAU;
                    let (ex, ey) = (a.cos() * 88.0, a.sin() * 22.0);
                    p.c(g, ex * 0.955 + ey * 0.296, ey * 0.955 - ex * 0.296, 3.5, 0xFFAA50, 0.85);
                }
                p.c(g, 0.0, 0.0, 50.0, 0xFFC878, 1.0);
                p.c(g, 0.0, 0.0, 46.0, 0x070302, 1.0);
                p.e(g, 0.0, 0.0, 11.0, 0xFF7828);
                p.r(g, 0.0, 0.0, 6.0, 22.0, 2.0, 0x000000, 1.0);
            }
            7 => {
                // envahisseur : trois yeux, antennes, tentacules
                p.c(g, 0.0, 0.0, 100.0, 0x3CFF8C, 0.16);
                for i in 0..4u32 {
                    let sw = (t * 2.0 + i as f32).sin() * 8.0;
                    p.s(g, -18.0 + i as f32 * 12.0, 34.0, -40.0 + i as f32 * 26.0 + sw * 1.5, 84.0, 6.0, 0x1D8A4A, 1.0);
                }
                p.r(g, 0.0, 20.0, 62.0, 40.0, 6.0, 0x34495A, 1.0);
                p.l(g, -14.0, 4.0, -8.0, 36.0, 2.0, 0x3CFF8C, 0.9);
                p.l(g, 14.0, 4.0, 8.0, 36.0, 2.0, 0x3CFF8C, 0.9);
                p.o(g, -38.0, 2.0, 16.0, 12.0, 0x4B6275, 1.0);
                p.o(g, 38.0, 2.0, 16.0, 12.0, 0x4B6275, 1.0);
                for sx in [-1.0f32, 1.0] {
                    p.l(g, sx * 16.0, -62.0, sx * 40.0, -84.0 + (t * 3.0).sin() * 3.0, 2.0, 0x1D8A4A, 1.0);
                    p.l(g, sx * 40.0, -84.0, sx * 48.0, -96.0, 2.0, 0x1D8A4A, 1.0);
                    p.e(g, sx * 48.0, -96.0, 4.0, 0x96FFBE);
                }
                p.o(g, 0.0, -36.0, 40.0, 50.0, 0x2FBF6B, 1.0);
                p.o(g, -12.0, -48.0, 22.0, 28.0, 0x8DFFB0, 0.4);
                p.o(g, -17.0, -34.0, 12.0, 19.0, 0x050A06, 1.0);
                p.o(g, 17.0, -34.0, 12.0, 19.0, 0x050A06, 1.0);
                p.e(g, -17.0, -36.0, 3.5, 0x78FFAA);
                p.e(g, 17.0, -36.0, 3.5, 0x78FFAA);
                p.e(g, 0.0, -60.0, 4.5, 0x78FFAA);
                p.l(g, -32.0, -60.0, -6.0, -48.0, 3.0, 0x052314, 1.0);
                p.l(g, 32.0, -60.0, 6.0, -48.0, 3.0, 0x052314, 1.0);
                for i in 0..3u32 {
                    p.s(g, -9.0 + i as f32 * 9.0, -12.0, -7.0 + i as f32 * 9.0, -4.0, 3.0, 0xEAFFF1, 1.0);
                }
            }
            8 => {
                // roi des malédictions (design original) : couronne de cornes, quatre yeux, crocs, flammes spectrales
                for (i, y) in [-84.0f32, -50.0, -16.0, 18.0].iter().enumerate() {
                    p.r(g, 0.0, *y + (t + i as f32).sin() * 3.0, 200.0, 12.0, 0.0, 0xC82846, 0.2);
                }
                p.c(g, 0.0, 0.0, 100.0, 0xE63250, 0.14);
                p.c(g, 0.0, 20.0, 90.0, 0x28E6D2, 0.12);
                p.r(g, 0.0, 30.0, 110.0, 88.0, 14.0, 0x0C1217, 1.0);
                p.r(g, 0.0, 8.0, 60.0, 34.0, 8.0, 0x10181E, 1.0);
                p.o(g, -46.0, -6.0, 22.0, 16.0, 0x16222B, 1.0);
                p.o(g, 46.0, -6.0, 22.0, 16.0, 0x16222B, 1.0);
                for i in 0..5u32 {
                    let fx = -52.0 + i as f32 * 26.0;
                    let h = 34.0 + (i % 2) as f32 * 10.0 + (t * 6.0 + i as f32).sin() * 6.0;
                    p.s(g, fx, 74.0, fx + (t * 4.0 + i as f32).sin() * 8.0, 74.0 - h, 10.0, 0x28E6D2, 0.8);
                }
                p.l(g, -30.0, 20.0, -14.0, 30.0, 1.6, 0x25D8C8, 0.9);
                p.l(g, -14.0, 30.0, -24.0, 52.0, 1.6, 0x25D8C8, 0.9);
                p.l(g, 30.0, 20.0, 14.0, 30.0, 1.6, 0x25D8C8, 0.9);
                p.l(g, 14.0, 30.0, 24.0, 52.0, 1.6, 0x25D8C8, 0.9);
                for sx in [-1.0f32, 1.0] {
                    p.s(g, sx * 52.0, 2.0, sx * 62.0, 42.0, 8.0, 0x0C1318, 1.0);
                    for q in 0..4u32 {
                        p.s(g, sx * (60.0 + q as f32 * 4.0), 42.0, sx * (62.0 + q as f32 * 5.0 + q as f32), 66.0 + q as f32 * 3.0, 3.0, 0x06080A, 1.0);
                    }
                }
                let ry = 14.0 + (t * 2.0).sin() * 5.0;
                p.c(g, -76.0, ry, 30.0, 0x28E6D2, 0.3);
                p.c(g, -76.0, ry, 11.0, 0xFF4A6A, 1.0);
                p.c(g, -76.0, ry, 5.0, 0xFFFFFF, 1.0);
                for q in 0..3u32 {
                    p.s(g, -84.0 + q as f32 * 8.0, ry - 4.0, -84.0 + q as f32 * 8.0 + (t * 4.0 + q as f32).sin() * 3.0, ry - 26.0, 5.0, 0x28E6D2, 0.8);
                }
                for (i, (sx, sc)) in [(-1.0f32, 1.0f32), (1.0, 1.0), (-1.0, 0.62), (1.0, 0.62)].iter().enumerate() {
                    p.s(g, sx * 12.0, -48.0, sx * (30.0 + i as f32 * 4.0) * sc, -(98.0 - i as f32 * 5.0) * sc, 9.0 * sc, 0x0A0E12, 1.0);
                    p.l(g, sx * 12.0, -48.0, sx * (30.0 + i as f32 * 4.0) * sc, -(98.0 - i as f32 * 5.0) * sc, 1.4, 0xE23A56, 0.8);
                }
                p.o(g, 0.0, -38.0, 26.0, 28.0, 0x3A444C, 1.0);
                p.l(g, -8.0, -60.0, 0.0, -52.0, 1.4, 0x25D8C8, 0.9);
                p.l(g, 0.0, -52.0, 8.0, -60.0, 1.4, 0x25D8C8, 0.9);
                p.l(g, -22.0, -40.0, -12.0, -34.0, 1.4, 0x25D8C8, 0.9);
                p.l(g, 22.0, -40.0, 12.0, -34.0, 1.4, 0x25D8C8, 0.9);
                p.e(g, -11.0, -44.0, 4.2, 0xFF3C5A);
                p.e(g, 11.0, -44.0, 4.2, 0xFF3C5A);
                p.e(g, -8.0, -34.0, 2.8, 0xFF788C);
                p.e(g, 8.0, -34.0, 2.8, 0xFF788C);
                p.r(g, 0.0, -21.0, 34.0, 7.0, 2.0, 0x0A0E12, 1.0);
                for i in 0..8u32 {
                    p.s(g, -14.0 + i as f32 * 4.0, -24.0, -14.0 + i as f32 * 4.0, -18.0 + (i % 2) as f32 * 4.0, 2.4, 0xFFFFFF, 1.0);
                }
                for i in 0..14u32 {
                    let f = (t * 0.35 + i as f32 * 0.17) % 1.0;
                    p.c(g, -60.0 + ((i * 47) % 120) as f32, 70.0 - f * 150.0, 1.5 + (i % 3) as f32, 0x28E6D2, 1.0 - f);
                }
            }
            11 => {
                // seigneur solaire : couronne de rayons, armure en fusion, mains de flammes
                p.c(g, 0.0, 0.0, 100.0, 0xFFAA28, 0.2);
                for i in 0..16u32 {
                    let a = i as f32 / 16.0 * TAU + t * 0.4;
                    let len = 50.0 + 10.0 * (t * 3.0 + i as f32).sin();
                    p.s(g, a.cos() * 24.0, -40.0 + a.sin() * 24.0, a.cos() * len, -40.0 + a.sin() * len, 6.0, if i % 2 == 0 { 0xFFD050 } else { 0xFF9A30 }, 0.9);
                }
                p.r(g, 0.0, 24.0, 82.0, 64.0, 8.0, 0x3A0F00, 1.0);
                p.r(g, 0.0, 24.0, 78.0, 60.0, 8.0, 0xF08A20, 1.0);
                p.r(g, -2.0, 18.0, 52.0, 40.0, 8.0, 0xFFC850, 0.7);
                p.r(g, -13.0, 68.0, 16.0, 28.0, 4.0, 0xC85A10, 1.0);
                p.r(g, 13.0, 68.0, 16.0, 28.0, 4.0, 0xC85A10, 1.0);
                p.o(g, -48.0, -6.0, 20.0, 17.0, 0xE07018, 1.0);
                p.o(g, 48.0, -6.0, 20.0, 17.0, 0xE07018, 1.0);
                p.s(g, -52.0, 2.0, -64.0, 36.0, 7.0, 0x5A1A00, 1.0);
                p.s(g, 52.0, 2.0, 64.0, 36.0, 7.0, 0x5A1A00, 1.0);
                for q in 0..3u32 {
                    let fl = 18.0 + (q % 2) as f32 * 8.0 + (t * 6.0 + q as f32).sin() * 4.0;
                    p.s(g, -70.0 + q as f32 * 6.0, 40.0, -70.0 + q as f32 * 6.0 + (t * 4.0 + q as f32).sin() * 3.0, 40.0 - fl, 5.0, 0xFFC850, 0.85);
                    p.s(g, 58.0 + q as f32 * 6.0, 40.0, 58.0 + q as f32 * 6.0 + (t * 4.0 + q as f32 + 5.0).sin() * 3.0, 40.0 - fl, 5.0, 0xFFC850, 0.85);
                }
                p.l(g, -18.0, 4.0, -6.0, 20.0, 1.6, 0x781400, 0.8);
                p.l(g, -6.0, 20.0, -16.0, 40.0, 1.6, 0x781400, 0.8);
                p.l(g, 16.0, 2.0, 6.0, 22.0, 1.6, 0x781400, 0.8);
                p.l(g, 6.0, 22.0, 18.0, 42.0, 1.6, 0x781400, 0.8);
                p.c(g, 0.0, -40.0, 26.0, 0x5A1A00, 1.0);
                p.c(g, 0.0, -40.0, 24.0, 0xFF7A1C, 1.0);
                p.c(g, -3.0, -43.0, 17.0, 0xFFD84A, 1.0);
                p.c(g, -5.0, -46.0, 8.0, 0xFFFBE0, 1.0);
                p.e(g, -8.0, -42.0, 3.5, 0xFFFFFF);
                p.e(g, 8.0, -42.0, 3.5, 0xFFFFFF);
                p.r(g, 0.0, -29.0, 16.0, 3.0, 1.0, 0x5A1A00, 1.0);
                for i in 0..14u32 {
                    let f = (t * 0.35 + i as f32 * 0.173) % 1.0;
                    p.c(g, -55.0 + ((i * 47) % 110) as f32, 60.0 - f * 150.0, 1.5 + (i % 3) as f32, 0xFFBE50, 1.0 - f);
                }
            }
            _ => {
                // gardien du temps : cadran, engrenages, sablier
                p.c(g, 0.0, 0.0, 100.0, 0x3FE0C8, 0.16);
                for (gx, gy, gr, tn, sp, col) in [(-62.0f32, -8.0f32, 24.0f32, 10u32, 0.8f32, 0xB8892Fu32), (62.0, -8.0, 24.0, 10, -0.8, 0xB8892F), (-52.0, 52.0, 16.0, 8, -1.2, 0x8C6620), (52.0, 52.0, 16.0, 8, 1.2, 0x8C6620)] {
                    p.c(g, gx, gy, gr * 0.8, col, 1.0);
                    for q in 0..tn {
                        let a = t * sp + q as f32 * TAU / tn as f32;
                        p.c(g, gx + a.cos() * gr, gy + a.sin() * gr, gr * 0.2, col, 1.0);
                    }
                }
                p.r(g, 0.0, 26.0, 78.0, 70.0, 8.0, 0x0A2A30, 1.0);
                p.c(g, 0.0, 22.0, 33.0, 0xD9B25A, 1.0);
                p.c(g, 0.0, 22.0, 30.0, 0xFFF4D0, 1.0);
                for i in 0..12u32 {
                    let a = i as f32 / 12.0 * TAU;
                    let r0 = if i % 3 == 0 { 22.0 } else { 26.0 };
                    p.l(g, a.sin() * r0, 22.0 - a.cos() * r0, a.sin() * 29.0, 22.0 - a.cos() * 29.0, if i % 3 == 0 { 1.8 } else { 1.0 }, 0x5A3F0A, 1.0);
                }
                let (ha, ma) = (t * 0.5, t * 3.0);
                p.l(g, 0.0, 22.0, ha.sin() * 15.0, 22.0 - ha.cos() * 15.0, 2.2, 0x3A2A08, 1.0);
                p.l(g, 0.0, 22.0, ma.sin() * 24.0, 22.0 - ma.cos() * 24.0, 1.4, 0x0F9A8A, 1.0);
                p.c(g, 0.0, 22.0, 3.0, 0x3A2A08, 1.0);
                p.s(g, -22.0, -70.0, 0.0, -40.0, 8.0, 0x3FE0C8, 0.5);
                p.s(g, 22.0, -70.0, 0.0, -40.0, 8.0, 0x3FE0C8, 0.5);
                p.s(g, -22.0, -10.0, 0.0, -40.0, 8.0, 0x3FE0C8, 0.5);
                p.s(g, 22.0, -10.0, 0.0, -40.0, 8.0, 0x3FE0C8, 0.5);
                p.s(g, -18.0, -14.0, 0.0, -34.0, 7.0, 0xFFD98A, 1.0);
                p.s(g, 18.0, -14.0, 0.0, -34.0, 7.0, 0xFFD98A, 1.0);
                p.o(g, 0.0, -74.0, 28.0, 6.0, 0xF0C060, 1.0);
                p.o(g, 0.0, -8.0, 28.0, 6.0, 0xF0C060, 1.0);
                p.e(g, -12.0, -42.0, 3.0, 0x3FE0C8);
                p.e(g, 12.0, -42.0, 3.0, 0x3FE0C8);
                for i in 0..10u32 {
                    let f = (t * 0.5 + i as f32 * 0.1) % 1.0;
                    p.c(g, -60.0 + ((i * 67) % 120) as f32, -70.0 + f * 150.0, 1.8, 0xFFD98A, 1.0 - f);
                }
            }
        }
    }

    fn draw_boss(&self, ctx: &Ctx, g: &mut Gfx) {
        use std::f32::consts::TAU;
        let s = ctx.scale();
        let t = self.anim_t;
        let l = self.layout(ctx);
        let w = ctx.width();
        if self.boss.is_some() {
            // vignette rouge qui pulse
            let a = 0.16 + 0.1 * (t * 4.5).sin();
            g.rect(0.0, l.head_h, 12.0 * s, l.shop_top - l.head_h, rgba(0xFF1E3C, a));
            g.rect(w - 12.0 * s, l.head_h, 12.0 * s, l.shop_top - l.head_h, rgba(0xFF1E3C, a));
            g.rect(0.0, l.head_h, w, 8.0 * s, rgba(0xFF1E3C, a * 0.8));
        }
        let (b, dp) = if let Some(b) = &self.boss {
            (b, 0.0f32)
        } else if let Some((b, tt)) = &self.boss_dying {
            (b, (tt / 0.9).min(1.0))
        } else {
            return;
        };
        let (bx, by, br) = self.boss_pos_side(ctx, b.side);
        let a = (b.age / 0.7).clamp(0.0, 1.0);
        let sc = if a < 1.0 { 0.35 + 0.65 * (1.0 - (1.0 - a).powi(3)) + (a * std::f32::consts::PI).sin() * 0.08 } else { 1.0 };
        let al = if dp > 0.0 { 1.0 - dp } else { a };
        let bob = (t * 1.6).sin() * 3.0 * s;
        let (mut dx, mut dy) = (0.0, 0.0);
        if b.kb > 0.0 {
            dx = (hash01((t * 60.0) as u32, 181) - 0.5) * 8.0 * s * b.kb;
            dy = (hash01((t * 60.0) as u32, 182) - 0.5) * 5.0 * s * b.kb;
        }
        let k = 0.4 * s * sc * (1.0 + dp * 0.25);
        let (cx, cy) = (bx + dx, by + bob + dy);
        self.draw_boss_figure(g, b.skin, cx, cy, k, al);
        if b.max > 0 && (b.hp as f32 / b.max as f32) < 0.35 {
            g.circle(cx, cy, 52.0 * k, rgba(0xFF2828, (0.12 + 0.1 * (t * 8.0).sin()) * al));
        }
        if b.fl > 0.0 {
            g.circle(cx, cy, 40.0 * k, rgba(0xFFFFFF, b.fl * 0.45));
        }
        if dp > 0.0 {
            for i in 0..3u32 {
                let rr = dp * (50.0 + i as f32 * 30.0) * 0.55 * s;
                for q in 0..48u32 {
                    let an = q as f32 / 48.0 * TAU;
                    g.circle(cx + an.cos() * rr, cy + an.sin() * rr, (3.0 - i as f32 * 0.7) * s, rgba(0xFFE6A0, 1.0 - dp));
                }
            }
            for i in 0..26u32 {
                let an = i as f32 * 2.4;
                let rr = dp * (30.0 + ((i * 37) % 70) as f32) * 0.55 * s;
                g.circle(cx + an.cos() * rr, cy + an.sin() * rr, (2.0 + (i % 3) as f32) * s, rgba(0xFFB478, 1.0 - dp));
            }
            return;
        }
        // barre de vie + nom + bouton Orbe
        let bw = 70.0 * s;
        let by2 = by + br + 4.0 * s;
        g.rrect(bx - bw / 2.0 - 1.0, by2 - 1.0, bw + 2.0, 8.0 * s + 2.0, 4.0 * s, rgba(0xFFFFFF, 0.35));
        g.rrect(bx - bw / 2.0, by2, bw, 8.0 * s, 4.0 * s, rgba(0x000000, 0.7));
        let f = b.hp as f32 / b.max.max(1) as f32;
        if f > 0.0 {
            g.rrect(bx - bw / 2.0, by2, (bw * f).max(6.0 * s), 8.0 * s, 4.0 * s, rgb(mix_color(0xFF2848, 0xFF9A3C, f)));
        }
        self.text_outline(ctx, g, &format!("{} · Nv {}", BOSS_NAMES[(b.skin as usize).min(11)], b.lvl), bx, by - br - 9.0 * s, 8.0 * s, 0xFFFFFF, 1.0, 126.0 * s);
        self.text_outline(ctx, g, &format!("{} / {}", b.hp, b.max), bx, by2 + 14.0 * s, 9.0 * s, 0xFFFFFF, 1.0, 80.0 * s);
        if self.is_ec() {
            let r = self.orb_btn_rect(ctx);
            g.rrect(r.x - 2.0, r.y - 2.0, r.w + 4.0, r.h + 4.0, 9.0 * s, rgba(0xC77DFF, 0.3));
            g.rrect(r.x, r.y, r.w, r.h, 8.0 * s, rgb(0x7B3BE0));
            ctx.text_mid(g, "Orbe violette", r.x + r.w / 2.0, r.y + r.h / 2.0, &Text::new(10.0 * s).bold().color(rgb(0xFFFFFF)).center().max_width(r.w - 4.0));
        }
    }

    /// Chrono avant le prochain boss (ou alerte), et bannière d'apparition.
    fn draw_boss_hud(&self, ctx: &Ctx, g: &mut Gfx) {
        let s = ctx.scale();
        let l = self.layout(ctx);
        let (w, h) = (ctx.width(), ctx.height());
        if self.boss.is_some() {
            let blink = 0.6 + 0.4 * (self.anim_t * 6.0).sin();
            ctx.text_mid(g, if self.boss.as_ref().map_or(false, |b| b.test) { "BOSS DE TEST · aucun butin" } else { "BOSS EN COURS · touche-le !" }, 6.0 * s, l.shop_top - 8.0 * s, &Text::new(9.5 * s).bold().color(rgba(0xFF5A6E, blink)).left().max_width(w * 0.62));
        } else if self.boss_next > 0.0 {
            let left = (self.boss_next - self.clock).max(0.0);
            ctx.text_mid(g, &format!("Prochain boss dans {}", fmt_t(left)), 6.0 * s, l.shop_top - 8.0 * s, &Text::new(9.5 * s).bold().color(rgba(0xFFFFFF, 0.75)).left().max_width(w * 0.62));
        }
        if self.def_g > 0 {
            let attack = !self.mons.is_empty() || !self.mon_q.is_empty();
            let txt = if attack { format!("ATTAQUE : {} !", self.mons.len() + self.mon_q.len()) } else { format!("Vague dans {}", fmt_t((self.def_next - self.clock).max(0.0))) };
            ctx.text_mid(g, &txt, w - 6.0 * s, l.shop_top - 8.0 * s, &Text::new(9.5 * s).bold().color(if attack { rgb(0xFF5A6E) } else { rgba(0xFFFFFF, 0.75) }).right().max_width(w * 0.36));
        }
        if self.boss_ban > 0.0 {
            let p = (self.boss_ban / 2.4).clamp(0.0, 1.0);
            let age = 1.0 - p;
            let pop = if age < 0.12 { 2.2 - 1.2 * (age / 0.12) } else { 1.0 };
            let al = if age < 0.12 { age / 0.12 } else if p < 0.2 { p / 0.2 } else { 1.0 };
            let name = self.boss.as_ref().map_or("", |b| BOSS_NAMES[(b.skin as usize).min(11)]);
            self.text_outline(ctx, g, &format!("BOSS · {}", name), w / 2.0, h * 0.2, (w * 0.075).min(32.0 * s) * pop, 0xFF3C50, al, w - 6.0);
        }
    }

    // ---- château Solaire --------------------------------------------------------------------
    fn sun_on(&self) -> bool {
        self.skin == 11 && self.clock < self.sun_until
    }
    /// Soleil : couronne de rayons, disque dégradé, granulation, taches ; devant : protubérances et étincelles en orbite.
    fn draw_solar(&self, g: &mut Gfx, cx: f32, cy: f32, r: f32, front: bool) {
        use std::f32::consts::TAU;
        let t = self.anim_t;
        let on = self.sun_on();
        let k = if on { 1.35 } else { 1.0 };
        let sr = r * 0.62;
        if !front {
            fx_glow(g, cx, cy, r * 1.3 * k, 0xFFB43C, if on { 0.7 } else { 0.5 });
            for i in 0..36u32 {
                let a = i as f32 / 36.0 * TAU + t * 0.15 * if i % 2 == 1 { 1.0 } else { -1.0 };
                let len = r * (0.85 + 0.35 * (0.5 + 0.5 * (t * 2.0 + i as f32 * 1.7).sin())) * k;
                fx_taper(g, cx + a.cos() * sr * 0.95, cy + a.sin() * sr * 0.95, cx + a.cos() * len, cy + a.sin() * len, r * 0.055, if i % 3 == 0 { 0xFFE07A } else { 0xFF9A30 }, 0.7);
            }
            for (f, c) in [(1.0f32, 0xC43800u32), (0.93, 0xFF6A10), (0.8, 0xFF8A1C), (0.63, 0xFFB030), (0.46, 0xFFD84A), (0.3, 0xFFF0A0), (0.15, 0xFFFBE0)] {
                g.circle(cx - sr * 0.2 * (1.0 - f), cy - sr * 0.25 * (1.0 - f), sr * f, rgb(c));
            }
            for i in 0..60u32 {
                let a = hash01(i, 501) * TAU + t * 0.05;
                let d = hash01(i + 7, 502).sqrt() * sr * 0.9;
                g.circle(cx + a.cos() * d, cy + a.sin() * d, sr * (0.04 + 0.04 * hash01(i + 3, 503)), rgba(0xFFF8C8, 0.16 + 0.12 * (t * 3.0 + i as f32).sin()));
            }
            for i in 0..3u32 {
                let a = t * 0.12 + i as f32 * 2.3;
                let d = sr * (0.35 + 0.15 * i as f32);
                g.circle(cx + a.cos() * d, cy + a.sin() * d * 0.85, sr * 0.06, rgba(0x5A1900, 0.55));
                g.circle(cx + a.cos() * d, cy + a.sin() * d * 0.85, sr * 0.03, rgba(0x320A00, 0.7));
            }
            if on {
                fx_glow(g, cx, cy, sr * 1.3, 0xFFFFF0, 0.5 * (0.5 + 0.5 * (t * 6.0).sin()));
            }
        } else {
            for i in 0..3u32 {
                let a0 = t * 0.25 + i as f32 * 2.09;
                let p = 0.6 + 0.4 * (t * 1.3 + i as f32 * 2.0).sin();
                let (x0, y0, x1, y1) = (cx + a0.cos() * sr, cy + a0.sin() * sr, cx + (a0 + 0.55).cos() * sr, cy + (a0 + 0.55).sin() * sr);
                let (qx, qy) = (cx + (a0 + 0.27).cos() * sr * (1.45 + 0.25 * p), cy + (a0 + 0.27).sin() * sr * (1.45 + 0.25 * p));
                for q in 0..=24u32 {
                    let u = q as f32 / 24.0;
                    let (bx, by) = ((1.0 - u).powi(2) * x0 + 2.0 * u * (1.0 - u) * qx + u * u * x1, (1.0 - u).powi(2) * y0 + 2.0 * u * (1.0 - u) * qy + u * u * y1);
                    g.circle(bx, by, r * 0.035, rgba(0xFF6E28, 0.7));
                    g.circle(bx, by, r * 0.013, rgba(0xFFDC8C, 0.8));
                }
            }
            for i in 0..3u32 {
                let b = t * 0.9 + i as f32 * 2.09;
                fx_glow(g, cx + b.cos() * r * 1.3, cy + b.sin() * r * 1.3, r * 0.12, 0xFFF0B0, 0.9);
            }
        }
    }

    /// Éclat du soleil (≈ 4,2 s).
    fn draw_sunfx(&mut self, ctx: &Ctx, g: &mut Gfx) {
        use std::f32::consts::TAU;
        let t = self.sun_t;
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let (cx, cy, m) = (w / 2.0, h * 0.45, w.min(h));
        let fade = if t < 0.3 { 1.0 - t / 0.3 } else if t > 3.6 { ((t - 3.6) / 0.6).min(1.0) } else { 0.0 };
        if fade > 0.0 {
            self.draw_game(ctx, g);
            g.rect(0.0, 0.0, w, h, rgba(0x1A0500, 1.0 - fade));
        } else {
            g.clear(rgb(0x1A0500));
        }
        let gr = ez(t / 1.5);
        let sr = m * (0.08 + 0.14 * gr);
        let br = ez((t - 1.5) / 0.25) * (1.0 - ez((t - 2.2) / 1.2));
        fx_glow(g, cx, cy, sr * 3.5, 0xFFDC8C, 0.8);
        for i in 0..40u32 {
            let a = i as f32 / 40.0 * TAU + t * 0.35 * if i % 2 == 1 { 1.0 } else { -1.0 };
            let len = sr * (1.6 + 1.6 * gr + 0.6 * (t * 4.0 + i as f32).sin()) * (1.0 + br * 2.0);
            fx_taper(g, cx + a.cos() * sr, cy + a.sin() * sr, cx + a.cos() * len, cy + a.sin() * len, sr * 0.1, 0xFFE178, 0.7);
        }
        for (f, c) in [(1.0f32, 0xFF7A1Cu32), (0.8, 0xFFA830), (0.6, 0xFFE066), (0.35, 0xFFF6C0), (0.18, 0xFFFDE8)] {
            g.circle(cx - sr * 0.2 * (1.0 - f), cy - sr * 0.25 * (1.0 - f), sr * f, rgb(c));
        }
        if t > 1.5 {
            for i in 0..4u32 {
                let rp = ((t - 1.5) * 0.6 + i as f32 * 0.25) % 1.0;
                fx_ring(g, cx, cy, sr + rp * m * 0.8, 90, 2.5 * s, 0xFFF0BE, 0.6 * (1.0 - rp));
            }
            for i in 0..6u32 {
                let u = i as f32 / 5.0;
                g.circle(cx + (w * 0.1 - cx) * -u * 0.8, cy + (h * 0.2 - cy) * -u * 0.8, m * (0.02 + 0.03 * (i % 3) as f32), rgba(0xFFD08C, 0.25));
            }
        }
        if br > 0.0 {
            g.rect(0.0, 0.0, w, h, rgba(0xFFFCEB, 0.9 * br));
        }
        if t > 0.6 {
            let tp = ez((t - 0.6) / 0.3);
            let al = tp * if t > 3.6 { (1.0 - (t - 3.6) / 0.5).max(0.0) } else { 1.0 };
            let fs = (w * 0.095).min(44.0 * s);
            self.text_outline(ctx, g, "ÉCLAT DU SOLEIL", cx, h * 0.12 + fs * 0.3, fs * (1.5 - 0.5 * tp), 0xFFF3B0, al, w - 8.0);
            if t > 2.0 {
                self.text_outline(ctx, g, "Château ×1 000 000 pendant 3 min", cx, h * 0.8, (w * 0.05).min(20.0 * s), 0xFFFFFF, ez((t - 2.0) / 0.3) * al, w - 8.0);
            }
        }
    }

    /// Déblocage du château Solaire (≈ 4,6 s) : étincelle, matière en spirale, allumage du soleil.
    fn draw_solarfx(&mut self, ctx: &Ctx, g: &mut Gfx) {
        use std::f32::consts::TAU;
        let t = self.solar_t;
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let (cx, cy, m) = (w / 2.0, h * 0.45, w.min(h));
        let fade = if t < 0.3 { 1.0 - t / 0.3 } else if t > 4.0 { ((t - 4.0) / 0.6).min(1.0) } else { 0.0 };
        if fade > 0.0 {
            self.draw_game(ctx, g);
            g.rect(0.0, 0.0, w, h, rgba(0x050100, 1.0 - fade));
        } else {
            g.clear(rgb(0x050100));
        }
        let ig = ez((t - 1.8) / 0.4);
        let sr = m * 0.2 * ig;
        if t < 2.1 {
            for i in 0..120u32 {
                let a = hash01(i, 511) * TAU + t * (1.2 + hash01(i + 4, 512));
                let f = (t * 0.55 + hash01(i + 2, 513) * 0.4).min(1.0);
                let d = m * (0.9 - 0.85 * f);
                g.circle(cx + a.cos() * d, cy + a.sin() * d, (1.0 + hash01(i + 6, 514) * 2.0) * s * 0.8, rgba(mix_color(0xFF9646, 0xFFF0A0, hash01(i + 5, 515)), 0.3 + 0.5 * f));
            }
        }
        fx_glow(g, cx, cy, m * (0.03 + 0.08 * (t / 1.8).min(1.0)), 0xFFFFF0, 1.0);
        if ig > 0.0 {
            fx_glow(g, cx, cy, sr * 3.0, 0xFFE696, 0.7);
            for i in 0..36u32 {
                let a = i as f32 / 36.0 * TAU + t * 0.2;
                let len = sr * (1.8 + 0.5 * (t * 3.0 + i as f32).sin());
                fx_line(g, cx + a.cos() * sr, cy + a.sin() * sr, cx + a.cos() * len, cy + a.sin() * len, 1.6 * s, 0xFFD264, 0.55);
            }
            for (f, c) in [(1.0f32, 0xD84A00u32), (0.8, 0xFF8A1C), (0.6, 0xFFB030), (0.4, 0xFFD84A), (0.2, 0xFFFDE8)] {
                g.circle(cx - sr * 0.2 * (1.0 - f), cy - sr * 0.25 * (1.0 - f), sr * f, rgb(c));
            }
            let sw = ez((t - 1.8) / 1.2);
            fx_ring(g, cx, cy, sr + sw * m * 0.9, 120, 3.5 * s, 0xFFF0BE, 1.0 - sw);
        }
        let fl = (1.0 - (t - 1.85).abs() / 0.2).max(0.0);
        if fl > 0.0 {
            g.rect(0.0, 0.0, w, h, rgba(0xFFFAE6, fl));
        }
        if t > 2.2 {
            let tp = ez((t - 2.2) / 0.3);
            let fs = (w * 0.09).min(42.0 * s);
            self.text_outline(ctx, g, "CHÂTEAU SOLAIRE", cx, h * 0.12 + fs * 0.3, fs * (1.5 - 0.5 * tp), 0xFFD84A, tp, w - 8.0);
            self.text_outline(ctx, g, "×10 milliards pour toujours", cx, h * 0.8, (w * 0.05).min(20.0 * s), 0xFFFFFF, tp, w - 8.0);
        }
    }

    // ---- Défense : gardes (un skin par château) contre des vagues de monstres ----------------
    fn g_cost(&self) -> f64 {
        (1000.0 * 3f64.powi(self.def_g as i32)).ceil()
    }
    fn gp_cost(&self) -> f64 {
        (5000.0 * 4f64.powi(self.def_pw as i32)).ceil()
    }
    fn gs_cost(&self) -> f64 {
        (8000.0 * 4f64.powi(self.def_sp as i32)).ceil()
    }
    fn gw_cost(&self) -> f64 {
        (20000.0 * 5f64.powi(self.def_wl as i32)).ceil()
    }
    fn g_dmg(&self) -> f64 {
        2f64.powi(self.def_pw as i32)
    }
    fn g_rate(&self) -> f32 {
        1.1 * 1.25f32.powi(self.def_sp as i32)
    }
    fn defense_row(&self, k: usize) -> (String, String, String, bool, bool) {
        let sk = self.skin.min(11) as usize;
        let (title, detail, lvl, max, c) = match k {
            0 => ("Recruter un garde", format!("+1 {} (8 max)", GNAME[sk]), self.def_g, self.def_g >= 8, self.g_cost()),
            1 => ("Force des gardes", format!("Dégâts ×2 (actuellement ×{})", fmt_mult(self.g_dmg())), self.def_pw, self.def_pw >= 20, self.gp_cost()),
            2 => ("Cadence de tir", format!("Tirs 25 % plus rapides ({:.1} tir/s)", self.g_rate()).replace('.', ","), self.def_sp, self.def_sp >= 10, self.gs_cost()),
            _ => ("Rempart", format!("Les monstres volent 20 % de moins (−{} %)", ((1.0 - 0.8f64.powi(self.def_wl as i32)) * 100.0).round()), self.def_wl, self.def_wl >= 5, self.gw_cost()),
        };
        let d = if max { format!("{} · MAX", detail) } else { format!("{} · coût {}", detail, fmt(c)) };
        (title.into(), d, lvl.to_string(), !max && self.pts >= c, false)
    }
    fn defense_tap(&mut self, ctx: &Ctx, k: usize) {
        let (c, max) = match k {
            0 => (self.g_cost(), self.def_g >= 8),
            1 => (self.gp_cost(), self.def_pw >= 20),
            2 => (self.gs_cost(), self.def_sp >= 10),
            3 => (self.gw_cost(), self.def_wl >= 5),
            _ => return,
        };
        if max {
            ctx.toast("Déjà au maximum");
            return;
        }
        if self.pts < c {
            ctx.toast("Pas assez de territoires");
            return;
        }
        self.pts -= c;
        match k {
            0 => {
                self.def_g += 1;
                if self.def_g == 1 {
                    self.def_next = self.clock + 20.0;
                    ctx.toast("Premier garde recruté ! Première vague de monstres dans 20 s");
                }
            }
            1 => self.def_pw += 1,
            2 => self.def_sp += 1,
            _ => self.def_wl += 1,
        }
        self.bump(ctx, 2, 1);
        self.save();
    }
    fn guard_pos(&self, l: &Layout, i: usize, n: usize) -> (f32, f32) {
        let a = -std::f32::consts::FRAC_PI_2 + (i as f32 + 0.5) / n as f32 * std::f32::consts::TAU;
        (l.cx + a.cos() * l.r * 1.55, l.cy + a.sin() * l.r * 1.08)
    }
    fn start_wave(&mut self, ctx: &Ctx) {
        self.def_wave += 1;
        let n = (3 + (self.def_wave as f32 * 0.7) as u32).min(16);
        let hp = ((3.0 + self.def_wave as f64 * 1.5) * 1.6f64.powi((self.def_wave / 5) as i32)).round();
        for i in 0..n {
            let big = self.def_wave % 5 == 0 && i == 0;
            self.mon_q.push((self.clock + i as f64 * 0.45, if big { hp * 4.0 } else { hp }, big));
        }
        self.def_next = self.clock + 75.0;
        self.save();
        ctx.toast(format!("Vague {} : {} monstres attaquent le château !", self.def_wave, n));
    }
    fn hit_mon(&mut self, idx: usize, d: f64) {
        if idx >= self.mons.len() {
            return;
        }
        let m = &mut self.mons[idx];
        m.hp -= d;
        m.fl = 1.0;
        if m.hp > 0.0 {
            return;
        }
        let m = self.mons.remove(idx);
        self.poofs.push((m.x, m.y, 0.0, false, m.big));
        let g = (self.income_ps() * 2.0).max(self.click_power() * 5.0) * if m.big { 5.0 } else { 1.0 };
        self.pts += g;
        self.total += g;
        self.def_kills += 1;
        if self.floats.len() < 10 {
            self.floats.push(Floater { x: m.x, y: m.y - 10.0, t: 0.0, txt: format!("+{}", fmt(g)) });
        }
    }
    /// Toucher un monstre le frappe (retourne vrai si un monstre a été touché).
    fn mon_tap(&mut self, ctx: &Ctx, x: f32, y: f32) -> bool {
        let s = ctx.scale();
        let mut best = None;
        let mut bd = (28.0 * s).powi(2);
        for (i, m) in self.mons.iter().enumerate() {
            let d = (m.x - x).powi(2) + (m.y - y).powi(2);
            if d < bd {
                bd = d;
                best = Some(i);
            }
        }
        if let Some(i) = best {
            let d = (self.g_dmg() / 2.0).max(1.0);
            self.hit_mon(i, d);
            ctx.animate();
            return true;
        }
        false
    }
    fn def_update(&mut self, ctx: &Ctx, dt: f32) {
        let l = self.layout(ctx);
        let s = ctx.scale();
        let w = ctx.width();
        if self.def_g > 0 && self.def_next > 0.0 && self.clock >= self.def_next && self.mon_q.is_empty() {
            self.start_wave(ctx);
        }
        let mut i = 0;
        while i < self.mon_q.len() {
            if self.clock >= self.mon_q[i].0 {
                let (_, hp, big) = self.mon_q.remove(i);
                let side = (self.rnd() * 3.0) as u32;
                let (x, y) = match side {
                    0 => (-8.0, l.head_h + (l.shop_top - l.head_h) * (0.1 + self.rnd() * 0.8)),
                    1 => (w + 8.0, l.head_h + (l.shop_top - l.head_h) * (0.1 + self.rnd() * 0.8)),
                    _ => (self.rnd() * w, l.head_h - 8.0),
                };
                self.mon_id += 1;
                let sp = (22.0 + self.rnd() * 10.0) * s * if big { 0.7 } else { 1.0 };
                let ph = self.rnd() * 6.0;
                self.mons.push(Mon { id: self.mon_id, x, y, hp, max: hp, big, fl: 0.0, ph, sp });
            } else {
                i += 1;
            }
        }
        let n = (self.def_g as usize).min(8);
        let sk = self.skin.min(11);
        while self.gcd.len() < n {
            let r = self.rnd() * 0.5;
            self.gcd.push(r);
            self.gfire.push(0.0);
        }
        for gi in 0..n {
            let (px, py) = self.guard_pos(&l, gi, n);
            self.gfire[gi] = (self.gfire[gi] - dt * 5.0).max(0.0);
            self.gcd[gi] -= dt;
            let tg = self.mons.iter().min_by(|a, b| ((a.x - px).powi(2) + (a.y - py).powi(2)).partial_cmp(&((b.x - px).powi(2) + (b.y - py).powi(2))).unwrap_or(std::cmp::Ordering::Equal));
            if let Some(m) = tg {
                if self.gcd[gi] <= 0.0 {
                    self.gcd[gi] = 1.0 / self.g_rate() * if sk == 10 { 1.6 } else { 1.0 };
                    self.gfire[gi] = 1.0;
                    let (dx, dy) = (m.x - px, m.y - py);
                    let dd = (dx * dx + dy * dy).sqrt().max(1.0);
                    let v = if sk == 10 { 260.0 } else { 380.0 } * s;
                    let id = m.id;
                    self.shots.push(Shot { x: px + 8.0 * s, y: py - 4.0 * s, vx: dx / dd * v, vy: dy / dd * v, tg: id, sk, alt: gi % 2 == 1, tr: Vec::new(), v });
                }
            }
        }
        // monstres
        let mut k = 0;
        while k < self.mons.len() {
            let m = &mut self.mons[k];
            m.fl = (m.fl - dt * 6.0).max(0.0);
            let (dx, dy) = (l.cx - m.x, l.cy - m.y);
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            if d < l.r * 0.95 {
                let big = m.big;
                let (mx, my) = (m.x, m.y);
                self.mons.remove(k);
                let st = (self.pts * 0.02).min(self.income_ps() * 30.0) * 0.8f64.powi(self.def_wl as i32) * if big { 3.0 } else { 1.0 };
                self.pts = (self.pts - st).max(0.0);
                self.shake = 0.16;
                if self.floats.len() < 10 {
                    self.floats.push(Floater { x: mx, y: my - 10.0, t: 0.0, txt: format!("−{}", fmt(st)) });
                }
                continue;
            }
            m.x += dx / d * m.sp * dt;
            m.y += dy / d * m.sp * dt;
            k += 1;
        }
        // tirs à tête chercheuse
        let mut j = 0;
        while j < self.shots.len() {
            let tgi = self.mons.iter().position(|m| m.id == self.shots[j].tg);
            let mut hit = None;
            {
                let sh = &mut self.shots[j];
                if let Some(ti) = tgi {
                    let (ex, ey) = (self.mons[ti].x - sh.x, self.mons[ti].y - sh.y);
                    let ed = (ex * ex + ey * ey).sqrt().max(1.0);
                    sh.vx = ex / ed * sh.v;
                    sh.vy = ey / ed * sh.v;
                    if ed < 10.0 * s {
                        hit = Some(ti);
                    }
                } else {
                    sh.tg = 0;
                }
                sh.tr.push((sh.x, sh.y));
                if sh.tr.len() > 10 {
                    sh.tr.remove(0);
                }
                sh.x += sh.vx * dt;
                sh.y += sh.vy * dt;
            }
            if let Some(ti) = hit {
                let sh = self.shots.remove(j);
                let dm = self.g_dmg() * if sh.sk == 10 { 3.0 } else { 1.0 };
                let tid = self.mons[ti].id;
                if sh.sk == 10 {
                    self.poofs.push((sh.x, sh.y, 0.0, true, false));
                    let near: Vec<u64> = self.mons.iter().filter(|m| m.id != tid && (m.x - sh.x).powi(2) + (m.y - sh.y).powi(2) < (34.0 * s).powi(2)).map(|m| m.id).collect();
                    for id in near {
                        if let Some(p) = self.mons.iter().position(|m| m.id == id) {
                            self.hit_mon(p, dm);
                        }
                    }
                }
                if let Some(p) = self.mons.iter().position(|m| m.id == tid) {
                    self.hit_mon(p, dm);
                }
                continue;
            }
            let sh = &self.shots[j];
            if sh.x < -30.0 || sh.y < -30.0 || sh.x > w + 30.0 || sh.y > ctx.height() + 30.0 {
                self.shots.remove(j);
                continue;
            }
            j += 1;
        }
        for p in self.poofs.iter_mut() {
            p.2 += dt;
        }
        self.poofs.retain(|p| p.2 < if p.3 { 0.5 } else { 0.4 });
        if !self.mons.is_empty() || !self.mon_q.is_empty() || !self.shots.is_empty() || !self.poofs.is_empty() || self.def_g > 0 {
            ctx.animate();
        }
    }

    /// Un garde (skin du château équipé). Pour l'Éclipse ultime : le Maître de l'Infini (design original), qui lance des orbes violettes.
    fn draw_guard(&self, g: &mut Gfx, x: f32, y: f32, sk: u8, fire: f32, face: f32, s: f32) {
        use std::f32::consts::TAU;
        let t = self.anim_t;
        let y = y + (t * 3.0 + x * 0.1).sin() * 1.2 * s;
        let u = s;
        let f = face;
        let body = |g: &mut Gfx, col: u32, edge: u32| {
            g.rrect(x - 7.0 * u, y - 2.5 * u, 14.0 * u, 13.0 * u, 3.0 * u, rgb(edge));
            g.rrect(x - 6.0 * u, y - 1.5 * u, 12.0 * u, 11.0 * u, 2.5 * u, rgb(col));
        };
        let head = |g: &mut Gfx, col: u32, edge: u32, r: f32| {
            g.circle(x, y - 8.0 * u, (r + 1.2) * u, rgb(edge));
            g.circle(x, y - 8.0 * u, r * u, rgb(col));
        };
        let eyes = |g: &mut Gfx, col: u32| {
            g.circle(x - 2.2 * u, y - 8.5 * u, 1.1 * u, rgb(col));
            g.circle(x + 2.2 * u, y - 8.5 * u, 1.1 * u, rgb(col));
        };
        if fire > 0.0 {
            fx_glow(g, x + 9.0 * u * f, y - 2.0 * u, 10.0 * u, 0xFFFFFF, 0.8 * fire);
        }
        match sk {
            0 => {
                body(g, 0x8C8F99, 0x2A2C33);
                g.rrect(x - 10.0 * u * f.max(0.0) - if f < 0.0 { -5.0 * u } else { 0.0 }, y, 5.0 * u, 8.0 * u, 1.0 * u, rgb(0xC5C8CF));
                head(g, 0xA3A7B0, 0x2A2C33, 5.5);
                g.rect(x - 4.0 * u, y - 9.0 * u, 8.0 * u, 2.0 * u, rgb(0x2A2C33));
                for q in 0..9u32 {
                    let a = -1.2 + q as f32 * 0.3;
                    g.circle(x + (8.0 + a.cos() * 7.0 - 7.0) * u * f + 7.0 * u * f, y + a.sin() * 7.0 * u, 1.0 * u, rgb(0x6B4A2A));
                }
            }
            1 => {
                body(g, 0xF2C94C, 0x6B4F0A);
                head(g, 0xFFE27A, 0x6B4F0A, 5.5);
                fx_taper(g, x, y - 13.0 * u, x, y - 19.0 * u, 3.0 * u, 0xD62839, 1.0);
                eyes(g, 0x6B4F0A);
                fx_line(g, x + 7.0 * u * f, y + 4.0 * u, x + 12.0 * u * f, y - 10.0 * u, 2.0 * u, 0xFFF6C8, 1.0);
            }
            2 => {
                fx_taper(g, x, y - 3.0 * u, x, y - 16.0 * u, 7.0 * u, 0x7FDCF5, 1.0);
                fx_taper(g, x, y - 3.0 * u, x, y + 10.0 * u, 7.0 * u, 0x7FDCF5, 1.0);
                fx_taper(g, x, y - 3.0 * u, x, y - 12.0 * u, 3.0 * u, 0xE6FBFF, 0.7);
                eyes(g, 0xFFFFFF);
            }
            3 => {
                body(g, 0x241C2E, 0x000000);
                head(g, 0x2A2136, 0x000000, 5.5);
                fx_taper(g, x - 4.0 * u, y - 12.0 * u, x - 9.0 * u, y - 19.0 * u, 2.5 * u, 0x2A2136, 1.0);
                fx_taper(g, x + 4.0 * u, y - 12.0 * u, x + 9.0 * u, y - 19.0 * u, 2.5 * u, 0x2A2136, 1.0);
                eyes(g, 0xFF821E);
                fx_line(g, x - 2.0 * u, y + 1.0 * u, x + 2.0 * u, y + 7.0 * u, 1.0 * u, 0xFF6E1E, 0.8);
            }
            4 | 5 => {
                let gd = sk == 5;
                fx_taper(g, x, y + 10.0 * u, x, y - 20.0 * u, 9.0 * u, if gd { 0xD9A020 } else { 0x5A2CA8 }, 1.0);
                for q in 0..5u32 {
                    g.circle(x - 4.0 * u + ((q * 7) % 9) as f32 * u, y - 6.0 * u + q as f32 * 3.0 * u, 0.8 * u, rgb(0xFFFFFF));
                }
                eyes(g, if gd { 0xFFF3B0 } else { 0xE2C6FF });
                let col = GCOL[sk as usize].0;
                fx_line(g, x + 8.0 * u * f, y + 8.0 * u, x + 11.0 * u * f, y - 10.0 * u, 1.5 * u, col, 1.0);
                g.circle(x + 11.0 * u * f, y - 11.0 * u, 2.5 * u, rgb(col));
            }
            6 => {
                fx_taper(g, x, y + 10.0 * u, x, y - 16.0 * u, 9.0 * u, 0xFFB070, 1.0);
                fx_taper(g, x, y + 9.0 * u, x, y - 14.5 * u, 8.0 * u, 0x0A0503, 1.0);
                eyes(g, 0xFF9A3C);
            }
            7 => {
                body(g, 0x34495A, 0x0B141A);
                g.circle(x, y - 9.0 * u, 7.0 * u, rgb(0x0B3A1E));
                g.circle(x, y - 9.0 * u, 6.0 * u, rgb(0x2FBF6B));
                g.circle(x - 2.5 * u, y - 9.0 * u, 2.0 * u, rgb(0x050A06));
                g.circle(x + 2.5 * u, y - 9.0 * u, 2.0 * u, rgb(0x050A06));
                g.rect(x + if f > 0.0 { 5.0 * u } else { -13.0 * u }, y + 2.0 * u, 8.0 * u, 3.0 * u, rgb(0x4B6275));
            }
            8 => {
                body(g, 0x0C1038, 0xE9EEFF);
                head(g, 0x141A3A, 0xE9EEFF, 5.5);
                eyes(g, 0xFFFFFF);
                g.circle(x - 9.0 * u, y - 2.0 * u, 2.5 * u, rgb(0xFF3B5C));
                g.circle(x + 9.0 * u, y - 2.0 * u, 2.5 * u, rgb(0x4D8DFF));
            }
            9 => {
                body(g, 0x0A2A30, 0xF0C060);
                g.circle(x, y + 3.0 * u, 4.0 * u, rgb(0xFFF4D0));
                fx_line(g, x, y + 3.0 * u, x + (t * 3.0).sin() * 3.0 * u, y + 3.0 * u - (t * 3.0).cos() * 3.0 * u, 1.0 * u, 0x5A3F0A, 1.0);
                head(g, 0xB8892F, 0x5A3F0A, 5.0);
                eyes(g, 0x3FE0C8);
            }
            10 => {
                // Maître de l'Infini : robe claire à capuche, visage dans l'ombre avec un œil lumineux, ceinture violette, orbe violette en main
                fx_taper(g, x, y + 11.0 * u, x, y - 19.0 * u, 11.5 * u, 0x5A3A9A, 1.0);
                fx_taper(g, x, y + 10.0 * u, x, y - 18.0 * u, 10.5 * u, 0xEEF4FF, 1.0);
                g.circle(x, y - 8.0 * u, 5.0 * u, rgb(0x1A1030));
                fx_glow(g, x, y - 8.0 * u, 5.0 * u, 0x3FD0FF, 0.9);
                g.circle(x, y - 8.0 * u, 1.8 * u, rgb(0xEAFFFF));
                fx_line(g, x - 8.0 * u, y + 2.0 * u, x + 8.0 * u, y + 2.0 * u, 1.5 * u, 0xA64DFF, 1.0);
                fx_glow(g, x + 10.0 * u * f, y - 3.0 * u, (5.0 + fire * 3.0) * u, 0xC77DFF, 1.0);
                g.circle(x + 10.0 * u * f, y - 3.0 * u, 2.0 * u, rgb(0xFFFFFF));
            }
            _ => {
                fx_taper(g, x, y + 10.0 * u, x, y - 14.0 * u, 8.0 * u, 0xFF9A1C, 1.0);
                for q in 0..8u32 {
                    let a = q as f32 / 8.0 * TAU + t;
                    fx_line(g, x + a.cos() * 6.0 * u, y - 10.0 * u + a.sin() * 6.0 * u, x + a.cos() * 9.0 * u, y - 10.0 * u + a.sin() * 9.0 * u, 1.2 * u, 0xFFDC64, 0.9);
                }
                head(g, 0xFFD84A, 0x5A1A00, 5.0);
                eyes(g, 0x5A1A00);
            }
        }
    }

    fn draw_defense(&self, g: &mut Gfx, l: &Layout, s: f32) {
        use std::f32::consts::TAU;
        let t = self.anim_t;
        let n = (self.def_g as usize).min(8);
        let sk = self.skin.min(11);
        for i in 0..n {
            let (px, py) = self.guard_pos(l, i, n);
            let tg = self.mons.iter().min_by(|a, b| ((a.x - px).powi(2) + (a.y - py).powi(2)).partial_cmp(&((b.x - px).powi(2) + (b.y - py).powi(2))).unwrap_or(std::cmp::Ordering::Equal));
            let face = match tg {
                Some(m) => if m.x < px { -1.0 } else { 1.0 },
                None => if px < l.cx { -1.0 } else { 1.0 },
            };
            self.draw_guard(g, px, py, sk, self.gfire.get(i).copied().unwrap_or(0.0), face, s);
        }
        for m in &self.mons {
            let k = if m.big { 1.7 } else { 1.0 } * s;
            let wob = (t * 10.0 + m.ph).sin() * 1.5 * k;
            let (bc, ec) = if m.fl > 0.0 { (0xFFD0FF, 0xFFFFFF) } else { (0x4A1A6E, 0x7A3AA8) };
            fx_taper(g, m.x - 5.0 * k, m.y - 6.0 * k, m.x - 9.0 * k, m.y - 13.0 * k, 2.5 * k, 0x22082E, 1.0);
            fx_taper(g, m.x + 5.0 * k, m.y - 6.0 * k, m.x + 9.0 * k, m.y - 13.0 * k, 2.5 * k, 0x22082E, 1.0);
            g.circle(m.x, m.y, 9.5 * k, rgb(0x0A0010));
            g.circle(m.x, m.y, 8.5 * k, rgb(bc));
            g.circle(m.x - 2.0 * k, m.y - 3.0 * k, 4.5 * k, rgba(ec, 0.6));
            g.circle(m.x - 4.0 * k, m.y + 8.0 * k + wob, 2.0 * k, rgb(0x22082E));
            g.circle(m.x + 4.0 * k, m.y + 8.0 * k - wob, 2.0 * k, rgb(0x22082E));
            g.circle(m.x - 3.0 * k, m.y - 2.0 * k, 1.8 * k, rgb(0xFF3040));
            g.circle(m.x + 3.0 * k, m.y - 2.0 * k, 1.8 * k, rgb(0xFF3040));
            g.rect(m.x - 3.0 * k, m.y + 3.0 * k, 1.5 * k.max(1.4), 2.0 * k, rgb(0xFFFFFF));
            g.rect(m.x + 1.5 * k, m.y + 3.0 * k, 1.5 * k.max(1.4), 2.0 * k, rgb(0xFFFFFF));
            if m.hp < m.max {
                g.rect(m.x - 10.0 * s, m.y - 17.0 * k, 20.0 * s, 3.0 * s, rgba(0x000000, 0.6));
                g.rect(m.x - 10.0 * s, m.y - 17.0 * k, (20.0 * s * (m.hp / m.max) as f32).max(2.0), 3.0 * s, rgb(0xFF4A5A));
            }
        }
        for sh in &self.shots {
            let c = GCOL[sh.sk as usize % 12];
            if sh.sk == 10 {
                for (i, p) in sh.tr.iter().rev().enumerate().take(8) {
                    let a = 1.0 - (i as f32 + 1.0) / 9.0;
                    g.circle(p.0 + (t * 20.0 + i as f32).cos() * 3.0 * s, p.1 + (t * 20.0 + i as f32).sin() * 3.0 * s, 6.0 * s * a, rgba(0xAA5AFF, 0.35 * a));
                }
                fx_glow(g, sh.x, sh.y, 9.0 * s, 0xB46EFF, 1.0);
                g.circle(sh.x, sh.y, 3.0 * s, rgb(0xFFFFFF));
            } else {
                let col = if sh.sk == 8 && sh.alt { c.1 } else { c.0 };
                fx_line(g, sh.x, sh.y, sh.x - sh.vx * 0.04, sh.y - sh.vy * 0.04, if sh.sk == 7 { 2.0 } else { 3.0 } * s, col, 1.0);
                g.circle(sh.x, sh.y, if sh.sk == 6 { 4.0 } else { 2.5 } * s, rgb(col));
            }
        }
        for p in &self.poofs {
            let k = p.2 / if p.3 { 0.5 } else { 0.4 };
            let rr = (if p.3 { 34.0 } else if p.4 { 22.0 } else { 14.0 } * k + 3.0) * s;
            fx_ring(g, p.0, p.1, rr, 40, if p.3 { 2.0 } else { 1.3 } * s, if p.3 { 0xC882FF } else { 0xFF78C8 }, 1.0 - k);
            for j in 0..8u32 {
                let a = j as f32 / 8.0 * TAU;
                g.circle(p.0 + a.cos() * 18.0 * s * k, p.1 + a.sin() * 18.0 * s * k, 2.0 * s, rgba(if p.3 { 0xE6C8FF } else { 0xAA50DC }, 1.0 - k));
            }
        }
    }

    /// Œil (dessin original) : amande claire, iris turquoise strié, anneau d'analyse qui tourne, pupille, reflets, cils.
    /// Tout est découpé en bandes verticales pour rester à l'intérieur des paupières (même à moitié fermé).
    fn draw_eye(&self, g: &mut Gfx, ex: f32, ey: f32, ew: f32, eh: f32, ir: f32, glow: f32) {
        use std::f32::consts::TAU;
        let t = self.anim_t;
        if glow > 0.0 {
            fx_glow(g, ex, ey, ew * 1.5, 0x5ADCFF, glow);
        }
        if eh < 1.5 {
            fx_line(g, ex - ew, ey, ex + ew, ey, (ew * 0.03).max(2.0), 0xCDF6FF, 1.0);
            return;
        }
        let half = |x: f32| -> f32 { (eh * (1.0 - (x / ew).powi(2))).max(0.0) };
        let inside = |x: f32, y: f32| -> bool { (x - ex).abs() < ew && (y - ey).abs() < half(x - ex) };
        let sw = (ew / 48.0).max(2.5);
        let n = (2.0 * ew / sw).ceil() as i32;
        // blanc de l'œil
        for i in 0..n {
            let x = -ew + i as f32 * sw;
            let hh = half(x + sw / 2.0);
            if hh >= 1.2 {
                let u = (x + sw / 2.0).abs() / ew;
                g.rect(ex + x, ey - hh, sw + 0.6, hh * 2.0, rgb(mix_color(0xF6FCFF, 0xA9C8E2, u)));
            }
        }
        // iris en anneaux de couleur, limité aux paupières
        let ix = ex + (t * 0.7).sin() * ew * 0.1;
        let iy = ey + (t * 0.9).cos() * ir * 0.08;
        let bands: [(f32, u32); 6] = [(1.0, 0x062742), (0.93, 0x0A3D6B), (0.8, 0x1B9FDC), (0.6, 0x45CBF2), (0.4, 0x7BECFF), (0.22, 0xD8FFFF)];
        for (fr, col) in bands {
            let rr = ir * fr;
            let m = (2.0 * rr / sw).ceil() as i32;
            for i in 0..m {
                let x = ix - rr + i as f32 * sw;
                let dx = x + sw / 2.0 - ix;
                if dx.abs() >= rr {
                    continue;
                }
                let cy_half = (rr * rr - dx * dx).sqrt();
                let lh = half(x + sw / 2.0 - ex);
                let top = (iy - cy_half).max(ey - lh);
                let bot = (iy + cy_half).min(ey + lh);
                if bot - top >= 1.2 {
                    g.rect(x, top, sw + 0.6, bot - top, rgb(col));
                }
            }
        }
        // stries de l'iris + points lumineux + anneau d'analyse
        for i in 0..40u32 {
            let a = i as f32 / 40.0 * TAU + t * 0.12;
            let (r0, r1) = (ir * 0.26, ir * (0.62 + 0.3 * ((i * 13) % 5) as f32 / 4.0));
            let col = if i % 3 == 0 { 0x063264 } else { 0xFFFFFF };
            let al = if i % 3 == 0 { 0.5 } else { 0.4 };
            let k = 6u32;
            for q in 0..=k {
                let rr = r0 + (r1 - r0) * q as f32 / k as f32;
                let (px, py) = (ix + a.cos() * rr, iy + a.sin() * rr);
                if inside(px, py) {
                    g.circle(px, py, (ir * 0.025).max(0.7), rgba(col, al));
                }
            }
        }
        for i in 0..36u32 {
            let a = i as f32 / 36.0 * TAU - t * 0.6;
            if i % 3 == 2 {
                continue;
            }
            let (px, py) = (ix + a.cos() * ir * 0.78, iy + a.sin() * ir * 0.78);
            if inside(px, py) {
                g.circle(px, py, (ir * 0.03).max(0.8), rgba(0xD2FCFF, 0.85));
            }
        }
        let pr = ir * (0.19 + 0.03 * (t * 2.0).sin());
        if inside(ix, iy - pr * 0.5) && inside(ix, iy + pr * 0.5) {
            g.circle(ix, iy, pr, rgb(0x020B16));
        }
        for (dx, dy, r) in [(-0.32f32, -0.34f32, 0.16f32), (0.33, 0.28, 0.07)] {
            let (px, py) = (ix + dx * ir, iy + dy * ir);
            if inside(px, py) {
                g.circle(px, py, ir * r, rgba(0xFFFFFF, 0.95));
            }
        }
        // ombre de la paupière du haut
        for i in 0..n {
            let x = -ew + i as f32 * sw;
            let hh = half(x + sw / 2.0);
            if hh >= 2.0 {
                g.rect(ex + x, ey - hh, sw + 0.6, hh * 0.45, rgba(0x061228, 0.4));
            }
        }
        // contour des paupières + cils
        let steps = 48;
        for i in 0..=steps {
            let x = -ew + 2.0 * ew * i as f32 / steps as f32;
            let hh = half(x);
            g.circle(ex + x, ey - hh, (ew * 0.03).max(1.2), rgb(0x0A0E1C));
            g.circle(ex + x, ey + hh, (ew * 0.012).max(0.7), rgb(0x0A0E1C));
        }
        for i in 0..13u32 {
            let u = (i as f32 + 0.5) / 13.0;
            let x = -ew + 2.0 * ew * u;
            let (px, py) = (ex + x, ey - half(x));
            let an = -1.5708 + (u - 0.5) * 1.9;
            let ln = ew * 0.17 * (0.5 + 0.9 * (u * std::f32::consts::PI).sin());
            fx_line(g, px, py, px + an.cos() * ln, py + an.sin() * ln - ln * 0.12, (ew * 0.025).max(1.2), 0x0A0E1C, 1.0);
        }
    }

    /// Ouverture de l'Œil absolu (≈ 5,4 s) quand le château Éclipse ultime est débloqué.
    fn draw_eyefx(&mut self, ctx: &Ctx, g: &mut Gfx) {
        use std::f32::consts::TAU;
        let t = self.eye_t;
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let m = w.min(h);
        let (cx, cy) = (w / 2.0, h * 0.42);
        let fade = if t < 0.3 { 1.0 - t / 0.3 } else if t > 4.7 { ((t - 4.7) / 0.7).min(1.0) } else { 0.0 };
        if fade > 0.0 {
            self.draw_game(ctx, g);
            g.rect(0.0, 0.0, w, h, rgba(0x01040B, 1.0 - fade));
        } else {
            g.clear(rgb(0x01040B));
        }
        let open = ez((t - 0.6) / 1.1);
        let ew = m * 0.42;
        let sc = ez((t - 1.5) / 0.8);
        for i in 0..90u32 {
            let a = hash01(i, 401) * TAU;
            let f = (t * 0.5 * (0.4 + hash01(i + 3, 402)) + hash01(i + 5, 403)) % 1.0;
            let d = m * (1.1 - f);
            g.circle(cx + a.cos() * d, cy + a.sin() * d * 0.8, (1.0 + hash01(i + 7, 404) * 2.0) * s * 0.8, rgba(0x78E6FF, f * 0.6));
        }
        if sc > 0.0 {
            for i in 0..3u32 {
                let rr = ew * (1.15 + i as f32 * 0.22);
                let nd = 90u32;
                for q in 0..nd {
                    if (q + (t * 10.0) as u32 * if i % 2 == 0 { 1 } else { 3 }) % 4 == 0 {
                        continue;
                    }
                    let a = q as f32 / nd as f32 * TAU;
                    g.circle(cx + a.cos() * rr, cy + a.sin() * rr, 1.1 * s, rgba(0x8CEBFF, 0.5 * sc));
                }
            }
            for i in 0..72u32 {
                let a = i as f32 / 72.0 * TAU - t * 0.3;
                let r0 = ew * 1.62;
                let r1 = r0 + if i % 6 == 0 { m * 0.04 } else { m * 0.015 };
                fx_line(g, cx + a.cos() * r0, cy + a.sin() * r0, cx + a.cos() * r1, cy + a.sin() * r1, if i % 6 == 0 { 1.6 * s } else { 0.9 * s }, 0xAAF5FF, 0.6 * sc);
            }
            let sw = ((t - 1.5) * 0.7) % 1.0;
            fx_ring(g, cx, cy, ew * (0.4 + sw * 2.2), 120, 1.5 * s, 0xC8FAFF, 0.6 * (1.0 - sw));
            for i in 0..16u32 {
                let a = i as f32 / 16.0 * TAU + t * 0.2;
                let rr = ew * 1.85;
                let txt = format!("{:04X}", (hash01(i + (t * 6.0) as u32, 405) * 65535.0) as u32);
                ctx.text_mid(g, &txt, cx + a.cos() * rr, cy + a.sin() * rr, &Text::new((m * 0.022).max(8.0 * s)).bold().color(rgba(0x96F0FF, 0.55 * sc * (0.5 + 0.5 * (t * 4.0 + i as f32).sin()))).center());
            }
        }
        self.draw_eye(g, cx, cy, ew * ez(t / 0.6).max(0.15), ew * 0.42 * open, ew * 0.4, 0.65 * open.max(0.2));
        let fl = (1.0 - (t - 1.7).abs() / 0.15).max(0.0);
        if fl > 0.0 {
            g.rect(0.0, 0.0, w, h, rgba(0xC8FAFF, fl * 0.8));
        }
        if t > 1.9 {
            let tp = ez((t - 1.9) / 0.35);
            let fs = (w * 0.09).min(40.0 * s);
            self.text_outline(ctx, g, "L’ŒIL ABSOLU", cx, h * 0.1 + fs * 0.3, fs * (1.5 - 0.5 * tp), 0xBFF6FF, tp, w - 8.0);
            let fs2 = (w * 0.036).clamp(10.0 * s, 16.0 * s);
            let lines = ["Perception absolue · butin des boss ×5", "Coût quasi nul · sorts gratuits, recharges ÷10", "Maîtrise de l’Infini · Extension ×10 milliards"];
            for (k, txt) in lines.iter().enumerate() {
                let a = ez((t - 2.3 - k as f32 * 0.35) / 0.3);
                if a > 0.0 {
                    self.text_outline(ctx, g, txt, cx, h * 0.72 + k as f32 * fs2 * 1.7, fs2, 0xFFFFFF, a, w - 8.0);
                }
            }
            let a5 = ez((t - 3.5) / 0.3);
            if a5 > 0.0 {
                self.text_outline(ctx, g, "ÉCLIPSE ULTIME · ×10 000 milliards", cx, h * 0.72 + 3.3 * fs2 * 1.7, fs2 * 1.25, 0x7BECFF, a5, w - 8.0);
            }
        }
    }

    /// Apparition du boss (≈ 3 s) : un décor différent par château ; le boss surgit à t = 1,7 s.
    fn draw_bossintro(&mut self, ctx: &Ctx, g: &mut Gfx) {
        use std::f32::consts::TAU;
        let t = self.boss_intro_t;
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let m = w.min(h);
        let (sk, side) = self.boss.as_ref().map_or((self.skin.min(11), 0), |b| (b.skin, b.side));
        let (tx, ty, _) = self.boss_pos_side(ctx, side);
        let fade = if t < 0.2 { 1.0 - t / 0.2 } else if t > 2.4 { ((t - 2.4) / 0.6).min(1.0) } else { 0.0 };
        if fade > 0.0 {
            self.draw_game(ctx, g);
            g.rect(0.0, 0.0, w, h, rgba(0x04050B, 1.0 - fade));
        } else {
            g.clear(rgb(0x04050B));
        }
        let g1 = ez(t / 1.2);
        match sk {
            0 => {
                // golem : le sol se fissure, des rochers tombent
                for k in 0..16u32 {
                    let mut a = -3.14 + k as f32 * 0.2 + (hash01(k * 3, 201) - 0.5) * 0.2;
                    let (mut px, mut py, mut l) = (tx, ty + 50.0 * s, 0.0f32);
                    let al = 0.5 + 0.5 * (t * 6.0 + k as f32).sin();
                    for i in 0..8u32 {
                        a += (hash01(k * 7 + i, 202) - 0.5) * 0.8;
                        l += m * 0.07 * ez(t / 1.3);
                        let (nx, ny) = (tx + a.cos() * l * (1.0 + i as f32 * 0.2), ty + 50.0 * s + a.sin() * l * 0.7);
                        fx_line(g, px, py, nx, ny, 1.6 * s, 0xFF8C28, al);
                        px = nx;
                        py = ny;
                    }
                }
                for i in 0..26u32 {
                    g.circle(hash01(i + 50, 203) * w, -60.0 + ((t * 420.0 + hash01(i + 9, 204) * 800.0) % (h + 120.0)), (6.0 + hash01(i + 90, 205) * 10.0) * s * 0.6, rgba(0x827D87, 0.75));
                }
                fx_glow(g, tx, ty + 60.0 * s, m * 0.5 * ez(t / 1.3), 0xFFAA50, 0.4);
            }
            1 => {
                // gardien doré : faisceaux de lumière, pluie de pièces
                for i in 0..5i32 {
                    let bx = tx + (i - 2) as f32 * m * 0.09;
                    let bw = m * 0.05 * (1.0 + g1);
                    for q in 0..4u32 {
                        g.rect(bx - bw / 2.0 + q as f32 * 2.0, 0.0, (bw - q as f32 * 4.0).max(3.0), ty + 60.0 * s, rgba(0xFFEB8C, 0.16 * g1));
                    }
                }
                for i in 0..40u32 {
                    let (x0, y0) = (hash01(i, 211) * w, ((t * 260.0 * (0.6 + hash01(i + 5, 212)) + hash01(i + 9, 213) * h) % (h + 40.0)) - 20.0);
                    g.circle(x0, y0, (3.0 + hash01(i + 1, 214) * 4.0) * s * (t * 6.0 + i as f32).cos().abs().max(0.3), rgba(0xFFD75A, 0.9));
                }
                fx_glow(g, tx, ty, m * 0.45 * g1, 0xFFF0B0, 0.5);
                for i in 0..16u32 {
                    let a = i as f32 / 16.0 * TAU + t;
                    fx_line(g, tx, ty, tx + a.cos() * m * 0.6 * g1, ty + a.sin() * m * 0.6 * g1, 1.6 * s, 0xFFF0AA, 0.5 * g1);
                }
            }
            2 => {
                // colosse de cristal : éclats qui jaillissent, fissures, prisme
                for i in 0..26u32 {
                    let a = i as f32 / 26.0 * TAU + hash01(i, 221) * 0.2;
                    let d = m * 0.95 + (m * (0.25 + hash01(i + 3, 222) * 0.2) - m * 0.95) * ez(t / 1.3);
                    let l = m * (0.12 + hash01(i + 7, 223) * 0.16) * ez(t / 1.3);
                    let (x, y) = (tx + a.cos() * d, ty + a.sin() * d);
                    fx_taper(g, x - a.cos() * l, y - a.sin() * l, x, y, 1.0, 0x90E6FF, 0.5 * ez(t / 1.3));
                    fx_taper(g, x, y, x - a.cos() * l, y - a.sin() * l, 6.0 * s, 0x90E6FF, 0.55 * ez(t / 1.3));
                }
                for i in 0..10u32 {
                    let (mut px, mut py) = (hash01(i + 20, 224) * w, hash01(i + 30, 225) * h);
                    for j in 0..5u32 {
                        let (nx, ny) = (px + (hash01(i * 5 + j, 226) - 0.5) * m * 0.5, py + (hash01(i * 5 + j + 40, 227) - 0.5) * m * 0.5);
                        fx_line(g, px, py, nx, ny, 1.0 * s, 0xE6FAFF, 0.5 * ez(t / 1.3));
                        px = nx;
                        py = ny;
                    }
                }
                let pr = ez((t - 1.0) / 0.7);
                if pr > 0.0 {
                    for i in 0..6u32 {
                        fx_ring(g, tx, ty, m * (0.05 + 0.4 * pr) + i as f32 * 5.0, 64, 2.2 * s, hsv_hex((i as f32 * 60.0 + t * 120.0) % 360.0, 0.8, 1.0), 0.5 * (1.0 - pr));
                    }
                }
                for i in 0..20u32 {
                    let sp = (t * 0.7 + hash01(i, 228)) % 1.0;
                    g.circle(hash01(i + 60, 229) * w, hash01(i + 70, 230) * h, 2.0 * s, rgba(0xFFFFFF, (sp * 3.14).sin()));
                }
            }
            3 => {
                // brute de netherite : faille de lave, colonnes de feu, braises
                let (mut px, mut py) = (tx, 0.0f32);
                for i in 1..=14u32 {
                    let yy = h * i as f32 / 14.0;
                    let xx = tx + (i as f32 * 1.7 + t * 3.0).sin() * m * 0.05 * g1 + (hash01(i, 231) - 0.5) * m * 0.08;
                    fx_line(g, px, py, xx, yy, (2.0 + 5.0 * g1) * s, 0xFF6E1E, 0.9);
                    px = xx;
                    py = yy;
                }
                for i in 0..14u32 {
                    let fx = w * (i as f32 + 0.5) / 14.0;
                    let fh = h * (0.25 + 0.5 * hash01(i, 232)) * g1 * (0.8 + 0.2 * (t * 7.0 + i as f32).sin());
                    for q in 0..4u32 {
                        let f = q as f32 / 4.0;
                        g.rect(fx - w / 30.0, h - fh * (1.0 - f * 0.55) , w / 16.0, fh * (1.0 - f * 0.55), rgba(mix_color(0xFF5A14, 0xFFA028, f), 0.2));
                    }
                }
                for i in 0..70u32 {
                    let f = (t * 0.5 * (0.5 + hash01(i, 233)) + hash01(i + 3, 234)) % 1.0;
                    g.circle(hash01(i + 11, 235) * w + (t * 3.0 + i as f32).sin() * 8.0, h - f * h * 0.9, (1.5 + hash01(i + 13, 236) * 2.5) * s, rgba(0xFF8C28, 1.0 - f));
                }
                fx_glow(g, tx, ty, m * 0.4 * g1, 0xBE5AFF, 0.6);
            }
            4 | 5 => {
                // titans stellaires : traversée des étoiles puis effondrement en nébuleuse
                let gd = sk == 5;
                let conv = ez((t - 0.9) / 0.8);
                for i in 0..100u32 {
                    let a = hash01(i, 241) * TAU;
                    let f = (t * 0.8 * (0.4 + hash01(i + 3, 242)) + hash01(i + 5, 243)) % 1.0;
                    let d = (m * 0.05 + f * f * m * 1.0) * (1.0 - conv * 0.85);
                    fx_line(g, tx + a.cos() * d, ty + a.sin() * d, tx + a.cos() * d * 0.8, ty + a.sin() * d * 0.8, (1.0 + f * 2.0) * s, if gd { 0xFFD078 } else { 0xBE9AFF }, f);
                }
                fx_glow(g, tx, ty, m * 0.4 * g1, if gd { 0xFF5AC8 } else { 0x7896FF }, 0.55);
                for i in 0..8u32 {
                    let (pa, pd, qa, qd) = (hash01(i + 20, 244) * TAU, m * (0.08 + hash01(i + 30, 245) * 0.3), hash01(i + 50, 246) * TAU, m * (0.08 + hash01(i + 60, 247) * 0.3));
                    fx_line(g, tx + pa.cos() * pd, ty + pa.sin() * pd, tx + qa.cos() * qd, ty + qa.sin() * qd, 0.8 * s, 0xFFFFFF, 0.5 * conv);
                }
                if gd {
                    for i in 0..60u32 {
                        let a = i as f32 / 60.0 * TAU;
                        let (ex, ey) = (a.cos() * m * 0.42 * conv, a.sin() * m * 0.07 * conv);
                        g.circle(tx + ex * 0.94 + ey * 0.34, ty + ey * 0.94 - ex * 0.34, 2.6 * s, rgba(0xFFD76E, 0.9 * conv));
                    }
                }
            }
            6 => {
                // dévoreur d'horizon : un trou noir s'ouvre et aspire l'écran
                for i in 0..60u32 {
                    let a = hash01(i, 251) * TAU;
                    let f = (t * 0.9 * (0.5 + hash01(i + 2, 252)) + hash01(i + 4, 253)) % 1.0;
                    let d = m * 1.1 * (1.0 - f);
                    fx_line(g, tx + a.cos() * d, ty + a.sin() * d, tx + a.cos() * (d + m * 0.1), ty + a.sin() * (d + m * 0.1), 1.4 * s, 0xFF9650, 0.6 * f);
                }
                for i in 0..6u32 {
                    let ea = i as f32 / 6.0 * TAU + 0.4;
                    let (ex, ey) = (tx + ea.cos() * m * 1.2, ty + ea.sin() * m * 1.2);
                    let k = ez((t - 0.3 - i as f32 * 0.1) / 0.8);
                    let (mx, my) = (tx + (ea + 0.6).cos() * m * 0.5, ty + (ea + 0.6).sin() * m * 0.5);
                    fx_taper(g, ex, ey, mx, my, 7.0 * s, 0x1E0C08, 0.95);
                    fx_taper(g, mx, my, ex + (tx - ex) * k * 0.85, ey + (ty - ey) * k * 0.85, 5.0 * s, 0x1E0C08, 0.95);
                }
                for i in 0..60u32 {
                    let a = i as f32 / 60.0 * TAU;
                    for (rx, ry, sz, col) in [(0.34f32, 0.08f32, 3.4f32, 0xFFAA50u32), (0.28, 0.06, 1.8, 0xFFEBC8)] {
                        let (ex, ey) = (a.cos() * m * rx * g1, a.sin() * m * ry * g1);
                        g.circle(tx + ex * 0.955 + ey * 0.296, ty + ey * 0.955 - ex * 0.296, sz * s, rgba(col, 0.9));
                    }
                }
                g.circle(tx, ty, (m * 0.19 * g1).max(0.5), rgb(0x000000));
                let ep = ez((t - 1.3) / 0.3);
                if ep > 0.0 {
                    fx_glow(g, tx, ty, m * 0.2 * ep, 0xFF8C32, ep);
                    g.circle(tx, ty, m * 0.06 * ep, rgba(0xFF963C, ep));
                    g.rrect(tx - 2.0 * s, ty - m * 0.045 * ep, 4.0 * s, m * 0.09 * ep, 2.0 * s, rgb(0x000000));
                }
            }
            7 => {
                // envahisseur : balayage radar, faisceau tracteur, parasites
                for i in 0..40u32 {
                    g.rect(0.0, (t * 400.0 + i as f32 * h / 40.0) % (h - 3.0), w, 2.0, rgba(0x3CFF8C, 0.1));
                }
                g.rect(0.0, ((t * 0.8) % 1.0) * (h - 8.0), w, 6.0, rgba(0x78FFB4, 0.35));
                let bw = m * 0.32 * g1;
                for q in 0..24u32 {
                    let f = q as f32 / 24.0;
                    let y = f * (ty + 80.0 * s);
                    let half = m * 0.05 + (bw - m * 0.05) * f;
                    g.rect(tx - half, y, half * 2.0, (ty + 80.0 * s) / 24.0 + 1.0, rgba(0x78FFAA, 0.55 - 0.4 * f));
                }
                let uy = -20.0 + 50.0 * g1;
                for (i, k) in (0..9i32).enumerate() {
                    g.circle(tx + (k - 4) as f32 * m * 0.035, uy, m * 0.045, rgba(0xC8EBDC, 0.9));
                    let _ = i;
                }
                for i in 0..7u32 {
                    g.circle(tx - m * 0.1 + i as f32 * m * 0.033, uy + 3.0 * s, 3.0 * s, if (t * 8.0) as u32 % 7 == i { rgb(0xFFFFFF) } else { rgba(0x3CFF8C, 0.9) });
                }
                for i in 0..6u32 {
                    let q = (t * 10.0) as u32;
                    if (t * 10.0 + i as f32) % 3.0 < 0.9 {
                        g.rect(hash01(i + q, 261) * (w - m * 0.3), hash01(i + 9 + q, 262) * (h - 10.0), m * 0.3, 4.0, rgba(0x3CFF8C, 0.28));
                    }
                }
                let rf = ez((t - 1.2) / 0.5);
                for i in 0..3u32 {
                    fx_ring(g, tx, ty, m * (0.05 + 0.3 * rf) + i as f32 * 10.0, 56, 2.0 * s, 0x78FFAA, 0.7 * (1.0 - rf));
                }
            }
            8 => {
                // roi des malédictions : entailles rouges, flammes spectrales, quatre yeux dans le noir
                for i in 0..3u32 {
                    let st = 0.15 + i as f32 * 0.3;
                    let p = ez((t - st) / 0.25);
                    if p <= 0.0 {
                        continue;
                    }
                    let y0 = h * (0.2 + 0.3 * i as f32);
                    let (x1, y1) = (-w * 0.1 + w * 1.2 * p, y0 + h * (0.25 - 0.05 * i as f32) * p);
                    let al = (1.0 - (t - st - 0.6).max(0.0)).max(0.0);
                    fx_line(g, -w * 0.1, y0, x1, y1, m * 0.03, 0xFF3C5A, 0.9 * al);
                    fx_line(g, -w * 0.1, y0, x1, y1, 1.6 * s, 0xFFFFFF, 0.9 * al);
                }
                for i in 0..22u32 {
                    let fx = w * (i as f32 + 0.5) / 22.0;
                    let fh = h * (0.15 + 0.35 * hash01(i, 271)) * ez(t / 1.1) * (0.8 + 0.2 * (t * 8.0 + i as f32).sin());
                    fx_taper(g, fx, h, fx + (t * 5.0 + i as f32).sin() * 8.0, h - fh, w / 44.0, 0x28E6D2, 0.7);
                }
                let v = 0.5 * ez(t / 1.1);
                g.rect(0.0, 0.0, w, h * 0.12, rgba(0x000000, v));
                g.rect(0.0, h * 0.88, w, h * 0.12, rgba(0x000000, v));
                g.rect(0.0, 0.0, w * 0.1, h, rgba(0x000000, v));
                g.rect(w * 0.9, 0.0, w * 0.1, h, rgba(0x000000, v));
                for (k, (ex, st)) in [(-0.5f32, 0.9f32), (0.5, 0.9), (-0.3, 1.25), (0.3, 1.25)].iter().enumerate() {
                    let p = ez((t - st) / 0.3);
                    if p <= 0.0 {
                        continue;
                    }
                    let (x, y) = (tx + ex * m * 0.1, ty - m * 0.08 + if k > 1 { m * 0.05 } else { 0.0 });
                    fx_glow(g, x, y, m * 0.07 * p, 0xFF5A78, p);
                    g.circle(x, y, (m * 0.012 * p).max(0.5), rgba(0xFFE6EB, p));
                }
            }
            11 => {
                // seigneur solaire : un soleil se forme, rayons géants, éjections de plasma, braises
                let sr = m * 0.16 * g1;
                for i in 0..20u32 {
                    let a = i as f32 / 20.0 * TAU + t * 0.6;
                    fx_taper(g, tx + a.cos() * m * 1.5, ty + a.sin() * m * 1.5, tx, ty, m * 0.07, 0xFFBE46, 0.16 * g1);
                }
                fx_glow(g, tx, ty, sr * 2.4 + 1.0, 0xFFD250, 1.0);
                g.circle(tx, ty, sr.max(0.5), rgb(0xFFFAE0));
                for i in 0..6u32 {
                    let a0 = i as f32 * 1.05 + 0.3;
                    let p = ez((t - 0.3 - i as f32 * 0.12) / 0.8);
                    if p <= 0.0 {
                        continue;
                    }
                    let al = 0.85 * (1.0 - (t - 1.6).max(0.0)).max(0.0);
                    let (x0, y0, x1, y1) = (tx + a0.cos() * sr, ty + a0.sin() * sr, tx + (a0 + 0.7).cos() * sr, ty + (a0 + 0.7).sin() * sr);
                    let (qx, qy) = (tx + (a0 + 0.35).cos() * (sr + m * 0.35 * p), ty + (a0 + 0.35).sin() * (sr + m * 0.35 * p));
                    for q in 0..=30u32 {
                        let u = q as f32 / 30.0;
                        g.circle((1.0 - u).powi(2) * x0 + 2.0 * u * (1.0 - u) * qx + u * u * x1, (1.0 - u).powi(2) * y0 + 2.0 * u * (1.0 - u) * qy + u * u * y1, m * 0.008, rgba(mix_color(0xFF7828, 0xFFC850, i as f32 / 6.0), al));
                    }
                }
                for i in 0..60u32 {
                    let f = (t * 0.6 * (0.5 + hash01(i, 531)) + hash01(i + 3, 532)) % 1.0;
                    g.circle(hash01(i + 11, 533) * w, h - f * h, (1.5 + hash01(i + 13, 534) * 2.5) * s, rgba(0xFFB43C, 1.0 - f));
                }
            }
            _ => {
                // gardien du temps : un cadran géant dont les aiguilles remontent le temps, engrenages, sable
                let r = m * 0.3 * ez(t / 1.0);
                fx_ring(g, tx, ty, r, 90, 3.0 * s, 0xF0C060, 0.9);
                for i in 0..60u32 {
                    let a = i as f32 / 60.0 * TAU;
                    let big = i % 5 == 0;
                    let (r0, r1) = (r * if big { 0.82 } else { 0.9 }, r * 0.96);
                    fx_line(g, tx + a.sin() * r0, ty - a.cos() * r0, tx + a.sin() * r1, ty - a.cos() * r1, if big { 2.2 * s } else { 1.0 * s }, 0xFFE6AA, if big { 0.95 } else { 0.5 });
                }
                let ma = if t < 0.9 { -t * 20.0 } else { -18.0 + (t - 0.9) * 9.0 };
                let ha = ma / 12.0;
                fx_line(g, tx, ty, tx + ma.sin() * r * 0.85, ty - ma.cos() * r * 0.85, 2.6 * s, 0x3FE0C8, 0.95);
                fx_line(g, tx, ty, tx + ha.sin() * r * 0.55, ty - ha.cos() * r * 0.55, 4.0 * s, 0xF0C060, 0.95);
                for (gx, gy, gr, tn, sp, col) in [(-1.35f32, 0.5f32, 0.35f32, 12u32, 2.0f32, 0xF0C060u32), (1.35, -0.4, 0.3, 10, -2.4, 0x3FE0C8), (0.9, 1.2, 0.22, 8, 3.0, 0xF0C060)] {
                    for q in 0..tn {
                        let a = t * sp + q as f32 * TAU / tn as f32;
                        g.circle(tx + gx * r + a.cos() * gr * r, ty + gy * r + a.sin() * gr * r, gr * r * 0.2, rgba(col, 0.3));
                    }
                }
                for i in 0..50u32 {
                    let f = (t * 0.6 * (0.5 + hash01(i, 281)) + hash01(i + 3, 282)) % 1.0;
                    g.circle(hash01(i + 9, 283) * w, f * h, (1.0 + hash01(i + 11, 284) * 2.0) * s, rgba(0xFFD98A, 1.0 - f));
                }
                let rp = ez((t - 1.0) / 0.7);
                for i in 0..3u32 {
                    fx_ring(g, tx, ty, r + rp * m * 0.4 + i as f32 * 14.0, 72, 2.0 * s, 0x3FE0C8, 0.5 * (1.0 - rp));
                }
            }
        }
        let fl = (1.0 - (t - 1.7).abs() / 0.12).max(0.0);
        if fl > 0.0 {
            let col = [0xFFBE6E, 0xFFF0AA, 0xD2F5FF, 0xFF9646, 0xC8B4FF, 0xFFE696, 0xFFAA5A, 0xAAFFC8, 0xFF5A78, 0xAAFFF0, 0xFF5A78, 0xFFF0B4][sk as usize % 12];
            g.rect(0.0, 0.0, w, h, rgba(col, fl.min(1.0)));
        }
        if t > 0.7 && t < 2.6 {
            let tp = ez((t - 0.7) / 0.25);
            let al = tp * if t > 2.2 { (1.0 - (t - 2.2) / 0.4).max(0.0) } else { 1.0 };
            let fs = (w * 0.075).min(40.0 * s);
            let col = [0xFFB45A, 0xFFE27A, 0x9FE8FF, 0xFF8A4A, 0xC9A8FF, 0xFFD76E, 0xFFA35A, 0x7DFFA8, 0xFF4A68, 0x7FF3E0, 0xFF4A68, 0xFFD84A][sk as usize % 12];
            self.text_outline(ctx, g, "BOSS", w / 2.0, h * 0.2, fs * (1.7 - 0.7 * tp), col, al, w - 8.0);
            self.text_outline(ctx, g, BOSS_NAMES[sk as usize % 12], w / 2.0, h * 0.2 + fs * 1.1, fs * 0.7, 0xFFFFFF, al, w - 8.0);
        }
    }

    /// Orbe violette (≈ 4,4 s) : tourbillon violet, énergie aspirée vers l'orbe, anneaux d'étincelles, rayons, lancement avec
    /// traînée en hélice, impact (onde de choc, fissures, débris), « ANÉANTI » et bandes de cinéma.
    fn draw_orbfx(&mut self, ctx: &Ctx, g: &mut Gfx) {
        use std::f32::consts::TAU;
        let t = self.orb_t;
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let (cx, cy) = (w * 0.5, h * 0.5);
        let m = w.min(h);
        let (tx, ty) = self.orb_target;
        let fade = if t < 0.3 { 1.0 - t / 0.3 } else if t > 3.8 { ((t - 3.8) / 0.6).min(1.0) } else { 0.0 };
        if fade > 0.0 {
            self.draw_game(ctx, g);
            g.rect(0.0, 0.0, w, h, rgba(0x06021A, 1.0 - fade));
        } else {
            g.clear(rgb(0x06021A));
        }
        let bgk = ez(t / 1.8);
        for (f, al, c) in [(0.95f32, 0.12f32, 0x28107A), (0.7, 0.14, 0x3C1C96), (0.45, 0.16, 0x5A2CC8), (0.25, 0.14 + 0.1 * bgk, 0x7A44E8)] {
            g.circle(cx, cy, m * f, rgba(c, al + 0.1 * bgk));
        }
        for i in 0..70u32 {
            let u = (i as f32 * 0.618) % 1.0;
            let a = i as f32 * 2.4 + u * 7.0 + t * (0.5 + 2.0 * (1.0 - u) * (1.0 + t * 0.3));
            let rr = m * (0.1 + 0.8 * u);
            g.circle(cx + a.cos() * rr, cy + a.sin() * rr * 0.85, m * (0.012 + 0.025 * u), rgba(mix_color(0x8C50FF, 0xE6BEFF, u), 0.06 + 0.1 * (1.0 - u)));
        }
        let ch = ez((t - 0.2) / 1.7);
        if t < 2.3 {
            for i in 0..110u32 {
                let (pa, pd, psp, ph) = (hash01(i, 301) * TAU, m * (0.3 + hash01(i, 302) * 0.8), 0.5 + hash01(i, 303) * 0.9, hash01(i, 304));
                let f = (t * psp * 0.9 + i as f32 / 110.0) % 1.0;
                let d = pd * (1.0 - f);
                let a2 = pa + f * 1.6;
                fx_line(g, cx + a2.cos() * d, cy + a2.sin() * d * 0.9, cx + (a2 - 0.07).cos() * d * 1.07, cy + (a2 - 0.07).sin() * d * 1.07 * 0.9, (0.8 + f * 1.2) * s, mix_color(0xA064FF, 0xE6C8FF, ph * 0.5 + f * 0.5), (0.15 + 0.7 * f) * ch);
            }
        }
        let gr = if t < 2.3 { ez((t - 0.2) / 1.9) } else { 1.0 };
        let mut r = m * (0.03 + 0.17 * gr) * (1.0 + 0.04 * (t * 22.0).sin());
        let (mut ox, mut oy) = (cx, cy);
        let launch = ez((t - 2.35) / 0.55);
        if (1.9..2.35).contains(&t) {
            r *= 1.0 - 0.35 * ez((t - 1.9) / 0.45);
        }
        if t >= 2.35 {
            ox = cx + (tx - cx) * launch * launch;
            oy = cy + (ty - cy) * launch * launch;
            r = m * 0.1 * (1.0 - 0.4 * launch);
        }
        if t < 2.35 {
            for (ri, r0) in [m * (0.55 - 0.3 * ez(t / 1.8)), m * (0.42 - 0.22 * ez(t / 1.8))].iter().enumerate() {
                for i in 0..80u32 {
                    let a = i as f32 / 80.0 * TAU + t * if ri == 1 { -0.7 } else { 0.5 };
                    let al = (0.25 + 0.6 * (i as f32 * 1.7 + t * 4.0).sin().abs()) * ch;
                    g.circle(cx + a.cos() * r0, cy + a.sin() * r0, if i % 7 == 0 { 3.0 * s } else { 1.4 * s }, rgba(0xEBD2FF, al));
                }
            }
        }
        if t > 0.5 && t < 2.0 {
            for k in 0..3u32 {
                let wv = ((t - 0.5) * 0.9 + k as f32 / 3.0) % 1.0;
                fx_ring(g, cx, cy, r + wv * m * 0.5, 64, 2.0 * s, 0xC896FF, 0.5 * (1.0 - wv));
            }
        }
        if t >= 2.35 {
            for i in 1..=22u32 {
                let fe = (launch - i as f32 * 0.025).max(0.0);
                let ee = fe * fe;
                let (px, py) = (cx + (tx - cx) * ee, cy + (ty - cy) * ee);
                let (hx, hy) = ((i as f32 * 0.9 + t * 20.0).cos() * 14.0 * (1.0 - ee) * s * 0.6, (i as f32 * 0.9 + t * 20.0).sin() * 14.0 * (1.0 - ee) * s * 0.6);
                g.circle(px + hx, py + hy, (r * (1.0 - i as f32 * 0.035)).max(0.5), rgba(0xBE78FF, 0.5 * (1.0 - i as f32 / 23.0)));
                g.circle(px - hx, py - hy, (r * 0.4 * (1.0 - i as f32 * 0.035)).max(0.5), rgba(0xFFFFFF, 0.3 * (1.0 - i as f32 / 23.0)));
            }
        }
        let rl = gr * if t < 2.35 { 1.0 } else { 1.0 - launch * 0.6 };
        for i in 0..24u32 {
            let ra = i as f32 / 24.0 * TAU + t * 0.9;
            let len = r * if i % 3 == 0 { 7.0 } else if i % 2 == 1 { 3.4 } else { 5.0 } * rl * (1.0 + if t > 1.9 && t < 2.35 { 3.0 } else { 0.0 });
            let wd = if i % 3 == 0 { 2.0 * s } else { 1.2 * s };
            for (a0, a1, al) in [(0.0f32, 0.3f32, 0.9f32), (0.3, 0.65, 0.5), (0.65, 1.0, 0.2)] {
                fx_line(g, ox + ra.cos() * len * a0, oy + ra.sin() * len * a0, ox + ra.cos() * len * a1, oy + ra.sin() * len * a1, wd, mix_color(0xFFFFFF, 0x9650FF, a1), al);
            }
        }
        for (f, al, c) in [(1.8f32, 0.16f32, 0x7828FFu32), (1.4, 0.3, 0x9646FF), (1.0, 0.65, 0xC896FF), (0.62, 0.95, 0xEBC8FF), (0.3, 1.0, 0xFFFFFF)] {
            g.circle(ox, oy, (r * f).max(0.5), rgba(c, al));
        }
        if t >= 2.9 {
            let s2 = t - 2.9;
            for k in 0..4u32 {
                fx_ring(g, tx, ty, s2 * (m * 0.9 + k as f32 * 90.0), 72, (4.0 - k as f32 * 0.8) * s, if k % 2 == 1 { 0xFFFFFF } else { 0xC88CFF }, (1.0 - s2 / (0.9 + k as f32 * 0.15)).max(0.0));
            }
            for k in 0..14u32 {
                let (mut px, mut py, mut a, mut l) = (tx, ty, hash01(k, 311) * TAU, 0.0f32);
                let pr = (s2 / 0.35).min(1.0);
                let n = (7.0 * pr).ceil() as u32;
                for j in 0..n {
                    a += (hash01(k * 7 + j, 312) - 0.5) * 0.9;
                    l += m * (0.05 + hash01(k * 9 + j, 313) * 0.06);
                    let (nx, ny) = (tx + a.cos() * l, ty + a.sin() * l);
                    fx_line(g, px, py, nx, ny, 1.4 * s, 0xFFEBFF, (1.0 - s2 / 0.9).max(0.0));
                    px = nx;
                    py = ny;
                }
            }
            for i in 0..70u32 {
                let (da, dv, dsz, dh) = (hash01(i, 321) * TAU, 80.0 + hash01(i, 322) * 420.0, 2.0 + hash01(i, 323) * 5.0, hash01(i, 324));
                g.circle(tx + da.cos() * dv * s2 * 0.6, ty + da.sin() * dv * s2 * 0.6 + 260.0 * s2 * s2 * 0.6, (dsz * (1.0 - s2 / 1.6) * s).max(0.4), rgba(mix_color(0xB478FF, 0xEBBEFF, dh), (1.0 - s2 / 1.4).max(0.0)));
            }
            g.rect(0.0, 0.0, w, h, rgba(0xFFFFFF, (1.0 - s2 / 0.45).max(0.0)));
        }
        if t > 0.5 && t < 2.5 {
            let tp = ez((t - 0.5) / 0.35);
            let al = tp * if t > 2.1 { (1.0 - (t - 2.1) / 0.4).max(0.0) } else { 1.0 };
            let fs = (w * 0.1).min(44.0 * s) * (0.8 + 0.2 * tp);
            if t < 0.9 && hash01((t * 60.0) as u32, 331) > 0.4 {
                self.text_outline(ctx, g, "ORBE VIOLETTE", cx - 5.0, h * 0.17, fs, 0xFF3CC8, al * 0.7, w - 6.0);
                self.text_outline(ctx, g, "ORBE VIOLETTE", cx + 5.0, h * 0.17, fs, 0x3CC8FF, al * 0.7, w - 6.0);
            }
            self.text_outline(ctx, g, "ORBE VIOLETTE", cx, h * 0.17, fs, 0xF3E2FF, al, w - 6.0);
        }
        if t >= 3.0 {
            let ap = ez((t - 3.0) / 0.2);
            let al = ap * if t > 3.9 { (1.0 - (t - 3.9) / 0.4).max(0.0) } else { 1.0 };
            self.text_outline(ctx, g, "ANÉANTI", cx, h * 0.7, (w * 0.14).min(60.0 * s) * (2.4 - 1.4 * ap), 0xFFFFFF, al, w - 6.0);
        }
        let bh = h * 0.09 * ez(t / 0.5) * if t > 3.8 { (1.0 - (t - 3.8) / 0.6).max(0.0) } else { 1.0 };
        g.rect(0.0, 0.0, w, bh.max(0.0), rgb(0x000000));
        g.rect(0.0, h - bh.max(0.0), w, bh.max(0.0) + 1.0, rgb(0x000000));
    }

    // ---- musique du château Éclipse (lecteur externe : mpv, ffplay ou vlc) -------------------
    fn music_update(&mut self) {
        let want = self.music_on && self.is_ec() && self.screen != Screen::Video;
        if let Some(c) = self.music.as_mut() {
            if let Ok(Some(_)) = c.try_wait() {
                self.music = None;
            }
        }
        if !want {
            if let Some(mut c) = self.music.take() {
                let _ = c.kill();
                let _ = c.wait();
            }
            return;
        }
        if self.music.is_none() && self.music_fail < 2 {
            let path = asset_path("assets/music.m4a");
            let cands: [(&str, &[&str]); 3] = [
                ("mpv", &["--no-video", "--really-quiet", "--loop=inf"]),
                ("ffplay", &["-nodisp", "-loglevel", "quiet", "-loop", "0"]),
                ("cvlc", &["--loop", "--quiet", "--no-video"]),
            ];
            for (cmd, args) in cands {
                if let Ok(c) = std::process::Command::new(cmd).args(args).arg(&path).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn() {
                    self.music = Some(c);
                    return;
                }
            }
            self.music_fail += 1;
        }
    }

    // ---- écran Progrès : succès, quêtes, talents -----------------------------------------------
    fn prog_rows(&self) -> Vec<PRow> {
        let mut v = Vec::new();
        match self.prog_tab {
            0 => {
                for i in 0..ACH.len() {
                    let cur = self.ach_cur(ACH[i].1);
                    let done = self.ach[i];
                    v.push(PRow {
                        title: ACH[i].0.to_string(),
                        detail: if done { "Débloqué".to_string() } else { format!("{} / {} {}", fmt(cur.min(ACH[i].2)), fmt(ACH[i].2), ACH[i].3) },
                        right: if done { "OK".into() } else { "+2 %".into() },
                        frac: if done { 1.0 } else { (cur / ACH[i].2).clamp(0.0, 1.0) as f32 },
                        done,
                        act: 0,
                        idx: i,
                    });
                }
            }
            1 => {
                for (i, q) in self.q.iter().enumerate() {
                    let target = QP[q.0].1;
                    v.push(PRow { title: QP[q.0].2.to_string(), detail: format!("{} / {}", q.1, target), right: if q.2 { "OK".into() } else { String::new() }, frac: q.1 as f32 / target as f32, done: q.2, act: 0, idx: i });
                }
                let (d, r) = match self.chest {
                    2 => ("Déjà ouvert aujourd’hui", ""),
                    1 => ("20 min de production + 1 point de talent", "Ouvrir"),
                    _ => ("Termine les 3 quêtes", ""),
                };
                v.push(PRow { title: "Coffre du jour".into(), detail: d.into(), right: r.into(), frac: if self.chest > 0 { 1.0 } else { 0.0 }, done: self.chest == 2, act: 1, idx: 0 });
            }
            _ => {
                for i in 0..TAL.len() {
                    let l = self.tl[i];
                    let c = TAL[i].3 + l;
                    v.push(PRow {
                        title: TAL[i].0.to_string(),
                        detail: TAL[i].1.to_string(),
                        right: if l >= TAL[i].2 { format!("{}/{} MAX", l, TAL[i].2) } else { format!("{}/{} · {}", l, TAL[i].2, c) },
                        frac: l as f32 / TAL[i].2 as f32,
                        done: l >= TAL[i].2,
                        act: 2,
                        idx: i,
                    });
                }
            }
        }
        v
    }

    fn draw_progress(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = self.theme_for(ctx);
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        g.clear(t.bg);
        let tabs_y = 44.0 * s;
        let top = tabs_y + 34.0 * s + 22.0 * s;
        let row_h = 60.0 * s;
        let rows = self.prog_rows();
        self.prog_scroll.set_content(rows.len() as f32 * row_h + 8.0 * s, h - top);
        for (i, r) in rows.iter().enumerate() {
            let y = top + i as f32 * row_h - self.prog_scroll.pos;
            if y + row_h < top - 2.0 || y > h {
                continue;
            }
            let rc = Rect::new(8.0 * s, y + 3.0 * s, w - 16.0 * s, row_h - 6.0 * s);
            g.rrect(rc.x, rc.y, rc.w, rc.h, 10.0 * s, if r.done { t.accent } else { t.line });
            g.rrect(rc.x + 1.5 * s, rc.y + 1.5 * s, rc.w - 3.0 * s, rc.h - 3.0 * s, 9.0 * s, t.surface);
            ctx.text_mid(g, &r.title, rc.x + 12.0 * s, rc.y + rc.h * 0.26, &Text::new(14.5 * s).bold().color(t.text).left().max_width(rc.w * 0.66));
            ctx.text_mid(g, &r.detail, rc.x + 12.0 * s, rc.y + rc.h * 0.55, &Text::new(10.5 * s).color(t.subtext).left().max_width(rc.w * 0.7));
            ctx.text_mid(g, &r.right, rc.right() - 12.0 * s, rc.y + rc.h * 0.4, &Text::new(13.0 * s).bold().color(t.accent).right().max_width(rc.w * 0.28));
            let bw = rc.w - 24.0 * s;
            g.rrect(rc.x + 12.0 * s, rc.bottom() - 13.0 * s, bw, 5.0 * s, 2.5 * s, t.line);
            if r.frac > 0.0 {
                g.rrect(rc.x + 12.0 * s, rc.bottom() - 13.0 * s, (bw * r.frac).max(5.0 * s), 5.0 * s, 2.5 * s, t.accent);
            }
        }
        self.hdr(ctx, g, "Progrès", Some("Retour"));
        g.rect(0.0, tabs_y, w, 34.0 * s + 22.0 * s, t.bg);
        g.rect(0.0, tabs_y, w, 34.0 * s, t.surface);
        for (i, name) in ["Succès", "Quêtes", "Talents"].iter().enumerate() {
            let on = self.prog_tab as usize == i;
            let tw = w / 3.0;
            ctx.text_mid(g, name, tw / 2.0 + i as f32 * tw, tabs_y + 17.0 * s, &Text::new(13.5 * s).bold().color(if on { t.accent } else { t.subtext }).center());
            if on {
                g.rect(i as f32 * tw + 10.0 * s, tabs_y + 31.0 * s, tw - 20.0 * s, 3.0 * s, t.accent);
            }
        }
        let sum = match self.prog_tab {
            0 => format!("{} / {} succès · bonus ×{} sur tout", self.ach.iter().filter(|b| **b).count(), ACH.len(), fmt_mult(self.ach_mult())),
            1 => "Quêtes du jour : +2 min de production chacune".to_string(),
            _ => format!("Points de talent : {} · +3 par Big Bang, +1 par coffre", self.tp),
        };
        ctx.text_mid(g, &sum, w / 2.0, tabs_y + 34.0 * s + 11.0 * s, &Text::new(11.0 * s).color(t.subtext).center().max_width(w - 8.0 * s));
    }

    // ---- capacités : Le Vide (trou noir), Extension du territoire (Éclipse), Avancement temporel (Temps), Orbe violette ----
    /// Capacités du château équipé (indices dans AB).
    fn abilities(&self) -> &'static [usize] {
        match self.skin {
            6 => &[0],
            8 | 10 => &[1, 3],
            11 => &[4],
            9 => &[2],
            _ => &[],
        }
    }
    fn ab_times(&self, i: usize) -> (f64, f64) {
        match i {
            0 => (self.vide_until, self.vide_cd),
            1 => (self.ext_until, self.ext_cd),
            2 => (0.0, self.temps_cd),
            4 => (self.sun_until, self.sun_cd),
            _ => (0.0, self.orb_cd),
        }
    }
    fn ability_row(&self, j: usize) -> (String, String, String, bool, bool) {
        let i = self.abilities().get(j).copied().unwrap_or(0);
        let (until, cd) = self.ab_times(i);
        let c = self.clock;
        let a = &AB[i];
        let eye = if self.skin == 10 && a.3 > 0.0 { " · recharge ÷10 (Œil absolu)" } else { "" };
        if i == 1 && self.inf_dom {
            (a.0.into(), format!("ACTIF À L’INFINI · {} sur tout", if self.skin == 10 { "×10 milliards" } else { "×1 milliard" }), "∞".into(), true, false)
        } else if c < until {
            (a.0.into(), format!("ACTIF · {}{}", a.5, eye), fmt_t(until - c), true, false)
        } else if c < cd {
            (a.0.into(), "Recharge en cours".into(), fmt_t(cd - c), false, false)
        } else if i == 3 && self.boss.is_none() {
            (a.0.into(), format!("{}{}", a.5, eye), "Aucun boss".into(), false, false)
        } else {
            (a.0.into(), format!("{}{}", a.5, eye), "Activer".into(), true, false)
        }
    }
    fn activate_ability(&mut self, ctx: &Ctx, j: usize) {
        let Some(&i) = self.abilities().get(j) else { return };
        if i == 1 && self.inf_dom {
            ctx.toast("L’Extension du territoire est déjà infinie");
            return;
        }
        let (until, cd) = self.ab_times(i);
        let c = self.clock;
        if c < until {
            ctx.toast("Déjà actif");
            return;
        }
        if c < cd {
            ctx.toast(format!("Recharge : {}", fmt_t(cd - c)));
            return;
        }
        let a = &AB[i];
        let rm = (1.0 - 0.1 * self.tl[4] as f64) * if self.skin == 10 { 0.1 } else { 1.0 };
        let (nu, nc) = (c + a.2, c + a.2 + a.3 * rm);
        if i == 3 {
            let Some(b) = &self.boss else {
                ctx.toast("Aucun boss à viser");
                return;
            };
            let _ = b;
            self.orb_cd = c + a.3 * rm;
            self.orb_target = {
                let (bx, by, _) = self.boss_pos(ctx);
                (bx, by)
            };
            self.orb_t = 0.0;
            self.screen = Screen::OrbFx;
        } else if i == 2 {
            let gain = self.income_ps() * 300.0 * 1.5f64.powi(self.tl[5] as i32);
            self.pts += gain;
            self.total += gain;
            self.temps_cd = c + a.3 * rm;
            self.time_gain = gain;
            self.time_t = 0.0;
            self.screen = Screen::TimeFx;
        } else if i == 0 {
            self.vide_until = nu;
            self.vide_cd = nc;
            self.void_t = 0.0;
            self.screen = Screen::VoidFx;
        } else if i == 4 {
            self.sun_until = nu;
            self.sun_cd = nc;
            self.sun_t = 0.0;
            self.screen = Screen::SunFx;
        } else {
            self.ext_until = nu;
            self.ext_cd = nc;
            self.dom_t = 0.0;
            self.screen = Screen::DomainFx;
        }
        self.bump(ctx, 4, 1);
        self.save();
    }

    // ---- thème par château (fond + boutons) : galaxie, galaxie MAX, trou noir, Éclipse -----
    fn has_theme(&self) -> bool {
        matches!(self.skin, 4..=11)
    }
    fn theme_for(&self, ctx: &Ctx) -> Theme {
        let mut t = ctx.theme;
        let set = |t: &mut Theme, bg: u32, surf: u32, text: u32, sub: u32, acc: u32, line: u32, press: u32| {
            t.bg = rgb(bg);
            t.surface = rgb(surf);
            t.text = rgb(text);
            t.subtext = rgb(sub);
            t.accent = rgb(acc);
            t.line = rgb(line);
            t.press = rgb(press);
        };
        match self.skin {
            4 => set(&mut t, 0x0B0320, 0x2A1160, 0xF3E9FF, 0xB69BE0, 0xD9A0FF, 0x5B3A9A, 0x3E1B8A),
            5 => set(&mut t, 0x1A0F05, 0x3A2208, 0xFFF3D6, 0xD9B97A, 0xFFD24D, 0x7A5A1A, 0x5A3A0A),
            6 => set(&mut t, 0x040201, 0x160C06, 0xFFEEDD, 0xC49A76, 0xFFB347, 0x6A3A18, 0x3A1E0C),
            7 => set(&mut t, 0x02100A, 0x0A2616, 0xE6FFF0, 0x8FD9AA, 0x39FF88, 0x1F6A3F, 0x124A2A),
            9 => set(&mut t, 0x04121A, 0x082A36, 0xE8F6F2, 0x8FB8B0, 0x3FE0C8, 0x7A6430, 0x0E3E4A),
            11 => set(&mut t, 0x1E0600, 0x30100A, 0xFFF4E0, 0xE0A878, 0xFFB12E, 0x7A3A10, 0x5A2A08),
            8 | 10 => set(&mut t, 0x04051A, 0x0C1038, 0xEEF3FF, 0x93A4D6, 0x8FB8FF, 0x2E3A7A, 0x1A2260),
            _ => {}
        }
        t
    }
    fn hdr(&self, ctx: &Ctx, g: &mut Gfx, title: &str, back: Option<&str>) -> ui::Header {
        if !self.has_theme() {
            return ui::header(ctx, g, title, back);
        }
        let t = self.theme_for(ctx);
        let s = ctx.scale();
        let w = ctx.width();
        let h = 44.0 * s;
        g.rect(0.0, 0.0, w, h, t.surface);
        g.rect(0.0, h - 1.5, w, 1.5, t.accent);
        ctx.text_mid(g, title, w / 2.0, h / 2.0, &Text::new(17.0 * s).bold().color(t.text).center().max_width(w * 0.56));
        let mut area = None;
        if let Some(label) = back {
            ctx.text_mid(g, &format!("‹ {label}"), 10.0 * s, h / 2.0, &Text::new(15.0 * s).color(t.accent).left().max_width(w * 0.3));
            area = Some(Rect::new(0.0, 0.0, w * 0.32, h));
        }
        ui::Header { height: h, back: area }
    }
    fn btn(&self, ctx: &Ctx, g: &mut Gfx, r: Rect, label: &str, style: ButtonStyle, pressed: bool) {
        if !self.has_theme() {
            return ui::button(ctx, g, r, label, style, pressed);
        }
        let t = self.theme_for(ctx);
        let s = ctx.scale();
        let rad = r.h * 0.28;
        let (fill, fg, glow) = match (style, self.skin) {
            (ButtonStyle::Danger, _) => (0xC8243C, 0xFFFFFF, 0xFF3B5C),
            (ButtonStyle::Secondary, _) => (0x000000, 0xFFFFFF, 0x000000),
            (_, 4) => (0x7B3BE0, 0xFFFFFF, 0xD9A0FF),
            (_, 5) => (0xD9A020, 0x2A1600, 0xFFD24D),
            (_, 6) => (0x8A3A0A, 0xFFE0B8, 0xFFB347),
            (_, 7) => (0x1E9A58, 0xFFFFFF, 0x39FF88),
            (_, 9) => (0x0E6A66, 0xFFFFFF, 0xF0C060),
            (_, 11) => (0xE0661A, 0x2A0A00, 0xFFD84A),
            _ => (0x7A5FD6, 0xFFFFFF, 0x8FB8FF),
        };
        let secondary = matches!(style, ButtonStyle::Secondary);
        let edge = if secondary { t.accent } else { rgb(glow) };
        g.rrect(r.x - 3.0 * s, r.y - 3.0 * s, r.w + 6.0 * s, r.h + 6.0 * s, rad + 3.0 * s, rgba(glow, 0.22));
        g.rrect(r.x - 1.0, r.y - 1.0, r.w + 2.0, r.h + 2.0, rad + 1.0, edge);
        let body = if secondary { t.surface } else { rgb(fill) };
        g.rrect(r.x, r.y, r.w, r.h, rad, if pressed { shade(body, 0.85) } else { body });
        if self.is_ec() && !secondary && !matches!(style, ButtonStyle::Danger) {
            // dégradé rouge -> bleu
            let n = 24u32;
            for i in 0..n {
                let f = i as f32 / (n - 1) as f32;
                let c = mix_color(0xFF3B5C, 0x4D8DFF, f);
                let x0 = r.x + rad + (r.w - 2.0 * rad) * i as f32 / n as f32;
                g.rect(x0, r.y, (r.w - 2.0 * rad) / n as f32 + 1.0, r.h, rgb(c));
            }
            g.circle(r.x + rad, r.y + r.h / 2.0, r.h / 2.0 * 0.98, rgb(0xFF3B5C));
            g.circle(r.right() - rad, r.y + r.h / 2.0, r.h / 2.0 * 0.98, rgb(0x4D8DFF));
        }
        let fgc = if secondary { t.accent } else { rgb(fg) };
        let (cx, cy) = r.center();
        ctx.text_mid(g, label, cx, cy, &Text::new(r.h * 0.42).bold().color(fgc).center().max_width(r.w - 12.0 * s));
    }
    fn tgl(&self, ctx: &Ctx, g: &mut Gfx, r: Rect, on: bool) {
        if !self.has_theme() {
            return ui::toggle(ctx, g, r, on);
        }
        let t = self.theme_for(ctx);
        g.rrect(r.x, r.y, r.w, r.h, r.h / 2.0, if on { t.accent } else { t.line });
        let k = r.h - 4.0;
        let kx = if on { r.right() - 2.0 - k } else { r.x + 2.0 };
        g.rrect(kx, r.y + 2.0, k, k, k / 2.0, rgb(0xFFFFFF));
    }

    fn ink_blob(&self, g: &mut Gfx, x: f32, y: f32, r: f32, seed: u32, col: u32, al: f32) {
        use std::f32::consts::TAU;
        g.circle(x, y, r, rgba(col, al));
        for i in 0..7u32 {
            let a = hash01(seed + i, 91) * TAU;
            let d = r * (0.55 + hash01(seed + i, 92) * 0.5);
            g.circle(x + a.cos() * d, y + a.sin() * d, r * (0.28 + hash01(seed + i, 93) * 0.3), rgba(col, al));
        }
        for i in 0..6u32 {
            let a = hash01(seed + i, 94) * TAU;
            let d = r * (1.15 + hash01(seed + i, 95) * 0.8);
            g.circle(x + a.cos() * d, y + a.sin() * d, r * (0.05 + hash01(seed + i, 96) * 0.1), rgba(col, al));
        }
    }

    /// Vortex sombre à anneau lumineux (blanc-or en bas à gauche, turquoise à droite) et fumée.
    fn draw_vortex(&self, g: &mut Gfx, cx: f32, cy: f32, r: f32, al: f32) {
        use std::f32::consts::{PI, TAU};
        let t = self.anim_t;
        if al <= 0.0 || r < 2.0 {
            return;
        }
        g.circle(cx, cy, r * 1.25, rgba(0x081018, 0.5 * al));
        g.circle(cx, cy, r * 1.05, rgba(0x081018, 0.9 * al));
        g.circle(cx, cy, r * 0.78, rgba(0x000000, al));
        for i in 0..56u32 {
            let u = (i as f32 * 0.618_034) % 1.0;
            let a = i as f32 * 2.4 + u * 5.0 + t * (0.5 + 1.4 * (1.0 - u));
            let rr = r * (0.8 + 0.75 * u);
            g.circle(cx + a.cos() * rr, cy + a.sin() * rr * 0.92, r * (0.06 + 0.07 * u), rgba(0x96AADC, (0.05 + 0.1 * (1.0 - u)) * al));
        }
        for (a0, a1) in [(PI * 0.55, PI * 1.35), (-PI * 0.45, PI * 0.35)] {
            for k in 0..40u32 {
                let f = k as f32 / 39.0;
                let a = a0 + (a1 - a0) * f + t * 0.2;
                let col = if (a0 > 0.0) == (f < 2.0) && a0 > 0.0 { mix_color(0xFFF0C8, 0x7896C8, f) } else { mix_color(0x7896C8, 0x78F0E6, f) };
                g.circle(cx + a.cos() * r * 0.86, cy + a.sin() * r * 0.86, r * 0.05, rgba(col, 0.95 * al));
            }
        }
        for k in 0..72u32 {
            let a = k as f32 / 72.0 * TAU;
            g.circle(cx + a.cos() * r * 0.95, cy + a.sin() * r * 0.95, r * 0.012, rgba(0xE6F0FF, 0.5 * al));
        }
    }

    /// Fond animé du château (galaxie, galaxie MAX, trou noir, Éclipse). Reste derrière toute l'interface.
    fn draw_theme_bg(&self, ctx: &Ctx, g: &mut Gfx, l: &Layout) {
        use std::f32::consts::TAU;
        let (w, h) = (ctx.width(), ctx.height());
        let s = ctx.scale();
        let t = self.anim_t;
        let (cx, cy) = (l.cx, l.cy);
        let m = w.max(h);
        let dom = self.ext_on();
        let vd = self.skin == 6 && self.clock < self.vide_until;
        match self.skin {
            4 | 5 => {
                let mx = self.skin == 5;
                for (f, al, c) in [(0.85f32, 0.12f32, if mx { 0x7A4A10u32 } else { 0x4B1FA0 }), (0.6, 0.14, if mx { 0xA86A00 } else { 0x6A2FD0 }), (0.35, 0.16, if mx { 0xD99A20 } else { 0x9A4DFF })] {
                    g.circle(cx, cy, m * f, rgba(c, al));
                }
                for i in 0..120u32 {
                    let u = hash01(i, 101).powf(0.8);
                    let a = hash01(i, 102) * TAU + u * 4.0 + t * 0.12 / (0.25 + u);
                    let rr = u * m * 0.7;
                    let col = if mx { hsv_hex((40.0 + hash01(i, 103) * 300.0 + t * 20.0) % 360.0, 0.7, 1.0) } else { hsv_hex(250.0 + hash01(i, 103) * 80.0, 0.6, 1.0) };
                    g.circle(cx + a.cos() * rr, cy + a.sin() * rr * 0.6, (1.0 + hash01(i, 104) * 1.6) * s, rgba(col, 0.1 + 0.22 * (1.0 - u)));
                }
                for i in 0..60u32 {
                    let tw = 0.5 + 0.5 * (t * (0.5 + hash01(i, 105) * 2.0) + hash01(i, 106) * TAU).sin();
                    g.circle(hash01(i, 107) * w, hash01(i, 108) * h, (0.6 + hash01(i, 109) * 1.2) * s, rgba(0xFFFFFF, 0.15 + 0.7 * tw));
                }
                if mx {
                    let f = (t * 0.35) % 1.0;
                    let sx = w * (0.2 + 0.6 * (((t * 0.35).floor() * 0.37) % 1.0));
                    for k in 0..10u32 {
                        let d = k as f32 * 6.0 * s;
                        g.circle(sx + f * w * 0.4 - d, h * 0.1 + f * h * 0.25 - d * 0.6, (2.6 - k as f32 * 0.2) * s, rgba(0xFFE696, (1.0 - f) * 0.7));
                    }
                }
            }
            6 => {
                let warm = if vd { 0xFFB070 } else { 0xFF8C32 };
                for (f, al) in [(0.7f32, 0.08f32), (0.5, 0.12), (0.3, 0.18), (0.16, 0.25)] {
                    g.circle(cx, cy, m * f, rgba(warm, al * if vd { 1.1 } else { 1.0 }));
                }
                for e in 0..3u32 {
                    let rx = w * (0.5 + e as f32 * 0.12);
                    let ry = w * (0.12 + e as f32 * 0.03);
                    for k in 0..70u32 {
                        let a = k as f32 / 70.0 * TAU;
                        let (px, py) = (a.cos() * rx, a.sin() * ry);
                        g.circle(cx + px * 0.99 + py * 0.12, cy + py - px * 0.12, (if vd { 3.0 } else { 2.0 }) * s, rgba(0xFFAA50 - e * 0x001E0F, (if vd { 0.4 } else { 0.22 }) - e as f32 * 0.05));
                    }
                }
                for i in 0..80u32 {
                    let f = (t * (0.03 + hash01(i, 111) * 0.07) * if vd { 3.0 } else { 1.0 } + hash01(i, 112)) % 1.0;
                    let rr = m * 0.7 * (1.0 - f);
                    let a = hash01(i, 113) * TAU + f * 5.0;
                    g.circle(cx + a.cos() * rr, cy + a.sin() * rr * 0.45, (1.0 + hash01(i, 114) * 1.6) * s, rgba(mix_color(0xFFC896, 0xFF6432, f), 0.65 * (1.0 - f) * if vd { 1.0 } else { 0.7 }));
                }
                if vd {
                    for i in 0..30u32 {
                        let a = i as f32 / 30.0 * TAU + t * 0.2;
                        let f = (t * 0.9 + i as f32 * 0.173) % 1.0;
                        let r1 = m * 0.75 * (1.0 - f);
                        for k in 0..5u32 {
                            let rr = (r1 - k as f32 * m * 0.025).max(0.0);
                            g.circle(cx + a.cos() * rr, cy + a.sin() * rr, 2.0 * s, rgba(0xFFF0DC, 0.35 * (1.0 - f)));
                        }
                    }
                }
            }
            7 => {
                for (f, al) in [(0.8f32, 0.07f32), (0.55, 0.1), (0.3, 0.16)] {
                    g.circle(cx, cy, m * f, rgba(0x1EA05A, al));
                }
                let hy = h * 0.62;
                for i in 0..14u32 {
                    let f = ((i as f32 + t * 0.6) % 14.0) / 14.0;
                    g.rect(0.0, (hy + (h - hy) * f * f).floor().min(h - 3.0), (w - 1.0).floor(), 2.0, rgba(0x39FF88, 0.14));
                }
                for i in -6i32..=6 {
                    self.dot_line(g, w / 2.0 + i as f32 * w * 0.03, hy, w / 2.0 + i as f32 * w * 0.16, h, 0.7 * s, 0x39FF88, 0.16);
                }
                for k in 1..5u32 {
                    for q in 0..48u32 {
                        let a = q as f32 / 48.0 * TAU;
                        g.circle(cx + a.cos() * m * 0.14 * k as f32, cy + a.sin() * m * 0.14 * k as f32, 0.8 * s, rgba(0x39FF88, 0.14));
                    }
                }
                let a0 = t * 1.4;
                for i in 0..12u32 {
                    let a = a0 - i as f32 * 0.06;
                    for j in 1..9u32 {
                        g.circle(cx + a.cos() * m * 0.07 * j as f32, cy + a.sin() * m * 0.07 * j as f32, 1.8 * s, rgba(0x3CFF8C, 0.1 * (1.0 - i as f32 / 12.0)));
                    }
                }
                for i in 0..60u32 {
                    let f = (t * (0.03 + hash01(i, 151) * 0.07) * 0.9 + hash01(i, 152)) % 1.0;
                    let sz = (1.5 + hash01(i, 153) * 2.0) * s;
                    g.circle(hash01(i, 154) * w, h * (1.0 - f), sz * 0.7, rgba(0x78FFAA, 0.7 * (1.0 - f)));
                }
            }
            9 => {
                let gears: [(f32, f32, f32, u32, f32, u32); 4] = [(0.1, 0.2, 0.22, 18, 0.12, 0xF0C060), (0.92, 0.55, 0.28, 22, -0.09, 0x3FE0C8), (0.2, 0.9, 0.2, 16, -0.14, 0xF0C060), (0.75, 0.12, 0.12, 10, 0.2, 0x3FE0C8)];
                for (gx, gy, gr, teeth, sp, col) in gears {
                    let (x0, y0, rr) = (gx * w, gy * h, gr * m);
                    for k in 0..teeth {
                        let a = t * sp + k as f32 * TAU / teeth as f32;
                        g.circle(x0 + a.cos() * rr, y0 + a.sin() * rr, rr * 0.09, rgba(col, 0.1));
                    }
                    for k in 0..48u32 {
                        let a = k as f32 / 48.0 * TAU;
                        g.circle(x0 + a.cos() * rr * 0.88, y0 + a.sin() * rr * 0.88, rr * 0.03, rgba(col, 0.09));
                    }
                }
                for i in 0..60u32 {
                    let a = i as f32 / 60.0 * TAU - TAU / 4.0;
                    let big = i % 5 == 0;
                    self.dot_line(g, cx + a.cos() * m * 0.5, cy + a.sin() * m * 0.5, cx + a.cos() * m * if big { 0.55 } else { 0.52 }, cy + a.sin() * m * if big { 0.55 } else { 0.52 }, if big { 1.6 * s } else { 0.8 * s }, 0xF0C060, if big { 0.3 } else { 0.14 });
                }
                for i in 0..3u32 {
                    let f = (t * 0.35 + i as f32 / 3.0) % 1.0;
                    for k in 0..60u32 {
                        let a = k as f32 / 60.0 * TAU;
                        let rr = m * 0.08 + m * 0.42 * f;
                        g.circle(cx + a.cos() * rr, cy + a.sin() * rr, 1.2 * s, rgba(0x3FE0C8, 0.3 * (1.0 - f)));
                    }
                }
                for i in 0..60u32 {
                    let f = (t * (0.03 + hash01(i, 155) * 0.05) + hash01(i, 156)) % 1.0;
                    g.circle(hash01(i, 157) * w, h * f, (1.0 + hash01(i, 158) * 1.6) * s, rgba(0xFFDC8C, 0.55 * (1.0 - f)));
                }
            }
            11 => {
                let on = self.sun_on();
                g.rect(0.0, 0.0, w, h, rgb(if on { 0x3A0E00 } else { 0x220700 }));
                for (f, al, c) in [(0.95f32, 0.3f32, 0x4A1400u32), (0.65, 0.3, 0x7A2A00), (0.4, 0.25, 0xFF8C28)] {
                    g.circle(cx, cy, m * f * if on { 1.2 } else { 1.0 }, rgba(c, (al * if on { 1.4 } else { 1.0 }).min(1.0)));
                }
                for i in 0..18u32 {
                    let a = i as f32 / 18.0 * std::f32::consts::TAU + t * 0.03;
                    fx_taper(g, cx + a.cos() * m * 1.2, cy + a.sin() * m * 1.2, cx, cy, m * 0.05, 0xFFB446, if on { 0.08 } else { 0.05 });
                }
                for i in 0..50u32 {
                    let f = (t * 0.06 * (0.5 + hash01(i, 521)) + hash01(i + 3, 522)) % 1.0;
                    g.circle(hash01(i + 11, 523) * w + (t + i as f32).sin() * 8.0 * s, h - f * h * 1.05, (1.0 + hash01(i + 13, 524) * 2.2) * s, rgba(mix_color(0xFF8C32, 0xFFE68C, hash01(i + 9, 525)), 0.8 * (1.0 - f)));
                }
                for i in 0..3u32 {
                    let x0 = w * (0.15 + 0.35 * i as f32);
                    let p = 0.5 + 0.5 * (t * 0.5 + i as f32 * 2.0).sin();
                    for q in 0..=30u32 {
                        let u = q as f32 / 30.0;
                        let (bx, by) = (x0 + u * m * 0.24, h - 4.0 * u * (1.0 - u) * m * (0.25 + 0.15 * p) * 0.5);
                        g.circle(bx, by, 3.0 * s, rgba(0xFF6E28, 0.18 + 0.12 * p));
                    }
                }
            }
            _ => {
                for (f, al) in [(0.9f32, 0.07f32), (0.65, 0.1), (0.4, 0.14)] {
                    g.circle(0.0, h * 0.35, m * f, rgba(0xFF3255, al * if dom { 0.6 } else { 1.0 }));
                    g.circle(w, h * 0.6, m * f, rgba(0x4682FF, al * if dom { 0.6 } else { 1.0 }));
                }
                let bucket = ((t / 0.12) as u32) % 5000;
                for k in 0..3u32 {
                    let seed = bucket * 13 + k * 101;
                    let left = k % 2 == 0;
                    let y0 = h * (0.1 + 0.8 * hash01(seed, 115));
                    let x1 = w * if left { 0.12 + 0.2 * hash01(seed, 116) } else { 0.88 - 0.2 * hash01(seed, 116) };
                    let y1 = y0 + (hash01(seed, 117) - 0.5) * h * 0.3;
                    draw_bolt(g, if left { 0.0 } else { w }, y0, x1, y1, seed, 7, h * 0.04, if left { 0xFF6E82 } else { 0x8CBEFF }, 0.7, 1.2 * s);
                }
                for i in 0..40u32 {
                    let f = (t * (0.03 + hash01(i, 118) * 0.07) * 0.6 + hash01(i, 119)) % 1.0;
                    let right = i % 2 == 1;
                    let x = if right { w - f * w * 0.5 } else { f * w * 0.5 };
                    g.circle(x, h * hash01(i, 120) + (t + hash01(i, 121) * 6.0).sin() * h * 0.05, (1.0 + hash01(i, 122) * 1.6) * s, rgba(if right { 0x78AAFF } else { 0xFF5A73 }, 0.5 * (1.0 - f)));
                }
                if dom {
                    self.draw_vortex(g, cx, cy, w.min(h) * 0.42, 1.0);
                    for i in 0..16u32 {
                        let side = i % 4;
                        let (rx, ry) = (hash01(i, 123), hash01(i, 124));
                        let (x, y) = match side {
                            0 => (rx, 0.0),
                            1 => (1.0, ry),
                            2 => (rx, 1.0),
                            _ => (0.0, ry),
                        };
                        let rr = (0.05 + hash01(i, 125) * 0.09) * w.min(h) * (0.9 + 0.1 * (t * 1.3 + hash01(i, 126) * TAU).sin());
                        self.ink_blob(g, x * w, y * h, rr, i * 7 + 3, 0xFFFFFF, 1.0);
                    }
                }
            }
        }
    }

    fn text_outline(&self, ctx: &Ctx, g: &mut Gfx, txt: &str, x: f32, y: f32, size: f32, fill: u32, al: f32, maxw: f32) {
        if al <= 0.0 {
            return;
        }
        let k = (size * 0.07).max(1.5);
        for (dx, dy) in [(-1.0f32, 0.0f32), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0), (-0.7, -0.7), (0.7, -0.7), (-0.7, 0.7), (0.7, 0.7)] {
            ctx.text_mid(g, txt, x + dx * k, y + dy * k, &Text::new(size).bold().color(rgba(0x000000, al)).center().max_width(maxw));
        }
        ctx.text_mid(g, txt, x, y, &Text::new(size).bold().color(rgba(fill, al)).center().max_width(maxw));
    }

    /// Extension du territoire (≈ 5,2 s) : lignes de vitesse, vortex sombre qui s'ouvre, encre blanche qui gicle
    /// depuis les bords, titre glitché, puis l'encre se retire.
    fn draw_domainfx(&mut self, ctx: &Ctx, g: &mut Gfx) {
        use std::f32::consts::TAU;
        let t = self.dom_t;
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let (cx, cy) = (w / 2.0, h * 0.46);
        let m = w.min(h);
        let fade = if t < 0.35 { 1.0 - t / 0.35 } else if t > 4.3 { ((t - 4.3) / 0.9).min(1.0) } else { 0.0 };
        if fade > 0.0 {
            self.draw_game(ctx, g);
            g.rect(0.0, 0.0, w, h, rgba(0x000000, 1.0 - fade));
        } else {
            g.clear(rgb(0x000000));
        }
        if t < 1.6 {
            for k in 0..54u32 {
                let a = k as f32 / 54.0 * TAU;
                let f = (t * 1.4 + k as f32 * 0.137) % 1.0;
                let r1 = m * (1.1 - f * 0.9);
                for j in 0..5u32 {
                    let rr = r1 + m * 0.18 * (1.0 - f) * j as f32 / 4.0;
                    g.circle(cx + a.cos() * rr, cy + a.sin() * rr, 1.2 * s, rgba(0xBED2FF, 0.28 * (1.0 - t / 1.6)));
                }
            }
        }
        let shrink = if t > 3.9 { (1.0 - (t - 3.9) / 1.0).max(0.0) } else { 1.0 };
        for i in 0..36u32 {
            let side = i % 4;
            let (rx, ry) = (hash01(i, 131), hash01(i, 132));
            let (x, y) = match side {
                0 => (rx * w, 0.0),
                1 => (w, ry * h),
                2 => (rx * w, h),
                _ => (0.0, ry * h),
            };
            let tx = cx + (hash01(i, 133) - 0.5) * w * 1.1;
            let ty = cy + (hash01(i, 134) - 0.5) * h;
            let r = m * (0.07 + hash01(i, 135) * 0.14);
            let off = 0.85 + i as f32 * 0.028;
            let p = ((t - off) / 0.55).clamp(0.0, 1.0);
            if p <= 0.0 {
                continue;
            }
            let e = 1.0 - (1.0 - p).powi(3);
            self.ink_blob(g, x + (tx - x) * e * 0.62, y + (ty - y) * e * 0.62, r * e * shrink, i * 11 + 5, 0xFFFFFF, 1.0);
        }
        let vp = ((t - 0.25) / 1.1).clamp(0.0, 1.0);
        let vb = {
            let c1 = 1.70158f32;
            1.0 + (c1 + 1.0) * (vp - 1.0).powi(3) + c1 * (vp - 1.0).powi(2)
        };
        self.draw_vortex(g, cx, cy, m * 0.36 * vb * shrink, (vp * 2.0).min(1.0));
        if t > 2.4 {
            let tp = ((t - 2.4) / 0.5).min(1.0);
            let fs = (w * 0.12).min(64.0 * s) * (0.8 + 0.2 * tp);
            let glitch = t < 2.75;
            let al = (tp * 1.6).min(1.0) * if t > 4.0 { (1.0 - (t - 4.0) / 0.7).max(0.0) } else { 1.0 };
            for (li, txt) in ["EXTENSION", "DU TERRITOIRE"].iter().enumerate() {
                let y = cy + if li == 1 { fs * 0.62 } else { -fs * 0.55 };
                let jx = if glitch { (hash01((t * 60.0) as u32 + li as u32, 136) - 0.5) * fs * 0.25 } else { 0.0 };
                if glitch {
                    ctx.text_mid(g, txt, cx + jx - 5.0 * s, y, &Text::new(fs).bold().color(rgba(0xFF2850, 0.8 * al)).center().max_width(w - 10.0));
                    ctx.text_mid(g, txt, cx + jx + 5.0 * s, y, &Text::new(fs).bold().color(rgba(0x3C8CFF, 0.8 * al)).center().max_width(w - 10.0));
                }
                self.text_outline(ctx, g, txt, cx + jx, y, fs, 0xFFFFFF, al, w - 10.0);
            }
            self.text_outline(ctx, g, "Tous les multiplicateurs ×1 milliard · 5:00", cx, cy + fs * 1.3, (fs * 0.26).max(11.0 * s), 0xFFFFFF, al, w - 12.0);
        }
        let fl = (1.0 - (t - 1.15).abs() / 0.14).max(0.0) + (1.0 - (t - 2.4).abs() / 0.1).max(0.0);
        if fl > 0.0 {
            g.rect(0.0, 0.0, w, h, rgba(0xFFFFFF, fl.min(1.0)));
        }
    }

    /// Le Vide (≈ 3,2 s) : tout est aspiré vers un point, l'anneau de lumière se resserre, flash, onde de choc, titre.
    fn draw_voidfx(&mut self, ctx: &Ctx, g: &mut Gfx) {
        use std::f32::consts::TAU;
        let t = self.void_t;
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let (cx, cy) = (w / 2.0, h * 0.46);
        let m = w.min(h);
        let fade = if t < 0.25 { 1.0 - t / 0.25 } else if t > 2.6 { ((t - 2.6) / 0.6).min(1.0) } else { 0.0 };
        if fade > 0.0 {
            self.draw_game(ctx, g);
            g.rect(0.0, 0.0, w, h, rgba(0x000000, 1.0 - fade));
        } else {
            g.clear(rgb(0x000000));
        }
        let con = (t / 1.2).min(1.0);
        let r = m * (0.55 * (1.0 - con * con) + 0.04);
        for k in 0..110u32 {
            let a = k as f32 * 2.399 + t * 0.6;
            let f = (t * 0.8 + k as f32 * 0.0913) % 1.0;
            let r1 = m * 1.1 * (1.0 - f * f);
            for j in 0..5u32 {
                let rr = (r1 - m * 0.14 * j as f32 / 4.0).max(0.0);
                g.circle(cx + a.cos() * rr, cy + a.sin() * rr * 0.55, 1.5 * s, rgba(mix_color(0xFFBE6E, 0xFF6E32, f), 0.5 * (1.0 - f) + 0.1));
            }
        }
        if t < 1.45 {
            for k in 0..90u32 {
                let a = k as f32 / 90.0 * TAU;
                g.circle(cx + a.cos() * r, cy + a.sin() * r, m * 0.018, rgba(0xFFE1AA, 0.9));
                g.circle(cx + a.cos() * r, cy + a.sin() * r, m * 0.034, rgba(0xFFA64D, 0.15));
            }
            g.circle(cx, cy, r * 0.94, rgb(0x000000));
        }
        if t > 1.2 {
            let s2 = t - 1.2;
            for k in 0..3u32 {
                let rr = s2 * (m * 0.9 + k as f32 * 70.0 * s);
                let al = (1.0 - s2 / 1.1).max(0.0);
                for q in 0..80u32 {
                    let a = q as f32 / 80.0 * TAU;
                    g.circle(cx + a.cos() * rr, cy + a.sin() * rr, (3.0 - k as f32 * 0.7) * s, rgba(mix_color(0xFFDCA0, 0xFFA064, k as f32 / 2.0), al));
                }
            }
            let pp = (s2 / 0.35).min(1.0);
            let al = pp * if t > 2.7 { (1.0 - (t - 2.7) / 0.5).max(0.0) } else { 1.0 };
            let fs = (w * 0.15).min(60.0 * s) * (0.7 + 0.3 * pp);
            self.text_outline(ctx, g, "LE VIDE", cx, cy, fs, 0xFFE6C0, al, w - 10.0);
            self.text_outline(ctx, g, "Multiplicateur du château ×100 · 3:00", cx, cy + fs * 0.7, (fs * 0.24).max(11.0 * s), 0xFFFFFF, al, w - 12.0);
        }
        let fl = (1.0 - (t - 1.2).abs() / 0.12).max(0.0);
        if fl > 0.0 {
            g.rect(0.0, 0.0, w, h, rgba(0xFFF0DC, fl));
        }
    }

    // ---- dessin --------------------------------------------------------------------------
    fn draw_game(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = self.theme_for(ctx);
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let mut l = self.layout(ctx);
        g.clear(t.bg);
        if self.has_theme() {
            self.draw_theme_bg(ctx, g, &l);
        }
        if self.demo_burst {
            self.demo_burst = false;
            self.burst(ctx, l.cx + 18.0 * s, l.cy - l.r * 0.3, 1.0);
            self.update_fx(0.16);
        }

        // liste (dessinée d'abord : le haut de l'écran la recouvre)
        let rows = match self.tab {
            Tab::Shop => self.tiers.len() + 2,
            Tab::Magic => MAGIC.len() + self.abilities().len(),
            Tab::Castles => CASTLES.len() + SKBUY.len(),
            Tab::Cursors => 2,
            Tab::Defense => 4,
        };
        self.scroll.set_content(rows as f32 * l.row_h + 8.0 * s, h - l.list_top);
        for row in 0..rows {
            let y = l.list_top + row as f32 * l.row_h - self.scroll.pos;
            if y + l.row_h < l.list_top || y > h || (self.has_theme() && y < l.list_top - 2.0) {
                continue;
            }
            let (title, detail, right, ok, locked) = match self.tab {
                Tab::Shop => self.shop_row(row),
                Tab::Magic => self.magic_row(row),
                Tab::Castles => self.castle_row(row),
                Tab::Cursors => self.cursor_row(row),
                Tab::Defense => self.defense_row(row),
            };
            let r = Rect::new(8.0 * s, y + 3.0 * s, w - 16.0 * s, l.row_h - 6.0 * s);
            if self.has_theme() {
                g.rrect(r.x, r.y, r.w, r.h, 10.0 * s, if ok { t.accent } else { t.line });
                g.rrect(r.x + 1.5 * s, r.y + 1.5 * s, r.w - 3.0 * s, r.h - 3.0 * s, 9.0 * s, if ok { t.surface } else { shade(t.surface, 0.9) });
                if self.is_ec() {
                    g.rrect(r.x, r.y + 6.0 * s, 3.0 * s, r.h - 12.0 * s, 1.5 * s, rgb(0xFF3B5C));
                    g.rrect(r.right() - 3.0 * s, r.y + 6.0 * s, 3.0 * s, r.h - 12.0 * s, 1.5 * s, rgb(0x4D8DFF));
                } else if self.skin == 7 {
                    g.rrect(r.x, r.y + 6.0 * s, 3.0 * s, r.h - 12.0 * s, 1.5 * s, rgb(0x39FF88));
                } else if self.skin == 9 {
                    g.rrect(r.x, r.y + 6.0 * s, 3.0 * s, r.h - 12.0 * s, 1.5 * s, rgb(0xF0C060));
                }
            } else {
                g.rrect(r.x, r.y, r.w, r.h, 10.0 * s, if ok { t.surface } else { shade(t.surface, 0.96) });
            }
            let tc = if ok { t.text } else { t.subtext };
            ctx.text_mid(g, &title, r.x + 12.0 * s, r.y + r.h * 0.32, &Text::new(15.0 * s).bold().color(tc).left().max_width(r.w * 0.62));
            let dc = if ok { t.accent } else { t.subtext };
            ctx.text_mid(g, &detail, r.x + 12.0 * s, r.y + r.h * 0.70, &Text::new(11.5 * s).color(dc).left().max_width(r.w * 0.74));
            let big = right.chars().count() <= 3 && !locked;
            let rs = Text::new(if big { 20.0 * s } else { 13.0 * s }).bold().color(if ok && !big { t.accent } else { t.subtext }).right().max_width(r.w * 0.26);
            ctx.text_mid(g, &right, r.right() - 12.0 * s, r.y + r.h / 2.0, &rs);
        }

        // zone du haut
        if !self.has_theme() {
            g.rect(0.0, 0.0, w, l.list_top, t.bg);
        }
        let head = self.hdr(ctx, g, "Conquis Cliqueur", None);
        for k in 0..3 {
            g.rrect(l.menu_btn.x + 14.0 * s, 14.0 * s + k as f32 * 7.0 * s, 20.0 * s, 3.0 * s, 1.5 * s, t.text);
        }
        let big = Text::new(30.0 * s).bold().color(t.text).center().max_width(w - 24.0 * s);
        ctx.text_mid(g, &fmt(self.pts), w / 2.0, head.height + 20.0 * s, &big);
        let hud = match self.abilities().iter().find(|&&i| self.clock < self.ab_times(i).0) {
            Some(&i) => format!(" · {} {}", AB[i].0, fmt_t(self.ab_times(i).0 - self.clock)),
            None => String::new(),
        };
        let combos = self.combo_list();
        let hud = if combos.is_empty() { hud } else { format!("{} · COMBO {} ×{}", hud, CMB[combos[0]].0, fmt_mult(self.combo_mult())) };
        ctx.text_mid(g, &format!("territoires conquis{}", hud), w / 2.0, head.height + 41.0 * s, &Text::new(12.0 * s).color(if hud.is_empty() { t.subtext } else { t.accent }).center().max_width(w - 8.0 * s));
        let mut mult = String::new();
        if self.skin_mult() > 1.0 {
            mult += &if self.skin_mult() >= 1e6 { format!(" · ×{}", fmt_mult(self.skin_mult())) } else { format!(" · ×{}", self.skin_mult()) };
        }
        if self.castle_mult() > 1.0 {
            mult += &format!(" · ch. ×{}", fmt_mult(self.castle_mult()));
        }
        if self.bb > 0 {
            mult += &format!(" · BB ×{}", fmt_mult(self.bb_mult()));
        }
        if self.ext_on() {
            mult += if self.skin == 10 { " · Ext ×10Md" } else { " · Ext ×1Md" };
        }
        if self.sun_on() {
            mult += " · Soleil ×1M";
        }
        if self.skin == 6 && self.clock < self.vide_until {
            mult += " · Vide ×100";
        }
        let rate_txt = format!("{} par seconde{}", fmt_rate(self.rate()), mult);
        let rate_size = if rate_txt.chars().count() > 46 { 9.5 } else if rate_txt.chars().count() > 38 { 10.5 } else { 12.5 };
        ctx.text_mid(g, &rate_txt, w / 2.0, head.height + 58.0 * s, &Text::new(rate_size * s).bold().color(t.accent).center().max_width(w - 6.0 * s));

        // grade
        let gi = self.grade_idx();
        let gr = &GRADES[gi];
        let next = match GRADES.get(gi + 1) {
            Some(n) => format!(" · suivant : {} ({})", n.0, fmt_short(n.1)),
            None => " · grade maximum".to_string(),
        };
        ctx.text_mid(
            g,
            &format!("{} ×{}{}", gr.0, gr.2, next),
            w / 2.0,
            head.height + 73.0 * s,
            &Text::new(11.0 * s).bold().color(rgb(0x7C4DFF)).center().max_width(w - 12.0 * s),
        );

        // gros bouton (il tremble un instant quand le château est netherite ou mieux)
        if self.shake > 0.0 {
            l.cx += (self.shake * 70.0).sin() * 4.0 * s * (self.shake / 0.16);
        }
        let k = 1.0 - 0.07 * self.pop;
        let r = l.r * k;
        let (outer, inner, body, door) = self.skin_colors();
        g.circle(l.cx, l.cy + 3.0 * s, r, rgba(0x000000, 0.15));
        g.circle(l.cx, l.cy, r, outer);
        g.circle(l.cx, l.cy, r * 0.9, inner);
        if self.skin == 6 {
            self.draw_blackhole(g, l.cx, l.cy, r, false);
            draw_castle(g, l.cx, l.cy, r * 0.62, body, door);
            self.draw_blackhole(g, l.cx, l.cy, r, true);
        } else if self.skin == 9 {
            self.draw_time(g, l.cx, l.cy, r, false);
            draw_castle(g, l.cx, l.cy + r * 0.32, r * 0.5, body, door);
            self.draw_time(g, l.cx, l.cy, r, true);
        } else if self.skin == 11 {
            self.draw_solar(g, l.cx, l.cy, r, false);
            draw_castle(g, l.cx, l.cy, r * 0.6, body, door);
            self.draw_solar(g, l.cx, l.cy, r, true);
        } else if self.skin == 10 {
            self.draw_eclipse(g, l.cx, l.cy, r, false);
            draw_castle(g, l.cx, l.cy + r * 0.4, r * 0.5, body, door);
            self.draw_eclipse(g, l.cx, l.cy, r, true);
            let p = self.anim_t % 5.2;
            let blink = if p > 4.95 { ((p - 4.95) / 0.25 * std::f32::consts::PI).cos().abs() } else { 1.0 };
            self.draw_eye(g, l.cx, l.cy - r * 0.3, r * 0.6, r * 0.25 * blink, r * 0.235, 0.5);
        } else if self.skin == 8 {
            self.draw_eclipse(g, l.cx, l.cy, r, false);
            draw_castle(g, l.cx, l.cy, r * 0.75, body, door);
            self.draw_eclipse(g, l.cx, l.cy, r, true);
        } else if self.skin == 7 {
            self.draw_alien(g, l.cx, l.cy, r, false);
            draw_castle(g, l.cx, l.cy + r * 0.22, r * 0.6, body, door);
            self.draw_alien(g, l.cx, l.cy, r, true);
        } else {
            if self.skin >= 4 {
                self.draw_galaxy(g, l.cx, l.cy, r * 0.9);
            }
            draw_castle(g, l.cx, l.cy, r * 1.2, body, door);
        }
        if self.skin == 3 {
            g.circle(l.cx - r * 0.45, l.cy - r * 0.5, r * 0.08, rgba(0xD1A54B, 0.8));
            g.circle(l.cx + r * 0.5, l.cy - r * 0.25, r * 0.06, rgba(0x9B5AFF, 0.8));
            g.circle(l.cx + r * 0.2, l.cy + r * 0.62, r * 0.05, rgba(0xD1A54B, 0.7));
        }
        if self.skin == 2 {
            g.circle(l.cx - r * 0.45, l.cy - r * 0.5, r * 0.12, rgba(0xFFFFFF, 0.8));
            g.circle(l.cx + r * 0.5, l.cy - r * 0.2, r * 0.07, rgba(0xFFFFFF, 0.7));
        }

        // cliqueurs (souris) puis particules d'animation de clic
        if self.is_ec() {
            self.draw_orbs(g, &l, s);
        }
        self.draw_cursors(g, &l, s);
        self.draw_defense(g, &l, s);
        self.draw_fx(g, &l, s);
        self.draw_boss(ctx, g);
        self.draw_boss_hud(ctx, g);

        // onglets
        g.rect(0.0, l.shop_top, w, l.tab_h, t.surface);
        g.rect(0.0, l.shop_top, w, 1.0, t.line);
        let tw = w / 5.0;
        let attack = !self.mons.is_empty() || !self.mon_q.is_empty();
        for (i, (name, tab)) in [("Boutique", Tab::Shop), ("Magie", Tab::Magic), ("Châteaux", Tab::Castles), ("Cliqueurs", Tab::Cursors), (if attack { "Défense !" } else { "Défense" }, Tab::Defense)].iter().enumerate() {
            let on = self.tab == *tab;
            let col = if i == 4 && attack { rgb(0xFF5A6E) } else if on { t.accent } else { t.subtext };
            ctx.text_mid(g, name, tw / 2.0 + i as f32 * tw, l.shop_top + l.tab_h / 2.0 - 1.0 * s, &Text::new(11.0 * s).bold().color(col).center().max_width(tw - 2.0));
            if on {
                g.rect(i as f32 * tw + 10.0 * s, l.shop_top + l.tab_h - 3.0 * s, tw - 20.0 * s, 3.0 * s, t.accent);
            }
        }
        if self.tab == Tab::Castles {
            let y = l.shop_top + l.tab_h;
            g.rect(0.0, y, w, 26.0 * s, t.bg);
            let n = self.cs.iter().filter(|b| **b).count();
            ctx.text_mid(g, &format!("Multiplicateur des châteaux : ×{} ({}/{})", fmt_mult(self.castle_mult()), n, CASTLES.len()), w / 2.0, y + 13.0 * s, &Text::new(12.0 * s).bold().color(t.accent).center());
        }
        if self.tab == Tab::Defense {
            let y = l.shop_top + l.tab_h;
            g.rect(0.0, y, w, 26.0 * s, t.bg);
            let txt = if self.def_g == 0 {
                format!("Recrute un {} : des monstres attaqueront ensuite toutes les 75 s", GNAME[self.skin.min(11) as usize])
            } else if !self.mons.is_empty() || !self.mon_q.is_empty() {
                format!("ATTAQUE ! {} monstre(s) · touche-les aussi · {} vaincus", self.mons.len() + self.mon_q.len(), self.def_kills)
            } else {
                format!("{} garde{} · vague {} dans {} · {} vaincus", self.def_g, if self.def_g > 1 { "s" } else { "" }, self.def_wave + 1, fmt_t((self.def_next - self.clock).max(0.0)), self.def_kills)
            };
            ctx.text_mid(g, &txt, w / 2.0, y + 13.0 * s, &Text::new(11.0 * s).bold().color(t.accent).center().max_width(w - 8.0 * s));
        }
        if self.tab == Tab::Cursors {
            let y = l.shop_top + l.tab_h;
            g.rect(0.0, y, w, 26.0 * s, t.bg);
            let txt = format!("{} cliqueur{} · {} clic/s chacun · auto ≈ {}/s", self.cur_n, if self.cur_n > 1 { "s" } else { "" }, fmt_mult(self.cur_cps()), fmt(self.cur_n as f64 * self.cur_cps() * self.click_power() * self.cursor_bonus()));
            ctx.text_mid(g, &txt, w / 2.0, y + 13.0 * s, &Text::new(11.0 * s).bold().color(t.accent).center().max_width(w - 8.0 * s));
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

    /// Galaxie animée : bras spiralés en rotation différentielle (le centre tourne plus vite),
    /// cœur lumineux qui pulse, nébuleuse et étoiles qui scintillent. MAX = 3 bras + anneau doré.
    fn draw_galaxy(&self, g: &mut Gfx, cx: f32, cy: f32, r: f32) {
        use std::f32::consts::TAU;
        let t = self.anim_t;
        let max = self.skin >= 5;
        let arms: u32 = if max { 3 } else { 2 };
        let pulse = 0.5 + 0.5 * (t * 1.3).sin();
        g.circle(cx, cy, r, rgb(0x0B0220));
        g.circle(cx, cy, r * 0.94, rgba(0x2A0C60, 0.95));
        g.circle(cx, cy, r * 0.7, rgba(0x5A1FB8, 0.25));
        g.circle(cx, cy, r * 0.5, rgba(0x8B3CE8, 0.22));
        g.circle(cx, cy, r * 0.32, rgba(0xD060F0, 0.2 + 0.08 * pulse));
        g.circle(cx, cy, r * 0.18, rgba(0xFFA6EE, 0.3 + 0.12 * pulse));
        g.circle(cx, cy, r * 0.09, rgba(0xFFF4D6, 0.6 + 0.2 * pulse));
        let n: u32 = if max { 260 } else { 190 };
        for i in 0..n {
            let arm = i % arms;
            let u = hash01(i, 1).powf(0.75);
            let off = (hash01(i, 2) - 0.5) * 0.5 * (0.4 + u);
            let omega = 1.1 / (0.3 + u * 1.4);
            let ang = arm as f32 / arms as f32 * TAU + u * 4.2 + t * omega * 0.55 + off;
            let rad = u * r * 0.93;
            let tw = 0.7 + 0.3 * (t * 3.0 + hash01(i, 4) * TAU).sin();
            let mut col = galaxy_color(u);
            if max {
                col = mix_color(col, 0xFFD24D, 0.3 * (0.5 + 0.5 * (t * 0.8 + u * 3.0).sin()));
            }
            let size = r * (0.012 + 0.026 * (1.0 - u) + 0.012 * hash01(i, 3));
            g.circle(cx + ang.cos() * rad, cy + ang.sin() * rad, size, rgba(col, (0.35 + 0.5 * (1.0 - u)) * tw));
        }
        for i in 0..26u32 {
            let a = hash01(i + 100, 5) * TAU;
            let rad = hash01(i + 100, 6).sqrt() * r * 0.93;
            let tw = 0.5 + 0.5 * (t * 2.2 + hash01(i, 7) * TAU).sin();
            g.circle(cx + a.cos() * rad, cy + a.sin() * rad, r * 0.02, rgba(0xFFFFFF, 0.2 + 0.75 * tw));
        }
        if max {
            for k in 0..60u32 {
                let a = k as f32 / 60.0 * TAU + t * 0.3;
                g.circle(cx + a.cos() * r * 0.95, cy + a.sin() * r * 0.95, r * 0.03, rgba(0xFFD24D, 0.4 + 0.3 * pulse));
            }
        }
    }

    /// Trou noir : étoiles aspirées en spirale, halo chaud, disque d'accrétion incliné (plus brillant du côté
    /// qui vient vers nous), horizon noir, arc lentillé au-dessus, anneau de photons, et un anneau de lumière
    /// autour du château. `front` = la partie dessinée devant le château.
    fn draw_blackhole(&self, g: &mut Gfx, cx: f32, cy: f32, r: f32, front: bool) {
        use std::f32::consts::{PI, TAU};
        let t = self.anim_t;
        let disk = |g: &mut Gfx, front: bool| {
            for i in 0..220u32 {
                let u = hash01(i, 31);
                let ang = hash01(i, 32) * TAU + t * (1.6 / (0.4 + u));
                let sn = ang.sin();
                if front != (sn >= 0.0) {
                    continue;
                }
                let rad = r * (0.48 + 0.44 * u);
                let dop = 0.6 - 0.5 * ang.cos();
                let col = mix_color(0xFFF4E0, 0xFF5A1F, u);
                g.circle(cx + ang.cos() * rad, cy + sn * rad * 0.24, r * (0.022 + 0.016 * (1.0 - u)), rgba(col, (0.3 * dop + 0.12).min(1.0)));
            }
        };
        if !front {
            g.circle(cx, cy, r * 0.9, rgb(0x0A0502));
            for i in 0..36u32 {
                let d = (hash01(i, 33) + t * 0.06) % 1.0;
                let a = hash01(i, 34) * TAU + t * 0.5 / (d + 0.25) + d * 3.0;
                let rr = r * (0.88 - 0.5 * d);
                g.circle(cx + a.cos() * rr, cy + a.sin() * rr, r * 0.018 * (1.0 - d * 0.6), rgba(0xFFF0DC, 0.15 + 0.6 * (1.0 - d)));
            }
            for (f, al) in [(0.86f32, 0.10f32), (0.7, 0.14), (0.56, 0.2), (0.48, 0.28)] {
                g.circle(cx, cy, r * f, rgba(0xFF8A3C, al));
            }
            disk(g, false);
            g.circle(cx, cy, r * 0.38, rgb(0x000000));
        } else {
            for k in 0..64u32 {
                let an = PI + k as f32 / 63.0 * PI;
                g.circle(cx + an.cos() * r * 0.5, cy + an.sin() * r * 0.47, r * 0.035, rgba(0xFFAA50, 0.5));
                let an2 = k as f32 / 63.0 * PI;
                g.circle(cx + an2.cos() * r * 0.44, cy + an2.sin() * r * 0.42, r * 0.018, rgba(0xFF9646, 0.3));
            }
            let fl = 0.85 + 0.15 * (t * 7.0).sin();
            for k in 0..90u32 {
                let an = k as f32 / 90.0 * TAU;
                g.circle(cx + an.cos() * r * 0.395, cy + an.sin() * r * 0.395, r * 0.02, rgba(0xFFECC8, fl));
            }
            disk(g, true);
            // anneau de lumière autour du château, avec 3 points chauds qui tournent
            for k in 0..96u32 {
                let an = k as f32 / 96.0 * TAU;
                let mut al = 0.35f32;
                let mut sz = 0.028f32;
                for hs in 0..3u32 {
                    let c = t * 0.9 + hs as f32 * TAU / 3.0;
                    let d = ((an - c + PI).rem_euclid(TAU) - PI).abs();
                    if d < 0.35 {
                        al = al.max(0.35 + 0.6 * (1.0 - d / 0.35));
                        sz = sz.max(0.028 + 0.02 * (1.0 - d / 0.35));
                    }
                }
                g.circle(cx + an.cos() * r * 1.09, cy + an.sin() * r * 1.09, r * 0.05, rgba(0xFF9A3C, al * 0.18));
                g.circle(cx + an.cos() * r * 1.09, cy + an.sin() * r * 1.09, r * sz, rgba(0xFFE2B8, al));
            }
        }
    }

    /// Château alien : radar qui balaie, grille, faisceau tracteur avec des runes qui montent,
    /// soucoupe qui flotte (avec son petit alien et ses lumières qui clignotent), anneau néon
    /// et deux anneaux en pointillés qui tournent en sens inverse autour du château.
    fn draw_alien(&self, g: &mut Gfx, cx: f32, cy: f32, r: f32, front: bool) {
        use std::f32::consts::TAU;
        let t = self.anim_t;
        let hb = (t * 2.0).sin() * r * 0.03;
        let sy = cy - r * 0.52 + hb;
        let by = cy + r * 0.62;
        if !front {
            g.circle(cx, cy, r * 0.9, rgb(0x04140C));
            g.circle(cx, cy + r * 0.1, r * 0.7, rgba(0x0F3A24, 0.8));
            for i in 0..22u32 {
                let a = hash01(i, 41) * TAU;
                let d = hash01(i, 42).sqrt() * r * 0.85;
                let tw = 0.5 + 0.5 * (t * 2.0 + hash01(i, 43) * TAU).sin();
                g.circle(cx + a.cos() * d, cy + a.sin() * d, (r * 0.018).max(1.0), rgba(0xC8FFDC, 0.2 + 0.6 * tw));
            }
            let a0 = t * 1.4;
            for i in 0..14u32 {
                let a = a0 - i as f32 * 0.06;
                let al = 0.22 * (1.0 - i as f32 / 14.0);
                for j in 1..10u32 {
                    let d = r * 0.09 * j as f32;
                    g.circle(cx + a.cos() * d, cy + a.sin() * d, r * 0.03, rgba(0x3CFF8C, al));
                }
            }
            for k in 1..4u32 {
                let rr = r * 0.24 * k as f32;
                for m in 0..48u32 {
                    let an = m as f32 / 48.0 * TAU;
                    g.circle(cx + an.cos() * rr, cy + an.sin() * rr, r * 0.009, rgba(0x3CFF8C, 0.25));
                }
            }
            let steps = 28u32;
            for i in 0..steps {
                let f = i as f32 / steps as f32;
                let y = sy + (by - sy) * f;
                let hw = r * (0.12 + 0.38 * f);
                g.rect(cx - hw, y, hw * 2.0, (by - sy) / steps as f32 + 1.0, rgba(0x8CFFB4, 0.28 * (1.0 - f * 0.8)));
            }
            for i in 0..6u32 {
                let f = (t * 0.6 + i as f32 / 6.0) % 1.0;
                let hw = r * (0.12 + 0.38 * f);
                g.rect(cx - hw, sy + (by - sy) * f, hw * 2.0, (r * 0.02).max(2.0), rgba(0xB4FFD2, 0.45 * (1.0 - f)));
            }
            for i in 0..18u32 {
                let f = (t * 0.35 + hash01(i, 44)) % 1.0;
                let y = by - (by - sy) * f;
                let hw = r * (0.12 + 0.38 * (1.0 - f));
                let x = cx + (hash01(i, 45) - 0.5) * 1.6 * hw;
                let sz = (r * 0.05).max(2.0);
                g.rect(x - sz / 2.0, y - sz / 2.0, sz, sz, rgba(0x78FFAA, 0.85 * (1.0 - f)));
            }
        } else {
            let pu = 0.6 + 0.4 * (t * 3.0).sin();
            for k in 0..80u32 {
                let an = k as f32 / 80.0 * TAU;
                g.circle(cx + an.cos() * r * 0.93, cy + an.sin() * r * 0.93, r * 0.035, rgba(0x39FF88, pu));
            }
            g.circle(cx, sy - r * 0.06, r * 0.2, rgba(0x78F0FF, 0.45));
            g.circle(cx, sy - r * 0.13, r * 0.08, rgb(0x6DFF7A));
            g.circle(cx - r * 0.03, sy - r * 0.14, r * 0.025, rgb(0x000000));
            g.circle(cx + r * 0.03, sy - r * 0.14, r * 0.025, rgb(0x000000));
            g.rrect(cx - r * 0.56, sy - r * 0.1, r * 1.12, r * 0.2, r * 0.1, rgb(0x6E7C77));
            g.rrect(cx - r * 0.5, sy - r * 0.1, r * 1.0, r * 0.09, r * 0.045, rgb(0xC9D6D1));
            for i in 0..7u32 {
                let on = ((t * 6.0) as u32) % 7 == i;
                let col = if on { 0xFFFFFF } else { [0x39FF88u32, 0x00E5FF, 0xD4FF3A][(i % 3) as usize] };
                g.circle(cx - r * 0.42 + i as f32 * r * 0.14, sy + r * 0.03, r * 0.028, rgba(col, if on { 1.0 } else { 0.8 }));
            }
            for k in 0..18u32 {
                let a = t * 0.8 + k as f32 * TAU / 18.0;
                for j in 0..4u32 {
                    let an = a + j as f32 * 0.05;
                    g.circle(cx + an.cos() * r * 1.12, cy + an.sin() * r * 1.12, r * 0.03, rgba(0x39FF88, 0.85));
                }
            }
            for k in 0..12u32 {
                let a = -t * 1.2 + k as f32 * TAU / 12.0;
                for j in 0..3u32 {
                    let an = a + j as f32 * 0.04;
                    g.circle(cx + an.cos() * r * 1.22, cy + an.sin() * r * 1.22, r * 0.022, rgba(0x00E5FF, 0.6));
                }
            }
        }
    }

    /// Château Éclipse : deux tourbillons d'énergie opposés (rouge / bleu), cœur blanc qui pulse,
    /// éclairs qui crépitent et anneau en traits qui tourne.
    fn draw_eclipse(&self, g: &mut Gfx, cx: f32, cy: f32, r: f32, front: bool) {
        use std::f32::consts::TAU;
        let t = self.anim_t;
        if !front {
            g.circle(cx, cy, r * 0.9, rgb(0x05061A));
            g.circle(cx, cy, r * 0.7, rgba(0x141A3A, 0.8));
            for i in 0..170u32 {
                let red = i % 2 == 0;
                let dir = if red { 1.0 } else { -1.0 };
                let u = hash01(i, 51);
                let a = hash01(i, 52) * TAU + u * 5.0 * dir + t * dir * (1.2 / (0.3 + u));
                let rad = r * (0.25 + 0.6 * u);
                let col = if red { 0xFF3B5C } else { 0x4D8DFF };
                g.circle(cx + a.cos() * rad, cy + a.sin() * rad, r * (0.02 + 0.022 * (1.0 - u)), rgba(col, 0.15 + 0.45 * (1.0 - u)));
            }
            let pu = 1.0 + 0.08 * (t * 6.0).sin();
            for (f, al, c) in [(0.5f32, 0.14f32, 0x6FA8FFu32), (0.36, 0.22, 0xC8E1FF), (0.24, 0.4, 0xFFFFFF), (0.14, 0.7, 0xFFFFFF)] {
                g.circle(cx, cy, r * f * pu, rgba(c, al));
            }
        } else {
            let bucket = ((t / 0.09) as u32) % 5000;
            for k in 0..4u32 {
                let seed = bucket * 13 + k * 101;
                let a0 = hash01(seed, 53) * TAU;
                let a1 = a0 + (hash01(seed, 54) - 0.5) * 1.4;
                let col = if k % 2 == 0 { 0xFF6E82 } else { 0x8CBEFF };
                draw_bolt(g, cx + a0.cos() * r * 0.28, cy + a0.sin() * r * 0.28, cx + a1.cos() * r * 0.9, cy + a1.sin() * r * 0.9, seed, 7, r * 0.16, col, 0.9, r * 0.016);
            }
            for k in 0..24u32 {
                let b = t * 0.7 + k as f32 * TAU / 24.0;
                for j in 0..3u32 {
                    let an = b + j as f32 * 0.045;
                    g.circle(cx + an.cos() * r * 0.9, cy + an.sin() * r * 0.9, r * 0.018, rgba(0xEBF0FF, 0.6));
                }
            }
        }
    }

    /// Deux orbes d'énergie (rouge et bleue) qui tournent autour du château, avec traînée et arcs électriques.
    fn draw_orbs(&self, g: &mut Gfx, l: &Layout, s: f32) {
        use std::f32::consts::{PI, TAU};
        let t = self.anim_t;
        let ro = l.r * 1.3;
        let sz = l.r * 0.14 * (1.0 + 0.1 * (t * 9.0).sin() + 0.35 * self.orb_pulse);
        for k in 0..2u32 {
            let col = if k == 0 { 0xFF3B5C } else { 0x4D8DFF };
            let a = t * 1.7 + k as f32 * PI;
            for j in (1..=10u32).rev() {
                let aa = a - j as f32 * 0.07;
                g.circle(l.cx + aa.cos() * ro, l.cy + aa.sin() * ro, sz * (1.0 - j as f32 * 0.05), rgba(col, 0.09 * (11 - j) as f32 / 10.0));
            }
            let (x, y) = (l.cx + a.cos() * ro, l.cy + a.sin() * ro);
            g.circle(x, y, sz * 2.2, rgba(col, 0.14));
            g.circle(x, y, sz * 1.5, rgba(col, 0.3));
            g.circle(x, y, sz, rgba(col, 0.95));
            g.circle(x, y, sz * 0.45, rgb(0xFFFFFF));
            let bucket = ((t / 0.06) as u32) % 5000;
            for j in 0..3u32 {
                let seed = bucket * 7 + k * 31 + j * 3;
                let b = hash01(seed, 57) * TAU;
                draw_bolt(g, x + b.cos() * sz * 0.5, y + b.sin() * sz * 0.5, x + b.cos() * sz * 1.7, y + b.sin() * sz * 1.7, seed, 4, sz * 0.4, col, 0.9, 0.8 * s);
            }
        }
    }

    /// Entrée en scène du château Éclipse (≈ 3,3 s) : orage d'éclairs, les deux orbes arrivent en
    /// spirale et se placent en orbite, onde de choc, flash et titre.
    fn draw_intro(&mut self, ctx: &Ctx, g: &mut Gfx) {
        use std::f32::consts::{PI, TAU};
        let t = self.intro_t;
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let (cx, cy) = (w / 2.0, h / 2.0);
        let m = w.min(h);
        if t > 2.8 {
            self.draw_game(ctx, g);
            g.rect(0.0, 0.0, w, h, rgba(0x020208, (1.0 - (t - 2.8) / 0.5).clamp(0.0, 1.0)));
            return;
        }
        if t < 0.25 {
            self.draw_game(ctx, g);
            g.rect(0.0, 0.0, w, h, rgba(0x020208, t / 0.25));
        } else {
            g.clear(rgb(0x020208));
        }
        let bucket = ((t / 0.07) as u32) % 5000;
        for k in 0..(if t < 1.8 { 5u32 } else { 2 }) {
            let seed = bucket * 13 + k * 101;
            let x0 = hash01(seed, 61) * w;
            let x1 = x0 + (hash01(seed, 62) - 0.5) * w * 0.6;
            let y1 = h * (0.4 + 0.6 * hash01(seed, 63));
            draw_bolt(g, x0, 0.0, x1, y1, seed, 10, w * 0.08, if k % 2 == 0 { 0xFF6E82 } else { 0x8CBEFF }, 0.85, 1.4 * s);
        }
        let p = (t / 1.8).min(1.0);
        let e = 1.0 - (1.0 - p).powi(3);
        let orb = m * (0.42 - 0.22 * e);
        let ang = t * (2.0 + 4.0 * (1.0 - p));
        let sz = m * 0.05 * (1.0 + 0.15 * (t * 10.0).sin());
        for (k, col) in [(0u32, 0xFF3B5Cu32), (1, 0x4D8DFF)] {
            let a = ang + k as f32 * PI;
            let f = 1.0 + (1.0 - e) * 1.4;
            let (x, y) = (cx + a.cos() * orb * f, cy + a.sin() * orb * f);
            g.circle(x, y, sz * 2.4, rgba(col, 0.18));
            g.circle(x, y, sz * 1.5, rgba(col, 0.4));
            g.circle(x, y, sz, rgba(col, 1.0));
            g.circle(x, y, sz * 0.45, rgb(0xFFFFFF));
        }
        if t > 1.8 {
            let s2 = t - 1.8;
            let al = (1.0 - s2 / 1.2).clamp(0.0, 1.0);
            for (k, col) in [(0u32, 0xFFFFFFu32), (1, 0xFF3B5C), (2, 0x4D8DFF)] {
                let rr = s2 * (m * 1.1 + k as f32 * 60.0 * s);
                for q in 0..90u32 {
                    let an = q as f32 / 90.0 * TAU;
                    g.circle(cx + an.cos() * rr, cy + an.sin() * rr, (3.0 - k as f32 * 0.7) * s, rgba(col, al));
                }
            }
            if s2 > 0.15 {
                let pp = ((s2 - 0.15) / 0.4).min(1.0);
                let size = (40.0 * s * (0.7 + 0.3 * pp)).round();
                g.circle(cx, cy, 80.0 * s, rgba(0x6FA8FF, 0.12 * pp));
                ctx.text_mid(g, "ÉCLIPSE", cx, cy, &Text::new(size).bold().color(rgb(0xFFFFFF)).center());
                if pp > 0.5 {
                    ctx.text_mid(g, "Clics et production ×100 milliards", cx, cy + 30.0 * s, &Text::new(13.0 * s).bold().color(rgb(0xFFFFFF)).center().max_width(w - 16.0 * s));
                }
            }
        }
        let fl = (1.0 - ((t - 1.85).abs() / 0.18)).clamp(0.0, 1.0);
        if fl > 0.0 {
            g.rect(0.0, 0.0, w, h, rgba(0xFFFFFF, fl));
        }
    }

    /// Animation du Big Bang (≈ 4,3 s) : implosion des étoiles vers un point, flash blanc,
    /// explosion arc-en-ciel avec traînées, nébuleuse, ondes de choc et titre.
    fn draw_bigbang(&mut self, ctx: &Ctx, g: &mut Gfx) {
        use std::f32::consts::TAU;
        let t = self.bbfx_t;
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let (cx, cy) = (w / 2.0, h / 2.0);
        let maxr = (w * w + h * h).sqrt() / 2.0;
        if t > 3.7 {
            self.draw_game(ctx, g);
            g.rect(0.0, 0.0, w, h, rgba(0x000000, (1.0 - (t - 3.7) / 0.6).clamp(0.0, 1.0)));
            return;
        }
        if t < 0.35 {
            self.draw_game(ctx, g);
            g.rect(0.0, 0.0, w, h, rgba(0x000000, t / 0.35));
        } else {
            g.clear(rgb(0x000000));
        }
        if t < 1.25 {
            let k = (t / 1.2).min(1.0);
            let e = k * k;
            for i in 0..220u32 {
                let a = hash01(i, 11) * TAU + k * 2.2 * (1.0 - hash01(i, 12));
                let d0 = (0.25 + 0.75 * hash01(i, 13)) * maxr;
                let d = d0 * (1.0 - e);
                let col = hsv_hex(hash01(i, 14) * 360.0, 0.55, 1.0);
                for j in 0..4u32 {
                    let dd = d + j as f32 * d0 * 0.05 * k;
                    g.circle(cx + a.cos() * dd, cy + a.sin() * dd, (1.6 - 0.3 * j as f32) * s, rgba(col, (0.9 - 0.2 * j as f32) * (0.3 + 0.7 * k)));
                }
            }
            let gr = (4.0 + 30.0 * k) * s;
            for (f, al, c) in [(3.0f32, 0.08f32, 0x7C4DFFu32), (2.0, 0.15, 0xC77DFF), (1.2, 0.35, 0xFFE6FF), (0.6, 0.95, 0xFFFFFF)] {
                g.circle(cx, cy, gr * f, rgba(c, al));
            }
        }
        if t >= 1.25 {
            let e = t - 1.25;
            let fa = (1.0 - e / 2.6).clamp(0.0, 1.0);
            for (i, c) in [0xFF4DE0u32, 0x4D7BFF, 0xFFD24D].iter().enumerate() {
                let rr = (e * 220.0 * s * (1.0 + i as f32 * 0.3)).min(maxr * 1.2);
                g.circle(cx, cy, rr, rgba(*c, 0.16 * fa));
            }
            for i in 0..320u32 {
                let a = hash01(i, 21) * TAU;
                let sp = (250.0 + 900.0 * hash01(i, 22)) * s;
                let dist = sp * (1.0 - (-e * 1.6).exp()) / 1.6;
                let col = hsv_hex((hash01(i, 23) * 360.0 + e * 90.0) % 360.0, 0.85, 1.0);
                for j in 0..3u32 {
                    let dd = dist * (1.0 - 0.05 * j as f32);
                    g.circle(cx + a.cos() * dd, cy + a.sin() * dd, (2.2 - 0.5 * j as f32) * s, rgba(col, fa * (1.0 - 0.3 * j as f32)));
                }
            }
            let al = (1.0 - e / 2.0).clamp(0.0, 1.0);
            if al > 0.0 {
                for i in 0..5u32 {
                    let rr = e * (380.0 + i as f32 * 140.0) * s;
                    let col = hsv_hex((i as f32 * 70.0 + e * 200.0) % 360.0, 0.8, 1.0);
                    for k in 0..72u32 {
                        let an = k as f32 / 72.0 * TAU;
                        g.circle(cx + an.cos() * rr, cy + an.sin() * rr, (3.5 - 0.5 * i as f32).max(1.0) * s, rgba(col, al * 0.9));
                    }
                }
            }
            if e > 0.25 {
                let p = ((e - 0.25) / 0.55).min(1.0);
                let size = (46.0 * s * (0.55 + 0.45 * ease_out_back(p))).round();
                let hue = ((e * 120.0) / 30.0).round() * 30.0;
                g.circle(cx, cy - 10.0 * s, 95.0 * s, rgba(0xFFFFFF, 0.08 * p));
                g.circle(cx, cy - 10.0 * s, 60.0 * s, rgba(0xFFFFFF, 0.08 * p));
                ctx.text_mid(g, "BIG BANG", cx, cy - 10.0 * s, &Text::new(size).bold().color(rgb(hsv_hex(hue, 0.35, 1.0))).center());
                if p > 0.5 {
                    ctx.text_mid(g, &format!("Multiplicateurs ×2 · Big Bang ×{}", fmt_mult(self.bb_mult())), cx, cy + 26.0 * s, &Text::new(13.0 * s).bold().color(rgb(0xFFFFFF)).center().max_width(w - 16.0 * s));
                }
            }
        }
        let fl = (1.0 - ((t - 1.3).abs() / 0.22)).clamp(0.0, 1.0);
        if fl > 0.0 {
            g.rect(0.0, 0.0, w, h, rgba(0xFFFFFF, fl));
        }
    }

    fn draw_code(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = self.theme_for(ctx);
        let s = ctx.scale();
        let w = ctx.width();
        g.clear(t.bg);
        let title = match self.screen {
            Screen::Tune => "Interface secrète",
            Screen::Tune3 => "Interface ultime",
            Screen::Tune4 => "Interface ???",
            _ => "Entrer un code",
        };
        self.hdr(ctx, g, title, Some("Retour"));
        let box_r = Rect::new(16.0 * s, 44.0 * s + 16.0 * s, w - 32.0 * s, 40.0 * s);
        g.rrect(box_r.x, box_r.y, box_r.w, box_r.h, 10.0 * s, t.surface);
        let (cx, cy) = box_r.center();
        if self.code.is_empty() {
            let ph = match self.screen {
                Screen::Tune => "2E CODE",
                Screen::Tune3 => "3E CODE",
                Screen::Tune4 => "4E CODE",
                _ => "CODE",
            };
            ctx.text_mid(g, ph, cx, cy, &Text::new(22.0 * s).bold().color(t.line).center());
        } else {
            ctx.text_mid(g, &self.code, cx, cy, &Text::new(22.0 * s).mono().bold().color(t.text).center().max_width(box_r.w - 16.0 * s));
        }
        for (r, k) in self.kb_keys(ctx) {
            g.rrect(r.x, r.y + 1.0 * s, r.w, r.h, 6.0 * s, rgba(0x000000, 0.12));
            g.rrect(r.x, r.y, r.w, r.h, 6.0 * s, t.surface);
            let (label, sz) = match k {
                Kb::Ch(c) => (c.to_string(), 16.0),
                Kb::Del => ("Effacer".to_string(), 10.0),
                Kb::Space => ("Espace".to_string(), 10.0),
            };
            let (x, y) = r.center();
            ctx.text_mid(g, &label, x, y, &Text::new(sz * s).bold().color(t.text).center());
        }
        self.btn(ctx, g, self.submit_rect(ctx), "Valider", ButtonStyle::Primary, false);
    }

    fn draw_menu(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = self.theme_for(ctx);
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        self.draw_game(ctx, g);
        g.rect(0.0, 0.0, w, h, rgba(0x000000, 0.55));
        let (panel, b) = self.menu_buttons(ctx);
        g.rrect(panel.x, panel.y, panel.w, panel.h, 16.0 * s, t.bg);
        ctx.text_mid(g, "Menu", w / 2.0, panel.y + 28.0 * s, &Text::new(20.0 * s).bold().color(t.text).center());
        self.btn(ctx, g, b[0], "Reprendre", ButtonStyle::Primary, false);
        self.btn(ctx, g, b[1], "Conquis", ButtonStyle::Secondary, false);
        self.btn(ctx, g, b[2], "Progrès", ButtonStyle::Secondary, false);
        self.btn(ctx, g, b[3], "Entrer un code", ButtonStyle::Secondary, false);
        self.btn(ctx, g, b[4], "Paramètres", ButtonStyle::Secondary, false);
        self.btn(ctx, g, b[5], "Quitter", ButtonStyle::Danger, false);
    }

    fn draw_settings(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = self.theme_for(ctx);
        let s = ctx.scale();
        let w = ctx.width();
        g.clear(t.bg);
        self.hdr(ctx, g, "Paramètres", Some("Retour"));
        let r = self.settings_row(ctx);
        g.rect(r.x, r.y, r.w, r.h, t.surface);
        g.rect(r.x, r.bottom() - 1.0, r.w, 1.0, t.line);
        ctx.text_mid(g, "Compteur de FPS", 14.0 * s, r.y + r.h * 0.34, &Text::new(16.0 * s).bold().color(t.text).left());
        ctx.text_mid(g, "FPS réels, sans limite d’écran", 14.0 * s, r.y + r.h * 0.70, &Text::new(11.0 * s).color(t.subtext).left().max_width(w - 90.0 * s));
        self.tgl(ctx, g, Rect::new(w - 62.0 * s, r.y + r.h / 2.0 - 14.0 * s, 48.0 * s, 28.0 * s), self.fps_on);
        let r2 = self.bench_row(ctx);
        g.rect(r2.x, r2.y, r2.w, r2.h, t.surface);
        g.rect(r2.x, r2.bottom() - 1.0, r2.w, 1.0, t.line);
        ctx.text_mid(g, "Test de puissance", 14.0 * s, r2.y + r2.h * 0.34, &Text::new(16.0 * s).bold().color(t.text).left());
        ctx.text_mid(g, "Mesure le processeur et les graphismes", 14.0 * s, r2.y + r2.h * 0.70, &Text::new(11.0 * s).color(t.subtext).left().max_width(w - 60.0 * s));
        ctx.text_mid(g, "›", w - 20.0 * s, r2.y + r2.h / 2.0, &Text::new(24.0 * s).bold().color(t.subtext).center());
        let r3 = self.music_row(ctx);
        g.rect(r3.x, r3.y, r3.w, r3.h, t.surface);
        g.rect(r3.x, r3.bottom() - 1.0, r3.w, 1.0, t.line);
        ctx.text_mid(g, "Musique Éclipse", 14.0 * s, r3.y + r3.h * 0.34, &Text::new(16.0 * s).bold().color(t.text).left().max_width(w - 90.0 * s));
        ctx.text_mid(g, "Nécessite mpv, ffplay ou vlc", 14.0 * s, r3.y + r3.h * 0.70, &Text::new(11.0 * s).color(t.subtext).left().max_width(w - 90.0 * s));
        self.tgl(ctx, g, Rect::new(w - 62.0 * s, r3.y + r3.h / 2.0 - 14.0 * s, 48.0 * s, 28.0 * s), self.music_on);
    }

    fn bench_btn(&self, ctx: &Ctx) -> Rect {
        let s = ctx.scale();
        Rect::new(24.0 * s, ctx.height() - 90.0 * s, ctx.width() - 48.0 * s, 46.0 * s)
    }

    fn draw_bench(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = self.theme_for(ctx);
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        g.clear(t.bg);
        self.hdr(ctx, g, "Puissance", Some("Retour"));
        let mid = h / 2.0;
        match self.bench_phase {
            0 => {
                ctx.text_mid(g, "Test du processeur…", w / 2.0, mid - 20.0 * s, &Text::new(18.0 * s).bold().color(t.text).center());
                let p = (self.bench_t / 1.2).clamp(0.0, 1.0);
                g.rrect(30.0 * s, mid + 10.0 * s, w - 60.0 * s, 10.0 * s, 5.0 * s, t.line);
                g.rrect(30.0 * s, mid + 10.0 * s, ((w - 60.0 * s) * p).max(10.0 * s), 10.0 * s, 5.0 * s, t.accent);
            }
            1 => {
                self.bench_frames += 1;
                let tt = self.bench_t;
                for i in 0..1500u32 {
                    let a = i as f32 * 0.37 + tt * (1.0 + (i % 5) as f32);
                    let rr = 20.0 * s + (i % 60) as f32 * 3.0 * s;
                    let col = mix_color(0xFF5FE0, 0x5C8BFF, (i % 17) as f32 / 17.0);
                    g.circle(w / 2.0 + a.cos() * rr * 0.9, mid + a.sin() * rr * 1.1, 2.6 * s, rgba(col, 0.85));
                }
                g.rrect(20.0 * s, 56.0 * s, w - 40.0 * s, 30.0 * s, 8.0 * s, rgba(0x000000, 0.6));
                ctx.text_mid(g, "Test graphique… (1500 formes)", w / 2.0, 71.0 * s, &Text::new(13.0 * s).bold().color(rgb(0xFFFFFF)).center());
            }
            _ => {
                let score = (self.bench_cpu * 2.0 + self.bench_fps as f64 * 10.0).round();
                let verdict = if self.bench_fps >= 55.0 {
                    "Excellente : tout est fluide"
                } else if self.bench_fps >= 35.0 {
                    "Bonne : le jeu tourne bien"
                } else if self.bench_fps >= 20.0 {
                    "Moyenne : évite la galaxie"
                } else {
                    "Faible : évite la galaxie"
                };
                let lines = [
                    format!("Processeur : {:.0} M op/s", self.bench_cpu),
                    format!("Graphismes : {:.0} FPS", self.bench_fps),
                    format!("Score : {}", score),
                ];
                ctx.text_mid(g, "Résultats", w / 2.0, 80.0 * s, &Text::new(22.0 * s).bold().color(t.text).center());
                for (i, l) in lines.iter().enumerate() {
                    ctx.text_mid(g, l, w / 2.0, 130.0 * s + i as f32 * 36.0 * s, &Text::new(17.0 * s).bold().color(t.text).center().max_width(w - 20.0 * s));
                }
                ctx.text_mid(g, verdict, w / 2.0, 130.0 * s + 3.0 * 36.0 * s + 14.0 * s, &Text::new(13.0 * s).bold().color(t.accent).center().max_width(w - 24.0 * s));
                self.btn(ctx, g, self.bench_btn(ctx), "Refaire le test", ButtonStyle::Primary, false);
            }
        }
        ctx.animate();
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

impl Drop for Conquis {
    fn drop(&mut self) {
        if let Some(mut c) = self.music.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

impl App for Conquis {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        if let Some(m) = self.pending_toast.take() {
            ctx.toast(m);
        }
        match self.screen {
            Screen::Game => self.draw_game(ctx, g),
            Screen::Menu => self.draw_menu(ctx, g),
            Screen::Code | Screen::Tune | Screen::Tune3 | Screen::Tune4 => self.draw_code(ctx, g),
            Screen::Intro => self.draw_intro(ctx, g),
            Screen::DomainFx => self.draw_domainfx(ctx, g),
            Screen::VoidFx => self.draw_voidfx(ctx, g),
            Screen::Settings => self.draw_settings(ctx, g),
            Screen::Bench => self.draw_bench(ctx, g),
            Screen::Skins => self.draw_skins(ctx, g),
            Screen::TimeFx => self.draw_timefx(ctx, g),
            Screen::Progress => self.draw_progress(ctx, g),
            Screen::OrbFx => self.draw_orbfx(ctx, g),
            Screen::BossIntro => self.draw_bossintro(ctx, g),
            Screen::EyeFx => self.draw_eyefx(ctx, g),
            Screen::SunFx => self.draw_sunfx(ctx, g),
            Screen::SolarFx => self.draw_solarfx(ctx, g),
            Screen::BigBang => self.draw_bigbang(ctx, g),
            Screen::Video => self.draw_video(ctx, g),
        }
        if self.fps_on {
            if self.fps_cycle < 0.3 {
                self.fps_frames += 1;
            }
            let s = ctx.scale();
            let label = if self.fps_val > 0.0 { format!("{:.0} FPS", self.fps_val) } else { "-- FPS".to_string() };
            let fy = ctx.height() - 21.0 * s;
            g.rrect(4.0 * s, fy, 62.0 * s, 17.0 * s, 5.0 * s, rgba(0x000000, 0.72));
            ctx.text_mid(g, &label, 4.0 * s + 31.0 * s, fy + 8.5 * s, &Text::new(11.0 * s).mono().bold().color(rgb(0x7CFC8A)).center());
        }
    }

    fn tick(&mut self, ctx: &Ctx, _dt: f32) {
        let now = Instant::now();
        let real = (now - self.last).as_secs_f32().min(3600.0);
        self.last = now;
        if let Some(rx) = &self.verify {
            match rx.try_recv() {
                Ok((level, ok, code)) => {
                    self.verify = None;
                    if !ok {
                        ctx.toast("Code invalide");
                    } else if level == 3 {
                        self.code3_ok = code;
                        self.code.clear();
                        self.screen = Screen::Tune4;
                    } else if level == 6 {
                        self.inf_dom = !self.inf_dom;
                        self.save();
                        if !self.inf_dom {
                            ctx.toast("Extension du territoire infinie désactivée");
                        } else if self.is_ec() {
                            self.dom_t = 0.0;
                            self.screen = Screen::DomainFx;
                        } else {
                            self.screen = Screen::Game;
                            ctx.toast("Extension du territoire infinie : elle s’activera dès que tu équiperas un château Éclipse");
                        }
                    } else if level == 5 {
                        if self.own[10] {
                            ctx.toast("Déjà débloqué : équipe-le dans Menu › Conquis");
                        } else {
                            self.own[10] = true;
                            self.skin = 10;
                            self.save();
                            self.eye_t = 0.0;
                            self.screen = Screen::EyeFx;
                        }
                    } else if self.own[8] {
                        ctx.toast("Déjà débloqué : équipe-le dans Menu › Conquis");
                    } else {
                        self.own[8] = true;
                        self.skin = 8;
                        self.save();
                        self.intro_t = 0.0;
                        self.screen = Screen::Intro;
                    }
                    ctx.animate();
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => ctx.redraw_in(Duration::from_millis(100)),
                Err(_) => self.verify = None,
            }
        }
        // compteur de FPS : une courte mesure sans limite d'écran (0,3 s toutes les 2 s)
        // au lieu de dessiner en continu au maximum (ce qui faisait laguer)
        if self.fps_on {
            let prev = self.fps_cycle;
            self.fps_cycle += real;
            if self.fps_cycle >= 2.0 {
                self.fps_cycle = 0.0;
                self.fps_frames = 0;
                self.fps_t = Instant::now();
            } else if prev < 0.3 && self.fps_cycle >= 0.3 {
                self.fps_val = self.fps_frames as f32 / self.fps_t.elapsed().as_secs_f32().max(0.01);
            }
            let probing = self.fps_cycle < 0.3;
            if probing {
                ctx.animate();
            } else {
                ctx.redraw_in(Duration::from_millis(250));
            }
        }
        self.clock = now_secs();
        self.slow_t += real;
        if matches!(self.screen, Screen::Game | Screen::Menu) {
            self.def_update(ctx, real.min(0.05));
        }
        if let Some(b) = self.boss.as_mut() {
            b.age += real;
            b.fl = (b.fl - real * 4.0).max(0.0);
            b.kb = (b.kb - real * 5.0).max(0.0);
            ctx.animate();
        }
        if let Some((_, tt)) = self.boss_dying.as_mut() {
            *tt += real;
            ctx.animate();
        }
        if matches!(self.boss_dying, Some((_, tt)) if tt >= 0.95) {
            self.boss_dying = None;
        }
        if self.boss_ban > 0.0 {
            self.boss_ban = (self.boss_ban - real).max(0.0);
            ctx.animate();
        }
        if self.slow_t >= 1.0 {
            self.slow_t = 0.0;
            if self.total > self.lt_max {
                self.lt_max = self.total;
            }
            if self.q_day != (self.clock / 86400.0) as i64 || self.q.is_empty() {
                self.gen_quests();
            }
            self.check_ach(ctx);
            if self.boss.is_none() && self.boss_next > 0.0 && self.clock >= self.boss_next && matches!(self.screen, Screen::Game | Screen::Menu) {
                self.spawn_boss(ctx, false);
            }
            let names = self.combo_list().iter().map(|&i| CMB[i].0).collect::<Vec<_>>().join("|");
            if names != self.last_combo {
                if !names.is_empty() {
                    ctx.toast(format!("COMBO ! {}", names.replace('|', " · ")));
                }
                self.last_combo = names;
            }
            self.music_update();
        }
        if self.screen == Screen::Progress && self.prog_scroll.update(real) {
            ctx.animate();
        }
        if self.screen == Screen::SunFx {
            self.sun_t += real;
            ctx.animate();
            if self.sun_t >= 4.2 {
                self.screen = Screen::Game;
                ctx.toast("ÉCLAT DU SOLEIL ! Multiplicateur du château ×1 000 000 pendant 3 min");
            }
            return;
        }
        if self.screen == Screen::SolarFx {
            self.solar_t += real;
            ctx.animate();
            if self.solar_t >= 4.6 {
                self.screen = Screen::Game;
            }
            return;
        }
        if self.screen == Screen::EyeFx {
            self.eye_t += real;
            ctx.animate();
            if self.eye_t >= 5.4 {
                self.screen = Screen::Game;
                ctx.toast("ÉCLIPSE ULTIME ! Clics et production ×10 000 milliards pour toujours, et l’Œil absolu s’ouvre");
            }
            return;
        }
        if self.screen == Screen::BossIntro {
            let prev = self.boss_intro_t;
            self.boss_intro_t += real;
            ctx.animate();
            if prev < 1.7 && self.boss_intro_t >= 1.7 {
                if let Some(b) = &self.boss {
                    ctx.toast(format!("{}{} !", if b.test { "BOSS DE TEST : " } else { "UN BOSS APPARAÎT : " }, BOSS_NAMES[(b.skin as usize).min(11)]));
                }
            }
            if self.boss_intro_t >= 3.0 {
                self.screen = Screen::Game;
            }
            return;
        }
        if self.screen == Screen::OrbFx {
            self.orb_t += real;
            ctx.animate();
            if self.orb_t >= 2.9 && self.boss.is_some() {
                self.kill_boss(ctx);
            }
            if self.orb_t >= 4.4 {
                self.screen = Screen::Game;
            }
            return;
        }
        if self.screen == Screen::Skins && self.skin_scroll.update(real) {
            ctx.animate();
        }
        if self.screen == Screen::TimeFx {
            self.time_t += real;
            ctx.animate();
            if self.time_t >= 4.0 {
                self.screen = Screen::Game;
                ctx.toast(format!("AVANCEMENT TEMPOREL ! +{} territoires (5 min de production)", fmt(self.time_gain)));
            }
            return;
        }
        if self.screen == Screen::DomainFx {
            self.dom_t += real;
            ctx.animate();
            if self.dom_t >= 5.2 {
                self.screen = Screen::Game;
                let m = if self.skin == 10 { "10 milliards" } else { "1 milliard" };
                ctx.toast(if self.inf_dom { format!("EXTENSION DU TERRITOIRE INFINIE ! Tous tes multiplicateurs ×{}, pour toujours", m) } else { format!("EXTENSION DU TERRITOIRE ! Tous tes multiplicateurs ×{} pendant 5 min", m) });
            }
            return;
        }
        if self.screen == Screen::VoidFx {
            self.void_t += real;
            ctx.animate();
            if self.void_t >= 3.2 {
                self.screen = Screen::Game;
                ctx.toast("LE VIDE ! Multiplicateur du château ×100 pendant 3 min");
            }
            return;
        }
        if self.screen == Screen::Intro {
            self.intro_t += real;
            ctx.animate();
            if self.intro_t >= 3.3 {
                self.screen = Screen::Game;
                ctx.toast("Château ÉCLIPSE ! ×100 milliards, et tes cliqueurs frappent ×8");
            }
            return;
        }
        if self.screen == Screen::BigBang {
            self.bbfx_t += real;
            ctx.animate();
            if self.bbfx_t >= 4.3 {
                self.screen = Screen::Game;
                ctx.toast(format!("BIG BANG ! Tous tes multiplicateurs sont doublés (×{})", fmt_mult(self.bb_mult())));
            }
            return;
        }
        if self.screen == Screen::Bench {
            match self.bench_phase {
                0 => {
                    // calcul pendant ~8 ms par image, on mesure le temps réellement passé à calculer
                    let st = Instant::now();
                    let mut x = self.bench_x;
                    while st.elapsed() < Duration::from_millis(8) {
                        for j in 0..5000u32 {
                            x = x.sin() * (x.abs() + 1.5).sqrt() + (j % 7) as f64;
                            x %= 100.0;
                        }
                        self.bench_ops += 5000;
                    }
                    self.bench_x = x;
                    self.bench_work += st.elapsed().as_secs_f64();
                    self.bench_t += real;
                    if self.bench_t >= 1.2 {
                        self.bench_cpu = self.bench_ops as f64 / self.bench_work.max(1e-6) / 1e6;
                        self.bench_phase = 1;
                        self.bench_t = 0.0;
                        self.bench_frames = 0;
                    }
                }
                1 => {
                    self.bench_t += real;
                    if self.bench_t >= 1.5 {
                        self.bench_fps = self.bench_frames as f32 / self.bench_t;
                        self.bench_phase = 2;
                    }
                }
                _ => {}
            }
            ctx.animate();
            return;
        }
        if self.screen == Screen::Video {
            self.vt += real;
            ctx.animate();
            if self.vt * VIDEO_FPS >= VIDEO_FRAMES as f32 {
                self.screen = Screen::Game;
                self.vt = 0.0;
            }
            return;
        }
        self.anim_t = (self.anim_t + real) % 1000.0;
        self.bb_confirm = (self.bb_confirm - real).max(0.0);
        // animation des souris : un petit « clic » visible à chaque coup
        let old_t = self.cur_t;
        self.cur_t += real as f64;
        self.shake = (self.shake - real).max(0.0);
        if self.cur_n > 0 && matches!(self.screen, Screen::Game | Screen::Menu) {
            let l = self.layout(ctx);
            let m = self.cur_n.min(24);
            let vc = self.cur_cps().min(3.0);
            for i in 0..m {
                let off = i as f64 / m as f64;
                if ((self.cur_t * vc + off) - 0.125).floor() > ((old_t * vc + off) - 0.125).floor() {
                    let ang = self.cur_t as f32 * 0.7 + i as f32 * std::f32::consts::TAU / m as f32;
                    self.burst(ctx, l.cx + ang.cos() * l.r * 0.95, l.cy + ang.sin() * l.r * 0.95, 0.45);
                    self.pop = 1.0;
                    if self.floats.len() < 5 {
                        self.floats.push(Floater { x: l.cx + ang.cos() * l.r * 0.6, y: l.cy + ang.sin() * l.r * 0.6 - 8.0, t: 0.0, txt: format!("+{}", fmt(self.click_power() * self.cursor_bonus())) });
                    }
                }
            }
            ctx.animate();
        }
        self.update_fx(real);
        if !self.parts.is_empty() || !self.rings.is_empty() || self.beam_l > 0.0 || self.bolt_l > 0.0 || self.orb_pulse > 0.0 {
            ctx.animate();
        }
        if self.skin >= 4 {
            ctx.animate(); // animation de galaxie
        }
        let mut gain = self.rate() * real as f64;
        if self.cur_n > 0 {
            // cliqueurs : chaque clic compte comme un vrai clic (tous les multiplicateurs)
            self.auto_acc += self.cur_n as f64 * self.cur_cps() * real as f64;
            let k = self.auto_acc.floor();
            if k >= 1.0 {
                self.auto_acc -= k;
                gain += self.click_power() * k * self.cursor_bonus();
                self.clicks += k as u64;
                self.bump(ctx, 0, k as u64);
            }
        }
        self.pts += gain;
        self.total += gain;
        self.mana = (self.mana + self.mana_regen() * real).min(self.max_mana());
        self.frenzy_left = (self.frenzy_left - real).max(0.0);
        self.bless_left = (self.bless_left - real).max(0.0);
        self.clone_left = (self.clone_left - real).max(0.0);
        self.asc_left = (self.asc_left - real).max(0.0);
        let gi = self.grade_idx();
        if gi > self.last_grade {
            ctx.toast(format!("Nouveau grade : {} ! ×{}", GRADES[gi].0, GRADES[gi].2));
        }
        self.last_grade = gi;
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
            Screen::Tune3 => {
                self.code.clear();
                self.screen = Screen::Tune;
            }
            Screen::Tune4 => {
                self.code.clear();
                self.screen = Screen::Tune3;
            }
            Screen::Intro | Screen::DomainFx | Screen::VoidFx | Screen::TimeFx | Screen::OrbFx | Screen::BossIntro | Screen::EyeFx | Screen::SunFx | Screen::SolarFx => {}
            Screen::Settings => self.screen = Screen::Menu,
            Screen::Bench => self.screen = Screen::Settings,
            Screen::Skins => self.screen = Screen::Menu,
            Screen::Progress => self.screen = Screen::Menu,
            Screen::BigBang => {}
            Screen::Video => {} // impossible de fermer la vidéo
        }
        true
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        match self.screen {
            Screen::Video | Screen::BigBang | Screen::Intro | Screen::DomainFx | Screen::VoidFx | Screen::TimeFx | Screen::OrbFx | Screen::BossIntro | Screen::EyeFx | Screen::SunFx | Screen::SolarFx => {}
            Screen::Game => {
                let l = self.layout(ctx);
                match ev {
                    Event::Down { .. } => self.scroll.stop(),
                    Event::Drag { y, dy, .. } if y >= l.list_top => self.scroll.drag(dy),
                    Event::Wheel { dy } => self.scroll.drag(dy),
                    Event::DragEnd { vy, .. } => self.scroll.fling(vy),
                    Event::Key(Key::Space) | Event::Key(Key::Enter) => self.do_click(ctx, None),
                    Event::Tap { x, y } => {
                        if y < l.shop_top && self.mon_tap(ctx, x, y) {
                            return;
                        }
                        if self.boss_tap(ctx, x, y) {
                            return;
                        }
                        let (dx, dy) = (x - l.cx, y - l.cy);
                        if l.menu_btn.contains(x, y) {
                            self.screen = Screen::Menu;
                        } else if y >= l.shop_top && y < l.shop_top + l.tab_h {
                            let new = match (x / (ctx.width() / 5.0)) as usize {
                                0 => Tab::Shop,
                                1 => Tab::Magic,
                                2 => Tab::Castles,
                                3 => Tab::Cursors,
                                _ => Tab::Defense,
                            };
                            if new != self.tab {
                                self.tab = new;
                                self.scroll.stop();
                                self.scroll.jump_to(0.0);
                            }
                        } else if y >= l.list_top {
                            let rel = y - l.list_top + self.scroll.pos;
                            if rel >= 0.0 {
                                let row = (rel / l.row_h) as usize;
                                match self.tab {
                                    Tab::Shop => self.buy(ctx, row),
                                    Tab::Magic => self.magic_tap(ctx, row),
                                    Tab::Castles => self.castle_tap(ctx, row),
                                    Tab::Cursors => self.cursor_tap(ctx, row),
                                    Tab::Defense => self.defense_tap(ctx, row),
                                }
                            }
                        } else if y < l.shop_top && dx * dx + dy * dy <= l.r * l.r * 1.2 {
                            self.do_click(ctx, Some((x, y)));
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
                        self.skin_scroll.stop();
                        self.skin_scroll.jump_to(0.0);
                        self.screen = Screen::Skins;
                    } else if b[2].contains(x, y) {
                        self.prog_scroll.stop();
                        self.prog_scroll.jump_to(0.0);
                        self.screen = Screen::Progress;
                    } else if b[3].contains(x, y) {
                        self.code.clear();
                        self.screen = Screen::Code;
                    } else if b[4].contains(x, y) {
                        self.screen = Screen::Settings;
                    } else if b[5].contains(x, y) {
                        self.save();
                        ctx.exit();
                    }
                }
            }
            Screen::Settings => {
                if let Event::Tap { x, y } = ev {
                    if y < 44.0 * ctx.scale() && x < ctx.width() * 0.32 {
                        self.screen = Screen::Menu;
                    } else if self.music_row(ctx).contains(x, y) {
                        self.music_on = !self.music_on;
                        self.music_update();
                        self.save();
                        ctx.animate();
                    } else if self.bench_row(ctx).contains(x, y) {
                        self.start_bench();
                    } else if self.settings_row(ctx).contains(x, y) {
                        self.fps_on = !self.fps_on;
                        self.fps_val = 0.0;
                        self.fps_frames = 0;
                        self.fps_t = Instant::now();
                        self.fps_cycle = 0.0;
                        self.save();
                        ctx.animate();
                    }
                }
            }
            Screen::Progress => {
                let s = ctx.scale();
                match ev {
                    Event::Down { .. } => self.prog_scroll.stop(),
                    Event::Drag { dy, .. } => self.prog_scroll.drag(dy),
                    Event::Wheel { dy } => self.prog_scroll.drag(dy),
                    Event::DragEnd { vy, .. } => self.prog_scroll.fling(vy),
                    Event::Tap { x, y } => {
                        let tabs_y = 44.0 * s;
                        let top = tabs_y + 34.0 * s + 22.0 * s;
                        if y < 44.0 * s && x < ctx.width() * 0.32 {
                            self.screen = Screen::Menu;
                        } else if y >= tabs_y && y < tabs_y + 34.0 * s {
                            let nt = ((x / (ctx.width() / 3.0)) as u8).min(2);
                            if nt != self.prog_tab {
                                self.prog_tab = nt;
                                self.prog_scroll.stop();
                                self.prog_scroll.jump_to(0.0);
                            }
                        } else if y >= top {
                            let rel = y - top + self.prog_scroll.pos;
                            let i = (rel / (60.0 * s)) as usize;
                            let rows = self.prog_rows();
                            if i < rows.len() {
                                let (act, idx) = (rows[i].act, rows[i].idx);
                                match act {
                                    1 => self.open_chest(ctx),
                                    2 => self.buy_talent(ctx, idx),
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Screen::Skins => {
                let s = ctx.scale();
                match ev {
                    Event::Down { .. } => self.skin_scroll.stop(),
                    Event::Drag { dy, .. } => self.skin_scroll.drag(dy),
                    Event::Wheel { dy } => self.skin_scroll.drag(dy),
                    Event::DragEnd { vy, .. } => self.skin_scroll.fling(vy),
                    Event::Tap { x, y } => {
                        let top = 44.0 * s + 22.0 * s;
                        if y < 44.0 * s && x < ctx.width() * 0.32 {
                            self.screen = Screen::Menu;
                        } else if y >= top {
                            let rel = y - top + self.skin_scroll.pos;
                            let i = (rel / (58.0 * s)) as usize;
                            if i < SKIN_NAMES.len() {
                                if !self.own[i] {
                                    ctx.toast("Verrouillé : débloque-le avec son code");
                                } else if self.skin as usize != i {
                                    self.skin = i as u8;
                                    self.save();
                                    ctx.toast(format!("Château équipé : {} (×{})", SKIN_NAMES[i], fmt_mult(skin_base(i as u8))));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Screen::Bench => {
                if let Event::Tap { x, y } = ev {
                    if y < 44.0 * ctx.scale() && x < ctx.width() * 0.32 {
                        self.screen = Screen::Settings;
                    } else if self.bench_phase == 2 && self.bench_btn(ctx).contains(x, y) {
                        self.start_bench();
                    }
                }
            }
            Screen::Code | Screen::Tune | Screen::Tune3 | Screen::Tune4 => match ev {
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
                                Kb::Space if self.code.len() < MAX_CODE => self.code.push(' '),
                                _ => {}
                            }
                            return;
                        }
                    }
                    if self.submit_rect(ctx).contains(x, y) {
                        self.submit_code(ctx);
                    }
                }
                Event::Key(Key::Char(c)) if (c.is_ascii_alphanumeric() || c == '?') && self.code.len() < MAX_CODE => {
                    self.code.push(c.to_ascii_uppercase());
                }
                Event::Key(Key::Space) if self.code.len() < MAX_CODE => self.code.push(' '),
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
        app.cur_n = 6;
        app.cur_spd = 1;
        app.cur_t = 0.37;
        app.cs[0] = true;
        app.cs[1] = true;
        app.mg[0] = true;
        app.mg[2] = true;
        for i in 0..4 {
            app.owned[i] = 3 + i as u32;
        }
    }
    if arg_flag("--demo") {
        for i in 0..9 {
            app.own[i] = true;
        }
    }
    if arg_flag("--boss") {
        app.boss = Some(Boss { hp: 14, max: 20, skin: arg_value("--bskin").or_else(|| arg_value("--skin")).and_then(|v| v.parse().ok()).unwrap_or(app.skin), lvl: 3, side: 1, fl: 0.0, kb: 0.0, age: 5.0, test: false });
    }
    if let Some(n) = arg_value("--def").and_then(|v| v.parse::<u32>().ok()) {
        app.def_g = n.min(8);
        app.def_next = now_secs() + 60.0;
        for i in 0..6u32 {
            let a = i as f32 * 1.05 + 0.3;
            app.mons.push(Mon { id: i as u64 + 1, x: 136.0 + a.cos() * 125.0, y: 150.0 + a.sin() * 70.0, hp: 6.0, max: 10.0, big: i == 0, fl: 0.0, ph: i as f32, sp: 0.0 });
        }
        app.mon_id = 10;
    }
    if arg_flag("--sun") {
        app.sun_until = now_secs() + 170.0;
    }
    if arg_flag("--infdom") {
        app.inf_dom = true;
    }
    if arg_flag("--progress") {
        app.tp = 5;
        for i in 0..8 {
            app.ach[i] = true;
        }
        app.st = [1500, 12, 80, 3, 2, 4];
        app.tl = [2, 1, 0, 1, 0, 0];
        app.q = vec![(0, 250, false), (2, 30, true), (3, 0, false)];
        app.chest = 0;
    }
    if arg_flag("--ability") {
        app.ext_until = now_secs() + 250.0;
        app.ext_cd = app.ext_until + 300.0;
        app.vide_until = now_secs() + 170.0;
        app.vide_cd = app.vide_until + 480.0;
    }
    if arg_flag("--burst") {
        app.demo_burst = true;
    }
    if arg_flag("--fps") {
        app.fps_on = true;
        app.fps_val = 60.0;
    }
    if let Some(k) = arg_value("--skin").and_then(|v| v.parse::<u8>().ok()) {
        app.skin = k.min(11);
        app.own[app.skin as usize] = true;
    }
    match arg_value("--page").as_deref() {
        Some("menu") => app.screen = Screen::Menu,
        Some("magic") => app.tab = Tab::Magic,
        Some("castles") => app.tab = Tab::Castles,
        Some("cursors") => app.tab = Tab::Cursors,
        Some("defense") => app.tab = Tab::Defense,
        Some("intro") => {
            app.skin = 8;
            app.screen = Screen::Intro;
            app.intro_t = arg_value("--bbt").and_then(|v| v.parse().ok()).unwrap_or(2.2);
        }
        Some("tune4") => app.screen = Screen::Tune4,
        Some("bigbang") => {
            app.bb = 1;
            app.screen = Screen::BigBang;
            app.bbfx_t = arg_value("--bbt").and_then(|v| v.parse().ok()).unwrap_or(2.0);
        }
        Some("tune3") => {
            app.screen = Screen::Tune3;
            app.code = "GAL".into();
        }
        Some("settings") => app.screen = Screen::Settings,
        Some("skins") => app.screen = Screen::Skins,
        Some("sunfx") => {
            app.sun_t = arg_value("--bbt").and_then(|v| v.parse().ok()).unwrap_or(1.0);
            app.skin = 11;
            app.screen = Screen::SunFx;
        }
        Some("solarfx") => {
            app.solar_t = arg_value("--bbt").and_then(|v| v.parse().ok()).unwrap_or(2.5);
            app.skin = 11;
            app.screen = Screen::SolarFx;
        }
        Some("eyefx") => {
            app.eye_t = arg_value("--bbt").and_then(|v| v.parse().ok()).unwrap_or(2.5);
            app.skin = 10;
            app.screen = Screen::EyeFx;
        }
        Some("bossintro") => {
            let bt: f32 = arg_value("--bbt").and_then(|v| v.parse().ok()).unwrap_or(1.2);
            let sk: u8 = arg_value("--bskin").and_then(|v| v.parse().ok()).unwrap_or(0);
            app.skin = sk;
            app.boss = Some(Boss { hp: 20, max: 20, skin: sk, lvl: 1, side: 1, fl: 0.0, kb: 0.0, age: bt - 1.7, test: false });
            app.screen = Screen::BossIntro;
            app.boss_intro_t = bt;
        }
        Some("progress") => {
            app.screen = Screen::Progress;
            app.prog_tab = arg_value("--tab").and_then(|v| v.parse().ok()).unwrap_or(0);
        }
        Some("orbfx") => {
            app.skin = 8;
            app.boss = Some(Boss { hp: 12, max: 20, skin: 8, lvl: 1, side: 1, fl: 0.0, kb: 0.0, age: 5.0, test: false });
            app.orb_target = (230.0, 170.0);
            app.screen = Screen::OrbFx;
            app.orb_t = arg_value("--bbt").and_then(|v| v.parse().ok()).unwrap_or(1.2);
        }
        Some("timefx") => {
            app.skin = 9;
            app.time_gain = 6.6e12;
            app.screen = Screen::TimeFx;
            app.time_t = arg_value("--bbt").and_then(|v| v.parse().ok()).unwrap_or(1.5);
        }
        Some("domainfx") => {
            app.skin = 8;
            app.screen = Screen::DomainFx;
            app.dom_t = arg_value("--bbt").and_then(|v| v.parse().ok()).unwrap_or(2.0);
        }
        Some("voidfx") => {
            app.skin = 6;
            app.screen = Screen::VoidFx;
            app.void_t = arg_value("--bbt").and_then(|v| v.parse().ok()).unwrap_or(1.6);
        }
        Some("bench") => {
            app.screen = Screen::Bench;
            app.bench_phase = 2;
            app.bench_cpu = 412.0;
            app.bench_fps = 59.0;
        }
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
