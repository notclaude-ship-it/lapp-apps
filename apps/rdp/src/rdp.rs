//! Connexion RDP (IronRDP, E/S bloquantes) dans un thread dédié.
//!
//! L'interface ne touche jamais au réseau : elle envoie des `Operation` (souris, clavier) au thread et lit
//! l'image du bureau distant dans `Shared` (protégée par un mutex, verrouillée brièvement).

use std::io::{self, ErrorKind, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use ironrdp::connector::sspi::generator::NetworkRequest;
use ironrdp::connector::sspi::network_client::NetworkClient;
use ironrdp::connector::sspi::{Error as SspiError, ErrorKind as SspiErrorKind};
use ironrdp::connector::{self, Credentials};
use ironrdp::graphics::image_processing::PixelFormat;
use ironrdp::pdu::gcc::KeyboardType;
use ironrdp::pdu::rdp::capability_sets::MajorPlatformType;
use ironrdp::pdu::rdp::client_info::{PerformanceFlags, TimezoneInfo};
use ironrdp::session::image::DecodedImage;
use ironrdp::session::{ActiveStageBuilder, ActiveStageOutput};
use ironrdp_blocking::Framed;
use sha2::{Digest, Sha256};

pub use ironrdp::input::{MouseButton, MousePosition, Operation, Scancode, WheelRotations};
use ironrdp::input::Database;

// ---------------------------------------------------------------------------------------------------------
// Touches : codes de balayage (scancodes) RDP
// ---------------------------------------------------------------------------------------------------------

pub const SC_ESC: u16 = 0x01;
pub const SC_BACKSPACE: u16 = 0x0E;
pub const SC_TAB: u16 = 0x0F;
pub const SC_ENTER: u16 = 0x1C;
pub const SC_CTRL: u16 = 0x1D;
pub const SC_SHIFT: u16 = 0x2A;
pub const SC_ALT: u16 = 0x38;
pub const SC_SPACE: u16 = 0x39;
pub const SC_WIN: u16 = 0xE05B;
pub const SC_HOME: u16 = 0xE047;
pub const SC_UP: u16 = 0xE048;
pub const SC_LEFT: u16 = 0xE04B;
pub const SC_RIGHT: u16 = 0xE04D;
pub const SC_END: u16 = 0xE04F;
pub const SC_DOWN: u16 = 0xE050;
pub const SC_DEL: u16 = 0xE053;

pub fn sc(code: u16) -> Scancode {
    Scancode::from_u16(code)
}

/// Code de balayage de la touche qui porte la lettre `c` sur le clavier DISTANT (pour les raccourcis Ctrl+C, etc.).
pub fn char_scancode(c: char, azerty: bool) -> Option<u16> {
    let c = c.to_ascii_lowercase();
    let rows: [(&str, u16); 3] = if azerty {
        [("azertyuiop", 0x10), ("qsdfghjklm", 0x1E), ("wxcvbn", 0x2C)]
    } else {
        [("qwertyuiop", 0x10), ("asdfghjkl", 0x1E), ("zxcvbnm", 0x2C)]
    };
    for (keys, base) in rows {
        if let Some(i) = keys.find(c) {
            return Some(base + i as u16);
        }
    }
    if !azerty {
        if let Some(d) = c.to_digit(10) {
            return Some(if d == 0 { 0x0B } else { 0x01 + d as u16 });
        }
    }
    None
}

// ---------------------------------------------------------------------------------------------------------
// Types partagés avec l'interface
// ---------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum State {
    Connecting,
    Connected,
    /// Session terminée proprement (déconnexion, fermeture côté serveur).
    Ended(String),
    /// Échec ; `new_fp` = empreinte du certificat quand elle diffère de celle mémorisée.
    Failed { msg: String, new_fp: Option<String> },
}

pub struct Shared {
    pub state: State,
    pub image: Option<DecodedImage>,
    /// Empreinte SHA-256 du certificat TLS du serveur (hexadécimal).
    pub fp: Option<String>,
    /// Image factice (mode `--demo`, captures d'écran).
    pub demo: Option<(Vec<u8>, usize, usize)>,
}

