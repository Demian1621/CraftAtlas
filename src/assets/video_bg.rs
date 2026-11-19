use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};
use std::sync::Arc;
use std::thread;

use eframe::egui::{self, ColorImage, TextureFilter, TextureHandle, TextureOptions, TextureWrapMode};
use image::ImageFormat;

use crate::state::prefs::UserPrefs;

pub const CAVF_MAGIC: &[u8; 4] = b"CAVF";
pub const CAVF_VERSION: u16 = 1;
pub const MAX_CAVF_BYTES: usize = 24 * 1024 * 1024;
pub const MAX_CUSTOM_INPUT_BYTES: u64 = 250 * 1024 * 1024;
pub const MAX_CAVF_FRAMES: u32 = 360;

pub const VID_DEEP_DARK: &[u8] = include_bytes!("../../assets/backgrounds/vid_deep_dark.cavf");
pub const VID_TRIAL_CHAMBERS: &[u8] =
    include_bytes!("../../assets/backgrounds/vid_trial_chambers.cavf");
pub const VID_PALE_GARDEN: &[u8] = include_bytes!("../../assets/backgrounds/vid_pale_garden.cavf");
pub const VID_OCEAN_MONUMENT: &[u8] =
    include_bytes!("../../assets/backgrounds/vid_ocean_monument.cavf");

