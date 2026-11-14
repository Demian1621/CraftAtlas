use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, TextureHandle, TextureOptions};
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use super::models::Language;

pub const TILE_PX: usize = 128;

/// Dimensions supported natively by Cubiomes & Seed Map Explorer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dimension {
    Overworld = 0,
    Nether = -1,
    TheEnd = 1,
}

impl Dimension {
    pub fn label(self, lang: Language) -> &'static str {
        match (self, lang) {
            (Dimension::Overworld, Language::EnUs) => "Overworld",
            (Dimension::Overworld, _) => "Overworld",
            (Dimension::Nether, _) => "Nether",
            (Dimension::TheEnd, Language::EnUs) => "The End",
            (Dimension::TheEnd, _) => "El End",
        }
    }

    pub fn icon_id(self) -> &'static str {
        match self {
            Dimension::Overworld => "grass_block",
            Dimension::Nether => "netherrack",
            Dimension::TheEnd => "end_stone",
        }
    }

    pub fn all() -> [Dimension; 3] {
        [Dimension::Overworld, Dimension::Nether, Dimension::TheEnd]
    }
}

/// Tile visual rendering styles inspired by Chunkbase & Minecraft's Filled Map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapStyle {
    Topo3D = 1,
    McMap = 2,
    Flat = 0,
}

impl MapStyle {
    pub fn label(self, lang: Language) -> &'static str {
        match (self, lang) {
            (MapStyle::Topo3D, Language::EnUs) => "3D Relief",
            (MapStyle::Topo3D, _) => "Relieve 3D",
            (MapStyle::McMap, Language::EnUs) => "MC Map",
            (MapStyle::McMap, _) => "Mapa MC",
            (MapStyle::Flat, Language::EnUs) => "Flat Biomes",
            (MapStyle::Flat, _) => "Biomas Planos",
        }
    }

    pub fn all() -> [MapStyle; 3] {
        [MapStyle::Topo3D, MapStyle::McMap, MapStyle::Flat]
    }
}

/// Every structure layer toggleable across dimensions, with crisp pixel-art atlas sprites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum StructureKind {
    SpawnPoint,
    SlimeChunk,
    Village,
    TrialChambers,
    AncientCity,
    Stronghold,
    Monument,
    Mansion,
    Outpost,
    DesertPyramid,
    JungleTemple,
    SwampHut,
    Igloo,
    TrailRuins,
    RuinedPortal,
    Fortress,
    Bastion,
    EndCity,
    EndShip,
    EndGateway,
}

