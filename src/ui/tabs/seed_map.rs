use eframe::egui::{self, Color32, CornerRadius, Margin, Pos2, Rect, RichText, Stroke, Vec2};
use std::path::PathBuf;

use crate::anim::ToastQueue;
use crate::assets::ItemAtlas;
use crate::audio::{AudioEngine, SoundCue};
use crate::data::models::Language;
use crate::data::seed_engine::{
    is_slime_chunk, parse_seed_input, Dimension, LayerToggles, MapStyle, SeedMapEngine,
    StructureKind, StructureMarker, TILE_PX,
};
use crate::data::worlds_nbt::{format_relative_time_ms, DetectedWorld};
use crate::i18n::{t, UiKey};
use crate::state::{
    AppState, MapContextMenuState, SavedSeedEntry, SeedBookTab, SeedHistoryEntry, SeedSidebarTab,
    WAYPOINT_ICON_PRESETS,
};
use crate::ui::theme::{
    filter_chip, paint_mc_frame, paint_mc_slot, show_mc_tooltip_at, McPalette, SlotStyle,
};
use crate::ui::RenderContext;

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
                    render_map_sidebar(ui, state, engine, palette, atlas, audio, toasts, lang);
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
        render_map_sidebar(ui, state, engine, palette, atlas, audio, toasts, lang);
    }
}

fn apply_seed_from_input(state: &mut AppState, engine: &mut SeedMapEngine) {
    let raw = state.seed_input.clone();
    let parsed = parse_seed_input(&raw);
    state.selected_world_folder = None;
    state.is_server_seed_active = false;
    state.commit_seed_change(engine, &raw, parsed, state.prefs.large_biomes, "Manual");
}

fn save_active_seed_bookmark(
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

fn palette_gold_fallback() -> Color32 {
    Color32::from_rgb(248, 198, 72)
}

fn activate_local_world(
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

fn activate_saved_seed(
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

fn sidebar_tab_button(
    ui: &mut egui::Ui,
    palette: McPalette,
    atlas: &ItemAtlas,
    icon_id: &str,
    label: &str,
    selected: bool,
) -> egui::Response {
    let desired = Vec2::new(ui.available_width(), 28.0);
    let (rect, resp) = ui.allocate_exact_size(desired, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let fill = if selected {
            Color32::from_rgba_unmultiplied(
                palette.accent_primary.r(),
                palette.accent_primary.g(),
                palette.accent_primary.b(),
                42,
            )
        } else if resp.hovered() {
            palette.bg_card_hover
        } else {
            palette.bg_panel
        };
        let stroke = if selected {
            Stroke::new(1.4_f32, palette.accent_primary)
        } else {
            Stroke::new(1.0_f32, palette.border_subtle)
        };
        ui.painter()
            .rect(rect, CornerRadius::same(5), fill, stroke, egui::StrokeKind::Inside);
        let text_col = if selected {
            Color32::WHITE
        } else {
            palette.text_muted
        };
        let galley = ui.painter().layout_no_wrap(
            label.to_string(),
            egui::FontId::proportional(10.5),
            text_col,
        );
        let icon_sz = 13.0;
        let gap = 3.5;
        let content_w = icon_sz + gap + galley.size().x;
        let start_x = (rect.center().x - content_w * 0.5).max(rect.left() + 4.0);
        let icon_rect = Rect::from_min_size(
            Pos2::new(start_x, rect.center().y - icon_sz * 0.5),
            Vec2::splat(icon_sz),
        );
        atlas.paint(ui.painter(), icon_rect, icon_id, Color32::WHITE);
        ui.painter().galley(
            Pos2::new(start_x + icon_sz + gap, rect.center().y - galley.size().y * 0.5),
            galley,
            text_col,
        );
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

#[allow(clippy::too_many_arguments)]
fn render_map_sidebar(
    ui: &mut egui::Ui,
    state: &mut AppState,
    engine: &mut SeedMapEngine,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    toasts: &mut ToastQueue,
    lang: Language,
) {
    let wp_count = state
        .prefs
        .waypoints
        .iter()
        .filter(|w| w.seed == engine.seed)
        .count();
    let wp_lbl = if wp_count > 0 {
        format!("{} ({})", t(lang, UiKey::SidebarTabWaypoints), wp_count)
    } else {
        t(lang, UiKey::SidebarTabWaypoints).to_string()
    };

    // Pinned Segmented Sidebar Sub-Tabs (never scroll offscreen; Waypoints come FIRST)
    ui.columns(3, |cols| {
        if sidebar_tab_button(
            &mut cols[0],
            palette,
            atlas,
            "red_bed",
            &wp_lbl,
            state.seed_sidebar_tab == SeedSidebarTab::Waypoints,
        )
        .clicked()
        {
            state.seed_sidebar_tab = SeedSidebarTab::Waypoints;
            audio.play(SoundCue::Click);
        }
        if sidebar_tab_button(
            &mut cols[1],
            palette,
            atlas,
            "grass_block",
            t(lang, UiKey::SidebarTabLayers),
            state.seed_sidebar_tab == SeedSidebarTab::Layers,
        )
        .clicked()
        {
            state.seed_sidebar_tab = SeedSidebarTab::Layers;
            audio.play(SoundCue::Click);
        }
        if sidebar_tab_button(
            &mut cols[2],
            palette,
            atlas,
            "ender_eye",
            t(lang, UiKey::SidebarTabWorlds),
            state.seed_sidebar_tab == SeedSidebarTab::Worlds,
        )
        .clicked()
        {
            state.seed_sidebar_tab = SeedSidebarTab::Worlds;
            audio.play(SoundCue::Click);
        }
    });

    ui.add_space(6.0);

    egui::ScrollArea::vertical()
        .id_salt("seed_map_side_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| match state.seed_sidebar_tab {
            SeedSidebarTab::Waypoints => {
                render_waypoints_sidebar_card(
                    ui, state, engine, palette, atlas, audio, toasts, lang, wp_count,
                );
                ui.add_space(7.0);
                render_quick_nav_sidebar_card(
                    ui, state, engine, palette, atlas, audio, toasts, lang,
                );
                ui.add_space(7.0);
                render_compact_player_strip(ui, state, engine, palette, audio, toasts, lang);
            }
            SeedSidebarTab::Layers => {
                render_layers_sidebar_tab(ui, state, palette, atlas, audio, lang);
            }
            SeedSidebarTab::Worlds => {
                render_player_skin_sidebar_card(ui, state, engine, palette, audio, toasts, lang);
                ui.add_space(7.0);
                render_saved_seeds_history_card(
                    ui, state, engine, palette, atlas, audio, toasts, lang,
                );
                ui.add_space(7.0);
                render_local_worlds_sidebar_card(ui, state, engine, palette, atlas, audio, lang);
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn render_waypoints_sidebar_card(
    ui: &mut egui::Ui,
    state: &mut AppState,
    engine: &SeedMapEngine,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    toasts: &mut ToastQueue,
    lang: Language,
    wp_count: usize,
) {
    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add(atlas.ui_image("red_bed", 15.0));
                ui.label(
                    RichText::new(t(lang, UiKey::WaypointsHeader))
                        .strong()
                        .size(12.0)
                        .color(palette.accent_gold),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("({})", wp_count))
                            .size(11.0)
                            .color(palette.text_muted),
                    );
                });
            });

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                let input_w = (ui.available_width() - 34.0).max(120.0);
                let te = ui.add_sized(
                    [input_w, 23.0],
                    egui::TextEdit::singleline(&mut state.new_waypoint_name)
                        .hint_text(t(lang, UiKey::WaypointNameHint)),
                );
                let add_enter = te.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                let add_btn = ui
                    .add_sized(
                        [28.0, 23.0],
                        egui::Button::new(RichText::new("+").strong().color(palette.accent_primary)),
                    )
                    .on_hover_text(t(lang, UiKey::AddWaypointBtn));
                if add_enter || add_btn.clicked() {
                    let cx = state.map_center_x.round() as i32;
                    let cz = state.map_center_z.round() as i32;
                    let def = format!("({}, {})", cx, cz);
                    state.add_waypoint(engine.seed, cx, cz, &def);
                    audio.play(SoundCue::Click);
                    toasts.push(
                        t(lang, UiKey::ToastWaypointAddedTitle),
                        def,
                        &state.new_waypoint_icon,
                        palette.accent_primary,
                    );
                }
            });

            ui.add_space(6.0);
            // Symmetric 6x2 preset slot grid (eliminates orphaned trailing row)
            for chunk in WAYPOINT_ICON_PRESETS.chunks(6) {
                let row_w = chunk.len() as f32 * 24.0 + (chunk.len().saturating_sub(1)) as f32 * 6.0;
                let left_pad = ((ui.available_width() - row_w) * 0.5).max(0.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);
                    ui.add_space(left_pad);
                    for (icon_id, es_lbl, en_lbl) in chunk {
                        let active = state.new_waypoint_icon == *icon_id;
                        let (sr, sresp) =
                            ui.allocate_exact_size(Vec2::splat(24.0), egui::Sense::click());
                        paint_mc_slot(
                            ui.painter(),
                            sr,
                            palette,
                            atlas,
                            Some(icon_id),
                            SlotStyle {
                                selected: active,
                                hovered: sresp.hovered(),
                                ..Default::default()
                            },
                        );
                        let tip = if matches!(lang, Language::EnUs) {
                            *en_lbl
                        } else {
                            *es_lbl
                        };
                        if sresp.on_hover_text(tip).clicked() {
                            state.new_waypoint_icon = (*icon_id).to_string();
                            audio.play(SoundCue::Click);
                        }
                    }
                });
                ui.add_space(3.0);
            }

            ui.separator();

            let wps: Vec<_> = state
                .prefs
                .waypoints
                .iter()
                .filter(|w| w.seed == engine.seed)
                .cloned()
                .collect();
            if wps.is_empty() {
                ui.add(
                    egui::Label::new(
                        RichText::new(t(lang, UiKey::SeedEmptyWaypointsHint))
                            .size(10.5)
                            .color(palette.text_muted),
                    )
                    .wrap(),
                );
            } else {
                let mut jump_to: Option<(i32, i32, Dimension)> = None;
                let mut copy_wp_tp: Option<(String, Dimension, i32, i32)> = None;
                let mut remove_id: Option<u64> = None;

                egui::ScrollArea::vertical()
                    .id_salt("waypoints_card_list_scroll")
                    .max_height(180.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for wp in &wps {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().interact_size.y = 22.0;
                                ui.add(atlas.ui_image(&wp.icon, 16.0));
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new(&wp.name)
                                                .size(11.5)
                                                .strong()
                                                .color(palette.text_primary),
                                        )
                                        .frame(false)
                                        .truncate(),
                                    )
                                    .on_hover_text(format!("({}, {})", wp.x, wp.z))
                                    .clicked()
                                {
                                    jump_to = Some((wp.x, wp.z, wp.dimension()));
                                }
                                ui.label(
                                    RichText::new(format!("({},{})", wp.x, wp.z))
                                        .monospace()
                                        .size(10.0)
                                        .color(palette.text_muted),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.small_button("x").clicked() {
                                            remove_id = Some(wp.id);
                                        }
                                        if ui
                                            .small_button("/tp")
                                            .on_hover_text(t(lang, UiKey::CopyTpBtn))
                                            .clicked()
                                        {
                                            copy_wp_tp = Some((
                                                wp.name.clone(),
                                                wp.dimension(),
                                                wp.x,
                                                wp.z,
                                            ));
                                        }
                                    },
                                );
                            });
                        }
                    });

                if let Some((wname, dim, x, z)) = copy_wp_tp {
                    let cmd = format_tp_command(dim, x, z);
                    ui.ctx().copy_text(cmd.clone());
                    audio.play(SoundCue::Toast);
                    toasts.push(
                        t(lang, UiKey::CopiedTpTitle),
                        format!("{}: {}", wname, cmd),
                        "compass",
                        palette.accent_primary,
                    );
                }
                if let Some((x, z, d)) = jump_to {
                    state.map_dim = d;
                    state.jump_map_to(x as f64, z as f64);
                    audio.play(SoundCue::Click);
                }
                if let Some(id) = remove_id {
                    state.remove_waypoint(id);
                    audio.play(SoundCue::Click);
                }
            }

            ui.add_space(4.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t(lang, UiKey::MapControlsHint))
                        .size(9.5)
                        .color(palette.text_muted),
                )
                .wrap(),
            );
        });
}

