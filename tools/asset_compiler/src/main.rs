use image::{imageops::FilterType, GenericImageView, ImageBuffer, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct AtlasEntry {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Serialize)]
struct AtlasManifest {
    width: u32,
    height: u32,
    cell_size: u32,
    entries: BTreeMap<String, AtlasEntry>,
}

#[derive(Serialize)]
struct CompiledRecipeDb {
    version: String,
    tags: BTreeMap<String, Vec<String>>,
    recipes: Vec<CompiledRecipe>,
}

#[derive(Serialize)]
struct CompiledRecipe {
    id: String,
    station: String,
    category: String,
    result_id: String,
    result_count: u32,
    shapeless: bool,
    is_2x2: bool,
    group: String,
    slots: Vec<String>,
    note: String,
}

fn load_first_frame(path: &Path) -> Option<RgbaImage> {
    let img = image::open(path).ok()?;
    let (w, h) = img.dimensions();
    if h > w && w > 0 && h % w == 0 {
        // Vertical animated strip: take frame 0 cleanly instead of squishing
        Some(img.crop_imm(0, 0, w, w).to_rgba8())
    } else {
        Some(img.to_rgba8())
    }
}

fn shade_pixel(p: Rgba<u8>, factor: f32, tint: Option<[f32; 3]>) -> Rgba<u8> {
    if p[3] == 0 {
        return p;
    }
    let tr = tint.map(|t| t[0]).unwrap_or(1.0);
    let tg = tint.map(|t| t[1]).unwrap_or(1.0);
    let tb = tint.map(|t| t[2]).unwrap_or(1.0);
    Rgba([
        ((p[0] as f32 * factor * tr).clamp(0.0, 255.0)) as u8,
        ((p[1] as f32 * factor * tg).clamp(0.0, 255.0)) as u8,
        ((p[2] as f32 * factor * tb).clamp(0.0, 255.0)) as u8,
        p[3],
    ])
}

/// Renders an authentic crisp 32x32 isometric Minecraft block icon from top/left/right textures.
fn render_iso_block(
    top: &RgbaImage,
    left: &RgbaImage,
    right: &RgbaImage,
    top_tint: Option<[f32; 3]>,
    side_overlay: Option<(&RgbaImage, [f32; 3])>,
) -> RgbaImage {
    let top_s = image::imageops::resize(top, 16, 16, FilterType::Nearest);
    let left_s = image::imageops::resize(left, 16, 16, FilterType::Nearest);
    let right_s = image::imageops::resize(right, 16, 16, FilterType::Nearest);
    let ov_s = side_overlay.map(|(ov, c)| (image::imageops::resize(ov, 16, 16, FilterType::Nearest), c));

    let mut out = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 0]));

    // Isometric projection centered at (16, 16):
    // For pixel (px, py) in 0..32: map into diamond top, left face, right face
    // Top face diamond: v in [0..16), u = px - 16 + ...
    // Let's use clean forward splatting on 2x oversampled UV -> super crisp pixel-art edges!
    for u in 0..16 {
        for v in 0..16 {
            // Top face
            let sx = 16i32 + (u as i32 - v as i32) * 14 / 16;
            let sy = 2i32 + (u as i32 + v as i32) * 7 / 16;
            let c = shade_pixel(*top_s.get_pixel(u, v), 1.0, top_tint);
            for dx in 0..=1 {
                for dy in 0..=1 {
                    let x = sx + dx;
                    let y = sy + dy;
                    if x >= 0 && x < 32 && y >= 0 && y < 32 && c[3] > 0 {
                        out.put_pixel(x as u32, y as u32, c);
                    }
                }
            }
        }
    }

    for u in 0..16 {
        for v in 0..16 {
            // Left face (south/west side)
            let sx = 2i32 + (u as i32) * 14 / 16;
            let sy = 9i32 + (u as i32) * 7 / 16 + (v as i32) * 14 / 16;
            let mut base = *left_s.get_pixel(u, v);
            if let Some((ref ov, tint_c)) = ov_s {
                let op = *ov.get_pixel(u, v);
                if op[3] > 128 {
                    base = shade_pixel(op, 1.0, Some(tint_c));
                }
            }
            let c = shade_pixel(base, 0.80, None);
            for dx in 0..=1 {
                for dy in 0..=1 {
                    let x = sx + dx;
                    let y = sy + dy;
                    if x >= 0 && x < 32 && y >= 0 && y < 32 && c[3] > 0 {
                        out.put_pixel(x as u32, y as u32, c);
                    }
                }
            }

            // Right face (east side, slightly darker)
            let rx = 16i32 + (u as i32) * 14 / 16;
            let ry = 16i32 - (u as i32) * 7 / 16 + (v as i32) * 14 / 16;
            let mut rbase = *right_s.get_pixel(u, v);
            if let Some((ref ov, tint_c)) = ov_s {
                let op = *ov.get_pixel(u, v);
                if op[3] > 128 {
                    rbase = shade_pixel(op, 1.0, Some(tint_c));
                }
            }
            let rc = shade_pixel(rbase, 0.62, None);
            for dx in 0..=1 {
                for dy in 0..=1 {
                    let x = rx + dx;
                    let y = ry + dy;
                    if x >= 0 && x < 32 && y >= 0 && y < 32 && rc[3] > 0 {
                        out.put_pixel(x as u32, y as u32, rc);
                    }
                }
            }
        }
    }

    out
}