pub struct Params {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub domain: String,
    pub width: u16,
    pub height: u16,
    pub azerty: bool,
    /// Exiger NLA (CredSSP) : refuse les serveurs qui ne proposent que TLS « simple ».
    pub nla_only: bool,
    pub known_fp: Option<String>,
}

enum Cmd {
    Ops(Vec<Operation>),
    Disconnect,
}

pub struct Session {
    pub shared: Arc<Mutex<Shared>>,
    pub frame: Arc<AtomicU64>,
    tx: Sender<Cmd>,
}

pub fn lock(m: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl Session {
    pub fn start(p: Params) -> Session {
        let shared = Arc::new(Mutex::new(Shared { state: State::Connecting, image: None, fp: None, demo: None }));
        let frame = Arc::new(AtomicU64::new(0));
        let (tx, rx) = channel();
        let (s2, f2) = (shared.clone(), frame.clone());
        std::thread::Builder::new()
            .name("rdp".into())
            .spawn(move || worker(p, s2, f2, rx))
            .expect("thread rdp");
        Session { shared, frame, tx }
    }

    /// Session factice déjà « connectée » avec une image de test (aucun réseau).
    pub fn demo(w: usize, h: usize) -> Session {
        let mut px = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) * 4;
                let (mut r, mut g, mut b) = (20 + (x * 60 / w) as u8, 90 + (y * 80 / h) as u8, 160u8);
                // « fenêtres » et barre des tâches
                if y > h - 40 {
                    (r, g, b) = (30, 30, 36);
                } else if (60..w / 2 + 60).contains(&x) && (50..h / 2).contains(&y) {
                    (r, g, b) = if y < 80 { (40, 40, 48) } else { (245, 245, 245) };
                    if y > 100 && y < 112 && x > 80 && x < w / 2 { (r, g, b) = (60, 60, 70); }
                } else if x > w / 3 && (h / 2 + 30..h - 80).contains(&y) && x < w - 60 {
                    (r, g, b) = (250, 250, 235);
                }
                px[i..i + 4].copy_from_slice(&[r, g, b, 255]);
            }
        }
        let shared = Arc::new(Mutex::new(Shared { state: State::Connected, image: None, fp: None, demo: Some((px, w, h)) }));
        let (tx, _rx) = channel();
        Session { shared, frame: Arc::new(AtomicU64::new(1)), tx }
    }

    pub fn send(&self, ops: Vec<Operation>) {
        if !ops.is_empty() {
            let _ = self.tx.send(Cmd::Ops(ops));
        }
    }

    pub fn frame(&self) -> u64 {
        self.frame.load(Ordering::Relaxed)
    }

    pub fn state(&self) -> State {
        lock(&self.shared).state.clone()
    }

    /// Donne à `f` les pixels RGBA du bureau (données, largeur, hauteur, pas en octets).
    pub fn with_frame<R>(&self, f: impl FnOnce(&[u8], usize, usize, usize) -> R) -> Option<R> {
        let g = lock(&self.shared);
        if let Some((px, w, h)) = &g.demo {
            return Some(f(px, *w, *h, *w * 4));
        }
        let img = g.image.as_ref()?;
        Some(f(img.data(), img.width() as usize, img.height() as usize, img.stride()))
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.tx.send(Cmd::Disconnect);
    }
}

// ---------------------------------------------------------------------------------------------------------
// Thread de connexion
// ---------------------------------------------------------------------------------------------------------

enum ConnErr {
    Msg(String),
    CertChanged(String),
}

fn chain(e: &dyn std::error::Error) -> String {
    let mut s = e.to_string();
    let mut cur = e.source();
    while let Some(c) = cur {
        let t = c.to_string();
        if !s.contains(&t) {
            s.push_str(" : ");
            s.push_str(&t);
        }
        cur = c.source();
    }
    s
}