#[allow(clippy::too_many_arguments)]
fn render_quick_nav_sidebar_card(
    ui: &mut egui::Ui,
    state: &mut AppState,
    engine: &SeedMapEngine,
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
                RichText::new(t(lang, UiKey::GoToCoordsTitle))
                    .strong()
                    .size(11.5)
                    .color(palette.accent_gold),
            );
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("X:");
                ui.add_sized([58.0, 22.0], egui::TextEdit::singleline(&mut state.goto_x_input));
                ui.label("Z:");
                ui.add_sized([58.0, 22.0], egui::TextEdit::singleline(&mut state.goto_z_input));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(t(lang, UiKey::GoBtn)).clicked() {
                        if let (Ok(x), Ok(z)) = (
                            state.goto_x_input.trim().parse::<f64>(),
                            state.goto_z_input.trim().parse::<f64>(),
                        ) {
                            state.jump_map_to(x, z);
                            audio.play(SoundCue::Click);
                        }
                    }
                });
            });

            ui.add_space(6.0);
            ui.columns(3, |cols| {
                let spawn_btn = cols[0]
                    .add_sized(
                        [cols[0].available_width(), 22.0],
                        egui::Button::new(RichText::new("Spawn").size(11.0)),
                    )
                    .on_hover_text(t(lang, UiKey::JumpSpawnBtn));
                if spawn_btn.clicked() {
                    let sp = engine.spawn_point();
                    if state.map_dim != Dimension::Overworld {
                        state.switch_map_dimension(Dimension::Overworld);
                    }
                    state.jump_map_to(sp.0 as f64, sp.1 as f64);
                    audio.play(SoundCue::Click);
                }

                let sh_btn = cols[1]
                    .add_sized(
                        [cols[1].available_width(), 22.0],
                        egui::Button::new(RichText::new("Stronghold").size(11.0)),
                    )
                    .on_hover_text(t(lang, UiKey::JumpStrongholdBtn));
                if sh_btn.clicked() {
                    if let Some(nearest) = engine.strongholds().iter().min_by_key(|s| {
                        let dx = (s.x as f64 - state.map_center_x) as i64;
                        let dz = (s.z as f64 - state.map_center_z) as i64;
                        dx * dx + dz * dz
                    }) {
                        if state.map_dim != Dimension::Overworld {
                            state.switch_map_dimension(Dimension::Overworld);
                        }
                        state.jump_map_to(nearest.x as f64, nearest.z as f64);
                        audio.play(SoundCue::Click);
                    }
                }

                let zero_btn = cols[2]
                    .add_sized(
                        [cols[2].available_width(), 22.0],
                        egui::Button::new(RichText::new("(0, 0)").size(11.0)),
                    )
                    .on_hover_text(t(lang, UiKey::PanResetBtn));
                if zero_btn.clicked() {
                    state.jump_map_to(0.0, 0.0);
                    audio.play(SoundCue::Click);
                }
            });

            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);
                ui.checkbox(&mut state.nether_sync_enabled, t(lang, UiKey::NetherSyncToggle))
                    .on_hover_ui(|ui| {
                        render_tool_tooltip(
                            ui,
                            palette,
                            t(lang, UiKey::TooltipNetherSyncTitle),
                            t(lang, UiKey::TooltipNetherSyncDesc),
                        );
                    });

                let ruler_resp = filter_chip(
                    ui,
                    palette,
                    atlas,
                    Some("iron_sword"),
                    t(lang, UiKey::RulerModeBtn),
                    state.ruler_active,
                )
                .on_hover_ui(|ui| {
                    render_tool_tooltip(
                        ui,
                        palette,
                        t(lang, UiKey::TooltipRulerTitle),
                        t(lang, UiKey::TooltipRulerDesc),
                    );
                });
                if ruler_resp.clicked() {
                    state.ruler_active = !state.ruler_active;
                    if !state.ruler_active {
                        state.ruler_p1 = None;
                        state.ruler_p2 = None;
                    }
                    audio.play(SoundCue::Click);
                }

                let export_resp = filter_chip(
                    ui,
                    palette,
                    atlas,
                    Some("spyglass"),
                    t(lang, UiKey::ExportPngBtn),
                    false,
                )
                .on_hover_ui(|ui| {
                    render_tool_tooltip(
                        ui,
                        palette,
                        t(lang, UiKey::TooltipExportPngTitle),
                        t(lang, UiKey::TooltipExportPngDesc),
                    );
                });
                if export_resp.clicked() {
                    if let Some(path) = default_export_png_path(engine.seed, state.map_dim) {
                        let scale = pick_cubiomes_scale(state.map_blocks_per_px);
                        if engine
                            .export_map_png(
                                &path,
                                state.map_dim,
                                state.map_style,
                                state.map_center_x.round() as i32,
                                state.map_center_z.round() as i32,
                                scale,
                            )
                            .is_ok()
                        {
                            state.last_export_path = Some(path.clone());
                            audio.play(SoundCue::Toast);
                            toasts.push(
                                t(lang, UiKey::ToastMapExportTitle),
                                path.file_name()
                                    .map(|n| n.to_string_lossy().into_owned())
                                    .unwrap_or_default(),
                                "filled_map",
                                palette.accent_xp,
                            );
                        }
                    }
                }
            });
        });
}

