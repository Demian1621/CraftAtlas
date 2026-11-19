use serde::Deserialize;
use std::collections::{BTreeMap, HashMap, HashSet};

use super::models::{Language, VersionRegistry};

/// Crafting station types extracted from Minecraft 26.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CraftingStation {
    Crafting,
    Smelting,
    Blasting,
    Smoking,
    Campfire,
    Stonecutting,
    Smithing,
    Brewing,
}

impl CraftingStation {
    pub fn from_str(s: &str) -> Self {
        match s {
            "smelting" => CraftingStation::Smelting,
            "blasting" => CraftingStation::Blasting,
            "smoking" => CraftingStation::Smoking,
            "campfire" => CraftingStation::Campfire,
            "stonecutting" => CraftingStation::Stonecutting,
            "smithing" => CraftingStation::Smithing,
            "brewing" => CraftingStation::Brewing,
            _ => CraftingStation::Crafting,
        }
    }

    pub fn icon_id(self) -> &'static str {
        match self {
            CraftingStation::Crafting => "crafting_table",
            CraftingStation::Smelting => "furnace",
            CraftingStation::Blasting => "blast_furnace",
            CraftingStation::Smoking => "smoker",
            CraftingStation::Campfire => "campfire",
            CraftingStation::Stonecutting => "stonecutter",
            CraftingStation::Smithing => "smithing_table",
            CraftingStation::Brewing => "brewing_stand",
        }
    }

    pub fn label(self, lang: Language) -> &'static str {
        match (self, lang) {
            (CraftingStation::Crafting, Language::EnUs) => "Crafting Table",
            (CraftingStation::Crafting, _) => "Mesa de crafteo",
            (CraftingStation::Smelting, Language::EnUs) => "Furnace",
            (CraftingStation::Smelting, _) => "Horno",
            (CraftingStation::Blasting, Language::EnUs) => "Blast Furnace",
            (CraftingStation::Blasting, Language::EsMx) => "Alto horno",
            (CraftingStation::Blasting, Language::EsEs) => "Alto horno",
            (CraftingStation::Smoking, Language::EnUs) => "Smoker",
            (CraftingStation::Smoking, _) => "Ahumador",
            (CraftingStation::Campfire, Language::EnUs) => "Campfire",
            (CraftingStation::Campfire, Language::EsMx) => "Fogata",
            (CraftingStation::Campfire, Language::EsEs) => "Hoguera",
            (CraftingStation::Stonecutting, Language::EnUs) => "Stonecutter",
            (CraftingStation::Stonecutting, _) => "Cortapiedras",
            (CraftingStation::Smithing, Language::EnUs) => "Smithing Table",
            (CraftingStation::Smithing, _) => "Mesa de herrería",
            (CraftingStation::Brewing, Language::EnUs) => "Brewing Stand",
            (CraftingStation::Brewing, Language::EsMx) => "Soporte para pociones",
            (CraftingStation::Brewing, Language::EsEs) => "Soporte para pociones",
        }
    }

    pub fn all() -> [CraftingStation; 8] {
        [
            CraftingStation::Crafting,
            CraftingStation::Smelting,
            CraftingStation::Blasting,
            CraftingStation::Smoking,
            CraftingStation::Campfire,
            CraftingStation::Stonecutting,
            CraftingStation::Smithing,
            CraftingStation::Brewing,
        ]
    }
}

/// Logical JEI/Crafty.gg item categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecipeCategory {
    All,
    Building,
    Decoration,
    Redstone,
    Transportation,
    Food,
    Tools,
    Combat,
    Armor,
    Brewing,
    Materials,
    Misc,
}

impl RecipeCategory {
    pub fn from_str(s: &str) -> Self {
        match s {
            "building" => RecipeCategory::Building,
            "decoration" => RecipeCategory::Decoration,
            "redstone" => RecipeCategory::Redstone,
            "transportation" => RecipeCategory::Transportation,
            "food" => RecipeCategory::Food,
            "tools" => RecipeCategory::Tools,
            "combat" => RecipeCategory::Combat,
            "armor" => RecipeCategory::Armor,
            "brewing" => RecipeCategory::Brewing,
            "materials" => RecipeCategory::Materials,
            _ => RecipeCategory::Misc,
        }
    }

