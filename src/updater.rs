use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use serde_json::Value;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GITHUB_REPO: &str = "Demian1621/minecraft-companion";
pub const RELEASES_PAGE_URL: &str = "https://github.com/Demian1621/minecraft-companion/releases";
const RELEASES_ALLOWED_PREFIX: &str =
    "https://github.com/Demian1621/minecraft-companion/releases/";
const ASSET_DOWNLOAD_PREFIX: &str =
    "https://github.com/Demian1621/minecraft-companion/releases/download/";
const RELEASES_LATEST_API: &str =
    "https://api.github.com/repos/Demian1621/minecraft-companion/releases/latest";
const MAX_RELEASE_JSON_BYTES: &str = "262144";

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

/// Checks GitHub Releases in a background thread using strict TLS and URL allowlisting.
pub struct UpdateChecker {
    status: UpdateStatus,
    rx: Option<Receiver<UpdateStatus>>,
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
        };
        checker.trigger_check();
        checker
    }

    pub fn status(&self) -> &UpdateStatus {
        &self.status
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

    pub fn poll(&mut self) {
        let Some(rx) = self.rx.as_ref() else {
            return;
        };
        if let Ok(next_status) = rx.try_recv() {
            self.status = next_status;
            self.rx = None;
        }
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
        &format!("User-Agent: minecraft-companion/{}", CURRENT_VERSION),
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
            "html_url": "https://github.com/Demian1621/minecraft-companion/releases/tag/v1.1.0",
            "draft": false,
            "prerelease": false,
            "body": "## Novedades\n- Mejora\u202E spoofing \u0007\n- Soporte Linux",
            "assets": [
                {
                    "name": "minecraft-companion-v1.1.0-windows-x86_64.exe",
                    "browser_download_url": "https://github.com/Demian1621/minecraft-companion/releases/download/v1.1.0/minecraft-companion-v1.1.0-windows-x86_64.exe",
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
}
