//! Rendu : décor, combattants, interface, commandes tactiles.
use crate::game::*;
use lapp_sdk::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Ctl {
    Left,
    Right,
    Jump,
    Block,
    Punch,
    Kick,
}

pub struct Layout {
    pub s: f32,
    pub vw: f32,
    pub vh: f32,
    pub ground: f32,
}

pub fn layout(ctx: &Ctx) -> Layout {
    let s = ctx.scale();
    let (vw, vh) = (ctx.width() / s, ctx.height() / s);
    Layout { s, vw, vh, ground: vh * 0.585 }
}

/// Rectangles (x, y, w, h) en points.
pub fn controls(l: &Layout) -> [(Ctl, (f32, f32, f32, f32)); 6] {
    let top = l.vh * 0.715;
    let bot = l.vh * 0.935;
    let hh = bot - top;
    let half = l.vw * 0.5;
    let aw = (half - 10.0 - 12.0) / 2.0;
    let rx = half + 2.0;
    let cw = (l.vw - 10.0 - rx - 6.0) / 2.0;
    let rh = (hh - 6.0) / 2.0;
    [
        (Ctl::Left, (10.0, top, aw, hh)),
        (Ctl::Right, (10.0 + aw + 6.0, top, aw, hh)),
        (Ctl::Jump, (rx, top, cw, rh)),
        (Ctl::Block, (rx + cw + 6.0, top, cw, rh)),
        (Ctl::Punch, (rx, top + rh + 6.0, cw, rh)),
        (Ctl::Kick, (rx + cw + 6.0, top + rh + 6.0, cw, rh)),
    ]
}

pub fn hit_test(l: &Layout, x: f32, y: f32) -> Option<Ctl> {
    let (px, py) = (x / l.s, y / l.s);
    controls(l).iter().find(|(_, (rx, ry, rw, rh))| px >= rx - 3.0 && px <= rx + rw + 3.0 && py >= ry - 3.0 && py <= ry + rh + 3.0).map(|c| c.0)
}

fn shadow(ctx: &Ctx, g: &mut Gfx, s: &str, x: f32, y: f32, px: f32, col: Color, align: u8) {
    let mk = |c: Color| {
        let t = Text::new(px).bold().color(c);
        match align {
            0 => t.left(),
            1 => t.center(),
            _ => t.right(),
        }
    };
    let d = (px * 0.07).max(1.0);
    ctx.text_mid(g, s, x + d, y + d, &mk(rgba(0x000000, 0.5)));
    ctx.text_mid(g, s, x, y, &mk(col));
}

fn lerp_col(a: u32, b: u32, t: f32) -> Color {
    let ch = |sh: u32| {
        let (x, y) = (((a >> sh) & 255) as f32, ((b >> sh) & 255) as f32);
        (x + (y - x) * t) as u32
    };
    rgb((ch(16) << 16) | (ch(8) << 8) | ch(0))
}

fn limb(g: &mut Gfx, x0: f32, y0: f32, x1: f32, y1: f32, r: f32, col: Color) {
    let (dx, dy) = (x1 - x0, y1 - y0);
    let d = (dx * dx + dy * dy).sqrt();
    let n = ((d / (r * 0.6)).ceil() as i32).max(1);
    for i in 0..=n {
        let t = i as f32 / n as f32;
        g.circle(x0 + dx * t, y0 + dy * t, r, col);
    }
}

