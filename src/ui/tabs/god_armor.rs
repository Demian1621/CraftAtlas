use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, Vec2};

use crate::anim::ToastQueue;
use crate::assets::ItemAtlas;
use crate::audio::SoundCue;
use crate::data::models::{Language, VersionRegistry};
use crate::data::version_rules::{enchant_since_version, is_item_active};
use crate::state::AppState;
use crate::ui::theme::{adaptive_sidebar_width, filter_chip, paint_mc_slot, McPalette, SlotStyle};
use crate::ui::RenderContext;

#[derive(Clone, Copy)]
pub enum SurvivalWindow {
    Java1140to1142,
    Java19to1111,
    CommandsOrDatapackOnly,
}

#[derive(Clone)]
pub struct IllegalGodPiece {
    pub id: &'static str,
    pub item_id: &'static str,
    pub is_armor: bool,
    pub survival_window: SurvivalWindow,
    pub min_ver: Option<&'static str>,
    pub name_mx: &'static str,
    pub name_es: &'static str,
    pub name_en: &'static str,
    pub badge_mx: &'static str,
    pub badge_es: &'static str,
    pub badge_en: &'static str,
    pub enchants: Vec<(&'static str, u8)>,
    pub conflict_mx: &'static str,
    pub conflict_es: &'static str,
    pub conflict_en: &'static str,
}

fn pick<'a>(lang: Language, mx: &'a str, es: &'a str, en: &'a str) -> &'a str {
    match lang {
        Language::EsMx => mx,
        Language::EsEs => es,
        Language::EnUs => en,
    }
}