fn render_compact_player_strip(
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
        .inner_margin(Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (face_rect, _) =
                    ui.allocate_exact_size(Vec2::splat(26.0), egui::Sense::hover());
                ui.painter().rect_stroke(
                    face_rect.expand(1.2),
                    CornerRadius::same(4),
                    Stroke::new(1.2_f32, palette.accent_gold),
                    egui::StrokeKind::Outside,
                );
                ui.painter().image(
                    face_tex_id,
                    face_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    if let Some((wname, maybe_pos, pdim, last_lbl)) = &active_info {
                        ui.label(
                            RichText::new(format!("{} • {}", wname, last_lbl))
                                .strong()
                                .size(11.0)
                                .color(palette.text_primary),
                        );
                        if let Some(pos) = maybe_pos {
                            ui.label(
                                RichText::new(format!(
                                    "{}: {} / {} / {} (no vivo)",
                                    pdim.label(lang),
                                    pos[0].round() as i32,
                                    pos[1].round() as i32,
                                    pos[2].round() as i32,
                                ))
                                .monospace()
                                .size(9.5)
                                .color(palette.accent_primary),
                            );
                        } else {
                            ui.label(
                                RichText::new(t(lang, UiKey::PlayerNotLiveBadge))
                                    .size(9.5)
                                    .color(palette.text_muted),
                            );
                        }
                    } else if state.is_server_seed_active {
                        ui.label(
                            RichText::new(t(lang, UiKey::SeedServerModeNote))
                                .size(10.5)
                                .color(palette.accent_primary),
                        );
                    } else {
                        ui.label(
                            RichText::new(t(lang, UiKey::PlayerMarkerTitle))
                                .size(10.5)
                                .color(palette.text_muted),
                        );
                    }
                });
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if let Some((_, Some(pos), pdim, _)) = active_info {
                    if ui.small_button(t(lang, UiKey::PlayerJumpBtn)).clicked() {
                        state.map_dim = pdim;
                        state.jump_map_to(pos[0], pos[2]);
                        audio.play(SoundCue::Click);
                    }
                    if ui.small_button("/tp").clicked() {
                        let cmd = format_tp_xyz(
                            pdim,
                            pos[0].round() as i32,
                            Some(pos[1].round() as i32),
                            pos[2].round() as i32,
                        );
                        ui.ctx().copy_text(cmd.clone());
                        audio.play(SoundCue::Toast);
                        toasts.push(
                            t(lang, UiKey::CopiedTpTitle),
                            cmd,
                            "compass",
                            palette.accent_gold,
                        );
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(t(lang, UiKey::SeedChangeSkinShortcut)).clicked() {
                        state.seed_sidebar_tab = SeedSidebarTab::Worlds;
                        audio.play(SoundCue::Click);
                    }
                });
            });
        });
}

fn render_layers_sidebar_tab(
    ui: &mut egui::Ui,
    state: &mut AppState,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    lang: Language,
) {
    // Quick Layer Presets
    ui.columns(3, |cols| {
        if cols[0]
            .add_sized(
                [cols[0].available_width(), 24.0],
                egui::Button::new(RichText::new(t(lang, UiKey::LayersPresetAll)).size(11.0)),
            )
            .clicked()
        {
            state.map_layers = LayerToggles::default();
            audio.play(SoundCue::Click);
        }
        if cols[1]
            .add_sized(
                [cols[1].available_width(), 24.0],
                egui::Button::new(RichText::new(t(lang, UiKey::LayersPresetCore)).size(11.0)),
            )
            .clicked()
        {
            state.map_layers = LayerToggles {
                biomes: true,
                grid_chunks: true,
                waypoints: true,
                player_marker: true,
                villages: true,
                trial_chambers: true,
                ancient_cities: true,
                strongholds: true,
                fortresses: true,
                bastions: true,
                end_cities: true,
                slime_chunks: false,
                monuments: false,
                mansions: false,
                outposts: false,
                temples: false,
                ruined_portals: false,
                end_gateways: false,
            };
            audio.play(SoundCue::Click);
        }
        if cols[2]
            .add_sized(
                [cols[2].available_width(), 24.0],
                egui::Button::new(RichText::new(t(lang, UiKey::LayersPresetNone)).size(11.0)),
            )
            .clicked()
        {
            state.map_layers = LayerToggles {
                biomes: true,
                grid_chunks: true,
                waypoints: true,
                player_marker: true,
                slime_chunks: false,
                villages: false,
                trial_chambers: false,
                ancient_cities: false,
                strongholds: false,
                monuments: false,
                mansions: false,
                outposts: false,
                temples: false,
                ruined_portals: false,
                fortresses: false,
                bastions: false,
                end_cities: false,
                end_gateways: false,
            };
            audio.play(SoundCue::Click);
        }
    });

    ui.add_space(7.0);

    // Card 1: Map & Overlay Toggles (clean 2-column grid)
    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(t(lang, UiKey::LayersTerrainHeader))
                    .strong()
                    .size(12.0)
                    .color(palette.accent_gold),
            );
            ui.add_space(6.0);

            ui.columns(2, |cols| {
                layer_tile_toggle(
                    &mut cols[0],
                    atlas,
                    "grass_block",
                    if matches!(lang, Language::EnUs) { "Biomes" } else { "Biomas" },
                    t(lang, UiKey::LayerBiomes),
                    &mut state.map_layers.biomes,
                    palette,
                );
                layer_tile_toggle(
                    &mut cols[1],
                    atlas,
                    "slime_ball",
                    t(lang, UiKey::LayerSlimeChunks),
                    t(lang, UiKey::LayerSlimeChunks),
                    &mut state.map_layers.slime_chunks,
                    palette,
                );
            });
            ui.add_space(4.0);
            ui.columns(2, |cols| {
                layer_tile_toggle(
                    &mut cols[0],
                    atlas,
                    "scaffolding",
                    if matches!(lang, Language::EnUs) { "Chunk Grid" } else { "Grilla" },
                    t(lang, UiKey::LayerGridChunks),
                    &mut state.map_layers.grid_chunks,
                    palette,
                );
                layer_tile_toggle(
                    &mut cols[1],
                    atlas,
                    "beacon",
                    t(lang, UiKey::LayerWaypoints),
                    t(lang, UiKey::LayerWaypoints),
                    &mut state.map_layers.waypoints,
                    palette,
                );
            });
            ui.add_space(4.0);
            ui.columns(2, |cols| {
                layer_tile_toggle(
                    &mut cols[0],
                    atlas,
                    "compass",
                    if matches!(lang, Language::EnUs) { "Player" } else { "Tu Jugador" },
                    t(lang, UiKey::PlayerLayerLabel),
                    &mut state.map_layers.player_marker,
                    palette,
                );
            });
        });

    ui.add_space(7.0);

    // Card 2: Dimension Structures (clean 2-column grid)
    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(t(lang, UiKey::LayersStructuresHeader))
                    .strong()
                    .size(12.0)
                    .color(palette.accent_gold),
            );
            ui.add_space(6.0);

            match state.map_dim {
                Dimension::Overworld => {
                    ui.columns(2, |cols| {
                        layer_tile_toggle(
                            &mut cols[0],
                            atlas,
                            StructureKind::Village.icon_id(),
                            StructureKind::Village.label(lang),
                            StructureKind::Village.label(lang),
                            &mut state.map_layers.villages,
                            palette,
                        );
                        layer_tile_toggle(
                            &mut cols[1],
                            atlas,
                            StructureKind::TrialChambers.icon_id(),
                            if matches!(lang, Language::EnUs) { "Trials" } else { "Desafío" },
                            StructureKind::TrialChambers.label(lang),
                            &mut state.map_layers.trial_chambers,
                            palette,
                        );
                    });
                    ui.add_space(4.0);
                    ui.columns(2, |cols| {
                        layer_tile_toggle(
                            &mut cols[0],
                            atlas,
                            StructureKind::AncientCity.icon_id(),
                            if matches!(lang, Language::EnUs) { "Ancient City" } else { "C. Antigua" },
                            StructureKind::AncientCity.label(lang),
                            &mut state.map_layers.ancient_cities,
                            palette,
                        );
                        layer_tile_toggle(
                            &mut cols[1],
                            atlas,
                            StructureKind::Stronghold.icon_id(),
                            "Stronghold",
                            StructureKind::Stronghold.label(lang),
                            &mut state.map_layers.strongholds,
                            palette,
                        );
                    });
                    ui.add_space(4.0);
                    ui.columns(2, |cols| {
                        layer_tile_toggle(
                            &mut cols[0],
                            atlas,
                            StructureKind::Monument.icon_id(),
                            if matches!(lang, Language::EnUs) { "Monument" } else { "Monumento" },
                            StructureKind::Monument.label(lang),
                            &mut state.map_layers.monuments,
                            palette,
                        );
                        layer_tile_toggle(
                            &mut cols[1],
                            atlas,
                            StructureKind::Mansion.icon_id(),
                            if matches!(lang, Language::EnUs) { "Mansion" } else { "Mansión" },
                            StructureKind::Mansion.label(lang),
                            &mut state.map_layers.mansions,
                            palette,
                        );
                    });
                    ui.add_space(4.0);
                    ui.columns(2, |cols| {
                        layer_tile_toggle(
                            &mut cols[0],
                            atlas,
                            StructureKind::Outpost.icon_id(),
                            "Outpost",
                            StructureKind::Outpost.label(lang),
                            &mut state.map_layers.outposts,
                            palette,
                        );
                        layer_tile_toggle(
                            &mut cols[1],
                            atlas,
                            StructureKind::DesertPyramid.icon_id(),
                            if matches!(lang, Language::EnUs) { "Temples" } else { "Templos" },
                            "Templos / Ruinas",
                            &mut state.map_layers.temples,
                            palette,
                        );
                    });
                    ui.add_space(4.0);
                    ui.columns(2, |cols| {
                        layer_tile_toggle(
                            &mut cols[0],
                            atlas,
                            StructureKind::RuinedPortal.icon_id(),
                            if matches!(lang, Language::EnUs) { "Ruined Portal" } else { "Portal Ruina" },
                            StructureKind::RuinedPortal.label(lang),
                            &mut state.map_layers.ruined_portals,
                            palette,
                        );
                    });
                }
                Dimension::Nether => {
                    ui.columns(2, |cols| {
                        layer_tile_toggle(
                            &mut cols[0],
                            atlas,
                            StructureKind::Fortress.icon_id(),
                            StructureKind::Fortress.label(lang),
                            StructureKind::Fortress.label(lang),
                            &mut state.map_layers.fortresses,
                            palette,
                        );
                        layer_tile_toggle(
                            &mut cols[1],
                            atlas,
                            StructureKind::Bastion.icon_id(),
                            StructureKind::Bastion.label(lang),
                            StructureKind::Bastion.label(lang),
                            &mut state.map_layers.bastions,
                            palette,
                        );
                    });
                    ui.add_space(4.0);
                    ui.columns(2, |cols| {
                        layer_tile_toggle(
                            &mut cols[0],
                            atlas,
                            StructureKind::RuinedPortal.icon_id(),
                            if matches!(lang, Language::EnUs) { "Ruined Portal" } else { "Portal Ruina" },
                            StructureKind::RuinedPortal.label(lang),
                            &mut state.map_layers.ruined_portals,
                            palette,
                        );
                    });
                }
                Dimension::TheEnd => {
                    ui.columns(2, |cols| {
                        layer_tile_toggle(
                            &mut cols[0],
                            atlas,
                            StructureKind::EndShip.icon_id(),
                            StructureKind::EndShip.label(lang),
                            StructureKind::EndShip.label(lang),
                            &mut state.map_layers.end_cities,
                            palette,
                        );
                        layer_tile_toggle(
                            &mut cols[1],
                            atlas,
                            StructureKind::EndGateway.icon_id(),
                            StructureKind::EndGateway.label(lang),
                            StructureKind::EndGateway.label(lang),
                            &mut state.map_layers.end_gateways,
                            palette,
                        );
                    });
                }
            }
        });
}

