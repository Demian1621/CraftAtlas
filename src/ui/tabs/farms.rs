use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, Vec2};
use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::anim::ToastQueue;
use crate::assets::ItemAtlas;
use crate::audio::{AudioEngine, SoundCue};
use crate::data::litematic::format_mc_quantity;
use crate::data::models::{format_stack_breakdown, parse_material_qty, FarmItem, Language};
use crate::i18n::{t, UiKey};
use crate::state::{AppState, RecipeViewMode, RightRailTab};
use crate::ui::chrome::render_inline_tab_search;
use crate::ui::tabs::creators;
use crate::ui::theme::{adaptive_sidebar_width, filter_chip, paint_mc_slot, McPalette, SlotStyle};
use crate::ui::RenderContext;

pub fn render(
    ui: &mut egui::Ui,
    state: &mut AppState,
    rcx: &RenderContext<'_>,
    toasts: &mut ToastQueue,
) {
    let RenderContext {
        db,
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
    let q_lower = state.search_query.trim().to_lowercase();

    let resolved_all: Vec<Cow<'_, FarmItem>> = db
        .farms
        .iter()
        .filter(|f| versions.is_active(f.since_version.as_deref(), f.until_version.as_deref(), &active_ver))
        .map(|f| f.resolved(&active_ver))
        .collect();

    // Dynamic unique categories in database order (from version-active farms)
    let mut cats: BTreeMap<String, String> = BTreeMap::new();
    for f in &resolved_all {
        cats.entry(f.category_key.clone())
            .or_insert_with(|| f.category.get(lang).to_string());
    }

    let filtered: Vec<&FarmItem> = resolved_all
        .iter()
        .map(|f| f.as_ref())
        .filter(|f| state.farm_category_filter == "all" || f.category_key == state.farm_category_filter)
        .filter(|f| state.farm_tier_filter == "all" || f.tier_key == state.farm_tier_filter)
        .filter(|f| {
            if q_lower.is_empty() {
                return true;
            }
            f.title.matches(&q_lower)
                || f.desc.matches(&q_lower)
                || f.rate.matches(&q_lower)
                || f.materials.iter().any(|m| m.name.matches(&q_lower))
        })
        .collect();

    if !filtered.is_empty() && !filtered.iter().any(|f| f.id == state.selected_farm_id) {
        state.selected_farm_id = filtered[0].id.clone();
    }

    // Filters Row + YT/Bilibili Creators Directory + Native .litematic Reader Toggle
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(5.0, 5.0);
        let mut in_creators = state.farms_creators_view;
        let showing_creators = creators::render_creators_filter_controls(
            ui,
            state,
            &mut in_creators,
            UiKey::BackToFarmsBtn,
            "farms",
            palette,
            atlas,
            audio,
            lang,
        );
        if in_creators != state.farms_creators_view {
            state.farms_creators_view = in_creators;
            if in_creators {
                state.farms_litematic_view = false;
            }
        }

        if !showing_creators
            && filter_chip(
                ui,
                palette,
                atlas,
                Some("structure_block"),
                t(lang, UiKey::LitematicViewBtn),
                state.farms_litematic_view,
            )
            .clicked()
        {
            state.farms_litematic_view = !state.farms_litematic_view;
            audio.play(SoundCue::Click);
        }

        if !showing_creators && !state.farms_litematic_view {
            ui.separator();
            if filter_chip(
                ui,
                palette,
                atlas,
                Some("wheat"),
                t(lang, UiKey::FilterAll),
                state.farm_category_filter == "all",
            )
            .clicked()
            {
                state.farm_category_filter = "all".to_string();
                audio.play(SoundCue::Click);
            }
            for (ckey, clbl) in &cats {
                if filter_chip(ui, palette, atlas, None, clbl, state.farm_category_filter == *ckey)
                    .clicked()
                {
                    state.farm_category_filter = ckey.clone();
                    audio.play(SoundCue::Click);
                }
            }
            ui.separator();
            // Match exact DB tier keys (`easy` | `medium` | `hard`) to Early/Mid/End filters
            for (tkey, tlbl) in [
                ("all", t(lang, UiKey::FilterAll)),
                ("easy", t(lang, UiKey::TierEarly)),
                ("medium", t(lang, UiKey::TierMid)),
                ("hard", t(lang, UiKey::TierEnd)),
            ] {
                if filter_chip(ui, palette, atlas, None, tlbl, state.farm_tier_filter == tkey).clicked()
                {
                    state.farm_tier_filter = tkey.to_string();
                    audio.play(SoundCue::Click);
                }
            }
        }
    });

    ui.add_space(6.0);
    if !state.farms_litematic_view {
        render_inline_tab_search(ui, state, palette, atlas, audio);
    }

    if state.farms_creators_view {
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

    if state.farms_litematic_view {
        render_litematic_reader(ui, state, rcx, toasts);
        return;
    }

    let total_w = ui.available_width();
    if total_w >= 820.0 {
        let side_w = adaptive_sidebar_width(total_w);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(side_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    render_farm_sidebar(ui, state, &filtered, palette, atlas, audio, lang);
                },
            );
            ui.add_space(10.0);
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    if let Some(farm) = resolved_all.iter().find(|f| f.id == state.selected_farm_id) {
                        render_farm_detail(ui, state, farm, rcx, &active_ver, toasts);
                    }
                },
            );
        });
    } else {
        let cur_title = filtered
            .iter()
            .find(|f| f.id == state.selected_farm_id)
            .map(|f| f.title.get(lang))
            .unwrap_or("...");
        egui::ComboBox::from_id_salt("farm_compact_combo")
            .selected_text(cur_title)
            .width(total_w.min(400.0))
            .show_ui(ui, |ui| {
                for f in &filtered {
                    if ui
                        .selectable_value(&mut state.selected_farm_id, f.id.clone(), f.title.get(lang))
                        .changed()
                    {
                        audio.play(SoundCue::Click);
                    }
                }
            });
        ui.add_space(8.0);
        if let Some(farm) = resolved_all.iter().find(|f| f.id == state.selected_farm_id) {
            render_farm_detail(ui, state, farm, rcx, &active_ver, toasts);
        }
    }
}

