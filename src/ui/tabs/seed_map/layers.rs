use eframe::egui::{self, Color32, CornerRadius, Margin, Pos2, Rect, RichText, Stroke, Vec2};

use crate::assets::ItemAtlas;
use crate::audio::SoundCue;
use crate::data::models::Language;
use crate::data::seed_engine::{Dimension, LayerToggles, StructureKind};
use crate::data::version_rules::structure_since_version;
use crate::i18n::{t, UiKey};
use crate::state::AppState;
use crate::ui::theme::McPalette;
use crate::ui::RenderContext;

#[derive(Clone, Copy)]
enum OwStructureLayer {
    Village,
    TrialChambers,
    AncientCity,
    Stronghold,
    Monument,
    Mansion,
    Outpost,
    Temples,
    RuinedPortal,
}

impl OwStructureLayer {
    fn kind(self) -> StructureKind {
        match self {
            OwStructureLayer::Village => StructureKind::Village,
            OwStructureLayer::TrialChambers => StructureKind::TrialChambers,
            OwStructureLayer::AncientCity => StructureKind::AncientCity,
            OwStructureLayer::Stronghold => StructureKind::Stronghold,
            OwStructureLayer::Monument => StructureKind::Monument,
            OwStructureLayer::Mansion => StructureKind::Mansion,
            OwStructureLayer::Outpost => StructureKind::Outpost,
            OwStructureLayer::Temples => StructureKind::DesertPyramid,
            OwStructureLayer::RuinedPortal => StructureKind::RuinedPortal,
        }
    }

    fn toggle_mut(self, layers: &mut LayerToggles) -> &mut bool {
        match self {
            OwStructureLayer::Village => &mut layers.villages,
            OwStructureLayer::TrialChambers => &mut layers.trial_chambers,
            OwStructureLayer::AncientCity => &mut layers.ancient_cities,
            OwStructureLayer::Stronghold => &mut layers.strongholds,
            OwStructureLayer::Monument => &mut layers.monuments,
            OwStructureLayer::Mansion => &mut layers.mansions,
            OwStructureLayer::Outpost => &mut layers.outposts,
            OwStructureLayer::Temples => &mut layers.temples,
            OwStructureLayer::RuinedPortal => &mut layers.ruined_portals,
        }
    }

    fn short_label(self, lang: Language) -> &'static str {
        match (self, lang) {
            (OwStructureLayer::Village, _) => StructureKind::Village.label(lang),
            (OwStructureLayer::TrialChambers, Language::EnUs) => "Trials",
            (OwStructureLayer::TrialChambers, _) => "Desafío",
            (OwStructureLayer::AncientCity, Language::EnUs) => "Ancient City",
            (OwStructureLayer::AncientCity, _) => "C. Antigua",
            (OwStructureLayer::Stronghold, _) => "Stronghold",
            (OwStructureLayer::Monument, Language::EnUs) => "Monument",
            (OwStructureLayer::Monument, _) => "Monumento",
            (OwStructureLayer::Mansion, Language::EnUs) => "Mansion",
            (OwStructureLayer::Mansion, _) => "Mansión",
            (OwStructureLayer::Outpost, _) => "Outpost",
            (OwStructureLayer::Temples, Language::EnUs) => "Temples",
            (OwStructureLayer::Temples, _) => "Templos",
            (OwStructureLayer::RuinedPortal, Language::EnUs) => "Ruined Portal",
            (OwStructureLayer::RuinedPortal, _) => "Portal Ruina",
        }
    }

    fn tooltip(self, lang: Language) -> &'static str {
        match self {
            OwStructureLayer::Temples => {
                if matches!(lang, Language::EnUs) {
                    "Temples & Ruins"
                } else {
                    "Templos / Ruinas"
                }
            }
            other => other.kind().label(lang),
        }
    }
}

pub(super) fn render_layers_sidebar_tab(
    ui: &mut egui::Ui,
    state: &mut AppState,
    rcx: &RenderContext<'_>,
    lang: Language,
) {
    let RenderContext {
        versions,
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
    let active_ver = versions.resolve_valid_id(&state.prefs.selected_version);
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
                    let all_ow = [
                        OwStructureLayer::Village,
                        OwStructureLayer::TrialChambers,
                        OwStructureLayer::AncientCity,
                        OwStructureLayer::Stronghold,
                        OwStructureLayer::Monument,
                        OwStructureLayer::Mansion,
                        OwStructureLayer::Outpost,
                        OwStructureLayer::Temples,
                        OwStructureLayer::RuinedPortal,
                    ];
                    let active_ow: Vec<OwStructureLayer> = all_ow
                        .into_iter()
                        .filter(|layer| {
                            versions.is_active(
                                structure_since_version(layer.kind()),
                                None,
                                active_ver,
                            )
                        })
                        .collect();

                    for (pair_idx, pair) in active_ow.chunks(2).enumerate() {
                        if pair_idx > 0 {
                            ui.add_space(4.0);
                        }
                        ui.columns(2, |cols| {
                            for (col_idx, &layer) in pair.iter().enumerate() {
                                layer_tile_toggle(
                                    &mut cols[col_idx],
                                    atlas,
                                    layer.kind().icon_id(),
                                    layer.short_label(lang),
                                    layer.tooltip(lang),
                                    layer.toggle_mut(&mut state.map_layers),
                                    palette,
                                );
                            }
                        });
                    }
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

pub(super) fn layer_tile_toggle(
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
