use eframe::egui::ColorImage;
use image::{imageops::FilterType, ImageBuffer, ImageFormat, Rgba, RgbaImage};
use std::fs;
use std::path::PathBuf;

use crate::state::UserPrefs;

pub const PLAYER_FACE_SIZE: u32 = 32;
pub const MAX_SKIN_FILE_BYTES: u64 = 2 * 1024 * 1024; // 2 MiB hard safety limit

/// Generates a crisp 32x32 classic Minecraft Steve face sprite in memory (no network needed).
pub fn default_steve_face_image() -> ColorImage {
    // Authentic 8x8 classic Steve head pixel grid (front face)
    // H = dark brown hair, h = mid hair, S = skin light, s = skin mid, d = skin shade
    // W = white sclera, E = indigo-blue iris, N = nose, M = beard/mouth
    const STEVE_8X8: [&str; 8] = [
        "HHHHHHHH",
        "HhhHHhhH",
        "hSSSSSSh",
        "SSssssSS",
        "SWESSEWS",
        "ssSMMssS",
        "dSMddMSd",
        "dMMMMMMd",
    ];

    let mut small: RgbaImage = ImageBuffer::new(8, 8);
    for (y, row) in STEVE_8X8.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let px = match ch {
                'H' => Rgba([47, 32, 18, 255]),
                'h' => Rgba([68, 47, 27, 255]),
                'S' => Rgba([198, 150, 114, 255]),
                's' => Rgba([182, 134, 98, 255]),
                'd' => Rgba([158, 110, 78, 255]),
                'W' => Rgba([244, 244, 248, 255]),
                'E' => Rgba([72, 58, 168, 255]),
                'M' => Rgba([102, 56, 40, 255]),
                _ => Rgba([182, 134, 98, 255]),
            };
            small.put_pixel(x as u32, y as u32, px);
        }
    }

    let upscaled = image::imageops::resize(&small, PLAYER_FACE_SIZE, PLAYER_FACE_SIZE, FilterType::Nearest);
    ColorImage::from_rgba_unmultiplied(
        [PLAYER_FACE_SIZE as usize, PLAYER_FACE_SIZE as usize],
        upscaled.as_raw(),
    )
}

/// Validates and crops the 8x8 head + hat/hair overlay from a Minecraft `.png` skin
/// (`64x64`, `64x32`, or HD multiples like `128x128`) into a crisp 32x32 RGBA buffer.
pub fn extract_player_face_rgba_from_png_bytes(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.len() as u64 > MAX_SKIN_FILE_BYTES {
        return Err("Skin PNG exceeds 2 MiB safety limit".to_string());
    }

    let img = image::load_from_memory_with_format(bytes, ImageFormat::Png)
        .map_err(|e| format!("Invalid PNG skin image: {}", e))?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    let valid_dims = w >= 64
        && h >= 32
        && w <= 4096
        && h <= 4096
        && w % 64 == 0
        && (w == h || w == h * 2);
    if !valid_dims {
        return Err(format!(
            "Unsupported Minecraft skin dimensions {}x{} (expected 64x64, 64x32, or HD multiple)",
            w, h
        ));
    }

    let scale = w / 64;
    let face_sz = 8 * scale;
    let base_x = 8 * scale;
    let base_y = 8 * scale;
    let hat_x = 40 * scale;
    let hat_y = 8 * scale;

    // Legacy 64x32 editors sometimes filled unused hat pixels with a flat solid color;
    // detect and skip only when an entire 64x32-aspect hat region is a uniform opaque color.
    let mut use_hat = true;
    if w == h * 2 {
        let first = *rgba.get_pixel(hat_x, hat_y);
        if first[3] == 255 {
            let mut uniform = true;
            'outer: for dy in 0..face_sz {
                for dx in 0..face_sz {
                    if *rgba.get_pixel(hat_x + dx, hat_y + dy) != first {
                        uniform = false;
                        break 'outer;
                    }
                }
            }
            if uniform {
                use_hat = false;
            }
        }
    }

    let mut cropped: RgbaImage = ImageBuffer::new(face_sz, face_sz);
    for dy in 0..face_sz {
        for dx in 0..face_sz {
            let mut base = *rgba.get_pixel(base_x + dx, base_y + dy);
            if use_hat {
                let top = *rgba.get_pixel(hat_x + dx, hat_y + dy);
                if top[3] > 0 {
                    base = blend_rgba(base, top);
                }
            }
            // Ensure head badge silhouette has crisp full opacity on opaque face areas
            if base[3] == 0 {
                base = Rgba([28, 28, 32, 255]);
            }
            cropped.put_pixel(dx, dy, base);
        }
    }

    let upscaled = image::imageops::resize(&cropped, PLAYER_FACE_SIZE, PLAYER_FACE_SIZE, FilterType::Nearest);
    Ok(upscaled.into_raw())
}

