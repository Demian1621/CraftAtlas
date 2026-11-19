use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use serde_json::Value;

use crate::crypto_sha256::{constant_time_eq_sha256_hex, sha256_file};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GITHUB_REPO: &str = "Demian1621/CraftAtlas";
pub const RELEASES_PAGE_URL: &str = "https://github.com/Demian1621/CraftAtlas/releases";
pub const FEEDBACK_URL: &str =
    "https://github.com/Demian1621/CraftAtlas/issues/new/choose";
pub const KOFI_URL: &str = "https://ko-fi.com/titusdb";
const RELEASES_ALLOWED_PREFIX: &str =
    "https://github.com/Demian1621/CraftAtlas/releases/";
const ASSET_DOWNLOAD_PREFIX: &str =
    "https://github.com/Demian1621/CraftAtlas/releases/download/";
const RELEASES_LATEST_API: &str =
    "https://api.github.com/repos/Demian1621/CraftAtlas/releases/latest";
const MAX_RELEASE_JSON_BYTES: &str = "262144";
const MAX_CHECKSUMS_BYTES: &str = "65536";
const MAX_BINARY_UPDATE_BYTES: &str = "67108864"; // 64 MiB hard cap

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Validated release binary asset metadata ready for future SHA-256 verified auto-updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseAsset {
    pub name: String,
    pub download_url: String,
    pub sha256_hex: Option<String>,
}

/// Metadata for a newer release fetched from GitHub Releases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseInfo {
    pub tag: String,
    pub version: String,
    pub html_url: String,
    pub notes_summary: String,
    pub assets: Vec<ReleaseAsset>,
}

/// Asynchronous status of the non-blocking GitHub Releases version check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    Checking,
    UpToDate,
    Available(ReleaseInfo),
    Unavailable,
}

/// State of an in-app SHA-256 verified binary download/update operation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum InstallState {
    #[default]
    Idle,
    Downloading {
        asset_name: String,
    },
    ReadyToRestart {
        version: String,
        relaunch_path: Option<PathBuf>,
        saved_only_path: Option<PathBuf>,
        verified_sha256: String,
    },
    Failed(String),
}

/// Checks GitHub Releases in a background thread using strict TLS and URL allowlisting,
/// and supports fail-closed SHA-256 verified downloads/installs.
pub struct UpdateChecker {
    status: UpdateStatus,
    rx: Option<Receiver<UpdateStatus>>,
    install_state: InstallState,
    install_rx: Option<Receiver<InstallState>>,
}

impl Default for UpdateChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateChecker {
    pub fn new() -> Self {
        let mut checker = Self {
            status: UpdateStatus::Checking,
            rx: None,
            install_state: InstallState::Idle,
            install_rx: None,
        };
        checker.trigger_check();
        checker
    }

    pub fn status(&self) -> &UpdateStatus {
        &self.status
    }

    pub fn install_state(&self) -> &InstallState {
        &self.install_state
    }

    pub fn reset_install_state(&mut self) {
        if !matches!(self.install_state, InstallState::Downloading { .. }) {
            self.install_state = InstallState::Idle;
        }
    }

    pub fn trigger_check(&mut self) {
        self.status = UpdateStatus::Checking;
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        thread::spawn(move || {
            let outcome = fetch_latest_release_sync();
            let _ = tx.send(outcome);
        });
    }

    /// Launches a background SHA-256 verified download (and atomic in-place replace where supported).
    pub fn start_verified_install(&mut self, info: ReleaseInfo) {
        if matches!(self.install_state, InstallState::Downloading { .. }) {
            return;
        }
        let Some(asset) = select_platform_asset(&info) else {
            self.install_state = InstallState::Failed(
                "No matching release binary found for this OS/architecture.".to_string(),
            );
            return;
        };
        let asset_name = asset.name.clone();
        self.install_state = InstallState::Downloading { asset_name };
        let (tx, rx) = mpsc::channel();
        self.install_rx = Some(rx);
        thread::spawn(move || {
            let outcome = download_and_verify_update_sync(&info, &asset);
            let _ = tx.send(outcome);
        });
    }

