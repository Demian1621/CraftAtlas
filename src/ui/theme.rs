use eframe::egui::{self, Color32, CornerRadius, Pos2, Rect, Stroke, Vec2};

use crate::anim::paint_enchant_glint;
use crate::assets::ItemAtlas;
use crate::state::ThemeId;

/// Unified visual tokens for all switchable Minecraft themes.
#[derive(Debug, Clone, Copy)]
pub struct McPalette {
    pub bg_canvas: Color32,
    pub bg_panel: Color32,
    pub bg_card: Color32,
    pub bg_card_hover: Color32,
    pub slot_bg: Color32,
    pub slot_shadow: Color32,
    pub slot_highlight: Color32,
    pub border_subtle: Color32,
    pub border_strong: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub accent_primary: Color32,
    pub accent_gold: Color32,
    pub accent_enchant: Color32,
    pub accent_xp: Color32,
    pub accent_danger: Color32,
}

impl McPalette {
    pub fn from_theme(id: ThemeId) -> Self {
        match id {
            ThemeId::DeepslateOre => Self {
                bg_canvas: Color32::from_rgba_unmultiplied(12, 15, 22, 92),
                bg_panel: Color32::from_rgba_unmultiplied(17, 21, 30, 218),
                bg_card: Color32::from_rgba_unmultiplied(24, 29, 41, 216),
                bg_card_hover: Color32::from_rgba_unmultiplied(34, 42, 58, 236),
                slot_bg: Color32::from_rgba_unmultiplied(14, 17, 24, 240),
                slot_shadow: Color32::from_rgb(7, 9, 13),
                slot_highlight: Color32::from_rgb(76, 88, 114),
                border_subtle: Color32::from_rgba_unmultiplied(54, 64, 86, 200),
                border_strong: Color32::from_rgb(86, 102, 132),
                text_primary: Color32::from_rgb(238, 242, 250),
                text_muted: Color32::from_rgb(162, 174, 198),
                accent_primary: Color32::from_rgb(56, 224, 198),
                accent_gold: Color32::from_rgb(248, 192, 68),
                accent_enchant: Color32::from_rgb(190, 114, 255),
                accent_xp: Color32::from_rgb(122, 245, 76),
                accent_danger: Color32::from_rgb(238, 86, 86),
            },
            ThemeId::ClassicJavaStone => Self {
                bg_canvas: Color32::from_rgba_unmultiplied(20, 24, 24, 98),
                bg_panel: Color32::from_rgba_unmultiplied(26, 30, 32, 220),
                bg_card: Color32::from_rgba_unmultiplied(36, 42, 44, 218),
                bg_card_hover: Color32::from_rgba_unmultiplied(48, 56, 58, 238),
                slot_bg: Color32::from_rgba_unmultiplied(22, 25, 27, 240),
                slot_shadow: Color32::from_rgb(12, 14, 15),
                slot_highlight: Color32::from_rgb(108, 118, 122),
                border_subtle: Color32::from_rgba_unmultiplied(72, 84, 84, 200),
                border_strong: Color32::from_rgb(112, 128, 126),
                text_primary: Color32::from_rgb(244, 246, 245),
                text_muted: Color32::from_rgb(180, 190, 186),
                accent_primary: Color32::from_rgb(88, 226, 128),
                accent_gold: Color32::from_rgb(252, 206, 74),
                accent_enchant: Color32::from_rgb(198, 132, 255),
                accent_xp: Color32::from_rgb(138, 255, 72),
                accent_danger: Color32::from_rgb(245, 92, 92),
            },
            ThemeId::NetherCrimsonEnd => Self {
                bg_canvas: Color32::from_rgba_unmultiplied(20, 10, 16, 94),
                bg_panel: Color32::from_rgba_unmultiplied(28, 15, 22, 220),
                bg_card: Color32::from_rgba_unmultiplied(39, 20, 30, 218),
                bg_card_hover: Color32::from_rgba_unmultiplied(54, 28, 42, 238),
                slot_bg: Color32::from_rgba_unmultiplied(22, 11, 18, 240),
                slot_shadow: Color32::from_rgb(10, 4, 8),
                slot_highlight: Color32::from_rgb(112, 62, 84),
                border_subtle: Color32::from_rgba_unmultiplied(84, 42, 58, 205),
                border_strong: Color32::from_rgb(136, 70, 92),
                text_primary: Color32::from_rgb(248, 238, 242),
                text_muted: Color32::from_rgb(198, 164, 178),
                accent_primary: Color32::from_rgb(255, 118, 74),
                accent_gold: Color32::from_rgb(255, 194, 68),
                accent_enchant: Color32::from_rgb(220, 134, 255),
                accent_xp: Color32::from_rgb(132, 244, 108),
                accent_danger: Color32::from_rgb(255, 86, 106),
            },
            ThemeId::EndVoidAurora => Self {
                bg_canvas: Color32::from_rgba_unmultiplied(12, 10, 24, 94),
                bg_panel: Color32::from_rgba_unmultiplied(19, 16, 36, 218),
                bg_card: Color32::from_rgba_unmultiplied(27, 22, 50, 216),
                bg_card_hover: Color32::from_rgba_unmultiplied(38, 32, 68, 236),
                slot_bg: Color32::from_rgba_unmultiplied(15, 12, 28, 240),
                slot_shadow: Color32::from_rgb(7, 5, 14),
                slot_highlight: Color32::from_rgb(108, 84, 168),
                border_subtle: Color32::from_rgba_unmultiplied(74, 58, 122, 205),
                border_strong: Color32::from_rgb(116, 92, 184),
                text_primary: Color32::from_rgb(242, 238, 252),
                text_muted: Color32::from_rgb(174, 166, 206),
                accent_primary: Color32::from_rgb(94, 232, 212),
                accent_gold: Color32::from_rgb(248, 198, 92),
                accent_enchant: Color32::from_rgb(214, 124, 255),
                accent_xp: Color32::from_rgb(132, 248, 122),
                accent_danger: Color32::from_rgb(248, 96, 132),
            },
            ThemeId::CherryBlossom => Self {
                bg_canvas: Color32::from_rgba_unmultiplied(24, 16, 24, 108),
                bg_panel: Color32::from_rgba_unmultiplied(32, 21, 32, 222),
                bg_card: Color32::from_rgba_unmultiplied(44, 29, 44, 220),
                bg_card_hover: Color32::from_rgba_unmultiplied(58, 39, 58, 238),
                slot_bg: Color32::from_rgba_unmultiplied(24, 15, 24, 240),
                slot_shadow: Color32::from_rgb(12, 7, 13),
                slot_highlight: Color32::from_rgb(168, 108, 138),
                border_subtle: Color32::from_rgba_unmultiplied(98, 62, 88, 205),
                border_strong: Color32::from_rgb(156, 98, 138),
                text_primary: Color32::from_rgb(252, 242, 246),
                text_muted: Color32::from_rgb(212, 178, 196),
                accent_primary: Color32::from_rgb(255, 148, 194),
                accent_gold: Color32::from_rgb(255, 206, 98),
                accent_enchant: Color32::from_rgb(218, 146, 255),
                accent_xp: Color32::from_rgb(148, 242, 118),
                accent_danger: Color32::from_rgb(255, 96, 116),
            },
            ThemeId::StarryOverworld => Self {
                bg_canvas: Color32::from_rgba_unmultiplied(10, 15, 26, 96),
                bg_panel: Color32::from_rgba_unmultiplied(15, 22, 36, 218),
                bg_card: Color32::from_rgba_unmultiplied(21, 31, 50, 216),
                bg_card_hover: Color32::from_rgba_unmultiplied(30, 44, 68, 236),
                slot_bg: Color32::from_rgba_unmultiplied(12, 18, 30, 240),
                slot_shadow: Color32::from_rgb(6, 9, 16),
                slot_highlight: Color32::from_rgb(76, 110, 158),
                border_subtle: Color32::from_rgba_unmultiplied(52, 76, 112, 205),
                border_strong: Color32::from_rgb(84, 120, 174),
                text_primary: Color32::from_rgb(238, 244, 252),
                text_muted: Color32::from_rgb(162, 184, 212),
                accent_primary: Color32::from_rgb(110, 212, 255),
                accent_gold: Color32::from_rgb(255, 212, 92),
                accent_enchant: Color32::from_rgb(188, 138, 255),
                accent_xp: Color32::from_rgb(128, 245, 112),
                accent_danger: Color32::from_rgb(245, 92, 96),
            },
            ThemeId::DeepDarkSculk => Self {
                bg_canvas: Color32::from_rgba_unmultiplied(7, 13, 18, 86),
                bg_panel: Color32::from_rgba_unmultiplied(11, 20, 27, 218),
                bg_card: Color32::from_rgba_unmultiplied(16, 28, 37, 216),
                bg_card_hover: Color32::from_rgba_unmultiplied(23, 40, 52, 236),
                slot_bg: Color32::from_rgba_unmultiplied(8, 14, 20, 240),
                slot_shadow: Color32::from_rgb(4, 7, 11),
                slot_highlight: Color32::from_rgb(56, 124, 138),
                border_subtle: Color32::from_rgba_unmultiplied(32, 74, 86, 205),
                border_strong: Color32::from_rgb(46, 146, 162),
                text_primary: Color32::from_rgb(234, 248, 250),
                text_muted: Color32::from_rgb(148, 188, 198),
                accent_primary: Color32::from_rgb(38, 224, 212),
                accent_gold: Color32::from_rgb(242, 198, 92),
                accent_enchant: Color32::from_rgb(168, 132, 255),
                accent_xp: Color32::from_rgb(94, 242, 162),
                accent_danger: Color32::from_rgb(245, 92, 96),
            },
            ThemeId::TrialChambers => Self {
                bg_canvas: Color32::from_rgba_unmultiplied(12, 18, 19, 88),
                bg_panel: Color32::from_rgba_unmultiplied(18, 27, 28, 218),
                bg_card: Color32::from_rgba_unmultiplied(26, 38, 39, 216),
                bg_card_hover: Color32::from_rgba_unmultiplied(37, 52, 53, 236),
                slot_bg: Color32::from_rgba_unmultiplied(14, 21, 22, 240),
                slot_shadow: Color32::from_rgb(7, 11, 12),
                slot_highlight: Color32::from_rgb(128, 98, 72),
                border_subtle: Color32::from_rgba_unmultiplied(64, 96, 90, 205),
                border_strong: Color32::from_rgb(212, 126, 68),
                text_primary: Color32::from_rgb(248, 244, 238),
                text_muted: Color32::from_rgb(176, 196, 190),
                accent_primary: Color32::from_rgb(72, 214, 178),
                accent_gold: Color32::from_rgb(252, 162, 64),
                accent_enchant: Color32::from_rgb(198, 138, 255),
                accent_xp: Color32::from_rgb(144, 242, 102),
                accent_danger: Color32::from_rgb(245, 92, 86),
            },
            ThemeId::PaleGarden => Self {
                bg_canvas: Color32::from_rgba_unmultiplied(16, 18, 22, 90),
                bg_panel: Color32::from_rgba_unmultiplied(24, 26, 31, 220),
                bg_card: Color32::from_rgba_unmultiplied(33, 36, 42, 218),
                bg_card_hover: Color32::from_rgba_unmultiplied(46, 50, 58, 238),
                slot_bg: Color32::from_rgba_unmultiplied(18, 20, 24, 240),
                slot_shadow: Color32::from_rgb(9, 10, 13),
                slot_highlight: Color32::from_rgb(122, 124, 132),
                border_subtle: Color32::from_rgba_unmultiplied(78, 82, 92, 205),
                border_strong: Color32::from_rgb(148, 138, 124),
                text_primary: Color32::from_rgb(244, 244, 246),
                text_muted: Color32::from_rgb(184, 186, 194),
                accent_primary: Color32::from_rgb(242, 168, 56),
                accent_gold: Color32::from_rgb(255, 196, 74),
                accent_enchant: Color32::from_rgb(194, 146, 255),
                accent_xp: Color32::from_rgb(154, 228, 128),
                accent_danger: Color32::from_rgb(244, 98, 92),
            },
            ThemeId::OceanMonument => Self {
                bg_canvas: Color32::from_rgba_unmultiplied(6, 16, 28, 86),
                bg_panel: Color32::from_rgba_unmultiplied(10, 26, 42, 216),
                bg_card: Color32::from_rgba_unmultiplied(15, 36, 56, 214),
                bg_card_hover: Color32::from_rgba_unmultiplied(22, 50, 76, 234),
                slot_bg: Color32::from_rgba_unmultiplied(8, 20, 34, 240),
                slot_shadow: Color32::from_rgb(4, 10, 18),
                slot_highlight: Color32::from_rgb(76, 168, 178),
                border_subtle: Color32::from_rgba_unmultiplied(44, 104, 132, 205),
                border_strong: Color32::from_rgb(74, 208, 198),
                text_primary: Color32::from_rgb(236, 250, 252),
                text_muted: Color32::from_rgb(158, 198, 214),
                accent_primary: Color32::from_rgb(76, 228, 214),
                accent_gold: Color32::from_rgb(252, 208, 88),
                accent_enchant: Color32::from_rgb(176, 146, 255),
                accent_xp: Color32::from_rgb(112, 246, 168),
                accent_danger: Color32::from_rgb(246, 98, 96),
            },
            ThemeId::CustomImage => Self {
                bg_canvas: Color32::from_rgba_unmultiplied(10, 12, 18, 92),
                bg_panel: Color32::from_rgba_unmultiplied(15, 18, 26, 222),
                bg_card: Color32::from_rgba_unmultiplied(22, 26, 36, 220),
                bg_card_hover: Color32::from_rgba_unmultiplied(32, 38, 52, 238),
                slot_bg: Color32::from_rgba_unmultiplied(12, 15, 22, 242),
                slot_shadow: Color32::from_rgb(6, 7, 11),
                slot_highlight: Color32::from_rgb(92, 106, 132),
                border_subtle: Color32::from_rgba_unmultiplied(64, 74, 96, 205),
                border_strong: Color32::from_rgb(98, 122, 164),
                text_primary: Color32::from_rgb(244, 246, 252),
                text_muted: Color32::from_rgb(172, 184, 204),
                accent_primary: Color32::from_rgb(78, 226, 184),
                accent_gold: Color32::from_rgb(252, 202, 74),
                accent_enchant: Color32::from_rgb(202, 136, 255),
                accent_xp: Color32::from_rgb(126, 246, 86),
                accent_danger: Color32::from_rgb(244, 92, 96),
            },
        }
    }