/// Builds the complete catalog of illegal/legacy God pieces filtered and adapted for `active_ver`.
pub fn god_pieces_for_version(
    versions: &VersionRegistry,
    active_ver: &str,
    include_thorns: bool,
) -> Vec<IllegalGodPiece> {
    let mut raw = vec![
        IllegalGodPiece {
            id: "god_helmet",
            item_id: "netherite_helmet",
            is_armor: true,
            survival_window: SurvivalWindow::Java1140to1142,
            min_ver: None,
            name_mx: "Casco God (4 Protecciones)",
            name_es: "Casco God (4 Protecciones)",
            name_en: "God Helmet (All 4 Protections)",
            badge_mx: "Survival Vanilla 1.14.0-1.14.2",
            badge_es: "Supervivencia Vanilla 1.14.0-1.14.2",
            badge_en: "Vanilla Survival 1.14.0-1.14.2",
            enchants: vec![
                ("protection", 4),
                ("fire_protection", 4),
                ("blast_protection", 4),
                ("projectile_protection", 4),
                ("respiration", 3),
                ("aqua_affinity", 1),
                ("unbreaking", 3),
                ("mending", 1),
            ],
            conflict_mx: "Protección + Fuego + Explosiones + Proyectiles IV simultáneas",
            conflict_es: "Protección + Fuego + Explosiones + Proyectiles IV simultáneas",
            conflict_en: "All 4 Protection types IV stacked simultaneously",
        },
        IllegalGodPiece {
            id: "god_chestplate",
            item_id: "netherite_chestplate",
            is_armor: true,
            survival_window: SurvivalWindow::Java1140to1142,
            min_ver: None,
            name_mx: "Pechera God (4 Protecciones)",
            name_es: "Pechera God (4 Protecciones)",
            name_en: "God Chestplate (All 4 Protections)",
            badge_mx: "Survival Vanilla 1.14.0-1.14.2",
            badge_es: "Supervivencia Vanilla 1.14.0-1.14.2",
            badge_en: "Vanilla Survival 1.14.0-1.14.2",
            enchants: vec![
                ("protection", 4),
                ("fire_protection", 4),
                ("blast_protection", 4),
                ("projectile_protection", 4),
                ("unbreaking", 3),
                ("mending", 1),
            ],
            conflict_mx: "Las 4 Protecciones IV juntas (EPF máximo contra Creeper/TNT/Fuego)",
            conflict_es: "Las 4 Protecciones IV juntas (EPF máximo frente a todo daño)",
            conflict_en: "All 4 Protection types IV stacked (20 EPF cap vs fire/blast/proj)",
        },
        IllegalGodPiece {
            id: "god_leggings",
            item_id: "netherite_leggings",
            is_armor: true,
            survival_window: SurvivalWindow::Java1140to1142,
            min_ver: None,
            name_mx: "Pantalones God (4 Protecciones)",
            name_es: "Grebas God (4 Protecciones)",
            name_en: "God Leggings (All 4 Protections)",
            badge_mx: "Survival Vanilla 1.14.0-1.14.2",
            badge_es: "Supervivencia Vanilla 1.14.0-1.14.2",
            badge_en: "Vanilla Survival 1.14.0-1.14.2",
            enchants: vec![
                ("protection", 4),
                ("fire_protection", 4),
                ("blast_protection", 4),
                ("projectile_protection", 4),
                ("swift_sneak", 3),
                ("unbreaking", 3),
                ("mending", 1),
            ],
            conflict_mx: "4 Protecciones IV + Swift Sneak III (en 1.19.4+)",
            conflict_es: "4 Protecciones IV + Sigilo Rápido III (en 1.19.4+)",
            conflict_en: "All 4 Protections IV + Swift Sneak III (in 1.19.4+)",
        },
        IllegalGodPiece {
            id: "god_boots",
            item_id: "netherite_boots",
            is_armor: true,
            survival_window: SurvivalWindow::Java1140to1142,
            min_ver: None,
            name_mx: "Botas God (4 Protecciones)",
            name_es: "Botas God (4 Protecciones)",
            name_en: "God Boots (All 4 Protections)",
            badge_mx: "Survival Vanilla 1.14.0-1.14.2",
            badge_es: "Supervivencia Vanilla 1.14.0-1.14.2",
            badge_en: "Vanilla Survival 1.14.0-1.14.2",
            enchants: vec![
                ("protection", 4),
                ("fire_protection", 4),
                ("blast_protection", 4),
                ("projectile_protection", 4),
                ("feather_falling", 4),
                ("depth_strider", 3),
                ("soul_speed", 3),
                ("unbreaking", 3),
                ("mending", 1),
            ],
            conflict_mx: "4 Protecciones IV + Velocidad del Alma III + Caída de Pluma IV",
            conflict_es: "4 Protecciones IV + Velocidad de Alma III + Caída de Pluma IV",
            conflict_en: "All 4 Protections IV + Soul Speed III + Feather Falling IV",
        },
        IllegalGodPiece {
            id: "god_bow",
            item_id: "bow",
            is_armor: false,
            survival_window: SurvivalWindow::Java19to1111,
            min_ver: None,
            name_mx: "Arco God (Infinito + Reparación)",
            name_es: "Arco God (Infinidad + Reparación)",
            name_en: "God Bow (Mending + Infinity)",
            badge_mx: "Survival Vanilla 1.9-1.11.1",
            badge_es: "Supervivencia Vanilla 1.9-1.11.1",
            badge_en: "Vanilla Survival 1.9-1.11.1",
            enchants: vec![
                ("power", 5),
                ("punch", 2),
                ("flame", 1),
                ("infinity", 1),
                ("mending", 1),
                ("unbreaking", 3),
            ],
            conflict_mx: "Infinidad + Reparación (Mending) en el mismo arco",
            conflict_es: "Infinidad + Reparación en el mismo arco",
            conflict_en: "Infinity + Mending combined on one Bow",
        },
        IllegalGodPiece {
            id: "god_crossbow",
            item_id: "crossbow",
            is_armor: false,
            survival_window: SurvivalWindow::CommandsOrDatapackOnly,
            min_ver: None,
            name_mx: "Ballesta God (Multidisparo + Perforación)",
            name_es: "Ballesta God (Multidisparo + Perforación)",
            name_en: "God Crossbow (Multishot + Piercing)",
            badge_mx: "Comandos / Datapack / Servidor",
            badge_es: "Comandos / Datapack / Servidor",
            badge_en: "Commands / Datapack / Server",
            enchants: vec![
                ("multishot", 1),
                ("piercing", 4),
                ("quick_charge", 3),
                ("unbreaking", 3),
                ("mending", 1),
            ],
            conflict_mx: "3 flechas por disparo (Multishot) que atraviesan escudos (Piercing IV)",
            conflict_es: "3 flechas por disparo que además atraviesan escudos y enemigos",
            conflict_en: "3 projectiles per shot that pierce shields and entities",
        },
        IllegalGodPiece {
            id: "god_sword",
            item_id: "netherite_sword",
            is_armor: false,
            survival_window: SurvivalWindow::CommandsOrDatapackOnly,
            min_ver: None,
            name_mx: "Espada God (Filo + Castigo + Artrópodos)",
            name_es: "Espada God (Filo + Castigo + Perdición)",
            name_en: "God Sword (Triple Damage Enchant)",
            badge_mx: "Comandos / Datapack / Servidor",
            badge_es: "Comandos / Datapack / Servidor",
            badge_en: "Commands / Datapack / Server",
            enchants: vec![
                ("sharpness", 5),
                ("smite", 5),
                ("bane_of_arthropods", 5),
                ("looting", 3),
                ("sweeping_edge", 3),
                ("fire_aspect", 2),
                ("unbreaking", 3),
                ("mending", 1),
            ],
            conflict_mx: "Sharpness V + Smite V + Bane V acumulados (~+15.5 daño vs no-muertos)",
            conflict_es: "Filo V + Castigo V + Perdición V simultáneos en un golpe",
            conflict_en: "Sharpness V + Smite V + Bane V stacked on every hit",
        },
        IllegalGodPiece {
            id: "god_pickaxe",
            item_id: "netherite_pickaxe",
            is_armor: false,
            survival_window: SurvivalWindow::CommandsOrDatapackOnly,
            min_ver: None,
            name_mx: "Pico God (Fortuna III + Toque de Seda)",
            name_es: "Pico God (Fortuna III + Toque de Seda)",
            name_en: "God Pickaxe (Fortune III + Silk Touch)",
            badge_mx: "Solo Demostración (Silk gana en drops)",
            badge_es: "Curiosidad / Comandos (Silk domina el drop)",
            badge_en: "Commands Only (Silk Touch overrides Fortune)",
            enchants: vec![
                ("efficiency", 5),
                ("fortune", 3),
                ("silk_touch", 1),
                ("unbreaking", 3),
                ("mending", 1),
            ],
            conflict_mx: "Fortuna III + Silk Touch juntos (nota: en loot tables Silk Touch tiene prioridad)",
            conflict_es: "Fortuna III + Toque de Seda (nota: Toque de Seda prevalece al picar)",
            conflict_en: "Fortune III + Silk Touch together (Silk Touch takes priority in loot tables)",
        },
        IllegalGodPiece {
            id: "god_mace",
            item_id: "mace",
            is_armor: false,
            survival_window: SurvivalWindow::CommandsOrDatapackOnly,
            min_ver: Some("1.21.1"),
            name_mx: "Mazo God (Densidad V + Fisura IV)",
            name_es: "Mazo God (Densidad V + Brecha IV)",
            name_en: "God Mace (Density V + Breach IV)",
            badge_mx: "Comandos / Datapack 1.21+",
            badge_es: "Comandos / Datapack 1.21+",
            badge_en: "Commands / Datapack 1.21+",
            enchants: vec![
                ("density", 5),
                ("breach", 4),
                ("wind_burst", 3),
                ("fire_aspect", 2),
                ("unbreaking", 3),
                ("mending", 1),
            ],
            conflict_mx: "Daño por caída infinito (Densidad V) ignorando armadura (Fisura IV)",
            conflict_es: "Daño por caída extremo ignorando el 60% de la armadura rival",
            conflict_en: "Extreme fall damage + 60% armor penetration + Wind Burst III",
        },
    ];

    raw.retain(|p| {
        versions.is_active(p.min_ver, None, active_ver)
            && is_item_active(versions, p.item_id, active_ver)
    });

    for piece in &mut raw {
        piece.enchants.retain(|(eid, _)| {
            versions.is_active(enchant_since_version(eid), None, active_ver)
        });
        if piece.is_armor && include_thorns {
            // Insert Thorns III right before Unbreaking
            let pos = piece
                .enchants
                .iter()
                .position(|(id, _)| *id == "unbreaking")
                .unwrap_or(piece.enchants.len());
            piece.enchants.insert(pos, ("thorns", 3));
        }
    }

    raw
}

