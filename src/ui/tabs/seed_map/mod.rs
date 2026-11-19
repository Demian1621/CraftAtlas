mod canvas;
mod export;
mod finder;
mod layers;
mod sidebar;
mod waypoints;
mod worlds;

use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, Vec2};

use crate::anim::ToastQueue;
use crate::assets::ItemAtlas;
use crate::audio::{AudioEngine, SoundCue};
use crate::data::models::Language;
use crate::data::seed_engine::{parse_seed_input, Dimension, MapStyle, SeedMapEngine};
use crate::data::worlds_nbt::DetectedWorld;
use crate::i18n::{t, UiKey};
use crate::state::{AppState, SavedSeedEntry, SeedHistoryEntry, SeedSidebarTab};
use crate::ui::theme::{filter_chip, McPalette};
use crate::ui::RenderContext;

use canvas::render_map_viewport;
use export::handle_dropped_skin_png;
use sidebar::render_map_sidebar;
/// Renders the native Cubiomes Seed Map Explorer across Overworld, Nether, and The End.
pub fn render(
    ui: &mut egui::Ui,
    state: &mut AppState,
    engine: &mut SeedMapEngine,
    rcx: &RenderContext<'_>,
    toasts: &mut ToastQueue,
    dt: f32,
) {
    let RenderContext {
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
    let lang = state.prefs.language;
    handle_dropped_skin_png(ui.ctx(), state, audio, toasts, lang);
    let _ = state.ensure_player_face_tex(ui.ctx());

    // Upload newly completed Cubiomes tiles from background thread
    if engine.poll_tiles(ui.ctx()) || engine.has_in_flight() {
        ui.ctx().request_repaint();
    }

    // Smoothly ease camera pan and zoom
    let ease = (1.0 - (-18.0 * dt).exp()).clamp(0.15, 1.0);
    if (state.map_center_x - state.map_target_center_x).abs() > 0.05
        || (state.map_center_z - state.map_target_center_z).abs() > 0.05
        || (state.map_blocks_per_px - state.map_target_bpp).abs() > 0.002
    {
        state.map_center_x += (state.map_target_center_x - state.map_center_x) * ease as f64;
        state.map_center_z += (state.map_target_center_z - state.map_center_z) * ease as f64;
        state.map_blocks_per_px += (state.map_target_bpp - state.map_blocks_per_px) * ease;
        ui.ctx().request_repaint();
    }

    // 1. Top Control Strip: Seed Input + Dimensions + Map Style + Clean World/Server Switcher
    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(10, 7))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 5.0);

                ui.add(atlas.ui_image("filled_map", 18.0));
                ui.label(
                    RichText::new(format!("{}:", t(lang, UiKey::SeedInputLabel)))
                        .strong()
                        .size(12.5)
                        .color(palette.accent_gold),
                );
                let te =
                    ui.add_sized([175.0, 24.0], egui::TextEdit::singleline(&mut state.seed_input));
                let enter_pressed = te.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if enter_pressed || ui.button(t(lang, UiKey::GoBtn)).clicked() {
                    apply_seed_from_input(state, engine);
                    audio.play(SoundCue::Click);
                }

                let cur_saved = state.is_seed_saved(engine.seed, state.prefs.large_biomes);
                let save_btn_txt = if cur_saved {
                    RichText::new(t(lang, UiKey::SeedSavedBtnBadge))
                        .size(11.5)
                        .color(palette.accent_gold)
                } else {
                    RichText::new(t(lang, UiKey::SaveSeedBtn))
                        .size(11.5)
                        .color(palette.accent_primary)
                };
                if ui.button(save_btn_txt).clicked() {
                    state.save_seed_open = !state.save_seed_open;
                    if state.save_seed_open {
                        if let Some(w) = state.active_world_for_seed(engine.seed) {
                            if state.save_seed_name.trim().is_empty() {
                                state.save_seed_name = w.level_name.clone();
                            }
                            state.save_seed_is_server = false;
                        } else {
                            state.save_seed_is_server = true;
                        }
                    }
                    audio.play(SoundCue::Click);
                }

                if ui.button(t(lang, UiKey::RandomSeedBtn)).clicked() {
                    let nanos = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_nanos() as u64)
                        .unwrap_or(987654321);
                    let mixed = (nanos ^ (nanos << 21) ^ 0x517cc1b727220a95) as i64;
                    state.selected_world_folder = None;
                    state.is_server_seed_active = false;
                    let src_lbl = if matches!(lang, Language::EnUs) {
                        "Random"
                    } else {
                        "Aleatorio"
                    };
                    state.commit_seed_change(
                        engine,
                        &mixed.to_string(),
                        mixed,
                        state.prefs.large_biomes,
                        src_lbl,
                    );
                    audio.play(SoundCue::Click);
                }

                let mut lb = state.prefs.large_biomes;
                if ui.checkbox(&mut lb, t(lang, UiKey::LargeBiomesToggle)).changed() {
                    let src_lbl = if state.is_server_seed_active {
                        "Server"
                    } else {
                        "Manual"
                    };
                    let cur_txt = state.seed_input.clone();
                    state.commit_seed_change(engine, &cur_txt, engine.seed, lb, src_lbl);
                    audio.play(SoundCue::Click);
                }

                ui.separator();

                // Dimension Switcher
                for dim in Dimension::all() {
                    if filter_chip(
                        ui,
                        palette,
                        atlas,
                        Some(dim.icon_id()),
                        dim.label(lang),
                        state.map_dim == dim,
                    )
                    .clicked()
                    {
                        state.switch_map_dimension(dim);
                        audio.play(SoundCue::Click);
                    }
                }

                ui.separator();

                // Map Rendering Style Switcher
                for st in MapStyle::all() {
                    if filter_chip(ui, palette, atlas, None, st.label(lang), state.map_style == st)
                        .clicked()
                    {
                        state.map_style = st;
                        audio.play(SoundCue::Click);
                    }
                }
            });

            // Inline Quick-Save Seed Strip (ideal for Server SMP seeds)
            if state.save_seed_open {
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);
                    ui.add(atlas.ui_image("ender_eye", 15.0));
                    let name_te = ui.add_sized(
                        [180.0, 22.0],
                        egui::TextEdit::singleline(&mut state.save_seed_name)
                            .hint_text(t(lang, UiKey::SeedSaveNameHint)),
                    );
                    ui.checkbox(&mut state.save_seed_is_server, t(lang, UiKey::SeedIsServerToggle));
                    let confirm_enter =
                        name_te.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if confirm_enter || ui.button(t(lang, UiKey::SeedSaveConfirmBtn)).clicked() {
                        save_active_seed_bookmark(state, engine, audio, toasts, lang);
                        state.save_seed_open = false;
                    }
                    if ui.small_button("x").clicked() {
                        state.save_seed_open = false;
                    }
                });
            }

            // Tidy World / Saved Server Picker + Quick Actions (never wraps/overflows across edges)
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);
                render_world_seed_combobox(ui, state, engine, palette, atlas, audio, lang);

                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(t(lang, UiKey::PlayerReloadWorldsBtn))
                                .size(11.0)
                                .color(palette.accent_primary),
                        )
                        .corner_radius(CornerRadius::same(4)),
                    )
                    .clicked()
                {
                    let count = state.reload_local_worlds();
                    audio.play(SoundCue::Click);
                    toasts.push(
                        t(lang, UiKey::PlayerReloadedToastTitle),
                        if matches!(lang, Language::EnUs) {
                            format!("Detected {} local worlds", count)
                        } else {
                            format!("{} mundos locales detectados", count)
                        },
                        "compass",
                        palette.accent_primary,
                    );
                }

                let worlds_active = state.seed_sidebar_tab == SeedSidebarTab::Worlds;
                let worlds_badge = state.prefs.saved_seeds.len() + state.local_worlds.len();
                let shortcut_lbl = if worlds_badge > 0 {
                    format!("{} ({})", t(lang, UiKey::SeedManageWorldsShortcut), worlds_badge)
                } else {
                    t(lang, UiKey::SeedManageWorldsShortcut).to_string()
                };
                if filter_chip(
                    ui,
                    palette,
                    atlas,
                    Some("ender_eye"),
                    &shortcut_lbl,
                    worlds_active,
                )
                .clicked()
                {
                    state.seed_sidebar_tab = SeedSidebarTab::Worlds;
                    audio.play(SoundCue::Click);
                }
            });
        });

    ui.add_space(6.0);

    // 2. Main Split Workspace: Adaptive Sidebar (Left) + Interactive Viewport
    let total_w = ui.available_width();
    if total_w >= 860.0 {
        let sidebar_w = if total_w >= 1100.0 { 296.0 } else { 272.0 };
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(sidebar_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    render_map_sidebar(ui, state, engine, rcx, toasts, lang);
                },
            );
            ui.add_space(8.0);
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    render_map_viewport(ui, state, engine, rcx, toasts);
                },
            );
        });
    } else {
        // Narrow mode: Viewport on top + tabbed sidebar underneath
        let map_h = (ui.available_height() * 0.62).max(260.0);
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), map_h),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                render_map_viewport(ui, state, engine, rcx, toasts);
            },
        );
        ui.add_space(6.0);
        render_map_sidebar(ui, state, engine, rcx, toasts, lang);
    }
}

