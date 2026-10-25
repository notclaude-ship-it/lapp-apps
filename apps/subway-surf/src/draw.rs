//! Rendu pseudo-3D : décor, rames, joueur, interface. Tout est dessiné avec des rectangles et des cercles
//! (les polygones sont remplis par bandes).
use crate::game::*;
use lapp_sdk::prelude::*;

const HAZE: u32 = 0xC9E6F4;

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let m = |x: f32, y: f32| (x + (y - x) * t).clamp(0.0, 1.0);
    Color::from_rgba(m(a.red(), b.red()), m(a.green(), b.green()), m(a.blue(), b.blue()), m(a.alpha(), b.alpha())).unwrap_or(a)
}

/// Brume lointaine.
fn fog(c: Color, z: f32) -> Color {
    let t = (z / FAR).clamp(0.0, 1.0).powf(1.5) * 0.92;
    mix(c, rgb(HAZE), t)
}

fn hsh(a: i64, b: i64) -> u32 {
    let mut x = (a as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (b as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    x ^= x >> 29;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 32;
    x as u32
}

/// Pinceau en points (l'échelle de l'écran est appliquée ici).
pub struct P<'a> {
    pub g: &'a mut Gfx,
    pub s: f32,
    pub ox: f32,
    pub oy: f32,
    pub cw: f32,
    pub ch: f32,
}

impl P<'_> {
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Color) {
        if w <= 0.0 || h <= 0.0 || x > self.cw + 8.0 || y > self.ch + 8.0 || x + w < -8.0 || y + h < -8.0 {
            return;
        }
        self.g.rect(x * self.s + self.ox, y * self.s + self.oy, w * self.s, h * self.s, c);
    }

    pub fn rrect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) {
        if w <= 0.0 || h <= 0.0 || x > self.cw + 8.0 || y > self.ch + 8.0 || x + w < -8.0 || y + h < -8.0 {
            return;
        }
        self.g.rrect(x * self.s + self.ox, y * self.s + self.oy, w * self.s, h * self.s, (r * self.s).min(w * self.s * 0.5).min(h * self.s * 0.5), c);
    }

    pub fn circle(&mut self, x: f32, y: f32, r: f32, c: Color) {
        if r <= 0.0 || x + r < -8.0 || x - r > self.cw + 8.0 || y + r < -8.0 || y - r > self.ch + 8.0 {
            return;
        }
        self.g.circle(x * self.s + self.ox, y * self.s + self.oy, r * self.s, c);
    }

    /// Remplit un polygone convexe par bandes (lignes ou colonnes, selon le moins coûteux).
    pub fn poly(&mut self, pts: &[(f32, f32)], c: Color) {
        let n = pts.len();
        if n < 3 {
            return;
        }
        let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for &(x, y) in pts {
            x0 = x0.min(x);
            x1 = x1.max(x);
            y0 = y0.min(y);
            y1 = y1.max(y);
        }
        x0 = x0.max(-4.0);
        x1 = x1.min(self.cw + 4.0);
        y0 = y0.max(-4.0);
        y1 = y1.min(self.ch + 4.0);
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let step = 1.6 / self.s;
        let span = |t: f32, swap: bool| -> Option<(f32, f32)> {
            let (mut lo, mut hi) = (f32::MAX, f32::MIN);
            for i in 0..n {
                let (p, q) = (pts[i], pts[(i + 1) % n]);
                let (a, b) = if swap { ((p.1, p.0), (q.1, q.0)) } else { (p, q) };
                if (a.1 <= t && b.1 > t) || (b.1 <= t && a.1 > t) {
                    let u = a.0 + (t - a.1) * (b.0 - a.0) / (b.1 - a.1);
                    lo = lo.min(u);
                    hi = hi.max(u);
                }
            }
            if hi > lo {
                Some((lo, hi))
            } else {
                None
            }
        };
        if (y1 - y0) <= (x1 - x0) {
            let mut y = y0;
            while y < y1 {
                if let Some((a, b)) = span(y + step * 0.5, false) {
                    let (a, b) = (a.max(-4.0), b.min(self.cw + 4.0));
                    self.rect(a, y, b - a, step + 0.45, c);
                }
                y += step;
            }
        } else {
            let mut x = x0;
            while x < x1 {
                if let Some((a, b)) = span(x + step * 0.5, true) {
                    let (a, b) = (a.max(-4.0), b.min(self.ch + 4.0));
                    self.rect(x, a, step + 0.45, b - a, c);
                }
                x += step;
            }
        }
    }

    /// Texte (coordonnées en points, y = milieu de ligne) avec ombre portée.
    pub fn txt(&mut self, ctx: &Ctx, s: &str, x: f32, y: f32, size: f32, col: Color, align: u8) {
        let mk = |c: Color| {
            let t = Text::new(size * self.s).bold().color(c);
            match align {
                0 => t.left(),
                1 => t.center(),
                _ => t.right(),
            }
        };
        let (px, py) = (x * self.s, y * self.s);
        let o = (size * 0.07).max(1.0) * self.s;
        ctx.text_mid(self.g, s, px + o, py + o, &mk(rgba(0x000000, 0.45 * col.alpha())));
        ctx.text_mid(self.g, s, px, py, &mk(col));
    }
}

