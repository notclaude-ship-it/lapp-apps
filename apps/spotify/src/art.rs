//! Pochettes : téléchargement HTTP (sans TLS : i.scdn.co sert aussi en http), décodage JPEG, mise à l'échelle.
//!
//! Tout se fait dans un fil d'arrière-plan ; l'interface demande `Art::want(url)` à chaque image et reçoit
//! `Some(image)` quand elle est prête (sinon `None` : elle affiche une pochette de remplacement).

use std::io::{Cursor, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use lapp_sdk::Pixmap;

/// Image décodée, RGB 8 bits.
pub struct Raw {
    pub w: usize,
    pub h: usize,
    pub rgb: Vec<u8>,
}

enum Slot {
    Empty,
    Loading(String),
    Ready(String, Arc<Raw>),
    Failed(String),
}

#[derive(Clone)]
pub struct Art {
    slot: Arc<Mutex<Slot>>,
}

fn lock(m: &Mutex<Slot>) -> std::sync::MutexGuard<'_, Slot> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl Art {
    pub fn new() -> Art {
        Art { slot: Arc::new(Mutex::new(Slot::Empty)) }
    }

    /// Pochette de `url` si elle est prête ; lance le chargement si c'est une nouvelle URL.
    pub fn want(&self, url: &str) -> Option<Arc<Raw>> {
        let mut s = lock(&self.slot);
        match &*s {
            Slot::Ready(u, r) if u == url => return Some(r.clone()),
            Slot::Loading(u) | Slot::Failed(u) if u == url => return None,
            _ => {}
        }
        if url.is_empty() {
            *s = Slot::Empty;
            return None;
        }
        *s = Slot::Loading(url.to_string());
        drop(s);
        let slot = self.slot.clone();
        let url = url.to_string();
        std::thread::spawn(move || {
            let res = fetch(&url).and_then(|b| decode(&b));
            let mut s = lock(&slot);
            // résultat périmé si une autre pochette a été demandée entre-temps
            if matches!(&*s, Slot::Loading(u) if *u == url) {
                *s = match res {
                    Ok(raw) => Slot::Ready(url, Arc::new(raw)),
                    Err(_) => Slot::Failed(url),
                };
            }
        });
        None
    }

    /// Charge la pochette tout de suite (captures d'écran hors écran).
    pub fn preload_blocking(&self, url: &str) {
        let res = fetch(url).and_then(|b| decode(&b));
        *lock(&self.slot) = match res {
            Ok(raw) => Slot::Ready(url.to_string(), Arc::new(raw)),
            Err(_) => Slot::Failed(url.to_string()),
        };
    }
}

/// GET http:// avec suivi de quelques redirections (https:// est réécrit en http://).
fn fetch(url: &str) -> Result<Vec<u8>, String> {
    let mut url = url.replacen("https://", "http://", 1);
    for _ in 0..4 {
        let rest = url.strip_prefix("http://").ok_or("URL non http")?.to_string();
        let (hostport, path) = match rest.find('/') {
            Some(i) => (rest[..i].to_string(), rest[i..].to_string()),
            None => (rest.clone(), "/".to_string()),
        };
        let host = hostport.split(':').next().unwrap_or("").to_string();
        let addr = if hostport.contains(':') { hostport.clone() } else { format!("{hostport}:80") };
        let sa = addr
            .to_socket_addrs()
            .map_err(|e| format!("DNS {host} : {e}"))?
            .next()
            .ok_or("DNS : aucune adresse")?;
        let mut s = TcpStream::connect_timeout(&sa, Duration::from_secs(6)).map_err(|e| format!("connexion : {e}"))?;
        let _ = s.set_read_timeout(Some(Duration::from_secs(8)));
        let _ = s.set_write_timeout(Some(Duration::from_secs(8)));
        let req = format!(
            "GET {path} HTTP/1.0\r\nHost: {host}\r\nUser-Agent: lapp-spotify/1.0\r\nAccept: image/jpeg\r\nConnection: close\r\n\r\n"
        );
        s.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        s.take(8 << 20).read_to_end(&mut buf).map_err(|e| e.to_string())?;
        let end = buf.windows(4).position(|w| w == b"\r\n\r\n").ok_or("réponse HTTP invalide")?;
        let head = String::from_utf8_lossy(&buf[..end]).to_string();
        let status: u32 = head.split_whitespace().nth(1).and_then(|c| c.parse().ok()).unwrap_or(0);
        match status {
            200 => return Ok(buf[end + 4..].to_vec()),
            301 | 302 | 303 | 307 | 308 => {
                let loc = head
                    .lines()
                    .find_map(|l| {
                        let (k, v) = l.split_once(':')?;
                        k.eq_ignore_ascii_case("location").then(|| v.trim().to_string())
                    })
                    .ok_or("redirection sans Location")?;
                url = if loc.starts_with('/') { format!("http://{hostport}{loc}") } else { loc.replacen("https://", "http://", 1) };
            }
            c => return Err(format!("HTTP {c}")),
        }
    }
    Err("trop de redirections".into())
}

