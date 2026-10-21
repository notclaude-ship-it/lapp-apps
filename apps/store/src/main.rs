// Store : client « app store » pour lapp.
//
// Liste les .lapp hébergés sur le serveur, affiche le détail, télécharge puis installe via `lapp install`.
//
// Serveur : --server hote:port, sinon clé « server » de state.conf, sinon DEFAULT_SERVER.
// Sur PC :  cargo run -- --demo --screenshot out.png     (aperçu avec de fausses données)
//           cargo run -- --demo --page com.moi.notes --screenshot out.png   (écran de détail)
//           cargo run -- --server 127.0.0.1:10078 --screenshot out.png [--page ID] [--auto-install ID]
//                                                        (test réel : réseau, téléchargement, `lapp install`)
//           cargo run -- --icon icon.png                 (icône)

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use lapp_sdk::prelude::*;
use lapp_sdk::{arg_flag, arg_value, data_dir, lapp_bin};

const DEFAULT_SERVER: &str = "atalo.duckdns.org:10078";

// ---------------------------------------------------------------------------------------------
// Données partagées avec les threads réseau
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Entry {
    file: String,
    id: String,
    name: String,
    version: String,
    size: u64,
    desc: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
enum Job {
    #[default]
    Idle,
    Loading,
    Download { done: u64, total: u64 },
    Installing,
}

#[derive(Default)]
struct Shared {
    apps: Vec<Entry>,
    job: Job,
    /// Message à afficher (pris par `render`).
    toast: Option<String>,
    /// (id, version) venant d'être installée (prise par `render` pour l'enregistrer).
    installed: Option<(String, String)>,
    /// Paquet téléchargé dont le signataire n'est pas approuvé : en attente de confirmation de l'utilisateur.
    pending: Option<Pending>,
}

/// Installation en attente de confirmation (signature valide mais clé inconnue de l'appareil).
#[derive(Clone, Debug)]
struct Pending {
    id: String,
    name: String,
    version: String,
    /// Fichier .lapp déjà téléchargé (supprimé si l'utilisateur annule).
    dest: std::path::PathBuf,
    /// Empreinte de la clé publique du signataire (hexadécimal), si lapp l'a indiquée.
    key: String,
}

type Sh = Arc<Mutex<Shared>>;

fn finish(sh: &Sh, installed: Option<(String, String)>, toast: String) {
    let mut s = sh.lock().unwrap();
    s.job = Job::Idle;
    s.installed = installed;
    s.toast = Some(toast);
}

// ---------------------------------------------------------------------------------------------
// Client HTTP minimal (HTTP/1.0, sans TLS : le serveur parle en clair)
// ---------------------------------------------------------------------------------------------

/// GET `path` sur `server` (« hote:port »). `on` reçoit chaque bloc du corps et la taille annoncée
/// (0 si inconnue) ; renvoyer `false` interrompt la lecture.
fn http_get(server: &str, path: &str, mut on: impl FnMut(&[u8], u64) -> bool) -> Result<(), String> {
    let addr = server
        .to_socket_addrs()
        .map_err(|e| format!("DNS : {e}"))?
        .next()
        .ok_or("hôte introuvable")?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(8)).map_err(|e| format!("connexion : {e}"))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(15)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
    let host = server.split(':').next().unwrap_or(server);
    let req = format!("GET {path} HTTP/1.0\r\nHost: {host}\r\nUser-Agent: lapp-store/1.0\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).map_err(|e| format!("envoi : {e}"))?;

    let mut r = BufReader::new(stream);
    let mut line = String::new();
    r.read_line(&mut line).map_err(|e| format!("lecture : {e}"))?;
    let status: u16 = line.split_whitespace().nth(1).and_then(|c| c.parse().ok()).ok_or("réponse invalide")?;

    let mut total = 0u64;
    loop {
        line.clear();
        let n = r.read_line(&mut line).map_err(|e| format!("lecture : {e}"))?;
        if n == 0 || line.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-length") {
                total = v.trim().parse().unwrap_or(0);
            }
        }
    }
    if status != 200 {
        return Err(format!("HTTP {status}"));
    }
    let mut buf = [0u8; 16 * 1024];
    loop {
        let n = r.read(&mut buf).map_err(|e| format!("lecture : {e}"))?;
        if n == 0 || !on(&buf[..n], total) {
            break;
        }
    }
    Ok(())
}