pub fn background(gm: &Game, g: &mut Gfx, l: &Layout, ox: f32, oy: f32) {
    let s = l.s;
    let w = l.vw * s + 40.0;
    let gy = l.ground;
    // ciel : dégradé en bandes
    let bands = 22;
    for i in 0..bands {
        let t = i as f32 / (bands - 1) as f32;
        let c = lerp_col(0x1A0F3C, 0xFF7B54, t * t);
        let y0 = gy * s * i as f32 / bands as f32;
        g.rect(-20.0 + ox, y0 + oy - 20.0, w, gy * s / bands as f32 + 22.0, c);
    }
    // lune et étoiles
    g.circle((l.vw * 0.5) * s + ox * 0.2, gy * 0.22 * s, 22.0 * s, rgba(0xFFE9C4, 0.95));
    g.circle((l.vw * 0.5 - 7.0) * s + ox * 0.2, gy * 0.22 * s - 4.0 * s, 20.0 * s, rgba(0xFFF6E3, 0.5));
    for i in 0..14 {
        let x = ((i * 97 + 31) % 272) as f32 / 272.0 * l.vw;
        let y = ((i * 53 + 11) % 100) as f32 / 100.0 * gy * 0.45;
        let tw = 0.5 + 0.5 * (gm.t * 2.0 + i as f32).sin();
        g.circle(x * s, y * s, 0.9 * s, rgba(0xFFFFFF, 0.35 + 0.4 * tw));
    }
    // skylines (deux plans)
    for layer in 0..2 {
        let (col, base, par) = if layer == 0 { (0x3A1F57, 62.0, 0.15) } else { (0x24103C, 40.0, 0.4) };
        let mut x = -10.0;
        let mut i = layer * 7;
        while x < l.vw + 20.0 {
            let bw = 22.0 + ((i * 37) % 24) as f32;
            let bh = base + ((i * 53) % 60) as f32 * if layer == 0 { 0.7 } else { 1.1 };
            let bx = (x * s) + ox * par;
            g.rect(bx, (gy - bh) * s + oy * par, bw * s, bh * s + 4.0, rgb(col));
            if layer == 1 {
                let rows = (bh / 14.0) as i32;
                for r in 0..rows {
                    for c in 0..(bw / 9.0) as i32 {
                        if (i as i32 + r * 3 + c * 5) % 4 < 2 {
                            g.rect(bx + (4.0 + c as f32 * 9.0) * s, (gy - bh + 6.0 + r as f32 * 14.0) * s + oy * par, 4.0 * s, 6.0 * s, rgba(0xFFD27A, 0.55));
                        }
                    }
                }
            }
            x += bw + 2.0;
            i += 1;
        }
    }
    // sol
    g.rect(-20.0 + ox, gy * s + oy, w, (l.vh - gy) * s + 40.0, rgb(0x2B1D3F));
    g.rect(-20.0 + ox, gy * s + oy, w, 4.0 * s, rgb(0xFF9D6B));
    g.rect(-20.0 + ox, gy * s + oy + 4.0 * s, w, 6.0 * s, rgb(0x4A3166));
    for i in 0..9 {
        let y = gy + 12.0 + (i * i) as f32 * 1.9;
        if y < l.vh {
            g.rect(-20.0 + ox, y * s + oy, w, 1.2 * s, rgba(0xFFFFFF, 0.05 + i as f32 * 0.008));
        }
    }
}

struct Pose {
    legs: [(f32, f32); 2], // pieds [avant, arrière]
    hands: [(f32, f32); 2], // mains [avant, arrière]
    lean: f32,
}

fn pose(f: &Fighter, t: f32) -> Pose {
    let k = f.strike();
    let kk = k.max(0.0);
    let bob = (t * 5.0 + f.x).sin();
    let mut p = Pose { legs: [(11.0, 0.0), (-11.0, 0.0)], hands: [(15.0, 54.0 + bob), (9.0, 58.0 + bob)], lean: 1.0 };
    match f.act {
        Act::Walk => {
            let w = f.walk;
            p.legs = [(10.0 + 9.0 * w.sin(), (6.0 * w.cos()).max(0.0)), (-10.0 - 9.0 * w.sin(), (-6.0 * w.cos()).max(0.0))];
            p.hands = [(14.0 + 3.0 * w.cos(), 54.0), (9.0 - 3.0 * w.cos(), 58.0)];
        }
        Act::Punch => {
            p.lean = 1.0 + 6.0 * k;
            p.hands[0] = (14.0 + 36.0 * kk + 6.0 * k.min(0.0), 61.0 + 2.0 * k.min(0.0));
            p.hands[1] = (8.0, 58.0);
            p.legs = [(14.0, 0.0), (-14.0, 0.0)];
        }
        Act::Kick => {
            p.lean = -5.0 * kk;
            p.legs[0] = (10.0 + 46.0 * kk - 6.0 * k.min(0.0).abs(), 4.0 + 32.0 * kk + 10.0 * k.min(0.0).abs());
            p.hands = [(12.0, 62.0), (-2.0 - 8.0 * kk, 58.0)];
        }
        Act::Block => {
            p.hands = [(17.0, 72.0), (13.0, 62.0)];
            p.legs = [(14.0, 0.0), (-12.0, 0.0)];
            p.lean = 3.0;
        }
        Act::Hurt => {
            p.lean = -9.0;
            p.hands = [(-2.0, 46.0), (-8.0, 50.0)];
            p.legs = [(8.0, 0.0), (-14.0, 0.0)];
        }
        _ => {}
    }
    if f.h > 1.0 && f.act != Act::Kick && f.act != Act::Ko {
        p.legs = [(12.0, 18.0), (-9.0, 11.0)];
    }
    if f.act == Act::Idle || f.act == Act::Block {
        // respiration
        p.legs[0].0 += 0.0;
    }
    p
}

