/// Plugin loader using libloading for dynamic library loading.

use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub use serialrun_plugin_api::{PluginCapability, PluginCallbacks, PluginResult, PluginStatus};

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Library load error: {0}")]
    LoadError(String),
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
    #[error("Plugin error: {0}")]
    PluginError(String),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type CoreResult<T> = Result<T, PluginError>;

/// Information about a loaded plugin.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

/// A command exposed by a plugin.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub parameters: Vec<PluginParameter>,
}

/// A parameter for a plugin command.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub param_type: String,
}

// FFI function signatures (required)
type FnGetInfo = unsafe extern "C" fn() -> *mut std::os::raw::c_char;
type FnGetCommands = unsafe extern "C" fn() -> *mut std::os::raw::c_char;
type FnExecute =
    unsafe extern "C" fn(*const std::os::raw::c_char, *const std::os::raw::c_char) -> *mut std::os::raw::c_char;
type FnFreeString = unsafe extern "C" fn(*mut std::os::raw::c_char);

// FFI function signatures (optional)
type FnGetCapabilities = unsafe extern "C" fn() -> *mut std::os::raw::c_char;
type FnInit = unsafe extern "C" fn(*const PluginCallbacks) -> bool;
type FnCleanup = unsafe extern "C" fn();
type FnGetUiLayout = unsafe extern "C" fn() -> *mut std::os::raw::c_char;

/// A loaded plugin from a dynamic library.
pub struct LoadedPlugin {
    path: PathBuf,
    library: Option<libloading::Library>,
    info: PluginInfo,
    commands: Vec<PluginCommand>,
    capabilities: Vec<PluginCapability>,
    is_enabled: bool,
    fn_execute: FnExecute,
    fn_free_string: FnFreeString,
    fn_init: Option<FnInit>,
    fn_cleanup: Option<FnCleanup>,
    fn_get_ui_layout: Option<FnGetUiLayout>,
    initialized: bool,
}

// SAFETY: libloading::Library is Send on all platforms. FFI function pointers are safe to
// call from any thread as long as the plugin itself is thread-safe (which is the plugin's
// responsibility to ensure).
unsafe impl Send for LoadedPlugin {}
unsafe impl Sync for LoadedPlugin {}

impl Drop for LoadedPlugin {
    fn drop(&mut self) {
        self.cleanup();
        // Library is automatically dropped here (Option<Library>)
    }
}

impl LoadedPlugin {
    /// Load a plugin from a dynamic library file.
    pub fn load(path: &Path) -> CoreResult<Self> {
        unsafe {
            let library = libloading::Library::new(path).map_err(|e| {
                PluginError::LoadError(format!("Failed to load {}: {}", path.display(), e))
            })?;

            let fn_get_info: FnGetInfo = *library
                .get(b"plugin_get_info")
                .map_err(|e| PluginError::SymbolNotFound(format!("plugin_get_info: {}", e)))?;

            let fn_get_commands: FnGetCommands = *library
                .get(b"plugin_get_commands")
                .map_err(|e| PluginError::SymbolNotFound(format!("plugin_get_commands: {}", e)))?;

            let fn_execute: FnExecute = *library
                .get(b"plugin_execute")
                .map_err(|e| PluginError::SymbolNotFound(format!("plugin_execute: {}", e)))?;

            let fn_free_string: FnFreeString = *library
                .get(b"plugin_free_string")
                .map_err(|e| PluginError::SymbolNotFound(format!("plugin_free_string: {}", e)))?;

            // Get info
            let info_ptr = fn_get_info();
            if info_ptr.is_null() {
                return Err(PluginError::PluginError(
                    "plugin_get_info returned null".to_string(),
                ));
            }
            let info_str = CStr::from_ptr(info_ptr).to_string_lossy().to_string();
            fn_free_string(info_ptr);

            let info: PluginInfo = serde_json::from_str(&info_str)?;

            // Get commands
            let commands_ptr = fn_get_commands();
            if commands_ptr.is_null() {
                return Err(PluginError::PluginError(
                    "plugin_get_commands returned null".to_string(),
                ));
            }
            let commands_str = CStr::from_ptr(commands_ptr).to_string_lossy().to_string();
            fn_free_string(commands_ptr);

            let commands: Vec<PluginCommand> = serde_json::from_str(&commands_str)?;

            // Detect optional capabilities
            let capabilities = match library.get::<FnGetCapabilities>(b"plugin_get_capabilities") {
                Ok(fn_get_caps) => {
                    let caps_ptr = (*fn_get_caps)();
                    if caps_ptr.is_null() {
                        Vec::new()
                    } else {
                        let caps_str = CStr::from_ptr(caps_ptr).to_string_lossy().to_string();
                        fn_free_string(caps_ptr);
                        match serde_json::from_str(&caps_str) {
                            Ok(caps) => caps,
                            Err(e) => {
                                log::warn!("Failed to parse capabilities JSON: {}", e);
                                Vec::new()
                            }
                        }
                    }
                }
                Err(_) => Vec::new(), // No capabilities function = basic plugin
            };

            // Detect optional init/cleanup functions
            let fn_init = library.get::<FnInit>(b"plugin_init").ok().map(|f| *f);
            let fn_cleanup = library.get::<FnCleanup>(b"plugin_cleanup").ok().map(|f| *f);

            // Detect optional UI layout function
            let fn_get_ui_layout = library.get::<FnGetUiLayout>(b"plugin_get_ui_layout").ok().map(|f| *f);

            log::info!(
                "Loaded plugin: {} v{} (capabilities: {:?})",
                info.name,
                info.version,
                capabilities
            );

            Ok(Self {
                path: path.to_path_buf(),
                library: Some(library),
                info,
                commands,
                capabilities,
                is_enabled: true,
                fn_execute,
                fn_free_string,
                fn_init,
                fn_cleanup,
                fn_get_ui_layout,
                initialized: false,
            })
        }
    }

