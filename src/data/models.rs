use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

/// Supported application languages aligned with Minecraft Java Edition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    EsMx,
    EsEs,
    EnUs,
}

impl Language {
    pub fn short_label(self) -> &'static str {
        match self {
            Language::EsMx => "ES-MX",
            Language::EsEs => "ES-ES",
            Language::EnUs => "EN-US",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Language::EsMx => "Español (LatAm)",
            Language::EsEs => "Español (España)",
            Language::EnUs => "English (US)",
        }
    }

    pub fn all() -> [Language; 3] {
        [Language::EsMx, Language::EsEs, Language::EnUs]
    }
}

/// Trilingual string container across Mexican Spanish, Spain Spanish, and US English.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalizedText {
    pub es_mx: String,
    pub es_es: String,
    pub en_us: String,
}

impl LocalizedText {
    pub fn get(&self, lang: Language) -> &str {
        match lang {
            Language::EsMx => &self.es_mx,
            Language::EsEs => &self.es_es,
            Language::EnUs => &self.en_us,
        }
    }

    pub fn matches(&self, query_lower: &str) -> bool {
        self.es_mx.to_lowercase().contains(query_lower)
            || self.es_es.to_lowercase().contains(query_lower)
            || self.en_us.to_lowercase().contains(query_lower)
    }
}

/// Declarative version definition enabling future updates (`26.4`, etc.) to add/modify/sunset items cleanly.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct GameVersionDef {
    pub id: String,
    pub label: String,
    pub subtitle: LocalizedText,
    #[serde(default)]
    pub cubiomes_mc: String,
    #[serde(default)]
    pub highlights: Vec<LocalizedText>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionRegistry {
    pub default_version: String,
    pub versions: Vec<GameVersionDef>,
}

impl VersionRegistry {
    pub fn find(&self, id: &str) -> Option<&GameVersionDef> {
        self.versions.iter().find(|v| v.id == id)
    }

    pub fn resolve_valid_id<'a>(&'a self, candidate: &'a str) -> &'a str {
        if self.find(candidate).is_some() {
            candidate
        } else {
            self.versions
                .iter()
                .find(|v| v.id == self.default_version)
                .or_else(|| self.versions.first())
                .map(|v| v.id.as_str())
                .unwrap_or("26.3")
        }
    }

    fn version_rank(&self, id: &str) -> Option<usize> {
        self.versions.iter().position(|v| v.id == id)
    }

    pub fn is_active(&self, since: Option<&str>, until: Option<&str>, active_version: &str) -> bool {
        let cmp = |a: &str, b: &str| -> std::cmp::Ordering {
            match (self.version_rank(a), self.version_rank(b)) {
                (Some(ia), Some(ib)) => ia.cmp(&ib),
                _ => parse_version_tuple(a).cmp(&parse_version_tuple(b)),
            }
        };
        if let Some(s) = since {
            if cmp(active_version, s) == std::cmp::Ordering::Less {
                return false;
            }
        }
        if let Some(u) = until {
            if cmp(active_version, u) == std::cmp::Ordering::Greater {
                return false;
            }
        }
        true
    }
}

fn parse_version_tuple(ver: &str) -> (u32, u32, u32) {
    let mut nums = ver
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<u32>().ok());
    (
        nums.next().unwrap_or(0),
        nums.next().unwrap_or(0),
        nums.next().unwrap_or(0),
    )
}

/// Optional per-version partial overrides so updating a database item for a new release never duplicates entries.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ContentVersionOverride {
    #[serde(default)]
    pub name: Option<LocalizedText>,
    #[serde(default)]
    pub title: Option<LocalizedText>,
    #[serde(default)]
    pub what_it_does: Option<LocalizedText>,
    #[serde(default)]
    pub what_it_is_for: Option<LocalizedText>,
    #[serde(default)]
    pub rate: Option<LocalizedText>,
    #[serde(default)]
    pub desc: Option<LocalizedText>,
    #[serde(default)]
    pub tips: Option<LocalizedText>,
    #[serde(default)]
    pub status_key: Option<String>,
    #[serde(default)]
    pub status: Option<LocalizedText>,
    #[serde(default)]
    pub how_it_works: Option<LocalizedText>,
    #[serde(default)]
    pub text: Option<LocalizedText>,
}