fn friendly(raw: String) -> String {
    let l = raw.to_lowercase();
    if l.contains("logon") || l.contains("credssp") || l.contains("authenticat") || l.contains("0xc000006d") {
        format!("Authentification refusée (identifiants ou NLA). {raw}")
    } else if l.contains("negotiat") || l.contains("security protocol") || l.contains("protocol") {
        format!("Négociation RDP refusée par le serveur (essayez sans « Exiger NLA »). {raw}")
    } else {
        raw
    }
}

fn ce<E: std::error::Error>(what: &'static str) -> impl FnOnce(E) -> ConnErr {
    move |e| ConnErr::Msg(friendly(format!("{what} : {}", chain(&e))))
}

fn msg(s: impl Into<String>) -> ConnErr {
    ConnErr::Msg(s.into())
}

/// Pas de réseau pour Kerberos : l'authentification se fait en NTLM.
struct NoNet;

impl NetworkClient for NoNet {
    fn send(&self, _request: &NetworkRequest) -> Result<Vec<u8>, SspiError> {
        Err(SspiError::new(SspiErrorKind::NoAuthenticatingAuthority, "Kerberos non pris en charge (NTLM uniquement)"))
    }
}

fn worker(p: Params, shared: Arc<Mutex<Shared>>, frame: Arc<AtomicU64>, rx: Receiver<Cmd>) {
    let r = run(&p, &shared, &frame, &rx);
    let mut s = lock(&shared);
    s.state = match r {
        Ok(reason) => State::Ended(reason),
        Err(ConnErr::Msg(m)) => State::Failed { msg: m, new_fp: None },
        Err(ConnErr::CertChanged(fp)) => State::Failed {
            msg: "Le certificat du serveur n'est plus celui mémorisé. Si vous n'avez rien changé sur ce PC, quelqu'un intercepte peut-être la connexion.".into(),
            new_fp: Some(fp),
        },
    };
    frame.fetch_add(1, Ordering::Relaxed);
}

fn build_config(p: &Params) -> connector::Config {
    connector::Config {
        credentials: Credentials::UsernamePassword { username: p.user.clone(), password: p.pass.clone() },
        domain: if p.domain.trim().is_empty() { None } else { Some(p.domain.trim().to_string()) },
        enable_tls: !p.nla_only,
        enable_credssp: true,
        keyboard_type: KeyboardType::IbmEnhanced,
        keyboard_subtype: 0,
        keyboard_layout: if p.azerty { 0x040C } else { 0x0409 },
        keyboard_functional_keys_count: 12,
        ime_file_name: String::new(),
        dig_product_id: String::new(),
        desktop_size: connector::DesktopSize { width: p.width, height: p.height },
        bitmap: None,
        client_build: 0,
        client_name: "lapp-rdp".to_owned(),
        client_dir: "C:\\Windows\\System32\\mstscax.dll".to_owned(),
        platform: MajorPlatformType::UNIX,
        enable_server_pointer: false, // le serveur dessine le curseur dans l'image
        request_data: None,
        autologon: false,
        enable_audio_playback: false,
        compression_type: None,
        pointer_software_rendering: true,
        multitransport_flags: None,
        performance_flags: PerformanceFlags::default(),
        desktop_scale_factor: 0,
        hardware_id: None,
        license_cache: None,
        timezone_info: TimezoneInfo::default(),
        alternate_shell: String::new(),
        work_dir: String::new(),
    }
}

type Tls = rustls::StreamOwned<rustls::ClientConnection, TcpStream>;

