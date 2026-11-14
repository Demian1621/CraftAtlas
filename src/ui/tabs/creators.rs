use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, Vec2};

use crate::anim::ToastQueue;
use crate::assets::ItemAtlas;
use crate::audio::{AudioEngine, SoundCue};
use crate::data::models::{Language, TechCreator};
use crate::i18n::{t, UiKey};
use crate::state::AppState;
use crate::ui::theme::{filter_chip, paint_mc_slot, McPalette, SlotStyle};

pub fn creator_platform_color(platform_key: &str) -> Color32 {
    if platform_key == "bilibili" {
        Color32::from_rgb(0, 161, 214)
    } else {
        Color32::from_rgb(235, 65, 65)
    }
}

pub fn creator_tag_color(tag_key: &str, palette: McPalette) -> Color32 {
    match tag_key {
        "farms" | "survival" => palette.accent_primary,
        "dupes" | "exploits" | "anarchy" => Color32::from_rgb(235, 90, 90),
        "redstone" | "storage" => Color32::from_rgb(230, 68, 56),
        "mechanics" | "physics" | "technical" => Color32::from_rgb(56, 189, 248),
        "tutorials" | "beginner" | "tools" => palette.accent_enchant,
        "bilibili" | "chinese_tech" => Color32::from_rgb(0, 175, 225),
        _ => palette.accent_gold,
    }
}

fn render_tag_pill(ui: &mut egui::Ui, label: &str, color: Color32, small: bool) {
    egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 34))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 170)))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::symmetric(if small { 4 } else { 6 }, if small { 1 } else { 2 }))
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(label)
                        .size(if small { 9.5 } else { 10.5 })
                        .strong()
                        .color(color),
                )
                .wrap_mode(egui::TextWrapMode::Extend),
            );
        });
}

pub fn render_creator_action_buttons(
    ui: &mut egui::Ui,
    creator: &TechCreator,
    palette: McPalette,
    audio: &AudioEngine,
    toasts: &mut ToastQueue,
    lang: Language,
    compact: bool,
) {
    let accent = creator_platform_color(&creator.platform_key);
    let open_btn = egui::Button::new(
        RichText::new(t(lang, UiKey::OpenChannelBtn))
            .size(if compact { 10.5 } else { 11.5 })
            .strong()
            .color(Color32::WHITE),
    )
    .fill(Color32::from_rgba_unmultiplied(
        accent.r(),
        accent.g(),
        accent.b(),
        190,
    ))
    .stroke(Stroke::new(1.0_f32, accent));

    if ui.add(open_btn).clicked() {
        if crate::updater::is_safe_https_url(&creator.url) {
            ui.ctx().open_url(egui::OpenUrl::new_tab(&creator.url));
        }
        audio.play(SoundCue::Click);
    }

    let copy_btn = egui::Button::new(
        RichText::new(t(lang, UiKey::CopyLinkBtn))
            .size(if compact { 10.5 } else { 11.5 })
            .color(palette.text_primary),
    )
    .fill(palette.slot_bg)
    .stroke(Stroke::new(1.0_f32, palette.border_subtle));

    if ui.add(copy_btn).clicked() {
        ui.ctx().copy_text(creator.url.clone());
        audio.play(SoundCue::Toast);
        toasts.push(
            t(lang, UiKey::CopiedLinkTitle),
            format!("{} ({})", creator.name, creator.platform_label),
            &creator.icon,
            accent,
        );
    }
}