    pub fn apply_to_egui(self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = self.bg_canvas;
        visuals.window_fill = Color32::from_rgba_unmultiplied(
            self.bg_panel.r(),
            self.bg_panel.g(),
            self.bg_panel.b(),
            246,
        );
        visuals.extreme_bg_color = self.slot_bg;
        visuals.faint_bg_color = self.bg_card;
        visuals.code_bg_color = self.slot_bg;
        visuals.override_text_color = Some(self.text_primary);

        visuals.widgets.noninteractive.bg_fill = self.bg_card;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, self.border_subtle);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, self.text_primary);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);

        visuals.widgets.inactive.bg_fill = self.bg_card;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, self.border_subtle);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, self.text_primary);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(4);

        visuals.widgets.hovered.bg_fill = self.bg_card_hover;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, self.accent_primary);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(4);

        visuals.widgets.active.bg_fill = self.bg_card_hover;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, self.accent_gold);
        visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
        visuals.widgets.active.corner_radius = CornerRadius::same(4);

        visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(
            self.accent_primary.r(),
            self.accent_primary.g(),
            self.accent_primary.b(),
            48,
        );
        visuals.selection.stroke = Stroke::new(1.5_f32, self.accent_primary);

        ctx.set_visuals(visuals);
    }
}

/// Computes an adaptive sidebar width that stays readable at 760px and spacious at 1440px+.
pub fn adaptive_sidebar_width(total_width: f32) -> f32 {
    if total_width < 880.0 {
        220.0
    } else if total_width < 1200.0 {
        265.0
    } else {
        300.0
    }
}