/// Lignes TSV : fichier \t id \t nom \t version \t taille \t description
fn parse_catalog(text: &str) -> Vec<Entry> {
    text.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            if f.len() < 6 {
                return None;
            }
            Some(Entry {
                file: f[0].to_string(),
                id: f[1].to_string(),
                name: f[2].to_string(),
                version: f[3].to_string(),
                size: f[4].parse().unwrap_or(0),
                desc: f[5].to_string(),
            })
        })
        .collect()
}

fn spawn_refresh(server: String, sh: Sh) {
    {
        let mut s = sh.lock().unwrap();
        if s.job != Job::Idle {
            return;
        }
        s.job = Job::Loading;
    }
    thread::spawn(move || {
        let mut body: Vec<u8> = Vec::new();
        eprintln!("store : chargement du catalogue depuis {server}");
        let res = http_get(&server, "/api/apps.tsv", |c, _| {
            body.extend_from_slice(c);
            body.len() < 1 << 20
        });
        let mut s = sh.lock().unwrap();
        s.job = Job::Idle;
        match res {
            Ok(()) => {
                s.apps = parse_catalog(&String::from_utf8_lossy(&body));
                eprintln!("store : catalogue chargé depuis {server} ({} apps)", s.apps.len());
            }
            Err(e) => {
                eprintln!("store : ERREUR catalogue ({server}) : {e}");
                s.toast = Some(format!("Erreur : {e}"));
            }
        }
    });
}

fn download(server: &str, e: &Entry, sh: &Sh, part: &Path) -> Result<(), String> {
    let mut f = fs::File::create(part).map_err(|x| format!("fichier : {x}"))?;
    let mut done: u64 = 0;
    let mut write_failed = false;
    http_get(server, &format!("/files/{}", e.file), |c, total| {
        if f.write_all(c).is_err() {
            write_failed = true;
            return false;
        }
        done += c.len() as u64;
        if let Ok(mut s) = sh.lock() {
            s.job = Job::Download { done, total: if total > 0 { total } else { e.size } };
        }
        true
    })?;
    if write_failed {
        return Err("écriture impossible".to_string());
    }
    if done == 0 || (e.size > 0 && done != e.size) {
        return Err("téléchargement incomplet".to_string());
    }
    f.flush().map_err(|x| format!("fichier : {x}"))
}

fn spawn_install(server: String, e: Entry, sh: Sh) {
    {
        let mut s = sh.lock().unwrap();
        if s.job != Job::Idle {
            return;
        }
        s.job = Job::Download { done: 0, total: e.size };
    }
    thread::spawn(move || {
        let dir = data_dir().join("downloads");
        let _ = fs::create_dir_all(&dir);
        let part = dir.join(format!("{}.part", e.file));
        let dest = dir.join(&e.file);

        eprintln!("store : téléchargement de {} depuis {server}", e.file);
        let res = download(&server, &e, &sh, &part).and_then(|_| fs::rename(&part, &dest).map_err(|x| format!("renommage : {x}")));
        if let Err(m) = res {
            eprintln!("store : ERREUR téléchargement de {} depuis {server} : {m}", e.file);
            let _ = fs::remove_file(&part);
            finish(&sh, None, format!("Erreur : {m}"));
            return;
        }

        sh.lock().unwrap().job = Job::Installing;
        run_install(&sh, &e.id, &e.name, &e.version, &dest, false);
    });
}

