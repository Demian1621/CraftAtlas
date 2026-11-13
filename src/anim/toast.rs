use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Vec2};
use simple_easing::cubic_out;

use crate::assets::ItemAtlas;

#[derive(Debug, Clone)]
struct ToastEntry {
    title: String,
    subtitle: String,
    icon_id: String,
    accent: Color32,
    elapsed: f32,
    duration: f32,
}

/// Minecraft Advancement-style sliding toast notifications.
#[derive(Default)]
pub struct ToastQueue {
    items: Vec<ToastEntry>,
}

impl ToastQueue {
    pub fn push(&mut self, title: impl Into<String>, subtitle: impl Into<String>, icon_id: &str, accent: Color32) {
        if self.items.len() >= 3 {
            self.items.remove(0);
        }
        self.items.push(ToastEntry {
            title: title.into(),
            subtitle: subtitle.into(),
            icon_id: icon_id.to_string(),
            accent,
            elapsed: 0.0,
            duration: 3.4,
        });
    }

    pub fn update_and_paint(&mut self, ctx: &egui::Context, atlas: &ItemAtlas, dt: f32) {
        if self.items.is_empty() {
            return;
        }
        let screen = ctx.screen_rect();
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Tooltip,
            egui::Id::new("mc_toast_layer"),
        ));

        let card_w = (screen.width() * 0.34).clamp(240.0, 320.0);
        let card_h = 56.0;
        let margin_r = 16.0;
        let mut base_y = screen.top() + 58.0;

        for toast in &mut self.items {
            toast.elapsed += dt;
            let in_t = (toast.elapsed / 0.32).clamp(0.0, 1.0);
            let out_t = ((toast.duration - toast.elapsed) / 0.32).clamp(0.0, 1.0);
            let slide = cubic_out(in_t.min(out_t));

            let target_x = screen.right() - card_w - margin_r;
            let hidden_x = screen.right() + 20.0;
            let x = hidden_x + (target_x - hidden_x) * slide;

            let rect = Rect::from_min_size(Pos2::new(x, base_y), Vec2::new(card_w, card_h));

            // Authentic dark advancement toast frame
            painter.rect_filled(rect, 4.0, Color32::from_rgb(28, 30, 36));
            painter.rect_stroke(
                rect,
                4.0,
                egui::Stroke::new(2.0_f32, Color32::from_rgb(84, 88, 102)),
                egui::StrokeKind::Inside,
            );
            painter.rect_stroke(
                rect.shrink(2.0),
                2.0,
                egui::Stroke::new(1.0_f32, Color32::from_rgb(16, 17, 22)),
                egui::StrokeKind::Inside,
            );

            // Icon slot inside toast
            let icon_box = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::splat(36.0));
            painter.rect_filled(icon_box, 2.0, Color32::from_rgb(44, 48, 58));
            atlas.paint(&painter, icon_box.shrink(4.0), &toast.icon_id, Color32::WHITE);

            let text_x = icon_box.right() + 10.0;
            painter.text(
                Pos2::new(text_x, rect.top() + 12.0),
                Align2::LEFT_TOP,
                &toast.title,
                FontId::proportional(13.0),
                toast.accent,
            );
            painter.text(
                Pos2::new(text_x, rect.top() + 30.0),
                Align2::LEFT_TOP,
                truncate_chars(&toast.subtitle, 36),
                FontId::proportional(12.0),
                Color32::from_rgb(232, 236, 244),
            );

            base_y += card_h + 8.0;
        }

        self.items.retain(|t| t.elapsed < t.duration);
        if !self.items.is_empty() {
            ctx.request_repaint();
        }
    }
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    let mut out: String = s.chars().take(max_chars).collect();
    if s.chars().count() > max_chars {
        out.push_str("...");
    }
    out
}
