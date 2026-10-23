//! Bluetooth via BlueZ : `bluetoothctl` est lancé sur l'hôte par le courtier de commandes (permission « commands »,
//! `commands = ["bluetoothctl", ...]` dans le manifeste). Tout ce module est bloquant : l'interface l'appelle dans un thread.

use std::time::Duration;

use crate::hostcmd;

#[derive(Clone, Debug, PartialEq)]
pub struct Dev {
    pub mac: String,
    pub name: String,
    pub paired: bool,
    pub connected: bool,
    pub trusted: bool,
}

#[derive(Clone, Default)]
pub struct State {
    /// pas de contrôleur Bluetooth (ou BlueZ arrêté)
    pub present: bool,
    pub powered: bool,
    pub devs: Vec<Dev>,
}

pub enum Action {
    /// rien d'autre que relire l'état
    None,
    Toggle(bool),
    /// recherche d'appareils à proximité (quelques secondes)
    Scan,
    /// jumelage (si besoin) + confiance + connexion
    Connect(Dev),
    Disconnect(Dev),
    Forget(Dev),
}

pub struct Done {
    pub note: Option<Result<String, String>>,
    pub state: Result<State, String>,
    /// l'action est terminée : l'interface peut quitter la page de l'appareil
    pub back: bool,
}

// ------------------------------------------------------------------ bluetoothctl

fn nice_error(raw: &str) -> String {
    let l = raw.to_lowercase();
    if l.contains("no default controller") {
        return "Aucun contrôleur Bluetooth".into();
    }
    if l.contains("rfkill") || l.contains("blocked") {
        return "Bluetooth bloqué (rfkill)".into();
    }
    if l.contains("not available") || l.contains("does not exist") {
        return "Appareil introuvable (relance la recherche)".into();
    }
    if l.contains("authenticationfailed") || l.contains("authentication failed") {
        return "Authentification refusée".into();
    }
    if l.contains("authenticationcanceled") || l.contains("authenticationrejected") || l.contains("rejected") {
        return "Jumelage refusé par l'appareil".into();
    }
    if l.contains("alreadyexists") || l.contains("already exists") {
        return "Déjà jumelé".into();
    }
    if l.contains("page timeout") || l.contains("timeout") || l.contains("timed out") {
        return "Délai dépassé (appareil éteint ou trop loin ?)".into();
    }
    if l.contains("connectionattemptfailed") || l.contains("connection attempt failed") {
        return "Connexion impossible".into();
    }
    if l.contains("inprogress") || l.contains("in progress") {
        return "Opération déjà en cours".into();
    }
    raw.trim().to_string()
}

fn failed(text: &str) -> bool {
    let l = text.to_lowercase();
    l.contains("failed to") || l.contains("no default controller") || l.contains("not available") || l.contains("org.bluez.error")
}

/// `bluetoothctl <args>` ; `Ok(stdout)` si la commande a réussi.
fn bt(args: &[&str]) -> Result<String, String> {
    let mut a: Vec<&str> = vec!["bluetoothctl"];
    a.extend_from_slice(args);
    let o = hostcmd::run(&a, "")?;
    // bluetoothctl affiche souvent ses erreurs sur stdout et peut sortir avec 0 : on regarde aussi le texte
    if o.code == 0 && !failed(&o.out) {
        return Ok(o.out);
    }
    let raw = [o.err.as_str(), o.out.as_str()]
        .iter()
        .flat_map(|s| s.lines())
        .map(|l| l.trim())
        .find(|l| {
            let ll = l.to_lowercase();
            ll.contains("fail") || ll.contains("error") || ll.contains("no default") || ll.contains("not available")
        })
        .or_else(|| [o.err.as_str(), o.out.as_str()].iter().flat_map(|s| s.lines()).map(|l| l.trim()).find(|l| !l.is_empty()))
        .unwrap_or("échec de bluetoothctl")
        .to_string();
    Err(nice_error(&raw))
}

fn valid_mac(s: &str) -> bool {
    s.len() == 17 && s.split(':').count() == 6 && s.split(':').all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
}

/// Lignes « Device AA:BB:… Nom » de `devices`.
fn parse_devices(out: &str) -> Vec<(String, String)> {
    let mut v = vec![];
    for l in out.lines() {
        // les lignes peuvent être préfixées par "[NEW] " ou par des codes de couleur
        let l = l.trim();
        let Some(i) = l.find("Device ") else { continue };
        let rest = &l[i + 7..];
        let mac = rest.get(..17).unwrap_or("");
        if !valid_mac(mac) {
            continue;
        }
        let name = rest[17..].trim().to_string();
        v.push((mac.to_uppercase(), name));
    }
    v
}

