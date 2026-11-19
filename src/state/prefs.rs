use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::data::models::Language;
use crate::data::seed_engine::Dimension;

/// Switchable Minecraft visual themes backed by scenic wallpapers, 360° panoramas, or `.cavf` video loops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ThemeId {
    #[default]
    DeepslateOre,
    DeepDarkSculk,
    TrialChambers,
    PaleGarden,
    OceanMonument,
    ClassicJavaStone,
    NetherCrimsonEnd,
    EndVoidAurora,
    CherryBlossom,
    StarryOverworld,
    CustomVideo,
}

impl ThemeId {
    pub fn all() -> [ThemeId; 11] {
        [
            ThemeId::DeepDarkSculk,
            ThemeId::TrialChambers,
            ThemeId::PaleGarden,
            ThemeId::OceanMonument,
            ThemeId::DeepslateOre,
            ThemeId::ClassicJavaStone,
            ThemeId::NetherCrimsonEnd,
            ThemeId::EndVoidAurora,
            ThemeId::CherryBlossom,
            ThemeId::StarryOverworld,
            ThemeId::CustomVideo,
        ]
    }

    pub fn is_video(self) -> bool {
        matches!(
            self,
            ThemeId::DeepDarkSculk
                | ThemeId::TrialChambers
                | ThemeId::PaleGarden
                | ThemeId::OceanMonument
                | ThemeId::CustomVideo
        )
    }

    pub fn label(self, lang: Language) -> &'static str {
        match (self, lang) {
            (ThemeId::DeepDarkSculk, Language::EnUs) => "Deep Dark [Video]",
            (ThemeId::DeepDarkSculk, _) => "Ciudad Antigua [Video]",
            (ThemeId::TrialChambers, Language::EnUs) => "Trial Chambers [Video]",
            (ThemeId::TrialChambers, Language::EsEs) => "Cámaras de Juicio [Video]",
            (ThemeId::TrialChambers, _) => "Trial Chambers [Video]",
            (ThemeId::PaleGarden, Language::EnUs) => "Pale Garden [Video]",
            (ThemeId::PaleGarden, _) => "Jardín Pálido [Video]",
            (ThemeId::OceanMonument, Language::EnUs) => "Ocean Monument [Video]",
            (ThemeId::OceanMonument, _) => "Monumento Oceánico [Video]",
            (ThemeId::DeepslateOre, Language::EnUs) => "Deep Cave Glow",
            (ThemeId::DeepslateOre, _) => "Cueva Profunda",
            (ThemeId::ClassicJavaStone, Language::EnUs) => "Wilderness Panorama",
            (ThemeId::ClassicJavaStone, _) => "Panorama Salvaje",
            (ThemeId::NetherCrimsonEnd, Language::EnUs) => "Crimson Nether",
            (ThemeId::NetherCrimsonEnd, _) => "Nether Carmesí",
            (ThemeId::EndVoidAurora, Language::EnUs) => "End Aurora",
            (ThemeId::EndVoidAurora, _) => "Aurora del End",
            (ThemeId::CherryBlossom, Language::EnUs) => "Cherry Sunset",
            (ThemeId::CherryBlossom, _) => "Cerezo Atardecer",
            (ThemeId::StarryOverworld, Language::EnUs) => "Starry Ruin",
            (ThemeId::StarryOverworld, _) => "Noche Estrellada",
            (ThemeId::CustomVideo, Language::EnUs) => "Custom Video [Video]",
            (ThemeId::CustomVideo, _) => "Mi Video [Video]",
        }
    }
}

/// Custom user waypoint stored per seed inside `UserPrefs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub id: u64,
    pub seed: i64,
    pub dim: i32,
    pub x: i32,
    pub z: i32,
    pub name: String,
    pub icon: String,
}

impl Waypoint {
    pub fn dimension(&self) -> Dimension {
        match self.dim {
            -1 => Dimension::Nether,
            1 => Dimension::TheEnd,
            _ => Dimension::Overworld,
        }
    }
}