/// Structured specialty tag displayed on creator cards and filterable in the directory.
#[derive(Debug, Clone, Deserialize)]
pub struct CreatorTag {
    pub key: String,
    pub label: LocalizedText,
}

/// Verified YouTube / Bilibili technical Minecraft creator recommendation.
#[derive(Debug, Clone, Deserialize)]
pub struct TechCreator {
    pub id: String,
    pub name: String,
    pub handle: String,
    pub region_key: String,
    pub platform_key: String,
    pub platform_label: String,
    pub region_label: LocalizedText,
    pub specialty: LocalizedText,
    pub desc: LocalizedText,
    pub url: String,
    pub icon: String,
    #[serde(default)]
    pub focus_keys: Vec<String>,
    #[serde(default)]
    pub tags: Vec<CreatorTag>,
    #[serde(default)]
    pub characteristics: Vec<LocalizedText>,
    #[serde(default)]
    pub recommended_for: Vec<String>,
}

impl TechCreator {
    /// Returns true if this creator covers the given specialty key (`"farms"`, `"dupes"`, `"redstone"`).
    pub fn has_focus(&self, focus: &str) -> bool {
        focus == "all" || self.focus_keys.iter().any(|k| k == focus)
    }
}

/// Root guide database bundled cleanly with extensible version filtering.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Database {
    pub version: String,
    pub gear: Vec<GearItem>,
    pub farms: Vec<FarmItem>,
    pub dupes: Vec<DupeItem>,
    pub pro_tips: Vec<ProTip>,
    #[serde(default)]
    pub creators: Vec<TechCreator>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GearItem {
    pub id: String,
    pub category: String,
    #[serde(default)]
    pub secondary_tags: Vec<String>,
    pub icon: String,
    pub name: LocalizedText,
    pub enchantments: Vec<EnchantmentItem>,
    pub anvil_order: Vec<LocalizedText>,
    pub incompatibilities: Vec<LocalizedText>,
    pub what_it_does: LocalizedText,
    pub what_it_is_for: LocalizedText,
    #[serde(default)]
    pub since_version: Option<String>,
    #[serde(default)]
    pub until_version: Option<String>,
    #[serde(default)]
    pub version_overrides: HashMap<String, ContentVersionOverride>,
}

impl GearItem {
    pub fn has_category_filter(&self, filter: &str) -> bool {
        filter == "all"
            || self.category == filter
            || self.secondary_tags.iter().any(|t| t == filter)
    }

