use eframe::egui::{self, Color32, CornerRadius, Margin, Pos2, Rect, RichText, Stroke, Vec2};
use std::collections::BTreeMap;

use crate::anim::ToastQueue;
use crate::audio::SoundCue;
use crate::data::models::Language;
use crate::data::recipes::{
    translate_brewing_note, CraftingStation, RecipeCategory, RecipeEntry,
};
use crate::i18n::{t, UiKey};
use crate::state::{AppState, RecipeViewMode};
use crate::ui::chrome::render_inline_tab_search;
use crate::ui::theme::{
    filter_chip, paint_mc_frame, paint_mc_slot, show_mc_tooltip_at, SlotStyle,
};
use crate::ui::RenderContext;

use super::alchemy::render_alchemy_guide;
use super::tree::render_craft_tree_section;
pub(super) fn render_jei_inspector(
    ui: &mut egui::Ui,
    state: &mut AppState,
    rcx: &RenderContext<'_>,
    active_ver: &str,
    toasts: &mut ToastQueue,
    tag_step: usize,
) {
    let RenderContext {
        recipes,
        versions,
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
    let lang = state.prefs.language;
    let target_id = state.selected_recipe_item.clone();
    let display_name = recipes.item_name_in(&target_id, versions, active_ver, lang);

    egui::ScrollArea::vertical()
        .id_salt("jei_inspector_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Frame::new()
                .fill(palette.bg_panel)
                .stroke(Stroke::new(1.5_f32, palette.border_strong))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    // Selected Item Header + Recipe/Uses Toggle + Favorite Pin
                    ui.horizontal(|ui| {
                        let (slot_r, _) =
                            ui.allocate_exact_size(Vec2::splat(44.0), egui::Sense::hover());
                        paint_mc_slot(
                            ui.painter(),
                            slot_r,
                            palette,
                            atlas,
                            Some(&target_id),
                            SlotStyle::selected(true),
                        );
                        ui.vertical(|ui| {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&display_name)
                                        .size(16.0)
                                        .strong()
                                        .color(palette.accent_primary),
                                )
                                .wrap(),
                            );
                            ui.label(
                                RichText::new(format!("minecraft:{}", target_id))
                                    .monospace()
                                    .size(10.5)
                                    .color(palette.text_muted),
                            );
                        });
                    });

                    ui.add_space(6.0);

                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(5.0, 4.0);
                        if filter_chip(
                            ui,
                            palette,
                            atlas,
                            Some("crafting_table"),
                            t(lang, UiKey::RecipesForTitle),
                            state.recipe_view_mode == RecipeViewMode::RecipesFor,
                        )
                        .clicked()
                        {
                            state.recipe_view_mode = RecipeViewMode::RecipesFor;
                            state.recipe_variant_idx = 0;
                            audio.play(SoundCue::Click);
                        }
                        if filter_chip(
                            ui,
                            palette,
                            atlas,
                            Some("knowledge_book"),
                            &format!(
                                "{} ({})",
                                t(lang, UiKey::UsesForTitle),
                                recipes.uses_for_in(&target_id, versions, active_ver).len()
                            ),
                            state.recipe_view_mode == RecipeViewMode::UsesFor,
                        )
                        .clicked()
                        {
                            state.recipe_view_mode = RecipeViewMode::UsesFor;
                            audio.play(SoundCue::Click);
                        }

                        let is_fav = state.prefs.favorite_items.contains(&target_id);
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
                            let added = state.prefs.toggle_favorite_item(&target_id);
                            audio.play(SoundCue::Click);
                            if added {
                                toasts.push(
                                    t(lang, UiKey::ToastFavoriteAddedTitle),
                                    &display_name,
                                    &target_id,
                                    palette.accent_gold,
                                );
                            }
                        }

                        let tree_lbl = if state.show_craft_tree {
                            t(lang, UiKey::HideTreeBtn)
                        } else {
                            t(lang, UiKey::ShowTreeBtn)
                        };
                        if filter_chip(
                            ui,
                            palette,
                            atlas,
                            Some("oak_sapling"),
                            tree_lbl,
                            state.show_craft_tree,
                        )
                        .clicked()
                        {
                            state.show_craft_tree = !state.show_craft_tree;
                            audio.play(SoundCue::Click);
                        }
                    });

                    ui.separator();

                    match state.recipe_view_mode {
                        RecipeViewMode::RecipesFor => {
                            let active_for_item: Vec<usize> = recipes
                                .recipes_for(&target_id)
                                .iter()
                                .copied()
                                .filter(|&idx| recipes.is_recipe_active(idx, versions, active_ver))
                                .collect();
                            let recipe_indices: Vec<usize> = active_for_item
                                .iter()
                                .copied()
                                .filter(|&idx| {
                                    state
                                        .recipe_station_filter
                                        .is_none_or(|st| recipes.recipes[idx].station == st)
                                })
                                .collect();

                            let fallback_list = if recipe_indices.is_empty() {
                                active_for_item
                            } else {
                                recipe_indices
                            };

                            if fallback_list.is_empty() {
                                ui.label(
                                    RichText::new(t(lang, UiKey::NoRecipeSelected))
                                        .color(palette.text_muted),
                                );
                            } else {
                                if state.recipe_variant_idx >= fallback_list.len() {
                                    state.recipe_variant_idx = 0;
                                }
                                let cur_idx = fallback_list[state.recipe_variant_idx];
                                let entry = recipes.recipes[cur_idx].clone();

                                // Station header + Variant Pager
                                ui.horizontal_wrapped(|ui| {
                                    ui.add(atlas.ui_image(entry.station.icon_id(), 18.0));
                                    ui.label(
                                        RichText::new(entry.station.label(lang))
                                            .strong()
                                            .size(13.0)
                                            .color(palette.accent_gold),
                                    );
                                    let badge = if entry.station == CraftingStation::Crafting {
                                        if entry.is_2x2 {
                                            t(lang, UiKey::Grid2x2Badge)
                                        } else if entry.shapeless {
                                            t(lang, UiKey::ShapelessBadge)
                                        } else {
                                            t(lang, UiKey::ShapedBadge)
                                        }
                                    } else {
                                        entry.category.label(lang)
                                    };
                                    ui.label(
                                        RichText::new(format!("[{}]", badge))
                                            .size(11.0)
                                            .color(palette.text_muted),
                                    );

                                    if fallback_list.len() > 1 {
                                        ui.add_space(6.0);
                                        if ui.small_button("<").clicked() {
                                            state.recipe_variant_idx =
                                                (state.recipe_variant_idx + fallback_list.len() - 1)
                                                    % fallback_list.len();
                                            audio.play(SoundCue::Click);
                                        }
                                        ui.label(
                                            RichText::new(format!(
                                                "{} {}/{}",
                                                t(lang, UiKey::RecipeVariantPrefix),
                                                state.recipe_variant_idx + 1,
                                                fallback_list.len()
                                            ))
                                            .size(11.5)
                                            .color(palette.text_primary),
                                        );
                                        if ui.small_button(">").clicked() {
                                            state.recipe_variant_idx =
                                                (state.recipe_variant_idx + 1) % fallback_list.len();
                                            audio.play(SoundCue::Click);
                                        }
                                    }
                                });

                                ui.add_space(10.0);

                                // Render Authentic Station GUI Layout
                                let mut pending_nav: Option<(String, RecipeViewMode)> = None;
                                render_station_gui(
                                    ui,
                                    &entry,
                                    rcx,
                                    active_ver,
                                    tag_step,
                                    lang,
                                    &mut pending_nav,
                                );

                                if !entry.note.is_empty() {
                                    ui.add_space(6.0);
                                    let shown_note = if entry.station == CraftingStation::Brewing {
                                        translate_brewing_note(&entry.note, lang)
                                    } else {
                                        entry.note.clone()
                                    };
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(shown_note)
                                                .size(12.0)
                                                .strong()
                                                .color(palette.accent_xp),
                                        )
                                        .wrap(),
                                    );
                                }

                                ui.add_space(6.0);
                                ui.label(
                                    RichText::new(format!(
                                        "{}  |  {}",
                                        t(lang, UiKey::LeftClickRecipeHint),
                                        t(lang, UiKey::RightClickUsesHint)
                                    ))
                                    .size(11.0)
                                    .color(palette.text_muted),
                                );

                                // Quick Ingredient Chips Breakdown
                                let mut counts: BTreeMap<String, u32> = BTreeMap::new();
                                for s in &entry.slots {
                                    if !s.is_empty() {
                                        *counts.entry(s.clone()).or_default() += 1;
                                    }
                                }
                                if !counts.is_empty() {
                                    ui.add_space(8.0);
                                    ui.label(
                                        RichText::new(t(lang, UiKey::MaterialsHeader))
                                            .size(11.5)
                                            .strong()
                                            .color(palette.accent_gold),
                                    );
                                    ui.horizontal_wrapped(|ui| {
                                        ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);
                                        for (tok, qty) in counts {
                                            let vis = recipes.resolve_slot_icon_in(
                                                &tok,
                                                tag_step,
                                                versions,
                                                active_ver,
                                            );
                                            let label = format!(
                                                "{}x {}",
                                                qty,
                                                recipes.item_name_in(&tok, versions, active_ver, lang)
                                            );
                                            if filter_chip(
                                                ui,
                                                palette,
                                                atlas,
                                                Some(&vis),
                                                &label,
                                                false,
                                            )
                                            .clicked()
                                            {
                                                pending_nav =
                                                    Some((vis, RecipeViewMode::RecipesFor));
                                            }
                                        }
                                    });
                                }

                                if let Some((nid, mode)) = pending_nav {
                                    state.open_recipe_item(&nid, mode);
                                    audio.play(SoundCue::Click);
                                }
                            }
                        }

                        RecipeViewMode::UsesFor => {
                            let uses = recipes.uses_for_in(&target_id, versions, active_ver);
                            if uses.is_empty() {
                                ui.label(
                                    RichText::new(t(lang, UiKey::NoUsesFound))
                                        .color(palette.text_muted),
                                );
                            } else {
                                ui.label(
                                    RichText::new(format!(
                                        "{} ({})",
                                        t(lang, UiKey::UsesForTitle),
                                        uses.len()
                                    ))
                                    .size(12.0)
                                    .strong()
                                    .color(palette.accent_gold),
                                );
                                ui.add_space(6.0);
                                let mut pick: Option<String> = None;
                                for &idx in uses.iter().take(60) {
                                    let r = &recipes.recipes[idx];
                                    let res_name = recipes.item_name(&r.result_id, lang);
                                    let row_r = ui.horizontal(|ui| {
                                        let (sr, sresp) = ui.allocate_exact_size(
                                            Vec2::splat(30.0),
                                            egui::Sense::click(),
                                        );
                                        paint_mc_slot(
                                            ui.painter(),
                                            sr,
                                            palette,
                                            atlas,
                                            Some(&r.result_id),
                                            SlotStyle {
                                                count: r.result_count,
                                                hovered: sresp.hovered(),
                                                ..Default::default()
                                            },
                                        );
                                        ui.add(atlas.ui_image(r.station.icon_id(), 16.0));
                                        let b = ui.add(
                                            egui::Button::new(
                                                RichText::new(format!(
                                                    "{} ({})",
                                                    res_name,
                                                    r.station.label(lang)
                                                ))
                                                .size(12.0)
                                                .color(palette.text_primary),
                                            )
                                            .frame(false),
                                        );
                                        if sresp.clicked() || b.clicked() {
                                            pick = Some(r.result_id.clone());
                                        }
                                    });
                                    let _ = row_r;
                                }
                                if let Some(id) = pick {
                                    state.open_recipe_item(&id, RecipeViewMode::RecipesFor);
                                    audio.play(SoundCue::Click);
                                }
                            }
                        }
                    }


                    // EMI Recursive Craft Tree & Raw Base Bill of Materials (+ Shopping List)
                    if state.show_craft_tree {
                        render_craft_tree_section(ui, state, rcx, active_ver, toasts, &target_id);
                    }

                    let brewing_filter_active =
                        state.recipe_station_filter == Some(CraftingStation::Brewing)
                            || state.recipe_category_filter == RecipeCategory::Brewing;
                    let is_potion_selected = matches!(
                        target_id.as_str(),
                        "potion"
                            | "splash_potion"
                            | "lingering_potion"
                            | "brewing_stand"
                            | "tipped_arrow"
                            | "nether_wart"
                    );
                    if !brewing_filter_active && is_potion_selected {
                        ui.add_space(12.0);
                        let mut guide_nav: Option<String> = None;
                        render_alchemy_guide(
                            ui,
                            rcx,
                            active_ver,
                            lang,
                            &state.search_query.trim().to_lowercase(),
                            &mut guide_nav,
                        );
                        if let Some(id) = guide_nav {
                            state.open_recipe_item(&id, RecipeViewMode::RecipesFor);
                            audio.play(SoundCue::Click);
                        }
                    }
                });
        });
}

