use eframe::egui::{self, Color32, CornerRadius, Margin, Pos2, Rect, RichText, Stroke, Vec2};
use simple_easing::cubic_out;

use std::borrow::Cow;

use crate::anim::{paint_enchant_glint, ParticleSystem, ToastQueue};
use crate::assets::ItemAtlas;
use crate::audio::{AudioEngine, SoundCue};
use crate::data::anvil_calc::compute_optimal_anvil_plan;
use crate::data::models::{GearItem, Language};
use crate::i18n::{t, UiKey};
use crate::state::{AppState, GearSubView, RecipeViewMode};
use crate::ui::chrome::render_inline_tab_search;
use crate::ui::theme::{adaptive_sidebar_width, filter_chip, paint_mc_frame, paint_mc_slot, McPalette, SlotStyle};
use crate::ui::RenderContext;

pub fn render(
    ui: &mut egui::Ui,
    state: &mut AppState,
    rcx: &RenderContext<'_>,
    particles: &mut ParticleSystem,
    toasts: &mut ToastQueue,
    dt: f32,
) {
    let RenderContext {
        db,
        versions,
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
    state.anvil_anim.tick(dt);
    let lang = state.prefs.language;
    let active_ver = versions
        .resolve_valid_id(&state.prefs.selected_version)
        .to_string();
    crate::ui::tabs::enchant_books::render_view_switcher(ui, state, rcx, &active_ver, lang);
    match state.gear_view {
        GearSubView::Books => {
            crate::ui::tabs::enchant_books::render(ui, state, rcx, &active_ver, lang);
            return;
        }
        GearSubView::IllegalGod => {
            crate::ui::tabs::god_armor::render(ui, state, rcx, toasts, &active_ver);
            return;
        }
        GearSubView::Standard => {}
    }
    let active_ver = active_ver.as_str();
    let q_lower = state.search_query.trim().to_lowercase();

    let resolved_all: Vec<Cow<'_, GearItem>> = db
        .gear
        .iter()
        .filter(|g| versions.is_active(g.since_version.as_deref(), g.until_version.as_deref(), active_ver))
        .map(|g| g.resolved_for(versions, active_ver))
        .collect();

    let filtered: Vec<&GearItem> = resolved_all
        .iter()
        .map(|g| g.as_ref())
        .filter(|g| g.has_category_filter(&state.gear_filter))
        .filter(|g| {
            if q_lower.is_empty() {
                return true;
            }
            g.name.matches(&q_lower)
                || g.what_it_does.matches(&q_lower)
                || g.what_it_is_for.matches(&q_lower)
                || g.enchantments
                    .iter()
                    .any(|e| e.name.matches(&q_lower) || e.desc.matches(&q_lower))
        })
        .collect();

    // Ensure a valid selection exists if current filter excludes it
    if !filtered.is_empty() && !filtered.iter().any(|g| g.id == state.selected_gear_id) {
        state.selected_gear_id = filtered[0].id.clone();
    }

    let total_w = ui.available_width();
    if total_w >= 780.0 {
        let side_w = adaptive_sidebar_width(total_w);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(side_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    render_gear_sidebar(ui, state, &filtered, palette, atlas, audio, lang);
                },
            );
            ui.add_space(10.0);
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    if let Some(item) = resolved_all.iter().find(|g| g.id == state.selected_gear_id) {
                        render_gear_detail(ui, state, item, rcx, particles, toasts);
                    }
                },
            );
        });
    } else {
        // Compact narrow stack: inline filter + top selector bar + full-width scrollable detail
        render_inline_tab_search(ui, state, palette, atlas, audio);
        render_gear_compact_picker(ui, state, &filtered, palette, atlas, audio, lang);
        ui.add_space(8.0);
        if let Some(item) = resolved_all.iter().find(|g| g.id == state.selected_gear_id) {
            render_gear_detail(ui, state, item, rcx, particles, toasts);
        }
    }
}

