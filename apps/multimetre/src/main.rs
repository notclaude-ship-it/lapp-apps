use lapp_sdk::prelude::*;

struct MultimetreApp;

impl App for MultimetreApp {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        g.clear(ctx.theme.bg);
        let (w, h) = ctx.size();
        let s = ctx.scale();

        let title_style = Text::new(24.0 * s).bold().color(ctx.theme.text).center();
        ctx.text_mid(g, "Multimètre", w as f32 / 2.0, h as f32 / 3.0, &title_style);

        let info_style = Text::new(16.0 * s).color(ctx.theme.subtext).center();
        ctx.text_mid(g, "Appuyez pour commencer", w as f32 / 2.0, h as f32 / 2.0, &info_style);
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        if let Event::Tap { .. } = ev {
            ctx.toast("Multimètre démarré !");
        }
    }

    fn on_back(&mut self, ctx: &Ctx) -> bool {
        // Default behavior: exit the app
        false
    }
}

fn main() {
    run(Config::new("Multimètre"), MultimetreApp);
}
