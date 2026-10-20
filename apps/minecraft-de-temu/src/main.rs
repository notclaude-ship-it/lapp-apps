use lapp_sdk::prelude::*;

const COLS: usize = 16;
const ROWS: usize = 10;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Block {
    Air,
    Grass,
    Dirt,
    Stone,
    Wood,
    Leaves,
    Diamond,
    Tnt,
}

impl Block {
    fn color(&self) -> Color {
        match self {
            Block::Air => rgba(0, 0),
            Block::Grass => rgb(0x48A121),
            Block::Dirt => rgb(0x795548),
            Block::Stone => rgb(0x757575),
            Block::Wood => rgb(0x5D4037),
            Block::Leaves => rgb(0x2E7D32),
            Block::Diamond => rgb(0x00E5FF),
            Block::Tnt => rgb(0xFF1744),
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Block::Air => "Air",
            Block::Grass => "Herbe",
            Block::Dirt => "Terre",
            Block::Stone => "Pierre",
            Block::Wood => "Bois",
            Block::Leaves => "Feuilles",
            Block::Diamond => "Diamant",
            Block::Tnt => "TNT",
        }
    }
}

const SELECTABLE: [Block; 6] = [
    Block::Grass,
    Block::Dirt,
    Block::Stone,
    Block::Wood,
    Block::Diamond,
    Block::Tnt,
];

struct TemuMinecraft {
    world: [[Block; COLS]; ROWS],
    selected_slot: usize,
    inventory: [u32; 6],
    temu_coins: u32,
    discount_timer: f32,
    show_shop: bool,
    store: Store,
}

impl TemuMinecraft {
    fn new() -> Self {
        let store = Store::open("temu_minecraft.txt");
        let temu_coins = store.get_i64("coins").unwrap_or(999) as u32;

        let mut world = [[Block::Air; COLS]; ROWS];
        for c in 0..COLS {
            world[3][c] = Block::Grass;
            world[4][c] = Block::Dirt;
            world[5][c] = Block::Dirt;
            for r in 6..ROWS {
                if (r + c) % 5 == 0 {
                    world[r][c] = Block::Diamond;
                } else if (r * c) % 7 == 0 {
                    world[r][c] = Block::Tnt;
                } else {
                    world[r][c] = Block::Stone;
                }
            }
        }
        world[2][3] = Block::Wood;
        world[1][3] = Block::Wood;
        world[0][2] = Block::Leaves;
        world[0][3] = Block::Leaves;
        world[0][4] = Block::Leaves;

        Self {
            world,
            selected_slot: 0,
            inventory: [20, 50, 30, 15, 5, 10],
            temu_coins,
            discount_timer: 0.0,
            show_shop: false,
            store,
        }
    }

    fn block_index(b: Block) -> Option<usize> {
        SELECTABLE.iter().position(|&x| x == b)
    }
}

