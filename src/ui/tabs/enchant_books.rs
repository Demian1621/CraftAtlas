use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, Vec2};

use crate::assets::ItemAtlas;
use crate::audio::{AudioEngine, SoundCue};
use crate::data::models::{EnchantBookEntry, Language};
use crate::i18n::{t, UiKey};
use crate::state::{AppState, GearSubView};
use crate::ui::chrome::render_inline_tab_search;
use crate::ui::theme::{adaptive_sidebar_width, filter_chip, paint_mc_slot, McPalette, SlotStyle};
use crate::ui::RenderContext;

/// Sub-view switch inside the Gear ("Equipamiento God y Yunque") tab.
pub fn render_view_switcher(
    ui: &mut egui::Ui,
    state: &mut AppState,
    rcx: &RenderContext<'_>,
    active_ver: &str,
    lang: Language,
) {
    let RenderContext {
        db,
        versions,
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
    let active_book_count = db
        .enchant_books
        .iter()
        .filter(|b| {
            versions.is_active(b.since_version.as_deref(), b.until_version.as_deref(), active_ver)
        })
        .count();

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 5.0);
        let gear_lbl = match lang {
            Language::EnUs => "God Gear & Anvil",
            _ => "Equipamiento God y Yunque",
        };
        if filter_chip(
            ui,
            palette,
            atlas,
            Some("anvil"),
            gear_lbl,
            state.gear_view == GearSubView::Standard,
        )
        .clicked()
            && state.gear_view != GearSubView::Standard
        {
            state.gear_view = GearSubView::Standard;
            audio.play(SoundCue::Click);
        }

        let books_lbl = format!("{} ({})", t(lang, UiKey::BooksViewBtn), active_book_count);
        if filter_chip(
            ui,
            palette,
            atlas,
            Some("enchanted_book"),
            &books_lbl,
            state.gear_view == GearSubView::Books,
        )
        .clicked()
            && state.gear_view != GearSubView::Books
        {
            state.gear_view = GearSubView::Books;
            audio.play(SoundCue::Click);
        }

        let god_lbl = match lang {
            Language::EnUs => "God Armor (Illegal / Glitches)",
            Language::EsEs => "Armadura God (Ilegal / Glitches)",
            Language::EsMx => "God Armor (Ilegal / Glitches)",
        };
        if filter_chip(
            ui,
            palette,
            atlas,
            Some("netherite_chestplate"),
            god_lbl,
            state.gear_view == GearSubView::IllegalGod,
        )
        .clicked()
            && state.gear_view != GearSubView::IllegalGod
        {
            state.gear_view = GearSubView::IllegalGod;
            audio.play(SoundCue::Click);
        }
    });
    ui.add_space(6.0);
}

/// Renders the complete Enchanted Books Encyclopedia sub-view inside the Gear tab.
pub fn render(
    ui: &mut egui::Ui,
    state: &mut AppState,
    rcx: &RenderContext<'_>,
    active_ver: &str,
    lang: Language,
) {
    let RenderContext {
        db,
        versions,
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
    let q_lower = state.search_query.trim().to_lowercase();

    render_book_filters(ui, state, palette, atlas, audio, lang);
    ui.add_space(6.0);

    let active_books: Vec<&EnchantBookEntry> = db
        .enchant_books
        .iter()
        .filter(|b| {
            versions.is_active(b.since_version.as_deref(), b.until_version.as_deref(), active_ver)
        })
        .collect();

    let filtered: Vec<&EnchantBookEntry> = active_books
        .iter()
        .copied()
        .filter(|b| b.matches_filter(&state.book_filter))
        .filter(|b| {
            if q_lower.is_empty() {
                return true;
            }
            b.name.matches(&q_lower)
                || b.id.to_lowercase().contains(&q_lower)
                || b.applies_to.matches(&q_lower)
                || b.incompatible_with.matches(&q_lower)
                || b.effect.matches(&q_lower)
                || b.best_method.matches(&q_lower)
                || b.sources.loot_chests.iter().any(|c| c.matches(&q_lower))
        })
        .collect();

    if !filtered.is_empty() && !filtered.iter().any(|b| b.id == state.selected_book_id) {
        state.selected_book_id = filtered[0].id.clone();
    }

    let selected_book = filtered
        .iter()
        .copied()
        .find(|b| b.id == state.selected_book_id)
        .or_else(|| active_books.iter().copied().find(|b| b.id == state.selected_book_id))
        .or_else(|| active_books.first().copied());

    let total_w = ui.available_width();
    if total_w >= 780.0 {
        let side_w = adaptive_sidebar_width(total_w);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(side_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    render_inline_tab_search(ui, state, palette, atlas, audio);
                    ui.add_space(6.0);
                    render_book_list(ui, state, &filtered, rcx, active_ver, lang);
                },
            );
            ui.add_space(10.0);
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    if let Some(book) = selected_book {
                        render_book_detail(ui, book, rcx, active_ver, lang);
                    }
                },
            );
        });
    } else {
        render_inline_tab_search(ui, state, palette, atlas, audio);
        ui.add_space(4.0);
        render_book_compact_picker(ui, state, &filtered, palette, atlas, audio, lang);
        ui.add_space(8.0);
        if let Some(book) = selected_book {
            render_book_detail(ui, book, rcx, active_ver, lang);
        }
    }
}