/// Generates an exact `/give` command formatted for the selected Minecraft Java version.
pub fn build_god_give_command(item_id: &str, enchants: &[(&str, u8)], active_ver: &str) -> String {
    if active_ver.starts_with("1.18") || active_ver.starts_with("1.19") {
        let entries: Vec<String> = enchants
            .iter()
            .map(|(id, lvl)| format!("{{id:\"minecraft:{id}\",lvl:{lvl}s}}"))
            .collect();
        format!("/give @p minecraft:{item_id}{{Enchantments:[{}]}} 1", entries.join(","))
    } else if active_ver.starts_with("1.20") || active_ver.starts_with("1.21") {
        let entries: Vec<String> = enchants
            .iter()
            .map(|(id, lvl)| format!("\"minecraft:{id}\":{lvl}"))
            .collect();
        format!(
            "/give @p minecraft:{item_id}[minecraft:enchantments={{levels:{{{}}}}}] 1",
            entries.join(",")
        )
    } else {
        // 26.3 / 1.21.5+ simplified component syntax (no `levels:{}` wrapper)
        let entries: Vec<String> = enchants
            .iter()
            .map(|(id, lvl)| format!("\"minecraft:{id}\":{lvl}"))
            .collect();
        format!(
            "/give @p minecraft:{item_id}[minecraft:enchantments={{{}}}] 1",
            entries.join(",")
        )
    }
}

fn enchant_display_name(id: &str, lvl: u8, lang: Language) -> String {
    let base = match (id, lang) {
        ("protection", Language::EnUs) => "Protection",
        ("protection", _) => "Protección",
        ("fire_protection", Language::EnUs) => "Fire Protection",
        ("fire_protection", _) => "Protección contra el fuego",
        ("blast_protection", Language::EnUs) => "Blast Protection",
        ("blast_protection", _) => "Protección contra explosiones",
        ("projectile_protection", Language::EnUs) => "Projectile Protection",
        ("projectile_protection", _) => "Protección contra proyectiles",
        ("thorns", Language::EnUs) => "Thorns",
        ("thorns", _) => "Espinas",
        ("respiration", Language::EnUs) => "Respiration",
        ("respiration", _) => "Respiración",
        ("aqua_affinity", Language::EnUs) => "Aqua Affinity",
        ("aqua_affinity", _) => "Afinidad acuática",
        ("feather_falling", Language::EnUs) => "Feather Falling",
        ("feather_falling", _) => "Caída de pluma",
        ("depth_strider", Language::EnUs) => "Depth Strider",
        ("depth_strider", _) => "Agilidad acuática",
        ("soul_speed", Language::EnUs) => "Soul Speed",
        ("soul_speed", _) => "Velocidad del alma",
        ("swift_sneak", Language::EnUs) => "Swift Sneak",
        ("swift_sneak", _) => "Sigilo rápido",
        ("unbreaking", Language::EnUs) => "Unbreaking",
        ("unbreaking", _) => "Irrompibilidad",
        ("mending", Language::EnUs) => "Mending",
        ("mending", _) => "Reparación",
        ("power", Language::EnUs) => "Power",
        ("power", _) => "Poder",
        ("punch", Language::EnUs) => "Punch",
        ("punch", _) => "Retroceso",
        ("flame", Language::EnUs) => "Flame",
        ("flame", _) => "Fuego",
        ("infinity", Language::EnUs) => "Infinity",
        ("infinity", _) => "Infinidad",
        ("multishot", Language::EnUs) => "Multishot",
        ("multishot", _) => "Multidisparo",
        ("piercing", Language::EnUs) => "Piercing",
        ("piercing", _) => "Perforación",
        ("quick_charge", Language::EnUs) => "Quick Charge",
        ("quick_charge", _) => "Carga rápida",
        ("sharpness", Language::EnUs) => "Sharpness",
        ("sharpness", _) => "Filo",
        ("smite", Language::EnUs) => "Smite",
        ("smite", _) => "Castigo",
        ("bane_of_arthropods", Language::EnUs) => "Bane of Arthropods",
        ("bane_of_arthropods", _) => "Perdición de los artrópodos",
        ("looting", Language::EnUs) => "Looting",
        ("looting", _) => "Saqueo / Botín",
        ("sweeping_edge", Language::EnUs) => "Sweeping Edge",
        ("sweeping_edge", _) => "Filo arrasador",
        ("fire_aspect", Language::EnUs) => "Fire Aspect",
        ("fire_aspect", _) => "Aspecto ígneo",
        ("efficiency", Language::EnUs) => "Efficiency",
        ("efficiency", _) => "Eficiencia",
        ("fortune", Language::EnUs) => "Fortune",
        ("fortune", _) => "Fortuna",
        ("silk_touch", Language::EnUs) => "Silk Touch",
        ("silk_touch", _) => "Toque de seda",
        ("density", Language::EnUs) => "Density",
        ("density", _) => "Densidad",
        ("breach", Language::EnUs) => "Breach",
        ("breach", Language::EsMx) => "Fisura",
        ("breach", Language::EsEs) => "Brecha",
        ("wind_burst", Language::EnUs) => "Wind Burst",
        ("wind_burst", _) => "Aeroimpulso",
        _ => id,
    };
    let roman = match lvl {
        1 => "",
        2 => " II",
        3 => " III",
        4 => " IV",
        5 => " V",
        _ => "",
    };
    format!("{base}{roman}")
}

