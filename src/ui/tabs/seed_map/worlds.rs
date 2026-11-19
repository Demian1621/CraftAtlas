use eframe::egui::{self, Color32, CornerRadius, Margin, Pos2, Rect, RichText, Stroke, Vec2};

use crate::anim::ToastQueue;
use crate::assets::ItemAtlas;
use crate::audio::{AudioEngine, SoundCue};
use crate::data::models::Language;
use crate::data::seed_engine::{Dimension, SeedMapEngine};
use crate::data::worlds_nbt::{format_relative_time_ms, DetectedWorld};
use crate::i18n::{t, UiKey};
use crate::state::{AppState, SavedSeedEntry, SeedBookTab, SeedHistoryEntry};
use crate::ui::theme::{filter_chip, McPalette};

use super::{activate_local_world, activate_saved_seed, format_tp_xyz, save_active_seed_bookmark};

pub(super) fn render_player_skin_sidebar_card(
    ui: &mut egui::Ui,
    state: &mut AppState,
    engine: &SeedMapEngine,
    palette: McPalette,
    audio: &AudioEngine,
    toasts: &mut ToastQueue,
    lang: Language,
) {
    let active_info: Option<(String, Option<[f64; 3]>, Dimension, String)> =
        state.active_world_for_seed(engine.seed).map(|w| {
            (
                w.level_name.clone(),
                w.player_pos,
                w.player_dimension(),
                w.last_played_label(lang),
            )
        });
    let face_tex_id = state.ensure_player_face_tex(ui.ctx()).id();

    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (face_rect, _) =
                    ui.allocate_exact_size(Vec2::splat(32.0), egui::Sense::hover());
                ui.painter().rect_stroke(
                    face_rect.expand(1.5),
                    CornerRadius::same(4),
                    Stroke::new(1.5_f32, palette.accent_gold),
                    egui::StrokeKind::Outside,
                );
                ui.painter().image(
                    face_tex_id,
                    face_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(
                        RichText::new(t(lang, UiKey::SeedSkinCardTitle))
                            .strong()
                            .size(12.0)
                            .color(palette.accent_gold),
                    );
                    ui.label(
                        RichText::new(t(lang, UiKey::PlayerNotLiveBadge))
                            .size(9.5)
                            .color(Color32::from_rgb(255, 205, 115)),
                    );
                });
            });

            if let Some((wname, maybe_pos, pdim, last_lbl)) = active_info {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("{} • {}", wname, last_lbl))
                        .size(10.5)
                        .color(palette.text_muted),
                );
                if let Some(pos) = maybe_pos {
                    let px = pos[0].round() as i32;
                    let py = pos[1].round() as i32;
                    let pz = pos[2].round() as i32;
                    ui.label(
                        RichText::new(format!(
                            "{}: X {}  Y {}  Z {}",
                            pdim.label(lang),
                            px,
                            py,
                            pz
                        ))
                        .monospace()
                        .size(10.5)
                        .color(palette.accent_primary),
                    );
                    if pdim != state.map_dim {
                        ui.label(
                            RichText::new(t(lang, UiKey::PlayerOtherDimHint))
                                .size(9.5)
                                .color(palette.accent_enchant),
                        );
                    }
                    ui.add_space(3.0);
                    ui.horizontal(|ui| {
                        if ui.small_button(t(lang, UiKey::PlayerJumpBtn)).clicked() {
                            state.map_dim = pdim;
                            state.jump_map_to(pos[0], pos[2]);
                            audio.play(SoundCue::Click);
                        }
                        if ui.small_button("/tp").clicked() {
                            let cmd = format_tp_xyz(pdim, px, Some(py), pz);
                            ui.ctx().copy_text(cmd.clone());
                            audio.play(SoundCue::Toast);
                            toasts.push(
                                t(lang, UiKey::CopiedTpTitle),
                                cmd,
                                "compass",
                                palette.accent_gold,
                            );
                        }
                    });
                }
            } else if state.is_server_seed_active {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(t(lang, UiKey::SeedServerModeNote))
                        .size(10.0)
                        .color(palette.accent_primary),
                );
            }

            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(5.0, 4.0);
                if ui.button(t(lang, UiKey::PlayerUploadSkinBtn)).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Minecraft Skin PNG", &["png"])
                        .pick_file()
                    {
                        match state.apply_custom_skin_file(ui.ctx(), &path) {
                            Ok(()) => {
                                audio.play(SoundCue::XpOrb);
                                toasts.push(
                                    t(lang, UiKey::PlayerSkinSavedToastTitle),
                                    path.file_name()
                                        .map(|n| n.to_string_lossy().into_owned())
                                        .unwrap_or_default(),
                                    "compass",
                                    palette.accent_primary,
                                );
                            }
                            Err(err) => {
                                audio.play(SoundCue::Toast);
                                toasts.push(
                                    t(lang, UiKey::PlayerSkinErrorToastTitle),
                                    err,
                                    "barrier",
                                    Color32::from_rgb(235, 85, 85),
                                );
                            }
                        }
                    }
                }
                if state.has_custom_skin && ui.button(t(lang, UiKey::PlayerResetSkinBtn)).clicked() {
                    state.reset_custom_skin(ui.ctx());
                    audio.play(SoundCue::Click);
                    toasts.push(
                        t(lang, UiKey::PlayerSkinResetToastTitle),
                        t(lang, UiKey::PlayerSkinDefaultBadge),
                        "compass",
                        palette.accent_gold,
                    );
                }
            });
            ui.add_space(2.0);
            ui.label(
                RichText::new(if state.has_custom_skin {
                    t(lang, UiKey::PlayerSkinCustomBadge)
                } else {
                    t(lang, UiKey::PlayerSkinDefaultBadge)
                })
                .size(9.5)
                .color(palette.text_muted),
            );
        });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_saved_seeds_history_card(
    ui: &mut egui::Ui,
    state: &mut AppState,
    engine: &mut SeedMapEngine,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    toasts: &mut ToastQueue,
    lang: Language,
) {
    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(t(lang, UiKey::SeedSavedHeader))
                    .strong()
                    .size(12.0)
                    .color(palette.accent_gold),
            );
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                let saved_lbl = format!(
                    "{} ({})",
                    t(lang, UiKey::SeedSavedTab),
                    state.prefs.saved_seeds.len()
                );
                if filter_chip(
                    ui,
                    palette,
                    atlas,
                    Some("ender_eye"),
                    &saved_lbl,
                    state.seed_book_tab == SeedBookTab::Saved,
                )
                .clicked()
                {
                    state.seed_book_tab = SeedBookTab::Saved;
                    audio.play(SoundCue::Click);
                }

                let hist_lbl = format!(
                    "{} ({})",
                    t(lang, UiKey::SeedHistoryTab),
                    state.prefs.seed_history.len()
                );
                if filter_chip(
                    ui,
                    palette,
                    atlas,
                    Some("clock"),
                    &hist_lbl,
                    state.seed_book_tab == SeedBookTab::History,
                )
                .clicked()
                {
                    state.seed_book_tab = SeedBookTab::History;
                    audio.play(SoundCue::Click);
                }
            });

            ui.add_space(6.0);

            match state.seed_book_tab {
                SeedBookTab::Saved => {
                    ui.horizontal(|ui| {
                        let sb_w = (ui.available_width() - 34.0).max(120.0);
                        let sb_te = ui.add_sized(
                            [sb_w, 23.0],
                            egui::TextEdit::singleline(&mut state.save_seed_name)
                                .hint_text(t(lang, UiKey::SeedSaveNameHint)),
                        );
                        let sb_enter =
                            sb_te.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if sb_enter
                            || ui
                                .add_sized(
                                    [28.0, 23.0],
                                    egui::Button::new(
                                        RichText::new("+").strong().color(palette.accent_primary),
                                    ),
                                )
                                .on_hover_text(t(lang, UiKey::SaveSeedBtn))
                                .clicked()
                        {
                            save_active_seed_bookmark(state, engine, audio, toasts, lang);
                        }
                    });
                    ui.checkbox(&mut state.save_seed_is_server, t(lang, UiKey::SeedIsServerToggle));
                    ui.add_space(4.0);

                    if state.prefs.saved_seeds.is_empty() {
                        ui.add(
                            egui::Label::new(
                                RichText::new(t(lang, UiKey::SeedSavedEmptyHint))
                                    .size(10.0)
                                    .color(palette.text_muted),
                            )
                            .wrap(),
                        );
                    } else {
                        let saved_list = state.prefs.saved_seeds.clone();
                        let mut load_entry: Option<SavedSeedEntry> = None;
                        let mut remove_id: Option<u64> = None;
                        let mut copy_entry: Option<(String, String, bool)> = None;

                        egui::ScrollArea::vertical()
                            .id_salt("saved_seeds_sidebar_scroll")
                            .max_height(165.0)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                for s in saved_list {
                                    let active = s.seed == engine.seed
                                        && s.large_biomes == state.prefs.large_biomes
                                        && (!s.is_server || state.is_server_seed_active);
                                    let icon = if s.is_server { "ender_eye" } else { "filled_map" };
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().interact_size.y = 20.0;
                                        ui.add(atlas.ui_image(icon, 15.0));
                                        let mut tags = String::new();
                                        if s.is_server {
                                            tags.push_str(&format!(" [{}]", t(lang, UiKey::SeedServerBadge)));
                                        }
                                        if s.large_biomes {
                                            tags.push_str(" [LB]");
                                        }
                                        let btn = egui::Button::new(
                                            RichText::new(format!("{}{}", s.name, tags))
                                                .size(11.0)
                                                .strong()
                                                .color(if active {
                                                    palette.accent_gold
                                                } else {
                                                    palette.text_primary
                                                }),
                                        )
                                        .frame(false)
                                        .truncate();
                                        if ui.add(btn).on_hover_text(&s.seed_text).clicked() {
                                            load_entry = Some(s.clone());
                                        }
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.small_button("x").clicked() {
                                                    remove_id = Some(s.id);
                                                }
                                                if ui
                                                    .small_button(t(lang, UiKey::SeedCopySeedBtn))
                                                    .on_hover_text(&s.seed_text)
                                                    .clicked()
                                                {
                                                    copy_entry = Some((
                                                        s.name.clone(),
                                                        s.seed_text.clone(),
                                                        s.is_server,
                                                    ));
                                                }
                                            },
                                        );
                                    });
                                }
                            });

                        if let Some(s) = load_entry {
                            activate_saved_seed(state, engine, &s, audio);
                        }
                        if let Some((name, txt, is_srv)) = copy_entry {
                            ui.ctx().copy_text(txt.clone());
                            audio.play(SoundCue::Toast);
                            toasts.push(
                                t(lang, UiKey::SeedCopiedToastTitle),
                                format!("{}: {}", name, txt),
                                if is_srv { "ender_eye" } else { "filled_map" },
                                palette.accent_primary,
                            );
                        }
                        if let Some(id) = remove_id {
                            state.prefs.remove_saved_seed(id);
                            audio.play(SoundCue::Click);
                        }
                    }
                }
                SeedBookTab::History => {
                    if state.prefs.seed_history.is_empty() {
                        ui.add(
                            egui::Label::new(
                                RichText::new(t(lang, UiKey::SeedHistoryEmptyHint))
                                    .size(10.0)
                                    .color(palette.text_muted),
                            )
                            .wrap(),
                        );
                    } else {
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button(t(lang, UiKey::SeedClearHistoryBtn)).clicked() {
                                    state.prefs.clear_seed_history();
                                    audio.play(SoundCue::Click);
                                }
                            });
                        });
                        let history_list = state.prefs.seed_history.clone();
                        let mut load_hist: Option<SeedHistoryEntry> = None;
                        let mut bookmark_hist: Option<SeedHistoryEntry> = None;
                        let mut copy_hist: Option<String> = None;

                        egui::ScrollArea::vertical()
                            .id_salt("seed_history_sidebar_scroll")
                            .max_height(175.0)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                for h in history_list {
                                    let active = h.seed == engine.seed
                                        && h.large_biomes == state.prefs.large_biomes;
                                    let already_saved = state.is_seed_saved(h.seed, h.large_biomes);
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().interact_size.y = 20.0;
                                        let lb_tag = if h.large_biomes { " [LB]" } else { "" };
                                        let lbl = format!("{}{}", h.seed_text, lb_tag);
                                        let hover = format!(
                                            "{} • {}",
                                            h.source_label,
                                            format_relative_time_ms(h.used_at_ms, lang)
                                        );
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new(lbl)
                                                        .monospace()
                                                        .size(10.5)
                                                        .color(if active {
                                                            palette.accent_gold
                                                        } else {
                                                            palette.text_primary
                                                        }),
                                                )
                                                .frame(false)
                                                .truncate(),
                                            )
                                            .on_hover_text(hover)
                                            .clicked()
                                        {
                                            load_hist = Some(h.clone());
                                        }

                                        if ui
                                            .small_button(if already_saved { "*" } else { "+" })
                                            .on_hover_text(t(lang, UiKey::SaveSeedBtn))
                                            .clicked()
                                        {
                                            bookmark_hist = Some(h.clone());
                                        }
                                        if ui
                                            .small_button(t(lang, UiKey::SeedCopySeedBtn))
                                            .clicked()
                                        {
                                            copy_hist = Some(h.seed_text.clone());
                                        }
                                    });
                                }
                            });

                        if let Some(h) = load_hist {
                            state.selected_world_folder = None;
                            state.is_server_seed_active = false;
                            let src = if h.source_label.is_empty() {
                                t(lang, UiKey::SeedHistoryTab).to_string()
                            } else {
                                h.source_label
                            };
                            state.commit_seed_change(
                                engine,
                                &h.seed_text,
                                h.seed,
                                h.large_biomes,
                                &src,
                            );
                            audio.play(SoundCue::Click);
                        }
                        if let Some(h) = bookmark_hist {
                            let has_local = state.local_worlds.iter().any(|w| w.seed == h.seed);
                            let default_name = if !h.source_label.is_empty()
                                && h.source_label != "Manual"
                                && h.source_label != "Aleatorio"
                                && h.source_label != "Random"
                            {
                                h.source_label.clone()
                            } else if has_local {
                                format!("Local ({})", h.seed)
                            } else {
                                format!("Server ({})", h.seed)
                            };
                            let saved = state.save_seed_bookmark(
                                &default_name,
                                &h.seed_text,
                                h.seed,
                                h.large_biomes,
                                !has_local,
                            );
                            audio.play(SoundCue::XpOrb);
                            toasts.push(
                                t(lang, UiKey::SeedSavedToastTitle),
                                format!("{} ({})", saved.name, saved.seed_text),
                                if saved.is_server { "ender_eye" } else { "filled_map" },
                                palette.accent_gold,
                            );
                        }
                        if let Some(txt) = copy_hist {
                            ui.ctx().copy_text(txt.clone());
                            audio.play(SoundCue::Toast);
                            toasts.push(
                                t(lang, UiKey::SeedCopiedToastTitle),
                                txt,
                                "filled_map",
                                palette.accent_primary,
                            );
                        }
                    }
                }
            }
        });
}

