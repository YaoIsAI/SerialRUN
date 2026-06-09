/// SerialRUN Example Plugin — Complete Template
///
/// This plugin demonstrates ALL plugin API features:
/// - Required FFI functions (4)
/// - Optional FFI functions (capabilities, init, cleanup, UI layout)
/// - Serial port access via host callbacks
/// - Progress reporting
/// - File dialog usage
/// - Logging via host callbacks
/// - Config storage
/// - Thread-safe callback storage
/// - Panic safety (catch_unwind on all extern "C" functions)
///
/// To create your own plugin:
/// 1. Copy this entire directory
/// 2. Rename in Cargo.toml and plugin.json
/// 3. Modify commands in plugin_execute()
/// 4. Build: cargo build --release -p your-plugin
/// 5. Install: cp target/release/libyour_plugin.dylib ~/.serialrun/plugins/your-plugin/

use serialrun_plugin_api::*;
use std::any::Any;
use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Mutex, OnceLock};

// ============================================================================
// Panic-safe wrapper
// ============================================================================

/// Catch a panic and return a JSON error result.
/// All extern "C" functions must use this to prevent UB.
fn catch_plugin_panic<F: FnOnce() -> *mut c_char + std::panic::UnwindSafe>(f: F) -> *mut c_char {
    match catch_unwind(f) {
        Ok(ptr) => ptr,
        Err(_panic) => {
            // Return a valid JSON error string — the host can deserialize it
            let err = PluginResult::error("Plugin panicked internally");
            let json = serde_json::to_string(&err).unwrap_or_else(|_| {
                r#"{"success":false,"error":"Plugin panicked and JSON serialization failed"}"#.to_string()
            });
            CString::new(json).unwrap_or_default().into_raw()
        }
    }
}

/// Safely convert a Result to a C string pointer, handling all error cases.
fn result_to_ptr(result: Result<String, Box<dyn Any + Send>>) -> *mut c_char {
    match result {
        Ok(json) => CString::new(json).unwrap_or_default().into_raw(),
        Err(_) => {
            let err = PluginResult::error("Plugin panicked internally");
            let json = serde_json::to_string(&err).unwrap_or_default();
            CString::new(json).unwrap_or_default().into_raw()
        }
    }
}

// ============================================================================
// Thread-safe callback storage
// ============================================================================

/// Store host callbacks using OnceLock<Mutex> for thread safety.
/// This is the standard pattern for all plugins.
static CALLBACKS: OnceLock<Mutex<Option<PluginCallbacks>>> = OnceLock::new();

/// Helper to get cloned callbacks.
/// Returns None if not initialized or if mutex is poisoned (recovers from poison).
fn get_callbacks() -> Option<PluginCallbacks> {
    let mutex = CALLBACKS.get()?;
    match mutex.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => {
            // Recover from poisoned mutex — the previous holder panicked but we can still use the data
            log::warn!("Mutex poisoned in plugin, recovering");
            poisoned.into_inner().clone()
        }
    }
}

// ============================================================================
// Required FFI Functions (4)
// ============================================================================

/// Returns plugin metadata as JSON.
/// The `name` field MUST match the `name` in plugin.json.
#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char {
    catch_plugin_panic(|| {
        let info = PluginInfo {
            name: "serialrun-example-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "Example plugin demonstrating ALL SerialRUN plugin API features".to_string(),
            author: "SerialRUN Team".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        CString::new(json).unwrap().into_raw()
    })
}

