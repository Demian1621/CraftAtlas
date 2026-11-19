use eframe::egui::{self, CornerRadius, Margin, RichText, Stroke, Vec2};

use crate::data::models::Language;
use crate::i18n::{t, UiKey};
use crate::ui::theme::{paint_mc_slot, SlotStyle};
use crate::ui::RenderContext;
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

pub(super) fn render_alchemy_guide(
    ui: &mut egui::Ui,
    rcx: &RenderContext<'_>,
    active_ver: &str,
    lang: Language,
    q_lower: &str,
    pending_nav: &mut Option<String>,
) {
    let RenderContext {
        recipes,
        versions,
        palette,
        ..
    } = *rcx;
    let trials_active = versions.is_active(Some("1.21.1"), None, active_ver);
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
                        RichText::new(format!(
                            "{} (Java {})",
                            t(lang, UiKey::AlchemyGuideTitle),
                            active_ver
                        ))
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

            let redstone_desc = if trials_active {
                pick_lang(
                    "Duplica/extiende la duración (3:00 -> 8:00, 0:45 -> 1:30). Incompatible con Glowstone (Nivel II) y con pociones Trials 1.21+.",
                    "Aumenta la duración (3:00 -> 8:00, 0:45 -> 1:30). Incompatible con Piedra Luminosa y con pociones de Pruebas 1.21+.",
                    "Extends duration (3:00 -> 8:00, 0:45 -> 1:30). Mutually exclusive with Glowstone II and 1.21+ Trial potions.",
                )
            } else {
                pick_lang(
                    "Duplica/extiende la duración (3:00 -> 8:00, 0:45 -> 1:30). Incompatible con Glowstone (Nivel II).",
                    "Aumenta la duración (3:00 -> 8:00, 0:45 -> 1:30). Incompatible con Piedra Luminosa (Nivel II).",
                    "Extends duration (3:00 -> 8:00, 0:45 -> 1:30). Mutually exclusive with Glowstone (Level II).",
                )
            };
            let modifiers = [
                (
                    "redstone",
                    palette.accent_danger,
                    "+ Tiempo / Duración Larga",
                    "+ Duración (Poción Larga)",
                    "+ Duration (Extended)",
                    redstone_desc,
                ),
                (
                    "glowstone_dust",
                    palette.accent_gold,
                    "+ Potencia (Nivel II)",
                    "+ Potencia (Nivel II)",
                    "+ Potency (Level II)",
                    pick_lang(
                        "Sube el efecto al Nivel II (Fuerza II, Curación II, Velocidad II) y reduce duración a la mitad (3:00 -> 1:30). Incompatible con Redstone.",
                        "Mejora la potencia al Nivel II reduciendo duración a la mitad (3:00 -> 1:30). Incompatible con Redstone.",
                        "Amplifies potency to Level II (halves duration: 3:00 -> 1:30). Mutually exclusive with Redstone.",
                    ),
                ),
                (
                    "gunpowder",
                    palette.accent_primary,
                    "Hacerla Lanzadora / Arrojadiza",
                    "Hacerla Arrojadiza (Lanzadora)",
                    "Make Splash (Throwable)",
                    pick_lang(
                        "Transforma cualquier poción en Arrojadiza (Splash Potion) para lanzar a mano o desde dispensadores.",
                        "Convierte una poción bebible en Arrojadiza (Splash Potion) para lanzar o usar con dispensadores.",
                        "Turns any drinkable potion into a throwable Splash Potion for combat or dispenser farms.",
                    ),
                ),
                (
                    "dragon_breath",
                    palette.accent_enchant,
                    "Hacerla Persistente + Flechas",
                    "Hacerla Persistente + Flechas",
                    "Make Lingering + Tipped Arrows",
                    pick_lang(
                        "Aplícalo a una Lanzadora -> nube persistente en el suelo. Rodea 1 Persistente con 8 Flechas en mesa -> 8 Flechas con Efecto.",
                        "Aplícalo sobre una Arrojadiza -> nube persistente. Combina 1 Persistente + 8 Flechas en mesa -> 8 Flechas con Efecto.",
                        "Apply on a Splash Potion -> lingering area cloud. Surround 1 Lingering with 8 Arrows in Crafting Table -> 8 Tipped Arrows.",
                    ),
                ),
                (
                    "fermented_spider_eye",
                    palette.accent_xp,
                    "Invertir / Corromper Efecto",
                    "Invertir / Corromper Efecto",
                    "Invert / Corrupt Effect",
                    pick_lang(
                        "Visión Nocturna -> Invisibilidad | Velocidad o Salto -> Lentitud | Curación o Veneno -> Daño Instantáneo | Agua -> Debilidad.",
                        "Visión Nocturna -> Invisibilidad | Rapidez o Salto -> Lentitud | Curación o Veneno -> Daño | Agua -> Debilidad.",
                        "Night Vision -> Invisibility | Speed or Leaping -> Slowness | Healing or Poison -> Harming | Water -> Weakness.",
                    ),
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
                    for &(icon, accent, t_mx, t_es, t_en, desc_txt) in row {
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
                                                        RichText::new(desc_txt)
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

            // Step 3: Complete Base Potions Table (15 pre-1.21 + 4 Trial potions in 1.21.1+)
            ui.label(
                RichText::new(t(lang, UiKey::AlchemyBaseRecipesTitle))
                    .size(12.5)
                    .strong()
                    .color(palette.accent_gold),
            );
            ui.add_space(4.0);

            let mut potion_rows = vec![
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
            ];
            if trials_active {
                potion_rows.extend_from_slice(&[
                    ("breeze_rod", "Carga de Viento (1.21+)", "Carga de Viento (1.21+)", "Wind Charged (1.21+)", "3:00", "—", "—"),
                    ("cobweb", "Tejido / Telarañas (1.21+)", "Tejido / Telarañas (1.21+)", "Weaving (1.21+)", "3:00", "—", "—"),
                    ("slime_block", "Viscosidad / Slimes (1.21+)", "Viscosidad / Limos (1.21+)", "Oozing (1.21+)", "3:00", "—", "—"),
                    ("stone", "Infestación / Silverfish (1.21+)", "Infestación / Lepismas (1.21+)", "Infested (1.21+)", "3:00", "—", "—"),
                ]);
            }

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