fn find_block_tex(raw_block: &Path, names: &[&str]) -> Option<RgbaImage> {
    for n in names {
        let p = raw_block.join(format!("{n}.png"));
        if p.exists() {
            if let Some(im) = load_first_frame(&p) {
                return Some(im);
            }
        }
    }
    None
}

fn resolve_missing_texture(id: &str, raw: &Path, existing_items: &Path) -> Option<RgbaImage> {
    let tex_item = raw.join("tex_item");
    let tex_block = raw.join("tex_block");
    let tex_entity = raw.join("tex_entity");
    let tex_extra = raw.join("tex_extra");

    // 1. Direct item texture in 26.3.jar
    let direct_item = tex_item.join(format!("{id}.png"));
    if direct_item.exists() {
        return load_first_frame(&direct_item);
    }

    // 2. Special iconic composites
    if id == "grass_block" {
        let top = find_block_tex(&tex_block, &["grass_block_top"])?;
        let side = find_block_tex(&tex_block, &["grass_block_side"])?;
        let ov = find_block_tex(&tex_block, &["grass_block_side_overlay"]);
        let grass_tint = [0.49, 0.74, 0.42];
        return Some(render_iso_block(
            &top,
            &side,
            &side,
            Some(grass_tint),
            ov.as_ref().map(|o| (o, grass_tint)),
        ));
    }

    if id == "enchanted_golden_apple" {
        let gapple = existing_items.join("golden_apple.png");
        if let Some(mut im) = load_first_frame(&gapple) {
            let im32 = image::imageops::resize(&mut im, 32, 32, FilterType::Nearest);
            let mut out = im32;
            for y in 0..32 {
                for x in 0..32 {
                    let p = out.get_pixel_mut(x, y);
                    if p[3] > 0 {
                        let band = ((x + y) % 16) as f32 / 16.0;
                        let boost = if band < 0.35 { 0.45 } else { 0.16 };
                        p[0] = (p[0] as f32 * (1.0 - boost) + 210.0 * boost) as u8;
                        p[1] = (p[1] as f32 * (1.0 - boost) + 90.0 * boost) as u8;
                        p[2] = (p[2] as f32 * (1.0 - boost) + 255.0 * boost) as u8;
                    }
                }
            }
            return Some(out);
        }
    }

    if id == "tipped_arrow" {
        let base = load_first_frame(&tex_item.join("tipped_arrow_base.png"))?;
        let head = load_first_frame(&tex_item.join("tipped_arrow_head.png"))?;
        let mut out = base;
        for (x, y, hp) in head.enumerate_pixels() {
            if hp[3] > 0 {
                // Teal/cyan splash potion head tint
                let c = shade_pixel(*hp, 1.0, Some([0.35, 0.85, 0.95]));
                out.put_pixel(x, y, c);
            }
        }
        return Some(out);
    }

    if id == "creeper_head" {
        if let Some(ent) = load_first_frame(&tex_entity.join("creeper.png")) {
            // Creeper skin face at (8, 8, 8, 8)
            let face = image::imageops::crop_imm(&ent, 8, 8, 8, 8).to_image();
            return Some(image::imageops::resize(&face, 32, 32, FilterType::Nearest));
        }
    }

    if id.contains("copper_golem_statue") {
        let suffix = if id.starts_with("exposed_") {
            "_exposed"
        } else if id.starts_with("weathered_") {
            "_weathered"
        } else if id.starts_with("oxidized_") {
            "_oxidized"
        } else {
            ""
        };
        let p = tex_extra.join(format!("entity__copper_golem__copper_golem{suffix}.png"));
        if let Some(ent) = load_first_frame(&p) {
            if ent.width() >= 16 && ent.height() >= 16 {
                let head = image::imageops::crop_imm(&ent, 8, 8, 8, 8).to_image();
                return Some(image::imageops::resize(&head, 32, 32, FilterType::Nearest));
            }
        }
    }

    if id.contains("copper_chest") {
        let key = if id.starts_with("exposed_") {
            "copper_exposed"
        } else if id.starts_with("weathered_") {
            "copper_weathered"
        } else if id.starts_with("oxidized_") {
            "copper_oxidized"
        } else {
            "copper"
        };
        let p = tex_extra.join(format!("entity__chest__{key}.png"));
        if let Some(ent) = load_first_frame(&p) {
            if ent.width() >= 42 && ent.height() >= 42 {
                let top = image::imageops::crop_imm(&ent, 14, 0, 14, 14).to_image();
                let side = image::imageops::crop_imm(&ent, 14, 19, 14, 14).to_image();
                return Some(render_iso_block(&top, &side, &side, None, None));
            }
        }
    }

    if id == "shelf_mushroom" {
        if let Some(im) = find_block_tex(&tex_block, &["shelf_mushroom_stage1", "shelf_mushroom_stage0"]) {
            return Some(im);
        }
    }

    if id.ends_with("_map") {
        if let Some(im) = find_block_tex(&tex_item, &[id, "filled_map"]) {
            return Some(im);
        }
    }

    // 3. Flat / tall plant / bars / shelf / sapling textures in tex_block
    let flat_candidates = [
        id.to_string(),
        format!("{id}_top"),
        format!("{id}_front"),
        format!("{id}_stage1"),
        format!("{id}_bottom"),
    ];
    let is_flat = matches!(
        id,
        "lilac"
            | "peony"
            | "rose_bush"
            | "sunflower"
            | "pale_oak_sapling"
            | "closed_eyeblossom"
            | "open_eyeblossom"
            | "wildflowers"
            | "cactus_flower"
    ) || id.contains("bars")
        || id.contains("chain")
        || id.contains("lantern")
        || id.contains("lightning_rod")
        || id.contains("door")
        || id.contains("trapdoor");

    if is_flat {
        for cand in &flat_candidates {
            let p = tex_block.join(format!("{cand}.png"));
            if p.exists() {
                return load_first_frame(&p);
            }
        }
    }

    // 4. Cube blocks -> render 3D isometric thumbnail from side/top
    if let Some(top) = find_block_tex(
        &tex_block,
        &[
            &format!("{id}_top"),
            &format!("{id}_end"),
            id,
        ],
    ) {
        let side = find_block_tex(&tex_block, &[&format!("{id}_side"), id]).unwrap_or_else(|| top.clone());
        let tint = if id.contains("leaves") {
            if id.contains("orange_poplar") || id.contains("red_poplar") || id.contains("yellow_poplar") {
                None
            } else {
                Some([0.75, 0.80, 0.75])
            }
        } else {
            None
        };
        return Some(render_iso_block(&top, &side, &side, tint, None));
    }

    // 5. Fallback variants (e.g. pale_oak_wood -> pale_oak_log, pale_oak_slab -> pale_oak_planks)
    let wood_base = id
        .replace("_wood", "_log")
        .replace("_stairs", "_planks")
        .replace("_slab", "_planks")
        .replace("_fence_gate", "_planks")
        .replace("_fence", "_planks")
        .replace("_button", "_planks")
        .replace("_pressure_plate", "_planks");
    if wood_base != id {
        if let Some(top) = find_block_tex(&tex_block, &[&format!("{wood_base}_top"), &wood_base]) {
            let side = find_block_tex(&tex_block, &[&format!("{wood_base}_side"), &wood_base])
                .unwrap_or_else(|| top.clone());
            return Some(render_iso_block(&top, &side, &side, None, None));
        }
    }

    // Last resort direct tex_block match
    find_block_tex(&tex_block, &[id])
}

