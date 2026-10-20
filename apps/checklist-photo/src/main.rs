use lapp_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

const PURPLE: Color = rgb(0x6750A4);
const PURPLE_CONTAINER: Color = rgb(0xE9DDFF);
const ON_PURPLE_CONTAINER: Color = rgb(0x22005D);
const BG: Color = rgb(0xFFFBFF);
const SURFACE_CARD: Color = rgb(0xF3EDF7);
const TEXT_MAIN: Color = rgb(0x1C1B1E);
const TEXT_MUTED: Color = rgb(0x49454E);
const BORDER_COLOR: Color = rgb(0xCCCCD4);
const GREEN_OK: Color = rgb(0x2E7D32);
const RED_DEL: Color = rgb(0xBA1A1A);

#[derive(Clone, Serialize, Deserialize)]
struct Item {
    id: String,
    name: String,
    steps: Vec<String>,
    last_session: Option<SessionSummary>,
}

#[derive(Clone, Serialize, Deserialize)]
struct SessionSummary {
    date_label: String,
    photos_count: usize,
    total_steps: usize,
    checked_steps: usize,
}

enum Screen {
    Home,
    Edit {
        index: Option<usize>,
        name: String,
        steps: Vec<String>,
        new_step: String,
        active_field: usize,
    },
    Session {
        item_idx: usize,
        phase: SessionPhase,
        photos: Vec<String>,
        checked: Vec<bool>,
    },
}

#[derive(PartialEq, Eq)]
enum SessionPhase {
    Photos,
    Checklist,
}

struct ChecklistPhotoApp {
    items: Vec<Item>,
    screen: Screen,
    scroller: Scroller,
    last_touch_y: f32,
}

impl ChecklistPhotoApp {
    fn new() -> Self {
        let mut app = ChecklistPhotoApp {
            items: Vec::new(),
            screen: Screen::Home,
            scroller: Scroller::new(),
            last_touch_y: 0.0,
        };
        app.load_data();
        if app.items.is_empty() {
            app.items = vec![
                Item {
                    id: "1".to_string(),
                    name: "Airbnb Appartement".to_string(),
                    steps: vec![
                        "Vérifier le salon et les draps".to_string(),
                        "Nettoyer la cuisine et vaisselle".to_string(),
                        "Contrôler la salle d'eau".to_string(),
                        "Déposer les clés dans la boîte".to_string(),
                    ],
                    last_session: Some(SessionSummary {
                        date_label: "12/04 14:30".to_string(),
                        photos_count: 3,
                        total_steps: 4,
                        checked_steps: 4,
                    }),
                },
                Item {
                    id: "2".to_string(),
                    name: "Véhicule de service".to_string(),
                    steps: vec![
                        "Inspection visuelle carrosserie".to_string(),
                        "Niveaux d'huile et lave-glace".to_string(),
                        "Pression des pneus".to_string(),
                        "Relevé kilométrique".to_string(),
                    ],
                    last_session: None,
                },
            ];
            app.save_data();
        }
        app
    }

    fn storage_path() -> std::path::PathBuf {
        lapp_sdk::data_dir().join("checklists.json")
    }

    fn load_data(&mut self) {
        let path = Self::storage_path();
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(items) = serde_json::from_str::<Vec<Item>>(&content) {
                self.items = items;
            }
        }
    }

    fn save_data(&self) {
        let path = Self::storage_path();
        if let Ok(json) = serde_json::to_string_pretty(&self.items) {
            let _ = fs::write(path, json);
        }
    }

    fn format_now() -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let min = (now / 60) % 60;
        let hr = (now / 3600 + 2) % 24;
        let day = ((now / 86400) % 30) + 1;
        let month = ((now / (86400 * 30)) % 12) + 1;
        format!("{:02}/{:02} {:02}:{:02}", day, month, hr, min)
    }
}

