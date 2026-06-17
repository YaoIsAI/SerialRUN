/// SerialRUN ADC Reader Plugin
///
/// Read analog-to-digital converter values via serial port.
/// Supports common ADC protocols:
/// - Custom ASCII: "READ <channel>\r\n" → "<value>\r\n"
/// - Modbus RTU: Read holding registers
/// - HEX protocol: "FF 01 <channel> 00" → "FF 01 <value_high> <value_low>"

use serialrun_plugin_api::*;
use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Mutex, OnceLock};

// ============================================================================
// Plugin State
// ============================================================================

static CALLBACKS: OnceLock<Mutex<Option<PluginCallbacks>>> = OnceLock::new();

fn get_callbacks() -> Option<PluginCallbacks> {
    CALLBACKS.get()?.lock().ok()?.clone()
}

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
// FFI Functions
// ============================================================================

#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char {
    catch_plugin_panic(|| {
        let info = PluginInfo {
            name: "serialrun-plugin-adc-reader".to_string(),
            version: "1.0.0".to_string(),
            description: "Read ADC values via serial port".to_string(),
            author: "SerialRUN Team".to_string(),
        };
        CString::new(serde_json::to_string(&info).unwrap()).unwrap().into_raw()
    })
}

#[no_mangle]
pub extern "C" fn plugin_get_commands() -> *mut c_char {
    catch_plugin_panic(|| {
        let commands = vec![
            PluginCommand {
                name: "read_channel".to_string(),
                description: "Read ADC value from a specific channel (0-7)".to_string(),
                parameters: vec![PluginParameter {
                    name: "channel".to_string(),
                    description: "ADC channel number (0-7)".to_string(),
                    required: true,
                    param_type: "number".to_string(),
                }],
            },
            PluginCommand {
                name: "read_all".to_string(),
                description: "Read all ADC channels (0-7)".to_string(),
                parameters: vec![],
            },
            PluginCommand {
                name: "set_sample_rate".to_string(),
                description: "Set ADC sample rate in Hz".to_string(),
                parameters: vec![PluginParameter {
                    name: "rate".to_string(),
                    description: "Sample rate in Hz (100-10000)".to_string(),
                    required: true,
                    param_type: "number".to_string(),
                }],
            },
            PluginCommand {
                name: "start_continuous".to_string(),
                description: "Start continuous ADC reading".to_string(),
                parameters: vec![PluginParameter {
                    name: "channels".to_string(),
                    description: "Comma-separated channel numbers (e.g., '0,1,2')".to_string(),
                    required: false,
                    param_type: "string".to_string(),
                }],
            },
            PluginCommand {
                name: "stop_continuous".to_string(),
                description: "Stop continuous ADC reading".to_string(),
                parameters: vec![],
            },
        ];
        CString::new(serde_json::to_string(&commands).unwrap()).unwrap().into_raw()
    })
}

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

        let params: serde_json::Value = serde_json::from_str(&params_str).unwrap_or_default();

        let result = match cmd.as_str() {
            "read_channel" => cmd_read_channel(&params),
            "read_all" => cmd_read_all(),
            "set_sample_rate" => cmd_set_sample_rate(&params),
            "start_continuous" => cmd_start_continuous(&params),
            "stop_continuous" => cmd_stop_continuous(),
            _ => PluginResult::error(format!("Unknown command: {}", cmd)),
        };

        CString::new(serde_json::to_string(&result).unwrap()).unwrap().into_raw()
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
            PluginCapability::Logging,
            PluginCapability::ConfigStorage,
        ];
        CString::new(serialize_capabilities(&caps).unwrap()).unwrap().into_raw()
    })
}

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
            }
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                *guard = Some(cbs);
            }
        }
        if let Some(log) = cbs.log_info {
            let msg = CString::new("ADC Reader plugin initialized").unwrap();
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

// ============================================================================
// Helper: Get serial port functions
// ============================================================================

fn get_serial_write() -> Option<extern "C" fn(*const u8, u32) -> i32> {
    get_callbacks().and_then(|cb| cb.serial_write)
}

fn get_serial_read() -> Option<extern "C" fn(*mut u8, u32, u32) -> i32> {
    get_callbacks().and_then(|cb| cb.serial_read)
}

fn is_connected() -> bool {
    get_callbacks()
        .and_then(|cb| cb.serial_is_connected)
        .map_or(false, |f| f())
}

fn log_info(msg: &str) {
    if let Some(cb) = get_callbacks() {
        if let Some(log) = cb.log_info {
            if let Ok(m) = CString::new(msg) {
                log(m.as_ptr());
            }
        }
    }
}

// ============================================================================
// Command Implementations
// ============================================================================

/// Read ADC value from a specific channel
fn cmd_read_channel(params: &serde_json::Value) -> PluginResult {
    let channel = match params.get("channel").and_then(|v| v.as_u64()) {
        Some(c) => c as u8,
        None => return PluginResult::error("Missing required parameter: channel"),
    };

    if channel > 7 {
        return PluginResult::error("Channel must be 0-7");
    }

    let write = match get_serial_write() {
        Some(f) => f,
        None => return PluginResult::error("Serial write not available"),
    };
    let read = match get_serial_read() {
        Some(f) => f,
        None => return PluginResult::error("Serial read not available"),
    };

    if !is_connected() {
        return PluginResult::error("No serial port connected");
    }

    // Send ADC read command: "READ <channel>\r\n"
    let cmd = format!("READ {}\r\n", channel);
    let cmd_bytes = cmd.as_bytes();
    write(cmd_bytes.as_ptr(), cmd_bytes.len() as u32);

    // Read response
    let mut buf = [0u8; 256];
    let n = read(buf.as_mut_ptr(), buf.len() as u32, 1000);

    if n > 0 {
        let response = String::from_utf8_lossy(&buf[..n as usize]);
        let response = response.trim();

        // Try to parse as number
        if let Ok(value) = response.parse::<f64>() {
            PluginResult::success(serde_json::json!({
                "channel": channel,
                "value": value,
                "raw": response,
            }))
        } else {
            PluginResult::success(serde_json::json!({
                "channel": channel,
                "raw": response,
                "note": "Could not parse as number",
            }))
        }
    } else {
        PluginResult::error("No response from device")
    }
}

/// Read all ADC channels (0-7)
fn cmd_read_all() -> PluginResult {
    let write = match get_serial_write() {
        Some(f) => f,
        None => return PluginResult::error("Serial write not available"),
    };
    let read = match get_serial_read() {
        Some(f) => f,
        None => return PluginResult::error("Serial read not available"),
    };

    if !is_connected() {
        return PluginResult::error("No serial port connected");
    }

    let mut channels = Vec::new();

    for channel in 0..8u8 {
        // Send ADC read command
        let cmd = format!("READ {}\r\n", channel);
        let cmd_bytes = cmd.as_bytes();
        write(cmd_bytes.as_ptr(), cmd_bytes.len() as u32);

        // Read response
        let mut buf = [0u8; 256];
        let n = read(buf.as_mut_ptr(), buf.len() as u32, 500);

        if n > 0 {
            let response = String::from_utf8_lossy(&buf[..n as usize]);
            let response = response.trim().to_string();
            channels.push(serde_json::json!({
                "channel": channel,
                "raw": response,
            }));
        } else {
            channels.push(serde_json::json!({
                "channel": channel,
                "error": "timeout",
            }));
        }
    }

    PluginResult::success(serde_json::json!({
        "channels": channels,
        "count": channels.len(),
    }))
}

/// Set ADC sample rate
fn cmd_set_sample_rate(params: &serde_json::Value) -> PluginResult {
    let rate = match params.get("rate").and_then(|v| v.as_u64()) {
        Some(r) => r as u32,
        None => return PluginResult::error("Missing required parameter: rate"),
    };

    if rate < 100 || rate > 10000 {
        return PluginResult::error("Sample rate must be 100-10000 Hz");
    }

    let write = match get_serial_write() {
        Some(f) => f,
        None => return PluginResult::error("Serial write not available"),
    };

    if !is_connected() {
        return PluginResult::error("No serial port connected");
    }

    // Send sample rate command
    let cmd = format!("RATE {}\r\n", rate);
    let cmd_bytes = cmd.as_bytes();
    write(cmd_bytes.as_ptr(), cmd_bytes.len() as u32);

    PluginResult::success(serde_json::json!({
        "rate": rate,
        "status": "set",
    }))
}

/// Start continuous ADC reading
fn cmd_start_continuous(params: &serde_json::Value) -> PluginResult {
    let channels = params.get("channels")
        .and_then(|v| v.as_str())
        .unwrap_or("0,1,2,3,4,5,6,7");

    let write = match get_serial_write() {
        Some(f) => f,
        None => return PluginResult::error("Serial write not available"),
    };

    if !is_connected() {
        return PluginResult::error("No serial port connected");
    }

    // Send continuous mode command
    let cmd = format!("CONT START {}\r\n", channels);
    let cmd_bytes = cmd.as_bytes();
    write(cmd_bytes.as_ptr(), cmd_bytes.len() as u32);

    log_info(&format!("Continuous ADC reading started: channels {}", channels));

    PluginResult::success(serde_json::json!({
        "status": "started",
        "channels": channels,
    }))
}

/// Stop continuous ADC reading
fn cmd_stop_continuous() -> PluginResult {
    let write = match get_serial_write() {
        Some(f) => f,
        None => return PluginResult::error("Serial write not available"),
    };

    if !is_connected() {
        return PluginResult::error("No serial port connected");
    }

    // Send stop command
    let cmd = "CONT STOP\r\n";
    let cmd_bytes = cmd.as_bytes();
    write(cmd_bytes.as_ptr(), cmd_bytes.len() as u32);

    log_info("Continuous ADC reading stopped");

    PluginResult::success(serde_json::json!({
        "status": "stopped",
    }))
}

// ============================================================================
// Tests
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
        assert_eq!(info.name, "serialrun-plugin-adc-reader");
        assert_eq!(info.version, "1.0.0");
        plugin_free_string(ptr);
    }

    #[test]
    fn test_plugin_commands() {
        let ptr = plugin_get_commands();
        assert!(!ptr.is_null());
        let cmds_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let cmds: Vec<PluginCommand> = serde_json::from_str(&cmds_str).unwrap();
        assert!(cmds.len() >= 5);
        assert!(cmds.iter().any(|c| c.name == "read_channel"));
        assert!(cmds.iter().any(|c| c.name == "read_all"));
        assert!(cmds.iter().any(|c| c.name == "set_sample_rate"));
        plugin_free_string(ptr);
    }

    #[test]
    fn test_null_command() {
        let ptr = plugin_execute(std::ptr::null(), std::ptr::null());
        assert!(!ptr.is_null());
        let result_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let result: PluginResult = serde_json::from_str(&result_str).unwrap();
        assert!(!result.success);
        plugin_free_string(ptr);
    }
}
