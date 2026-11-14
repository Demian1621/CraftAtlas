use eframe::egui::{self, Color32, CornerRadius, Margin, Pos2, Rect, RichText, Stroke, Vec2};
use std::collections::BTreeMap;

use crate::anim::ToastQueue;
use crate::audio::SoundCue;
use crate::data::models::{format_stack_breakdown, Language};
use crate::data::recipes::{
    translate_brewing_note, CraftTreeNode, CraftingStation, RecipeCategory, RecipeEntry,
};
use crate::i18n::{t, UiKey};
use crate::state::{AppState, RecipeViewMode};
use crate::ui::chrome::render_inline_tab_search;
use crate::ui::theme::{
    filter_chip, paint_mc_frame, paint_mc_slot, show_mc_tooltip_at, SlotStyle,
};
use crate::ui::RenderContext;

pub fn render(
    ui: &mut egui::Ui,
    state: &mut AppState,
    rcx: &RenderContext<'_>,
    toasts: &mut ToastQueue,
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
    let active_ver = versions
        .resolve_valid_id(&state.prefs.selected_version)
        .to_string();
    let time = ui.ctx().input(|i| i.time);
    let tag_step = (time / 0.95) as usize;
    state.hovered_tooltip_id = None;

    // 1. Crafting Station Tabs + Nav History Bar
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(5.0, 5.0);

        if ui
            .add_enabled(state.recipe_history_pos > 0, egui::Button::new(t(lang, UiKey::NavBack)))
            .clicked()
            && state.recipe_nav_back()
        {
            audio.play(SoundCue::Click);
        }
        if ui
            .add_enabled(
                state.recipe_history_pos + 1 < state.recipe_history.len(),
                egui::Button::new(t(lang, UiKey::NavForward)),
            )
            .clicked()
            && state.recipe_nav_forward()
        {
            audio.play(SoundCue::Click);
        }

        ui.separator();

        if filter_chip(
            ui,
            palette,
            atlas,
            Some("knowledge_book"),
            t(lang, UiKey::FilterAll),
            state.recipe_station_filter.is_none(),
        )
        .clicked()
        {
            state.recipe_station_filter = None;
            state.recipe_catalog_page = 0;
            audio.play(SoundCue::Click);
        }

        for station in CraftingStation::all() {
            if filter_chip(
                ui,
                palette,
                atlas,
                Some(station.icon_id()),
                station.label(lang),
                state.recipe_station_filter == Some(station),
            )
            .clicked()
            {
                state.recipe_station_filter = Some(station);
                state.recipe_catalog_page = 0;
                audio.play(SoundCue::Click);
            }
        }
    });

    ui.add_space(4.0);

    // 2. Logical Category Filter Strip
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
        for cat in RecipeCategory::all() {
            if filter_chip(
                ui,
                palette,
                atlas,
                Some(cat.icon_id()),
                cat.label(lang),
                state.recipe_category_filter == cat,
            )
            .clicked()
            {
                state.recipe_category_filter = cat;
                state.recipe_catalog_page = 0;
                audio.play(SoundCue::Click);
            }
        }
    });

    ui.add_space(8.0);

    // 3. Build Filtered Catalog strictly matching version + query + station + category
    let q_lower = state.search_query.trim().to_lowercase();
    let filtered_items: Vec<&str> = recipes
        .catalog_items
        .iter()
        .filter(|id| recipes.item_active_in(id, versions, &active_ver))
        .filter(|id| {
            let item_recipes = recipes.recipes_for(id);
            let station_ok = state.recipe_station_filter.is_none_or(|st| {
                item_recipes.iter().any(|&idx| {
                    recipes.recipes[idx].is_active_in(versions, &active_ver)
                        && recipes.recipes[idx].station == st
                })
            });
            let cat_ok = state.recipe_category_filter == RecipeCategory::All
                || item_recipes.iter().any(|&idx| {
                    recipes.recipes[idx].is_active_in(versions, &active_ver)
                        && recipes.recipes[idx].category == state.recipe_category_filter
                });
            station_ok && cat_ok && recipes.matches_item_query(id, &q_lower)
        })
        .map(|s| s.as_str())
        .collect();

    // Keyboard shortcuts when typing isn't capturing focus (R = Recipe, U = Uses, A = Favorite)
    let want_kb = !ui.ctx().wants_keyboard_input();

    // 4. Adaptive Split: Side-by-side at >= 720px so narrow windows show both Crafting + JEI Grid!
    let total_w = ui.available_width();
    if total_w >= 720.0 {
        let left_w = (total_w * 0.43).clamp(290.0, 480.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(left_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    render_jei_inspector(ui, state, rcx, &active_ver, toasts, tag_step);
                },
            );
            ui.add_space(10.0);
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    render_jei_catalog_grid(
                        ui,
                        state,
                        rcx,
                        &active_ver,
                        &filtered_items,
                        toasts,
                        want_kb,
                    );
                },
            );
        });
    } else {
        egui::ScrollArea::vertical()
            .id_salt("recipes_compact_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                render_jei_inspector(ui, state, rcx, &active_ver, toasts, tag_step);
                ui.add_space(10.0);
                render_jei_catalog_grid(
                    ui,
                    state,
                    rcx,
                    &active_ver,
                    &filtered_items,
                    toasts,
                    want_kb,
                );
            });
    }

    // Repaint periodically if active recipe contains animated #tag slots
    let has_tags = recipes
        .recipes_for(&state.selected_recipe_item)
        .get(state.recipe_variant_idx)
        .and_then(|&idx| recipes.recipes.get(idx))
        .is_some_and(|r| r.slots.iter().any(|s| s.starts_with('#')));
    if has_tags && crate::ui::chrome::is_window_active(ui.ctx()) {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
    }
}

