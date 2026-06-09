/// SerialRUN MicroPython IDE Plugin
///
/// Provides REPL terminal, file management, and code editing
/// for MicroPython devices (ESP32, ESP8266, RP2040, etc.)

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Mutex, OnceLock};

use serialrun_plugin_api::*;

// ============================================================================
// Panic-safe wrapper
// ============================================================================

fn catch_plugin_panic<F: FnOnce() -> *mut c_char + std::panic::UnwindSafe>(f: F) -> *mut c_char {
    match catch_unwind(f) {
        Ok(ptr) => ptr,
        Err(_) => {
            let err = PluginResult::error("Plugin panicked internally");
            let json = serde_json::to_string(&err).unwrap_or_default();
            CString::new(json).unwrap_or_default().into_raw()
        }
    }
}

// ============================================================================
// Plugin State
// ============================================================================

static CALLBACKS: OnceLock<Mutex<Option<PluginCallbacks>>> = OnceLock::new();

fn get_callbacks() -> Option<PluginCallbacks> {
    let mutex = CALLBACKS.get()?;
    match mutex.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => {
            log::warn!("Mutex poisoned in mpy-ide, recovering");
            poisoned.into_inner().clone()
        }
    }
}

// ============================================================================
// MicroPython REPL Protocol
// ============================================================================

fn enter_raw_repl() -> bool {
    let Some(cb) = get_callbacks() else { return false };
    let Some(write) = cb.serial_write else { return false };
    let Some(read) = cb.serial_read else { return false };

    let ctrl_a = [0x01u8];
    if write(ctrl_a.as_ptr(), 1) < 0 {
        return false;
    }

    let mut buf = [0u8; 256];
    let n = read(buf.as_mut_ptr(), buf.len() as u32, 1000);
    if n > 0 {
        let resp = String::from_utf8_lossy(&buf[..n as usize]);
        resp.contains("raw REPL")
    } else {
        false
    }
}

fn exit_raw_repl() -> bool {
    let Some(cb) = get_callbacks() else { return false };
    let Some(write) = cb.serial_write else { return false };

    let ctrl_b = [0x02u8];
    write(ctrl_b.as_ptr(), 1) >= 0
}

fn send_interrupt() -> bool {
    let Some(cb) = get_callbacks() else { return false };
    let Some(write) = cb.serial_write else { return false };

    let ctrl_c = [0x03u8];
    write(ctrl_c.as_ptr(), 1) >= 0
}

fn send_soft_reset() -> bool {
    let Some(cb) = get_callbacks() else { return false };
    let Some(write) = cb.serial_write else { return false };

    let ctrl_d = [0x04u8];
    write(ctrl_d.as_ptr(), 1) >= 0
}

fn execute_code(code: &str) -> Option<String> {
    let cb = get_callbacks()?;
    let write = cb.serial_write?;
    let read = cb.serial_read?;

    if !enter_raw_repl() {
        return Some("Error: Failed to enter Raw REPL".to_string());
    }

    let exec_cmd = format!("exec(compile(\"{}\", '<input>', 'exec'))\x04", code.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"));

    if write(exec_cmd.as_ptr(), exec_cmd.len() as u32) < 0 {
        exit_raw_repl();
        return Some("Error: Failed to send code".to_string());
    }

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
            if response.contains("\x04>") || response.ends_with(">") {
                break;
            }
        } else if !response.is_empty() {
            break;
        }
    }

    exit_raw_repl();

    let cleaned = response
        .replace("raw REPL; CTRL-B to exit\r\n", "")
        .replace(">>>", "")
        .replace("\r\n>", "")
        .replace("\x04", "")
        .trim()
        .to_string();

    Some(cleaned)
}

fn detect_device() -> Option<String> {
    let cb = get_callbacks()?;
    let write = cb.serial_write?;
    let read = cb.serial_read?;
    let is_connected = cb.serial_is_connected?;

    if !is_connected() {
        return None;
    }

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
// File Management
// ============================================================================

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
    execute_code(&code)
}

fn read_file(path: &str) -> Option<String> {
    let code = format!(
        r#"with open('{}', 'r') as f:
    print(f.read())"#,
        path
    );
    execute_code(&code)
}