impl App for ChecklistPhotoApp {
    fn tick(&mut self, ctx: &Ctx, dt: f32) {
        if self.scroller.update(dt) {
            ctx.animate();
        }
    }

    fn on_back(&mut self, _ctx: &Ctx) -> bool {
        match self.screen {
            Screen::Home => false,
            _ => {
                self.screen = Screen::Home;
                self.scroller.jump_to(0.0);
                true
            }
        }
    }

    fn render(&mut self, ctx: &Ctx, g: &mut Gfx) {
        g.clear(BG);
        let s = ctx.scale();
        let w = ctx.width();
        let h = ctx.height();

        match &self.screen {
            Screen::Home => {
                let hdr_h = 56.0 * s;
                g.rect(0.0, 0.0, w, hdr_h, PURPLE);
                let st_title = Text::new(20.0 * s).bold().color(rgb(0xFFFFFF)).left();
                ctx.text_mid(g, "Checklist Photo", 16.0 * s, hdr_h / 2.0, &st_title);

                let mut content_h = 20.0 * s;
                for item in &self.items {
                    content_h += 108.0 * s;
                }
                content_h += 70.0 * s + 30.0 * s;
                self.scroller.set_content(content_h, h - hdr_h);

                let start_y = hdr_h + 16.0 * s - self.scroller.pos;
                let mut cur_y = start_y;

                for (i, item) in self.items.iter().enumerate() {
                    let card_r = Rect::new(14.0 * s, cur_y, w - 28.0 * s, 96.0 * s);
                    g.rrect(card_r.x, card_r.y + 1.5 * s, card_r.w, card_r.h, 16.0 * s, rgba(0, 0.06));
                    g.rrect(card_r.x, card_r.y, card_r.w, card_r.h, 16.0 * s, SURFACE_CARD);

                    let thumb_r = Rect::new(card_r.x + 10.0 * s, card_r.y + 12.0 * s, 64.0 * s, 72.0 * s);
                    g.rrect(thumb_r.x, thumb_r.y, thumb_r.w, thumb_r.h, 12.0 * s, PURPLE_CONTAINER);
                    let st_initial = Text::new(28.0 * s).bold().color(ON_PURPLE_CONTAINER).center();
                    let init = item.name.chars().next().unwrap_or('C').to_uppercase().to_string();
                    ctx.text_mid(g, &init, thumb_r.x + thumb_r.w / 2.0, thumb_r.y + thumb_r.h / 2.0, &st_initial);

                    let text_x = thumb_r.right() + 12.0 * s;
                    let st_name = Text::new(16.0 * s).bold().color(TEXT_MAIN).left().max_width(card_r.w - 150.0 * s);
                    ctx.text(g, &item.name, text_x, card_r.y + 16.0 * s, &st_name);

                    let st_sub = Text::new(12.0 * s).color(TEXT_MUTED).left();
                    let step_count = format!("{} étape(s)", item.steps.len());
                    ctx.text(g, &step_count, text_x, card_r.y + 38.0 * s, &st_sub);

                    if let Some(ref ses) = item.last_session {
                        let st_badge = Text::new(10.5 * s).bold().color(GREEN_OK).left();
                        let btxt = format!("✓ {} ({} ph)", ses.date_label, ses.photos_count);
                        ctx.text(g, &btxt, text_x, card_r.y + 56.0 * s, &st_badge);
                    } else {
                        let st_badge = Text::new(10.5 * s).color(TEXT_MUTED).left();
                        ctx.text(g, "Aucune session", text_x, card_r.y + 56.0 * s, &st_badge);
                    }

                    let btn_start_r = Rect::new(card_r.right() - 56.0 * s, card_r.y + 16.0 * s, 44.0 * s, 32.0 * s);
                    g.rrect(btn_start_r.x, btn_start_r.y, btn_start_r.w, btn_start_r.h, 8.0 * s, PURPLE);
                    let st_play = Text::new(12.0 * s).bold().color(rgb(0xFFFFFF)).center();
                    ctx.text_mid(g, "GO", btn_start_r.x + btn_start_r.w / 2.0, btn_start_r.y + btn_start_r.h / 2.0, &st_play);

                    let btn_edit_r = Rect::new(card_r.right() - 56.0 * s, card_r.y + 52.0 * s, 44.0 * s, 28.0 * s);
                    g.rrect(btn_edit_r.x, btn_edit_r.y, btn_edit_r.w, btn_edit_r.h, 8.0 * s, rgb(0xFFFFFF));
                    let st_edit = Text::new(11.0 * s).color(PURPLE).center();
                    ctx.text_mid(g, "Édit", btn_edit_r.x + btn_edit_r.w / 2.0, btn_edit_r.y + btn_edit_r.h / 2.0, &st_edit);

                    cur_y += 108.0 * s;
                }

                let add_r = Rect::new(14.0 * s, cur_y, w - 28.0 * s, 54.0 * s);
                g.rrect(add_r.x, add_r.y, add_r.w, add_r.h, 14.0 * s, PURPLE_CONTAINER);
                let st_add = Text::new(15.0 * s).bold().color(ON_PURPLE_CONTAINER).center();
                ctx.text_mid(g, "+ Nouvelle check-list", add_r.x + add_r.w / 2.0, add_r.y + add_r.h / 2.0, &st_add);
            }
            Screen::Edit { index, name, steps, new_step, active_field } => {
                let hdr_title = if index.is_some() { "Modifier" } else { "Nouvel élément" };
                let hdr = ui::header(ctx, g, hdr_title, Some("Accueil"));

                let mut content_h = 240.0 * s;
                content_h += steps.len() as f32 * 48.0 * s;
                self.scroller.set_content(content_h, h - hdr.height);

                let start_y = hdr.height + 14.0 * s - self.scroller.pos;
                let mut cur_y = start_y;

                let st_lbl = Text::new(13.0 * s).bold().color(PURPLE).left();
                ctx.text(g, "NOM DU MODÈLE", 16.0 * s, cur_y, &st_lbl);
                cur_y += 20.0 * s;

                let field_r = Rect::new(14.0 * s, cur_y, w - 28.0 * s, 44.0 * s);
                let is_name_active = *active_field == 0;
                g.rrect(field_r.x, field_r.y, field_r.w, field_r.h, 10.0 * s, rgb(0xFFFFFF));
                g.rrect(field_r.x, field_r.y, field_r.w, field_r.h, 10.0 * s, if is_name_active { PURPLE } else { BORDER_COLOR });
                g.rrect(field_r.x + 1.5 * s, field_r.y + 1.5 * s, field_r.w - 3.0 * s, field_r.h - 3.0 * s, 8.5 * s, rgb(0xFFFFFF));

                let disp_name = if name.is_empty() { "Ex: Airbnb Appartement..." } else { name.as_str() };
                let st_txt = Text::new(14.0 * s).color(if name.is_empty() { TEXT_MUTED } else { TEXT_MAIN }).left();
                ctx.text_mid(g, disp_name, field_r.x + 12.0 * s, field_r.y + field_r.h / 2.0, &st_txt);
                cur_y += 56.0 * s;

                ctx.text(g, "ÉTAPES DE LA CHECK-LIST", 16.0 * s, cur_y, &st_lbl);
                cur_y += 20.0 * s;

                for (i, stp) in steps.iter().enumerate() {
                    let row_r = Rect::new(14.0 * s, cur_y, w - 28.0 * s, 40.0 * s);
                    g.rrect(row_r.x, row_r.y, row_r.w, row_r.h, 8.0 * s, SURFACE_CARD);

                    let st_num = Text::new(12.0 * s).bold().color(PURPLE).left();
                    let num_txt = format!("{}.", i + 1);
                    ctx.text_mid(g, &num_txt, row_r.x + 10.0 * s, row_r.y + row_r.h / 2.0, &st_num);

                    let st_stptxt = Text::new(13.0 * s).color(TEXT_MAIN).left().max_width(row_r.w - 70.0 * s);
                    ctx.text_mid(g, stp, row_r.x + 30.0 * s, row_r.y + row_r.h / 2.0, &st_stptxt);

                    let del_btn_r = Rect::new(row_r.right() - 32.0 * s, row_r.y + 6.0 * s, 26.0 * s, 28.0 * s);
                    let st_x = Text::new(14.0 * s).bold().color(RED_DEL).center();
                    ctx.text_mid(g, "✕", del_btn_r.x + del_btn_r.w / 2.0, del_btn_r.y + del_btn_r.h / 2.0, &st_x);

                    cur_y += 46.0 * s;
                }

                let add_step_r = Rect::new(14.0 * s, cur_y, w - 28.0 * s, 40.0 * s);
                let is_step_active = *active_field == 1;
                g.rrect(add_step_r.x, add_step_r.y, add_step_r.w, add_step_r.h, 8.0 * s, rgb(0xFFFFFF));
                g.rrect(add_step_r.x, add_step_r.y, add_step_r.w, add_step_r.h, 8.0 * s, if is_step_active { PURPLE } else { BORDER_COLOR });
                g.rrect(add_step_r.x + 1.0 * s, add_step_r.y + 1.0 * s, add_step_r.w - 2.0 * s, add_step_r.h - 2.0 * s, 7.0 * s, rgb(0xFFFFFF));
                let disp_step = if new_step.is_empty() { "+ Ajouter une étape..." } else { new_step.as_str() };
                let st_add_step = Text::new(13.0 * s).color(if new_step.is_empty() { TEXT_MUTED } else { TEXT_MAIN }).left();
                ctx.text_mid(g, disp_step, add_step_r.x + 12.0 * s, add_step_r.y + add_step_r.h / 2.0, &st_add_step);
                cur_y += 56.0 * s;

                let save_btn_r = Rect::new(14.0 * s, cur_y, w - 28.0 * s, 46.0 * s);
                g.rrect(save_btn_r.x, save_btn_r.y, save_btn_r.w, save_btn_r.h, 12.0 * s, PURPLE);
                let st_save = Text::new(15.0 * s).bold().color(rgb(0xFFFFFF)).center();
                ctx.text_mid(g, "Enregistrer", save_btn_r.x + save_btn_r.w / 2.0, save_btn_r.y + save_btn_r.h / 2.0, &st_save);
                cur_y += 54.0 * s;

                if index.is_some() {
                    let del_btn_r = Rect::new(14.0 * s, cur_y, w - 28.0 * s, 42.0 * s);
                    g.rrect(del_btn_r.x, del_btn_r.y, del_btn_r.w, del_btn_r.h, 10.0 * s, rgba(0xBA1A1A, 0.12));
                    let st_del = Text::new(14.0 * s).bold().color(RED_DEL).center();
                    ctx.text_mid(g, "Supprimer ce modèle", del_btn_r.x + del_btn_r.w / 2.0, del_btn_r.y + del_btn_r.h / 2.0, &st_del);
                }
            }
            Screen::Session { item_idx, phase, photos, checked } => {
                let item_name = self.items.get(*item_idx).map(|it| it.name.as_str()).unwrap_or("Session");
                let hdr = ui::header(ctx, g, item_name, Some("Quitter"));

                let step_bar_h = 32.0 * s;
                let bar_y = hdr.height;
                g.rect(0.0, bar_y, w, step_bar_h, PURPLE_CONTAINER);
                let step_info = match phase {
                    SessionPhase::Photos => "Étape 1/2 : Photos de contrôle",
                    SessionPhase::Checklist => "Étape 2/2 : Check-list",
                };
                let st_step_info = Text::new(12.5 * s).bold().color(ON_PURPLE_CONTAINER).center();
                ctx.text_mid(g, step_info, w / 2.0, bar_y + step_bar_h / 2.0, &st_step_info);

                let content_top = bar_y + step_bar_h;
                match phase {
                    SessionPhase::Photos => {
                        let mut cur_y = content_top + 18.0 * s;
                        let count_txt = format!("{} photo(s) enregistrée(s)", photos.len());
                        let st_counter = Text::new(15.0 * s).bold().color(TEXT_MAIN).center();
                        ctx.text_mid(g, &count_txt, w / 2.0, cur_y, &st_counter);
                        cur_y += 24.0 * s;

                        let box_r = Rect::new(20.0 * s, cur_y, w - 40.0 * s, 160.0 * s);
                        g.rrect(box_r.x, box_r.y, box_r.w, box_r.h, 16.0 * s, SURFACE_CARD);
                        g.rrect(box_r.x, box_r.y, box_r.w, box_r.h, 16.0 * s, BORDER_COLOR);
                        g.rrect(box_r.x + 1.5 * s, box_r.y + 1.5 * s, box_r.w - 3.0 * s, box_r.h - 3.0 * s, 14.5 * s, SURFACE_CARD);

                        if photos.is_empty() {
                            let st_hint = Text::new(13.0 * s).color(TEXT_MUTED).center();
                            ctx.text_mid(g, "Touchez le bouton caméra", box_r.x + box_r.w / 2.0, box_r.y + box_r.h / 2.0 - 10.0 * s, &st_hint);
                            ctx.text_mid(g, "pour ajouter des photos", box_r.x + box_r.w / 2.0, box_r.y + box_r.h / 2.0 + 10.0 * s, &st_hint);
                        } else {
                            let mut px = box_r.x + 10.0 * s;
                            for (idx, ptxt) in photos.iter().take(4).enumerate() {
                                let thumb = Rect::new(px, box_r.y + 16.0 * s, 50.0 * s, 68.0 * s);
                                g.rrect(thumb.x, thumb.y, thumb.w, thumb.h, 8.0 * s, PURPLE_CONTAINER);
                                let st_p = Text::new(11.0 * s).bold().color(PURPLE).center();
                                let lbl = format!("#{}", idx + 1);
                                ctx.text_mid(g, &lbl, thumb.x + thumb.w / 2.0, thumb.y + 24.0 * s, &st_p);
                                let st_t = Text::new(8.0 * s).color(TEXT_MUTED).center();
                                ctx.text_mid(g, ptxt, thumb.x + thumb.w / 2.0, thumb.y + 48.0 * s, &st_t);
                                px += 56.0 * s;
                            }
                        }

                        let cam_btn_r = Rect::new(w / 2.0 - 32.0 * s, box_r.bottom() + 18.0 * s, 64.0 * s, 64.0 * s);
                        g.circle(cam_btn_r.x + cam_btn_r.w / 2.0, cam_btn_r.y + cam_btn_r.h / 2.0, 32.0 * s, PURPLE);
                        g.circle(cam_btn_r.x + cam_btn_r.w / 2.0, cam_btn_r.y + cam_btn_r.h / 2.0, 26.0 * s, rgb(0xFFFFFF));
                        g.circle(cam_btn_r.x + cam_btn_r.w / 2.0, cam_btn_r.y + cam_btn_r.h / 2.0, 20.0 * s, PURPLE);

                        let next_btn_r = Rect::new(16.0 * s, h - 56.0 * s, w - 32.0 * s, 44.0 * s);
                        g.rrect(next_btn_r.x, next_btn_r.y, next_btn_r.w, next_btn_r.h, 12.0 * s, PURPLE);
                        let st_btn = Text::new(15.0 * s).bold().color(rgb(0xFFFFFF)).center();
                        ctx.text_mid(g, "Continuer vers la check-list ›", next_btn_r.x + next_btn_r.w / 2.0, next_btn_r.y + next_btn_r.h / 2.0, &st_btn);
                    }
                    SessionPhase::Checklist => {
                        let steps = self.items.get(*item_idx).map(|it| &it.steps[..]).unwrap_or(&[]);
                        let mut content_h = steps.len() as f32 * 60.0 * s + 80.0 * s;
                        self.scroller.set_content(content_h, h - content_top - 60.0 * s);

                        let mut cur_y = content_top + 14.0 * s - self.scroller.pos;
                        for (i, stp) in steps.iter().enumerate() {
                            let is_chk = checked.get(i).copied().unwrap_or(false);
                            let row_r = Rect::new(14.0 * s, cur_y, w - 28.0 * s, 50.0 * s);
                            g.rrect(row_r.x, row_r.y, row_r.w, row_r.h, 12.0 * s, if is_chk { PURPLE_CONTAINER } else { SURFACE_CARD });

                            let box_chk_r = Rect::new(row_r.x + 12.0 * s, row_r.y + 14.0 * s, 22.0 * s, 22.0 * s);
                            g.rrect(box_chk_r.x, box_chk_r.y, box_chk_r.w, box_chk_r.h, 6.0 * s, if is_chk { PURPLE } else { BORDER_COLOR });
                            if is_chk {
                                let st_v = Text::new(14.0 * s).bold().color(rgb(0xFFFFFF)).center();
                                ctx.text_mid(g, "✓", box_chk_r.x + box_chk_r.w / 2.0, box_chk_r.y + box_chk_r.h / 2.0, &st_v);
                            } else {
                                g.rrect(box_chk_r.x + 2.0 * s, box_chk_r.y + 2.0 * s, box_chk_r.w - 4.0 * s, box_chk_r.h - 4.0 * s, 4.0 * s, rgb(0xFFFFFF));
                            }

                            let st_lbl = Text::new(14.0 * s).bold().color(if is_chk { ON_PURPLE_CONTAINER } else { TEXT_MAIN }).left().max_width(row_r.w - 56.0 * s);
                            ctx.text_mid(g, stp, row_r.x + 44.0 * s, row_r.y + row_r.h / 2.0, &st_lbl);

                            cur_y += 58.0 * s;
                        }

                        let fin_btn_r = Rect::new(16.0 * s, h - 56.0 * s, w - 32.0 * s, 44.0 * s);
                        g.rrect(fin_btn_r.x, fin_btn_r.y, fin_btn_r.w, fin_btn_r.h, 12.0 * s, PURPLE);
                        let st_fin = Text::new(15.0 * s).bold().color(rgb(0xFFFFFF)).center();
                        ctx.text_mid(g, "Valider et Terminer", fin_btn_r.x + fin_btn_r.w / 2.0, fin_btn_r.y + fin_btn_r.h / 2.0, &st_fin);
                    }
                }
            }
        }
    }