/// Lance `lapp install` sur `dest`. Si le signataire n'est pas approuvé (et que `allow_untrusted` est faux),
/// le fichier est conservé et une confirmation est demandée à l'utilisateur (`Shared::pending`).
fn run_install(sh: &Sh, id: &str, name: &str, version: &str, dest: &Path, allow_untrusted: bool) {
    let mut cmd = Command::new(lapp_bin());
    cmd.arg("install").arg(dest);
    if allow_untrusted {
        cmd.arg("--allow-untrusted");
    }
    match cmd.output() {
        Ok(o) if o.status.success() => {
            eprintln!("store : {id} {version} installée{}", if allow_untrusted { " (signataire non approuvé, confirmé par l'utilisateur)" } else { "" });
            finish(sh, Some((id.to_string(), version.to_string())), format!("{name} installée"));
        }
        Ok(o) => {
            let raw = String::from_utf8_lossy(if o.stderr.is_empty() { o.stdout.as_slice() } else { o.stderr.as_slice() }).to_string();
            // lapp : « signataire non approuvé (clé <hex>) : ajoute sa .pub dans etc/lapp/trusted/ ou utilise --allow-untrusted »
            // (signature valide mais clé inconnue). Une signature invalide, elle, reste refusée sans appel.
            if !allow_untrusted && raw.contains("non approuvé") {
                let key = raw.split_once("(clé ").and_then(|(_, r)| r.split_once(')')).map(|(k, _)| k.trim().to_string()).unwrap_or_default();
                eprintln!("store : signataire non approuvé pour {id} {version} (clé {key}) : confirmation demandée");
                let mut s = sh.lock().unwrap();
                s.job = Job::Idle;
                s.pending = Some(Pending { id: id.to_string(), name: name.to_string(), version: version.to_string(), dest: dest.to_path_buf(), key });
                return; // le fichier est conservé pour une éventuelle installation confirmée
            }
            eprintln!("store : ERREUR installation de {id} ({}) :\n{}", o.status, raw.trim_end());
            let line: String = raw.lines().find(|l| !l.trim().is_empty()).unwrap_or("échec").chars().take(70).collect();
            finish(sh, None, format!("Échec : {line}"));
        }
        Err(err) => {
            eprintln!("store : ERREUR lancement de {} install : {err}", lapp_bin().display());
            finish(sh, None, format!("lapp : {err}"));
        }
    }
    let _ = fs::remove_file(dest);
}

/// L'utilisateur a confirmé : réinstalle le paquet en attente en acceptant le signataire inconnu.
fn spawn_trusted_install(sh: Sh) {
    let p = {
        let mut s = sh.lock().unwrap();
        if s.job != Job::Idle {
            return;
        }
        let Some(p) = s.pending.take() else { return };
        s.job = Job::Installing;
        p
    };
    eprintln!("store : installation de {} {} confirmée malgré le signataire non approuvé (clé {})", p.id, p.version, p.key);
    thread::spawn(move || run_install(&sh, &p.id, &p.name, &p.version, &p.dest, true));
}

/// L'utilisateur a refusé : supprime le paquet téléchargé.
fn cancel_pending(sh: &Sh) {
    let mut s = sh.lock().unwrap();
    if let Some(p) = s.pending.take() {
        let _ = fs::remove_file(&p.dest);
        eprintln!("store : installation de {} {} refusée (signataire non approuvé)", p.id, p.version);
        s.toast = Some("Installation annulée".to_string());
    }
}

// ---------------------------------------------------------------------------------------------
// Interface
// ---------------------------------------------------------------------------------------------

fn human(n: u64) -> String {
    let f = n as f64;
    if f < 1024.0 {
        format!("{n} o")
    } else if f < 1024.0 * 1024.0 {
        format!("{:.0} Ko", f / 1024.0)
    } else {
        format!("{:.1} Mo", f / (1024.0 * 1024.0))
    }
}

/// Coupe `s` en lignes de largeur <= `max` (px).
fn wrap(ctx: &Ctx, s: &str, st: &Text, max: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for para in s.split('\n') {
        let mut cur = String::new();
        for word in para.split_whitespace() {
            let cand = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
            if cur.is_empty() || ctx.measure(&cand, st) <= max {
                cur = cand;
            } else {
                lines.push(std::mem::take(&mut cur));
                cur = word.to_string();
            }
        }
        lines.push(cur);
    }
    lines
}