    pub fn icon_id(self) -> &'static str {
        match self {
            RecipeCategory::All => "knowledge_book",
            RecipeCategory::Building => "bricks",
            RecipeCategory::Decoration => "lantern",
            RecipeCategory::Redstone => "redstone",
            RecipeCategory::Transportation => "powered_rail",
            RecipeCategory::Food => "golden_apple",
            RecipeCategory::Tools => "diamond_pickaxe",
            RecipeCategory::Combat => "diamond_sword",
            RecipeCategory::Armor => "diamond_chestplate",
            RecipeCategory::Brewing => "brewing_stand",
            RecipeCategory::Materials => "diamond",
            RecipeCategory::Misc => "compass",
        }
    }

    pub fn label(self, lang: Language) -> &'static str {
        match (self, lang) {
            (RecipeCategory::All, Language::EnUs) => "All",
            (RecipeCategory::All, _) => "Todo",
            (RecipeCategory::Building, Language::EnUs) => "Building",
            (RecipeCategory::Building, _) => "Construcción",
            (RecipeCategory::Decoration, Language::EnUs) => "Decoration",
            (RecipeCategory::Decoration, _) => "Decoración",
            (RecipeCategory::Redstone, _) => "Redstone",
            (RecipeCategory::Transportation, Language::EnUs) => "Transport",
            (RecipeCategory::Transportation, _) => "Transporte",
            (RecipeCategory::Food, Language::EnUs) => "Food",
            (RecipeCategory::Food, _) => "Comida",
            (RecipeCategory::Tools, Language::EnUs) => "Tools",
            (RecipeCategory::Tools, _) => "Herramientas",
            (RecipeCategory::Combat, Language::EnUs) => "Combat",
            (RecipeCategory::Combat, _) => "Combate",
            (RecipeCategory::Armor, Language::EnUs) => "Armor",
            (RecipeCategory::Armor, _) => "Armadura",
            (RecipeCategory::Brewing, Language::EnUs) => "Brewing",
            (RecipeCategory::Brewing, _) => "Pociones",
            (RecipeCategory::Materials, Language::EnUs) => "Materials",
            (RecipeCategory::Materials, _) => "Materiales",
            (RecipeCategory::Misc, Language::EnUs) => "Misc",
            (RecipeCategory::Misc, _) => "Varios",
        }
    }

    pub fn all() -> [RecipeCategory; 12] {
        [
            RecipeCategory::All,
            RecipeCategory::Building,
            RecipeCategory::Decoration,
            RecipeCategory::Redstone,
            RecipeCategory::Transportation,
            RecipeCategory::Food,
            RecipeCategory::Tools,
            RecipeCategory::Combat,
            RecipeCategory::Armor,
            RecipeCategory::Brewing,
            RecipeCategory::Materials,
            RecipeCategory::Misc,
        ]
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RawRecipesFile {
    pub version: String,
    pub tags: HashMap<String, Vec<String>>,
    pub recipes: Vec<RawRecipeEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawRecipeEntry {
    pub id: String,
    pub station: String,
    pub category: String,
    pub result_id: String,
    pub result_count: u32,
    pub shapeless: bool,
    pub is_2x2: bool,
    #[serde(default)]
    pub group: String,
    pub slots: Vec<String>,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub since_version: Option<String>,
    #[serde(default)]
    pub until_version: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RecipeEntry {
    pub index: usize,
    pub id: String,
    pub station: CraftingStation,
    pub category: RecipeCategory,
    pub result_id: String,
    pub result_count: u32,
    pub shapeless: bool,
    pub is_2x2: bool,
    pub group: String,
    pub slots: Vec<String>,
    pub note: String,
    pub since_version: Option<String>,
    pub until_version: Option<String>,
}

impl RecipeEntry {
    pub fn is_active_in(&self, versions: &VersionRegistry, active_ver: &str) -> bool {
        if !versions.is_active(
            self.since_version.as_deref(),
            self.until_version.as_deref(),
            active_ver,
        ) {
            return false;
        }
        if !super::version_rules::is_item_active(versions, &self.result_id, active_ver) {
            return false;
        }
        self.slots.iter().all(|slot| {
            slot.is_empty()
                || slot.starts_with('#')
                || super::version_rules::is_item_active(versions, slot, active_ver)
        })
    }
}

/// Aggregated EMI-style node in a recursive crafting tree.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CraftTreeNode {
    pub item_id: String,
    pub total_needed: u32,
    pub via_recipe: Option<usize>,
    pub batches: u32,
    pub children: Vec<CraftTreeNode>,
}

/// Complete JEI + EMI recipe index with O(1) result and usage lookup.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RecipeDatabase {
    pub version: String,
    pub tags: HashMap<String, Vec<String>>,
    pub recipes: Vec<RecipeEntry>,
    pub lang_items: HashMap<String, [String; 3]>,
    pub catalog_items: Vec<String>,
    pub by_result: HashMap<String, Vec<usize>>,
    pub by_ingredient: HashMap<String, Vec<usize>>,
}

impl RecipeDatabase {
    pub fn from_json_bytes(recipes_json: &[u8], lang_json: &[u8]) -> Result<Self, String> {
        let raw: RawRecipesFile = serde_json::from_slice(recipes_json)
            .map_err(|e| format!("Failed parsing recipes.json: {}", e))?;
        let lang_items: HashMap<String, [String; 3]> = serde_json::from_slice(lang_json)
            .map_err(|e| format!("Failed parsing lang_items.json: {}", e))?;

        let mut recipes = Vec::with_capacity(raw.recipes.len());
        let mut by_result: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_ingredient: HashMap<String, Vec<usize>> = HashMap::new();
        let mut catalog_seen = HashSet::new();
        let mut catalog_items = Vec::new();

        for (idx, r) in raw.recipes.into_iter().enumerate() {
            let entry = RecipeEntry {
                index: idx,
                id: r.id,
                station: CraftingStation::from_str(&r.station),
                category: RecipeCategory::from_str(&r.category),
                result_id: r.result_id.clone(),
                result_count: r.result_count.max(1),
                shapeless: r.shapeless,
                is_2x2: r.is_2x2,
                group: r.group,
                slots: r.slots,
                note: r.note,
                since_version: r.since_version,
                until_version: r.until_version,
            };

            if catalog_seen.insert(entry.result_id.clone()) {
                catalog_items.push(entry.result_id.clone());
            }
            by_result.entry(entry.result_id.clone()).or_default().push(idx);

            let mut ing_dedup = HashSet::new();
            for slot in &entry.slots {
                if slot.is_empty() {
                    continue;
                }
                if let Some(tag_name) = slot.strip_prefix('#') {
                    if let Some(variants) = raw.tags.get(tag_name) {
                        for v in variants {
                            if ing_dedup.insert(v.clone()) {
                                by_ingredient.entry(v.clone()).or_default().push(idx);
                            }
                        }
                    }
                    if ing_dedup.insert(slot.clone()) {
                        by_ingredient.entry(slot.clone()).or_default().push(idx);
                    }
                } else if ing_dedup.insert(slot.clone()) {
                    by_ingredient.entry(slot.clone()).or_default().push(idx);
                }
            }

            recipes.push(entry);
        }

        // Sort catalog alphabetically by English ID cleanly after grouping stations/types
        catalog_items.sort();

        Ok(Self {
            version: raw.version,
            tags: raw.tags,
            recipes,
            lang_items,
            catalog_items,
            by_result,
            by_ingredient,
        })
    }

    /// Returns official Mojang localized display name for any item ID or `#tag`.
    pub fn item_name(&self, id_or_tag: &str, lang: Language) -> String {
        if let Some(tag) = id_or_tag.strip_prefix('#') {
            if let Some(list) = self.tags.get(tag) {
                if let Some(first) = list.first() {
                    let prefix = match lang {
                        Language::EnUs => "Any ",
                        _ => "Cualquier ",
                    };
                    return format!("{}{} ({})", prefix, self.item_name(first, lang), tag);
                }
            }
            return pretty_id(tag);
        }
        if let Some(names) = self.lang_items.get(id_or_tag) {
            let idx = match lang {
                Language::EsMx => 0,
                Language::EsEs => 1,
                Language::EnUs => 2,
            };
            return names[idx].clone();
        }
        pretty_id(id_or_tag)
    }

    /// Returns active concrete item variants for a `#tag` token in `active_ver`.
    pub fn resolve_tag_in<'a>(
        &'a self,
        slot_token: &str,
        versions: &VersionRegistry,
        active_ver: &str,
    ) -> Vec<&'a str> {
        let Some(tag) = slot_token.strip_prefix('#') else {
            return Vec::new();
        };
        self.tags
            .get(tag)
            .map(|list| {
                list.iter()
                    .map(|s| s.as_str())
                    .filter(|id| super::version_rules::is_item_active(versions, id, active_ver))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Localized display name using active-version tag examples (never picks future blocks for `#tags`).
    pub fn item_name_in(
        &self,
        id_or_tag: &str,
        versions: &VersionRegistry,
        active_ver: &str,
        lang: Language,
    ) -> String {
        if let Some(tag) = id_or_tag.strip_prefix('#') {
            let active_list = self.resolve_tag_in(id_or_tag, versions, active_ver);
            if let Some(&first) = active_list.first() {
                let prefix = match lang {
                    Language::EnUs => "Any ",
                    _ => "Cualquier ",
                };
                return format!("{}{} ({})", prefix, self.item_name(first, lang), tag);
            }
            return pretty_id(tag);
        }
        self.item_name(id_or_tag, lang)
    }

    /// Returns concrete item ID for a slot token at animation step `step`, respecting `active_ver`.
    pub fn resolve_slot_icon_in(
        &self,
        slot_token: &str,
        step: usize,
        versions: &VersionRegistry,
        active_ver: &str,
    ) -> String {
        if slot_token.starts_with('#') {
            let variants = self.resolve_tag_in(slot_token, versions, active_ver);
            if !variants.is_empty() {
                return variants[step % variants.len()].to_string();
            }
        }
        slot_token.to_string()
    }

    /// Returns `true` when recipe `idx` is active in `active_ver` (including `#tag` slots).
    pub fn is_recipe_active(
        &self,
        idx: usize,
        versions: &VersionRegistry,
        active_ver: &str,
    ) -> bool {
        let Some(rec) = self.recipes.get(idx) else {
            return false;
        };
        if !rec.is_active_in(versions, active_ver) {
            return false;
        }
        rec.slots.iter().all(|slot| {
            !slot.starts_with('#') || !self.resolve_tag_in(slot, versions, active_ver).is_empty()
        })
    }

    /// Returns concrete item ID for a slot token at animation step `tick_sec` (cycles `#tag` lists cleanly).
    pub fn resolve_slot_icon(&self, slot_token: &str, step: usize) -> String {
        if let Some(tag) = slot_token.strip_prefix('#') {
            if let Some(variants) = self.tags.get(tag) {
                if !variants.is_empty() {
                    return variants[step % variants.len()].clone();
                }
            }
        }
        slot_token.to_string()
    }

    /// Exact ID O(1) lookup for recipes producing `item_id`.
    pub fn recipes_for(&self, item_id: &str) -> &[usize] {
        self.by_result
            .get(item_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// True if `item_id` existed in `active_ver` AND has an active recipe (or is a raw ingredient).
    pub fn item_active_in(
        &self,
        item_id: &str,
        versions: &VersionRegistry,
        active_ver: &str,
    ) -> bool {
        if !super::version_rules::is_item_active(versions, item_id, active_ver) {
            return false;
        }
        let idxs = self.recipes_for(item_id);
        if idxs.is_empty() {
            return true;
        }
        idxs.iter()
            .any(|&idx| self.is_recipe_active(idx, versions, active_ver))
    }

    /// Exact ID O(1) lookup for recipes consuming `item_id` (including `#tag` matches).
    pub fn uses_for(&self, item_id: &str) -> &[usize] {
        self.by_ingredient
            .get(item_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Version-filtered lookup for active recipes consuming `item_id`.
    pub fn uses_for_in(
        &self,
        item_id: &str,
        versions: &VersionRegistry,
        active_ver: &str,
    ) -> Vec<usize> {
        self.uses_for(item_id)
            .iter()
            .copied()
            .filter(|&idx| self.is_recipe_active(idx, versions, active_ver))
            .collect()
    }

    /// Picks the canonical recipe index (prefers standard Crafting Table over alternate smelting/stonecutting).
    pub fn primary_recipe_for(&self, item_id: &str) -> Option<usize> {
        let list = self.recipes_for(item_id);
        list.iter()
            .copied()
            .find(|&idx| self.recipes[idx].station == CraftingStation::Crafting)
            .or_else(|| list.first().copied())
    }

    /// Picks the canonical recipe index active inside `active_ver`.
    pub fn primary_recipe_for_in(
        &self,
        item_id: &str,
        versions: &VersionRegistry,
        active_ver: &str,
    ) -> Option<usize> {
        let list = self.recipes_for(item_id);
        list.iter()
            .copied()
            .filter(|&idx| self.is_recipe_active(idx, versions, active_ver))
            .find(|&idx| self.recipes[idx].station == CraftingStation::Crafting)
            .or_else(|| {
                list.iter()
                    .copied()
                    .find(|&idx| self.is_recipe_active(idx, versions, active_ver))
            })
    }

    /// Computes EMI-style recursive bill of materials + raw base ingredients with cycle protection.
    #[allow(dead_code)]
    pub fn build_craft_tree(&self, item_id: &str, quantity: u32) -> (CraftTreeNode, BTreeMap<String, u32>) {
        let mut raw_totals: BTreeMap<String, u32> = BTreeMap::new();
        let mut visiting = HashSet::new();
        let tree = self.build_tree_rec(
            item_id,
            quantity.max(1),
            0,
            None,
            &mut visiting,
            &mut raw_totals,
        );
        (tree, raw_totals)
    }

    /// Version-aware recursive bill of materials respecting altered recipes (e.g. 1.18.2 vs 26.3).
    pub fn build_craft_tree_in(
        &self,
        item_id: &str,
        quantity: u32,
        versions: &VersionRegistry,
        active_ver: &str,
    ) -> (CraftTreeNode, BTreeMap<String, u32>) {
        let mut raw_totals: BTreeMap<String, u32> = BTreeMap::new();
        let mut visiting = HashSet::new();
        let tree = self.build_tree_rec(
            item_id,
            quantity.max(1),
            0,
            Some((versions, active_ver)),
            &mut visiting,
            &mut raw_totals,
        );
        (tree, raw_totals)
    }

    fn build_tree_rec(
        &self,
        item_id: &str,
        needed: u32,
        depth: usize,
        ver_ctx: Option<(&VersionRegistry, &str)>,
        visiting: &mut HashSet<String>,
        raw_totals: &mut BTreeMap<String, u32>,
    ) -> CraftTreeNode {
        let concrete = match ver_ctx {
            Some((versions, active_ver)) => self.resolve_slot_icon_in(item_id, 0, versions, active_ver),
            None => self.resolve_slot_icon(item_id, 0),
        };

        if depth >= 6 || visiting.contains(&concrete) {
            *raw_totals.entry(concrete.clone()).or_default() += needed;
            return CraftTreeNode {
                item_id: concrete,
                total_needed: needed,
                via_recipe: None,
                batches: 0,
                children: Vec::new(),
            };
        }

        let primary = match ver_ctx {
            Some((versions, active_ver)) => self.primary_recipe_for_in(&concrete, versions, active_ver),
            None => self.primary_recipe_for(&concrete),
        };
        let Some(rec_idx) = primary else {
            *raw_totals.entry(concrete.clone()).or_default() += needed;
            return CraftTreeNode {
                item_id: concrete,
                total_needed: needed,
                via_recipe: None,
                batches: 0,
                children: Vec::new(),
            };
        };

        let rec = &self.recipes[rec_idx];
        let out_count = rec.result_count.max(1);
        let batches = needed.div_ceil(out_count);

        let mut per_batch_inputs: BTreeMap<String, u32> = BTreeMap::new();
        for slot in &rec.slots {
            if !slot.is_empty() {
                let canonical_in = match ver_ctx {
                    Some((versions, active_ver)) => {
                        self.resolve_slot_icon_in(slot, 0, versions, active_ver)
                    }
                    None => self.resolve_slot_icon(slot, 0),
                };
                *per_batch_inputs.entry(canonical_in).or_default() += 1;
            }
        }

        visiting.insert(concrete.clone());
        let mut children = Vec::with_capacity(per_batch_inputs.len());
        for (child_id, per_batch) in per_batch_inputs {
            // Avoid self-decomposition on reversible block->ingot->block compression cycles
            let child_node = self.build_tree_rec(
                &child_id,
                per_batch * batches,
                depth + 1,
                ver_ctx,
                visiting,
                raw_totals,
            );
            children.push(child_node);
        }
        visiting.remove(&concrete);

        CraftTreeNode {
            item_id: concrete,
            total_needed: needed,
            via_recipe: Some(rec_idx),
            batches,
            children,
        }
    }

    pub fn matches_item_query(&self, item_id: &str, query_lower: &str) -> bool {
        if query_lower.is_empty() {
            return true;
        }
        let q = strip_accents_lower(query_lower);
        if item_id.to_lowercase().contains(&q) {
            return true;
        }
        if let Some(names) = self.lang_items.get(item_id) {
            if names.iter().any(|n| strip_accents_lower(n).contains(&q)) {
                return true;
            }
        }
        if matches_alchemy_keyword(item_id, &q) {
            return true;
        }
        if matches!(item_id, "potion" | "splash_potion" | "lingering_potion" | "tipped_arrow") {
            for &idx in self.recipes_for(item_id) {
                let note = &self.recipes[idx].note;
                if !note.is_empty()
                    && (strip_accents_lower(note).contains(&q)
                        || strip_accents_lower(&translate_brewing_note(note, Language::EsMx))
                            .contains(&q))
                {
                    return true;
                }
            }
        }
        false
    }
}

fn strip_accents_lower(s: &str) -> String {
    s.chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| match c {
            'á' | 'à' | 'ä' => 'a',
            'é' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'ï' => 'i',
            'ó' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            'ñ' => 'n',
            other => other,
        })
        .collect()
}

fn matches_alchemy_keyword(item_id: &str, q_norm: &str) -> bool {
    if q_norm.len() < 3 {
        return false;
    }
    let keywords: &[&str] = match item_id {
        "potion" => &[
            "pocion", "pociones", "alquimia", "tiempo", "duracion", "potente", "potencia",
            "nivel ii", "nivel 2", "larga", "fuerza", "invisibilidad", "curacion",
            "regeneracion", "velocidad", "rapidez", "salto", "veneno", "dano", "debilidad",
            "lentitud", "caida lenta", "respiracion", "tortuga", "infest", "oozing",
            "weaving", "wind", "rara", "awkward",
        ],
        "splash_potion" => &[
            "pocion", "pociones", "lanzadora", "lanzador", "arrojadiza", "lanzar", "splash",
            "alquimia", "polvora", "tiempo", "potente",
        ],
        "lingering_potion" => &[
            "pocion", "pociones", "persistente", "nube", "lingering", "aliento", "dragon",
            "flecha", "tipped", "alquimia",
        ],
        "tipped_arrow" => &["flecha", "efecto", "pocion", "persistente", "tipped"],
        "brewing_stand" => &[
            "pocion", "pociones", "alquimia", "soporte", "destilar", "lanzadora", "duracion",
            "potente", "tiempo",
        ],
        "redstone" => &["tiempo", "duracion", "extender", "alargar", "pocion", "pociones", "long"],
        "glowstone_dust" => &["potente", "potencia", "nivel ii", "nivel 2", "subir nivel", "pocion", "strong"],
        "gunpowder" => &["lanzadora", "lanzador", "arrojadiza", "lanzar", "splash", "pocion"],
        "dragon_breath" => &["persistente", "lingering", "nube", "flechas", "pocion"],
        "fermented_spider_eye" => &["invertir", "corromper", "invisibilidad", "lentitud", "dano", "debilidad", "pocion"],
        "nether_wart" => &["pocion rara", "awkward", "base", "pocion", "pociones"],
        "blaze_powder" => &["combustible", "fuerza", "pocion", "pociones", "brewing"],
        _ => &[],
    };
    keywords.iter().any(|kw| kw.contains(q_norm) || q_norm.contains(kw))
}

/// Translates `"Awkward -> Long Strength"` into the selected language cleanly.
pub fn translate_brewing_note(note: &str, lang: Language) -> String {
    if let Some((src, dst)) = note.split_once(" -> ") {
        format!(
            "{}  ==>  {}",
            translate_potion_term(src.trim(), lang),
            translate_potion_term(dst.trim(), lang)
        )
    } else {
        translate_potion_term(note, lang)
    }
}

pub fn translate_potion_term(term: &str, lang: Language) -> String {
    if let Some(rest) = term.strip_prefix("Long ") {
        let base = translate_base_effect(rest, lang);
        return match lang {
            Language::EnUs => format!("{} (+Duration)", base),
            _ => format!("{} (+Duración / Larga)", base),
        };
    }
    if let Some(rest) = term.strip_prefix("Strong ") {
        let base = translate_base_effect(rest, lang);
        return match lang {
            Language::EnUs => format!("{} II (+Potency)", base),
            _ => format!("{} II (+Potente)", base),
        };
    }
    translate_base_effect(term, lang).to_string()
}

fn translate_base_effect(raw: &str, lang: Language) -> &'static str {
    match (raw, lang) {
        ("Water", Language::EnUs) => "Water Bottle",
        ("Water", _) => "Frasco con Agua",
        ("Awkward", Language::EnUs) => "Awkward Potion (Base)",
        ("Awkward", _) => "Poción Rara (Base)",
        ("Mundane", Language::EnUs) => "Mundane Potion (No Effect)",
        ("Mundane", _) => "Poción Vulgar (Sin efecto)",
        ("Thick", Language::EnUs) => "Thick Potion (No Effect)",
        ("Thick", _) => "Poción Espesa (Sin efecto)",
        ("Fire Resistance", Language::EnUs) => "Fire Resistance",
        ("Fire Resistance", _) => "Resistencia al Fuego",
        ("Harming", Language::EnUs) => "Instant Harming",
        ("Harming", _) => "Daño Instantáneo",
        ("Healing", Language::EnUs) => "Instant Healing",
        ("Healing", _) => "Curación Instantánea",
        ("Infested", Language::EnUs) => "Infested (Silverfish)",
        ("Infested", _) => "Infestación (Silverfish)",
        ("Invisibility", Language::EnUs) => "Invisibility",
        ("Invisibility", _) => "Invisibilidad",
        ("Leaping", Language::EnUs) => "Jump Boost",
        ("Leaping", _) => "Salto",
        ("Night Vision", Language::EnUs) => "Night Vision",
        ("Night Vision", _) => "Visión Nocturna",
        ("Oozing", Language::EnUs) => "Oozing (Slimes)",
        ("Oozing", _) => "Viscosidad (Slimes)",
        ("Poison", Language::EnUs) => "Poison",
        ("Poison", _) => "Veneno",
        ("Regeneration", Language::EnUs) => "Regeneration",
        ("Regeneration", _) => "Regeneración",
        ("Slow Falling", Language::EnUs) => "Slow Falling",
        ("Slow Falling", _) => "Caída Lenta",
        ("Slowness", Language::EnUs) => "Slowness",
        ("Slowness", _) => "Lentitud",
        ("Strength", Language::EnUs) => "Strength",
        ("Strength", _) => "Fuerza",
        ("Swiftness", Language::EnUs) => "Speed / Swiftness",
        ("Swiftness", Language::EsMx) => "Velocidad (Rapidez)",
        ("Swiftness", Language::EsEs) => "Rapidez (Velocidad)",
        ("Turtle Master", Language::EnUs) => "The Turtle Master",
        ("Turtle Master", _) => "Maestro Tortuga",
        ("Water Breathing", Language::EnUs) => "Water Breathing",
        ("Water Breathing", _) => "Respiración Acuática",
        ("Weakness", Language::EnUs) => "Weakness",
        ("Weakness", _) => "Debilidad",
        ("Weaving", Language::EnUs) => "Weaving (Cobwebs)",
        ("Weaving", _) => "Tejido (Telarañas)",
        ("Wind Charged", Language::EnUs) => "Wind Charged",
        ("Wind Charged", _) => "Carga de Viento",
        _ => "Potion",
    }
}

fn pretty_id(raw: &str) -> String {
    raw.split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use crate::data::loader::load_recipes;

    #[test]
    fn recipes_load_26_3_without_missing_translations() {
        let db = load_recipes().expect("recipes must parse cleanly");
        assert!(db.recipes.len() > 2000, "expected >= 2000 26.3 recipes");
        assert!(!db.by_result.is_empty());
        assert!(!db.by_ingredient.is_empty());

        // Verify exact ID lookup for iron_ingot uses doesn't accidentally match raw_iron_block, etc.
        let beacon_uses = db.uses_for("nether_star");
        assert!(beacon_uses.iter().any(|&i| db.recipes[i].result_id == "beacon"));

        let iron_recipes = db.recipes_for("iron_pickaxe");
        assert!(!iron_recipes.is_empty());

        let (tree, raw) = db.build_craft_tree("beacon", 1);
        assert_eq!(tree.item_id, "beacon");
        assert!(raw.contains_key("nether_star"));
        assert!(raw.contains_key("obsidian"));
        assert!(raw.contains_key("sand") || raw.contains_key("glass"));

        assert_eq!(
            super::translate_brewing_note("Awkward -> Strong Strength", super::Language::EsMx),
            "Poción Rara (Base)  ==>  Fuerza II (+Potente)"
        );
        assert!(db.matches_item_query("splash_potion", "lanzadora"));
        assert!(db.matches_item_query("redstone", "tiempo"));
        assert!(db.matches_item_query("glowstone_dust", "potente"));
    }
}