fn decode(bytes: &[u8]) -> Result<Raw, String> {
    let mut d = jpeg_decoder::Decoder::new(Cursor::new(bytes));
    let px = d.decode().map_err(|e| e.to_string())?;
    let info = d.info().ok_or("JPEG sans en-tête")?;
    let (w, h) = (info.width as usize, info.height as usize);
    let rgb = match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => px,
        jpeg_decoder::PixelFormat::L8 => px.iter().flat_map(|&g| [g, g, g]).collect(),
        _ => return Err("format JPEG non géré".into()),
    };
    if w == 0 || h == 0 || rgb.len() < w * h * 3 {
        return Err("JPEG invalide".into());
    }
    Ok(Raw { w, h, rgb })
}

/// Rogne au carré central, réduit à `size` × `size` par moyenne de zones, et arrondit les coins (`radius` px).
pub fn cover_pixmap(raw: &Raw, size: u32, radius: f32) -> Pixmap {
    let n = size.max(1) as usize;
    let side = raw.w.min(raw.h);
    let (ox, oy) = ((raw.w - side) / 2, (raw.h - side) / 2);
    let mut pm = Pixmap::new(n as u32, n as u32).expect("pixmap");
    let data = pm.data_mut();
    for y in 0..n {
        let sy0 = oy + y * side / n;
        let sy1 = (oy + (y + 1) * side / n).max(sy0 + 1);
        for x in 0..n {
            let sx0 = ox + x * side / n;
            let sx1 = (ox + (x + 1) * side / n).max(sx0 + 1);
            let (mut r, mut g, mut b, mut c) = (0u32, 0u32, 0u32, 0u32);
            for sy in sy0..sy1 {
                let row = sy * raw.w;
                for sx in sx0..sx1 {
                    let i = (row + sx) * 3;
                    r += raw.rgb[i] as u32;
                    g += raw.rgb[i + 1] as u32;
                    b += raw.rgb[i + 2] as u32;
                    c += 1;
                }
            }
            let a = corner_coverage(x, y, n, radius);
            let o = (y * n + x) * 4;
            data[o] = ((r / c) as f32 * a) as u8;
            data[o + 1] = ((g / c) as f32 * a) as u8;
            data[o + 2] = ((b / c) as f32 * a) as u8;
            data[o + 3] = (a * 255.0) as u8;
        }
    }
    pm
}

/// Couverture (0 à 1) du pixel (x, y) d'un carré de côté `n` aux coins arrondis de rayon `r`.
fn corner_coverage(x: usize, y: usize, n: usize, r: f32) -> f32 {
    let (fx, fy, nf) = (x as f32 + 0.5, y as f32 + 0.5, n as f32);
    let cx = if fx < r {
        r
    } else if fx > nf - r {
        nf - r
    } else {
        return 1.0;
    };
    let cy = if fy < r {
        r
    } else if fy > nf - r {
        nf - r
    } else {
        return 1.0;
    };
    let d = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt();
    (r - d + 0.5).clamp(0.0, 1.0)
}