/// Returns the list of commands this plugin exposes.
/// Each command has a name, description, and parameter list.
#[no_mangle]
pub extern "C" fn plugin_get_commands() -> *mut c_char {
    catch_plugin_panic(|| {
        let commands = vec![
            PluginCommand {
                name: "echo".to_string(),
                description: "Echo back the input data".to_string(),
                parameters: vec![PluginParameter {
                    name: "data".to_string(),
                    description: "Data to echo".to_string(),
                    required: true,
                    param_type: "string".to_string(),
                }],
            },
            PluginCommand {
                name: "serial_send".to_string(),
                description: "Send data to serial port and read response".to_string(),
                parameters: vec![
                    PluginParameter {
                        name: "data".to_string(),
                        description: "Hex-encoded data to send (e.g., '48656C6C6F')".to_string(),
                        required: true,
                        param_type: "string".to_string(),
                    },
                    PluginParameter {
                        name: "timeout_ms".to_string(),
                        description: "Read timeout in milliseconds".to_string(),
                        required: false,
                        param_type: "number".to_string(),
                    },
                ],
            },
            PluginCommand {
                name: "add".to_string(),
                description: "Add two numbers".to_string(),
                parameters: vec![
                    PluginParameter {
                        name: "a".to_string(),
                        description: "First number".to_string(),
                        required: true,
                        param_type: "number".to_string(),
                    },
                    PluginParameter {
                        name: "b".to_string(),
                        description: "Second number".to_string(),
                        required: true,
                        param_type: "number".to_string(),
                    },
                ],
            },
            PluginCommand {
                name: "demo_progress".to_string(),
                description: "Demo: run a task with progress reporting".to_string(),
                parameters: vec![],
            },
            PluginCommand {
                name: "open_file".to_string(),
                description: "Open a file dialog and read the selected file".to_string(),
                parameters: vec![],
            },
            PluginCommand {
                name: "get_setting".to_string(),
                description: "Get a saved setting".to_string(),
                parameters: vec![PluginParameter {
                    name: "key".to_string(),
                    description: "Setting key".to_string(),
                    required: true,
                    param_type: "string".to_string(),
                }],
            },
            PluginCommand {
                name: "set_setting".to_string(),
                description: "Save a setting".to_string(),
                parameters: vec![
                    PluginParameter {
                        name: "key".to_string(),
                        description: "Setting key".to_string(),
                        required: true,
                        param_type: "string".to_string(),
                    },
                    PluginParameter {
                        name: "value".to_string(),
                        description: "Setting value".to_string(),
                        required: true,
                        param_type: "string".to_string(),
                    },
                ],
            },
        ];
        let json = serde_json::to_string(&commands).unwrap();
        CString::new(json).unwrap().into_raw()
    })
}

/// Execute a command. This is the main entry point for all plugin logic.
/// Returns a JSON PluginResult: {"success": true, "result": ...} or {"success": false, "error": "..."}
#[no_mangle]
pub extern "C" fn plugin_execute(command: *const c_char, params: *const c_char) -> *mut c_char {
    catch_plugin_panic(AssertUnwindSafe(|| {
        let cmd = unsafe {
            if command.is_null() {
                return CString::new(r#"{"success":false,"error":"Null command"}"#).unwrap().into_raw();
            }
            CStr::from_ptr(command).to_string_lossy().to_string()
        };

        let params_str = unsafe {
            if params.is_null() {
                "{}".to_string()
            } else {
                CStr::from_ptr(params).to_string_lossy().to_string()
            }
        };

        let result = match cmd.as_str() {
            "echo" => cmd_echo(&params_str),
            "serial_send" => cmd_serial_send(&params_str),
            "add" => cmd_add(&params_str),
            "demo_progress" => cmd_demo_progress(),
            "open_file" => cmd_open_file(),
            "get_setting" => cmd_get_setting(&params_str),
            "set_setting" => cmd_set_setting(&params_str),
            _ => PluginResult::error(format!("Unknown command: {}", cmd)),
        };

        let json = serde_json::to_string(&result).unwrap();
        CString::new(json).unwrap().into_raw()
    }))
}

/// Free a string allocated by the plugin. MUST be called for every string returned by FFI.
#[no_mangle]
pub extern "C" fn plugin_free_string(s: *mut c_char) {
    // catch_unwind not needed here — CString::from_raw is safe even after panic
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}

// ============================================================================
// Optional FFI Functions
// ============================================================================

/// Declare what host capabilities this plugin uses.
#[no_mangle]
pub extern "C" fn plugin_get_capabilities() -> *mut c_char {
    catch_plugin_panic(|| {
        let caps = vec![
            PluginCapability::SerialPort,
            PluginCapability::Progress,
            PluginCapability::Logging,
            PluginCapability::FileDialog,
            PluginCapability::ConfigStorage,
        ];
        let json = serialize_capabilities(&caps).unwrap();
        CString::new(json).unwrap().into_raw()
    })
}

/// Initialize the plugin with host callbacks.
#[no_mangle]
pub extern "C" fn plugin_init(callbacks: *const PluginCallbacks) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        if callbacks.is_null() {
            return false;
        }
        let cbs = unsafe { *callbacks };
        let store = CALLBACKS.get_or_init(|| Mutex::new(None));
        match store.lock() {
            Ok(mut guard) => {
                *guard = Some(cbs);
                if let Some(log) = cbs.log_info {
                    let msg = CString::new("Example Plugin initialized").unwrap();
                    log(msg.as_ptr());
                }
                true
            }
            Err(poisoned) => {
                // Recover from poisoned mutex
                let mut guard = poisoned.into_inner();
                *guard = Some(cbs);
                true
            }
        }
    }))
    .unwrap_or(false)
}

