/// SerialRUN MicroPython IDE Plugin
///
/// Provides REPL terminal, file management, and code editing
/// for MicroPython devices (ESP32, ESP8266, RP2040, etc.)

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};

use serialrun_plugin_api::*;

// ============================================================================
// Plugin State
// ============================================================================

static CALLBACKS: OnceLock<Mutex<Option<PluginCallbacks>>> = OnceLock::new();

fn get_callbacks() -> Option<PluginCallbacks> {
    CALLBACKS.get()?.lock().ok()?.clone()
}

fn store_callbacks(cb: PluginCallbacks) {
    let _ = CALLBACKS.set(Mutex::new(Some(cb)));
}

// ============================================================================
// MicroPython REPL Protocol
// ============================================================================

/// Enter Raw REPL mode (Ctrl-A = 0x01)
fn enter_raw_repl() -> bool {
    let Some(cb) = get_callbacks() else { return false };
    let Some(write) = cb.serial_write else { return false };
    let Some(read) = cb.serial_read else { return false };

    // Send Ctrl-A to enter raw REPL
    let ctrl_a = [0x01u8];
    if write(ctrl_a.as_ptr(), 1) < 0 {
        return false;
    }

    // Wait for "raw REPL; CTRL-B to exit\r\n>"
    let mut buf = [0u8; 256];
    let n = read(buf.as_mut_ptr(), buf.len() as u32, 1000);
    if n > 0 {
        let resp = String::from_utf8_lossy(&buf[..n as usize]);
        resp.contains("raw REPL")
    } else {
        false
    }
}

/// Exit Raw REPL mode (Ctrl-B = 0x02)
fn exit_raw_repl() -> bool {
    let Some(cb) = get_callbacks() else { return false };
    let Some(write) = cb.serial_write else { return false };

    let ctrl_b = [0x02u8];
    write(ctrl_b.as_ptr(), 1) >= 0
}

/// Send Ctrl-C to interrupt current execution
fn send_interrupt() -> bool {
    let Some(cb) = get_callbacks() else { return false };
    let Some(write) = cb.serial_write else { return false };

    let ctrl_c = [0x03u8];
    write(ctrl_c.as_ptr(), 1) >= 0
}

/// Send Ctrl-D for soft reset
fn send_soft_reset() -> bool {
    let Some(cb) = get_callbacks() else { return false };
    let Some(write) = cb.serial_write else { return false };

    let ctrl_d = [0x04u8];
    write(ctrl_d.as_ptr(), 1) >= 0
}

