//! GitHub Releases based update checking, download and checksum verification.

use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub const REPO_OWNER: &str = "YaoIsAI";
pub const REPO_NAME: &str = "SerialRUN";
pub const RELEASES_URL: &str = "https://github.com/YaoIsAI/SerialRUN/releases";
pub const LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/YaoIsAI/SerialRUN/releases/latest";
pub const CHECKSUM_ASSET_NAME: &str = "SHA256SUMS";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseAsset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub version: String,
    pub tag: String,
    pub title: String,
    pub body: String,
    pub html_url: String,
    pub published_at: String,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone)]
pub struct UpdateCandidate {
    pub release: ReleaseInfo,
    pub asset: ReleaseAsset,
    pub checksum_asset: Option<ReleaseAsset>,
}

#[derive(Debug, Clone)]
pub enum UpdateCheckOutcome {
    UpToDate,
    Available(UpdateCandidate),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadOutcome {
    pub path: PathBuf,
    pub checksum_verified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdatePlatform {
    WindowsX64,
    MacosArm64,
    MacosX64,
    LinuxX64,
}

impl UpdatePlatform {
    /// Preference order: auto-installable package first, manual package second.
    fn asset_suffixes(self) -> &'static [&'static str] {
        match self {
            UpdatePlatform::WindowsX64 => {
                &["-windows-x64-setup.exe", "-windows-x64.zip"]
            }
            UpdatePlatform::MacosArm64 => &["-macos-arm64.zip", "-macos-arm64.dmg"],
            UpdatePlatform::MacosX64 => &["-macos-x64.zip", "-macos-x64.dmg"],
            UpdatePlatform::LinuxX64 => &["-linux-x64.tar.gz", "_amd64.deb"],
        }
    }
}

pub fn current_platform() -> Option<UpdatePlatform> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Some(UpdatePlatform::WindowsX64),
        ("macos", "aarch64") => Some(UpdatePlatform::MacosArm64),
        ("macos", "x86_64") => Some(UpdatePlatform::MacosX64),
        ("linux", "x86_64") => Some(UpdatePlatform::LinuxX64),
        _ => None,
    }
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

