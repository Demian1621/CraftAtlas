use eframe::egui::{self, Align, CornerRadius, Layout, Margin, RichText, Stroke, Vec2};

use crate::assets::ItemAtlas;
use crate::audio::{AudioEngine, SoundCue};
use crate::data::litematic::format_mc_quantity;
use crate::data::models::Language;
use crate::i18n::{t, UiKey};
use crate::state::{AppState, AppTab, GearSubView, RecipeViewMode, RightRailTab};
use crate::ui::theme::{filter_chip, paint_mc_slot, McPalette, SlotStyle};
use crate::ui::RenderContext;

/// Renders collapsible Right Utility Rail: Favorites & Unified Shopping List.
pub fn render_favorites_rail(ctx: &egui::Context, state: &mut AppState, rcx: &RenderContext<'_>) {
    let RenderContext {
        versions,
        db,
        recipes,
        palette,
        atlas,
        audio,
    } = *rcx;
    if !state.prefs.favorites_open || ctx.screen_rect().width() < 860.0 {
        return;
    }
    let lang = state.prefs.language;
    let active_ver = versions
        .resolve_valid_id(&state.prefs.selected_version)
        .to_string();

    egui::SidePanel::right("mc_favorites_rail")
        .resizable(false)
        .default_width(234.0)
        .frame(
            egui::Frame::new()
                .fill(palette.bg_panel)
                .inner_margin(Margin::symmetric(10, 10))
                .stroke(Stroke::new(1.0_f32, palette.border_subtle)),
        )
        .show(ctx, |ui| {
            let pending_shop = state
                .prefs
                .shopping_list
                .iter()
                .filter(|i| !i.done)
                .count();
            ui.horizontal(|ui| {
                let fav_active = state.right_rail_tab == RightRailTab::Favorites;
                if filter_chip(
                    ui,
                    palette,
                    atlas,
                    Some("nether_star"),
                    t(lang, UiKey::RailTabFavorites),
                    fav_active,
                )
                .clicked()
                {
                    state.right_rail_tab = RightRailTab::Favorites;
                    audio.play(SoundCue::Click);
                }
                let shop_label = if pending_shop > 0 {
                    format!("{} ({})", t(lang, UiKey::RailTabShopping), pending_shop)
                } else {
                    t(lang, UiKey::RailTabShopping).to_string()
                };
                let shop_active = state.right_rail_tab == RightRailTab::ShoppingList;
                if filter_chip(
                    ui,
                    palette,
                    atlas,
                    Some("shulker_box"),
                    &shop_label,
                    shop_active,
                )
                .clicked()
                {
                    state.right_rail_tab = RightRailTab::ShoppingList;
                    audio.play(SoundCue::Click);
                }
            });
            ui.separator();

            if state.right_rail_tab == RightRailTab::ShoppingList {
                render_shopping_list_rail(ui, state, recipes, palette, atlas, audio, lang);
                return;
            }

            if state.prefs.favorite_items.is_empty()
                && state.prefs.favorite_gear.is_empty()
                && state.prefs.favorite_farms.is_empty()
            {
                ui.label(
                    RichText::new(t(lang, UiKey::FavoritesEmpty))
                        .size(11.5)
                        .color(palette.text_muted),
                );
                return;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Pinned JEI Items
                let items_clone = state.prefs.favorite_items.clone();
                if !items_clone.is_empty() {
                    ui.label(
                        RichText::new(t(lang, UiKey::TabRecipes))
                            .size(11.0)
                            .strong()
                            .color(palette.text_muted),
                    );
                    ui.add_space(4.0);
                    let mut remove_item: Option<String> = None;
                    for id in &items_clone {
                        if !recipes.item_active_in(id, versions, &active_ver) {
                            continue;
                        }
                        let name = recipes.item_name(id, lang);
                        ui.horizontal(|ui| {
                            let (slot_r, sresp) =
                                ui.allocate_exact_size(Vec2::splat(30.0), egui::Sense::click());
                            paint_mc_slot(
                                ui.painter(),
                                slot_r,
                                palette,
                                atlas,
                                Some(id),
                                SlotStyle {
                                    selected: state.active_tab == AppTab::Recipes
                                        && state.selected_recipe_item == *id,
                                    hovered: sresp.hovered(),
                                    ..Default::default()
                                },
                            );
                            if sresp.clicked() {
                                state.open_recipe_item(id, RecipeViewMode::RecipesFor);
                                audio.play(SoundCue::Click);
                            }
                            let lbl_resp = ui.add(
                                egui::Button::new(
                                    RichText::new(&name).size(11.5).color(palette.text_primary),
                                )
                                .frame(false)
                                .truncate(),
                            );
                            if lbl_resp.clicked() {
                                state.open_recipe_item(id, RecipeViewMode::RecipesFor);
                                audio.play(SoundCue::Click);
                            }
                            if ui.small_button("x").clicked() {
                                remove_item = Some(id.clone());
                            }
                        });
                    }
                    if let Some(rm) = remove_item {
                        state.prefs.toggle_favorite_item(&rm);
                        audio.play(SoundCue::Click);
                    }
                    ui.add_space(8.0);
                }

                // Pinned God Gear
                let gear_clone = state.prefs.favorite_gear.clone();
                if !gear_clone.is_empty() {
                    ui.label(
                        RichText::new(t(lang, UiKey::TabGear))
                            .size(11.0)
                            .strong()
                            .color(palette.text_muted),
                    );
                    ui.add_space(4.0);
                    let mut remove_gear: Option<String> = None;
                    for gid in &gear_clone {
                        if let Some(g) = db.gear.iter().find(|x| {
                            x.id == *gid
                                && versions.is_active(
                                    x.since_version.as_deref(),
                                    x.until_version.as_deref(),
                                    &active_ver,
                                )
                        }) {
                            let res = g.resolved_for(versions, &active_ver);
                            ui.horizontal(|ui| {
                                ui.add(atlas.ui_image(&g.icon, 20.0));
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new(res.name.get(lang))
                                                .size(11.5)
                                                .color(palette.accent_primary),
                                        )
                                        .frame(false)
                                        .truncate(),
                                    )
                                    .clicked()
                                {
                                    state.gear_view = GearSubView::Standard;
                                    state.selected_gear_id = g.id.clone();
                                    state.switch_tab(AppTab::Gear);
                                    audio.play(SoundCue::Click);
                                }
                                if ui.small_button("x").clicked() {
                                    remove_gear = Some(gid.clone());
                                }
                            });
                        }
                    }
                    if let Some(rm) = remove_gear {
                        state.prefs.toggle_favorite_gear(&rm);
                        audio.play(SoundCue::Click);
                    }
                    ui.add_space(8.0);
                }

                // Pinned Farms
                let farms_clone = state.prefs.favorite_farms.clone();
                if !farms_clone.is_empty() {
                    ui.label(
                        RichText::new(t(lang, UiKey::TabFarms))
                            .size(11.0)
                            .strong()
                            .color(palette.text_muted),
                    );
                    ui.add_space(4.0);
                    let mut remove_farm: Option<String> = None;
                    for fid in &farms_clone {
                        if let Some(f) = db.farms.iter().find(|x| {
                            x.id == *fid
                                && versions.is_active(
                                    x.since_version.as_deref(),
                                    x.until_version.as_deref(),
                                    &active_ver,
                                )
                        }) {
                            let res = f.resolved(&active_ver);
                            ui.horizontal(|ui| {
                                let icon = res.resolved_icon();
                                ui.add(atlas.ui_image(icon, 18.0));
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new(res.title.get(lang))
                                                .size(11.5)
                                                .color(palette.accent_xp),
                                        )
                                        .frame(false)
                                        .truncate(),
                                    )
                                    .clicked()
                                {
                                    state.selected_farm_id = f.id.clone();
                                    state.switch_tab(AppTab::Farms);
                                    audio.play(SoundCue::Click);
                                }
                                if ui.small_button("x").clicked() {
                                    remove_farm = Some(fid.clone());
                                }
                            });
                        }
                    }
                    if let Some(rm) = remove_farm {
                        state.prefs.toggle_favorite_farm(&rm);
                        audio.play(SoundCue::Click);
                    }
                }
            });
        });
}