pub fn render(
    ui: &mut egui::Ui,
    state: &mut AppState,
    rcx: &RenderContext<'_>,
    toasts: &mut ToastQueue,
    active_ver: &str,
) {
    let lang = state.prefs.language;
    let pieces = god_pieces_for_version(rcx.versions, active_ver, state.god_include_thorns);
    if pieces.is_empty() {
        return;
    }
    if state.god_piece_idx >= pieces.len() {
        state.god_piece_idx = 0;
    }

    let total_w = ui.available_width();
    if total_w >= 780.0 {
        let side_w = adaptive_sidebar_width(total_w);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(side_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    render_piece_sidebar(ui, state, &pieces, rcx, lang);
                },
            );
            ui.add_space(10.0);
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let selected = &pieces[state.god_piece_idx];
                    render_piece_detail(ui, state, selected, &pieces, rcx, toasts, active_ver);
                },
            );
        });
    } else {
        egui::ScrollArea::horizontal()
            .id_salt("god_armor_compact_picker")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (idx, p) in pieces.iter().enumerate() {
                        if filter_chip(
                            ui,
                            rcx.palette,
                            rcx.atlas,
                            Some(p.item_id),
                            pick(lang, p.name_mx, p.name_es, p.name_en),
                            state.god_piece_idx == idx,
                        )
                        .clicked()
                        {
                            state.god_piece_idx = idx;
                            rcx.audio.play(SoundCue::Click);
                        }
                    }
                });
            });
        ui.add_space(8.0);
        let selected = &pieces[state.god_piece_idx];
        render_piece_detail(ui, state, selected, &pieces, rcx, toasts, active_ver);
    }
}

fn render_piece_sidebar(
    ui: &mut egui::Ui,
    state: &mut AppState,
    pieces: &[IllegalGodPiece],
    rcx: &RenderContext<'_>,
    lang: Language,
) {
    let RenderContext {
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
    let time = ui.ctx().input(|i| i.time);

    let thorns_lbl = match lang {
        Language::EnUs => "Include Thorns III on Armor",
        _ => "Incluir Espinas III en Armadura",
    };
    if ui.checkbox(&mut state.god_include_thorns, thorns_lbl).changed() {
        audio.play(SoundCue::Click);
    }
    ui.add_space(6.0);

    egui::ScrollArea::vertical()
        .id_salt("god_armor_sidebar_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (idx, p) in pieces.iter().enumerate() {
                let selected = state.god_piece_idx == idx;
                let accent = match p.survival_window {
                    SurvivalWindow::Java1140to1142 | SurvivalWindow::Java19to1111 => {
                        palette.accent_gold
                    }
                    SurvivalWindow::CommandsOrDatapackOnly => palette.accent_enchant,
                };
                let fill = if selected {
                    Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 34)
                } else {
                    palette.bg_card
                };
                let stroke = if selected {
                    Stroke::new(1.5_f32, accent)
                } else {
                    Stroke::new(1.0_f32, palette.border_subtle)
                };

                let frame_resp = egui::Frame::new()
                    .fill(fill)
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
                                Some(p.item_id),
                                SlotStyle {
                                    selected,
                                    glint_time: Some(time),
                                    ..Default::default()
                                },
                            );
                            ui.vertical(|ui| {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(pick(lang, p.name_mx, p.name_es, p.name_en))
                                            .size(12.0)
                                            .strong()
                                            .color(if selected { Color32::WHITE } else { palette.text_primary }),
                                    )
                                    .wrap(),
                                );
                                ui.label(
                                    RichText::new(pick(lang, p.badge_mx, p.badge_es, p.badge_en))
                                        .size(10.0)
                                        .color(accent),
                                );
                            });
                        });
                    });
                if frame_resp.response.interact(egui::Sense::click()).clicked() {
                    state.god_piece_idx = idx;
                    audio.play(SoundCue::Click);
                }
                ui.add_space(4.0);
            }
        });
}

fn render_piece_detail(
    ui: &mut egui::Ui,
    state: &mut AppState,
    piece: &IllegalGodPiece,
    all_pieces: &[IllegalGodPiece],
    rcx: &RenderContext<'_>,
    toasts: &mut ToastQueue,
    active_ver: &str,
) {
    let RenderContext {
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
    let lang = state.prefs.language;
    let time = ui.ctx().input(|i| i.time);

    egui::ScrollArea::vertical()
        .id_salt("god_armor_detail_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // 1. Hero Header + Enchantments grid
            egui::Frame::new()
                .fill(palette.bg_card)
                .stroke(Stroke::new(1.5_f32, palette.accent_gold))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let (slot_r, _) =
                            ui.allocate_exact_size(Vec2::splat(56.0), egui::Sense::hover());
                        paint_mc_slot(
                            ui.painter(),
                            slot_r,
                            palette,
                            atlas,
                            Some(piece.item_id),
                            SlotStyle {
                                selected: true,
                                glint_time: Some(time),
                                ..Default::default()
                            },
                        );
                        ui.add_space(6.0);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(pick(lang, piece.name_mx, piece.name_es, piece.name_en))
                                    .size(18.0)
                                    .strong()
                                    .color(palette.accent_gold),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "{} | Java {}",
                                    pick(lang, piece.conflict_mx, piece.conflict_es, piece.conflict_en),
                                    active_ver
                                ))
                                .size(11.5)
                                .color(palette.accent_enchant),
                            );
                        });
                    });

                    ui.add_space(8.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(5.0, 5.0);
                        for &(eid, lvl) in &piece.enchants {
                            let label = enchant_display_name(eid, lvl, lang);
                            egui::Frame::new()
                                .fill(Color32::from_rgba_unmultiplied(
                                    palette.accent_enchant.r(),
                                    palette.accent_enchant.g(),
                                    palette.accent_enchant.b(),
                                    28,
                                ))
                                .stroke(Stroke::new(1.0_f32, palette.accent_enchant))
                                .corner_radius(CornerRadius::same(4))
                                .inner_margin(Margin::symmetric(7, 3))
                                .show(ui, |ui| {
                                    ui.label(RichText::new(label).size(11.0).strong().color(Color32::WHITE));
                                });
                        }
                    });
                });

            ui.add_space(8.0);

            // 2. EPF 80% Math + Critical Safety Banner
            render_epf_and_warning_cards(ui, palette, atlas, lang);
            ui.add_space(10.0);

            // 3. Method Selector Tabs
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 5.0);
                let tabs = [
                    (
                        0u8,
                        "anvil",
                        pick(
                            lang,
                            "Paso a Paso Survival (Glitch 1.14.2)",
                            "Paso a Paso Supervivencia (1.14.2)",
                            "Vanilla Survival Step-by-Step",
                        ),
                    ),
                    (
                        1u8,
                        "command_block",
                        pick(
                            lang,
                            "Comando /give Exacto",
                            "Comando /give Exacto",
                            "Exact /give Command",
                        ),
                    ),
                    (
                        2u8,
                        "writable_book",
                        pick(
                            lang,
                            "Datapack / Servidor",
                            "Datapack / Servidor",
                            "Datapack / Server",
                        ),
                    ),
                ];
                for (id, icon, lbl) in tabs {
                    if filter_chip(
                        ui,
                        palette,
                        atlas,
                        Some(icon),
                        lbl,
                        state.god_method_tab == id,
                    )
                    .clicked()
                    {
                        state.god_method_tab = id;
                        audio.play(SoundCue::Click);
                    }
                }
            });

            ui.add_space(8.0);

            match state.god_method_tab {
                0 => render_survival_steps_panel(ui, piece, palette, lang, active_ver),
                1 => render_give_command_panel(ui, piece, all_pieces, rcx, toasts, active_ver, lang),
                _ => render_datapack_panel(ui, rcx, toasts, active_ver, lang),
            }
        });
}

