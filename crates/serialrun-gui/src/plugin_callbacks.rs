/// Host callback adapter for plugins.
/// Implements the PluginCallbacks struct that plugins can use to access
/// serial port, file dialogs, progress, and logging.

use crate::state::AppState;
use crate::port_owner::PortCommand;
use serialrun_plugin_api::{PluginCallbacks, PluginStatus};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int};
use std::sync::{Arc, Mutex, OnceLock, atomic::{AtomicBool, Ordering}, mpsc};

/// Create a PluginCallbacks struct from the current AppState.
/// The callbacks are safe to call from any thread.
pub fn create_callbacks() -> PluginCallbacks {
    PluginCallbacks {
        // Serial port
        serial_read: Some(callback_serial_read),
        serial_write: Some(callback_serial_write),
        serial_set_baud: Some(callback_serial_set_baud),
        serial_is_connected: Some(callback_serial_is_connected),

        // Progress
        progress_set: Some(callback_progress_set),
        progress_set_status: Some(callback_progress_set_status),
        progress_is_cancelled: Some(callback_progress_is_cancelled),

        // File operations
        file_open_dialog: Some(callback_file_open_dialog),
        file_save_dialog: Some(callback_file_save_dialog),
        file_read: Some(callback_file_read),
        free_string: Some(callback_free_string),

        // Logging
        log_info: Some(callback_log_info),
        log_warn: Some(callback_log_warn),
        log_error: Some(callback_log_error),

        // File system (device-side) - stub implementations for now
        fs_list_dir: Some(callback_fs_list_dir),
        fs_read_file: Some(callback_fs_read_file),
        fs_write_file: Some(callback_fs_write_file),
        fs_delete_file: Some(callback_fs_delete_file),
        fs_mkdir: Some(callback_fs_mkdir),
        fs_exists: Some(callback_fs_exists),

        // Event system
        on_serial_data: Some(callback_on_serial_data),
        on_connection_changed: Some(callback_on_connection_changed),

        // Config storage
        config_get: Some(callback_config_get),
        config_set: Some(callback_config_set),

        // Async execution
        execute_async: Some(callback_execute_async),
    }
}

// ============================================================================
// Serial Port Callbacks
// ============================================================================

static PLUGIN_STATE: OnceLock<Arc<Mutex<AppState>>> = OnceLock::new();
static PLUGIN_CANCELLED: OnceLock<Arc<AtomicBool>> = OnceLock::new();

/// Set the global cancellation token for plugin progress operations.
pub fn set_cancellation_token(token: Arc<AtomicBool>) {
    let _ = PLUGIN_CANCELLED.set(token);
}

/// Check if a plugin operation has been cancelled.
pub fn is_cancelled() -> bool {
    PLUGIN_CANCELLED.get().map_or(false, |t| t.load(Ordering::Relaxed))
}

/// Reset the cancellation token (call before starting a new operation).
pub fn reset_cancellation() {
    if let Some(token) = PLUGIN_CANCELLED.get() {
        token.store(false, Ordering::Relaxed);
    }
}

/// Set the global state reference for plugin callbacks.
pub fn set_plugin_state(state: Arc<Mutex<AppState>>) {
    let _ = PLUGIN_STATE.set(state);
}

/// Get the global state reference.
fn get_state() -> Option<&'static Arc<Mutex<AppState>>> {
    PLUGIN_STATE.get()
}

extern "C" fn callback_serial_read(buf: *mut u8, len: u32, timeout_ms: u32) -> c_int {
    let Some(state) = get_state() else { return -1 };
    // Extract cmd_tx under lock, then release lock before blocking on recv()
    let cmd_tx = {
        let Ok(state) = state.lock() else { return -1 };
        let Some(po) = state.port_owner.as_ref() else { return -1 };
        po.cmd_tx()
    };

    let (resp_tx, resp_rx) = mpsc::channel();
    let _ = cmd_tx.send(PortCommand::ReadExclusive {
        data: Vec::new(),
        timeout_ms: timeout_ms as u64,
        resp_tx,
    });

    // Block on recv without holding the state lock
    match resp_rx.recv() {
        Ok(Ok(data)) => {
            let copy_len = std::cmp::min(data.len(), len as usize);
            unsafe {
                std::ptr::copy_nonoverlapping(data.as_ptr(), buf, copy_len);
            }
            copy_len as c_int
        }
        _ => -1,
    }
}