fn flag(info: &str, key: &str) -> bool {
    info.lines().any(|l| {
        let l = l.trim();
        l.strip_prefix(key).map(|r| r.trim_start_matches(':').trim() == "yes").unwrap_or(false)
    })
}

pub fn state() -> Result<State, String> {
    let show = match hostcmd::run(&["bluetoothctl", "show"], "") {
        Ok(o) => format!("{}\n{}", o.out, o.err),
        Err(e) => return Err(e),
    };
    if show.contains("No default controller") || !show.contains("Powered") {
        return Ok(State { present: false, powered: false, devs: vec![] });
    }
    let powered = flag(&show, "Powered");
    if !powered {
        return Ok(State { present: true, powered: false, devs: vec![] });
    }
    // `devices` = tous les appareils connus (jumelés + découverts)
    let all = bt(&["devices"]).unwrap_or_default();
    let mut devs: Vec<Dev> = vec![];
    for (mac, name) in parse_devices(&all).into_iter().take(24) {
        let info = bt(&["info", &mac]).unwrap_or_default();
        // appareil dont BlueZ n'a gardé que l'adresse (nom = adresse avec des tirets) : on le masque s'il n'est pas jumelé
        let paired = flag(&info, "Paired");
        let anon = name.is_empty() || name.replace('-', ":").eq_ignore_ascii_case(&mac);
        if anon && !paired {
            continue;
        }
        devs.push(Dev {
            name: if name.is_empty() { mac.clone() } else { name },
            mac,
            paired,
            connected: flag(&info, "Connected"),
            trusted: flag(&info, "Trusted"),
        });
    }
    devs.sort_by(|a, b| {
        b.connected.cmp(&a.connected).then(b.paired.cmp(&a.paired)).then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(State { present: true, powered: true, devs })
}

/// Exécute l'action puis relit l'état.
pub fn run(a: Action) -> Done {
    let mut back = false;
    let note = match a {
        Action::None => None,
        Action::Toggle(on) => {
            let mut r = bt(&["power", if on { "on" } else { "off" }]);
            if on && r.is_err() {
                // radio bloquée par rfkill : on débloque puis on réessaie
                let _ = hostcmd::run(&["rfkill", "unblock", "bluetooth"], "");
                std::thread::sleep(Duration::from_millis(800));
                r = bt(&["power", "on"]);
            }
            if on && r.is_ok() {
                let _ = bt(&["pairable", "on"]);
                std::thread::sleep(Duration::from_secs(1));
            }
            Some(r.map(|_| if on { "Bluetooth activé".to_string() } else { "Bluetooth désactivé".to_string() }))
        }
        Action::Scan => {
            // `scan on` ne reste actif que le temps du --timeout
            let r = bt(&["--timeout", "8", "scan", "on"]);
            let _ = bt(&["scan", "off"]);
            Some(r.map(|_| String::new()))
        }
        Action::Connect(d) => {
            let mut r: Result<String, String> = Ok(String::new());
            if !d.paired {
                r = bt(&["--timeout", "30", "pair", &d.mac]);
            }
            if r.is_ok() {
                let _ = bt(&["trust", &d.mac]);
                r = bt(&["--timeout", "20", "connect", &d.mac]);
            }
            match r {
                Ok(_) => {
                    back = true;
                    Some(Ok(format!("Connecté à {}", d.name)))
                }
                Err(e) => Some(Err(e)),
            }
        }
        Action::Disconnect(d) => {
            let r = bt(&["disconnect", &d.mac]);
            back = r.is_ok();
            Some(r.map(|_| format!("Déconnecté de {}", d.name)))
        }
        Action::Forget(d) => {
            let _ = bt(&["disconnect", &d.mac]);
            let r = bt(&["remove", &d.mac]);
            back = r.is_ok();
            Some(r.map(|_| format!("{} oublié", d.name)))
        }
    };
    Done { note, state: state(), back }
}

/// Appareils factices pour les captures d'écran (`--demo`).
pub fn demo() -> State {
    let d = |name: &str, mac: &str, paired: bool, connected: bool| Dev {
        name: name.into(),
        mac: mac.into(),
        paired,
        connected,
        trusted: paired,
    };
    State {
        present: true,
        powered: true,
        devs: vec![
            d("Casque JBL Tune 510", "AA:BB:CC:11:22:33", true, true),
            d("Manette Xbox", "AA:BB:CC:44:55:66", true, false),
            d("Enceinte Salon", "AA:BB:CC:77:88:99", true, false),
            d("Galaxy S21", "11:22:33:44:55:66", false, false),
            d("Mi Band 6", "11:22:33:AA:BB:CC", false, false),
            d("Souris MX Anywhere", "DE:AD:BE:EF:00:01", false, false),
        ],
    }
}