fn render_station_gui(
    ui: &mut egui::Ui,
    entry: &RecipeEntry,
    rcx: &RenderContext<'_>,
    active_ver: &str,
    tag_step: usize,
    lang: Language,
    pending_nav: &mut Option<(String, RecipeViewMode)>,
) {
    let RenderContext {
        recipes,
        versions,
        palette,
        atlas,
        ..
    } = *rcx;
    let avail_w = ui.available_width();
    let slot_sz = if avail_w < 340.0 { 36.0 } else { 44.0 };
    let gap = 4.0;
    let out_sz = if avail_w < 340.0 { 44.0 } else { 54.0 };

    egui::Frame::new()
        .fill(palette.bg_card)
        .stroke(Stroke::new(1.5_f32, palette.border_strong))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            match entry.station {
                CraftingStation::Crafting => {
                    ui.horizontal(|ui| {
                        // 3x3 Grid
                        let grid_w = slot_sz * 3.0 + gap * 2.0;
                        let (grid_rect, _) = ui.allocate_exact_size(
                            Vec2::splat(grid_w),
                            egui::Sense::hover(),
                        );
                        for idx in 0..9 {
                            let row = idx / 3;
                            let col = idx % 3;
                            let pos = Pos2::new(
                                grid_rect.left() + (col as f32) * (slot_sz + gap),
                                grid_rect.top() + (row as f32) * (slot_sz + gap),
                            );
                            let srect = Rect::from_min_size(pos, Vec2::splat(slot_sz));
                            let token = entry.slots.get(idx).map(|s| s.as_str()).unwrap_or("");
                            handle_recipe_slot(
                                ui,
                                srect,
                                RecipeSlotSpec {
                                    token,
                                    count: 1,
                                    tag_step,
                                    active_ver,
                                },
                                rcx,
                                lang,
                                pending_nav,
                            );
                        }

                        ui.add_space(if avail_w < 340.0 { 6.0 } else { 12.0 });
                        ui.vertical(|ui| {
                            ui.add_space(slot_sz + gap);
                            ui.label(
                                RichText::new("->")
                                    .monospace()
                                    .size(if avail_w < 340.0 { 14.0 } else { 18.0 })
                                    .strong()
                                    .color(palette.accent_gold),
                            );
                        });
                        ui.add_space(if avail_w < 340.0 { 6.0 } else { 12.0 });

                        // Output slot centered vertically beside 3x3 grid
                        ui.vertical(|ui| {
                            ui.add_space(((grid_w - out_sz) * 0.5).max(0.0));
                            let (out_r, _) =
                                ui.allocate_exact_size(Vec2::splat(out_sz), egui::Sense::hover());
                            handle_recipe_slot(
                                ui,
                                out_r,
                                RecipeSlotSpec {
                                    token: &entry.result_id,
                                    count: entry.result_count,
                                    tag_step,
                                    active_ver,
                                },
                                rcx,
                                lang,
                                pending_nav,
                            );
                        });
                    });
                }

                CraftingStation::Smelting
                | CraftingStation::Blasting
                | CraftingStation::Smoking
                | CraftingStation::Campfire => {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            let (in_r, _) =
                                ui.allocate_exact_size(Vec2::splat(slot_sz), egui::Sense::hover());
                            let tok = entry.slots.first().map(|s| s.as_str()).unwrap_or("");
                            handle_recipe_slot(
                                ui,
                                in_r,
                                RecipeSlotSpec {
                                    token: tok,
                                    count: 1,
                                    tag_step,
                                    active_ver,
                                },
                                rcx,
                                lang,
                                pending_nav,
                            );
                            ui.add_space(4.0);
                            let (fuel_r, _) =
                                ui.allocate_exact_size(Vec2::splat(slot_sz), egui::Sense::hover());
                            handle_recipe_slot(
                                ui,
                                fuel_r,
                                RecipeSlotSpec {
                                    token: "coal",
                                    count: 1,
                                    tag_step,
                                    active_ver,
                                },
                                rcx,
                                lang,
                                pending_nav,
                            );
                        });
                        ui.add_space(14.0);
                        ui.vertical(|ui| {
                            ui.add_space(12.0);
                            ui.add(atlas.ui_image("blaze_powder", 20.0));
                            ui.label(
                                RichText::new("===>")
                                    .monospace()
                                    .size(15.0)
                                    .strong()
                                    .color(palette.accent_gold),
                            );
                            ui.label(
                                RichText::new(t(lang, UiKey::SmeltingTimeHint))
                                    .size(10.0)
                                    .color(palette.text_muted),
                            );
                        });
                        ui.add_space(14.0);
                        let (out_r, _) =
                            ui.allocate_exact_size(Vec2::splat(54.0), egui::Sense::hover());
                        handle_recipe_slot(
                            ui,
                            out_r,
                            RecipeSlotSpec {
                                token: &entry.result_id,
                                count: entry.result_count,
                                tag_step,
                                active_ver,
                            },
                            rcx,
                            lang,
                            pending_nav,
                        );
                    });
                }

                CraftingStation::Smithing => {
                    let legacy_2slot = entry.slots.first().is_some_and(|s| s.is_empty());
                    ui.horizontal_wrapped(|ui| {
                        let start_idx = if legacy_2slot { 1 } else { 0 };
                        for i in start_idx..3 {
                            let (sr, _) =
                                ui.allocate_exact_size(Vec2::splat(slot_sz), egui::Sense::hover());
                            let tok = entry.slots.get(i).map(|s| s.as_str()).unwrap_or("");
                            handle_recipe_slot(
                                ui,
                                sr,
                                RecipeSlotSpec {
                                    token: tok,
                                    count: 1,
                                    tag_step,
                                    active_ver,
                                },
                                rcx,
                                lang,
                                pending_nav,
                            );
                        }
                        ui.label(
                            RichText::new("===>")
                                .monospace()
                                .size(15.0)
                                .strong()
                                .color(palette.accent_gold),
                        );
                        let (out_r, _) =
                            ui.allocate_exact_size(Vec2::splat(52.0), egui::Sense::hover());
                        handle_recipe_slot(
                            ui,
                            out_r,
                            RecipeSlotSpec {
                                token: &entry.result_id,
                                count: entry.result_count,
                                tag_step,
                                active_ver,
                            },
                            rcx,
                            lang,
                            pending_nav,
                        );
                    });
                    if legacy_2slot {
                        ui.add_space(4.0);
                        let hint = match lang {
                            Language::EnUs => "Legacy Smithing Table (no Upgrade Template required in <=1.19.4)",
                            _ => "Mesa de Herrería clásica (sin Plantilla de Mejora en <=1.19.4)",
                        };
                        ui.label(RichText::new(hint).size(10.5).color(palette.accent_xp));
                    }
                }

                CraftingStation::Stonecutting => {
                    ui.horizontal(|ui| {
                        for (i, tok) in entry.slots.iter().enumerate() {
                            if tok.is_empty() && i > 1 {
                                continue;
                            }
                            let (sr, _) =
                                ui.allocate_exact_size(Vec2::splat(slot_sz), egui::Sense::hover());
                            handle_recipe_slot(
                                ui,
                                sr,
                                RecipeSlotSpec {
                                    token: tok,
                                    count: 1,
                                    tag_step,
                                    active_ver,
                                },
                                rcx,
                                lang,
                                pending_nav,
                            );
                        }
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("===>")
                                .monospace()
                                .size(15.0)
                                .strong()
                                .color(palette.accent_gold),
                        );
                        ui.add_space(10.0);
                        let (out_r, _) =
                            ui.allocate_exact_size(Vec2::splat(52.0), egui::Sense::hover());
                        handle_recipe_slot(
                            ui,
                            out_r,
                            RecipeSlotSpec {
                                token: &entry.result_id,
                                count: entry.result_count,
                                tag_step,
                                active_ver,
                            },
                            rcx,
                            lang,
                            pending_nav,
                        );
                    });
                }

                CraftingStation::Brewing => {
                    ui.horizontal(|ui| {
                        // Left fuel gauge column (Blaze Powder powers 20 brews)
                        ui.vertical(|ui| {
                            let (fuel_r, _) =
                                ui.allocate_exact_size(Vec2::splat(36.0), egui::Sense::hover());
                            handle_recipe_slot(
                                ui,
                                fuel_r,
                                RecipeSlotSpec {
                                    token: "blaze_powder",
                                    count: 1,
                                    tag_step,
                                    active_ver,
                                },
                                rcx,
                                lang,
                                pending_nav,
                            );
                            ui.label(
                                RichText::new(t(lang, UiKey::BrewingFuelHint))
                                    .size(9.5)
                                    .color(palette.accent_gold),
                            );
                        });
                        ui.add_space(10.0);

                        // Center Brewing Stand column: Ingredient top -> Bottle(s) bottom -> Result
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.add_space(6.0);
                                let ing_tok = entry.slots.get(1).map(|s| s.as_str()).unwrap_or("");
                                let (ing_r, _) = ui.allocate_exact_size(
                                    Vec2::splat(slot_sz),
                                    egui::Sense::hover(),
                                );
                                handle_recipe_slot(
                                    ui,
                                    ing_r,
                                    RecipeSlotSpec {
                                        token: ing_tok,
                                        count: 1,
                                        tag_step,
                                        active_ver,
                                    },
                                    rcx,
                                    lang,
                                    pending_nav,
                                );
                                ui.label(
                                    RichText::new(
                                        recipes.item_name_in(ing_tok, versions, active_ver, lang),
                                    )
                                        .size(11.5)
                                        .strong()
                                        .color(palette.accent_primary),
                                );
                            });
                            ui.add_space(2.0);
                            ui.label(
                                RichText::new("   |   v   (+)")
                                    .monospace()
                                    .size(12.5)
                                    .strong()
                                    .color(palette.accent_enchant),
                            );
                            ui.add_space(2.0);
                            ui.horizontal(|ui| {
                                let base_tok =
                                    entry.slots.first().map(|s| s.as_str()).unwrap_or("potion");
                                let (base_r, _) = ui.allocate_exact_size(
                                    Vec2::splat(slot_sz),
                                    egui::Sense::hover(),
                                );
                                handle_recipe_slot(
                                    ui,
                                    base_r,
                                    RecipeSlotSpec {
                                        token: base_tok,
                                        count: 1,
                                        tag_step,
                                        active_ver,
                                    },
                                    rcx,
                                    lang,
                                    pending_nav,
                                );
                                ui.add_space(6.0);
                                ui.label(
                                    RichText::new("===>")
                                        .monospace()
                                        .size(15.0)
                                        .strong()
                                        .color(palette.accent_gold),
                                );
                                ui.add_space(6.0);
                                let (out_r, _) = ui.allocate_exact_size(
                                    Vec2::splat(52.0),
                                    egui::Sense::hover(),
                                );
                                handle_recipe_slot(
                                    ui,
                                    out_r,
                                    RecipeSlotSpec {
                                        token: &entry.result_id,
                                        count: entry.result_count,
                                        tag_step,
                                        active_ver,
                                    },
                                    rcx,
                                    lang,
                                    pending_nav,
                                );
                            });
                        });
                    });
                }
            }
        });
}

