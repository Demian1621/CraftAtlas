use super::models::VersionRegistry;
use super::seed_engine::StructureKind;

/// Returns `true` when `item_id` is available in `active_ver` according to Java release rules.
pub fn is_item_active(versions: &VersionRegistry, item_id: &str, active_ver: &str) -> bool {
    versions.is_active(item_since_version(item_id), None, active_ver)
}

/// Returns the minimum CraftAtlas version ID in which a Minecraft item/block existed,
/// or `None` if it already existed in Java 1.18.2 or earlier.
///
/// NOTE: Newest versions MUST be evaluated before older ones so compound IDs
/// (e.g. `mangrove_shelf` in 26.3, `pale_oak_hanging_sign` in 1.21.4) match accurately.
pub fn item_since_version(item_id: &str) -> Option<&'static str> {
    // --- Java 26.3 ("Wilderness Bound" / 1.21.5-26.3 copper & mount expansions) ---
    if item_id.ends_with("_spear")
        || (item_id.ends_with("_shelf")
            && item_id != "bookshelf"
            && item_id != "chiseled_bookshelf")
        || item_id == "shelf_mushroom"
        || item_id.ends_with("_harness")
        || item_id.ends_with("_cushion")
        || item_id.ends_with("_wool_slab")
        || item_id.ends_with("_wool_stairs")
        || item_id.ends_with("_concrete_slab")
        || item_id.ends_with("_concrete_stairs")
        || item_id.contains("poplar")
        || item_id.contains("copper_golem_statue")
        || item_id.contains("copper_chest")
        || item_id.contains("copper_bars")
        || item_id.contains("copper_chain")
        || item_id.contains("copper_lantern")
        || item_id.contains("nautilus")
        || matches!(
            item_id,
            "copper_helmet"
                | "copper_chestplate"
                | "copper_leggings"
                | "copper_boots"
                | "copper_sword"
                | "copper_pickaxe"
                | "copper_axe"
                | "copper_shovel"
                | "copper_hoe"
                | "copper_nugget"
                | "copper_torch"
                | "copper_horse_armor"
                | "dried_ghast"
                | "happy_ghast"
                | "netherite_horse_armor"
                | "blue_egg"
                | "brown_egg"
                | "leaf_litter"
                | "cactus_flower"
                | "wildflowers"
                | "firefly_bush"
                | "bush"
                | "short_dry_grass"
                | "tall_dry_grass"
                | "music_disc_tears"
                | "music_disc_lava_chicken"
        )
    {
        return Some("26.3");
    }

    // --- Java 1.21.4 ("The Garden Awakens" + Bundles of Bravery 1.21.2) ---
    if item_id.contains("pale_oak")
        || item_id.contains("pale_moss")
        || item_id.contains("resin")
        || item_id.contains("eyeblossom")
        || item_id == "creaking_heart"
        || item_id == "bundle"
        || item_id.ends_with("_bundle")
        || matches!(
            item_id,
            "bordure_indented_banner_pattern" | "field_masoned_banner_pattern"
        )
    {
        return Some("1.21.4");
    }

    // --- Java 1.21.1 ("Tricky Trials") ---
    if item_id.contains("tuff_brick")
        || item_id.starts_with("polished_tuff")
        || item_id.contains("copper_bulb")
        || item_id.contains("copper_grate")
        || item_id.contains("copper_door")
        || item_id.contains("copper_trapdoor")
        || item_id.contains("chiseled_copper")
        || matches!(
            item_id,
            "crafter"
                | "mace"
                | "heavy_core"
                | "breeze_rod"
                | "wind_charge"
                | "trial_key"
                | "ominous_trial_key"
                | "ominous_bottle"
                | "vault"
                | "trial_spawner"
                | "bolt_armor_trim_smithing_template"
                | "flow_armor_trim_smithing_template"
                | "flow_banner_pattern"
                | "guster_banner_pattern"
                | "flow_pottery_sherd"
                | "guster_pottery_sherd"
                | "scrape_pottery_sherd"
                | "chiseled_tuff"
                | "tuff_slab"
                | "tuff_stairs"
                | "tuff_wall"
                | "tuff_bricks"
                | "music_disc_creator"
                | "music_disc_creator_music_box"
                | "music_disc_precipice"
        )
    {
        return Some("1.21.1");
    }

    // --- Java 1.20.6 ("Trails & Tales" + 1.20.5 Armored Paws) ---
    if item_id.contains("cherry_")
        || item_id.ends_with("_hanging_sign")
        || item_id.ends_with("_pottery_sherd")
        || item_id.ends_with("_armor_trim_smithing_template")
        || item_id.starts_with("bamboo_")
        || item_id == "stripped_bamboo_block"
        || matches!(
            item_id,
            "netherite_upgrade_smithing_template"
                | "chiseled_bookshelf"
                | "brush"
                | "decorated_pot"
                | "calibrated_sculk_sensor"
                | "pink_petals"
                | "torchflower"
                | "torchflower_seeds"
                | "pitcher_plant"
                | "pitcher_pod"
                | "sniffer_egg"
                | "wolf_armor"
                | "armadillo_scute"
                | "suspicious_sand"
                | "suspicious_gravel"
                | "piglin_head"
                | "music_disc_relic"
        )
    {
        return Some("1.20.6");
    }

    // --- Java 1.19.4 ("The Wild Update") ---
    if item_id.contains("mangrove")
        || item_id.ends_with("_chest_boat")
        || matches!(
            item_id,
            "mud"
                | "packed_mud"
                | "mud_bricks"
                | "mud_brick_slab"
                | "mud_brick_stairs"
                | "mud_brick_wall"
                | "recovery_compass"
                | "echo_shard"
                | "disc_fragment_5"
                | "music_disc_5"
                | "goat_horn"
                | "tadpole_bucket"
                | "ochre_froglight"
                | "verdant_froglight"
                | "pearlescent_froglight"
                | "sculk"
                | "sculk_vein"
                | "sculk_catalyst"
                | "sculk_shrieker"
                | "sculk_sensor"
                | "reinforced_deepslate"
                | "warden_head"
        )
    {
        return Some("1.19.4");
    }

    None
}