#[inline]
fn blend_rgba(dst: Rgba<u8>, src: Rgba<u8>) -> Rgba<u8> {
    let sa = src[3] as u32;
    if sa == 255 {
        return src;
    }
    if sa == 0 {
        return dst;
    }
    let inv = 255 - sa;
    let out_r = ((src[0] as u32 * sa + dst[0] as u32 * inv + 127) / 255) as u8;
    let out_g = ((src[1] as u32 * sa + dst[1] as u32 * inv + 127) / 255) as u8;
    let out_b = ((src[2] as u32 * sa + dst[2] as u32 * inv + 127) / 255) as u8;
    let out_a = ((sa * 255 + dst[3] as u32 * inv + 127) / 255) as u8;
    Rgba([out_r, out_g, out_b, out_a])
}

/// Converts a 32x32 RGBA face byte slice into an `egui::ColorImage`.
pub fn face_rgba_to_color_image(rgba: &[u8]) -> Result<ColorImage, String> {
    let expected = (PLAYER_FACE_SIZE * PLAYER_FACE_SIZE * 4) as usize;
    if rgba.len() != expected {
        return Err(format!(
            "Invalid extracted face byte length: {} (expected {})",
            rgba.len(),
            expected
        ));
    }
    Ok(ColorImage::from_rgba_unmultiplied(
        [PLAYER_FACE_SIZE as usize, PLAYER_FACE_SIZE as usize],
        rgba,
    ))
}

/// Location where the cropped 32x32 custom player skin head is persisted locally next to `prefs.json`.
pub fn custom_skin_face_path() -> Option<PathBuf> {
    UserPrefs::prefs_path().and_then(|p| p.parent().map(|d| d.join("custom_skin_face.png")))
}

fn legacy_custom_skin_face_path() -> Option<PathBuf> {
    UserPrefs::legacy_prefs_path().and_then(|p| p.parent().map(|d| d.join("custom_skin_face.png")))
}

/// Reads a skin `.png` file from disk safely, enforcing size limits before reading.
pub fn load_skin_face_from_png_file(path: &std::path::Path) -> Result<(Vec<u8>, ColorImage), String> {
    let meta = fs::metadata(path).map_err(|e| format!("Cannot inspect file: {}", e))?;
    if meta.len() > MAX_SKIN_FILE_BYTES {
        return Err("Skin file is larger than 2 MiB".to_string());
    }
    let bytes = fs::read(path).map_err(|e| format!("Cannot read file: {}", e))?;
    let rgba = extract_player_face_rgba_from_png_bytes(&bytes)?;
    let img = face_rgba_to_color_image(&rgba)?;
    Ok((rgba, img))
}

/// Persists the extracted 32x32 custom player skin face atomically in the user config directory.
pub fn save_custom_skin_face_rgba(rgba: &[u8]) -> Result<PathBuf, String> {
    let path = custom_skin_face_path().ok_or_else(|| "Config directory unavailable".to_string())?;
    let img: RgbaImage = ImageBuffer::from_raw(PLAYER_FACE_SIZE, PLAYER_FACE_SIZE, rgba.to_vec())
        .ok_or_else(|| "Invalid 32x32 RGBA skin face buffer".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Cannot create skin dir: {}", e))?;
    }
    let tmp = path.with_extension("png.tmp");
    img.save_with_format(&tmp, ImageFormat::Png)
        .map_err(|e| format!("Cannot save skin PNG: {}", e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600));
    }
    if fs::rename(&tmp, &path).is_err() {
        let _ = fs::remove_file(&path);
        fs::rename(&tmp, &path).map_err(|e| format!("Cannot replace skin file: {}", e))?;
    }
    Ok(path)
}