    pub fn poll(&mut self) {
        if let Some(rx) = self.rx.as_ref() {
            if let Ok(next_status) = rx.try_recv() {
                self.status = next_status;
                self.rx = None;
            }
        }
        if let Some(rx) = self.install_rx.as_ref() {
            if let Ok(next_install) = rx.try_recv() {
                self.install_state = next_install;
                self.install_rx = None;
            }
        }
    }
}

/// Removes any leftover `.old.exe` backup from a previous atomic Windows self-update.
pub fn cleanup_stale_update_backup() {
    #[cfg(target_os = "windows")]
    if let Ok(exe) = std::env::current_exe() {
        let backup = exe.with_extension("old.exe");
        if backup.is_file() {
            let _ = fs::remove_file(backup);
        }
    }
}

/// Spawns the newly verified executable cleanly (without invoking any shell).
pub fn relaunch_updated_binary(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    Command::new(path).spawn().is_ok()
}

/// Selects the best matching official release asset for the current OS and architecture.
pub fn select_platform_asset(info: &ReleaseInfo) -> Option<ReleaseAsset> {
    select_asset_for_target(&info.assets, std::env::consts::OS, std::env::consts::ARCH)
}

fn select_asset_for_target(assets: &[ReleaseAsset], os: &str, arch: &str) -> Option<ReleaseAsset> {
    if os == "windows" && arch == "x86_64" {
        return assets
            .iter()
            .find(|a| {
                a.name.starts_with("craft-atlas-") && a.name.ends_with("-windows-x86_64.exe")
            })
            .cloned();
    }
    if os == "linux" && arch == "x86_64" {
        return assets
            .iter()
            .find(|a| {
                a.name.starts_with("craft-atlas-") && a.name.ends_with("-linux-x86_64.AppImage")
            })
            .or_else(|| {
                assets.iter().find(|a| {
                    a.name.starts_with("craft-atlas-") && a.name.ends_with("-linux-x86_64.tar.gz")
                })
            })
            .cloned();
    }
    if os == "macos" {
        return assets
            .iter()
            .find(|a| {
                a.name.starts_with("craft-atlas-") && a.name.ends_with("-macos-universal.dmg")
            })
            .cloned();
    }
    None
}

/// Parses standard `sha256sum` output (`<64-hex>  <filename>`) and extracts the digest for `target_name`.
pub fn parse_sha256sums_for_asset(sums_txt: &str, target_name: &str) -> Option<String> {
    for line in sums_txt.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let hash = parts.next()?;
        let raw_name = parts.next()?;
        let clean_name = raw_name.trim_start_matches('*').trim_start_matches("./");
        if clean_name == target_name
            && hash.len() == 64
            && hash.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Some(hash.to_ascii_lowercase());
        }
    }
    None
}

/// Validates expected executable/archive magic header bytes to reject HTML error pages or corrupt files.
pub fn verify_binary_magic(path: &Path, asset_name: &str) -> Result<(), String> {
    let mut f = File::open(path).map_err(|e| format!("Cannot read downloaded file: {e}"))?;
    let meta = f
        .metadata()
        .map_err(|e| format!("Cannot stat downloaded file: {e}"))?;
    if meta.len() < 1024 {
        return Err("Downloaded payload is unexpectedly small.".to_string());
    }
    let mut hdr = [0u8; 4];
    f.read_exact(&mut hdr)
        .map_err(|e| format!("Failed reading file header: {e}"))?;

    let lower = asset_name.to_ascii_lowercase();
    if lower.ends_with(".exe") && &hdr[..2] != b"MZ" {
        return Err("Invalid Windows executable header (expected MZ).".to_string());
    }
    if lower.ends_with(".appimage") && &hdr != b"\x7fELF" {
        return Err("Invalid Linux AppImage header (expected ELF).".to_string());
    }
    if lower.ends_with(".zip") && &hdr != b"PK\x03\x04" {
        return Err("Invalid ZIP archive header.".to_string());
    }
    if lower.ends_with(".tar.gz") && &hdr[..2] != b"\x1f\x8b" {
        return Err("Invalid GZip archive header.".to_string());
    }
    Ok(())
}