/// Query GitHub for the latest published release and pick the asset for this platform.
pub fn check_for_update(current_version: &str) -> Result<UpdateCheckOutcome, String> {
    let platform = current_platform().ok_or_else(|| {
        format!(
            "Automatic updates are not supported on {} {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        )
    })?;
    let client = http_client()?;
    let response = client
        .get(LATEST_RELEASE_API)
        .header("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(30))
        .send()
        .map_err(friendly_network_error)?;

    let status = response.status();
    if !status.is_success() {
        return Err(http_error("Checking for updates", status));
    }
    let release: GithubRelease = response
        .json()
        .map_err(|e| format!("Invalid response from GitHub: {}", e))?;

    evaluate_release(release, current_version, platform)
}

fn evaluate_release(
    release: GithubRelease,
    current_version: &str,
    platform: UpdatePlatform,
) -> Result<UpdateCheckOutcome, String> {
    let latest = release
        .tag_name
        .trim()
        .trim_start_matches(|c| c == 'v' || c == 'V');
    let latest_version = Version::parse(latest)
        .map_err(|e| format!("Invalid release version '{}': {}", release.tag_name, e))?;
    let current_version = current_version
        .trim()
        .trim_start_matches(|c| c == 'v' || c == 'V');
    let current = Version::parse(current_version)
        .map_err(|e| format!("Invalid current version '{}': {}", current_version, e))?;

    if latest_version <= current {
        return Ok(UpdateCheckOutcome::UpToDate);
    }

    let assets: Vec<ReleaseAsset> = release
        .assets
        .iter()
        .map(|a| ReleaseAsset {
            name: a.name.clone(),
            url: a.browser_download_url.clone(),
            size: a.size,
        })
        .collect();

    let asset = select_asset(&assets, platform).ok_or_else(|| {
        format!(
            "Release v{} has no package for this platform. Download it manually from {}",
            latest_version, RELEASES_URL
        )
    })?;
    let checksum_asset = assets
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case(CHECKSUM_ASSET_NAME))
        .cloned();

    Ok(UpdateCheckOutcome::Available(UpdateCandidate {
        release: ReleaseInfo {
            version: latest_version.to_string(),
            tag: release.tag_name,
            title: release.name.unwrap_or_default(),
            body: release.body.unwrap_or_default(),
            html_url: release.html_url,
            published_at: release.published_at.unwrap_or_default(),
            assets,
        },
        asset,
        checksum_asset,
    }))
}

pub fn select_asset(
    assets: &[ReleaseAsset],
    platform: UpdatePlatform,
) -> Option<ReleaseAsset> {
    for suffix in platform.asset_suffixes() {
        if let Some(asset) = assets.iter().find(|a| {
            let lower = a.name.to_ascii_lowercase();
            lower.starts_with("serialrun") && lower.ends_with(suffix)
        }) {
            return Some(asset.clone());
        }
    }
    None
}

/// Download the selected asset into `dest_dir`, verifying SHA256 when the release
/// publishes a SHA256SUMS file.
pub fn download_update(
    candidate: &UpdateCandidate,
    dest_dir: &Path,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(u64, u64),
) -> Result<DownloadOutcome, String> {
    let file_name = Path::new(&candidate.asset.name)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid asset file name".to_string())?;
    std::fs::create_dir_all(dest_dir)
        .map_err(|e| format!("Cannot create update directory: {}", e))?;
    let dest = dest_dir.join(file_name);
    let part = dest.with_extension(format!(
        "{}.part",
        dest.extension().and_then(|e| e.to_str()).unwrap_or("download")
    ));

    let expected_checksum = match &candidate.checksum_asset {
        Some(asset) => {
            let text = fetch_text(asset)?;
            Some(
                checksum_for(&text, file_name).ok_or_else(|| {
                    format!("{} does not list {}", CHECKSUM_ASSET_NAME, file_name)
                })?,
            )
        }
        None => None,
    };

    let client = http_client()?;
    let mut response = client
        .get(&candidate.asset.url)
        .timeout(Duration::from_secs(30 * 60))
        .send()
        .map_err(friendly_network_error)?;
    let status = response.status();
    if !status.is_success() {
        return Err(http_error("Downloading update", status));
    }

    let total = response
        .content_length()
        .filter(|n| *n > 0)
        .unwrap_or(candidate.asset.size);
    let mut file = std::fs::File::create(&part)
        .map_err(|e| format!("Cannot create download file: {}", e))?;
    let mut downloaded = 0u64;
    let mut buf = [0u8; 64 * 1024];
    on_progress(0, total);
    loop {
        if cancel.load(Ordering::Relaxed) {
            drop(file);
            let _ = std::fs::remove_file(&part);
            return Err("Download cancelled".to_string());
        }
        let read = response
            .read(&mut buf)
            .map_err(|e| format!("Download interrupted: {}", e))?;
        if read == 0 {
            break;
        }
        file.write_all(&buf[..read])
            .map_err(|e| format!("Cannot write update file: {}", e))?;
        downloaded += read as u64;
        on_progress(downloaded, total);
    }
    file.flush()
        .map_err(|e| format!("Cannot flush update file: {}", e))?;
    drop(file);

    if total > 0 && downloaded != total {
        let _ = std::fs::remove_file(&part);
        return Err(format!(
            "Download incomplete (got {} of {} bytes)",
            downloaded, total
        ));
    }

    if let Some(ref expected) = expected_checksum {
        if let Err(e) = verify_sha256(&part, expected) {
            let _ = std::fs::remove_file(&part);
            return Err(format!("Checksum verification failed: {}", e));
        }
    }

    if dest.exists() {
        std::fs::remove_file(&dest)
            .map_err(|e| format!("Cannot replace old download: {}", e))?;
    }
    std::fs::rename(&part, &dest)
        .map_err(|e| format!("Cannot finalize download: {}", e))?;

    Ok(DownloadOutcome {
        path: dest,
        checksum_verified: expected_checksum.is_some(),
    })
}

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent(format!(
            "SerialRUN/{} (+https://github.com/{}/{})",
            env!("CARGO_PKG_VERSION"),
            REPO_OWNER,
            REPO_NAME
        ))
        .connect_timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Cannot initialize network client: {}", e))
}

fn fetch_text(asset: &ReleaseAsset) -> Result<String, String> {
    let client = http_client()?;
    let response = client
        .get(&asset.url)
        .timeout(Duration::from_secs(30))
        .send()
        .map_err(friendly_network_error)?;
    let status = response.status();
    if !status.is_success() {
        return Err(http_error("Downloading checksums", status));
    }
    response
        .text()
        .map_err(|e| format!("Cannot read checksums: {}", e))
}

