//! Native Rust `.litematic` (Litematica NBT) parser with gzip-bomb limits,
//! overflow-safe bit-packed `BlockStates` decoding, and Shulker/Stack formatting.

use fastnbt::LongArray;
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::data::models::Language;

const MAX_COMPRESSED_BYTES: u64 = 32 * 1024 * 1024;
const MAX_UNCOMPRESSED_BYTES: u64 = 64 * 1024 * 1024;
const MAX_PALETTE_LEN: usize = 65_536;
const MAX_TOTAL_VOLUME: u64 = 64_000_000;

/// One aggregated material requirement decoded from a `.litematic` schematic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LitematicMaterial {
    pub item_id: String,
    pub count: u64,
}

/// Decoded summary + exact material breakdown of a `.litematic` schematic.
#[derive(Debug, Clone)]
pub struct ParsedLitematic {
    pub file_path: PathBuf,
    pub name: String,
    pub author: String,
    pub dims: (u32, u32, u32),
    pub region_count: usize,
    pub total_blocks: u64,
    pub materials: Vec<LitematicMaterial>,
}

#[derive(Debug, Deserialize)]
struct LitematicRoot {
    #[serde(rename = "Metadata")]
    metadata: Option<LitematicMetadata>,
    #[serde(rename = "Regions")]
    regions: Option<HashMap<String, LitematicRegion>>,
}

#[derive(Debug, Deserialize)]
struct LitematicMetadata {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "Author")]
    author: Option<String>,
    #[serde(rename = "EnclosingSize")]
    enclosing_size: Option<Vec3I32>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct Vec3I32 {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug, Deserialize)]
struct LitematicRegion {
    #[serde(rename = "Size")]
    size: Option<Vec3I32>,
    #[serde(rename = "BlockStatePalette")]
    palette: Option<Vec<BlockStateEntry>>,
    #[serde(rename = "BlockStates")]
    block_states: Option<LongArray>,
}

#[derive(Debug, Deserialize)]
struct BlockStateEntry {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Properties")]
    properties: Option<HashMap<String, String>>,
}

/// Parses a `.litematic` file safely from disk with strict size & decompression bounds.
pub fn parse_litematic_file(path: &Path) -> Result<ParsedLitematic, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("Cannot read file metadata: {e}"))?;
    if meta.len() == 0 {
        return Err("Schematic file is empty.".to_string());
    }
    if meta.len() > MAX_COMPRESSED_BYTES {
        return Err(format!(
            "Schematic exceeds {} MiB compressed limit.",
            MAX_COMPRESSED_BYTES / (1024 * 1024)
        ));
    }

    let file = File::open(path).map_err(|e| format!("Cannot open schematic: {e}"))?;
    let mut decoder = GzDecoder::new(file).take(MAX_UNCOMPRESSED_BYTES + 1);
    let mut raw_nbt = Vec::new();
    decoder
        .read_to_end(&mut raw_nbt)
        .map_err(|e| format!("Invalid GZip/Litematica archive: {e}"))?;

    if raw_nbt.len() as u64 > MAX_UNCOMPRESSED_BYTES {
        return Err(format!(
            "Schematic exceeds {} MiB decompressed safety cap.",
            MAX_UNCOMPRESSED_BYTES / (1024 * 1024)
        ));
    }

    let fallback_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Schematic")
        .to_string();
    parse_litematic_nbt_bytes(&raw_nbt, path.to_path_buf(), &fallback_name)
}

