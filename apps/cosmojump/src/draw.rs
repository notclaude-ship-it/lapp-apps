//! Rendu : ciel à dégradé selon l'altitude, décor, plateformes, bonus, ennemis, joueur.
use crate::game::*;
use lapp_sdk::prelude::*;

const TOP: [u32; 3] = [0x3d85d8, 0x1d2166, 0x05040f];
const BOT: [u32; 3] = [0xa6dcf7, 0x7a45a8, 0x1b1244];

fn mixu(a: u32, b: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let ch = |sh: u32| {
        let x = ((a >> sh) & 255) as f32;
        let y = ((b >> sh) & 255) as f32;
        (x + (y - x) * t) as u32
    };
    (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

fn sky(alt: f32, tab: &[u32; 3]) -> u32 {
    let i = (alt.floor() as usize).min(1);
    mixu(tab[i], tab[i + 1], alt - i as f32)
}

pub struct V {
    pub s: f32,
    pub cam: f32,
    pub vh: f32,
    pub oy: f32,
}
impl V {
    pub fn x(&self, x: f32) -> f32 {
        x * self.s
    }
    pub fn y(&self, y: f32) -> f32 {
        (self.cam + self.vh - y) * self.s + self.oy
    }
}

fn hash(i: u32, k: u32) -> f32 {
    ((i.wrapping_mul(2654435761).wrapping_add(k.wrapping_mul(40503)) >> 8) % 1000) as f32 / 1000.0
}

fn background(gm: &Game, g: &mut Gfx, w: f32, h: f32, s: f32) {
    let alt = (gm.cam / 2200.0).clamp(0.0, 1.99);
    let n = 28;
    let (t, b) = (sky(alt, &TOP), sky(alt, &BOT));
    for i in 0..n {
        let f = i as f32 / (n - 1) as f32;
        let sh = h / n as f32;
        g.rect(0.0, i as f32 * sh - 1.0, w, sh + 2.0, rgb(mixu(t, b, f)));
    }
    let pa = (alt - 0.7).clamp(0.0, 1.0) * 0.9;
    if pa > 0.0 {
        let py = h * 0.25 + gm.cam * s * 0.04;
        g.circle(w * 0.78, py, 46.0 * s, rgba(0xE98A5B, pa));
        g.circle(w * 0.78 + 12.0 * s, py + 8.0 * s, 40.0 * s, rgba(0x8B3F5E, pa * 0.55));
    }
    let sa = (alt - 0.25).clamp(0.0, 1.0);
    if sa > 0.0 {
        for i in 0..70u32 {
            let par = 0.12 + (i % 3) as f32 * 0.07;
            let x = hash(i, 1) * w;
            let y = (hash(i, 2) * h + gm.cam * s * par).rem_euclid(h);
            let tw = 0.55 + 0.45 * (gm.t * 2.0 + i as f32).sin();
            g.circle(x, y, (0.8 + (i % 3) as f32 * 0.5) * s, rgba(0xFFFFFF, sa * tw));
        }
    }
    let ca = (1.0 - alt * 1.25).clamp(0.0, 0.85);
    if ca > 0.0 {
        for i in 0..7u32 {
            let span = h + 200.0 * s;
            let y = (hash(i, 3) * span + gm.cam * s * 0.35).rem_euclid(span) - 100.0 * s;
            let x = hash(i, 4) * w;
            let k = (0.7 + hash(i, 5) * 0.8) * s;
            let c = rgba(0xFFFFFF, ca);
            g.rrect(x - 36.0 * k, y, 72.0 * k, 16.0 * k, 8.0 * k, c);
            g.circle(x - 10.0 * k, y, 13.0 * k, c);
            g.circle(x + 10.0 * k, y - 4.0 * k, 16.0 * k, c);
        }
    }
}

fn plats(gm: &Game, g: &mut Gfx, v: &V) {
    let s = v.s;
    for p in &gm.plats {
        let (top, body) = match p.kind {
            Kind::Normal => (0x7BE07F, 0x3FAE52),
            Kind::Moving => (0x6AB7FF, 0x2F78D0),
            Kind::Crumble => (0xE0B07A, 0x9B6A3C),
        };
        let mut x = v.x(p.x);
        let y = v.y(p.y);
        let (w, hh) = (p.w * s, 11.0 * s);
        let mut a = 1.0;
        if p.crack >= 0.0 {
            x += (p.crack * 90.0).sin() * 2.0 * s;
            a = (1.0 - p.crack * 1.3).clamp(0.0, 1.0);
        }
        g.rrect(x, y, w, hh, 5.0 * s, rgba(body, a));
        g.rrect(x, y, w, hh * 0.58, 5.0 * s, rgba(top, a));
        match p.kind {
            Kind::Crumble => {
                g.rect(x + w * 0.3, y + 1.0 * s, 1.6 * s, hh * 0.8, rgba(0x5A3A1E, a));
                g.rect(x + w * 0.68, y + 1.0 * s, 1.6 * s, hh * 0.8, rgba(0x5A3A1E, a));
            }
            Kind::Moving => {
                g.circle(x + 6.0 * s, y + hh * 0.5, 1.8 * s, rgba(0xFFFFFF, 0.9));
                g.circle(x + w - 6.0 * s, y + hh * 0.5, 1.8 * s, rgba(0xFFFFFF, 0.9));
            }
            _ => {}
        }
        let cx = x + w / 2.0;
        if p.spring {
            let ext = 1.0 + (p.spring_t.max(0.0) / 0.35) * 0.9;
            let hgt = 9.0 * s * ext;
            g.rect(cx - 6.0 * s, y - hgt, 12.0 * s, hgt, rgb(0xC7CCD6));
            g.rect(cx - 6.0 * s, y - hgt * 0.65, 12.0 * s, 1.6 * s, rgb(0x7B8294));
            g.rect(cx - 6.0 * s, y - hgt * 0.3, 12.0 * s, 1.6 * s, rgb(0x7B8294));
            g.rrect(cx - 10.0 * s, y - hgt - 4.0 * s, 20.0 * s, 5.0 * s, 2.5 * s, rgb(0xFF4D5E));
        }
        if p.rocket {
            let bob = (gm.t * 4.0).sin() * 2.0 * s;
            let by = y - 30.0 * s + bob;
            g.rrect(cx - 11.0 * s, by + 12.0 * s, 6.0 * s, 10.0 * s, 2.0 * s, rgb(0xC83A3A));
            g.rrect(cx + 5.0 * s, by + 12.0 * s, 6.0 * s, 10.0 * s, 2.0 * s, rgb(0xC83A3A));
            g.rrect(cx - 6.0 * s, by, 12.0 * s, 24.0 * s, 6.0 * s, rgb(0xF2F2F7));
            g.rrect(cx - 6.0 * s, by, 12.0 * s, 8.0 * s, 4.0 * s, rgb(0xFF4D5E));
            g.circle(cx, by + 13.0 * s, 3.0 * s, rgb(0x6AB7FF));
            g.circle(cx, by + 27.0 * s, 3.5 * s, rgba(0xFFA23C, 0.9));
        }
    }
}

pub fn alien(g: &mut Gfx, cx: f32, feet: f32, s: f32, face: f32, vy: f32, sq: f32, rocket: bool, t: f32) {
    let st = (vy / 1600.0).clamp(-0.2, 0.25);
    let kx = 1.0 - st * 0.6 + sq * 0.22;
    let ky = 1.0 + st - sq * 0.22;
    let bw = 28.0 * s * kx;
    let bh = 26.0 * s * ky;
    let by = feet - 5.0 * s - bh;
    if rocket {
        g.rrect(cx - 11.0 * s, by + 4.0 * s, 22.0 * s, 18.0 * s, 5.0 * s, rgb(0xC83A3A));
        let f = 6.0 + (t * 40.0).sin() * 2.0;
        g.circle(cx, feet + 2.0 * s, f * s, rgba(0xFFA23C, 0.9));
        g.circle(cx, feet + 1.0 * s, f * 0.55 * s, rgba(0xFFF1A8, 0.95));
    }
    g.rrect(cx - 9.0 * s, feet - 8.0 * s, 6.0 * s, 8.0 * s, 3.0 * s, rgb(0x2FA874));
    g.rrect(cx + 3.0 * s, feet - 8.0 * s, 6.0 * s, 8.0 * s, 3.0 * s, rgb(0x2FA874));
    g.rect(cx - 1.0 * s + face * 3.0 * s, by - 9.0 * s, 2.0 * s, 11.0 * s, rgb(0x2FA874));
    g.circle(cx + face * 3.0 * s, by - 10.0 * s, 3.4 * s, rgb(0xFF5A8A));
    g.rrect(cx - bw / 2.0, by, bw, bh, bw.min(bh) / 2.2, rgb(0x7CFFB2));
    g.rrect(cx - bw / 2.0 + 3.0 * s, by + 2.0 * s, bw * 0.45, bh * 0.28, 4.0 * s, rgba(0xFFFFFF, 0.28));
    let ex = face * 2.5 * s;
    for k in [-1.0f32, 1.0] {
        let x = cx + k * 6.0 * s + ex;
        g.circle(x, by + bh * 0.38, 5.2 * s, rgb(0xFFFFFF));
        g.circle(x + face * 1.6 * s, by + bh * 0.38 + (vy / 1500.0).clamp(-1.0, 1.0) * -1.2 * s, 2.5 * s, rgb(0x16213E));
    }
    let mh = (2.5 + sq * 1.5) * s;
    g.rrect(cx - 4.0 * s + ex, by + bh * 0.68, 8.0 * s, mh, mh / 2.0, rgb(0x1E6B4A));
}

fn ufo(g: &mut Gfx, x: f32, y: f32, s: f32, t: f32) {
    g.circle(x, y - 5.0 * s, 10.0 * s, rgba(0x9FEFFF, 0.9));
    g.circle(x + 2.0 * s, y - 5.0 * s, 3.2 * s, rgb(0x6B1E3A));
    g.circle(x - 4.0 * s, y - 5.0 * s, 2.4 * s, rgb(0x6B1E3A));
    g.rrect(x - 19.0 * s, y - 3.0 * s, 38.0 * s, 12.0 * s, 6.0 * s, rgb(0x9AA5C4));
    g.rrect(x - 19.0 * s, y - 3.0 * s, 38.0 * s, 5.0 * s, 3.0 * s, rgb(0xC9D2EE));
    for i in 0..3 {
        let on = ((t * 4.0) as i32 + i) % 3 == 0;
        let c = if on { rgb(0xFFE14A) } else { rgb(0xE24A4A) };
        g.circle(x + (i as f32 - 1.0) * 10.0 * s, y + 4.0 * s, 2.0 * s, c);
    }
}

pub fn world(gm: &Game, ctx: &Ctx, g: &mut Gfx) {
    let s = ctx.scale();
    let (w, h) = (ctx.width(), ctx.height());
    background(gm, g, w, h, s);
    let oy = (gm.t * 70.0).sin() * 3.0 * s * gm.shake;
    let v = V { s, cam: gm.cam, vh: ctx.height() / s, oy };
    plats(gm, g, &v);
    for c in &gm.coins {
        let (cx, cy) = (v.x(c.x), v.y(c.y));
        let r = 8.0 * s;
        let k = c.ph.cos().abs() * 0.8 + 0.2;
        g.rrect(cx - r * k, cy - r, 2.0 * r * k, 2.0 * r, r * k, rgb(0xE0A21C));
        g.rrect(cx - r * k * 0.7, cy - r * 0.8, 1.4 * r * k, 1.6 * r, r * k * 0.7, rgb(0xFFD23C));
    }
    for e in &gm.enemies {
        ufo(g, v.x(e.x), v.y(e.y + (gm.t * 3.0 + e.ph).sin() * 4.0), s, gm.t);
    }
    alien(g, v.x(gm.px), v.y(gm.py), s, gm.face, gm.pvy, gm.squash, gm.rocket > 0.0, gm.t);
    for p in &gm.parts {
        let k = (p.life / p.max).clamp(0.0, 1.0);
        g.circle(v.x(p.x), v.y(p.y), p.size * s * k, rgba(p.col, k));
    }
}
