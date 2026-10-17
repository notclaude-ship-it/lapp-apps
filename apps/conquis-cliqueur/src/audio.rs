//! Son de la vidéo : PCM S16 48 kHz stéréo (`assets/video.pcm`) envoyé au serveur PulseAudio/PipeWire
//! de l'hôte avec libpulse-simple, chargée à l'exécution (permission « audio » du manifeste).
//!
//! La lecture tourne dans son propre fil. `pa_simple_write` bloque quand le tampon est plein, donc le
//! son avance en temps réel ; l'image attend que le son soit prêt (`state() == READY`) pour partir en même temps.

use lapp_sdk::app_path;
use std::ffi::{c_char, c_int, c_void, CStr};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;

pub const PENDING: u8 = 0;
pub const READY: u8 = 1;
pub const FAILED: u8 = 2;

const RATE: u32 = 48_000;
const FRAME_BYTES: usize = 4; // S16 stéréo
const CHUNK: usize = (RATE as usize / 10) * FRAME_BYTES; // 100 ms

extern "C" {
    fn dlopen(file: *const c_char, flag: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
}

#[repr(C)]
struct PaSpec {
    format: c_int,
    rate: u32,
    channels: u8,
}

#[repr(C)]
struct PaAttr {
    maxlength: u32,
    tlength: u32,
    prebuf: u32,
    minreq: u32,
    fragsize: u32,
}

struct PaApi {
    new: unsafe extern "C" fn(*const c_char, *const c_char, c_int, *const c_char, *const c_char, *const PaSpec, *const c_void, *const PaAttr, *mut c_int) -> *mut c_void,
    write: unsafe extern "C" fn(*mut c_void, *const c_void, usize, *mut c_int) -> c_int,
    flush: unsafe extern "C" fn(*mut c_void, *mut c_int) -> c_int,
    drain: unsafe extern "C" fn(*mut c_void, *mut c_int) -> c_int,
    free: unsafe extern "C" fn(*mut c_void),
}

fn pa_api() -> Option<&'static PaApi> {
    static API: std::sync::OnceLock<Option<PaApi>> = std::sync::OnceLock::new();
    API.get_or_init(|| unsafe {
        let h = dlopen(c"libpulse-simple.so.0".as_ptr(), 2);
        if h.is_null() {
            eprintln!("conquis-cliqueur : libpulse-simple introuvable : pas de son");
            return None;
        }
        let f = |n: &CStr| dlsym(h, n.as_ptr());
        let (a, b, c, d, e) = (f(c"pa_simple_new"), f(c"pa_simple_write"), f(c"pa_simple_flush"), f(c"pa_simple_drain"), f(c"pa_simple_free"));
        if [a, b, c, d, e].iter().any(|p| p.is_null()) {
            eprintln!("conquis-cliqueur : libpulse-simple incomplète : pas de son");
            return None;
        }
        Some(PaApi {
            new: std::mem::transmute(a),
            write: std::mem::transmute(b),
            flush: std::mem::transmute(c),
            drain: std::mem::transmute(d),
            free: std::mem::transmute(e),
        })
    })
    .as_ref()
}

/// Un son en cours de lecture. Se coupe quand il est détruit ou sur `stop()`.
pub struct Sound {
    stop: Arc<AtomicBool>,
    state: Arc<AtomicU8>,
    detached: bool,
}

impl Sound {
    /// Lance la lecture de `assets/video.pcm` dans un fil.
    pub fn start_video() -> Sound {
        let stop = Arc::new(AtomicBool::new(false));
        let state = Arc::new(AtomicU8::new(PENDING));
        let (st, sa) = (stop.clone(), state.clone());
        let spawned = std::thread::Builder::new().name("son".into()).spawn(move || play(&st, &sa));
        if spawned.is_err() {
            state.store(FAILED, Ordering::SeqCst);
        }
        Sound { stop, state, detached: false }
    }

    /// `PENDING` (en préparation), `READY` (le son joue) ou `FAILED` (pas de son, l'image continue seule).
    pub fn state(&self) -> u8 {
        self.state.load(Ordering::SeqCst)
    }

    /// Laisse le son finir tout seul (il ne sera plus coupé à la destruction).
    pub fn detach(mut self) {
        self.detached = true;
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

impl Drop for Sound {
    fn drop(&mut self) {
        if !self.detached {
            self.stop();
        }
    }
}

fn play(stop: &AtomicBool, state: &AtomicU8) {
    let path = app_path("assets/video.pcm");
    let data = match std::fs::read(&path) {
        Ok(d) if !d.is_empty() => d,
        r => {
            eprintln!("conquis-cliqueur : son illisible ({}) : {:?}", path.display(), r.err());
            state.store(FAILED, Ordering::SeqCst);
            return;
        }
    };
    let Some(api) = pa_api() else {
        state.store(FAILED, Ordering::SeqCst);
        return;
    };
    unsafe {
        let spec = PaSpec { format: 3, rate: RATE, channels: 2 }; // PA_SAMPLE_S16LE
        let attr = PaAttr { maxlength: u32::MAX, tlength: (RATE as usize * FRAME_BYTES * 15 / 100) as u32, prebuf: u32::MAX, minreq: u32::MAX, fragsize: u32::MAX };
        let mut err: c_int = 0;
        let pa = (api.new)(std::ptr::null(), c"Conquis Cliqueur".as_ptr(), 1, std::ptr::null(), c"video".as_ptr(), &spec, std::ptr::null(), &attr, &mut err);
        if pa.is_null() {
            eprintln!("conquis-cliqueur : sortie audio indisponible (pulse : {err})");
            state.store(FAILED, Ordering::SeqCst);
            return;
        }
        let mut played = 0usize;
        for (i, chunk) in data.chunks(CHUNK).enumerate() {
            if stop.load(Ordering::SeqCst) {
                break;
            }
            let mut e: c_int = 0;
            if (api.write)(pa, chunk.as_ptr() as *const c_void, chunk.len() - chunk.len() % FRAME_BYTES, &mut e) < 0 {
                eprintln!("conquis-cliqueur : écriture audio échouée (pulse : {e})");
                break;
            }
            played = i + 1;
            // Le tampon (150 ms) est plein après ~2 morceaux : la lecture démarre, l'image peut partir.
            if i == 1 {
                state.store(READY, Ordering::SeqCst);
            }
        }
        state.store(READY, Ordering::SeqCst); // courte piste, ou arrêt avant le 2e morceau
        let mut e: c_int = 0;
        if stop.load(Ordering::SeqCst) || played == 0 {
            (api.flush)(pa, &mut e);
        } else {
            (api.drain)(pa, &mut e);
        }
        (api.free)(pa);
    }
}