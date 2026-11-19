use std::collections::HashMap;

use eframe::egui::{
    self, Color32, ColorImage, Pos2, Rect, TextureFilter, TextureHandle, TextureOptions,
    TextureWrapMode,
};

use crate::state::ThemeId;
use crate::ui::theme::McPalette;

use super::video_bg::{
    load_custom_cavf_bytes, CavfSource, VideoBackdropPlayer, VID_DEEP_DARK, VID_OCEAN_MONUMENT,
    VID_PALE_GARDEN, VID_TRIAL_CHAMBERS,
};

struct LoadedBackdrop {
    tex: TextureHandle,
    aspect: f32,
    is_panorama: bool,
}

/// Manages embedded theme wallpapers, 360° panoramas, built-in `.cavf` loops, and custom video walls.
pub struct ThemeBackgrounds {
    cache: HashMap<ThemeId, LoadedBackdrop>,
    video_player: VideoBackdropPlayer,
    custom_video_src: Option<CavfSource>,
    custom_checked: bool,
}

impl Default for ThemeBackgrounds {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeBackgrounds {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            video_player: VideoBackdropPlayer::new(),
            custom_video_src: None,
            custom_checked: false,
        }
    }

    /// Forces reload of `%APPDATA%\craft-atlas\video_themes\custom.cavf` right after user import.
    pub fn reload_custom_video(&mut self) {
        self.custom_checked = false;
        self.custom_video_src = None;
        self.video_player.invalidate_key("custom_video");
    }

    /// Ensures the active theme's wallpaper or video frame is displayed + dark readability scrim.
    pub fn paint_viewport_backdrop(
        &mut self,
        ctx: &egui::Context,
        rect: Rect,
        theme: ThemeId,
        palette: McPalette,
        timing: (f64, f32, bool),
    ) {
        let (time, dt, active) = timing;
        let painter = ctx.layer_painter(egui::LayerId::background());
        painter.rect_filled(
            rect,
            0.0,
            Color32::from_rgb(palette.bg_canvas.r(), palette.bg_canvas.g(), palette.bg_canvas.b()),
        );

        let vw = rect.width().max(1.0);
        let vh = rect.height().max(1.0);
        let view_aspect = vw / vh;

        let mut painted_video = false;
        if let Some((clip_key, src)) = self.video_source_for_theme(theme) {
            if self.video_player.ensure_clip(ctx, clip_key, src) {
                if let Some((tex_id, aspect)) = self.video_player.tick_and_texture(dt, active) {
                    let uv = cover_uv(view_aspect, aspect);
                    painter.image(tex_id, rect, uv, Color32::WHITE);
                    painted_video = true;
                }
            }
        }

        if !painted_video {
            let static_theme = if theme.is_video() {
                ThemeId::DeepslateOre
            } else {
                theme
            };
            if let std::collections::hash_map::Entry::Vacant(slot) =
                self.cache.entry(static_theme)
            {
                if let Some(loaded) = load_theme_backdrop(ctx, static_theme) {
                    slot.insert(loaded);
                }
            }
            if let Some(bg) = self.cache.get(&static_theme) {
                let t = time as f32;
                let uv = if bg.is_panorama {
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
        }

        // Dark-glass readability scrim keeping crisp foreground cards readable over animated video
        let scrim_alpha = match theme {
            ThemeId::ClassicJavaStone | ThemeId::CherryBlossom => 138,
            ThemeId::DeepslateOre | ThemeId::StarryOverworld => 112,
            ThemeId::NetherCrimsonEnd | ThemeId::EndVoidAurora => 104,
            ThemeId::DeepDarkSculk
            | ThemeId::TrialChambers
            | ThemeId::PaleGarden
            | ThemeId::OceanMonument
            | ThemeId::CustomVideo => 108,
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

    fn video_source_for_theme(&mut self, theme: ThemeId) -> Option<(&'static str, CavfSource)> {
        match theme {
            ThemeId::DeepDarkSculk => Some(("deep_dark", CavfSource::Embedded(VID_DEEP_DARK))),
            ThemeId::TrialChambers => {
                Some(("trial_chambers", CavfSource::Embedded(VID_TRIAL_CHAMBERS)))
            }
            ThemeId::PaleGarden => Some(("pale_garden", CavfSource::Embedded(VID_PALE_GARDEN))),
            ThemeId::OceanMonument => {
                Some(("ocean_monument", CavfSource::Embedded(VID_OCEAN_MONUMENT)))
            }
            ThemeId::CustomVideo => {
                if !self.custom_checked {
                    self.custom_checked = true;
                    self.custom_video_src = load_custom_cavf_bytes();
                }
                self.custom_video_src
                    .clone()
                    .map(|src| ("custom_video", src))
            }
            _ => None,
        }
    }
}

fn cover_uv(view_aspect: f32, media_aspect: f32) -> Rect {
    let (uv_w, uv_h) = if view_aspect > media_aspect {
        (1.0, (media_aspect / view_aspect).clamp(0.15, 1.0))
    } else {
        ((view_aspect / media_aspect).clamp(0.15, 1.0), 1.0)
    };
    Rect::from_center_size(Pos2::new(0.5, 0.5), egui::vec2(uv_w, uv_h))
}

fn backdrop_spec(theme: ThemeId) -> (&'static str, &'static [u8], bool) {
    match theme {
        ThemeId::DeepslateOre
        | ThemeId::DeepDarkSculk
        | ThemeId::TrialChambers
        | ThemeId::PaleGarden
        | ThemeId::OceanMonument
        | ThemeId::CustomVideo => (
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
    use crate::assets::video_bg::parse_cavf;

    #[test]
    fn all_eleven_theme_backdrops_and_videos_decode_cleanly() {
        for theme in ThemeId::all() {
            match theme {
                ThemeId::DeepDarkSculk => {
                    assert!(parse_cavf(VID_DEEP_DARK).is_ok());
                }
                ThemeId::TrialChambers => {
                    assert!(parse_cavf(VID_TRIAL_CHAMBERS).is_ok());
                }
                ThemeId::PaleGarden => {
                    assert!(parse_cavf(VID_PALE_GARDEN).is_ok());
                }
                ThemeId::OceanMonument => {
                    assert!(parse_cavf(VID_OCEAN_MONUMENT).is_ok());
                }
                ThemeId::CustomVideo => {
                    assert!(theme.is_video());
                }
                _ => {
                    let (name, bytes, _) = backdrop_spec(theme);
                    let decoded = image::load_from_memory(bytes)
                        .unwrap_or_else(|e| panic!("failed to decode {}: {}", name, e));
                    assert!(decoded.width() >= 1000 && decoded.height() >= 500, "{} too small", name);
                }
            }
        }
    }
}