fn render_jei_inspector(
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
    let display_name = recipes.item_name(&target_id, lang);

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
                                recipes
                                    .uses_for(&target_id)
                                    .iter()
                                    .filter(|&&idx| recipes.recipes[idx].is_active_in(versions, active_ver))
                                    .count()
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
                                .filter(|&idx| recipes.recipes[idx].is_active_in(versions, active_ver))
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
                                            let vis = recipes.resolve_slot_icon(&tok, tag_step);
                                            let label = format!(
                                                "{}x {}",
                                                qty,
                                                recipes.item_name(&tok, lang)
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
                            let uses: Vec<usize> = recipes
                                .uses_for(&target_id)
                                .iter()
                                .copied()
                                .filter(|&idx| recipes.recipes[idx].is_active_in(versions, active_ver))
                                .collect();
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

                    // EMI Recursive Craft Tree & Raw Base Bill of Materials
                    if state.show_craft_tree {
                        ui.add_space(10.0);
                        ui.separator();
                        ui.horizontal_wrapped(|ui| {
                            ui.add(atlas.ui_image("oak_sapling", 16.0));
                            ui.label(
                                RichText::new(t(lang, UiKey::CraftTreeTitle))
                                    .size(12.5)
                                    .strong()
                                    .color(palette.accent_xp),
                            );
                            for q in [1u32, 8, 16, 64] {
                                if filter_chip(
                                    ui,
                                    palette,
                                    atlas,
                                    None,
                                    &format!("{}x", q),
                                    state.craft_tree_qty == q,
                                )
                                .clicked()
                                {
                                    state.craft_tree_qty = q;
                                    audio.play(SoundCue::Click);
                                }
                            }
                        });
                        ui.add_space(6.0);

                        let (tree, raw_totals) =
                            recipes.build_craft_tree(&target_id, state.craft_tree_qty);
                        let mut nav_target: Option<String> = None;
                        render_tree_node(ui, &tree, 0, rcx, lang, &mut nav_target);

                        if !raw_totals.is_empty() {
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new(t(lang, UiKey::RawMaterialsTitle))
                                    .size(12.0)
                                    .strong()
                                    .color(palette.accent_gold),
                            );
                            ui.horizontal_wrapped(|ui| {
                                for (mat_id, total_qty) in raw_totals {
                                    let lbl = format!(
                                        "{}: {}",
                                        recipes.item_name(&mat_id, lang),
                                        format_stack_breakdown(total_qty, 64, lang)
                                    );
                                    if filter_chip(
                                        ui,
                                        palette,
                                        atlas,
                                        Some(&mat_id),
                                        &lbl,
                                        false,
                                    )
                                    .clicked()
                                    {
                                        nav_target = Some(mat_id);
                                    }
                                }
                            });
                        }

                        if let Some(id) = nav_target {
                            state.open_recipe_item(&id, RecipeViewMode::RecipesFor);
                            audio.play(SoundCue::Click);
                        }
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
    tag_step: usize,
    lang: Language,
    pending_nav: &mut Option<(String, RecipeViewMode)>,
) {
    let RenderContext {
        recipes,
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
                            },
                            rcx,
                            lang,
                            pending_nav,
                        );
                    });
                }

                CraftingStation::Smithing => {
                    ui.horizontal_wrapped(|ui| {
                        for (i, fallback) in [
                            (0, ""),
                            (1, ""),
                            (2, ""),
                        ] {
                            let (sr, _) =
                                ui.allocate_exact_size(Vec2::splat(slot_sz), egui::Sense::hover());
                            let tok = entry.slots.get(i).map(|s| s.as_str()).unwrap_or(fallback);
                            handle_recipe_slot(
                                ui,
                                sr,
                                RecipeSlotSpec {
                                    token: tok,
                                    count: 1,
                                    tag_step,
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
                            },
                            rcx,
                            lang,
                            pending_nav,
                        );
                    });
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
                                    },
                                    rcx,
                                    lang,
                                    pending_nav,
                                );
                                ui.label(
                                    RichText::new(recipes.item_name(ing_tok, lang))
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
    } = spec;
    let RenderContext {
        recipes,
        palette,
        atlas,
        ..
    } = *rcx;
    let resolved = if token.is_empty() {
        String::new()
    } else {
        recipes.resolve_slot_icon(token, tag_step)
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
                let title = recipes.item_name(&resolved, lang);
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

fn render_tree_node(
    ui: &mut egui::Ui,
    node: &CraftTreeNode,
    depth: usize,
    rcx: &RenderContext<'_>,
    lang: Language,
    nav_target: &mut Option<String>,
) {
    let RenderContext {
        recipes,
        palette,
        atlas,
        ..
    } = *rcx;
    let indent = (depth as f32) * 16.0;
    ui.horizontal(|ui| {
        ui.add_space(indent);
        ui.add(atlas.ui_image(&node.item_id, 18.0));
        let lbl = if node.children.is_empty() {
            format!("{}x {}", node.total_needed, recipes.item_name(&node.item_id, lang))
        } else {
            format!(
                "{}x {} ({}x batch)",
                node.total_needed,
                recipes.item_name(&node.item_id, lang),
                node.batches
            )
        };
        if ui
            .add(
                egui::Button::new(
                    RichText::new(lbl).size(12.0).color(if node.children.is_empty() {
                        palette.text_primary
                    } else {
                        palette.accent_primary
                    }),
                )
                .frame(false),
            )
            .clicked()
        {
            *nav_target = Some(node.item_id.clone());
        }
    });
    for child in &node.children {
        render_tree_node(ui, child, depth + 1, rcx, lang, nav_target);
    }
}

fn render_jei_catalog_grid(
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
                            .primary_recipe_for(item_id)
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
                                let name = recipes.item_name(item_id, lang);
                                let uses_n = recipes.uses_for(item_id).len();
                                let rec_n = recipes.recipes_for(item_id).len();
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
                        render_alchemy_guide(ui, rcx, lang, &q_lower, &mut guide_nav);
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

fn alchemy_slot_button(
    ui: &mut egui::Ui,
    item_id: &str,
    size: f32,
    rcx: &RenderContext<'_>,
    lang: Language,
    pending_nav: &mut Option<String>,
) {
    let RenderContext {
        recipes,
        palette,
        atlas,
        ..
    } = *rcx;
    let (r, resp) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::click());
    paint_mc_slot(
        ui.painter(),
        r,
        palette,
        atlas,
        Some(item_id),
        SlotStyle::hovered(resp.hovered()),
    );
    let tip = format!(
        "{}\n{}",
        recipes.item_name(item_id, lang),
        t(lang, UiKey::LeftClickRecipeHint)
    );
    if resp.on_hover_text(tip).clicked() {
        *pending_nav = Some(item_id.to_string());
    }
}

fn render_alchemy_guide(
    ui: &mut egui::Ui,
    rcx: &RenderContext<'_>,
    lang: Language,
    q_lower: &str,
    pending_nav: &mut Option<String>,
) {
    let RenderContext {
        recipes, palette, ..
    } = *rcx;
    let pick_lang = |mx: &'static str, es: &'static str, en: &'static str| -> &'static str {
        match lang {
            Language::EsMx => mx,
            Language::EsEs => es,
            Language::EnUs => en,
        }
    };

    egui::Frame::new()
        .fill(palette.bg_card)
        .stroke(Stroke::new(1.5_f32, palette.accent_enchant))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                alchemy_slot_button(ui, "brewing_stand", 28.0, rcx, lang, pending_nav);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(t(lang, UiKey::AlchemyGuideTitle))
                            .size(14.0)
                            .strong()
                            .color(palette.accent_enchant),
                    );
                    ui.add(
                        egui::Label::new(
                            RichText::new(t(lang, UiKey::AlchemyGuideSubtitle))
                                .size(11.0)
                                .color(palette.text_muted),
                        )
                        .wrap(),
                    );
                });
            });

            ui.add_space(8.0);

            // Step 1: Awkward Base + Blaze Fuel
            egui::Frame::new()
                .fill(palette.bg_panel)
                .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                .corner_radius(CornerRadius::same(5))
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(t(lang, UiKey::AlchemyStep1Title))
                            .size(12.0)
                            .strong()
                            .color(palette.accent_gold),
                    );
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        alchemy_slot_button(ui, "glass_bottle", 28.0, rcx, lang, pending_nav);
                        ui.label(RichText::new("+").strong().color(palette.text_muted));
                        alchemy_slot_button(ui, "nether_wart", 28.0, rcx, lang, pending_nav);
                        ui.label(RichText::new("===>").monospace().strong().color(palette.accent_gold));
                        alchemy_slot_button(ui, "potion", 28.0, rcx, lang, pending_nav);
                        ui.add_space(8.0);
                        alchemy_slot_button(ui, "blaze_powder", 26.0, rcx, lang, pending_nav);
                        ui.label(
                            RichText::new(t(lang, UiKey::BrewingFuelHint))
                                .size(11.0)
                                .color(palette.accent_primary),
                        );
                    });
                    ui.add_space(3.0);
                    ui.add(
                        egui::Label::new(
                            RichText::new(t(lang, UiKey::AlchemyStep1Desc))
                                .size(11.0)
                                .color(palette.text_primary),
                        )
                        .wrap(),
                    );
                });

            ui.add_space(8.0);

            // Step 2: 5 Potion Modifiers (Time, Power II, Splash, Lingering, Invert)
            ui.label(
                RichText::new(t(lang, UiKey::AlchemyModifiersTitle))
                    .size(12.5)
                    .strong()
                    .color(palette.accent_gold),
            );
            ui.add_space(4.0);

            let modifiers = [
                (
                    "redstone",
                    palette.accent_danger,
                    "+ Tiempo / Duración Larga",
                    "+ Duración (Poción Larga)",
                    "+ Duration (Extended)",
                    "Duplica/extiende la duración (3:00 -> 8:00, 0:45 -> 1:30). Incompatible con Glowstone (Nivel II) y con pociones Trials 26.3.",
                    "Aumenta la duración (3:00 -> 8:00, 0:45 -> 1:30). Incompatible con Piedra Luminosa y con pociones de Pruebas 26.3.",
                    "Extends duration (3:00 -> 8:00, 0:45 -> 1:30). Mutually exclusive with Glowstone II and 26.3 Trial potions.",
                ),
                (
                    "glowstone_dust",
                    palette.accent_gold,
                    "+ Potencia (Nivel II)",
                    "+ Potencia (Nivel II)",
                    "+ Potency (Level II)",
                    "Sube el efecto al Nivel II (Fuerza II, Curación II, Velocidad II) y reduce duración a la mitad (3:00 -> 1:30). Incompatible con Redstone.",
                    "Mejora la potencia al Nivel II reduciendo duración a la mitad (3:00 -> 1:30). Incompatible con Redstone.",
                    "Amplifies potency to Level II (halves duration: 3:00 -> 1:30). Mutually exclusive with Redstone.",
                ),
                (
                    "gunpowder",
                    palette.accent_primary,
                    "Hacerla Lanzadora / Arrojadiza",
                    "Hacerla Arrojadiza (Lanzadora)",
                    "Make Splash (Throwable)",
                    "Transforma cualquier poción en Arrojadiza (Splash Potion) para lanzar a mano o desde dispensadores.",
                    "Convierte una poción bebible en Arrojadiza (Splash Potion) para lanzar o usar con dispensadores.",
                    "Turns any drinkable potion into a throwable Splash Potion for combat or dispenser farms.",
                ),
                (
                    "dragon_breath",
                    palette.accent_enchant,
                    "Hacerla Persistente + Flechas",
                    "Hacerla Persistente + Flechas",
                    "Make Lingering + Tipped Arrows",
                    "Aplícalo a una Lanzadora -> nube persistente en el suelo. Rodea 1 Persistente con 8 Flechas en mesa -> 8 Flechas con Efecto.",
                    "Aplícalo sobre una Arrojadiza -> nube persistente. Combina 1 Persistente + 8 Flechas en mesa -> 8 Flechas con Efecto.",
                    "Apply on a Splash Potion -> lingering area cloud. Surround 1 Lingering with 8 Arrows in Crafting Table -> 8 Tipped Arrows.",
                ),
                (
                    "fermented_spider_eye",
                    palette.accent_xp,
                    "Invertir / Corromper Efecto",
                    "Invertir / Corromper Efecto",
                    "Invert / Corrupt Effect",
                    "Visión Nocturna -> Invisibilidad | Velocidad o Salto -> Lentitud | Curación o Veneno -> Daño Instantáneo | Agua -> Debilidad.",
                    "Visión Nocturna -> Invisibilidad | Rapidez o Salto -> Lentitud | Curación o Veneno -> Daño | Agua -> Debilidad.",
                    "Night Vision -> Invisibility | Speed or Leaping -> Slowness | Healing or Poison -> Harming | Water -> Weakness.",
                ),
            ];

            let avail_w = ui.available_width();
            let mod_cols = if avail_w >= 600.0 { 2 } else { 1 };
            let mod_w = if mod_cols == 2 {
                (avail_w - 8.0) * 0.5
            } else {
                avail_w
            };
            for row in modifiers.chunks(mod_cols) {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    for &(icon, accent, t_mx, t_es, t_en, d_mx, d_es, d_en) in row {
                        ui.allocate_ui_with_layout(
                            Vec2::new(mod_w, 0.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                egui::Frame::new()
                                    .fill(palette.bg_panel)
                                    .stroke(Stroke::new(1.0_f32, accent))
                                    .corner_radius(CornerRadius::same(5))
                                    .inner_margin(Margin::same(7))
                                    .show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            alchemy_slot_button(
                                                ui,
                                                icon,
                                                28.0,
                                                rcx,
                                                lang,
                                                pending_nav,
                                            );
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    RichText::new(pick_lang(t_mx, t_es, t_en))
                                                        .size(12.0)
                                                        .strong()
                                                        .color(accent),
                                                );
                                                ui.add(
                                                    egui::Label::new(
                                                        RichText::new(pick_lang(d_mx, d_es, d_en))
                                                            .size(10.5)
                                                            .color(palette.text_primary),
                                                    )
                                                    .wrap(),
                                                );
                                            });
                                        });
                                    });
                            },
                        );
                    }
                });
                ui.add_space(5.0);
            }

            // Order Tip + Tipped Arrows Quick Visual
            egui::Frame::new()
                .fill(palette.slot_bg)
                .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(Margin::symmetric(8, 6))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_wrapped(|ui| {
                        alchemy_slot_button(
                            ui,
                            "lingering_potion",
                            24.0,
                            rcx,
                            lang,
                            pending_nav,
                        );
                        ui.label(RichText::new("+ 8x").size(11.0).color(palette.text_muted));
                        alchemy_slot_button(
                            ui,
                            "arrow",
                            24.0,
                            rcx,
                            lang,
                            pending_nav,
                        );
                        ui.label(RichText::new("=> 8x").monospace().strong().color(palette.accent_gold));
                        alchemy_slot_button(
                            ui,
                            "tipped_arrow",
                            24.0,
                            rcx,
                            lang,
                            pending_nav,
                        );
                    });
                    ui.add_space(3.0);
                    ui.add(
                        egui::Label::new(
                            RichText::new(t(lang, UiKey::AlchemyOrderTip))
                                .size(11.0)
                                .color(palette.accent_primary),
                        )
                        .wrap(),
                    );
                });

            ui.add_space(8.0);

            // Step 3: Complete Base Potions Table (19 rows covering all Java 26.3 potions)
            ui.label(
                RichText::new(t(lang, UiKey::AlchemyBaseRecipesTitle))
                    .size(12.5)
                    .strong()
                    .color(palette.accent_gold),
            );
            ui.add_space(4.0);

            let potion_rows = [
                ("blaze_powder", "Fuerza (+3 / +6 daño)", "Fuerza (+3 / +6 daño)", "Strength (+3 / +6 dmg)", "3:00", "8:00", "II (1:30)"),
                ("golden_carrot", "Visión Nocturna", "Visión Nocturna", "Night Vision", "3:00", "8:00", "—"),
                ("fermented_spider_eye", "Invisibilidad (en Visión Noct.)", "Invisibilidad (en Visión Noct.)", "Invisibility (on Night Vis.)", "3:00", "8:00", "—"),
                ("glistering_melon_slice", "Curación (+4 / +8 HP)", "Curación (+4 / +8 HP)", "Healing (+4 / +8 HP)", "Instant.", "—", "II (Instant.)"),
                ("ghast_tear", "Regeneración", "Regeneración", "Regeneration", "0:45", "1:30", "II (0:22)"),
                ("magma_cream", "Resistencia al Fuego", "Resistencia al Fuego", "Fire Resistance", "3:00", "8:00", "—"),
                ("sugar", "Velocidad (+20% / +40%)", "Rapidez (+20% / +40%)", "Swiftness (+20% / +40%)", "3:00", "8:00", "II (1:30)"),
                ("rabbit_foot", "Salto", "Salto", "Jump Boost (Leaping)", "3:00", "8:00", "II (1:30)"),
                ("pufferfish", "Respiración Acuática", "Respiración Acuática", "Water Breathing", "3:00", "8:00", "—"),
                ("phantom_membrane", "Caída Lenta (0 daño caída)", "Caída Lenta (0 daño caída)", "Slow Falling (0 fall dmg)", "1:30", "4:00", "—"),
                ("turtle_helmet", "Maestro Tortuga (Res. III-IV)", "Maestro Tortuga (Res. III-IV)", "Turtle Master (Res. III-IV)", "0:20", "0:40", "IV (0:20)"),
                ("spider_eye", "Veneno", "Veneno", "Poison", "0:45", "1:30", "II (0:21)"),
                ("fermented_spider_eye", "Lentitud (en Veloc./Salto)", "Lentitud (en Rapidez/Salto)", "Slowness (on Speed/Leap)", "1:30", "4:00", "IV (0:20)"),
                ("fermented_spider_eye", "Daño Inst. (en Curación/Ven.)", "Daño Inst. (en Curac./Ven.)", "Harming (on Heal/Poison)", "Instant.", "—", "II (Instant.)"),
                ("fermented_spider_eye", "Debilidad (directo en Agua)", "Debilidad (directo en Agua)", "Weakness (direct on Water)", "1:30", "4:00", "—"),
                ("breeze_rod", "Carga de Viento (26.3)", "Carga de Viento (26.3)", "Wind Charged (26.3)", "3:00", "—", "—"),
                ("cobweb", "Tejido / Telarañas (26.3)", "Tejido / Telarañas (26.3)", "Weaving (26.3)", "3:00", "—", "—"),
                ("slime_block", "Viscosidad / Slimes (26.3)", "Viscosidad / Limos (26.3)", "Oozing (26.3)", "3:00", "—", "—"),
                ("stone", "Infestación / Silverfish (26.3)", "Infestación / Lepismas (26.3)", "Infested (26.3)", "3:00", "—", "—"),
            ];

            let any_row_matches = !q_lower.is_empty()
                && potion_rows.iter().any(|&(ing, e_mx, e_es, e_en, ..)| {
                    ing.contains(q_lower)
                        || e_mx.to_lowercase().contains(q_lower)
                        || e_es.to_lowercase().contains(q_lower)
                        || e_en.to_lowercase().contains(q_lower)
                        || recipes.item_name(ing, lang).to_lowercase().contains(q_lower)
                });

            let row_cols = if avail_w >= 620.0 { 2 } else { 1 };
            let card_w = if row_cols == 2 {
                (avail_w - 8.0) * 0.5
            } else {
                avail_w
            };

            let visible_rows: Vec<_> = potion_rows
                .iter()
                .copied()
                .filter(|&(ing, e_mx, e_es, e_en, ..)| {
                    if !any_row_matches {
                        return true;
                    }
                    ing.contains(q_lower)
                        || e_mx.to_lowercase().contains(q_lower)
                        || e_es.to_lowercase().contains(q_lower)
                        || e_en.to_lowercase().contains(q_lower)
                        || recipes.item_name(ing, lang).to_lowercase().contains(q_lower)
                })
                .collect();

            for chunk in visible_rows.chunks(row_cols) {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    for &(ing, e_mx, e_es, e_en, d_base, d_red, d_glow) in chunk {
                        ui.allocate_ui_with_layout(
                            Vec2::new(card_w, 0.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                egui::Frame::new()
                                    .fill(palette.bg_panel)
                                    .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                                    .corner_radius(CornerRadius::same(4))
                                    .inner_margin(Margin::symmetric(7, 5))
                                    .show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            alchemy_slot_button(
                                                ui,
                                                ing,
                                                26.0,
                                                rcx,
                                                lang,
                                                pending_nav,
                                            );
                                            ui.vertical(|ui| {
                                                ui.add(
                                                    egui::Label::new(
                                                        RichText::new(pick_lang(e_mx, e_es, e_en))
                                                            .size(11.5)
                                                            .strong()
                                                            .color(palette.text_primary),
                                                    )
                                                    .wrap(),
                                                );
                                                ui.label(
                                                    RichText::new(format!(
                                                        "{}: {} | {}: {} | {}: {}",
                                                        t(lang, UiKey::AlchemyColBase),
                                                        d_base,
                                                        t(lang, UiKey::AlchemyColExtended),
                                                        d_red,
                                                        t(lang, UiKey::AlchemyColStrong),
                                                        d_glow
                                                    ))
                                                    .size(10.0)
                                                    .color(palette.text_muted),
                                                );
                                            });
                                        });
                                    });
                            },
                        );
                    }
                });
                ui.add_space(4.0);
            }
        });
}