impl StructureKind {
    pub fn icon_id(self) -> &'static str {
        match self {
            StructureKind::SpawnPoint => "compass",
            StructureKind::SlimeChunk => "slime_ball",
            StructureKind::Village => "bell",
            StructureKind::TrialChambers => "ominous_trial_key",
            StructureKind::AncientCity => "warden_head",
            StructureKind::Stronghold => "ender_eye",
            StructureKind::Monument => "prismarine_shard",
            StructureKind::Mansion => "totem_of_undying",
            StructureKind::Outpost => "crossbow",
            StructureKind::DesertPyramid => "chiseled_sandstone",
            StructureKind::JungleTemple => "mossy_cobblestone",
            StructureKind::SwampHut => "cauldron",
            StructureKind::Igloo => "packed_ice",
            StructureKind::TrailRuins => "brush",
            StructureKind::RuinedPortal => "crying_obsidian",
            StructureKind::Fortress => "blaze_rod",
            StructureKind::Bastion => "gilded_blackstone",
            StructureKind::EndCity => "shulker_shell",
            StructureKind::EndShip => "elytra",
            StructureKind::EndGateway => "end_crystal",
        }
    }

    pub fn accent_color(self) -> Color32 {
        match self {
            StructureKind::SpawnPoint => Color32::from_rgb(90, 235, 160),
            StructureKind::SlimeChunk => Color32::from_rgb(106, 238, 92),
            StructureKind::Village => Color32::from_rgb(245, 200, 72),
            StructureKind::TrialChambers => Color32::from_rgb(232, 128, 60),
            StructureKind::AncientCity => Color32::from_rgb(54, 214, 218),
            StructureKind::Stronghold => Color32::from_rgb(138, 255, 224),
            StructureKind::Monument => Color32::from_rgb(80, 210, 220),
            StructureKind::Mansion => Color32::from_rgb(168, 122, 74),
            StructureKind::Outpost => Color32::from_rgb(188, 178, 160),
            StructureKind::DesertPyramid => Color32::from_rgb(235, 205, 118),
            StructureKind::JungleTemple => Color32::from_rgb(92, 188, 96),
            StructureKind::SwampHut => Color32::from_rgb(186, 118, 240),
            StructureKind::Igloo => Color32::from_rgb(180, 235, 255),
            StructureKind::TrailRuins => Color32::from_rgb(206, 148, 96),
            StructureKind::RuinedPortal => Color32::from_rgb(198, 92, 244),
            StructureKind::Fortress => Color32::from_rgb(220, 74, 74),
            StructureKind::Bastion => Color32::from_rgb(246, 186, 56),
            StructureKind::EndCity => Color32::from_rgb(228, 164, 248),
            StructureKind::EndShip => Color32::from_rgb(112, 244, 255),
            StructureKind::EndGateway => Color32::from_rgb(166, 124, 255),
        }
    }

    pub fn label(self, lang: Language) -> &'static str {
        match (self, lang) {
            (StructureKind::SpawnPoint, Language::EnUs) => "World Spawn",
            (StructureKind::SpawnPoint, _) => "Punto de Spawn",
            (StructureKind::SlimeChunk, Language::EnUs) => "Slime Chunks",
            (StructureKind::SlimeChunk, _) => "Slime Chunks",
            (StructureKind::Village, Language::EnUs) => "Village",
            (StructureKind::Village, _) => "Aldea",
            (StructureKind::TrialChambers, Language::EnUs) => "Trial Chambers",
            (StructureKind::TrialChambers, Language::EsMx) => "Cámaras de Desafío",
            (StructureKind::TrialChambers, Language::EsEs) => "Cámaras de Juicio",
            (StructureKind::AncientCity, Language::EnUs) => "Ancient City",
            (StructureKind::AncientCity, _) => "Ciudad Antigua",
            (StructureKind::Stronghold, Language::EnUs) => "Stronghold",
            (StructureKind::Stronghold, _) => "Fortaleza del End",
            (StructureKind::Monument, Language::EnUs) => "Ocean Monument",
            (StructureKind::Monument, _) => "Monumento Oceánico",
            (StructureKind::Mansion, Language::EnUs) => "Woodland Mansion",
            (StructureKind::Mansion, _) => "Mansión del Bosque",
            (StructureKind::Outpost, Language::EnUs) => "Pillager Outpost",
            (StructureKind::Outpost, Language::EsMx) => "Puesto de Saqueadores",
            (StructureKind::Outpost, Language::EsEs) => "Puesto de Saqueadores",
            (StructureKind::DesertPyramid, Language::EnUs) => "Desert Pyramid",
            (StructureKind::DesertPyramid, _) => "Pirámide del Desierto",
            (StructureKind::JungleTemple, Language::EnUs) => "Jungle Temple",
            (StructureKind::JungleTemple, _) => "Templo de Jungla",
            (StructureKind::SwampHut, Language::EnUs) => "Witch Hut",
            (StructureKind::SwampHut, Language::EsMx) => "Cabaña de Bruja",
            (StructureKind::SwampHut, Language::EsEs) => "Choza de Bruja",
            (StructureKind::Igloo, Language::EnUs) => "Igloo",
            (StructureKind::Igloo, _) => "Iglú",
            (StructureKind::TrailRuins, Language::EnUs) => "Trail Ruins",
            (StructureKind::TrailRuins, _) => "Ruinas de Sendero",
            (StructureKind::RuinedPortal, Language::EnUs) => "Ruined Portal",
            (StructureKind::RuinedPortal, _) => "Portal en Ruinas",
            (StructureKind::Fortress, Language::EnUs) => "Nether Fortress",
            (StructureKind::Fortress, _) => "Fortaleza del Nether",
            (StructureKind::Bastion, Language::EnUs) => "Bastion Remnant",
            (StructureKind::Bastion, _) => "Bastión en Ruinas",
            (StructureKind::EndCity, Language::EnUs) => "End City",
            (StructureKind::EndCity, _) => "Ciudad del End",
            (StructureKind::EndShip, Language::EnUs) => "End City + Elytra Ship",
            (StructureKind::EndShip, Language::EsMx) => "Ciudad del End + Barco Elytra",
            (StructureKind::EndShip, Language::EsEs) => "Ciudad del End + Barco Élitros",
            (StructureKind::EndGateway, Language::EnUs) => "End Gateway Portal",
            (StructureKind::EndGateway, _) => "Portal Gateway del End",
        }
    }

    pub fn min_zoom_visible(self) -> f32 {
        match self {
            StructureKind::SpawnPoint | StructureKind::Stronghold | StructureKind::EndGateway => 0.01,
            StructureKind::Mansion => 0.02,
            StructureKind::Monument
            | StructureKind::AncientCity
            | StructureKind::TrialChambers
            | StructureKind::EndShip => 0.05,
            StructureKind::Fortress
            | StructureKind::Bastion
            | StructureKind::EndCity
            | StructureKind::Village => 0.11,
            _ => 0.18,
        }
    }
}

/// UI Toggle bitflags corresponding to C bridge masks.
#[derive(Debug, Clone, Copy)]
pub struct LayerToggles {
    pub slime_chunks: bool,
    pub biomes: bool,
    pub villages: bool,
    pub trial_chambers: bool,
    pub ancient_cities: bool,
    pub strongholds: bool,
    pub monuments: bool,
    pub mansions: bool,
    pub outposts: bool,
    pub temples: bool, // Desert, Jungle, Swamp Hut, Igloo, Trail Ruins
    pub ruined_portals: bool,
    pub fortresses: bool,
    pub bastions: bool,
    pub end_cities: bool,
    pub end_gateways: bool,
    pub grid_chunks: bool,
    pub waypoints: bool,
    pub player_marker: bool,
}

impl Default for LayerToggles {
    fn default() -> Self {
        Self {
            slime_chunks: false,
            biomes: true,
            villages: true,
            trial_chambers: true,
            ancient_cities: true,
            strongholds: true,
            monuments: true,
            mansions: true,
            outposts: false,
            temples: false,
            ruined_portals: false,
            fortresses: true,
            bastions: true,
            end_cities: true,
            end_gateways: true,
            grid_chunks: true,
            waypoints: true,
            player_marker: true,
        }
    }
}