fn expand_tok(tok: &Value, tags: &BTreeMap<String, Vec<String>>, out: &mut Vec<String>, depth: u8) {
    if depth > 6 {
        return;
    }
    match tok {
        Value::String(s) => {
            if let Some(tag_key) = s.strip_prefix("#minecraft:") {
                if let Some(vals) = tags.get(tag_key) {
                    if vals.len() == 1 {
                        out.push(vals[0].clone());
                    } else {
                        out.push(format!("#{tag_key}"));
                    }
                    return;
                }
            }
            out.push(s.trim_start_matches("minecraft:").to_string());
        }
        Value::Array(arr) => {
            // Multiple direct alternatives -> create synthetic inline list or first representative
            if let Some(first) = arr.first() {
                expand_tok(first, tags, out, depth + 1);
            }
        }
        Value::Object(map) => {
            if let Some(item) = map.get("item").and_then(|v| v.as_str()) {
                out.push(item.trim_start_matches("minecraft:").to_string());
            } else if let Some(tag) = map.get("tag").and_then(|v| v.as_str()) {
                let k = tag.trim_start_matches("minecraft:");
                out.push(format!("#{k}"));
            }
        }
        _ => {}
    }
}

fn classify_category(raw_cat: &str, id: &str, station: &str) -> &'static str {
    if station == "brewing" || id.contains("potion") || id.contains("tipped_arrow") {
        return "brewing";
    }
    if id.contains("helmet")
        || id.contains("chestplate")
        || id.contains("leggings")
        || id.contains("boots")
        || id.contains("horse_armor")
        || id.contains("nautilus_armor")
        || id.contains("wolf_armor")
        || id == "shield"
        || id == "turtle_helmet"
    {
        return "armor";
    }
    if id.ends_with("_sword")
        || id == "bow"
        || id == "crossbow"
        || id == "arrow"
        || id == "spectral_arrow"
        || id == "mace"
        || id == "trident"
        || id.contains("firework")
        || id == "tnt"
    {
        return "combat";
    }
    if id.ends_with("_pickaxe")
        || id.ends_with("_axe")
        || id.ends_with("_shovel")
        || id.ends_with("_hoe")
        || id == "shears"
        || id == "flint_and_steel"
        || id == "fishing_rod"
        || id == "brush"
        || id == "compass"
        || id == "recovery_compass"
        || id == "clock"
        || id == "spyglass"
        || id.ends_with("_bundle")
        || id.ends_with("_harness")
    {
        return "tools";
    }
    if id.contains("rail")
        || id.contains("minecart")
        || id.ends_with("_boat")
        || id.ends_with("_chest_boat")
        || id.ends_with("_raft")
        || id.ends_with("_chest_raft")
        || id == "saddle"
        || id == "carrot_on_a_stick"
        || id == "warped_fungus_on_a_stick"
        || id == "elytra"
    {
        return "transportation";
    }
    match raw_cat {
        "redstone" => "redstone",
        "building" => "building",
        "misc" => {
            if id.contains("ingot")
                || id.contains("nugget")
                || id.contains("dye")
                || id.contains("plank")
                || id == "stick"
                || id == "paper"
                || id == "leather"
                || id == "blaze_powder"
                || id == "magma_cream"
                || id == "fermented_spider_eye"
                || id == "bone_meal"
                || id == "sugar"
            {
                "materials"
            } else if id.contains("bread")
                || id.contains("stew")
                || id.contains("soup")
                || id.contains("pie")
                || id.contains("cake")
                || id.contains("cookie")
                || id.contains("golden_apple")
                || id.contains("golden_carrot")
                || id.contains("cooked_")
                || id.contains("dried_kelp")
            {
                "food"
            } else if id.contains("banner")
                || id.contains("candle")
                || id.contains("carpet")
                || id.contains("bed")
                || id.contains("pot")
                || id.contains("cushion")
                || id.contains("painting")
                || id.contains("item_frame")
                || id.contains("shelf")
            {
                "decoration"
            } else {
                "misc"
            }
        }
        "equipment" => "tools",
        _ => "misc",
    }
}

