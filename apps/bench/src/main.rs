// Bench : l'établi électronique de poche (écran portrait 272x480).
//
//  - Bagues  : code couleur des résistances, dessiné et interactif (4 ou 5 bagues), avec navigation
//              dans les séries E24 / E96 (‹ Préc. / Suiv. ›)
//  - Diviseur: trouve les meilleures paires R1/R2 (E12 / E24 / E96) pour un pont diviseur ou pour
//              le réseau de retour (FB) d'un régulateur : Vout = Vref (1 + R1/R2)
//  - LED     : résistance de limitation, valeur E24 supérieure, courant réel, dissipation
//  - RC      : fréquence de coupure, constante de temps, temps de montée
//
// Sur PC : cargo run -- --screenshot out.png [--tab 0..3] [--fb]

use lapp_sdk::prelude::*;

// ---------------------------------------------------------------- données

const NAMES: [&str; 12] = ["Noir", "Marron", "Rouge", "Orange", "Jaune", "Vert", "Bleu", "Violet", "Gris", "Blanc", "Or", "Argent"];
const COLORS: [u32; 12] = [
    0x16161A, 0x7B4A1E, 0xD62828, 0xF77F00, 0xF2C500, 0x2E9E48, 0x2563EB, 0x8B3FD9, 0x8D8D96, 0xF4F4F4, 0xC9A227, 0xB8BCC4,
];
/// (couleur, tolérance en %)
const TOL: [(usize, f64); 8] = [(1, 1.0), (2, 2.0), (5, 0.5), (6, 0.25), (7, 0.1), (8, 0.05), (10, 5.0), (11, 10.0)];

const E12: [u32; 12] = [100, 120, 150, 180, 220, 270, 330, 390, 470, 560, 680, 820];
const E24: [u32; 24] = [
    100, 110, 120, 130, 150, 160, 180, 200, 220, 240, 270, 300, 330, 360, 390, 430, 470, 510, 560, 620, 680, 750, 820, 910,
];
const E96: [u32; 96] = [
    100, 102, 105, 107, 110, 113, 115, 118, 121, 124, 127, 130, 133, 137, 140, 143, 147, 150, 154, 158, 162, 165, 169, 174, 178, 182,
    187, 191, 196, 200, 205, 210, 215, 221, 226, 232, 237, 243, 249, 255, 261, 267, 274, 280, 287, 294, 301, 309, 316, 324, 332, 340,
    348, 357, 365, 374, 383, 392, 402, 412, 422, 432, 442, 453, 464, 475, 487, 499, 511, 523, 536, 549, 562, 576, 590, 604, 619, 634,
    649, 665, 681, 698, 715, 732, 750, 768, 787, 806, 825, 845, 866, 887, 909, 931, 953, 976,
];

fn mantissas(series: usize) -> &'static [u32] {
    match series {
        0 => &E12,
        1 => &E24,
        _ => &E96,
    }
}

/// Toutes les valeurs d'une série entre les décades 10^lo et 10^hi (incluses), triées.
fn series_values(series: usize, lo: i32, hi: i32) -> Vec<f64> {
    let mut v = Vec::new();
    for d in lo..=hi {
        let k = 10f64.powi(d);
        for &m in mantissas(series) {
            v.push(m as f64 / 100.0 * k);
        }
    }
    v
}

// ---------------------------------------------------------------- formats

/// Valeur avec préfixe SI et `sig` chiffres significatifs, virgule décimale française.
fn si(v: f64, unit: &str, sig: i32) -> String {
    if !v.is_finite() {
        return "—".into();
    }
    if v == 0.0 {
        return format!("0 {unit}");
    }
    const P: [(f64, &str); 8] = [(1e9, "G"), (1e6, "M"), (1e3, "k"), (1.0, ""), (1e-3, "m"), (1e-6, "µ"), (1e-9, "n"), (1e-12, "p")];
    let a = v.abs();
    let (mut mult, mut pre) = P.iter().copied().find(|(m, _)| a >= m * 0.9995).unwrap_or((1e-12, "p"));
    if unit == "Ω" && mult < 1.0 && a >= 0.01 {
        mult = 1.0;
        pre = "";
    }
    let x = v / mult;
    let int_digits = if x.abs() >= 1.0 { x.abs().log10().floor() as i32 + 1 } else { 0 };
    let dec = (sig - int_digits).max(0) as usize;
    let mut s = format!("{:.*}", dec, x);
    if s.contains('.') {
        s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    format!("{} {}{}", s.replace('.', ","), pre, unit)
}

fn pct(e: f64) -> String {
    format!("{:+.2} %", e * 100.0).replace('.', ",")
}

// ---------------------------------------------------------------- modèle

#[derive(Clone, Copy, PartialEq, Debug)]
enum Hit {
    Tab(usize),
    Band(usize),
    Swatch(usize),
    Prev,
    Next,
    Nb,
    Field(usize),
    Unit(usize),
    Key(usize),
}

#[derive(Clone, Copy)]
enum FKind {
    Num(&'static [(&'static str, f64)]),
    Choice(&'static [&'static str]),
}