impl LayerToggles {
    pub fn to_c_mask(self, dim: Dimension) -> u32 {
        let mut m = 0u32;
        match dim {
            Dimension::Overworld => {
                if self.villages { m |= 1 << 0; }
                if self.temples {
                    m |= (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 10);
                }
                if self.monuments { m |= 1 << 5; }
                if self.mansions { m |= 1 << 6; }
                if self.outposts { m |= 1 << 7; }
                if self.ruined_portals { m |= 1 << 8; }
                if self.ancient_cities { m |= 1 << 9; }
                if self.trial_chambers { m |= 1 << 11; }
            }
            Dimension::Nether => {
                if self.fortresses { m |= 1 << 12; }
                if self.bastions { m |= 1 << 13; }
                if self.ruined_portals { m |= 1 << 8; }
            }
            Dimension::TheEnd => {
                if self.end_cities { m |= 1 << 14; }
                if self.end_gateways { m |= 1 << 15; }
            }
        }
        m
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StructureMarker {
    pub kind: StructureKind,
    pub x: i32,
    pub z: i32,
    pub variant: i32,
    pub biome_id: i32,
}

#[repr(C)]
struct McWorldHandleOpaque {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct CMcStructureHit {
    stype: i32,
    x: i32,
    z: i32,
    variant: i32,
    biome: i32,
}

extern "C" {
    fn mc_world_new(seed: u64, large_biomes: i32) -> *mut McWorldHandleOpaque;
    fn mc_world_free(h: *mut McWorldHandleOpaque);
    fn mc_render_tile(
        h: *const McWorldHandleOpaque,
        dim: i32,
        scale: i32,
        block_x0: i32,
        block_z0: i32,
        w: i32,
        h_px: i32,
        map_style: i32,
        out_rgba: *mut u8,
    );
    fn mc_sample_point(
        h: *const McWorldHandleOpaque,
        dim: i32,
        block_x: i32,
        block_z: i32,
        out_height: *mut f32,
        out_name: *mut c_char,
        name_cap: i32,
    ) -> i32;
    fn mc_get_spawn(h: *mut McWorldHandleOpaque, out_x: *mut i32, out_z: *mut i32);
    fn mc_get_strongholds(
        h: *mut McWorldHandleOpaque,
        out: *mut CMcStructureHit,
        max_out: i32,
    ) -> i32;
    fn mc_get_end_gateways(
        h: *mut McWorldHandleOpaque,
        out_inner_20: *mut CMcStructureHit,
        out_linked_20: *mut CMcStructureHit,
    ) -> i32;
    fn mc_scan_structures(
        h: *mut McWorldHandleOpaque,
        dim: i32,
        min_bx: i32,
        min_bz: i32,
        max_bx: i32,
        max_bz: i32,
        mask: u32,
        out: *mut CMcStructureHit,
        max_out: i32,
    ) -> i32;
}

struct OwnedWorld {
    ptr: *mut McWorldHandleOpaque,
}

unsafe impl Send for OwnedWorld {}

impl OwnedWorld {
    fn new(seed: i64, large_biomes: bool) -> Self {
        let ptr = unsafe { mc_world_new(seed as u64, if large_biomes { 1 } else { 0 }) };
        assert!(!ptr.is_null(), "mc_world_new returned null");
        Self { ptr }
    }
}

impl Drop for OwnedWorld {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { mc_world_free(self.ptr) };
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileKey {
    pub seed: i64,
    pub large_biomes: bool,
    pub dim: Dimension,
    pub scale: i32,
    pub style: MapStyle,
    pub tx: i32,
    pub tz: i32,
}

struct TileJob {
    key: TileKey,
}

struct TileQueueState {
    jobs: VecDeque<TileJob>,
    shutdown: bool,
}

struct TileResult {
    key: TileKey,
    rgba: Vec<u8>,
}

pub struct CachedTile {
    pub tex: TextureHandle,
    pub last_used_frame: u64,
}

/// Interactive Cubiomes Engine owning multi-threaded tile workers, cached GPU tiles, spawn & stronghold state.
pub struct SeedMapEngine {
    pub seed: i64,
    pub large_biomes: bool,
    world: OwnedWorld,
    spawn_pos: (i32, i32),
    strongholds: Vec<StructureMarker>,
    end_gateways: Vec<StructureMarker>,

    // Structure area cache key
    last_scan_key: Option<(i64, Dimension, u32, i32, i32, i32, i32)>,
    cached_structures: Vec<StructureMarker>,

    // Multi-threaded tile rasterizer + GPU texture cache
    job_queue: Arc<(Mutex<TileQueueState>, Condvar)>,
    res_rx: Receiver<TileResult>,
    in_flight: HashSet<TileKey>,
    tiles: HashMap<TileKey, CachedTile>,
    frame_counter: u64,
}

impl Drop for SeedMapEngine {
    fn drop(&mut self) {
        let (lock, cvar) = &*self.job_queue;
        if let Ok(mut guard) = lock.lock() {
            guard.jobs.clear();
            guard.shutdown = true;
            cvar.notify_all();
        }
    }
}

impl SeedMapEngine {
    pub fn new(initial_seed: i64, large_biomes: bool) -> Self {
        let (res_tx, res_rx) = mpsc::channel::<TileResult>();
        let job_queue = Arc::new((
            Mutex::new(TileQueueState {
                jobs: VecDeque::with_capacity(128),
                shutdown: false,
            }),
            Condvar::new(),
        ));

        let worker_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(2)
            .clamp(2, 6);
        for idx in 0..worker_count {
            let queue = Arc::clone(&job_queue);
            let tx = res_tx.clone();
            thread::Builder::new()
                .name(format!("cubiomes-tile-worker-{}", idx))
                .spawn(move || tile_worker_loop(queue, tx))
                .expect("spawn tile worker thread");
        }

        let mut world = OwnedWorld::new(initial_seed, large_biomes);
        let (spawn_pos, strongholds, end_gateways) = compute_global_markers(&mut world);

        Self {
            seed: initial_seed,
            large_biomes,
            world,
            spawn_pos,
            strongholds,
            end_gateways,
            last_scan_key: None,
            cached_structures: Vec::new(),
            job_queue,
            res_rx,
            in_flight: HashSet::new(),
            tiles: HashMap::new(),
            frame_counter: 0,
        }
    }

    pub fn set_seed(&mut self, new_seed: i64, large_biomes: bool) {
        if self.seed == new_seed && self.large_biomes == large_biomes {
            return;
        }
        self.seed = new_seed;
        self.large_biomes = large_biomes;
        self.world = OwnedWorld::new(new_seed, large_biomes);
        let (sp, sh, gw) = compute_global_markers(&mut self.world);
        self.spawn_pos = sp;
        self.strongholds = sh;
        self.end_gateways = gw;
        self.last_scan_key = None;
        self.cached_structures.clear();
        if let Ok(mut guard) = self.job_queue.0.lock() {
            guard.jobs.clear();
        }
        self.in_flight.clear();
        self.tiles.clear();
    }

    pub fn spawn_point(&self) -> (i32, i32) {
        self.spawn_pos
    }

    pub fn strongholds(&self) -> &[StructureMarker] {
        &self.strongholds
    }

    #[allow(dead_code)]
    pub fn end_gateways(&self) -> &[StructureMarker] {
        &self.end_gateways
    }

    /// Drains finished tiles from the background workers into `egui` GPU textures. Returns true if new tiles uploaded.
    pub fn poll_tiles(&mut self, ctx: &egui::Context) -> bool {
        self.frame_counter += 1;
        let mut uploaded = false;
        while let Ok(res) = self.res_rx.try_recv() {
            self.in_flight.remove(&res.key);
            if res.key.seed != self.seed || res.key.large_biomes != self.large_biomes {
                continue;
            }
            let image = ColorImage::from_rgba_unmultiplied([TILE_PX, TILE_PX], &res.rgba);
            let tex_name = format!(
                "mc_tile_{}_{}_{}_{}_{}_{}",
                res.key.seed, res.key.dim as i32, res.key.scale, res.key.style as i32, res.key.tx, res.key.tz
            );
            let tex = ctx.load_texture(tex_name, image, TextureOptions::NEAREST);
            self.tiles.insert(
                res.key,
                CachedTile {
                    tex,
                    last_used_frame: self.frame_counter,
                },
            );
            uploaded = true;
        }

        // Trim LRU cache gracefully beyond 900 tiles
        if self.tiles.len() > 900 {
            let cutoff = self.frame_counter.saturating_sub(180);
            self.tiles.retain(|_, v| v.last_used_frame >= cutoff);
            if self.tiles.len() > 900 {
                let mut entries: Vec<(TileKey, u64)> = self
                    .tiles
                    .iter()
                    .map(|(k, v)| (*k, v.last_used_frame))
                    .collect();
                entries.sort_by_key(|&(_, f)| f);
                for (k, _) in entries.into_iter().take(self.tiles.len() - 750) {
                    self.tiles.remove(&k);
                }
            }
        }
        uploaded
    }

    /// Requests `(tx, tz)` if not cached, returning either the exact tile UV=(0..1) or a cached
    /// coarser-scale LOD tile with its exact sub-UV crop so zooming never flashes blank.
    pub fn lookup_tile_or_fallback(
        &mut self,
        dim: Dimension,
        scale: i32,
        style: MapStyle,
        tx: i32,
        tz: i32,
    ) -> Option<(&TextureHandle, Rect)> {
        let key = TileKey {
            seed: self.seed,
            large_biomes: self.large_biomes,
            dim,
            scale,
            style,
            tx,
            tz,
        };
        if let Some(cached) = self.tiles.get_mut(&key) {
            cached.last_used_frame = self.frame_counter;
        } else if self.in_flight.len() < 160 && self.in_flight.insert(key) {
            let (lock, cvar) = &*self.job_queue;
            if let Ok(mut guard) = lock.lock() {
                guard.jobs.push_back(TileJob { key });
                cvar.notify_one();
            } else {
                self.in_flight.remove(&key);
            }
        }

        if let Some(cached) = self.tiles.get(&key) {
            return Some((
                &cached.tex,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            ));
        }
        self.find_fallback_tile(dim, scale, style, tx, tz)
    }

    /// Finds an already-cached coarser scale tile covering `(tx, tz)` and its exact UV sub-rect
    /// so zooming/panning never flashes blank background while native detail tiles rasterize.
    pub fn find_fallback_tile(
        &self,
        dim: Dimension,
        scale: i32,
        style: MapStyle,
        tx: i32,
        tz: i32,
    ) -> Option<(&TextureHandle, Rect)> {
        for &c_scale in &[4, 16, 64, 256] {
            if c_scale <= scale {
                continue;
            }
            let factor = c_scale / scale;
            if factor <= 1 {
                continue;
            }
            let c_tx = tx.div_euclid(factor);
            let c_tz = tz.div_euclid(factor);
            let key = TileKey {
                seed: self.seed,
                large_biomes: self.large_biomes,
                dim,
                scale: c_scale,
                style,
                tx: c_tx,
                tz: c_tz,
            };
            if let Some(cached) = self.tiles.get(&key) {
                let ox = tx.rem_euclid(factor) as f32;
                let oz = tz.rem_euclid(factor) as f32;
                let inv = 1.0 / (factor as f32);
                let uv = Rect::from_min_max(
                    Pos2::new(ox * inv, oz * inv),
                    Pos2::new((ox + 1.0) * inv, (oz + 1.0) * inv),
                );
                return Some((&cached.tex, uv));
            }
        }
        None
    }

    pub fn has_in_flight(&self) -> bool {
        !self.in_flight.is_empty()
    }

    /// Samples exact biome ID, approximate height, and readable biome name at block coordinates.
    pub fn sample_point(&self, dim: Dimension, bx: i32, bz: i32, lang: Language) -> (i32, f32, String) {
        let mut h = 64.0f32;
        let mut buf = [0 as c_char; 64];
        let id = unsafe {
            mc_sample_point(
                self.world.ptr,
                dim as i32,
                bx,
                bz,
                &mut h,
                buf.as_mut_ptr(),
                buf.len() as i32,
            )
        };
        let raw_name = unsafe { CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned() };
        (id, h, localized_biome_name(&raw_name, lang))
    }

    /// Scans region structures in expanded block bounds, memoizing across unchanged pan frames.
    pub fn structures_in_view(
        &mut self,
        dim: Dimension,
        min_bx: i32,
        min_bz: i32,
        max_bx: i32,
        max_bz: i32,
        toggles: LayerToggles,
    ) -> &[StructureMarker] {
        let mask = toggles.to_c_mask(dim);
        // Quantize scan box to 512-block buckets so micro-panning doesn't re-scan every pixel move
        let qmin_x = min_bx.div_euclid(512) * 512 - 512;
        let qmin_z = min_bz.div_euclid(512) * 512 - 512;
        let qmax_x = max_bx.div_euclid(512) * 512 + 1024;
        let qmax_z = max_bz.div_euclid(512) * 512 + 1024;

        let scan_key = (self.seed, dim, mask, qmin_x, qmin_z, qmax_x, qmax_z);
        if self.last_scan_key == Some(scan_key) {
            return &self.cached_structures;
        }

        self.cached_structures.clear();

        if mask != 0 {
            let mut hits = vec![CMcStructureHit::default(); 4096];
            let found = unsafe {
                mc_scan_structures(
                    self.world.ptr,
                    dim as i32,
                    qmin_x,
                    qmin_z,
                    qmax_x,
                    qmax_z,
                    mask,
                    hits.as_mut_ptr(),
                    hits.len() as i32,
                )
            };
            for h in &hits[..(found.max(0) as usize)] {
                if let Some(kind) = map_c_struct_type(h.stype, h.variant) {
                    self.cached_structures.push(StructureMarker {
                        kind,
                        x: h.x,
                        z: h.z,
                        variant: h.variant,
                        biome_id: h.biome,
                    });
                }
            }
        }

        // Global dimension-specific markers inside bounds
        match dim {
            Dimension::Overworld => {
                if toggles.strongholds {
                    for s in &self.strongholds {
                        if s.x >= qmin_x && s.x <= qmax_x && s.z >= qmin_z && s.z <= qmax_z {
                            self.cached_structures.push(s.clone());
                        }
                    }
                }
            }
            Dimension::TheEnd => {
                if toggles.end_gateways {
                    for gw in &self.end_gateways {
                        if gw.x >= qmin_x && gw.x <= qmax_x && gw.z >= qmin_z && gw.z <= qmax_z {
                            self.cached_structures.push(gw.clone());
                        }
                    }
                }
            }
            Dimension::Nether => {}
        }

        self.last_scan_key = Some(scan_key);
        &self.cached_structures
    }

    /// Exports a crisp high-res PNG map screenshot centered around `(center_x, center_z)` at `scale`.
    pub fn export_map_png(
        &self,
        path: &std::path::Path,
        dim: Dimension,
        style: MapStyle,
        center_x: i32,
        center_z: i32,
        scale: i32,
    ) -> Result<(), String> {
        let w = 1024i32;
        let h_px = 1024i32;
        let x0 = center_x - (w * scale) / 2;
        let z0 = center_z - (h_px * scale) / 2;
        let mut rgba = vec![0u8; (w * h_px * 4) as usize];
        unsafe {
            mc_render_tile(
                self.world.ptr,
                dim as i32,
                scale,
                x0,
                z0,
                w,
                h_px,
                style as i32,
                rgba.as_mut_ptr(),
            );
        }
        let img = image::RgbaImage::from_raw(w as u32, h_px as u32, rgba)
            .ok_or_else(|| "Failed to allocate RGBA export buffer".to_string())?;
        img.save(path).map_err(|e| format!("PNG save error: {}", e))
    }
}

fn tile_worker_loop(queue: Arc<(Mutex<TileQueueState>, Condvar)>, tx: Sender<TileResult>) {
    let mut cached_world: Option<(i64, bool, OwnedWorld)> = None;
    let mut rgba_buf = vec![0u8; TILE_PX * TILE_PX * 4];
    let (lock, cvar) = &*queue;

    loop {
        let job = {
            let mut guard = match lock.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            while guard.jobs.is_empty() && !guard.shutdown {
                guard = match cvar.wait(guard) {
                    Ok(g) => g,
                    Err(_) => return,
                };
            }
            if guard.shutdown && guard.jobs.is_empty() {
                return;
            }
            match guard.jobs.pop_front() {
                Some(j) => j,
                None => continue,
            }
        };

        let need_reinit = match &cached_world {
            Some((s, lb, _)) => *s != job.key.seed || *lb != job.key.large_biomes,
            None => true,
        };
        if need_reinit {
            cached_world = Some((
                job.key.seed,
                job.key.large_biomes,
                OwnedWorld::new(job.key.seed, job.key.large_biomes),
            ));
        }
        if let Some((_, _, ref world)) = cached_world {
            let bx0 = job.key.tx.saturating_mul((TILE_PX as i32) * job.key.scale);
            let bz0 = job.key.tz.saturating_mul((TILE_PX as i32) * job.key.scale);
            unsafe {
                mc_render_tile(
                    world.ptr,
                    job.key.dim as i32,
                    job.key.scale,
                    bx0,
                    bz0,
                    TILE_PX as i32,
                    TILE_PX as i32,
                    job.key.style as i32,
                    rgba_buf.as_mut_ptr(),
                );
            }
            if tx
                .send(TileResult {
                    key: job.key,
                    rgba: rgba_buf.clone(),
                })
                .is_err()
            {
                return;
            }
        }
    }
}

fn compute_global_markers(world: &mut OwnedWorld) -> ((i32, i32), Vec<StructureMarker>, Vec<StructureMarker>) {
    let mut sx = 0;
    let mut sz = 0;
    unsafe { mc_get_spawn(world.ptr, &mut sx, &mut sz) };

    let mut sh_raw = vec![CMcStructureHit::default(); 128];
    let n_sh = unsafe { mc_get_strongholds(world.ptr, sh_raw.as_mut_ptr(), 128) };
    let strongholds = sh_raw[..(n_sh.max(0) as usize)]
        .iter()
        .map(|h| StructureMarker {
            kind: StructureKind::Stronghold,
            x: h.x,
            z: h.z,
            variant: h.variant,
            biome_id: h.biome,
        })
        .collect();

    let mut gw_in = vec![CMcStructureHit::default(); 20];
    let mut gw_out = vec![CMcStructureHit::default(); 20];
    unsafe { mc_get_end_gateways(world.ptr, gw_in.as_mut_ptr(), gw_out.as_mut_ptr()) };
    let mut end_gateways = Vec::with_capacity(40);
    for h in gw_in.iter().chain(gw_out.iter()) {
        end_gateways.push(StructureMarker {
            kind: StructureKind::EndGateway,
            x: h.x,
            z: h.z,
            variant: h.variant,
            biome_id: h.biome,
        });
    }

    ((sx, sz), strongholds, end_gateways)
}

fn map_c_struct_type(stype: i32, variant: i32) -> Option<StructureKind> {
    // Matches enum StructureType in third_party/cubiomes/finders.h verbatim
    match stype {
        1 => Some(StructureKind::DesertPyramid),
        2 => Some(StructureKind::JungleTemple),
        3 => Some(StructureKind::SwampHut),
        4 => Some(StructureKind::Igloo),
        5 => Some(StructureKind::Village),
        8 => Some(StructureKind::Monument),
        9 => Some(StructureKind::Mansion),
        10 => Some(StructureKind::Outpost),
        11 | 12 => Some(StructureKind::RuinedPortal),
        13 => Some(StructureKind::AncientCity),
        18 => Some(StructureKind::Fortress),
        19 => Some(StructureKind::Bastion),
        20 => Some(if (variant & 1) != 0 {
            StructureKind::EndShip
        } else {
            StructureKind::EndCity
        }),
        21 => Some(StructureKind::EndGateway),
        23 => Some(StructureKind::TrailRuins),
        24 => Some(StructureKind::TrialChambers),
        _ => None,
    }
}

/// Official Minecraft Java Slime Chunk formula matching C `isSlimeChunk`.
pub fn is_slime_chunk(seed: i64, chunk_x: i32, chunk_z: i32) -> bool {
    let cx = chunk_x as i64;
    let cz = chunk_z as i64;
    let a = (cx.wrapping_mul(cx).wrapping_mul(0x4c1906)) as i32 as i64;
    let b = (cx.wrapping_mul(0x5ac0db)) as i32 as i64;
    let c = (cz.wrapping_mul(cz)) as i32 as i64;
    let c_mul = c.wrapping_mul(0x4307a7);
    let d = (cz.wrapping_mul(0x5f24f)) as i32 as i64;
    let raw = seed
        .wrapping_add(a)
        .wrapping_add(b)
        .wrapping_add(c_mul)
        .wrapping_add(d)
        ^ 0x3ad8025f;

    let mut lcg = ((raw as u64) ^ 0x5deece66d) & ((1u64 << 48) - 1);
    lcg = (lcg.wrapping_mul(0x5deece66d).wrapping_add(0xb)) & ((1u64 << 48) - 1);
    let bits = (lcg >> 17) as i32;
    let val = bits.rem_euclid(10);
    val == 0
}

/// Parses numeric i64 seed OR computes official Java `String.hashCode()` for text seeds.
pub fn parse_seed_input(raw: &str) -> i64 {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return 0;
    }
    if let Ok(num) = trimmed.parse::<i64>() {
        return num;
    }
    // Java String.hashCode(): 32-bit rolling hash signed-extended to i64
    let mut hash: i32 = 0;
    for unit in trimmed.encode_utf16() {
        hash = hash.wrapping_mul(31).wrapping_add(unit as i32);
    }
    hash as i64
}

pub fn localized_biome_name(id_str: &str, lang: Language) -> String {
    if lang == Language::EnUs {
        return title_case_snake(id_str);
    }
    let tr = match id_str {
        "plains" => "Llanura",
        "sunflower_plains" => "Llanura de girasoles",
        "forest" => "Bosque",
        "flower_forest" => "Bosque de flores",
        "birch_forest" => "Bosque de abedules",
        "old_growth_birch_forest" => "Bosque antiguo de abedules",
        "dark_forest" => "Bosque oscuro",
        "pale_garden" => "Jardín pálido",
        "cherry_grove" => "Arboleda de cerezos",
        "meadow" => "Prado",
        "taiga" => "Taiga",
        "snowy_taiga" => "Taiga nevada",
        "old_growth_pine_taiga" => "Taiga antigua de pinos",
        "old_growth_spruce_taiga" => "Taiga antigua de abetos",
        "jungle" => "Jungla",
        "sparse_jungle" => "Jungla dispersa",
        "bamboo_jungle" => "Jungla de bambú",
        "swamp" => "Pantano",
        "mangrove_swamp" => "Pantano de manglar",
        "desert" => "Desierto",
        "savanna" => "Sabana",
        "savanna_plateau" => "Meseta de sabana",
        "windswept_savanna" => "Sabana ventosa",
        "badlands" => "Tierras baldías (Mesa)",
        "eroded_badlands" => "Tierras baldías erosionadas",
        "wooded_badlands" => "Tierras baldías arboladas",
        "mushroom_fields" => "Campos de hongos",
        "ocean" => "Océano",
        "deep_ocean" => "Océano profundo",
        "warm_ocean" => "Océano cálido",
        "lukewarm_ocean" => "Océano templado",
        "deep_lukewarm_ocean" => "Océano templado profundo",
        "cold_ocean" => "Océano frío",
        "deep_cold_ocean" => "Océano frío profundo",
        "frozen_ocean" => "Océano congelado",
        "deep_frozen_ocean" => "Océano congelado profundo",
        "river" => "Río",
        "frozen_river" => "Río congelado",
        "beach" => "Playa",
        "snowy_beach" => "Playa nevada",
        "stony_shore" => "Costa pedregosa",
        "snowy_plains" => "Llanuras nevadas",
        "ice_spikes" => "Picos de hielo",
        "snowy_slopes" => "Laderas nevadas",
        "jagged_peaks" => "Picos escarpados",
        "frozen_peaks" => "Picos helados",
        "stony_peaks" => "Picos rocosos",
        "windswept_hills" => "Colinas ventosas",
        "windswept_forest" => "Bosque ventoso",
        "windswept_gravelly_hills" => "Colinas de grava ventosas",
        "grove" => "Arboleda nevada",
        "deep_dark" => "Oscuridad profunda",
        "dripstone_caves" => "Cuevas de espeleotemas",
        "lush_caves" => "Cuevas frondosas",
        "nether_wastes" => "Desechos del Nether",
        "crimson_forest" => "Bosque carmesí",
        "warped_forest" => "Bosque distorsionado",
        "soul_sand_valley" => "Valle de arena de almas",
        "basalt_deltas" => "Deltas de basalto",
        "the_end" => "El End (Isla Central)",
        "end_highlands" => "Tierras altas del End",
        "end_midlands" => "Tierras medias del End",
        "end_barrens" => "Tierras áridas del End",
        "small_end_islands" => "Islas pequeñas / Vacío del End",
        _ => return title_case_snake(id_str),
    };
    tr.to_string()
}

fn title_case_snake(s: &str) -> String {
    s.split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut ch = w.chars();
            match ch.next() {
                Some(c) => c.to_uppercase().collect::<String>() + ch.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cubiomes_overworld_nether_end_queries() {
        let mut engine = SeedMapEngine::new(1234567890i64, false);
        let (sx, sz) = engine.spawn_point();
        assert!(sx.abs() < 2048 && sz.abs() < 2048);
        assert_eq!(engine.strongholds().len(), 128);
        assert_eq!(engine.end_gateways().len(), 40);

        let (id, h, name) = engine.sample_point(Dimension::Overworld, 0, 0, Language::EnUs);
        assert!(id >= 0 && !name.is_empty());
        assert!(h > -64.0 && h < 320.0, "Realistic Y height: {}", h);

        let toggles = LayerToggles::default();
        let ow = engine.structures_in_view(Dimension::Overworld, -1500, -1500, 1500, 1500, toggles);
        assert!(
            ow.iter().any(|m| m.kind == StructureKind::Village),
            "Expected Overworld Village hits"
        );
        assert!(
            !ow.iter().any(|m| m.kind == StructureKind::TrailRuins && !toggles.temples),
            "TrailRuins must not appear when temples toggle is off"
        );

        let nether = engine.structures_in_view(Dimension::Nether, -1500, -1500, 1500, 1500, toggles);
        assert!(
            nether.iter().all(|m| matches!(m.kind, StructureKind::Fortress | StructureKind::Bastion)),
            "Nether scan must only contain Nether structures"
        );

        let end = engine.structures_in_view(Dimension::TheEnd, -2500, -2500, 2500, 2500, toggles);
        assert!(
            end.iter().all(|m| matches!(m.kind, StructureKind::EndCity | StructureKind::EndShip | StructureKind::EndGateway)),
            "End scan must never emit Overworld/Nether structures"
        );

        let (nid, _, nname) = engine.sample_point(Dimension::Nether, 0, 0, Language::EnUs);
        assert!(nid >= 0 && !nname.is_empty());

        let (eid, _, ename) = engine.sample_point(Dimension::TheEnd, 0, 0, Language::EnUs);
        assert!(eid >= 0 && !ename.is_empty());
    }

    #[test]
    fn cubiomes_tile_scales_1_to_256_are_distinct_across_adjacent_tiles() {
        let world = OwnedWorld::new(469686492058373120i64, false);
        for &dim in &[Dimension::Overworld, Dimension::Nether, Dimension::TheEnd] {
            for &scale in &[1i32, 4, 16, 64, 256] {
                let mut t0 = vec![0u8; TILE_PX * TILE_PX * 4];
                let mut t1 = vec![0u8; TILE_PX * TILE_PX * 4];
                let stride = (TILE_PX as i32) * scale;
                unsafe {
                    mc_render_tile(
                        world.ptr,
                        dim as i32,
                        scale,
                        0,
                        0,
                        TILE_PX as i32,
                        TILE_PX as i32,
                        MapStyle::Topo3D as i32,
                        t0.as_mut_ptr(),
                    );
                    mc_render_tile(
                        world.ptr,
                        dim as i32,
                        scale,
                        stride,
                        0,
                        TILE_PX as i32,
                        TILE_PX as i32,
                        MapStyle::Topo3D as i32,
                        t1.as_mut_ptr(),
                    );
                }
                assert!(t0.iter().any(|&b| b != 0), "dim={:?} scale={} blank", dim, scale);
                assert_ne!(
                    t0, t1,
                    "adjacent tiles at dim={:?} scale={} must differ (no repeating-tile bug)",
                    dim, scale
                );
            }
        }
    }

    #[test]
    fn java_text_seed_and_slime_chunk() {
        assert_eq!(parse_seed_input("404"), 404);
        assert_eq!(parse_seed_input("abc"), 96354);
        assert_eq!(parse_seed_input("Minecraft"), -1595926131); // exact Java String.hashCode("Minecraft")
        // Verify deterministic Java slime chunk consistency
        let mut true_count = 0;
        for x in -10..10 {
            for z in -10..10 {
                if is_slime_chunk(12345, x, z) {
                    true_count += 1;
                }
            }
        }
        assert!(true_count > 10 && true_count < 80);
    }

    #[test]
    fn mansions_not_starved_by_crowded_overworld_scans() {
        let all_toggles = LayerToggles {
            outposts: true,
            temples: true,
            ruined_portals: true,
            ..Default::default()
        };

        let mansions_only = LayerToggles {
            villages: false,
            trial_chambers: false,
            ancient_cities: false,
            monuments: false,
            outposts: false,
            temples: false,
            ruined_portals: false,
            ..Default::default()
        };

        let mut total_mansions = 0usize;
        for seed in 1i64..=5 {
            let mut engine = SeedMapEngine::new(seed, false);
            let full_hits = engine.structures_in_view(
                Dimension::Overworld,
                -10_000,
                -10_000,
                10_000,
                10_000,
                all_toggles,
            );
            let full_mansions: Vec<(i32, i32)> = full_hits
                .iter()
                .filter(|m| m.kind == StructureKind::Mansion)
                .map(|m| (m.x, m.z))
                .collect();

            let iso_hits = engine.structures_in_view(
                Dimension::Overworld,
                -10_000,
                -10_000,
                10_000,
                10_000,
                mansions_only,
            );
            let iso_mansions: Vec<(i32, i32)> = iso_hits
                .iter()
                .filter(|m| m.kind == StructureKind::Mansion)
                .map(|m| (m.x, m.z))
                .collect();

            assert_eq!(
                full_mansions, iso_mansions,
                "Seed {} dropped Woodland Mansions when crowded Overworld structures were enabled",
                seed
            );
            total_mansions += iso_mansions.len();
        }
        assert!(
            total_mansions >= 15,
            "Expected multiple Woodland Mansions across 5 seeds, got {}",
            total_mansions
        );
    }
}