/// Renders the filter strip controls (Back + Specialty filter + Region filter) shared by Farms & Dupes.
#[allow(clippy::too_many_arguments)]
pub fn render_creators_filter_controls(
    ui: &mut egui::Ui,
    state: &mut AppState,
    in_creators_view: &mut bool,
    back_key: UiKey,
    default_specialty: &str,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    lang: Language,
) -> bool {
    if filter_chip(
        ui,
        palette,
        atlas,
        Some("knowledge_book"),
        if *in_creators_view {
            t(lang, back_key)
        } else {
            t(lang, UiKey::CreatorsViewBtn)
        },
        *in_creators_view,
    )
    .clicked()
    {
        *in_creators_view = !*in_creators_view;
        if *in_creators_view {
            state.creators_specialty_filter = default_specialty.to_string();
        }
        audio.play(SoundCue::Click);
    }

    if !*in_creators_view {
        return false;
    }

    ui.separator();
    for (skey, slbl) in [
        ("all", t(lang, UiKey::CreatorsSpecialtyAll)),
        ("farms", t(lang, UiKey::CreatorsSpecialtyFarms)),
        ("dupes", t(lang, UiKey::CreatorsSpecialtyDupes)),
        ("redstone", t(lang, UiKey::CreatorsSpecialtyRedstone)),
    ] {
        if filter_chip(
            ui,
            palette,
            atlas,
            None,
            slbl,
            state.creators_specialty_filter == skey,
        )
        .clicked()
        {
            state.creators_specialty_filter = skey.to_string();
            audio.play(SoundCue::Click);
        }
    }

    ui.separator();
    for (rkey, rlbl) in [
        ("all", t(lang, UiKey::CreatorFilterAll)),
        ("es", t(lang, UiKey::CreatorFilterEs)),
        ("en", t(lang, UiKey::CreatorFilterEn)),
        ("zh", t(lang, UiKey::CreatorFilterZh)),
    ] {
        if filter_chip(
            ui,
            palette,
            atlas,
            None,
            rlbl,
            state.creators_region_filter == rkey,
        )
        .clicked()
        {
            state.creators_region_filter = rkey.to_string();
            audio.play(SoundCue::Click);
        }
    }

    true
}

/// Renders a shared language- & version-aware YouTube tutorial search bar with Open + Copy actions.
#[allow(clippy::too_many_arguments)]
pub fn render_tutorial_search_bar(
    ui: &mut egui::Ui,
    query: &str,
    search_url: &str,
    active_ver: &str,
    author_badge: Option<&str>,
    schematic_badge: Option<&str>,
    toast_icon: &str,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    toasts: &mut ToastQueue,
    lang: Language,
) {
    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.0_f32, palette.accent_primary))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.add(atlas.ui_image("knowledge_book", 18.0));
                ui.label(
                    RichText::new(t(lang, UiKey::TutorialVideo))
                        .size(12.0)
                        .strong()
                        .color(palette.accent_gold),
                );
                if let Some(author) = author_badge {
                    if !author.is_empty() {
                        ui.label(
                            RichText::new(format!("• {author}"))
                                .size(11.5)
                                .color(palette.text_muted),
                        );
                    }
                }
                if let Some(schem) = schematic_badge {
                    if !schem.is_empty() {
                        ui.label(
                            RichText::new(format!("[{schem}]"))
                                .monospace()
                                .size(10.5)
                                .color(palette.accent_primary),
                        );
                    }
                }
            });

            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                egui::Frame::new()
                    .fill(palette.slot_bg)
                    .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(query)
                                .monospace()
                                .size(11.5)
                                .color(palette.text_primary),
                        );
                    });

                let search_btn = egui::Button::new(
                    RichText::new(format!("{} ({active_ver})", t(lang, UiKey::SearchTutorialBtn)))
                        .size(11.5)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgba_unmultiplied(
                    palette.accent_primary.r(),
                    palette.accent_primary.g(),
                    palette.accent_primary.b(),
                    205,
                ))
                .stroke(Stroke::new(1.0_f32, palette.accent_primary));

                if ui.add(search_btn).clicked() {
                    if crate::updater::is_safe_https_url(search_url) {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(search_url));
                    }
                    audio.play(SoundCue::Click);
                }

                let copy_btn = egui::Button::new(
                    RichText::new(t(lang, UiKey::CopyQueryBtn))
                        .size(11.5)
                        .color(palette.text_primary),
                )
                .fill(palette.bg_card)
                .stroke(Stroke::new(1.0_f32, palette.border_subtle));

                if ui.add(copy_btn).clicked() {
                    ui.ctx().copy_text(query.to_string());
                    audio.play(SoundCue::Toast);
                    toasts.push(
                        t(lang, UiKey::CopiedQueryTitle),
                        query,
                        toast_icon,
                        palette.accent_primary,
                    );
                }
            });
        });
}

fn stable_pair_hash(a: &str, b: &str) -> usize {
    a.bytes()
        .chain(b.bytes())
        .fold(0usize, |acc, byte| acc.wrapping_mul(33).wrapping_add(byte as usize))
}