/// Caméra : horizon, échelle (points par unité au niveau du joueur), hauteur de l'œil.
pub struct Cam {
    pub vw: f32,
    pub vh: f32,
    pub hy: f32,
    pub k: f32,
    pub hc: f32,
    pub cx: f32,
    pub camx: f32,
}

impl Cam {
    pub fn new(vw: f32, vh: f32, gm: &Game) -> Cam {
        let hy = vh * 0.30;
        let k = vw * 0.255;
        let hc0 = (vh * 0.80 - hy) / k;
        Cam { vw, vh, hy, k, hc: hc0 + gm.py * 0.35, cx: vw * 0.5, camx: gm.px * 0.55 }
    }

    pub fn sc(&self, z: f32) -> f32 {
        self.k * D / (D + z.max(-2.4))
    }

    /// Projection (x latéral, z profondeur devant le joueur, h hauteur) -> point écran.
    pub fn pt(&self, x: f32, z: f32, h: f32) -> (f32, f32) {
        let sc = self.sc(z);
        (self.cx + (x - self.camx) * sc, self.hy + (self.hc - h) * sc)
    }
}

const TRAINS: [(u32, u32); 4] = [(0xD9483B, 0xFFD23C), (0x2F7FD8, 0xF5F5F5), (0x2FA36B, 0xFFD23C), (0xE8A33D, 0x2B2B2B)];
const WALLS: [u32; 5] = [0x8E4F43, 0x5D6E80, 0x4D7A63, 0x7B5D8E, 0x9A7B4F];
const POSTERS: [u32; 4] = [0xFFD23C, 0xFF6FA5, 0x4FD1C5, 0xF5F5F5];

fn sky(p: &mut P, cam: &Cam, gm: &Game) {
    let top = rgb(0x3F8FD8);
    let hz = rgb(HAZE);
    let n = 14;
    let bh = cam.hy / n as f32;
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        p.rect(0.0, bh * i as f32, cam.vw, bh + 1.0, mix(top, hz, t.powf(1.4)));
    }
    p.circle(cam.vw * 0.74, cam.hy * 0.52, 30.0, rgba(0xFFF4C0, 0.35));
    p.circle(cam.vw * 0.74, cam.hy * 0.52, 17.0, rgb(0xFFF6D0));
    for i in 0..4 {
        let span = cam.vw + 160.0;
        let x = cam.vw + 70.0 - ((gm.t * (5.0 + i as f32 * 2.0) + i as f32 * 97.0) % span);
        let y = cam.hy * (0.18 + 0.17 * i as f32);
        p.rrect(x, y, 56.0, 12.0, 6.0, rgba(0xFFFFFF, 0.55));
        p.rrect(x + 12.0, y - 7.0, 30.0, 12.0, 6.0, rgba(0xFFFFFF, 0.55));
    }
    // horizon : immeubles lointains
    for layer in 0..2 {
        let (w0, base, col) = if layer == 0 { (26.0, 0x9DBDD8, 0.0) } else { (20.0, 0x86A8C8, 7.0) };
        let par = cam.camx * (6.0 + layer as f32 * 4.0);
        for i in -1..(cam.vw / w0) as i32 + 3 {
            let h = hsh(i as i64, layer as i64);
            let bh = 18.0 + (h % 40) as f32 + layer as f32 * 8.0;
            let bw = w0 - 2.0 + ((h >> 8) % 8) as f32;
            let x = i as f32 * w0 - par - col;
            p.rect(x, cam.hy - bh, bw, bh + 2.0, mix(rgb(base), rgb(HAZE), 0.25 + 0.2 * layer as f32));
        }
    }
}

fn ground(p: &mut P, cam: &Cam) {
    let gravel = rgb(0x6C717B);
    p.rect(0.0, cam.hy, cam.vw, cam.vh - cam.hy + 2.0, gravel);
    for i in 0..7 {
        p.rect(0.0, cam.hy + i as f32 * 3.0, cam.vw, 3.2, mix(rgb(HAZE), gravel, i as f32 / 7.0));
    }
}