impl App for TemuMinecraft {
    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        self.discount_timer += dt;
        if self.discount_timer > 0.5 {
            ctx.animate();
        }
    }

    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let (w, h) = ctx.size();
        let (wf, hf) = (w as f32, h as f32);
        let s = ctx.scale();

        g.clear(rgb(0x87CEEB));
        g.circle(wf - 35.0 * s, 45.0 * s, 22.0 * s, rgb(0xFFD700));

        let banner_h = 36.0 * s;
        g.rect(0.0, 0.0, wf, banner_h, rgb(0xFF4500));
        let flash_text = if (self.discount_timer * 3.0) as u32 % 2 == 0 {
            "⚡ TEMU CRAFT - 99% DE RÉDUCTION SUR LE DIAMANT! ⚡"
        } else {
            "🎁 TOUT À 0,01€ - LIVRAISON GRATUITE SUR LE MINECRAFT 🎁"
        };
        let st_banner = Text::new(11.0 * s).bold().color(rgb(0xFFFFFF)).center();
        ctx.text_mid(g, flash_text, wf / 2.0, banner_h / 2.0, &st_banner);

        let shop_btn = Rect::new(wf - 70.0 * s, banner_h + 5.0 * s, 65.0 * s, 24.0 * s);
        ui::button(ctx, g, shop_btn, "BOUTIQUE", ButtonStyle::Primary, false);

        let coin_str = format!("💰 {} Coins", self.temu_coins);
        let st_coins = Text::new(13.0 * s).bold().color(rgb(0x000000)).left();
        ctx.text_mid(g, &coin_str, 10.0 * s, banner_h + 17.0 * s, &st_coins);

        let grid_y_start = banner_h + 35.0 * s;
        let cell_size = (wf / COLS as f32).min((hf - grid_y_start - 70.0 * s) / ROWS as f32);

        for r in 0..ROWS {
            for c in 0..COLS {
                let bx = c as f32 * cell_size;
                let by = grid_y_start + r as f32 * cell_size;
                let block = self.world[r][c];

                if block != Block::Air {
                    g.rect(bx, by, cell_size - 1.0, cell_size - 1.0, block.color());
                    g.rect(bx, by, cell_size - 1.0, 2.0 * s, rgba(0xFFFFFF, 0.3));
                    if block == Block::Tnt {
                        let st_tnt = Text::new(cell_size * 0.4).bold().color(rgb(0xFFFFFF)).center();
                        ctx.text_mid(g, "TNT", bx + cell_size / 2.0, by + cell_size / 2.0, &st_tnt);
                    } else if block == Block::Diamond {
                        let st_d = Text::new(cell_size * 0.4).bold().color(rgb(0x000000)).center();
                        ctx.text_mid(g, "💎", bx + cell_size / 2.0, by + cell_size / 2.0, &st_d);
                    }
                }
            }
        }

        let hotbar_h = 60.0 * s;
        let hotbar_y = hf - hotbar_h;
        g.rect(0.0, hotbar_y, wf, hotbar_h, rgba(0x1C1C21, 0.9));

        let slot_w = wf / SELECTABLE.len() as f32;
        for (i, &blk) in SELECTABLE.iter().enumerate() {
            let sx = i as f32 * slot_w;
            let sy = hotbar_y + 5.0 * s;
            let is_sel = i == self.selected_slot;

            let slot_r = Rect::new(sx + 2.0, sy, slot_w - 4.0, hotbar_h - 10.0 * s);
            let border_color = if is_sel { rgb(0xFFD700) } else { rgb(0x555555) };
            g.rrect(slot_r.x, slot_r.y, slot_r.w, slot_r.h, 6.0 * s, border_color);
            g.rrect(slot_r.x + 2.0, slot_r.y + 2.0, slot_r.w - 4.0, slot_r.h - 4.0, 4.0 * s, rgb(0x222222));

            let icon_sz = 20.0 * s;
            let cx = slot_r.x + (slot_r.w - icon_sz) / 2.0;
            g.rect(cx, slot_r.y + 6.0 * s, icon_sz, icon_sz, blk.color());

            let count = self.inventory[i];
            let st_cnt = Text::new(10.0 * s).color(rgb(0xFFFFFF)).center();
            ctx.text_mid(g, &format!("{}", count), slot_r.x + slot_r.w / 2.0, slot_r.y + slot_r.h - 10.0 * s, &st_cnt);
        }

        if self.show_shop {
            g.rect(0.0, 0.0, wf, hf, rgba(0x000000, 0.7));

            let modal_r = Rect::new(20.0 * s, 50.0 * s, wf - 40.0 * s, hf - 100.0 * s);
            g.rrect(modal_r.x, modal_r.y, modal_r.w, modal_r.h, 12.0 * s, rgb(0xFFFFFF));

            let st_title = Text::new(18.0 * s).bold().color(rgb(0xFF4500)).center();
            ctx.text_mid(g, "🛒 TEMU CRAFT STORE 🛒", modal_r.x + modal_r.w / 2.0, modal_r.y + 25.0 * s, &st_title);

            let st_sub = Text::new(12.0 * s).color(rgb(0x333333)).center();
            ctx.text_mid(g, "Offres exclusives 0€ !", modal_r.x + modal_r.w / 2.0, modal_r.y + 48.0 * s, &st_sub);

            let buy1 = Rect::new(modal_r.x + 15.0 * s, modal_r.y + 70.0 * s, modal_r.w - 30.0 * s, 40.0 * s);
            ui::button(ctx, g, buy1, "💎 +50 Diamants (100 Coins)", ButtonStyle::Primary, false);

            let buy2 = Rect::new(modal_r.x + 15.0 * s, modal_r.y + 120.0 * s, modal_r.w - 30.0 * s, 40.0 * s);
            ui::button(ctx, g, buy2, "🧨 +20 TNT (50 Coins)", ButtonStyle::Secondary, false);

            let close = Rect::new(modal_r.x + 15.0 * s, modal_r.y + modal_r.h - 45.0 * s, modal_r.w - 30.0 * s, 35.0 * s);
            ui::button(ctx, g, close, "Fermer", ButtonStyle::Danger, false);
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        let (w, h) = ctx.size();
        let (wf, hf) = (w as f32, h as f32);
        let s = ctx.scale();

        if let Event::Tap { x, y } = ev {
            let banner_h = 36.0 * s;

            if self.show_shop {
                let modal_r = Rect::new(20.0 * s, 50.0 * s, wf - 40.0 * s, hf - 100.0 * s);
                let buy1 = Rect::new(modal_r.x + 15.0 * s, modal_r.y + 70.0 * s, modal_r.w - 30.0 * s, 40.0 * s);
                let buy2 = Rect::new(modal_r.x + 15.0 * s, modal_r.y + 120.0 * s, modal_r.w - 30.0 * s, 40.0 * s);
                let close = Rect::new(modal_r.x + 15.0 * s, modal_r.y + modal_r.h - 45.0 * s, modal_r.w - 30.0 * s, 35.0 * s);

                if buy1.contains(x, y) {
                    if self.temu_coins >= 100 {
                        self.temu_coins -= 100;
                        self.inventory[4] += 50;
                        let _ = self.store.set("coins", self.temu_coins);
                        ctx.toast("Achat réussi ! +50 Diamants 💎");
                    } else {
                        ctx.toast("Pas assez de Temu Coins !");
                    }
                    return;
                }
                if buy2.contains(x, y) {
                    if self.temu_coins >= 50 {
                        self.temu_coins -= 50;
                        self.inventory[5] += 20;
                        let _ = self.store.set("coins", self.temu_coins);
                        ctx.toast("Achat réussi ! +20 TNT 🧨");
                    } else {
                        ctx.toast("Pas assez de Temu Coins !");
                    }
                    return;
                }
                if close.contains(x, y) || !modal_r.contains(x, y) {
                    self.show_shop = false;
                    return;
                }
                return;
            }

            let shop_btn = Rect::new(wf - 70.0 * s, banner_h + 5.0 * s, 65.0 * s, 24.0 * s);
            if shop_btn.contains(x, y) {
                self.show_shop = true;
                return;
            }

            let hotbar_h = 60.0 * s;
            let hotbar_y = hf - hotbar_h;
            if y >= hotbar_y {
                let slot_w = wf / SELECTABLE.len() as f32;
                let idx = (x / slot_w) as usize;
                if idx < SELECTABLE.len() {
                    self.selected_slot = idx;
                    ctx.toast(format!("Sélection : {}", SELECTABLE[idx].label()));
                }
                return;
            }

            let grid_y_start = banner_h + 35.0 * s;
            let cell_size = (wf / COLS as f32).min((hf - grid_y_start - 70.0 * s) / ROWS as f32);

            if y >= grid_y_start {
                let col = (x / cell_size) as usize;
                let row = ((y - grid_y_start) / cell_size) as usize;

                if col < COLS && row < ROWS {
                    let current = self.world[row][col];
                    if current != Block::Air {
                        self.world[row][col] = Block::Air;
                        if let Some(inv_idx) = Self::block_index(current) {
                            self.inventory[inv_idx] += 1;
                        }
                        self.temu_coins += 5;
                        let _ = self.store.set("coins", self.temu_coins);
                        ctx.toast("Bloc miné ! (+5 Temu Coins)");
                    } else {
                        let sel_blk = SELECTABLE[self.selected_slot];
                        if self.inventory[self.selected_slot] > 0 {
                            self.inventory[self.selected_slot] -= 1;
                            self.world[row][col] = sel_blk;
                        } else {
                            ctx.toast("Plus de blocs en stock ! Visitez la boutique !");
                        }
                    }
                }
            }
        }
    }

    fn on_back(&mut self, _ctx: &Ctx) -> bool {
        if self.show_shop {
            self.show_shop = false;
            true
        } else {
            false
        }
    }
}

fn main() {
    run(
        Config::new("Temu Minecraft").size(272, 480).gpu(true),
        TemuMinecraft::new(),
    );
}