#[derive(Clone)]
pub enum CavfSource {
    Embedded(&'static [u8]),
    Owned(Arc<[u8]>),
}

impl CavfSource {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            CavfSource::Embedded(b) => b,
            CavfSource::Owned(a) => a,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CavfFrameEntry {
    pub offset: usize,
    pub len: usize,
}

#[derive(Debug, Clone)]
pub struct CavfMeta {
    pub width: u16,
    pub height: u16,
    pub fps: u8,
    pub frames: Vec<CavfFrameEntry>,
}

/// Fail-closed parser for `.cavf` (CraftAtlas Video Frames) container files.
pub fn parse_cavf(bytes: &[u8]) -> Result<CavfMeta, String> {
    if bytes.len() < 16 {
        return Err("CAVF file truncated (< 16 bytes)".to_string());
    }
    if bytes.len() > MAX_CAVF_BYTES {
        return Err("CAVF file exceeds 24 MiB safety limit".to_string());
    }
    if &bytes[0..4] != CAVF_MAGIC {
        return Err("Invalid CAVF magic header".to_string());
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != CAVF_VERSION {
        return Err(format!("Unsupported CAVF version {}", version));
    }
    let width = u16::from_le_bytes([bytes[6], bytes[7]]);
    let height = u16::from_le_bytes([bytes[8], bytes[9]]);
    if !(16..=1280).contains(&width) || !(16..=720).contains(&height) {
        return Err(format!("CAVF dimensions out of range: {}x{}", width, height));
    }
    let fps = bytes[10];
    if !(1..=30).contains(&fps) {
        return Err(format!("CAVF fps out of range: {}", fps));
    }
    let frame_count = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
    if frame_count == 0 || frame_count > MAX_CAVF_FRAMES {
        return Err(format!("CAVF frame count out of range: {}", frame_count));
    }
    let index_bytes = (frame_count as usize)
        .checked_mul(8)
        .and_then(|n| 16usize.checked_add(n))
        .ok_or_else(|| "CAVF header overflow".to_string())?;
    if bytes.len() < index_bytes {
        return Err("CAVF frame index table truncated".to_string());
    }

    let mut frames = Vec::with_capacity(frame_count as usize);
    for idx in 0..(frame_count as usize) {
        let p = 16 + idx * 8;
        let offset = u32::from_le_bytes([bytes[p], bytes[p + 1], bytes[p + 2], bytes[p + 3]]) as usize;
        let len = u32::from_le_bytes([bytes[p + 4], bytes[p + 5], bytes[p + 6], bytes[p + 7]]) as usize;
        if offset < index_bytes || len < 4 {
            return Err(format!("Invalid CAVF frame entry #{}", idx));
        }
        let end = offset
            .checked_add(len)
            .ok_or_else(|| "CAVF frame offset overflow".to_string())?;
        if end > bytes.len() {
            return Err(format!("CAVF frame #{} out of bounds", idx));
        }
        frames.push(CavfFrameEntry { offset, len });
    }

    Ok(CavfMeta {
        width,
        height,
        fps,
        frames,
    })
}

fn decode_cavf_frame(src: &CavfSource, meta: &CavfMeta, frame_idx: usize) -> Option<ColorImage> {
    let entry = meta.frames.get(frame_idx)?;
    let slice = &src.as_bytes()[entry.offset..entry.offset + entry.len];
    let img = image::load_from_memory_with_format(slice, ImageFormat::Jpeg)
        .ok()?
        .to_rgba8();
    let (w, h) = img.dimensions();
    Some(ColorImage::from_rgba_unmultiplied([w as usize, h as usize], img.as_raw()))
}

struct DecodeJob {
    generation: u64,
    src: CavfSource,
    meta: CavfMeta,
    frame_idx: usize,
}

struct DecodedFrame {
    generation: u64,
    frame_idx: usize,
    image: ColorImage,
}

/// Asynchronous 1-frame-ahead streaming player reusing a single GPU texture cleanly.
pub struct VideoBackdropPlayer {
    active_key: String,
    generation: u64,
    source: Option<(CavfSource, CavfMeta)>,
    tex: Option<TextureHandle>,
    shown_frame: usize,
    pending_frame: Option<usize>,
    elapsed_sec: f64,
    job_tx: SyncSender<DecodeJob>,
    res_rx: Receiver<DecodedFrame>,
}

impl Default for VideoBackdropPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoBackdropPlayer {
    pub fn new() -> Self {
        let (job_tx, job_rx) = mpsc::sync_channel::<DecodeJob>(1);
        let (res_tx, res_rx) = mpsc::channel::<DecodedFrame>();
        thread::Builder::new()
            .name("craftatlas-video-bg".to_string())
            .spawn(move || {
                while let Ok(job) = job_rx.recv() {
                    if let Some(image) = decode_cavf_frame(&job.src, &job.meta, job.frame_idx) {
                        let _ = res_tx.send(DecodedFrame {
                            generation: job.generation,
                            frame_idx: job.frame_idx,
                            image,
                        });
                    }
                }
            })
            .expect("spawn craftatlas video decoder thread");

        Self {
            active_key: String::new(),
            generation: 0,
            source: None,
            tex: None,
            shown_frame: 0,
            pending_frame: None,
            elapsed_sec: 0.0,
            job_tx,
            res_rx,
        }
    }

    pub fn invalidate_key(&mut self, key: &str) {
        if self.active_key == key {
            self.active_key.clear();
            self.source = None;
            self.tex = None;
            self.pending_frame = None;
        }
    }

    /// Selects a clip when switching video themes, keeping decode idle if already loaded.
    pub fn ensure_clip(&mut self, ctx: &egui::Context, key: &str, src: CavfSource) -> bool {
        if self.active_key == key && self.source.is_some() {
            return true;
        }
        let Ok(meta) = parse_cavf(src.as_bytes()) else {
            return false;
        };
        self.active_key = key.to_string();
        self.generation = self.generation.wrapping_add(1);
        self.shown_frame = 0;
        self.pending_frame = None;
        self.elapsed_sec = 0.0;

        // Decode frame 0 immediately so the very first painted frame has zero black flash
        if let Some(first) = decode_cavf_frame(&src, &meta, 0) {
            let opts = TextureOptions {
                magnification: TextureFilter::Linear,
                minification: TextureFilter::Linear,
                mipmap_mode: None,
                wrap_mode: TextureWrapMode::ClampToEdge,
            };
            if let Some(tex) = &mut self.tex {
                tex.set(first, opts);
            } else {
                self.tex = Some(ctx.load_texture("mc_video_backdrop", first, opts));
            }
        }
        self.source = Some((src, meta));
        true
    }

    /// Advances playback ONLY while `active` (0% decode CPU when unfocused), returning `(tex_id, aspect)`.
    pub fn tick_and_texture(&mut self, dt: f32, active: bool) -> Option<(egui::TextureId, f32)> {
        while let Ok(done) = self.res_rx.try_recv() {
            if done.generation == self.generation {
                let opts = TextureOptions {
                    magnification: TextureFilter::Linear,
                    minification: TextureFilter::Linear,
                    mipmap_mode: None,
                    wrap_mode: TextureWrapMode::ClampToEdge,
                };
                if let Some(tex) = &mut self.tex {
                    tex.set(done.image, opts);
                }
                self.shown_frame = done.frame_idx;
                self.pending_frame = None;
            }
        }

        let (src, meta) = self.source.as_ref()?;
        let aspect = (meta.width as f32) / (meta.height.max(1) as f32);

        if active && meta.frames.len() > 1 {
            self.elapsed_sec += dt.clamp(0.0, 0.1) as f64;
            let frame_dur = 1.0 / (meta.fps as f64);
            let target_idx = ((self.elapsed_sec / frame_dur).floor() as usize) % meta.frames.len();
            if target_idx != self.shown_frame && self.pending_frame != Some(target_idx) {
                let job = DecodeJob {
                    generation: self.generation,
                    src: src.clone(),
                    meta: meta.clone(),
                    frame_idx: target_idx,
                };
                if self.job_tx.try_send(job).is_ok() {
                    self.pending_frame = Some(target_idx);
                }
            }
        }

        self.tex.as_ref().map(|t| (t.id(), aspect))
    }
}

/// Returns `%APPDATA%\craft-atlas\video_themes\custom.cavf` (or platform config equivalent).
pub fn custom_video_cache_path() -> Option<PathBuf> {
    Some(UserPrefs::app_data_dir()?.join("video_themes").join("custom.cavf"))
}

pub fn load_custom_cavf_bytes() -> Option<CavfSource> {
    let path = custom_video_cache_path()?;
    let meta = fs::metadata(&path).ok()?;
    if meta.len() > MAX_CAVF_BYTES as u64 {
        return None;
    }
    let bytes = fs::read(&path).ok()?;
    parse_cavf(&bytes).ok()?;
    Some(CavfSource::Owned(Arc::from(bytes.into_boxed_slice())))
}

/// Locates `ffmpeg` strictly from absolute system/PATH locations (never CWD relative `./ffmpeg`).
fn find_trusted_ffmpeg() -> Option<PathBuf> {
    let exe_name = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            if !dir.is_absolute() {
                continue;
            }
            let cand = dir.join(exe_name);
            if cand.is_file() {
                return Some(cand);
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        for fallback in ["/usr/bin/ffmpeg", "/usr/local/bin/ffmpeg", "/opt/homebrew/bin/ffmpeg"] {
            let p = Path::new(fallback);
            if p.is_file() {
                return Some(p.to_path_buf());
            }
        }
    }
    None
}

struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn pack_jpegs_into_cavf(
    frames_dir: &Path,
    dest_file: &Path,
    width: u16,
    height: u16,
    fps: u8,
) -> Result<(), String> {
    let mut entries: Vec<PathBuf> = fs::read_dir(frames_dir)
        .map_err(|e| format!("Failed reading transcoded frames: {}", e))?
        .filter_map(|e| e.ok().map(|d| d.path()))
        .filter(|p| {
            p.extension()
                .and_then(|s| s.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("jpg"))
        })
        .collect();
    entries.sort();

    if entries.is_empty() {
        return Err("ffmpeg produced 0 video frames".to_string());
    }
    if entries.len() > MAX_CAVF_FRAMES as usize {
        entries.truncate(MAX_CAVF_FRAMES as usize);
    }

    let mut frame_blobs: Vec<Vec<u8>> = Vec::with_capacity(entries.len());
    let mut total_payload = 0usize;
    for path in &entries {
        let b = fs::read(path).map_err(|e| format!("Failed reading frame: {}", e))?;
        total_payload = total_payload
            .checked_add(b.len())
            .ok_or_else(|| "Video too large".to_string())?;
        if 16 + entries.len() * 8 + total_payload > MAX_CAVF_BYTES {
            return Err("Converted loop exceeds 24 MiB limit".to_string());
        }
        frame_blobs.push(b);
    }

    let frame_count = frame_blobs.len() as u32;
    let index_end = 16 + frame_blobs.len() * 8;
    let mut out = Vec::with_capacity(index_end + total_payload);
    out.extend_from_slice(CAVF_MAGIC);
    out.extend_from_slice(&CAVF_VERSION.to_le_bytes());
    out.extend_from_slice(&width.to_le_bytes());
    out.extend_from_slice(&height.to_le_bytes());
    out.push(fps);
    out.push(0);
    out.extend_from_slice(&frame_count.to_le_bytes());

    let mut offset = index_end as u32;
    for blob in &frame_blobs {
        let len = blob.len() as u32;
        out.extend_from_slice(&offset.to_le_bytes());
        out.extend_from_slice(&len.to_le_bytes());
        offset += len;
    }
    for blob in &frame_blobs {
        out.extend_from_slice(blob);
    }

    parse_cavf(&out)?;
    if let Some(parent) = dest_file.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Create dir failed: {}", e))?;
    }
    let tmp_dest = dest_file.with_extension("cavf.tmp");
    fs::write(&tmp_dest, &out).map_err(|e| format!("Failed writing .cavf cache: {}", e))?;
    if dest_file.exists() {
        let _ = fs::remove_file(dest_file);
    }
    fs::rename(&tmp_dest, dest_file).map_err(|e| format!("Rename failed: {}", e))?;
    Ok(())
}

/// Validates and converts any user video (`.mp4/.webm/.mov/.mkv/.gif/.cavf`) safely into `custom.cavf`.
pub fn import_custom_video_to_cavf(src: &Path) -> Result<PathBuf, String> {
    let meta = fs::metadata(src).map_err(|e| format!("Cannot read selected file: {}", e))?;
    if !meta.is_file() || meta.len() == 0 {
        return Err("Selected path is empty or not a regular file".to_string());
    }
    if meta.len() > MAX_CUSTOM_INPUT_BYTES {
        return Err("Selected video exceeds 250 MiB limit".to_string());
    }

    let ext = src
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    if !matches!(
        ext.as_str(),
        "mp4" | "webm" | "mov" | "mkv" | "gif" | "cavf"
    ) {
        return Err(format!("Unsupported format '.{}'", ext));
    }

    let canonical = fs::canonicalize(src).map_err(|e| format!("Path validation failed: {}", e))?;
    let dest = custom_video_cache_path()
        .ok_or_else(|| "Unable to resolve user config directory".to_string())?;
    let parent = dest
        .parent()
        .ok_or_else(|| "Invalid destination path".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("Create dir failed: {}", e))?;

    if ext == "cavf" {
        if meta.len() > MAX_CAVF_BYTES as u64 {
            return Err("CAVF file exceeds 24 MiB limit".to_string());
        }
        let bytes = fs::read(&canonical).map_err(|e| format!("Failed reading .cavf: {}", e))?;
        parse_cavf(&bytes)?;
        let tmp_dest = dest.with_extension("cavf.tmp");
        fs::write(&tmp_dest, &bytes).map_err(|e| format!("Failed writing .cavf cache: {}", e))?;
        if dest.exists() {
            let _ = fs::remove_file(&dest);
        }
        fs::rename(&tmp_dest, &dest).map_err(|e| format!("Failed saving custom.cavf: {}", e))?;
        return Ok(dest);
    }

    let ffmpeg = find_trusted_ffmpeg().ok_or_else(|| {
        "ffmpeg was not found in system PATH. Install FFmpeg or pick a .cavf file.".to_string()
    })?;

    let tmp_frames = parent.join("tmp_import");
    let _ = fs::remove_dir_all(&tmp_frames);
    fs::create_dir_all(&tmp_frames).map_err(|e| format!("Create temp dir failed: {}", e))?;
    let _guard = TempDirGuard(tmp_frames.clone());

    let out_pattern = tmp_frames.join("frame_%03d.jpg");
    let mut cmd = Command::new(&ffmpeg);
    cmd.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostdin",
        "-y",
        "-t",
        "12",
        "-i",
    ])
    .arg(&canonical)
    .args([
        "-an",
        "-sn",
        "-vf",
        "fps=12,scale=854:480:force_original_aspect_ratio=increase,crop=854:480",
        "-q:v",
        "9",
    ])
    .arg(&out_pattern);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let status = cmd
        .status()
        .map_err(|e| format!("Failed launching ffmpeg: {}", e))?;
    if !status.success() {
        return Err("ffmpeg could not decode this video file".to_string());
    }