fn walls(p: &mut P, cam: &Cam, gm: &Game) {
    let seg = 4.0;
    let k0 = ((gm.dist - 2.4) / seg).floor() as i64;
    for side in [-1.0f32, 1.0] {
        let wx = side * 2.5;
        for j in 0..((FAR + 6.0) / seg) as i64 + 2 {
            let id = k0 + j;
            let z0 = id as f32 * seg - gm.dist;
            let z1 = z0 + seg;
            if z1 < -2.4 || z0 > FAR {
                continue;
            }
            let z0 = z0.max(-2.4);
            let h = hsh(id, side as i64 + 5);
            let col = rgb(WALLS[(h as usize / 7) % 5]);
            let fz = z0.max(0.0);
            let q = |a: f32, b: f32, h0: f32, h1: f32| -> [(f32, f32); 4] { [cam.pt(wx, a, h1), cam.pt(wx, b, h1), cam.pt(wx, b, h0), cam.pt(wx, a, h0)] };
            p.poly(&q(z0, z1, 0.0, 4.6), fog(col, fz));
            p.poly(&q(z0, z1, 0.0, 0.5), fog(shade(col, 0.6), fz));
            p.poly(&q(z0, z1, 3.7, 4.0), fog(shade(col, 1.25), fz));
            if h & 3 == 0 {
                let (a, b) = ((z0 + 0.7).max(z0), z1 - 0.7);
                if b > a {
                    p.poly(&q(a, b, 1.1, 2.7), fog(rgb(POSTERS[(h as usize >> 4) % 4]), fz));
                    p.poly(&q(a + 0.3, b - 0.3, 1.5, 2.3), fog(shade(rgb(POSTERS[(h as usize >> 4) % 4]), 0.7), fz));
                }
            }
            // pilier
            p.poly(&q(z0, (z0 + 0.35).min(z1), 0.0, 4.6), fog(shade(col, 0.7), fz));
        }
    }
}

fn track(p: &mut P, cam: &Cam, gm: &Game) {
    let step = 2.4;
    let sleeper = rgb(0x8A7458);
    let ballast = rgb(0x4A4E57);
    let steel = rgb(0xDADFE6);
    let kerb = rgb(0xE6B83A);
    let mut y = cam.hy + 1.0;
    while y < cam.vh {
        let den = y + step * 0.5 - cam.hy;
        let sc = den / cam.hc;
        let z = cam.k * D / sc - D;
        if z > FAR + 30.0 {
            y += step;
            continue;
        }
        let fz = z.max(0.0);
        let wz = gm.dist + z;
        let col = if (wz / 1.3).fract() < 0.4 { sleeper } else { ballast };
        let cxr = cam.cx - cam.camx * sc;
        let h = step + 0.5;
        p.rect(cxr - 1.55 * sc, y, 3.1 * sc, h, fog(col, fz));
        let kw = (0.08 * sc).max(0.8);
        p.rect(cxr - 1.55 * sc - kw, y, kw, h, fog(kerb, fz));
        p.rect(cxr + 1.55 * sc, y, kw, h, fog(kerb, fz));
        let w = (0.085 * sc).max(0.7);
        let rc = fog(steel, fz);
        for l in [-1.0f32, 0.0, 1.0] {
            for r in [-0.3f32, 0.3] {
                p.rect(cxr + (l + r) * sc - w * 0.5, y, w, h, rc);
            }
        }
        y += step;
    }
}

/// Parallélépipède : face avant, dessus et face latérale visible.
fn boxed(p: &mut P, cam: &Cam, xa: f32, xb: f32, zf: f32, zb: f32, h0: f32, h1: f32, col: Color, fz: f32) {
    let zf = zf.max(-2.4);
    let zb = zb.min(FAR + 8.0);
    if zb <= zf {
        return;
    }
    if cam.camx > xb {
        p.poly(&[cam.pt(xb, zf, h1), cam.pt(xb, zb, h1), cam.pt(xb, zb, h0), cam.pt(xb, zf, h0)], shade(col, 0.78));
    }
    if cam.camx < xa {
        p.poly(&[cam.pt(xa, zf, h1), cam.pt(xa, zb, h1), cam.pt(xa, zb, h0), cam.pt(xa, zf, h0)], shade(col, 0.78));
    }
    if h1 < cam.hc {
        p.poly(&[cam.pt(xa, zf, h1), cam.pt(xb, zf, h1), cam.pt(xb, zb, h1), cam.pt(xa, zb, h1)], shade(col, 1.18));
    }
    let _ = fz;
    let (a, b) = (cam.pt(xa, zf, h1), cam.pt(xb, zf, h0));
    p.rect(a.0, a.1, b.0 - a.0, b.1 - a.1, col);
}