fn render_gear_filters(
    ui: &mut egui::Ui,
    state: &mut AppState,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    lang: Language,
) {
    let filters = [
        ("all", None, t(lang, UiKey::FilterAll)),
        ("armor", Some("netherite_chestplate"), t(lang, UiKey::FilterArmor)),
        ("weapon", Some("netherite_sword"), t(lang, UiKey::FilterWeapon)),
        ("tool", Some("netherite_pickaxe"), t(lang, UiKey::FilterTool)),
    ];
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(5.0, 5.0);
        for (key, icon, lbl) in filters {
            if filter_chip(ui, palette, atlas, icon, lbl, state.gear_filter == key).clicked() {
                state.gear_filter = key.to_string();
                audio.play(SoundCue::Click);
            }
        }
    });
}

fn render_gear_sidebar(
    ui: &mut egui::Ui,
    state: &mut AppState,
    filtered: &[&GearItem],
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    lang: Language,
) {
    render_inline_tab_search(ui, state, palette, atlas, audio);
    render_gear_filters(ui, state, palette, atlas, audio, lang);
    ui.add_space(8.0);

    let time = ui.ctx().input(|i| i.time);
    egui::ScrollArea::vertical()
        .id_salt("gear_list_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for item in filtered {
                let selected = item.id == state.selected_gear_id;
                let bg = if selected {
                    Color32::from_rgba_unmultiplied(
                        palette.accent_primary.r(),
                        palette.accent_primary.g(),
                        palette.accent_primary.b(),
                        32,
                    )
                } else {
                    palette.bg_card
                };
                let stroke = if selected {
                    Stroke::new(1.5_f32, palette.accent_primary)
                } else {
                    Stroke::new(1.0_f32, palette.border_subtle)
                };

                let frame_resp = egui::Frame::new()
                    .fill(bg)
                    .stroke(stroke)
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::symmetric(8, 6))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            let (slot_r, _) =
                                ui.allocate_exact_size(Vec2::splat(34.0), egui::Sense::hover());
                            paint_mc_slot(
                                ui.painter(),
                                slot_r,
                                palette,
                                atlas,
                                Some(&item.icon),
                                SlotStyle {
                                    selected,
                                    glint_time: if selected { Some(time) } else { None },
                                    ..Default::default()
                                },
                            );
                            ui.vertical(|ui| {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(item.name.get(lang))
                                            .size(12.5)
                                            .strong()
                                            .color(if selected {
                                                Color32::WHITE
                                            } else {
                                                palette.text_primary
                                            }),
                                    )
                                    .wrap(),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "{} enchants",
                                        item.enchantments.len()
                                    ))
                                    .size(10.5)
                                    .color(palette.text_muted),
                                );
                            });
                        });
                    });

                let resp = frame_resp.response.interact(egui::Sense::click());
                if resp.clicked() {
                    state.selected_gear_id = item.id.clone();
                    state.anvil_anim.sync_gear(&item.id);
                    audio.play(SoundCue::Click);
                }
                ui.add_space(4.0);
            }
        });
}

fn render_gear_compact_picker(
    ui: &mut egui::Ui,
    state: &mut AppState,
    filtered: &[&GearItem],
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    lang: Language,
) {
    render_gear_filters(ui, state, palette, atlas, audio, lang);
    ui.add_space(4.0);
    let current_name = filtered
        .iter()
        .find(|g| g.id == state.selected_gear_id)
        .map(|g| g.name.get(lang))
        .unwrap_or("...");
    egui::ComboBox::from_id_salt("gear_compact_combo")
        .selected_text(current_name)
        .width(ui.available_width().min(360.0))
        .show_ui(ui, |ui| {
            for g in filtered {
                if ui
                    .selectable_value(&mut state.selected_gear_id, g.id.clone(), g.name.get(lang))
                    .changed()
                {
                    state.anvil_anim.sync_gear(&g.id);
                    audio.play(SoundCue::Click);
                }
            }
        });
}

