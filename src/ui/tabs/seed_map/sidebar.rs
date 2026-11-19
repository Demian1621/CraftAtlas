use eframe::egui::{self, Color32, CornerRadius, Margin, Pos2, Rect, RichText, Stroke, Vec2};

use crate::anim::ToastQueue;
use crate::assets::ItemAtlas;
use crate::audio::{AudioEngine, SoundCue};
use crate::data::models::Language;
use crate::data::seed_engine::{Dimension, SeedMapEngine};
use crate::i18n::{t, UiKey};
use crate::state::{AppState, SeedSidebarTab};
use crate::ui::theme::{filter_chip, McPalette};
use crate::ui::RenderContext;

use super::canvas::pick_cubiomes_scale;
use super::export::default_export_png_path;
use super::finder::render_finder_sidebar_tab;
use super::layers::render_layers_sidebar_tab;
use super::waypoints::render_waypoints_sidebar_card;
use super::worlds::{
    render_local_worlds_sidebar_card, render_player_skin_sidebar_card,
    render_saved_seeds_history_card,
};
use super::{format_tp_xyz, render_tool_tooltip};
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


pub(super) fn render_map_sidebar(
    ui: &mut egui::Ui,
    state: &mut AppState,
    engine: &mut SeedMapEngine,
    rcx: &RenderContext<'_>,
    toasts: &mut ToastQueue,
    lang: Language,
) {
    let RenderContext {
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
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

    // Pinned 2x2 Segmented Sidebar Sub-Tabs (never scroll offscreen)
    ui.columns(2, |cols| {
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
            "compass",
            t(lang, UiKey::SidebarTabFinder),
            state.seed_sidebar_tab == SeedSidebarTab::Finder,
        )
        .clicked()
        {
            state.seed_sidebar_tab = SeedSidebarTab::Finder;
            audio.play(SoundCue::Click);
        }
    });
    ui.add_space(4.0);
    ui.columns(2, |cols| {
        if sidebar_tab_button(
            &mut cols[0],
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
            &mut cols[1],
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
                    ui, state, engine, palette, atlas, audio, toasts,
                );
                ui.add_space(7.0);
                render_compact_player_strip(ui, state, engine, palette, audio, toasts, lang);
            }
            SeedSidebarTab::Finder => {
                render_finder_sidebar_tab(ui, state, engine, rcx, toasts);
            }
            SeedSidebarTab::Layers => {
                render_layers_sidebar_tab(ui, state, rcx, lang);
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

fn render_quick_nav_sidebar_card(
    ui: &mut egui::Ui,
    state: &mut AppState,
    engine: &SeedMapEngine,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    toasts: &mut ToastQueue,
) {
    let lang = state.prefs.language;
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