/// Apps réellement installées sur l'appareil : (id -> version) lue dans `<apps>/<id>/manifest.toml`.
/// `<apps>` = `$LAPP_APPS`, sinon `/apps` (permission `install`). `None` si ce dossier n'est pas accessible.
fn read_installed() -> Option<HashMap<String, String>> {
    let dir = std::env::var("LAPP_APPS").unwrap_or_else(|_| "/apps".to_string());
    let rd = fs::read_dir(&dir).ok()?;
    let mut map = HashMap::new();
    for e in rd.flatten() {
        let Ok(t) = fs::read_to_string(e.path().join("manifest.toml")) else { continue };
        let Some(ver) = t.lines().find_map(|l| {
            let l = l.trim();
            let r = l.strip_prefix("version")?.trim_start().strip_prefix('=')?.trim();
            Some(r.trim_matches('"').to_string())
        }) else {
            continue;
        };
        map.insert(e.file_name().to_string_lossy().to_string(), ver);
    }
    Some(map)
}

fn version_tuple(s: &str) -> Vec<u64> {
    s.trim().split('.').map(|p| p.parse().unwrap_or(0)).collect()
}

#[derive(Clone, Debug, PartialEq)]
enum Act {
    Open(String),
    Back,
    Refresh,
    Install(String),
    TrustYes,
    TrustNo,
}

#[derive(Clone, Debug)]
enum Screen {
    List,
    Detail(String),
}

struct StoreApp {
    sh: Sh,
    server: String,
    store: Store,
    screen: Screen,
    scroll: Scroller,
    hits: Vec<(Rect, Act)>,
    pressed: Option<usize>,
    head_h: f32,
    /// Apps installées sur l'appareil (relu au plus une fois par seconde) ; `None` si /apps est inaccessible.
    inst: Option<HashMap<String, String>>,
    inst_at: Option<Instant>,
}

impl StoreApp {
    fn refresh_installed(&mut self) {
        if self.inst_at.map(|t| t.elapsed() < Duration::from_secs(1)).unwrap_or(false) {
            return;
        }
        self.inst = read_installed();
        self.inst_at = Some(Instant::now());
    }

    /// Version installée : d'après les apps présentes sur l'appareil (l'id suffit), sinon d'après ce que le store a lui-même installé.
    fn installed_version(&self, id: &str) -> Option<String> {
        match &self.inst {
            Some(m) => m.get(id).cloned(),
            None => self.store.get(&format!("installed.{id}")).map(|v| v.to_string()),
        }
    }

    /// Libellé d'état dans la liste : version proposée, « Installée » ou « Mise à jour ».
    fn status_label(&self, a: &Entry) -> String {
        match self.installed_version(&a.id) {
            None => format!("v{}", a.version),
            Some(v) if version_tuple(&v) >= version_tuple(&a.version) => "Installée".to_string(),
            Some(_) => "Mise à jour".to_string(),
        }
    }

    fn hit_at(&self, x: f32, y: f32) -> Option<usize> {
        // Dernier ajouté = dessiné au-dessus (l'en-tête passe avant les lignes).
        (0..self.hits.len()).rev().find(|&i| {
            let (r, act) = &self.hits[i];
            r.contains(x, y) && (y >= self.head_h || *act == Act::Back)
        })
    }

    fn run(&mut self, ctx: &Ctx, act: Act) {
        match act {
            Act::Open(id) => {
                self.screen = Screen::Detail(id);
                self.scroll = Scroller::new();
            }
            Act::Back => {
                self.screen = Screen::List;
                self.scroll = Scroller::new();
            }
            Act::Refresh => spawn_refresh(self.server.clone(), self.sh.clone()),
            Act::TrustYes => spawn_trusted_install(self.sh.clone()),
            Act::TrustNo => cancel_pending(&self.sh),
            Act::Install(id) => {
                let entry = self.sh.lock().unwrap().apps.iter().find(|a| a.id == id).cloned();
                if let Some(e) = entry {
                    spawn_install(self.server.clone(), e, self.sh.clone());
                }
            }
        }
        ctx.animate();
    }

