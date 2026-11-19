use eframe::egui::{
    self, Align, Color32, CornerRadius, Layout, Margin, Pos2, RichText, Sense, Stroke, Vec2,
};

use crate::anim::ToastQueue;
use crate::assets::backgrounds::{
    has_custom_backdrop, import_custom_backdrop, remove_custom_backdrop, ThemeBackgrounds,
};
use crate::assets::ItemAtlas;
use crate::audio::{AudioEngine, SoundCue};
use crate::data::models::Language;
use crate::state::{AppState, ThemeId};
use crate::ui::theme::{filter_chip, McPalette};

fn pick_txt(lang: Language, mx: &'static str, es: &'static str, en: &'static str) -> &'static str {
    match lang {
        Language::EsMx => mx,
        Language::EsEs => es,
        Language::EnUs => en,
    }
}

/// Centered Visual Theme Gallery modal with live 16:9 previews, color swatches,
/// background intensity slider, and native `.png`/`.jpg` custom wallpaper picker.
pub fn render_theme_modal(
    ctx: &egui::Context,
    state: &mut AppState,
    backgrounds: &mut ThemeBackgrounds,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
    toasts: &mut ToastQueue,
) {
    if !state.theme_modal_open {
        return;
    }
    let lang = state.prefs.language;

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.theme_modal_open = false;
        return;
    }

    let screen = ctx.screen_rect();
    // Subtle dimmer keeping live 30 FPS theme + FX clearly visible behind the gallery
    egui::Area::new(egui::Id::new("mc_theme_gallery_backdrop"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (_r, bg_resp) = ui.allocate_exact_size(screen.size(), Sense::click());
            ui.painter()
                .rect_filled(screen, 0.0, Color32::from_black_alpha(95));
            if bg_resp.clicked() {
                state.theme_modal_open = false;
            }
        });

    let modal_w = (screen.width() - 56.0).clamp(500.0, 860.0);
    let est_h = (screen.height() - 56.0).clamp(440.0, 680.0);
    let pos = Pos2::new(
        screen.center().x - modal_w * 0.5,
        (screen.center().y - est_h * 0.5).max(28.0),
    );

    egui::Area::new(egui::Id::new("mc_theme_gallery_modal"))
        .order(egui::Order::Tooltip)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(Color32::from_rgba_unmultiplied(
                    palette.bg_panel.r(),
                    palette.bg_panel.g(),
                    palette.bg_panel.b(),
                    244,
                ))
                .stroke(Stroke::new(1.5_f32, palette.accent_primary))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.set_width(modal_w);

                    // Header row
                    ui.horizontal(|ui| {
                        ui.set_width(modal_w);
                        ui.add(atlas.ui_image("painting", 18.0));
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(pick_txt(
                                    lang,
                                    "Galería Visual de Temas",
                                    "Galería Visual de Temas",
                                    "Visual Theme Gallery",
                                ))
                                .size(15.5)
                                .strong()
                                .color(palette.text_primary),
                            );
                            ui.label(
                                RichText::new(pick_txt(
                                    lang,
                                    "Vista previa en vivo a 30 FPS · Elige escena, paleta e intensidad del fondo",
                                    "Vista previa en directo a 30 FPS · Elige escena, paleta e intensidad del fondo",
                                    "Live 30 FPS preview · Pick biome scenery, color palette & backdrop intensity",
                                ))
                                .size(11.0)
                                .color(palette.text_muted),
                            );
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if filter_chip(
                                ui,
                                palette,
                                atlas,
                                None,
                                pick_txt(lang, "Cerrar", "Cerrar", "Close"),
                                false,
                            )
                            .clicked()
                            {
                                state.theme_modal_open = false;
                                audio.play(SoundCue::Click);
                            }
                        });
                    });

                    ui.add_space(10.0);

                    // 10 Built-in Theme Thumbnail Grid
                    let grid_max_h = (screen.height() - 255.0).clamp(240.0, 440.0);
                    let cols = if modal_w >= 720.0 { 3 } else { 2 };
                    let gap = 10.0_f32;
                    let card_w = ((modal_w - 18.0) - gap * ((cols - 1) as f32)) / (cols as f32);

                    egui::ScrollArea::vertical()
                        .id_salt("theme_gallery_grid_scroll")
                        .max_height(grid_max_h)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let themes = ThemeId::builtin();
                            for row in themes.chunks(cols) {
                                ui.horizontal_top(|ui| {
                                    ui.spacing_mut().item_spacing = Vec2::new(gap, gap);
                                    for &th in row {
                                        render_theme_card(
                                            ui,
                                            state,
                                            backgrounds,
                                            palette,
                                            th,
                                            card_w,
                                            lang,
                                            audio,
                                        );
                                    }
                                });
                                ui.add_space(gap);
                            }
                        });

                    ui.add_space(10.0);

                    // Bottom Customization Controls: Intensity Slider + Custom Image Wallpaper
                    egui::Frame::new()
                        .fill(palette.bg_card)
                        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(Margin::same(11))
                        .show(ui, |ui| {
                            let bottom_w = (modal_w - 22.0).max(240.0);
                            ui.set_width(bottom_w);

                            // Background intensity slider row
                            ui.horizontal(|ui| {
                                ui.set_width(bottom_w);
                                ui.spacing_mut().slider_width = 220.0;
                                {
                                    let vis = &mut ui.style_mut().visuals;
                                    vis.slider_trailing_fill = true;
                                    vis.widgets.inactive.bg_fill = palette.slot_bg;
                                    vis.widgets.inactive.bg_stroke =
                                        Stroke::new(1.0_f32, palette.border_strong);
                                    vis.selection.bg_fill = palette.accent_primary;
                                }
                                ui.add(atlas.ui_image("glowstone_dust", 15.0));
                                ui.label(
                                    RichText::new(pick_txt(
                                        lang,
                                        "Intensidad del fondo:",
                                        "Intensidad del fondo:",
                                        "Backdrop brightness:",
                                    ))
                                    .size(12.0)
                                    .strong()
                                    .color(palette.text_primary),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "{:.0}%",
                                        state.prefs.bg_intensity.clamp(0.20, 1.0) * 100.0
                                    ))
                                    .size(12.0)
                                    .monospace()
                                    .strong()
                                    .color(palette.accent_gold),
                                );
                                let resp = ui.add(
                                    egui::Slider::new(&mut state.prefs.bg_intensity, 0.20..=1.0)
                                        .show_value(false),
                                );
                                if resp.drag_stopped() || (resp.changed() && !resp.dragged()) {
                                    state.prefs.save();
                                }
                            });

                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(6.0);

                            // Native custom wallpaper (.png / .jpg) strip
                            let custom_exists = has_custom_backdrop();
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = Vec2::new(8.0, 6.0);
                                if custom_exists {
                                    if let Some(tex_id) =
                                        backgrounds.thumbnail_for(ctx, ThemeId::CustomImage)
                                    {
                                        ui.add(
                                            egui::Image::new((tex_id, Vec2::new(64.0, 36.0)))
                                                .corner_radius(CornerRadius::same(4)),
                                        );
                                    }
                                } else {
                                    ui.add(atlas.ui_image("item_frame", 18.0));
                                }

                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(pick_txt(
                                            lang,
                                            "Mi Fondo Propio (.png / .jpg)",
                                            "Mi Fondo Propio (.png / .jpg)",
                                            "Custom Wallpaper (.png / .jpg)",
                                        ))
                                        .size(12.0)
                                        .strong()
                                        .color(palette.text_primary),
                                    );
                                    ui.label(
                                        RichText::new(pick_txt(
                                            lang,
                                            "Usá cualquier captura de tu mundo o shaderpack (100% local)",
                                            "Usa cualquier captura de tu mundo o shaderpack (100% local)",
                                            "Use any world or shaderpack screenshot (100% local)",
                                        ))
                                        .size(10.5)
                                        .color(palette.text_muted),
                                    );
                                });

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if custom_exists
                                        && filter_chip(
                                            ui,
                                            palette,
                                            atlas,
                                            Some("barrier"),
                                            pick_txt(lang, "Quitar", "Quitar", "Remove"),
                                            false,
                                        )
                                        .clicked()
                                    {
                                        let _ = remove_custom_backdrop();
                                        backgrounds.reload_custom_backdrop();
                                        if state.prefs.theme == ThemeId::CustomImage {
                                            state.prefs.theme = ThemeId::DeepDarkSculk;
                                            state.prefs.save();
                                        }
                                        state.custom_bg_error = None;
                                        audio.play(SoundCue::Click);
                                    }

                                    if filter_chip(
                                        ui,
                                        palette,
                                        atlas,
                                        Some("ender_eye"),
                                        pick_txt(
                                            lang,
                                            "Elegir imagen...",
                                            "Elegir imagen...",
                                            "Choose Image...",
                                        ),
                                        false,
                                    )
                                    .clicked()
                                    {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .add_filter("Images (PNG / JPG)", &["png", "jpg", "jpeg"])
                                            .pick_file()
                                        {
                                            match import_custom_backdrop(&path) {
                                                Ok(saved) => {
                                                    backgrounds.reload_custom_backdrop();
                                                    state.prefs.theme = ThemeId::CustomImage;
                                                    state.prefs.save();
                                                    state.custom_bg_error = None;
                                                    audio.play(SoundCue::LevelUp);
                                                    let name = saved
                                                        .file_name()
                                                        .map(|n| n.to_string_lossy().into_owned())
                                                        .unwrap_or_default();
                                                    toasts.push(
                                                        "painting",
                                                        pick_txt(
                                                            lang,
                                                            "Fondo Personalizado Activo",
                                                            "Fondo Personalizado Activo",
                                                            "Custom Wallpaper Active",
                                                        ),
                                                        &name,
                                                        palette.accent_xp,
                                                    );
                                                }
                                                Err(err) => {
                                                    state.custom_bg_error = Some(err);
                                                    audio.play(SoundCue::Toast);
                                                }
                                            }
                                        }
                                    }

                                    if custom_exists
                                        && filter_chip(
                                            ui,
                                            palette,
                                            atlas,
                                            Some("painting"),
                                            pick_txt(lang, "Usar Mi Fondo", "Usar Mi Fondo", "Use Custom"),
                                            state.prefs.theme == ThemeId::CustomImage,
                                        )
                                        .clicked()
                                    {
                                        state.prefs.theme = ThemeId::CustomImage;
                                        state.prefs.save();
                                        audio.play(SoundCue::Click);
                                    }
                                });
                            });

                            if let Some(ref err) = state.custom_bg_error {
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(err)
                                        .size(11.0)
                                        .color(palette.accent_danger),
                                );
                            }
                        });
                });
        });
}

