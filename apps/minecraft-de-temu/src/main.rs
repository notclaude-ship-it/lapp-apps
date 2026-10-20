use lapp_sdk::prelude::*;

struct MinecraftTemuApp;

impl App for MinecraftTemuApp {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        g.clear(ctx.theme.bg);
        let (w, h) = ctx.size();
        let s = ctx.scale();

        // Draw a green background
        g.rect(0.0, 0.0, w as f32, h as f32, ui::rgb(0x4CAF50)); // Green like grass

        // Draw a brown block
        let block_size = 80.0 * s;
        let block_x = (w as f32 - block_size) / 2.0;
        let block_y = (h as f32 - block_size) / 2.0;
        g.rect(block_x, block_y, block_size, block_size, ui::rgb(0x795548)); // Brown

        // Draw some text
        let text_style = Text::new(20.0 * s).bold().color(ui::rgb(0xFFFFFF)).center();
        ctx.text_mid(g, "Minecraft de Temu", w as f32 / 2.0, h as f32 / 4.0, &text_style);
        ctx.text_mid(g, "Coming Soon!", w as f32 / 2.0, h as f32 * 3.0 / 4.0, &text_style);
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        if let Event::Tap { .. } = ev {
            ctx.toast("You tapped the TemuCraft!");
        }
    }

    fn tick(&mut self, _ctx: &Ctx, _dt: f32) {
        // No animations for this placeholder
    }
}

fn main() {
    run(Config::new("Minecraft de Temu").gpu(true), MinecraftTemuApp);
}
