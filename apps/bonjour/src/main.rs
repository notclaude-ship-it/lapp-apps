use lapp_sdk::prelude::*;

struct BonjourApp;

impl App for BonjourApp {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        g.clear(ctx.theme.bg);
        let s = ctx.scale();
        let st = Text::new(24.0 * s).bold().color(ctx.theme.text).center();
        ctx.text_mid(g, "Bonjour !", ctx.width() / 2.0, ctx.height() / 2.0, &st);
    }
}

fn main() {
    run(Config::new("Bonjour"), BonjourApp);
}
