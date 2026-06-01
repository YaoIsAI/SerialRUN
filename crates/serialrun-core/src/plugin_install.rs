/// Plugin manager for install, uninstall, enable, disable operations.
///
/// Manages plugins in ~/.serialrun/plugins/ directory.
/// Each plugin is a subdirectory containing the binary and plugin.json.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serialrun_plugin_api::manifest::{PluginManifest, current_platform, supports_current_platform};
use crate::plugin::{LoadedPlugin, PluginError, PluginResult};

/// Plugin installation state
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstalledPlugin {
    pub manifest: PluginManifest,
    pub enabled: bool,
    pub install_path: PathBuf,
}

/// Manages plugin installation, removal, and state
pub struct PluginManager {
    plugins_dir: PathBuf,
    state_file: PathBuf,
    installed: HashMap<String, InstalledPlugin>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        let home_path = PathBuf::from(home);
        let plugins_dir = home_path.join(".serialrun").join("plugins");
        let state_file = home_path.join(".serialrun").join("plugin_state.json");

        let mut manager = Self {
            plugins_dir,
            state_file,
            installed: HashMap::new(),
        };
        manager.load_state();
        manager
    }

    /// Create with custom plugins directory
    pub fn with_dir(plugins_dir: PathBuf) -> Self {
        let state_file = plugins_dir.parent()
            .unwrap_or(&PathBuf::from("."))
            .join("plugin_state.json");

        let mut manager = Self {
            plugins_dir,
            state_file,
            installed: HashMap::new(),
        };
        manager.load_state();
        manager
    }

    /// Get the plugins directory
    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    /// Get all installed plugins
    pub fn installed(&self) -> &HashMap<String, InstalledPlugin> {
        &self.installed
    }

    /// Get a specific installed plugin
    pub fn get(&self, name: &str) -> Option<&InstalledPlugin> {
        self.installed.get(name)
    }

    /// Install a plugin from a zip file using Rust native zip extraction
    pub fn install_from_zip(&mut self, zip_path: &Path) -> PluginResult<String> {
        log::info!("Installing plugin from: {}", zip_path.display());

        // Create unique temp directory per install
        let temp_dir = std::env::temp_dir().join(format!(
            "serialrun_plugin_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_millis()
        ));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir)
            .map_err(|e| PluginError::IoError(e))?;

        // Extract using Rust zip crate (no external tools needed)
        let zip_file = fs::File::open(zip_path)
            .map_err(|e| PluginError::IoError(e))?;
        let mut archive = zip::ZipArchive::new(zip_file)
            .map_err(|e| PluginError::PluginError(format!("Invalid zip file: {}", e)))?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)
                .map_err(|e| PluginError::PluginError(format!("Zip read error: {}", e)))?;
            let out_path = temp_dir.join(entry.mangled_name());

            if entry.is_dir() {
                fs::create_dir_all(&out_path).map_err(|e| PluginError::IoError(e))?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| PluginError::IoError(e))?;
                }
                let mut out_file = fs::File::create(&out_path)
                    .map_err(|e| PluginError::IoError(e))?;
                std::io::copy(&mut entry, &mut out_file)
                    .map_err(|e| PluginError::IoError(e))?;
            }
        }

        log::info!("Extracted to: {}", temp_dir.display());

        // Find plugin.json in extracted files
        let manifest_path = Self::find_plugin_json(&temp_dir)
            .ok_or_else(|| {
                let _ = fs::remove_dir_all(&temp_dir);
                PluginError::PluginError("No plugin.json found in zip".to_string())
            })?;
        log::info!("Found plugin.json at: {}", manifest_path.display());

        let manifest_json = fs::read_to_string(&manifest_path)
            .map_err(|e| PluginError::IoError(e))?;
        let manifest = PluginManifest::from_json(&manifest_json)
            .map_err(|e| PluginError::PluginError(format!("Invalid plugin.json: {}", e)))?;

        // Check platform compatibility
        if !supports_current_platform(&manifest) {
            let _ = fs::remove_dir_all(&temp_dir);
            return Err(PluginError::PluginError(format!(
                "Plugin '{}' does not support platform {}",
                manifest.name,
                current_platform()
            )));
        }

        // Move to final location (with copy fallback for cross-filesystem)
        let plugin_dir = self.plugins_dir.join(&manifest.name);
        if plugin_dir.exists() {
            fs::remove_dir_all(&plugin_dir)
                .map_err(|e| PluginError::IoError(e))?;
        }
        if fs::rename(&temp_dir, &plugin_dir).is_err() {
            // Cross-filesystem: copy then delete
            copy_dir_all(&temp_dir, &plugin_dir)
                .map_err(|e| PluginError::IoError(e))?;
            let _ = fs::remove_dir_all(&temp_dir);
        }

        // Record installation
        let installed = InstalledPlugin {
            manifest: manifest.clone(),
            enabled: true,
            install_path: plugin_dir,
        };
        self.installed.insert(manifest.name.clone(), installed);
        self.save_state();

        Ok(manifest.name)
    }

    /// Find plugin.json recursively in a directory
    fn find_plugin_json(dir: &Path) -> Option<PathBuf> {
        if dir.join("plugin.json").exists() {
            return Some(dir.join("plugin.json"));
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(p) = Self::find_plugin_json(&entry.path()) {
                        return Some(p);
                    }
                }
            }
        }
        None
    }

    /// Install a plugin from a directory (for development)
    pub fn install_from_dir(&mut self, dir: &Path) -> PluginResult<String> {
        let manifest_path = dir.join("plugin.json");
        if !manifest_path.exists() {
            return Err(PluginError::PluginError("No plugin.json found".to_string()));
        }

        let manifest_json = fs::read_to_string(&manifest_path)
            .map_err(|e| PluginError::IoError(e))?;
        let manifest: PluginManifest = PluginManifest::from_json(&manifest_json)
            .map_err(|e| PluginError::PluginError(format!("Invalid plugin.json: {}", e)))?;

        let installed = InstalledPlugin {
            manifest: manifest.clone(),
            enabled: true,
            install_path: dir.to_path_buf(),
        };
        self.installed.insert(manifest.name.clone(), installed);
        self.save_state();

        Ok(manifest.name)
    }

    /// Uninstall a plugin
    pub fn uninstall(&mut self, name: &str) -> PluginResult<()> {
        let plugin = self.installed.remove(name)
            .ok_or_else(|| PluginError::PluginError(format!("Plugin '{}' not found", name)))?;

        // Remove plugin directory
        if plugin.install_path.exists() {
            fs::remove_dir_all(&plugin.install_path)
                .map_err(|e| PluginError::IoError(e))?;
        }

        self.save_state();
        Ok(())
    }

    /// Enable a plugin
    pub fn enable(&mut self, name: &str) -> bool {
        if let Some(plugin) = self.installed.get_mut(name) {
            plugin.enabled = true;
            self.save_state();
            true
        } else {
            false
        }
    }

    /// Disable a plugin
    pub fn disable(&mut self, name: &str) -> bool {
        if let Some(plugin) = self.installed.get_mut(name) {
            plugin.enabled = false;
            self.save_state();
            true
        } else {
            false
        }
    }

    /// Check if a plugin is enabled
    pub fn is_enabled(&self, name: &str) -> bool {
        self.installed.get(name).map_or(false, |p| p.enabled)
    }

    /// Discover all installed plugins from the plugins directory
    pub fn discover(&mut self) {
        if !self.plugins_dir.exists() {
            return;
        }

        let entries: Vec<std::fs::DirEntry> = match fs::read_dir(&self.plugins_dir) {
            Ok(e) => e.flatten().collect(),
            Err(_) => return,
        };

        for entry in entries {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            // Look for plugin.json directly or in subdirectories
            let manifest_path = Self::find_plugin_json_in_dir(&path);
            if let Some(manifest_path) = manifest_path {
                if let Ok(json) = fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) = PluginManifest::from_json(&json) {
                        let plugin_dir = manifest_path.parent().unwrap_or(&path).to_path_buf();
                        let enabled = self.installed.get(&manifest.name)
                            .map(|p| p.enabled)
                            .unwrap_or(true);

                        self.installed.insert(manifest.name.clone(), InstalledPlugin {
                            manifest,
                            enabled,
                            install_path: plugin_dir,
                        });
                    }
                }
            }
        }
    }

    /// Find plugin.json in a directory recursively
    fn find_plugin_json_in_dir(dir: &Path) -> Option<PathBuf> {
        Self::find_plugin_json(dir)
    }

    /// Load plugin state from disk
    fn load_state(&mut self) {
        if let Ok(json) = fs::read_to_string(&self.state_file) {
            if let Ok(state) = serde_json::from_str::<HashMap<String, InstalledPlugin>>(&json) {
                self.installed = state;
            }
        }
    }

    /// Save plugin state to disk
    fn save_state(&self) {
        if let Some(parent) = self.state_file.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                log::error!("Failed to create plugin state directory: {}", e);
            }
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.installed) {
            if let Err(e) = fs::write(&self.state_file, json) {
                log::error!("Failed to save plugin state: {}", e);
            }
        }
    }
}

/// Recursively copy a directory (fallback for cross-filesystem rename)
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_new() {
        let manager = PluginManager::new();
        assert!(manager.installed().is_empty());
    }

    #[test]
    fn test_plugin_manager_custom_dir() {
        let dir = std::env::temp_dir().join("test_plugins");
        let manager = PluginManager::with_dir(dir);
        assert!(manager.installed().is_empty());
    }
}
