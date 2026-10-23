//! Droits administrateur pour le Wi-Fi : toutes les commandes `nmcli` passent par `sudo` (lancé sur l'hôte par
//! le courtier de commandes, d'où `"sudo"` dans `commands` du manifeste).
//!
//! - Mot de passe inconnu : on essaie `sudo -n` (marche si sudo est configuré sans mot de passe). Si sudo en demande
//!   un, le module le signale (`take_need`) et l'interface le demande à l'utilisateur.
//! - Mot de passe saisi : il est transmis à `sudo -S` par l'entrée standard (jamais dans les arguments), et n'est
//!   enregistré (`<données>/sudo.pw`, droits 0600) qu'une fois que sudo l'a accepté.
//! - Mot de passe refusé : il est oublié (mémoire et fichier) et l'interface le redemande.

use std::fs;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;

use crate::hostcmd::{self, Out};

pub const NEEDED: &str = "Mot de passe sudo requis";
pub const WRONG: &str = "Mot de passe sudo incorrect";

struct Pw {
    value: Option<String>,
    /// le mot de passe en mémoire est celui du fichier
    saved: bool,
    loaded: bool,
}

static PW: Mutex<Pw> = Mutex::new(Pw { value: None, saved: false, loaded: false });
/// 0 = rien à signaler, 1 = mot de passe requis, 2 = mot de passe refusé
static NEED: AtomicU8 = AtomicU8::new(0);

fn lock() -> std::sync::MutexGuard<'static, Pw> {
    PW.lock().unwrap_or_else(|e| e.into_inner())
}

/// Fichier du mot de passe : `/data` dans le bac à sable, sinon `$LAPP_DATA`, sinon `./.lapp-data`.
fn file() -> PathBuf {
    let dir = if Path::new("/app").is_dir() {
        PathBuf::from("/data")
    } else {
        std::env::var("LAPP_DATA").ok().filter(|d| !d.is_empty()).map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".lapp-data"))
    };
    dir.join("sudo.pw")
}

fn load(g: &mut Pw) {
    g.loaded = true;
    if let Ok(t) = fs::read_to_string(file()) {
        let t = t.trim_end_matches(['\n', '\r']).to_string();
        if !t.is_empty() {
            g.value = Some(t);
            g.saved = true;
        }
    }
}

fn save(pw: &str) -> bool {
    let p = file();
    if let Some(d) = p.parent() {
        let _ = fs::create_dir_all(d);
    }
    let tmp = p.with_extension("tmp");
    let _ = fs::remove_file(&tmp);
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp)
        .and_then(|mut f| f.write_all(pw.as_bytes()))
        .and_then(|_| fs::rename(&tmp, &p))
        .is_ok()
}

/// Mot de passe saisi par l'utilisateur : utilisé tout de suite, enregistré après la première réussite.
pub fn set(pw: String) {
    let mut g = lock();
    g.value = Some(pw);
    g.saved = false;
    g.loaded = true;
}

/// Oublie le mot de passe (mémoire et fichier).
pub fn forget() {
    let mut g = lock();
    g.value = None;
    g.saved = false;
    g.loaded = true;
    let _ = fs::remove_file(file());
}

/// `Some(false)` : un mot de passe est requis ; `Some(true)` : celui qu'on avait a été refusé. Remet à zéro.
pub fn take_need() -> Option<bool> {
    match NEED.swap(0, Ordering::SeqCst) {
        1 => Some(false),
        2 => Some(true),
        _ => None,
    }
}

/// `true` si `stderr` est un refus de sudo lui-même (et pas une erreur de la commande lancée).
fn sudo_refused(stderr: &str) -> bool {
    stderr.lines().map(str::trim).any(|l| {
        l.starts_with("Sorry, try again")
            || (l.starts_with("sudo:")
                && (l.contains("a password is required")
                    || l.contains("incorrect password")
                    || l.contains("no password was provided")
                    || l.contains("a terminal is required")))
    })
}

/// Lance `args` (ex. `["nmcli", ...]`) en root sur l'hôte.
/// `Err` : courtier injoignable, mot de passe sudo requis ou refusé (voir `take_need`), etc.
pub fn run(args: &[&str]) -> Result<Out, String> {
    let pw = {
        let mut g = lock();
        if !g.loaded {
            load(&mut g);
        }
        g.value.clone()
    };
    let mut a: Vec<&str> = vec!["sudo"];
    match pw {
        Some(_) => a.extend_from_slice(&["-S", "-p", ""]),
        None => a.push("-n"),
    }
    a.extend_from_slice(args);
    let stdin = pw.as_ref().map(|p| format!("{p}\n")).unwrap_or_default();
    let o = hostcmd::run(&a, &stdin)?;
    if o.code != 0 && sudo_refused(&o.err) {
        if pw.is_some() {
            forget();
            NEED.store(2, Ordering::SeqCst);
            return Err(WRONG.into());
        }
        NEED.store(1, Ordering::SeqCst);
        return Err(NEEDED.into());
    }
    // sudo a accepté le mot de passe (la commande a réussi, ou a échoué pour une autre raison) : on l'enregistre
    if let Some(p) = pw {
        let mut g = lock();
        if !g.saved && g.value.as_deref() == Some(p.as_str()) && save(&p) {
            g.saved = true;
        }
    }
    Ok(o)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_sudo_refusals() {
        assert!(sudo_refused("sudo: a password is required\n"));
        assert!(sudo_refused("Sorry, try again.\nsudo: 1 incorrect password attempt\n"));
        assert!(sudo_refused("sudo: 3 incorrect password attempts"));
        assert!(sudo_refused("sudo: no password was provided"));
        assert!(sudo_refused("sudo: a terminal is required to read the password; either use the -S option"));
    }

    #[test]
    fn ignores_command_errors() {
        assert!(!sudo_refused("Error: Connection activation failed: (7) Secrets were required, but not provided."));
        assert!(!sudo_refused("Error: Timeout expired (30 seconds)"));
        assert!(!sudo_refused(""));
    }
}