fn render_farm_sidebar(
    ui: &mut egui::Ui,
    state: &mut AppState,
    filtered: &[&FarmItem],
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    lang: Language,
) {
    egui::ScrollArea::vertical()
        .id_salt("farm_list_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for farm in filtered {
                let selected = farm.id == state.selected_farm_id;
                let bg = if selected {
                    Color32::from_rgba_unmultiplied(
                        palette.accent_xp.r(),
                        palette.accent_xp.g(),
                        palette.accent_xp.b(),
                        28,
                    )
                } else {
                    palette.bg_card
                };
                let stroke = if selected {
                    Stroke::new(1.5_f32, palette.accent_xp)
                } else {
                    Stroke::new(1.0_f32, palette.border_subtle)
                };

                let icon = farm.resolved_icon();

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
                                Some(icon),
                                SlotStyle::selected(selected),
                            );
                            ui.vertical(|ui| {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(farm.title.get(lang))
                                            .size(12.0)
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
                                        "{} • {}",
                                        farm.category.get(lang),
                                        farm.tier.get(lang)
                                    ))
                                    .size(10.5)
                                    .color(palette.text_muted),
                                );
                            });
                        });
                    })
                    .response
                    .interact(egui::Sense::click());

                if resp.clicked() {
                    state.selected_farm_id = farm.id.clone();
                    audio.play(SoundCue::Click);
                }
                ui.add_space(4.0);
            }
        });
}