fn tls_upgrade(stream: TcpStream, host: &str) -> Result<(Tls, Vec<u8>, String), ConnErr> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(ce("TLS"))?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(danger::AcceptAny))
        .with_no_client_auth();
    // CredSSP ne supporte pas la reprise de session TLS.
    config.resumption = rustls::client::Resumption::disabled();

    let name = rustls::pki_types::ServerName::try_from(host.to_owned()).map_err(ce("Nom du serveur"))?;
    let conn = rustls::ClientConnection::new(Arc::new(config), name).map_err(ce("TLS"))?;
    let mut tls = rustls::StreamOwned::new(conn, stream);
    while tls.conn.is_handshaking() {
        tls.conn.complete_io(&mut tls.sock).map_err(ce("Poignée de main TLS"))?;
    }
    tls.flush().map_err(ce("TLS"))?;

    let cert = tls.conn.peer_certificates().and_then(|c| c.first()).ok_or_else(|| msg("Certificat du serveur absent"))?;
    let fp: String = Sha256::digest(cert.as_ref()).iter().map(|b| format!("{b:02x}")).collect();
    let spk = {
        use x509_cert::der::Decode as _;
        let c = x509_cert::Certificate::from_der(cert.as_ref()).map_err(ce("Certificat"))?;
        c.tbs_certificate
            .subject_public_key_info
            .subject_public_key
            .as_bytes()
            .ok_or_else(|| msg("Clé publique du certificat illisible"))?
            .to_owned()
    };
    Ok((tls, spk, fp))
}

mod danger {
    use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
    use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
    use rustls::{DigitallySignedStruct, Error, SignatureScheme};

    /// Les serveurs RDP ont presque toujours un certificat auto-signé : on ne valide pas la chaîne, on mémorise
    /// l'empreinte à la première connexion et on refuse un changement (voir `run`).
    #[derive(Debug)]
    pub struct AcceptAny;

    impl ServerCertVerifier for AcceptAny {
        fn verify_server_cert(&self, _: &CertificateDer<'_>, _: &[CertificateDer<'_>], _: &ServerName<'_>, _: &[u8], _: UnixTime) -> Result<ServerCertVerified, Error> {
            Ok(ServerCertVerified::assertion())
        }
        fn verify_tls12_signature(&self, _: &[u8], _: &CertificateDer<'_>, _: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> {
            Ok(HandshakeSignatureValid::assertion())
        }
        fn verify_tls13_signature(&self, _: &[u8], _: &CertificateDer<'_>, _: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> {
            Ok(HandshakeSignatureValid::assertion())
        }
        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            vec![
                SignatureScheme::RSA_PKCS1_SHA1,
                SignatureScheme::ECDSA_SHA1_Legacy,
                SignatureScheme::RSA_PKCS1_SHA256,
                SignatureScheme::ECDSA_NISTP256_SHA256,
                SignatureScheme::RSA_PKCS1_SHA384,
                SignatureScheme::ECDSA_NISTP384_SHA384,
                SignatureScheme::RSA_PKCS1_SHA512,
                SignatureScheme::ECDSA_NISTP521_SHA512,
                SignatureScheme::RSA_PSS_SHA256,
                SignatureScheme::RSA_PSS_SHA384,
                SignatureScheme::RSA_PSS_SHA512,
                SignatureScheme::ED25519,
                SignatureScheme::ED448,
            ]
        }
    }
}