fn render_book_filters(
    ui: &mut egui::Ui,
    state: &mut AppState,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    lang: Language,
) {
    let filters = [
        ("all", Some("enchanted_book"), t(lang, UiKey::BookFilterAll)),
        (
            "no_villager",
            Some("warden_head"),
            t(lang, UiKey::BookFilterNoVillager),
        ),
        (
            "treasure",
            Some("ominous_trial_key"),
            t(lang, UiKey::BookFilterTreasure),
        ),
        (
            "armor",
            Some("netherite_chestplate"),
            t(lang, UiKey::BookFilterArmor),
        ),
        (
            "melee",
            Some("netherite_sword"),
            t(lang, UiKey::BookFilterMelee),
        ),
        ("ranged", Some("bow"), t(lang, UiKey::BookFilterRanged)),
        (
            "tool",
            Some("netherite_pickaxe"),
            t(lang, UiKey::BookFilterTool),
        ),
        (
            "universal",
            Some("anvil"),
            t(lang, UiKey::BookFilterUniversal),
        ),
    ];

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(5.0, 5.0);
        for (idx, (key, icon, lbl)) in filters.iter().enumerate() {
            if idx == 3 {
                ui.separator();
            }
            if filter_chip(ui, palette, atlas, *icon, lbl, state.book_filter == *key).clicked() {
                state.book_filter = (*key).to_string();
                audio.play(SoundCue::Click);
            }
        }
    });
}