extern "C" fn callback_serial_write(data: *const u8, len: u32) -> c_int {
    let Some(state) = get_state() else { return -1 };
    let cmd_tx = {
        let Ok(state) = state.lock() else { return -1 };
        let Some(po) = state.port_owner.as_ref() else { return -1 };
        po.cmd_tx()
    };

    let slice = unsafe { std::slice::from_raw_parts(data, len as usize) };
    let bytes = slice.to_vec();

    let _ = cmd_tx.send(PortCommand::Write(bytes));
    len as c_int
}

extern "C" fn callback_serial_set_baud(baud: u32) -> bool {
    let Some(state) = get_state() else { return false };
    let Ok(state) = state.lock() else { return false };
    if let Some(po) = state.port_owner.as_ref() {
        let _ = po.cmd_tx().send(PortCommand::ChangeBaud(baud));
        true
    } else {
        false
    }
}

extern "C" fn callback_serial_is_connected() -> bool {
    let Some(state) = get_state() else { return false };
    let Ok(state) = state.lock() else { return false };
    state.is_connected
}

// ============================================================================
// Progress Callbacks
// ============================================================================

extern "C" fn callback_progress_set(percent: c_float, message: *const c_char) {
    if message.is_null() {
        return;
    }
    let msg = unsafe { CStr::from_ptr(message).to_string_lossy() };
    log::info!("[Plugin Progress] {:.0}%: {}", percent, msg);
    // Update STC panel progress
    if let Some(state) = get_state() {
        if let Ok(mut state) = state.lock() {
            state.stc_flash_progress = percent;
            state.stc_flash_status = msg.to_string();
        }
    }
}

extern "C" fn callback_progress_set_status(status: PluginStatus) {
    log::info!("[Plugin Status] {:?}", status);
}

extern "C" fn callback_progress_is_cancelled() -> bool {
    is_cancelled()
}

// ============================================================================
// File Dialog Callbacks
// ============================================================================

extern "C" fn callback_file_open_dialog(filter: *const c_char) -> *mut c_char {
    let mut dialog = rfd::FileDialog::new()
        .set_title("Open File");

    // BUG 12 FIX: Parse and apply filter parameter
    if !filter.is_null() {
        let filter_str = unsafe { CStr::from_ptr(filter).to_string_lossy() };
        let parts: Vec<&str> = filter_str.split('|').collect();
        if parts.len() == 2 {
            let name = parts[0].trim();
            let exts: Vec<&str> = parts[1].split(',').map(|s| s.trim()).collect();
            if !name.is_empty() && !exts.is_empty() {
                dialog = dialog.add_filter(name, &exts);
            }
        }
    }
    dialog = dialog.add_filter("All Files", &["*"]);

    let result = dialog.pick_file();

    match result {
        Some(path) => CString::new(path.to_string_lossy().to_string())
            .unwrap_or_default()
            .into_raw(),
        None => std::ptr::null_mut(),
    }
}

extern "C" fn callback_file_save_dialog(filter: *const c_char) -> *mut c_char {
    let mut dialog = rfd::FileDialog::new()
        .set_title("Save File");

    // BUG 12 FIX: Parse and apply filter parameter
    if !filter.is_null() {
        let filter_str = unsafe { CStr::from_ptr(filter).to_string_lossy() };
        let parts: Vec<&str> = filter_str.split('|').collect();
        if parts.len() == 2 {
            let name = parts[0].trim();
            let exts: Vec<&str> = parts[1].split(',').map(|s| s.trim()).collect();
            if !name.is_empty() && !exts.is_empty() {
                dialog = dialog.add_filter(name, &exts);
            }
        }
    }
    dialog = dialog.add_filter("All Files", &["*"]);

    let result = dialog.save_file();

    match result {
        Some(path) => CString::new(path.to_string_lossy().to_string())
            .unwrap_or_default()
            .into_raw(),
        None => std::ptr::null_mut(),
    }
}