fn render_epf_and_warning_cards(
    ui: &mut egui::Ui,
    palette: McPalette,
    atlas: &ItemAtlas,
    lang: Language,
) {
    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.0_f32, palette.accent_primary))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add(atlas.ui_image("netherite_chestplate", 16.0));
                ui.label(
                    RichText::new(pick(
                        lang,
                        "Matemática EPF real: ¿Por qué sirve la God Armor?",
                        "Matemática EPF real: ¿Por qué merece la pena?",
                        "EPF Math: How God Armor Actually Works",
                    ))
                    .strong()
                    .size(12.0)
                    .color(palette.accent_primary),
                );
            });
            ui.add_space(4.0);
            ui.add(
                egui::Label::new(
                    RichText::new(pick(
                        lang,
                        "Armadura legal con Protección IV x4 suma EPF 16 (64% reducción adicional tras armadura). God Armor añade Fuego IV, Explosiones IV y Proyectiles IV hasta alcanzar el tope global EPF 20 (80% reducción) contra TODOS esos daños simultáneamente manteniendo 64% general. Además el efecto secundario anti-fuego y anti-retroceso de Blast Protection se aplica en todas las piezas.",
                        "La armadura legal con Protección IV x4 alcanza EPF 16 (64% de reducción extra). God Armor alcanza el tope EPF 20 (80% máximo) contra Fuego, Explosiones y Proyectiles a la vez, y mantiene los bonus secundarios de menor quemadura y retroceso.",
                        "Legal Protection IV x4 yields EPF 16 (64% extra reduction). Stacking Fire/Blast/Projectile Prot IV reaches the hard EPF 20 cap (80% reduction) against ALL elemental/blast/arrow damage at once, while retaining burn-time & explosion-knockback reduction.",
                    ))
                    .size(11.5)
                    .color(palette.text_primary),
                )
                .wrap(),
            );
        });

    ui.add_space(6.0);

    let warn_col = Color32::from_rgb(235, 86, 86);
    egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(warn_col.r(), warn_col.g(), warn_col.b(), 22))
        .stroke(Stroke::new(1.2_f32, warn_col))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(9))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add(
                egui::Label::new(
                    RichText::new(pick(
                        lang,
                        "ADVERTENCIA CRÍTICA DE MUNDOS: Nunca hagas downgrade de un mundo moderno a 1.14.2 (corrompe chunks, inventarios y entidades). La técnica Survival Vanilla se crea SIEMPRE empezando el mundo en 1.14.2 y actualizando hacia adelante (con respaldo .zip antes de cada salto).",
                        "ADVERTENCIA CRÍTICA: Jamás bajes de versión (downgrade) un mundo actual a 1.14.2 porque corrompe chunks e inventarios. En Supervivencia se crea en un mundo nuevo en 1.14.2 y se sube hacia adelante haciendo copia .zip previa.",
                        "CRITICAL WORLD WARNING: Never downgrade a modern save back to 1.14.2—downgrading corrupts chunks and inventories. Pure Vanilla Survival creation MUST start in 1.14.2 and upgrade forward only (always keep a .zip backup first).",
                    ))
                    .size(11.0)
                    .strong()
                    .color(Color32::from_rgb(255, 195, 195)),
                )
                .wrap(),
            );
        });
}