struct RecipeSlotSpec<'a> {
    token: &'a str,
    count: u32,
    tag_step: usize,
    active_ver: &'a str,
}

fn handle_recipe_slot(
    ui: &mut egui::Ui,
    rect: Rect,
    spec: RecipeSlotSpec<'_>,
    rcx: &RenderContext<'_>,
    lang: Language,
    pending_nav: &mut Option<(String, RecipeViewMode)>,
) {
    let RecipeSlotSpec {
        token,
        count,
        tag_step,
        active_ver,
    } = spec;
    let RenderContext {
        recipes,
        versions,
        palette,
        atlas,
        ..
    } = *rcx;
    let resolved = if token.is_empty() {
        String::new()
    } else {
        recipes.resolve_slot_icon_in(token, tag_step, versions, active_ver)
    };
    let resp = ui.interact(rect, ui.make_persistent_id(rect.min.x as i32 ^ ((rect.min.y as i32) << 12)), egui::Sense::click());
    let hovered = resp.hovered() && !resolved.is_empty();
    paint_mc_slot(
        ui.painter(),
        rect,
        palette,
        atlas,
        if resolved.is_empty() { None } else { Some(&resolved) },
        SlotStyle {
            count,
            hovered,
            ..Default::default()
        },
    );

    if !resolved.is_empty() {
        if resp.clicked() {
            *pending_nav = Some((resolved.clone(), RecipeViewMode::RecipesFor));
        } else if resp.secondary_clicked() {
            *pending_nav = Some((resolved.clone(), RecipeViewMode::UsesFor));
        }
        if hovered {
            if let Some(pointer) = ui.ctx().pointer_latest_pos() {
                let title = recipes.item_name_in(&resolved, versions, active_ver, lang);
                let mut lines: Vec<(&str, Color32)> = Vec::new();
                if token.starts_with('#') {
                    lines.push((t(lang, UiKey::AnyTagCycleHint), palette.accent_gold));
                }
                lines.push((t(lang, UiKey::LeftClickRecipeHint), palette.accent_primary));
                lines.push((t(lang, UiKey::RightClickUsesHint), palette.text_muted));
                show_mc_tooltip_at(ui.ctx(), pointer, &title, &lines, palette);
            }
        }
    }
}


