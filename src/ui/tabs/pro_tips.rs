use eframe::egui::{self, Color32, CornerRadius, Margin, Pos2, Rect, RichText, Stroke, Vec2};
use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::assets::ItemAtlas;
use crate::audio::{AudioEngine, SoundCue};
use crate::data::models::{Database, Language, ProTip, VersionRegistry};
use crate::i18n::{t, UiKey};
use crate::state::AppState;
use crate::ui::chrome::render_inline_tab_search;
use crate::ui::theme::{filter_chip, McPalette};

pub fn render(
    ui: &mut egui::Ui,
    state: &mut AppState,
    db: &Database,
    versions: &VersionRegistry,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
) {
    let lang = state.prefs.language;
    let active_ver = versions.resolve_valid_id(&state.prefs.selected_version);
    let q_lower = state.search_query.trim().to_lowercase();

    let resolved_all: Vec<Cow<'_, ProTip>> = db
        .pro_tips
        .iter()
        .filter(|tp| versions.is_active(tp.since_version.as_deref(), tp.until_version.as_deref(), active_ver))
        .map(|tp| tp.resolved(active_ver))
        .collect();

    let mut cats: BTreeMap<String, String> = BTreeMap::new();
    for tip in &resolved_all {
        cats.entry(tip.category_key.clone())
            .or_insert_with(|| tip.category.get(lang).to_string());
    }

    let filtered: Vec<&ProTip> = resolved_all
        .iter()
        .map(|tp| tp.as_ref())
        .filter(|tp| state.tip_category_filter == "all" || tp.category_key == state.tip_category_filter)
        .filter(|tp| {
            if q_lower.is_empty() {
                return true;
            }
            tp.title.matches(&q_lower) || tp.text.matches(&q_lower)
        })
        .collect();

    if !filtered.is_empty() && !filtered.iter().any(|tp| tp.id == state.selected_tip_id) {
        state.selected_tip_id = filtered[0].id.clone();
    }

    // Category Filter Bar
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(5.0, 5.0);
        if filter_chip(
            ui,
            palette,
            atlas,
            Some("knowledge_book"),
            t(lang, UiKey::FilterAll),
            state.tip_category_filter == "all",
        )
        .clicked()
        {
            state.tip_category_filter = "all".to_string();
            audio.play(SoundCue::Click);
        }
        for (ck, cl) in &cats {
            if filter_chip(ui, palette, atlas, None, cl, state.tip_category_filter == *ck).clicked()
            {
                state.tip_category_filter = ck.clone();
                audio.play(SoundCue::Click);
            }
        }
    });

    ui.add_space(6.0);
    render_inline_tab_search(ui, state, palette, atlas, audio);

    egui::ScrollArea::vertical()
        .id_salt("protips_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Interactive Technical Visual Charts (Spawning Sphere + 26.3 Ore Y-Levels)
            let avail_w = ui.available_width();
            if avail_w >= 760.0 {
                let half_w = (avail_w - 10.0) * 0.5;
                ui.horizontal_top(|ui| {
                    ui.allocate_ui_with_layout(
                        Vec2::new(half_w, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| render_spawning_sphere_card(ui, palette, atlas, lang),
                    );
                    ui.add_space(10.0);
                    ui.allocate_ui_with_layout(
                        Vec2::new(half_w, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| render_ore_y_chart_card(ui, palette, atlas, lang),
                    );
                });
            } else {
                render_spawning_sphere_card(ui, palette, atlas, lang);
                ui.add_space(8.0);
                render_ore_y_chart_card(ui, palette, atlas, lang);
            }

            ui.add_space(12.0);

            // Responsive Tip Cards Grid (1 col narrow, 2 cols wide)
            let cols = if avail_w >= 860.0 { 2 } else { 1 };
            let card_w = (avail_w - ((cols - 1) as f32) * 10.0) / (cols as f32);

            for chunk in filtered.chunks(cols) {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    for tip in chunk {
                        ui.allocate_ui_with_layout(
                            Vec2::new(card_w, 0.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                egui::Frame::new()
                                    .fill(palette.bg_panel)
                                    .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                                    .corner_radius(CornerRadius::same(6))
                                    .inner_margin(Margin::same(12))
                                    .show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            ui.add(atlas.ui_image(&tip.icon, 26.0));
                                            ui.vertical(|ui| {
                                                ui.add(
                                                    egui::Label::new(
                                                        RichText::new(tip.title.get(lang))
                                                            .size(14.0)
                                                            .strong()
                                                            .color(palette.accent_primary),
                                                    )
                                                    .wrap(),
                                                );
                                                ui.label(
                                                    RichText::new(tip.category.get(lang))
                                                        .size(10.5)
                                                        .color(palette.accent_gold),
                                                );
                                            });
                                        });
                                        ui.add_space(6.0);
                                        ui.add(
                                            egui::Label::new(
                                                RichText::new(tip.text.get(lang))
                                                    .size(12.0)
                                                    .color(palette.text_primary),
                                            )
                                            .wrap(),
                                        );
                                    });
                            },
                        );
                    }
                });
                ui.add_space(10.0);
            }
        });
}