fn score_creator_for_context(
    creator: &TechCreator,
    focus_key: &str,
    item_id: &str,
    category_key: &str,
    author_hint: Option<&str>,
) -> usize {
    let mut score = if creator.has_focus(focus_key) { 10 } else { 0 };
    if creator.recommended_for.iter().any(|r| r == item_id) {
        score += 120;
    }
    if creator.recommended_for.iter().any(|r| r == category_key) {
        score += 35;
    }
    if let Some(hint) = author_hint {
        let hint_lower = hint.to_lowercase();
        let name_lower = creator.name.to_lowercase();
        let id_lower = creator.id.to_lowercase();
        if (!name_lower.is_empty() && hint_lower.contains(&name_lower))
            || (!id_lower.is_empty() && hint_lower.contains(&id_lower))
        {
            score += 90;
        }
    }
    score
}

/// Selects up to 3 relevant specialists for the active tab focus (`"farms"` or `"dupes"`),
/// balancing ES / EN / ZH channels instead of picking unrelated creators.
pub fn select_recommended_creators<'a>(
    creators: &'a [TechCreator],
    focus_key: &str,
    item_id: &str,
    category_key: &str,
    author_hint: Option<&str>,
) -> Vec<&'a TechCreator> {
    let pool: Vec<&TechCreator> = creators
        .iter()
        .filter(|c| c.has_focus(focus_key))
        .collect();
    if pool.is_empty() {
        return Vec::new();
    }

    let mut ranked: Vec<(&TechCreator, usize)> = pool
        .iter()
        .map(|&c| {
            let s = score_creator_for_context(c, focus_key, item_id, category_key, author_hint);
            (c, s)
        })
        .collect();

    ranked.sort_by(|(ca, sa), (cb, sb)| {
        sb.cmp(sa)
            .then_with(|| stable_pair_hash(item_id, &ca.id).cmp(&stable_pair_hash(item_id, &cb.id)))
    });

    let mut selected: Vec<&TechCreator> = Vec::with_capacity(3);

    // Always lead with a direct item/author match when one exists
    if let Some(&(best, best_score)) = ranked.first() {
        if best_score >= 45 {
            selected.push(best);
        }
    }

    // Ensure ES / EN / ZH coverage using highest-scoring creators within each region
    for region in ["es", "en", "zh"] {
        if selected.len() >= 3 {
            break;
        }
        if selected.iter().any(|c| c.region_key == region) {
            continue;
        }
        if let Some(&(pick, _)) = ranked.iter().find(|(c, _)| c.region_key == region) {
            if !selected.iter().any(|existing| existing.id == pick.id) {
                selected.push(pick);
            }
        }
    }

    // Fill remaining slots up to 3 from top-ranked specialists if fewer than 3 regions exist
    for &(pick, _) in &ranked {
        if selected.len() >= 3 {
            break;
        }
        if !selected.iter().any(|existing| existing.id == pick.id) {
            selected.push(pick);
        }
    }

    selected
}

