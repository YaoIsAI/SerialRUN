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

    /// Install a plugin from a zip file
    pub fn install_from_zip(&mut self, zip_path: &Path) -> PluginResult<String> {
        // Extract zip to temp directory first to read manifest
        let temp_dir = std::env::temp_dir().join("serialrun_plugin_install");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir)
            .map_err(|e| PluginError::IoError(e))?;

        // Try multiple extraction methods
        let mut extracted = false;

        // Method 1: PowerShell Expand-Archive (Windows 10+)
        if !extracted {
            let status = std::process::Command::new("powershell")
                .args(["-Command", &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    zip_path.display(), temp_dir.display()
                )])
                .status();
            if let Ok(s) = status {
                extracted = s.success();
            }
        }

        // Method 2: tar (available on Windows 10+ and Unix)
        if !extracted {
            let status = std::process::Command::new("tar")
                .args(["xf", zip_path.to_str().unwrap(), "-C", temp_dir.to_str().unwrap()])
                .status();
            if let Ok(s) = status {
                extracted = s.success();
            }
        }

        // Method 3: unzip (Unix)
        if !extracted {
            let status = std::process::Command::new("unzip")
                .arg("-o")
                .arg(zip_path)
                .arg("-d")
                .arg(&temp_dir)
                .status();
            if let Ok(s) = status {
                extracted = s.success();
            }
        }

        if !extracted {
            let _ = fs::remove_dir_all(&temp_dir);
            return Err(PluginError::PluginError(
                "Failed to extract zip. No supported extraction tool found.".to_string()
            ));
        }

        // Find plugin.json in extracted files
        let manifest_path = Self::find_plugin_json(&temp_dir)
            .ok_or_else(|| {
                let _ = fs::remove_dir_all(&temp_dir);
                PluginError::PluginError("No plugin.json found in zip".to_string())
            })?;

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

        // Move to final location
        let plugin_dir = self.plugins_dir.join(&manifest.name);
        if plugin_dir.exists() {
            fs::remove_dir_all(&plugin_dir)
                .map_err(|e| PluginError::IoError(e))?;
        }
        fs::rename(&temp_dir, &plugin_dir)
            .map_err(|e| PluginError::IoError(e))?;

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

            let manifest_path = path.join("plugin.json");
            if !manifest_path.exists() {
                continue;
            }

            if let Ok(json) = fs::read_to_string(&manifest_path) {
                if let Ok(manifest) = PluginManifest::from_json(&json) {
                    let enabled = self.installed.get(&manifest.name)
                        .map(|p| p.enabled)
                        .unwrap_or(true);

                    self.installed.insert(manifest.name.clone(), InstalledPlugin {
                        manifest,
                        enabled,
                        install_path: path,
                    });
                }
            }
        }
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
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.installed) {
            let _ = fs::write(&self.state_file, json);
        }
    }
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
