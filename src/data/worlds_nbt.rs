use flate2::read::GzDecoder;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

/// A detected local Minecraft Java Edition singleplayer world save.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DetectedWorld {
    pub folder_name: String,
    pub level_name: String,
    pub seed: i64,
    pub game_type: i32,
    pub player_pos: Option<[f64; 3]>,
    pub player_dim: Option<String>,
    pub last_played_ms: i64,
}

/// A local schematic file found in `.minecraft/schematics` or `.minecraft/config/litematica`.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DetectedSchematic {
    pub file_name: String,
    pub full_path: PathBuf,
    pub size_bytes: u64,
}

#[derive(Debug, Deserialize)]
struct RootNbt {
    #[serde(rename = "Data")]
    data: Option<LevelDataNbt>,
}

#[derive(Debug, Deserialize)]
struct LevelDataNbt {
    #[serde(rename = "LevelName")]
    level_name: Option<String>,
    #[serde(rename = "RandomSeed")]
    random_seed: Option<i64>,
    #[serde(rename = "WorldGenSettings")]
    world_gen_settings: Option<WorldGenSettingsNbt>,
    #[serde(rename = "GameType")]
    game_type: Option<i32>,
    #[serde(rename = "LastPlayed")]
    last_played: Option<i64>,
    #[serde(rename = "Player")]
    player: Option<PlayerNbt>,
}

#[derive(Debug, Deserialize)]
struct WorldGenSettingsNbt {
    seed: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct PlayerNbt {
    #[serde(rename = "Pos")]
    pos: Option<Vec<f64>>,
    #[serde(rename = "Dimension")]
    dimension: Option<fastnbt::Value>,
}

/// Resolves candidate Minecraft data directories across Windows, macOS, and Linux launchers.
pub fn discover_minecraft_roots() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(appdata) = std::env::var("APPDATA") {
        if !appdata.is_empty() {
            candidates.push(PathBuf::from(&appdata).join(".minecraft"));
            candidates.push(PathBuf::from(&appdata).join("PrismLauncher/instances"));
        }
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        if !profile.is_empty() {
            candidates.push(PathBuf::from(profile).join("AppData/Roaming/.minecraft"));
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            let h = PathBuf::from(&home);
            #[cfg(target_os = "macos")]
            {
                candidates.push(h.join("Library/Application Support/minecraft"));
                candidates.push(h.join("Library/Application Support/PrismLauncher/instances"));
            }
            candidates.push(h.join(".minecraft"));
            candidates.push(h.join(".var/app/com.mojang.Minecraft/.minecraft"));
            candidates.push(h.join(".var/app/com.mojang.Minecraft/data/minecraft"));
            candidates.push(h.join(".local/share/PrismLauncher/instances"));
        }
    }

    let mut roots = Vec::new();
    for cand in candidates {
        if !cand.exists() {
            continue;
        }
        if cand.file_name().and_then(|n| n.to_str()) == Some("instances") {
            if let Ok(entries) = fs::read_dir(&cand) {
                for inst in entries.flatten() {
                    let inst_path = inst.path();
                    for sub in [".minecraft", "minecraft"] {
                        let p = inst_path.join(sub);
                        if p.is_dir() && !roots.contains(&p) {
                            roots.push(p);
                        }
                    }
                }
            }
        } else if !roots.contains(&cand) {
            roots.push(cand);
        }
    }
    roots
}

/// Resolves the primary `.minecraft` / `minecraft` root directory on the host OS.
pub fn minecraft_root_dir() -> Option<PathBuf> {
    discover_minecraft_roots().into_iter().next()
}

/// Safety caps against malformed or decompression-bomb `level.dat` inputs.
const MAX_LEVEL_DAT_COMPRESSED_BYTES: u64 = 16 * 1024 * 1024;
const MAX_LEVEL_DAT_UNCOMPRESSED_BYTES: u64 = 32 * 1024 * 1024;
const MAX_ASSET_INDEX_BYTES: u64 = 32 * 1024 * 1024;