#[allow(clippy::too_many_arguments)]
pub fn render_recommended_creators_footer(
    ui: &mut egui::Ui,
    state: &mut AppState,
    open_creators_view: &mut bool,
    creators: &[TechCreator],
    focus_key: &str,
    item_id: &str,
    category_key: &str,
    author_hint: Option<&str>,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    toasts: &mut ToastQueue,
    lang: Language,
) {
    let featured = select_recommended_creators(creators, focus_key, item_id, category_key, author_hint);
    if featured.is_empty() {
        return;
    }

    let title_key = if focus_key == "farms" {
        UiKey::CreatorsRecommendedFarmsFooter
    } else {
        UiKey::CreatorsRecommendedDupesFooter
    };

    egui::Frame::new()
        .fill(palette.bg_card)
        .stroke(Stroke::new(1.0_f32, palette.accent_enchant))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.add(atlas.ui_image("knowledge_book", 20.0));
                ui.label(
                    RichText::new(t(lang, title_key))
                        .size(12.5)
                        .strong()
                        .color(palette.accent_gold),
                );
                let dir_btn = egui::Button::new(
                    RichText::new(t(lang, UiKey::CreatorsViewBtn))
                        .size(11.0)
                        .strong()
                        .color(palette.text_primary),
                )
                .fill(Color32::from_rgba_unmultiplied(
                    palette.accent_enchant.r(),
                    palette.accent_enchant.g(),
                    palette.accent_enchant.b(),
                    45,
                ))
                .stroke(Stroke::new(1.0_f32, palette.accent_enchant));
                if ui.add(dir_btn).clicked() {
                    *open_creators_view = true;
                    state.creators_specialty_filter = focus_key.to_string();
                    audio.play(SoundCue::Click);
                }
            });

            ui.add_space(6.0);
            for creator in featured {
                let accent = creator_platform_color(&creator.platform_key);
                egui::Frame::new()
                    .fill(palette.bg_panel)
                    .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::symmetric(8, 6))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal_wrapped(|ui| {
                            let (sr, _) =
                                ui.allocate_exact_size(Vec2::splat(28.0), egui::Sense::hover());
                            paint_mc_slot(
                                ui.painter(),
                                sr,
                                palette,
                                atlas,
                                Some(&creator.icon),
                                SlotStyle::default(),
                            );
                            ui.label(
                                RichText::new(&creator.name)
                                    .size(12.0)
                                    .strong()
                                    .color(palette.text_primary),
                            );
                            ui.label(
                                RichText::new(format!("[{}]", creator.platform_label))
                                    .size(10.5)
                                    .strong()
                                    .color(accent),
                            );
                            let mut display_tags: Vec<_> = creator.tags.iter().collect();
                            display_tags.sort_by_key(|tg| if tg.key == focus_key { 0 } else { 1 });
                            for tag in display_tags.into_iter().take(2) {
                                render_tag_pill(
                                    ui,
                                    tag.label.get(lang),
                                    creator_tag_color(&tag.key, palette),
                                    true,
                                );
                            }
                            ui.label(
                                RichText::new(format!(
                                    "{} • {}",
                                    creator.region_label.get(lang),
                                    creator.specialty.get(lang)
                                ))
                                .size(11.0)
                                .color(palette.text_muted),
                            );
                            render_creator_action_buttons(
                                ui, creator, palette, audio, toasts, lang, true,
                            );
                        });
                    });
                ui.add_space(4.0);
            }
        });
}

#[allow(clippy::too_many_arguments)]
pub fn render_creators_directory(
    ui: &mut egui::Ui,
    state: &AppState,
    creators: &[TechCreator],
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    toasts: &mut ToastQueue,
    lang: Language,
    q_lower: &str,
) {
    let filtered: Vec<&TechCreator> = creators
        .iter()
        .filter(|c| c.has_focus(&state.creators_specialty_filter))
        .filter(|c| {
            state.creators_region_filter == "all" || c.region_key == state.creators_region_filter
        })
        .filter(|c| {
            if q_lower.is_empty() {
                return true;
            }
            c.name.to_lowercase().contains(q_lower)
                || c.handle.to_lowercase().contains(q_lower)
                || c.platform_label.to_lowercase().contains(q_lower)
                || c.specialty.matches(q_lower)
                || c.desc.matches(q_lower)
                || c.region_label.matches(q_lower)
                || c.tags.iter().any(|tg| tg.label.matches(q_lower))
                || c.characteristics.iter().any(|ch| ch.matches(q_lower))
        })
        .collect();

    ui.add_space(4.0);
    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.2_f32, palette.accent_gold))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.add(atlas.ui_image("knowledge_book", 26.0));
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(t(lang, UiKey::CreatorsSectionTitle))
                            .size(16.0)
                            .strong()
                            .color(palette.accent_gold),
                    );
                    ui.add(
                        egui::Label::new(
                            RichText::new(t(lang, UiKey::CreatorsSectionSubtitle))
                                .size(11.5)
                                .color(palette.text_muted),
                        )
                        .wrap(),
                    );
                });
            });
        });

    ui.add_space(8.0);
    if filtered.is_empty() {
        ui.label(
            RichText::new(t(lang, UiKey::CommandPaletteEmpty))
                .size(13.0)
                .color(palette.text_muted),
        );
        return;
    }

    egui::ScrollArea::vertical()
        .id_salt("creators_directory_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let avail_w = ui.available_width().max(260.0);
            let cols = if avail_w >= 1020.0 {
                3
            } else if avail_w >= 620.0 {
                2
            } else {
                1
            };
            let spacing = 10.0;
            let card_w =
                ((avail_w - spacing * (cols as f32 - 1.0) - 8.0) / cols as f32).max(240.0);

            for row in filtered.chunks(cols) {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = spacing;
                    for creator in row {
                        ui.allocate_ui_with_layout(
                            Vec2::new(card_w, 0.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                render_creator_card(ui, creator, palette, atlas, audio, toasts, lang);
                            },
                        );
                    }
                });
                ui.add_space(10.0);
            }
        });
}