fn resolve_expected_sha256(
    curl_bin: &Path,
    info: &ReleaseInfo,
    asset: &ReleaseAsset,
) -> Result<String, String> {
    if let Some(ref hex) = asset.sha256_hex {
        return Ok(hex.clone());
    }
    let sums_asset = info
        .assets
        .iter()
        .find(|a| a.name == "SHA256SUMS.txt")
        .ok_or_else(|| {
            "Release lacks an official SHA-256 digest and SHA256SUMS.txt (fail-closed abort)."
                .to_string()
        })?;
    if !is_safe_https_url(&sums_asset.download_url)
        || !sums_asset.download_url.starts_with(ASSET_DOWNLOAD_PREFIX)
    {
        return Err("Untrusted SHA256SUMS.txt URL blocked.".to_string());
    }

    let mut cmd = Command::new(curl_bin);
    cmd.args([
        "--proto",
        "=https",
        "--proto-redir",
        "=https",
        "--tlsv1.2",
        "--connect-timeout",
        "8",
        "--max-time",
        "15",
        "--max-filesize",
        MAX_CHECKSUMS_BYTES,
        "-fsSL",
        "-H",
        &format!("User-Agent: craft-atlas/{}", CURRENT_VERSION),
        &sums_asset.download_url,
    ]);
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let out = cmd
        .output()
        .map_err(|e| format!("Failed invoking curl for SHA256SUMS.txt: {e}"))?;
    if !out.status.success() {
        return Err("Could not fetch official SHA256SUMS.txt from GitHub Release.".to_string());
    }
    let text = std::str::from_utf8(&out.stdout)
        .map_err(|_| "SHA256SUMS.txt is not valid UTF-8.".to_string())?;
    parse_sha256sums_for_asset(text, &asset.name).ok_or_else(|| {
        format!(
            "No SHA-256 entry found for {} inside SHA256SUMS.txt.",
            asset.name
        )
    })
}