fn format_id_title(id: &str) -> String {
    id.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn main() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    if std::env::args().any(|a| a == "--generate-v110-icons") {
        generate_v110_icons(&repo);
        pack_items_atlas(&repo);
        return;
    }
    if std::env::args().any(|a| a == "--pack-atlas-only") {
        pack_items_atlas(&repo);
        return;
    }
    let raw = std::env::var_os("MC_RAW_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo.join("mc26_raw"));

    let items_dir = repo.join("assets/items");
    let data_dir = repo.join("data");

    // 1. Load official 26.3 tags
    let mut raw_tags: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for entry in fs::read_dir(raw.join("tags_item")).unwrap() {
        let p = entry.unwrap().path();
        if p.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let key = p
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .replace("__", "/");
        let txt = fs::read_to_string(&p).unwrap();
        let val: Value = serde_json::from_str(&txt).unwrap();
        let vals = val["values"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        raw_tags.insert(key, vals);
    }

    // Flatten tags recursively
    fn flatten_tag(
        key: &str,
        raw_tags: &BTreeMap<String, Vec<String>>,
        seen: &mut BTreeSet<String>,
        out: &mut Vec<String>,
    ) {
        if !seen.insert(key.to_string()) {
            return;
        }
        if let Some(vals) = raw_tags.get(key) {
            for v in vals {
                if let Some(sub) = v.strip_prefix("#minecraft:") {
                    flatten_tag(sub, raw_tags, seen, out);
                } else {
                    out.push(v.trim_start_matches("minecraft:").to_string());
                }
            }
        }
    }

    let mut flat_tags: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for k in raw_tags.keys() {
        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        flatten_tag(k, &raw_tags, &mut seen, &mut out);
        // Deduplicate while preserving order
        let mut uniq = Vec::new();
        let mut s2 = BTreeSet::new();
        for x in out {
            if s2.insert(x.clone()) {
                uniq.push(x);
            }
        }
        flat_tags.insert(k.clone(), uniq);
    }

    // 2. Parse all 2042 official 26.3 recipes
    let mut recipes = Vec::new();
    let mut referenced_ids: BTreeSet<String> = BTreeSet::new();
    let mut used_tags: BTreeSet<String> = BTreeSet::new();

    // Seed important app/SeedMap IDs
    for extra in [
        "grass_block", "ancient_debris", "sculk_catalyst", "pale_oak_sapling", "netherrack",
        "end_stone", "slime_ball", "emerald", "bell", "trial_key", "ominous_trial_key",
        "breeze_rod", "ender_eye", "echo_shard", "recovery_compass", "prismarine_crystals",
        "trident", "chiseled_sandstone", "tnt", "mossy_cobblestone", "potion", "cauldron",
        "crossbow", "ominous_bottle", "totem_of_undying", "crying_obsidian", "obsidian",
        "creaking_heart", "blaze_rod", "nether_bricks", "gilded_blackstone", "gold_ingot",
        "piglin_banner_pattern", "netherite_scrap", "end_crystal", "shulker_shell", "purpur_block",
        "elytra", "compass", "spyglass", "paper", "redstone", "diamond", "rail", "dragon_egg",
        "anvil", "experience_bottle", "crafting_table", "furnace", "stonecutter", "smithing_table",
        "brewing_stand", "campfire", "blast_furnace", "smoker", "book", "barrier", "nether_star",
        "enchanted_golden_apple", "wheat", "nether_portal", "sticky_piston", "sand", "string",
        "chest", "oak_sign", "arrow", "lead", "bedrock", "wither_skeleton_skull", "gunpowder",
        "slime_block", "turtle_egg", "amethyst_shard", "netherite_helmet", "netherite_chestplate",
        "netherite_leggings", "netherite_boots", "netherite_sword", "netherite_pickaxe",
        "netherite_axe", "netherite_shovel", "netherite_hoe", "bow", "mace", "fishing_rod",
        "shield", "shears", "flint_and_steel", "wolf_armor", "turtle_helmet",
        "carrot_on_a_stick", "warped_fungus_on_a_stick", "brush", "carved_pumpkin",
        "lodestone_compass", "netherite_ingot", "iron_ingot",
    ] {
        referenced_ids.insert(extra.to_string());
    }

    let mut recipe_files: Vec<_> = fs::read_dir(raw.join("recipe"))
        .unwrap()
        .filter_map(|e| e.ok().map(|x| x.path()))
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    recipe_files.sort();

    for rp in recipe_files {
        let stem = rp.file_stem().unwrap().to_string_lossy().to_string();
        let txt = fs::read_to_string(&rp).unwrap();
        let v: Value = serde_json::from_str(&txt).unwrap();
        let rtype = v["type"].as_str().unwrap_or("");
        let raw_cat = v["category"].as_str().unwrap_or("misc");
        let group = v["group"].as_str().unwrap_or("").to_string();

        let (station, result_id, result_count, shapeless, is_2x2, slots, note) = match rtype {
            "minecraft:crafting_shaped" => {
                let res_id = v["result"]["id"]
                    .as_str()
                    .unwrap_or("")
                    .trim_start_matches("minecraft:")
                    .to_string();
                let res_cnt = v["result"]["count"].as_u64().unwrap_or(1) as u32;
                let pattern = v["pattern"].as_array().cloned().unwrap_or_default();
                let rows = pattern.len();
                let max_cols = pattern
                    .iter()
                    .filter_map(|r| r.as_str().map(|s| s.len()))
                    .max()
                    .unwrap_or(0);
                let is_2 = rows <= 2 && max_cols <= 2;

                let mut grid = vec![String::new(); 9];
                for (r_idx, row_val) in pattern.iter().take(3).enumerate() {
                    if let Some(row_str) = row_val.as_str() {
                        for (c_idx, ch) in row_str.chars().take(3).enumerate() {
                            if ch != ' ' {
                                let ch_key = ch.to_string();
                                if let Some(tok) = v["key"].get(&ch_key) {
                                    let mut tmp = Vec::new();
                                    expand_tok(tok, &flat_tags, &mut tmp, 0);
                                    if let Some(first) = tmp.into_iter().next() {
                                        grid[r_idx * 3 + c_idx] = first;
                                    }
                                }
                            }
                        }
                    }
                }
                ("crafting".to_string(), res_id, res_cnt, false, is_2, grid, String::new())
            }
            "minecraft:crafting_shapeless" => {
                let res_id = v["result"]["id"]
                    .as_str()
                    .unwrap_or("")
                    .trim_start_matches("minecraft:")
                    .to_string();
                let res_cnt = v["result"]["count"].as_u64().unwrap_or(1) as u32;
                let ings = v["ingredients"].as_array().cloned().unwrap_or_default();
                let is_2 = ings.len() <= 4;
                let mut grid = vec![String::new(); 9];
                for (idx, ing) in ings.iter().take(9).enumerate() {
                    let mut tmp = Vec::new();
                    expand_tok(ing, &flat_tags, &mut tmp, 0);
                    if let Some(first) = tmp.into_iter().next() {
                        grid[idx] = first;
                    }
                }
                ("crafting".to_string(), res_id, res_cnt, true, is_2, grid, String::new())
            }
            "minecraft:crafting_transmute" => {
                let res_id = v["result"]["id"]
                    .as_str()
                    .unwrap_or("")
                    .trim_start_matches("minecraft:")
                    .to_string();
                let res_cnt = v["result"]["count"].as_u64().unwrap_or(1) as u32;
                let mut grid = vec![String::new(); 9];
                let mut a = Vec::new();
                expand_tok(&v["input"], &flat_tags, &mut a, 0);
                let mut b = Vec::new();
                expand_tok(&v["material"], &flat_tags, &mut b, 0);
                if let Some(x) = a.into_iter().next() {
                    grid[0] = x;
                }
                if let Some(x) = b.into_iter().next() {
                    grid[1] = x;
                }
                ("crafting".to_string(), res_id, res_cnt, true, true, grid, String::new())
            }
            "minecraft:crafting_dye" => {
                let res_id = v["result"]["id"]
                    .as_str()
                    .unwrap_or("")
                    .trim_start_matches("minecraft:")
                    .to_string();
                let mut grid = vec![String::new(); 9];
                let mut a = Vec::new();
                expand_tok(&v["target"], &flat_tags, &mut a, 0);
                let mut b = Vec::new();
                expand_tok(&v["dye"], &flat_tags, &mut b, 0);
                if let Some(x) = a.into_iter().next() {
                    grid[0] = x;
                }
                if let Some(x) = b.into_iter().next() {
                    grid[1] = x;
                }
                ("crafting".to_string(), res_id, 1, true, true, grid, "Dye".to_string())
            }
            "minecraft:crafting_imbue" => {
                let res_id = v["result"]["id"]
                    .as_str()
                    .unwrap_or("")
                    .trim_start_matches("minecraft:")
                    .to_string();
                let res_cnt = v["result"]["count"].as_u64().unwrap_or(1) as u32;
                let mut grid = vec![String::new(); 9];
                for i in [0, 1, 2, 3, 5, 6, 7, 8] {
                    let mut m = Vec::new();
                    expand_tok(&v["material"], &flat_tags, &mut m, 0);
                    if let Some(x) = m.into_iter().next() {
                        grid[i] = x;
                    }
                }
                let mut src = Vec::new();
                expand_tok(&v["source"], &flat_tags, &mut src, 0);
                if let Some(x) = src.into_iter().next() {
                    grid[4] = x;
                }
                ("crafting".to_string(), res_id, res_cnt, false, false, grid, String::new())
            }
            "minecraft:smelting" | "minecraft:blasting" | "minecraft:smoking" | "minecraft:campfire_cooking" => {
                let st = rtype.trim_start_matches("minecraft:").trim_end_matches("_cooking");
                let res_id = v["result"]["id"]
                    .as_str()
                    .unwrap_or("")
                    .trim_start_matches("minecraft:")
                    .to_string();
                let res_cnt = v["result"]["count"].as_u64().unwrap_or(1) as u32;
                let mut ing = Vec::new();
                expand_tok(&v["ingredient"], &flat_tags, &mut ing, 0);
                let fuel = match st {
                    "campfire" => "campfire".to_string(),
                    _ => "coal".to_string(),
                };
                let mut slots = vec![String::new(); 2];
                slots[0] = ing.into_iter().next().unwrap_or_default();
                slots[1] = fuel;
                (st.to_string(), res_id, res_cnt, false, false, slots, String::new())
            }
            "minecraft:stonecutting" => {
                let res_id = v["result"]["id"]
                    .as_str()
                    .unwrap_or("")
                    .trim_start_matches("minecraft:")
                    .to_string();
                let res_cnt = v["result"]["count"].as_u64().unwrap_or(1) as u32;
                let mut ing = Vec::new();
                expand_tok(&v["ingredient"], &flat_tags, &mut ing, 0);
                (
                    "stonecutting".to_string(),
                    res_id,
                    res_cnt,
                    false,
                    false,
                    vec![ing.into_iter().next().unwrap_or_default()],
                    String::new(),
                )
            }
            "minecraft:smithing_transform" | "minecraft:smithing_trim" => {
                let res_id = v["result"]["id"]
                    .as_str()
                    .unwrap_or("")
                    .trim_start_matches("minecraft:")
                    .to_string();
                let mut tmpl = Vec::new();
                expand_tok(&v["template"], &flat_tags, &mut tmpl, 0);
                let mut base = Vec::new();
                expand_tok(&v["base"], &flat_tags, &mut base, 0);
                let mut add = Vec::new();
                expand_tok(&v["addition"], &flat_tags, &mut add, 0);
                let slots = vec![
                    tmpl.into_iter().next().unwrap_or_default(),
                    base.into_iter().next().unwrap_or_default(),
                    add.into_iter().next().unwrap_or_default(),
                ];
                let note = if rtype == "minecraft:smithing_trim" {
                    "Armor Trim".to_string()
                } else {
                    String::new()
                };
                ("smithing".to_string(), res_id, 1, false, false, slots, note)
            }
            "minecraft:brewing" => {
                let out_id = v["output"]["id"]
                    .as_str()
                    .unwrap_or("potion")
                    .trim_start_matches("minecraft:")
                    .to_string();
                let in_id = v["input"]["item"]
                    .as_str()
                    .unwrap_or("potion")
                    .trim_start_matches("minecraft:")
                    .to_string();
                let reg_id = v["reagent"]["item"]
                    .as_str()
                    .unwrap_or("")
                    .trim_start_matches("minecraft:")
                    .to_string();
                let in_pot = v["input"]["potion_contents"]["potions"]
                    .as_str()
                    .or_else(|| v["input"]["potion_contents"]["potion"].as_str())
                    .unwrap_or("")
                    .trim_start_matches("minecraft:");
                let out_pot = v["output"]["components"]["minecraft:potion_contents"]["potion"]
                    .as_str()
                    .unwrap_or("")
                    .trim_start_matches("minecraft:");
                let note = if !out_pot.is_empty() {
                    format!("{} -> {}", format_id_title(in_pot), format_id_title(out_pot))
                } else {
                    stem.clone()
                };
                (
                    "brewing".to_string(),
                    out_id,
                    1,
                    false,
                    false,
                    vec![in_id, reg_id],
                    note,
                )
            }
            "minecraft:crafting_special_bannerduplicate" => (
                "crafting".to_string(),
                "white_banner".to_string(),
                2,
                true,
                true,
                vec!["white_banner".into(), "white_banner".into(), "".into(), "".into(), "".into(), "".into(), "".into(), "".into(), "".into()],
                "Copy Banner".to_string(),
            ),
            "minecraft:crafting_special_bookcloning" => (
                "crafting".to_string(),
                "written_book".to_string(),
                1,
                true,
                true,
                vec!["written_book".into(), "writable_book".into(), "".into(), "".into(), "".into(), "".into(), "".into(), "".into(), "".into()],
                "Clone Book".to_string(),
            ),
            "minecraft:crafting_special_firework_rocket" => (
                "crafting".to_string(),
                "firework_rocket".to_string(),
                3,
                true,
                true,
                vec!["paper".into(), "gunpowder".into(), "".into(), "".into(), "".into(), "".into(), "".into(), "".into(), "".into()],
                "Custom Rocket".to_string(),
            ),
            "minecraft:crafting_special_firework_star" => (
                "crafting".to_string(),
                "firework_star".to_string(),
                1,
                true,
                true,
                vec!["gunpowder".into(), "#dyes".into(), "".into(), "".into(), "".into(), "".into(), "".into(), "".into(), "".into()],
                "Custom Star".to_string(),
            ),
            "minecraft:crafting_special_firework_star_fade" => (
                "crafting".to_string(),
                "firework_star".to_string(),
                1,
                true,
                true,
                vec!["firework_star".into(), "#dyes".into(), "".into(), "".into(), "".into(), "".into(), "".into(), "".into(), "".into()],
                "Fade Effect".to_string(),
            ),
            "minecraft:crafting_special_mapextending" => (
                "crafting".to_string(),
                "filled_map".to_string(),
                1,
                false,
                false,
                vec!["paper".into(); 9].into_iter().enumerate().map(|(i, p)| if i == 4 { "filled_map".into() } else { p }).collect(),
                "Zoom Out Map".to_string(),
            ),
            "minecraft:crafting_special_shielddecoration" => (
                "crafting".to_string(),
                "shield".to_string(),
                1,
                true,
                true,
                vec!["shield".into(), "#banners".into(), "".into(), "".into(), "".into(), "".into(), "".into(), "".into(), "".into()],
                "Banner Pattern".to_string(),
            ),
            "minecraft:crafting_special_repairitem" => (
                "crafting".to_string(),
                "iron_pickaxe".to_string(),
                1,
                true,
                true,
                vec!["iron_pickaxe".into(), "iron_pickaxe".into(), "".into(), "".into(), "".into(), "".into(), "".into(), "".into(), "".into()],
                "Tool Repair".to_string(),
            ),
            "minecraft:crafting_decorated_pot" => (
                "crafting".to_string(),
                "decorated_pot".to_string(),
                1,
                false,
                false,
                vec!["".into(), "#decorated_pot_ingredients".into(), "".into(), "#decorated_pot_ingredients".into(), "".into(), "#decorated_pot_ingredients".into(), "".into(), "#decorated_pot_ingredients".into(), "".into()],
                "Pottery Sherds".to_string(),
            ),
            _ => continue,
        };

        if result_id.is_empty() {
            continue;
        }
        let category = classify_category(raw_cat, &result_id, &station).to_string();
        referenced_ids.insert(result_id.clone());
        for s in &slots {
            if let Some(tk) = s.strip_prefix('#') {
                used_tags.insert(tk.to_string());
                if let Some(items) = flat_tags.get(tk) {
                    for it in items {
                        referenced_ids.insert(it.clone());
                    }
                }
            } else if !s.is_empty() {
                referenced_ids.insert(s.clone());
            }
        }

        recipes.push(CompiledRecipe {
            id: stem,
            station,
            category,
            result_id,
            result_count,
            shapeless,
            is_2x2,
            group,
            slots,
            note,
        });
    }

    // Filter tags to only used ones
    let mut final_tags = BTreeMap::new();
    for tk in used_tags {
        if let Some(vals) = flat_tags.get(&tk) {
            final_tags.insert(tk, vals.clone());
        }
    }

    // 3. Complete missing icons in assets/items/
    let mut generated_count = 0;
    for id in &referenced_ids {
        let dest = items_dir.join(format!("{id}.png"));
        if !dest.exists() {
            if let Some(im) = resolve_missing_texture(id, &raw, &items_dir) {
                im.save(&dest).unwrap();
                generated_count += 1;
            } else {
                println!("WARN: Could not synthesize icon for: {id}");
            }
        }
    }
    println!("Synthesized/copied {generated_count} missing PNGs into assets/items/");

    // 4. Write compiled recipes.json
    let compiled_db = CompiledRecipeDb {
        version: "26.3".to_string(),
        tags: final_tags,
        recipes,
    };
    let rec_json = serde_json::to_string(&compiled_db).unwrap();
    fs::write(data_dir.join("recipes.json"), rec_json).unwrap();
    println!("Wrote data/recipes.json with {} recipes", compiled_db.recipes.len());

    // 5. Build official trilingual item names data/lang_items.json
    let en_raw: HashMap<String, String> =
        serde_json::from_str(&fs::read_to_string(raw.join("lang/en_us.json")).unwrap()).unwrap();
    let mx_raw: HashMap<String, String> =
        serde_json::from_str(&fs::read_to_string(raw.join("lang/es_mx.json")).unwrap()).unwrap();
    let es_raw: HashMap<String, String> =
        serde_json::from_str(&fs::read_to_string(raw.join("lang/es_es.json")).unwrap()).unwrap();

    let lookup_name = |map: &HashMap<String, String>, id: &str| -> Option<String> {
        map.get(&format!("item.minecraft.{id}"))
            .or_else(|| map.get(&format!("block.minecraft.{id}")))
            .cloned()
    };

    let mut lang_items: BTreeMap<String, [String; 3]> = BTreeMap::new();
    for id in &referenced_ids {
        let en = lookup_name(&en_raw, id).unwrap_or_else(|| format_id_title(id));
        let mx = lookup_name(&mx_raw, id).unwrap_or_else(|| en.clone());
        let es = lookup_name(&es_raw, id).unwrap_or_else(|| mx.clone());
        lang_items.insert(id.clone(), [mx, es, en]);
    }
    fs::write(
        data_dir.join("lang_items.json"),
        serde_json::to_string(&lang_items).unwrap(),
    )
    .unwrap();
    println!("Wrote data/lang_items.json with {} items", lang_items.len());

    pack_items_atlas(&repo);
}

/// Packs all `assets/items/*.png` into `assets/atlas.png` and `assets/atlas.json` at 32x32 nearest-neighbor.
fn pack_items_atlas(repo: &Path) {
    let items_dir = repo.join("assets/items");
    let mut png_files: Vec<_> = fs::read_dir(&items_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|x| x.path()))
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("png"))
        .collect();
    png_files.sort();

    let cell = 32u32;
    let total = png_files.len() as u32;
    let cols = (total as f32).sqrt().ceil() as u32;
    let rows = (total + cols - 1) / cols;
    let atlas_w = cols * cell;
    let atlas_h = rows * cell;
    let mut atlas_img: RgbaImage = ImageBuffer::from_pixel(atlas_w, atlas_h, Rgba([0, 0, 0, 0]));
    let mut entries = BTreeMap::new();

    for (idx, path) in png_files.iter().enumerate() {
        let id = path.file_stem().unwrap().to_string_lossy().to_string();
        if let Some(im) = load_first_frame(path) {
            let scaled = if im.width() == cell && im.height() == cell {
                im
            } else {
                image::imageops::resize(&im, cell, cell, FilterType::Nearest)
            };
            let gx = (idx as u32 % cols) * cell;
            let gy = (idx as u32 / cols) * cell;
            image::imageops::overlay(&mut atlas_img, &scaled, gx as i64, gy as i64);
            entries.insert(
                id,
                AtlasEntry {
                    x: gx,
                    y: gy,
                    w: cell,
                    h: cell,
                },
            );
        }
    }

    atlas_img.save(repo.join("assets/atlas.png")).unwrap();
    let manifest = AtlasManifest {
        width: atlas_w,
        height: atlas_h,
        cell_size: cell,
        entries,
    };
    fs::write(
        repo.join("assets/atlas.json"),
        serde_json::to_string(&manifest).unwrap(),
    )
    .unwrap();
    println!(
        "Built assets/atlas.png ({atlas_w}x{atlas_h}) packing {} crisp 32x32 icons!",
        manifest.entries.len()
    );
}