fn render_survival_steps_panel(
    ui: &mut egui::Ui,
    piece: &IllegalGodPiece,
    palette: McPalette,
    lang: Language,
    active_ver: &str,
) {
    let is_legacy_netherite = matches!(active_ver, "1.18.2" | "1.19.4");
    let has_swift_sneak = !active_ver.starts_with("1.18");

    egui::Frame::new()
        .fill(palette.bg_card)
        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            match piece.survival_window {
                SurvivalWindow::Java1140to1142 => {
                    ui.label(
                        RichText::new(pick(
                            lang,
                            "Método Survival Vanilla (Ventana oficial Java 1.14.0 - 1.14.2)",
                            "Método Supervivencia Vanilla (Java 1.14.0 - 1.14.2)",
                            "Pure Vanilla Survival Method (Java 1.14.0 - 1.14.2 Bug Window)",
                        ))
                        .size(13.5)
                        .strong()
                        .color(palette.accent_gold),
                    );
                    ui.add_space(6.0);

                    let mut steps = vec![
                        pick(
                            lang,
                            "1. Crea o abre un mundo nuevo específicamente en Minecraft Java 1.14.2 (donde Mojang eliminó temporalmente el bloqueo entre las 4 Protecciones).",
                            "1. Crea un mundo en Minecraft Java 1.14.2 (versión exacta donde las 4 Protecciones se podían combinar en yunque).",
                            "1. Create a world in Minecraft Java 1.14.2 (the exact release window where Mojang allowed combining all 4 Protection types).",
                        ).to_string(),
                        pick(
                            lang,
                            "2. Consigue armadura de Diamante limpia (sin encantamientos previos para mantener el Prior Work Penalty en 0).",
                            "2. Prepara piezas de Diamante limpias con coste previo = 0 (prior work penalty en cero).",
                            "2. Craft fresh Diamond armor (0 prior work penalty) plus Level IV books for Protection, Fire, Blast, and Projectile Prot.",
                        ).to_string(),
                        pick(
                            lang,
                            "3. Orden binario óptimo en Yunque (1.14.2): Junta los libros en pares ANTES de aplicarlos a la pieza -> Par A (Protección IV + Fuego IV), Par B (Explosiones IV + Proyectiles IV), Par C (Irrompibilidad III + Reparación), y luego une la pieza de Diamante con los pares en pirámide (2-3 trabajos por objeto máximo).",
                            "3. Pirámide óptima en Yunque (1.14.2): Combina libros por parejas antes de tocar la armadura (Prot+Fuego, Explosiones+Proyectiles, Irromp.+Reparación) y aplica en pirámide sin superar el límite de 39 niveles.",
                            "3. Binary Anvil Pyramid in 1.14.2: Combine books into pairs FIRST -> Pair A (Prot IV + Fire Prot IV), Pair B (Blast IV + Proj IV), Pair C (Unbreaking III + Mending + extras), then merge in balanced pairs so anvil cost never hits 'Too Expensive!'.",
                        ).to_string(),
                    ];

                    if piece.id == "god_helmet" {
                        steps.push(pick(
                            lang,
                            "4. Casco extra en 1.14.2: Añade Respiración III y Afinidad Acuática emparejándolos entre sí antes de sumarlos al casco de Diamante.",
                            "4. Casco extra en 1.14.2: Junta Respiración III + Afinidad Acuática antes de aplicarlos al casco.",
                            "4. Helmet extras in 1.14.2: Pair Respiration III + Aqua Affinity together before merging into the Helmet.",
                        ).to_string());
                    } else if piece.id == "god_boots" {
                        steps.push(pick(
                            lang,
                            "4. Botas extra en 1.14.2: Aplica Caída de Pluma IV + Agilidad Acuática III primero en 1.14.2, dejando un hueco de Prior Work libre para Velocidad del Alma III en versiones posteriores.",
                            "4. Botas extra en 1.14.2: Aplica Caída de Pluma IV + Agilidad Acuática III dejando margen de trabajo para añadir Velocidad del Alma III tras subir de versión.",
                            "4. Boots extras in 1.14.2: Apply Feather Falling IV + Depth Strider III in 1.14.2 while keeping anvil penalty <= 3 so Soul Speed III fits after upgrading.",
                        ).to_string());
                    }

                    if is_legacy_netherite {
                        steps.push(format!(
                            "5. {}",
                            pick(
                                lang,
                                "Actualiza el mundo hacia Java 1.16.5 -> tu versión activa (",
                                "Sube el mundo hacia Java 1.16.5 -> ",
                                "Upgrade world forward to 1.16.5 -> ",
                            )
                            .to_string()
                                + active_ver
                                + pick(
                                    lang,
                                    "). En esta versión transformas Diamante -> Netherita en Mesa de Herrería usando solo 1 Lingote de Netherita (sin Plantilla de Mejora) y conservas los 4 encantamientos legales en inventario.",
                                    ": transforma Diamante en Netherita en la Mesa de Herrería con solo 1 lingote (sin plantilla) conservando las 4 Protecciones intactas.",
                                    ": Smith Diamond -> Netherite using only 1 Ingot (no template required here!) while keeping all 4 Protections.",
                                )
                        ));
                    } else {
                        steps.push(format!(
                            "5. {}",
                            pick(
                                lang,
                                "Truco de ahorro para llegar a ",
                                "Consejo clave al subir hasta ",
                                "Upgrade Pro-Tip for ",
                            )
                            .to_string()
                                + active_ver
                                + pick(
                                    lang,
                                    ": haz parada intermedia en 1.19.4 para convertir todo a Netherita SIN gastar 4 Plantillas de Mejora, y luego sube a tu versión final. (Si pasas directo a 1.20.6+, necesitarás 1 Plantilla de Bastión por pieza).",
                                    ": pásate brevemente por 1.19.4 para mejorar a Netherita gratis sin gastar Plantillas de Herrería antes de entrar a ",
                                    ": stop at 1.19.4 along the way to smith Diamond -> Netherite WITHOUT needing 4 Bastion Upgrade Templates, then upgrade onward.",
                                )
                        ));
                    }

                    if piece.id == "god_boots" {
                        steps.push(pick(
                            lang,
                            "6. Finalmente (1.16+): Consigue Velocidad del Alma III por trueque con Piglins o cofres de Bastión y aplícalo al final con libro en el Yunque.",
                            "6. Último paso (1.16+): Obtén Velocidad del Alma III en Bastiones o trueque Piglin y añádelo a las Botas en el Yunque.",
                            "6. Final Boot step (1.16+): Barter with Piglins for Soul Speed III and apply last in Anvil.",
                        ).to_string());
                    } else if piece.id == "god_leggings" && has_swift_sneak {
                        steps.push(pick(
                            lang,
                            "6. Finalmente (1.19+ en adelante): Consigue Sigilo Rápido III en cofres de Ciudad Antigua y aplícalo al último sobre tus Pantalones God.",
                            "6. Último paso (1.19+): Saquea Sigilo Rápido III en una Ciudad Antigua y combínalo al final en el Yunque.",
                            "6. Final Leggings step (1.19.4+): Loot Swift Sneak III from Ancient Cities and apply last in Anvil.",
                        ).to_string());
                    }

                    for s in steps {
                        ui.add(egui::Label::new(RichText::new(s).size(12.0).color(palette.text_primary)).wrap());
                        ui.add_space(4.0);
                    }
                }
                SurvivalWindow::Java19to1111 => {
                    ui.label(
                        RichText::new(pick(
                            lang,
                            "Método Survival Vanilla (Ventana oficial Java 1.9 - 1.11.1)",
                            "Método Supervivencia Vanilla (Java 1.9 - 1.11.1)",
                            "Pure Vanilla Survival Method (Java 1.9 - 1.11.1 Window)",
                        ))
                        .size(13.5)
                        .strong()
                        .color(palette.accent_gold),
                    );
                    ui.add_space(6.0);
                    for s in [
                        pick(
                            lang,
                            "1. En Java 1.9 hasta 1.11.1 (antes del parche 1.11.1 / 1.11.2), Reparación (Mending) e Infinidad (Infinity) NO eran incompatibles.",
                            "1. Entre Java 1.9 y 1.11.1, Reparación e Infinidad eran compatibles en Survival puro.",
                            "1. From Java 1.9 through 1.11 (patched in 1.11.1), Mending and Infinity had NO mutual conflict.",
                        ),
                        pick(
                            lang,
                            "2. Crea un mundo en Java 1.11, encanta tu arco con Poder V, Retroceso II, Fuego, Irrompibilidad III, Infinidad y Reparación en Yunque.",
                            "2. En un mundo Java 1.11, aplica Poder V, Fuego, Retroceso II, Irrompibilidad III, Infinidad y Reparación en el Yunque.",
                            "2. Enchant a Bow in Java 1.11 with Power V, Punch II, Flame, Unbreaking III, Infinity, and Mending via Anvil.",
                        ),
                        pick(
                            lang,
                            "3. Actualiza el mundo hacia adelante hasta tu versión actual conservando un arco 100% indestructible y con flechas infinitas.",
                            "3. Actualiza el mundo hacia adelante hasta tu versión activa: el arco conserva ambos encantamientos eternamente.",
                            "3. Upgrade your world forward safely to your target version—the bow keeps both enchants permanently.",
                        ),
                    ] {
                        ui.add(egui::Label::new(RichText::new(s).size(12.0).color(palette.text_primary)).wrap());
                        ui.add_space(4.0);
                    }
                }
                SurvivalWindow::CommandsOrDatapackOnly => {
                    ui.label(
                        RichText::new(pick(
                            lang,
                            "No existe método Vanilla Survival puro sin comandos/datapack",
                            "Sin método Supervivencia Vanilla puro",
                            "No Pure Vanilla Survival Glitch Exists for This Item",
                        ))
                        .size(13.5)
                        .strong()
                        .color(palette.accent_enchant),
                    );
                    ui.add_space(6.0);
                    ui.add(
                        egui::Label::new(
                            RichText::new(pick(
                                lang,
                                "A diferencia de la God Armor (1.14.2) o el Arco God (1.11), Minecraft Java nunca tuvo una versión pública donde esta combinación pudiera fusionarse en Survival sin modificar tags. Usa la pestaña 'Comando /give Exacto' para creativo/admin o la pestaña 'Datapack / Servidor' para desbloquearla en yunques.",
                                "A diferencia de la Armadura God (1.14.2) o el Arco (1.11), esta pieza nunca se pudo encantar así en Survival sin datapacks/plugins. Usa 'Comando /give Exacto' o 'Datapack / Servidor'.",
                                "Unlike 1.14.2 God Armor or the 1.11 Bow, no public vanilla release ever allowed anvil-combining these mutually exclusive enchants. Use the Exact /give Command or Datapack / Server tab.",
                            ))
                            .size(12.0)
                            .color(palette.text_primary),
                        )
                        .wrap(),
                    );
                }
            }
        });
}

