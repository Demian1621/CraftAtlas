use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, TextureHandle, TextureOptions, Vec2};
use serde::Deserialize;
use std::collections::HashMap;

const ATLAS_PNG_BYTES: &[u8] = include_bytes!("../../assets/atlas.png");
const ATLAS_JSON_BYTES: &[u8] = include_bytes!("../../assets/atlas.json");

#[derive(Debug, Deserialize)]
struct AtlasJson {
    width: u32,
    height: u32,
    #[allow(dead_code)]
    cell_size: u32,
    entries: HashMap<String, AtlasEntry>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct AtlasEntry {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

/// GPU-backed Minecraft 26.3 texture atlas (single 1536x1504 texture with NEAREST pixel filtering).
pub struct ItemAtlas {
    texture: TextureHandle,
    uvs: HashMap<String, Rect>,
}

impl ItemAtlas {
    pub fn load(ctx: &egui::Context) -> Self {
        let meta: AtlasJson = serde_json::from_slice(ATLAS_JSON_BYTES)
            .expect("embedded assets/atlas.json must be valid JSON");

        let decoded = image::load_from_memory_with_format(ATLAS_PNG_BYTES, image::ImageFormat::Png)
            .expect("embedded assets/atlas.png must decode")
            .to_rgba8();

        let size = [decoded.width() as usize, decoded.height() as usize];
        let image = ColorImage::from_rgba_unmultiplied(size, decoded.as_raw());
        let texture = ctx.load_texture("mc_item_atlas", image, TextureOptions::NEAREST);

        let inv_w = 1.0 / (meta.width.max(1) as f32);
        let inv_h = 1.0 / (meta.height.max(1) as f32);

        // Inset by a sub-texel epsilon to prevent neighbor bleeding on fractional UI scales
        let eps_x = 0.05 * inv_w;
        let eps_y = 0.05 * inv_h;

        let mut uvs = HashMap::with_capacity(meta.entries.len());
        for (id, e) in meta.entries {
            let min = Pos2::new((e.x as f32) * inv_w + eps_x, (e.y as f32) * inv_h + eps_y);
            let max = Pos2::new(
                ((e.x + e.w) as f32) * inv_w - eps_x,
                ((e.y + e.h) as f32) * inv_h - eps_y,
            );
            uvs.insert(id, Rect::from_min_max(min, max));
        }

        Self { texture, uvs }
    }

    /// Resolves canonical ID or alias to precomputed normalized UV coordinates inside `atlas.png`.
    pub fn uv_for(&self, raw_id: &str) -> Rect {
        let clean = raw_id.strip_prefix("minecraft:").unwrap_or(raw_id);
        if let Some(&uv) = self.uvs.get(clean) {
            return uv;
        }
        if let Some(alias) = fallback_alias(clean) {
            if let Some(&uv) = self.uvs.get(alias) {
                return uv;
            }
        }
        // Final guaranteed fallbacks present in atlas
        self.uvs
            .get("knowledge_book")
            .or_else(|| self.uvs.get("barrier"))
            .or_else(|| self.uvs.values().next())
            .copied()
            .unwrap_or(Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)))
    }

    /// Paints a crisp pixel-art sprite centered into `rect` preserving aspect ratio.
    pub fn paint(&self, painter: &egui::Painter, rect: Rect, item_id: &str, tint: Color32) {
        let uv = self.uv_for(item_id);
        painter.image(self.texture.id(), rect, uv, tint);
    }

    /// Renders an inline `egui::Image` sized cleanly inside layouts.
    pub fn ui_image(&self, item_id: &str, size_px: f32) -> egui::Image<'static> {
        let uv = self.uv_for(item_id);
        egui::Image::new((self.texture.id(), Vec2::splat(size_px)))
            .uv(uv)
            .fit_to_exact_size(Vec2::splat(size_px))
    }
}

fn fallback_alias(clean: &str) -> Option<&'static str> {
    if clean.starts_with("potion") || clean == "water_bottle" {
        return Some("potion");
    }
    if clean.starts_with("splash_potion") {
        return Some("splash_potion");
    }
    if clean.starts_with("lingering_potion") {
        return Some("lingering_potion");
    }
    if clean.starts_with("tipped_arrow") {
        return Some("tipped_arrow");
    }
    if clean.ends_with("_banner") {
        return Some("white_banner");
    }
    if clean.ends_with("_bed") {
        return Some("red_bed");
    }
    if clean.ends_with("_shulker_box") {
        return Some("shulker_box");
    }
    if clean.ends_with("_candle") {
        return Some("candle");
    }
    if clean.ends_with("_harness") {
        return Some("saddle");
    }
    match clean {
        "netherite_pickaxe_silk" | "netherite_pickaxe_fortune" => Some("netherite_pickaxe"),
        "air" => Some("glass_pane"),
        _ => None,
    }
}

/// Validates that embedded atlas JSON parses cleanly and core UI sprites exist.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::{load_database, load_recipes};

    #[test]
    fn embedded_atlas_contains_core_icons_and_all_db_icons() {
        let meta: AtlasJson = serde_json::from_slice(ATLAS_JSON_BYTES).expect("valid atlas JSON");
        assert!(meta.entries.len() >= 2000);

        let resolve_ok = |raw: &str| -> bool {
            let clean = raw.strip_prefix("minecraft:").unwrap_or(raw);
            meta.entries.contains_key(clean)
                || fallback_alias(clean).is_some_and(|a| meta.entries.contains_key(a))
        };

        for req in [
            "totem_of_undying",
            "crafting_table",
            "anvil",
            "enchanted_book",
            "filled_map",
            "mace",
            "elytra",
            "end_crystal",
            "compass",
            "knowledge_book",
            "red_bed",
            "warden_head",
        ] {
            assert!(meta.entries.contains_key(req), "Missing sprite: {}", req);
        }

        let db = load_database().expect("valid database");
        for b in &db.enchant_books {
            assert!(resolve_ok(&b.icon), "Missing enchant book icon: {}", b.icon);
            for (item_icon, _, _) in b.applicable_item_icons() {
                assert!(
                    resolve_ok(item_icon),
                    "Missing applicable item icon '{}' for book {}",
                    item_icon,
                    b.id
                );
            }
        }
        for g in &db.gear {
            assert!(resolve_ok(&g.icon), "Missing gear icon: {}", g.icon);
        }
        for f in &db.farms {
            assert!(
                resolve_ok(f.resolved_icon()),
                "Missing farm display icon '{}' for farm {}",
                f.resolved_icon(),
                f.id
            );
            for m in &f.materials {
                assert!(resolve_ok(&m.icon), "Missing farm material icon: {}", m.icon);
            }
        }
        for d in &db.dupes {
            assert!(resolve_ok(&d.icon), "Missing dupe icon: {}", d.icon);
        }
        for tip in &db.pro_tips {
            assert!(resolve_ok(&tip.icon), "Missing tip icon: {}", tip.icon);
        }
        for c in &db.creators {
            assert!(resolve_ok(&c.icon), "Missing creator icon: {}", c.icon);
        }
        for (icon_id, _, _) in crate::state::WAYPOINT_ICON_PRESETS {
            assert!(
                resolve_ok(icon_id),
                "Missing waypoint icon preset in atlas: {}",
                icon_id
            );
        }

        let recipes = load_recipes().expect("valid recipes");
        for item_id in &recipes.catalog_items {
            assert!(
                resolve_ok(item_id),
                "Catalog item '{}' missing in atlas or fallback alias",
                item_id
            );
        }
    }
}