extern "C" fn callback_file_read(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let path_str = unsafe { CStr::from_ptr(path).to_string_lossy() };

    match std::fs::read(path_str.as_ref()) {
        Ok(data) => {
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
            CString::new(encoded).unwrap_or_default().into_raw()
        }
        Err(_) => std::ptr::null_mut(),
    }
}

extern "C" fn callback_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

// ============================================================================
// Logging Callbacks
// ============================================================================

extern "C" fn callback_log_info(msg: *const c_char) {
    if msg.is_null() {
        return;
    }
    let text = unsafe { CStr::from_ptr(msg).to_string_lossy() };
    log::info!("[Plugin] {}", text);
}

extern "C" fn callback_log_warn(msg: *const c_char) {
    if msg.is_null() {
        return;
    }
    let text = unsafe { CStr::from_ptr(msg).to_string_lossy() };
    log::warn!("[Plugin] {}", text);
}

extern "C" fn callback_log_error(msg: *const c_char) {
    if msg.is_null() {
        return;
    }
    let text = unsafe { CStr::from_ptr(msg).to_string_lossy() };
    log::error!("[Plugin] {}", text);
}

// ============================================================================
// File System Callbacks (device-side, for MicroPython-style devices)
// ============================================================================

extern "C" fn callback_fs_list_dir(path: *const c_char) -> *mut c_char {
    // For now, return an empty JSON array.
    // Real implementation will go through the serial port using Raw REPL protocol.
    let path_str = if path.is_null() {
        "/".to_string()
    } else {
        unsafe { CStr::from_ptr(path).to_string_lossy().to_string() }
    };
    log::info!("[Plugin FS] list_dir: {}", path_str);
    CString::new("[]").unwrap_or_default().into_raw()
}

extern "C" fn callback_fs_read_file(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let path_str = unsafe { CStr::from_ptr(path).to_string_lossy() };
    log::info!("[Plugin FS] read_file: {}", path_str);
    // Placeholder - real implementation reads via serial port
    std::ptr::null_mut()
}

extern "C" fn callback_fs_write_file(path: *const c_char, data: *const c_char) -> bool {
    if path.is_null() || data.is_null() {
        return false;
    }
    let path_str = unsafe { CStr::from_ptr(path).to_string_lossy() };
    log::info!("[Plugin FS] write_file: {}", path_str);
    // Placeholder - real implementation writes via serial port
    false
}

extern "C" fn callback_fs_delete_file(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let path_str = unsafe { CStr::from_ptr(path).to_string_lossy() };
    log::info!("[Plugin FS] delete_file: {}", path_str);
    false
}

extern "C" fn callback_fs_mkdir(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let path_str = unsafe { CStr::from_ptr(path).to_string_lossy() };
    log::info!("[Plugin FS] mkdir: {}", path_str);
    false
}

extern "C" fn callback_fs_exists(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let path_str = unsafe { CStr::from_ptr(path).to_string_lossy() };
    log::info!("[Plugin FS] exists: {}", path_str);
    false
}

// ============================================================================
// Event System Callbacks
// ============================================================================

/// Storage for registered event callbacks
static SERIAL_DATA_CALLBACK: OnceLock<Arc<Mutex<Option<extern "C" fn(*const u8, u32)>>>> = OnceLock::new();
static CONNECTION_CALLBACK: OnceLock<Arc<Mutex<Option<extern "C" fn(bool)>>>> = OnceLock::new();

extern "C" fn callback_on_serial_data(data_callback: extern "C" fn(*const u8, u32)) {
    let cb_store = SERIAL_DATA_CALLBACK.get_or_init(|| Arc::new(Mutex::new(None)));
    if let Ok(mut cb) = cb_store.lock() {
        *cb = Some(data_callback);
        log::info!("[Plugin] Serial data callback registered");
    }
}