    /// Initialize the plugin with host callbacks. Returns true on success.
    pub fn init(&mut self, callbacks: *const PluginCallbacks) -> bool {
        if let Some(fn_init) = self.fn_init {
            unsafe {
                self.initialized = fn_init(callbacks);
                if self.initialized {
                    log::info!("Plugin {} initialized with callbacks", self.info.name);
                } else {
                    log::warn!("Plugin {} init returned false", self.info.name);
                }
                self.initialized
            }
        } else {
            // No init function = basic plugin, considered initialized
            self.initialized = true;
            true
        }
    }

    /// Cleanup the plugin before unloading.
    /// Safe to call multiple times — only executes cleanup once.
    pub fn cleanup(&mut self) {
        if let Some(fn_cleanup) = self.fn_cleanup.take() {
            unsafe {
                fn_cleanup();
            }
            log::info!("Plugin {} cleaned up", self.info.name);
        }
        self.initialized = false;
    }

    /// Explicitly unload the plugin library (drops the DLL handle).
    /// This must be called BEFORE trying to delete the plugin directory on Windows,
    /// because Windows locks loaded DLL files.
    pub fn unload(&mut self) {
        // Call cleanup BEFORE dropping the library (cleanup code is in the DLL)
        if let Some(fn_cleanup) = self.fn_cleanup.take() {
            unsafe { fn_cleanup(); }
            log::info!("Plugin {} cleaned up", self.info.name);
        }
        self.initialized = false;
        // Drop the library to release the DLL file lock
        if self.library.take().is_some() {
            log::info!("Plugin {} library unloaded", self.info.name);
        }
    }

    /// Get the plugin's declared capabilities.
    pub fn capabilities(&self) -> &[PluginCapability] {
        &self.capabilities
    }

    /// Check if the plugin has a specific capability.
    pub fn has_capability(&self, cap: &PluginCapability) -> bool {
        self.capabilities.contains(cap)
    }

    /// Get the plugin's UI layout as JSON string, if it declares one.
    pub fn get_ui_layout(&self) -> Option<String> {
        let fn_get_layout = self.fn_get_ui_layout?;
        if !self.initialized {
            return None;
        }
        unsafe {
            let ptr = fn_get_layout();
            if ptr.is_null() {
                return None;
            }
            let json = CStr::from_ptr(ptr).to_string_lossy().to_string();
            (self.fn_free_string)(ptr);
            Some(json)
        }
    }