fn render_gear_detail(
    ui: &mut egui::Ui,
    state: &mut AppState,
    item: &GearItem,
    rcx: &RenderContext<'_>,
    particles: &mut ParticleSystem,
    toasts: &mut ToastQueue,
) {
    let RenderContext {
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
    let lang = state.prefs.language;
    state.anvil_anim.sync_gear(&item.id);
    let time = ui.ctx().input(|i| i.time);
    let plan = compute_optimal_anvil_plan(item, lang);
    let total_steps = plan.steps.len();

    egui::ScrollArea::vertical()
        .id_salt("gear_detail_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Header Banner Card
            egui::Frame::new()
                .fill(palette.bg_panel)
                .stroke(Stroke::new(1.5_f32, palette.accent_enchant))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(14))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_wrapped(|ui| {
                        let (hero_r, _) =
                            ui.allocate_exact_size(Vec2::splat(60.0), egui::Sense::hover());
                        paint_mc_slot(
                            ui.painter(),
                            hero_r,
                            palette,
                            atlas,
                            Some(&item.icon),
                            SlotStyle {
                                selected: true,
                                glint_time: Some(time),
                                ..Default::default()
                            },
                        );

                        ui.add_space(6.0);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(item.name.get(lang))
                                    .size(20.0)
                                    .strong()
                                    .color(palette.accent_primary),
                            );
                            ui.horizontal_wrapped(|ui| {
                                let cat_lbl = match item.category.as_str() {
                                    "armor" => t(lang, UiKey::FilterArmor),
                                    "weapon" => t(lang, UiKey::FilterWeapon),
                                    _ => t(lang, UiKey::FilterTool),
                                };
                                ui.label(
                                    RichText::new(format!("[{}]", cat_lbl.to_uppercase()))
                                        .size(11.5)
                                        .strong()
                                        .color(palette.accent_gold),
                                );

                                let is_fav = state.prefs.favorite_gear.contains(&item.id);
                                if filter_chip(
                                    ui,
                                    palette,
                                    atlas,
                                    Some("nether_star"),
                                    if is_fav {
                                        t(lang, UiKey::UnpinFavorite)
                                    } else {
                                        t(lang, UiKey::PinFavorite)
                                    },
                                    is_fav,
                                )
                                .clicked()
                                {
                                    let pinned = state.prefs.toggle_favorite_gear(&item.id);
                                    audio.play(SoundCue::Click);
                                    if pinned {
                                        toasts.push(
                                            t(lang, UiKey::ToastFavoriteAddedTitle),
                                            item.name.get(lang),
                                            &item.icon,
                                            palette.accent_gold,
                                        );
                                    }
                                }

                                if filter_chip(
                                    ui,
                                    palette,
                                    atlas,
                                    Some("crafting_table"),
                                    t(lang, UiKey::RecipesForTitle),
                                    false,
                                )
                                .clicked()
                                {
                                    state.open_recipe_item(&item.icon, RecipeViewMode::RecipesFor);
                                    audio.play(SoundCue::Click);
                                }
                            });
                        });
                    });
                });

            ui.add_space(10.0);

            // Responsive What-It-Does / What-It-Is-For section
            let pane_w = ui.available_width();
            if pane_w >= 620.0 {
                let col_w = (pane_w - 10.0) * 0.5;
                ui.horizontal_top(|ui| {
                    render_info_box(
                        ui,
                        col_w,
                        palette,
                        atlas,
                        InfoCard {
                            icon_id: "knowledge_book",
                            title: t(lang, UiKey::WhatItDoes),
                            body: item.what_it_does.get(lang),
                            accent: palette.accent_primary,
                        },
                    );
                    ui.add_space(10.0);
                    render_info_box(
                        ui,
                        col_w,
                        palette,
                        atlas,
                        InfoCard {
                            icon_id: "compass",
                            title: t(lang, UiKey::WhatItIsFor),
                            body: item.what_it_is_for.get(lang),
                            accent: palette.accent_gold,
                        },
                    );
                });
            } else {
                render_info_box(
                    ui,
                    pane_w,
                    palette,
                    atlas,
                    InfoCard {
                        icon_id: "knowledge_book",
                        title: t(lang, UiKey::WhatItDoes),
                        body: item.what_it_does.get(lang),
                        accent: palette.accent_primary,
                    },
                );
                ui.add_space(8.0);
                render_info_box(
                    ui,
                    pane_w,
                    palette,
                    atlas,
                    InfoCard {
                        icon_id: "compass",
                        title: t(lang, UiKey::WhatItIsFor),
                        body: item.what_it_is_for.get(lang),
                        accent: palette.accent_gold,
                    },
                );
            }

            ui.add_space(12.0);

            // God Enchantments List
            egui::Frame::new()
                .fill(palette.bg_panel)
                .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.add(atlas.ui_image("enchanted_book", 18.0));
                        ui.label(
                            RichText::new(t(lang, UiKey::GodEnchantsHeader))
                                .size(13.0)
                                .strong()
                                .color(palette.accent_gold),
                        );
                    });
                    ui.add_space(6.0);

                    for ench in &item.enchantments {
                        ui.horizontal_wrapped(|ui| {
                            let (br, _) =
                                ui.allocate_exact_size(Vec2::splat(26.0), egui::Sense::hover());
                            paint_mc_slot(
                                ui.painter(),
                                br,
                                palette,
                                atlas,
                                Some("enchanted_book"),
                                SlotStyle {
                                    glint_time: Some(time),
                                    ..Default::default()
                                },
                            );
                            let full_title = if ench.level.is_empty() {
                                ench.name.get(lang).to_string()
                            } else {
                                format!("{} {}", ench.name.get(lang), ench.level)
                            };
                            ui.label(
                                RichText::new(full_title)
                                    .size(13.0)
                                    .strong()
                                    .color(palette.accent_enchant),
                            );
                            ui.label(
                                RichText::new(format!("— {}", ench.desc.get(lang)))
                                    .size(12.5)
                                    .color(palette.text_primary),
                            );
                        });
                        ui.add_space(3.0);
                    }
                });

            ui.add_space(12.0);

            // Interactive "Yunque Vivo" Binary Tree Anvil Optimizer
            let forge_border = if state.anvil_anim.flash_timer > 0.0 {
                palette.accent_xp
            } else {
                palette.border_strong
            };
            let anvil_frame = egui::Frame::new()
                .fill(palette.bg_panel)
                .stroke(Stroke::new(1.5_f32, forge_border))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_wrapped(|ui| {
                        // Animated Anvil Icon with hammer recoil
                        let hammer_p = (state.anvil_anim.hammer_timer / 0.36).clamp(0.0, 1.0);
                        let bounce = (hammer_p * std::f32::consts::PI).sin() * 5.0;
                        let (anvil_r, _) =
                            ui.allocate_exact_size(Vec2::splat(40.0), egui::Sense::hover());
                        let shifted = anvil_r.translate(Vec2::new(0.0, bounce));
                        paint_mc_frame(ui.painter(), anvil_r, palette, Some(forge_border));
                        atlas.paint(ui.painter(), shifted.shrink(5.0), "anvil", Color32::WHITE);
                        if state.anvil_anim.flash_timer > 0.0 {
                            paint_enchant_glint(
                                ui.painter(),
                                anvil_r,
                                time,
                                cubic_out(state.anvil_anim.flash_timer / 0.42),
                            );
                        }

                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(t(lang, UiKey::AnvilOptimizerHeader))
                                    .size(13.0)
                                    .strong()
                                    .color(palette.accent_xp),
                            );

                            // Live Optimal vs Naive Cost summary bar
                            ui.horizontal_wrapped(|ui| {
                                let saved = plan.naive_total_levels.saturating_sub(plan.total_levels);
                                ui.label(
                                    RichText::new(format!(
                                        "{}: {} Lv ({}: {} Lv)",
                                        t(lang, UiKey::AnvilOptimalLabel),
                                        plan.total_levels,
                                        t(lang, UiKey::AnvilMaxStep),
                                        plan.max_step_cost
                                    ))
                                    .size(12.0)
                                    .strong()
                                    .color(palette.accent_xp),
                                );
                                ui.label(RichText::new("|").color(palette.text_muted));
                                let naive_col = if plan.naive_hits_too_expensive {
                                    palette.accent_danger
                                } else {
                                    palette.accent_gold
                                };
                                let naive_suffix = if plan.naive_hits_too_expensive {
                                    format!(" ({})", t(lang, UiKey::AnvilTooExpensive))
                                } else {
                                    String::new()
                                };
                                ui.label(
                                    RichText::new(format!(
                                        "{}: {} Lv{}",
                                        t(lang, UiKey::AnvilNaiveLabel),
                                        plan.naive_total_levels,
                                        naive_suffix
                                    ))
                                    .size(12.0)
                                    .color(naive_col),
                                );
                                if saved > 0 {
                                    ui.label(
                                        RichText::new(format!(
                                            "[+{} {}]",
                                            saved,
                                            t(lang, UiKey::AnvilSavedLevels)
                                        ))
                                        .size(11.5)
                                        .strong()
                                        .color(palette.accent_primary),
                                    );
                                }
                            });
                        });
                    });

                    ui.add_space(8.0);

                    // Action Buttons Row
                    let anvil_burst_pos = ui.next_widget_position() + Vec2::new(60.0, 10.0);
                    ui.horizontal_wrapped(|ui| {
                        let done_all = state.anvil_anim.completed_steps >= total_steps;
                        if ui
                            .add_enabled(
                                !done_all,
                                egui::Button::new(
                                    RichText::new(t(lang, UiKey::AnvilForgeButton))
                                        .strong()
                                        .color(if done_all {
                                            palette.text_muted
                                        } else {
                                            palette.accent_xp
                                        }),
                                ),
                            )
                            .clicked()
                        {
                            state.anvil_anim.completed_steps += 1;
                            state.anvil_anim.trigger_strike();
                            particles.spawn_anvil_burst(anvil_burst_pos);
                            if state.anvil_anim.completed_steps >= total_steps {
                                audio.play(SoundCue::LevelUp);
                                toasts.push(
                                    t(lang, UiKey::ToastForgeCompleteTitle),
                                    item.name.get(lang),
                                    &item.icon,
                                    palette.accent_xp,
                                );
                            } else {
                                audio.play(SoundCue::AnvilUse);
                            }
                        }

                        if ui
                            .add_enabled(
                                !done_all && total_steps > 1,
                                egui::Button::new(t(lang, UiKey::AnvilSimulateStep)),
                            )
                            .clicked()
                        {
                            state.anvil_anim.completed_steps = total_steps;
                            state.anvil_anim.trigger_strike();
                            particles.spawn_anvil_burst(anvil_burst_pos);
                            audio.play(SoundCue::LevelUp);
                            toasts.push(
                                t(lang, UiKey::ToastForgeCompleteTitle),
                                item.name.get(lang),
                                &item.icon,
                                palette.accent_xp,
                            );
                        }

                        if state.anvil_anim.completed_steps > 0
                            && ui.button(t(lang, UiKey::AnvilResetButton)).clicked()
                        {
                            state.anvil_anim.completed_steps = 0;
                            audio.play(SoundCue::Click);
                        }
                    });

                    ui.add_space(8.0);

                    // Computed Binary Merge Steps
                    for (idx, step) in plan.steps.iter().enumerate() {
                        let done = idx < state.anvil_anim.completed_steps;
                        let is_next = idx == state.anvil_anim.completed_steps;
                        let step_bg = if done {
                            Color32::from_rgba_unmultiplied(
                                palette.accent_xp.r(),
                                palette.accent_xp.g(),
                                palette.accent_xp.b(),
                                22,
                            )
                        } else if is_next {
                            palette.bg_card_hover
                        } else {
                            palette.bg_card
                        };
                        let step_stroke = if step.too_expensive {
                            Stroke::new(1.5_f32, palette.accent_danger)
                        } else if is_next {
                            Stroke::new(1.5_f32, palette.accent_primary)
                        } else {
                            Stroke::new(1.0_f32, palette.border_subtle)
                        };

                        egui::Frame::new()
                            .fill(step_bg)
                            .stroke(step_stroke)
                            .corner_radius(CornerRadius::same(4))
                            .inner_margin(Margin::symmetric(8, 5))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal_wrapped(|ui| {
                                    let prefix = if done {
                                        "[OK]"
                                    } else if is_next {
                                        "[>>]"
                                    } else {
                                        "[  ]"
                                    };
                                    ui.label(
                                        RichText::new(format!(
                                            "{} {} {}",
                                            prefix,
                                            t(lang, UiKey::AnvilStepsPrefix),
                                            step.step_num
                                        ))
                                        .monospace()
                                        .size(11.5)
                                        .strong()
                                        .color(if done {
                                            palette.accent_xp
                                        } else {
                                            palette.accent_primary
                                        }),
                                    );
                                    ui.label(
                                        RichText::new(format!(
                                            "{}  +  {}  ->  {}",
                                            step.target_label,
                                            step.sacrifice_label,
                                            step.result_label
                                        ))
                                        .size(12.0)
                                        .color(palette.text_primary),
                                    );
                                    let xp_col = if step.too_expensive {
                                        palette.accent_danger
                                    } else {
                                        palette.accent_xp
                                    };
                                    ui.label(
                                        RichText::new(format!("[{} Lv]", step.level_cost))
                                            .size(11.5)
                                            .strong()
                                            .color(xp_col),
                                    );
                                });
                            });
                        ui.add_space(3.0);
                    }

                    // Prose anvil guide bullets (localized reference)
                    if !item.anvil_order.is_empty() {
                        ui.add_space(4.0);
                        for line in &item.anvil_order {
                            ui.label(
                                RichText::new(format!("• {}", line.get(lang)))
                                    .size(11.5)
                                    .color(palette.text_muted),
                            );
                        }
                    }

                    // Incompatibilities warning
                    if !item.incompatibilities.is_empty() {
                        ui.add_space(8.0);
                        egui::Frame::new()
                            .fill(Color32::from_rgba_unmultiplied(238, 86, 86, 26))
                            .stroke(Stroke::new(1.0_f32, palette.accent_danger))
                            .corner_radius(CornerRadius::same(4))
                            .inner_margin(Margin::symmetric(8, 6))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal_wrapped(|ui| {
                                    ui.add(atlas.ui_image("barrier", 16.0));
                                    ui.label(
                                        RichText::new(format!("{}:", t(lang, UiKey::AnvilIncompatHeader)))
                                            .size(11.5)
                                            .strong()
                                            .color(palette.accent_danger),
                                    );
                                    for inc in &item.incompatibilities {
                                        ui.label(
                                            RichText::new(format!("[{}]", inc.get(lang)))
                                                .size(11.5)
                                                .color(palette.text_primary),
                                        );
                                    }
                                });
                            });
                    }
                });

            let _ = anvil_frame;
        });

    if crate::ui::chrome::is_window_active(ui.ctx()) {
        if state.anvil_anim.hammer_timer > 0.0 || state.anvil_anim.flash_timer > 0.0 {
            ui.ctx().request_repaint();
        }
        // Keep glint shimmer flowing smoothly at ~30 FPS while window is active
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(33));
    }
    let _ = Pos2::ZERO;
    let _ = Rect::NOTHING;
}

struct InfoCard<'a> {
    icon_id: &'a str,
    title: &'a str,
    body: &'a str,
    accent: Color32,
}

fn render_info_box(
    ui: &mut egui::Ui,
    width: f32,
    palette: McPalette,
    atlas: &ItemAtlas,
    card: InfoCard<'_>,
) {
    let InfoCard {
        icon_id,
        title,
        body,
        accent,
    } = card;
    ui.allocate_ui_with_layout(
        Vec2::new(width, 0.0),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            egui::Frame::new()
                .fill(palette.bg_card)
                .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(10))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.add(atlas.ui_image(icon_id, 16.0));
                        ui.label(
                            RichText::new(title)
                                .size(12.0)
                                .strong()
                                .color(accent),
                        );
                    });
                    ui.add_space(4.0);
                    ui.add(
                        egui::Label::new(
                            RichText::new(body)
                                .size(12.5)
                                .color(palette.text_primary),
                        )
                        .wrap(),
                    );
                });
        },
    );
}