fn http_error(context: &str, status: reqwest::StatusCode) -> String {
    match status.as_u16() {
        403 | 429 => format!(
            "GitHub API rate limit reached (HTTP {}). Try again later, or download manually from {}",
            status.as_u16(),
            RELEASES_URL
        ),
        404 => format!(
            "No published release was found. Open {} to check manually",
            RELEASES_URL
        ),
        _ => format!("{} failed (HTTP {})", context, status),
    }
}

fn friendly_network_error(error: reqwest::Error) -> String {
    if error.is_timeout() {
        "Network timeout while contacting GitHub. Check your connection and retry.".to_string()
    } else if error.is_connect() {
        format!(
            "Cannot connect to GitHub. Check your network or proxy settings, or download manually from {}",
            RELEASES_URL
        )
    } else {
        format!("Network error while contacting GitHub: {}", error)
    }
}

/// Find the SHA256 digest for `file_name` in a `sha256sum`-style file.
pub fn checksum_for(text: &str, file_name: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let name = parts.next()?.trim_start_matches('*');
        if name == file_name
            && hash.len() == 64
            && hash.chars().all(|c| c.is_ascii_hexdigit())
        {
            Some(hash.to_ascii_lowercase())
        } else {
            None
        }
    })
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file =
        std::fs::File::open(path).map_err(|e| format!("Cannot open file for checksum: {}", e))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buf)
            .map_err(|e| format!("Cannot read file for checksum: {}", e))?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn verify_sha256(path: &Path, expected_hex: &str) -> Result<(), String> {
    let actual = sha256_file(path)?;
    if actual.eq_ignore_ascii_case(expected_hex.trim()) {
        Ok(())
    } else {
        Err(format!(
            "expected {}, got {}",
            expected_hex.trim(),
            actual
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(name: &str) -> ReleaseAsset {
        ReleaseAsset {
            name: name.to_string(),
            url: format!("https://example.com/{}", name),
            size: 1,
        }
    }

    #[test]
    fn selects_platform_specific_assets() {
        let assets = vec![
            asset("serialrun-0.5.0-windows-x64.zip"),
            asset("serialrun-0.5.0-windows-x64-setup.exe"),
            asset("serialrun-0.5.0-macos-arm64.dmg"),
            asset("serialrun-0.5.0-macos-arm64.zip"),
            asset("serialrun-0.5.0-macos-x64.zip"),
            asset("serialrun_0.5.0_amd64.deb"),
            asset("serialrun-0.5.0-linux-x64.tar.gz"),
        ];
        assert_eq!(
            select_asset(&assets, UpdatePlatform::WindowsX64)
                .unwrap()
                .name,
            "serialrun-0.5.0-windows-x64-setup.exe"
        );
        assert_eq!(
            select_asset(&assets, UpdatePlatform::MacosArm64)
                .unwrap()
                .name,
            "serialrun-0.5.0-macos-arm64.zip"
        );
        assert_eq!(
            select_asset(&assets, UpdatePlatform::LinuxX64)
                .unwrap()
                .name,
            "serialrun-0.5.0-linux-x64.tar.gz"
        );
    }

    #[test]
    fn parses_checksum_lines() {
        let text = "\
0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  serialrun-0.5.0-linux-x64.tar.gz
abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789 *serialrun-0.5.0-windows-x64.zip
not-a-hash  file.txt
";
        assert_eq!(
            checksum_for(text, "serialrun-0.5.0-linux-x64.tar.gz").unwrap(),
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        );
        assert_eq!(
            checksum_for(text, "serialrun-0.5.0-windows-x64.zip").unwrap(),
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
        );
        assert!(checksum_for(text, "file.txt").is_none());
        assert!(checksum_for(text, "missing.zip").is_none());
    }

    #[test]
    fn compares_versions_semantically() {
        let release = |tag: &str| GithubRelease {
            tag_name: tag.to_string(),
            name: None,
            body: None,
            html_url: String::new(),
            published_at: None,
            assets: vec![GithubAsset {
                name: "serialrun-9.9.9-linux-x64.tar.gz".into(),
                browser_download_url: String::new(),
                size: 0,
            }],
        };
        assert!(matches!(
            evaluate_release(release("v0.4.10"), "0.4.9", UpdatePlatform::LinuxX64),
            Ok(UpdateCheckOutcome::Available(_))
        ));
        assert!(matches!(
            evaluate_release(release("v0.4.6"), "0.4.6", UpdatePlatform::LinuxX64),
            Ok(UpdateCheckOutcome::UpToDate)
        ));
        assert!(matches!(
            evaluate_release(release("v0.4.5"), "0.4.6", UpdatePlatform::LinuxX64),
            Ok(UpdateCheckOutcome::UpToDate)
        ));
    }
}