fn layer_tile_toggle(
    ui: &mut egui::Ui,
    atlas: &ItemAtlas,
    icon: &str,
    label: &str,
    tooltip: &str,
    val: &mut bool,
    palette: McPalette,
) {
    let desired = Vec2::new(ui.available_width(), 27.0);
    let (rect, resp) = ui.allocate_exact_size(desired, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let fill = if *val {
            Color32::from_rgba_unmultiplied(
                palette.accent_primary.r(),
                palette.accent_primary.g(),
                palette.accent_primary.b(),
                34,
            )
        } else if resp.hovered() {
            palette.bg_card_hover
        } else {
            palette.bg_card
        };
        let stroke = if *val {
            Stroke::new(1.2_f32, palette.accent_primary)
        } else {
            Stroke::new(1.0_f32, palette.border_subtle)
        };
        ui.painter()
            .rect(rect, CornerRadius::same(4), fill, stroke, egui::StrokeKind::Inside);

        let icon_rect = Rect::from_min_size(
            Pos2::new(rect.left() + 6.0, rect.center().y - 8.0),
            Vec2::splat(16.0),
        );
        let tint = if *val {
            Color32::WHITE
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 130)
        };
        atlas.paint(ui.painter(), icon_rect, icon, tint);

        let text_col = if *val {
            palette.text_primary
        } else {
            palette.text_muted
        };
        let galley = ui.painter().layout(
            label.to_string(),
            egui::FontId::proportional(11.0),
            text_col,
            (rect.width() - 30.0).max(20.0),
        );
        ui.painter().galley(
            Pos2::new(rect.left() + 26.0, rect.center().y - galley.size().y * 0.5),
            galley,
            text_col,
        );
    }
    let clicked = resp
        .on_hover_text(tooltip)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked();
    if clicked {
        *val = !*val;
    }
}

fn render_tool_tooltip(ui: &mut egui::Ui, palette: McPalette, title: &str, desc: &str) {
    ui.set_max_width(275.0);
    ui.label(RichText::new(title).strong().size(12.0).color(palette.accent_gold));
    ui.add_space(2.0);
    ui.add(
        egui::Label::new(RichText::new(desc).size(11.0).color(palette.text_primary)).wrap(),
    );
}

