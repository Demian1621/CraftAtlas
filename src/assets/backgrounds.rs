use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use eframe::egui::{
    self, Color32, ColorImage, Pos2, Rect, TextureFilter, TextureHandle, TextureOptions,
    TextureWrapMode,
};
use image::imageops::FilterType;
use image::ImageFormat;

use crate::state::prefs::UserPrefs;
use crate::state::ThemeId;
use crate::ui::theme::McPalette;

pub const MAX_CUSTOM_BG_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_CUSTOM_BG_PIXELS: u64 = 64_000_000;

struct LoadedBackdrop {
    tex: TextureHandle,
    aspect: f32,
    is_panorama: bool,
}

/// Manages embedded HD Minecraft theme wallpapers, 360° panoramas, thumbnails, and custom user wallpaper.
pub struct ThemeBackgrounds {
    cache: HashMap<ThemeId, LoadedBackdrop>,
    thumbs: HashMap<ThemeId, TextureHandle>,
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
            thumbs: HashMap::new(),
        }
    }

    /// Clears cached custom wallpaper textures so a newly selected user image displays immediately.
    pub fn reload_custom_backdrop(&mut self) {
        self.cache.remove(&ThemeId::CustomImage);
        self.thumbs.remove(&ThemeId::CustomImage);
    }

    /// Returns a lazily built `260×146` preview texture for the Visual Theme Gallery Modal.
    pub fn thumbnail_for(&mut self, ctx: &egui::Context, theme: ThemeId) -> Option<egui::TextureId> {
        if let Some(tex) = self.thumbs.get(&theme) {
            return Some(tex.id());
        }
        let img = load_theme_rgba(theme)?;
        let (w, h) = img.dimensions();
        if w == 0 || h == 0 {
            return None;
        }
        let thumb_img = if theme.is_panorama() {
            // Frame the opening 16:9 hero slice of 2048x640 panoramas (matches initial u0=0.0 view)
            let cw = ((h as f32) * (16.0 / 9.0)).round().min(w as f32) as u32;
            let view = image::imageops::crop_imm(&img, 0, 0, cw.max(1), h).to_image();
            image::imageops::resize(&view, 260, 146, FilterType::Triangle)
        } else {
            image::imageops::resize(&img, 260, 146, FilterType::Triangle)
        };
        let color_img = ColorImage::from_rgba_unmultiplied([260, 146], thumb_img.as_raw());
        let opts = TextureOptions {
            magnification: TextureFilter::Linear,
            minification: TextureFilter::Linear,
            mipmap_mode: Some(TextureFilter::Linear),
            wrap_mode: TextureWrapMode::ClampToEdge,
        };
        let tex = ctx.load_texture(format!("mc_theme_thumb_{:?}", theme), color_img, opts);
        let id = tex.id();
        self.thumbs.insert(theme, tex);
        Some(id)
    }

    /// Paints the active steady HD scene (or smooth 360° panorama) plus readability scrim.
    pub fn paint_viewport_backdrop(
        &mut self,
        ctx: &egui::Context,
        rect: Rect,
        theme: ThemeId,
        palette: McPalette,
        bg_intensity: f32,
        time: f64,
    ) {
        let painter = ctx.layer_painter(egui::LayerId::background());
        painter.rect_filled(
            rect,
            0.0,
            Color32::from_rgb(palette.bg_canvas.r(), palette.bg_canvas.g(), palette.bg_canvas.b()),
        );

        let vw = rect.width().max(1.0);
        let vh = rect.height().max(1.0);
        let view_aspect = vw / vh;

        if let std::collections::hash_map::Entry::Vacant(slot) = self.cache.entry(theme) {
            if let Some(loaded) = load_theme_backdrop(ctx, theme) {
                slot.insert(loaded);
            }
        }

        if let Some(bg) = self.cache.get(&theme) {
            let t = time as f32;
            let uv = if bg.is_panorama {
                let uv_h = 0.88_f32;
                let uv_w = ((view_aspect / bg.aspect) * uv_h).clamp(0.18, 0.72);
                let v0 = 0.5 * (1.0 - uv_h);
                let u0 = (t * 0.008).fract();
                Rect::from_min_max(Pos2::new(u0, v0), Pos2::new(u0 + uv_w, v0 + uv_h))
            } else {
                // Calm, ultra-slow horizontal-only drift (no circular orbit or vertical wobble)
                let base_scale = 1.04_f32;
                let (mut uv_w, mut uv_h) = if view_aspect > bg.aspect {
                    (1.0 / base_scale, (bg.aspect / view_aspect) / base_scale)
                } else {
                    ((view_aspect / bg.aspect) / base_scale, 1.0 / base_scale)
                };
                uv_w = uv_w.clamp(0.25, 1.0);
                uv_h = uv_h.clamp(0.25, 1.0);
                let max_dx = (1.0 - uv_w) * 0.5;
                let cx = 0.5 + max_dx * 0.65 * (t * 0.065).sin();
                Rect::from_center_size(Pos2::new(cx, 0.5), egui::vec2(uv_w, uv_h))
            };
            painter.image(bg.tex.id(), rect, uv, Color32::WHITE);
        }

        // Smooth intensity-driven dark scrim keeping cards crisp while letting biome scenes breathe
        let intensity = bg_intensity.clamp(0.15, 1.0);
        let bias = match theme {
            ThemeId::ClassicJavaStone | ThemeId::CherryBlossom => 16.0_f32,
            _ => 0.0_f32,
        };
        let scrim_alpha = (208.0 - intensity * 160.0 + bias).clamp(24.0, 224.0).round() as u8;
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
        ThemeId::DeepDarkSculk => (
            "bg_deep_dark",
            include_bytes!("../../assets/backgrounds/bg_deep_dark.jpg"),
            true,
        ),
        ThemeId::CustomImage => (
            "bg_pale_garden",
            include_bytes!("../../assets/backgrounds/bg_pale_garden.jpg"),
            false,
        ),
        ThemeId::TrialChambers => (
            "bg_trial_chambers",
            include_bytes!("../../assets/backgrounds/bg_trial_chambers.jpg"),
            true,
        ),
        ThemeId::PaleGarden => (
            "bg_pale_garden",
            include_bytes!("../../assets/backgrounds/bg_pale_garden.jpg"),
            false,
        ),
        ThemeId::OceanMonument => (
            "bg_ocean_monument",
            include_bytes!("../../assets/backgrounds/bg_ocean_monument.jpg"),
            true,
        ),
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

fn load_theme_rgba(theme: ThemeId) -> Option<image::RgbaImage> {
    if theme == ThemeId::CustomImage {
        if let Some(path) = custom_backdrop_path() {
            if let Ok(meta) = fs::metadata(&path) {
                if meta.is_file() && meta.len() > 0 && meta.len() <= MAX_CUSTOM_BG_BYTES {
                    if let Ok((w, h)) = image::image_dimensions(&path) {
                        if (w as u64).saturating_mul(h as u64) <= MAX_CUSTOM_BG_PIXELS {
                            if let Ok(bytes) = fs::read(&path) {
                                if let Ok(img) = image::load_from_memory(&bytes) {
                                    return Some(img.to_rgba8());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let (_name, bytes, _) = backdrop_spec(theme);
    Some(image::load_from_memory(bytes).ok()?.to_rgba8())
}

fn load_theme_backdrop(ctx: &egui::Context, theme: ThemeId) -> Option<LoadedBackdrop> {
    let img = load_theme_rgba(theme)?;
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return None;
    }
    let aspect = (w as f32) / (h as f32);
    let is_panorama = theme.is_panorama();
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
    let tex = ctx.load_texture(format!("mc_backdrop_{:?}", theme), color_img, opts);
    Some(LoadedBackdrop {
        tex,
        aspect,
        is_panorama,
    })
}

/// Returns `%APPDATA%\craft-atlas\custom_backdrop.png` (or XDG/macOS equivalent).
pub fn custom_backdrop_path() -> Option<PathBuf> {
    Some(UserPrefs::app_data_dir()?.join("custom_backdrop.png"))
}

pub fn has_custom_backdrop() -> bool {
    custom_backdrop_path().is_some_and(|p| p.is_file())
}

/// Validates and stores a user `.png`/`.jpg` screenshot natively as `custom_backdrop.png` (no FFmpeg).
pub fn import_custom_backdrop(src: &Path) -> Result<PathBuf, String> {
    let dest = custom_backdrop_path()
        .ok_or_else(|| "Unable to resolve user config directory".to_string())?;
    import_custom_backdrop_to(src, &dest)
}

fn import_custom_backdrop_to(src: &Path, dest: &Path) -> Result<PathBuf, String> {
    let meta = fs::metadata(src).map_err(|e| format!("Cannot inspect image file: {e}"))?;
    if !meta.is_file() || meta.len() == 0 {
        return Err("Selected path is empty or not a regular file".to_string());
    }
    if meta.len() > MAX_CUSTOM_BG_BYTES {
        return Err("Image exceeds 16 MiB safety limit".to_string());
    }

    let ext = src
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    if !matches!(ext.as_str(), "png" | "jpg" | "jpeg") {
        return Err(format!(
            "Unsupported image format '.{ext}' (use .png or .jpg)"
        ));
    }

    let canonical =
        fs::canonicalize(src).map_err(|e| format!("Image path validation failed: {e}"))?;
    let (w, h) = image::image_dimensions(&canonical)
        .map_err(|e| format!("Invalid or corrupt image header: {e}"))?;
    if w < 64 || h < 64 {
        return Err("Image resolution too small (minimum 64x64)".to_string());
    }
    if (w as u64).saturating_mul(h as u64) > MAX_CUSTOM_BG_PIXELS {
        return Err("Image exceeds 64 megapixel safety limit".to_string());
    }

    let bytes = fs::read(&canonical).map_err(|e| format!("Failed reading image bytes: {e}"))?;
    let decoded = image::load_from_memory(&bytes)
        .map_err(|e| format!("Could not decode image: {e}"))?;
    let final_img = if w > 1920 || h > 1080 {
        decoded.resize(1920, 1080, FilterType::Triangle)
    } else {
        decoded
    };

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Create config dir failed: {e}"))?;
    }
    let tmp = dest.with_extension("png.tmp");
    final_img
        .to_rgba8()
        .save_with_format(&tmp, ImageFormat::Png)
        .map_err(|e| format!("Failed writing wallpaper PNG: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600));
    }
    if dest.exists() {
        let _ = fs::remove_file(dest);
    }
    fs::rename(&tmp, dest).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("Failed activating custom wallpaper: {e}")
    })?;
    Ok(dest.to_path_buf())
}

pub fn remove_custom_backdrop() -> bool {
    custom_backdrop_path().is_some_and(|p| p.is_file() && fs::remove_file(p).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_ten_builtin_theme_backdrops_decode_at_hd() {
        for theme in ThemeId::builtin() {
            let (name, bytes, _) = backdrop_spec(theme);
            let decoded = image::load_from_memory(bytes)
                .unwrap_or_else(|e| panic!("failed to decode {}: {}", name, e));
            assert!(
                decoded.width() >= 1000 && decoded.height() >= 500,
                "{} too small: {}x{}",
                name,
                decoded.width(),
                decoded.height()
            );
        }
    }

    #[test]
    fn custom_backdrop_importer_enforces_safety_and_roundtrips() {
        let tmp_dir = std::env::temp_dir().join("craft_atlas_custom_bg_test");
        let _ = fs::create_dir_all(&tmp_dir);
        let dest = tmp_dir.join("out_backdrop.png");

        // Reject unsupported format
        let bad_ext = tmp_dir.join("movie.mp4");
        fs::write(&bad_ext, b"fake video data").unwrap();
        assert!(import_custom_backdrop_to(&bad_ext, &dest).is_err());

        // Reject empty file
        let empty_png = tmp_dir.join("empty.png");
        fs::write(&empty_png, b"").unwrap();
        assert!(import_custom_backdrop_to(&empty_png, &dest).is_err());

        // Reject undersized PNG (16x16 < 64x64)
        let tiny_png = tmp_dir.join("tiny.png");
        image::RgbaImage::new(16, 16)
            .save_with_format(&tiny_png, ImageFormat::Png)
            .unwrap();
        assert!(import_custom_backdrop_to(&tiny_png, &dest).is_err());

        // Accept valid 128x96 PNG cleanly
        let ok_png = tmp_dir.join("valid.png");
        image::RgbaImage::from_pixel(128, 96, image::Rgba([40, 180, 160, 255]))
            .save_with_format(&ok_png, ImageFormat::Png)
            .unwrap();
        let saved = import_custom_backdrop_to(&ok_png, &dest).expect("valid PNG imports");
        assert!(saved.is_file());
        let (rw, rh) = image::image_dimensions(&saved).unwrap();
        assert_eq!((rw, rh), (128, 96));

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