/// Cleanup before unloading.
#[no_mangle]
pub extern "C" fn plugin_cleanup() {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if let Some(store) = CALLBACKS.get() {
            if let Ok(mut guard) = store.lock() {
                *guard = None;
            }
        }
    }));
}

/// Return UI layout as JSON.
/// Demonstrates ButtonGroup component for one-click command execution.
#[no_mangle]
pub extern "C" fn plugin_get_ui_layout() -> *mut c_char {
    catch_plugin_panic(|| {
        let layout = UiLayoutNode::SplitVertical {
            children: vec![
                // Button panel — one-click actions
                UiLayoutNode::Panel {
                    id: "actions".to_string(),
                    title: "⚡ Quick Actions".to_string(),
                    content: UiContent::ButtonGroup {
                        buttons: vec![
                            UiButton {
                                id: "echo".to_string(),
                                label: "Echo".to_string(),
                                icon: Some("📢".to_string()),
                                command: "echo".to_string(),
                                params: Some(r#"{"data": "Hello from plugin!"}"#.to_string()),
                                style: ButtonStyle::Primary,
                                tooltip: Some("Echo test data".to_string()),
                            },
                            UiButton {
                                id: "add".to_string(),
                                label: "Add 3+4".to_string(),
                                icon: Some("🔢".to_string()),
                                command: "add".to_string(),
                                params: Some(r#"{"a": 3, "b": 4}"#.to_string()),
                                style: ButtonStyle::Success,
                                tooltip: Some("Calculate 3 + 4".to_string()),
                            },
                            UiButton {
                                id: "progress".to_string(),
                                label: "Demo Progress".to_string(),
                                icon: Some("📊".to_string()),
                                command: "demo_progress".to_string(),
                                params: None,
                                style: ButtonStyle::Secondary,
                                tooltip: Some("Run a task with progress bar".to_string()),
                            },
                            UiButton {
                                id: "file".to_string(),
                                label: "Open File".to_string(),
                                icon: Some("📂".to_string()),
                                command: "open_file".to_string(),
                                params: None,
                                style: ButtonStyle::Secondary,
                                tooltip: Some("Open a file dialog".to_string()),
                            },
                        ],
                        direction: "horizontal".to_string(),
                    },
                    width: None,
                    height: Some(80.0),
                },
                // Serial port quick send panel
                UiLayoutNode::Panel {
                    id: "serial_send".to_string(),
                    title: "🔌 Serial Port Quick Send".to_string(),
                    content: UiContent::Input {
                        placeholder: "Enter hex data (e.g., 48656C6C6F)".to_string(),
                        command: "serial_send".to_string(),
                        button_label: "Send".to_string(),
                    },
                    width: None,
                    height: Some(60.0),
                },
                // Output panel
                UiLayoutNode::Panel {
                    id: "output".to_string(),
                    title: "💬 Output".to_string(),
                    content: UiContent::Terminal,
                    width: None,
                    height: None,
                },
            ],
            ratio: 0.3,
        };
        let json = serialize_ui_layout(&layout).unwrap();
        CString::new(json).unwrap().into_raw()
    })
}

// ============================================================================
// Command Implementations
// ============================================================================

fn cmd_echo(params_str: &str) -> PluginResult {
    let params: serde_json::Value = serde_json::from_str(params_str).unwrap_or_default();
    match params.get("data").and_then(|v| v.as_str()) {
        Some(data) => PluginResult::success(serde_json::json!(data)),
        None => PluginResult::error("Missing required parameter: data"),
    }
}

fn cmd_serial_send(params_str: &str) -> PluginResult {
    let params: serde_json::Value = serde_json::from_str(params_str).unwrap_or_default();

    let Some(cb) = get_callbacks() else {
        return PluginResult::error("Plugin not initialized");
    };

    let Some(write_fn) = cb.serial_write else {
        return PluginResult::error("Serial write not available");
    };
    let Some(read_fn) = cb.serial_read else {
        return PluginResult::error("Serial read not available");
    };
    let Some(is_connected) = cb.serial_is_connected else {
        return PluginResult::error("Connection check not available");
    };

    if !is_connected() {
        return PluginResult::error("No serial port connected");
    }

    let data_hex = match params.get("data").and_then(|v| v.as_str()) {
        Some(h) => h,
        None => return PluginResult::error("Missing required parameter: data"),
    };

    let data = match hex::decode(data_hex) {
        Ok(d) => d,
        Err(e) => return PluginResult::error(format!("Invalid hex: {}", e)),
    };

    let timeout_ms = params.get("timeout_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(1000) as u32;

    let written = write_fn(data.as_ptr(), data.len() as u32);
    if written <= 0 {
        return PluginResult::error("Failed to write to serial port");
    }

    let mut buf = [0u8; 4096];
    let n = read_fn(buf.as_mut_ptr(), buf.len() as u32, timeout_ms);

    if n > 0 {
        let response_hex = hex::encode(&buf[..n as usize]);
        PluginResult::success(serde_json::json!({
            "bytes_sent": data.len(),
            "bytes_received": n,
            "response_hex": response_hex,
        }))
    } else {
        PluginResult::success(serde_json::json!({
            "bytes_sent": data.len(),
            "bytes_received": 0,
            "response_hex": "",
            "note": "No response within timeout",
        }))
    }
}

fn cmd_add(params_str: &str) -> PluginResult {
    let params: serde_json::Value = serde_json::from_str(params_str).unwrap_or_default();
    let a = params.get("a").and_then(|v| v.as_f64());
    let b = params.get("b").and_then(|v| v.as_f64());
    match (a, b) {
        (Some(a), Some(b)) => PluginResult::success(serde_json::json!(a + b)),
        _ => PluginResult::error("Missing required parameters: a, b"),
    }
}

fn cmd_demo_progress() -> PluginResult {
    let Some(cb) = get_callbacks() else {
        return PluginResult::error("Plugin not initialized");
    };

    for i in 0..10 {
        if let Some(is_cancelled) = cb.progress_is_cancelled {
            if is_cancelled() {
                return PluginResult::error("Cancelled by user");
            }
        }
        if let Some(progress) = cb.progress_set {
            let msg = CString::new(format!("Processing step {} / 10...", i + 1)).unwrap();
            progress((i as f32 + 1.0) * 10.0, msg.as_ptr());
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    if let Some(set_status) = cb.progress_set_status {
        set_status(PluginStatus::Success);
    }

    PluginResult::success(serde_json::json!({"completed": true, "steps": 10}))
}

fn cmd_open_file() -> PluginResult {
    let Some(cb) = get_callbacks() else {
        return PluginResult::error("Plugin not initialized");
    };

    let Some(open_dialog) = cb.file_open_dialog else {
        return PluginResult::error("File dialog not available");
    };

    let Some(read_file) = cb.file_read else {
        return PluginResult::error("File read not available");
    };

    let filter = CString::new("Firmware").unwrap();
    let path_ptr = open_dialog(filter.as_ptr());

    if path_ptr.is_null() {
        return PluginResult::success(serde_json::json!({"cancelled": true}));
    }

    let path = unsafe { CStr::from_ptr(path_ptr).to_string_lossy().to_string() };

    if let Some(free) = cb.free_string {
        free(path_ptr);
    }

    let path_c = CString::new(path.as_str()).unwrap();
    let data_ptr = read_file(path_c.as_ptr());

    if data_ptr.is_null() {
        return PluginResult::error(format!("Failed to read file: {}", path));
    }

    let b64 = unsafe { CStr::from_ptr(data_ptr).to_string_lossy().to_string() };

    if let Some(free) = cb.free_string {
        free(data_ptr);
    }

    PluginResult::success(serde_json::json!({
        "path": path,
        "size_bytes": b64.len() * 3 / 4,
    }))
}

fn cmd_get_setting(params_str: &str) -> PluginResult {
    let params: serde_json::Value = serde_json::from_str(params_str).unwrap_or_default();
    let key = match params.get("key").and_then(|v| v.as_str()) {
        Some(k) => k,
        None => return PluginResult::error("Missing required parameter: key"),
    };

    let Some(cb) = get_callbacks() else {
        return PluginResult::error("Plugin not initialized");
    };

    let Some(config_get) = cb.config_get else {
        return PluginResult::error("Config storage not available");
    };

    let key_c = CString::new(key).unwrap();
    let value_ptr = config_get(key_c.as_ptr());

    if value_ptr.is_null() {
        return PluginResult::success(serde_json::json!({"key": key, "value": null}));
    }

    let value = unsafe { CStr::from_ptr(value_ptr).to_string_lossy().to_string() };

    if let Some(free) = cb.free_string {
        free(value_ptr);
    }

    PluginResult::success(serde_json::json!({"key": key, "value": value}))
}

fn cmd_set_setting(params_str: &str) -> PluginResult {
    let params: serde_json::Value = serde_json::from_str(params_str).unwrap_or_default();
    let key = match params.get("key").and_then(|v| v.as_str()) {
        Some(k) => k,
        None => return PluginResult::error("Missing required parameter: key"),
    };
    let value = match params.get("value").and_then(|v| v.as_str()) {
        Some(v) => v,
        None => return PluginResult::error("Missing required parameter: value"),
    };

    let Some(cb) = get_callbacks() else {
        return PluginResult::error("Plugin not initialized");
    };

    let Some(config_set) = cb.config_set else {
        return PluginResult::error("Config storage not available");
    };

    let key_c = CString::new(key).unwrap();
    let value_c = CString::new(value).unwrap();
    let success = config_set(key_c.as_ptr(), value_c.as_ptr());

    if success {
        PluginResult::success(serde_json::json!({"key": key, "value": value, "saved": true}))
    } else {
        PluginResult::error("Failed to save setting")
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_info() {
        let ptr = plugin_get_info();
        assert!(!ptr.is_null());
        let info_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let info: PluginInfo = serde_json::from_str(&info_str).unwrap();
        assert_eq!(info.name, "serialrun-example-plugin");
        assert_eq!(info.version, "0.1.0");
        plugin_free_string(ptr);
    }

    #[test]
    fn test_plugin_commands() {
        let ptr = plugin_get_commands();
        assert!(!ptr.is_null());
        let cmds_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let cmds: Vec<PluginCommand> = serde_json::from_str(&cmds_str).unwrap();
        assert!(cmds.len() >= 5);
        assert!(cmds.iter().any(|c| c.name == "echo"));
        assert!(cmds.iter().any(|c| c.name == "serial_send"));
        plugin_free_string(ptr);
    }

    #[test]
    fn test_plugin_capabilities() {
        let ptr = plugin_get_capabilities();
        assert!(!ptr.is_null());
        let caps_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let caps: Vec<PluginCapability> = parse_capabilities(&caps_str).unwrap();
        assert!(caps.contains(&PluginCapability::SerialPort));
        assert!(caps.contains(&PluginCapability::Progress));
        plugin_free_string(ptr);
    }

    #[test]
    fn test_execute_echo() {
        let cmd = CString::new("echo").unwrap();
        let params = CString::new(r#"{"data": "hello"}"#).unwrap();
        let ptr = plugin_execute(cmd.as_ptr(), params.as_ptr());
        let result_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let result: PluginResult = serde_json::from_str(&result_str).unwrap();
        assert!(result.success);
        assert_eq!(result.result.unwrap(), serde_json::json!("hello"));
        plugin_free_string(ptr);
    }

    #[test]
    fn test_execute_add() {
        let cmd = CString::new("add").unwrap();
        let params = CString::new(r#"{"a": 3.0, "b": 4.0}"#).unwrap();
        let ptr = plugin_execute(cmd.as_ptr(), params.as_ptr());
        let result_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let result: PluginResult = serde_json::from_str(&result_str).unwrap();
        assert!(result.success);
        assert_eq!(result.result.unwrap(), serde_json::json!(7.0));
        plugin_free_string(ptr);
    }

    #[test]
    fn test_execute_echo_missing_param() {
        let cmd = CString::new("echo").unwrap();
        let params = CString::new("{}").unwrap();
        let ptr = plugin_execute(cmd.as_ptr(), params.as_ptr());
        let result_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let result: PluginResult = serde_json::from_str(&result_str).unwrap();
        assert!(!result.success);
        plugin_free_string(ptr);
    }

    #[test]
    fn test_execute_unknown_command() {
        let cmd = CString::new("unknown").unwrap();
        let params = CString::new("{}").unwrap();
        let ptr = plugin_execute(cmd.as_ptr(), params.as_ptr());
        let result_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let result: PluginResult = serde_json::from_str(&result_str).unwrap();
        assert!(!result.success);
        plugin_free_string(ptr);
    }

    #[test]
    fn test_plugin_ui_layout() {
        let ptr = plugin_get_ui_layout();
        assert!(!ptr.is_null());
        let json_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let layout = parse_ui_layout(&json_str).unwrap();
        match layout {
            UiLayoutNode::SplitVertical { children, .. } => {
                assert_eq!(children.len(), 3); // buttons, serial send, output
                // Verify first panel has ButtonGroup
                if let UiLayoutNode::Panel { content: UiContent::ButtonGroup { buttons, .. }, .. } = &children[0] {
                    assert_eq!(buttons.len(), 4);
                    assert_eq!(buttons[0].command, "echo");
                    assert_eq!(buttons[1].command, "add");
                } else {
                    panic!("Expected ButtonGroup in first panel");
                }
                // Verify second panel has Input
                if let UiLayoutNode::Panel { content: UiContent::Input { command, .. }, .. } = &children[1] {
                    assert_eq!(command, "serial_send");
                } else {
                    panic!("Expected Input in second panel");
                }
            }
            _ => panic!("Expected SplitVertical"),
        }
        plugin_free_string(ptr);
    }

    #[test]
    fn test_null_command_returns_error() {
        let ptr = plugin_execute(std::ptr::null(), std::ptr::null());
        assert!(!ptr.is_null());
        let result_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let result: PluginResult = serde_json::from_str(&result_str).unwrap();
        assert!(!result.success);
        plugin_free_string(ptr);
    }
}