fn render_shopping_list_rail(
    ui: &mut egui::Ui,
    state: &mut AppState,
    recipes: &crate::data::recipes::RecipeDatabase,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    lang: Language,
) {
    if state.prefs.shopping_list.is_empty() {
        ui.label(
            RichText::new(t(lang, UiKey::ShoppingEmptyHint))
                .size(11.5)
                .color(palette.text_muted),
        );
        return;
    }

    let total = state.prefs.shopping_list.len();
    let done_count = state.prefs.shopping_list.iter().filter(|i| i.done).count();
    let frac = if total > 0 {
        done_count as f32 / total as f32
    } else {
        0.0
    };

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "{}: {}/{}",
                t(lang, UiKey::ShoppingProgressLabel),
                done_count,
                total
            ))
            .size(11.0)
            .strong()
            .color(palette.accent_xp),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let mode_lbl = if state.prefs.shopping_shulker_mode {
                t(lang, UiKey::ShoppingShulkerModeBtn)
            } else {
                t(lang, UiKey::ShoppingExactModeBtn)
            };
            if filter_chip(
                ui,
                palette,
                atlas,
                None,
                mode_lbl,
                state.prefs.shopping_shulker_mode,
            )
            .clicked()
            {
                state.prefs.shopping_shulker_mode = !state.prefs.shopping_shulker_mode;
                state.prefs.save();
                audio.play(SoundCue::Click);
            }
        });
    });

    ui.add(
        egui::ProgressBar::new(frac)
            .desired_height(6.0)
            .fill(palette.accent_xp),
    );
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        if done_count > 0 && ui.small_button(t(lang, UiKey::ShoppingClearDoneBtn)).clicked() {
            state.prefs.clear_done_shopping_items();
            audio.play(SoundCue::Click);
        }
        if ui.small_button(t(lang, UiKey::ShoppingClearAllBtn)).clicked() {
            state.prefs.clear_shopping_list();
            audio.play(SoundCue::Click);
        }
    });
    ui.add_space(4.0);

    let shulker_mode = state.prefs.shopping_shulker_mode;
    let items_snapshot = state.prefs.shopping_list.clone();
    let mut toggle_id: Option<String> = None;
    let mut remove_id: Option<String> = None;

    egui::ScrollArea::vertical()
        .id_salt("shopping_rail_scroll")
        .show(ui, |ui| {
            for item in &items_snapshot {
                let display_name = if !item.label.is_empty() {
                    item.label.clone()
                } else {
                    recipes.item_name(&item.item_id, lang)
                };
                let qty_txt = format_mc_quantity(item.count, shulker_mode, lang);
                let mut checked = item.done;

                egui::Frame::new()
                    .fill(if item.done {
                        palette.bg_card.gamma_multiply(0.55)
                    } else {
                        palette.bg_card
                    })
                    .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                    .corner_radius(CornerRadius::same(5))
                    .inner_margin(Margin::symmetric(6, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut checked, "").changed() {
                                toggle_id = Some(item.item_id.clone());
                            }
                            ui.add(atlas.ui_image(&item.item_id, 18.0));
                            ui.vertical(|ui| {
                                let name_col = if item.done {
                                    palette.text_muted
                                } else {
                                    palette.text_primary
                                };
                                let mut rt = RichText::new(&display_name).size(11.0).color(name_col);
                                if item.done {
                                    rt = rt.strikethrough();
                                }
                                ui.label(rt);
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(&qty_txt)
                                            .size(10.5)
                                            .strong()
                                            .monospace()
                                            .color(if item.done {
                                                palette.text_muted
                                            } else {
                                                palette.accent_gold
                                            }),
                                    );
                                    if !item.source.is_empty() {
                                        ui.label(
                                            RichText::new(format!("· {}", item.source))
                                                .size(9.5)
                                                .color(palette.text_muted),
                                        );
                                    }
                                });
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.small_button("x").clicked() {
                                    remove_id = Some(item.item_id.clone());
                                }
                            });
                        });
                    });
                ui.add_space(3.0);
            }
        });

    if let Some(id) = toggle_id {
        state.prefs.toggle_shopping_done(&id);
        audio.play(SoundCue::Click);
    }
    if let Some(id) = remove_id {
        state.prefs.remove_shopping_item(&id);
        audio.play(SoundCue::Click);
    }
}