/// Paints a Minecraft GUI container frame with subtle inner highlight and clean border.
pub fn paint_mc_frame(
    painter: &egui::Painter,
    rect: Rect,
    palette: McPalette,
    border_accent: Option<Color32>,
) {
    painter.rect_filled(rect, 5.0, palette.bg_panel);
    painter.rect_stroke(
        rect,
        5.0,
        Stroke::new(1.5_f32, border_accent.unwrap_or(palette.border_strong)),
        egui::StrokeKind::Inside,
    );
    painter.rect_stroke(
        rect.shrink(1.5),
        4.0,
        Stroke::new(1.0_f32, palette.slot_shadow),
        egui::StrokeKind::Inside,
    );
}

/// Visual presentation options for [`paint_mc_slot`].
#[derive(Clone, Copy)]
pub struct SlotStyle {
    pub count: u32,
    pub selected: bool,
    pub hovered: bool,
    pub glint_time: Option<f64>,
}

impl Default for SlotStyle {
    fn default() -> Self {
        Self {
            count: 1,
            selected: false,
            hovered: false,
            glint_time: None,
        }
    }
}

impl SlotStyle {
    pub fn hovered(hovered: bool) -> Self {
        Self {
            hovered,
            ..Default::default()
        }
    }

    pub fn selected(selected: bool) -> Self {
        Self {
            selected,
            ..Default::default()
        }
    }
}