fn render_spawning_sphere_card(
    ui: &mut egui::Ui,
    palette: McPalette,
    atlas: &ItemAtlas,
    lang: Language,
) {
    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.0_f32, palette.border_strong))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add(atlas.ui_image("spawner", 20.0));
                ui.label(
                    RichText::new(t(lang, UiKey::TipSpawningSphereTitle))
                        .size(13.0)
                        .strong()
                        .color(palette.accent_gold),
                );
            });
            ui.add_space(6.0);

            let avail_w = ui.available_width();
            let (rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, 128.0), egui::Sense::hover());
            let p = ui.painter();
            let c = rect.center();

            // 128-block Despawn outer ring, 32-block wander ring, 24-block exclusion inner sphere
            p.circle_stroke(c, 58.0, Stroke::new(2.0_f32, palette.accent_danger));
            p.circle_filled(
                c,
                58.0,
                Color32::from_rgba_unmultiplied(122, 245, 76, 18),
            );
            p.circle_stroke(c, 34.0, Stroke::new(1.5_f32, palette.accent_xp));
            p.circle_filled(
                c,
                16.0,
                Color32::from_rgba_unmultiplied(238, 86, 86, 48),
            );
            p.circle_stroke(c, 16.0, Stroke::new(1.5_f32, palette.accent_gold));
            atlas.paint(p, Rect::from_center_size(c, Vec2::splat(16.0)), "player_head", Color32::WHITE);

            p.text(
                Pos2::new(c.x + 20.0, c.y - 8.0),
                egui::Align2::LEFT_CENTER,
                "0-24 No Spawn",
                egui::FontId::monospace(10.5),
                palette.accent_gold,
            );
            p.text(
                Pos2::new(c.x + 38.0, c.y + 14.0),
                egui::Align2::LEFT_CENTER,
                "24-32 Core",
                egui::FontId::monospace(10.5),
                palette.accent_xp,
            );
            p.text(
                Pos2::new(c.x + 62.0, c.y - 34.0),
                egui::Align2::LEFT_CENTER,
                ">128 Despawn",
                egui::FontId::monospace(10.5),
                palette.accent_danger,
            );

            ui.add_space(6.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t(lang, UiKey::TipSpawningDesc))
                        .size(11.5)
                        .color(palette.text_muted),
                )
                .wrap(),
            );
        });
}

fn render_ore_y_chart_card(
    ui: &mut egui::Ui,
    palette: McPalette,
    atlas: &ItemAtlas,
    lang: Language,
) {
    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.0_f32, palette.border_strong))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add(atlas.ui_image("diamond_ore", 20.0));
                ui.label(
                    RichText::new(t(lang, UiKey::TipMiningYLevelsTitle))
                        .size(13.0)
                        .strong()
                        .color(palette.accent_primary),
                );
            });
            ui.add_space(6.0);

            let ores = [
                ("netherite_ingot", "Ancient Debris / Netherite", "Y = 15 (Nether)", Color32::from_rgb(176, 128, 118)),
                ("diamond", "Diamond", "Y = -58", Color32::from_rgb(56, 224, 198)),
                ("iron_ingot", "Iron (Peaks / Caves)", "Y = 232 / Y = 16", Color32::from_rgb(218, 220, 226)),
                ("gold_ingot", "Gold (Badlands / Caves)", "Y = 32+ / Y = -16", Color32::from_rgb(248, 192, 68)),
                ("redstone", "Redstone", "Y = -59", Color32::from_rgb(238, 72, 72)),
                ("copper_ingot", "Copper", "Y = 48", Color32::from_rgb(228, 128, 86)),
                ("lapis_lazuli", "Lapis Lazuli", "Y = -1", Color32::from_rgb(92, 146, 255)),
            ];

            for (icon, name, y_lbl, col) in ores {
                ui.horizontal(|ui| {
                    ui.add(atlas.ui_image(icon, 16.0));
                    ui.label(RichText::new(name).size(11.5).color(palette.text_primary));
                    ui.label(
                        RichText::new(y_lbl)
                            .monospace()
                            .strong()
                            .size(11.5)
                            .color(col),
                    );
                });
            }

            ui.add_space(4.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t(lang, UiKey::TipMiningDesc))
                        .size(11.0)
                        .color(palette.text_muted),
                )
                .wrap(),
            );
        });
}
