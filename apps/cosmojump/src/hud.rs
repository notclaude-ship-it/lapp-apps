//! Interface : score, écran titre, écran de fin.
use crate::game::*;
use lapp_sdk::prelude::*;

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
    ctx.text_mid(g, s, x + d, y + d, &mk(rgba(0x000000, 0.45)));
    ctx.text_mid(g, s, x, y, &mk(col));
}

pub fn draw(gm: &Game, ctx: &Ctx, g: &mut Gfx) {
    let (w, h, s) = (ctx.width(), ctx.height(), ctx.scale());
    let white = rgb(0xFFFFFF);
    match gm.st {
        St::Title => {
            shadow(ctx, g, "COSMO", w / 2.0, h * 0.17, 46.0 * s, rgb(0x7CFFB2), 1);
            shadow(ctx, g, "JUMP", w / 2.0, h * 0.17 + 46.0 * s, 46.0 * s, rgb(0xFFD23C), 1);
            let pulse = 0.6 + 0.4 * (gm.t * 3.0).sin();
            shadow(ctx, g, "Touche pour jouer", w / 2.0, h * 0.62, 18.0 * s, with_alpha(white, pulse), 1);
            shadow(ctx, g, "Glisse le doigt pour te déplacer", w / 2.0, h * 0.62 + 26.0 * s, 12.0 * s, rgba(0xFFFFFF, 0.85), 1);
            shadow(ctx, g, "Écrase les ovnis, vise les fusées !", w / 2.0, h * 0.62 + 44.0 * s, 10.0 * s, rgba(0xFFFFFF, 0.7), 1);
            if gm.best > 0 {
                shadow(ctx, g, &format!("Record : {}", gm.best), w / 2.0, h * 0.62 + 74.0 * s, 15.0 * s, rgb(0xFFD23C), 1);
            }
        }
        St::Play | St::Over => {
            shadow(ctx, g, &format!("{}", gm.score()), 14.0 * s, 26.0 * s, 28.0 * s, white, 0);
            shadow(ctx, g, &format!("record {}", gm.best.max(gm.score())), 14.0 * s, 50.0 * s, 11.0 * s, rgba(0xFFFFFF, 0.8), 0);
            let cx = w - 22.0 * s;
            g.circle(cx, 26.0 * s, 8.0 * s, rgb(0xE0A21C));
            g.circle(cx, 26.0 * s, 5.5 * s, rgb(0xFFD23C));
            shadow(ctx, g, &format!("{}", gm.coins_n), cx - 14.0 * s, 26.0 * s, 18.0 * s, white, 2);
            if gm.rocket > 0.0 {
                let bw = 90.0 * s * (gm.rocket / 2.2).clamp(0.0, 1.0);
                g.rrect(w - 104.0 * s, 44.0 * s, 90.0 * s, 6.0 * s, 3.0 * s, rgba(0x000000, 0.35));
                g.rrect(w - 104.0 * s, 44.0 * s, bw, 6.0 * s, 3.0 * s, rgb(0xFFA23C));
            }
        }
    }
    if gm.st == St::Over {
        let a = (gm.over_t * 3.0).clamp(0.0, 1.0);
        g.rect(0.0, 0.0, w, h, rgba(0x05030F, 0.62 * a));
        let (pw, ph) = ((w - 40.0 * s).min(320.0 * s), 250.0 * s);
        let (px, py) = ((w - pw) / 2.0, h * 0.5 - ph / 2.0 - 30.0 * s + (1.0 - a) * 30.0 * s);
        g.rrect(px, py, pw, ph, 20.0 * s, rgba(0x1E1B4B, 0.95 * a));
        g.rrect(px, py, pw, 5.0 * s, 2.5 * s, rgba(0x7CFFB2, a));
        let cx = w / 2.0;
        shadow(ctx, g, "Perdu !", cx, py + 38.0 * s, 30.0 * s, with_alpha(white, a), 1);
        shadow(ctx, g, &format!("{}", gm.score()), cx, py + 88.0 * s, 44.0 * s, with_alpha(rgb(0x7CFFB2), a), 1);
        let alt = ((gm.maxy - 40.0).max(0.0) / 10.0) as u32;
        let info = format!("{} m  ·  {} pièces  ·  {} ovnis", alt, gm.coins_n, gm.stomps);
        shadow(ctx, g, &info, cx, py + 128.0 * s, 11.0 * s, rgba(0xFFFFFF, 0.8 * a), 1);
        if gm.new_record {
            shadow(ctx, g, "Nouveau record !", cx, py + 152.0 * s, 17.0 * s, with_alpha(rgb(0xFFD23C), a), 1);
        } else {
            shadow(ctx, g, &format!("Record : {}", gm.best), cx, py + 152.0 * s, 15.0 * s, rgba(0xFFD23C, a), 1);
        }
        ui::button(ctx, g, replay_rect(ctx), "Rejouer", ButtonStyle::Primary, false);
    }
}

/// Zone du bouton "Rejouer".
pub fn replay_rect(ctx: &Ctx) -> Rect {
    let (w, h, s) = (ctx.width(), ctx.height(), ctx.scale());
    let ph = 250.0 * s;
    let py = h * 0.5 - ph / 2.0 - 30.0 * s;
    Rect::new(w / 2.0 - 80.0 * s, py + 178.0 * s, 160.0 * s, 46.0 * s)
}