    pack_jpegs_into_cavf(&tmp_frames, &dest, 854, 480, 12)?;
    Ok(dest)
}

/// Non-blocking UI state manager for importing a custom wallpaper video on a background thread.
#[derive(Default)]
pub struct CustomVideoImporter {
    importing: bool,
    rx: Option<Receiver<Result<PathBuf, String>>>,
}

impl CustomVideoImporter {
    pub fn is_importing(&self) -> bool {
        self.importing
    }

    pub fn start_import(&mut self, src: PathBuf) {
        if self.importing {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.importing = true;
        self.rx = Some(rx);
        thread::Builder::new()
            .name("craftatlas-video-import".to_string())
            .spawn(move || {
                let res = import_custom_video_to_cavf(&src);
                let _ = tx.send(res);
            })
            .expect("spawn video importer thread");
    }

    pub fn poll(&mut self) -> Option<Result<PathBuf, String>> {
        let rx = self.rx.as_ref()?;
        match rx.try_recv() {
            Ok(res) => {
                self.importing = false;
                self.rx = None;
                Some(res)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.importing = false;
                self.rx = None;
                Some(Err("Video import thread terminated unexpectedly".to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_video_themes_parse_and_decode_frame_zero() {
        for (name, bytes) in [
            ("vid_deep_dark", VID_DEEP_DARK),
            ("vid_trial_chambers", VID_TRIAL_CHAMBERS),
            ("vid_pale_garden", VID_PALE_GARDEN),
            ("vid_ocean_monument", VID_OCEAN_MONUMENT),
        ] {
            let meta = parse_cavf(bytes).unwrap_or_else(|e| panic!("{name} parse error: {e}"));
            assert_eq!(meta.width, 854);
            assert_eq!(meta.height, 480);
            assert_eq!(meta.fps, 12);
            assert_eq!(meta.frames.len(), 24);
            let src = CavfSource::Embedded(bytes);
            let frame0 = decode_cavf_frame(&src, &meta, 0)
                .unwrap_or_else(|| panic!("{name} failed decoding frame 0"));
            assert_eq!(frame0.size, [854, 480]);
        }
    }

    #[test]
    fn malformed_cavf_inputs_fail_closed() {
        assert!(parse_cavf(&[]).is_err());
        assert!(parse_cavf(b"NOTCAVF000000000").is_err());

        // Valid magic, bad dimensions (width=0)
        let mut bad_dim = [0u8; 24];
        bad_dim[0..4].copy_from_slice(CAVF_MAGIC);
        bad_dim[4..6].copy_from_slice(&1u16.to_le_bytes());
        bad_dim[8..10].copy_from_slice(&480u16.to_le_bytes());
        bad_dim[10] = 12;
        bad_dim[12..16].copy_from_slice(&1u32.to_le_bytes());
        assert!(parse_cavf(&bad_dim).is_err());

        // Out-of-bounds frame offset
        let mut bad_off = [0u8; 24];
        bad_off[0..4].copy_from_slice(CAVF_MAGIC);
        bad_off[4..6].copy_from_slice(&1u16.to_le_bytes());
        bad_off[6..8].copy_from_slice(&640u16.to_le_bytes());
        bad_off[8..10].copy_from_slice(&360u16.to_le_bytes());
        bad_off[10] = 12;
        bad_off[12..16].copy_from_slice(&1u32.to_le_bytes());
        bad_off[16..20].copy_from_slice(&9999u32.to_le_bytes());
        bad_off[20..24].copy_from_slice(&32u32.to_le_bytes());
        assert!(parse_cavf(&bad_off).is_err());
    }
}
