//! Client D-Bus minimal, sans dépendance : authentification EXTERNAL, appels de méthode, types usuels.
//!
//! Suffisant pour MPRIS (chaînes, entiers, flottants, booléens, tableaux, dictionnaires, variants).
//! Entrée : `DBUS_SESSION_BUS_ADDRESS` (`unix:path=...`), sinon `/run/user/app/bus` (bus filtré de lapp,
//! permission `mpris`).

use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

/// Valeur reçue.
#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F64(f64),
    /// Chaîne, chemin d'objet ou signature.
    Str(String),
    Array(Vec<Value>),
    Dict(Vec<(Value, Value)>),
    Struct(Vec<Value>),
    Variant(Box<Value>),
}

impl Value {
    pub fn unwrap_variant(&self) -> &Value {
        match self {
            Value::Variant(v) => v.unwrap_variant(),
            o => o,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self.unwrap_variant() {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self.unwrap_variant() {
            Value::F64(f) => Some(*f),
            v => v.as_i64().map(|i| i as f64),
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self.unwrap_variant() {
            Value::I64(v) => Some(*v),
            Value::U64(v) => Some(*v as i64),
            Value::I32(v) => Some(*v as i64),
            Value::U32(v) => Some(*v as i64),
            Value::I16(v) => Some(*v as i64),
            Value::U16(v) => Some(*v as i64),
            Value::U8(v) => Some(*v as i64),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self.unwrap_variant() {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    /// Tableau de chaînes (`as`) ; vide si ce n'en est pas un.
    pub fn as_strings(&self) -> Vec<String> {
        match self.unwrap_variant() {
            Value::Array(a) => a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
            _ => vec![],
        }
    }
    /// Entrée `key` d'un dictionnaire `a{sv}`.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self.unwrap_variant() {
            Value::Dict(d) => d.iter().find(|(k, _)| k.as_str() == Some(key)).map(|(_, v)| v),
            _ => None,
        }
    }
}

/// Argument d'un appel.
pub enum Arg<'a> {
    Str(&'a str),
    Path(&'a str),
    I64(i64),
    F64(f64),
    Bool(bool),
    Variant(Box<Arg<'a>>),
}

impl Arg<'_> {
    fn sig(&self) -> String {
        match self {
            Arg::Str(_) => "s",
            Arg::Path(_) => "o",
            Arg::I64(_) => "x",
            Arg::F64(_) => "d",
            Arg::Bool(_) => "b",
            Arg::Variant(_) => "v",
        }
        .to_string()
    }

    fn write(&self, w: &mut W) {
        match self {
            Arg::Str(s) | Arg::Path(s) => w.string(s),
            Arg::I64(v) => {
                w.align(8);
                w.b.extend_from_slice(&v.to_le_bytes());
            }
            Arg::F64(v) => {
                w.align(8);
                w.b.extend_from_slice(&v.to_le_bytes());
            }
            Arg::Bool(v) => w.u32(*v as u32),
            Arg::Variant(inner) => {
                w.sig(&inner.sig());
                inner.write(w);
            }
        }
    }
}

struct W {
    b: Vec<u8>,
}

impl W {
    fn new() -> W {
        W { b: Vec::new() }
    }
    fn align(&mut self, n: usize) {
        while self.b.len() % n != 0 {
            self.b.push(0);
        }
    }
    fn u32(&mut self, v: u32) {
        self.align(4);
        self.b.extend_from_slice(&v.to_le_bytes());
    }
    fn string(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.b.extend_from_slice(s.as_bytes());
        self.b.push(0);
    }
    fn sig(&mut self, s: &str) {
        self.b.push(s.len() as u8);
        self.b.extend_from_slice(s.as_bytes());
        self.b.push(0);
    }
    /// Champ d'en-tête : (code, signature, valeur chaîne / chemin / signature).
    fn field_str(&mut self, code: u8, sig: &str, v: &str) {
        self.align(8);
        self.b.push(code);
        self.sig(sig);
        if sig == "g" {
            self.sig(v);
        } else {
            self.string(v);
        }
    }
}

struct R<'a> {
    b: &'a [u8],
    p: usize,
}

fn align_of(c: u8) -> usize {
    match c {
        b'y' | b'g' | b'v' => 1,
        b'n' | b'q' => 2,
        b'x' | b't' | b'd' | b'(' | b'{' => 8,
        _ => 4,
    }
}

/// Longueur (en octets de signature) du premier type complet de `s`.
fn type_len(s: &[u8]) -> usize {
    match s.first() {
        Some(b'a') => 1 + type_len(&s[1..]),
        Some(b'(') | Some(b'{') => {
            let mut d = 0;
            for (i, &c) in s.iter().enumerate() {
                if c == b'(' || c == b'{' {
                    d += 1;
                } else if c == b')' || c == b'}' {
                    d -= 1;
                    if d == 0 {
                        return i + 1;
                    }
                }
            }
            s.len()
        }
        _ => 1,
    }
}

impl<'a> R<'a> {
    fn align(&mut self, n: usize) {
        self.p = self.p.div_ceil(n) * n;
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        if self.p + n > self.b.len() {
            return Err("message D-Bus tronqué".into());
        }
        let s = &self.b[self.p..self.p + n];
        self.p += n;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, String> {
        self.align(4);
        let s = self.take(4)?;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
    fn sig(&mut self) -> Result<String, String> {
        let n = self.u8()? as usize;
        let s = self.take(n)?.to_vec();
        self.take(1)?;
        String::from_utf8(s).map_err(|e| e.to_string())
    }
    fn eight(&mut self) -> Result<[u8; 8], String> {
        self.align(8);
        let s = self.take(8)?;
        Ok([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]])
    }

    /// Lit une valeur du type `sig` (premier type complet) ; renvoie aussi la longueur de signature consommée.
    fn value(&mut self, sig: &[u8]) -> Result<(Value, usize), String> {
        let c = *sig.first().ok_or("signature vide")?;
        match c {
            b'y' => Ok((Value::U8(self.u8()?), 1)),
            b'b' => Ok((Value::Bool(self.u32()? != 0), 1)),
            b'n' => {
                self.align(2);
                let s = self.take(2)?;
                Ok((Value::I16(i16::from_le_bytes([s[0], s[1]])), 1))
            }
            b'q' => {
                self.align(2);
                let s = self.take(2)?;
                Ok((Value::U16(u16::from_le_bytes([s[0], s[1]])), 1))
            }
            b'i' => Ok((Value::I32(self.u32()? as i32), 1)),
            b'u' => Ok((Value::U32(self.u32()?), 1)),
            b'x' => Ok((Value::I64(i64::from_le_bytes(self.eight()?)), 1)),
            b't' => Ok((Value::U64(u64::from_le_bytes(self.eight()?)), 1)),
            b'd' => Ok((Value::F64(f64::from_le_bytes(self.eight()?)), 1)),
            b's' | b'o' => {
                let n = self.u32()? as usize;
                let s = self.take(n)?.to_vec();
                self.take(1)?;
                Ok((Value::Str(String::from_utf8_lossy(&s).into_owned()), 1))
            }
            b'g' => Ok((Value::Str(self.sig()?), 1)),
            b'v' => {
                let s = self.sig()?;
                let (v, _) = self.value(s.as_bytes())?;
                Ok((Value::Variant(Box::new(v)), 1))
            }
            b'a' => {
                let n = self.u32()? as usize;
                let el = &sig[1..];
                let tl = type_len(el);
                let et = &el[..tl];
                self.align(align_of(et[0]));
                let end = self.p + n;
                if end > self.b.len() {
                    return Err("tableau D-Bus tronqué".into());
                }
                if et[0] == b'{' {
                    let mut v = vec![];
                    while self.p < end {
                        self.align(8);
                        let (k, kl) = self.value(&et[1..])?;
                        let (val, _) = self.value(&et[1 + kl..])?;
                        v.push((k, val));
                    }
                    Ok((Value::Dict(v), 1 + tl))
                } else {
                    let mut v = vec![];
                    while self.p < end {
                        let (x, _) = self.value(et)?;
                        v.push(x);
                    }
                    Ok((Value::Array(v), 1 + tl))
                }
            }
            b'(' => {
                self.align(8);
                let mut i = 1;
                let mut v = vec![];
                while sig.get(i).copied() != Some(b')') {
                    if i >= sig.len() {
                        return Err("signature de structure invalide".into());
                    }
                    let (x, l) = self.value(&sig[i..])?;
                    v.push(x);
                    i += l;
                }
                Ok((Value::Struct(v), i + 1))
            }
            _ => Err(format!("type D-Bus non géré : {}", c as char)),
        }
    }
}

struct Msg {
    kind: u8,
    reply_serial: Option<u32>,
    error: Option<String>,
    sig: String,
    body: Vec<u8>,
}

fn io_err(e: std::io::Error) -> String {
    match e.kind() {
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut => "D-Bus : délai dépassé".into(),
        _ => format!("D-Bus : {e}"),
    }
}

fn uid() -> u32 {
    // sous lapp, le bus filtré attend l'uid de la session (variable fournie par le lanceur)
    if let Some(u) = std::env::var("LAPP_DBUS_UID").ok().and_then(|v| v.parse().ok()) {
        return u;
    }
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|t| t.lines().find(|l| l.starts_with("Uid:")).and_then(|l| l.split_whitespace().nth(1)?.parse().ok()))
        .unwrap_or(0)
}

pub struct Conn {
    s: UnixStream,
    serial: u32,
}

impl Conn {
    pub fn connect() -> Result<Conn, String> {
        let addr = std::env::var("DBUS_SESSION_BUS_ADDRESS")
            .ok()
            .and_then(|a| a.strip_prefix("unix:path=").map(|p| p.split(',').next().unwrap_or(p).to_string()))
            .unwrap_or_else(|| "/run/user/app/bus".to_string());
        let mut s = UnixStream::connect(&addr).map_err(|e| format!("bus D-Bus inaccessible ({addr}) : {e}"))?;
        s.set_read_timeout(Some(Duration::from_millis(800))).map_err(io_err)?;
        s.set_write_timeout(Some(Duration::from_millis(800))).map_err(io_err)?;
        let hex: String = uid().to_string().bytes().map(|b| format!("{b:02x}")).collect();
        s.write_all(format!("\0AUTH EXTERNAL {hex}\r\n").as_bytes()).map_err(io_err)?;
        let mut line = Vec::new();
        loop {
            let mut b = [0u8; 1];
            s.read_exact(&mut b).map_err(io_err)?;
            line.push(b[0]);
            if b[0] == b'\n' || line.len() > 256 {
                break;
            }
        }
        if !line.starts_with(b"OK") {
            return Err(format!("D-Bus : authentification refusée ({})", String::from_utf8_lossy(&line).trim()));
        }
        s.write_all(b"BEGIN\r\n").map_err(io_err)?;
        let mut c = Conn { s, serial: 1 };
        c.call("org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus", "Hello", &[])?;
        Ok(c)
    }

    fn read_msg(&mut self) -> Result<Msg, String> {
        let mut h = [0u8; 16];
        self.s.read_exact(&mut h).map_err(io_err)?;
        if h[0] != b'l' {
            return Err("D-Bus : ordre des octets non géré".into());
        }
        let blen = u32::from_le_bytes([h[4], h[5], h[6], h[7]]) as usize;
        let flen = u32::from_le_bytes([h[12], h[13], h[14], h[15]]) as usize;
        if flen > (1 << 20) || blen > (1 << 24) {
            return Err("D-Bus : message trop grand".into());
        }
        let pad = (8 - flen % 8) % 8;
        let mut rest = vec![0u8; flen + pad + blen];
        self.s.read_exact(&mut rest).map_err(io_err)?;
        let mut r = R { b: &rest[..flen], p: 0 };
        let (mut reply_serial, mut error, mut sig) = (None, None, String::new());
        while r.p < flen {
            r.align(8);
            if r.p >= flen {
                break;
            }
            let code = r.u8()?;
            let s = r.sig()?;
            let (v, _) = r.value(s.as_bytes())?;
            match code {
                4 => error = v.as_str().map(|s| s.to_string()),
                5 => {
                    if let Value::U32(x) = v {
                        reply_serial = Some(x)
                    }
                }
                8 => sig = v.as_str().unwrap_or("").to_string(),
                _ => {}
            }
        }
        Ok(Msg { kind: h[1], reply_serial, error, sig, body: rest[flen + pad..].to_vec() })
    }

    /// Appel de méthode synchrone ; renvoie les valeurs de la réponse.
    pub fn call(&mut self, dest: &str, path: &str, iface: &str, member: &str, args: &[Arg]) -> Result<Vec<Value>, String> {
        let mut body = W::new();
        for a in args {
            a.write(&mut body);
        }
        let sig: String = args.iter().map(|a| a.sig()).collect();
        let mut f = W::new();
        f.field_str(1, "o", path);
        f.field_str(2, "s", iface);
        f.field_str(3, "s", member);
        f.field_str(6, "s", dest);
        if !sig.is_empty() {
            f.field_str(8, "g", &sig);
        }
        let serial = self.serial;
        self.serial += 1;
        let mut msg = vec![b'l', 1, 0, 1];
        msg.extend_from_slice(&(body.b.len() as u32).to_le_bytes());
        msg.extend_from_slice(&serial.to_le_bytes());
        msg.extend_from_slice(&(f.b.len() as u32).to_le_bytes());
        msg.extend_from_slice(&f.b);
        while msg.len() % 8 != 0 {
            msg.push(0);
        }
        msg.extend_from_slice(&body.b);
        self.s.write_all(&msg).map_err(io_err)?;

        loop {
            let m = self.read_msg()?;
            if m.reply_serial != Some(serial) {
                continue; // signal ou réponse à autre chose
            }
            let mut r = R { b: &m.body, p: 0 };
            let mut out = vec![];
            let sg = m.sig.as_bytes();
            let mut i = 0;
            while i < sg.len() {
                let (v, l) = r.value(&sg[i..])?;
                out.push(v);
                i += l;
            }
            return match m.kind {
                2 => Ok(out),
                3 => {
                    let detail = out.first().and_then(|v| v.as_str()).unwrap_or("");
                    Err(format!("{} {}", m.error.unwrap_or_default(), detail).trim().to_string())
                }
                _ => continue,
            };
        }
    }
}