fn train(p: &mut P, cam: &Cam, gm: &Game, o: &Obj) {
    let (body, acc) = TRAINS[o.color % 4];
    let rel = o.z - gm.dist;
    let fz = rel.max(0.0);
    let (xa, xb) = (o.x - 0.42, o.x + 0.42);
    let zf = rel.max(-2.4);
    let zb = (rel + o.len).min(FAR + 8.0);
    let bc = fog(rgb(body), fz);
    // côtés : fenêtres et bande de couleur
    for (x, vis) in [(xb, cam.camx > xb), (xa, cam.camx < xa)] {
        if !vis {
            continue;
        }
        let q = |a: f32, b: f32, h0: f32, h1: f32| [cam.pt(x, a, h1), cam.pt(x, b, h1), cam.pt(x, b, h0), cam.pt(x, a, h0)];
        p.poly(&q(zf, zb, 0.25, 2.0), shade(bc, 0.78));
        if zb - 0.6 > zf + 0.6 {
            p.poly(&q(zf + 0.6, zb - 0.6, 1.15, 1.7), fog(rgb(0x1F2F4A), fz));
            p.poly(&q(zf + 0.1, zb - 0.1, 0.5, 0.7), fog(rgb(acc), fz));
        }
    }
    p.poly(&[cam.pt(xa, zf, 2.0), cam.pt(xb, zf, 2.0), cam.pt(xb, zb, 2.0), cam.pt(xa, zb, 2.0)], shade(bc, 1.15));
    if rel > -2.3 {
        let sc = cam.sc(zf);
        let tl = cam.pt(xa, zf, 2.0);
        let br = cam.pt(xb, zf, 0.25);
        p.rect(tl.0, tl.1, br.0 - tl.0, br.1 - tl.1, bc);
        let sk = cam.pt(xb, zf, 0.0);
        p.rect(tl.0, br.1, br.0 - tl.0, sk.1 - br.1, fog(rgb(0x2B2E36), fz));
        let w0 = cam.pt(xa + 0.07, zf, 1.78);
        let w1 = cam.pt(xb - 0.07, zf, 1.1);
        p.rrect(w0.0, w0.1, w1.0 - w0.0, w1.1 - w0.1, 0.05 * sc, fog(rgb(0x1F2F4A), fz));
        p.rect(w0.0 + 0.05 * sc, w0.1 + 0.05 * sc, (w1.0 - w0.0) * 0.4, 0.07 * sc, fog(rgba(0xFFFFFF, 0.35), fz));
        let s0 = cam.pt(xa, zf, 1.0);
        p.rect(s0.0, s0.1, br.0 - tl.0, 0.14 * sc, fog(rgb(acc), fz));
        for x in [xa + 0.15, xb - 0.15] {
            let c = cam.pt(x, zf, 0.58);
            p.circle(c.0, c.1, 0.075 * sc, fog(rgb(0xFFF6C0), fz));
        }
    }
}

fn low_barrier(p: &mut P, cam: &Cam, gm: &Game, o: &Obj) {
    let rel = o.z - gm.dist;
    let fz = rel.max(0.0);
    for dx in [-0.40f32, 0.34] {
        boxed(p, cam, o.x + dx, o.x + dx + 0.07, rel + 0.05, rel + 0.2, 0.0, 0.4, fog(rgb(0x8A8F99), fz), fz);
    }
    boxed(p, cam, o.x - 0.45, o.x + 0.45, rel, rel + 0.3, 0.35, 0.85, fog(rgb(0xF2F2F2), fz), fz);
    if rel > -2.3 {
        let sc = cam.sc(rel);
        for i in 0..6 {
            if i % 2 == 0 {
                let a = cam.pt(o.x - 0.45 + i as f32 * 0.15, rel, 0.85);
                p.rect(a.0, a.1, 0.15 * sc, 0.5 * sc, fog(rgb(0xE23B3B), fz));
            }
        }
    }
}

fn over_bar(p: &mut P, cam: &Cam, gm: &Game, o: &Obj) {
    let rel = o.z - gm.dist;
    let fz = rel.max(0.0);
    for dx in [-0.5f32, 0.4] {
        boxed(p, cam, o.x + dx, o.x + dx + 0.1, rel, rel + 0.15, 0.0, 1.7, fog(rgb(0x7A808C), fz), fz);
    }
    boxed(p, cam, o.x - 0.5, o.x + 0.5, rel, rel + 0.2, 0.95, 1.45, fog(rgb(0xFFC928), fz), fz);
    if rel > -2.3 {
        let sc = cam.sc(rel);
        for i in 0..8 {
            if i % 2 == 1 {
                let a = cam.pt(o.x - 0.5 + i as f32 * 0.125, rel, 1.45);
                p.rect(a.0, a.1, 0.125 * sc, 0.5 * sc, fog(rgb(0x23262E), fz));
            }
        }
    }
}

fn coin(p: &mut P, cam: &Cam, gm: &Game, o: &Obj) {
    let rel = o.z - gm.dist;
    let fz = rel.max(0.0);
    let (cx, cy) = cam.pt(o.x, rel, o.h);
    let sc = cam.sc(rel);
    let r = 0.2 * sc;
    let w = r * (0.3 + 0.7 * (gm.t * 6.0 + o.z * 0.35).cos().abs());
    p.rrect(cx - w, cy - r, w * 2.0, r * 2.0, w, fog(rgb(0xF2A900), fz));
    p.rrect(cx - w * 0.6, cy - r * 0.68, w * 1.2, r * 1.36, w * 0.6, fog(rgb(0xFFDE59), fz));
}