fn render_farm_detail(
    ui: &mut egui::Ui,
    state: &mut AppState,
    farm: &FarmItem,
    rcx: &RenderContext<'_>,
    active_ver: &str,
    toasts: &mut ToastQueue,
) {
    let RenderContext {
        db,
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
    let creators = &db.creators;
    let lang = state.prefs.language;
    let icon = farm.resolved_icon();

    egui::ScrollArea::vertical()
        .id_salt("farm_detail_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Header Card
            egui::Frame::new()
                .fill(palette.bg_panel)
                .stroke(Stroke::new(1.5_f32, palette.accent_xp))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_wrapped(|ui| {
                        ui.add(atlas.ui_image(icon, 36.0));
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(farm.title.get(lang))
                                    .size(18.0)
                                    .strong()
                                    .color(palette.accent_xp),
                            );
                            ui.horizontal_wrapped(|ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "[{}]  [{}]",
                                        farm.category.get(lang),
                                        farm.tier.get(lang)
                                    ))
                                    .size(11.5)
                                    .strong()
                                    .color(palette.accent_gold),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "{}: {}",
                                        t(lang, UiKey::FarmRateLabel),
                                        farm.rate.get(lang)
                                    ))
                                    .size(12.0)
                                    .strong()
                                    .color(palette.accent_primary),
                                );

                                let is_fav = state.prefs.favorite_farms.contains(&farm.id);
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
                                    let added = state.prefs.toggle_favorite_farm(&farm.id);
                                    audio.play(SoundCue::Click);
                                    if added {
                                        toasts.push(
                                            t(lang, UiKey::ToastFavoriteAddedTitle),
                                            farm.title.get(lang),
                                            icon,
                                            palette.accent_gold,
                                        );
                                    }
                                }
                            });
                        });
                    });

                    ui.add_space(6.0);
                    ui.add(
                        egui::Label::new(
                            RichText::new(farm.desc.get(lang))
                                .size(12.5)
                                .color(palette.text_primary),
                        )
                        .wrap(),
                    );
                });

            ui.add_space(10.0);

            // Interactive Material Checklist with Stacks/Shulker Mode
            egui::Frame::new()
                .fill(palette.bg_panel)
                .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_wrapped(|ui| {
                        ui.add(atlas.ui_image("chest", 18.0));
                        ui.label(
                            RichText::new(t(lang, UiKey::ChecklistHeader))
                                .size(13.0)
                                .strong()
                                .color(palette.accent_gold),
                        );

                        if filter_chip(
                            ui,
                            palette,
                            atlas,
                            Some("shulker_box"),
                            if state.farm_stacks_mode {
                                t(lang, UiKey::StacksMode)
                            } else {
                                t(lang, UiKey::RawCountMode)
                            },
                            state.farm_stacks_mode,
                        )
                        .clicked()
                        {
                            state.farm_stacks_mode = !state.farm_stacks_mode;
                            audio.play(SoundCue::Click);
                        }

                        if ui.button(t(lang, UiKey::ResetChecklist)).clicked() {
                            let prefix = format!("{}::", farm.id);
                            state.prefs.checked_materials.retain(|k| !k.starts_with(&prefix));
                            state.prefs.save();
                            audio.play(SoundCue::Click);
                        }

                        if ui.button(t(lang, UiKey::ShoppingAddBtn)).clicked() {
                            let batch: Vec<(String, String, u64)> = farm
                                .materials
                                .iter()
                                .map(|m| {
                                    let qty = parse_material_qty(&m.count).unwrap_or(1) as u64;
                                    (m.icon.clone(), m.name.get(lang).to_string(), qty.max(1))
                                })
                                .collect();
                            let src_title = farm.title.get(lang).to_string();
                            state.prefs.add_shopping_items(&batch, &src_title);
                            state.prefs.favorites_open = true;
                            state.right_rail_tab = RightRailTab::ShoppingList;
                            audio.play(SoundCue::LevelUp);
                            toasts.push(
                                "shulker_box",
                                t(lang, UiKey::ShoppingAddedToastTitle),
                                &src_title,
                                palette.accent_xp,
                            );
                        }
                    });
                    ui.add_space(6.0);

                    let mut nav_recipe: Option<String> = None;
                    for (idx, mat) in farm.materials.iter().enumerate() {
                        let ckey = format!("{}::{}", farm.id, idx);
                        let mut checked = state.prefs.checked_materials.contains(&ckey);
                        let mut toggled = false;

                        ui.horizontal(|ui| {
                            ui.spacing_mut().interact_size.y = 28.0;
                            if ui.checkbox(&mut checked, "").changed() {
                                toggled = true;
                            }

                            let (sr, sresp) =
                                ui.allocate_exact_size(Vec2::splat(28.0), egui::Sense::click());
                            paint_mc_slot(
                                ui.painter(),
                                sr,
                                palette,
                                atlas,
                                Some(&mat.icon),
                                SlotStyle::hovered(sresp.hovered()),
                            );
                            let sresp = sresp.on_hover_text(t(lang, UiKey::MaterialRowTooltip));
                            if sresp.clicked() {
                                nav_recipe = Some(mat.icon.clone());
                            }

                            let qty_label = if state.farm_stacks_mode {
                                parse_material_qty(&mat.count)
                                    .map(|q| format_stack_breakdown(q, 64, lang))
                                    .unwrap_or_else(|| mat.count.clone())
                            } else {
                                mat.count.clone()
                            };

                            let name_col = if checked {
                                palette.text_muted
                            } else {
                                palette.text_primary
                            };
                            let rt = RichText::new(format!("{} — {}", qty_label, mat.name.get(lang)))
                                .size(12.5)
                                .color(name_col);
                            let lbl = ui
                                .add(
                                    egui::Label::new(if checked { rt.strikethrough() } else { rt })
                                        .wrap()
                                        .sense(egui::Sense::click()),
                                )
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .on_hover_text(t(lang, UiKey::MaterialRowTooltip));
                            if lbl.clicked() {
                                checked = !checked;
                                toggled = true;
                            }
                        });

                        if toggled {
                            if checked {
                                state.prefs.checked_materials.insert(ckey);
                                audio.play(SoundCue::XpOrb);
                            } else {
                                state.prefs.checked_materials.remove(&ckey);
                                audio.play(SoundCue::Click);
                            }
                            state.prefs.save();
                        }
                    }
                    if let Some(id) = nav_recipe {
                        state.open_recipe_item(&id, RecipeViewMode::RecipesFor);
                        audio.play(SoundCue::Click);
                    }
                });

            ui.add_space(10.0);

            // Blueprint ASCII Diagram + Tips + Tutorial Metadata
            egui::Frame::new()
                .fill(palette.bg_card)
                .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(t(lang, UiKey::DiagramHeader))
                            .size(12.5)
                            .strong()
                            .color(palette.accent_primary),
                    );
                    ui.add_space(4.0);
                    egui::Frame::new()
                        .fill(palette.slot_bg)
                        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(Margin::same(8))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(
                                RichText::new(farm.diagram.get(lang))
                                    .monospace()
                                    .size(11.5)
                                    .color(palette.text_primary),
                            );
                        });

                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(t(lang, UiKey::TipsHeader))
                            .size(12.5)
                            .strong()
                            .color(palette.accent_gold),
                    );
                    ui.add(
                        egui::Label::new(
                            RichText::new(farm.tips.get(lang))
                                .size(12.0)
                                .color(palette.text_primary),
                        )
                        .wrap(),
                    );

                    ui.add_space(8.0);
                    let (query, search_url) = farm.tutorial_search(lang, active_ver);
                    creators::render_tutorial_search_bar(
                        ui,
                        &query,
                        &search_url,
                        active_ver,
                        Some(&farm.tutorial_author),
                        Some(&farm.schematic),
                        icon,
                        palette,
                        atlas,
                        audio,
                        toasts,
                        lang,
                    );
                });

            ui.add_space(10.0);

            // Detected .minecraft/schematics (.litematic / .schem)
            let mut open_schematic_path: Option<std::path::PathBuf> = None;
            egui::Frame::new()
                .fill(palette.bg_panel)
                .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(10))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(t(lang, UiKey::DetectedSchematics))
                                .size(12.0)
                                .strong()
                                .color(palette.accent_primary),
                        );
                        if ui.small_button(t(lang, UiKey::LitematicOpenFileBtn)).clicked() {
                            if let Some(p) = rfd::FileDialog::new()
                                .add_filter("Litematica", &["litematic"])
                                .pick_file()
                            {
                                open_schematic_path = Some(p);
                            }
                        }
                    });
                    if state.local_schematics.is_empty() {
                        ui.label(
                            RichText::new(t(lang, UiKey::NoSchematics))
                                .size(11.5)
                                .color(palette.text_muted),
                        );
                    } else {
                        for sc in state.local_schematics.iter().take(12) {
                            let is_litematic = sc
                                .file_name
                                .to_ascii_lowercase()
                                .ends_with(".litematic");
                            let lbl = format!(
                                "• {} ({} KB)",
                                sc.file_name,
                                (sc.size_bytes / 1024).max(1)
                            );
                            if is_litematic {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new(lbl)
                                                .monospace()
                                                .size(11.0)
                                                .color(palette.accent_xp),
                                        )
                                        .frame(false),
                                    )
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    open_schematic_path = Some(sc.full_path.clone());
                                }
                            } else {
                                ui.label(
                                    RichText::new(lbl)
                                        .monospace()
                                        .size(11.0)
                                        .color(palette.text_primary),
                                );
                            }
                        }
                    }
                });
            if let Some(path) = open_schematic_path {
                match state.load_litematic_path(&path) {
                    Ok(()) => {
                        audio.play(SoundCue::LevelUp);
                        let name = state
                            .active_litematic
                            .as_ref()
                            .map(|l| l.name.clone())
                            .unwrap_or_default();
                        toasts.push(
                            "structure_block",
                            t(lang, UiKey::LitematicLoadedToastTitle),
                            &name,
                            palette.accent_xp,
                        );
                    }
                    Err(err) => {
                        toasts.push(
                            "barrier",
                            t(lang, UiKey::LitematicErrorToastTitle),
                            &err,
                            palette.accent_danger,
                        );
                    }
                }
            }

            if !creators.is_empty() {
                ui.add_space(10.0);
                let mut open_creators = state.farms_creators_view;
                creators::render_recommended_creators_footer(
                    ui,
                    state,
                    &mut open_creators,
                    creators,
                    "farms",
                    &farm.id,
                    &farm.category_key,
                    Some(&farm.tutorial_author),
                    palette,
                    atlas,
                    audio,
                    toasts,
                    lang,
                );
                if open_creators {
                    state.farms_creators_view = true;
                }
            }
        });
}

