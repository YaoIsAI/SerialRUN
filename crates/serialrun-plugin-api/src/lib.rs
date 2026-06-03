/// SerialRUN Plugin API - shared types for plugin development.

pub mod manifest;

use serde::{Deserialize, Serialize};
use std::os::raw::{c_char, c_float, c_int};

/// Current plugin API version.
pub const PLUGIN_API_VERSION: &str = "0.3.0";

// ============================================================================
// Plugin Capabilities
// ============================================================================

/// Capabilities that a plugin can declare.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapability {
    /// Plugin needs serial port access (read/write)
    SerialPort,
    /// Plugin provides a custom UI panel
    UiPanel,
    /// Plugin needs file open/save dialogs
    FileDialog,
    /// Plugin reports progress during operations
    Progress,
    /// Plugin uses host logging
    Logging,
    /// Plugin needs file system access (list/read/write/delete on device)
    FileSystem,
    /// Plugin subscribes to serial port events (data received, connection changes)
    EventSubscription,
    /// Plugin stores persistent configuration
    ConfigStorage,
    /// Plugin uses async command execution
    AsyncExecution,
    /// Plugin declares a UI layout (JSON-based)
    UiLayout,
    /// Unknown capability (forward-compatible with newer API versions)
    #[serde(other)]
    Unknown,
}

/// Status codes for progress callbacks.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginStatus {
    Idle = 0,
    Running = 1,
    Success = 2,
    Error = 3,
}

// ============================================================================
// Host Callbacks (provided to plugins by the host)
// ============================================================================

/// Callback functions provided by the host to the plugin.
/// All function pointers are Option - plugins should check before calling.
/// Uses extern "C" ABI for safe FFI across different compilation units.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PluginCallbacks {
    // Serial port access
    pub serial_read: Option<extern "C" fn(buf: *mut u8, len: u32, timeout_ms: u32) -> c_int>,
    pub serial_write: Option<extern "C" fn(data: *const u8, len: u32) -> c_int>,
    pub serial_set_baud: Option<extern "C" fn(baud: u32) -> bool>,
    pub serial_is_connected: Option<extern "C" fn() -> bool>,

    // Progress callbacks
    pub progress_set: Option<extern "C" fn(percent: c_float, message: *const c_char)>,
    pub progress_set_status: Option<extern "C" fn(status: PluginStatus)>,
    pub progress_is_cancelled: Option<extern "C" fn() -> bool>,

    // File operations
    pub file_open_dialog: Option<extern "C" fn(filter: *const c_char) -> *mut c_char>,
    pub file_save_dialog: Option<extern "C" fn(filter: *const c_char) -> *mut c_char>,
    pub file_read: Option<extern "C" fn(path: *const c_char) -> *mut c_char>, // returns base64
    pub free_string: Option<extern "C" fn(s: *mut c_char)>, // free strings returned by callbacks

    // Logging
    pub log_info: Option<extern "C" fn(msg: *const c_char)>,
    pub log_warn: Option<extern "C" fn(msg: *const c_char)>,
    pub log_error: Option<extern "C" fn(msg: *const c_char)>,

    // File system (device-side, for plugins like MicroPython IDE)
    /// List directory contents on device. Returns JSON array of {name, is_dir, size}.
    pub fs_list_dir: Option<extern "C" fn(path: *const c_char) -> *mut c_char>,
    /// Read file from device. Returns base64-encoded data.
    pub fs_read_file: Option<extern "C" fn(path: *const c_char) -> *mut c_char>,
    /// Write file to device. data is base64-encoded.
    pub fs_write_file: Option<extern "C" fn(path: *const c_char, data: *const c_char) -> bool>,
    /// Delete file on device.
    pub fs_delete_file: Option<extern "C" fn(path: *const c_char) -> bool>,
    /// Create directory on device.
    pub fs_mkdir: Option<extern "C" fn(path: *const c_char) -> bool>,
    /// Check if file exists on device.
    pub fs_exists: Option<extern "C" fn(path: *const c_char) -> bool>,

    // Event system
    /// Register a callback for serial data events. data_callback is called when data is received.
    pub on_serial_data: Option<extern "C" fn(data_callback: extern "C" fn(*const u8, u32))>,
    /// Register a callback for connection state changes.
    pub on_connection_changed: Option<extern "C" fn(conn_callback: extern "C" fn(bool))>,

    // Config storage
    /// Get a config value for this plugin. Returns JSON string or null.
    pub config_get: Option<extern "C" fn(key: *const c_char) -> *mut c_char>,
    /// Set a config value for this plugin.
    pub config_set: Option<extern "C" fn(key: *const c_char, value: *const c_char) -> bool>,

    // Async execution
    /// Execute a command asynchronously. callback is called with the result JSON when done.
    pub execute_async: Option<extern "C" fn(command: *const c_char, params: *const c_char, callback: extern "C" fn(*const c_char))>,
}