pub fn fighter(g: &mut Gfx, l: &Layout, f: &Fighter, t: f32, gi: u32, trim: u32, ox: f32, oy: f32, is_ko_frame: bool) {
    let s = l.s;
    let gy = l.ground;
    // ombre
    let sh = (1.0 - f.h / 90.0).clamp(0.3, 1.0);
    g.rrect((f.x - 18.0 * sh) * s + ox, (gy + 1.0) * s + oy, 36.0 * sh * s, 6.0 * s, 3.0 * s, rgba(0x000000, 0.35 * sh));

    let flash = f.flash > 0.0 && ((t * 50.0) as i32) % 2 == 0;
    let skin = if flash { rgb(0xFFFFFF) } else { rgb(0xF0C49C) };
    let suit = if flash { rgb(0xFFFFFF) } else { rgb(gi) };
    let pants = if flash { rgb(0xFFFFFF) } else { shade(rgb(gi), 0.62) };
    let band = if flash { rgb(0xFFFFFF) } else { rgb(trim) };

    let p = pose(f, t);
    let fc = f.face;
    // angle de chute pour le K.O.
    let th = if f.act == Act::Ko { (f.t / 0.35).min(1.0) * 1.45 } else { 0.0 };
    let (sn, cs) = (th.sin(), th.cos());
    let pt = |px: f32, py: f32| -> (f32, f32) {
        let (x, y) = (px * cs - py * sn, px * sn + py * cs);
        let y = if th > 0.0 { y.max(4.0) } else { y };
        (f.x * s + fc * x * s + ox, (gy - f.h - y) * s + oy)
    };
    let _ = is_ko_frame;

    let bob = if f.act == Act::Idle || f.act == Act::Block { (t * 5.0 + f.x).sin() * 0.8 } else { 0.0 };
    let hip = (0.0, 36.0 + bob);
    let sho = (p.lean, 64.0 + bob);
    let head = (p.lean * 1.3 + 2.0, 77.0 + bob);

    let lim = |g: &mut Gfx, a: (f32, f32), b: (f32, f32), r: f32, c: Color| {
        let (x0, y0) = pt(a.0, a.1);
        let (x1, y1) = pt(b.0, b.1);
        limb(g, x0, y0, x1, y1, r * s, c);
    };
    let leg = |g: &mut Gfx, foot: (f32, f32), c: Color| {
        let kn = ((hip.0 + foot.0) * 0.5 + 5.0, (hip.1 + foot.1) * 0.5 + 1.0);
        lim(g, hip, kn, 6.0, c);
        lim(g, kn, foot, 5.2, c);
        let (fx, fy) = pt(foot.0 + 3.0, foot.1 + 2.0);
        g.circle(fx, fy, 5.0 * s, if flash { rgb(0xFFFFFF) } else { rgb(0x20122E) });
    };
    let arm = |g: &mut Gfx, hand: (f32, f32), c: Color, cuff: Color| {
        let el = ((sho.0 + hand.0) * 0.5 - 3.0, (sho.1 + hand.1) * 0.5 - 4.0);
        lim(g, sho, el, 4.8, c);
        lim(g, el, hand, 4.2, c);
        let (hx, hy) = pt(hand.0, hand.1);
        g.circle(hx, hy, 5.2 * s, cuff);
    };

    // plan arrière : bras et jambe arrière
    arm(g, p.hands[1], skin, shade(band, 0.8));
    leg(g, p.legs[1], pants);
    // torse (haut + ceinture)
    lim(g, hip, sho, 8.8, suit);
    lim(g, (hip.0, hip.1 + 1.0), (hip.0 + 0.5, hip.1 + 3.0), 9.2, band);
    // jambe avant
    leg(g, p.legs[0], pants);
    // tête
    let (hx, hy) = pt(head.0, head.1);
    g.circle(hx, hy, 9.2 * s, skin);
    // cheveux / bandeau
    let (bx, by) = pt(head.0 + 0.5, head.1 + 2.5);
    g.circle(bx - fc * 0.0, by - 2.0 * s, 9.4 * s, if flash { rgb(0xFFFFFF) } else { rgb(0x2A1B1B) });
    g.circle(hx + fc * 1.5 * s, hy + 3.0 * s, 8.4 * s, skin);
    let (ax, ay) = pt(head.0, head.1 + 1.0);
    g.rrect(ax - 9.5 * s, ay - 1.2 * s, 19.0 * s, 3.4 * s, 1.7 * s, band);
    // œil
    let (ex, ey) = pt(head.0 + 4.5, head.1 - 1.0);
    if f.act == Act::Ko || f.act == Act::Hurt {
        g.rect(ex - 2.0 * s, ey - 0.5 * s, 4.0 * s, 1.2 * s, rgb(0x111111));
    } else {
        g.circle(ex, ey, 1.5 * s, rgb(0x111111));
    }
    // bras avant
    arm(g, p.hands[0], suit, skin);
    if f.act != Act::Ko {
        let (cx, cy) = pt(p.hands[0].0, p.hands[0].1);
        g.circle(cx, cy, 5.2 * s, band);
    }
}