/// Renders an authentic 3D beveled Minecraft inventory slot with optional hover pickup glow,
/// item count stack badge, and diagonal Enchantment Glint animation.
pub fn paint_mc_slot(
    painter: &egui::Painter,
    rect: Rect,
    palette: McPalette,
    atlas: &ItemAtlas,
    item_id: Option<&str>,
    style: SlotStyle,
) {
    let SlotStyle {
        count,
        selected,
        hovered,
        glint_time,
    } = style;
    // Inset bevel fill
    painter.rect_filled(rect, 2.0, palette.slot_bg);

    // Classic MC 3D slot bevel: dark top+left, bright bottom+right
    let tl = rect.min;
    let tr = Pos2::new(rect.right(), rect.top());
    let bl = Pos2::new(rect.left(), rect.bottom());
    let br = rect.max;
    painter.line_segment([tl, tr], Stroke::new(1.5_f32, palette.slot_shadow));
    painter.line_segment([tl, bl], Stroke::new(1.5_f32, palette.slot_shadow));
    painter.line_segment([bl, br], Stroke::new(1.2_f32, palette.slot_highlight));
    painter.line_segment([tr, br], Stroke::new(1.2_f32, palette.slot_highlight));

    if selected {
        painter.rect_stroke(
            rect,
            2.0,
            Stroke::new(2.0_f32, palette.accent_primary),
            egui::StrokeKind::Outside,
        );
    }

    if let Some(id) = item_id {
        if !id.is_empty() {
            let pad = if hovered { 3.0 } else { 4.5 };
            let sprite_rect = rect.shrink(pad);
            atlas.paint(painter, sprite_rect, id, Color32::WHITE);

            if let Some(t) = glint_time {
                paint_enchant_glint(painter, rect.shrink(1.5), t, 0.85);
            }

            if count > 1 {
                let text = count.to_string();
                let pos = Pos2::new(rect.right() - 2.5, rect.bottom() - 1.0);
                let font = egui::FontId::monospace((rect.height() * 0.30).clamp(10.0, 14.0));
                painter.text(
                    pos + Vec2::new(1.0, 1.0),
                    egui::Align2::RIGHT_BOTTOM,
                    &text,
                    font.clone(),
                    Color32::BLACK,
                );
                painter.text(
                    pos,
                    egui::Align2::RIGHT_BOTTOM,
                    &text,
                    font,
                    Color32::WHITE,
                );
            }
        }
    }

    if hovered {
        painter.rect_filled(
            rect.shrink(1.5),
            1.5,
            Color32::from_rgba_unmultiplied(255, 255, 255, 42),
        );
    }
}