// ============================================================================
// Optional FFI function signatures (for documentation, not called directly)
// ============================================================================

/// Optional: Plugin declares its capabilities.
/// Returns JSON array of capability strings.
/// Signature: `fn() -> *mut c_char`
///
/// Example return value: `["serial_port", "logging"]`
pub type FnGetCapabilities = extern "C" fn() -> *mut c_char;

/// Optional: Plugin initialization with host callbacks.
/// Called once after loading. Plugin should store the callbacks pointer.
/// Signature: `fn(callbacks: *const PluginCallbacks) -> bool`
pub type FnInit = extern "C" fn(callbacks: *const PluginCallbacks) -> bool;

/// Optional: Plugin cleanup. Called before unloading.
/// Signature: `fn()`
pub type FnCleanup = extern "C" fn();

/// Optional: Plugin returns its UI layout as JSON.
/// The host renders this layout as egui components.
/// Signature: `fn() -> *mut c_char`
pub type FnGetUiLayout = extern "C" fn() -> *mut c_char;

/// Optional: Plugin receives an event from the host.
/// Signature: `fn(event_type: *const c_char, data: *const c_char)`
pub type FnOnEvent = extern "C" fn(event_type: *const c_char, data: *const c_char);

/// Optional: Plugin returns its config schema as JSON.
/// Signature: `fn() -> *mut c_char`
pub type FnGetConfigSchema = extern "C" fn() -> *mut c_char;

// ============================================================================
// Capability helpers
// ============================================================================

/// Parse capabilities from a JSON string.
pub fn parse_capabilities(json: &str) -> Result<Vec<PluginCapability>, serde_json::Error> {
    serde_json::from_str(json)
}

/// Serialize capabilities to a JSON string.
pub fn serialize_capabilities(caps: &[PluginCapability]) -> Result<String, serde_json::Error> {
    serde_json::to_string(caps)
}

/// Information about a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

/// A command exposed by a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub parameters: Vec<PluginParameter>,
}

/// A parameter for a plugin command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub param_type: String,
}

/// Result of executing a plugin command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl PluginResult {
    pub fn success(result: serde_json::Value) -> Self {
        Self {
            success: true,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            result: None,
            error: Some(error.into()),
        }
    }
}

/// Parse plugin info from a JSON string.
pub fn parse_plugin_info(json: &str) -> Result<PluginInfo, serde_json::Error> {
    serde_json::from_str(json)
}

/// Parse plugin commands from a JSON string.
pub fn parse_plugin_commands(json: &str) -> Result<Vec<PluginCommand>, serde_json::Error> {
    serde_json::from_str(json)
}

/// Parse a plugin result from a JSON string.
pub fn parse_plugin_result(json: &str) -> Result<PluginResult, serde_json::Error> {
    serde_json::from_str(json)
}

// ============================================================================
// UI Layout Types (for dynamic plugin UI)
// ============================================================================

/// A node in the plugin UI layout tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UiLayoutNode {
    /// Horizontal split with draggable divider
    #[serde(rename = "split_horizontal")]
    SplitHorizontal {
        children: Vec<UiLayoutNode>,
        #[serde(default = "default_ratio")]
        ratio: f32,
    },
    /// Vertical split with draggable divider
    #[serde(rename = "split_vertical")]
    SplitVertical {
        children: Vec<UiLayoutNode>,
        #[serde(default = "default_ratio")]
        ratio: f32,
    },
    /// A panel with title and content
    #[serde(rename = "panel")]
    Panel {
        id: String,
        title: String,
        content: UiContent,
        #[serde(default)]
        width: Option<f32>,
        #[serde(default)]
        height: Option<f32>,
    },
}

fn default_ratio() -> f32 {
    0.5
}

/// Content types for UI panels.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UiContent {
    /// Tree view (file browser, etc.)
    #[serde(rename = "tree_view")]
    TreeView,
    /// Code editor with syntax highlighting
    #[serde(rename = "code_editor")]
    CodeEditor {
        #[serde(default = "default_language")]
        language: String,
    },
    /// Terminal/console component
    #[serde(rename = "terminal")]
    Terminal,
    /// Plain text display
    #[serde(rename = "text")]
    Text,
    /// Custom HTML content
    #[serde(rename = "html")]
    Html,
}