/// Loads a previously persisted custom skin face if one exists on disk (migrating from legacy path if needed).
pub fn load_persisted_custom_skin_face() -> Option<ColorImage> {
    for (path, is_legacy) in [
        custom_skin_face_path().map(|p| (p, false)),
        legacy_custom_skin_face_path().map(|p| (p, true)),
    ]
    .into_iter()
    .flatten()
    {
        let Ok(meta) = fs::metadata(&path) else { continue };
        if meta.len() > MAX_SKIN_FILE_BYTES {
            continue;
        }
        let Ok(bytes) = fs::read(&path) else { continue };
        let Ok(decoded) = image::load_from_memory_with_format(&bytes, ImageFormat::Png) else {
            continue;
        };
        let rgba = decoded.to_rgba8();
        if rgba.width() != PLAYER_FACE_SIZE || rgba.height() != PLAYER_FACE_SIZE {
            continue;
        }
        if is_legacy {
            let _ = save_custom_skin_face_rgba(rgba.as_raw());
        }
        return Some(ColorImage::from_rgba_unmultiplied(
            [PLAYER_FACE_SIZE as usize, PLAYER_FACE_SIZE as usize],
            rgba.as_raw(),
        ));
    }
    None
}

/// Removes any persisted custom skin face so the viewer reverts to Steve.
pub fn clear_persisted_custom_skin_face() {
    for path in [custom_skin_face_path(), legacy_custom_skin_face_path()]
        .into_iter()
        .flatten()
    {
        let _ = fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn encode_png(img: &RgbaImage) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        img.write_to(&mut cursor, ImageFormat::Png).unwrap();
        cursor.into_inner()
    }

    #[test]
    fn default_steve_face_has_expected_dimensions() {
        let img = default_steve_face_image();
        assert_eq!(img.size, [32, 32]);
        assert_eq!(img.pixels.len(), 32 * 32);
    }

    #[test]
    fn extracts_64x64_face_with_hat_overlay() {
        let mut skin = ImageBuffer::from_pixel(64, 64, Rgba([0, 0, 0, 0]));
        for y in 8..16 {
            for x in 8..16 {
                skin.put_pixel(x, y, Rgba([200, 150, 100, 255]));
            }
        }
        // Semi-transparent blue visor at (40, 8)
        skin.put_pixel(40, 8, Rgba([0, 100, 255, 255]));

        let bytes = encode_png(&skin);
        let rgba = extract_player_face_rgba_from_png_bytes(&bytes).expect("valid 64x64 skin");
        assert_eq!(rgba.len(), 32 * 32 * 4);
        // Top-left 4x4 block corresponds to pixel (0, 0) of cropped 8x8 -> hat pixel (40, 8)
        assert_eq!(&rgba[0..4], &[0, 100, 255, 255]);
        // Pixel at (1, 1) of cropped 8x8 (scaled to pixel x=4, y=4) has base skin
        let off = ((4 * 32) + 4) * 4;
        assert_eq!(&rgba[off..off + 4], &[200, 150, 100, 255]);
    }

    #[test]
    fn supports_hd_128x128_and_legacy_64x32_skins() {
        let mut legacy = ImageBuffer::from_pixel(64, 32, Rgba([0, 0, 0, 255]));
        for y in 8..16 {
            for x in 8..16 {
                legacy.put_pixel(x, y, Rgba([120, 90, 70, 255]));
            }
        }
        let legacy_rgba = extract_player_face_rgba_from_png_bytes(&encode_png(&legacy))
            .expect("valid 64x32 skin");
        assert_eq!(&legacy_rgba[0..4], &[120, 90, 70, 255]);

        let mut hd = ImageBuffer::from_pixel(128, 128, Rgba([0, 0, 0, 0]));
        for y in 16..32 {
            for x in 16..32 {
                hd.put_pixel(x, y, Rgba([40, 210, 160, 255]));
            }
        }
        let hd_rgba =
            extract_player_face_rgba_from_png_bytes(&encode_png(&hd)).expect("valid 128x128 skin");
        assert_eq!(&hd_rgba[0..4], &[40, 210, 160, 255]);
    }

    #[test]
    fn rejects_invalid_skin_formats_and_dimensions() {
        assert!(extract_player_face_rgba_from_png_bytes(b"not a png").is_err());

        let square_32 = ImageBuffer::from_pixel(32, 32, Rgba([255, 0, 0, 255]));
        assert!(extract_player_face_rgba_from_png_bytes(&encode_png(&square_32)).is_err());

        let odd = ImageBuffer::from_pixel(100, 50, Rgba([255, 0, 0, 255]));
        assert!(extract_player_face_rgba_from_png_bytes(&encode_png(&odd)).is_err());
    }
}