/// Renders an authentic Minecraft Java Edition purple-bordered floating item tooltip
/// anchored neatly beside hovered items without stealing clicks.
pub fn show_mc_tooltip_at(
    ctx: &egui::Context,
    anchor: Pos2,
    title: &str,
    sublines: &[(&str, Color32)],
    palette: McPalette,
) {
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        egui::Id::new("mc_custom_tooltip"),
    ));
    let screen = ctx.screen_rect();

    let title_galley = painter.layout_no_wrap(
        title.to_string(),
        egui::FontId::proportional(13.5),
        Color32::WHITE,
    );
    let mut max_w = title_galley.size().x;
    let mut line_galleys = Vec::with_capacity(sublines.len());
    for &(line, col) in sublines {
        let g = painter.layout_no_wrap(
            line.to_string(),
            egui::FontId::proportional(11.5),
            col,
        );
        max_w = max_w.max(g.size().x);
        line_galleys.push((g, col));
    }

    let pad_x = 10.0;
    let pad_y = 7.0;
    let w = (max_w + pad_x * 2.0).max(90.0);
    let mut h = pad_y * 2.0 + title_galley.size().y;
    for (g, _) in &line_galleys {
        h += 3.0 + g.size().y;
    }

    let mut pos = anchor + Vec2::new(14.0, -h - 8.0);
    if pos.x + w > screen.right() - 8.0 {
        pos.x = (screen.right() - w - 8.0).max(screen.left() + 8.0);
    }
    if pos.y < screen.top() + 8.0 {
        pos.y = (anchor.y + 18.0).min(screen.bottom() - h - 8.0);
    }

    let rect = Rect::from_min_size(pos, Vec2::new(w, h));
    painter.rect_filled(rect, 3.0, Color32::from_rgba_unmultiplied(16, 0, 16, 244));
    painter.rect_stroke(
        rect,
        3.0,
        Stroke::new(1.5_f32, Color32::from_rgb(80, 0, 255)),
        egui::StrokeKind::Inside,
    );
    painter.rect_stroke(
        rect.shrink(1.5),
        2.0,
        Stroke::new(1.0_f32, Color32::from_rgb(40, 0, 127)),
        egui::StrokeKind::Inside,
    );

    let mut cy = rect.top() + pad_y;
    painter.galley(Pos2::new(rect.left() + pad_x, cy), title_galley, palette.text_primary);
    cy += 18.0;
    for (g, col) in line_galleys {
        let lh = g.size().y;
        painter.galley(Pos2::new(rect.left() + pad_x, cy), g, col);
        cy += lh + 3.0;
    }
}

