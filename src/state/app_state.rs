use std::path::PathBuf;

use crate::data::recipes::{CraftingStation, RecipeCategory};
use crate::data::seed_engine::{Dimension, LayerToggles, MapStyle};
use crate::data::worlds_nbt::{
    scan_local_schematics, scan_local_worlds, DetectedSchematic, DetectedWorld,
};

use super::prefs::{UserPrefs, Waypoint};

/// Primary top-level modules of Minecraft Companion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppTab {
    Gear,
    Recipes,
    SeedMap,
    Farms,
    Dupes,
    ProTips,
}

impl AppTab {
    pub fn all() -> [AppTab; 6] {
        [
            AppTab::Gear,
            AppTab::Recipes,
            AppTab::SeedMap,
            AppTab::Farms,
            AppTab::Dupes,
            AppTab::ProTips,
        ]
    }

    pub fn icon_id(self) -> &'static str {
        match self {
            AppTab::Gear => "netherite_sword",
            AppTab::Recipes => "crafting_table",
            AppTab::SeedMap => "filled_map",
            AppTab::Farms => "wheat",
            AppTab::Dupes => "tnt",
            AppTab::ProTips => "knowledge_book",
        }
    }
}

/// Interactive "Yunque Vivo" (Living Anvil) forging animation & active step progression.
#[derive(Debug, Clone, Default)]
pub struct AnvilAnimState {
    pub gear_id: String,
    pub completed_steps: usize,
    pub hammer_timer: f32,
    pub flash_timer: f32,
}

impl AnvilAnimState {
    pub fn sync_gear(&mut self, gear_id: &str) {
        if self.gear_id != gear_id {
            self.gear_id = gear_id.to_string();
            self.completed_steps = 0;
            self.hammer_timer = 0.0;
            self.flash_timer = 0.0;
        }
    }

    pub fn trigger_strike(&mut self) {
        self.hammer_timer = 0.36;
        self.flash_timer = 0.42;
    }

    pub fn tick(&mut self, dt: f32) {
        if self.hammer_timer > 0.0 {
            self.hammer_timer = (self.hammer_timer - dt).max(0.0);
        }
        if self.flash_timer > 0.0 {
            self.flash_timer = (self.flash_timer - dt).max(0.0);
        }
    }
}

/// What JEI/EMI pane is currently shown in the Recipes tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeViewMode {
    RecipesFor,
    UsesFor,
}

#[derive(Debug, Clone)]
pub struct RecipeHistoryEntry {
    pub item_id: String,
    pub mode: RecipeViewMode,
    pub variant_idx: usize,
}

/// Complete mutable UI & interactive session state.
pub struct AppState {
    pub prefs: UserPrefs,
    pub active_tab: AppTab,
    pub tab_anim_t: f32,
    pub search_query: String,
    pub search_palette_open: bool,
    pub palette_query: String,
    pub palette_selected_idx: usize,
    pub focus_search_next_frame: bool,
    pub hovered_tooltip_id: Option<String>,

    // Gear Tab
    pub gear_filter: String,
    pub selected_gear_id: String,
    pub anvil_anim: AnvilAnimState,

    // Recipes JEI + EMI Tab
    pub recipe_station_filter: Option<CraftingStation>,
    pub recipe_category_filter: RecipeCategory,
    pub recipe_catalog_page: usize,
    pub selected_recipe_item: String,
    pub recipe_view_mode: RecipeViewMode,
    pub recipe_variant_idx: usize,
    pub recipe_history: Vec<RecipeHistoryEntry>,
    pub recipe_history_pos: usize,
    pub show_craft_tree: bool,
    pub craft_tree_qty: u32,

    // Seed Map Explorer Tab
    pub seed_input: String,
    pub map_dim: Dimension,
    pub map_style: MapStyle,
    pub map_center_x: f64,
    pub map_center_z: f64,
    pub map_blocks_per_px: f32,
    pub map_target_center_x: f64,
    pub map_target_center_z: f64,
    pub map_target_bpp: f32,
    pub map_layers: LayerToggles,
    pub nether_sync_enabled: bool,
    pub goto_x_input: String,
    pub goto_z_input: String,
    pub ruler_active: bool,
    pub ruler_p1: Option<(i32, i32)>,
    pub ruler_p2: Option<(i32, i32)>,
    pub new_waypoint_name: String,
    pub new_waypoint_icon: String,
    pub local_worlds: Vec<DetectedWorld>,
    pub local_schematics: Vec<DetectedSchematic>,
    pub last_cursor_world: Option<(i32, i32)>,

    // Farms / Dupes / Pro Tips Tabs
    pub farm_category_filter: String,
    pub farm_tier_filter: String,
    pub selected_farm_id: String,
    pub farm_stacks_mode: bool,

