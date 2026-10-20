use lapp_sdk::prelude::*;
use std::collections::VecDeque;
use std::process::Command;

const MAX_OUTPUT_LINES: usize = 1000; // Limite la sortie pour éviter une consommation excessive de mémoire

struct SimpleTerminal {
    input_buffer: String,
    output_lines: VecDeque<String>,
    scroller: Scroller,
    current_dir: String, // Simule un répertoire de travail
}

impl SimpleTerminal {
    fn new() -> Self {
        SimpleTerminal {
            input_buffer: String::new(),
            output_lines: VecDeque::new(),
            scroller: Scroller::new(),
            current_dir: lapp_sdk::sys::data_dir().to_string_lossy().into_owned(),
        }
    }

    fn add_output(&mut self, line: String) {
        self.output_lines.push_back(line);
        if self.output_lines.len() > MAX_OUTPUT_LINES {
            self.output_lines.pop_front();
        }
        // Défilement vers le bas sur nouvelle sortie
        self.scroller.jump_to(self.scroller.max);
    }

    fn execute_command(&mut self, _ctx: &Ctx, command_line: &str) {
        self.add_output(format!("$ {}", command_line));

        // Gestion basique de la commande "cd"
        if command_line.trim().starts_with("cd ") {
            let new_dir = command_line.trim_start_matches("cd ").trim();
            let path = std::path::PathBuf::from(&self.current_dir).join(new_dir);
            if path.is_dir() {
                self.current_dir = path.canonicalize().unwrap_or(path).to_string_lossy().into_owned();
                self.add_output(format!("Répertoire actuel : {}", self.current_dir));
            } else {
                self.add_output(format!("cd: {}: Aucun répertoire de ce type", new_dir));
            }
            return;
        }

        let output = Command::new("bash")
            .arg("-c")
            .arg(command_line)
            .current_dir(&self.current_dir) // Utilise le répertoire de travail simulé
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                for line in stdout.lines() {
                    self.add_output(line.to_string());
                }
                for line in stderr.lines() {
                    self.add_output(format!("ERR: {}", line));
                }
                if !output.status.success() {
                    self.add_output(format!("La commande s'est terminée avec le statut : {:?}", output.status));
                }
            }
            Err(e) => {
                self.add_output(format!("Échec de l'exécution de la commande : {}", e));
            }
        }
    }
}

impl App for SimpleTerminal {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let s = ctx.scale();
        let t = ctx.theme;
        let (w, h) = ctx.size();
        let (w, h) = (w as f32, h as f32);

        g.clear(t.bg);

        // En-tête
        let header_h = 44.0 * s;
        ui::header(ctx, g, "Simple Terminal", None);

        // Zone de sortie
        let output_y = header_h;
        let output_h = h - header_h - (30.0 * s); // Espace pour le champ de saisie
        let output_rect = Rect::new(0.0, output_y, w, output_h);
        g.rect(output_rect.x, output_rect.y, output_rect.w, output_rect.h, t.surface);

        let text_style = Text::new(12.0 * s).color(t.text).left();
        let line_height = ctx.line_height(&text_style);
        let padding = 8.0 * s;

        let mut current_y = padding - self.scroller.pos;
        for line in self.output_lines.iter() {
            if current_y + line_height > padding && current_y < output_h - padding {
                ctx.text(g, line, output_rect.x + padding, output_rect.y + current_y, &text_style);
            }
            current_y += line_height;
        }

        // Calcule la hauteur du contenu pour le défileur
        let content_height = self.output_lines.len() as f32 * line_height + 2.0 * padding;
        self.scroller.set_content(content_height, output_h);

        // Zone de saisie
        let input_y = h - (30.0 * s);
        let input_rect = Rect::new(0.0, input_y, w, 30.0 * s);
        g.rect(input_rect.x, input_rect.y, input_rect.w, input_rect.h, t.surface);
        g.rect(input_rect.x, input_rect.y, input_rect.w, 1.0, t.line); // Bordure supérieure pour la saisie

        let prompt_style = Text::new(12.0 * s).color(t.accent).left();
        let input_text_style = Text::new(12.0 * s).color(t.text).left();

        let prompt = format!("{}> ", self.current_dir);
        let prompt_width = ctx.measure(&prompt, &prompt_style);

        ctx.text(g, &prompt, input_rect.x + padding, input_rect.y + (input_rect.h - line_height) / 2.0, &prompt_style);
        ctx.text(g, &self.input_buffer, input_rect.x + padding + prompt_width, input_rect.y + (input_rect.h - line_height) / 2.0, &input_text_style);
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        match ev {
            Event::Key(Key::Char(c)) => {
                self.input_buffer.push(c);
                ctx.animate();
            }
            Event::Key(Key::Back) => {
                self.input_buffer.pop();
                ctx.animate();
            }
            Event::Key(Key::Enter) => {
                let command = self.input_buffer.trim().to_string();
                self.input_buffer.clear();
                if !command.is_empty() {
                    self.execute_command(ctx, &command);
                }
                ctx.animate();
            }
            Event::Drag { dy, .. } | Event::Wheel { dy } => {
                self.scroller.drag(dy);
                ctx.animate();
            }
            Event::DragEnd { vy, .. } => {
                self.scroller.fling(vy);
                ctx.animate();
            }
            Event::Down { .. } => {
                self.scroller.stop();
                ctx.animate();
            }
            _ => {}
        }
    }

    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        if self.scroller.update(dt) {
            ctx.animate();
        }
    }

    fn on_back(&mut self, _ctx: &Ctx) -> bool {
        // Si une saisie est en cours, l'effacer. Sinon, quitter.
        if !self.input_buffer.is_empty() {
            self.input_buffer.clear();
            true
        } else {
            false
        }
    }
}

fn main() {
    let mut app = SimpleTerminal::new();
    app.add_output("Bienvenue dans Simple Terminal !".to_string());
    app.add_output("Tapez 'ls', 'pwd', 'echo hello', ou 'cd /' et appuyez sur Entrée.".to_string());
    app.add_output("Utilisez 'cd ..' pour remonter. 'cd /app' pour voir les fichiers de l'app.".to_string());
    app.add_output("Note : Ceci est un exécuteur de commandes basique, pas un TTY interactif complet.".to_string());
    app.add_output("Il exécute les commandes via 'bash -c \"commande\"'.".to_string());
    app.add_output(format!("Répertoire actuel : {}", lapp_sdk::sys::data_dir().to_string_lossy()));

    run(Config::new("Simple Terminal").edge_exit(false), app);
}
