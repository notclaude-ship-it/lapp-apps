use lapp_sdk::prelude::*;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq)]
enum MultimeterMode {
    Voltage,    // Volts (V)
    Current,    // Amperes (A)
    Resistance, // Ohms (Ω)
}

impl MultimeterMode {
    fn to_string(&self) -> &'static str {
        match self {
            MultimeterMode::Voltage => "Voltage",
            MultimeterMode::Current => "Current",
            MultimeterMode::Resistance => "Resistance",
        }
    }

    fn unit(&self) -> &'static str {
        match self {
            MultimeterMode::Voltage => "V",
            MultimeterMode::Current => "A",
            MultimeterMode::Resistance => "Ω",
        }
    }

    fn from_string(s: &str) -> Option<Self> {
        match s {
            "Voltage" => Some(MultimeterMode::Voltage),
            "Current" => Some(MultimeterMode::Current),
            "Resistance" => Some(MultimeterMode::Resistance),
            _ => None,
        }
    }
}

struct MultimetreApp {
    mode: MultimeterMode,
    current_value: f32,
    last_update_time: Instant,
    store: Store,
    // For button press visual feedback
    pressed_button_rect: Option<Rect>,
    // Simple PRNG seed
    rng_seed: u64,
}

impl MultimetreApp {
    fn new() -> Self {
        let mut store = Store::open("multimetre_settings");
        let mode_str = store.get_or("mode", "Voltage");
        let mode = MultimeterMode::from_string(&mode_str).unwrap_or(MultimeterMode::Voltage);

        MultimetreApp {
            mode,
            current_value: 0.0,
            last_update_time: Instant::now(),
            store,
            pressed_button_rect: None,
            rng_seed: Instant::now().elapsed().as_nanos() as u64, // Initialize seed
        }
    }

    // A very simple pseudo-random number generator (LCG)
    fn next_rand(&mut self) -> f32 {
        let a = 1103515245;
        let c = 12345;
        let m = 2u64.pow(31); // Modulo for 31-bit numbers
        self.rng_seed = (self.rng_seed.wrapping_mul(a).wrapping_add(c)) % m;
        (self.rng_seed as f32) / (m as f32)
    }

    fn simulate_value(&mut self, dt: f32) {
        // Simple simulation: value oscillates and has some noise
        let base_oscillation = (self.last_update_time.elapsed().as_secs_f32() * 2.0).sin() * 0.5 + 0.5; // 0 to 1
        let noise = (self.next_rand() - 0.5) * 0.1; // -0.05 to 0.05

        let (min, max, factor) = match self.mode {
            MultimeterMode::Voltage => (0.0, 12.0, 10.0), // 0-12V
            MultimeterMode::Current => (0.0, 2.0, 1.0),   // 0-2A
            MultimeterMode::Resistance => (0.0, 1000.0, 100.0), // 0-1000Ω
        };

        self.current_value = (min + base_oscillation * (max - min) + noise * factor).clamp(min, max);
    }
}

impl App for MultimetreApp {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        g.clear(ctx.theme.bg);
        let (w, h) = ctx.size();
        let s = ctx.scale();

        // Header
        let header_h = 44.0 * s;
        ui::header(ctx, g, "Multimètre", None);

        // Value display area
        let display_rect = Rect::new(0.0, header_h, w as f32, h as f32 - header_h - 80.0 * s); // Leave space for buttons

        let value_text = format!("{:.2} {}", self.current_value, self.mode.unit());
        let value_style = Text::new(48.0 * s).bold().color(ctx.theme.text).center();
        ctx.text_mid(g, &value_text, display_rect.center().0, display_rect.center().1, &value_style);

        // Mode buttons at the bottom
        let button_h = 48.0 * s;
        let button_spacing = 10.0 * s;
        let total_button_width = (w as f32 - 4.0 * button_spacing) / 3.0; // 3 buttons, 4 spacings

        let y_buttons = h as f32 - button_h - button_spacing;

        let modes = [MultimeterMode::Voltage, MultimeterMode::Current, MultimeterMode::Resistance];
        for (i, &mode) in modes.iter().enumerate() {
            let x = button_spacing + i as f32 * (total_button_width + button_spacing);
            let r = Rect::new(x, y_buttons, total_button_width, button_h);
            let is_pressed = self.pressed_button_rect.map_or(false, |pr| pr == r);
            let style = if mode == self.mode { ButtonStyle::Primary } else { ButtonStyle::Secondary };
            ui::button(ctx, g, r, mode.to_string(), style, is_pressed);
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        let (w, h) = ctx.size();
        let s = ctx.scale();
        let button_h = 48.0 * s;
        let button_spacing = 10.0 * s;
        let total_button_width = (w as f32 - 4.0 * button_spacing) / 3.0;
        let y_buttons = h as f32 - button_h - button_spacing;

        let modes = [MultimeterMode::Voltage, MultimeterMode::Current, MultimeterMode::Resistance];

        match ev {
            Event::Down { x, y } => {
                for (i, _) in modes.iter().enumerate() {
                    let btn_x = button_spacing + i as f32 * (total_button_width + button_spacing);
                    let r = Rect::new(btn_x, y_buttons, total_button_width, button_h);
                    if r.contains(x, y) {
                        self.pressed_button_rect = Some(r);
                        break;
                    }
                }
            }
            Event::Up { .. } => {
                self.pressed_button_rect = None;
            }
            Event::Tap { x, y } => {
                for (i, &mode) in modes.iter().enumerate() {
                    let btn_x = button_spacing + i as f32 * (total_button_width + button_spacing);
                    let r = Rect::new(btn_x, y_buttons, total_button_width, button_h);
                    if r.contains(x, y) {
                        if self.mode != mode {
                            self.mode = mode;
                            let _ = self.store.set("mode", mode.to_string());
                            ctx.toast(format!("Mode: {}", mode.to_string()));
                        }
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        self.simulate_value(dt);
        self.last_update_time = Instant::now();
        ctx.animate(); // Keep redrawing for the simulation
    }

    fn on_back(&mut self, _ctx: &Ctx) -> bool {
        // Default behavior: exit the app
        false
    }
}

fn main() {
    run(Config::new("Multimètre").size(272, 480), MultimetreApp::new());
}