fn default_language() -> String {
    "python".to_string()
}

/// Parse a UI layout from JSON.
pub fn parse_ui_layout(json: &str) -> Result<UiLayoutNode, serde_json::Error> {
    serde_json::from_str(json)
}

/// Serialize a UI layout to JSON.
pub fn serialize_ui_layout(layout: &UiLayoutNode) -> Result<String, serde_json::Error> {
    serde_json::to_string(layout)
}

impl UiLayoutNode {
    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Parse from JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_api_version() {
        assert_eq!(PLUGIN_API_VERSION, "0.3.0");
    }

    #[test]
    fn test_plugin_info_serde() {
        let info = PluginInfo {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        let parsed: PluginInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.version, "1.0.0");
    }

    #[test]
    fn test_plugin_command_serde() {
        let cmd = PluginCommand {
            name: "echo".to_string(),
            description: "Echo input".to_string(),
            parameters: vec![PluginParameter {
                name: "data".to_string(),
                description: "Data to echo".to_string(),
                required: true,
                param_type: "string".to_string(),
            }],
        };

        let json = serde_json::to_string(&cmd).unwrap();
        let parsed: PluginCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "echo");
        assert_eq!(parsed.parameters.len(), 1);
        assert!(parsed.parameters[0].required);
    }

    #[test]
    fn test_plugin_result_success() {
        let result = PluginResult::success(serde_json::json!({"value": 42}));
        assert!(result.success);
        assert!(result.result.is_some());
        assert!(result.error.is_none());

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("42"));
    }

    #[test]
    fn test_plugin_result_error() {
        let result = PluginResult::error("Something went wrong");
        assert!(!result.success);
        assert!(result.result.is_none());
        assert!(result.error.is_some());

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Something went wrong"));
    }

    #[test]
    fn test_parse_plugin_info() {
        let json = r#"{"name":"test","version":"1.0","description":"desc","author":"auth"}"#;
        let info = parse_plugin_info(json).unwrap();
        assert_eq!(info.name, "test");
    }

    #[test]
    fn test_parse_plugin_commands() {
        let json = r#"[{"name":"cmd","description":"desc","parameters":[]}]"#;
        let cmds = parse_plugin_commands(json).unwrap();
        assert_eq!(cmds.len(), 1);
    }

    #[test]
    fn test_parse_plugin_result() {
        let json = r#"{"success":true,"result":null}"#;
        let result = parse_plugin_result(json).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_capabilities_serde() {
        let caps = vec![PluginCapability::SerialPort, PluginCapability::Logging];
        let json = serialize_capabilities(&caps).unwrap();
        assert!(json.contains("serial_port"));
        assert!(json.contains("logging"));

        let parsed = parse_capabilities(&json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert!(parsed.contains(&PluginCapability::SerialPort));
    }

    #[test]
    fn test_plugin_status() {
        assert_eq!(PluginStatus::Idle as i32, 0);
        assert_eq!(PluginStatus::Running as i32, 1);
        assert_eq!(PluginStatus::Success as i32, 2);
        assert_eq!(PluginStatus::Error as i32, 3);
    }

    #[test]
    fn test_ui_layout_serde() {
        let layout = UiLayoutNode::SplitHorizontal {
            children: vec![
                UiLayoutNode::Panel {
                    id: "files".to_string(),
                    title: "Files".to_string(),
                    content: UiContent::TreeView,
                    width: Some(250.0),
                    height: None,
                },
                UiLayoutNode::SplitVertical {
                    children: vec![
                        UiLayoutNode::Panel {
                            id: "editor".to_string(),
                            title: "Editor".to_string(),
                            content: UiContent::CodeEditor { language: "python".to_string() },
                            width: None,
                            height: None,
                        },
                        UiLayoutNode::Panel {
                            id: "repl".to_string(),
                            title: "REPL".to_string(),
                            content: UiContent::Terminal,
                            width: None,
                            height: None,
                        },
                    ],
                    ratio: 0.6,
                },
            ],
            ratio: 0.3,
        };

        let json = serialize_ui_layout(&layout).unwrap();
        assert!(json.contains("split_horizontal"));
        assert!(json.contains("code_editor"));

        let parsed = parse_ui_layout(&json).unwrap();
        assert!(parsed.to_json().unwrap().contains("split_horizontal"));
    }
}