    fn draw_list(&mut self, ctx: &Ctx, g: &mut Gfx, apps: &[Entry], job: &Job) {
        let t = ctx.theme;
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let head_h = 44.0 * s;
        self.head_h = head_h;
        let row_h = 56.0 * s;
        let btn_h = 40.0 * s;

        let content_h = apps.len() as f32 * row_h + 16.0 * s + btn_h + 14.0 * s + 16.0 * s + h * 0.1;
        self.scroll.set_content(content_h, h - head_h);
        let mut y = head_h - self.scroll.pos;

        for a in apps {
            let r = Rect::new(0.0, y, w, row_h);
            y += row_h;
            if r.bottom() < head_h || r.y > h {
                continue;
            }
            let label = self.status_label(a);
            let idx = self.hits.len();
            self.hits.push((r, Act::Open(a.id.clone())));
            ui::list_row(ctx, g, r, &a.name, Some(&label), self.pressed == Some(idx));
        }

        if apps.is_empty() {
            let st = Text::new(14.0 * s).color(t.subtext).center();
            let msg = if *job == Job::Loading { "Chargement…" } else { "Aucune application" };
            ctx.text_mid(g, msg, w / 2.0, head_h + 60.0 * s, &st);
            y = head_h + 90.0 * s;
        }

        y += 16.0 * s;
        let bw = w * 0.6;
        let br = Rect::new((w - bw) / 2.0, y, bw, btn_h);
        if br.bottom() > head_h && br.y < h {
            let idx = self.hits.len();
            self.hits.push((br, Act::Refresh));
            let label = if *job == Job::Loading { "Actualisation…" } else { "Actualiser" };
            ui::button(ctx, g, br, label, ButtonStyle::Secondary, self.pressed == Some(idx));
        }
        let st = Text::new(11.0 * s).color(t.subtext).center().max_width(w - 24.0 * s);
        ctx.text_mid(g, &self.server, w / 2.0, br.bottom() + 16.0 * s, &st);

        let head = ui::header(ctx, g, "Store", None);
        self.head_h = head.height;
    }

