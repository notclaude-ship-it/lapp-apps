//! Client du courtier de commandes de lapp (permission « commands ») : exécute une commande sur l'hôte,
//! hors du bac à sable. Protocole : voir lapp/src/broker.rs.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

pub struct Out {
    pub code: i32,
    pub out: String,
    pub err: String,
}

const DEFAULT_SOCK: &str = "/run/lapp/commands.sock";

/// Lance `args[0] args[1..]` sur l'hôte (entrée standard = `stdin`).
/// `Err` : courtier injoignable, commande refusée par le manifeste, délai dépassé.
pub fn run(args: &[&str], stdin: &str) -> Result<Out, String> {
    if let Ok(path) = std::env::var("LAPP_COMMANDS_SOCKET") {
        return via_broker(&path, args, stdin);
    }
    if Path::new("/app").is_dir() {
        if Path::new(DEFAULT_SOCK).exists() {
            return via_broker(DEFAULT_SOCK, args, stdin);
        }
        // dans un bac à sable lapp sans courtier : lapp trop ancien
        return Err("permission « commands » indisponible : mets à jour lapp".into());
    }
    direct(args, stdin) // hors bac à sable (développement sur PC)
}

fn via_broker(path: &str, args: &[&str], stdin: &str) -> Result<Out, String> {
    let mut s = UnixStream::connect(path).map_err(|e| format!("courtier de commandes injoignable ({e})"))?;
    let _ = s.set_read_timeout(Some(Duration::from_secs(150)));
    let mut m: Vec<u8> = Vec::new();
    m.extend_from_slice(&(args.len() as u32).to_be_bytes());
    for a in args {
        m.extend_from_slice(&(a.len() as u32).to_be_bytes());
        m.extend_from_slice(a.as_bytes());
    }
    m.extend_from_slice(&(stdin.len() as u32).to_be_bytes());
    m.extend_from_slice(stdin.as_bytes());
    let _ = s.write_all(&m); // le courtier peut déjà avoir répondu (refus) : on lit sa réponse quand même

    let mut code = [0u8; 4];
    s.read_exact(&mut code).map_err(|e| format!("réponse du courtier illisible ({e})"))?;
    let code = i32::from_be_bytes(code);
    let out = read_block(&mut s)?;
    let err = read_block(&mut s)?;
    let out = String::from_utf8_lossy(&out).into_owned();
    let err = String::from_utf8_lossy(&err).into_owned();
    if code == -1 {
        return Err(err.trim().to_string()); // refus ou erreur du courtier
    }
    Ok(Out { code, out, err })
}

fn read_block(s: &mut UnixStream) -> Result<Vec<u8>, String> {
    let mut n = [0u8; 4];
    s.read_exact(&mut n).map_err(|e| format!("réponse du courtier tronquée ({e})"))?;
    let n = u32::from_be_bytes(n) as usize;
    if n > 2 * 1024 * 1024 {
        return Err("réponse du courtier trop grande".into());
    }
    let mut b = vec![0u8; n];
    s.read_exact(&mut b).map_err(|e| format!("réponse du courtier tronquée ({e})"))?;
    Ok(b)
}

fn direct(args: &[&str], stdin: &str) -> Result<Out, String> {
    let mut c = Command::new(args[0]);
    c.args(&args[1..]).env("LC_ALL", "C").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut ch = c.spawn().map_err(|e| format!("{} introuvable ({e})", args[0]))?;
    if let Some(mut si) = ch.stdin.take() {
        let _ = si.write_all(stdin.as_bytes());
    }
    let o = ch.wait_with_output().map_err(|e| e.to_string())?;
    Ok(Out {
        code: o.status.code().unwrap_or(-1),
        out: String::from_utf8_lossy(&o.stdout).into_owned(),
        err: String::from_utf8_lossy(&o.stderr).into_owned(),
    })
}