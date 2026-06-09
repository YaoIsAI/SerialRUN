/// SerialRUN STC ISP Flashing Plugin
///
/// Supports STC89, STC12, STC15, STC8, STC8G, STC8H series MCUs.
/// Uses the STC proprietary ISP protocol over UART.

mod chip;
mod protocol;

use chip::{ChipInfo, parse_hex_file};
use protocol::*;
use serialrun_plugin_api::{PluginCapability, PluginCallbacks, PluginInfo, PluginCommand, PluginParameter, PluginResult, PluginStatus, serialize_capabilities};
use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Mutex, OnceLock};

static CALLBACKS: OnceLock<Mutex<Option<PluginCallbacks>>> = OnceLock::new();

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
// FFI Functions (Required)
// ============================================================================

#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char {
    catch_plugin_panic(|| {
        let info = PluginInfo {
            name: "STC ISP Flasher".to_string(),
            version: "0.1.0".to_string(),
            description: "Flash STC series MCU via ISP protocol".to_string(),
            author: "SerialRUN".to_string(),
        };
        CString::new(serde_json::to_string(&info).unwrap()).unwrap().into_raw()
    })
}

#[no_mangle]
pub extern "C" fn plugin_get_commands() -> *mut c_char {
    catch_plugin_panic(|| {
        let commands = vec![
            PluginCommand {
                name: "flash".to_string(),
                description: "Flash firmware to STC MCU".to_string(),
                parameters: vec![
                    PluginParameter {
                        name: "firmware_path".to_string(),
                        description: "Path to HEX/BIN firmware file".to_string(),
                        required: true,
                        param_type: "string".to_string(),
                    },
                    PluginParameter {
                        name: "baud_rate".to_string(),
                        description: "Baud rate for ISP communication (default: 115200)".to_string(),
                        required: false,
                        param_type: "number".to_string(),
                    },
                ],
            },
            PluginCommand {
                name: "detect".to_string(),
                description: "Detect connected STC MCU".to_string(),
                parameters: vec![
                    PluginParameter {
                        name: "baud_rate".to_string(),
                        description: "Baud rate for detection (default: 9600)".to_string(),
                        required: false,
                        param_type: "number".to_string(),
                    },
                ],
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

        let params: serde_json::Value = unsafe {
            if params.is_null() {
                serde_json::json!({})
            } else {
                serde_json::from_str(&CStr::from_ptr(params).to_string_lossy()).unwrap_or_default()
            }
        };

        let result = match cmd.as_str() {
            "flash" => handle_flash(&params),
            "detect" => handle_detect(&params),
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

// ============================================================================
// Optional FFI Functions
// ============================================================================

#[no_mangle]
pub extern "C" fn plugin_get_capabilities() -> *mut c_char {
    catch_plugin_panic(|| {
        let caps = vec![PluginCapability::SerialPort, PluginCapability::Progress, PluginCapability::Logging];
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
            let msg = CString::new("STC ISP Flasher plugin initialized").unwrap();
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
// Command Handlers
// ============================================================================

fn get_callbacks() -> Option<PluginCallbacks> {
    let mutex = CALLBACKS.get()?;
    match mutex.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => {
            log::warn!("Mutex poisoned in stc-isp, recovering");
            poisoned.into_inner().clone()
        }
    }
}

fn log_info(msg: &str) {
    if let Some(cbs) = get_callbacks() {
        if let Some(log) = cbs.log_info {
            if let Ok(m) = CString::new(msg) {
                log(m.as_ptr());
            }
        }
    }
}

#[allow(dead_code)]
fn log_warn(msg: &str) {
    if let Some(cbs) = get_callbacks() {
        if let Some(log) = cbs.log_warn {
            if let Ok(m) = CString::new(msg) {
                log(m.as_ptr());
            }
        }
    }
}

#[allow(dead_code)]
fn log_error(msg: &str) {
    if let Some(cbs) = get_callbacks() {
        if let Some(log) = cbs.log_error {
            if let Ok(m) = CString::new(msg) {
                log(m.as_ptr());
            }
        }
    }
}

fn set_progress(percent: f32, msg: &str) {
    if let Some(cbs) = get_callbacks() {
        if let Some(progress) = cbs.progress_set {
            if let Ok(m) = CString::new(msg) {
                progress(percent, m.as_ptr());
            }
        }
    }
}

fn set_status(status: PluginStatus) {
    if let Some(cbs) = get_callbacks() {
        if let Some(set_status) = cbs.progress_set_status {
            set_status(status);
        }
    }
}

fn is_cancelled() -> bool {
    get_callbacks()
        .and_then(|cbs| cbs.progress_is_cancelled)
        .map_or(false, |f| f())
}

fn serial_write(data: &[u8]) -> bool {
    get_callbacks()
        .and_then(|cbs| cbs.serial_write)
        .map_or(false, |f| f(data.as_ptr(), data.len() as u32) > 0)
}

fn serial_read(buf: &mut [u8], timeout_ms: u32) -> Option<usize> {
    get_callbacks()
        .and_then(|cbs| cbs.serial_read)
        .map(|f| {
            let n = f(buf.as_mut_ptr(), buf.len() as u32, timeout_ms);
            if n > 0 { Some(n as usize) } else { None }
        })
        .flatten()
}

fn handle_detect(params: &serde_json::Value) -> PluginResult {
    let baud_rate = params["baud_rate"].as_u64().unwrap_or(9600) as u32;

    log_info(&format!("Detecting STC MCU at {} baud...", baud_rate));
    log_info("Please power cycle the MCU (disconnect then reconnect power) during detection.");
    set_progress(0.0, "Detecting MCU...");
    set_status(PluginStatus::Running);

    // STC ISP protocol: continuously send 0x7F at 9600 baud.
    // MCU bootloader listens for 0x7F after power-on and responds with chip info.
    // Send 0x7F every ~10ms for up to 15 seconds (user must power cycle MCU).
    let trigger = isp_trigger_packet();
    let total_duration = std::time::Duration::from_secs(15);
    let send_interval = std::time::Duration::from_millis(10);
    let start = std::time::Instant::now();
    let mut response_buf = [0u8; 128];

    loop {
        if is_cancelled() {
            set_status(PluginStatus::Idle);
            return PluginResult::error("Cancelled by user");
        }

        let elapsed = start.elapsed();
        if elapsed >= total_duration {
            break;
        }

        let percent = (elapsed.as_secs_f32() / total_duration.as_secs_f32()) * 100.0;
        if elapsed.as_secs() % 2 == 0 && elapsed.as_millis() % 500 < 10 {
            set_progress(percent, &format!("Sending ISP trigger... ({:.0}s / 15s) - Power cycle MCU NOW!", elapsed.as_secs_f32()));
        }

        // Send trigger byte
        if !serial_write(&trigger) {
            set_status(PluginStatus::Error);
            return PluginResult::error("Failed to send ISP trigger. Check serial connection.");
        }

        // Check for response (long enough to receive full handshake at 9600 baud)
        if let Some(n) = serial_read(&mut response_buf, 2000) {
            let hex: String = response_buf[..n].iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
            log_info(&format!("Received {} bytes: {}", n, hex));
            if let Some(info) = parse_handshake_response(&response_buf[..n]) {
                let chip = ChipInfo::from_handshake(
                    info.family_code, info.header_version,
                    info.mcu_id, info.flash_size_kb, info.eeprom_size_kb,
                );
                set_progress(100.0, "MCU detected");
                set_status(PluginStatus::Success);
                log_info(&format!("Detected: {}", chip.info_message));

                return PluginResult::success(serde_json::json!({
                    "family": chip.family.name(),
                    "family_code": format!("0x{:02X}", chip.family_code),
                    "flash_size": chip.flash_size,
                    "eeprom_size": chip.eeprom_size,
                    "info": chip.info_message,
                }));
            }
        }

        std::thread::sleep(send_interval);
    }

    set_status(PluginStatus::Error);
    PluginResult::error("No response after 15 seconds. Ensure: 1) MCU is STC series, 2) Power cycle the MCU during detection, 3) Correct serial port selected.")
}

fn handle_flash(params: &serde_json::Value) -> PluginResult {
    let firmware_path = match params["firmware_path"].as_str() {
        Some(p) => p,
        None => return PluginResult::error("Missing firmware_path parameter"),
    };
    let baud_rate = params["baud_rate"].as_u64().unwrap_or(115200) as u32;

    log_info(&format!("Flashing {} at {} baud", firmware_path, baud_rate));
    set_status(PluginStatus::Running);

    // Read firmware file
    set_progress(5.0, "Reading firmware file...");
    let firmware = if firmware_path.to_lowercase().ends_with(".hex") {
        let content = match std::fs::read_to_string(firmware_path) {
            Ok(c) => c,
            Err(e) => return PluginResult::error(format!("Failed to read HEX file: {}", e)),
        };
        match parse_hex_file(&content) {
            Ok(f) => f,
            Err(e) => return PluginResult::error(format!("HEX parse error: {}", e)),
        }
    } else {
        match std::fs::read(firmware_path) {
            Ok(f) => f,
            Err(e) => return PluginResult::error(format!("Failed to read binary: {}", e)),
        }
    };

    if firmware.is_empty() {
        return PluginResult::error("Firmware file is empty");
    }

    log_info(&format!("Firmware size: {} bytes", firmware.len()));
    set_progress(10.0, &format!("Firmware: {} bytes", firmware.len()));

    // Detect MCU
    set_progress(15.0, "Detecting MCU...");
    let trigger = isp_trigger_packet();
    if !serial_write(&trigger) {
        set_status(PluginStatus::Error);
        return PluginResult::error("Failed to send ISP trigger");
    }

    let mut buf = [0u8; 64];
    let chip = match serial_read(&mut buf, 2000) {
        Some(n) => {
            if let Some(info) = parse_handshake_response(&buf[..n]) {
                ChipInfo::from_handshake(
                    info.family_code, info.header_version,
                    info.mcu_id, info.flash_size_kb, info.eeprom_size_kb,
                )
            } else {
                set_status(PluginStatus::Error);
                return PluginResult::error("Invalid handshake. Power cycle MCU while sending ISP trigger.");
            }
        }
        None => {
            set_status(PluginStatus::Error);
            return PluginResult::error("No response. Power cycle MCU while sending ISP trigger.");
        }
    };

    log_info(&format!("Detected: {}", chip.info_message));
    set_progress(20.0, &chip.info_message);

    if is_cancelled() {
        set_status(PluginStatus::Idle);
        return PluginResult::error("Cancelled by user");
    }

    // Erase flash
    set_progress(25.0, "Erasing flash...");
    let erase_end = std::cmp::min(
        (firmware.len() as u32 + 0xFF) & !0xFF,
        chip.flash_size,
    );
    let erase_pkt = erase_packet(0, erase_end);
    if !serial_write(&erase_pkt) {
        set_status(PluginStatus::Error);
        return PluginResult::error("Failed to send erase command");
    }

    match serial_read(&mut buf, 5000) {
        Some(n) if n > 0 && is_ack(&buf[..n]) => {
            log_info("Flash erased successfully");
        }
        _ => {
            set_status(PluginStatus::Error);
            return PluginResult::error("Erase failed or timed out");
        }
    }

    set_progress(40.0, "Writing firmware...");

    // Write firmware in 128-byte blocks
    let block_size = 128;
    let total_blocks = (firmware.len() + block_size - 1) / block_size;

    for (i, chunk) in firmware.chunks(block_size).enumerate() {
        if is_cancelled() {
            set_status(PluginStatus::Idle);
            return PluginResult::error("Cancelled by user");
        }

        let addr = (i * block_size) as u32;
        let write_pkt = write_packet(addr, chunk);

        if !serial_write(&write_pkt) {
            set_status(PluginStatus::Error);
            return PluginResult::error(format!("Failed to write block {}/{}", i + 1, total_blocks));
        }

        match serial_read(&mut buf, 2000) {
            Some(n) if n > 0 && is_ack(&buf[..n]) => {
                let percent = 40.0 + (i as f32 / total_blocks as f32) * 40.0;
                set_progress(percent, &format!("Writing block {}/{}", i + 1, total_blocks));
            }
            _ => {
                set_status(PluginStatus::Error);
                return PluginResult::error(format!("Write failed at block {}/{}", i + 1, total_blocks));
            }
        }
    }

    set_progress(80.0, "Verifying...");

    // Calculate CRC for verification
    let crc = stc_crc16(&firmware);
    let verify_pkt = verify_packet(0, firmware.len() as u32, crc);
    if !serial_write(&verify_pkt) {
        set_status(PluginStatus::Error);
        return PluginResult::error("Failed to send verify command");
    }

    match serial_read(&mut buf, 5000) {
        Some(n) if n > 0 && is_ack(&buf[..n]) => {
            log_info("Verification passed");
        }
        _ => {
            set_status(PluginStatus::Error);
            return PluginResult::error("Verification failed");
        }
    }

    set_progress(90.0, "Resetting MCU...");

    // Reset MCU
    let reset_pkt = reset_packet();
    let _ = serial_write(&reset_pkt);

    set_progress(100.0, "Flash complete!");
    set_status(PluginStatus::Success);
    log_info("Firmware flash completed successfully");

    PluginResult::success(serde_json::json!({
        "status": "success",
        "firmware_size": firmware.len(),
        "chip": chip.info_message,
        "blocks_written": total_blocks,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_info() {
        let ptr = plugin_get_info();
        assert!(!ptr.is_null());
        let info_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let info: PluginInfo = serde_json::from_str(&info_str).unwrap();
        assert_eq!(info.name, "STC ISP Flasher");
        plugin_free_string(ptr);
    }

    #[test]
    fn test_plugin_commands() {
        let ptr = plugin_get_commands();
        assert!(!ptr.is_null());
        let cmds_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let cmds: Vec<PluginCommand> = serde_json::from_str(&cmds_str).unwrap();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].name, "flash");
        assert_eq!(cmds[1].name, "detect");
        plugin_free_string(ptr);
    }

    #[test]
    fn test_plugin_capabilities() {
        let ptr = plugin_get_capabilities();
        assert!(!ptr.is_null());
        let caps_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let caps: Vec<PluginCapability> = serde_json::from_str(&caps_str).unwrap();
        assert!(caps.contains(&PluginCapability::SerialPort));
        assert!(caps.contains(&PluginCapability::Progress));
        assert!(caps.contains(&PluginCapability::Logging));
        plugin_free_string(ptr);
    }

    #[test]
    fn test_plugin_execute_unknown() {
        let cmd = CString::new("unknown").unwrap();
        let params = CString::new("{}").unwrap();
        let ptr = plugin_execute(cmd.as_ptr(), params.as_ptr());
        assert!(!ptr.is_null());
        let result: PluginResult = serde_json::from_str(
            &unsafe { CStr::from_ptr(ptr).to_string_lossy() }
        ).unwrap();
        assert!(!result.success);
        plugin_free_string(ptr);
    }
}