fn user_downloads_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    if let Ok(profile) = std::env::var("USERPROFILE") {
        if !profile.is_empty() {
            return PathBuf::from(profile).join("Downloads");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return PathBuf::from(home).join("Downloads");
        }
    }
    std::env::temp_dir()
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn dir_is_writable(dir: &Path) -> bool {
    let probe = dir.join(format!(".craft-atlas-write-test-{}", std::process::id()));
    match File::create(&probe) {
        Ok(_) => {
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

fn determine_install_target(asset: &ReleaseAsset) -> (Option<PathBuf>, PathBuf) {
    #[cfg(target_os = "windows")]
    if asset.name.ends_with(".exe") {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                if dir_is_writable(dir) {
                    let tmp = dir.join(format!("craft-atlas-update-{}.tmp", std::process::id()));
                    return (Some(exe), tmp);
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    if asset.name.ends_with(".AppImage") {
        if let Some(appimage) = std::env::var_os("APPIMAGE") {
            let target = PathBuf::from(appimage);
            if let Some(dir) = target.parent() {
                if dir_is_writable(dir) {
                    let tmp = dir.join(format!("craft-atlas-update-{}.tmp", std::process::id()));
                    return (Some(target), tmp);
                }
            }
        }
    }

    let dl = user_downloads_dir();
    let _ = fs::create_dir_all(&dl);
    let tmp = dl.join(format!("{}.downloading.tmp", asset.name));
    (None, tmp)
}

fn download_and_verify_update_sync(info: &ReleaseInfo, asset: &ReleaseAsset) -> InstallState {
    if !is_safe_https_url(&asset.download_url)
        || !asset.download_url.starts_with(ASSET_DOWNLOAD_PREFIX)
    {
        return InstallState::Failed("Blocked untrusted download URL.".to_string());
    }
    let Some(curl_bin) = resolve_trusted_curl_path() else {
        return InstallState::Failed("System curl binary not found.".to_string());
    };

    let expected_sha = match resolve_expected_sha256(&curl_bin, info, asset) {
        Ok(h) => h,
        Err(msg) => return InstallState::Failed(msg),
    };

    let (inplace_target, tmp_path) = determine_install_target(asset);
    let _ = fs::remove_file(&tmp_path);

    let mut cmd = Command::new(&curl_bin);
    cmd.args([
        "--proto",
        "=https",
        "--proto-redir",
        "=https",
        "--tlsv1.2",
        "--connect-timeout",
        "10",
        "--max-time",
        "180",
        "--max-filesize",
        MAX_BINARY_UPDATE_BYTES,
        "-fsSL",
        "-H",
        &format!("User-Agent: craft-atlas/{}", CURRENT_VERSION),
        "-o",
    ]);
    cmd.arg(&tmp_path);
    cmd.arg(&asset.download_url);
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) => {
            let _ = fs::remove_file(&tmp_path);
            return InstallState::Failed(format!("Download failed to start: {e}"));
        }
    };
    if !status.success() {
        let _ = fs::remove_file(&tmp_path);
        return InstallState::Failed("Download interrupted or rejected by server.".to_string());
    }

    if let Err(err) = verify_binary_magic(&tmp_path, &asset.name) {
        let _ = fs::remove_file(&tmp_path);
        return InstallState::Failed(err);
    }

    let actual_sha = match sha256_file(&tmp_path) {
        Ok(h) => h,
        Err(e) => {
            let _ = fs::remove_file(&tmp_path);
            return InstallState::Failed(format!("Failed hashing downloaded binary: {e}"));
        }
    };

    if !constant_time_eq_sha256_hex(&actual_sha, &expected_sha) {
        let _ = fs::remove_file(&tmp_path);
        return InstallState::Failed(
            "SHA-256 verification FAILED! Downloaded file was discarded.".to_string(),
        );
    }

    #[cfg(unix)]
    if asset.name.ends_with(".AppImage") {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o755));
    }

    if let Some(target_exe) = inplace_target {
        #[cfg(target_os = "windows")]
        {
            let backup = target_exe.with_extension("old.exe");
            let _ = fs::remove_file(&backup);
            if let Err(e) = fs::rename(&target_exe, &backup) {
                let _ = fs::remove_file(&tmp_path);
                return InstallState::Failed(format!("Could not stage existing binary: {e}"));
            }
            if let Err(e) = fs::rename(&tmp_path, &target_exe) {
                let _ = fs::rename(&backup, &target_exe);
                let _ = fs::remove_file(&tmp_path);
                return InstallState::Failed(format!("Atomic replace failed (rolled back): {e}"));
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            if let Err(e) = fs::rename(&tmp_path, &target_exe) {
                let _ = fs::remove_file(&tmp_path);
                return InstallState::Failed(format!("Atomic replace failed: {e}"));
            }
        }
        return InstallState::ReadyToRestart {
            version: info.version.clone(),
            relaunch_path: Some(target_exe),
            saved_only_path: None,
            verified_sha256: actual_sha,
        };
    }

    let final_path = user_downloads_dir().join(&asset.name);
    let _ = fs::remove_file(&final_path);
    if let Err(e) = fs::rename(&tmp_path, &final_path) {
        let _ = fs::remove_file(&tmp_path);
        return InstallState::Failed(format!("Could not save verified download: {e}"));
    }
    InstallState::ReadyToRestart {
        version: info.version.clone(),
        relaunch_path: None,
        saved_only_path: Some(final_path),
        verified_sha256: actual_sha,
    }
}

/// Validates that an external link uses strictly HTTPS and contains no whitespace or control characters.
pub fn is_safe_https_url(url: &str) -> bool {
    url.starts_with("https://")
        && url.len() <= 2048
        && !url.chars().any(|c| c.is_control() || c.is_whitespace())
}

/// Resolves `curl` from an explicit OS-trusted directory to avoid CWD/PATH binary planting.
fn resolve_trusted_curl_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        for var in ["SystemRoot", "WINDIR"] {
            if let Ok(root) = std::env::var(var) {
                if !root.is_empty() {
                    let candidate = Path::new(&root).join("System32").join("curl.exe");
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
        }
        None
    }
    #[cfg(not(target_os = "windows"))]
    {
        for candidate in [
            "/usr/bin/curl",
            "/opt/homebrew/bin/curl",
            "/usr/local/bin/curl",
        ] {
            let path = Path::new(candidate);
            if path.is_file() {
                return Some(path.to_path_buf());
            }
        }
        None
    }
}

fn fetch_latest_release_sync() -> UpdateStatus {
    let Some(curl_bin) = resolve_trusted_curl_path() else {
        return UpdateStatus::Unavailable;
    };

    let mut cmd = Command::new(curl_bin);
    cmd.args([
        "--proto",
        "=https",
        "--proto-redir",
        "=https",
        "--tlsv1.2",
        "--connect-timeout",
        "5",
        "--max-time",
        "7",
        "--max-filesize",
        MAX_RELEASE_JSON_BYTES,
        "-fsSL",
        "-H",
        "Accept: application/vnd.github+json",
        "-H",
        &format!("User-Agent: craft-atlas/{}", CURRENT_VERSION),
        RELEASES_LATEST_API,
    ]);
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let Ok(output) = cmd.output() else {
        return UpdateStatus::Unavailable;
    };
    if !output.status.success() {
        return UpdateStatus::Unavailable;
    }
    let Ok(body) = std::str::from_utf8(&output.stdout) else {
        return UpdateStatus::Unavailable;
    };
    let Some(info) = parse_github_release_json(body) else {
        return UpdateStatus::Unavailable;
    };
    if is_newer_version(CURRENT_VERSION, &info.tag) {
        UpdateStatus::Available(info)
    } else {
        UpdateStatus::UpToDate
    }
}

fn is_safe_release_tag(tag: &str) -> bool {
    !tag.is_empty()
        && tag.len() <= 48
        && !tag.contains("..")
        && tag
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_' | b'+'))
}

fn is_bidi_or_control(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            '\u{200E}'
                | '\u{200F}'
                | '\u{061C}'
                | '\u{202A}'..='\u{202E}'
                | '\u{2066}'..='\u{2069}'
        )
}

fn sanitize_display_text(raw: &str) -> String {
    raw.chars()
        .filter(|&c| c == '\n' || !is_bidi_or_control(c))
        .collect()
}

fn parse_sha256_digest(raw: &str) -> Option<String> {
    let hex = raw.trim().strip_prefix("sha256:")?.trim();
    if hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(hex.to_ascii_lowercase())
    } else {
        None
    }
}