fn bar(g: &mut Gfx, x: f32, y: f32, w: f32, h: f32, frac: f32, from_right: bool, s: f32) {
    g.rrect(x - 2.0 * s, y - 2.0 * s, w + 4.0 * s, h + 4.0 * s, (h / 2.0 + 2.0 * s).min(10.0 * s), rgba(0x000000, 0.6));
    g.rrect(x, y, w, h, h / 2.0, rgb(0x3B2442));
    let fw = (w * frac.clamp(0.0, 1.0)).max(0.0);
    if fw > 1.0 {
        let col = if frac > 0.5 { 0x5BE37A } else if frac > 0.25 { 0xFFD23C } else { 0xFF4D4D };
        let fx = if from_right { x + w - fw } else { x };
        g.rrect(fx, y, fw, h, h / 2.0, rgb(col));
        g.rrect(fx, y, fw, h * 0.4, h / 2.0, rgba(0xFFFFFF, 0.3));
    }
}

pub fn hud(gm: &Game, ctx: &Ctx, g: &mut Gfx, l: &Layout) {
    let s = l.s;
    let mid = l.vw / 2.0;
    let bw = mid - 36.0;
    bar(g, 10.0 * s, 16.0 * s, bw * s, 13.0 * s, gm.p[0].hp / 100.0, false, s);
    bar(g, (l.vw - 10.0 - bw) * s, 16.0 * s, bw * s, 13.0 * s, gm.p[1].hp / 100.0, true, s);
    shadow(ctx, g, "TOI", 12.0 * s, 42.0 * s, 11.0 * s, rgb(0x9CD0FF), 0);
    shadow(ctx, g, &format!("IA niv. {}", gm.level), (l.vw - 12.0) * s, 42.0 * s, 11.0 * s, rgb(0xFF9C9C), 2);
    // manches gagnées
    for i in 0..2u32 {
        let on0 = i < gm.wins[0];
        let on1 = i < gm.wins[1];
        g.circle((70.0 + i as f32 * 13.0) * s, 42.0 * s, 4.2 * s, if on0 { rgb(0xFFD23C) } else { rgba(0xFFFFFF, 0.2) });
        g.circle((l.vw - 70.0 - i as f32 * 13.0) * s, 42.0 * s, 4.2 * s, if on1 { rgb(0xFFD23C) } else { rgba(0xFFFFFF, 0.2) });
    }
    // chrono
    g.circle(mid * s, 24.0 * s, 17.0 * s, rgba(0x000000, 0.65));
    let low = gm.timer < 10.0 && gm.st == St::Fight;
    let col = if low && ((gm.t * 4.0) as i32) % 2 == 0 { rgb(0xFF5555) } else { rgb(0xFFFFFF) };
    shadow(ctx, g, &format!("{}", gm.timer.ceil().max(0.0) as i32), mid * s, 24.0 * s, 20.0 * s, col, 1);
    if gm.combo >= 2 && gm.combo_t > 0.0 {
        let a = (gm.combo_t * 2.0).clamp(0.0, 1.0);
        shadow(ctx, g, &format!("COMBO x{}", gm.combo), 12.0 * s, 62.0 * s, 15.0 * s, with_alpha(rgb(0xFFD23C), a), 0);
    }
}