/// Returns the minimum CraftAtlas version ID for a vanilla enchantment ID.
pub fn enchant_since_version(enchant_id: &str) -> Option<&'static str> {
    match enchant_id {
        "lunge" => Some("26.3"),
        "density" | "breach" | "wind_burst" => Some("1.21.1"),
        "swift_sneak" => Some("1.19.4"),
        _ => None,
    }
}

/// Returns the minimum CraftAtlas version ID in which a Seed Map structure existed.
pub fn structure_since_version(kind: StructureKind) -> Option<&'static str> {
    match kind {
        StructureKind::TrialChambers => Some("1.21.1"),
        StructureKind::TrailRuins => Some("1.20.6"),
        StructureKind::AncientCity => Some("1.19.4"),
        _ => None,
    }
}

/// Returns the minimum CraftAtlas version ID in which a Cubiomes biome ID existed.
pub fn biome_since_version(biome_id: i32) -> Option<&'static str> {
    match biome_id {
        186 => Some("1.21.4"),       // Pale Garden
        185 => Some("1.20.6"),       // Cherry Grove
        183 | 184 => Some("1.19.4"), // Deep Dark | Mangrove Swamp
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::load_versions;

    #[test]
    fn item_rules_order_compound_names_and_keep_legacy_items() {
        let versions = load_versions().expect("versions.json must load");

        // Pre-1.19 items stay active in 1.18.2
        for legacy in [
            "bamboo",
            "bookshelf",
            "tuff",
            "copper_block",
            "cut_copper",
            "saddle",
            "netherite_chestplate",
            "lodestone",
            "lead",
        ] {
            assert!(
                is_item_active(&versions, legacy, "1.18.2"),
                "legacy item '{legacy}' must be active in 1.18.2"
            );
        }

        // Strict gating in 1.18.2
        for modern in [
            "mangrove_planks",
            "mud_bricks",
            "sculk_catalyst",
            "oak_chest_boat",
            "cherry_planks",
            "bamboo_block",
            "oak_hanging_sign",
            "netherite_upgrade_smithing_template",
            "wolf_armor",
            "brush",
            "crafter",
            "mace",
            "heavy_core",
            "breeze_rod",
            "pale_oak_planks",
            "creaking_heart",
            "bundle",
            "copper_sword",
            "iron_spear",
            "mangrove_shelf",
            "pale_oak_hanging_sign",
        ] {
            assert!(
                !is_item_active(&versions, modern, "1.18.2"),
                "modern item '{modern}' must NOT be active in 1.18.2"
            );
        }

        // Compound name ordering checks
        assert_eq!(item_since_version("mangrove_shelf"), Some("26.3"));
        assert_eq!(item_since_version("pale_oak_hanging_sign"), Some("1.21.4"));
        assert_eq!(item_since_version("chiseled_tuff_bricks"), Some("1.21.1"));
        assert_eq!(item_since_version("flow_armor_trim_smithing_template"), Some("1.21.1"));
        assert_eq!(item_since_version("coast_armor_trim_smithing_template"), Some("1.20.6"));
        assert_eq!(item_since_version("mangrove_propagule"), Some("1.19.4"));
        assert_eq!(item_since_version("bamboo_shelf"), Some("26.3"));

        // Everything active in 26.3
        assert!(is_item_active(&versions, "iron_spear", "26.3"));
        assert!(is_item_active(&versions, "mangrove_shelf", "26.3"));
        assert!(is_item_active(&versions, "bundle", "26.3"));
        assert!(is_item_active(&versions, "mace", "26.3"));
    }

    #[test]
    fn strict_per_version_filtering_across_databases() {
        use crate::data::loader::{load_database, load_recipes};

        let versions = load_versions().expect("versions.json must load");
        let db = load_database().expect("database.json must load");
        let recipes = load_recipes().expect("recipes.json must load");

        // 1. Enchanted books progression: 38 -> 39 -> 39 -> 42 -> 42 -> 43
        let count_books = |ver: &str| {
            db.enchant_books
                .iter()
                .filter(|b| versions.is_active(b.since_version.as_deref(), b.until_version.as_deref(), ver))
                .count()
        };
        assert_eq!(count_books("1.18.2"), 38);
        assert_eq!(count_books("1.19.4"), 39);
        assert_eq!(count_books("1.20.6"), 39);
        assert_eq!(count_books("1.21.1"), 42);
        assert_eq!(count_books("1.21.4"), 42);
        assert_eq!(count_books("26.3"), 43);

        // 2. Leggings drops Swift Sneak in 1.18.2 and keeps it in 1.19.4+
        let leggings = db.gear.iter().find(|g| g.id == "leggings").expect("leggings");
        assert!(
            !leggings.resolved_for(&versions, "1.18.2").enchantments.iter().any(|e| e.id == "swift_sneak"),
            "1.18.2 leggings must not include swift_sneak"
        );
        assert!(
            leggings.resolved_for(&versions, "1.19.4").enchantments.iter().any(|e| e.id == "swift_sneak"),
            "1.19.4 leggings must include swift_sneak"
        );

        // 3. Recipes swap accurately per version
        let netherite_118 = recipes
            .primary_recipe_for_in("netherite_chestplate", &versions, "1.18.2")
            .map(|i| &recipes.recipes[i])
            .expect("legacy smithing recipe in 1.18.2");
        assert_eq!(netherite_118.slots[0], "");
        let netherite_120 = recipes
            .primary_recipe_for_in("netherite_chestplate", &versions, "1.20.6")
            .map(|i| &recipes.recipes[i])
            .expect("modern smithing recipe in 1.20.6");
        assert_eq!(netherite_120.slots[0], "netherite_upgrade_smithing_template");

        let lodes_old = recipes
            .primary_recipe_for_in("lodestone", &versions, "1.21.4")
            .map(|i| &recipes.recipes[i])
            .expect("legacy lodestone in 1.21.4");
        assert_eq!(lodes_old.slots[4], "netherite_ingot");
        let lodes_new = recipes
            .primary_recipe_for_in("lodestone", &versions, "26.3")
            .map(|i| &recipes.recipes[i])
            .expect("modern lodestone in 26.3");
        assert_eq!(lodes_new.slots[4], "iron_ingot");

        assert!(recipes.primary_recipe_for_in("saddle", &versions, "1.21.4").is_none());
        assert!(recipes.primary_recipe_for_in("saddle", &versions, "26.3").is_some());
        assert!(!recipes.item_active_in("crafter", &versions, "1.20.6"));
        assert!(recipes.item_active_in("crafter", &versions, "1.21.1"));

        // 4. Dupes flip status accurately per version
        let string_duper = db.dupes.iter().find(|d| d.id == "string_duper").expect("string_duper");
        assert_eq!(string_duper.resolved("1.21.1").status_key, "working");
        assert_eq!(string_duper.resolved("1.21.4").status_key, "patched");
        let void_trading = db.dupes.iter().find(|d| d.id == "void_trading_patched").expect("void_trading_patched");
        assert_eq!(void_trading.resolved("1.20.6").status_key, "working");
        assert_eq!(void_trading.resolved("1.21.1").status_key, "patched");
    }
}