    pub fn resolved(&self, active_version: &str) -> Cow<'_, GearItem> {
        let Some(ov) = self.version_overrides.get(active_version) else {
            return Cow::Borrowed(self);
        };
        let mut cloned = self.clone();
        if let Some(ref v) = ov.name {
            cloned.name = v.clone();
        }
        if let Some(ref v) = ov.what_it_does {
            cloned.what_it_does = v.clone();
        }
        if let Some(ref v) = ov.what_it_is_for {
            cloned.what_it_is_for = v.clone();
        }
        Cow::Owned(cloned)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnchantmentItem {
    pub id: String,
    pub level: String,
    #[serde(default)]
    pub level_num: u8,
    pub name: LocalizedText,
    pub desc: LocalizedText,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MaterialItem {
    pub icon: String,
    pub name: LocalizedText,
    pub count: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FarmItem {
    pub id: String,
    pub category_key: String,
    pub category: LocalizedText,
    pub tier_key: String,
    pub tier: LocalizedText,
    #[serde(default)]
    pub icon: String,
    pub title: LocalizedText,
    pub rate: LocalizedText,
    pub desc: LocalizedText,
    pub tips: LocalizedText,
    pub schematic: String,
    pub materials: Vec<MaterialItem>,
    pub tutorial_author: String,
    #[allow(dead_code)]
    pub tutorial_title: LocalizedText,
    #[serde(default)]
    pub tutorial_query: Option<LocalizedText>,
    #[allow(dead_code)]
    pub tutorial_url: String,
    pub diagram: LocalizedText,
    #[serde(default)]
    pub since_version: Option<String>,
    #[serde(default)]
    pub until_version: Option<String>,
    #[serde(default)]
    pub version_overrides: HashMap<String, ContentVersionOverride>,
}

impl FarmItem {
    pub fn resolved(&self, active_version: &str) -> Cow<'_, FarmItem> {
        let Some(ov) = self.version_overrides.get(active_version) else {
            return Cow::Borrowed(self);
        };
        let mut cloned = self.clone();
        if let Some(ref v) = ov.title {
            cloned.title = v.clone();
        }
        if let Some(ref v) = ov.rate {
            cloned.rate = v.clone();
        }
        if let Some(ref v) = ov.desc {
            cloned.desc = v.clone();
        }
        if let Some(ref v) = ov.tips {
            cloned.tips = v.clone();
        }
        Cow::Owned(cloned)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct DupeItem {
    pub id: String,
    pub category_key: String,
    pub category: LocalizedText,
    pub status_key: String,
    pub status: LocalizedText,
    pub difficulty_key: String,
    pub difficulty: LocalizedText,
    pub icon: String,
    pub title: LocalizedText,
    pub components: Vec<LocalizedText>,
    pub how_it_works: LocalizedText,
    pub use_cases: Vec<LocalizedText>,
    #[serde(default)]
    pub tutorial_query: Option<LocalizedText>,
    pub diagram: LocalizedText,
    #[serde(default)]
    pub since_version: Option<String>,
    #[serde(default)]
    pub until_version: Option<String>,
    #[serde(default)]
    pub version_overrides: HashMap<String, ContentVersionOverride>,
}

impl FarmItem {
    /// Returns the explicit farm icon when set, otherwise falls back to the first material icon or `"wheat"`.
    pub fn resolved_icon(&self) -> &str {
        if !self.icon.is_empty() {
            self.icon.as_str()
        } else {
            self.materials
                .first()
                .map(|m| m.icon.as_str())
                .unwrap_or("wheat")
        }
    }

    /// Builds a language-aware and active-version-aware `(query_text, youtube_search_url)` pair.
    pub fn tutorial_search(&self, lang: Language, active_version: &str) -> (String, String) {
        if let Some(ref tq) = self.tutorial_query {
            let q = tq.get(lang).trim();
            if !q.is_empty() {
                return build_youtube_tutorial_search(q, active_version);
            }
        }
        let fallback = match lang {
            Language::EnUs => format!("minecraft {} tutorial", self.title.get(lang)),
            _ => format!("tutorial {} minecraft", self.title.get(lang)),
        };
        build_youtube_tutorial_search(&fallback, active_version)
    }
}

impl DupeItem {
    pub fn resolved(&self, active_version: &str) -> Cow<'_, DupeItem> {
        let Some(ov) = self.version_overrides.get(active_version) else {
            return Cow::Borrowed(self);
        };
        let mut cloned = self.clone();
        if let Some(ref v) = ov.title {
            cloned.title = v.clone();
        }
        if let Some(ref v) = ov.status_key {
            cloned.status_key = v.clone();
        }
        if let Some(ref v) = ov.status {
            cloned.status = v.clone();
        }
        if let Some(ref v) = ov.how_it_works {
            cloned.how_it_works = v.clone();
        }
        Cow::Owned(cloned)
    }

    /// Builds a language-aware and active-version-aware `(query_text, youtube_search_url)` pair.
    pub fn tutorial_search(&self, lang: Language, active_version: &str) -> (String, String) {
        if let Some(ref tq) = self.tutorial_query {
            let q = tq.get(lang).trim();
            if !q.is_empty() {
                return build_youtube_tutorial_search(q, active_version);
            }
        }
        let fallback = match lang {
            Language::EnUs => format!("minecraft {} tutorial", self.title.get(lang)),
            _ => format!("tutorial {} minecraft", self.title.get(lang)),
        };
        build_youtube_tutorial_search(&fallback, active_version)
    }
}

fn is_mc_version_token(raw: &str) -> bool {
    let trimmed = raw.trim_matches(|c: char| matches!(c, '(' | ')' | '[' | ']' | ',' | ';'));
    let core = trimmed.trim_end_matches('+');
    let Some((major, rest)) = core.split_once('.') else {
        return false;
    };
    !major.is_empty()
        && major.chars().all(|c| c.is_ascii_digit())
        && !rest.is_empty()
        && rest.split('.').all(|seg| !seg.is_empty() && seg.chars().all(|c| c.is_ascii_digit()))
}

fn encode_query_component(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 16);
    for &b in input.as_bytes() {
        if b == b' ' {
            out.push('+');
        } else if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

/// Normalizes a tutorial search phrase by replacing any stale version numbers with `active_version`
/// and returning `(display_query, youtube_results_url)`.
pub fn build_youtube_tutorial_search(base_query: &str, active_version: &str) -> (String, String) {
    let ver = active_version.trim();
    let mut cleaned: Vec<&str> = Vec::new();
    let mut prev_dropped_ver = false;
    for tok in base_query.split_whitespace() {
        if is_mc_version_token(tok) {
            prev_dropped_ver = true;
            continue;
        }
        if prev_dropped_ver && tok == "/" {
            continue;
        }
        prev_dropped_ver = false;
        cleaned.push(tok);
    }
    let mut query = cleaned.join(" ");
    if !ver.is_empty() {
        if query.is_empty() {
            query = format!("minecraft {ver}");
        } else {
            query.push(' ');
            query.push_str(ver);
        }
    }
    let url = format!(
        "https://www.youtube.com/results?search_query={}",
        encode_query_component(&query)
    );
    (query, url)
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProTip {
    pub id: String,
    pub category_key: String,
    pub category: LocalizedText,
    pub icon: String,
    pub title: LocalizedText,
    pub text: LocalizedText,
    #[serde(default)]
    pub since_version: Option<String>,
    #[serde(default)]
    pub until_version: Option<String>,
    #[serde(default)]
    pub version_overrides: HashMap<String, ContentVersionOverride>,
}

impl ProTip {
    pub fn resolved(&self, active_version: &str) -> Cow<'_, ProTip> {
        let Some(ov) = self.version_overrides.get(active_version) else {
            return Cow::Borrowed(self);
        };
        let mut cloned = self.clone();
        if let Some(ref v) = ov.title {
            cloned.title = v.clone();
        }
        if let Some(ref v) = ov.text {
            cloned.text = v.clone();
        }
        Cow::Owned(cloned)
    }
}

/// Parses numeric quantity inside strings like `"128x"` or `"64"` and formats Shulker / Stack equivalents.
pub fn parse_material_qty(count_str: &str) -> Option<u32> {
    let digits: String = count_str
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse::<u32>().ok()
    }
}

/// Formats an item count into human stacks + shulker boxes (`64`-item stacks, `1728`-item shulkers).
pub fn format_stack_breakdown(total: u32, stack_size: u32, lang: Language) -> String {
    if total == 0 {
        return "0".to_string();
    }
    let stack = stack_size.max(1);
    let shulker_cap = stack * 27;
    if total >= shulker_cap {
        let sb = total / shulker_cap;
        let rem = total % shulker_cap;
        let st = rem / stack;
        let it = rem % stack;
        let mut parts = vec![format!("{} SB", sb)];
        if st > 0 {
            parts.push(format!("{}x{}", st, stack));
        }
        if it > 0 {
            parts.push(format!("+{}", it));
        }
        format!("{} ({})", total, parts.join(" "))
    } else if total > stack {
        let st = total / stack;
        let it = total % stack;
        let lbl = match lang {
            Language::EnUs => "st",
            _ => "st",
        };
        if it > 0 {
            format!("{} ({}{} + {})", total, st, lbl, it)
        } else {
            format!("{} ({}{})", total, st, lbl)
        }
    } else {
        format!("{}", total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::load_versions;

    #[test]
    fn versions_registry_loads_and_filters_items_cleanly() {
        let base = load_versions().expect("versions.json must load cleanly");
        assert_eq!(base.default_version, "26.3");
        assert!(base.find("26.3").is_some());

        // Simulate adding future release 26.4 in-memory to verify range & override behavior
        let sim = VersionRegistry {
            default_version: "26.4".to_string(),
            versions: vec![
                GameVersionDef {
                    id: "26.3".to_string(),
                    label: "Java 26.3".to_string(),
                    subtitle: LocalizedText::default(),
                    cubiomes_mc: "26.3".to_string(),
                    highlights: vec![],
                },
                GameVersionDef {
                    id: "26.4".to_string(),
                    label: "Java 26.4".to_string(),
                    subtitle: LocalizedText::default(),
                    cubiomes_mc: "26.4".to_string(),
                    highlights: vec![],
                },
            ],
        };

        assert!(sim.is_active(None, None, "26.3"));
        assert!(!sim.is_active(Some("26.4"), None, "26.3"));
        assert!(sim.is_active(Some("26.4"), None, "26.4"));
        assert!(sim.is_active(None, Some("26.3"), "26.3"));
        assert!(!sim.is_active(None, Some("26.3"), "26.4"));
    }

    #[test]
    fn database_farms_and_dupes_filter_keys_cover_all_ui_tabs() {
        use crate::data::loader::load_database;
        let db = load_database().expect("database.json must load cleanly");
        let versions = load_versions().expect("versions.json must load cleanly");
        let active_ver = &versions.default_version;

        for tier in ["easy", "medium", "hard"] {
            let count = db
                .farms
                .iter()
                .filter(|f| versions.is_active(f.since_version.as_deref(), f.until_version.as_deref(), active_ver))
                .filter(|f| f.resolved(active_ver).tier_key == tier)
                .count();
            assert!(
                count > 0,
                "Expected at least one farm in version {active_ver} for tier_key='{tier}'"
            );
        }

        let working_dupes = db
            .dupes
            .iter()
            .filter(|d| versions.is_active(d.since_version.as_deref(), d.until_version.as_deref(), active_ver))
            .filter(|d| d.resolved(active_ver).status_key == "working")
            .count();
        let patched_dupes = db
            .dupes
            .iter()
            .filter(|d| versions.is_active(d.since_version.as_deref(), d.until_version.as_deref(), active_ver))
            .filter(|d| {
                matches!(
                    d.resolved(active_ver).status_key.as_str(),
                    "patched" | "patched_workaround"
                )
            })
            .count();
        assert!(working_dupes > 0, "Expected at least one working dupe in {active_ver}");
        assert!(patched_dupes > 0, "Expected at least one patched dupe in {active_ver}");

        for region in ["es", "en", "zh"] {
            let count = db.creators.iter().filter(|c| c.region_key == region).count();
            assert!(count > 0, "Expected at least one creator in region '{region}'");
        }
        for focus in ["farms", "dupes"] {
            let count = db.creators.iter().filter(|c| c.has_focus(focus)).count();
            assert!(count >= 4, "Expected multiple creators for focus '{focus}', got {count}");
        }
        for c in &db.creators {
            assert!(!c.tags.is_empty(), "Creator {} missing specialty tags", c.id);
            assert!(!c.characteristics.is_empty(), "Creator {} missing characteristics", c.id);
        }
        for f in &db.farms {
            assert!(!f.icon.is_empty(), "Farm {} missing explicit icon", f.id);
            assert!(f.tutorial_query.is_some(), "Farm {} missing localized tutorial_query", f.id);
        }
        for d in &db.dupes {
            assert!(d.tutorial_query.is_some(), "Dupe {} missing localized tutorial_query", d.id);
        }
    }

    #[test]
    fn tutorial_search_adapts_version_and_encodes_safely() {
        let (q, url) = build_youtube_tutorial_search("minecraft day 1 iron farm tutorial 1.21+", "26.3");
        assert_eq!(q, "minecraft day 1 iron farm tutorial 26.3");
        assert_eq!(
            url,
            "https://www.youtube.com/results?search_query=minecraft+day+1+iron+farm+tutorial+26.3"
        );

        let (q2, url2) = build_youtube_tutorial_search("cañón orbital tnt minecraft 26.3", "26.4");
        assert_eq!(q2, "cañón orbital tnt minecraft 26.4");
        assert!(url2.contains("26.4"));
        assert!(!url2.contains(' '));
        assert!(!url2.contains("26.3"));

        let (q3, _) = build_youtube_tutorial_search("enlace de portales 1:8 nether minecraft", "26.3");
        assert_eq!(q3, "enlace de portales 1:8 nether minecraft 26.3");
    }
}
