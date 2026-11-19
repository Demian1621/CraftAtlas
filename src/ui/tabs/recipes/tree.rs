use eframe::egui::{self, RichText};

use crate::anim::ToastQueue;
use crate::audio::SoundCue;
use crate::data::models::{format_stack_breakdown, Language};
use crate::data::recipes::CraftTreeNode;
use crate::i18n::{t, UiKey};
use crate::state::{AppState, RecipeViewMode, RightRailTab};
use crate::ui::theme::filter_chip;
use crate::ui::RenderContext;

pub(super) fn render_craft_tree_section(
    ui: &mut egui::Ui,
    state: &mut AppState,
    rcx: &RenderContext<'_>,
    active_ver: &str,
    toasts: &mut ToastQueue,
    target_id: &str,
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
        recipes.build_craft_tree_in(target_id, state.craft_tree_qty, versions, active_ver);
    let mut nav_target: Option<String> = None;
    render_tree_node(ui, &tree, 0, rcx, lang, &mut nav_target);

    if !raw_totals.is_empty() {
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(t(lang, UiKey::RawMaterialsTitle))
                    .size(12.0)
                    .strong()
                    .color(palette.accent_gold),
            );
            if ui.small_button(t(lang, UiKey::ShoppingAddBtn)).clicked() {
                let batch: Vec<(String, String, u64)> = raw_totals
                    .iter()
                    .map(|(id, qty)| (id.clone(), recipes.item_name(id, lang), *qty as u64))
                    .collect();
                let source = format!("{}x {}", state.craft_tree_qty, recipes.item_name(target_id, lang));
                state.prefs.add_shopping_items(&batch, &source);
                state.prefs.favorites_open = true;
                state.right_rail_tab = RightRailTab::ShoppingList;
                audio.play(SoundCue::LevelUp);
                toasts.push(
                    "shulker_box",
                    t(lang, UiKey::ShoppingAddedToastTitle),
                    &source,
                    palette.accent_xp,
                );
            }
        });
        ui.horizontal_wrapped(|ui| {
            for (mat_id, total_qty) in raw_totals {
                let lbl = format!(
                    "{}: {}",
                    recipes.item_name(&mat_id, lang),
                    format_stack_breakdown(total_qty, 64, lang)
                );
                if filter_chip(ui, palette, atlas, Some(&mat_id), &lbl, false).clicked() {
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