/// Execute Python code in Raw REPL and return output
fn execute_code(code: &str) -> Option<String> {
    let cb = get_callbacks()?;
    let write = cb.serial_write?;
    let read = cb.serial_read?;

    if !enter_raw_repl() {
        return Some("Error: Failed to enter Raw REPL".to_string());
    }

    // Send the code as a raw REPL command
    // Format: exec(compile(<code>, '<input>', 'exec'))
    let exec_cmd = format!("exec(compile(\"{}\", '<input>', 'exec'))\x04", code.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"));

    if write(exec_cmd.as_ptr(), exec_cmd.len() as u32) < 0 {
        exit_raw_repl();
        return Some("Error: Failed to send code".to_string());
    }

    // Read response
    let mut buf = [0u8; 4096];
    let mut response = String::new();
    let start = std::time::Instant::now();

    loop {
        if start.elapsed().as_millis() > 5000 {
            break;
        }
        let n = read(buf.as_mut_ptr(), buf.len() as u32, 100);
        if n > 0 {
            response.push_str(&String::from_utf8_lossy(&buf[..n as usize]));
            // Check for end markers
            if response.contains("\x04>") || response.ends_with(">") {
                break;
            }
        } else if !response.is_empty() {
            break;
        }
    }

    exit_raw_repl();

    // Clean up the response (remove REPL artifacts)
    let cleaned = response
        .replace("raw REPL; CTRL-B to exit\r\n", "")
        .replace(">>>", "")
        .replace("\r\n>", "")
        .replace("\x04", "")
        .trim()
        .to_string();

    Some(cleaned)
}

/// Detect if a MicroPython device is connected
fn detect_device() -> Option<String> {
    let cb = get_callbacks()?;
    let write = cb.serial_write?;
    let read = cb.serial_read?;
    let is_connected = cb.serial_is_connected?;

    if !is_connected() {
        return None;
    }

    // Send a simple Python command to check if it's MicroPython
    let test_cmd = "import sys; print(sys.version)\r\n";
    if write(test_cmd.as_ptr(), test_cmd.len() as u32) < 0 {
        return None;
    }

    let mut buf = [0u8; 256];
    let n = read(buf.as_mut_ptr(), buf.len() as u32, 2000);
    if n > 0 {
        let resp = String::from_utf8_lossy(&buf[..n as usize]);
        if resp.contains("MicroPython") || resp.contains("micropython") {
            return Some(resp.trim().to_string());
        }
    }
    None
}

// ============================================================================
// File Management (via Raw REPL)
// ============================================================================

/// List files in a directory on the device
fn list_dir(path: &str) -> Option<String> {
    let code = format!(
        r#"import os
items = os.listdir('{}')
result = []
for item in items:
    try:
        st = os.stat('{}' + '/' + item)
        is_dir = bool(st[0] & 0x4000)
        result.append({{'name': item, 'is_dir': is_dir, 'size': st[6]}})
    except:
        result.append({{'name': item, 'is_dir': False, 'size': 0}})
import json
print(json.dumps(result))"#,
        path, path
    );

    let output = execute_code(&code)?;
    Some(output)
}

/// Read a file from the device
fn read_file(path: &str) -> Option<String> {
    let code = format!(
        r#"with open('{}', 'r') as f:
    print(f.read())"#,
        path
    );
    execute_code(&code)
}

/// Write content to a file on the device
fn write_file(path: &str, content: &str) -> bool {
    let code = format!(
        r#"with open('{}', 'w') as f:
    f.write("""{}""")"#,
        path,
        content.replace('\\', "\\\\").replace('`', "\\`")
    );
    execute_code(&code).is_some()
}

/// Delete a file on the device
#[allow(dead_code)]
fn delete_file(path: &str) -> bool {
    let code = format!("import os\nos.remove('{}')", path);
    execute_code(&code).is_some()
}

/// Create a directory on the device
#[allow(dead_code)]
fn make_dir(path: &str) -> bool {
    let code = format!("import os\nos.mkdir('{}')", path);
    execute_code(&code).is_some()
}

/// Check if a file exists on the device
#[allow(dead_code)]
fn file_exists(path: &str) -> bool {
    let code = format!("import os\nprint(os.path.exists('{}'))", path);
    if let Some(output) = execute_code(&code) {
        output.contains("True")
    } else {
        false
    }
}

// ============================================================================
// Plugin FFI Implementation
// ============================================================================

#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char {
    let info = PluginInfo {
        name: "serialrun-mpy-ide".to_string(),
        version: "0.1.0".to_string(),
        description: "MicroPython IDE - REPL, file management, code editing".to_string(),
        author: "YaoIsAI".to_string(),
    };
    let json = serde_json::to_string(&info).unwrap_or_default();
    CString::new(json).unwrap_or_default().into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_get_commands() -> *mut c_char {
    let commands = vec![
        PluginCommand {
            name: "detect".to_string(),
            description: "Detect MicroPython device".to_string(),
            parameters: vec![],
        },
        PluginCommand {
            name: "execute".to_string(),
            description: "Execute Python code on device".to_string(),
            parameters: vec![PluginParameter {
                name: "code".to_string(),
                description: "Python code to execute".to_string(),
                required: true,
                param_type: "string".to_string(),
            }],
        },
        PluginCommand {
            name: "list_dir".to_string(),
            description: "List files in device directory".to_string(),
            parameters: vec![PluginParameter {
                name: "path".to_string(),
                description: "Directory path".to_string(),
                required: false,
                param_type: "string".to_string(),
            }],
        },
        PluginCommand {
            name: "read_file".to_string(),
            description: "Read file from device".to_string(),
            parameters: vec![PluginParameter {
                name: "path".to_string(),
                description: "File path".to_string(),
                required: true,
                param_type: "string".to_string(),
            }],
        },
        PluginCommand {
            name: "write_file".to_string(),
            description: "Write file to device".to_string(),
            parameters: vec![
                PluginParameter {
                    name: "path".to_string(),
                    description: "File path".to_string(),
                    required: true,
                    param_type: "string".to_string(),
                },
                PluginParameter {
                    name: "content".to_string(),
                    description: "File content".to_string(),
                    required: true,
                    param_type: "string".to_string(),
                },
            ],
        },
        PluginCommand {
            name: "interrupt".to_string(),
            description: "Interrupt current execution (Ctrl-C)".to_string(),
            parameters: vec![],
        },
        PluginCommand {
            name: "reset".to_string(),
            description: "Soft reset device (Ctrl-D)".to_string(),
            parameters: vec![],
        },
    ];
    let json = serde_json::to_string(&commands).unwrap_or_default();
    CString::new(json).unwrap_or_default().into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_execute(command: *const c_char, params: *const c_char) -> *mut c_char {
    if command.is_null() {
        let result = PluginResult::error("No command specified");
        return CString::new(serde_json::to_string(&result).unwrap_or_default())
            .unwrap_or_default()
            .into_raw();
    }

    let cmd = unsafe { CStr::from_ptr(command).to_string_lossy() };
    let params_str = if params.is_null() {
        "{}".to_string()
    } else {
        unsafe { CStr::from_ptr(params).to_string_lossy().to_string() }
    };

    let params_json: serde_json::Value = serde_json::from_str(&params_str)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    let result = match cmd.as_ref() {
        "detect" => {
            if let Some(version) = detect_device() {
                PluginResult::success(serde_json::json!({
                    "detected": true,
                    "firmware": version
                }))
            } else {
                PluginResult::success(serde_json::json!({
                    "detected": false,
                    "error": "No MicroPython device found"
                }))
            }
        }
        "execute" => {
            let code = params_json.get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if let Some(output) = execute_code(code) {
                PluginResult::success(serde_json::json!({
                    "output": output
                }))
            } else {
                PluginResult::error("Failed to execute code")
            }
        }
        "list_dir" => {
            let path = params_json.get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("/");
            if let Some(output) = list_dir(path) {
                PluginResult::success(serde_json::json!({
                    "entries": output
                }))
            } else {
                PluginResult::error("Failed to list directory")
            }
        }
        "read_file" => {
            let path = params_json.get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if let Some(content) = read_file(path) {
                PluginResult::success(serde_json::json!({
                    "content": content
                }))
            } else {
                PluginResult::error("Failed to read file")
            }
        }
        "write_file" => {
            let path = params_json.get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let content = params_json.get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if write_file(path, content) {
                PluginResult::success(serde_json::json!({
                    "success": true
                }))
            } else {
                PluginResult::error("Failed to write file")
            }
        }
        "interrupt" => {
            if send_interrupt() {
                PluginResult::success(serde_json::json!({"interrupted": true}))
            } else {
                PluginResult::error("Failed to send interrupt")
            }
        }
        "reset" => {
            if send_soft_reset() {
                PluginResult::success(serde_json::json!({"reset": true}))
            } else {
                PluginResult::error("Failed to reset device")
            }
        }
        _ => PluginResult::error(format!("Unknown command: {}", cmd)),
    };

    CString::new(serde_json::to_string(&result).unwrap_or_default())
        .unwrap_or_default()
        .into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_get_capabilities() -> *mut c_char {
    let caps = vec![
        PluginCapability::SerialPort,
        PluginCapability::UiPanel,
        PluginCapability::UiLayout,
        PluginCapability::FileSystem,
        PluginCapability::Logging,
    ];
    let json = serialize_capabilities(&caps).unwrap_or_default();
    CString::new(json).unwrap_or_default().into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_init(callbacks: *const PluginCallbacks) -> bool {
    if callbacks.is_null() {
        return false;
    }
    let cb = unsafe { *callbacks };
    store_callbacks(cb);

    if let Some(log) = cb.log_info {
        let msg = CString::new("MicroPython IDE plugin initialized").unwrap_or_default();
        log(msg.as_ptr());
    }
    true
}

#[no_mangle]
pub extern "C" fn plugin_cleanup() {
    if let Some(cb) = get_callbacks() {
        if let Some(log) = cb.log_info {
            let msg = CString::new("MicroPython IDE plugin cleaned up").unwrap_or_default();
            log(msg.as_ptr());
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_get_ui_layout() -> *mut c_char {
    let layout = UiLayoutNode::SplitHorizontal {
        children: vec![
            UiLayoutNode::Panel {
                id: "file_browser".to_string(),
                title: "\u{1F4C1} Files".to_string(),
                content: UiContent::TreeView,
                width: Some(250.0),
                height: None,
            },
            UiLayoutNode::SplitVertical {
                children: vec![
                    UiLayoutNode::Panel {
                        id: "editor".to_string(),
                        title: "\u{1F4DD} Editor".to_string(),
                        content: UiContent::CodeEditor { language: "python".to_string() },
                        width: None,
                        height: None,
                    },
                    UiLayoutNode::Panel {
                        id: "repl".to_string(),
                        title: "\u{1F4AC} REPL".to_string(),
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

    let json = serialize_ui_layout(&layout).unwrap_or_default();
    CString::new(json).unwrap_or_default().into_raw()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_info() {
        let info_json = plugin_get_info();
        let info_str = unsafe { CStr::from_ptr(info_json).to_string_lossy() };
        let info: PluginInfo = serde_json::from_str(&info_str).unwrap();
        assert_eq!(info.name, "serialrun-mpy-ide");
        assert_eq!(info.version, "0.1.0");
        // Free the string
        plugin_free_string(info_json);
    }

    #[test]
    fn test_plugin_commands() {
        let cmds_json = plugin_get_commands();
        let cmds_str = unsafe { CStr::from_ptr(cmds_json).to_string_lossy() };
        let cmds: Vec<PluginCommand> = serde_json::from_str(&cmds_str).unwrap();
        assert!(cmds.len() >= 5);
        assert!(cmds.iter().any(|c| c.name == "detect"));
        assert!(cmds.iter().any(|c| c.name == "execute"));
        assert!(cmds.iter().any(|c| c.name == "list_dir"));
        plugin_free_string(cmds_json);
    }

    #[test]
    fn test_plugin_capabilities() {
        let caps_json = plugin_get_capabilities();
        let caps_str = unsafe { CStr::from_ptr(caps_json).to_string_lossy() };
        let caps: Vec<PluginCapability> = parse_capabilities(&caps_str).unwrap();
        assert!(caps.contains(&PluginCapability::SerialPort));
        assert!(caps.contains(&PluginCapability::UiLayout));
        assert!(caps.contains(&PluginCapability::FileSystem));
        plugin_free_string(caps_json);
    }

    #[test]
    fn test_plugin_ui_layout() {
        let layout_json = plugin_get_ui_layout();
        let layout_str = unsafe { CStr::from_ptr(layout_json).to_string_lossy() };
        let layout = parse_ui_layout(&layout_str).unwrap();
        // Verify it's a horizontal split with 2 children
        match layout {
            UiLayoutNode::SplitHorizontal { children, .. } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected SplitHorizontal"),
        }
        plugin_free_string(layout_json);
    }

    #[test]
    fn test_execute_unknown_command() {
        let cmd = CString::new("unknown").unwrap();
        let params = CString::new("{}").unwrap();
        let result_json = plugin_execute(cmd.as_ptr(), params.as_ptr());
        let result_str = unsafe { CStr::from_ptr(result_json).to_string_lossy() };
        let result: PluginResult = serde_json::from_str(&result_str).unwrap();
        assert!(!result.success);
        plugin_free_string(result_json);
    }
}
