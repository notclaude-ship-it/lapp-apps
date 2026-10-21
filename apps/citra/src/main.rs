// Citra (Azahar) : lanceur lapp pour l'émulateur 3DS.
//
// - Bibliothèque de ROMs + explorateur de fichiers pour en ajouter
// - Réglages : rendu, disposition des écrans, touches clavier / manette
// - Lance /app/bin/azahar (frontend SDL2) avec un sdl2-config.ini généré depuis les réglages
//
// Capture des boutons de manette : /app/bin/joycap (petit programme SDL2) écrit sur stdout les
// événements "B guid port idx", "H guid port hat dir", "A guid port axe signe".

use lapp_sdk::prelude::*;
use lapp_sdk::{app_path, arg_value, data_dir};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};

const ROM_EXT: &[&str] = &["3ds", "cci", "cxi", "cia", "3dsx", "app", "elf", "axf"];

// (libellé, clé du fichier ini)
const BUTTONS: &[(&str, &str)] = &[
    ("Bouton A", "button_a"),
    ("Bouton B", "button_b"),
    ("Bouton X", "button_x"),
    ("Bouton Y", "button_y"),
    ("Gâchette L", "button_l"),
    ("Gâchette R", "button_r"),
    ("ZL (New 3DS)", "button_zl"),
    ("ZR (New 3DS)", "button_zr"),
    ("Start", "button_start"),
    ("Select", "button_select"),
    ("Home", "button_home"),
    ("Croix : haut", "button_up"),
    ("Croix : bas", "button_down"),
    ("Croix : gauche", "button_left"),
    ("Croix : droite", "button_right"),
];
const STICKS: &[(&str, &str)] = &[("Circle Pad", "circle_pad"), ("C-Stick", "c_stick")];

const LAYOUTS: &[(i64, &str)] = &[(0, "Haut / bas"), (1, "Écran unique"), (2, "Grand + petit"), (3, "Côte à côte"), (5, "Hybride")];
const RESOLUTIONS: &[(i64, &str)] = &[(1, "×1 (native)"), (2, "×2"), (3, "×3")];
const SPEEDS: &[(i64, &str)] = &[(100, "100 %"), (150, "150 %"), (200, "200 %"), (0, "Illimitée")];

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Library,
    Explorer,
    Settings,
    Controls,
    Log,
}

#[derive(Clone, Debug, PartialEq)]
enum Bind {
    Key(u32),
    Btn { guid: String, port: u32, idx: u32 },
    Hat { guid: String, port: u32, hat: u32, dir: String },
    Axis { guid: String, port: u32, axis: u32, dir: String },
}

fn key_name(code: u32) -> String {
    match code {
        4..=29 => ((b'A' + (code - 4) as u8) as char).to_string(),
        30..=38 => ((b'1' + (code - 30) as u8) as char).to_string(),
        39 => "0".into(),
        40 => "Entrée".into(),
        44 => "Espace".into(),
        79 => "→".into(),
        80 => "←".into(),
        81 => "↓".into(),
        82 => "↑".into(),
        n => format!("#{n}"),
    }
}

fn key_code(k: Key) -> Option<u32> {
    match k {
        Key::Char(c) => {
            let c = c.to_ascii_lowercase();
            match c {
                'a'..='z' => Some(4 + (c as u32 - 'a' as u32)),
                '1'..='9' => Some(30 + (c as u32 - '1' as u32)),
                '0' => Some(39),
                _ => None,
            }
        }
        Key::Enter => Some(40),
        Key::Space => Some(44),
        Key::Right => Some(79),
        Key::Left => Some(80),
        Key::Down => Some(81),
        Key::Up => Some(82),
        Key::Back => None,
    }
}

fn esc(s: &str) -> String {
    s.replace('$', "$2").replace(':', "$0").replace(',', "$1")
}