/// Decodes uncompressed NBT bytes from a `.litematic` payload into aggregated block counts.
pub fn parse_litematic_nbt_bytes(
    nbt_bytes: &[u8],
    file_path: PathBuf,
    fallback_name: &str,
) -> Result<ParsedLitematic, String> {
    let root: LitematicRoot =
        fastnbt::from_bytes(nbt_bytes).map_err(|e| format!("Invalid Litematica NBT: {e}"))?;

    let regions = root
        .regions
        .ok_or_else(|| "Missing 'Regions' compound in .litematic file.".to_string())?;
    if regions.is_empty() {
        return Err("Schematic contains no regions.".to_string());
    }

    let mut counts: HashMap<String, u64> = HashMap::new();
    let mut total_blocks: u64 = 0;
    let mut cumulative_volume: u64 = 0;
    let mut max_dims = (0u32, 0u32, 0u32);

    for (region_name, region) in &regions {
        let size = region
            .size
            .ok_or_else(|| format!("Region '{}' missing Size.", region_name))?;
        let sx = size.x.unsigned_abs();
        let sy = size.y.unsigned_abs();
        let sz = size.z.unsigned_abs();
        max_dims = (max_dims.0.max(sx), max_dims.1.max(sy), max_dims.2.max(sz));

        let volume = (sx as u64)
            .checked_mul(sy as u64)
            .and_then(|v| v.checked_mul(sz as u64))
            .ok_or_else(|| "Region volume arithmetic overflow.".to_string())?;
        cumulative_volume = cumulative_volume
            .checked_add(volume)
            .ok_or_else(|| "Total schematic volume overflow.".to_string())?;
        if cumulative_volume > MAX_TOTAL_VOLUME {
            return Err(format!(
                "Schematic volume exceeds {}M block safety limit.",
                MAX_TOTAL_VOLUME / 1_000_000
            ));
        }
        if volume == 0 {
            continue;
        }

        let palette = region
            .palette
            .as_ref()
            .ok_or_else(|| format!("Region '{}' missing BlockStatePalette.", region_name))?;
        if palette.is_empty() || palette.len() > MAX_PALETTE_LEN {
            return Err(format!(
                "Invalid BlockStatePalette length ({}) in region '{}'.",
                palette.len(),
                region_name
            ));
        }

        // Precompute normalized item ID and multiplier per palette entry (e.g. double slabs = 2)
        let palette_info: Vec<Option<(String, u64)>> = palette
            .iter()
            .map(|entry| normalize_block_state_to_item(&entry.name, entry.properties.as_ref()))
            .collect();

        if palette.len() == 1 {
            if let Some((item_id, mult)) = &palette_info[0] {
                let added = volume.saturating_mul(*mult);
                *counts.entry(item_id.clone()).or_insert(0) += added;
                total_blocks = total_blocks.saturating_add(added);
            }
            continue;
        }

        let states = region
            .block_states
            .as_ref()
            .ok_or_else(|| format!("Region '{}' missing BlockStates.", region_name))?;
        let raw_longs: &[i64] = states;

        let bits = palette
            .len()
            .saturating_sub(1)
            .checked_ilog2()
            .map(|v| (v + 1) as usize)
            .unwrap_or(2)
            .max(2);
        let needed_bits = volume
            .checked_mul(bits as u64)
            .ok_or_else(|| "BlockStates bit-length overflow.".to_string())?;
        let needed_longs = needed_bits.div_ceil(64) as usize;
        if raw_longs.len() < needed_longs {
            return Err(format!(
                "Truncated BlockStates in region '{}' (expected >= {} longs, got {}).",
                region_name,
                needed_longs,
                raw_longs.len()
            ));
        }

        // Accumulate palette indices first in a small dense array (much faster than hashing per block!)
        let mut palette_hits = vec![0u64; palette.len()];
        let mask: u64 = (1u64 << bits) - 1;
        let bits_u32 = bits as u32;

        for idx in 0..volume {
            let start_bit = idx * (bits as u64);
            let long_idx = (start_bit >> 6) as usize;
            let bit_off = (start_bit & 63) as u32;

            let low = (raw_longs[long_idx] as u64) >> bit_off;
            let pal_idx = if bit_off + bits_u32 > 64 {
                let high = (raw_longs[long_idx + 1] as u64) << (64 - bit_off);
                ((low | high) & mask) as usize
            } else {
                (low & mask) as usize
            };

            if let Some(slot) = palette_hits.get_mut(pal_idx) {
                *slot += 1;
            }
        }

        for (pal_idx, hit_count) in palette_hits.into_iter().enumerate() {
            if hit_count == 0 {
                continue;
            }
            if let Some((item_id, mult)) = &palette_info[pal_idx] {
                let added = hit_count.saturating_mul(*mult);
                *counts.entry(item_id.clone()).or_insert(0) += added;
                total_blocks = total_blocks.saturating_add(added);
            }
        }
    }

    let mut materials: Vec<LitematicMaterial> = counts
        .into_iter()
        .map(|(item_id, count)| LitematicMaterial { item_id, count })
        .collect();
    materials.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.item_id.cmp(&b.item_id)));

    let dims = root
        .metadata
        .as_ref()
        .and_then(|m| m.enclosing_size)
        .map(|s| (s.x.unsigned_abs(), s.y.unsigned_abs(), s.z.unsigned_abs()))
        .unwrap_or(max_dims);

    let name = root
        .metadata
        .as_ref()
        .and_then(|m| m.name.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(fallback_name)
        .to_string();

    let author = root
        .metadata
        .as_ref()
        .and_then(|m| m.author.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Unknown")
        .to_string();

    Ok(ParsedLitematic {
        file_path,
        name,
        author,
        dims,
        region_count: regions.len(),
        total_blocks,
        materials,
    })
}

fn normalize_block_state_to_item(
    raw_name: &str,
    props: Option<&HashMap<String, String>>,
) -> Option<(String, u64)> {
    let id = raw_name.strip_prefix("minecraft:").unwrap_or(raw_name);
    if matches!(
        id,
        "air"
            | "cave_air"
            | "void_air"
            | "moving_piston"
            | "piston_head"
            | "bubble_column"
            | "fire"
            | "soul_fire"
            | "light"
            | "structure_void"
            | "nether_portal"
            | "end_portal"
            | "end_gateway"
    ) {
        return None;
    }

    // Two-block tall plants / doors / beds: only count the lower/foot half so totals match survival items
    if let Some(p) = props {
        if let Some(half) = p.get("half") {
            if half == "upper"
                && (id.ends_with("_door")
                    || matches!(
                        id,
                        "sunflower"
                            | "lilac"
                            | "rose_bush"
                            | "peony"
                            | "tall_grass"
                            | "large_fern"
                            | "tall_seagrass"
                            | "pitcher_plant"
                    ))
            {
                return None;
            }
        }
        if id.ends_with("_bed") && p.get("part").map(String::as_str) == Some("head") {
            return None;
        }
    }

    let mult: u64 = if let Some(p) = props {
        if id.ends_with("_slab") && p.get("type").map(String::as_str) == Some("double") {
            2
        } else if let Some(candles) = p.get("candles").and_then(|v| v.parse::<u64>().ok()) {
            candles.clamp(1, 4)
        } else if let Some(pickles) = p.get("pickles").and_then(|v| v.parse::<u64>().ok()) {
            pickles.clamp(1, 4)
        } else {
            1
        }
    } else {
        1
    };

    let mapped = match id {
        "redstone_wire" => "redstone",
        "wall_torch" => "torch",
        "soul_wall_torch" => "soul_torch",
        "redstone_wall_torch" => "redstone_torch",
        "tripwire" => "string",
        "water" => "water_bucket",
        "lava" => "lava_bucket",
        "powder_snow" => "powder_snow_bucket",
        "wheat" => "wheat_seeds",
        "carrots" => "carrot",
        "potatoes" => "potato",
        "beetroots" => "beetroot_seeds",
        "melon_stem" | "attached_melon_stem" => "melon_seeds",
        "pumpkin_stem" | "attached_pumpkin_stem" => "pumpkin_seeds",
        "bamboo_sapling" => "bamboo",
        "sweet_berry_bush" => "sweet_berries",
        "cave_vines" | "cave_vines_plant" => "glow_berries",
        "kelp_plant" => "kelp",
        "tall_seagrass" => "seagrass",
        "tall_grass" => "short_grass",
        "twisting_vines_plant" => "twisting_vines",
        "weeping_vines_plant" => "weeping_vines",
        "big_dripleaf_stem" => "big_dripleaf",
        "water_cauldron" | "lava_cauldron" | "powder_snow_cauldron" => "cauldron",
        "frosted_ice" => "ice",
        "farmland" | "dirt_path" => "dirt",
        other if other.starts_with("potted_") => "flower_pot",
        other if other.ends_with("_wall_sign") => return Some((other.replace("_wall_sign", "_sign"), mult)),
        other if other.ends_with("_wall_hanging_sign") => {
            return Some((other.replace("_wall_hanging_sign", "_hanging_sign"), mult))
        }
        other if other.ends_with("_wall_banner") => {
            return Some((other.replace("_wall_banner", "_banner"), mult))
        }
        other if other.ends_with("_wall_head") => {
            return Some((other.replace("_wall_head", "_head"), mult))
        }
        other if other.ends_with("_wall_skull") => {
            return Some((other.replace("_wall_skull", "_skull"), mult))
        }
        other => other,
    };

    Some((mapped.to_string(), mult))
}

/// Formats an item count into human-friendly Shulker Boxes (x1,728) + Stacks (x64) + remainder,
/// or comma-grouped exact units.
pub fn format_mc_quantity(count: u64, shulker_stacks_mode: bool, lang: Language) -> String {
    if !shulker_stacks_mode {
        return format_with_commas(count);
    }
    const STACK: u64 = 64;
    const SHULKER: u64 = 27 * 64; // 1,728 items

    let shulkers = count / SHULKER;
    let rem_after_sb = count % SHULKER;
    let stacks = rem_after_sb / STACK;
    let singles = rem_after_sb % STACK;

    let sb_lbl = match lang {
        Language::EnUs => "SB",
        _ => "Shulk",
    };

    if shulkers > 0 {
        match (stacks, singles) {
            (0, 0) => format!("{} {}", shulkers, sb_lbl),
            (s, 0) => format!("{} {} + {}x64", shulkers, sb_lbl, s),
            (0, u) => format!("{} {} + {}", shulkers, sb_lbl, u),
            (s, u) => format!("{} {} + {}x64 + {}", shulkers, sb_lbl, s, u),
        }
    } else if stacks > 0 {
        if singles > 0 {
            format!("{}x64 + {}", stacks, singles)
        } else {
            format!("{}x64", stacks)
        }
    } else {
        count.to_string()
    }
}

pub fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    let len = s.len();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use fastnbt::Value;

    /// Packs palette indices using Litematica's cross-word bit-packing format for test validation.
    fn pack_litematica_states(indices: &[usize], palette_len: usize) -> Vec<i64> {
        let bits = palette_len
            .saturating_sub(1)
            .checked_ilog2()
            .map(|v| (v + 1) as usize)
            .unwrap_or(2)
            .max(2);
        let needed_longs = (indices.len() * bits).div_ceil(64);
        let mut out = vec![0u64; needed_longs];
        for (i, &val) in indices.iter().enumerate() {
            let start_bit = i * bits;
            let li = start_bit / 64;
            let off = start_bit % 64;
            out[li] |= (val as u64) << off;
            if off + bits > 64 {
                out[li + 1] |= (val as u64) >> (64 - off);
            }
        }
        out.into_iter().map(|u| u as i64).collect()
    }

    #[test]
    fn parses_synthetic_litematic_with_cross_word_bits_and_double_slabs() {
        // Palette with 5 entries => 3 bits per block (crosses 64-bit word boundary at index 21!)
        let palette_vals: Vec<Value> = vec![
            HashMap::from([("Name".to_string(), Value::String("minecraft:air".into()))]),
            HashMap::from([("Name".to_string(), Value::String("minecraft:hopper".into()))]),
            HashMap::from([("Name".to_string(), Value::String("minecraft:redstone_wire".into()))]),
            HashMap::from([
                ("Name".to_string(), Value::String("minecraft:smooth_stone_slab".into())),
                (
                    "Properties".to_string(),
                    Value::Compound(HashMap::from([(
                        "type".to_string(),
                        Value::String("double".into()),
                    )])),
                ),
            ]),
            HashMap::from([("Name".to_string(), Value::String("minecraft:glass".into()))]),
        ]
        .into_iter()
        .map(Value::Compound)
        .collect();

        // Region Size = (-3, 3, 3) => 27 blocks (negative X tests unsigned_abs!)
        let mut indices = vec![0usize; 27];
        indices[0] = 1; // hopper
        indices[20] = 2; // redstone_wire -> redstone
        indices[21] = 3; // crosses bit 63..65 boundary! double smooth_stone_slab => +2
        indices[22] = 4; // glass
        indices[26] = 1; // hopper

        let packed = pack_litematica_states(&indices, 5);
        let region = HashMap::from([
            (
                "Size".to_string(),
                Value::Compound(HashMap::from([
                    ("x".to_string(), Value::Int(-3)),
                    ("y".to_string(), Value::Int(3)),
                    ("z".to_string(), Value::Int(3)),
                ])),
            ),
            ("BlockStatePalette".to_string(), Value::List(palette_vals)),
            ("BlockStates".to_string(), Value::LongArray(LongArray::new(packed))),
        ]);

        let root = Value::Compound(HashMap::from([
            (
                "Metadata".to_string(),
                Value::Compound(HashMap::from([
                    ("Name".to_string(), Value::String("Iron Mini".into())),
                    ("Author".to_string(), Value::String("Tester".into())),
                ])),
            ),
            (
                "Regions".to_string(),
                Value::Compound(HashMap::from([("Main".to_string(), Value::Compound(region))])),
            ),
        ]));

        let nbt_bytes = fastnbt::to_bytes(&root).expect("serialize synthetic litematic NBT");
        let parsed = parse_litematic_nbt_bytes(&nbt_bytes, PathBuf::from("test.litematic"), "fallback")
            .expect("decode synthetic litematic");

        assert_eq!(parsed.name, "Iron Mini");
        assert_eq!(parsed.author, "Tester");
        assert_eq!(parsed.dims, (3, 3, 3));
        assert_eq!(parsed.total_blocks, 6); // 2 hoppers + 1 redstone + 2 slabs + 1 glass
        let map: HashMap<_, _> = parsed
            .materials
            .iter()
            .map(|m| (m.item_id.as_str(), m.count))
            .collect();
        assert_eq!(map.get("hopper"), Some(&2));
        assert_eq!(map.get("redstone"), Some(&1));
        assert_eq!(map.get("smooth_stone_slab"), Some(&2));
        assert_eq!(map.get("glass"), Some(&1));
        assert!(!map.contains_key("air"));
    }

    #[test]
    fn formats_shulker_and_stack_quantities() {
        assert_eq!(format_mc_quantity(48, true, Language::EnUs), "48");
        assert_eq!(format_mc_quantity(128, true, Language::EnUs), "2x64");
        assert_eq!(format_mc_quantity(130, true, Language::EsMx), "2x64 + 2");
        assert_eq!(format_mc_quantity(1728, true, Language::EnUs), "1 SB");
        assert_eq!(
            format_mc_quantity(1728 * 2 + 64 * 3 + 5, true, Language::EsMx),
            "2 Shulk + 3x64 + 5"
        );
        assert_eq!(format_mc_quantity(12480, false, Language::EnUs), "12,480");
    }
}