fn write_file(path: &str, content: &str) -> bool {
    let code = format!(
        r#"with open('{}', 'w') as f:
    f.write("""{}""")"#,
        path,
        content.replace('\\', "\\\\").replace('`', "\\`")
    );
    execute_code(&code).is_some()
}

#[allow(dead_code)]
fn delete_file(path: &str) -> bool {
    let code = format!("import os\nos.remove('{}')", path);
    execute_code(&code).is_some()
}

#[allow(dead_code)]
fn make_dir(path: &str) -> bool {
    let code = format!("import os\nos.mkdir('{}')", path);
    execute_code(&code).is_some()
}

#[allow(dead_code)]
fn file_exists(path: &str) -> bool {
    let code = format!("import os\nprint(os.path.exists('{}'))", path);
    execute_code(&code).map_or(false, |o| o.contains("True"))
}

// ============================================================================
// Plugin FFI Implementation
// ============================================================================

#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char {
    catch_plugin_panic(|| {
        let info = PluginInfo {
            name: "serialrun-mpy-ide".to_string(),
            version: "0.1.0".to_string(),
            description: "MicroPython IDE - REPL, file management, code editing".to_string(),
            author: "YaoIsAI".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        CString::new(json).unwrap().into_raw()
    })
}

#[no_mangle]
pub extern "C" fn plugin_get_commands() -> *mut c_char {
    catch_plugin_panic(|| {
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
        let json = serde_json::to_string(&commands).unwrap();
        CString::new(json).unwrap().into_raw()
    })
}

#[no_mangle]
pub extern "C" fn plugin_execute(command: *const c_char, params: *const c_char) -> *mut c_char {
    catch_plugin_panic(AssertUnwindSafe(|| {
        if command.is_null() {
            let result = PluginResult::error("No command specified");
            return CString::new(serde_json::to_string(&result).unwrap())
                .unwrap()
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

        CString::new(serde_json::to_string(&result).unwrap())
            .unwrap()
            .into_raw()
    }))
}

#[no_mangle]
pub extern "C" fn plugin_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}

#[no_mangle]
pub extern "C" fn plugin_get_capabilities() -> *mut c_char {
    catch_plugin_panic(|| {
        let caps = vec![
            PluginCapability::SerialPort,
            PluginCapability::UiPanel,
            PluginCapability::UiLayout,
            PluginCapability::FileSystem,
            PluginCapability::Logging,
        ];
        let json = serialize_capabilities(&caps).unwrap();
        CString::new(json).unwrap().into_raw()
    })
}

#[no_mangle]
pub extern "C" fn plugin_init(callbacks: *const PluginCallbacks) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        if callbacks.is_null() {
            return false;
        }
        let cb = unsafe { *callbacks };
        let store = CALLBACKS.get_or_init(|| Mutex::new(None));
        match store.lock() {
            Ok(mut guard) => {
                *guard = Some(cb);
            }
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                *guard = Some(cb);
            }
        }
        if let Some(log) = cb.log_info {
            let msg = CString::new("MicroPython IDE plugin initialized").unwrap();
            log(msg.as_ptr());
        }
        true
    }))
    .unwrap_or(false)
}

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

#[no_mangle]
pub extern "C" fn plugin_get_ui_layout() -> *mut c_char {
    catch_plugin_panic(|| {
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
        let json = serialize_ui_layout(&layout).unwrap();
        CString::new(json).unwrap().into_raw()
    })
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
        plugin_free_string(cmds_json);
    }

    #[test]
    fn test_plugin_capabilities() {
        let caps_json = plugin_get_capabilities();
        let caps_str = unsafe { CStr::from_ptr(caps_json).to_string_lossy() };
        let caps: Vec<PluginCapability> = parse_capabilities(&caps_str).unwrap();
        assert!(caps.contains(&PluginCapability::SerialPort));
        assert!(caps.contains(&PluginCapability::UiLayout));
        plugin_free_string(caps_json);
    }

    #[test]
    fn test_plugin_ui_layout() {
        let layout_json = plugin_get_ui_layout();
        let layout_str = unsafe { CStr::from_ptr(layout_json).to_string_lossy() };
        let layout = parse_ui_layout(&layout_str).unwrap();
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
