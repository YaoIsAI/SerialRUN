/// Plugin community registry — GitHub-driven online plugin discovery and installation.
///
/// Uses GitHub Search API to find plugins tagged with `serialrun-plugin` topic,
/// and GitHub Releases to download pre-built binaries.

use std::path::PathBuf;
use serde::Deserialize;
use crate::plugin_install::{PluginManager, InstalledPlugin};
use serialrun_plugin_api::manifest::PluginManifest;

const GITHUB_API: &str = "https://api.github.com";
const SEARCH_TOPIC: &str = "serialrun-plugin";

/// A plugin entry from GitHub search results.
#[derive(Debug, Clone)]
pub struct RegistryPlugin {
    pub repo_name: String,          // "owner/repo"
    pub repo_url: String,           // "https://github.com/owner/repo"
    pub description: String,
    pub stars: u32,
    pub topics: Vec<String>,
    pub manifest: Option<PluginManifest>,
    pub latest_release: Option<Release>,
    pub installed_version: Option<String>, // If already installed locally
}

/// A GitHub release.
#[derive(Debug, Clone)]
pub struct Release {
    pub tag: String,
    pub assets: Vec<ReleaseAsset>,
}

/// A release asset (downloadable file).
#[derive(Debug, Clone)]
pub struct ReleaseAsset {
    pub name: String,
    pub download_url: String,
    pub size: u64,
}

// --- GitHub API response types ---

#[derive(Deserialize)]
struct GhSearchResponse {
    items: Vec<GhRepo>,
    total_count: u32,
}

#[derive(Deserialize)]
struct GhRepo {
    full_name: String,
    html_url: String,
    description: Option<String>,
    stargazers_count: u32,
    topics: Option<Vec<String>>,
    default_branch: Option<String>,
}

#[derive(Deserialize)]
struct GhContent {
    content: Option<String>,
    encoding: Option<String>,
}

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    assets: Vec<GhReleaseAsset>,
}

#[derive(Deserialize)]
struct GhReleaseAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

/// Plugin community registry client.
pub struct PluginRegistry {
    client: reqwest::Client,
}