fn pickup(p: &mut P, cam: &Cam, gm: &Game, o: &Obj) {
    let rel = o.z - gm.dist;
    let fz = rel.max(0.0);
    let bob = (gm.t * 4.0 + o.z).sin() * 0.08;
    let (cx, cy) = cam.pt(o.x, rel, o.h + bob);
    let u = cam.sc(rel);
    p.circle(cx, cy, 0.5 * u, fog(rgba(0x6FE3FF, 0.3), fz));
    if o.kind == Kind::Jet {
        p.rrect(cx - 0.17 * u, cy - 0.28 * u, 0.34 * u, 0.5 * u, 0.12 * u, fog(rgb(0xE23B3B), fz));
        p.rrect(cx - 0.12 * u, cy - 0.22 * u, 0.24 * u, 0.1 * u, 0.04 * u, fog(rgb(0xF5F5F5), fz));
        p.circle(cx, cy + 0.3 * u, 0.1 * u, fog(rgb(0xFFB020), fz));
        p.circle(cx, cy + 0.3 * u, 0.05 * u, fog(rgb(0xFFF0A0), fz));
    } else {
        let red = fog(rgb(0xD92B2B), fz);
        p.rrect(cx - 0.26 * u, cy - 0.26 * u, 0.14 * u, 0.4 * u, 0.05 * u, red);
        p.rrect(cx + 0.12 * u, cy - 0.26 * u, 0.14 * u, 0.4 * u, 0.05 * u, red);
        p.rrect(cx - 0.26 * u, cy + 0.0 * u, 0.52 * u, 0.2 * u, 0.1 * u, red);
        p.rect(cx - 0.26 * u, cy - 0.26 * u, 0.14 * u, 0.1 * u, fog(rgb(0xF5F5F5), fz));
        p.rect(cx + 0.12 * u, cy - 0.26 * u, 0.14 * u, 0.1 * u, fog(rgb(0xF5F5F5), fz));
    }
}

/// Le coureur, vu de dos. `(bx, by)` = pieds, `u` = points par unité.
fn runner(p: &mut P, gm: &Game, bx: f32, by: f32, u: f32, a: f32, dark: bool) {
    let c = |h: u32| {
        let col = rgb(h);
        with_alpha(if dark { shade(col, 0.28) } else { col }, a)
    };
    let jet = gm.jet_t > 0.0;
    let air = gm.airborne();
    let rolling = gm.roll_t > 0.0 && !jet;
    let skin = c(0xF0BE96);
    let jacket = c(0x2D6BE8);
    let cap = c(0xE63946);
    if rolling {
        let cy = by - 0.3 * u;
        p.rrect(bx - 0.42 * u, cy - 0.3 * u, 0.84 * u, 0.6 * u, 0.3 * u, jacket);
        p.rrect(bx - 0.3 * u, cy - 0.34 * u, 0.6 * u, 0.2 * u, 0.1 * u, c(0xF2A33A));
        p.circle(bx, cy - 0.28 * u, 0.17 * u, cap);
        let ang = gm.run * 1.6;
        p.circle(bx + ang.cos() * 0.3 * u, cy + ang.sin() * 0.2 * u, 0.06 * u, skin);
        return;
    }
    let ph = gm.run;
    // jambes
    for (i, side) in [-1.0f32, 1.0].iter().enumerate() {
        let s = (ph + if i == 0 { 0.0 } else { std::f32::consts::PI }).sin();
        let lift = if jet { 0.06 } else if air { 0.22 + 0.06 * side } else { s.max(0.0) * 0.26 };
        let x = bx + side * 0.14 * u - 0.08 * u;
        let top = by - 0.72 * u;
        let bot = by - lift * u;
        p.rrect(x, top, 0.16 * u, bot - top, 0.07 * u, c(0x2B2E4A));
        p.rrect(x - 0.03 * u, bot - 0.1 * u, 0.22 * u, 0.12 * u, 0.05 * u, c(0xF4F4F4));
    }
    // bras
    for (i, side) in [-1.0f32, 1.0].iter().enumerate() {
        let s = (ph + if i == 0 { std::f32::consts::PI } else { 0.0 }).sin();
        let off = if air || jet { -0.16 } else { s * 0.13 };
        let x = bx + side * 0.32 * u - 0.065 * u;
        p.rrect(x, by - 1.12 * u + off * u, 0.13 * u, 0.44 * u, 0.065 * u, jacket);
        p.circle(x + 0.065 * u, by - 0.68 * u + off * u, 0.065 * u, skin);
    }
    // buste, sac
    p.rrect(bx - 0.25 * u, by - 1.2 * u, 0.5 * u, 0.56 * u, 0.14 * u, jacket);
    p.rect(bx - 0.25 * u, by - 0.95 * u, 0.5 * u, 0.06 * u, with_alpha(c(0xFFFFFF), a * 0.45));
    if jet {
        for sgn in [-1.0f32, 1.0] {
            p.rrect(bx + sgn * 0.2 * u - 0.085 * u, by - 1.1 * u, 0.17 * u, 0.5 * u, 0.08 * u, c(0xC9CED6));
            p.rrect(bx + sgn * 0.2 * u - 0.085 * u, by - 0.78 * u, 0.17 * u, 0.1 * u, 0.04 * u, c(0xE23B3B));
            let fl = 0.14 + 0.05 * (gm.t * 45.0 + sgn).sin();
            p.circle(bx + sgn * 0.2 * u, by - 0.5 * u, fl * u, with_alpha(c(0xFF8A1E), a * 0.9));
            p.circle(bx + sgn * 0.2 * u, by - 0.38 * u, fl * 0.8 * u, with_alpha(c(0xFFD23C), a * 0.85));
            p.circle(bx + sgn * 0.2 * u, by - 0.27 * u, fl * 0.5 * u, with_alpha(c(0xFFF4B0), a * 0.8));
        }
    } else {
        p.rrect(bx - 0.19 * u, by - 1.12 * u, 0.38 * u, 0.42 * u, 0.08 * u, c(0xF2A33A));
        p.rect(bx - 0.04 * u, by - 1.12 * u, 0.08 * u, 0.42 * u, with_alpha(c(0xC57E1E), a));
    }
    // tête : cheveux + casquette
    p.rrect(bx - 0.17 * u, by - 1.42 * u, 0.34 * u, 0.26 * u, 0.12 * u, c(0x5B3A24));
    p.circle(bx, by - 1.42 * u, 0.2 * u, cap);
    p.rect(bx - 0.04 * u, by - 1.6 * u, 0.08 * u, 0.1 * u, with_alpha(c(0xFFFFFF), a * 0.5));
}