fn render_creator_card(
    ui: &mut egui::Ui,
    creator: &TechCreator,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    toasts: &mut ToastQueue,
    lang: Language,
) {
    let accent = creator_platform_color(&creator.platform_key);
    egui::Frame::new()
        .fill(palette.bg_card)
        .stroke(Stroke::new(1.2_f32, accent))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(196.0);
            ui.horizontal(|ui| {
                let (sr, _) = ui.allocate_exact_size(Vec2::splat(36.0), egui::Sense::hover());
                paint_mc_slot(
                    ui.painter(),
                    sr,
                    palette,
                    atlas,
                    Some(&creator.icon),
                    SlotStyle::default(),
                );
                ui.vertical(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(&creator.name)
                                .size(13.5)
                                .strong()
                                .color(palette.text_primary),
                        );
                        render_tag_pill(ui, &creator.platform_label, accent, true);
                    });
                    ui.label(
                        RichText::new(format!(
                            "{} • {}",
                            creator.handle,
                            creator.region_label.get(lang)
                        ))
                        .monospace()
                        .size(10.5)
                        .color(palette.text_muted),
                    );
                });
            });

            if !creator.tags.is_empty() {
                ui.add_space(5.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
                    for tag in &creator.tags {
                        render_tag_pill(
                            ui,
                            tag.label.get(lang),
                            creator_tag_color(&tag.key, palette),
                            false,
                        );
                    }
                });
            }

            ui.add_space(6.0);
            ui.add(
                egui::Label::new(
                    RichText::new(creator.specialty.get(lang))
                        .size(11.5)
                        .strong()
                        .color(palette.accent_gold),
                )
                .wrap(),
            );
            ui.add_space(3.0);
            ui.add(
                egui::Label::new(
                    RichText::new(creator.desc.get(lang))
                        .size(11.5)
                        .color(palette.text_primary),
                )
                .wrap(),
            );

            if !creator.characteristics.is_empty() {
                ui.add_space(5.0);
                ui.label(
                    RichText::new(t(lang, UiKey::CreatorsCharacteristicsHeader))
                        .size(10.5)
                        .strong()
                        .color(palette.text_muted),
                );
                for ch in &creator.characteristics {
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!("• {}", ch.get(lang)))
                                .size(11.0)
                                .color(palette.text_muted),
                        )
                        .wrap(),
                    );
                }
            }

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                render_creator_action_buttons(ui, creator, palette, audio, toasts, lang, false);
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::load_database;

    #[test]
    fn recommended_creators_respect_farm_and_dupe_specialties() {
        let db = load_database().expect("valid database");

        let iron_basic = db.farms.iter().find(|f| f.id == "iron_basic").unwrap();
        let farm_picks = select_recommended_creators(
            &db.creators,
            "farms",
            &iron_basic.id,
            &iron_basic.category_key,
            Some(&iron_basic.tutorial_author),
        );
        assert_eq!(farm_picks.len(), 3);
        assert!(farm_picks.iter().all(|c| c.has_focus("farms")));
        assert!(
            farm_picks.iter().any(|c| c.id == "shulkercraft" || c.id == "logicalgeekboy"),
            "Expected Shulkercraft or LogicalGeekBoy for iron_basic, got: {:?}",
            farm_picks.iter().map(|c| &c.id).collect::<Vec<_>>()
        );
        for pure_dupe in ["salc1", "themisterepic", "thedupertrooper"] {
            assert!(
                !farm_picks.iter().any(|c| c.id == pure_dupe),
                "Pure dupe creator {pure_dupe} leaked into farm recommendations"
            );
        }

        let tnt_duper = db.dupes.iter().find(|d| d.id == "tnt_duper").unwrap();
        let dupe_picks = select_recommended_creators(
            &db.creators,
            "dupes",
            &tnt_duper.id,
            &tnt_duper.category_key,
            None,
        );
        assert_eq!(dupe_picks.len(), 3);
        assert!(dupe_picks.iter().all(|c| c.has_focus("dupes")));
        for pure_farm in ["wattles", "pixlriffs", "eyecraftmc", "ianxofour"] {
            assert!(
                !dupe_picks.iter().any(|c| c.id == pure_farm),
                "Pure farm creator {pure_farm} leaked into dupe recommendations"
            );
        }
    }
}
