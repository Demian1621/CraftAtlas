use crate::assets::ItemAtlas;
use crate::audio::AudioEngine;
use crate::data::models::{Database, VersionRegistry};
use crate::data::recipes::RecipeDatabase;
use crate::ui::theme::McPalette;

pub mod chrome;
pub mod right_rail;
pub mod tabs;
pub mod theme;
pub mod theme_modal;
pub mod updater_modal;

/// Borrowed immutable UI environment passed cleanly across top-level chrome and catalog tabs.
#[derive(Clone, Copy)]
pub struct RenderContext<'a> {
    pub versions: &'a VersionRegistry,
    pub db: &'a Database,
    pub recipes: &'a RecipeDatabase,
    pub palette: McPalette,
    pub atlas: &'a ItemAtlas,
    pub audio: &'a AudioEngine,
}