fn guard(p: &mut P, cam: &Cam, gm: &Game) {
    if gm.guard_t <= 0.0 || gm.st == St::Title {
        return;
    }
    let age = 5.0 - gm.guard_t;
    let enter = (age * 4.0).clamp(0.0, 1.0);
    let leave = gm.guard_t.min(0.8) / 0.8;
    let z = -1.5 - (1.0 - enter) * 4.0;
    let a = (leave * enter).clamp(0.0, 1.0);
    let (gx, gy) = cam.pt(gm.px * 0.6 + 0.25, z, 0.0);
    let u = cam.sc(z);
    let bob = (gm.run * 1.0).sin().abs() * 0.06 * u;
    let k = |h: u32| with_alpha(rgb(h), a);
    for side in [-1.0f32, 1.0] {
        p.rrect(gx + side * 0.13 * u - 0.09 * u, gy - 0.75 * u - bob, 0.18 * u, 0.75 * u, 0.07 * u, k(0x1B2038));
    }
    p.rrect(gx - 0.3 * u, gy - 1.3 * u - bob, 0.6 * u, 0.65 * u, 0.15 * u, k(0x24407A));
    p.circle(gx, gy - 1.5 * u - bob, 0.2 * u, k(0xE8B890));
    p.rrect(gx - 0.23 * u, gy - 1.78 * u - bob, 0.46 * u, 0.2 * u, 0.08 * u, k(0x1B2A5A));
    p.rect(gx - 0.28 * u, gy - 1.62 * u - bob, 0.56 * u, 0.07 * u, k(0x1B2A5A));
    // chien
    let (dx, dy) = cam.pt(gm.px * 0.6 - 0.55, z + 0.3, 0.0);
    let du = cam.sc(z + 0.3);
    p.rrect(dx - 0.25 * du, dy - 0.5 * du - bob, 0.5 * du, 0.32 * du, 0.14 * du, k(0x8A5A32));
    p.circle(dx, dy - 0.58 * du - bob, 0.14 * du, k(0x8A5A32));
    p.rect(dx - 0.2 * du, dy - 0.2 * du, 0.08 * du, 0.2 * du, k(0x6E4524));
    p.rect(dx + 0.12 * du, dy - 0.2 * du, 0.08 * du, 0.2 * du, k(0x6E4524));
}

