use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use lapp_sdk::prelude::*;

#[derive(Clone, Debug)]
pub struct PlayerInfo {
    pub is_running: bool,
    pub status: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub volume: i32,
}

impl Default for PlayerInfo {
    fn default() -> Self {
        Self {
            is_running: false,
            status: "Vérification...".into(),
            title: "Aucune lecture".into(),
            artist: "-".into(),
            album: "-".into(),
            volume: 100,
        }
    }
}

pub enum Action {
    PlayPause,
    Next,
    Prev,
    VolumeUp,
    VolumeDown,
    StartDaemon,
    StopDaemon,
    Refresh,
}

fn run_shell(sh: &str) -> Option<String> {
    Command::new("sh").arg("-c").arg(sh).output().ok().and_then(|o| {
        if o.status.success() {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        } else {
            None
        }
    })
}

fn check_running() -> bool {
    run_shell("pgrep -x spotifyd >/dev/null || pgrep -f spotifyd >/dev/null").is_some()
        || run_shell("systemctl --user is-active spotifyd 2>/dev/null")
            .map(|s| s == "active")
            .unwrap_or(false)
}

fn query_info() -> PlayerInfo {
    let running = check_running();
    if !running {
        return PlayerInfo {
            is_running: false,
            status: "Démon inactif".into(),
            title: "Spotifyd arrêté".into(),
            artist: "Cliquez sur Démarrer ci-dessous".into(),
            album: "".into(),
            volume: 0,
        };
    }

    let status = run_shell("playerctl -p spotifyd status 2>/dev/null")
        .or_else(|| run_shell("playerctl status 2>/dev/null"))
        .unwrap_or_else(|| "Connecté".into());

    let title = run_shell("playerctl -p spotifyd metadata title 2>/dev/null")
        .or_else(|| run_shell("playerctl metadata title 2>/dev/null"))
        .unwrap_or_else(|| "Titre en attente".into());

    let artist = run_shell("playerctl -p spotifyd metadata artist 2>/dev/null")
        .or_else(|| run_shell("playerctl metadata artist 2>/dev/null"))
        .unwrap_or_else(|| "Spotifyd actif".into());

    let album = run_shell("playerctl -p spotifyd metadata album 2>/dev/null")
        .or_else(|| run_shell("playerctl metadata album 2>/dev/null"))
        .unwrap_or_default();

    let vol_str = run_shell("playerctl -p spotifyd volume 2>/dev/null")
        .unwrap_or_else(|| "1.0".into());
    let volume = (vol_str.parse::<f32>().unwrap_or(1.0) * 100.0).round() as i32;

    PlayerInfo {
        is_running: true,
        status,
        title,
        artist,
        album,
        volume: volume.clamp(0, 100),
    }
}

fn exec_action(act: Action) {
    match act {
        Action::PlayPause => {
            let _ = run_shell("playerctl -p spotifyd play-pause 2>/dev/null || dbus-send --reply-timeout=400 --print-reply --dest=org.mpris.MediaPlayer2.spotifyd /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player.PlayPause 2>/dev/null");
        }
        Action::Next => {
            let _ = run_shell("playerctl -p spotifyd next 2>/dev/null || dbus-send --reply-timeout=400 --print-reply --dest=org.mpris.MediaPlayer2.spotifyd /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player.Next 2>/dev/null");
        }
        Action::Prev => {
            let _ = run_shell("playerctl -p spotifyd previous 2>/dev/null || dbus-send --reply-timeout=400 --print-reply --dest=org.mpris.MediaPlayer2.spotifyd /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player.Previous 2>/dev/null");
        }
        Action::VolumeUp => {
            let _ = run_shell("playerctl -p spotifyd volume 0.05+ 2>/dev/null");
        }
        Action::VolumeDown => {
            let _ = run_shell("playerctl -p spotifyd volume 0.05- 2>/dev/null");
        }
        Action::StartDaemon => {
            let _ = run_shell("systemctl --user start spotifyd 2>/dev/null || nohup spotifyd --no-daemon >/dev/null 2>&1 &");
        }
        Action::StopDaemon => {
            let _ = run_shell("systemctl --user stop spotifyd 2>/dev/null || killall spotifyd 2>/dev/null");
        }
        Action::Refresh => {}
    }
}