pub(super) fn apply_seed_from_input(state: &mut AppState, engine: &mut SeedMapEngine) {
    let raw = state.seed_input.clone();
    let parsed = parse_seed_input(&raw);
    state.selected_world_folder = None;
    state.is_server_seed_active = false;
    state.commit_seed_change(engine, &raw, parsed, state.prefs.large_biomes, "Manual");
}

pub(super) fn save_active_seed_bookmark(
    state: &mut AppState,
    engine: &mut SeedMapEngine,
    audio: &AudioEngine,
    toasts: &mut ToastQueue,
    lang: Language,
) {
    let raw_seed = state.seed_input.clone();
    let parsed = parse_seed_input(&raw_seed);
    let lb = state.prefs.large_biomes;
    if parsed != engine.seed {
        let src_lbl = if state.save_seed_is_server {
            "Server"
        } else {
            "Manual"
        };
        state.commit_seed_change(engine, &raw_seed, parsed, lb, src_lbl);
    }
    let name_draft = state.save_seed_name.clone();
    let is_srv = state.save_seed_is_server;
    let saved = state.save_seed_bookmark(&name_draft, &raw_seed, parsed, lb, is_srv);
    state.is_server_seed_active = is_srv;
    state.save_seed_name.clear();
    audio.play(SoundCue::XpOrb);
    toasts.push(
        t(lang, UiKey::SeedSavedToastTitle),
        format!("{} ({})", saved.name, saved.seed_text),
        if is_srv { "ender_eye" } else { "filled_map" },
        palette_gold_fallback(),
    );
}