fn player(p: &mut P, cam: &Cam, gm: &Game) {
    let (bx, by) = cam.pt(gm.px, 0.0, gm.py);
    let u = cam.k;
    // ombre
    let (sx, sy) = cam.pt(gm.px, 0.0, 0.0);
    let w = 0.62 * u * (1.0 - gm.py.min(3.5) * 0.1);
    p.rrect(sx - w * 0.5, sy - 0.06 * u, w, 0.12 * u, 0.06 * u, rgba(0x000000, 0.28));
    let blink = gm.st == St::Play && gm.inv_t > 0.0 && gm.jet_t <= 0.0 && ((gm.t * 20.0) as i32) % 2 == 0;
    runner(p, gm, bx, by, u, if blink { 0.4 } else { 1.0 }, gm.st == St::Over && gm.caught);
}

fn particles(p: &mut P, cam: &Cam, gm: &Game) {
    for q in &gm.parts {
        let rel = q.z - gm.dist;
        let (x, y) = cam.pt(q.x, rel, q.y);
        let a = (q.life / q.max).clamp(0.0, 1.0);
        p.circle(x, y, q.size * cam.sc(rel) * (0.5 + 0.5 * a), with_alpha(rgb(q.col), a));
    }
}

fn draw_obj(p: &mut P, cam: &Cam, gm: &Game, o: &Obj) {
    match o.kind {
        Kind::Train => train(p, cam, gm, o),
        Kind::Low => low_barrier(p, cam, gm, o),
        Kind::Over => over_bar(p, cam, gm, o),
        Kind::Coin => coin(p, cam, gm, o),
        Kind::Jet | Kind::Magnet => pickup(p, cam, gm, o),
    }
}

/// Toute la scène 3D (hors interface).
pub fn scene(p: &mut P, cam: &Cam, gm: &Game) {
    sky(p, cam, gm);
    ground(p, cam);
    walls(p, cam, gm);
    track(p, cam, gm);
    let mut order: Vec<&Obj> = gm.objs.iter().filter(|o| !o.taken && o.z - gm.dist < FAR && o.z - gm.dist + o.len > -2.4).collect();
    order.sort_by(|a, b| b.z.partial_cmp(&a.z).unwrap_or(std::cmp::Ordering::Equal));
    let mut i = 0;
    while i < order.len() && order[i].z - gm.dist > 0.0 {
        draw_obj(p, cam, gm, order[i]);
        i += 1;
    }
    guard(p, cam, gm);
    player(p, cam, gm);
    while i < order.len() {
        draw_obj(p, cam, gm, order[i]);
        i += 1;
    }
    particles(p, cam, gm);
}

pub fn hud(ctx: &Ctx, p: &mut P, vw: f32, gm: &Game, best_shown: u32) {
    let white = rgb(0xFFFFFF);
    p.txt(ctx, &format!("{}", gm.score()), 12.0, 22.0, 24.0, white, 0);
    p.txt(ctx, &format!("Record {}", best_shown.max(gm.best)), 12.0, 42.0, 10.0, rgba(0xFFFFFF, 0.85), 0);
    p.circle(vw - 16.0, 22.0, 8.0, rgb(0xF2A900));
    p.circle(vw - 16.0, 22.0, 5.0, rgb(0xFFDE59));
    p.txt(ctx, &format!("{}", gm.coins), vw - 29.0, 22.0, 18.0, white, 2);
    let mut y = 62.0;
    for (t, tot, name, col) in [(gm.jet_t, 7.5, "JETPACK", 0xFF8A1E), (gm.mag_t, 10.0, "AIMANT", 0xE23B3B)] {
        if t > 0.0 {
            p.rrect(12.0, y, 84.0, 9.0, 4.5, rgba(0x000000, 0.4));
            p.rrect(12.0, y, (84.0 * t / tot).max(5.0), 9.0, 4.5, rgb(col));
            p.txt(ctx, name, 102.0, y + 4.5, 9.0, white, 0);
            y += 15.0;
        }
    }
}