pub(super) fn render_local_worlds_sidebar_card(
    ui: &mut egui::Ui,
    state: &mut AppState,
    engine: &mut SeedMapEngine,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    lang: Language,
) {
    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add(atlas.ui_image("grass_block", 15.0));
                ui.label(
                    RichText::new(t(lang, UiKey::LocalSavesHeader))
                        .strong()
                        .size(11.5)
                        .color(palette.accent_gold),
                );
            });
            ui.add_space(5.0);

            if state.local_worlds.is_empty() {
                ui.add(
                    egui::Label::new(
                        RichText::new(t(lang, UiKey::NoLocalSaves))
                            .size(10.0)
                            .color(palette.text_muted),
                    )
                    .wrap(),
                );
                return;
            }

            let worlds = state.local_worlds.clone();
            let mut pick_world: Option<DetectedWorld> = None;

            egui::ScrollArea::vertical()
                .id_salt("local_worlds_sidebar_scroll")
                .max_height(175.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for w in &worlds {
                        let is_folder_match = state
                            .selected_world_folder
                            .as_deref()
                            .is_some_and(|f| f == w.folder_name);
                        let active = !state.is_server_seed_active
                            && w.seed == engine.seed
                            && (state.selected_world_folder.is_none() || is_folder_match);
                        let detail = if let Some(p) = w.player_pos {
                            format!(
                                "Seed: {} • {} • {} X:{} Y:{} Z:{}",
                                w.seed,
                                w.last_played_label(lang),
                                w.player_dimension().label(lang),
                                p[0].round() as i32,
                                p[1].round() as i32,
                                p[2].round() as i32
                            )
                        } else {
                            format!("Seed: {} • {}", w.seed, w.last_played_label(lang))
                        };
                        ui.horizontal(|ui| {
                            ui.spacing_mut().interact_size.y = 22.0;
                            ui.add(atlas.ui_image("grass_block", 14.0));
                            let btn = egui::Button::new(
                                RichText::new(&w.level_name)
                                    .size(11.0)
                                    .strong()
                                    .color(if active {
                                        palette.accent_gold
                                    } else {
                                        palette.text_primary
                                    }),
                            )
                            .frame(false)
                            .truncate();
                            if ui.add(btn).on_hover_text(detail).clicked() {
                                pick_world = Some(w.clone());
                            }
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        RichText::new(w.last_played_label(lang))
                                            .size(9.5)
                                            .color(palette.text_muted),
                                    );
                                },
                            );
                        });
                    }
                });

            if let Some(w) = pick_world {
                activate_local_world(state, engine, &w, audio);
            }
        });
}
