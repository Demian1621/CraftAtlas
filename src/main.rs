#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod updater;

mod anim;
mod assets;
mod audio;
mod data;
mod i18n;
mod state;
mod ui;

use eframe::egui::{self, Margin};
use simple_easing::cubic_out;
use std::sync::Arc;
use std::time::Duration;

use anim::{ParticleSystem, ToastQueue};
use assets::backgrounds::ThemeBackgrounds;
use assets::{install_fonts, ItemAtlas};
use audio::AudioEngine;
use data::loader::{load_database, load_recipes, load_versions};
use data::models::{Database, VersionRegistry};
use data::recipes::RecipeDatabase;
use data::seed_engine::{parse_seed_input, SeedMapEngine};
use data::worlds_nbt::find_minecraft_sound_paths;
use state::{AppState, AppTab, UserPrefs};
use ui::chrome::{
    render_borderless_window_frame, render_favorites_rail, render_search_palette, render_top_chrome,
};
use ui::tabs;
use ui::theme::McPalette;
use ui::RenderContext;

const APP_ICON_PNG: &[u8] = include_bytes!("../assets/custom_app_icon.png");

struct CraftAtlasApp {
    versions: VersionRegistry,
    db: Database,
    recipes: RecipeDatabase,
    atlas: ItemAtlas,
    backgrounds: ThemeBackgrounds,
    state: AppState,
    seed_engine: SeedMapEngine,
    audio: AudioEngine,
    particles: ParticleSystem,
    toasts: ToastQueue,
    anim_time: f64,
}

impl CraftAtlasApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        let atlas = ItemAtlas::load(&cc.egui_ctx);

        let versions = load_versions().expect("embedded data/versions.json must load cleanly");
        let db = load_database().expect("embedded data/database.json must load cleanly");
        let recipes = load_recipes().expect("embedded data/recipes.json must load cleanly");

        let mut prefs = UserPrefs::load();
        prefs.selected_version = versions
            .resolve_valid_id(&prefs.selected_version)
            .to_string();
        let initial_seed = parse_seed_input(&prefs.last_seed);
        let seed_engine = SeedMapEngine::new(initial_seed, prefs.large_biomes);
        let sound_paths = find_minecraft_sound_paths();
        let audio = AudioEngine::new(prefs.audio_enabled, prefs.audio_volume, sound_paths.clone());

        let initial_gear = db.gear.first().map(|g| g.id.clone()).unwrap_or_default();
        let initial_farm = db.farms.first().map(|f| f.id.clone()).unwrap_or_default();
        let initial_dupe = db.dupes.first().map(|d| d.id.clone()).unwrap_or_default();
        let initial_tip = db.pro_tips.first().map(|p| p.id.clone()).unwrap_or_default();

        let state = AppState::new(
            prefs,
            initial_gear,
            initial_farm,
            initial_dupe,
            initial_tip,
        );

        let palette = McPalette::from_theme(state.prefs.theme);
        palette.apply_to_egui(&cc.egui_ctx);

        Self {
            versions,
            db,
            recipes,
            atlas,
            backgrounds: ThemeBackgrounds::new(),
            state,
            seed_engine,
            audio,
            particles: ParticleSystem::new(),
            toasts: ToastQueue::default(),
            anim_time: 0.0,
        }
    }
}