    fn draw_detail(&mut self, ctx: &Ctx, g: &mut Gfx, apps: &[Entry], job: &Job, id: &str) {
        let t = ctx.theme;
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        let head_h = 44.0 * s;
        self.head_h = head_h;

        let Some(a) = apps.iter().find(|a| a.id == id) else {
            let st = Text::new(14.0 * s).color(t.subtext).center();
            ctx.text_mid(g, "Application introuvable", w / 2.0, head_h + 60.0 * s, &st);
            let head = ui::header(ctx, g, "Store", Some("Retour"));
            if let Some(b) = head.back {
                self.hits.push((b, Act::Back));
            }
            return;
        };

        let installed = self.installed_version(&a.id);
        let btn_h = 46.0 * s;
        let btn = Rect::new(24.0 * s, h - h * 0.075 - btn_h, w - 48.0 * s, btn_h);
        let area_bottom = btn.y - 22.0 * s;

        // Mise en page (calculée d'abord pour borner le défilement).
        let name_st = Text::new(22.0 * s).bold().color(t.text).center().max_width(w - 32.0 * s);
        let id_st = Text::new(11.0 * s).mono().color(t.subtext).center().max_width(w - 32.0 * s);
        let meta_st = Text::new(14.0 * s).color(t.text).center();
        let inst_st = Text::new(13.0 * s).color(t.subtext).center();
        let desc_st = Text::new(14.0 * s).color(t.text).left();
        let desc = if a.desc.is_empty() { "Aucune description." } else { a.desc.as_str() };
        let lines = wrap(ctx, desc, &desc_st, w - 40.0 * s);
        let desc_lh = ctx.line_height(&desc_st) + 3.0 * s;

        let mut content_h = 22.0 * s;
        content_h += ctx.line_height(&name_st) + 4.0 * s;
        content_h += ctx.line_height(&id_st) + 14.0 * s;
        content_h += ctx.line_height(&meta_st) + 4.0 * s;
        if installed.is_some() {
            content_h += ctx.line_height(&inst_st) + 4.0 * s;
        }
        content_h += 16.0 * s + lines.len() as f32 * desc_lh + 12.0 * s;
        self.scroll.set_content(content_h, area_bottom - head_h);

        let mut y = head_h + 22.0 * s - self.scroll.pos;
        ctx.text(g, &a.name, w / 2.0, y, &name_st);
        y += ctx.line_height(&name_st) + 4.0 * s;
        ctx.text(g, &a.id, w / 2.0, y, &id_st);
        y += ctx.line_height(&id_st) + 14.0 * s;
        ctx.text(g, &format!("Version {} · {}", a.version, human(a.size)), w / 2.0, y, &meta_st);
        y += ctx.line_height(&meta_st) + 4.0 * s;
        if let Some(v) = &installed {
            ctx.text(g, &format!("Installée : v{v}"), w / 2.0, y, &inst_st);
            y += ctx.line_height(&inst_st) + 4.0 * s;
        }
        y += 16.0 * s;
        for l in &lines {
            ctx.text(g, l, 20.0 * s, y, &desc_st);
            y += desc_lh;
        }

        // Zone du bas : masque le texte qui dépasse, puis progression et bouton.
        g.rect(0.0, area_bottom, w, h - area_bottom, t.bg);

        let (label, enabled) = match job {
            Job::Idle => {
                let l = match installed.as_deref() {
                    None => "Installer",
                    Some(v) if version_tuple(v) >= version_tuple(&a.version) => "Réinstaller",
                    Some(_) => "Mettre à jour",
                };
                (l.to_string(), true)
            }
            Job::Loading => ("Actualisation…".to_string(), false),
            Job::Download { done, total } => {
                let frac = if *total > 0 { (*done as f32 / *total as f32).clamp(0.0, 1.0) } else { 0.0 };
                let bar = Rect::new(btn.x, btn.y - 14.0 * s, btn.w, 6.0 * s);
                g.rrect(bar.x, bar.y, bar.w, bar.h, bar.h / 2.0, t.line);
                if frac > 0.0 {
                    g.rrect(bar.x, bar.y, (bar.w * frac).max(bar.h), bar.h, bar.h / 2.0, t.accent);
                }
                (format!("Téléchargement {}%", (frac * 100.0) as u32), false)
            }
            Job::Installing => ("Installation…".to_string(), false),
        };
        let idx = self.hits.len();
        if enabled {
            self.hits.push((btn, Act::Install(a.id.clone())));
        }
        let style = if enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary };
        ui::button(ctx, g, btn, &label, style, enabled && self.pressed == Some(idx));

        let head = ui::header(ctx, g, &a.name, Some("Retour"));
        if let Some(b) = head.back {
            self.hits.push((b, Act::Back));
        }
    }
}