    /// Check if the plugin has been initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Execute a command on this plugin.
    pub fn execute_command(&self, command: &str, params: &str) -> CoreResult<PluginResult> {
        if !self.is_enabled {
            return Err(PluginError::PluginError(
                "Plugin is disabled".to_string(),
            ));
        }

        if !self.initialized {
            return Err(PluginError::PluginError(
                "Plugin is not initialized".to_string(),
            ));
        }

        let cmd_c = CString::new(command).map_err(|e| {
            PluginError::PluginError(format!("Invalid command string: {}", e))
        })?;

        let params_c = CString::new(params).map_err(|e| {
            PluginError::PluginError(format!("Invalid params string: {}", e))
        })?;

        unsafe {
            let result_ptr = (self.fn_execute)(cmd_c.as_ptr(), params_c.as_ptr());
            if result_ptr.is_null() {
                return Ok(PluginResult {
                    success: false,
                    result: None,
                    error: Some("Plugin returned null".to_string()),
                });
            }

            let result_str = CStr::from_ptr(result_ptr).to_string_lossy().to_string();
            (self.fn_free_string)(result_ptr);

            // BUG 10 FIX: Deserialize directly into PluginResult
            match serde_json::from_str::<PluginResult>(&result_str) {
                Ok(data) => Ok(data),
                Err(e) => Ok(PluginResult {
                    success: false,
                    result: None,
                    error: Some(format!("Failed to parse plugin result: {}", e)),
                }),
            }
        }
    }

    /// Get plugin info.
    pub fn info(&self) -> &PluginInfo {
        &self.info
    }

    /// Get the list of commands this plugin provides.
    pub fn commands(&self) -> &[PluginCommand] {
        &self.commands
    }

    /// Get the path to the plugin library.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Check if the plugin is enabled.
    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    /// Enable or disable the plugin.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_info_serde() {
        let info = PluginInfo {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: "test plugin".to_string(),
            author: "test".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: PluginInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "test");
    }

    #[test]
    fn test_plugin_result_data_serde() {
        let result = PluginResult {
            success: true,
            result: Some(serde_json::json!({"value": 42})),
            error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: PluginResult = serde_json::from_str(&json).unwrap();
        assert!(parsed.success);
    }

    #[test]
    fn test_plugin_result_error_serde() {
        let result = PluginResult::error("something went wrong");
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("something went wrong"));
        assert!(!result.success);
        assert!(result.result.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn test_plugin_result_success_serde() {
        let result = PluginResult::success(serde_json::json!({"key": "value"}));
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("key"));
        assert!(result.success);
        assert!(result.result.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_plugin_result_null_fields_omitted() {
        let result = PluginResult::success(serde_json::json!(42));
        let json = serde_json::to_string(&result).unwrap();
        // With skip_serializing_if, null fields should be omitted
        assert!(!json.contains("error"));
        assert!(json.contains("success"));
        assert!(json.contains("42"));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_load_example_plugin() {
        // This test requires the example plugin to be built
        let plugin_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/release/libserialrun_example_plugin.dylib");

        if !plugin_path.exists() {
            eprintln!("Skipping integration test: plugin not built at {}", plugin_path.display());
            return;
        }

        let result = LoadedPlugin::load(&plugin_path);
        assert!(result.is_ok(), "Failed to load plugin: {:?}", result.err());

        let mut plugin = result.unwrap();
        assert_eq!(plugin.info().name, "serialrun-example-plugin");
        assert_eq!(plugin.info().version, "0.1.0");
        assert!(!plugin.commands().is_empty());

        // Note: plugin_init returns false for null callbacks, which is expected.
        // In production, the host provides real callbacks.
        // For testing, we mark initialized manually.
        plugin.initialized = true;

        // Test execute_command
        let result = plugin.execute_command("echo", r#"{"data": "hello"}"#);
        assert!(result.is_ok());
        let plugin_result = result.unwrap();
        assert!(plugin_result.success);
        assert_eq!(plugin_result.result.unwrap(), serde_json::json!("hello"));

        // Test unknown command
        let result = plugin.execute_command("unknown", "{}");
        assert!(result.is_ok());
        let plugin_result = result.unwrap();
        assert!(!plugin_result.success);

        // Test cleanup
        plugin.cleanup();
        assert!(!plugin.is_initialized());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_plugin_execute_disabled() {
        let plugin_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/release/libserialrun_example_plugin.dylib");

        if !plugin_path.exists() {
            return;
        }

        let mut plugin = LoadedPlugin::load(&plugin_path).unwrap();
        plugin.set_enabled(false);

        let result = plugin.execute_command("echo", "{}");
        assert!(result.is_err());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_plugin_execute_not_initialized() {
        let plugin_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/release/libserialrun_example_plugin.dylib");

        if !plugin_path.exists() {
            return;
        }

        let mut plugin = LoadedPlugin::load(&plugin_path).unwrap();
        plugin.initialized = false;

        let result = plugin.execute_command("echo", "{}");
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_load_nonexistent() {
        let result = LoadedPlugin::load(std::path::Path::new("/nonexistent/path.dylib"));
        assert!(result.is_err());
    }
}
