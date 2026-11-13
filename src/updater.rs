use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use serde_json::Value;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GITHUB_REPO: &str = "Demian1621/minecraft-companion";
pub const RELEASES_PAGE_URL: &str = "https://github.com/Demian1621/minecraft-companion/releases";
const RELEASES_LATEST_API: &str =
    "https://api.github.com/repos/Demian1621/minecraft-companion/releases/latest";

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Metadata for a newer release fetched from GitHub Releases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseInfo {
    pub tag: String,
    pub version: String,
    pub html_url: String,
    pub notes_summary: String,
}

/// Asynchronous status of the non-blocking GitHub Releases version check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    Checking,
    UpToDate,
    Available(ReleaseInfo),
    Unavailable,
}

/// Checks GitHub Releases in a background thread and exposes future-ready release metadata
/// (so v1.x can extend into direct asset download / auto-update cleanly).
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

fn fetch_latest_release_sync() -> UpdateStatus {
    let mut cmd = Command::new("curl");
    cmd.args([
        "-fsSL",
        "--max-time",
        "6",
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

/// Parses GitHub's `/releases/latest` JSON payload into normalized release metadata.
pub fn parse_github_release_json(raw_json: &str) -> Option<ReleaseInfo> {
    let v: Value = serde_json::from_str(raw_json).ok()?;
    if v.get("draft").and_then(|d| d.as_bool()).unwrap_or(false)
        || v.get("prerelease").and_then(|p| p.as_bool()).unwrap_or(false)
    {
        return None;
    }
    let tag = v.get("tag_name")?.as_str()?.trim().to_string();
    if tag.is_empty() {
        return None;
    }
    let version = tag.trim_start_matches(['v', 'V']).to_string();
    let html_url = v
        .get("html_url")
        .and_then(|u| u.as_str())
        .filter(|u| u.starts_with("https://github.com/"))
        .unwrap_or(RELEASES_PAGE_URL)
        .to_string();

    let raw_body = v.get("body").and_then(|b| b.as_str()).unwrap_or("");
    let notes_summary = summarize_release_notes(raw_body);

    Some(ReleaseInfo {
        tag,
        version,
        html_url,
        notes_summary,
    })
}

fn summarize_release_notes(body: &str) -> String {
    let lines: Vec<&str> = body
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
    fn parses_valid_github_release_payload() {
        let payload = r###"{
            "tag_name": "v1.1.0",
            "html_url": "https://github.com/Demian1621/minecraft-companion/releases/tag/v1.1.0",
            "draft": false,
            "prerelease": false,
            "body": "## Novedades\n- Mejora de Seed Map\n- Soporte Linux ARM"
        }"###;
        let parsed = parse_github_release_json(payload).expect("valid release JSON");
        assert_eq!(parsed.tag, "v1.1.0");
        assert_eq!(parsed.version, "1.1.0");
        assert!(is_newer_version("1.0.0", &parsed.tag));
        assert!(parsed.notes_summary.contains("Mejora de Seed Map"));
    }

    #[test]
    fn rejects_drafts_or_prereleases() {
        let draft = r#"{"tag_name": "v2.0.0", "draft": true, "prerelease": false}"#;
        let pre = r#"{"tag_name": "v2.0.0-beta", "draft": false, "prerelease": true}"#;
        assert!(parse_github_release_json(draft).is_none());
        assert!(parse_github_release_json(pre).is_none());
    }
}