fn render_book_list(
    ui: &mut egui::Ui,
    state: &mut AppState,
    filtered: &[&EnchantBookEntry],
    rcx: &RenderContext<'_>,
    active_ver: &str,
    lang: Language,
) {
    let RenderContext {
        versions,
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
    let time = ui.ctx().input(|i| i.time);
    egui::ScrollArea::vertical()
        .id_salt("enchant_books_list_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for book in filtered {
                let selected = book.id == state.selected_book_id;
                let bg = if selected {
                    Color32::from_rgba_unmultiplied(
                        palette.accent_enchant.r(),
                        palette.accent_enchant.g(),
                        palette.accent_enchant.b(),
                        34,
                    )
                } else {
                    palette.bg_card
                };
                let stroke = if selected {
                    Stroke::new(1.5_f32, palette.accent_enchant)
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
                                ui.allocate_exact_size(Vec2::splat(32.0), egui::Sense::hover());
                            paint_mc_slot(
                                ui.painter(),
                                slot_r,
                                palette,
                                atlas,
                                Some("enchanted_book"),
                                SlotStyle {
                                    selected,
                                    glint_time: if selected { Some(time) } else { None },
                                    ..Default::default()
                                },
                            );
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 2.0;
                                let title_color = if book.is_curse {
                                    Color32::from_rgb(240, 105, 105)
                                } else if selected {
                                    Color32::WHITE
                                } else {
                                    palette.text_primary
                                };
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(format!(
                                            "{} {}",
                                            book.name.get(lang),
                                            book.max_level
                                        ))
                                        .size(12.0)
                                        .strong()
                                        .color(title_color),
                                    )
                                    .wrap(),
                                );

                                let (badge_txt, badge_color) = if !book.sources.librarian_villager {
                                    let lbl = match lang {
                                        Language::EnUs => "NO Villager",
                                        _ => "NO Aldeanos",
                                    };
                                    (lbl, Color32::from_rgb(240, 96, 96))
                                } else if book.is_curse {
                                    let lbl = match lang {
                                        Language::EnUs => "Curse • Treasure",
                                        _ => "Maldición • Tesoro",
                                    };
                                    (lbl, Color32::from_rgb(235, 110, 110))
                                } else if book.is_treasure {
                                    let lbl = match lang {
                                        Language::EnUs => "Treasure • Villager OK",
                                        _ => "Tesoro • Aldeano OK",
                                    };
                                    (lbl, palette.accent_enchant)
                                } else {
                                    let lbl = match lang {
                                        Language::EnUs => "Table + Villager",
                                        _ => "Mesa + Aldeano",
                                    };
                                    (lbl, palette.accent_primary)
                                };
                                ui.label(RichText::new(badge_txt).size(10.0).color(badge_color));

                                let icons = book.applicable_item_icons_in(versions, active_ver);
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing = Vec2::new(3.0, 2.0);
                                    for (icon_id, es_tip, en_tip) in icons.iter().take(6) {
                                        let tip = if matches!(lang, Language::EnUs) {
                                            *en_tip
                                        } else {
                                            *es_tip
                                        };
                                        ui.add(atlas.ui_image(icon_id, 14.0))
                                            .on_hover_text(tip);
                                    }
                                    if icons.len() > 6 {
                                        ui.label(
                                            RichText::new(format!("+{}", icons.len() - 6))
                                                .size(9.5)
                                                .color(palette.text_muted),
                                        );
                                    }
                                });
                            });
                        });
                    });

                let resp = frame_resp.response.interact(egui::Sense::click());
                if resp.clicked() {
                    state.selected_book_id = book.id.clone();
                    audio.play(SoundCue::Click);
                }
                ui.add_space(4.0);
            }
        });
}

fn render_book_compact_picker(
    ui: &mut egui::Ui,
    state: &mut AppState,
    filtered: &[&EnchantBookEntry],
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    lang: Language,
) {
    egui::ScrollArea::horizontal()
        .id_salt("book_compact_picker_scroll")
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for book in filtered {
                    let selected = book.id == state.selected_book_id;
                    let lbl = format!("{} {}", book.name.get(lang), book.max_level);
                    if filter_chip(ui, palette, atlas, Some("enchanted_book"), &lbl, selected).clicked() {
                        state.selected_book_id = book.id.clone();
                        audio.play(SoundCue::Click);
                    }
                }
            });
        });
}