extern "C" fn callback_on_connection_changed(conn_callback: extern "C" fn(bool)) {
    let cb_store = CONNECTION_CALLBACK.get_or_init(|| Arc::new(Mutex::new(None)));
    if let Ok(mut cb) = cb_store.lock() {
        *cb = Some(conn_callback);
        log::info!("[Plugin] Connection changed callback registered");
    }
}

/// Notify all registered plugins about received serial data.
pub fn notify_serial_data(data: &[u8]) {
    if let Some(cb_store) = SERIAL_DATA_CALLBACK.get() {
        if let Ok(cb) = cb_store.lock() {
            if let Some(callback) = *cb {
                let ptr = data.as_ptr();
                let len = data.len() as u32;
                callback(ptr, len);
            }
        }
    }
}

/// Notify all registered plugins about connection state changes.
pub fn notify_connection_changed(connected: bool) {
    if let Some(cb_store) = CONNECTION_CALLBACK.get() {
        if let Ok(cb) = cb_store.lock() {
            if let Some(callback) = *cb {
                callback(connected);
            }
        }
    }
}

// ============================================================================
// Config Storage Callbacks
// ============================================================================

extern "C" fn callback_config_get(key: *const c_char) -> *mut c_char {
    if key.is_null() {
        return std::ptr::null_mut();
    }
    let key_str = unsafe { CStr::from_ptr(key).to_string_lossy() };
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let config_path = std::path::PathBuf::from(home)
        .join(".serialrun").join("plugin_config.json");

    if let Ok(data) = std::fs::read_to_string(&config_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
            if let Some(val) = json.get(key_str.as_ref()) {
                let result = serde_json::to_string(val).unwrap_or_default();
                return CString::new(result).unwrap_or_default().into_raw();
            }
        }
    }
    std::ptr::null_mut()
}

extern "C" fn callback_config_set(key: *const c_char, value: *const c_char) -> bool {
    if key.is_null() || value.is_null() {
        return false;
    }
    let key_str = unsafe { CStr::from_ptr(key).to_string_lossy() };
    let value_str = unsafe { CStr::from_ptr(value).to_string_lossy() };

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let config_dir = std::path::PathBuf::from(home).join(".serialrun");
    let config_path = config_dir.join("plugin_config.json");

    // Read existing config
    let mut json: serde_json::Value = if let Ok(data) = std::fs::read_to_string(&config_path) {
        serde_json::from_str(&data).unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
    } else {
        serde_json::Value::Object(serde_json::Map::new())
    };

    // Parse and set value
    let parsed_value: serde_json::Value = serde_json::from_str(&value_str)
        .unwrap_or(serde_json::Value::String(value_str.to_string()));

    if let Some(obj) = json.as_object_mut() {
        obj.insert(key_str.to_string(), parsed_value);
    }

    // Write back
    if let Ok(data) = serde_json::to_string_pretty(&json) {
        let _ = std::fs::create_dir_all(&config_dir);
        std::fs::write(&config_path, data).is_ok()
    } else {
        false
    }
}

// ============================================================================
// Async Execution Callbacks
// ============================================================================

extern "C" fn callback_execute_async(
    command: *const c_char,
    params: *const c_char,
    callback: extern "C" fn(*const c_char),
) {
    if command.is_null() {
        return;
    }
    let cmd_str = unsafe { CStr::from_ptr(command).to_string_lossy() };
    let params_str = if params.is_null() {
        "{}".to_string()
    } else {
        unsafe { CStr::from_ptr(params).to_string_lossy().to_string() }
    };

    log::info!("[Plugin] Async execute: {} with {}", cmd_str, params_str);

    // Spawn a thread for async execution
    std::thread::spawn(move || {
        // Placeholder - real implementation would execute the command
        // and call the callback with the result
        let result = serde_json::json!({
            "success": true,
            "result": format!("Async command '{}' executed", cmd_str)
        });
        let result_str = serde_json::to_string(&result).unwrap_or_default();
        let c_result = CString::new(result_str).unwrap_or_default();
        callback(c_result.as_ptr());
    });
}