fn render_litematic_reader(
    ui: &mut egui::Ui,
    state: &mut AppState,
    rcx: &RenderContext<'_>,
    toasts: &mut ToastQueue,
) {
    let RenderContext {
        recipes,
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
    let lang = state.prefs.language;

    let mut pick_path: Option<std::path::PathBuf> = None;

    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.add(atlas.ui_image("structure_block", 20.0));
                ui.label(
                    RichText::new(t(lang, UiKey::LitematicViewBtn))
                        .size(14.0)
                        .strong()
                        .color(palette.accent_primary),
                );
                if ui.button(t(lang, UiKey::LitematicOpenFileBtn)).clicked() {
                    audio.play(SoundCue::Click);
                    if let Some(p) = rfd::FileDialog::new()
                        .add_filter("Litematica", &["litematic"])
                        .pick_file()
                    {
                        pick_path = Some(p);
                    }
                }
                if ui.button(t(lang, UiKey::LitematicReloadBtn)).clicked() {
                    audio.play(SoundCue::Click);
                    state.reload_local_schematics();
                }
                if filter_chip(
                    ui,
                    palette,
                    atlas,
                    Some("shulker_box"),
                    t(lang, UiKey::LitematicShulkerToggle),
                    state.litematic_shulker_mode,
                )
                .clicked()
                {
                    state.litematic_shulker_mode = !state.litematic_shulker_mode;
                    audio.play(SoundCue::Click);
                }
            });
            ui.add_space(4.0);
            ui.label(
                RichText::new(t(lang, UiKey::LitematicDropHint))
                    .size(11.5)
                    .color(palette.text_muted),
            );
            if let Some(ref err) = state.litematic_error {
                ui.add_space(4.0);
                ui.label(RichText::new(err).size(11.5).color(palette.accent_danger));
            }
        });

    ui.add_space(8.0);

    // Quick clickable list of local .litematic files from .minecraft/schematics
    let local_litematics: Vec<_> = state
        .local_schematics
        .iter()
        .filter(|s| s.file_name.to_ascii_lowercase().ends_with(".litematic"))
        .cloned()
        .collect();
    if !local_litematics.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(5.0, 5.0);
            for sc in local_litematics.iter().take(12) {
                let selected = state
                    .active_litematic
                    .as_ref()
                    .map(|l| l.file_path == sc.full_path)
                    .unwrap_or(false);
                if filter_chip(
                    ui,
                    palette,
                    atlas,
                    Some("paper"),
                    &sc.file_name,
                    selected,
                )
                .clicked()
                {
                    pick_path = Some(sc.full_path.clone());
                }
            }
        });
        ui.add_space(6.0);
    }

    if let Some(p) = pick_path {
        match state.load_litematic_path(&p) {
            Ok(()) => {
                audio.play(SoundCue::LevelUp);
                let name = state
                    .active_litematic
                    .as_ref()
                    .map(|l| l.name.clone())
                    .unwrap_or_default();
                toasts.push(
                    "structure_block",
                    t(lang, UiKey::LitematicLoadedToastTitle),
                    &name,
                    palette.accent_xp,
                );
            }
            Err(err) => {
                toasts.push(
                    "barrier",
                    t(lang, UiKey::LitematicErrorToastTitle),
                    &err,
                    palette.accent_danger,
                );
            }
        }
    }

    let Some(parsed) = state.active_litematic.clone() else {
        return;
    };

    // Loaded Schematic Summary & Actions
    egui::Frame::new()
        .fill(palette.bg_card)
        .stroke(Stroke::new(1.0_f32, palette.accent_primary))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(&parsed.name)
                        .size(16.0)
                        .strong()
                        .color(palette.text_primary),
                );
                ui.label(
                    RichText::new(format!(
                        "· {}: {}  · {}: {}x{}x{} ({} reg)  · {}: {} ({} tipos)",
                        t(lang, UiKey::LitematicAuthorLabel),
                        parsed.author,
                        t(lang, UiKey::LitematicDimsLabel),
                        parsed.dims.0,
                        parsed.dims.1,
                        parsed.dims.2,
                        parsed.region_count,
                        t(lang, UiKey::LitematicBlocksLabel),
                        format_mc_quantity(parsed.total_blocks, false, lang),
                        parsed.materials.len(),
                    ))
                    .size(11.5)
                    .color(palette.text_muted),
                );
            });
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut state.litematic_filter)
                        .hint_text(t(lang, UiKey::LitematicFilterHint))
                        .desired_width(220.0),
                );
                if ui.button(t(lang, UiKey::LitematicSendToListBtn)).clicked() {
                    let batch: Vec<(String, String, u64)> = parsed
                        .materials
                        .iter()
                        .map(|m| (m.item_id.clone(), recipes.item_name(&m.item_id, lang), m.count))
                        .collect();
                    state.prefs.add_shopping_items(&batch, &parsed.name);
                    state.prefs.favorites_open = true;
                    state.right_rail_tab = RightRailTab::ShoppingList;
                    audio.play(SoundCue::LevelUp);
                    toasts.push(
                        "shulker_box",
                        t(lang, UiKey::ShoppingAddedToastTitle),
                        &parsed.name,
                        palette.accent_xp,
                    );
                }
            });
        });

    ui.add_space(6.0);

    let filt = state.litematic_filter.trim().to_lowercase();
    let shulker_mode = state.litematic_shulker_mode;
    let file_key = parsed.name.clone();
    let mut nav_recipe: Option<String> = None;

    egui::ScrollArea::vertical()
        .id_salt("litematic_materials_scroll")
        .show(ui, |ui| {
            for mat in &parsed.materials {
                let localized = recipes.item_name(&mat.item_id, lang);
                if !filt.is_empty()
                    && !localized.to_lowercase().contains(&filt)
                    && !mat.item_id.to_lowercase().contains(&filt)
                {
                    continue;
                }
                let ckey = format!("litematic::{}::{}", file_key, mat.item_id);
                let mut checked = state.prefs.checked_materials.contains(&ckey);
                let mut toggled = false;

                egui::Frame::new()
                    .fill(if checked {
                        palette.bg_card.gamma_multiply(0.55)
                    } else {
                        palette.bg_card
                    })
                    .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut checked, "").changed() {
                                toggled = true;
                            }
                            let (sr, sresp) =
                                ui.allocate_exact_size(Vec2::splat(26.0), egui::Sense::click());
                            paint_mc_slot(
                                ui.painter(),
                                sr,
                                palette,
                                atlas,
                                Some(&mat.item_id),
                                SlotStyle::hovered(sresp.hovered()),
                            );
                            if sresp.clicked() {
                                nav_recipe = Some(mat.item_id.clone());
                            }
                            let mut rt = RichText::new(&localized).size(12.5).color(if checked {
                                palette.text_muted
                            } else {
                                palette.text_primary
                            });
                            if checked {
                                rt = rt.strikethrough();
                            }
                            ui.label(rt);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        RichText::new(format_mc_quantity(
                                            mat.count,
                                            shulker_mode,
                                            lang,
                                        ))
                                        .monospace()
                                        .strong()
                                        .size(12.0)
                                        .color(if checked {
                                            palette.text_muted
                                        } else {
                                            palette.accent_gold
                                        }),
                                    );
                                },
                            );
                        });
                    });
                ui.add_space(3.0);

                if toggled {
                    if checked {
                        state.prefs.checked_materials.insert(ckey);
                        audio.play(SoundCue::XpOrb);
                    } else {
                        state.prefs.checked_materials.remove(&ckey);
                        audio.play(SoundCue::Click);
                    }
                    state.prefs.save();
                }
            }
        });

    if let Some(id) = nav_recipe {
        state.open_recipe_item(&id, RecipeViewMode::RecipesFor);
        audio.play(SoundCue::Click);
    }
}