fn render_give_command_panel(
    ui: &mut egui::Ui,
    piece: &IllegalGodPiece,
    all_pieces: &[IllegalGodPiece],
    rcx: &RenderContext<'_>,
    toasts: &mut ToastQueue,
    active_ver: &str,
    lang: Language,
) {
    let RenderContext {
        palette, audio, ..
    } = *rcx;
    let cmd = build_god_give_command(piece.item_id, &piece.enchants, active_ver);

    egui::Frame::new()
        .fill(palette.bg_card)
        .stroke(Stroke::new(1.0_f32, palette.accent_primary))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(pick(
                    lang,
                    &format!("Comando /give exacto para Java {active_ver}"),
                    &format!("Comando /give exacto para Java {active_ver}"),
                    &format!("Exact /give Command for Java {active_ver}"),
                ))
                .size(13.0)
                .strong()
                .color(palette.accent_gold),
            );
            ui.add_space(6.0);

            egui::Frame::new()
                .fill(palette.slot_bg)
                .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.add(
                        egui::Label::new(
                            RichText::new(&cmd).monospace().size(11.5).color(palette.accent_xp),
                        )
                        .wrap(),
                    );
                });

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if ui
                    .button(pick(
                        lang,
                        "Copiar Comando /give",
                        "Copiar Comando /give",
                        "Copy /give Command",
                    ))
                    .clicked()
                {
                    ui.ctx().copy_text(cmd.clone());
                    audio.play(SoundCue::Toast);
                    toasts.push(
                        pick(lang, "/give Copiado", "/give Copiado", "/give Copied"),
                        format!("{} (Java {})", piece.item_id, active_ver),
                        piece.item_id,
                        palette.accent_xp,
                    );
                }
                if ui
                    .button(pick(
                        lang,
                        "Copiar Set Completo Armadura (4 comandos)",
                        "Copiar Set Completo Armadura (4 comandos)",
                        "Copy Full 4-Piece Armor Set",
                    ))
                    .clicked()
                {
                    let full_set: Vec<String> = all_pieces
                        .iter()
                        .filter(|p| p.is_armor)
                        .map(|p| build_god_give_command(p.item_id, &p.enchants, active_ver))
                        .collect();
                    ui.ctx().copy_text(full_set.join("\n"));
                    audio.play(SoundCue::LevelUp);
                    toasts.push(
                        pick(
                            lang,
                            "Set Completo /give",
                            "Set Completo /give",
                            "Full Armor Set Copied",
                        ),
                        format!("4x Java {}", active_ver),
                        "netherite_chestplate",
                        palette.accent_gold,
                    );
                }
            });
        });
}

