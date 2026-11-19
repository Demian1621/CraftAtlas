mod alchemy;
mod browser;
mod tree;

use eframe::egui::{self, Vec2};

use crate::anim::ToastQueue;
use crate::audio::SoundCue;
use crate::data::recipes::{CraftingStation, RecipeCategory};
use crate::i18n::{t, UiKey};
use crate::state::AppState;
use crate::ui::theme::filter_chip;
use crate::ui::RenderContext;

use browser::{render_jei_catalog_grid, render_jei_inspector};
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
                    recipes.is_recipe_active(idx, versions, &active_ver)
                        && recipes.recipes[idx].station == st
                })
            });
            let cat_ok = state.recipe_category_filter == RecipeCategory::All
                || item_recipes.iter().any(|&idx| {
                    recipes.is_recipe_active(idx, versions, &active_ver)
                        && recipes.recipes[idx].category == state.recipe_category_filter
                });
            station_ok && cat_ok && recipes.matches_item_query(id, &q_lower)
        })
        .map(|s| s.as_str())
        .collect();

    if !recipes.item_active_in(&state.selected_recipe_item, versions, &active_ver) {
        if let Some(&first) = filtered_items.first() {
            state.selected_recipe_item = first.to_string();
            state.recipe_variant_idx = 0;
        } else {
            state.selected_recipe_item = "crafting_table".to_string();
            state.recipe_variant_idx = 0;
        }
    }

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