pub fn pad(ctx: &Ctx, g: &mut Gfx, l: &Layout, held: Option<Ctl>) {
    let s = l.s;
    g.rect(0.0, l.vh * 0.70 * s - 6.0 * s, l.vw * s + 2.0, l.vh * s, rgba(0x0E0818, 0.78));
    for (c, (x, y, w, h)) in controls(l) {
        let on = held == Some(c);
        let (base, lab, size) = match c {
            Ctl::Left => (0x3A3366, "‹", 54.0),
            Ctl::Right => (0x3A3366, "›", 54.0),
            Ctl::Jump => (0x2E5A7A, "SAUT", 13.0),
            Ctl::Block => (0x3E6B4B, "GARDE", 13.0),
            Ctl::Punch => (0xA8452F, "POING", 14.0),
            Ctl::Kick => (0xB8792A, "PIED", 14.0),
        };
        let col = if on { shade(rgb(base), 1.45) } else { rgb(base) };
        g.rrect(x * s, (y + 3.0) * s, w * s, h * s, 14.0 * s, rgba(0x000000, 0.45));
        g.rrect(x * s, y * s, w * s, h * s, 14.0 * s, col);
        g.rrect(x * s, y * s, w * s, h * 0.45 * s, 14.0 * s, rgba(0xFFFFFF, if on { 0.05 } else { 0.12 }));
        let ty = y + h * 0.5 - if matches!(c, Ctl::Left | Ctl::Right) { 3.0 } else { 0.0 };
        shadow(ctx, g, lab, (x + w * 0.5) * s, ty * s, size * s, rgb(0xFFFFFF), 1);
    }
}