fn run(p: &Params, shared: &Arc<Mutex<Shared>>, frame: &Arc<AtomicU64>, rx: &Receiver<Cmd>) -> Result<String, ConnErr> {
    let addr = (p.host.as_str(), p.port)
        .to_socket_addrs()
        .map_err(|e| msg(format!("Adresse « {} » introuvable : {e}", p.host)))?
        .next()
        .ok_or_else(|| msg(format!("Adresse « {} » introuvable", p.host)))?;
    let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(10))
        .map_err(|e| msg(format!("Impossible de joindre {}:{} : {e}", p.host, p.port)))?;
    let _ = tcp.set_nodelay(true);
    let _ = tcp.set_read_timeout(Some(Duration::from_secs(30)));
    let _ = tcp.set_write_timeout(Some(Duration::from_secs(30)));
    let client_addr = tcp.local_addr().map_err(ce("Socket"))?;

    let mut framed = Framed::new(tcp);
    let mut connector = connector::ClientConnector::new(build_config(p), client_addr);
    let should_upgrade = ironrdp_blocking::connect_begin(&mut framed, &mut connector).map_err(ce("Négociation"))?;

    let tcp = framed.into_inner_no_leftover();
    let (tls, spk, fp) = tls_upgrade(tcp, &p.host)?;
    if let Some(known) = &p.known_fp {
        if *known != fp {
            return Err(ConnErr::CertChanged(fp));
        }
    }
    lock(shared).fp = Some(fp);

    let upgraded = ironrdp_blocking::mark_as_upgraded(should_upgrade, &mut connector);
    let mut framed = Framed::new(tls);
    let result = ironrdp_blocking::connect_finalize(upgraded, connector, &mut framed, &mut NoNet, p.host.clone().into(), spk, None)
        .map_err(ce("Connexion"))?;

    {
        let mut s = lock(shared);
        s.image = Some(DecodedImage::new(PixelFormat::RgbA32, result.desktop_size.width, result.desktop_size.height));
        s.state = State::Connected;
    }
    frame.fetch_add(1, Ordering::Relaxed);

    // Lectures courtes : le thread alterne entre les paquets du serveur et les commandes de l'interface.
    let _ = framed.get_inner_mut().0.sock.set_read_timeout(Some(Duration::from_millis(12)));

    let mut stage = ActiveStageBuilder {
        static_channels: result.static_channels,
        user_channel_id: result.user_channel_id,
        io_channel_id: result.io_channel_id,
        message_channel_id: result.message_channel_id,
        share_id: result.share_id,
        compression_type: result.compression_type,
        enable_server_pointer: result.enable_server_pointer,
        pointer_software_rendering: result.pointer_software_rendering,
    }
    .build();
    let mut db = Database::new();

    loop {
        // 1. commandes de l'interface (souris, clavier)
        loop {
            match rx.try_recv() {
                Ok(Cmd::Ops(ops)) => {
                    let events = db.apply(ops);
                    let outs = {
                        let mut g = lock(shared);
                        let img = g.image.as_mut().ok_or_else(|| msg("Image absente"))?;
                        stage.process_fastpath_input(img, &events).map_err(ce("Entrée"))?
                    };
                    if handle(outs, &mut framed, frame)? {
                        return Ok("Session fermée par le serveur".into());
                    }
                }
                Ok(Cmd::Disconnect) | Err(TryRecvError::Disconnected) => {
                    if let Ok(outs) = stage.graceful_shutdown() {
                        let _ = handle(outs, &mut framed, frame);
                    }
                    return Ok("Déconnecté".into());
                }
                Err(TryRecvError::Empty) => break,
            }
        }

        // 2. paquets du serveur
        match framed.read_pdu() {
            Ok((action, payload)) => {
                let outs = {
                    let mut g = lock(shared);
                    let img = g.image.as_mut().ok_or_else(|| msg("Image absente"))?;
                    stage.process(img, action, &payload).map_err(ce("Session"))?
                };
                for o in &outs {
                    match o {
                        ActiveStageOutput::Terminate(reason) => return Ok(reason.description()),
                        ActiveStageOutput::DeactivateAll => {
                            return Err(msg("Le serveur a demandé une réactivation de la session (non prise en charge). Reconnectez-vous."));
                        }
                        _ => {}
                    }
                }
                handle(outs, &mut framed, frame)?;
            }
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(e) if matches!(e.kind(), ErrorKind::UnexpectedEof | ErrorKind::ConnectionReset | ErrorKind::BrokenPipe) => {
                return Ok("Connexion fermée par le serveur".into());
            }
            Err(e) => return Err(msg(format!("Lecture : {}", chain(&e)))),
        }
    }
}

/// Envoie les réponses au serveur et signale les mises à jour d'image. `true` = la session est terminée.
fn handle(outs: Vec<ActiveStageOutput>, framed: &mut Framed<Tls>, frame: &AtomicU64) -> Result<bool, ConnErr> {
    let mut gfx = false;
    let mut end = false;
    for o in outs {
        match o {
            ActiveStageOutput::ResponseFrame(f) => framed.write_all(&f).map_err(|e: io::Error| msg(format!("Écriture : {e}")))?,
            ActiveStageOutput::GraphicsUpdate(_) => gfx = true,
            ActiveStageOutput::Terminate(_) => end = true,
            _ => {}
        }
    }
    if gfx {
        frame.fetch_add(1, Ordering::Relaxed);
    }
    Ok(end)
}