/// Parses a single gzip-compressed `level.dat` NBT byte slice cleanly with bounded decompression.
pub fn parse_level_dat_bytes(bytes: &[u8], folder_name: &str) -> Result<DetectedWorld, String> {
    if (bytes.len() as u64) > MAX_LEVEL_DAT_COMPRESSED_BYTES {
        return Err("level.dat compressed payload exceeds safety limit".to_string());
    }
    let decoder = GzDecoder::new(bytes);
    let mut uncompressed = Vec::new();
    decoder
        .take(MAX_LEVEL_DAT_UNCOMPRESSED_BYTES + 1)
        .read_to_end(&mut uncompressed)
        .map_err(|e| format!("Gzip decode failed: {}", e))?;
    if (uncompressed.len() as u64) > MAX_LEVEL_DAT_UNCOMPRESSED_BYTES {
        return Err("level.dat uncompressed NBT exceeds safety limit".to_string());
    }

    let root: RootNbt = fastnbt::from_bytes(&uncompressed)
        .map_err(|e| format!("NBT parse failed: {}", e))?;
    let data = root.data.ok_or_else(|| "Missing Data compound".to_string())?;

    let seed = data
        .world_gen_settings
        .and_then(|w| w.seed)
        .or(data.random_seed)
        .ok_or_else(|| "No seed present in level.dat".to_string())?;

    let level_name = data
        .level_name
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| folder_name.to_string());

    let (player_pos, player_dim) = if let Some(pl) = data.player {
        let pos = pl.pos.and_then(|v| {
            if v.len() >= 3 {
                Some([v[0], v[1], v[2]])
            } else {
                None
            }
        });
        let dim = pl.dimension.map(|v| match v {
            fastnbt::Value::String(s) => s,
            fastnbt::Value::Int(-1) => "minecraft:the_nether".to_string(),
            fastnbt::Value::Int(1) => "minecraft:the_end".to_string(),
            _ => "minecraft:overworld".to_string(),
        });
        (pos, dim)
    } else {
        (None, None)
    };

    Ok(DetectedWorld {
        folder_name: folder_name.to_string(),
        level_name,
        seed,
        game_type: data.game_type.unwrap_or(0),
        player_pos,
        player_dim,
        last_played_ms: data.last_played.unwrap_or(0),
    })
}

/// Scans `.minecraft/saves/*/level.dat` sorted by most recently played.
pub fn scan_local_worlds() -> Vec<DetectedWorld> {
    let mut out = Vec::new();
    let Some(mc_dir) = minecraft_root_dir() else {
        return out;
    };
    let saves_dir = mc_dir.join("saves");
    let Ok(entries) = fs::read_dir(&saves_dir) else {
        return out;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let level_dat = path.join("level.dat");
        if !level_dat.exists() {
            continue;
        }
        let Ok(file) = File::open(&level_dat) else {
            continue;
        };
        let mut buf = Vec::new();
        if file
            .take(MAX_LEVEL_DAT_COMPRESSED_BYTES + 1)
            .read_to_end(&mut buf)
            .is_err()
        {
            continue;
        }
        let folder = entry.file_name().to_string_lossy().to_string();
        if let Ok(world) = parse_level_dat_bytes(&buf, &folder) {
            out.push(world);
        }
    }

    out.sort_by(|a, b| b.last_played_ms.cmp(&a.last_played_ms));
    out
}

/// Scans `.minecraft/schematics`, `.minecraft/config/litematica`, and local `./schematics` for `.litematic` / `.schem` files.
pub fn scan_local_schematics() -> Vec<DetectedSchematic> {
    let mut dirs = Vec::new();
    if let Some(mc) = minecraft_root_dir() {
        dirs.push(mc.join("schematics"));
        dirs.push(mc.join("config/litematica"));
    }
    dirs.push(PathBuf::from("schematics"));

    let mut found = Vec::new();
    for dir in dirs {
        collect_schematics_in(&dir, &mut found);
    }
    found.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    found
}

