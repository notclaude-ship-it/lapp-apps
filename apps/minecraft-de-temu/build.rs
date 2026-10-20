use std::path::Path;
use lapp_sdk::{Config, App, Ctx, Gfx, icon_png};

// A dummy app implementation for icon generation
struct IconGeneratorApp;

impl App for IconGeneratorApp {
    fn render(&mut self, _ctx: &Ctx, _g: &mut Gfx) {
        // No rendering needed for the dummy app
    }
    // The default icon method will be used, which draws a tile with the app's initial.
}

fn main() {
    let icon_path = Path::new("icon.png");

    // Only generate if the icon file does not exist
    if !icon_path.exists() {
        let config = Config::new("Temu Minecraft"); // Use the app's config for icon generation
        let mut app = IconGeneratorApp;
        if let Err(e) = icon_png(&mut app, &config, icon_path.to_str().unwrap()) {
            eprintln!("Failed to generate icon.png: {}", e);
        } else {
            println!("Generated default icon.png");
        }
    }

    // Rerun build script if build.rs or Cargo.toml changes
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
}