pub fn overlay(ctx: &Ctx, p: &mut P, vw: f32, vh: f32, gm: &Game) {
    let white = rgb(0xFFFFFF);
    let cx = vw * 0.5;
    if gm.flash > 0.0 {
        p.rect(0.0, 0.0, vw, vh, rgba(if gm.st == St::Over { 0xFF3030 } else { 0xFFFFFF }, (gm.flash * 0.6).min(0.4)));
    }
    match gm.st {
        St::Title => {
            p.rect(0.0, 0.0, vw, vh, rgba(0x0A1A33, 0.25));
            let bob = (gm.t * 2.0).sin() * 3.0;
            p.txt(ctx, "SUBWAY", cx, vh * 0.17 + bob, 46.0, rgb(0xFFD23C), 1);
            p.txt(ctx, "SURF", cx, vh * 0.17 + 48.0 + bob, 58.0, rgb(0xFF5A3C), 1);
            if gm.best > 0 {
                p.txt(ctx, &format!("Record : {}", gm.best), cx, vh * 0.53, 14.0, white, 1);
            }
            let pulse = 0.55 + 0.45 * (gm.t * 3.0).sin();
            p.txt(ctx, "Touche pour jouer", cx, vh * 0.60, 16.0, with_alpha(white, pulse), 1);
            p.txt(ctx, "← → changer de voie", cx, vh - 56.0, 11.0, rgba(0xFFFFFF, 0.9), 1);
            p.txt(ctx, "↑ sauter    ↓ rouler", cx, vh - 40.0, 11.0, rgba(0xFFFFFF, 0.9), 1);
        }
        St::Over => {
            let a = (gm.st_t * 3.0).clamp(0.0, 1.0);
            p.rect(0.0, 0.0, vw, vh, rgba(0x05030F, 0.55 * a));
            let (pw, ph) = (vw - 36.0, 188.0);
            let py = vh * 0.2 + (1.0 - a) * 24.0;
            p.rrect(18.0, py, pw, ph, 20.0, rgba(0x1B2342, 0.95 * a));
            p.rrect(18.0, py, pw, 5.0, 2.5, with_alpha(rgb(0xFF5A5A), a));
            let title = if gm.caught { "ATTRAPÉ !" } else { "BOUM !" };
            p.txt(ctx, title, cx, py + 36.0, 30.0, with_alpha(rgb(0xFF7A7A), a), 1);
            p.txt(ctx, &format!("{}", gm.score()), cx, py + 78.0, 34.0, with_alpha(white, a), 1);
            p.txt(ctx, &format!("{} m  ·  {} pièces", gm.dist as u32, gm.coins), cx, py + 106.0, 12.0, with_alpha(white, 0.85 * a), 1);
            if gm.new_best {
                p.txt(ctx, "NOUVEAU RECORD !", cx, py + 130.0, 14.0, with_alpha(rgb(0xFFD23C), a), 1);
            } else {
                p.txt(ctx, &format!("Record : {}", gm.best), cx, py + 130.0, 13.0, with_alpha(white, 0.8 * a), 1);
            }
            let pulse = 0.55 + 0.45 * (gm.t * 3.0).sin();
            p.txt(ctx, "Touche pour rejouer", cx, py + 164.0, 15.0, with_alpha(white, pulse * a), 1);
        }
        St::Play => {}
    }
}

pub fn icon(g: &mut Gfx, size: u32) {
    let f = size as f32;
    let mut p = P { g, s: 1.0, ox: 0.0, oy: 0.0, cw: f, ch: f };
    p.rrect(0.0, 0.0, f, f, f * 0.22, rgb(0x4FA3E8));
    p.rect(0.0, f * 0.46, f, f * 0.54, rgb(0x4A4E57));
    p.rrect(0.0, 0.0, f, f * 0.5, f * 0.22, rgb(0x4FA3E8));
    p.rect(0.0, f * 0.3, f, f * 0.2, rgb(0x4FA3E8));
    // rails qui fuient
    for k in [-1.0f32, 1.0] {
        p.poly(&[(f * 0.5 + k * 0.02 * f, f * 0.46), (f * 0.5 + k * 0.05 * f, f * 0.46), (f * 0.5 + k * 0.44 * f, f), (f * 0.5 + k * 0.34 * f, f)], rgb(0xDADFE6));
    }
    for i in 0..5 {
        let t = (i as f32 + 1.0) / 6.0;
        let y = f * (0.46 + 0.54 * t * t);
        let w = f * (0.06 + 0.4 * t);
        p.rect(f * 0.5 - w, y, w * 2.0, f * 0.02 + f * 0.03 * t, rgb(0x8A7458));
    }
    // rame au loin
    p.rrect(f * 0.36, f * 0.3, f * 0.28, f * 0.2, f * 0.03, rgb(0xD9483B));
    p.rrect(f * 0.39, f * 0.33, f * 0.22, f * 0.07, f * 0.02, rgb(0x1F2F4A));
    // coureur
    p.rrect(f * 0.42, f * 0.56, f * 0.16, f * 0.2, f * 0.04, rgb(0x2D6BE8));
    p.rrect(f * 0.455, f * 0.6, f * 0.09, f * 0.13, f * 0.02, rgb(0xF2A33A));
    p.circle(f * 0.5, f * 0.52, f * 0.065, rgb(0xE63946));
    p.rrect(f * 0.43, f * 0.76, f * 0.05, f * 0.1, f * 0.02, rgb(0x2B2E4A));
    p.rrect(f * 0.52, f * 0.76, f * 0.05, f * 0.1, f * 0.02, rgb(0x2B2E4A));
    // pièces
    for (cx, cy) in [(0.2, 0.7), (0.8, 0.66), (0.78, 0.82)] {
        p.circle(f * cx, f * cy, f * 0.05, rgb(0xF2A900));
        p.circle(f * cx, f * cy, f * 0.032, rgb(0xFFDE59));
    }
}