fn render_book_detail(
    ui: &mut egui::Ui,
    book: &EnchantBookEntry,
    rcx: &RenderContext<'_>,
    active_ver: &str,
    lang: Language,
) {
    let RenderContext {
        versions,
        palette,
        atlas,
        ..
    } = *rcx;
    let time = ui.ctx().input(|i| i.time);
    egui::ScrollArea::vertical()
        .id_salt("enchant_book_detail_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // 1. Hero Header Card
            egui::Frame::new()
                .fill(palette.bg_card)
                .stroke(Stroke::new(1.5_f32, palette.border_subtle))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(14))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let (slot_r, _) =
                            ui.allocate_exact_size(Vec2::splat(60.0), egui::Sense::hover());
                        paint_mc_slot(
                            ui.painter(),
                            slot_r,
                            palette,
                            atlas,
                            Some("enchanted_book"),
                            SlotStyle {
                                selected: true,
                                glint_time: Some(time),
                                ..Default::default()
                            },
                        );

                        ui.add_space(6.0);
                        ui.vertical(|ui| {
                            ui.horizontal_wrapped(|ui| {
                                let title_color = if book.is_curse {
                                    Color32::from_rgb(245, 95, 95)
                                } else {
                                    palette.accent_enchant
                                };
                                ui.label(
                                    RichText::new(format!(
                                        "{} {}",
                                        book.name.get(lang),
                                        book.max_level
                                    ))
                                    .size(20.0)
                                    .strong()
                                    .color(title_color),
                                );
                            });
                            ui.label(
                                RichText::new(format!("minecraft:{}", book.id))
                                    .monospace()
                                    .size(10.5)
                                    .color(palette.text_muted),
                            );
                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                if !book.sources.librarian_villager {
                                    render_alert_pill(
                                        ui,
                                        t(lang, UiKey::BookBadgeNoVillager),
                                        Color32::from_rgb(235, 76, 76),
                                    );
                                }
                                if book.is_curse {
                                    render_alert_pill(
                                        ui,
                                        t(lang, UiKey::BookBadgeCurse),
                                        Color32::from_rgb(220, 68, 92),
                                    );
                                } else if book.is_treasure {
                                    render_alert_pill(
                                        ui,
                                        t(lang, UiKey::BookBadgeTreasure),
                                        palette.accent_gold,
                                    );
                                }
                            });
                        });
                    });

                    ui.add_space(10.0);

                    // Metadata summary grid
                    egui::Frame::new()
                        .fill(palette.bg_panel)
                        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                        .corner_radius(CornerRadius::same(5))
                        .inner_margin(Margin::same(10))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = Vec2::new(18.0, 6.0);
                                ui.label(
                                    RichText::new(format!(
                                        "{}: {}",
                                        t(lang, UiKey::BookMaxLevelLabel),
                                        book.max_level
                                    ))
                                    .size(12.0)
                                    .strong()
                                    .color(palette.accent_primary),
                                );
                                let xp_note = match lang {
                                    Language::EnUs => format!(
                                        "{}: {}x ({} lvls from book)",
                                        t(lang, UiKey::BookAnvilWeightLabel),
                                        book.anvil_multiplier,
                                        book.anvil_max_level_cost()
                                    ),
                                    _ => format!(
                                        "{}: {}x ({} niv. con libro max)",
                                        t(lang, UiKey::BookAnvilWeightLabel),
                                        book.anvil_multiplier,
                                        book.anvil_max_level_cost()
                                    ),
                                };
                                ui.label(
                                    RichText::new(xp_note)
                                        .size(12.0)
                                        .strong()
                                        .color(palette.accent_xp),
                                );
                            });
                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.label(
                                    RichText::new(format!("{}:", t(lang, UiKey::BookAppliesToLabel)))
                                        .size(11.5)
                                        .strong()
                                        .color(palette.text_muted),
                                );
                                ui.label(
                                    RichText::new(book.applies_to.get(lang))
                                        .size(11.5)
                                        .color(palette.text_primary),
                                );
                            });
                            ui.add_space(5.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = Vec2::new(5.0, 5.0);
                                for (icon_id, es_tip, en_tip) in
                                    book.applicable_item_icons_in(versions, active_ver)
                                {
                                    let tip = if matches!(lang, Language::EnUs) {
                                        en_tip
                                    } else {
                                        es_tip
                                    };
                                    egui::Frame::new()
                                        .fill(palette.slot_bg)
                                        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                                        .corner_radius(CornerRadius::same(4))
                                        .inner_margin(Margin::same(3))
                                        .show(ui, |ui| {
                                            ui.add(atlas.ui_image(icon_id, 22.0))
                                                .on_hover_text(tip);
                                        });
                                }
                            });
                            ui.add_space(5.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.label(
                                    RichText::new(format!("{}:", t(lang, UiKey::BookIncompatLabel)))
                                        .size(11.5)
                                        .strong()
                                        .color(palette.text_muted),
                                );
                                ui.label(
                                    RichText::new(book.incompatible_with.get(lang))
                                        .size(11.5)
                                        .color(Color32::from_rgb(242, 168, 120)),
                                );
                            });
                        });
                });

            ui.add_space(10.0);

            // 2. Effect Card
            let frame = egui::Frame::new()
                .fill(palette.bg_card)
                .stroke(Stroke::new(1.5_f32, palette.border_subtle))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(12));
            frame.show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(t(lang, UiKey::BookEffectLabel))
                        .strong()
                        .size(13.0)
                        .color(palette.accent_gold),
                );
                ui.add_space(4.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(book.effect.get(lang))
                            .size(12.5)
                            .color(palette.text_primary),
                    )
                    .wrap(),
                );
            });

            ui.add_space(10.0);

            // 3. Acquisition Matrix + Chest Locations
            frame.show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(t(lang, UiKey::BookSourcesLabel))
                        .strong()
                        .size(13.0)
                        .color(palette.accent_gold),
                );
                ui.add_space(6.0);

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                    render_source_badge(
                        ui,
                        atlas,
                        "enchanting_table",
                        t(lang, UiKey::BookSrcTable),
                        book.sources.enchanting_table,
                        palette.accent_enchant,
                        palette,
                        lang,
                    );
                    render_source_badge(
                        ui,
                        atlas,
                        "lectern",
                        t(lang, UiKey::BookSrcVillager),
                        book.sources.librarian_villager,
                        palette.accent_primary,
                        palette,
                        lang,
                    );
                    render_source_badge(
                        ui,
                        atlas,
                        "fishing_rod",
                        t(lang, UiKey::BookSrcFishing),
                        book.sources.fishing,
                        Color32::from_rgb(92, 188, 240),
                        palette,
                        lang,
                    );
                    render_source_badge(
                        ui,
                        atlas,
                        "gold_ingot",
                        t(lang, UiKey::BookSrcBarter),
                        book.sources.piglin_bartering,
                        palette.accent_gold,
                        palette,
                        lang,
                    );
                });

                if !book.sources.loot_chests.is_empty() {
                    ui.add_space(8.0);
                    for chest in &book.sources.loot_chests {
                        ui.horizontal_wrapped(|ui| {
                            ui.add(atlas.ui_image("chest", 16.0));
                            ui.add(
                                egui::Label::new(
                                    RichText::new(chest.get(lang))
                                        .size(11.5)
                                        .color(palette.text_primary),
                                )
                                .wrap(),
                            );
                        });
                    }
                }
            });

            ui.add_space(10.0);

            // 4. Optimal Strategy Card
            let strat_accent = if !book.sources.librarian_villager {
                Color32::from_rgb(240, 108, 92)
            } else {
                palette.accent_primary
            };
            egui::Frame::new()
                .fill(Color32::from_rgba_unmultiplied(
                    strat_accent.r(),
                    strat_accent.g(),
                    strat_accent.b(),
                    24,
                ))
                .stroke(Stroke::new(1.5_f32, strat_accent))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.add(atlas.ui_image(
                            if !book.sources.librarian_villager {
                                "enchanted_book"
                            } else {
                                "emerald"
                            },
                            18.0,
                        ));
                        ui.label(
                            RichText::new(t(lang, UiKey::BookBestMethodLabel))
                                .strong()
                                .size(13.5)
                                .color(strat_accent),
                        );
                    });
                    ui.add_space(5.0);
                    ui.add(
                        egui::Label::new(
                            RichText::new(book.best_method.get(lang))
                                .size(12.5)
                                .color(Color32::WHITE),
                        )
                        .wrap(),
                    );
                });
        });
}

