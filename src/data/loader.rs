use super::models::{Database, VersionRegistry};
use super::recipes::RecipeDatabase;

static DATABASE_BYTES: &[u8] = include_bytes!("../../data/database.json");
static RECIPES_BYTES: &[u8] = include_bytes!("../../data/recipes.json");
static LANG_ITEMS_BYTES: &[u8] = include_bytes!("../../data/lang_items.json");
static VERSIONS_BYTES: &[u8] = include_bytes!("../../data/versions.json");

fn strip_utf8_bom(bytes: &[u8]) -> &[u8] {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    }
}

/// Loads the bundled version registry so content filters and per-version overrides can evolve cleanly.
pub fn load_versions() -> Result<VersionRegistry, String> {
    let slice = strip_utf8_bom(VERSIONS_BYTES);
    serde_json::from_slice(slice).map_err(|e| format!("Failed to parse versions.json: {}", e))
}

/// Loads the bundled trilingual 26.3 companion database cleanly without filesystem fragility.
pub fn load_database() -> Result<Database, String> {
    let slice = strip_utf8_bom(DATABASE_BYTES);
    serde_json::from_slice(slice).map_err(|e| format!("Failed to parse database.json: {}", e))
}

/// Loads the bundled 26.3 JEI recipe catalog and trilingual Mojang item names.
pub fn load_recipes() -> Result<RecipeDatabase, String> {
    let rec_slice = strip_utf8_bom(RECIPES_BYTES);
    let lang_slice = strip_utf8_bom(LANG_ITEMS_BYTES);
    RecipeDatabase::from_json_bytes(rec_slice, lang_slice)
}