/// Bookmarked world or multiplayer server seed stored persistently inside `UserPrefs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedSeedEntry {
    pub id: u64,
    pub name: String,
    pub seed_text: String,
    pub seed: i64,
    #[serde(default)]
    pub large_biomes: bool,
    #[serde(default)]
    pub is_server: bool,
    #[serde(default)]
    pub saved_at_ms: i64,
}

/// Recently explored seed history entry stored persistently inside `UserPrefs` (max 20 entries).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeedHistoryEntry {
    pub seed_text: String,
    pub seed: i64,
    #[serde(default)]
    pub large_biomes: bool,
    #[serde(default)]
    pub source_label: String,
    #[serde(default)]
    pub used_at_ms: i64,
}

pub const MAX_SEED_HISTORY: usize = 20;

/// One material row inside the unified persistent Shopping List (fed by JEI trees, Farms, or .litematic).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShoppingListItem {
    pub item_id: String,
    #[serde(default)]
    pub label: String,
    pub count: u64,
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub source: String,
}

/// Persistent user configuration serialized safely under `%APPDATA%\craft-atlas\prefs.json`
/// (with automatic read fallback from legacy `%APPDATA%\minecraft-companion\prefs.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UserPrefs {
    pub language: Language,
    pub selected_version: String,
    pub theme: ThemeId,
    pub audio_enabled: bool,
    pub audio_volume: f32,
    pub last_seed: String,
    pub large_biomes: bool,
    pub favorites_open: bool,
    pub favorite_items: Vec<String>,
    pub favorite_gear: Vec<String>,
    pub favorite_farms: Vec<String>,
    pub waypoints: Vec<Waypoint>,
    pub checked_materials: HashSet<String>,
    #[serde(default)]
    pub saved_seeds: Vec<SavedSeedEntry>,
    #[serde(default)]
    pub seed_history: Vec<SeedHistoryEntry>,
    #[serde(default)]
    pub shopping_list: Vec<ShoppingListItem>,
    #[serde(default = "default_true")]
    pub shopping_shulker_mode: bool,
}

fn default_true() -> bool {
    true
}

impl Default for UserPrefs {
    fn default() -> Self {
        Self {
            language: Language::EsMx,
            selected_version: "26.3".to_string(),
            theme: ThemeId::DeepslateOre,
            audio_enabled: true,
            audio_volume: 0.45,
            last_seed: "-4172144997902289642".to_string(),
            large_biomes: false,
            favorites_open: true,
            favorite_items: vec![
                "crafting_table".to_string(),
                "crafter".to_string(),
                "mace".to_string(),
                "shulker_box".to_string(),
            ],
            favorite_gear: vec!["netherite_pickaxe_silk".to_string()],
            favorite_farms: Vec::new(),
            waypoints: Vec::new(),
            checked_materials: HashSet::new(),
            saved_seeds: Vec::new(),
            seed_history: Vec::new(),
            shopping_list: Vec::new(),
            shopping_shulker_mode: true,
        }
    }
}