fn render_alert_pill(ui: &mut egui::Ui, label: &str, color: Color32) {
    egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 36))
        .stroke(Stroke::new(1.0_f32, color))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::symmetric(7, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(label).size(10.5).strong().color(color));
        });
}

#[allow(clippy::too_many_arguments)]
fn render_source_badge(
    ui: &mut egui::Ui,
    atlas: &ItemAtlas,
    icon: &str,
    label: &str,
    available: bool,
    active_color: Color32,
    palette: McPalette,
    lang: Language,
) {
    let border = if available {
        active_color
    } else {
        Color32::from_rgb(135, 72, 72)
    };
    let fill = if available {
        Color32::from_rgba_unmultiplied(active_color.r(), active_color.g(), active_color.b(), 26)
    } else {
        Color32::from_rgba_unmultiplied(80, 42, 42, 34)
    };
    let status = match (available, lang) {
        (true, Language::EnUs) => "YES",
        (true, _) => "SI",
        (false, _) => "NO",
    };

    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.2_f32, border))
        .corner_radius(CornerRadius::same(5))
        .inner_margin(Margin::symmetric(8, 5))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(atlas.ui_image(icon, 16.0));
                ui.label(
                    RichText::new(label)
                        .size(11.0)
                        .color(if available { palette.text_primary } else { palette.text_muted }),
                );
                ui.label(
                    RichText::new(status)
                        .size(10.5)
                        .strong()
                        .color(if available { active_color } else { Color32::from_rgb(235, 95, 95) }),
                );
            });
        });
}