/// Interactive pill filter chip with optional crisp atlas sprite.
pub fn filter_chip(
    ui: &mut egui::Ui,
    palette: McPalette,
    atlas: &ItemAtlas,
    icon_id: Option<&str>,
    label: &str,
    selected: bool,
) -> egui::Response {
    let fill = if selected {
        Color32::from_rgba_unmultiplied(
            palette.accent_primary.r(),
            palette.accent_primary.g(),
            palette.accent_primary.b(),
            44,
        )
    } else {
        palette.bg_card
    };
    let stroke = if selected {
        Stroke::new(1.5_f32, palette.accent_primary)
    } else {
        Stroke::new(1.0_f32, palette.border_subtle)
    };
    let text_col = if selected {
        Color32::WHITE
    } else {
        palette.text_muted
    };

    let desired_h = 26.0;
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(12.0),
        text_col,
    );
    let icon_w = if icon_id.is_some() { 18.0 } else { 0.0 };
    let desired_w = (galley.size().x + icon_w + 18.0).max(46.0);

    let (rect, resp) = ui.allocate_exact_size(Vec2::new(desired_w, desired_h), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let bg = if resp.hovered() && !selected {
            palette.bg_card_hover
        } else {
            fill
        };
        ui.painter().rect_filled(rect, 4.0, bg);
        ui.painter().rect_stroke(rect, 4.0, stroke, egui::StrokeKind::Inside);

        let mut tx = rect.left() + 8.0;
        if let Some(icon) = icon_id {
            let irec = Rect::from_center_size(
                Pos2::new(tx + 7.0, rect.center().y),
                Vec2::splat(15.0),
            );
            atlas.paint(ui.painter(), irec, icon, Color32::WHITE);
            tx += 19.0;
        }
        let ty = rect.center().y - galley.size().y * 0.5;
        ui.painter().galley(Pos2::new(tx, ty), galley, text_col);
    }
    resp
}