pub fn overlay(gm: &Game, ctx: &Ctx, g: &mut Gfx, l: &Layout, best: u32) {
    let s = l.s;
    let cx = l.vw * 0.5 * s;
    let gy = l.ground;
    let white = rgb(0xFFFFFF);
    match gm.st {
        St::Title => {
            g.rect(0.0, 0.0, l.vw * s + 2.0, l.vh * s, rgba(0x08040F, 0.55));
            let bob = (gm.t * 2.0).sin() * 3.0;
            shadow(ctx, g, "COMBAT", cx, (gy * 0.36 + bob) * s, 50.0 * s, rgb(0xFFD23C), 1);
            shadow(ctx, g, "ARENA", cx, (gy * 0.36 + 44.0 + bob) * s, 36.0 * s, rgb(0xFF6B4A), 1);
            let pulse = 0.55 + 0.45 * (gm.t * 3.0).sin();
            shadow(ctx, g, &format!("Niveau {}", gm.level), cx, (gy * 0.78) * s, 18.0 * s, white, 1);
            shadow(ctx, g, "Touche pour combattre", cx, (gy * 0.78 + 28.0) * s, 14.0 * s, with_alpha(white, pulse), 1);
            shadow(ctx, g, "Gagne 2 manches · 45 s par manche", cx, (gy * 0.78 + 50.0) * s, 10.0 * s, rgba(0xFFFFFF, 0.75), 1);
            if best > 1 {
                shadow(ctx, g, &format!("Meilleur niveau : {}", best), cx, (gy * 0.78 + 68.0) * s, 10.0 * s, rgb(0xFFD23C), 1);
            }
        }
        St::Intro => {
            let a = (gm.st_t * 5.0).clamp(0.0, 1.0);
            if gm.st_t < 0.9 {
                shadow(ctx, g, &format!("MANCHE {}", gm.round), cx, gy * 0.42 * s, 36.0 * s * (1.4 - 0.4 * a), with_alpha(white, a), 1);
            } else {
                let z = 1.0 + (1.5 - gm.st_t).max(0.0) * 0.6;
                shadow(ctx, g, "FIGHT !", cx, gy * 0.42 * s, 46.0 * s * z, rgb(0xFF5A3C), 1);
            }
        }
        St::RoundEnd => {
            let ko = gm.p[0].act == Act::Ko || gm.p[1].act == Act::Ko;
            shadow(ctx, g, if ko { "K.O. !" } else { "TEMPS !" }, cx, gy * 0.38 * s, 56.0 * s, rgb(0xFF5A3C), 1);
            if gm.st_t > 0.6 {
                let t = match gm.round_winner {
                    0 => "Manche pour toi",
                    1 => "Manche pour l'IA",
                    _ => "Égalité",
                };
                shadow(ctx, g, t, cx, (gy * 0.38 + 40.0) * s, 17.0 * s, white, 1);
            }
        }
        St::MatchEnd => {
            let a = (gm.st_t * 3.0).clamp(0.0, 1.0);
            g.rect(0.0, 0.0, l.vw * s + 2.0, l.vh * s, rgba(0x05030F, 0.6 * a));
            let won = gm.player_won();
            let (pw, ph) = (l.vw * s - 36.0 * s, 150.0 * s);
            let py = gy * 0.2 * s + (1.0 - a) * 24.0 * s;
            g.rrect(18.0 * s, py, pw, ph, 20.0 * s, rgba(0x1E1240, 0.95 * a));
            g.rrect(18.0 * s, py, pw, 5.0 * s, 2.5 * s, with_alpha(if won { rgb(0x5BE37A) } else { rgb(0xFF5A5A) }, a));
            shadow(ctx, g, if won { "VICTOIRE !" } else { "DÉFAITE" }, cx, py + 40.0 * s, 32.0 * s, with_alpha(if won { rgb(0xFFD23C) } else { rgb(0xFF7A7A) }, a), 1);
            let sub = if won { format!("Niveau {} débloqué", gm.level + 1) } else { format!("Niveau {} · manches {}–{}", gm.level, gm.wins[0], gm.wins[1]) };
            shadow(ctx, g, &sub, cx, py + 78.0 * s, 12.0 * s, with_alpha(white, 0.85 * a), 1);
            let pulse = 0.55 + 0.45 * (gm.t * 3.0).sin();
            shadow(ctx, g, if won { "Touche : niveau suivant" } else { "Touche pour rejouer" }, cx, py + 116.0 * s, 14.0 * s, with_alpha(white, pulse * a), 1);
        }
        St::Fight => {}
    }
    if gm.flash > 0.0 {
        g.rect(0.0, 0.0, l.vw * s + 2.0, l.vh * s, rgba(0xFFFFFF, gm.flash.min(0.5)));
    }
}

pub fn particles(gm: &Game, g: &mut Gfx, l: &Layout, ox: f32, oy: f32) {
    let s = l.s;
    for q in &gm.parts {
        let a = (q.life / q.max).clamp(0.0, 1.0);
        g.circle(q.x * s + ox, (l.ground - q.h) * s + oy, q.size * s * (0.4 + 0.6 * a), with_alpha(rgb(q.col), a));
    }
}