/// Parses GitHub's `/releases/latest` JSON payload into validated, repo-scoped metadata.
pub fn parse_github_release_json(raw_json: &str) -> Option<ReleaseInfo> {
    let v: Value = serde_json::from_str(raw_json).ok()?;
    if v.get("draft").and_then(|d| d.as_bool()).unwrap_or(false)
        || v.get("prerelease").and_then(|p| p.as_bool()).unwrap_or(false)
    {
        return None;
    }
    let tag = v.get("tag_name")?.as_str()?.trim().to_string();
    if !is_safe_release_tag(&tag) || parse_semver(&tag).is_none() {
        return None;
    }
    let version = tag.trim_start_matches(['v', 'V']).to_string();
    let html_url = v
        .get("html_url")
        .and_then(|u| u.as_str())
        .filter(|u| is_safe_https_url(u) && u.starts_with(RELEASES_ALLOWED_PREFIX))
        .unwrap_or(RELEASES_PAGE_URL)
        .to_string();

    let raw_body = v.get("body").and_then(|b| b.as_str()).unwrap_or("");
    let notes_summary = summarize_release_notes(raw_body);

    let assets = v
        .get("assets")
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let name = item
                        .get("name")
                        .and_then(|n| n.as_str())
                        .map(sanitize_display_text)
                        .filter(|n| !n.is_empty() && !n.contains('/') && !n.contains('\\'))?;
                    let download_url = item
                        .get("browser_download_url")
                        .and_then(|u| u.as_str())
                        .filter(|u| is_safe_https_url(u) && u.starts_with(ASSET_DOWNLOAD_PREFIX))?
                        .to_string();
                    let sha256_hex = item
                        .get("digest")
                        .and_then(|d| d.as_str())
                        .and_then(parse_sha256_digest);
                    Some(ReleaseAsset {
                        name,
                        download_url,
                        sha256_hex,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Some(ReleaseInfo {
        tag,
        version,
        html_url,
        notes_summary,
        assets,
    })
}

fn summarize_release_notes(body: &str) -> String {
    let clean = sanitize_display_text(body);
    let lines: Vec<&str> = clean
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("```"))
        .take(4)
        .collect();
    let joined = lines.join("\n");
    if joined.chars().count() > 260 {
        let truncated: String = joined.chars().take(257).collect();
        format!("{}...", truncated.trim_end())
    } else {
        joined
    }
}

/// Parses a SemVer string (`v1.2.3` or `1.2.0-rc1`) into `(major, minor, patch, is_prerelease)`.
fn parse_semver(raw: &str) -> Option<(u64, u64, u64, bool)> {
    let trimmed = raw.trim().trim_start_matches(['v', 'V']);
    let without_build = trimmed.split('+').next()?;
    let (core, is_pre) = match without_build.split_once('-') {
        Some((c, _pre)) => (c, true),
        None => (without_build, false),
    };
    let mut parts = core.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    let patch = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch, is_pre))
}