fn spawn_worker(tx_info: Sender<PlayerInfo>, rx_act: Receiver<Action>) {
    thread::spawn(move || {
        let mut last_poll = Instant::now() - Duration::from_secs(10);
        loop {
            match rx_act.recv_timeout(Duration::from_millis(800)) {
                Ok(act) => {
                    exec_action(act);
                    thread::sleep(Duration::from_millis(150));
                    let info = query_info();
                    let _ = tx_info.send(info);
                    last_poll = Instant::now();
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if last_poll.elapsed() >= Duration::from_millis(1500) {
                        let info = query_info();
                        let _ = tx_info.send(info);
                        last_poll = Instant::now();
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });
}

struct SpotifyApp {
    info: PlayerInfo,
    tx_act: Sender<Action>,
    rx_info: Receiver<PlayerInfo>,
    r_prev: Rect,
    r_play: Rect,
    r_next: Rect,
    r_vol_down: Rect,
    r_vol_up: Rect,
    r_start: Rect,
    r_stop: Rect,
    r_refresh: Rect,
}

impl SpotifyApp {
    fn new() -> Self {
        let (tx_info, rx_info) = channel();
        let (tx_act, rx_act) = channel();
        spawn_worker(tx_info, rx_act);

        Self {
            info: PlayerInfo::default(),
            tx_act,
            rx_info,
            r_prev: Rect::default(),
            r_play: Rect::default(),
            r_next: Rect::default(),
            r_vol_down: Rect::default(),
            r_vol_up: Rect::default(),
            r_start: Rect::default(),
            r_stop: Rect::default(),
            r_refresh: Rect::default(),
        }
    }
}

impl App for SpotifyApp {
    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        let w = ctx.width();
        let s = ctx.scale();

        g.clear(rgb(0x121212));

        let header_h = 44.0 * s;
        g.rect(0.0, 0.0, w, header_h, rgb(0x181818));
        g.rect(0.0, header_h - 1.0, w, 1.0, rgb(0x282828));
        let st_title = Text::new(16.0 * s).bold().color(rgb(0x1DB954)).center();
        ctx.text_mid(g, "SPOTIFYD", w / 2.0, header_h / 2.0, &st_title);

        let dot_color = if self.info.is_running { rgb(0x1DB954) } else { rgb(0x888888) };
        g.circle(w - 18.0 * s, header_h / 2.0, 4.5 * s, dot_color);

        let card_x = 12.0 * s;
        let card_y = header_h + 10.0 * s;
        let card_w = w - 24.0 * s;
        let card_h = 128.0 * s;
        g.rrect(card_x, card_y, card_w, card_h, 10.0 * s, rgb(0x1E1E1E));

        let st_status = Text::new(11.0 * s).bold().color(rgb(0x1DB954)).left().max_width(card_w - 20.0 * s);
        let status_text = format!("● {}", self.info.status.to_uppercase());
        ctx.text_mid(g, &status_text, card_x + 12.0 * s, card_y + 18.0 * s, &st_status);

        let st_song = Text::new(15.0 * s).bold().color(rgb(0xFFFFFF)).left().max_width(card_w - 24.0 * s);
        ctx.text_mid(g, &self.info.title, card_x + 12.0 * s, card_y + 44.0 * s, &st_song);

        let st_artist = Text::new(13.0 * s).color(rgb(0xB3B3B3)).left().max_width(card_w - 24.0 * s);
        ctx.text_mid(g, &self.info.artist, card_x + 12.0 * s, card_y + 70.0 * s, &st_artist);

        if !self.info.album.is_empty() {
            let st_album = Text::new(11.0 * s).color(rgb(0x727272)).left().max_width(card_w - 24.0 * s);
            ctx.text_mid(g, &self.info.album, card_x + 12.0 * s, card_y + 92.0 * s, &st_album);
        }

        let ctrl_y = card_y + card_h + 14.0 * s;
        let btn_h = 48.0 * s;
        let side_w = 56.0 * s;
        let play_w = 84.0 * s;
        let space = (card_w - side_w * 2.0 - play_w) / 2.0;

        self.r_prev = Rect::new(card_x, ctrl_y + 4.0 * s, side_w, btn_h - 8.0 * s);
        self.r_play = Rect::new(card_x + side_w + space, ctrl_y, play_w, btn_h);
        self.r_next = Rect::new(card_x + side_w + space + play_w + space, ctrl_y + 4.0 * s, side_w, btn_h - 8.0 * s);

        g.rrect(self.r_prev.x, self.r_prev.y, self.r_prev.w, self.r_prev.h, 8.0 * s, rgb(0x282828));
        let st_ctrl = Text::new(18.0 * s).bold().color(rgb(0xFFFFFF)).center();
        ctx.text_mid(g, "|◄", self.r_prev.center().0, self.r_prev.center().1, &st_ctrl);

        g.rrect(self.r_play.x, self.r_play.y, self.r_play.w, self.r_play.h, 24.0 * s, rgb(0x1DB954));
        let play_icon = if self.info.status.to_lowercase().contains("play") { "⏸" } else { "▶" };
        let st_play = Text::new(22.0 * s).bold().color(rgb(0x000000)).center();
        ctx.text_mid(g, play_icon, self.r_play.center().0, self.r_play.center().1, &st_play);

        g.rrect(self.r_next.x, self.r_next.y, self.r_next.w, self.r_next.h, 8.0 * s, rgb(0x282828));
        ctx.text_mid(g, "►|", self.r_next.center().0, self.r_next.center().1, &st_ctrl);

        let vol_y = ctrl_y + btn_h + 16.0 * s;
        let vol_h = 36.0 * s;
        let vol_btn_w = 38.0 * s;
        let vol_gauge_w = card_w - vol_btn_w * 2.0 - 16.0 * s;

        self.r_vol_down = Rect::new(card_x, vol_y, vol_btn_w, vol_h);
        let r_gauge = Rect::new(card_x + vol_btn_w + 8.0 * s, vol_y, vol_gauge_w, vol_h);
        self.r_vol_up = Rect::new(card_x + vol_btn_w + 8.0 * s + vol_gauge_w + 8.0 * s, vol_y, vol_btn_w, vol_h);

        g.rrect(self.r_vol_down.x, self.r_vol_down.y, self.r_vol_down.w, self.r_vol_down.h, 6.0 * s, rgb(0x282828));
        ctx.text_mid(g, "-", self.r_vol_down.center().0, self.r_vol_down.center().1, &st_ctrl);

        g.rrect(r_gauge.x, r_gauge.y, r_gauge.w, r_gauge.h, 6.0 * s, rgb(0x1A1A1A));
        let pct = (self.info.volume as f32 / 100.0).clamp(0.0, 1.0);
        if pct > 0.0 {
            g.rrect(r_gauge.x, r_gauge.y, r_gauge.w * pct, r_gauge.h, 6.0 * s, rgb(0x333333));
        }
        let st_vol = Text::new(12.0 * s).bold().color(rgb(0xE0E0E0)).center();
        let vol_label = format!("Volume : {}%", self.info.volume);
        ctx.text_mid(g, &vol_label, r_gauge.center().0, r_gauge.center().1, &st_vol);

        g.rrect(self.r_vol_up.x, self.r_vol_up.y, self.r_vol_up.w, self.r_vol_up.h, 6.0 * s, rgb(0x282828));
        ctx.text_mid(g, "+", self.r_vol_up.center().0, self.r_vol_up.center().1, &st_ctrl);

        let mgt_y = vol_y + vol_h + 18.0 * s;
        let mgt_h = 38.0 * s;
        let half_w = (card_w - 10.0 * s) / 2.0;
        self.r_start = Rect::new(card_x, mgt_y, half_w, mgt_h);
        self.r_stop = Rect::new(card_x + half_w + 10.0 * s, mgt_y, half_w, mgt_h);

        let st_btn = Text::new(13.0 * s).bold().color(rgb(0xFFFFFF)).center();
        g.rrect(self.r_start.x, self.r_start.y, self.r_start.w, self.r_start.h, 6.0 * s, rgb(0x146C36));
        ctx.text_mid(g, "Démarrer", self.r_start.center().0, self.r_start.center().1, &st_btn);

        g.rrect(self.r_stop.x, self.r_stop.y, self.r_stop.w, self.r_stop.h, 6.0 * s, rgb(0x8B1E1E));
        ctx.text_mid(g, "Arrêter", self.r_stop.center().0, self.r_stop.center().1, &st_btn);

        let ref_y = mgt_y + mgt_h + 10.0 * s;
        self.r_refresh = Rect::new(card_x, ref_y, card_w, 34.0 * s);
        g.rrect(self.r_refresh.x, self.r_refresh.y, self.r_refresh.w, self.r_refresh.h, 6.0 * s, rgb(0x222222));
        let st_ref = Text::new(12.0 * s).color(rgb(0x999999)).center();
        ctx.text_mid(g, "Rafraîchir statut", self.r_refresh.center().0, self.r_refresh.center().1, &st_ref);
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        if let Event::Tap { x, y } = ev {
            if self.r_play.contains(x, y) {
                let _ = self.tx_act.send(Action::PlayPause);
                ctx.toast("Lecture / Pause");
            } else if self.r_prev.contains(x, y) {
                let _ = self.tx_act.send(Action::Prev);
                ctx.toast("Précédent");
            } else if self.r_next.contains(x, y) {
                let _ = self.tx_act.send(Action::Next);
                ctx.toast("Suivant");
            } else if self.r_vol_down.contains(x, y) {
                let _ = self.tx_act.send(Action::VolumeDown);
                ctx.toast("Volume -");
            } else if self.r_vol_up.contains(x, y) {
                let _ = self.tx_act.send(Action::VolumeUp);
                ctx.toast("Volume +");
            } else if self.r_start.contains(x, y) {
                let _ = self.tx_act.send(Action::StartDaemon);
                ctx.toast("Lancement de spotifyd...");
            } else if self.r_stop.contains(x, y) {
                let _ = self.tx_act.send(Action::StopDaemon);
                ctx.toast("Arrêt de spotifyd...");
            } else if self.r_refresh.contains(x, y) {
                let _ = self.tx_act.send(Action::Refresh);
                ctx.toast("Actualisation...");
            }
        }
    }

    fn tick(&mut self, ctx: &Ctx, _dt: f32) {
        let mut updated = false;
        while let Ok(info) = self.rx_info.try_recv() {
            self.info = info;
            updated = true;
        }
        if updated {
            ctx.animate();
        }
        ctx.redraw_in(Duration::from_millis(600));
    }

    fn icon(&mut self, ctx: &Ctx, g: &mut Gfx, size: u32) {
        let f = size as f32;
        g.rrect(0.0, 0.0, f, f, f * 0.22, rgb(0x1DB954));
        let st = Text::new(f * 0.52).bold().color(rgb(0xFFFFFF)).center();
        ctx.text_mid(g, "♫", f / 2.0, f * 0.5, &st);
    }
}

fn main() {
    run(Config::new("Spotify").size(272, 480), SpotifyApp::new());
}
