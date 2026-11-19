use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, Vec2};

use crate::anim::ToastQueue;
use crate::audio::SoundCue;
use crate::data::litematic::format_with_commas;
use crate::data::seed_engine::{
    NearestSearchState, NearestTarget, SearchableBiome, SeedMapEngine, StructureKind,
};
use crate::data::version_rules::{biome_since_version, structure_since_version};
use crate::i18n::{t, UiKey};
use crate::state::{AppState, NearestFinderMode};
use crate::ui::theme::filter_chip;
use crate::ui::RenderContext;

use super::format_tp_command;

/// Renders the radial Nearest Structure / Biome Finder inside the Seed Map sidebar.
pub(super) fn render_finder_sidebar_tab(
    ui: &mut egui::Ui,
    state: &mut AppState,
    engine: &mut SeedMapEngine,
    rcx: &RenderContext<'_>,
    toasts: &mut ToastQueue,
) {
    let RenderContext {
        versions,
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
    let active_ver = versions.resolve_valid_id(&state.prefs.selected_version);
    let lang = state.prefs.language;
    let dim = state.map_dim;
    let origin = (
        state.map_center_x.round() as i32,
        state.map_center_z.round() as i32,
    );

    egui::Frame::new()
        .fill(palette.bg_card)
        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Mode switcher: Structures vs Biomes
            ui.horizontal(|ui| {
                if filter_chip(
                    ui,
                    palette,
                    atlas,
                    Some("totem_of_undying"),
                    t(lang, UiKey::FinderModeStructures),
                    state.finder_mode == NearestFinderMode::Structure,
                )
                .clicked()
                {
                    state.finder_mode = NearestFinderMode::Structure;
                    engine.clear_nearest_results();
                    audio.play(SoundCue::Click);
                }
                if filter_chip(
                    ui,
                    palette,
                    atlas,
                    Some("cherry_sapling"),
                    t(lang, UiKey::FinderModeBiomes),
                    state.finder_mode == NearestFinderMode::Biome,
                )
                .clicked()
                {
                    state.finder_mode = NearestFinderMode::Biome;
                    engine.clear_nearest_results();
                    audio.play(SoundCue::Click);
                }
            });

            ui.add_space(6.0);

            // Pick structure or biome for active dimension (strictly filtered to active Java version)
            let selected_target = match state.finder_mode {
                NearestFinderMode::Structure => {
                    let list: Vec<StructureKind> = StructureKind::searchable_for_dim(dim)
                        .iter()
                        .copied()
                        .filter(|&kind| {
                            versions.is_active(structure_since_version(kind), None, active_ver)
                        })
                        .collect();
                    if list.is_empty() || state.finder_structure_idx >= list.len() {
                        state.finder_structure_idx = 0;
                    }
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
                        for (idx, &kind) in list.iter().enumerate() {
                            if filter_chip(
                                ui,
                                palette,
                                atlas,
                                Some(kind.icon_id()),
                                kind.label(lang),
                                state.finder_structure_idx == idx,
                            )
                            .clicked()
                            {
                                state.finder_structure_idx = idx;
                                audio.play(SoundCue::Click);
                            }
                        }
                    });
                    list.get(state.finder_structure_idx)
                        .copied()
                        .map(NearestTarget::Structure)
                }
                NearestFinderMode::Biome => {
                    let list: Vec<SearchableBiome> = SearchableBiome::for_dim(dim)
                        .iter()
                        .copied()
                        .filter(|b| {
                            versions.is_active(biome_since_version(b.id), None, active_ver)
                        })
                        .collect();
                    if list.is_empty() || state.finder_biome_idx >= list.len() {
                        state.finder_biome_idx = 0;
                    }
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
                        for (idx, &biome) in list.iter().enumerate() {
                            let lbl = biome.label(lang);
                            if filter_chip(
                                ui,
                                palette,
                                atlas,
                                Some(biome.icon_id),
                                &lbl,
                                state.finder_biome_idx == idx,
                            )
                            .clicked()
                            {
                                state.finder_biome_idx = idx;
                                audio.play(SoundCue::Click);
                            }
                        }
                    });
                    list.get(state.finder_biome_idx)
                        .copied()
                        .map(|b| NearestTarget::Biome {
                            id: b.id,
                            slug: b.slug,
                            icon_id: b.icon_id,
                            accent: b.accent,
                        })
                }
            };

            ui.add_space(6.0);

            // Radius selector
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(t(lang, UiKey::FinderRadiusLabel))
                        .size(11.0)
                        .color(palette.text_muted),
                );
                for (r, lbl) in [(4_000, "4k"), (8_000, "8k"), (16_000, "16k"), (24_000, "24k")] {
                    if filter_chip(ui, palette, atlas, None, lbl, state.finder_radius == r)
                        .clicked()
                    {
                        state.finder_radius = r;
                        audio.play(SoundCue::Click);
                    }
                }
            });

            ui.add_space(6.0);

            let is_searching = matches!(engine.nearest_state(), NearestSearchState::Searching);
            let search_btn = egui::Button::new(
                RichText::new(t(lang, UiKey::FinderSearchBtn))
                    .strong()
                    .size(12.0)
                    .color(Color32::WHITE),
            )
            .fill(palette.accent_xp.gamma_multiply(0.9))
            .corner_radius(CornerRadius::same(5));

            if ui
                .add_enabled(!is_searching && selected_target.is_some(), search_btn)
                .clicked()
            {
                if let Some(target) = selected_target {
                    audio.play(SoundCue::Click);
                    engine.start_nearest_search(dim, target, origin, state.finder_radius, 10, lang);
                }
            }

            ui.add_space(4.0);
            ui.label(
                RichText::new(format!(
                    "{} ({}, {})",
                    dim.label(lang),
                    origin.0,
                    origin.1
                ))
                .size(10.5)
                .monospace()
                .color(palette.text_muted),
            );
            ui.label(
                RichText::new(t(lang, UiKey::FinderNotInVersionHint))
                    .size(10.0)
                    .color(palette.text_muted),
            );
        });

    ui.add_space(6.0);

    // Search progress or sorted results list
    let snapshot = engine.nearest_state().clone();
    match snapshot {
        NearestSearchState::Idle => {}
        NearestSearchState::Searching => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    RichText::new(t(lang, UiKey::FinderSearchingLabel))
                        .size(11.5)
                        .color(palette.accent_primary),
                );
            });
        }
        NearestSearchState::Ready { hits } => {
            if hits.is_empty() {
                ui.label(
                    RichText::new(t(lang, UiKey::FinderNoResultsHint))
                        .size(11.5)
                        .color(palette.text_muted),
                );
                return;
            }

            for (idx, hit) in hits.iter().enumerate() {
                egui::Frame::new()
                    .fill(palette.bg_card)
                    .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                    .corner_radius(CornerRadius::same(5))
                    .inner_margin(Margin::symmetric(7, 5))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.add(atlas.ui_image(hit.icon_id, 18.0));
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("#{} {}", idx + 1, hit.label))
                                            .size(11.5)
                                            .strong()
                                            .color(hit.accent),
                                    );
                                    ui.label(
                                        RichText::new(format!(
                                            "{} b",
                                            format_with_commas(hit.dist_blocks.max(0) as u64)
                                        ))
                                        .size(10.5)
                                        .monospace()
                                        .color(palette.accent_gold),
                                    );
                                });
                                ui.label(
                                    RichText::new(format!("X: {}  Z: {}", hit.x, hit.z))
                                        .size(10.5)
                                        .monospace()
                                        .color(palette.text_primary),
                                );
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("/tp").clicked() {
                                        let cmd = format_tp_command(hit.dim, hit.x, hit.z);
                                        ui.ctx().copy_text(cmd.clone());
                                        audio.play(SoundCue::Click);
                                        toasts.push(
                                            hit.icon_id,
                                            t(lang, UiKey::CopiedToast),
                                            &cmd,
                                            hit.accent,
                                        );
                                    }
                                    if ui.small_button(t(lang, UiKey::FinderJumpBtn)).clicked() {
                                        state.map_dim = hit.dim;
                                        state.jump_map_to(hit.x as f64, hit.z as f64);
                                        if state.map_target_bpp > 6.0 {
                                            state.map_target_bpp = 4.0;
                                        }
                                        state.temp_map_pin = Some((hit.dim, hit.x, hit.z));
                                        audio.play(SoundCue::Click);
                                    }
                                },
                            );
                        });
                    });
                ui.add_space(4.0);
            }
        }
    }
}
