use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, Vec2};
use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::anim::ToastQueue;
use crate::assets::ItemAtlas;
use crate::audio::{AudioEngine, SoundCue};
use crate::data::models::{Database, DupeItem, Language, TechCreator, VersionRegistry};
use crate::i18n::{t, UiKey};
use crate::state::AppState;
use crate::ui::chrome::render_inline_tab_search;
use crate::ui::tabs::creators;
use crate::ui::theme::{adaptive_sidebar_width, filter_chip, paint_mc_slot, McPalette};

pub fn render(
    ui: &mut egui::Ui,
    state: &mut AppState,
    db: &Database,
    versions: &VersionRegistry,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    toasts: &mut ToastQueue,
) {
    let lang = state.prefs.language;
    let active_ver = versions
        .resolve_valid_id(&state.prefs.selected_version)
        .to_string();
    let q_lower = state.search_query.trim().to_lowercase();

    let resolved_all: Vec<Cow<'_, DupeItem>> = db
        .dupes
        .iter()
        .filter(|d| versions.is_active(d.since_version.as_deref(), d.until_version.as_deref(), &active_ver))
        .map(|d| d.resolved(&active_ver))
        .collect();

    let mut cats: BTreeMap<String, String> = BTreeMap::new();
    for d in &resolved_all {
        cats.entry(d.category_key.clone())
            .or_insert_with(|| d.category.get(lang).to_string());
    }

    let filtered: Vec<&DupeItem> = resolved_all
        .iter()
        .map(|d| d.as_ref())
        .filter(|d| state.dupe_category_filter == "all" || d.category_key == state.dupe_category_filter)
        .filter(|d| match state.dupe_status_filter.as_str() {
            "all" => true,
            "patched" => d.status_key == "patched" || d.status_key == "patched_workaround",
            other => d.status_key == other,
        })
        .filter(|d| {
            if q_lower.is_empty() {
                return true;
            }
            d.title.matches(&q_lower)
                || d.how_it_works.matches(&q_lower)
                || d.components.iter().any(|c| c.matches(&q_lower))
        })
        .collect();

    if !filtered.is_empty() && !filtered.iter().any(|d| d.id == state.selected_dupe_id) {
        state.selected_dupe_id = filtered[0].id.clone();
    }

    // Filter Strip + Directory Toggle for YT & Bilibili Creators + Nether 1:8 Synchronizer
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(5.0, 5.0);
        let mut in_creators = state.dupes_creators_view;
        let showing_creators = creators::render_creators_filter_controls(
            ui,
            state,
            &mut in_creators,
            UiKey::BackToDupesBtn,
            "dupes",
            palette,
            atlas,
            audio,
            lang,
        );
        state.dupes_creators_view = in_creators;

        if !showing_creators {
            ui.separator();
            if filter_chip(
                ui,
                palette,
                atlas,
                Some("tnt"),
                t(lang, UiKey::FilterAll),
                state.dupe_category_filter == "all",
            )
            .clicked()
            {
                state.dupe_category_filter = "all".to_string();
                audio.play(SoundCue::Click);
            }
            for (k, lbl) in &cats {
                if filter_chip(ui, palette, atlas, None, lbl, state.dupe_category_filter == *k)
                    .clicked()
                {
                    state.dupe_category_filter = k.clone();
                    audio.play(SoundCue::Click);
                }
            }
            ui.separator();
            for (skey, slbl) in [
                ("all", t(lang, UiKey::FilterAll)),
                ("working", t(lang, UiKey::StatusWorking)),
                ("patched", t(lang, UiKey::StatusFixed)),
            ] {
                if filter_chip(ui, palette, atlas, None, slbl, state.dupe_status_filter == skey)
                    .clicked()
                {
                    state.dupe_status_filter = skey.to_string();
                    audio.play(SoundCue::Click);
                }
            }
        }
    });

    ui.add_space(6.0);
    render_inline_tab_search(ui, state, palette, atlas, audio);

    if state.dupes_creators_view {
        creators::render_creators_directory(
            ui,
            state,
            &db.creators,
            palette,
            atlas,
            audio,
            toasts,
            lang,
            &q_lower,
        );
        return;
    }

    // Quick 1:8 Nether Portal Synchronizer Calculator Strip
    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.add(atlas.ui_image("obsidian", 18.0));
                ui.label(
                    RichText::new(t(lang, UiKey::PortalSyncCalcTitle))
                        .strong()
                        .size(12.0)
                        .color(palette.accent_enchant),
                );
                ui.label(
                    RichText::new(format!("{}:", t(lang, UiKey::OverworldCoordLabel)))
                        .size(11.5)
                        .color(palette.text_muted),
                );
                ui.add_sized([72.0, 22.0], egui::TextEdit::singleline(&mut state.portal_calc_ow_x));
                ui.add_sized([72.0, 22.0], egui::TextEdit::singleline(&mut state.portal_calc_ow_z));
                let ox = state.portal_calc_ow_x.trim().parse::<i32>().unwrap_or(0);
                let oz = state.portal_calc_ow_z.trim().parse::<i32>().unwrap_or(0);
                ui.label(
                    RichText::new(format!(
                        "-> {}: ({}, {})",
                        t(lang, UiKey::NetherCoordLabel),
                        ox.div_euclid(8),
                        oz.div_euclid(8)
                    ))
                    .monospace()
                    .strong()
                    .size(12.0)
                    .color(palette.accent_gold),
                );
            });
        });

    ui.add_space(8.0);

    let total_w = ui.available_width();
    if total_w >= 820.0 {
        let side_w = adaptive_sidebar_width(total_w);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(side_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    render_dupe_sidebar(ui, state, &filtered, palette, atlas, audio, lang);
                },
            );
            ui.add_space(10.0);
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let selected_id = state.selected_dupe_id.clone();
                    if let Some(dupe) = resolved_all.iter().find(|d| d.id == selected_id) {
                        render_dupe_detail(
                            ui,
                            state,
                            dupe,
                            &db.creators,
                            &active_ver,
                            palette,
                            atlas,
                            audio,
                            toasts,
                            lang,
                        );
                    }
                },
            );
        });
    } else {
        let cur_title = filtered
            .iter()
            .find(|d| d.id == state.selected_dupe_id)
            .map(|d| d.title.get(lang))
            .unwrap_or("...");
        egui::ComboBox::from_id_salt("dupe_compact_combo")
            .selected_text(cur_title)
            .width(total_w.min(400.0))
            .show_ui(ui, |ui| {
                for d in &filtered {
                    if ui
                        .selectable_value(&mut state.selected_dupe_id, d.id.clone(), d.title.get(lang))
                        .changed()
                    {
                        audio.play(SoundCue::Click);
                    }
                }
            });
        ui.add_space(8.0);
        let selected_id = state.selected_dupe_id.clone();
        if let Some(dupe) = resolved_all.iter().find(|d| d.id == selected_id) {
            render_dupe_detail(
                ui,
                state,
                dupe,
                &db.creators,
                &active_ver,
                palette,
                atlas,
                audio,
                toasts,
                lang,
            );
        }
    }
}

