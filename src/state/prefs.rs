use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::data::models::Language;
use crate::data::seed_engine::Dimension;

/// Switchable Minecraft visual themes backed by scenic wallpapers / 360° panoramas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ThemeId {
    #[default]
    DeepslateOre,
    ClassicJavaStone,
    NetherCrimsonEnd,
    EndVoidAurora,
    CherryBlossom,
    StarryOverworld,
}

impl ThemeId {
    pub fn all() -> [ThemeId; 6] {
        [
            ThemeId::DeepslateOre,
            ThemeId::ClassicJavaStone,
            ThemeId::NetherCrimsonEnd,
            ThemeId::EndVoidAurora,
            ThemeId::CherryBlossom,
            ThemeId::StarryOverworld,
        ]
    }

    pub fn label(self, lang: Language) -> &'static str {
        match (self, lang) {
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

/// Persistent user configuration serialized safely under `%APPDATA%\minecraft-companion\prefs.json`.
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
        }
    }
}

impl UserPrefs {
    pub fn prefs_path() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                if !appdata.is_empty() {
                    return Some(
                        PathBuf::from(appdata)
                            .join("minecraft-companion")
                            .join("prefs.json"),
                    );
                }
            }
            if let Ok(profile) = std::env::var("USERPROFILE") {
                if !profile.is_empty() {
                    return Some(
                        PathBuf::from(profile)
                            .join("AppData/Roaming")
                            .join("minecraft-companion")
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
                            .join("minecraft-companion")
                            .join("prefs.json"),
                    );
                }
            }
        }
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            if !xdg.trim().is_empty() {
                return Some(
                    PathBuf::from(xdg)
                        .join("minecraft-companion")
                        .join("prefs.json"),
                );
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            if !home.is_empty() {
                return Some(
                    PathBuf::from(home)
                        .join(".config")
                        .join("minecraft-companion")
                        .join("prefs.json"),
                );
            }
        }
        None
    }

    pub fn load() -> Self {
        const MAX_PREFS_BYTES: u64 = 2 * 1024 * 1024;
        if let Some(path) = Self::prefs_path() {
            if path.metadata().map(|m| m.len()).unwrap_or(u64::MAX) <= MAX_PREFS_BYTES {
                if let Ok(text) = fs::read_to_string(&path) {
                    if let Ok(parsed) = serde_json::from_str::<UserPrefs>(&text) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_prefs_roundtrip_serialization() {
        let mut p = UserPrefs::default();
        p.language = Language::EnUs;
        p.theme = ThemeId::NetherCrimsonEnd;
        p.audio_enabled = false;
        p.waypoints.push(Waypoint {
            id: 42,
            seed: 123456,
            dim: 0,
            x: 100,
            z: -250,
            name: "Main Base".to_string(),
            icon: "beacon".to_string(),
        });

        let json = serde_json::to_string(&p).expect("serialize prefs");
        let restored: UserPrefs = serde_json::from_str(&json).expect("deserialize prefs");
        assert_eq!(restored.language, Language::EnUs);
        assert_eq!(restored.theme, ThemeId::NetherCrimsonEnd);
        assert!(!restored.audio_enabled);
        assert_eq!(restored.waypoints.len(), 1);
        assert_eq!(restored.waypoints[0].name, "Main Base");
    }
}