pub(super) fn render_jei_catalog_grid(
    ui: &mut egui::Ui,
    state: &mut AppState,
    rcx: &RenderContext<'_>,
    active_ver: &str,
    filtered_items: &[&str],
    toasts: &mut ToastQueue,
    want_kb: bool,
) {
    let RenderContext {
        recipes,
        versions,
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
    let lang = state.prefs.language;
    // Strictly compute grid dimensions from ACTUAL available panel width so items NEVER overflow right!
    let avail_w = ui.available_width().max(200.0);
    let inner_pad = 10.0;
    let slot_sz = 38.0;
    let spacing = 5.0;
    let usable_w = (avail_w - inner_pad * 2.0).max(slot_sz);
    let cols = (((usable_w + spacing) / (slot_sz + spacing)).floor() as usize).max(4);
    let avail_h = ui.available_height().max(260.0);
    let rows = ((((avail_h - 68.0) + spacing) / (slot_sz + spacing)).floor() as usize).clamp(4, 14);
    let per_page = (cols * rows).max(16);

    let total_pages = filtered_items.len().div_ceil(per_page).max(1);
    if state.recipe_catalog_page >= total_pages {
        state.recipe_catalog_page = total_pages - 1;
    }

    let start = state.recipe_catalog_page * per_page;
    let end = (start + per_page).min(filtered_items.len());
    let page_slice = &filtered_items[start..end];

    let panel_rect = ui.available_rect_before_wrap();
    let _ = paint_mc_frame;
    let _ = panel_rect;

    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.5_f32, palette.border_strong))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            // JEI Header + Inline Catalog Search + Page Nav
            render_inline_tab_search(ui, state, palette, atlas, audio);
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(format!("{} ({})", t(lang, UiKey::CatalogHeader), active_ver))
                        .size(13.0)
                        .strong()
                        .color(palette.accent_gold),
                );
                ui.label(
                    RichText::new(format!(
                        "({} {})",
                        filtered_items.len(),
                        t(lang, UiKey::ItemsCount)
                    ))
                    .size(11.5)
                    .color(palette.text_muted),
                );

                ui.add_space(10.0);
                if ui
                    .add_enabled(
                        state.recipe_catalog_page > 0,
                        egui::Button::new(format!("< {}", t(lang, UiKey::RecipePagePrev))),
                    )
                    .clicked()
                {
                    state.recipe_catalog_page -= 1;
                    audio.play(SoundCue::Click);
                }
                ui.label(
                    RichText::new(format!(
                        "{} / {}",
                        state.recipe_catalog_page + 1,
                        total_pages
                    ))
                    .monospace()
                    .size(12.0)
                    .color(palette.text_primary),
                );
                if ui
                    .add_enabled(
                        state.recipe_catalog_page + 1 < total_pages,
                        egui::Button::new(format!("{} >", t(lang, UiKey::RecipePageNext))),
                    )
                    .clicked()
                {
                    state.recipe_catalog_page += 1;
                    audio.play(SoundCue::Click);
                }
            });

            ui.add_space(6.0);

            // Responsive Wrapped Grid (never overflows horizontally!)
            let mut hovered_item: Option<String> = None;
            let mut clicked_recipe: Option<(String, RecipeViewMode)> = None;

            for chunk in page_slice.chunks(cols) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(spacing);
                    for &item_id in chunk {
                        let (srect, sresp) =
                            ui.allocate_exact_size(Vec2::splat(slot_sz), egui::Sense::click());
                        let selected = state.selected_recipe_item == item_id;
                        let hovered = sresp.hovered();
                        let count = recipes
                            .primary_recipe_for_in(item_id, versions, active_ver)
                            .map(|i| recipes.recipes[i].result_count)
                            .unwrap_or(1);

                        paint_mc_slot(
                            ui.painter(),
                            srect,
                            palette,
                            atlas,
                            Some(item_id),
                            SlotStyle {
                                count,
                                selected,
                                hovered,
                                glint_time: None,
                            },
                        );

                        if sresp.clicked() {
                            clicked_recipe = Some((item_id.to_string(), RecipeViewMode::RecipesFor));
                        } else if sresp.secondary_clicked() {
                            clicked_recipe = Some((item_id.to_string(), RecipeViewMode::UsesFor));
                        }

                        if hovered {
                            hovered_item = Some(item_id.to_string());
                            if let Some(ptr) = ui.ctx().pointer_latest_pos() {
                                let name = recipes.item_name_in(item_id, versions, active_ver, lang);
                                let uses_n = recipes.uses_for_in(item_id, versions, active_ver).len();
                                let rec_n = recipes
                                    .recipes_for(item_id)
                                    .iter()
                                    .filter(|&&idx| {
                                        recipes.is_recipe_active(idx, versions, active_ver)
                                    })
                                    .count();
                                let sub = format!("{} recipe(s) | {} use(s)", rec_n, uses_n);
                                show_mc_tooltip_at(
                                    ui.ctx(),
                                    ptr,
                                    &name,
                                    &[
                                        (&sub, palette.text_muted),
                                        (t(lang, UiKey::LeftClickRecipeHint), palette.accent_primary),
                                        (t(lang, UiKey::RightClickUsesHint), palette.accent_gold),
                                        (t(lang, UiKey::MiddleOrFavHint), palette.accent_xp),
                                    ],
                                    palette,
                                );
                            }
                        }
                    }
                });
                ui.add_space(spacing);
            }

            if let Some((id, mode)) = clicked_recipe {
                state.open_recipe_item(&id, mode);
                audio.play(SoundCue::Click);
            }

            let brewing_filter_active = state.recipe_station_filter == Some(CraftingStation::Brewing)
                || state.recipe_category_filter == RecipeCategory::Brewing;
            if brewing_filter_active {
                ui.add_space(8.0);
                let q_lower = state.search_query.trim().to_lowercase();
                let mut guide_nav: Option<String> = None;
                egui::ScrollArea::vertical()
                    .id_salt("brewing_guide_catalog_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        render_alchemy_guide(ui, rcx, active_ver, lang, &q_lower, &mut guide_nav);
                    });
                if let Some(id) = guide_nav {
                    state.open_recipe_item(&id, RecipeViewMode::RecipesFor);
                    audio.play(SoundCue::Click);
                }
            }

            // Keyboard shortcuts R / U / A acting on hovered item (or currently selected item)
            if want_kb {
                let kb_target = hovered_item.unwrap_or_else(|| state.selected_recipe_item.clone());
                if ui.ctx().input(|i| i.key_pressed(egui::Key::R)) {
                    state.open_recipe_item(&kb_target, RecipeViewMode::RecipesFor);
                    audio.play(SoundCue::Click);
                } else if ui.ctx().input(|i| i.key_pressed(egui::Key::U)) {
                    state.open_recipe_item(&kb_target, RecipeViewMode::UsesFor);
                    audio.play(SoundCue::Click);
                } else if ui.ctx().input(|i| i.key_pressed(egui::Key::A)) {
                    let added = state.prefs.toggle_favorite_item(&kb_target);
                    audio.play(SoundCue::Click);
                    if added {
                        toasts.push(
                            t(lang, UiKey::ToastFavoriteAddedTitle),
                            recipes.item_name(&kb_target, lang),
                            &kb_target,
                            palette.accent_gold,
                        );
                    }
                }
            }
        });
}

