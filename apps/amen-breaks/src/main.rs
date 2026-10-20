use lapp_sdk::audio::AudioOut;
use lapp_sdk::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;

const ST_PLAYING: u8 = 0;
const ST_DONE: u8 = 1;
const ST_ERROR: u8 = 2;

/// Lecture d'un WAV dans un fil : decodage (hound) puis envoi PCM au serveur audio
/// PulseAudio / PipeWire de l'hote (AudioOut). `aplay` n'est plus utilise : sur l'appareil il n'y a
/// pas de carte ALSA directe, seulement le serveur de son.
struct Player {
    stop: Arc<AtomicBool>,
    state: Arc<AtomicU8>,
}

impl Player {
    fn start(path: PathBuf) -> Player {
        let stop = Arc::new(AtomicBool::new(false));
        let state = Arc::new(AtomicU8::new(ST_PLAYING));
        let (stop2, state2) = (stop.clone(), state.clone());
        thread::spawn(move || {
            let res = play_wav(&path, &stop2);
            if let Err(e) = &res {
                eprintln!("Amen Breaks : {e}");
            }
            state2.store(if res.is_ok() { ST_DONE } else { ST_ERROR }, Ordering::Relaxed);
        });
        Player { stop, state }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn load_wav(path: &Path) -> Result<(u32, u8, Vec<i16>), String> {
    let mut r = hound::WavReader::open(path).map_err(|e| format!("WAV illisible : {e}"))?;
    let spec = r.spec();
    if spec.channels == 0 || spec.channels > 2 {
        return Err(format!("{} canaux non pris en charge", spec.channels));
    }
    let bits = spec.bits_per_sample;
    let samples: Vec<i16> = match (spec.sample_format, bits) {
        (hound::SampleFormat::Float, _) => r
            .samples::<f32>()
            .map(|s| s.map(|v| (v.clamp(-1.0, 1.0) * 32767.0) as i16))
            .collect::<Result<_, _>>(),
        (hound::SampleFormat::Int, 8) => r
            .samples::<i8>()
            .map(|s| s.map(|v| (v as i16) << 8))
            .collect::<Result<_, _>>(),
        (hound::SampleFormat::Int, 16) => r.samples::<i16>().collect::<Result<_, _>>(),
        (hound::SampleFormat::Int, 24) | (hound::SampleFormat::Int, 32) => r
            .samples::<i32>()
            .map(|s| s.map(|v| (v >> (bits - 16)) as i16))
            .collect::<Result<_, _>>(),
        _ => return Err(format!("WAV {bits} bits non pris en charge")),
    }
    .map_err(|e| format!("WAV illisible : {e}"))?;
    Ok((spec.sample_rate, spec.channels as u8, samples))
}

fn play_wav(path: &Path, stop: &AtomicBool) -> Result<(), String> {
    let (rate, ch, data) = load_wav(path)?;
    let mut out = AudioOut::new("Amen Breaks", rate, ch).latency_ms(100);
    let chunk = ((rate as usize / 10) * ch as usize).max(ch as usize);
    for part in data.chunks(chunk) {
        if stop.load(Ordering::Relaxed) {
            out.flush();
            return Ok(());
        }
        if !out.write(part) {
            return Err("sortie audio indisponible (serveur PulseAudio/PipeWire injoignable)".into());
        }
    }
    if !stop.load(Ordering::Relaxed) {
        out.drain();
    }
    Ok(())
}

struct AmenApp {
    files: Vec<String>,
    scroll: Scroller,
    playing_file: Option<String>,
    player: Option<Player>,
    child: Option<Child>, // repli pour les fichiers non WAV (mp3)
}

impl AmenApp {
    fn new() -> Self {
        let mut files = Vec::new();

        if let Ok(entries) = fs::read_dir(lapp_sdk::app_path("assets")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        let ext_lower = ext.to_lowercase();
                        if ext_lower == "wav" || ext_lower == "mp3" {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                files.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        files.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        AmenApp {
            files,
            scroll: Scroller::new(),
            playing_file: None,
            player: None,
            child: None,
        }
    }

    fn stop_playback(&mut self) {
        self.player = None; // Drop : demande l'arret du fil de lecture
        if let Some(mut old) = self.child.take() {
            let _ = old.kill();
        }
    }

    fn play(&mut self, name: &str) {
        self.stop_playback();
        let full_path = lapp_sdk::app_path(&format!("assets/{}", name));
        let is_wav = full_path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("wav"));
        if is_wav {
            self.player = Some(Player::start(full_path));
        } else {
            self.child = lapp_sdk::audio::play_file(&full_path);
        }
        self.playing_file = Some(name.to_string());
    }
}

impl App for AmenApp {
    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        if self.scroll.update(dt) {
            ctx.animate();
        }
        if let Some(ref mut child) = self.child {
            if let Ok(Some(_)) = child.try_wait() {
                self.child = None;
                self.playing_file = None;
            }
        }
        let st = self.player.as_ref().map(|p| p.state.load(Ordering::Relaxed));
        match st {
            Some(ST_DONE) => {
                self.player = None;
                self.playing_file = None;
            }
            Some(ST_ERROR) => {
                self.player = None;
                self.playing_file = None;
                ctx.toast("Lecture impossible (voir les logs)".to_string());
            }
            _ => {}
        }
    }

    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        g.clear(ctx.theme.bg);

        let head = ui::header(ctx, g, "Amen Breaks", None);
        let s = ctx.scale();
        let row_h = 48.0 * s;
        let view_h = (ctx.height() - head.height).max(1.0);
        let content_h = self.files.len() as f32 * row_h;

        self.scroll.set_content(content_h, view_h);

        let start_y = head.height;
        let scroll_pos = self.scroll.pos;

        let first_idx = (scroll_pos / row_h).floor() as usize;
        let visible_count = (view_h / row_h).ceil() as usize + 2;
        let last_idx = (first_idx + visible_count).min(self.files.len());

        for i in first_idx..last_idx {
            let y = start_y + (i as f32 * row_h) - scroll_pos;
            let r = Rect::new(0.0, y, ctx.width(), row_h);
            let name = &self.files[i];
            let is_playing = self.playing_file.as_deref() == Some(name.as_str());

            let detail = if is_playing { "▶ Lecture" } else { "" };
            ui::list_row(ctx, g, r, name, Some(detail), is_playing);
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        let s = ctx.scale();
        let head_h = 44.0 * s;

        match ev {
            Event::Down { .. } => self.scroll.stop(),
            Event::Drag { dy, .. } | Event::Wheel { dy } => self.scroll.drag(dy),
            Event::DragEnd { vy, .. } => self.scroll.fling(vy),
            Event::Tap { x: _, y } => {
                if y > head_h {
                    let row_h = 48.0 * s;
                    let idx = ((y - head_h + self.scroll.pos) / row_h).floor() as usize;
                    if idx < self.files.len() {
                        let name = self.files[idx].clone();
                        self.play(&name);
                        ctx.toast(format!("▶ {}", name));
                    }
                }
            }
            _ => {}
        }
    }
}

impl Drop for AmenApp {
    fn drop(&mut self) {
        self.stop_playback();
    }
}

fn main() {
    run(Config::new("Amen Breaks"), AmenApp::new());
