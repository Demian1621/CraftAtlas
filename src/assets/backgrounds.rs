use std::collections::HashMap;

use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, TextureFilter, TextureHandle, TextureOptions, TextureWrapMode};

use crate::state::ThemeId;
use crate::ui::theme::McPalette;

struct LoadedBackdrop {
    tex: TextureHandle,
    aspect: f32,
    is_panorama: bool,
}

/// Manages embedded theme wallpapers & 360° cubemap panoramas decoded lazily onto GPU textures.
#[derive(Default)]
pub struct ThemeBackgrounds {
    cache: HashMap<ThemeId, LoadedBackdrop>,
}

impl ThemeBackgrounds {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Ensures the active theme's wallpaper is uploaded and paints the animated backdrop + scrim.
    pub fn paint_viewport_backdrop(
        &mut self,
        ctx: &egui::Context,
        rect: Rect,
        theme: ThemeId,
        palette: McPalette,
        time: f64,
    ) {
        if let std::collections::hash_map::Entry::Vacant(slot) = self.cache.entry(theme) {
            if let Some(loaded) = load_theme_backdrop(ctx, theme) {
                slot.insert(loaded);
            }
        }

        let painter = ctx.layer_painter(egui::LayerId::background());
        painter.rect_filled(rect, 0.0, Color32::from_rgb(palette.bg_canvas.r(), palette.bg_canvas.g(), palette.bg_canvas.b()));

        if let Some(bg) = self.cache.get(&theme) {
            let vw = rect.width().max(1.0);
            let vh = rect.height().max(1.0);
            let view_aspect = vw / vh;
            let t = time as f32;

            let uv = if bg.is_panorama {
                // Smooth infinite 360° cylindrical panorama drift (~125s full revolution)
                let zoom = 0.78_f32;
                let uv_w = ((view_aspect / bg.aspect) * zoom).clamp(0.18, 0.72);
                let uv_h = zoom.clamp(0.55, 0.96);
                let v_drift = 0.5 * (1.0 - uv_h) + ((1.0 - uv_h) * 0.35) * (t * 0.11).sin();
                let u0 = (t * 0.008).fract();
                Rect::from_min_max(
                    Pos2::new(u0, v_drift.clamp(0.0, 1.0 - uv_h)),
                    Pos2::new(u0 + uv_w, (v_drift + uv_h).clamp(uv_h, 1.0)),
                )
            } else {
                // Smooth Ken Burns cover-fit crop within [0..1]
                let base_scale = 1.06 + 0.025 * (t * 0.18).sin();
                let (mut uv_w, mut uv_h) = if view_aspect > bg.aspect {
                    (1.0 / base_scale, (bg.aspect / view_aspect) / base_scale)
                } else {
                    ((view_aspect / bg.aspect) / base_scale, 1.0 / base_scale)
                };
                uv_w = uv_w.clamp(0.25, 1.0);
                uv_h = uv_h.clamp(0.25, 1.0);
                let max_dx = (1.0 - uv_w) * 0.5;
                let max_dy = (1.0 - uv_h) * 0.5;
                let cx = 0.5 + max_dx * 0.72 * (t * 0.13).sin();
                let cy = 0.5 + max_dy * 0.72 * (t * 0.09 + 1.1).cos();
                Rect::from_center_size(Pos2::new(cx, cy), egui::vec2(uv_w, uv_h))
            };

            painter.image(bg.tex.id(), rect, uv, Color32::WHITE);
        }

        // Subtle dark-glass readability scrim preserving vibrant scenic hues
        let scrim_alpha = match theme {
            ThemeId::ClassicJavaStone | ThemeId::CherryBlossom => 138,
            ThemeId::DeepslateOre | ThemeId::StarryOverworld => 112,
            ThemeId::NetherCrimsonEnd | ThemeId::EndVoidAurora => 104,
        };
        painter.rect_filled(
            rect,
            0.0,
            Color32::from_rgba_unmultiplied(
                palette.bg_canvas.r(),
                palette.bg_canvas.g(),
                palette.bg_canvas.b(),
                scrim_alpha,
            ),
        );
    }
}

fn backdrop_spec(theme: ThemeId) -> (&'static str, &'static [u8], bool) {
    match theme {
        ThemeId::DeepslateOre => (
            "bg_deep_cave",
            include_bytes!("../../assets/backgrounds/bg_deep_cave.jpg"),
            false,
        ),
        ThemeId::ClassicJavaStone => (
            "bg_wilderness_26_3",
            include_bytes!("../../assets/backgrounds/bg_wilderness_26_3.jpg"),
            true,
        ),
        ThemeId::NetherCrimsonEnd => (
            "bg_nether_crimson",
            include_bytes!("../../assets/backgrounds/bg_nether_crimson.jpg"),
            true,
        ),
        ThemeId::EndVoidAurora => (
            "bg_end_aurora",
            include_bytes!("../../assets/backgrounds/bg_end_aurora.jpg"),
            false,
        ),
        ThemeId::CherryBlossom => (
            "bg_cherry_grove",
            include_bytes!("../../assets/backgrounds/bg_cherry_grove.jpg"),
            false,
        ),
        ThemeId::StarryOverworld => (
            "bg_starry_night",
            include_bytes!("../../assets/backgrounds/bg_starry_night.jpg"),
            false,
        ),
    }
}

fn load_theme_backdrop(ctx: &egui::Context, theme: ThemeId) -> Option<LoadedBackdrop> {
    let (name, bytes, is_panorama) = backdrop_spec(theme);
    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    let aspect = (w as f32) / (h.max(1) as f32);
    let color_img = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], img.as_raw());
    let opts = TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        mipmap_mode: Some(TextureFilter::Linear),
        wrap_mode: if is_panorama {
            TextureWrapMode::Repeat
        } else {
            TextureWrapMode::ClampToEdge
        },
    };
    let tex = ctx.load_texture(format!("mc_backdrop_{}", name), color_img, opts);
    Some(LoadedBackdrop {
        tex,
        aspect,
        is_panorama,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_six_theme_backgrounds_decode_cleanly() {
        for theme in ThemeId::all() {
            let (name, bytes, _) = backdrop_spec(theme);
            let decoded = image::load_from_memory(bytes)
                .unwrap_or_else(|e| panic!("failed to decode {}: {}", name, e));
            assert!(decoded.width() >= 1000 && decoded.height() >= 500, "{} too small", name);
        }
    }
}