    pub dupe_category_filter: String,
    pub dupe_status_filter: String,
    pub selected_dupe_id: String,
    pub portal_calc_ow_x: String,
    pub portal_calc_ow_z: String,

    pub tip_category_filter: String,
    pub selected_tip_id: String,

    // Interactive map temp Google-Maps-style pin & right-click waypoint menu
    pub temp_map_pin: Option<(Dimension, i32, i32)>,
    pub map_context_menu: Option<MapContextMenuState>,

    // Creators showcase inside Farms & Dupes/Glitches
    pub farms_creators_view: bool,
    pub dupes_creators_view: bool,
    pub creators_specialty_filter: String,
    pub creators_region_filter: String,

    // Export cache metadata & release update checker
    pub last_export_path: Option<PathBuf>,
    pub updater: crate::updater::UpdateChecker,
}

/// Active right-click floating menu to place a custom waypoint at exact clicked world coordinates.
#[derive(Debug, Clone)]
pub struct MapContextMenuState {
    pub dim: Dimension,
    pub x: i32,
    pub z: i32,
    pub screen_pos: eframe::egui::Pos2,
    pub name: String,
    pub icon: String,
    pub focus_name_next: bool,
}

/// Curated 12 icons for custom map waypoints (home, base, portals, storage, farms, combat).
pub const WAYPOINT_ICON_PRESETS: [(&str, &str, &str); 12] = [
    ("red_bed", "Casa / Cama", "Home / Bed"),
    ("beacon", "Base Principal", "Main Base"),
    ("obsidian", "Portal Nether", "Nether Portal"),
    ("chest", "Almacén / Cofres", "Storage"),
    ("wheat", "Granja", "Farm"),
    ("netherite_pickaxe", "Mina / Excavación", "Mine"),
    ("bell", "Aldea / Trading", "Village"),
    ("ender_eye", "Stronghold / End", "Stronghold / End"),
    ("trial_key", "Trial / Dungeon", "Trial / Dungeon"),
    ("diamond_sword", "Spawner / Combate", "Combat / Spawner"),
    ("compass", "Punto de Referencia", "Landmark"),
    ("totem_of_undying", "Raid / Seguridad", "Outpost / Safe"),
];

impl AppState {
    pub fn new(
        prefs: UserPrefs,
        initial_gear_id: String,
        initial_farm_id: String,
        initial_dupe_id: String,
        initial_tip_id: String,
    ) -> Self {
        let seed_str = prefs.last_seed.clone();
        Self {
            prefs,
            active_tab: AppTab::Gear,
            tab_anim_t: 1.0,
            search_query: String::new(),
            search_palette_open: false,
            palette_query: String::new(),
            palette_selected_idx: 0,
            focus_search_next_frame: false,
            hovered_tooltip_id: None,

            gear_filter: "all".to_string(),
            selected_gear_id: initial_gear_id.clone(),
            anvil_anim: AnvilAnimState {
                gear_id: initial_gear_id,
                ..Default::default()
            },

            recipe_station_filter: None,
            recipe_category_filter: RecipeCategory::All,
            recipe_catalog_page: 0,
            selected_recipe_item: "crafter".to_string(),
            recipe_view_mode: RecipeViewMode::RecipesFor,
            recipe_variant_idx: 0,
            recipe_history: vec![RecipeHistoryEntry {
                item_id: "crafter".to_string(),
                mode: RecipeViewMode::RecipesFor,
                variant_idx: 0,
            }],
            recipe_history_pos: 0,
            show_craft_tree: false,
            craft_tree_qty: 1,

            seed_input: seed_str,
            map_dim: Dimension::Overworld,
            map_style: MapStyle::Topo3D,
            map_center_x: 0.0,
            map_center_z: 0.0,
            map_blocks_per_px: 6.0,
            map_target_center_x: 0.0,
            map_target_center_z: 0.0,
            map_target_bpp: 6.0,
            map_layers: LayerToggles::default(),
            nether_sync_enabled: true,
            goto_x_input: "0".to_string(),
            goto_z_input: "0".to_string(),
            ruler_active: false,
            ruler_p1: None,
            ruler_p2: None,
            new_waypoint_name: String::new(),
            new_waypoint_icon: "beacon".to_string(),
            local_worlds: scan_local_worlds(),
            local_schematics: scan_local_schematics(),
            last_cursor_world: None,

            farm_category_filter: "all".to_string(),
            farm_tier_filter: "all".to_string(),
            selected_farm_id: initial_farm_id,
            farm_stacks_mode: true,

            dupe_category_filter: "all".to_string(),
            dupe_status_filter: "all".to_string(),
            selected_dupe_id: initial_dupe_id,
            portal_calc_ow_x: "800".to_string(),
            portal_calc_ow_z: "-800".to_string(),

            tip_category_filter: "all".to_string(),
            selected_tip_id: initial_tip_id,

            temp_map_pin: None,
            map_context_menu: None,
            farms_creators_view: false,
            dupes_creators_view: false,
            creators_specialty_filter: "all".to_string(),
            creators_region_filter: "all".to_string(),

            last_export_path: None,
            updater: crate::updater::UpdateChecker::new(),
        }
    }

