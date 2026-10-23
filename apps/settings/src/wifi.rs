//! Wi-Fi via NetworkManager : `sudo nmcli` est lancé sur l'hôte par le courtier de commandes (permission « commands »,
//! `commands = ["sudo"]` dans le manifeste). Toutes les commandes passent par `sudo::run` (voir sudo.rs) : si le mot de
//! passe est inconnu ou refusé, `Done::sudo` le signale et l'interface le demande puis relance l'action. Tout ce module est bloquant : l'interface l'appelle dans un thread.

use std::time::Duration;

use crate::sudo;

#[derive(Clone, Debug, PartialEq)]
pub struct Net {
    pub ssid: String,
    /// 0..100
    pub signal: u8,
    /// "" = réseau ouvert, sinon « WPA2 », « WPA1 WPA2 », …
    pub security: String,
    pub active: bool,
    /// nom du profil NetworkManager enregistré pour ce SSID
    pub profile: Option<String>,
}

impl Net {
    pub fn secured(&self) -> bool {
        !self.security.is_empty()
    }
    /// Un mot de passe est à saisir : réseau protégé, jamais enregistré, pas connecté.
    pub fn needs_pass(&self) -> bool {
        self.secured() && self.profile.is_none() && !self.active
    }
}

#[derive(Clone, Default)]
pub struct State {
    pub enabled: bool,
    pub nets: Vec<Net>,
}

#[derive(Clone)]
pub enum Action {
    /// rien d'autre que relire l'état
    None,
    Toggle(bool),
    /// réseau + mot de passe (vide : profil enregistré ou réseau ouvert)
    Connect(Net, String),
    Disconnect(Net),
    Forget(Net),
}

pub struct Done {
    pub note: Option<Result<String, String>>,
    pub state: Result<State, String>,
    /// l'action est terminée : l'interface peut quitter la page du réseau
    pub back: bool,
    /// mot de passe sudo à demander avant de relancer l'action
    pub sudo: Option<Sudo>,
}

pub struct Sudo {
    /// le mot de passe qu'on avait a été refusé
    pub wrong: bool,
    /// l'action à relancer une fois le mot de passe saisi
    pub retry: Action,
    pub rescan: bool,
}

// ------------------------------------------------------------------ nmcli

fn nice_error(raw: &str) -> String {
    let l = raw.to_lowercase();
    if l.contains("secrets were required") || l.contains("no secrets") || l.contains("802-1x") || l.contains("4-way handshake") {
        return "Mot de passe incorrect".into();
    }
    if l.contains("no network with ssid") {
        return "Réseau introuvable".into();
    }
    if l.contains("timeout") || l.contains("timed out") {
        return "Délai dépassé".into();
    }
    if l.contains("not authorized") || l.contains("insufficient privileges") {
        return "Action refusée par NetworkManager (droits)".into();
    }
    if l.contains("wi-fi is disabled") || l.contains("wireless is disabled") {
        return "Wi-Fi désactivé".into();
    }
    raw.trim_start_matches("Error:").trim().to_string()
}

/// `sudo nmcli -w 30 <args>` ; `Ok(stdout)` si le code de sortie est 0.
/// `Err(sudo::NEEDED)` / `Err(sudo::WRONG)` : mot de passe sudo à saisir (voir `sudo::take_need`).
fn nm(args: &[&str]) -> Result<String, String> {
    let mut a: Vec<&str> = vec!["nmcli", "-w", "30"];
    a.extend_from_slice(args);
    let o = sudo::run(&a)?;
    if o.code == 0 {
        return Ok(o.out);
    }
    let raw = [o.err.as_str(), o.out.as_str()]
        .iter()
        .flat_map(|s| s.lines())
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("échec de nmcli")
        .to_string();
    Err(nice_error(&raw))
}

/// Exécute une commande nmcli avec sudo et demande le mot de passe si nécessaire.
fn nm_sudo(args: &[&str]) -> Result<String, String> {
    let mut cmd = vec!["sudo", "nmcli"];
    cmd.extend_from_slice(args);
    let o = hostcmd::run(&cmd, "")?;
    if o.code == 0 {
        return Ok(o.out);
    }
    let raw = [o.err.as_str(), o.out.as_str()]
        .iter()
        .flat_map(|s| s.lines())
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("échec de nmcli")
        .to_string();
    Err(nice_error(&raw))
}

/// Découpe une ligne du mode « terse » de nmcli : séparateur `:`, `\:` et `\\` échappés.
fn split_terse(line: &str) -> Vec<String> {
    let mut out = vec![];
    let mut cur = String::new();
    let mut esc = false;
    for c in line.chars() {
        if esc {
            cur.push(c);
            esc = false;
        } else if c == '\\' {
            esc = true;
        } else if c == ':' {
            out.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    out.push(cur);
    out
}

/// Profils Wi-Fi enregistrés : (SSID, nom du profil).
fn saved_profiles() -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = vec![];
    let Ok(out) = nm(&["-t", "-f", "NAME,TYPE", "connection", "show"]) else { return v };
    for l in out.lines() {
        let f = split_terse(l);
        if f.len() >= 2 && f[1] == "802-11-wireless" && v.len() < 16 {
            let name = f[0].clone();
            let ssid = nm(&["-g", "802-11-wireless.ssid", "connection", "show", "id", &name])
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| name.clone());
            v.push((ssid, name));
        }
    }
    v
}

