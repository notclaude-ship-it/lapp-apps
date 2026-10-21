//! Fil d'arrière-plan : interroge le lecteur MPRIS de spotifyd (D-Bus) et exécute les commandes.
//!
//! L'interface graphique ne bloque jamais sur D-Bus : elle lit un instantané partagé (`Snapshot`) et
//! envoie des commandes (`Cmd`) au fil, qui les exécute puis relit l'état aussitôt.
//!
//! spotifyd n'expose MPRIS (`org.mpris.MediaPlayer2.spotifyd.instance<pid>`) que lorsqu'il est le
//! lecteur actif de Spotify Connect : tant qu'aucun client ne l'a sélectionné, `player` reste vide.

use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::dbus::{Arg, Conn};

const OBJ: &str = "/org/mpris/MediaPlayer2";
const IFACE: &str = "org.mpris.MediaPlayer2.Player";
const PROPS: &str = "org.freedesktop.DBus.Properties";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Playing,
    Paused,
    Stopped,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LoopMode {
    Off,
    Track,
    Playlist,
}

impl LoopMode {
    fn parse(s: &str) -> LoopMode {
        match s {
            "Track" => LoopMode::Track,
            "Playlist" => LoopMode::Playlist,
            _ => LoopMode::Off,
        }
    }
    fn next(self) -> LoopMode {
        match self {
            LoopMode::Off => LoopMode::Playlist,
            LoopMode::Playlist => LoopMode::Track,
            LoopMode::Track => LoopMode::Off,
        }
    }
    fn as_str(self) -> &'static str {
        match self {
            LoopMode::Off => "None",
            LoopMode::Track => "Track",
            LoopMode::Playlist => "Playlist",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Track {
    /// `mpris:trackid` (chemin d'objet), nécessaire pour `SetPosition`.
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub length_us: i64,
    pub art_url: String,
}

/// État du lecteur à un instant donné ; la position se prolonge côté interface avec `sampled`.
#[derive(Clone, Debug)]
pub struct Snapshot {
    /// Erreur de connexion au bus D-Bus (bus filtré absent, refus d'authentification…).
    pub bus_error: Option<String>,
    /// Nom D-Bus du lecteur Spotify trouvé ; `None` = spotifyd n'est pas le lecteur actif.
    pub player: Option<String>,
    pub status: Status,
    pub track: Track,
    pub position_us: i64,
    pub sampled: Instant,
    pub shuffle: bool,
    pub loop_mode: LoopMode,
    pub volume: Option<f64>,
    pub can_seek: bool,
    pub can_next: bool,
    pub can_prev: bool,
    /// (numéro, message) de la dernière commande refusée ; le numéro change à chaque erreur.
    pub error: Option<(u64, String)>,
}

impl Snapshot {
    fn empty() -> Snapshot {
        Snapshot {
            bus_error: None,
            player: None,
            status: Status::Stopped,
            track: Track::default(),
            position_us: 0,
            sampled: Instant::now(),
            shuffle: false,
            loop_mode: LoopMode::Off,
            volume: None,
            can_seek: false,
            can_next: true,
            can_prev: true,
            error: None,
        }
    }

    pub fn playing(&self) -> bool {
        self.status == Status::Playing
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Cmd {
    PlayPause,
    Next,
    Prev,
    /// Position absolue en microsecondes.
    Seek(i64),
    Shuffle,
    Loop,
}

pub struct Player {
    shared: Arc<Mutex<Snapshot>>,
    tx: Option<Sender<Cmd>>,
}

fn lock(m: &Mutex<Snapshot>) -> MutexGuard<'_, Snapshot> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl Player {
    /// Démarre le fil D-Bus.
    pub fn spawn() -> Player {
        let shared = Arc::new(Mutex::new(Snapshot::empty()));
        let (tx, rx) = channel();
        let sh = shared.clone();
        std::thread::spawn(move || worker(sh, rx));
        Player { shared, tx: Some(tx) }
    }

    /// État fictif, sans D-Bus (captures d'écran : `--demo`).
    pub fn demo() -> Player {
        let mut s = Snapshot::empty();
        s.player = Some("org.mpris.MediaPlayer2.spotifyd.instance1".into());
        s.status = Status::Playing;
        s.track = Track {
            id: "/org/mpris/MediaPlayer2/Track/demo".into(),
            title: "Midnight City".into(),
            artist: "M83".into(),
            album: "Hurry Up, We're Dreaming".into(),
            length_us: 243_000_000,
            art_url: "https://i.scdn.co/image/ab67616d0000b273ba5db46f4b838ef6027e6f96".into(),
        };
        s.position_us = 94_000_000;
        s.shuffle = true;
        s.loop_mode = LoopMode::Playlist;
        s.volume = Some(0.65);
        s.can_seek = true;
        Player { shared: Arc::new(Mutex::new(s)), tx: None }
    }

    pub fn snapshot(&self) -> Snapshot {
        lock(&self.shared).clone()
    }

    pub fn send(&self, c: Cmd) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(c);
        }
    }
}

struct Polled {
    name: String,
    status: Status,
    track: Track,
    position_us: i64,
    shuffle: bool,
    loop_mode: LoopMode,
    volume: Option<f64>,
    can_seek: bool,
    can_next: bool,
    can_prev: bool,
}

struct Worker {
    conn: Option<Conn>,
    name: Option<String>,
    errors: u64,
}

fn worker(shared: Arc<Mutex<Snapshot>>, rx: std::sync::mpsc::Receiver<Cmd>) {
    let mut w = Worker { conn: None, name: None, errors: 0 };
    let mut wait = Duration::from_millis(100);
    loop {
        let mut cmds = Vec::new();
        match rx.recv_timeout(wait) {
            Ok(c) => {
                cmds.push(c);
                while let Ok(c) = rx.try_recv() {
                    cmds.push(c);
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        for c in cmds {
            let snap = lock(&shared).clone();
            if let Err(e) = w.exec(c, &snap) {
                w.errors += 1;
                lock(&shared).error = Some((w.errors, e));
            }
        }
        let polled = w.poll();
        w.publish(&shared, polled);
        let playing = lock(&shared).playing();
        wait = Duration::from_millis(if playing { 500 } else { 1000 });
    }
}

impl Worker {
    fn ensure(&mut self) -> Result<&mut Conn, String> {
        if self.conn.is_none() {
            self.conn = Some(Conn::connect()?);
        }
        Ok(self.conn.as_mut().unwrap())
    }

    /// Cherche le lecteur Spotify parmi les noms MPRIS visibles (le bus filtré n'en montre pas d'autres).
    fn find_player(&mut self) -> Result<Option<String>, String> {
        let conn = self.ensure()?;
        let out = conn.call("org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus", "ListNames", &[])?;
        let names: Vec<String> = out
            .first()
            .map(|v| v.as_strings())
            .unwrap_or_default()
            .into_iter()
            .filter(|n| n.starts_with("org.mpris.MediaPlayer2.") && n.contains("spotify"))
            .collect();
        if let Some(cur) = &self.name {
            if names.contains(cur) {
                return Ok(Some(cur.clone()));
            }
        }
        let pick = names.iter().find(|n| n.contains("spotifyd")).or_else(|| names.first()).cloned();
        self.name = pick.clone();
        Ok(pick)
    }

    fn poll(&mut self) -> Result<Option<Polled>, String> {
        let Some(name) = self.find_player()? else { return Ok(None) };
        let conn = self.ensure()?;
        let all = match conn.call(&name, OBJ, PROPS, "GetAll", &[Arg::Str(IFACE)]) {
            Ok(v) => v,
            Err(e) if e.starts_with("D-Bus :") => return Err(e),
            Err(_) => {
                // le lecteur a disparu entre ListNames et GetAll
                self.name = None;
                return Ok(None);
            }
        };
        let props = all.into_iter().next().ok_or("réponse GetAll vide")?;
        let position_us = conn
            .call(&name, OBJ, PROPS, "Get", &[Arg::Str(IFACE), Arg::Str("Position")])
            .ok()
            .and_then(|v| v.first().and_then(|p| p.as_i64()))
            .unwrap_or(0);

        let status = match props.get("PlaybackStatus").and_then(|v| v.as_str()) {
            Some("Playing") => Status::Playing,
            Some("Paused") => Status::Paused,
            _ => Status::Stopped,
        };
        let meta = props.get("Metadata");
        let text = |k: &str| meta.and_then(|m| m.get(k)).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let artist = match meta.and_then(|m| m.get("xesam:artist")) {
            Some(v) => {
                let list = v.as_strings();
                if list.is_empty() {
                    v.as_str().unwrap_or("").to_string()
                } else {
                    list.join(", ")
                }
            }
            None => String::new(),
        };
        let track = Track {
            id: text("mpris:trackid"),
            title: text("xesam:title"),
            artist,
            album: text("xesam:album"),
            length_us: meta.and_then(|m| m.get("mpris:length")).and_then(|v| v.as_i64()).unwrap_or(0),
            art_url: text("mpris:artUrl"),
        };
        let flag = |k: &str, d: bool| props.get(k).and_then(|v| v.as_bool()).unwrap_or(d);
        Ok(Some(Polled {
            name,
            status,
            track,
            position_us,
            shuffle: flag("Shuffle", false),
            loop_mode: LoopMode::parse(props.get("LoopStatus").and_then(|v| v.as_str()).unwrap_or("None")),
            volume: props.get("Volume").and_then(|v| v.as_f64()),
            can_seek: flag("CanSeek", true),
            can_next: flag("CanGoNext", true),
            can_prev: flag("CanGoPrevious", true),
        }))
    }

    fn publish(&mut self, shared: &Mutex<Snapshot>, polled: Result<Option<Polled>, String>) {
        let mut s = lock(shared);
        match polled {
            Ok(Some(p)) => {
                s.bus_error = None;
                s.player = Some(p.name);
                s.status = p.status;
                s.track = p.track;
                s.position_us = p.position_us;
                s.sampled = Instant::now();
                s.shuffle = p.shuffle;
                s.loop_mode = p.loop_mode;
                s.volume = p.volume;
                s.can_seek = p.can_seek;
                s.can_next = p.can_next;
                s.can_prev = p.can_prev;
            }
            Ok(None) => {
                s.bus_error = None;
                s.player = None;
                s.status = Status::Stopped;
                s.track = Track::default();
                s.position_us = 0;
                s.sampled = Instant::now();
            }
            Err(e) => {
                s.bus_error = Some(e);
                s.player = None;
                s.status = Status::Stopped;
                s.track = Track::default();
                self.conn = None; // reconnexion au prochain tour
            }
        }
    }

    fn exec(&mut self, cmd: Cmd, snap: &Snapshot) -> Result<(), String> {
        let Some(name) = self.name.clone() else { return Err("spotifyd n'est pas le lecteur actif".into()) };
        let conn = self.ensure()?;
        let r = match cmd {
            Cmd::PlayPause => conn.call(&name, OBJ, IFACE, "PlayPause", &[]),
            Cmd::Next => conn.call(&name, OBJ, IFACE, "Next", &[]),
            Cmd::Prev => conn.call(&name, OBJ, IFACE, "Previous", &[]),
            Cmd::Seek(us) => {
                if snap.track.id.is_empty() {
                    conn.call(&name, OBJ, IFACE, "Seek", &[Arg::I64(us - snap.position_us)])
                } else {
                    conn.call(&name, OBJ, IFACE, "SetPosition", &[Arg::Path(&snap.track.id), Arg::I64(us)])
                }
            }
            Cmd::Shuffle => conn.call(
                &name,
                OBJ,
                PROPS,
                "Set",
                &[Arg::Str(IFACE), Arg::Str("Shuffle"), Arg::Variant(Box::new(Arg::Bool(!snap.shuffle)))],
            ),
            Cmd::Loop => conn.call(
                &name,
                OBJ,
                PROPS,
                "Set",
                &[Arg::Str(IFACE), Arg::Str("LoopStatus"), Arg::Variant(Box::new(Arg::Str(snap.loop_mode.next().as_str())))],
            ),
        };
        match r {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.starts_with("D-Bus :") {
                    self.conn = None;
                }
                Err(e)
            }
        }
    }
}