#[derive(Clone)]
struct Field {
    label: &'static str,
    kind: FKind,
    text: String,
    /// unité choisie (Num) ou option choisie (Choice)
    unit: usize,
}

static U_V: [(&str, f64); 1] = [("V", 1.0)];
static U_I: [(&str, f64); 2] = [("mA", 1e-3), ("µA", 1e-6)];
static U_R: [(&str, f64); 3] = [("Ω", 1.0), ("kΩ", 1e3), ("MΩ", 1e6)];
static U_C: [(&str, f64); 3] = [("pF", 1e-12), ("nF", 1e-9), ("µF", 1e-6)];
static MODES: [&str; 2] = ["Diviseur", "Retour FB"];
static SERIES: [&str; 3] = ["E12", "E24", "E96"];

fn fnum(label: &'static str, units: &'static [(&'static str, f64)], text: &str, unit: usize) -> Field {
    Field { label, kind: FKind::Num(units), text: text.to_string(), unit }
}

fn fchoice(label: &'static str, opts: &'static [&'static str], sel: usize) -> Field {
    Field { label, kind: FKind::Choice(opts), text: String::new(), unit: sel }
}

fn default_fields() -> Vec<Vec<Field>> {
    vec![
        vec![],
        vec![fchoice("Type", &MODES, 0), fchoice("Série", &SERIES, 2), fnum("Vin", &U_V, "5", 0), fnum("Vout", &U_V, "3.3", 0)],
        vec![fnum("Vs", &U_V, "5", 0), fnum("Vf LED", &U_V, "2", 0), fnum("Courant", &U_I, "10", 0)],
        vec![fnum("R", &U_R, "10", 1), fnum("C", &U_C, "100", 1)],
    ]
}

#[derive(Debug)]
struct Row {
    left: String,
    right: String,
    /// 0 normale, 1 mise en avant, 2 erreur
    kind: u8,
}

fn err(msg: &str) -> Row {
    Row { left: msg.to_string(), right: String::new(), kind: 2 }
}

#[derive(Clone)]
struct Cand {
    r1: f64,
    r2: f64,
    v: f64,
    err: f64,
    score: f64,
}

struct Bench {
    store: Store,
    tab: usize,
    pressed: Option<Hit>,
    // bagues
    nb: usize,
    bands: Vec<usize>,
    band: usize,
    // champs numériques
    fields: Vec<Vec<Field>>,
    sel: [usize; 4],
    fresh: bool,
    div_cache: Option<((usize, usize, u64, u64), Vec<Cand>)>,
}

const KEYS: [char; 12] = ['7', '8', '9', '4', '5', '6', '1', '2', '3', '.', '0', '\x08'];

fn role(nb: usize, i: usize) -> u8 {
    let nd = nb - 2;
    if i < nd {
        0
    } else if i == nd {
        1
    } else {
        2
    }
}

fn mult_val(c: usize) -> f64 {
    match c {
        0..=9 => 10f64.powi(c as i32),
        10 => 0.1,
        _ => 0.01,
    }
}

fn tol_of(c: usize) -> Option<f64> {
    TOL.iter().find(|(k, _)| *k == c).map(|(_, v)| *v)
}

impl Bench {
    fn new(store: Store) -> Bench {
        Bench {
            store,
            tab: 0,
            pressed: None,
            nb: 4,
            bands: vec![4, 7, 2, 10],
            band: 0,
            fields: default_fields(),
            sel: [0, 2, 0, 0],
            fresh: true,
            div_cache: None,
        }
    }

    // ------------------------------------------------------- persistance

    fn save(&mut self) {
        let bands = self.bands.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(",");
        let mut parts = vec![self.tab.to_string(), self.nb.to_string(), bands];
        for t in 1..4 {
            parts.push(self.fields[t].iter().map(|f| format!("{}~{}", f.text, f.unit)).collect::<Vec<_>>().join(","));
        }
        let _ = self.store.set("state", parts.join(";"));
    }

    fn load(&mut self) {
        let Some(s) = self.store.get("state").map(|s| s.to_string()) else { return };
        let p: Vec<&str> = s.split(';').collect();
        if p.len() != 6 {
            return;
        }
        if let (Ok(tab), Ok(nb)) = (p[0].parse::<usize>(), p[1].parse::<usize>()) {
            let bands: Vec<usize> = p[2].split(',').filter_map(|b| b.parse().ok()).collect();
            if tab < 4 && (nb == 4 || nb == 5) && bands.len() == nb && bands.iter().all(|&b| b < 12) {
                self.tab = tab;
                self.nb = nb;
                self.bands = bands;
            }
        }
        for t in 1..4 {
            let items: Vec<&str> = p[t + 2].split(',').collect();
            if items.len() != self.fields[t].len() {
                continue;
            }
            for (f, it) in self.fields[t].iter_mut().zip(items) {
                let Some((text, unit)) = it.split_once('~') else { continue };
                let Ok(unit) = unit.parse::<usize>() else { continue };
                let n = match f.kind {
                    FKind::Num(u) => u.len(),
                    FKind::Choice(o) => o.len(),
                };
                if unit < n && text.len() <= 9 && text.chars().all(|c| c.is_ascii_digit() || c == '.') {
                    f.text = text.to_string();
                    f.unit = unit;
                }
            }
        }
    }

    // ------------------------------------------------------- bagues

    fn allowed(&self, band: usize, c: usize) -> bool {
        match role(self.nb, band) {
            0 => c <= 9 && !(band == 0 && c == 0),
            1 => true,
            _ => tol_of(c).is_some(),
        }
    }

    fn bands_value(&self) -> f64 {
        let nd = self.nb - 2;
        let mut n = 0.0;
        for i in 0..nd {
            n = n * 10.0 + self.bands[i].min(9) as f64;
        }
        n * mult_val(self.bands[nd])
    }

    fn bands_tol(&self) -> f64 {
        tol_of(self.bands[self.nb - 1]).unwrap_or(5.0)
    }

    /// Règle les chiffres et le multiplicateur pour approcher `v` (la tolérance est conservée).
    fn set_value(&mut self, v: f64) {
        if !(v > 0.0) || !v.is_finite() {
            return;
        }
        let nd = self.nb - 2;
        let mut e = v.log10().floor() as i32 - (nd as i32 - 1);
        let mut n = (v / 10f64.powi(e)).round() as i64;
        if n >= 10i64.pow(nd as u32) {
            n /= 10;
            e += 1;
        }
        let mult = if e >= 0 { (e as usize).min(9) } else if e == -1 { 10 } else { 11 };
        for i in 0..nd {
            let p = 10i64.pow((nd - 1 - i) as u32);
            self.bands[i] = ((n / p) % 10) as usize;
        }
        self.bands[nd] = mult;
    }

    fn step(&mut self, ctx: &Ctx, dir: i32) {
        let list = series_values(if self.nb == 4 { 1 } else { 2 }, -1, 8);
        let cur = self.bands_value();
        let target = if dir > 0 {
            list.iter().copied().find(|v| *v > cur * 1.0001)
        } else {
            list.iter().copied().rev().find(|v| *v < cur * 0.9999)
        };
        match target {
            Some(v) => self.set_value(v),
            None => ctx.toast("Limite de la série"),
        }
    }

    fn toggle_bands(&mut self) {
        let v = self.bands_value();
        let tol = self.bands[self.nb - 1];
        self.nb = if self.nb == 4 { 5 } else { 4 };
        self.bands = vec![0; self.nb];
        self.set_value(v);
        let last = self.nb - 1;
        self.bands[last] = tol;
        self.band = self.band.min(self.nb - 1);
    }

    // ------------------------------------------------------- champs

    fn label(&self, tab: usize, i: usize) -> &'static str {
        if tab == 1 && i == 2 && self.fields[1][0].unit == 1 {
            "Vref"
        } else {
            self.fields[tab][i].label
        }
    }

    fn num(&self, tab: usize, i: usize) -> Option<f64> {
        let f = &self.fields[tab][i];
        let FKind::Num(u) = f.kind else { return None };
        let v: f64 = f.text.parse().ok()?;
        Some(v * u[f.unit].1)
    }

    fn cycle(&mut self, tab: usize, i: usize) {
        let f = &mut self.fields[tab][i];
        let n = match f.kind {
            FKind::Num(u) => u.len(),
            FKind::Choice(o) => o.len(),
        };
        f.unit = (f.unit + 1) % n;
        // Diviseur <-> Retour FB : on propose une valeur d'entrée adaptée
        if tab == 1 && i == 0 {
            let mode = self.fields[1][0].unit;
            self.fields[1][2].text = if mode == 1 { "0.6".into() } else { "5".into() };
            self.fresh = true;
            self.div_cache = None;
        }
    }

    fn type_char(&mut self, c: char) {
        let tab = self.tab;
        if tab == 0 {
            return;
        }
        let i = self.sel[tab];
        let fresh = self.fresh;
        let f = &mut self.fields[tab][i];
        if !matches!(f.kind, FKind::Num(_)) {
            return;
        }
        match c {
            '\x08' => {
                if fresh {
                    f.text.clear();
                } else {
                    f.text.pop();
                }
            }
            '.' => {
                if fresh {
                    f.text.clear();
                }
                if !f.text.contains('.') {
                    if f.text.is_empty() {
                        f.text.push('0');
                    }
                    f.text.push('.');
                }
            }
            d => {
                if fresh || f.text == "0" {
                    f.text.clear();
                }
                if f.text.len() < 9 {
                    f.text.push(d);
                }
            }
        }
        self.fresh = false;
        self.save();
    }

    fn next_field(&mut self, dir: i32) {
        let tab = self.tab;
        let n = self.fields[tab].len() as i32;
        let mut i = self.sel[tab] as i32;
        for _ in 0..n {
            i = (i + dir).rem_euclid(n);
            if matches!(self.fields[tab][i as usize].kind, FKind::Num(_)) {
                self.sel[tab] = i as usize;
                self.fresh = true;
                return;
            }
        }
    }

    // ------------------------------------------------------- résultats

    fn rows_divider(&mut self) -> Vec<Row> {
        let mode = self.fields[1][0].unit;
        let ser = self.fields[1][1].unit;
        let (Some(a), Some(vout)) = (self.num(1, 2), self.num(1, 3)) else {
            return vec![err("Saisissez les deux tensions")];
        };
        if a <= 0.0 || vout <= 0.0 {
            return vec![err("Les tensions doivent être > 0")];
        }
        if mode == 0 && vout >= a {
            return vec![err("Vout doit être inférieure à Vin")];
        }
        if mode == 1 && vout <= a {
            return vec![err("Vout doit dépasser Vref")];
        }
        let key = (mode, ser, a.to_bits(), vout.to_bits());
        let fresh_cache = matches!(&self.div_cache, Some((k, _)) if *k == key);
        if !fresh_cache {
            let vals = series_values(ser, 3, 5); // 1 kΩ .. < 1 MΩ
            let mut c: Vec<Cand> = Vec::new();
            for &r1 in &vals {
                for &r2 in &vals {
                    let v = if mode == 0 { a * r2 / (r1 + r2) } else { a * (1.0 + r1 / r2) };
                    let e = (v - vout) / vout;
                    if e.abs() > 0.05 {
                        continue;
                    }
                    // erreur d'abord ; à égalité, on préfère un courant de pont raisonnable (~50 kΩ au total)
                    let score = e.abs() * 100.0 + 0.03 * ((r1 + r2) / 47000.0).log10().abs();
                    c.push(Cand { r1, r2, v, err: e, score });
                }
            }
            c.sort_by(|x, y| x.score.partial_cmp(&y.score).unwrap_or(std::cmp::Ordering::Equal));
            let mut out: Vec<Cand> = Vec::new();
            for cand in c {
                if out.iter().all(|o| ((o.r1 / o.r2) / (cand.r1 / cand.r2) - 1.0).abs() > 1e-6) {
                    out.push(cand);
                    if out.len() == 3 {
                        break;
                    }
                }
            }
            self.div_cache = Some((key, out));
        }
        let cands = &self.div_cache.as_ref().unwrap().1;
        if cands.is_empty() {
            return vec![err("Aucune paire à moins de 5 %")];
        }
        cands
            .iter()
            .enumerate()
            .map(|(i, c)| Row {
                left: format!("R1 {}   R2 {}", si(c.r1, "Ω", 3), si(c.r2, "Ω", 3)),
                right: format!("{}  {}", si(c.v, "V", 4), pct(c.err)),
                kind: if i == 0 { 1 } else { 0 },
            })
            .collect()
    }

    fn rows_led(&self) -> Vec<Row> {
        let (Some(vs), Some(vf), Some(i)) = (self.num(2, 0), self.num(2, 1), self.num(2, 2)) else {
            return vec![err("Saisissez Vs, Vf et le courant")];
        };
        if vs <= vf {
            return vec![err("Vs doit dépasser Vf")];
        }
        if i <= 0.0 {
            return vec![err("Le courant doit être > 0")];
        }
        let r = (vs - vf) / i;
        let std = series_values(1, -1, 7).into_iter().find(|v| *v >= r * (1.0 - 1e-9)).unwrap_or(r);
        let i2 = (vs - vf) / std;
        let p = (vs - vf) * i2;
        vec![
            Row { left: "R exacte".into(), right: si(r, "Ω", 3), kind: 1 },
            Row { left: "E24 supérieure".into(), right: format!("{} → {}", si(std, "Ω", 3), si(i2, "A", 3)), kind: 0 },
            Row { left: "Dissipation".into(), right: format!("{}{}", si(p, "W", 2), if p > 0.25 { "  (> 1/4 W)" } else { "" }), kind: 0 },
        ]
    }

    fn rows_rc(&self) -> Vec<Row> {
        let (Some(r), Some(c)) = (self.num(3, 0), self.num(3, 1)) else {
            return vec![err("Saisissez R et C")];
        };
        if r <= 0.0 || c <= 0.0 {
            return vec![err("R et C doivent être > 0")];
        }
        let tau = r * c;
        let fc = 1.0 / (2.0 * std::f64::consts::PI * tau);
        vec![
            Row { left: "Fréquence de coupure".into(), right: si(fc, "Hz", 3), kind: 1 },
            Row { left: "Constante τ = R·C".into(), right: si(tau, "s", 3), kind: 0 },
            Row { left: "Montée 10–90 %".into(), right: si(2.197 * tau, "s", 3), kind: 0 },
        ]
    }

    // ------------------------------------------------------- interaction

    fn hit(&self, ctx: &Ctx, x: f32, y: f32) -> Option<Hit> {
        let s = ctx.scale();
        for i in 0..4 {
            if tab_rect(ctx, i).contains(x, y) {
                return Some(Hit::Tab(i));
            }
        }
        if self.tab == 0 {
            for (i, r) in band_rects(ctx, self.nb).iter().enumerate() {
                if Rect::new(r.x - 3.0 * s, r.y - 8.0 * s, r.w + 6.0 * s, r.h + 16.0 * s).contains(x, y) {
                    return Some(Hit::Band(i));
                }
            }
            let btns = [Hit::Prev, Hit::Nb, Hit::Next];
            for (i, h) in btns.iter().enumerate() {
                if btn_rect(ctx, i).contains(x, y) {
                    return Some(*h);
                }
            }
            for c in 0..12 {
                if swatch_rect(ctx, c).contains(x, y) {
                    return Some(Hit::Swatch(c));
                }
            }
        } else {
            for (i, f) in self.fields[self.tab].iter().enumerate() {
                if unit_rect(ctx, i, matches!(f.kind, FKind::Choice(_))).contains(x, y) {
                    return Some(Hit::Unit(i));
                }
                if field_rect(ctx, i).contains(x, y) {
                    return Some(Hit::Field(i));
                }
            }
            for k in 0..12 {
                if key_rect(ctx, k).contains(x, y) {
                    return Some(Hit::Key(k));
                }
            }
        }
        None
    }

    fn act(&mut self, ctx: &Ctx, h: Hit) {
        match h {
            Hit::Tab(i) => {
                self.tab = i;
                self.fresh = true;
            }
            Hit::Band(i) => self.band = i,
            Hit::Swatch(c) => {
                if self.allowed(self.band, c) {
                    self.bands[self.band] = c;
                    if self.band + 1 < self.nb {
                        self.band += 1;
                    }
                }
            }
            Hit::Prev => self.step(ctx, -1),
            Hit::Next => self.step(ctx, 1),
            Hit::Nb => self.toggle_bands(),
            Hit::Field(i) => {
                if matches!(self.fields[self.tab][i].kind, FKind::Choice(_)) {
                    self.cycle(self.tab, i);
                } else {
                    self.sel[self.tab] = i;
                    self.fresh = true;
                }
            }
            Hit::Unit(i) => {
                if matches!(self.fields[self.tab][i].kind, FKind::Num(_)) {
                    if self.sel[self.tab] != i {
                        self.sel[self.tab] = i;
                        self.fresh = true;
                    }
                }
                self.cycle(self.tab, i);
            }
            Hit::Key(k) => self.type_char(KEYS[k]),
        }
        self.save();
    }
}

// ---------------------------------------------------------------- géométrie

fn tab_rect(ctx: &Ctx, i: usize) -> Rect {
    let w = ctx.width() / 4.0;
    Rect::new(w * i as f32, 44.0 * ctx.scale(), w, 32.0 * ctx.scale())
}

fn field_rect(ctx: &Ctx, i: usize) -> Rect {
    let s = ctx.scale();
    Rect::new(0.0, (82.0 + 32.0 * i as f32) * s, ctx.width(), 32.0 * s)
}

fn unit_rect(ctx: &Ctx, i: usize, wide: bool) -> Rect {
    let s = ctx.scale();
    let r = field_rect(ctx, i);
    let w = if wide { 104.0 * s } else { 52.0 * s };
    Rect::new(r.right() - w - 10.0 * s, r.y + 4.0 * s, w, 24.0 * s)
}

fn key_rect(ctx: &Ctx, k: usize) -> Rect {
    let s = ctx.scale();
    let (m, gap, kh) = (10.0 * s, 6.0 * s, 30.0 * s);
    let kw = (ctx.width() - 2.0 * m - 2.0 * gap) / 3.0;
    let top = ctx.height() - 34.0 * s - (4.0 * 30.0 + 3.0 * 6.0) * s;
    let (row, col) = ((k / 3) as f32, (k % 3) as f32);
    Rect::new(m + col * (kw + gap), top + row * (kh + gap), kw, kh)
}

fn band_rects(ctx: &Ctx, nb: usize) -> Vec<Rect> {
    let s = ctx.scale();
    let xs: &[f32] = if nb == 4 { &[74.0, 96.0, 118.0, 190.0] } else { &[68.0, 87.0, 106.0, 125.0, 192.0] };
    xs.iter().map(|x| Rect::new(x * s, 98.0 * s, 13.0 * s, 54.0 * s)).collect()
}

fn btn_rect(ctx: &Ctx, i: usize) -> Rect {
    let s = ctx.scale();
    let (m, gap) = (10.0 * s, 6.0 * s);
    let bw = (ctx.width() - 2.0 * m - 2.0 * gap) / 3.0;
    Rect::new(m + i as f32 * (bw + gap), 226.0 * s, bw, 32.0 * s)
}

fn swatch_rect(ctx: &Ctx, c: usize) -> Rect {
    let s = ctx.scale();
    let (m, gap) = (10.0 * s, 6.0 * s);
    let kw = (ctx.width() - 2.0 * m - 3.0 * gap) / 4.0;
    let (row, col) = ((c / 4) as f32, (c % 4) as f32);
    Rect::new(m + col * (kw + gap), 270.0 * s + row * 38.0 * s, kw, 32.0 * s)
}

// ---------------------------------------------------------------- dessin

impl Bench {
    fn draw_tabs(&self, ctx: &Ctx, g: &mut Gfx) {
        let t = ctx.theme;
        let s = ctx.scale();
        let names = ["Bagues", "Diviseur", "LED", "RC"];
        let strip = tab_rect(ctx, 0);
        g.rect(0.0, strip.y, ctx.width(), strip.h, t.surface);
        for (i, n) in names.iter().enumerate() {
            let r = tab_rect(ctx, i);
            let on = self.tab == i;
            if self.pressed == Some(Hit::Tab(i)) {
                g.rect(r.x, r.y, r.w, r.h, t.press);
            }
            let mut st = Text::new(13.0 * s).color(if on { t.accent } else { t.subtext }).center();
            if on {
                st = st.bold();
            }
            let (cx, cy) = r.center();
            ctx.text_mid(g, n, cx, cy - 1.0 * s, &st);
            if on {
                g.rect(r.x + 10.0 * s, r.bottom() - 3.0 * s, r.w - 20.0 * s, 3.0 * s, t.accent);
            }
        }
        g.rect(0.0, strip.bottom() - 1.0, ctx.width(), 1.0, t.line);
    }

    fn render_bands(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = ctx.theme;
        let s = ctx.scale();
        let w = ctx.width();
        // carte de la résistance
        g.rrect(10.0 * s, 84.0 * s, w - 20.0 * s, 88.0 * s, 12.0 * s, t.surface);
        let rects = band_rects(ctx, self.nb);
        let body_top = 98.0 * s;
        let beige = rgb(0xE6D3A1);
        g.rect(18.0 * s, body_top + 25.0 * s, w - 36.0 * s, 4.0 * s, rgb(0xB9BDC7));
        g.rrect(51.0 * s, body_top - 1.0 * s, 170.0 * s, 56.0 * s, 15.0 * s, shade(beige, 0.78));
        g.rrect(52.0 * s, body_top, 168.0 * s, 54.0 * s, 14.0 * s, beige);
        for (i, r) in rects.iter().enumerate() {
            if i == self.band {
                g.rect(r.x - 2.0 * s, body_top - 5.0 * s, r.w + 4.0 * s, 64.0 * s, t.accent);
            }
            g.rect(r.x, r.y, r.w, r.h, rgb(COLORS[self.bands[i]]));
        }
        if let Some(r) = rects.get(self.band) {
            let (cx, _) = r.center();
            g.circle(cx, body_top + 54.0 * s + 12.0 * s, 3.0 * s, t.accent);
        }
        // valeur
        let v = self.bands_value();
        let tol = self.bands_tol();
        let sig = if self.nb == 4 { 2 } else { 3 };
        let big = Text::new(30.0 * s).bold().color(t.text).center();
        ctx.text_mid(g, &si(v, "Ω", sig), w / 2.0, 192.0 * s, &big);
        let sub = Text::new(12.0 * s).color(t.subtext).center().max_width(w - 20.0 * s);
        let line = format!(
            "±{} %   {} … {}",
            format!("{}", tol).replace('.', ","),
            si(v * (1.0 - tol / 100.0), "Ω", 3),
            si(v * (1.0 + tol / 100.0), "Ω", 3)
        );
        ctx.text_mid(g, &line, w / 2.0, 212.0 * s, &sub);
        // boutons
        let labels = ["‹ Préc.".to_string(), format!("{} bagues", self.nb), "Suiv. ›".to_string()];
        let hits = [Hit::Prev, Hit::Nb, Hit::Next];
        for i in 0..3 {
            ui::button(ctx, g, btn_rect(ctx, i), &labels[i], ButtonStyle::Secondary, self.pressed == Some(hits[i]));
        }
        // palette
        for c in 0..12 {
            let r = swatch_rect(ctx, c);
            let ok = self.allowed(self.band, c);
            if self.bands[self.band] == c {
                g.rrect(r.x - 2.5 * s, r.y - 2.5 * s, r.w + 5.0 * s, r.h + 5.0 * s, 10.0 * s, t.accent);
            }
            g.rrect(r.x, r.y, r.w, r.h, 8.0 * s, rgb(COLORS[c]));
            let hex = COLORS[c];
            let lum = (0.299 * ((hex >> 16) & 255) as f32 + 0.587 * ((hex >> 8) & 255) as f32 + 0.114 * (hex & 255) as f32) / 255.0;
            let fg = if lum > 0.6 { rgb(0x1C1C21) } else { rgb(0xFFFFFF) };
            let st = Text::new(11.0 * s).bold().color(fg).center();
            let (cx, cy) = r.center();
            ctx.text_mid(g, NAMES[c], cx, cy, &st);
            if !ok {
                g.rrect(r.x, r.y, r.w, r.h, 8.0 * s, with_alpha(t.bg, 0.72));
            } else if self.pressed == Some(Hit::Swatch(c)) {
                g.rrect(r.x, r.y, r.w, r.h, 8.0 * s, rgba(0x000000, 0.22));
            }
        }
        let hint = Text::new(11.0 * s).color(t.subtext).center();
        ctx.text_mid(g, "Touchez une bague, puis une couleur", w / 2.0, 396.0 * s, &hint);
    }

    fn render_fields(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = ctx.theme;
        let s = ctx.scale();
        let w = ctx.width();
        let tab = self.tab;
        let n = self.fields[tab].len();
        for i in 0..n {
            let r = field_rect(ctx, i);
            let f = self.fields[tab][i].clone();
            let is_num = matches!(f.kind, FKind::Num(_));
            let selected = is_num && self.sel[tab] == i;
            let pressed = self.pressed == Some(Hit::Field(i));
            g.rect(r.x, r.y, r.w, r.h, if selected || pressed { t.press } else { t.surface });
            g.rect(r.x + 12.0 * s, r.bottom() - 1.0, (r.w - 12.0 * s).max(0.0), 1.0, t.line);
            if selected {
                g.rect(r.x, r.y, 3.0 * s, r.h, t.accent);
            }
            let lab = Text::new(13.0 * s).color(t.subtext).left();
            ctx.text_mid(g, self.label(tab, i), r.x + 12.0 * s, r.y + r.h / 2.0, &lab);
            let chip = unit_rect(ctx, i, !is_num);
            g.rrect(chip.x, chip.y, chip.w, chip.h, chip.h / 2.0, if self.pressed == Some(Hit::Unit(i)) { t.press } else { t.bg });
            let chip_label = match f.kind {
                FKind::Num(u) => u[f.unit].0,
                FKind::Choice(o) => o[f.unit],
            };
            let cst = Text::new(12.0 * s).bold().color(t.accent).center();
            let (cx, cy) = chip.center();
            ctx.text_mid(g, chip_label, cx, cy, &cst);
            if let FKind::Num(_) = f.kind {
                let shown = if f.text.is_empty() { "0".to_string() } else { f.text.replace('.', ",") };
                let vst = Text::new(17.0 * s).bold().mono().color(if selected { t.accent } else { t.text }).right();
                let rx = chip.x - 10.0 * s;
                if selected && self.fresh {
                    let tw = ctx.measure(&shown, &vst);
                    g.rrect(rx - tw - 5.0 * s, r.y + 5.0 * s, tw + 10.0 * s, r.h - 10.0 * s, 5.0 * s, rgba(0x007AFF, 0.16));
                }
                ctx.text_mid(g, &shown, rx, r.y + r.h / 2.0, &vst);
            }
        }
        // résultats
        let rows = match tab {
            1 => self.rows_divider(),
            2 => self.rows_led(),
            _ => self.rows_rc(),
        };
        let top = field_rect(ctx, n - 1).bottom() + 6.0 * s;
        let heights: Vec<f32> = rows.iter().map(|r| if r.kind == 1 { 30.0 * s } else { 24.0 * s }).collect();
        let card_h = heights.iter().sum::<f32>() + 8.0 * s;
        g.rrect(10.0 * s, top, w - 20.0 * s, card_h, 10.0 * s, t.surface);
        let mut y = top + 4.0 * s;
        for (row, h) in rows.iter().zip(&heights) {
            let cy = y + h / 2.0;
            match row.kind {
                2 => {
                    let st = Text::new(13.0 * s).color(t.danger).center().max_width(w - 40.0 * s);
                    ctx.text_mid(g, &row.left, w / 2.0, cy, &st);
                }
                1 => {
                    let l = Text::new(13.0 * s).bold().color(t.text).left().max_width(w * 0.5);
                    let r = Text::new(15.0 * s).bold().color(t.accent).right().max_width(w * 0.5);
                    ctx.text_mid(g, &row.left, 20.0 * s, cy, &l);
                    ctx.text_mid(g, &row.right, w - 20.0 * s, cy, &r);
                }
                _ => {
                    let l = Text::new(12.0 * s).color(t.text).left().max_width(w * 0.5);
                    let r = Text::new(12.0 * s).color(t.subtext).right().max_width(w * 0.5);
                    ctx.text_mid(g, &row.left, 20.0 * s, cy, &l);
                    ctx.text_mid(g, &row.right, w - 20.0 * s, cy, &r);
                }
            }
            y += h;
        }
        // clavier numérique
        for k in 0..12 {
            let r = key_rect(ctx, k);
            let pressed = self.pressed == Some(Hit::Key(k));
            g.rrect(r.x, r.y + 1.5 * s, r.w, r.h, 8.0 * s, rgba(0x000000, 0.14));
            g.rrect(r.x, r.y, r.w, r.h, 8.0 * s, if pressed { t.press } else { t.surface });
            let label = match KEYS[k] {
                '.' => ",".to_string(),
                '\x08' => "←".to_string(),
                c => c.to_string(),
            };
            let st = Text::new(17.0 * s).bold().color(t.text).center();
            let (cx, cy) = r.center();
            ctx.text_mid(g, &label, cx, cy, &st);
        }
    }
}

impl App for Bench {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        g.clear(ctx.theme.bg);
        ui::header(ctx, g, "Bench", None);
        self.draw_tabs(ctx, g);
        if self.tab == 0 {
            self.render_bands(ctx, g);
        } else {
            self.render_fields(ctx, g);
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        match ev {
            Event::Down { x, y } => self.pressed = self.hit(ctx, x, y),
            Event::Up { .. } => self.pressed = None,
            Event::Tap { x, y } => {
                if let Some(h) = self.hit(ctx, x, y) {
                    self.act(ctx, h);
                }
            }
            Event::Key(k) => match k {
                Key::Char(c) if c.is_ascii_digit() => self.type_char(c),
                Key::Char(',') | Key::Char('.') => self.type_char('.'),
                Key::Back => self.type_char('\x08'),
                Key::Enter | Key::Down if self.tab > 0 => self.next_field(1),
                Key::Up if self.tab > 0 => self.next_field(-1),
                Key::Left | Key::Right => {
                    if self.tab == 0 {
                        let d: i32 = if matches!(k, Key::Right) { 1 } else { -1 };
                        self.band = (self.band as i32 + d).rem_euclid(self.nb as i32) as usize;
                    } else {
                        let d: i32 = if matches!(k, Key::Right) { 1 } else { 3 };
                        self.tab = ((self.tab as i32 + d) % 4) as usize;
                        self.fresh = true;
                    }
                    self.save();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn icon(&mut self, ctx: &Ctx, g: &mut Gfx, size: u32) {
        let _ = ctx;
        let f = size as f32;
        g.rrect(0.0, 0.0, f, f, f * 0.22, rgb(0x1F2937));
        g.rect(f * 0.08, f * 0.47, f * 0.84, f * 0.06, rgb(0xC0C4CC));
        g.rrect(f * 0.2, f * 0.32, f * 0.6, f * 0.36, f * 0.15, rgb(0xE6D3A1));
        for (x, c) in [(0.32, 0xF2C500), (0.44, 0x8B3FD9), (0.56, 0xD62828), (0.69, 0xC9A227)] {
            g.rect(f * x, f * 0.32, f * 0.06, f * 0.36, rgb(c));
        }
    }
}

fn main() {
    let mut app = Bench::new(Store::open("state.conf"));
    app.load();
    if let Some(t) = lapp_sdk::arg_value("--tab").and_then(|s| s.parse::<usize>().ok()) {
        app.tab = t.min(3);
    }
    if lapp_sdk::arg_flag("--fb") {
        app.tab = 1;
        app.fields[1][0].unit = 1;
        app.fields[1][2].text = "0.6".into();
    }
    run(Config::new("Bench"), app);
}
#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> Bench {
        Bench::new(Store::open_at("/tmp/bench-test-state.conf"))
    }

    fn dump(rows: &[Row]) -> Vec<String> {
        rows.iter().map(|r| format!("{} | {}", r.left, r.right)).collect()
    }

    #[test]
    fn series_sizes() {
        assert_eq!(E12.len(), 12);
        assert_eq!(E24.len(), 24);
        assert_eq!(E96.len(), 96);
        assert!(E96.windows(2).all(|w| w[0] < w[1]));
        assert!(E24.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn formats() {
        assert_eq!(si(4700.0, "Ω", 2), "4,7 kΩ");
        assert_eq!(si(0.47, "Ω", 2), "0,47 Ω");
        assert_eq!(si(100e-9, "F", 3), "100 nF");
        assert_eq!(si(1591.549, "Hz", 3), "1,59 kHz");
        assert_eq!(si(3.302, "V", 4), "3,302 V");
        assert_eq!(pct(0.0006), "+0,06 %");
    }

    #[test]
    fn bands_roundtrip() {
        let mut a = app();
        assert!((a.bands_value() - 4700.0).abs() < 1e-6);
        a.set_value(330.0);
        assert!((a.bands_value() - 330.0).abs() < 1e-6, "{}", a.bands_value());
        a.set_value(0.47);
        assert!((a.bands_value() - 0.47).abs() < 1e-9, "{}", a.bands_value());
        a.toggle_bands();
        assert_eq!(a.nb, 5);
        a.set_value(4990.0);
        assert!((a.bands_value() - 4990.0).abs() < 1e-6, "{}", a.bands_value());
        a.toggle_bands();
        assert_eq!(a.nb, 4);
        assert!((a.bands_value() - 5000.0).abs() < 1e-6, "{}", a.bands_value());
    }

    #[test]
    fn divider_fb_led_rc() {
        let mut a = app();
        let rows = a.rows_divider();
        println!("DIV: {:?}", dump(&rows));
        assert!(rows.len() == 3 && rows[0].kind == 1);
        a.fields[1][0].unit = 1;
        a.fields[1][2].text = "0.6".into();
        let rows = a.rows_divider();
        println!("FB: {:?}", dump(&rows));
        assert_eq!(rows.len(), 3);
        println!("LED: {:?}", dump(&a.rows_led()));
        println!("RC: {:?}", dump(&a.rows_rc()));
    }
}