fn format_tp_xyz(dim: Dimension, x: i32, y: Option<i32>, z: i32) -> String {
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

fn format_tp_command(dim: Dimension, x: i32, z: i32) -> String {
    format_tp_xyz(dim, x, None, z)
}

fn render_map_viewport(
    ui: &mut egui::Ui,
    state: &mut AppState,
    engine: &mut SeedMapEngine,
    rcx: &RenderContext<'_>,
    toasts: &mut ToastQueue,
) {
    let RenderContext {
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
    let lang = state.prefs.language;
    let avail = ui.available_size();
    let hud_h = 52.0;
    let canvas_h = (avail.y - hud_h - 6.0).max(220.0);
    let (canvas_rect, resp) = ui.allocate_exact_size(
        Vec2::new(avail.x, canvas_h),
        egui::Sense::click_and_drag(),
    );

    paint_mc_frame(ui.painter(), canvas_rect, palette, Some(palette.border_strong));

    let view_rect = canvas_rect.shrink(2.0);
    let painter = ui.painter().with_clip_rect(view_rect);

    // 1. Dark dimension canvas background
    let dim_bg = match state.map_dim {
        Dimension::Overworld => Color32::from_rgb(16, 28, 36),
        Dimension::Nether => Color32::from_rgb(38, 12, 14),
        Dimension::TheEnd => Color32::from_rgb(12, 8, 22),
    };
    painter.rect_filled(view_rect, 2.0, dim_bg);

    // 2. Drag Pan (never triggers click!)
    if resp.dragged() {
        let delta = resp.drag_delta();
        let bpp = state.map_blocks_per_px as f64;
        state.map_center_x -= (delta.x as f64) * bpp;
        state.map_center_z -= (delta.y as f64) * bpp;
        state.map_target_center_x = state.map_center_x;
        state.map_target_center_z = state.map_center_z;
    }

    // 3. Smooth Cursor-Anchored Zoom on Scroll
    let hovered = resp.hovered();
    let pointer_pos = ui.ctx().pointer_latest_pos().filter(|p| view_rect.contains(*p));
    if hovered {
        let scroll_y = ui.ctx().input(|i| i.smooth_scroll_delta.y);
        if scroll_y.abs() > 0.5 {
            let factor = (-scroll_y * 0.0028).exp();
            let old_bpp = state.map_target_bpp;
            let new_bpp = (old_bpp * factor).clamp(0.30, 160.0);
            if let Some(p) = pointer_pos {
                let dx_px = (p.x - view_rect.center().x) as f64;
                let dy_px = (p.y - view_rect.center().y) as f64;
                let world_x = state.map_target_center_x + dx_px * (old_bpp as f64);
                let world_z = state.map_target_center_z + dy_px * (old_bpp as f64);
                state.map_target_center_x = world_x - dx_px * (new_bpp as f64);
                state.map_target_center_z = world_z - dy_px * (new_bpp as f64);
            }
            state.map_target_bpp = new_bpp;
        }
    }

    let bpp = state.map_blocks_per_px;
    let center_x = state.map_center_x;
    let center_z = state.map_center_z;
    let half_w_blocks = (view_rect.width() * 0.5 * bpp) as f64;
    let half_h_blocks = (view_rect.height() * 0.5 * bpp) as f64;
    let min_bx = center_x - half_w_blocks;
    let max_bx = center_x + half_w_blocks;
    let min_bz = center_z - half_h_blocks;
    let max_bz = center_z + half_h_blocks;

    let world_to_screen = |wx: f64, wz: f64| -> Pos2 {
        Pos2::new(
            view_rect.center().x + ((wx - center_x) / (bpp as f64)) as f32,
            view_rect.center().y + ((wz - center_z) / (bpp as f64)) as f32,
        )
    };
    let screen_to_world = |pos: Pos2| -> (f64, f64) {
        (
            center_x + ((pos.x - view_rect.center().x) as f64) * (bpp as f64),
            center_z + ((pos.y - view_rect.center().y) as f64) * (bpp as f64),
        )
    };

    // 4. Async Cubiomes Terrain Tile Pyramid
    if state.map_layers.biomes {
        let scale = pick_cubiomes_scale(bpp);
        let tile_blocks = (TILE_PX as i32 * scale) as f64;
        let tx0 = (min_bx / tile_blocks).floor() as i32;
        let tx1 = (max_bx / tile_blocks).ceil() as i32;
        let tz0 = (min_bz / tile_blocks).floor() as i32;
        let tz1 = (max_bz / tile_blocks).ceil() as i32;

        // Bound iteration safely on extreme aspect ratios and prioritize center-out tile streaming
        if (tx1 - tx0) <= 48 && (tz1 - tz0) <= 48 {
            let center_tx = (state.map_center_x / tile_blocks) - 0.5;
            let center_tz = (state.map_center_z / tile_blocks) - 0.5;
            let mut order: Vec<(f64, i32, i32)> =
                Vec::with_capacity(((tx1 - tx0 + 1) * (tz1 - tz0 + 1)).max(0) as usize);
            for tz in tz0..=tz1 {
                for tx in tx0..=tx1 {
                    let dx = (tx as f64) - center_tx;
                    let dz = (tz as f64) - center_tz;
                    order.push((dx * dx + dz * dz, tx, tz));
                }
            }
            order.sort_by(|a, b| a.0.total_cmp(&b.0));

            for (_, tx, tz) in order {
                let bx0 = (tx as f64) * tile_blocks;
                let bz0 = (tz as f64) * tile_blocks;
                let p0 = world_to_screen(bx0, bz0);
                let p1 = world_to_screen(bx0 + tile_blocks, bz0 + tile_blocks);
                let trect = Rect::from_min_max(p0, p1);
                if let Some((tex, uv)) =
                    engine.lookup_tile_or_fallback(state.map_dim, scale, state.map_style, tx, tz)
                {
                    painter.image(tex.id(), trect, uv, Color32::WHITE);
                } else {
                    painter.rect_filled(trect.shrink(0.5), 0.0, Color32::from_black_alpha(45));
                }
            }
        }
    }

    // 5. Slime Chunks Overlay (Overworld only when zoomed in cleanly so zero clutter!)
    if state.map_layers.slime_chunks && state.map_dim == Dimension::Overworld && bpp <= 4.0 {
        let cx0 = (min_bx / 16.0).floor() as i32;
        let cx1 = (max_bx / 16.0).ceil() as i32;
        let cz0 = (min_bz / 16.0).floor() as i32;
        let cz1 = (max_bz / 16.0).ceil() as i32;
        if (cx1 - cx0) <= 80 && (cz1 - cz0) <= 60 {
            for cz in cz0..=cz1 {
                for cx in cx0..=cx1 {
                    if is_slime_chunk(engine.seed, cx, cz) {
                        let p0 = world_to_screen((cx * 16) as f64, (cz * 16) as f64);
                        let p1 = world_to_screen(((cx + 1) * 16) as f64, ((cz + 1) * 16) as f64);
                        let crect = Rect::from_min_max(p0, p1);
                        painter.rect_filled(
                            crect,
                            0.0,
                            Color32::from_rgba_unmultiplied(64, 240, 64, 82),
                        );
                        painter.rect_stroke(
                            crect,
                            0.0,
                            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(88, 255, 88, 175)),
                            egui::StrokeKind::Inside,
                        );
                    }
                }
            }
        }
    }

    // 6. Subtle Chunk / Region Grid + (0,0) Origin Axes
    if state.map_layers.grid_chunks {
        // 512-block Region (.mca) lines always subtle
        let rx0 = (min_bx / 512.0).floor() as i32;
        let rx1 = (max_bx / 512.0).ceil() as i32;
        let rz0 = (min_bz / 512.0).floor() as i32;
        let rz1 = (max_bz / 512.0).ceil() as i32;
        if (rx1 - rx0) <= 32 {
            for rx in rx0..=rx1 {
                let x = world_to_screen((rx * 512) as f64, 0.0).x;
                painter.line_segment(
                    [Pos2::new(x, view_rect.top()), Pos2::new(x, view_rect.bottom())],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 28)),
                );
            }
            for rz in rz0..=rz1 {
                let y = world_to_screen(0.0, (rz * 512) as f64).y;
                painter.line_segment(
                    [Pos2::new(view_rect.left(), y), Pos2::new(view_rect.right(), y)],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 28)),
                );
            }
        }

        // 16-block chunk grid when zoomed in close
        if bpp <= 1.8 {
            let gx0 = (min_bx / 16.0).floor() as i32;
            let gx1 = (max_bx / 16.0).ceil() as i32;
            let gz0 = (min_bz / 16.0).floor() as i32;
            let gz1 = (max_bz / 16.0).ceil() as i32;
            for gx in gx0..=gx1 {
                let x = world_to_screen((gx * 16) as f64, 0.0).x;
                painter.line_segment(
                    [Pos2::new(x, view_rect.top()), Pos2::new(x, view_rect.bottom())],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 16)),
                );
            }
            for gz in gz0..=gz1 {
                let y = world_to_screen(0.0, (gz * 16) as f64).y;
                painter.line_segment(
                    [Pos2::new(view_rect.left(), y), Pos2::new(view_rect.right(), y)],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 16)),
                );
            }
        }
    }

    // Center crosshairs at X=0, Z=0
    let origin = world_to_screen(0.0, 0.0);
    if origin.x >= view_rect.left() && origin.x <= view_rect.right() {
        painter.line_segment(
            [Pos2::new(origin.x, view_rect.top()), Pos2::new(origin.x, view_rect.bottom())],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 55)),
        );
    }
    if origin.y >= view_rect.top() && origin.y <= view_rect.bottom() {
        painter.line_segment(
            [Pos2::new(view_rect.left(), origin.y), Pos2::new(view_rect.right(), origin.y)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 55)),
        );
    }

    // 7. Structure Markers with Screen-Space Clustering + Crisp PNG Sprites
    let zoom_level = 1.0 / bpp.max(0.1);
    let scan = engine.structures_in_view(
        state.map_dim,
        min_bx.floor() as i32,
        min_bz.floor() as i32,
        max_bx.ceil() as i32,
        max_bz.ceil() as i32,
        state.map_layers,
    );

    let mut all_markers: Vec<StructureMarker> = scan.to_vec();
    if state.map_dim == Dimension::Overworld {
        let sp = engine.spawn_point();
        all_markers.push(StructureMarker {
            kind: StructureKind::SpawnPoint,
            x: sp.0,
            z: sp.1,
            variant: 0,
            biome_id: 0,
        });
    }

    // Sort highest priority first so clustering keeps rare/vital structures over common villages
    all_markers.sort_by_key(|m| structure_priority(m.kind));

    let mut placed_screen_pts: Vec<Pos2> = Vec::new();
    let mut hovered_marker: Option<(StructureMarker, String)> = None;

    for m in all_markers {
        if zoom_level < m.kind.min_zoom_visible() {
            continue;
        }
        // Hide inner 96-block End pillar gateway ring unless zoomed in close so (0,0) stays clear
        if m.kind == StructureKind::EndGateway && m.variant >= 1 && m.variant <= 20 && bpp > 2.2 {
            continue;
        }
        let sp = world_to_screen(m.x as f64, m.z as f64);
        if !view_rect.expand(18.0).contains(sp) {
            continue;
        }
        // Keep rare landmark icons visible even if a denser structure sits nearby; only cull dense overlaps
        let cull_dist_sq = match m.kind {
            StructureKind::SpawnPoint
            | StructureKind::Stronghold
            | StructureKind::Mansion
            | StructureKind::EndShip => 0.0,
            StructureKind::AncientCity | StructureKind::Monument => 12.0 * 12.0,
            _ => 28.0 * 28.0,
        };
        if cull_dist_sq > 0.0
            && placed_screen_pts
                .iter()
                .any(|p| p.distance_sq(sp) < cull_dist_sq)
        {
            continue;
        }
        placed_screen_pts.push(sp);

        let badge_r = if matches!(
            m.kind,
            StructureKind::SpawnPoint | StructureKind::Stronghold | StructureKind::EndShip
        ) {
            13.0
        } else {
            11.0
        };
        let accent = m.kind.accent_color();
        painter.circle_filled(sp, badge_r, Color32::from_rgba_unmultiplied(12, 14, 20, 224));
        painter.circle_stroke(sp, badge_r, Stroke::new(1.8_f32, accent));
        let icon_rect = Rect::from_center_size(sp, Vec2::splat(badge_r * 1.45));
        atlas.paint(&painter, icon_rect, m.kind.icon_id(), Color32::WHITE);

        if let Some(ptr) = pointer_pos {
            if ptr.distance(sp) <= badge_r + 4.0 && hovered_marker.is_none() {
                hovered_marker = Some((m.clone(), m.kind.label(lang).to_string()));
            }
        }
    }

    // 8. User Seed Waypoints Overlay
    let mut hovered_waypoint: Option<crate::state::Waypoint> = None;
    if state.map_layers.waypoints {
        for wp in &state.prefs.waypoints {
            if wp.seed != engine.seed || wp.dim != state.map_dim as i32 {
                continue;
            }
            let sp = world_to_screen(wp.x as f64, wp.z as f64);
            if view_rect.expand(24.0).contains(sp) {
                painter.circle_filled(sp, 12.0, Color32::from_rgba_unmultiplied(16, 22, 34, 235));
                painter.circle_stroke(sp, 12.0, Stroke::new(2.0_f32, palette.accent_primary));
                atlas.paint(
                    &painter,
                    Rect::from_center_size(sp, Vec2::splat(16.0)),
                    &wp.icon,
                    Color32::WHITE,
                );
                painter.text(
                    sp + Vec2::new(0.0, 15.0),
                    egui::Align2::CENTER_TOP,
                    &wp.name,
                    egui::FontId::proportional(11.0),
                    Color32::WHITE,
                );
                if let Some(ptr) = pointer_pos {
                    if ptr.distance(sp) <= 16.0 && hovered_waypoint.is_none() {
                        hovered_waypoint = Some(wp.clone());
                    }
                }
            }
        }
    }

    // 8c. Saved Player Location Marker (with uploaded skin face or classic Steve)
    let player_marker_data: Option<(String, [f64; 3], Dimension, String)> = if state.map_layers.player_marker {
        state.active_world_for_seed(engine.seed).and_then(|w| {
            w.player_pos.map(|pos| {
                (
                    w.level_name.clone(),
                    pos,
                    w.player_dimension(),
                    w.last_played_label(lang),
                )
            })
        })
    } else {
        None
    };
    let face_tex_id = state.ensure_player_face_tex(ui.ctx()).id();
    let mut hovered_player: Option<(String, [f64; 3], Dimension, String)> = None;
    if let Some((ref wname, pos, pdim, ref last_lbl)) = player_marker_data {
        if pdim == state.map_dim {
            let sp = world_to_screen(pos[0], pos[2]);
            if view_rect.expand(48.0).contains(sp) {
                let face_size = 24.0;
                let head_rect = Rect::from_center_size(sp, Vec2::splat(face_size));
                // Crisp contrasting ring + pixel-art player face square
                painter.rect_filled(
                    head_rect.expand(3.0),
                    CornerRadius::same(5),
                    Color32::from_rgba_unmultiplied(10, 14, 22, 235),
                );
                painter.rect_stroke(
                    head_rect.expand(2.5),
                    CornerRadius::same(5),
                    Stroke::new(2.0_f32, palette.accent_gold),
                    egui::StrokeKind::Outside,
                );
                painter.image(
                    face_tex_id,
                    head_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );

                // Explicit non-live disclosure badge beneath player marker
                let badge_txt = match lang {
                    Language::EnUs => "Last saved (not live)",
                    _ => "Última guardada (no en vivo)",
                };
                let badge_w = (badge_txt.len() as f32 * 5.6 + 12.0).max(122.0);
                let badge_rect = Rect::from_center_size(
                    sp + Vec2::new(0.0, face_size * 0.5 + 12.0),
                    Vec2::new(badge_w, 16.0),
                );
                painter.rect_filled(
                    badge_rect,
                    CornerRadius::same(4),
                    Color32::from_rgba_unmultiplied(12, 16, 26, 226),
                );
                painter.rect_stroke(
                    badge_rect,
                    CornerRadius::same(4),
                    Stroke::new(1.0_f32, palette.accent_gold),
                    egui::StrokeKind::Inside,
                );
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    badge_txt,
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(255, 224, 130),
                );

                if let Some(ptr) = pointer_pos {
                    if (ptr.distance(sp) <= 18.0 || badge_rect.expand(4.0).contains(ptr))
                        && hovered_marker.is_none()
                        && hovered_waypoint.is_none()
                    {
                        hovered_player = Some((wname.clone(), pos, pdim, last_lbl.clone()));
                    }
                }
            }
        }
    }

    // 8b. Temporary Google-Maps-Style Pin Overlay
    let mut temp_pin_screen: Option<Pos2> = None;
    if let Some((pdim, px, pz)) = state.temp_map_pin {
        if pdim == state.map_dim {
            let sp = world_to_screen(px as f64, pz as f64);
            if view_rect.expand(36.0).contains(sp) {
                temp_pin_screen = Some(sp);
                let head = sp - Vec2::new(0.0, 22.0);
                // Ground drop shadow
                painter.circle_filled(
                    sp + Vec2::new(0.0, 1.0),
                    5.5,
                    Color32::from_black_alpha(120),
                );
                // Pointer stem triangle + pin body
                painter.add(egui::Shape::convex_polygon(
                    vec![
                        sp,
                        head + Vec2::new(-8.5, 4.5),
                        head + Vec2::new(8.5, 4.5),
                    ],
                    Color32::from_rgb(225, 58, 52),
                    Stroke::new(1.5_f32, Color32::WHITE),
                ));
                painter.circle_filled(head, 10.5, Color32::from_rgb(232, 62, 54));
                painter.circle_stroke(head, 10.5, Stroke::new(1.8_f32, Color32::WHITE));
                painter.circle_filled(head, 4.0, Color32::from_rgb(36, 14, 16));

                let pin_txt = format!("{}, {}", px, pz);
                let tag_rect = Rect::from_center_size(
                    sp + Vec2::new(0.0, 14.0),
                    Vec2::new((pin_txt.len() as f32 * 6.8 + 12.0).max(58.0), 17.0),
                );
                painter.rect_filled(tag_rect, 4.0, Color32::from_rgba_unmultiplied(16, 18, 26, 220));
                painter.rect_stroke(
                    tag_rect,
                    4.0,
                    Stroke::new(1.0_f32, Color32::from_rgb(232, 62, 54)),
                    egui::StrokeKind::Inside,
                );
                painter.text(
                    tag_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    pin_txt,
                    egui::FontId::monospace(10.5),
                    Color32::WHITE,
                );
            }
        }
    }

    // Prevent clicks on floating map overlays (temp-pin bar or right-click menu) from hitting canvas
    let overlay_rects: (Option<Rect>, Option<Rect>) = ui.ctx().data(|d| {
        (
            d.get_temp::<Rect>(egui::Id::new("seed_map_pin_bar_rect")),
            d.get_temp::<Rect>(egui::Id::new("seed_map_ctx_menu_rect")),
        )
    });
    let pointer_on_overlay = ui.ctx().pointer_latest_pos().is_some_and(|p| {
        overlay_rects.0.is_some_and(|r| r.contains(p))
            || overlay_rects.1.is_some_and(|r| r.contains(p))
    });

    // 9. Interactive Clicks: Ruler | Structure /tp Copy | Temp Google-Maps Pin | Right-Click Menu
    let shift_held = ui.ctx().input(|i| i.modifiers.shift);
    let left_clicked = resp.clicked_by(egui::PointerButton::Primary) && !pointer_on_overlay;
    let right_clicked = resp.clicked_by(egui::PointerButton::Secondary) && !pointer_on_overlay;

    if left_clicked {
        if state.map_context_menu.is_some() {
            state.map_context_menu = None;
        } else if let Some(ptr) = resp.interact_pointer_pos() {
            if state.ruler_active || shift_held {
                let (wx, wz) = screen_to_world(ptr);
                let pt = (wx.round() as i32, wz.round() as i32);
                if state.ruler_p1.is_none() || state.ruler_p2.is_some() {
                    state.ruler_p1 = Some(pt);
                    state.ruler_p2 = None;
                } else {
                    state.ruler_p2 = Some(pt);
                }
                audio.play(SoundCue::Click);
            } else if let Some((ref m, ref title)) = hovered_marker {
                let cmd = format_tp_command(state.map_dim, m.x, m.z);
                ui.ctx().copy_text(cmd.clone());
                audio.play(SoundCue::Toast);
                toasts.push(
                    t(lang, UiKey::CopiedTpTitle),
                    format!("{}: {}", title, cmd),
                    m.kind.icon_id(),
                    m.kind.accent_color(),
                );
            } else if let Some(ref wp) = hovered_waypoint {
                let cmd = format_tp_command(wp.dimension(), wp.x, wp.z);
                ui.ctx().copy_text(cmd.clone());
                audio.play(SoundCue::Toast);
                toasts.push(
                    t(lang, UiKey::CopiedTpTitle),
                    format!("{}: {}", wp.name, cmd),
                    &wp.icon,
                    palette.accent_primary,
                );
            } else if let Some((ref wname, pos, pdim, _)) = hovered_player {
                let cmd = format_tp_xyz(
                    pdim,
                    pos[0].round() as i32,
                    Some(pos[1].round() as i32),
                    pos[2].round() as i32,
                );
                ui.ctx().copy_text(cmd.clone());
                audio.play(SoundCue::Toast);
                toasts.push(
                    t(lang, UiKey::CopiedTpTitle),
                    format!("{}: {}", wname, cmd),
                    "compass",
                    palette.accent_gold,
                );
            } else {
                let clicked_existing_pin = temp_pin_screen.is_some_and(|ps| {
                    ptr.distance(ps) <= 22.0 || ptr.distance(ps - Vec2::new(0.0, 18.0)) <= 16.0
                });
                if clicked_existing_pin {
                    state.temp_map_pin = None;
                } else {
                    let (wx, wz) = screen_to_world(ptr);
                    state.temp_map_pin =
                        Some((state.map_dim, wx.round() as i32, wz.round() as i32));
                }
                audio.play(SoundCue::Click);
            }
        }
    }

    if right_clicked {
        if let Some(ptr) = resp.interact_pointer_pos() {
            let (bx, bz, suggested) = if let Some((ref m, ref title)) = hovered_marker {
                (m.x, m.z, title.clone())
            } else {
                let (wx, wz) = screen_to_world(ptr);
                (wx.round() as i32, wz.round() as i32, String::new())
            };
            state.map_context_menu = Some(MapContextMenuState {
                dim: state.map_dim,
                x: bx,
                z: bz,
                screen_pos: ptr,
                name: suggested,
                icon: state.new_waypoint_icon.clone(),
                focus_name_next: true,
            });
            audio.play(SoundCue::Click);
        }
    }
    if let Some(p1) = state.ruler_p1 {
        let s1 = world_to_screen(p1.0 as f64, p1.1 as f64);
        painter.circle_filled(s1, 5.0, palette.accent_gold);
        let p2_world = state.ruler_p2.or_else(|| {
            pointer_pos.map(|p| {
                let (wx, wz) = screen_to_world(p);
                (wx.round() as i32, wz.round() as i32)
            })
        });
        if let Some(p2) = p2_world {
            let s2 = world_to_screen(p2.0 as f64, p2.1 as f64);
            painter.line_segment([s1, s2], Stroke::new(2.0_f32, palette.accent_gold));
            painter.circle_filled(s2, 5.0, palette.accent_gold);
            let dx = (p2.0 - p1.0) as f64;
            let dz = (p2.1 - p1.1) as f64;
            let dist = (dx * dx + dz * dz).sqrt().round() as i64;
            let mid = Pos2::new((s1.x + s2.x) * 0.5, (s1.y + s2.y) * 0.5);
            painter.text(
                mid + Vec2::new(0.0, -10.0),
                egui::Align2::CENTER_BOTTOM,
                format!("{} blocks (Nether: {})", dist, (dist as f64 / 8.0).round() as i64),
                egui::FontId::monospace(12.0),
                palette.accent_gold,
            );
        }
    }

    // Floating Temp-Pin Quick Bar inside top-right corner of map
    let mut new_pin_bar_rect: Option<Rect> = None;
    if let Some((pdim, px, pz)) = state.temp_map_pin {
        if pdim == state.map_dim {
            let area_resp = egui::Area::new(egui::Id::new("seed_map_temp_pin_bar"))
                .order(egui::Order::Middle)
                .fixed_pos(Pos2::new(
                    (view_rect.right() - 370.0).max(view_rect.left() + 8.0),
                    view_rect.top() + 8.0,
                ))
                .constrain_to(view_rect)
                .show(ui.ctx(), |ui| {
                    egui::Frame::new()
                        .fill(Color32::from_rgba_unmultiplied(
                            palette.bg_panel.r(),
                            palette.bg_panel.g(),
                            palette.bg_panel.b(),
                            235,
                        ))
                        .stroke(Stroke::new(1.5_f32, Color32::from_rgb(232, 62, 54)))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin::symmetric(8, 5))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(atlas.ui_image("compass", 15.0));
                                ui.label(
                                    RichText::new(format!(
                                        "{}: {}, {}",
                                        t(lang, UiKey::TempPinLabel),
                                        px,
                                        pz
                                    ))
                                    .monospace()
                                    .strong()
                                    .size(11.5)
                                    .color(palette.text_primary),
                                );
                                if ui.small_button(t(lang, UiKey::CopyTpBtn)).clicked() {
                                    let cmd = format_tp_command(pdim, px, pz);
                                    ui.ctx().copy_text(cmd.clone());
                                    audio.play(SoundCue::Toast);
                                    toasts.push(
                                        t(lang, UiKey::CopiedTpTitle),
                                        cmd,
                                        "compass",
                                        palette.accent_primary,
                                    );
                                }
                                if ui.small_button(t(lang, UiKey::SaveWaypointHereBtn)).clicked() {
                                    let sp = world_to_screen(px as f64, pz as f64);
                                    state.map_context_menu = Some(MapContextMenuState {
                                        dim: pdim,
                                        x: px,
                                        z: pz,
                                        screen_pos: sp,
                                        name: String::new(),
                                        icon: state.new_waypoint_icon.clone(),
                                        focus_name_next: true,
                                    });
                                    audio.play(SoundCue::Click);
                                }
                                if ui
                                    .small_button("✕")
                                    .on_hover_text(t(lang, UiKey::RemovePinBtn))
                                    .clicked()
                                {
                                    state.temp_map_pin = None;
                                    audio.play(SoundCue::Click);
                                }
                            });
                        });
                });
            new_pin_bar_rect = Some(area_resp.response.rect);
        }
    }

    // Floating Right-Click Context Menu for Custom Waypoints (12 Icons + Exact Coords)
    let mut new_ctx_menu_rect: Option<Rect> = None;
    let mut close_ctx_menu = false;
    if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
        close_ctx_menu = true;
    }
    if let Some(mut menu) = state.map_context_menu.take() {
        let area_resp = egui::Area::new(egui::Id::new("seed_map_ctx_menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(menu.screen_pos + Vec2::new(8.0, 8.0))
            .constrain_to(ui.ctx().screen_rect().shrink(8.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(15, 19, 28, 246))
                    .stroke(Stroke::new(1.5_f32, palette.accent_primary))
                    .corner_radius(CornerRadius::same(7))
                    .inner_margin(Margin::same(10))
                    .show(ui, |ui| {
                        ui.set_width(250.0);
                        ui.horizontal(|ui| {
                            ui.add(atlas.ui_image(&menu.icon, 18.0));
                            ui.label(
                                RichText::new(t(lang, UiKey::RightClickMenuTitle))
                                    .strong()
                                    .size(12.5)
                                    .color(palette.accent_gold),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("✕").clicked() {
                                    close_ctx_menu = true;
                                }
                            });
                        });
                        ui.label(
                            RichText::new(format!(
                                "{} • X: {}  Z: {}",
                                menu.dim.label(lang),
                                menu.x,
                                menu.z
                            ))
                            .monospace()
                            .size(11.0)
                            .color(palette.text_muted),
                        );
                        ui.add_space(6.0);

                        let te = ui.add_sized(
                            [230.0, 24.0],
                            egui::TextEdit::singleline(&mut menu.name)
                                .hint_text(t(lang, UiKey::WaypointNameHint)),
                        );
                        if menu.focus_name_next {
                            te.request_focus();
                            menu.focus_name_next = false;
                        }
                        let submit_enter = te.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(format!("{}:", t(lang, UiKey::WaypointIconLabel)))
                                .size(11.0)
                                .color(palette.text_muted),
                        );
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
                            for (icon_id, es_lbl, en_lbl) in WAYPOINT_ICON_PRESETS {
                                let active = menu.icon == icon_id;
                                let (sr, sresp) =
                                    ui.allocate_exact_size(Vec2::splat(28.0), egui::Sense::click());
                                paint_mc_slot(
                                    ui.painter(),
                                    sr,
                                    palette,
                                    atlas,
                                    Some(icon_id),
                                    SlotStyle {
                                        selected: active,
                                        hovered: sresp.hovered(),
                                        ..Default::default()
                                    },
                                );
                                let tip = if matches!(lang, Language::EnUs) { en_lbl } else { es_lbl };
                                if sresp.on_hover_text(tip).clicked() {
                                    menu.icon = icon_id.to_string();
                                    audio.play(SoundCue::Click);
                                }
                            }
                        });

                        ui.add_space(8.0);
                        let save_clicked = ui
                            .add_sized(
                                [230.0, 26.0],
                                egui::Button::new(
                                    RichText::new(t(lang, UiKey::SaveWaypointHereBtn))
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgba_unmultiplied(
                                    palette.accent_xp.r(),
                                    palette.accent_xp.g(),
                                    palette.accent_xp.b(),
                                    65,
                                ))
                                .stroke(Stroke::new(1.5_f32, palette.accent_xp)),
                            )
                            .clicked();

                        if save_clicked || submit_enter {
                            let label_shown = if menu.name.trim().is_empty() {
                                format!("({}, {})", menu.x, menu.z)
                            } else {
                                menu.name.trim().to_string()
                            };
                            state.new_waypoint_icon = menu.icon.clone();
                            state.add_waypoint_at(
                                engine.seed,
                                menu.dim,
                                menu.x,
                                menu.z,
                                &label_shown,
                                &menu.icon,
                            );
                            audio.play(SoundCue::XpOrb);
                            toasts.push(
                                t(lang, UiKey::ToastWaypointAddedTitle),
                                label_shown,
                                &menu.icon,
                                palette.accent_primary,
                            );
                            close_ctx_menu = true;
                        }

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui.button(t(lang, UiKey::CopyTpBtn)).clicked() {
                                let cmd = format_tp_command(menu.dim, menu.x, menu.z);
                                ui.ctx().copy_text(cmd.clone());
                                audio.play(SoundCue::Toast);
                                toasts.push(
                                    t(lang, UiKey::CopiedTpTitle),
                                    cmd,
                                    "compass",
                                    palette.accent_primary,
                                );
                                close_ctx_menu = true;
                            }
                            if ui.button(t(lang, UiKey::TempPinLabel)).clicked() {
                                state.temp_map_pin = Some((menu.dim, menu.x, menu.z));
                                audio.play(SoundCue::Click);
                                close_ctx_menu = true;
                            }
                        });
                    });
            });
        new_ctx_menu_rect = Some(area_resp.response.rect);
        if !close_ctx_menu {
            state.map_context_menu = Some(menu);
        }
    }

    ui.ctx().data_mut(|d| {
        if let Some(r) = new_pin_bar_rect {
            d.insert_temp(egui::Id::new("seed_map_pin_bar_rect"), r);
        } else {
            d.remove::<Rect>(egui::Id::new("seed_map_pin_bar_rect"));
        }
        if let Some(r) = new_ctx_menu_rect {
            d.insert_temp(egui::Id::new("seed_map_ctx_menu_rect"), r);
        } else {
            d.remove::<Rect>(egui::Id::new("seed_map_ctx_menu_rect"));
        }
    });

    // Tooltip for hovered structure badge or waypoint (only when menu isn't covering pointer)
    if !pointer_on_overlay {
        if let (Some(ptr), Some((m, title))) = (pointer_pos, hovered_marker) {
            let coords = format!("X: {}  Z: {}", m.x, m.z);
            let nether_link = format!("Nether (÷8): {}, {}", m.x.div_euclid(8), m.z.div_euclid(8));
            let tp_cmd = format_tp_command(state.map_dim, m.x, m.z);
            let tp_hint = match lang {
                Language::EnUs => format!("Left-Click -> {}", tp_cmd),
                _ => format!("Clic izq. -> {}", tp_cmd),
            };
            show_mc_tooltip_at(
                ui.ctx(),
                ptr,
                &title,
                &[
                    (&coords, palette.accent_primary),
                    (&nether_link, palette.text_muted),
                    (&tp_hint, palette.accent_gold),
                ],
                palette,
            );
        } else if let (Some(ptr), Some(wp)) = (pointer_pos, hovered_waypoint) {
            let coords = format!("X: {}  Z: {}", wp.x, wp.z);
            let tp_cmd = format_tp_command(wp.dimension(), wp.x, wp.z);
            let tp_hint = match lang {
                Language::EnUs => format!("Left-Click -> {}", tp_cmd),
                _ => format!("Clic izq. -> {}", tp_cmd),
            };
            show_mc_tooltip_at(
                ui.ctx(),
                ptr,
                &wp.name,
                &[
                    (&coords, palette.accent_primary),
                    (&tp_hint, palette.accent_gold),
                ],
                palette,
            );
        } else if let (Some(ptr), Some((wname, pos, pdim, last_lbl))) = (pointer_pos, hovered_player) {
            let not_live = t(lang, UiKey::PlayerNotLiveBadge);
            let coords = format!(
                "{} • X: {:.0}  Y: {:.0}  Z: {:.0}",
                pdim.label(lang),
                pos[0],
                pos[1],
                pos[2]
            );
            let played = match lang {
                Language::EnUs => format!("Last played: {}", last_lbl),
                _ => format!("Último guardado: {}", last_lbl),
            };
            let tp_hint = match lang {
                Language::EnUs => "Left-Click -> Copy /tp to saved position",
                _ => "Clic izq. -> Copiar /tp a posición guardada",
            };
            show_mc_tooltip_at(
                ui.ctx(),
                ptr,
                &format!("{} - {}", t(lang, UiKey::PlayerMarkerTitle), wname),
                &[
                    (not_live, Color32::from_rgb(255, 210, 110)),
                    (&coords, palette.accent_primary),
                    (&played, palette.text_muted),
                    (tp_hint, palette.accent_gold),
                ],
                palette,
            );
        }
    }

    ui.add_space(6.0);

    // 10. Bottom Live Coordinate / Biome / Chunk / Slime HUD
    let sample_pos = if let Some(ptr) = pointer_pos {
        let (wx, wz) = screen_to_world(ptr);
        (wx.round() as i32, wz.round() as i32)
    } else {
        (state.map_center_x.round() as i32, state.map_center_z.round() as i32)
    };
    state.last_cursor_world = Some(sample_pos);

    let (_bid, approx_y, biome_name) =
        engine.sample_point(state.map_dim, sample_pos.0, sample_pos.1, lang);
    let chunk_x = sample_pos.0.div_euclid(16);
    let chunk_z = sample_pos.1.div_euclid(16);
    let reg_x = sample_pos.0.div_euclid(512);
    let reg_z = sample_pos.1.div_euclid(512);
    let slime =
        state.map_dim == Dimension::Overworld && is_slime_chunk(engine.seed, chunk_x, chunk_z);

    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(12.0, 4.0);
                ui.label(
                    RichText::new(format!(
                        "{}: {}, {}",
                        t(lang, UiKey::CursorHudBlock),
                        sample_pos.0,
                        sample_pos.1
                    ))
                    .monospace()
                    .strong()
                    .size(12.0)
                    .color(palette.accent_primary),
                );
                ui.label(
                    RichText::new(format!(
                        "{}: {}, {}",
                        t(lang, UiKey::CursorHudChunk),
                        chunk_x,
                        chunk_z
                    ))
                    .monospace()
                    .size(11.5)
                    .color(palette.text_primary),
                );
                ui.label(
                    RichText::new(format!(
                        "{}: r.{}.{}.mca",
                        t(lang, UiKey::CursorHudRegion),
                        reg_x,
                        reg_z
                    ))
                    .monospace()
                    .size(11.5)
                    .color(palette.text_muted),
                );
                ui.label(
                    RichText::new(format!(
                        "{}: {}",
                        t(lang, UiKey::CursorHudBiome),
                        biome_name
                    ))
                    .size(12.0)
                    .strong()
                    .color(palette.accent_gold),
                );
                if state.map_dim == Dimension::Overworld {
                    ui.label(
                        RichText::new(format!("{}: ~{}", t(lang, UiKey::CursorHudHeight), approx_y.round() as i32))
                            .size(11.5)
                            .color(palette.text_muted),
                    );
                    ui.label(
                        RichText::new(if slime {
                            t(lang, UiKey::CursorHudSlimeYes)
                        } else {
                            t(lang, UiKey::CursorHudSlimeNo)
                        })
                        .size(11.5)
                        .strong()
                        .color(if slime { palette.accent_xp } else { palette.text_muted }),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{}: {}, {}",
                            t(lang, UiKey::LinkedCoordsNether),
                            sample_pos.0.div_euclid(8),
                            sample_pos.1.div_euclid(8)
                        ))
                        .monospace()
                        .size(11.5)
                        .color(Color32::from_rgb(255, 130, 96)),
                    );
                } else if state.map_dim == Dimension::Nether {
                    ui.label(
                        RichText::new(format!(
                            "{}: {}, {}",
                            t(lang, UiKey::LinkedCoordsOverworld),
                            sample_pos.0 * 8,
                            sample_pos.1 * 8
                        ))
                        .monospace()
                        .size(11.5)
                        .color(palette.accent_xp),
                    );
                }
                if engine.has_in_flight() {
                    ui.label(
                        RichText::new(t(lang, UiKey::LoadingTiles))
                            .size(11.0)
                            .color(palette.accent_gold),
                    );
                }
            });
        });
}