fn generate_v110_icons(repo: &Path) {
    let items_dir = repo.join("assets/items");

    // 1. Classic 3D isometric Red Bed derived faithfully from white_bed.png
    let white_bed_path = items_dir.join("white_bed.png");
    if let Ok(wb) = image::open(&white_bed_path) {
        let mut rb = wb.to_rgba8();
        let (w, h) = rb.dimensions();
        for y in 0..h {
            for x in 0..w {
                let p = *rb.get_pixel(x, y);
                if p[3] < 20 {
                    continue;
                }
                // Keep wooden headboard rails & legs untouched
                let is_wood = (p[0] as i32 - p[2] as i32) > 14;
                if is_wood {
                    continue;
                }
                // Blanket cuff line separating white pillow + white cuff strip from red quilt
                let seam_y = 7.4_f32 + 0.215_f32 * (x as f32);
                if (y as f32) > seam_y {
                    let luma = (p[0] as f32) / 245.0;
                    let r = (218.0 * luma).clamp(28.0, 248.0) as u8;
                    let g = (34.0 * luma).clamp(8.0, 72.0) as u8;
                    let b = (30.0 * luma).clamp(8.0, 68.0) as u8;
                    rb.put_pixel(x, y, Rgba([r, g, b, p[3]]));
                }
            }
        }
        rb.save(items_dir.join("red_bed.png")).unwrap();
        println!("Generated classic isometric assets/items/red_bed.png");
    }

    // 2. Unmistakable 32x32 Warden Head icon for Ancient City map landmarks
    const GRID: [&str; 16] = [
        "..H..........H..",
        ".HCH........HCH.",
        ".CH.H.KKKK.H.HC.",
        ".CCHCKDLLDKCHCC.",
        "..CCKDLBBLDKCC..",
        "..KDDDDDDDDDDK..",
        "..KDD00DD00DDK..",
        "..KDDDDDDDDDDK..",
        "..KLDDCHHCDDLK..",
        "..KD0B0BB0B0DK..",
        "..KD00000000DK..",
        "..KD0b0bb0b0DK..",
        "..KDD000000DDK..",
        "...KDDLLLLDDK...",
        "....KKKKKKKK....",
        "................",
    ];
    let mut small: RgbaImage = ImageBuffer::from_pixel(16, 16, Rgba([0, 0, 0, 0]));
    for (y, row) in GRID.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let color = match ch {
                'H' => Rgba([72, 238, 232, 255]),
                'C' => Rgba([26, 168, 182, 255]),
                'K' => Rgba([8, 20, 28, 255]),
                'D' => Rgba([18, 56, 68, 255]),
                'L' => Rgba([30, 84, 98, 255]),
                'B' => Rgba([212, 204, 174, 255]),
                'b' => Rgba([238, 232, 208, 255]),
                '0' => Rgba([5, 11, 16, 255]),
                _ => Rgba([0, 0, 0, 0]),
            };
            small.put_pixel(x as u32, y as u32, color);
        }
    }
    let upscaled = image::imageops::resize(&small, 32, 32, FilterType::Nearest);
    upscaled.save(items_dir.join("warden_head.png")).unwrap();
    println!("Generated assets/items/warden_head.png");
}