/// Returns `true` strictly when `candidate` represents a newer release than `current`.
pub fn is_newer_version(current: &str, candidate: &str) -> bool {
    let Some((cmaj, cmin, cpat, c_pre)) = parse_semver(current) else {
        return false;
    };
    let Some((nmaj, nmin, npat, n_pre)) = parse_semver(candidate) else {
        return false;
    };
    match (nmaj, nmin, npat).cmp(&(cmaj, cmin, cpat)) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => c_pre && !n_pre,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_comparison_handles_prefixes_and_prereleases() {
        assert!(is_newer_version("1.0.0", "v1.0.1"));
        assert!(is_newer_version("1.0.0", "1.1.0"));
        assert!(is_newer_version("1.2.3", "v2.0.0"));
        assert!(!is_newer_version("1.0.0", "v1.0.0"));
        assert!(!is_newer_version("1.2.0", "1.1.9"));
        assert!(!is_newer_version("1.0.1", "1.0.1-rc1"));
        assert!(is_newer_version("1.0.1-rc1", "1.0.1"));
    }

    #[test]
    fn parses_valid_github_release_payload_and_sha256_assets() {
        let payload = r###"{
            "tag_name": "v1.1.0",
            "html_url": "https://github.com/Demian1621/CraftAtlas/releases/tag/v1.1.0",
            "draft": false,
            "prerelease": false,
            "body": "## Novedades\n- Mejora\u202E spoofing \u0007\n- Soporte Linux",
            "assets": [
                {
                    "name": "craft-atlas-v1.1.0-windows-x86_64.exe",
                    "browser_download_url": "https://github.com/Demian1621/CraftAtlas/releases/download/v1.1.0/craft-atlas-v1.1.0-windows-x86_64.exe",
                    "digest": "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                },
                {
                    "name": "rogue.exe",
                    "browser_download_url": "https://github.com/attacker/evil/releases/download/v1.1.0/rogue.exe",
                    "digest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                }
            ]
        }"###;
        let parsed = parse_github_release_json(payload).expect("valid release JSON");
        assert_eq!(parsed.tag, "v1.1.0");
        assert_eq!(parsed.version, "1.1.0");
        assert!(!parsed.notes_summary.contains('\u{202E}'));
        assert!(!parsed.notes_summary.contains('\u{0007}'));
        assert_eq!(parsed.assets.len(), 1);
        assert!(parsed.assets[0].sha256_hex.is_some());
    }

    #[test]
    fn rejects_external_html_url_traversal_tags_and_bad_schemes() {
        let foreign_url = r#"{"tag_name":"v1.2.0","html_url":"https://github.com/other/repo/releases/tag/v1.2.0"}"#;
        let parsed = parse_github_release_json(foreign_url).expect("falls back html_url cleanly");
        assert_eq!(parsed.html_url, RELEASES_PAGE_URL);

        let bad_tag = r#"{"tag_name":"v1.0.0/../evil"}"#;
        assert!(parse_github_release_json(bad_tag).is_none());

        assert!(!is_safe_https_url("http://example.com"));
        assert!(!is_safe_https_url("file:///C:/Windows/System32/calc.exe"));
        assert!(!is_safe_https_url("https://example.com/\nevil"));
        assert!(is_safe_https_url(RELEASES_PAGE_URL));
    }

    #[test]
    fn rejects_drafts_or_prereleases() {
        let draft = r#"{"tag_name": "v2.0.0", "draft": true, "prerelease": false}"#;
        let pre = r#"{"tag_name": "v2.0.0-beta", "draft": false, "prerelease": true}"#;
        assert!(parse_github_release_json(draft).is_none());
        assert!(parse_github_release_json(pre).is_none());
    }

    #[test]
    fn parses_sha256sums_txt_and_selects_platform_assets() {
        let sums = "\
0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  craft-atlas-v1.2.0-windows-x86_64.exe
abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd *./craft-atlas-v1.2.0-linux-x86_64.AppImage
invalid_short_hash  evil.exe
";
        assert_eq!(
            parse_sha256sums_for_asset(sums, "craft-atlas-v1.2.0-windows-x86_64.exe").as_deref(),
            Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        );
        assert_eq!(
            parse_sha256sums_for_asset(sums, "craft-atlas-v1.2.0-linux-x86_64.AppImage").as_deref(),
            Some("abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd")
        );
        assert!(parse_sha256sums_for_asset(sums, "evil.exe").is_none());

        let assets = vec![
            ReleaseAsset {
                name: "craft-atlas-v1.2.0-windows-x86_64.exe".to_string(),
                download_url: "https://github.com/Demian1621/CraftAtlas/releases/download/v1.2.0/craft-atlas-v1.2.0-windows-x86_64.exe".to_string(),
                sha256_hex: None,
            },
            ReleaseAsset {
                name: "craft-atlas-v1.2.0-linux-x86_64.AppImage".to_string(),
                download_url: "https://github.com/Demian1621/CraftAtlas/releases/download/v1.2.0/craft-atlas-v1.2.0-linux-x86_64.AppImage".to_string(),
                sha256_hex: None,
            },
            ReleaseAsset {
                name: "craft-atlas-v1.2.0-macos-universal.dmg".to_string(),
                download_url: "https://github.com/Demian1621/CraftAtlas/releases/download/v1.2.0/craft-atlas-v1.2.0-macos-universal.dmg".to_string(),
                sha256_hex: None,
            },
        ];
        assert_eq!(
            select_asset_for_target(&assets, "windows", "x86_64").map(|a| a.name),
            Some("craft-atlas-v1.2.0-windows-x86_64.exe".to_string())
        );
        assert_eq!(
            select_asset_for_target(&assets, "linux", "x86_64").map(|a| a.name),
            Some("craft-atlas-v1.2.0-linux-x86_64.AppImage".to_string())
        );
        assert_eq!(
            select_asset_for_target(&assets, "macos", "aarch64").map(|a| a.name),
            Some("craft-atlas-v1.2.0-macos-universal.dmg".to_string())
        );
    }

    #[test]
    fn binary_magic_rejects_html_and_accepts_mz_or_elf() {
        let dir = std::env::temp_dir().join("craft_atlas_updater_magic_test");
        let _ = fs::create_dir_all(&dir);

        let fake_exe = dir.join("fake.exe");
        let mut html = b"<!DOCTYPE html><html>404</html>".to_vec();
        html.resize(2048, b' ');
        fs::write(&fake_exe, &html).unwrap();
        assert!(verify_binary_magic(&fake_exe, "craft-atlas-v1.2.0-windows-x86_64.exe").is_err());

        let valid_exe = dir.join("valid.exe");
        let mut mz = b"MZ\x90\x00".to_vec();
        mz.resize(2048, 0);
        fs::write(&valid_exe, &mz).unwrap();
        assert!(verify_binary_magic(&valid_exe, "craft-atlas-v1.2.0-windows-x86_64.exe").is_ok());

        let valid_elf = dir.join("valid.AppImage");
        let mut elf = b"\x7fELF".to_vec();
        elf.resize(2048, 0);
        fs::write(&valid_elf, &elf).unwrap();
        assert!(
            verify_binary_magic(&valid_elf, "craft-atlas-v1.2.0-linux-x86_64.AppImage").is_ok()
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