impl UserPrefs {
    fn prefs_path_for(app_dir: &str) -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                if !appdata.is_empty() {
                    return Some(PathBuf::from(appdata).join(app_dir).join("prefs.json"));
                }
            }
            if let Ok(profile) = std::env::var("USERPROFILE") {
                if !profile.is_empty() {
                    return Some(
                        PathBuf::from(profile)
                            .join("AppData/Roaming")
                            .join(app_dir)
                            .join("prefs.json"),
                    );
                }
            }
        }
        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                if !home.is_empty() {
                    return Some(
                        PathBuf::from(home)
                            .join("Library/Application Support")
                            .join(app_dir)
                            .join("prefs.json"),
                    );
                }
            }
        }
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            if !xdg.trim().is_empty() {
                return Some(PathBuf::from(xdg).join(app_dir).join("prefs.json"));
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            if !home.is_empty() {
                return Some(
                    PathBuf::from(home)
                        .join(".config")
                        .join(app_dir)
                        .join("prefs.json"),
                );
            }
        }
        None
    }

    pub fn prefs_path() -> Option<PathBuf> {
        Self::prefs_path_for("craft-atlas")
    }

    /// Root application configuration/cache directory (`%APPDATA%\craft-atlas` or XDG equivalent).
    pub fn app_data_dir() -> Option<PathBuf> {
        Self::prefs_path().and_then(|p| p.parent().map(|d| d.to_path_buf()))
    }

    /// Legacy v1.0.x config path kept for seamless upgrade migration.
    pub fn legacy_prefs_path() -> Option<PathBuf> {
        Self::prefs_path_for("minecraft-companion")
    }

    pub fn load() -> Self {
        const MAX_PREFS_BYTES: u64 = 2 * 1024 * 1024;
        for (path, is_legacy) in [
            Self::prefs_path().map(|p| (p, false)),
            Self::legacy_prefs_path().map(|p| (p, true)),
        ]
        .into_iter()
        .flatten()
        {
            if path.metadata().map(|m| m.len()).unwrap_or(u64::MAX) <= MAX_PREFS_BYTES {
                if let Ok(text) = fs::read_to_string(&path) {
                    if let Ok(parsed) = serde_json::from_str::<UserPrefs>(&text) {
                        if is_legacy {
                            parsed.save();
                        }
                        return parsed;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::prefs_path() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let tmp_path = path.with_extension("json.tmp");
                if fs::write(&tmp_path, json).is_ok() {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600));
                    }
                    if fs::rename(&tmp_path, &path).is_err() {
                        let _ = fs::remove_file(&tmp_path);
                    }
                }
            }
        }
    }

    pub fn toggle_favorite_item(&mut self, id: &str) -> bool {
        if let Some(pos) = self.favorite_items.iter().position(|x| x == id) {
            self.favorite_items.remove(pos);
            self.save();
            false
        } else {
            self.favorite_items.insert(0, id.to_string());
            self.save();
            true
        }
    }

    pub fn toggle_favorite_gear(&mut self, id: &str) -> bool {
        if let Some(pos) = self.favorite_gear.iter().position(|x| x == id) {
            self.favorite_gear.remove(pos);
            self.save();
            false
        } else {
            self.favorite_gear.insert(0, id.to_string());
            self.save();
            true
        }
    }

    pub fn toggle_favorite_farm(&mut self, id: &str) -> bool {
        if let Some(pos) = self.favorite_farms.iter().position(|x| x == id) {
            self.favorite_farms.remove(pos);
            self.save();
            false
        } else {
            self.favorite_farms.insert(0, id.to_string());
            self.save();
            true
        }
    }

    /// Bookmarks (or updates) a seed bookmark without persisting disk writes when called in pure unit tests.
    pub fn upsert_saved_seed_in_memory(
        &mut self,
        name: &str,
        seed_text: &str,
        seed: i64,
        large_biomes: bool,
        is_server: bool,
        now_ms: i64,
    ) -> SavedSeedEntry {
        let clean_text = if seed_text.trim().is_empty() {
            seed.to_string()
        } else {
            seed_text.trim().to_string()
        };
        let clean_name = if name.trim().is_empty() {
            if is_server {
                format!("Server ({})", seed)
            } else {
                format!("Seed ({})", seed)
            }
        } else {
            name.trim().to_string()
        };

        if let Some(pos) = self
            .saved_seeds
            .iter()
            .position(|s| s.seed == seed && s.large_biomes == large_biomes)
        {
            let mut updated = self.saved_seeds.remove(pos);
            updated.name = clean_name;
            updated.seed_text = clean_text;
            updated.is_server = is_server;
            updated.saved_at_ms = now_ms;
            self.saved_seeds.insert(0, updated.clone());
            return updated;
        }

        let next_id = self.saved_seeds.iter().map(|s| s.id).max().unwrap_or(0) + 1;
        let entry = SavedSeedEntry {
            id: next_id,
            name: clean_name,
            seed_text: clean_text,
            seed,
            large_biomes,
            is_server,
            saved_at_ms: now_ms,
        };
        self.saved_seeds.insert(0, entry.clone());
        entry
    }

    pub fn upsert_saved_seed(
        &mut self,
        name: &str,
        seed_text: &str,
        seed: i64,
        large_biomes: bool,
        is_server: bool,
        now_ms: i64,
    ) -> SavedSeedEntry {
        let entry = self.upsert_saved_seed_in_memory(
            name,
            seed_text,
            seed,
            large_biomes,
            is_server,
            now_ms,
        );
        self.save();
        entry
    }

    pub fn remove_saved_seed(&mut self, id: u64) {
        self.saved_seeds.retain(|s| s.id != id);
        self.save();
    }

    /// Records a seed visit in memory, deduplicating `(seed, large_biomes)` and capping at `MAX_SEED_HISTORY`.
    pub fn record_seed_history_in_memory(
        &mut self,
        seed_text: &str,
        seed: i64,
        large_biomes: bool,
        source_label: &str,
        now_ms: i64,
    ) {
        let clean_text = if seed_text.trim().is_empty() {
            seed.to_string()
        } else {
            seed_text.trim().to_string()
        };
        self.seed_history
            .retain(|e| !(e.seed == seed && e.large_biomes == large_biomes));
        self.seed_history.insert(
            0,
            SeedHistoryEntry {
                seed_text: clean_text,
                seed,
                large_biomes,
                source_label: source_label.to_string(),
                used_at_ms: now_ms,
            },
        );
        self.seed_history.truncate(MAX_SEED_HISTORY);
    }

    pub fn record_seed_history(
        &mut self,
        seed_text: &str,
        seed: i64,
        large_biomes: bool,
        source_label: &str,
        now_ms: i64,
    ) {
        self.record_seed_history_in_memory(seed_text, seed, large_biomes, source_label, now_ms);
        self.save();
    }

    pub fn clear_seed_history(&mut self) {
        self.seed_history.clear();
        self.save();
    }

    /// Merges a batch of `(item_id, label, count)` into the unified shopping list in memory.
    pub fn add_shopping_items_in_memory(&mut self, items: &[(String, String, u64)], source: &str) {
        for (item_id, label, count) in items {
            if *count == 0 || item_id.trim().is_empty() {
                continue;
            }
            if let Some(existing) = self.shopping_list.iter_mut().find(|e| e.item_id == *item_id) {
                existing.count = existing.count.saturating_add(*count);
                existing.done = false;
                if existing.label.is_empty() && !label.is_empty() {
                    existing.label = label.clone();
                }
                if !source.is_empty() {
                    existing.source = source.to_string();
                }
            } else {
                self.shopping_list.push(ShoppingListItem {
                    item_id: item_id.clone(),
                    label: label.clone(),
                    count: *count,
                    done: false,
                    source: source.to_string(),
                });
            }
        }
        self.shopping_list
            .sort_by(|a, b| a.done.cmp(&b.done).then_with(|| b.count.cmp(&a.count)));
    }

    pub fn add_shopping_items(&mut self, items: &[(String, String, u64)], source: &str) {
        self.add_shopping_items_in_memory(items, source);
        self.save();
    }

    pub fn toggle_shopping_done(&mut self, item_id: &str) {
        if let Some(entry) = self.shopping_list.iter_mut().find(|e| e.item_id == item_id) {
            entry.done = !entry.done;
        }
        self.shopping_list
            .sort_by(|a, b| a.done.cmp(&b.done).then_with(|| b.count.cmp(&a.count)));
        self.save();
    }

    pub fn remove_shopping_item(&mut self, item_id: &str) {
        self.shopping_list.retain(|e| e.item_id != item_id);
        self.save();
    }

    pub fn clear_done_shopping_items(&mut self) {
        self.shopping_list.retain(|e| !e.done);
        self.save();
    }

    pub fn clear_shopping_list(&mut self) {
        self.shopping_list.clear();
        self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_prefs_roundtrip_serialization() {
        let mut p = UserPrefs {
            language: Language::EnUs,
            theme: ThemeId::NetherCrimsonEnd,
            audio_enabled: false,
            ..Default::default()
        };
        p.waypoints.push(Waypoint {
            id: 42,
            seed: 123456,
            dim: 0,
            x: 100,
            z: -250,
            name: "Main Base".to_string(),
            icon: "beacon".to_string(),
        });
        p.upsert_saved_seed_in_memory("SMP Amigos", "-12345", -12345, false, true, 1_700_000_000_000);
        p.record_seed_history_in_memory("Notch", 75544711, false, "Manual", 1_700_000_001_000);

        let json = serde_json::to_string(&p).expect("serialize prefs");
        let restored: UserPrefs = serde_json::from_str(&json).expect("deserialize prefs");
        assert_eq!(restored.language, Language::EnUs);
        assert_eq!(restored.theme, ThemeId::NetherCrimsonEnd);
        assert!(!restored.audio_enabled);
        assert_eq!(restored.waypoints.len(), 1);
        assert_eq!(restored.waypoints[0].name, "Main Base");
        assert_eq!(restored.saved_seeds.len(), 1);
        assert_eq!(restored.saved_seeds[0].name, "SMP Amigos");
        assert!(restored.saved_seeds[0].is_server);
        assert_eq!(restored.seed_history.len(), 1);
        assert_eq!(restored.seed_history[0].seed_text, "Notch");
    }

    #[test]
    fn seed_bookmarks_and_history_dedup_cap_and_legacy_deser() {
        let legacy_json = r#"{"language":"EsMx","selected_version":"26.3","last_seed":"123"}"#;
        let mut p: UserPrefs = serde_json::from_str(legacy_json).expect("legacy json loads");
        assert!(p.saved_seeds.is_empty());
        assert!(p.seed_history.is_empty());

        let s1 = p.upsert_saved_seed_in_memory("SMP v1", "999", 999, false, true, 1000);
        let s2 = p.upsert_saved_seed_in_memory("SMP v2", "999", 999, false, true, 2000);
        assert_eq!(s1.id, s2.id);
        assert_eq!(p.saved_seeds.len(), 1);
        assert_eq!(p.saved_seeds[0].name, "SMP v2");

        for i in 0..25_i64 {
            p.record_seed_history_in_memory(&i.to_string(), i, false, "Manual", i * 1000);
        }
        assert_eq!(p.seed_history.len(), MAX_SEED_HISTORY);
        assert_eq!(p.seed_history[0].seed, 24);
        // Re-visiting an older seed moves it back to index 0 without duplicating
        p.record_seed_history_in_memory("10", 10, false, "History", 99_000);
        assert_eq!(p.seed_history.len(), MAX_SEED_HISTORY);
        assert_eq!(p.seed_history[0].seed, 10);

        // Unified shopping list merge & legacy deserialization check
        assert!(p.shopping_list.is_empty());
        assert!(p.shopping_shulker_mode);
        p.add_shopping_items_in_memory(
            &[
                ("hopper".into(), "Hopper".into(), 64),
                ("glass".into(), "Glass".into(), 128),
            ],
            "Iron Farm",
        );
        p.add_shopping_items_in_memory(&[("hopper".into(), "Hopper".into(), 32)], "Litematic");
        assert_eq!(p.shopping_list.len(), 2);
        let hopper = p.shopping_list.iter().find(|i| i.item_id == "hopper").unwrap();
        assert_eq!(hopper.count, 96);
    }
}