impl StoreApp {
    /// Dialogue : le signataire du paquet n'est pas approuvé par l'appareil.
    fn draw_trust_modal(&mut self, ctx: &Ctx, g: &mut Gfx, p: &Pending) {
        let t = ctx.theme;
        let s = ctx.scale();
        let (w, h) = (ctx.width(), ctx.height());
        g.rect(0.0, 0.0, w, h, rgba(0x000000, 0.45));

        let cw = w - 36.0 * s;
        let inner = cw - 32.0 * s;
        let title_st = Text::new(16.0 * s).bold().color(t.text).center().max_width(cw - 24.0 * s);
        let body_st = Text::new(13.0 * s).color(t.text).center();
        let key_st = Text::new(11.0 * s).mono().color(t.subtext).center().max_width(inner);
        let warn_st = Text::new(12.0 * s).color(t.subtext).center();

        let intro = format!("« {} » v{} n'est pas signée par une clé approuvée sur cet appareil.", p.name, p.version);
        let intro_l = wrap(ctx, &intro, &body_st, inner);
        let warn_l = wrap(ctx, "N'installez que des apps dont vous connaissez la source. L'installer quand même ?", &warn_st, inner);
        let key_txt = if p.key.is_empty() { String::new() } else { format!("clé {}…", p.key.chars().take(16).collect::<String>()) };

        let (title_lh, body_lh, key_lh, warn_lh) =
            (ctx.line_height(&title_st), ctx.line_height(&body_st) + 2.0 * s, ctx.line_height(&key_st), ctx.line_height(&warn_st) + 2.0 * s);
        let btn_h = 42.0 * s;
        let mut ch = 18.0 * s + title_lh + 10.0 * s + intro_l.len() as f32 * body_lh + 6.0 * s;
        if !key_txt.is_empty() {
            ch += key_lh + 8.0 * s;
        }
        ch += warn_l.len() as f32 * warn_lh + 16.0 * s + btn_h + 16.0 * s;
        let (cx, cy) = (18.0 * s, ((h - ch) / 2.0).max(10.0 * s));

        g.rrect(cx, cy, cw, ch, 14.0 * s, t.line);
        g.rrect(cx + 1.0, cy + 1.0, cw - 2.0, ch - 2.0, 13.0 * s, t.surface);

        let mx = cx + cw / 2.0;
        let mut y = cy + 18.0 * s;
        ctx.text(g, "Signataire non approuvé", mx, y, &title_st);
        y += title_lh + 10.0 * s;
        for l in &intro_l {
            ctx.text(g, l, mx, y, &body_st);
            y += body_lh;
        }
        y += 6.0 * s;
        if !key_txt.is_empty() {
            ctx.text(g, &key_txt, mx, y, &key_st);
            y += key_lh + 8.0 * s;
        }
        for l in &warn_l {
            ctx.text(g, l, mx, y, &warn_st);
            y += warn_lh;
        }
        y += 16.0 * s;

        let bw = (cw - 48.0 * s) / 2.0;
        let no = Rect::new(cx + 16.0 * s, y, bw, btn_h);
        let yes = Rect::new(cx + 32.0 * s + bw, y, bw, btn_h);
        // Seuls les deux boutons restent actifs tant que le dialogue est ouvert.
        self.hits.clear();
        self.head_h = 0.0;
        let i_no = self.hits.len();
        self.hits.push((no, Act::TrustNo));
        let i_yes = self.hits.len();
        self.hits.push((yes, Act::TrustYes));
        ui::button(ctx, g, no, "Annuler", ButtonStyle::Secondary, self.pressed == Some(i_no));
        ui::button(ctx, g, yes, "Installer", ButtonStyle::Danger, self.pressed == Some(i_yes));
    }
}

impl App for StoreApp {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let (apps, job, toast, done, pending) = {
            let mut sh = self.sh.lock().unwrap();
            (sh.apps.clone(), sh.job.clone(), sh.toast.take(), sh.installed.take(), sh.pending.clone())
        };
        if let Some(m) = toast {
            ctx.toast(m);
        }
        if let Some((id, ver)) = done {
            let _ = self.store.set(&format!("installed.{id}"), ver);
        }
        if job != Job::Idle {
            ctx.redraw_in(Duration::from_millis(120));
        }

        self.refresh_installed();
        if job != Job::Idle {
            self.inst_at = None; // pendant une installation : relecture à chaque image
        }
        self.hits.clear();
        g.clear(ctx.theme.bg);
        match self.screen.clone() {
            Screen::List => self.draw_list(ctx, g, &apps, &job),
            Screen::Detail(id) => self.draw_detail(ctx, g, &apps, &job, &id),
        }
        if let Some(p) = pending {
            self.draw_trust_modal(ctx, g, &p);
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        match ev {
            Event::Down { x, y } => {
                self.scroll.stop();
                self.pressed = self.hit_at(x, y);
            }
            Event::Up { .. } => self.pressed = None,
            Event::Drag { dy, .. } | Event::Wheel { dy } => {
                self.pressed = None;
                self.scroll.drag(dy);
            }
            Event::DragEnd { vy, .. } => {
                self.scroll.fling(vy);
                ctx.animate();
            }
            Event::Tap { x, y } => {
                if let Some(i) = self.hit_at(x, y) {
                    let act = self.hits[i].1.clone();
                    self.run(ctx, act);
                }
            }
            _ => {}
        }
    }

    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        if self.scroll.update(dt) {
            ctx.animate();
        }
    }

    fn on_back(&mut self, ctx: &Ctx) -> bool {
        if self.sh.lock().unwrap().pending.is_some() {
            cancel_pending(&self.sh);
            ctx.animate();
            return true;
        }
        if let Screen::Detail(_) = self.screen {
            self.screen = Screen::List;
            self.scroll = Scroller::new();
            return true;
        }
        false
    }
}