impl PluginRegistry {
    /// Create a new registry client.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("serialrun-plugin-registry")
            .build()
            .unwrap_or_default();
        Self { client }
    }

    /// Search for plugins on GitHub by query string.
    /// Returns plugins tagged with `serialrun-plugin` topic, sorted by stars.
    pub async fn search(&self, query: &str) -> anyhow::Result<Vec<RegistryPlugin>> {
        let q = if query.is_empty() {
            format!("topic:{}", SEARCH_TOPIC)
        } else {
            format!("topic:{}+{}", SEARCH_TOPIC, query)
        };

        let url = format!("{}/search/repositories?q={}&sort=stars&per_page=20", GITHUB_API, q);
        let resp = self.client.get(&url).send().await?;
        let search: GhSearchResponse = resp.json().await?;

        let mut plugins = Vec::new();
        for repo in search.items {
            let plugin = self.enrich_repo(repo).await;
            plugins.push(plugin);
        }

        Ok(plugins)
    }

    /// Get details for a specific plugin by repo name (owner/repo).
    pub async fn get_plugin(&self, repo: &str) -> anyhow::Result<RegistryPlugin> {
        let url = format!("{}/repos/{}", GITHUB_API, repo);
        let resp = self.client.get(&url).send().await?;
        let repo_data: GhRepo = resp.json().await?;
        Ok(self.enrich_repo(repo_data).await)
    }

    /// Download and install a plugin from a GitHub release.
    /// Returns the installed plugin name.
    pub async fn install(&self, plugin: &RegistryPlugin) -> anyhow::Result<String> {
        let release = plugin.latest_release.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No release available for {}", plugin.repo_name))?;

        // Find platform-matched asset
        let platform = serialrun_plugin_api::manifest::current_platform();
        let asset = release.assets.iter().find(|a| {
            let name_lower = a.name.to_lowercase();
            match platform {
                "windows-x64" => name_lower.contains("windows") || name_lower.ends_with(".zip"),
                "macos-arm64" | "macos-x64" => name_lower.contains("macos") || name_lower.contains("darwin"),
                "linux-x64" | "linux-arm64" => name_lower.contains("linux"),
                _ => true,
            }
        }).or_else(|| release.assets.first())
            .ok_or_else(|| anyhow::anyhow!("No downloadable asset found"))?;

        // Download to temp file
        let resp = self.client.get(&asset.download_url).send().await?;
        let bytes = resp.bytes().await?;

        let temp_zip = std::env::temp_dir().join(format!(
            "serialrun_plugin_download_{}.zip",
            std::process::id()
        ));
        std::fs::write(&temp_zip, &bytes)?;

        // Install via PluginManager
        let mut mgr = PluginManager::new();
        let name = mgr.install_from_zip(&temp_zip)?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_zip);

        Ok(name)
    }

    /// Check which installed plugins have updates available.
    pub async fn check_updates(&self, installed: &[InstalledPlugin]) -> anyhow::Result<Vec<(String, String)>> {
        let mut updates = Vec::new();

        for plugin in installed {
            if !plugin.manifest.repository.is_empty() {
                if let Some(repo_name) = extract_repo_name(&plugin.manifest.repository) {
                    if let Ok(latest) = self.get_latest_release(&repo_name).await {
                        if latest.tag != plugin.manifest.version {
                            updates.push((plugin.manifest.name.clone(), latest.tag));
                        }
                    }
                }
            }
        }

        Ok(updates)
    }

    /// Get the latest release tag for a repo.
    async fn get_latest_release(&self, repo: &str) -> anyhow::Result<Release> {
        let url = format!("{}/repos/{}/releases/latest", GITHUB_API, repo);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("No releases found"));
        }

        let release: GhRelease = resp.json().await?;
        Ok(Release {
            tag: release.tag_name,
            assets: release.assets.into_iter().map(|a| ReleaseAsset {
                name: a.name,
                download_url: a.browser_download_url,
                size: a.size,
            }).collect(),
        })
    }

    /// Enrich a repo with manifest and release info.
    async fn enrich_repo(&self, repo: GhRepo) -> RegistryPlugin {
        let manifest = self.fetch_manifest(&repo.full_name, repo.default_branch.as_deref().unwrap_or("main")).await;
        let latest_release = self.get_latest_release(&repo.full_name).await.ok();

        RegistryPlugin {
            repo_name: repo.full_name,
            repo_url: repo.html_url,
            description: repo.description.unwrap_or_default(),
            stars: repo.stargazers_count,
            topics: repo.topics.unwrap_or_default(),
            manifest,
            latest_release,
            installed_version: None,
        }
    }

    /// Fetch plugin.json from a repo's default branch.
    async fn fetch_manifest(&self, repo: &str, branch: &str) -> Option<PluginManifest> {
        // Try root first
        let url = format!("{}/repos/{}/contents/plugin.json?ref={}", GITHUB_API, repo, branch);
        if let Ok(resp) = self.client.get(&url).send().await {
            if resp.status().is_success() {
                if let Ok(content) = resp.json::<GhContent>().await {
                    if let Some(encoded) = content.content {
                        use base64::Engine;
                        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(encoded.replace('\n', "")) {
                            if let Ok(json) = String::from_utf8(bytes) {
                                if let Ok(manifest) = PluginManifest::from_json(&json) {
                                    return Some(manifest);
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

/// Extract "owner/repo" from a GitHub URL.
fn extract_repo_name(url: &str) -> Option<String> {
    let url = url.trim_end_matches('/');
    let url = url.trim_end_matches(".git");
    // Handle both https://github.com/owner/repo and github.com/owner/repo
    let path = url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let path = path.strip_prefix("github.com/").unwrap_or(path);
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 {
        Some(format!("{}/{}", parts[0], parts[1]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_repo_name() {
        assert_eq!(extract_repo_name("https://github.com/user/repo"), Some("user/repo".into()));
        assert_eq!(extract_repo_name("https://github.com/user/repo.git"), Some("user/repo".into()));
        assert_eq!(extract_repo_name("github.com/user/repo"), Some("user/repo".into()));
        assert_eq!(extract_repo_name("not-a-url"), None);
    }
}