#[allow(clippy::too_many_arguments)]
fn render_theme_card(
    ui: &mut egui::Ui,
    state: &mut AppState,
    backgrounds: &mut ThemeBackgrounds,
    palette: McPalette,
    th: ThemeId,
    card_w: f32,
    lang: Language,
    audio: &AudioEngine,
) {
    let selected = state.prefs.theme == th;
    let th_pal = McPalette::from_theme(th);
    let inner_w = (card_w - 16.0).max(120.0);
    let thumb_h = (inner_w * 9.0 / 16.0).round();

    let border_col = if selected {
        th_pal.accent_primary
    } else {
        palette.border_subtle
    };
    let fill_col = if selected {
        Color32::from_rgba_unmultiplied(
            th_pal.bg_card_hover.r(),
            th_pal.bg_card_hover.g(),
            th_pal.bg_card_hover.b(),
            238,
        )
    } else {
        palette.bg_card
    };

    let resp = egui::Frame::new()
        .fill(fill_col)
        .stroke(Stroke::new(if selected { 2.0_f32 } else { 1.0_f32 }, border_col))
        .corner_radius(CornerRadius::same(7))
        .inner_margin(Margin::same(7))
        .show(ui, |ui| {
            // Explicit vertical layout overrides parent horizontal_top row cleanly
            ui.vertical(|ui| {
                ui.set_width(inner_w);
                if let Some(tex_id) = backgrounds.thumbnail_for(ui.ctx(), th) {
                    ui.add(
                        egui::Image::new((tex_id, Vec2::new(inner_w, thumb_h)))
                            .corner_radius(CornerRadius::same(5)),
                    );
                } else {
                    let (r, _) =
                        ui.allocate_exact_size(Vec2::new(inner_w, thumb_h), Sense::hover());
                    ui.painter().rect_filled(r, CornerRadius::same(5), th_pal.bg_panel);
                }
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.set_width(inner_w);
                    ui.label(
                        RichText::new(th.label(lang))
                            .size(11.5)
                            .strong()
                            .color(if selected {
                                th_pal.accent_primary
                            } else {
                                palette.text_primary
                            }),
                    );
                    if selected {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            egui::Frame::new()
                                .fill(Color32::from_rgba_unmultiplied(
                                    th_pal.accent_primary.r(),
                                    th_pal.accent_primary.g(),
                                    th_pal.accent_primary.b(),
                                    42,
                                ))
                                .stroke(Stroke::new(1.0_f32, th_pal.accent_primary))
                                .corner_radius(CornerRadius::same(4))
                                .inner_margin(Margin::symmetric(5, 1))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(pick_txt(lang, "ACTIVO", "ACTIVO", "ACTIVE"))
                                            .size(9.0)
                                            .strong()
                                            .color(th_pal.accent_primary),
                                    );
                                });
                        });
                    }
                });

                ui.horizontal(|ui| {
                    ui.set_width(inner_w);
                    ui.label(
                        RichText::new(th.fx_badge(lang))
                            .size(10.0)
                            .color(palette.text_muted),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let (sw_rect, _) =
                            ui.allocate_exact_size(Vec2::new(38.0, 12.0), Sense::hover());
                        for (idx, col) in [
                            th_pal.accent_enchant,
                            th_pal.accent_gold,
                            th_pal.accent_primary,
                        ]
                        .iter()
                        .enumerate()
                        {
                            let cx = sw_rect.right() - 6.0 - (idx as f32) * 12.0;
                            let c = Pos2::new(cx, sw_rect.center().y);
                            ui.painter().circle_filled(c, 4.5, *col);
                            ui.painter()
                                .circle_stroke(c, 4.5, Stroke::new(1.0_f32, palette.slot_shadow));
                        }
                    });
                });
            });
        })
        .response
        .interact(Sense::click());

    if resp.hovered() {
        ui.painter().rect_stroke(
            resp.rect,
            CornerRadius::same(7),
            Stroke::new(1.5_f32, th_pal.accent_primary),
            egui::StrokeKind::Inside,
        );
    }
    if resp.clicked() {
        state.prefs.theme = th;
        state.prefs.save();
        audio.play(SoundCue::Click);
    }
}