fn collect_schematics_in(dir: &Path, out: &mut Vec<DetectedSchematic>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let lower = ext.to_ascii_lowercase();
                if lower == "litematic" || lower == "schem" || lower == "schematic" || lower == "nbt" {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    out.push(DetectedSchematic {
                        file_name: entry.file_name().to_string_lossy().to_string(),
                        full_path: path,
                        size_bytes: size,
                    });
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct AssetIndexFile {
    objects: HashMap<String, AssetObjectEntry>,
}

#[derive(Debug, Deserialize)]
struct AssetObjectEntry {
    hash: String,
}

/// Attempts to locate real Minecraft `.ogg` sound files from local `.minecraft/assets/indexes/`.
pub fn find_minecraft_sound_paths() -> HashMap<String, PathBuf> {
    let mut resolved = HashMap::new();
    let Some(mc_dir) = minecraft_root_dir() else {
        return resolved;
    };
    let indexes_dir = mc_dir.join("assets/indexes");
    let objects_dir = mc_dir.join("assets/objects");
    if !indexes_dir.exists() || !objects_dir.exists() {
        return resolved;
    }

    // Pick newest numeric/semver index JSON automatically
    let mut index_files: Vec<(std::time::SystemTime, PathBuf)> = fs::read_dir(&indexes_dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("json") {
                let mtime = e.metadata().ok()?.modified().ok()?;
                Some((mtime, p))
            } else {
                None
            }
        })
        .collect();
    index_files.sort_by(|a, b| b.0.cmp(&a.0));

    let wanted = [
        ("click", "minecraft/sounds/random/click.ogg"),
        ("anvil_use", "minecraft/sounds/random/anvil_use.ogg"),
        ("orb", "minecraft/sounds/random/orb.ogg"),
        ("levelup", "minecraft/sounds/random/levelup.ogg"),
        ("toast", "minecraft/sounds/ui/toast/in.ogg"),
    ];

    for (_, idx_path) in index_files {
        if idx_path.metadata().map(|m| m.len()).unwrap_or(u64::MAX) > MAX_ASSET_INDEX_BYTES {
            continue;
        }
        let Ok(txt) = fs::read_to_string(&idx_path) else {
            continue;
        };
        let Ok(parsed) = serde_json::from_str::<AssetIndexFile>(&txt) else {
            continue;
        };
        for (alias, key) in &wanted {
            if resolved.contains_key(*alias) {
                continue;
            }
            if let Some(obj) = parsed.objects.get(*key) {
                if is_valid_sha1_hex(&obj.hash) {
                    let p = objects_dir.join(&obj.hash[..2]).join(&obj.hash);
                    if p.exists() {
                        resolved.insert((*alias).to_string(), p);
                    }
                }
            }
        }
        if resolved.len() == wanted.len() {
            break;
        }
    }
    resolved
}

fn is_valid_sha1_hex(hash: &str) -> bool {
    hash.len() == 40 && hash.bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use serde::Serialize;
    use std::io::Write;

    #[derive(Serialize)]
    struct FakeRoot {
        #[serde(rename = "Data")]
        data: FakeData,
    }
    #[derive(Serialize)]
    struct FakeData {
        #[serde(rename = "LevelName")]
        level_name: String,
        #[serde(rename = "WorldGenSettings")]
        world_gen: FakeWorldGen,
        #[serde(rename = "GameType")]
        game_type: i32,
        #[serde(rename = "LastPlayed")]
        last_played: i64,
    }
    #[derive(Serialize)]
    struct FakeWorldGen {
        seed: i64,
    }

    #[test]
    fn parses_synthetic_level_dat() {
        let root = FakeRoot {
            data: FakeData {
                level_name: "Technical 26.3".to_string(),
                world_gen: FakeWorldGen { seed: -42198374129834i64 },
                game_type: 0,
                last_played: 1_700_000_000_000,
            },
        };
        let raw_nbt = fastnbt::to_bytes(&root).expect("serialize synthetic nbt");
        let mut gz = GzEncoder::new(Vec::new(), Compression::fast());
        gz.write_all(&raw_nbt).expect("gzip write");
        let gz_bytes = gz.finish().expect("gzip finish");

        let parsed = parse_level_dat_bytes(&gz_bytes, "WorldFolder").expect("parse level.dat");
        assert_eq!(parsed.level_name, "Technical 26.3");
        assert_eq!(parsed.seed, -42198374129834i64);
    }

    #[test]
    fn rejects_oversized_gzip_level_dat_and_traversal_hashes() {
        assert!(!is_valid_sha1_hex("../../Windows/System32/cmd.exe"));
        assert!(!is_valid_sha1_hex("áé"));
        assert!(is_valid_sha1_hex("0123456789abcdef0123456789abcdef01234567"));

        let huge_payload = vec![b'A'; (MAX_LEVEL_DAT_UNCOMPRESSED_BYTES as usize) + 1024];
        let mut gz = GzEncoder::new(Vec::new(), Compression::fast());
        gz.write_all(&huge_payload).expect("compress zeros");
        let bomb = gz.finish().expect("finish bomb");
        let err = parse_level_dat_bytes(&bomb, "BombWorld").unwrap_err();
        assert!(err.contains("safety limit"));
    }
}