    fn event(&mut self, ctx: &Ctx, ev: Event) {
        let s = ctx.scale();
        let w = ctx.width();
        let h = ctx.height();

        match ev {
            Event::Drag { dy, .. } | Event::Wheel { dy } => {
                self.scroller.drag(dy);
            }
            Event::DragEnd { vy, .. } => {
                self.scroller.fling(vy);
            }
            Event::Down { y, .. } => {
                self.scroller.stop();
                self.last_touch_y = y;
            }
            Event::Key(k) => match &mut self.screen {
                Screen::Edit { name, new_step, active_field, steps, .. } => match k {
                    Key::Char(c) => {
                        if *active_field == 0 {
                            name.push(c);
                        } else {
                            new_step.push(c);
                        }
                    }
                    Key::Back => {
                        if *active_field == 0 {
                            name.pop();
                        } else {
                            new_step.pop();
                        }
                    }
                    Key::Enter => {
                        if *active_field == 1 && !new_step.is_empty() {
                            steps.push(new_step.clone());
                            new_step.clear();
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            Event::Tap { x, y } => match &mut self.screen {
                Screen::Home => {
                    let hdr_h = 56.0 * s;
                    let start_y = hdr_h + 16.0 * s - self.scroller.pos;
                    let mut cur_y = start_y;

                    for (i, item) in self.items.iter().enumerate() {
                        let card_r = Rect::new(14.0 * s, cur_y, w - 28.0 * s, 96.0 * s);
                        let btn_start_r = Rect::new(card_r.right() - 56.0 * s, card_r.y + 16.0 * s, 44.0 * s, 32.0 * s);
                        let btn_edit_r = Rect::new(card_r.right() - 56.0 * s, card_r.y + 52.0 * s, 44.0 * s, 28.0 * s);

                        if btn_start_r.contains(x, y) || (card_r.contains(x, y) && !btn_edit_r.contains(x, y)) {
                            let step_count = item.steps.len();
                            self.screen = Screen::Session {
                                item_idx: i,
                                phase: SessionPhase::Photos,
                                photos: Vec::new(),
                                checked: vec![false; step_count],
                            };
                            self.scroller.jump_to(0.0);
                            return;
                        } else if btn_edit_r.contains(x, y) {
                            self.screen = Screen::Edit {
                                index: Some(i),
                                name: item.name.clone(),
                                steps: item.steps.clone(),
                                new_step: String::new(),
                                active_field: 0,
                            };
                            self.scroller.jump_to(0.0);
                            return;
                        }
                        cur_y += 108.0 * s;
                    }

                    let add_r = Rect::new(14.0 * s, cur_y, w - 28.0 * s, 54.0 * s);
                    if add_r.contains(x, y) {
                        self.screen = Screen::Edit {
                            index: None,
                            name: String::new(),
                            steps: vec!["Contrôle d'entrée".to_string(), "État des lieux photos".to_string()],
                            new_step: String::new(),
                            active_field: 0,
                        };
                        self.scroller.jump_to(0.0);
                    }
                }
                Screen::Edit { index, name, steps, new_step, active_field } => {
                    let hdr_h = 44.0 * s;
                    if y < hdr_h && x < w * 0.35 {
                        self.screen = Screen::Home;
                        self.scroller.jump_to(0.0);
                        return;
                    }

                    let start_y = hdr_h + 14.0 * s - self.scroller.pos;
                    let mut cur_y = start_y + 20.0 * s;
                    let field_r = Rect::new(14.0 * s, cur_y, w - 28.0 * s, 44.0 * s);
                    if field_r.contains(x, y) {
                        *active_field = 0;
                        return;
                    }

                    cur_y += 56.0 * s + 20.0 * s;
                    let mut to_delete = None;
                    for (i, _) in steps.iter().enumerate() {
                        let row_r = Rect::new(14.0 * s, cur_y, w - 28.0 * s, 40.0 * s);
                        let del_btn_r = Rect::new(row_r.right() - 32.0 * s, row_r.y + 6.0 * s, 26.0 * s, 28.0 * s);
                        if del_btn_r.contains(x, y) {
                            to_delete = Some(i);
                            break;
                        }
                        cur_y += 46.0 * s;
                    }
                    if let Some(d) = to_delete {
                        steps.remove(d);
                        return;
                    }

                    let add_step_r = Rect::new(14.0 * s, cur_y, w - 28.0 * s, 40.0 * s);
                    if add_step_r.contains(x, y) {
                        *active_field = 1;
                        if new_step.is_empty() {
                            new_step.push_str("Étape ");
                            new_step.push_str(&(steps.len() + 1).to_string());
                        }
                        return;
                    }
                    cur_y += 56.0 * s;

                    let save_btn_r = Rect::new(14.0 * s, cur_y, w - 28.0 * s, 46.0 * s);
                    if save_btn_r.contains(x, y) {
                        if !new_step.is_empty() {
                            steps.push(new_step.clone());
                            new_step.clear();
                        }
                        let final_name = if name.trim().is_empty() { "Nouvelle checklist".to_string() } else { name.trim().to_string() };
                        if let Some(idx) = *index {
                            if let Some(it) = self.items.get_mut(idx) {
                                it.name = final_name;
                                it.steps = steps.clone();
                            }
                        } else {
                            self.items.push(Item {
                                id: (self.items.len() + 1).to_string(),
                                name: final_name,
                                steps: steps.clone(),
                                last_session: None,
                            });
                        }
                        self.save_data();
                        ctx.toast("Modèle enregistré !");
                        self.screen = Screen::Home;
                        self.scroller.jump_to(0.0);
                        return;
                    }
                    cur_y += 54.0 * s;

                    if let Some(idx) = *index {
                        let del_btn_r = Rect::new(14.0 * s, cur_y, w - 28.0 * s, 42.0 * s);
                        if del_btn_r.contains(x, y) {
                            self.items.remove(idx);
                            self.save_data();
                            ctx.toast("Modèle supprimé");
                            self.screen = Screen::Home;
                            self.scroller.jump_to(0.0);
                            return;
                        }
                    }
                }
                Screen::Session { item_idx, phase, photos, checked } => {
                    let hdr_h = 44.0 * s;
                    if y < hdr_h && x < w * 0.35 {
                        self.screen = Screen::Home;
                        self.scroller.jump_to(0.0);
                        return;
                    }

                    let step_bar_h = 32.0 * s;
                    let content_top = hdr_h + step_bar_h;
                    match phase {
                        SessionPhase::Photos => {
                            let box_r = Rect::new(20.0 * s, content_top + 42.0 * s, w - 40.0 * s, 160.0 * s);
                            let cam_btn_r = Rect::new(w / 2.0 - 32.0 * s, box_r.bottom() + 18.0 * s, 64.0 * s, 64.0 * s);
                            if cam_btn_r.contains(x, y) {
                                let time_label = format!("{:02}s", (photos.len() * 12 + 10) % 60);
                                photos.push(time_label);
                                ctx.toast(format!("Photo {} capturée !", photos.len()));
                                return;
                            }
                            let next_btn_r = Rect::new(16.0 * s, h - 56.0 * s, w - 32.0 * s, 44.0 * s);
                            if next_btn_r.contains(x, y) {
                                *phase = SessionPhase::Checklist;
                                self.scroller.jump_to(0.0);
                            }
                        }
                        SessionPhase::Checklist => {
                            let steps_count = self.items.get(*item_idx).map(|it| it.steps.len()).unwrap_or(0);
                            let mut cur_y = content_top + 14.0 * s - self.scroller.pos;
                            for i in 0..steps_count {
                                let row_r = Rect::new(14.0 * s, cur_y, w - 28.0 * s, 50.0 * s);
                                if row_r.contains(x, y) {
                                    if let Some(c) = checked.get_mut(i) {
                                        *c = !*c;
                                    }
                                    return;
                                }
                                cur_y += 58.0 * s;
                            }
                            let fin_btn_r = Rect::new(16.0 * s, h - 56.0 * s, w - 32.0 * s, 44.0 * s);
                            if fin_btn_r.contains(x, y) {
                                let total = checked.len();
                                let valid = checked.iter().filter(|&&v| v).count();
                                let summary = SessionSummary {
                                    date_label: Self::format_now(),
                                    photos_count: photos.len(),
                                    total_steps: total,
                                    checked_steps: valid,
                                };
                                if let Some(it) = self.items.get_mut(*item_idx) {
                                    it.last_session = Some(summary);
                                }
                                self.save_data();
                                ctx.toast("Session terminée et enregistrée !");
                                self.screen = Screen::Home;
                                self.scroller.jump_to(0.0);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let cfg = Config::new("Checklist Photo").size(272, 480);
    run(cfg, ChecklistPhotoApp::new());
}