fn pick_cubiomes_scale(bpp: f32) -> i32 {
    if bpp <= 1.6 {
        1
    } else if bpp <= 6.5 {
        4
    } else if bpp <= 24.0 {
        16
    } else if bpp <= 68.0 {
        64
    } else {
        256
    }
}

fn structure_priority(kind: StructureKind) -> u8 {
    match kind {
        StructureKind::SpawnPoint => 0,
        StructureKind::Stronghold => 1,
        StructureKind::EndShip => 2,
        StructureKind::Mansion => 3,
        StructureKind::AncientCity => 4,
        StructureKind::Monument => 5,
        StructureKind::EndGateway => 6,
        StructureKind::Bastion => 7,
        StructureKind::Fortress => 8,
        StructureKind::EndCity => 9,
        StructureKind::TrialChambers => 10,
        StructureKind::Village => 11,
        StructureKind::Outpost => 12,
        StructureKind::DesertPyramid | StructureKind::JungleTemple => 13,
        _ => 15,
    }
}

fn default_export_png_path(seed: i64, dim: Dimension) -> Option<PathBuf> {
    let xdg_pic = std::env::var_os("XDG_PICTURES_DIR")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .filter(|p| p.exists());
    let dir = if let Some(p) = xdg_pic {
        p
    } else {
        let base = std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)?;
        let pictures = base.join("Pictures");
        let imagenes = base.join("Imágenes");
        if pictures.exists() {
            pictures
        } else if imagenes.exists() {
            imagenes
        } else {
            base
        }
    };
    let dname = match dim {
        Dimension::Overworld => "overworld",
        Dimension::Nether => "nether",
        Dimension::TheEnd => "end",
    };
    Some(dir.join(format!("seed_{}_{}.png", seed, dname)))
}

fn handle_dropped_skin_png(
    ctx: &egui::Context,
    state: &mut AppState,
    audio: &AudioEngine,
    toasts: &mut ToastQueue,
    lang: Language,
) {
    let dropped = ctx.input(|i| i.raw.dropped_files.clone());
    if dropped.is_empty() {
        return;
    }
    for f in dropped {
        let is_png = f
            .path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("png"))
            || f.name.to_ascii_lowercase().ends_with(".png");
        if !is_png {
            continue;
        }
        let res = if let Some(ref path) = f.path {
            state.apply_custom_skin_file(ctx, path)
        } else if let Some(ref bytes) = f.bytes {
            state.apply_custom_skin_bytes(ctx, bytes)
        } else {
            continue;
        };
        match res {
            Ok(()) => {
                audio.play(SoundCue::XpOrb);
                toasts.push(
                    t(lang, UiKey::PlayerSkinSavedToastTitle),
                    t(lang, UiKey::PlayerSkinCustomBadge),
                    "compass",
                    Color32::from_rgb(80, 215, 130),
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
        break;
    }
}

fn render_player_skin_sidebar_card(
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
fn render_saved_seeds_history_card(
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

fn render_local_worlds_sidebar_card(
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
