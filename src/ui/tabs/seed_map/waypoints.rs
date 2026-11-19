use eframe::egui::{self, CornerRadius, Margin, RichText, Stroke, Vec2};

use crate::anim::ToastQueue;
use crate::assets::ItemAtlas;
use crate::audio::{AudioEngine, SoundCue};
use crate::data::models::Language;
use crate::data::seed_engine::{Dimension, SeedMapEngine};
use crate::i18n::{t, UiKey};
use crate::state::{AppState, WAYPOINT_ICON_PRESETS};
use crate::ui::theme::{paint_mc_slot, McPalette, SlotStyle};

use super::format_tp_command;
#[allow(clippy::too_many_arguments)]
pub(super) fn render_waypoints_sidebar_card(
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