pub(super) fn palette_gold_fallback() -> Color32 {
    Color32::from_rgb(248, 198, 72)
}

pub(super) fn activate_local_world(
    state: &mut AppState,
    engine: &mut SeedMapEngine,
    world: &DetectedWorld,
    audio: &AudioEngine,
) {
    state.selected_world_folder = Some(world.folder_name.clone());
    state.is_server_seed_active = false;
    state.commit_seed_change(
        engine,
        &world.seed.to_string(),
        world.seed,
        state.prefs.large_biomes,
        &world.level_name,
    );
    state.map_dim = world.player_dimension();
    if let Some(pos) = world.player_pos {
        state.jump_map_to(pos[0], pos[2]);
    }
    audio.play(SoundCue::XpOrb);
}

pub(super) fn activate_saved_seed(
    state: &mut AppState,
    engine: &mut SeedMapEngine,
    entry: &SavedSeedEntry,
    audio: &AudioEngine,
) {
    state.selected_world_folder = None;
    state.is_server_seed_active = entry.is_server;
    state.commit_seed_change(
        engine,
        &entry.seed_text,
        entry.seed,
        entry.large_biomes,
        &entry.name,
    );
    audio.play(SoundCue::XpOrb);
}

fn render_world_seed_combobox(
    ui: &mut egui::Ui,
    state: &mut AppState,
    engine: &mut SeedMapEngine,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    lang: Language,
) {
    let lb = state.prefs.large_biomes;
    let active_saved = state
        .prefs
        .saved_seeds
        .iter()
        .find(|s| {
            s.seed == engine.seed
                && s.large_biomes == lb
                && (!s.is_server || state.is_server_seed_active)
        })
        .cloned();
    let active_local = if !state.is_server_seed_active {
        state.active_world_for_seed(engine.seed).cloned()
    } else {
        None
    };

    let selected_text = if let Some(s) = &active_saved {
        let tag = if s.is_server {
            t(lang, UiKey::SeedServerBadge)
        } else {
            t(lang, UiKey::SeedSavedTab)
        };
        RichText::new(format!("[{}] {}", tag, s.name))
            .size(11.5)
            .strong()
            .color(palette.accent_gold)
    } else if let Some(w) = &active_local {
        RichText::new(format!("{} • {}", w.level_name, w.last_played_label(lang)))
            .size(11.5)
            .color(palette.accent_primary)
    } else {
        RichText::new(t(lang, UiKey::WorldPickerPlaceholder))
            .size(11.5)
            .color(palette.text_muted)
    };

    let saved_list = state.prefs.saved_seeds.clone();
    let worlds_list = state.local_worlds.clone();
    let hist_list: Vec<SeedHistoryEntry> =
        state.prefs.seed_history.iter().take(6).cloned().collect();

    let mut pick_saved: Option<SavedSeedEntry> = None;
    let mut pick_world: Option<DetectedWorld> = None;
    let mut pick_hist: Option<SeedHistoryEntry> = None;

    egui::ComboBox::from_id_salt("seed_world_combo")
        .selected_text(selected_text)
        .width(245.0)
        .show_ui(ui, |ui| {
            ui.set_min_width(285.0);

            if !saved_list.is_empty() {
                ui.label(
                    RichText::new(t(lang, UiKey::SeedSavedTab))
                        .strong()
                        .size(11.0)
                        .color(palette.accent_gold),
                );
                for s in &saved_list {
                    let is_sel = s.seed == engine.seed
                        && s.large_biomes == lb
                        && (!s.is_server || state.is_server_seed_active);
                    let icon = if s.is_server { "ender_eye" } else { "filled_map" };
                    ui.horizontal(|ui| {
                        ui.add(atlas.ui_image(icon, 14.0));
                        let tag = if s.is_server {
                            format!(" [{}]", t(lang, UiKey::SeedServerBadge))
                        } else {
                            String::new()
                        };
                        if ui
                            .selectable_label(is_sel, format!("{}{}  ({})", s.name, tag, s.seed_text))
                            .clicked()
                        {
                            pick_saved = Some(s.clone());
                        }
                    });
                }
            }

            if !worlds_list.is_empty() {
                if !saved_list.is_empty() {
                    ui.separator();
                }
                ui.label(
                    RichText::new(t(lang, UiKey::LocalSavesHeader))
                        .strong()
                        .size(11.0)
                        .color(palette.accent_primary),
                );
                for w in &worlds_list {
                    let is_folder_match = state
                        .selected_world_folder
                        .as_deref()
                        .is_some_and(|f| f == w.folder_name);
                    let is_sel = !state.is_server_seed_active
                        && w.seed == engine.seed
                        && (state.selected_world_folder.is_none() || is_folder_match);
                    ui.horizontal(|ui| {
                        ui.add(atlas.ui_image("grass_block", 14.0));
                        let lbl = format!("{}  ({})", w.level_name, w.last_played_label(lang));
                        if ui.selectable_label(is_sel, lbl).clicked() {
                            pick_world = Some(w.clone());
                        }
                    });
                }
            }

            if !hist_list.is_empty() {
                if !saved_list.is_empty() || !worlds_list.is_empty() {
                    ui.separator();
                }
                ui.label(
                    RichText::new(t(lang, UiKey::SeedHistoryTab))
                        .strong()
                        .size(11.0)
                        .color(palette.text_muted),
                );
                for h in &hist_list {
                    let is_sel = h.seed == engine.seed && h.large_biomes == lb;
                    if ui
                        .selectable_label(
                            is_sel,
                            format!("{}  •  {}", h.seed_text, h.source_label),
                        )
                        .clicked()
                    {
                        pick_hist = Some(h.clone());
                    }
                }
            }

            if saved_list.is_empty() && worlds_list.is_empty() && hist_list.is_empty() {
                ui.label(
                    RichText::new(t(lang, UiKey::NoLocalSaves))
                        .size(11.0)
                        .color(palette.text_muted),
                );
            }
        });

    if let Some(s) = pick_saved {
        activate_saved_seed(state, engine, &s, audio);
    } else if let Some(w) = pick_world {
        activate_local_world(state, engine, &w, audio);
    } else if let Some(h) = pick_hist {
        state.selected_world_folder = None;
        state.is_server_seed_active = false;
        state.commit_seed_change(engine, &h.seed_text, h.seed, h.large_biomes, &h.source_label);
        audio.play(SoundCue::Click);
    }
}


pub(super) fn render_tool_tooltip(ui: &mut egui::Ui, palette: McPalette, title: &str, desc: &str) {
    ui.set_max_width(275.0);
    ui.label(RichText::new(title).strong().size(12.0).color(palette.accent_gold));
    ui.add_space(2.0);
    ui.add(
        egui::Label::new(RichText::new(desc).size(11.0).color(palette.text_primary)).wrap(),
    );
}

pub(super) fn format_tp_xyz(dim: Dimension, x: i32, y: Option<i32>, z: i32) -> String {
    let y_str = match y {
        Some(yv) => yv.to_string(),
        None => "~".to_string(),
    };
    match dim {
        Dimension::Overworld => format!("/tp @s {} {} {}", x, y_str, z),
        Dimension::Nether => format!("/execute in minecraft:the_nether run tp @s {} {} {}", x, y_str, z),
        Dimension::TheEnd => format!("/execute in minecraft:the_end run tp @s {} {} {}", x, y_str, z),
    }
}

pub(super) fn format_tp_command(dim: Dimension, x: i32, z: i32) -> String {
    format_tp_xyz(dim, x, None, z)
}