/// Attend la fin du travail réseau en cours (captures d'écran uniquement).
fn wait_idle(sh: &Sh) {
    for _ in 0..600 {
        thread::sleep(Duration::from_millis(50));
        if sh.lock().unwrap().job == Job::Idle {
            break;
        }
    }
}

fn demo_catalog() -> Vec<Entry> {
    let mk = |id: &str, name: &str, ver: &str, size: u64, desc: &str| Entry {
        file: format!("{id}-{ver}.lapp"),
        id: id.to_string(),
        name: name.to_string(),
        version: ver.to_string(),
        size,
        desc: desc.to_string(),
    };
    vec![
        mk("com.moi.notes", "Notes", "1.2.0", 812_345, "Prise de notes rapide, enregistrées sur l'appareil."),
        mk("com.moi.horloge", "Horloge", "0.9.1", 420_000, "Horloge, minuteur et chronomètre en plein écran."),
        mk("com.moi.meteo", "Météo", "2.0.0", 1_950_000, ""),
    ]
}

fn main() {
    let store = Store::open("state.conf");
    let server = arg_value("--server")
        .or_else(|| store.get("server").map(|s| s.to_string()))
        .unwrap_or_else(|| DEFAULT_SERVER.to_string());

    let sh: Sh = Arc::new(Mutex::new(Shared::default()));
    let shot = arg_flag("--screenshot");
    if arg_flag("--demo") {
        sh.lock().unwrap().apps = demo_catalog();
    } else if !arg_flag("--icon") {
        spawn_refresh(server.clone(), sh.clone());
    }
    // Options de développement pour les captures : attendre le réseau, installer, ouvrir un détail.
    if shot && !arg_flag("--demo") {
        wait_idle(&sh);
        if let Some(id) = arg_value("--auto-install") {
            let entry = sh.lock().unwrap().apps.iter().find(|a| a.id == id).cloned();
            if let Some(e) = entry {
                spawn_install(server.clone(), e, sh.clone());
                wait_idle(&sh);
            }
        }
    }
    let screen = match arg_value("--page") {
        Some(id) => Screen::Detail(id),
        None => Screen::List,
    };

    let app = StoreApp {
        sh,
        server,
        store,
        screen,
        scroll: Scroller::new(),
        hits: Vec::new(),
        pressed: None,
        head_h: 44.0,
        inst: None,
        inst_at: None,
    };
    run(Config::new("Store"), app);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_from_manifests() {
        let d = std::env::temp_dir().join(format!("lapp-test-apps-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        for (id, ver) in [("com.a.un", "1.2.0"), ("com.a.deux", "0.9")] {
            fs::create_dir_all(d.join(id)).unwrap();
            fs::write(d.join(id).join("manifest.toml"), format!("manifest_version = 1\nid = \"{id}\"\nversion     = \"{ver}\"\nexec = \"bin/x\"\n")).unwrap();
        }
        fs::create_dir_all(d.join("com.a.vide")).unwrap(); // dossier sans manifeste : ignoré
        std::env::set_var("LAPP_APPS", &d);
        let m = read_installed().unwrap();
        assert_eq!(m.get("com.a.un").map(String::as_str), Some("1.2.0"));
        assert_eq!(m.get("com.a.deux").map(String::as_str), Some("0.9"));
        assert!(!m.contains_key("com.a.vide"));
        std::env::set_var("LAPP_APPS", d.join("absent"));
        assert!(read_installed().is_none());
        let _ = fs::remove_dir_all(&d);
        assert!(version_tuple("1.10.0") > version_tuple("1.9.9"));
    }
}