pub fn state(rescan: bool) -> Result<State, String> {
    let radio = nm(&["radio", "wifi"])?;
    if radio.trim() != "enabled" {
        return Ok(State { enabled: false, nets: vec![] });
    }
    let profiles = saved_profiles();
    let cols = ["-t", "-f", "IN-USE,SIGNAL,SECURITY,SSID", "dev", "wifi", "list", "--rescan"];
    let run = |r: &str| {
        let mut a = cols.to_vec();
        a.push(r);
        nm(&a)
    };
    // un balayage trop rapproché du précédent peut être refusé : on retombe sur la liste en cache
    let list = if rescan { run("yes").or_else(|_| run("no"))? } else { run("no")? };

    let mut nets: Vec<Net> = vec![];
    for l in list.lines() {
        let f = split_terse(l);
        if f.len() < 4 || f[3].is_empty() {
            continue;
        }
        let ssid = f[3].clone();
        let active = f[0].trim() == "*";
        let signal = f[1].trim().parse::<u8>().unwrap_or(0).min(100);
        let security = match f[2].trim() {
            "--" => "".to_string(),
            s => s.to_string(),
        };
        if let Some(n) = nets.iter_mut().find(|n| n.ssid == ssid) {
            // même SSID vu par plusieurs points d'accès : on garde le plus fort
            n.active |= active;
            n.signal = n.signal.max(signal);
            continue;
        }
        let profile = profiles.iter().find(|(s, _)| *s == ssid).map(|(_, n)| n.clone());
        nets.push(Net { ssid, signal, security, active, profile });
    }
    nets.sort_by(|a, b| b.active.cmp(&a.active).then(b.signal.cmp(&a.signal)).then(a.ssid.cmp(&b.ssid)));
    Ok(State { enabled: true, nets })
}

/// Exécute l'action puis relit l'état (avec ou sans nouveau balayage).
pub fn run(a: Action, rescan: bool) -> Done {
    let again = (a.clone(), rescan);
    let mut back = false;
    let mut rescan = rescan;
    let note = match a {
        Action::None => None,
        Action::Toggle(on) => {
            let r = nm_sudo(&["radio", "wifi", if on { "on" } else { "off" }]);
            if on && r.is_ok() {
                std::thread::sleep(Duration::from_secs(2)); // laisse la puce se réveiller avant de balayer
                rescan = true;
            }
            Some(r.map(|_| if on { "Wi-Fi activé".to_string() } else { "Wi-Fi désactivé".to_string() }))
        }
        Action::Connect(n, pw) => {
            let r = match (&n.profile, pw.is_empty()) {
                (Some(p), true) => nm_sudo(&["connection", "up", "id", p]),
                (_, true) => nm_sudo(&["dev", "wifi", "connect", &n.ssid]),
                (_, false) => nm_sudo(&["dev", "wifi", "connect", &n.ssid, "password", &pw]),
            };
            match r {
                Ok(_) => {
                    back = true;
                    Some(Ok(format!("Connecté à {}", n.ssid)))
                }
                Err(e) => {
                    if n.profile.is_none() && !pw.is_empty() && e != sudo::NEEDED && e != sudo::WRONG {
                        // nmcli laisse un profil inutilisable quand le mot de passe est refusé
                        let _ = nm_sudo(&["connection", "delete", "id", &n.ssid]);
                    }
                    Some(Err(e))
                }
            }
        }
        Action::Disconnect(n) => {
            let name = n.profile.clone().unwrap_or_else(|| n.ssid.clone());
            let r = nm_sudo(&["connection", "down", "id", &name]);
            back = r.is_ok();
            Some(r.map(|_| format!("Déconnecté de {}", n.ssid)))
        }
        Action::Forget(n) => {
            let name = n.profile.clone().unwrap_or_else(|| n.ssid.clone());
            let r = nm_sudo(&["connection", "delete", "id", &name]);
            back = r.is_ok();
            Some(r.map(|_| format!("{} oublié", n.ssid)))
        }
    };
    let state = state(rescan);
    // mot de passe sudo requis ou refusé : l'interface le demande, puis relance l'action
    if let Some(wrong) = sudo::take_need() {
        return Done { note: None, state, back: false, sudo: Some(Sudo { wrong, retry: again.0, rescan: again.1 }) };
    }
    Done { note, state, back, sudo: None }
}

/// Réseaux factices pour les captures d'écran (`--demo`).
pub fn demo() -> State {
    let n = |ssid: &str, signal: u8, security: &str, active: bool, profile: bool| Net {
        ssid: ssid.into(),
        signal,
        security: security.into(),
        active,
        profile: if profile { Some(ssid.into()) } else { None },
    };
    State {
        enabled: true,
        nets: vec![
            n("Livebox-A1B2", 82, "WPA2", true, true),
            n("Hotspot Alexis", 67, "WPA2", false, true),
            n("Cafe Wifi", 55, "", false, false),
            n("Voisin_5G", 40, "WPA2", false, false),
            n("FreeWifi_secure", 22, "WPA1 WPA2", false, false),
            n("SFR WiFi Mobile", 12, "WPA2", false, false),
            n("Bbox-7F3C9A", 9, "WPA2", false, false),
            n("TP-Link_C4E1", 6, "WPA2", false, false),
        ],
    }
}