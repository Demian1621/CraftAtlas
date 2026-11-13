use eframe::egui::{self, Color32, Pos2, Rect, Shape, Vec2};

/// Paints a diagonal shimmering Minecraft Enchantment Glint sweep clipped inside `rect`.
/// Uses smooth periodic triangular interpolation so it loops seamlessly without sudden pops.
pub fn paint_enchant_glint(
    painter: &egui::Painter,
    rect: Rect,
    time_sec: f64,
    intensity: f32,
) {
    if intensity <= 0.01 {
        return;
    }

    let sub = painter.with_clip_rect(rect);
    let w = rect.width();
    let h = rect.height();

    // Two opposing diagonal glint passes mimicking vanilla glint_entity.png
    let passes = [(2.6_f64, 0.24_f32, 0.0_f64), (3.9_f64, 0.16_f32, 0.45_f64)];
    for (period, alpha_mul, offset) in passes {
        let phase = ((time_sec / period + offset).fract()) as f32;
        // Sweep across -0.6..1.6 horizontally
        let cx = rect.left() + (phase * 2.2 - 0.6) * w;
        let band_w = (w * 0.34).max(8.0);

        let a = (255.0 * alpha_mul * intensity).clamp(0.0, 140.0) as u8;
        let color_core = Color32::from_rgba_unmultiplied(186, 108, 255, a);
        let skew = h * 0.65;

        let p0 = Pos2::new(cx - band_w * 0.5, rect.bottom() + 2.0);
        let p1 = Pos2::new(cx + band_w * 0.5, rect.bottom() + 2.0);
        let p2 = Pos2::new(cx + band_w * 0.5 + skew, rect.top() - 2.0);
        let p3 = Pos2::new(cx - band_w * 0.5 + skew, rect.top() - 2.0);

        sub.add(Shape::convex_polygon(
            vec![p0, p1, p2, p3],
            color_core,
            egui::Stroke::NONE,
        ));
    }

    // Subtle soft violet rim sheen
    let pulse = ((time_sec * 2.8).sin() as f32 * 0.5 + 0.5) * intensity;
    let rim_alpha = (28.0 + 30.0 * pulse) as u8;
    let _ = Vec2::ZERO;
    sub.rect_stroke(
        rect.shrink(1.0),
        2.0,
        egui::Stroke::new(
            1.0_f32,
            Color32::from_rgba_unmultiplied(178, 102, 255, rim_alpha),
        ),
        egui::StrokeKind::Inside,
    );
}