fn render_datapack_panel(
    ui: &mut egui::Ui,
    rcx: &RenderContext<'_>,
    toasts: &mut ToastQueue,
    active_ver: &str,
    lang: Language,
) {
    let RenderContext {
        palette, audio, ..
    } = *rcx;
    let supports_exclusive_set = matches!(active_ver, "1.21.1" | "1.21.4" | "26.3");
    egui::Frame::new()
        .fill(palette.bg_card)
        .stroke(Stroke::new(1.0_f32, palette.accent_enchant))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if supports_exclusive_set {
                ui.label(
                    RichText::new(pick(
                        lang,
                        &format!("Datapack Vanilla (Java {active_ver} - Sin Mods)"),
                        &format!("Datapack Vanilla (Java {active_ver} - Sin Mods)"),
                        &format!("Vanilla Datapack (Java {active_ver} - Zero Mods)"),
                    ))
                    .size(13.0)
                    .strong()
                    .color(palette.accent_xp),
                );
                ui.add_space(6.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(pick(
                            lang,
                            "Desde Java 1.21+, las incompatibilidades de encantamientos viven en tags JSON. Creando este archivo dentro de un Datapack desbloqueas las 4 Protecciones juntas en cualquier Yunque Survival:",
                            "Desde Java 1.21+, las incompatibilidades de encantamientos viven en tags JSON. Con este Datapack podrás combinar las 4 Protecciones juntas en el Yunque:",
                            "Since Java 1.21+, enchantment conflicts live inside data-driven JSON tags. Dropping this file inside a Datapack lets you combine all 4 Protections legally in any survival Anvil:",
                        ))
                        .size(12.0)
                        .color(palette.text_primary),
                    )
                    .wrap(),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(
                        "data/minecraft/tags/enchantment/exclusive_set/armor.json",
                    )
                    .monospace()
                    .size(11.0)
                    .color(palette.accent_gold),
                );
                let json_body = "{\n  \"replace\": true,\n  \"values\": []\n}";
                egui::Frame::new()
                    .fill(palette.slot_bg)
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new(json_body).monospace().size(11.5).color(palette.accent_xp));
                    });
                ui.add_space(6.0);
                if ui
                    .button(pick(
                        lang,
                        "Copiar armor.json (exclusive_set)",
                        "Copiar armor.json (exclusive_set)",
                        "Copy armor.json (exclusive_set)",
                    ))
                    .clicked()
                {
                    ui.ctx().copy_text(json_body.to_string());
                    audio.play(SoundCue::Toast);
                    toasts.push(
                        pick(lang, "JSON Copiado", "JSON Copiado", "Datapack JSON Copied"),
                        "exclusive_set/armor.json",
                        "writable_book",
                        palette.accent_xp,
                    );
                }
            } else {
                ui.label(
                    RichText::new(pick(
                        lang,
                        &format!("Servidor / Mods en Java {active_ver}"),
                        &format!("Servidor / Mods en Java {active_ver}"),
                        &format!("Server Plugins / Mods in Java {active_ver}"),
                    ))
                    .size(13.0)
                    .strong()
                    .color(palette.accent_gold),
                );
                ui.add_space(6.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(pick(
                            lang,
                            "Los tags #minecraft:exclusive_set recién existen a partir de Java 1.21. En Java 1.18.2 - 1.20.6 puedes conseguirla en Survival usando el método 1.14.2 + actualización hacia adelante, o con plugins/mods de servidor como 'GodArmor' / 'StackableProtections' / 'UnsafeEnchants'.",
                            "Los tags #minecraft:exclusive_set se añadieron en Java 1.21. En 1.18.2 - 1.20.6 usa la ruta desde 1.14.2 hacia adelante o plugins de servidor como 'GodArmor' / 'StackableProtections'.",
                            "Data-driven #minecraft:exclusive_set tags were introduced in Java 1.21. For Java 1.18.2 - 1.20.6 worlds, use the forward 1.14.2 survival upgrade path or server plugins/mods such as 'GodArmor' / 'StackableProtections'.",
                        ))
                        .size(12.0)
                        .color(palette.text_primary),
                    )
                    .wrap(),
                );
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::load_versions;

    #[test]
    fn god_give_command_matches_exact_mc_syntax_across_eras() {
        let enchs = [("protection", 4), ("fire_protection", 4)];

        let cmd_118 = build_god_give_command("netherite_helmet", &enchs, "1.18.2");
        assert_eq!(
            cmd_118,
            "/give @p minecraft:netherite_helmet{Enchantments:[{id:\"minecraft:protection\",lvl:4s},{id:\"minecraft:fire_protection\",lvl:4s}]} 1"
        );

        let cmd_121 = build_god_give_command("netherite_helmet", &enchs, "1.21.1");
        assert_eq!(
            cmd_121,
            "/give @p minecraft:netherite_helmet[minecraft:enchantments={levels:{\"minecraft:protection\":4,\"minecraft:fire_protection\":4}}] 1"
        );

        let cmd_263 = build_god_give_command("netherite_helmet", &enchs, "26.3");
        assert_eq!(
            cmd_263,
            "/give @p minecraft:netherite_helmet[minecraft:enchantments={\"minecraft:protection\":4,\"minecraft:fire_protection\":4}] 1"
        );
    }

    #[test]
    fn god_pieces_filter_by_version_and_toggle_thorns() {
        let versions = load_versions().expect("versions.json");

        let p_118 = god_pieces_for_version(&versions, "1.18.2", false);
        assert!(!p_118.iter().any(|p| p.id == "god_mace"), "no Mace in 1.18.2");
        let legs_118 = p_118.iter().find(|p| p.id == "god_leggings").expect("leggings");
        assert!(
            !legs_118.enchants.iter().any(|(id, _)| *id == "swift_sneak"),
            "no Swift Sneak in 1.18.2"
        );
        assert!(!legs_118.enchants.iter().any(|(id, _)| *id == "thorns"));

        let p_263 = god_pieces_for_version(&versions, "26.3", true);
        assert!(p_263.iter().any(|p| p.id == "god_mace"), "Mace active in 26.3");
        let legs_263 = p_263.iter().find(|p| p.id == "god_leggings").expect("leggings");
        assert!(legs_263.enchants.iter().any(|(id, _)| *id == "swift_sneak"));
        assert!(legs_263.enchants.iter().any(|(id, _)| *id == "thorns"));
    }
}