impl Bind {
    fn to_store(&self) -> String {
        match self {
            Bind::Key(c) => format!("kb|{c}"),
            Bind::Btn { guid, port, idx } => format!("jb|{guid}|{port}|{idx}"),
            Bind::Hat { guid, port, hat, dir } => format!("jh|{guid}|{port}|{hat}|{dir}"),
            Bind::Axis { guid, port, axis, dir } => format!("ja|{guid}|{port}|{axis}|{dir}"),
        }
    }
    fn from_store(s: &str) -> Option<Bind> {
        let p: Vec<&str> = s.split('|').collect();
        let n = |i: usize| p.get(i).and_then(|x| x.parse::<u32>().ok());
        match p.first().copied()? {
            "kb" => Some(Bind::Key(n(1)?)),
            "jb" => Some(Bind::Btn { guid: p.get(1)?.to_string(), port: n(2)?, idx: n(3)? }),
            "jh" => Some(Bind::Hat { guid: p.get(1)?.to_string(), port: n(2)?, hat: n(3)?, dir: p.get(4)?.to_string() }),
            "ja" => Some(Bind::Axis { guid: p.get(1)?.to_string(), port: n(2)?, axis: n(3)?, dir: p.get(4)?.to_string() }),
            _ => None,
        }
    }
    fn to_param(&self) -> String {
        match self {
            Bind::Key(c) => format!("engine:keyboard,code:{c}"),
            Bind::Btn { guid, port, idx } => format!("engine:sdl,guid:{guid},port:{port},button:{idx}"),
            Bind::Hat { guid, port, hat, dir } => format!("engine:sdl,guid:{guid},port:{port},hat:{hat},direction:{dir}"),
            Bind::Axis { guid, port, axis, dir } => {
                format!("engine:sdl,guid:{guid},port:{port},axis:{axis},direction:{dir},threshold:0.5")
            }
        }
    }
    fn label(&self) -> String {
        match self {
            Bind::Key(c) => format!("Clavier {}", key_name(*c)),
            Bind::Btn { idx, .. } => format!("Manette bouton {idx}"),
            Bind::Hat { dir, .. } => format!("Manette croix {dir}"),
            Bind::Axis { axis, dir, .. } => format!("Manette axe {axis}{dir}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Stick {
    Axes { guid: String, port: u32, x: u32, y: u32 },
    Keys([u32; 4]), // haut, bas, gauche, droite
}

impl Stick {
    fn to_store(&self) -> String {
        match self {
            Stick::Axes { guid, port, x, y } => format!("sa|{guid}|{port}|{x}|{y}"),
            Stick::Keys(k) => format!("sk|{}|{}|{}|{}", k[0], k[1], k[2], k[3]),
        }
    }
    fn from_store(s: &str) -> Option<Stick> {
        let p: Vec<&str> = s.split('|').collect();
        let n = |i: usize| p.get(i).and_then(|x| x.parse::<u32>().ok());
        match p.first().copied()? {
            "sa" => Some(Stick::Axes { guid: p.get(1)?.to_string(), port: n(2)?, x: n(3)?, y: n(4)? }),
            "sk" => Some(Stick::Keys([n(1)?, n(2)?, n(3)?, n(4)?])),
            _ => None,
        }
    }
    fn to_param(&self) -> String {
        match self {
            Stick::Axes { guid, port, x, y } => format!("engine:sdl,guid:{guid},port:{port},axis_x:{x},axis_y:{y}"),
            Stick::Keys(k) => {
                let kb = |c: u32| esc(&format!("engine:keyboard,code:{c}"));
                format!(
                    "engine:analog_from_button,up:{},down:{},left:{},right:{},modifier:{},modifier_scale:0.5",
                    kb(k[0]),
                    kb(k[1]),
                    kb(k[2]),
                    kb(k[3]),
                    kb(0)
                )
            }
        }
    }
    fn label(&self) -> String {
        match self {
            Stick::Axes { x, y, .. } => format!("Manette axes {x}/{y}"),
            Stick::Keys(k) => format!("Clavier {}{}{}{}", key_name(k[0]), key_name(k[1]), key_name(k[2]), key_name(k[3])),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Target {
    Button(usize),
    Stick(usize),
}

struct Capture {
    target: Target,
    keys: Vec<u32>,
    axes: Vec<(String, u32, u32)>,
}

#[derive(Clone)]
enum Action {
    Back,
    OpenExplorer,
    OpenSettings,
    OpenControls,
    OpenLog,
    Launch(usize),
    Remove(usize),
    ExplorerUp,
    ExplorerGo(PathBuf),
    ExplorerPick(PathBuf),
    BindStart(Target),
    ClearBind(Target),
    ResetBinds,
    CaptureCancel,
    Cycle(&'static str),
    Toggle(&'static str),
    StopEmu,
}

struct Joy {
    child: Child,
    rx: Receiver<String>,
}

impl Drop for Joy {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct Running {
    child: Child,
    name: String,
    started: Instant,
}

struct CitraApp {
    store: Store,
    page: Page,
    scroll: Scroller,
    hits: Vec<(Rect, Action)>,
    down: Option<(f32, f32)>,
    roms: Vec<PathBuf>,
    dir: PathBuf,
    entries: Vec<(String, PathBuf, bool)>,
    capture: Option<Capture>,
    joy: Option<Joy>,
    running: Option<Running>,
    log_lines: Vec<String>,
}

fn config_dir() -> PathBuf {
    data_dir().join("config").join("azahar-emu")
}

fn log_path() -> PathBuf {
    data_dir().join("azahar.log")
}

impl CitraApp {
    fn new() -> CitraApp {
        let store = Store::open("state.conf");
        let roms = store
            .get("roms")
            .unwrap_or("")
            .lines()
            .filter(|l| !l.is_empty())
            .map(PathBuf::from)
            .collect();
        let start = store
            .get("last_dir")
            .map(PathBuf::from)
            .filter(|p| p.is_dir())
            .or_else(|| ["/host/home", "/host"].iter().map(PathBuf::from).find(|p| p.is_dir()))
            .unwrap_or_else(|| PathBuf::from("/"));
        let mut a = CitraApp {
            store,
            page: Page::Library,
            scroll: Scroller::new(),
            hits: vec![],
            down: None,
            roms,
            dir: start,
            entries: vec![],
            capture: None,
            joy: None,
            running: None,
            log_lines: vec![],
        };
        a.refresh_dir();
        match arg_value("--page").as_deref() {
            Some("explorer") => a.page = Page::Explorer,
            Some("settings") => a.page = Page::Settings,
            Some("controls") => a.page = Page::Controls,
            _ => {}
        }
        a
    }

    fn geti(&self, k: &str, d: i64) -> i64 {
        self.store.get_i64(k).unwrap_or(d)
    }

    fn save_roms(&mut self) {
        let s = self.roms.iter().map(|p| p.to_string_lossy().into_owned()).collect::<Vec<_>>().join("\n");
        let _ = self.store.set("roms", s);
    }

    fn refresh_dir(&mut self) {
        let mut v: Vec<(String, PathBuf, bool)> = vec![];
        if let Ok(rd) = fs::read_dir(&self.dir) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.starts_with('.') {
                    continue;
                }
                let p = e.path();
                if p.is_dir() {
                    v.push((name, p, true));
                } else {
                    let ext = p.extension().map(|x| x.to_string_lossy().to_lowercase()).unwrap_or_default();
                    if ROM_EXT.contains(&ext.as_str()) {
                        v.push((name, p, false));
                    }
                }
            }
        }
        v.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.to_lowercase().cmp(&b.0.to_lowercase())));
        self.entries = v;
    }

    fn goto(&mut self, p: Page) {
        self.page = p;
        self.scroll = Scroller::new();
        if p == Page::Log {
            self.log_lines = fs::read_to_string(log_path())
                .unwrap_or_else(|_| "(journal vide)".into())
                .lines()
                .rev()
                .take(60)
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
        }
    }

    fn bind_of(&self, ini: &str) -> Option<Bind> {
        self.store.get(&format!("bind.{ini}")).and_then(Bind::from_store)
    }
    fn stick_of(&self, ini: &str) -> Option<Stick> {
        self.store.get(&format!("bind.{ini}")).and_then(Stick::from_store)
    }

    fn target_key(t: Target) -> &'static str {
        match t {
            Target::Button(i) => BUTTONS[i].1,
            Target::Stick(i) => STICKS[i].1,
        }
    }

    // --- configuration de l'émulateur ---
    fn write_config(&self) -> std::io::Result<()> {
        let mut s = String::from("[Controls]\n");
        for (_, ini) in BUTTONS {
            if let Some(b) = self.bind_of(ini) {
                s += &format!("{ini}={}\n", b.to_param());
            }
        }
        for (_, ini) in STICKS {
            if let Some(b) = self.stick_of(ini) {
                s += &format!("{ini}={}\n", b.to_param());
            }
        }
        s += &format!("\n[Core]\nuse_cpu_jit={}\n", self.geti("set.jit", 1));
        s += &format!(
            "\n[Renderer]\ngraphics_api=1\nuse_gles=0\nuse_vsync={}\nuse_disk_shader_cache={}\nresolution_factor={}\nframe_limit={}\n",
            self.geti("set.vsync", 1),
            self.geti("set.shadercache", 1),
            self.geti("set.res", 1),
            self.geti("set.speed", 100)
        );
        s += &format!("\n[Layout]\nlayout_option={}\nswap_screen={}\n", self.geti("set.layout", 0), self.geti("set.swap", 0));
        let dir = config_dir();
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("sdl2-config.ini"), s)
    }

    fn launch(&mut self, ctx: &Ctx, idx: usize) {
        if self.running.is_some() {
            return;
        }
        let Some(path) = self.roms.get(idx).cloned() else { return };
        if !path.exists() {
            ctx.toast("ROM introuvable (déplacée ?)");
            return;
        }
        if let Err(e) = self.write_config() {
            ctx.toast(format!("Config impossible : {e}"));
            return;
        }
        self.joy = None;
        let log = fs::File::create(log_path()).ok();
        let (out, err) = match log.and_then(|f| f.try_clone().ok().map(|g| (f, g))) {
            Some((a, b)) => (Stdio::from(a), Stdio::from(b)),
            None => (Stdio::null(), Stdio::null()),
        };
        let bin = app_path("bin/azahar");
        let res = Command::new(&bin)
            .arg("-f")
            .arg(&path)
            .env("LD_LIBRARY_PATH", app_path("lib"))
            .env("XDG_CONFIG_HOME", data_dir().join("config"))
            .env("XDG_DATA_HOME", data_dir().join("share"))
            .env("XDG_CACHE_HOME", data_dir().join("cache"))
            .env("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", "1")
            .stdout(out)
            .stderr(err)
            .spawn();
        match res {
            Ok(child) => {
                let name = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                self.running = Some(Running { child, name, started: Instant::now() });
                // la ROM jouée passe en tête de la bibliothèque
                let p = self.roms.remove(idx);
                self.roms.insert(0, p);
                self.save_roms();
            }
            Err(e) => ctx.toast(format!("Lancement impossible : {e}")),
        }
    }

    // --- capture de touches / manette ---
    fn start_capture(&mut self, t: Target) {
        self.capture = Some(Capture { target: t, keys: vec![], axes: vec![] });
        self.joy = None;
        let bin = app_path("bin/joycap");
        if let Ok(mut child) = Command::new(bin).arg("wait").stdout(Stdio::piped()).stderr(Stdio::null()).spawn() {
            if let Some(out) = child.stdout.take() {
                let (tx, rx) = channel();
                std::thread::spawn(move || {
                    for l in BufReader::new(out).lines().map_while(Result::ok) {
                        if tx.send(l).is_err() {
                            break;
                        }
                    }
                });
                self.joy = Some(Joy { child, rx });
            }
        }
    }

    fn end_capture(&mut self) {
        self.capture = None;
        self.joy = None;
    }

    fn set_bind(&mut self, t: Target, value: Option<String>) {
        let key = format!("bind.{}", Self::target_key(t));
        match value {
            Some(v) => {
                let _ = self.store.set(&key, v);
            }
            None => {
                let _ = self.store.remove(&key);
            }
        }
    }

    fn capture_key(&mut self, ctx: &Ctx, code: u32) {
        let Some(cap) = self.capture.as_mut() else { return };
        match cap.target {
            Target::Button(_) => {
                let t = cap.target;
                self.set_bind(t, Some(Bind::Key(code).to_store()));
                self.end_capture();
                ctx.toast(format!("Touche {}", key_name(code)));
            }
            Target::Stick(_) => {
                cap.keys.push(code);
                if cap.keys.len() == 4 {
                    let k = [cap.keys[0], cap.keys[1], cap.keys[2], cap.keys[3]];
                    let t = cap.target;
                    self.set_bind(t, Some(Stick::Keys(k).to_store()));
                    self.end_capture();
                    ctx.toast("Stick enregistré");
                }
            }
        }
    }

    fn capture_joy(&mut self, ctx: &Ctx, line: &str) {
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() < 4 {
            return;
        }
        let guid = p[1].to_string();
        let port: u32 = p[2].parse().unwrap_or(0);
        let Some(cap) = self.capture.as_mut() else { return };
        match (cap.target, p[0]) {
            (Target::Button(_), "B") => {
                let b = Bind::Btn { guid, port, idx: p[3].parse().unwrap_or(0) };
                let t = cap.target;
                self.set_bind(t, Some(b.to_store()));
                self.end_capture();
                ctx.toast("Bouton enregistré");
            }
            (Target::Button(_), "H") if p.len() >= 5 => {
                let b = Bind::Hat { guid, port, hat: p[3].parse().unwrap_or(0), dir: p[4].to_string() };
                let t = cap.target;
                self.set_bind(t, Some(b.to_store()));
                self.end_capture();
                ctx.toast("Croix enregistrée");
            }
            (Target::Button(_), "A") if p.len() >= 5 => {
                let b = Bind::Axis { guid, port, axis: p[3].parse().unwrap_or(0), dir: p[4].to_string() };
                let t = cap.target;
                self.set_bind(t, Some(b.to_store()));
                self.end_capture();
                ctx.toast("Axe enregistré");
            }
            (Target::Stick(_), "A") => {
                let axis: u32 = p[3].parse().unwrap_or(0);
                if cap.axes.iter().any(|a| a.2 == axis) {
                    return;
                }
                cap.axes.push((guid, port, axis));
                if cap.axes.len() == 2 {
                    let s = Stick::Axes { guid: cap.axes[0].0.clone(), port: cap.axes[0].1, x: cap.axes[0].2, y: cap.axes[1].2 };
                    let t = cap.target;
                    self.set_bind(t, Some(s.to_store()));
                    self.end_capture();
                    ctx.toast("Stick enregistré");
                }
            }
            _ => {}
        }
    }

    fn cycle(&mut self, key: &'static str) {
        let (list, def): (Vec<i64>, i64) = match key {
            "set.layout" => (LAYOUTS.iter().map(|x| x.0).collect(), 0),
            "set.res" => (RESOLUTIONS.iter().map(|x| x.0).collect(), 1),
            "set.speed" => (SPEEDS.iter().map(|x| x.0).collect(), 100),
            _ => return,
        };
        let cur = self.geti(key, def);
        let i = list.iter().position(|v| *v == cur).unwrap_or(0);
        let _ = self.store.set(key, list[(i + 1) % list.len()]);
    }

    fn act(&mut self, ctx: &Ctx, a: Action) {
        match a {
            Action::Back => match self.page {
                Page::Controls | Page::Log => self.goto(Page::Settings),
                _ => self.goto(Page::Library),
            },
            Action::OpenExplorer => {
                self.refresh_dir();
                self.goto(Page::Explorer);
            }
            Action::OpenSettings => self.goto(Page::Settings),
            Action::OpenControls => self.goto(Page::Controls),
            Action::OpenLog => self.goto(Page::Log),
            Action::Launch(i) => self.launch(ctx, i),
            Action::Remove(i) => {
                if i < self.roms.len() {
                    self.roms.remove(i);
                    self.save_roms();
                }
            }
            Action::ExplorerUp => {
                if let Some(p) = self.dir.parent() {
                    self.dir = p.to_path_buf();
                    let _ = self.store.set("last_dir", self.dir.to_string_lossy());
                    self.refresh_dir();
                    self.scroll = Scroller::new();
                }
            }
            Action::ExplorerGo(p) => {
                self.dir = p;
                let _ = self.store.set("last_dir", self.dir.to_string_lossy());
                self.refresh_dir();
                self.scroll = Scroller::new();
            }
            Action::ExplorerPick(p) => {
                if !self.roms.contains(&p) {
                    self.roms.insert(0, p.clone());
                    self.save_roms();
                }
                ctx.toast(format!("Ajoutée : {}", p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()));
                self.goto(Page::Library);
            }
            Action::BindStart(t) => self.start_capture(t),
            Action::ClearBind(t) => {
                self.set_bind(t, None);
                self.end_capture();
                ctx.toast("Valeur par défaut");
            }
            Action::ResetBinds => {
                for (_, k) in BUTTONS.iter().chain(STICKS.iter()) {
                    let _ = self.store.remove(&format!("bind.{k}"));
                }
                ctx.toast("Touches réinitialisées");
            }
            Action::CaptureCancel => self.end_capture(),
            Action::Cycle(k) => self.cycle(k),
            Action::Toggle(k) => {
                let cur = self.geti(k, 1);
                let _ = self.store.set(k, 1 - cur);
            }
            Action::StopEmu => {
                if let Some(r) = self.running.as_mut() {
                    let _ = r.child.kill();
                }
            }
        }
    }

    // --- dessin ---
    fn is_pressed(&self, r: Rect) -> bool {
        self.down.map(|(x, y)| r.contains(x, y)).unwrap_or(false)
    }

    fn row(&mut self, ctx: &Ctx, g: &mut Gfx, y: f32, title: &str, detail: Option<&str>, a: Action) -> f32 {
        let s = ctx.scale();
        let h = 52.0 * s;
        let r = Rect::new(0.0, y, ctx.width(), h);
        let pressed = self.is_pressed(r);
        ui::list_row(ctx, g, r, title, detail, pressed);
        self.hits.push((r, a));
        y + h
    }

    fn section(&self, ctx: &Ctx, g: &mut Gfx, y: f32, text: &str) -> f32 {
        let s = ctx.scale();
        let st = Text::new(12.0 * s).bold().color(ctx.theme.subtext).left();
        ctx.text_mid(g, &text.to_uppercase(), 14.0 * s, y + 16.0 * s, &st);
        y + 30.0 * s
    }

    fn toggle_row(&mut self, ctx: &Ctx, g: &mut Gfx, y: f32, title: &str, key: &'static str, def: i64) -> f32 {
        let s = ctx.scale();
        let on = self.geti(key, def) != 0;
        let ny = self.row(ctx, g, y, title, None, Action::Toggle(key));
        let r = Rect::new(ctx.width() - 62.0 * s, y + 13.0 * s, 48.0 * s, 26.0 * s);
        ui::toggle(ctx, g, r, on);
        ny
    }

    fn bottom_buttons(&mut self, ctx: &Ctx, g: &mut Gfx, left: (&str, Action), right: (&str, Action)) -> f32 {
        let s = ctx.scale();
        let h = 64.0 * s;
        let y = ctx.height() - h;
        let t = ctx.theme;
        g.rect(0.0, y, ctx.width(), h, t.surface);
        g.rect(0.0, y, ctx.width(), 1.0, t.line);
        let bw = (ctx.width() - 36.0 * s) / 2.0;
        let r1 = Rect::new(12.0 * s, y + 10.0 * s, bw, 44.0 * s);
        let r2 = Rect::new(24.0 * s + bw, y + 10.0 * s, bw, 44.0 * s);
        let p1 = self.is_pressed(r1);
        let p2 = self.is_pressed(r2);
        ui::button(ctx, g, r1, left.0, ButtonStyle::Primary, p1);
        ui::button(ctx, g, r2, right.0, ButtonStyle::Secondary, p2);
        self.hits.push((r1, left.1));
        self.hits.push((r2, right.1));
        y
    }

    fn centered(&self, ctx: &Ctx, g: &mut Gfx, y: f32, text: &str, size: f32, bold: bool, color: Color) {
        let s = ctx.scale();
        let mut st = Text::new(size * s).color(color).center().max_width(ctx.width() - 32.0 * s);
        if bold {
            st = st.bold();
        }
        ctx.text_mid(g, text, ctx.width() / 2.0, y, &st);
    }

    fn render_running(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = ctx.theme;
        let s = ctx.scale();
        g.clear(t.bg);
        let head = ui::header(ctx, g, "Citra", None);
        let name = self.running.as_ref().map(|r| r.name.clone()).unwrap_or_default();
        self.centered(ctx, g, head.height + 80.0 * s, "Émulation en cours", 18.0, true, t.text);
        self.centered(ctx, g, head.height + 112.0 * s, &name, 14.0, false, t.subtext);
        let r = Rect::new(40.0 * s, head.height + 160.0 * s, ctx.width() - 80.0 * s, 48.0 * s);
        let p = self.is_pressed(r);
        ui::button(ctx, g, r, "Arrêter", ButtonStyle::Danger, p);
        self.hits.push((r, Action::StopEmu));
    }

    fn render_library(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = ctx.theme;
        let s = ctx.scale();
        let head_h = 44.0 * s;
        let bottom = ctx.height() - 64.0 * s;
        let mut y = head_h - self.scroll.pos;
        if self.roms.is_empty() {
            self.centered(ctx, g, head_h + 90.0 * s, "Aucune ROM", 18.0, true, t.text);
            self.centered(ctx, g, head_h + 120.0 * s, "Ajoutez-en avec l'explorateur de fichiers", 13.0, false, t.subtext);
        }
        let roms = self.roms.clone();
        for (i, p) in roms.iter().enumerate() {
            let name = p.file_stem().map(|x| x.to_string_lossy().into_owned()).unwrap_or_default();
            let dir = p.parent().map(|x| x.to_string_lossy().into_owned()).unwrap_or_default();
            let h = 56.0 * s;
            let r = Rect::new(0.0, y, ctx.width(), h);
            let pressed = self.is_pressed(r);
            g.rect(r.x, r.y, r.w, r.h, if pressed { t.press } else { t.surface });
            g.rect(r.x + 12.0 * s, r.bottom() - 1.0, r.w - 12.0 * s, 1.0, t.line);
            let st = Text::new(16.0 * s).color(t.text).left().max_width(r.w - 70.0 * s);
            ctx.text_mid(g, &name, 12.0 * s, y + 20.0 * s, &st);
            let st2 = Text::new(11.0 * s).color(t.subtext).left().ellipsis_left().max_width(r.w - 70.0 * s);
            ctx.text_mid(g, &dir, 12.0 * s, y + 40.0 * s, &st2);
            let xr = Rect::new(r.right() - 52.0 * s, y, 52.0 * s, h);
            let stx = Text::new(20.0 * s).color(t.danger).center();
            ctx.text_mid(g, "×", xr.center().0, xr.center().1, &stx);
            self.hits.push((xr, Action::Remove(i)));
            self.hits.push((Rect::new(0.0, y, r.w - 52.0 * s, h), Action::Launch(i)));
            y += h;
        }
        self.scroll.set_content(y + self.scroll.pos - head_h, bottom - head_h);
        ui::header(ctx, g, "Citra", None);
        self.bottom_buttons(ctx, g, ("Ajouter une ROM", Action::OpenExplorer), ("Réglages", Action::OpenSettings));
    }

    fn render_explorer(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = ctx.theme;
        let s = ctx.scale();
        let head_h = 44.0 * s;
        let path_h = 30.0 * s;
        let mut y = head_h + path_h - self.scroll.pos;
        if self.dir.parent().is_some() {
            y = self.row(ctx, g, y, "‹ Dossier parent", None, Action::ExplorerUp);
        }
        let entries = self.entries.clone();
        if entries.is_empty() {
            self.centered(ctx, g, y + 40.0 * s, "Aucun dossier ni ROM ici", 14.0, false, t.subtext);
            y += 80.0 * s;
        }
        for (name, p, is_dir) in entries {
            y = if is_dir {
                self.row(ctx, g, y, &format!("📁 {name}"), None, Action::ExplorerGo(p))
            } else {
                self.row(ctx, g, y, &name, Some("Ajouter"), Action::ExplorerPick(p))
            };
        }
        self.scroll.set_content(y + self.scroll.pos - head_h - path_h, ctx.height() - head_h - path_h);
        g.rect(0.0, head_h, ctx.width(), path_h, t.bg);
        let st = Text::new(11.0 * s).color(t.subtext).left().ellipsis_left().max_width(ctx.width() - 24.0 * s);
        ctx.text_mid(g, &self.dir.to_string_lossy(), 12.0 * s, head_h + path_h / 2.0, &st);
        let hd = ui::header(ctx, g, "Choisir une ROM", Some("Retour"));
        if let Some(b) = hd.back {
            self.hits.push((b, Action::Back));
        }
    }

    fn render_settings(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let head_h = 44.0 * ctx.scale();
        let mut y = head_h - self.scroll.pos;
        y = self.section(ctx, g, y, "Manette et clavier");
        y = self.row(ctx, g, y, "Touches et manette", Some("›"), Action::OpenControls);
        y = self.section(ctx, g, y, "Affichage");
        let l = LAYOUTS.iter().find(|x| x.0 == self.geti("set.layout", 0)).map(|x| x.1).unwrap_or("?");
        y = self.row(ctx, g, y, "Disposition des écrans", Some(l), Action::Cycle("set.layout"));
        y = self.toggle_row(ctx, g, y, "Échanger les écrans", "set.swap", 0);
        let r = RESOLUTIONS.iter().find(|x| x.0 == self.geti("set.res", 1)).map(|x| x.1).unwrap_or("?");
        y = self.row(ctx, g, y, "Résolution interne", Some(r), Action::Cycle("set.res"));
        y = self.toggle_row(ctx, g, y, "Synchro verticale", "set.vsync", 1);
        y = self.section(ctx, g, y, "Performances");
        let sp = SPEEDS.iter().find(|x| x.0 == self.geti("set.speed", 100)).map(|x| x.1).unwrap_or("?");
        y = self.row(ctx, g, y, "Limite de vitesse", Some(sp), Action::Cycle("set.speed"));
        y = self.toggle_row(ctx, g, y, "Compilateur JIT", "set.jit", 1);
        y = self.toggle_row(ctx, g, y, "Cache de shaders", "set.shadercache", 1);
        y = self.section(ctx, g, y, "Diagnostic");
        y = self.row(ctx, g, y, "Journal de l'émulateur", Some("›"), Action::OpenLog);
        self.scroll.set_content(y + self.scroll.pos - head_h, ctx.height() - head_h);
        let hd = ui::header(ctx, g, "Réglages", Some("Retour"));
        if let Some(b) = hd.back {
            self.hits.push((b, Action::Back));
        }
    }

    fn render_controls(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let head_h = 44.0 * ctx.scale();
        let mut y = head_h - self.scroll.pos;
        y = self.section(ctx, g, y, "Appuyez sur une ligne pour la modifier");
        for (i, (label, ini)) in BUTTONS.iter().enumerate() {
            let d = self.bind_of(ini).map(|b| b.label()).unwrap_or_else(|| "Par défaut".into());
            y = self.row(ctx, g, y, label, Some(&d), Action::BindStart(Target::Button(i)));
        }
        y = self.section(ctx, g, y, "Sticks");
        for (i, (label, ini)) in STICKS.iter().enumerate() {
            let d = self.stick_of(ini).map(|b| b.label()).unwrap_or_else(|| "Par défaut".into());
            y = self.row(ctx, g, y, label, Some(&d), Action::BindStart(Target::Stick(i)));
        }
        y += 12.0 * ctx.scale();
        let s = ctx.scale();
        let r = Rect::new(24.0 * s, y, ctx.width() - 48.0 * s, 44.0 * s);
        let p = self.is_pressed(r);
        ui::button(ctx, g, r, "Tout réinitialiser", ButtonStyle::Danger, p);
        self.hits.push((r, Action::ResetBinds));
        y += 60.0 * s;
        self.scroll.set_content(y + self.scroll.pos - head_h, ctx.height() - head_h);
        let hd = ui::header(ctx, g, "Touches et manette", Some("Retour"));
        if let Some(b) = hd.back {
            self.hits.push((b, Action::Back));
        }
    }

    fn render_log(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = ctx.theme;
        let s = ctx.scale();
        let head_h = 44.0 * s;
        let st = Text::new(10.0 * s).mono().color(t.text).left().max_width(ctx.width() - 16.0 * s);
        let lh = 13.0 * s;
        let mut y = head_h + 8.0 * s - self.scroll.pos;
        let lines = self.log_lines.clone();
        for l in &lines {
            ctx.text_mid(g, l, 8.0 * s, y + lh / 2.0, &st);
            y += lh;
        }
        self.scroll.set_content(y + self.scroll.pos - head_h, ctx.height() - head_h);
        let hd = ui::header(ctx, g, "Journal", Some("Retour"));
        if let Some(b) = hd.back {
            self.hits.push((b, Action::Back));
        }
    }

    fn render_capture(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let t = ctx.theme;
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        g.rect(0.0, 0.0, w, h, rgba(0x000000, 0.55));
        let card = Rect::new(16.0 * s, h * 0.2, w - 32.0 * s, 250.0 * s);
        g.rrect(card.x, card.y, card.w, card.h, 14.0 * s, t.surface);
        let cap = self.capture.as_ref().unwrap();
        let (name, prompt, sub) = match cap.target {
            Target::Button(i) => (
                BUTTONS[i].0.to_string(),
                "Appuyez sur une touche du clavier ou un bouton de la manette…".to_string(),
                String::new(),
            ),
            Target::Stick(i) => {
                let keyn = cap.keys.len();
                let axn = cap.axes.len();
                let (p, sub) = if keyn > 0 {
                    let names = ["HAUT", "BAS", "GAUCHE", "DROITE"];
                    (format!("Touche pour {}", names[keyn.min(3)]), format!("{keyn}/4"))
                } else if axn == 1 {
                    ("Poussez le stick vers le HAUT".to_string(), "2/2".to_string())
                } else {
                    ("Poussez le stick vers la DROITE (ou appuyez sur la touche pour HAUT au clavier)".to_string(), "1/2".to_string())
                };
                (STICKS[i].0.to_string(), p, sub)
            }
        };
        let cx = w / 2.0;
        let st = Text::new(18.0 * s).bold().color(t.text).center().max_width(card.w - 24.0 * s);
        ctx.text_mid(g, &name, cx, card.y + 28.0 * s, &st);
        let st = Text::new(14.0 * s).color(t.subtext).center().max_width(card.w - 32.0 * s);
        ctx.text_mid(g, &prompt, cx, card.y + 84.0 * s, &st);
        if !sub.is_empty() {
            ctx.text_mid(g, &sub, cx, card.y + 120.0 * s, &st);
        }
        let target = cap.target;
        let r1 = Rect::new(card.x + 14.0 * s, card.bottom() - 62.0 * s, (card.w - 38.0 * s) / 2.0, 44.0 * s);
        let r2 = Rect::new(r1.right() + 10.0 * s, r1.y, r1.w, r1.h);
        let p1 = self.is_pressed(r1);
        let p2 = self.is_pressed(r2);
        ui::button(ctx, g, r1, "Annuler", ButtonStyle::Secondary, p1);
        ui::button(ctx, g, r2, "Par défaut", ButtonStyle::Danger, p2);
        self.hits.clear();
        self.hits.push((r1, Action::CaptureCancel));
        self.hits.push((r2, Action::ClearBind(target)));
    }
}

impl App for CitraApp {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        g.clear(ctx.theme.bg);
        self.hits.clear();
        if self.running.is_some() {
            self.render_running(ctx, g);
            return;
        }
        match self.page {
            Page::Library => self.render_library(ctx, g),
            Page::Explorer => self.render_explorer(ctx, g),
            Page::Settings => self.render_settings(ctx, g),
            Page::Controls => self.render_controls(ctx, g),
            Page::Log => self.render_log(ctx, g),
        }
        if self.capture.is_some() {
            self.render_capture(ctx, g);
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        match ev {
            Event::Down { x, y } => {
                self.down = Some((x, y));
                self.scroll.stop();
            }
            Event::Up { .. } => self.down = None,
            Event::Move { .. } => {}
            Event::Drag { dy, .. } | Event::Wheel { dy } => {
                self.down = None;
                if self.capture.is_none() && self.running.is_none() {
                    self.scroll.drag(dy);
                }
            }
            Event::DragEnd { vy, .. } => {
                if self.capture.is_none() && self.running.is_none() {
                    self.scroll.fling(vy);
                }
            }
            Event::Tap { x, y } => {
                let hit = self.hits.iter().rev().find(|(r, _)| r.contains(x, y)).map(|(_, a)| a.clone());
                if let Some(a) = hit {
                    self.act(ctx, a);
                }
            }
            Event::Key(k) => {
                if self.capture.is_some() {
                    if let Some(code) = key_code(k) {
                        self.capture_key(ctx, code);
                    }
                }
            }
        }
    }

    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        if self.scroll.update(dt) {
            ctx.animate();
        }
        // événements de manette pendant une capture
        let mut lines = vec![];
        if let Some(j) = &self.joy {
            while let Ok(l) = j.rx.try_recv() {
                lines.push(l);
            }
        }
        for l in lines {
            self.capture_joy(ctx, &l);
            ctx.animate();
        }
        if self.capture.is_some() {
            ctx.redraw_in(Duration::from_millis(150));
        }
        // émulateur en cours
        let mut finished = None;
        if let Some(r) = self.running.as_mut() {
            if let Ok(Some(st)) = r.child.try_wait() {
                finished = Some((st.code().unwrap_or(-1), r.started.elapsed()));
            }
        }
        if let Some((code, dur)) = finished {
            self.running = None;
            if code != 0 && dur < Duration::from_secs(20) {
                ctx.toast(format!("L'émulateur s'est arrêté (code {code}) : voir le journal"));
                self.goto(Page::Log);
            } else {
                ctx.toast("Émulateur fermé");
            }
            ctx.animate();
        } else if self.running.is_some() {
            ctx.redraw_in(Duration::from_millis(500));
        }
    }

    fn on_back(&mut self, _ctx: &Ctx) -> bool {
        if self.running.is_some() {
            return true;
        }
        if self.capture.is_some() {
            self.end_capture();
            return true;
        }
        match self.page {
            Page::Library => false,
            Page::Controls | Page::Log => {
                self.goto(Page::Settings);
                true
            }
            _ => {
                self.goto(Page::Library);
                true
            }
        }
    }

    fn icon(&mut self, ctx: &Ctx, g: &mut Gfx, size: u32) {
        let sz = size as f32;
        g.clear(rgb(0xD0312D));
        g.rrect(sz * 0.14, sz * 0.30, sz * 0.72, sz * 0.40, sz * 0.06, rgba(0xFFFFFF, 0.95));
        let st = Text::new(sz * 0.26).bold().color(rgb(0xD0312D)).center();
        ctx.text_mid(g, "3DS", sz / 2.0, sz * 0.5, &st);
    }
}

fn main() {
    let _ = Path::new("/");
    run(Config::new("Citra").gpu(true), CitraApp::new());
}