impl eframe::App for CraftAtlasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let active = ui::chrome::is_window_active(ctx);
        let raw_dt = ctx.input(|i| i.unstable_dt.clamp(0.001, 0.08));
        let dt = if active { raw_dt } else { 0.0 };
        if active {
            self.anim_time += raw_dt as f64;
        }
        let time = self.anim_time;

        self.audio.enabled = self.state.prefs.audio_enabled;
        self.audio.volume = self.state.prefs.audio_volume;
        self.state.updater.poll();

        let palette = McPalette::from_theme(self.state.prefs.theme);
        palette.apply_to_egui(ctx);

        let screen_rect = ctx.screen_rect();
        self.backgrounds.paint_viewport_backdrop(
            ctx,
            screen_rect,
            self.state.prefs.theme,
            palette,
            time,
        );
        self.particles.update_and_paint_ambient(
            ctx,
            screen_rect,
            self.state.prefs.theme,
            time,
            dt,
        );

        if !active {
            // Pause background panorama/particles cleanly when unfocused or minimized (~0% idle CPU).
            if matches!(self.state.updater.status(), updater::UpdateStatus::Checking) {
                ctx.request_repaint_after(Duration::from_millis(500));
            }
        } else if self.state.tab_anim_t < 1.0 {
            self.state.tab_anim_t = (self.state.tab_anim_t + dt * 4.5).min(1.0);
            ctx.request_repaint();
        } else {
            // Smooth 30 FPS scenic panorama / ambient particle refresh while window is active
            ctx.request_repaint_after(Duration::from_millis(33));
        }

        let rcx = RenderContext {
            versions: &self.versions,
            db: &self.db,
            recipes: &self.recipes,
            palette,
            atlas: &self.atlas,
            audio: &self.audio,
        };

        render_top_chrome(
            ctx,
            &mut self.state,
            &self.versions,
            palette,
            &self.atlas,
            &self.audio,
        );

        render_favorites_rail(ctx, &mut self.state, &rcx);

        let anim_offset = (1.0 - cubic_out(self.state.tab_anim_t)) * 10.0;

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(palette.bg_canvas)
                    .inner_margin(Margin {
                        left: 12,
                        right: 12,
                        top: 10 + anim_offset.round() as i8,
                        bottom: 10,
                    }),
            )
            .show(ctx, |ui| {
                match self.state.active_tab {
                    AppTab::Gear => tabs::gear::render(
                        ui,
                        &mut self.state,
                        &rcx,
                        &mut self.particles,
                        &mut self.toasts,
                        dt,
                    ),
                    AppTab::Recipes => {
                        tabs::recipes::render(ui, &mut self.state, &rcx, &mut self.toasts)
                    }
                    AppTab::SeedMap => tabs::seed_map::render(
                        ui,
                        &mut self.state,
                        &mut self.seed_engine,
                        &rcx,
                        &mut self.toasts,
                        dt,
                    ),
                    AppTab::Farms => {
                        tabs::farms::render(ui, &mut self.state, &rcx, &mut self.toasts)
                    }
                    AppTab::Dupes => {
                        tabs::dupes::render(ui, &mut self.state, &rcx, &mut self.toasts)
                    }
                    AppTab::ProTips => tabs::pro_tips::render(
                        ui,
                        &mut self.state,
                        &self.db,
                        &self.versions,
                        palette,
                        &self.atlas,
                        &self.audio,
                    ),
                }
            });

        render_search_palette(ctx, &mut self.state, &rcx);
        self.particles.update_and_paint(ctx, dt);
        self.toasts.update_and_paint(ctx, &self.atlas, dt);
        render_borderless_window_frame(ctx, palette);
    }
}

fn load_app_icon() -> Option<Arc<egui::IconData>> {
    let img = image::load_from_memory_with_format(APP_ICON_PNG, image::ImageFormat::Png)
        .ok()?
        .to_rgba8();
    Some(Arc::new(egui::IconData {
        rgba: img.as_raw().to_vec(),
        width: img.width(),
        height: img.height(),
    }))
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("CraftAtlas")
        .with_app_id("craft-atlas")
        .with_decorations(false)
        .with_inner_size([1320.0, 840.0])
        .with_min_inner_size([760.0, 520.0]);
    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "CraftAtlas",
        options,
        Box::new(|cc| Ok(Box::new(CraftAtlasApp::new(cc)))),
    )
}

#[cfg(test)]
mod tests {
    use super::load_app_icon;

    #[test]
    fn app_window_icon_decodes_at_256x256_rgba() {
        let icon = load_app_icon().expect("embedded custom_app_icon.png should decode");
        assert_eq!(icon.width, 256);
        assert_eq!(icon.height, 256);
        assert_eq!(icon.rgba.len(), 256 * 256 * 4);
    }
}