fn render_dupe_sidebar(
    ui: &mut egui::Ui,
    state: &mut AppState,
    filtered: &[&DupeItem],
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    lang: Language,
) {
    egui::ScrollArea::vertical()
        .id_salt("dupe_list_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for dupe in filtered {
                let selected = dupe.id == state.selected_dupe_id;
                let working = dupe.status_key == "working";
                let status_col = if working {
                    palette.accent_xp
                } else {
                    palette.accent_danger
                };
                let bg = if selected {
                    Color32::from_rgba_unmultiplied(
                        palette.accent_gold.r(),
                        palette.accent_gold.g(),
                        palette.accent_gold.b(),
                        28,
                    )
                } else {
                    palette.bg_card
                };
                let stroke = if selected {
                    Stroke::new(1.5_f32, palette.accent_gold)
                } else {
                    Stroke::new(1.0_f32, palette.border_subtle)
                };

                let resp = egui::Frame::new()
                    .fill(bg)
                    .stroke(stroke)
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::symmetric(8, 6))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            let (sr, _) =
                                ui.allocate_exact_size(Vec2::splat(30.0), egui::Sense::hover());
                            paint_mc_slot(
                                ui.painter(),
                                sr,
                                palette,
                                atlas,
                                Some(&dupe.icon),
                                1,
                                selected,
                                false,
                                None,
                            );
                            ui.vertical(|ui| {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(dupe.title.get(lang))
                                            .size(12.0)
                                            .strong()
                                            .color(palette.text_primary),
                                    )
                                    .wrap(),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "{} • {}",
                                        dupe.status.get(lang),
                                        dupe.difficulty.get(lang)
                                    ))
                                    .size(10.5)
                                    .color(status_col),
                                );
                            });
                        });
                    })
                    .response
                    .interact(egui::Sense::click());

                if resp.clicked() {
                    state.selected_dupe_id = dupe.id.clone();
                    audio.play(SoundCue::Click);
                }
                ui.add_space(4.0);
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn render_dupe_detail(
    ui: &mut egui::Ui,
    state: &mut AppState,
    dupe: &DupeItem,
    creators: &[TechCreator],
    active_ver: &str,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    toasts: &mut ToastQueue,
    lang: Language,
) {
    let working = dupe.status_key == "working";
    let status_col = if working {
        palette.accent_xp
    } else {
        palette.accent_danger
    };

    egui::ScrollArea::vertical()
        .id_salt("dupe_detail_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Frame::new()
                .fill(palette.bg_panel)
                .stroke(Stroke::new(1.5_f32, status_col))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_wrapped(|ui| {
                        ui.add(atlas.ui_image(&dupe.icon, 36.0));
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(dupe.title.get(lang))
                                    .size(18.0)
                                    .strong()
                                    .color(palette.text_primary),
                            );
                            ui.horizontal_wrapped(|ui| {
                                ui.label(
                                    RichText::new(format!("[{}]", dupe.status.get(lang)))
                                        .size(11.5)
                                        .strong()
                                        .color(status_col),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "{} • {}",
                                        dupe.category.get(lang),
                                        dupe.difficulty.get(lang)
                                    ))
                                    .size(11.5)
                                    .color(palette.accent_gold),
                                );
                            });
                        });
                    });

                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(t(lang, UiKey::ComponentsHeader))
                            .size(12.5)
                            .strong()
                            .color(palette.accent_gold),
                    );
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        for comp in &dupe.components {
                            egui::Frame::new()
                                .fill(palette.bg_card)
                                .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                                .corner_radius(CornerRadius::same(4))
                                .inner_margin(Margin::symmetric(8, 4))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(comp.get(lang))
                                            .size(11.5)
                                            .color(palette.text_primary),
                                    );
                                });
                        }
                    });

                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(t(lang, UiKey::HowItWorksHeader))
                            .size(12.5)
                            .strong()
                            .color(palette.accent_primary),
                    );
                    ui.add(
                        egui::Label::new(
                            RichText::new(dupe.how_it_works.get(lang))
                                .size(12.5)
                                .color(palette.text_primary),
                        )
                        .wrap(),
                    );

                    if !dupe.use_cases.is_empty() {
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new(t(lang, UiKey::UseCasesHeader))
                                .size(12.5)
                                .strong()
                                .color(palette.accent_xp),
                        );
                        for u in &dupe.use_cases {
                            ui.label(
                                RichText::new(format!("• {}", u.get(lang)))
                                    .size(12.0)
                                    .color(palette.text_primary),
                            );
                        }
                    }

                    ui.add_space(10.0);
                    egui::Frame::new()
                        .fill(palette.slot_bg)
                        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(Margin::same(10))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(
                                RichText::new(dupe.diagram.get(lang))
                                    .monospace()
                                    .size(11.5)
                                    .color(palette.accent_primary),
                            );
                        });

                    ui.add_space(10.0);
                    let (query, search_url) = dupe.tutorial_search(lang, active_ver);
                    creators::render_tutorial_search_bar(
                        ui,
                        &query,
                        &search_url,
                        active_ver,
                        None,
                        None,
                        &dupe.icon,
                        palette,
                        atlas,
                        audio,
                        toasts,
                        lang,
                    );

                    if !creators.is_empty() {
                        ui.add_space(12.0);
                        let mut open_creators = state.dupes_creators_view;
                        creators::render_recommended_creators_footer(
                            ui,
                            state,
                            &mut open_creators,
                            creators,
                            "dupes",
                            &dupe.id,
                            &dupe.category_key,
                            None,
                            palette,
                            atlas,
                            audio,
                            toasts,
                            lang,
                        );
                        if open_creators {
                            state.dupes_creators_view = true;
                        }
                    }
                });
        });
}