    pub fn switch_tab(&mut self, tab: AppTab) {
        if self.active_tab != tab {
            self.active_tab = tab;
            self.tab_anim_t = 0.0;
        }
    }

    pub fn open_recipe_item(&mut self, item_id: &str, mode: RecipeViewMode) {
        let entry = RecipeHistoryEntry {
            item_id: item_id.to_string(),
            mode,
            variant_idx: 0,
        };
        if self.recipe_history_pos + 1 < self.recipe_history.len() {
            self.recipe_history.truncate(self.recipe_history_pos + 1);
        }
        let same_as_cur = self
            .recipe_history
            .get(self.recipe_history_pos)
            .map(|h| h.item_id == entry.item_id && h.mode == entry.mode)
            .unwrap_or(false);
        if !same_as_cur {
            self.recipe_history.push(entry);
            if self.recipe_history.len() > 60 {
                self.recipe_history.remove(0);
            }
            self.recipe_history_pos = self.recipe_history.len() - 1;
        }
        self.selected_recipe_item = item_id.to_string();
        self.recipe_view_mode = mode;
        self.recipe_variant_idx = 0;
        self.active_tab = AppTab::Recipes;
    }

    pub fn recipe_nav_back(&mut self) -> bool {
        if self.recipe_history_pos > 0 {
            self.recipe_history_pos -= 1;
            let e = self.recipe_history[self.recipe_history_pos].clone();
            self.selected_recipe_item = e.item_id;
            self.recipe_view_mode = e.mode;
            self.recipe_variant_idx = e.variant_idx;
            true
        } else {
            false
        }
    }

    pub fn recipe_nav_forward(&mut self) -> bool {
        if self.recipe_history_pos + 1 < self.recipe_history.len() {
            self.recipe_history_pos += 1;
            let e = self.recipe_history[self.recipe_history_pos].clone();
            self.selected_recipe_item = e.item_id;
            self.recipe_view_mode = e.mode;
            self.recipe_variant_idx = e.variant_idx;
            true
        } else {
            false
        }
    }

    pub fn switch_map_dimension(&mut self, new_dim: Dimension) {
        if self.map_dim == new_dim {
            return;
        }
        if self.nether_sync_enabled {
            match (self.map_dim, new_dim) {
                (Dimension::Overworld, Dimension::Nether) => {
                    self.map_target_center_x /= 8.0;
                    self.map_target_center_z /= 8.0;
                    self.map_center_x = self.map_target_center_x;
                    self.map_center_z = self.map_target_center_z;
                }
                (Dimension::Nether, Dimension::Overworld) => {
                    self.map_target_center_x *= 8.0;
                    self.map_target_center_z *= 8.0;
                    self.map_center_x = self.map_target_center_x;
                    self.map_center_z = self.map_target_center_z;
                }
                (_, Dimension::TheEnd) => {
                    self.map_target_center_x = 0.0;
                    self.map_target_center_z = 0.0;
                    self.map_center_x = 0.0;
                    self.map_center_z = 0.0;
                }
                _ => {}
            }
        }
        self.map_dim = new_dim;
    }

    pub fn jump_map_to(&mut self, x: f64, z: f64) {
        self.map_target_center_x = x;
        self.map_target_center_z = z;
    }

    pub fn add_waypoint(&mut self, seed: i64, x: i32, z: i32, default_name: &str) {
        let name = if self.new_waypoint_name.trim().is_empty() {
            default_name.to_string()
        } else {
            self.new_waypoint_name.trim().to_string()
        };
        let icon = self.new_waypoint_icon.clone();
        self.add_waypoint_at(seed, self.map_dim, x, z, &name, &icon);
        self.new_waypoint_name.clear();
    }

    pub fn add_waypoint_at(
        &mut self,
        seed: i64,
        dim: Dimension,
        x: i32,
        z: i32,
        raw_name: &str,
        icon: &str,
    ) {
        let name = if raw_name.trim().is_empty() {
            format!("({}, {})", x, z)
        } else {
            raw_name.trim().to_string()
        };
        let next_id = self.prefs.waypoints.iter().map(|w| w.id).max().unwrap_or(0) + 1;
        self.prefs.waypoints.push(Waypoint {
            id: next_id,
            seed,
            dim: dim as i32,
            x,
            z,
            name,
            icon: icon.to_string(),
        });
        self.prefs.save();
    }

    pub fn remove_waypoint(&mut self, id: u64) {
        self.prefs.waypoints.retain(|w| w.id != id);
        self.prefs.save();
    }
}
