/// Plugin community registry — GitHub-driven online plugin discovery and installation.
///
/// Plugins are distributed as ZIP files in the main SerialRUN repo's Releases.
/// Each ZIP contains `plugin.json` + platform-specific DLLs.
/// Community search fetches Releases from `YaoIsAI/SerialRUN` and extracts plugin info.

use std::path::PathBuf;
use serde::Deserialize;
use crate::plugin_install::{PluginManager, InstalledPlugin};
use serialrun_plugin_api::manifest::PluginManifest;

const GITHUB_API: &str = "https://api.github.com";
const PLUGINS_REPO: &str = "YaoIsAI/serialrun-plugins";

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
struct GhContent {
    content: Option<String>,
    encoding: Option<String>,
}

#[derive(Deserialize)]
struct GhDirEntry {
    name: String,
    #[serde(rename = "type")]
    type_field: String,
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

    /// Search for local plugins in the source plugins directory.
    /// Returns plugins that exist in the source tree but aren't installed yet.
    pub fn search_local(&self, query: &str) -> Vec<RegistryPlugin> {
        let mut plugins = Vec::new();
        let mut seen_names = std::collections::HashSet::new();

        // Search in the source plugins directory
        let source_dirs = [
            std::path::PathBuf::from("plugins"),
            std::env::current_dir().unwrap_or_default().join("plugins"),
        ];

        for source_dir in &source_dirs {
            if !source_dir.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(source_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    let manifest_path = path.join("plugin.json");
                    if !manifest_path.exists() {
                        continue;
                    }

                    if let Ok(json) = std::fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = PluginManifest::from_json(&json) {
                            // Deduplicate by name
                            if !seen_names.insert(manifest.name.clone()) {
                                continue;
                            }

                            // Filter by query
                            if !query.is_empty() {
                                let q = query.to_lowercase();
                                if !manifest.name.to_lowercase().contains(&q)
                                    && !manifest.description.to_lowercase().contains(&q)
                                    && !manifest.tags.iter().any(|t| t.to_lowercase().contains(&q)) {
                                    continue;
                                }
                            }

                            plugins.push(RegistryPlugin {
                                repo_name: format!("local/{}", manifest.name),
                                repo_url: path.to_string_lossy().to_string(),
                                description: manifest.description.clone(),
                                stars: 0,
                                topics: manifest.tags.clone(),
                                manifest: Some(manifest.clone()),
                                latest_release: None,
                                installed_version: None,
                            });
                        }
                    }
                }
            }
        }

        plugins
    }

    /// Search for plugins from the main SerialRUN repo's releases.
    /// Fetches plugin manifests from the `plugins/` directory on GitHub,
    /// then matches them with release assets for download.
    pub async fn search(&self, query: &str) -> anyhow::Result<Vec<RegistryPlugin>> {
        // 1. List plugin directories in the main repo's plugins/ folder
        let plugins_dir_url = format!("{}/repos/{}/contents/plugins", GITHUB_API, PLUGINS_REPO);
        let dir_resp = self.client.get(&plugins_dir_url).send().await?;

        if !dir_resp.status().is_success() {
            return Ok(Vec::new());
        }

        let entries: Vec<GhDirEntry> = dir_resp.json().await?;

        // 2. For each plugin dir, fetch its plugin.json
        let mut plugins = Vec::new();
        for entry in &entries {
            if entry.type_field != "dir" {
                continue;
            }

            let manifest = self.fetch_manifest(PLUGINS_REPO, &entry.name).await;
            if let Some(manifest) = manifest {
                // Filter by query
                if !query.is_empty() {
                    let q = query.to_lowercase();
                    if !manifest.name.to_lowercase().contains(&q)
                        && !manifest.description.to_lowercase().contains(&q)
                        && !manifest.tags.iter().any(|t| t.to_lowercase().contains(&q))
                    {
                        continue;
                    }
                }

                // Find matching release asset
                let latest_release = self.get_release_with_plugin(&manifest.name).await.ok();

                plugins.push(RegistryPlugin {
                    repo_name: entry.name.clone(),
                    repo_url: format!("https://github.com/{}", PLUGINS_REPO),
                    description: manifest.description.clone(),
                    stars: 0,
                    topics: manifest.tags.clone(),
                    manifest: Some(manifest),
                    latest_release,
                    installed_version: None,
                });
            }
        }

        Ok(plugins)
    }

    /// Get details for a specific plugin by name.
    pub async fn get_plugin(&self, plugin_name: &str) -> anyhow::Result<RegistryPlugin> {
        // Fetch the specific plugin's manifest from the plugins repo
        if let Some(manifest) = self.fetch_manifest(PLUGINS_REPO, plugin_name).await {
            let latest_release = self.get_release_with_plugin(&manifest.name).await.ok();
            return Ok(RegistryPlugin {
                repo_name: plugin_name.to_string(),
                repo_url: format!("https://github.com/{}", PLUGINS_REPO),
                description: manifest.description.clone(),
                stars: 0,
                topics: manifest.tags.clone(),
                manifest: Some(manifest),
                latest_release,
                installed_version: None,
            });
        }

        Err(anyhow::anyhow!("Plugin '{}' not found", plugin_name))
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

    /// Fetch plugin.json from a repo's plugins/ subdirectory.
    /// `plugin_dir` is the directory name under `plugins/` (e.g., "serialrun-mpy-ide").
    async fn fetch_manifest(&self, repo: &str, plugin_dir: &str) -> Option<PluginManifest> {
        let url = format!("{}/repos/{}/contents/plugins/{}/plugin.json", GITHUB_API, repo, plugin_dir);
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

    /// Find a release that contains a ZIP asset matching the plugin name.
    async fn get_release_with_plugin(&self, plugin_name: &str) -> anyhow::Result<Release> {
        let url = format!("{}/repos/{}/releases?per_page=10", GITHUB_API, PLUGINS_REPO);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("No releases found"));
        }

        let releases: Vec<GhRelease> = resp.json().await?;

        // Find a release whose assets contain a ZIP matching the plugin name
        for release in &releases {
            let has_plugin = release.assets.iter().any(|a| {
                let name_lower = a.name.to_lowercase();
                name_lower.contains(&plugin_name.to_lowercase()) && name_lower.ends_with(".zip")
            });
            if has_plugin {
                return Ok(Release {
                    tag: release.tag_name.clone(),
                    assets: release.assets.iter().map(|a| ReleaseAsset {
                        name: a.name.clone(),
                        download_url: a.browser_download_url.clone(),
                        size: a.size,
                    }).collect(),
                });
            }
        }

        // Fallback: return latest release if it has any ZIP
        if let Some(release) = releases.first() {
            let has_zip = release.assets.iter().any(|a| a.name.ends_with(".zip"));
            if has_zip {
                return Ok(Release {
                    tag: release.tag_name.clone(),
                    assets: release.assets.iter().map(|a| ReleaseAsset {
                        name: a.name.clone(),
                        download_url: a.browser_download_url.clone(),
                        size: a.size,
                    }).collect(),
                });
            }
        }

        Err(anyhow::anyhow!("No release with plugin ZIP found"))
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
