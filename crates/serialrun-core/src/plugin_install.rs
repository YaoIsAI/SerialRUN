/// Plugin manager for install, uninstall, enable, disable operations.
///
/// Manages plugins in ~/.serialrun/plugins/ directory.
/// Each plugin is a subdirectory containing the binary and plugin.json.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serialrun_plugin_api::manifest::{PluginManifest, current_platform, supports_current_platform};
use crate::plugin::{LoadedPlugin, PluginError, CoreResult};

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
    pub fn install_from_zip(&mut self, zip_path: &Path) -> CoreResult<String> {
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
    pub fn install_from_dir(&mut self, dir: &Path) -> CoreResult<String> {
        let manifest_path = dir.join("plugin.json");
        if !manifest_path.exists() {
            return Err(PluginError::PluginError("No plugin.json found".to_string()));
        }

        let manifest_json = fs::read_to_string(&manifest_path)
            .map_err(|e| PluginError::IoError(e))?;
        let manifest: PluginManifest = PluginManifest::from_json(&manifest_json)
            .map_err(|e| PluginError::PluginError(format!("Invalid plugin.json: {}", e)))?;

        // BUG 9 FIX: Check platform compatibility
        if !supports_current_platform(&manifest) {
            return Err(PluginError::PluginError(format!(
                "Plugin '{}' does not support platform {}",
                manifest.name,
                current_platform()
            )));
        }

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
    pub fn uninstall(&mut self, name: &str) -> CoreResult<()> {
        let plugin = self.installed.remove(name)
            .ok_or_else(|| PluginError::PluginError(format!("Plugin '{}' not found", name)))?;

        // Remove plugin directory (with retry on Windows due to DLL file locks)
        if plugin.install_path.exists() {
            let mut retries = 3;
            loop {
                match fs::remove_dir_all(&plugin.install_path) {
                    Ok(()) => {
                        log::info!("Removed plugin directory: {}", plugin.install_path.display());
                        break;
                    }
                    Err(e) if retries > 0 && e.kind() == std::io::ErrorKind::PermissionDenied => {
                        log::warn!("Directory locked, retrying removal... ({} left)", retries);
                        retries -= 1;
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                    Err(e) => {
                        log::error!("Failed to remove plugin directory: {}", e);
                        return Err(PluginError::IoError(e));
                    }
                }
            }
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

        // Collect names found on disk
        let mut found = std::collections::HashSet::new();

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
                        let name = manifest.name.clone();

                        self.installed.insert(name.clone(), InstalledPlugin {
                            manifest,
                            enabled,
                            install_path: plugin_dir,
                        });
                        found.insert(name);
                    }
                }
            }
        }

        // BUG 3 FIX: Remove stale entries whose directories no longer exist
        self.installed.retain(|name, plugin| {
            if !found.contains(name) && !plugin.install_path.exists() {
                log::info!("Removing stale plugin entry: {}", name);
                false
            } else {
                true
            }
        });
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

    fn test_plugin_dir(suffix: &str) -> PathBuf {
        std::env::temp_dir().join(format!("serialrun_test_{}_{}", std::process::id(), suffix))
    }

    fn create_test_plugin(dir: &Path) {
        fs::create_dir_all(dir).unwrap();
        let manifest = PluginManifest::from_info(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            "Test plugin".to_string(),
            "Test Author".to_string(),
        );
        let json = manifest.to_json().unwrap();
        fs::write(dir.join("plugin.json"), json).unwrap();
        // Create a dummy DLL file (empty, won't load but tests file operations)
        fs::write(dir.join("test_plugin.dll"), b"dummy").unwrap();
    }

    #[test]
    fn test_plugin_manager_new() {
        // Use a temp dir to avoid interference with real installed plugins
        let dir = std::env::temp_dir().join(format!("test_pm_new_{}", std::process::id()));
        let manager = PluginManager::with_dir(dir);
        assert!(manager.installed().is_empty());
        let _ = fs::remove_dir_all(manager.plugins_dir());
    }

    #[test]
    fn test_plugin_manager_custom_dir() {
        let dir = std::env::temp_dir().join("test_plugins_empty");
        let manager = PluginManager::with_dir(dir);
        assert!(manager.installed().is_empty());
    }

    #[test]
    fn test_install_and_uninstall() {
        let test_dir = test_plugin_dir("install_uninstall");
        let plugins_dir = test_dir.join("plugins");
        let mut mgr = PluginManager::with_dir(plugins_dir.clone());

        // Create test plugin in a source directory
        let plugin_src = test_dir.join("src");
        create_test_plugin(&plugin_src);

        // install_from_dir references the source dir directly (dev mode)
        let result = mgr.install_from_dir(&plugin_src);
        assert!(result.is_ok(), "Install failed: {:?}", result);

        // Verify installed in manager
        assert!(mgr.installed().contains_key("test-plugin"));
        assert!(mgr.get("test-plugin").is_some());

        // Uninstall
        let result = mgr.uninstall("test-plugin");
        assert!(result.is_ok(), "Uninstall failed: {:?}", result);

        // Verify removed from manager
        assert!(!mgr.installed().contains_key("test-plugin"));

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_uninstall_nonexistent() {
        let dir = test_plugin_dir("nonexist");
        let mut mgr = PluginManager::with_dir(dir.clone());
        let result = mgr.uninstall("nonexistent-plugin");
        assert!(result.is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_enable_disable() {
        let test_dir = test_plugin_dir("enable_disable");
        let plugins_dir = test_dir.join("plugins");
        let mut mgr = PluginManager::with_dir(plugins_dir.clone());

        let plugin_src = test_dir.join("src");
        create_test_plugin(&plugin_src);
        let _ = mgr.install_from_dir(&plugin_src);

        // Initially enabled
        assert!(mgr.get("test-plugin").unwrap().enabled);

        // Disable
        let result = mgr.disable("test-plugin");
        assert!(result);
        assert!(!mgr.get("test-plugin").unwrap().enabled);

        // Enable
        let result = mgr.enable("test-plugin");
        assert!(result);
        assert!(mgr.get("test-plugin").unwrap().enabled);

        // Cleanup
        let _ = mgr.uninstall("test-plugin");
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_uninstall_removes_from_manager() {
        let test_dir = test_plugin_dir("uninstall_removes");
        let plugins_dir = test_dir.join("plugins");
        let plugin_dest = plugins_dir.join("test-plugin");
        create_test_plugin(&plugin_dest);

        // Discover and uninstall
        let mut mgr = PluginManager::with_dir(plugins_dir.clone());
        mgr.discover();
        assert!(mgr.installed().contains_key("test-plugin"));

        let result = mgr.uninstall("test-plugin");
        assert!(result.is_ok(), "Uninstall failed: {:?}", result);

        // Verify removed from manager AND directory deleted
        assert!(!mgr.installed().contains_key("test-plugin"));
        assert!(!plugin_dest.exists(), "Plugin directory should be deleted");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_discover_after_uninstall() {
        let test_dir = test_plugin_dir("discover_after");
        let plugins_dir = test_dir.join("plugins");

        // Install and create the plugin directory
        let plugin_dest = plugins_dir.join("test-plugin");
        create_test_plugin(&plugin_dest);

        // Now install via discover (which reads from disk)
        {
            let mut mgr = PluginManager::with_dir(plugins_dir.clone());
            mgr.discover();
            assert!(mgr.installed().contains_key("test-plugin"),
                "Plugin should be discovered on disk");
        }

        // Uninstall
        {
            let mut mgr = PluginManager::with_dir(plugins_dir.clone());
            mgr.discover();
            let _ = mgr.uninstall("test-plugin");
        }

        // Create a NEW manager (simulates app restart)
        let mut mgr2 = PluginManager::with_dir(plugins_dir.clone());
        mgr2.discover();

        // Should NOT find the uninstalled plugin
        assert!(!mgr2.installed().contains_key("test-plugin"),
            "Plugin should not be found after uninstall, but was discovered");
        assert!(!plugin_dest.exists(),
            "Plugin directory should be deleted after uninstall");

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }
}
