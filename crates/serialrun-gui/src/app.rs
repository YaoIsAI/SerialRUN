use crate::state::{AppState, Language, Theme, T, UserPrefs};
use crate::ui;
use eframe::egui;
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;

/// Global loaded plugins storage (separate from AppState to avoid deadlocks).
/// Plugin callbacks lock PLUGIN_STATE (AppState), so plugins must NOT be inside AppState.
pub static LOADED_PLUGINS: OnceLock<Arc<Mutex<HashMap<String, serialrun_core::plugin::LoadedPlugin>>>> = OnceLock::new();

pub fn get_loaded_plugins() -> &'static Arc<Mutex<HashMap<String, serialrun_core::plugin::LoadedPlugin>>> {
    LOADED_PLUGINS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

fn config_path() -> std::path::PathBuf {
    if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        std::path::PathBuf::from(home).join(".serialrun").join("config.toml")
    } else {
        std::path::PathBuf::from(".serialrun").join("config.toml")
    }
}

fn load_prefs() -> UserPrefs {
    let path = config_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| toml::from_str(&c).ok())
        .unwrap_or_default()
}

pub fn save_prefs(prefs: &UserPrefs) {
    let path = config_path();
    if let Ok(content) = toml::to_string_pretty(prefs) {
        let _ = std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")));
        let _ = std::fs::write(&path, content);
    }
}

pub struct SerialRunApp {
    state: Arc<Mutex<AppState>>,
    current_theme: Theme,
    mcp_handle: crate::mcp_server::McpHandle,
    last_prefs: crate::state::UserPrefs,
    /// Tracks which viewports are currently open (OS-level windows)
    open_viewports: std::collections::HashSet<egui::ViewportId>,
}

impl SerialRunApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, mcp_handle: crate::mcp_server::McpHandle) -> Self {
        let prefs = load_prefs();
        let mut state = AppState::new();
        prefs.apply_to(&mut state);
        state.refresh_ports();
        crate::ui::plugin::discover_plugins(&mut state);

        let last_prefs = load_prefs();
        let (mcp_serial_tx, mcp_serial_rx) = std::sync::mpsc::channel();
        mcp_handle.send(crate::mcp_server::McpCommand::SetSerialRequestTx(Some(mcp_serial_tx)));
        let state_clone = Arc::new(Mutex::new(state));
        crate::plugin_callbacks::set_plugin_state(state_clone.clone());
        let app_state = state_clone.clone();
        std::thread::spawn(move || {
            mcp_serial_request_loop(mcp_serial_rx, app_state);
        });
        Self { state: state_clone, current_theme: Theme::Light, mcp_handle, last_prefs, open_viewports: std::collections::HashSet::new() }
    }
}

fn sync_theme_visuals(ctx: &egui::Context, theme: Theme, current_theme: &mut Theme) {
    if *current_theme != theme {
        let mut visuals = match theme {
            Theme::Dark => egui::Visuals::dark(),
            Theme::Light => {
                let mut v = egui::Visuals::light();
                v.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(230, 230, 235);
                v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(30, 30, 30));
                v.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(200, 200, 210);
                v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 0, 0));
                v.widgets.active.weak_bg_fill = egui::Color32::from_rgb(170, 170, 185);
                v.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 0, 0));
                v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 50));
                v
            }
        };
        visuals.window_rounding = egui::Rounding::same(8.0);
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(6.0);
        visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
        visuals.widgets.active.rounding = egui::Rounding::same(6.0);
        ctx.set_visuals(visuals);
        *current_theme = theme;
    }
}

fn poll_mcp_status(mcp_handle: &crate::mcp_server::McpHandle, state: &mut AppState) {
    while let Some(status) = mcp_handle.poll_status() {
        match status {
            crate::mcp_server::McpStatus::Running { addr } => {
                state.mcp_running = true;
                state.mcp_status = addr;
            }
            crate::mcp_server::McpStatus::Stopped => {
                state.mcp_running = false;
                state.mcp_status.clear();
            }
            crate::mcp_server::McpStatus::Error(e) => {
                state.mcp_running = false;
                state.mcp_status = format!("Error: {}", e);
            }
        }
    }
    if state.mcp_cmd_tx.is_none() {
        state.mcp_cmd_tx = Some(mcp_handle.cmd_tx());
    }
}

fn poll_mcp_log(mcp_handle: &crate::mcp_server::McpHandle, state: &mut AppState) {
    let mut changed = false;
    while let Some(entry) = mcp_handle.poll_log() {
        state.mcp_access_log.push_back(entry);
        if state.mcp_access_log.len() > 1000 {
            state.mcp_access_log.pop_front();
        }
        changed = true;
    }
    if changed {
        state.save_mcp_log();
    }
}

/// Check if bytes are printable text (not binary/hex data)
/// Returns true if valid UTF-8 with no disallowed control characters
fn is_printable_text(bytes: &[u8]) -> bool {
    if let Ok(s) = std::str::from_utf8(bytes) {
        s.chars().all(|c| !c.is_control() || c == '\t' || c == '\n' || c == '\r')
    } else {
        false
    }
}

/// Build a config map from AppState for MCP get_config
fn build_config_map(state: &crate::state::AppState) -> std::collections::HashMap<String, serde_json::Value> {
    let mut m = std::collections::HashMap::new();
    m.insert("baud_rate".into(), serde_json::json!(state.config.baud_rate));
    m.insert("data_bits".into(), serde_json::json!(match state.config.data_bits {
        serialrun_core::config::DataBits::Five => 5,
        serialrun_core::config::DataBits::Six => 6,
        serialrun_core::config::DataBits::Seven => 7,
        serialrun_core::config::DataBits::Eight => 8,
    }));
    m.insert("stop_bits".into(), serde_json::json!(match state.config.stop_bits {
        serialrun_core::config::StopBits::One => 1,
        serialrun_core::config::StopBits::Two => 2,
    }));
    m.insert("parity".into(), serde_json::json!(match state.config.parity {
        serialrun_core::config::Parity::None => "None",
        serialrun_core::config::Parity::Odd => "Odd",
        serialrun_core::config::Parity::Even => "Even",
    }));
    m.insert("flow_control".into(), serde_json::json!(match state.config.flow_control {
        serialrun_core::config::FlowControl::None => "None",
        serialrun_core::config::FlowControl::Software => "Software",
        serialrun_core::config::FlowControl::Hardware => "Hardware",
    }));
    m.insert("hex_mode".into(), serde_json::json!(state.hex_mode));
    m.insert("show_timestamp".into(), serde_json::json!(state.show_timestamp));
    m.insert("auto_scroll".into(), serde_json::json!(state.auto_scroll));
    m.insert("terminal_checksum_mode".into(), serde_json::json!(state.terminal_checksum_mode.label(state.language)));
    m.insert("auto_send_enabled".into(), serde_json::json!(state.auto_send_enabled));
    m.insert("auto_send_interval_ms".into(), serde_json::json!(state.auto_send_interval_ms));
    m.insert("keep_input".into(), serde_json::json!(state.keep_input));
    m.insert("line_ending".into(), serde_json::json!(match state.line_ending {
        crate::state::LineEnding::None => "None",
        crate::state::LineEnding::CR => "CR",
        crate::state::LineEnding::LF => "LF",
        crate::state::LineEnding::CRLF => "CRLF",
    }));
    m.insert("dtr".into(), serde_json::json!(state.dtr));
    m.insert("rts".into(), serde_json::json!(state.rts));
    m.insert("auto_reply_enabled".into(), serde_json::json!(state.auto_reply_enabled));
    m.insert("auto_reply_pattern".into(), serde_json::json!(state.auto_reply_pattern));
    m.insert("auto_reply_response".into(), serde_json::json!(state.auto_reply_response));
    m.insert("rx_auto_aggregate".into(), serde_json::json!(state.rx_auto_aggregate));
    m.insert("rx_aggregate_ms".into(), serde_json::json!(state.rx_aggregate_ms));
    m
}

/// Apply a config change from MCP set_config
fn apply_config(state: &mut crate::state::AppState, key: &str, value: &serde_json::Value) -> Result<String, String> {
    match key {
        "baud_rate" => {
            state.config.baud_rate = value.as_u64().ok_or("value must be integer")? as u32;
            state.baud_rate_text = state.config.baud_rate.to_string();
            Ok(format!("baud_rate = {}", state.config.baud_rate))
        }
        "data_bits" => {
            let v = value.as_u64().ok_or("value must be integer")? as u8;
            state.config.data_bits = match v {
                5 => serialrun_core::config::DataBits::Five,
                6 => serialrun_core::config::DataBits::Six,
                7 => serialrun_core::config::DataBits::Seven,
                _ => serialrun_core::config::DataBits::Eight,
            };
            Ok(format!("data_bits = {}", v))
        }
        "stop_bits" => {
            let v = value.as_u64().ok_or("value must be integer")? as u8;
            state.config.stop_bits = match v {
                2 => serialrun_core::config::StopBits::Two,
                _ => serialrun_core::config::StopBits::One,
            };
            Ok(format!("stop_bits = {}", v))
        }
        "parity" => {
            let v = value.as_str().ok_or("value must be string")?;
            state.config.parity = match v.to_lowercase().as_str() {
                "odd" => serialrun_core::config::Parity::Odd,
                "even" => serialrun_core::config::Parity::Even,
                _ => serialrun_core::config::Parity::None,
            };
            Ok(format!("parity = {}", v))
        }
        "flow_control" => {
            let v = value.as_str().ok_or("value must be string")?;
            state.config.flow_control = match v.to_lowercase().as_str() {
                "software" => serialrun_core::config::FlowControl::Software,
                "hardware" => serialrun_core::config::FlowControl::Hardware,
                _ => serialrun_core::config::FlowControl::None,
            };
            Ok(format!("flow_control = {}", v))
        }
        "hex_mode" => {
            state.hex_mode = value.as_bool().ok_or("value must be boolean")?;
            Ok(format!("hex_mode = {}", state.hex_mode))
        }
        "show_timestamp" => {
            state.show_timestamp = value.as_bool().ok_or("value must be boolean")?;
            Ok(format!("show_timestamp = {}", state.show_timestamp))
        }
        "auto_scroll" => {
            state.auto_scroll = value.as_bool().ok_or("value must be boolean")?;
            Ok(format!("auto_scroll = {}", state.auto_scroll))
        }
        "auto_send_enabled" => {
            state.auto_send_enabled = value.as_bool().ok_or("value must be boolean")?;
            Ok(format!("auto_send_enabled = {}", state.auto_send_enabled))
        }
        "auto_send_interval_ms" => {
            state.auto_send_interval_ms = value.as_u64().ok_or("value must be integer")?;
            Ok(format!("auto_send_interval_ms = {}", state.auto_send_interval_ms))
        }
        "keep_input" => {
            state.keep_input = value.as_bool().ok_or("value must be boolean")?;
            Ok(format!("keep_input = {}", state.keep_input))
        }
        "line_ending" => {
            let v = value.as_str().ok_or("value must be string")?;
            state.line_ending = match v.to_uppercase().as_str() {
                "CR" => crate::state::LineEnding::CR,
                "LF" => crate::state::LineEnding::LF,
                "CRLF" => crate::state::LineEnding::CRLF,
                _ => crate::state::LineEnding::None,
            };
            Ok(format!("line_ending = {}", v))
        }
        "dtr" => {
            state.dtr = value.as_bool().ok_or("value must be boolean")?;
            if let Some(ref po) = state.port_owner {
                po.send(crate::port_owner::PortCommand::SetDtr(state.dtr));
            }
            Ok(format!("dtr = {}", state.dtr))
        }
        "rts" => {
            state.rts = value.as_bool().ok_or("value must be boolean")?;
            if let Some(ref po) = state.port_owner {
                po.send(crate::port_owner::PortCommand::SetRts(state.rts));
            }
            Ok(format!("rts = {}", state.rts))
        }
        "auto_reply_enabled" => {
            state.auto_reply_enabled = value.as_bool().ok_or("value must be boolean")?;
            Ok(format!("auto_reply_enabled = {}", state.auto_reply_enabled))
        }
        "auto_reply_pattern" => {
            state.auto_reply_pattern = value.as_str().ok_or("value must be string")?.to_string();
            Ok(format!("auto_reply_pattern = {}", state.auto_reply_pattern))
        }
        "auto_reply_response" => {
            state.auto_reply_response = value.as_str().ok_or("value must be string")?.to_string();
            Ok(format!("auto_reply_response = {}", state.auto_reply_response))
        }
        "rx_auto_aggregate" => {
            state.rx_auto_aggregate = value.as_bool().ok_or("value must be boolean")?;
            Ok(format!("rx_auto_aggregate = {}", state.rx_auto_aggregate))
        }
        "rx_aggregate_ms" => {
            state.rx_aggregate_ms = value.as_u64().ok_or("value must be integer")?;
            // Sync accumulation timeout with port_owner
            if let Some(ref po) = state.port_owner {
                po.sync_timeout(state.rx_aggregate_ms);
            }
            Ok(format!("rx_aggregate_ms = {}", state.rx_aggregate_ms))
        }
        _ => Err(format!("Unknown key: {}", key)),
    }
}

/// Lock the AppState mutex, recovering from poisoning if needed.
fn lock_state(state: &Arc<Mutex<crate::state::AppState>>) -> std::sync::MutexGuard<'_, crate::state::AppState> {
    state.lock().unwrap_or_else(|e| e.into_inner())
}

/// Dedicated thread that processes MCP serial requests independently of the GUI event loop.
/// Each handler acquires/releases the lock independently to avoid blocking the GUI during I/O.
fn mcp_serial_request_loop(
    rx: std::sync::mpsc::Receiver<crate::mcp_server::McpSerialRequest>,
    app_state: Arc<Mutex<crate::state::AppState>>,
) {
    eprintln!("[MCP-Serial] Request processing thread started");
    while let Ok(req) = rx.recv() {
        match req {
            crate::mcp_server::McpSerialRequest::Connect { port_name, baud_rate, data_bits, stop_bits, parity, flow_control, resp } => {
                // Phase 1: Prepare under lock
                let config = {
                    let mut state = lock_state(&app_state);
                    state.mcp_connect_in_progress = true;
                    if state.is_connected {
                        if let Some(po) = state.port_owner.take() {
                            drop(state);
                            drop(po);
                            state = lock_state(&app_state);
                        }
                        state.is_connected = false;
                        state.selected_port = None;
                    }
                    let db = match data_bits { 5=>serialrun_core::config::DataBits::Five, 6=>serialrun_core::config::DataBits::Six, 7=>serialrun_core::config::DataBits::Seven, _=>serialrun_core::config::DataBits::Eight };
                    let sb = match stop_bits { 2=>serialrun_core::config::StopBits::Two, _=>serialrun_core::config::StopBits::One };
                    let pr = match parity.as_str() { "Odd"|"odd"=>serialrun_core::config::Parity::Odd, "Even"|"even"=>serialrun_core::config::Parity::Even, _=>serialrun_core::config::Parity::None };
                    let fc = match flow_control.as_str() { "Software"|"software"=>serialrun_core::config::FlowControl::Software, "Hardware"|"hardware"=>serialrun_core::config::FlowControl::Hardware, _=>serialrun_core::config::FlowControl::None };
                    serialrun_core::config::SerialConfig {
                        port_name: port_name.clone(), baud_rate, data_bits: db, stop_bits: sb, parity: pr, flow_control: fc, timeout_ms: 1000,
                    }
                };
                // Phase 2: Connect without lock (blocking I/O)
                let po_handle = crate::port_owner::PortOwnerHandle::start();
                po_handle.send(crate::port_owner::PortCommand::Open(config));
                let mut connected = false;
                for _ in 0..50 {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    if let Some(evt) = po_handle.poll() {
                        if let crate::port_owner::PortEvent::Opened(ok, msg) = evt {
                            if ok { connected = true; eprintln!("[MCP-Serial] Connected to {}", msg); }
                            else { eprintln!("[MCP-Serial] Connect failed: {}", msg); }
                        }
                        if connected { break; }
                    }
                }
                // Phase 3: Update state under lock
                {
                    let mut state = lock_state(&app_state);
                    if connected {
                        state.selected_port = Some(port_name.clone());
                        state.config.baud_rate = baud_rate;
                        state.baud_rate_text = baud_rate.to_string();
                        state.is_connected = true;
                        state.port_owner = Some(po_handle);
                        // Sync accumulation timeout with port_owner
                        if let Some(ref po) = state.port_owner {
                            po.sync_timeout(state.rx_aggregate_ms);
                        }
                        state.ai_connected = true;
                        state.ai_port_name = port_name.clone();
                        state.ai_baud_rate = baud_rate;
                        state.connected_by = "MCP".to_string();
                        state.mcp_connect_in_progress = false;
                        let _ = resp.send(Ok(format!("Connected to {} at {} baud", port_name, baud_rate)));
                    } else {
                        drop(po_handle);
                        state.mcp_connect_in_progress = false;
                        let _ = resp.send(Err(format!("Failed to connect to {}", port_name)));
                    }
                }
            }
            crate::mcp_server::McpSerialRequest::Disconnect { resp } => {
                let po = {
                    let mut state = lock_state(&app_state);
                    state.mcp_connect_in_progress = true;
                    state.port_owner.take()
                };
                if let Some(po) = po {
                    drop(po); // Drop without lock held
                    let mut state = lock_state(&app_state);
                    state.is_connected = false;
                    state.selected_port = None;
                    state.ai_connected = false;
                    state.ai_port_name.clear();
                    state.ai_baud_rate = 0;
                    state.connected_by.clear();
                    state.mcp_connect_in_progress = false;
                    let _ = resp.send(Ok("Disconnected".into()));
                } else {
                    let _ = resp.send(Err("Not connected".into()));
                }
            }
            crate::mcp_server::McpSerialRequest::Send { data, pause_after, resp } => {
                let po_tx = {
                    let state = lock_state(&app_state);
                    state.port_owner.as_ref().map(|po| po.cmd_tx())
                };
                if let Some(po_tx) = po_tx {
                    let len = data.len();
                    let cmd = if pause_after {
                        crate::port_owner::PortCommand::WriteAndPause(data.clone())
                    } else {
                        crate::port_owner::PortCommand::Write(data.clone())
                    };
                    let _ = po_tx.send(cmd);
                    // Update state under lock (quick, no I/O)
                    let mut state = lock_state(&app_state);
                    let hex_preview = crate::ui::terminal::format_hex_bytes(&data);
                    let text_preview = String::from_utf8_lossy(&data).to_string();
                    state.tx_count += len as u64;
                    state.ai_tx_count += len as u64;
                    state.add_chart_data(len as f64);
                    let is_text = is_printable_text(&data);
                    let display = if is_text { text_preview.clone() } else { hex_preview.clone() };
                    state.add_terminal_line_source(crate::state::Direction::Tx, display, !is_text, "MCP");
                    state.add_log_entry(crate::state::LogLevel::Info, &format!("MCP TX {} bytes: {} | {}", len, hex_preview, text_preview));
                    let _ = resp.send(Ok(len));
                } else {
                    let _ = resp.send(Err("Not connected".into()));
                }
            }
            crate::mcp_server::McpSerialRequest::Read { timeout_ms, resume, resp } => {
                // Get MCP-specific rx_buffer Arc (brief lock), then poll directly (no state lock)
                let rx_buf = {
                    let state = lock_state(&app_state);
                    state.port_owner.as_ref().map(|po| po.mcp_rx_buffer_arc())
                };
                let rx_buf = match rx_buf {
                    Some(b) => b,
                    None => { let _ = resp.send(Ok(Vec::new())); continue; }
                };
                let deadline = std::time::Instant::now() + std::time::Duration::from_millis(std::cmp::min(timeout_ms, 3000));
                let mut data = Vec::new();
                while std::time::Instant::now() < deadline {
                    if let Ok(mut buf) = rx_buf.lock() {
                        if !buf.is_empty() {
                            data = buf.drain(..).collect();
                            break;
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                // If buffer empty and timeout allows, try direct serial read (for when continuous reader is paused)
                if data.is_empty() {
                    let po_cmd_tx = {
                        let state = lock_state(&app_state);
                        state.port_owner.as_ref().map(|po| po.cmd_tx())
                    };
                    if let Some(po_tx) = po_cmd_tx {
                        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                        if !remaining.is_zero() {
                            let (rtx, rrx) = std::sync::mpsc::channel();
                            let _ = po_tx.send(crate::port_owner::PortCommand::ReadWait { timeout_ms: remaining.as_millis() as u64, resp_tx: rtx });
                            if let Ok(Ok(d)) = rrx.recv_timeout(std::time::Duration::from_millis(remaining.as_millis() as u64 + 500)) {
                                if !d.is_empty() { data = d; }
                            }
                        }
                    }
                }
                if !data.is_empty() {
                    let mut state = lock_state(&app_state);
                    state.rx_count += data.len() as u64;
                    state.ai_rx_count += data.len() as u64;
                    state.add_chart_data(data.len() as f64);
                    // Display RX in terminal so user can see what MCP read
                    let hex_preview = crate::ui::terminal::format_hex_bytes(&data);
                    let text_preview = String::from_utf8_lossy(&data).to_string();
                    let (received, is_hex_display) = if state.hex_mode {
                        (hex_preview.clone(), true)
                    } else {
                        (text_preview.clone(), false)
                    };
                    state.add_terminal_line_source(crate::state::Direction::Rx, received, is_hex_display, "MCP");
                    state.add_log_entry(crate::state::LogLevel::Info, &format!("MCP RX {} bytes: {} | {}", data.len(), hex_preview, text_preview));
                    // Clear MCP buffer after read to prevent duplicate display
                    // (continuous reader already displays via PortEvent::Data)
                    drop(state);
                    if let Ok(mut buf) = rx_buf.lock() {
                        buf.clear();
                    }
                }
                let _ = resp.send(Ok(data));
            }
            crate::mcp_server::McpSerialRequest::ReadWait { timeout_ms, resp } => {
                let rx_buf = {
                    let state = lock_state(&app_state);
                    state.port_owner.as_ref().map(|po| po.mcp_rx_buffer_arc())
                };
                let rx_buf = match rx_buf {
                    Some(b) => b,
                    None => { let _ = resp.send(Ok(Vec::new())); continue; }
                };
                let deadline = std::time::Instant::now() + std::time::Duration::from_millis(std::cmp::min(timeout_ms, 3000));
                let mut data = Vec::new();
                while std::time::Instant::now() < deadline {
                    if let Ok(mut buf) = rx_buf.lock() {
                        if !buf.is_empty() {
                            data = buf.drain(..).collect();
                            break;
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                // If buffer empty, try direct serial read
                if data.is_empty() {
                    let po_cmd_tx = {
                        let state = lock_state(&app_state);
                        state.port_owner.as_ref().map(|po| po.cmd_tx())
                    };
                    if let Some(po_tx) = po_cmd_tx {
                        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                        if !remaining.is_zero() {
                            let (rtx, rrx) = std::sync::mpsc::channel();
                            let _ = po_tx.send(crate::port_owner::PortCommand::ReadWait { timeout_ms: remaining.as_millis() as u64, resp_tx: rtx });
                            if let Ok(Ok(d)) = rrx.recv_timeout(std::time::Duration::from_millis(remaining.as_millis() as u64 + 500)) {
                                if !d.is_empty() { data = d; }
                            }
                        }
                    }
                }
                if !data.is_empty() {
                    let mut state = lock_state(&app_state);
                    state.rx_count += data.len() as u64;
                    state.ai_rx_count += data.len() as u64;
                    // Display RX in terminal
                    let hex_preview = crate::ui::terminal::format_hex_bytes(&data);
                    let text_preview = String::from_utf8_lossy(&data).to_string();
                    let (received, is_hex_display) = if state.hex_mode {
                        (hex_preview.clone(), true)
                    } else {
                        (text_preview.clone(), false)
                    };
                    state.add_terminal_line_source(crate::state::Direction::Rx, received, is_hex_display, "MCP");
                    state.add_log_entry(crate::state::LogLevel::Info, &format!("MCP RX {} bytes: {} | {}", data.len(), hex_preview, text_preview));
                    // Clear MCP buffer after read to prevent duplicate display
                    drop(state);
                    if let Ok(mut buf) = rx_buf.lock() {
                        buf.clear();
                    }
                }
                let _ = resp.send(Ok(data));
            }
            crate::mcp_server::McpSerialRequest::SendRead { data, timeout_ms, resp } => {
                let hex_preview = crate::ui::terminal::format_hex_bytes(&data);
                // Phase 1: Send write + display TX (lock held briefly)
                {
                    let mut state = lock_state(&app_state);
                    state.tx_count += data.len() as u64;
                    state.ai_tx_count += data.len() as u64;
                    state.add_chart_data(data.len() as f64);
                    let is_tx_text = is_printable_text(&data);
                    let tx_display = if is_tx_text { String::from_utf8_lossy(&data).to_string() } else { hex_preview.clone() };
                    state.add_terminal_line_source(crate::state::Direction::Tx, tx_display, !is_tx_text, "MCP");
                    state.add_log_entry(crate::state::LogLevel::Info, &format!("MCP TX {} bytes: {}", data.len(), hex_preview));
                }
                // Phase 2: Use exclusive write-then-read (pauses continuous reader, reads directly)
                let result = {
                    let state = lock_state(&app_state);
                    match state.port_owner {
                        Some(ref po) => po.write_read_exclusive(data, timeout_ms),
                        None => { let _ = resp.send(Err("Not connected".into())); continue; }
                    }
                };
                match result {
                    Ok(resp_data) => {
                        if !resp_data.is_empty() {
                            let mut state = lock_state(&app_state);
                            state.rx_count += resp_data.len() as u64;
                            state.ai_rx_count += resp_data.len() as u64;
                            state.add_chart_data(resp_data.len() as f64);
                            // Display RX in terminal so user can see MCP responses
                            let hex_preview = crate::ui::terminal::format_hex_bytes(&resp_data);
                            let text_preview = String::from_utf8_lossy(&resp_data).to_string();
                            let (received, is_hex_display) = if state.hex_mode {
                                (hex_preview.clone(), true)
                            } else {
                                (text_preview.clone(), false)
                            };
                            state.add_terminal_line_source(crate::state::Direction::Rx, received, is_hex_display, "MCP");
                            state.add_log_entry(crate::state::LogLevel::Info, &format!("MCP RX {} bytes: {} | {}", resp_data.len(), hex_preview, text_preview));
                        }
                        let _ = resp.send(Ok(resp_data));
                    }
                    Err(e) => { let _ = resp.send(Err(e)); }
                }
            }
            crate::mcp_server::McpSerialRequest::IsConnected { resp } => {
                let state = lock_state(&app_state);
                let _ = resp.send(state.is_connected);
            }
            crate::mcp_server::McpSerialRequest::SubscribeEvents { resp } => {
                let state = lock_state(&app_state);
                if let Some(ref po) = state.port_owner {
                    let rx = po.subscribe_events();
                    let _ = resp.send(Some(rx));
                } else {
                    let _ = resp.send(None);
                }
            }
            crate::mcp_server::McpSerialRequest::GetConfig { key, resp } => {
                let state = lock_state(&app_state);
                let config = build_config_map(&state);
                let result = match key {
                    Some(k) => match config.get(&k) {
                        Some(v) => v.clone(),
                        None => serde_json::json!({"error": format!("Unknown key: {}", k)}),
                    },
                    None => serde_json::json!(config),
                };
                let _ = resp.send(result);
            }
            crate::mcp_server::McpSerialRequest::SetConfig { key, value, resp } => {
                // Clone prefs before lock to save outside lock
                let prefs_clone;
                let reconfigure_info;
                {
                    let mut state = lock_state(&app_state);
                    let result = apply_config(&mut state, &key, &value);
                    if result.is_ok() {
                        state.mcp_config_dirty = true;
                        prefs_clone = Some(crate::state::UserPrefs::from_state(&state));
                        // Detect MCP settings change
                        if key == "mcp_port" || key == "mcp_bind_lan" {
                            let bind_addr = if state.mcp_bind_lan { "0.0.0.0".to_string() } else { "127.0.0.1".to_string() };
                            reconfigure_info = Some((bind_addr, state.mcp_port));
                        } else {
                            reconfigure_info = None;
                        }
                        let _ = resp.send(result);
                    } else {
                        prefs_clone = None;
                        reconfigure_info = None;
                        let _ = resp.send(result);
                    }
                }
                // Save prefs OUTSIDE the lock to avoid blocking UI
                if let Some(prefs) = prefs_clone {
                    crate::app::save_prefs(&prefs);
                }
                // Reconfigure MCP server outside the lock
                if let Some((ref bind_addr, port)) = reconfigure_info {
                    let state = lock_state(&app_state);
                    if let Some(ref cmd_tx) = state.mcp_cmd_tx {
                        let _ = cmd_tx.send(crate::mcp_server::McpCommand::Reconfigure { bind_addr: bind_addr.clone(), port });
                        eprintln!("[MCP] Reconfiguring server to {}:{}", bind_addr, port);
                    }
                }
            }
        }
    }
    eprintln!("[MCP-Serial] Request processing thread stopped");
}

/// Poll all pending port owner events and apply them to state.
/// Returns true if any RX data was received.
fn poll_port_events(state: &mut AppState) -> bool {
    let mut has_rx_data = false;
    let events: Vec<_> = if let Some(ref port_owner) = state.port_owner {
        let mut evts = Vec::new();
        while let Some(evt) = port_owner.poll() {
            evts.push(evt);
        }
        evts
    } else {
        Vec::new()
    };
    for evt in events {
        match evt {
            crate::port_owner::PortEvent::Data(data) => {
                state.rx_count += data.len() as u64;
                state.ai_rx_count += data.len() as u64;
                state.add_chart_data(data.len() as f64);
                let hex_preview = crate::ui::terminal::format_hex_bytes(&data);
                let text_preview = String::from_utf8_lossy(&data).to_string();
                let (received, is_hex_display) = if state.hex_mode {
                    (hex_preview.clone(), true)
                } else {
                    (text_preview.clone(), false)
                };
                state.add_terminal_line(crate::state::Direction::Rx, received, is_hex_display);
                state.add_log_entry(crate::state::LogLevel::Info, &format!("RX {} bytes: {} | {}", data.len(), hex_preview, text_preview));
                super::ui::data_logger::log_data(state, "RX", &data);
                has_rx_data = true;
                // Auto-reply
                if state.auto_reply_enabled && !state.auto_reply_pattern.is_empty() && !state.auto_reply_response.is_empty() {
                    if text_preview.contains(&state.auto_reply_pattern) {
                        let reply = state.auto_reply_response.clone();
                        let reply_bytes = reply.as_bytes().to_vec();
                        state.tx_count += reply_bytes.len() as u64;
                        state.ai_tx_count += reply_bytes.len() as u64;
                        state.add_chart_data(reply_bytes.len() as f64);
                        if let Some(ref po) = state.port_owner {
                            po.send(crate::port_owner::PortCommand::Write(reply_bytes));
                        }
                        state.add_terminal_line(crate::state::Direction::Tx, reply.clone(), false);
                    }
                }
            }
            crate::port_owner::PortEvent::Opened(ok, msg) => {
                if ok {
                    state.is_connected = true;
                    state.add_log_entry(crate::state::LogLevel::Info, &format!("Connected to {}", msg));
                } else {
                    state.is_connected = false;
                    state.show_error(&msg);
                }
            }
            crate::port_owner::PortEvent::Closed => {
                state.is_connected = false;
                state.replay_running = false;
                state.replay_index = 0;
                state.replay_commands.clear();
                state.auto_send_enabled = false;
                state.dtr = true;
                state.rts = false;
                state.ai_connected = false;
                state.ai_port_name.clear();
                state.ai_baud_rate = 0;
                state.connected_by.clear();
            }
            crate::port_owner::PortEvent::Written(_) => {}
            crate::port_owner::PortEvent::Error(e) => {
                state.show_error(&e);
            }
        }
    }
    has_rx_data
}

/// Process replay commands if a replay is active.
fn process_replay(state: &mut AppState, ctx: &egui::Context) {
    if !state.replay_running || !state.is_connected {
        return;
    }
    let now = chrono::Utc::now().timestamp_millis();
    let elapsed = (now - state.replay_start_time) as u64;

    let mut cumulative_delay: u64 = 0;
    for i in 0..state.replay_index {
        cumulative_delay += state.replay_commands[i].delay_ms;
    }

    while state.replay_index < state.replay_commands.len() {
        let cmd = &state.replay_commands[state.replay_index];
        if cmd.action == crate::state::ScriptAction::Wait {
            cumulative_delay += cmd.delay_ms;
            state.replay_index += 1;
            continue;
        }
        if elapsed >= cumulative_delay {
            if let Some(ref data) = cmd.data {
                let hex_mode = state.hex_mode;
                let checksum_mode = state.terminal_checksum_mode;
                let line_ending = state.line_ending;

                let mut bytes = if hex_mode {
                    match crate::ui::terminal::parse_hex(data) {
                        Some(b) => b,
                        None => { state.replay_index += 1; continue; }
                    }
                } else {
                    let mut b = data.as_bytes().to_vec();
                    b.extend_from_slice(line_ending.suffix());
                    b
                };
                bytes = checksum_mode.append_checksum(&bytes);

                let display = if hex_mode { data.clone() } else { data.replace("\r", "\\r").replace("\n", "\\n") };
                let idx = state.replay_index;
                let byte_len = bytes.len();
                let hex_preview = crate::ui::terminal::format_hex_bytes(&bytes);
                let text_preview = String::from_utf8_lossy(&bytes).to_string();
                state.tx_count += byte_len as u64;
                state.add_chart_data(byte_len as f64);
                state.add_terminal_line(crate::state::Direction::Tx, display, false);
                state.add_log_entry(crate::state::LogLevel::Info, &format!("Replay [{}] TX {} bytes: {} | {}", idx, byte_len, hex_preview, text_preview));
                super::ui::data_logger::log_data(state, "TX", &bytes);

                if let Some(ref po) = state.port_owner {
                    po.send(crate::port_owner::PortCommand::Write(bytes));
                }
            }
            state.replay_index += 1;
        } else {
            break;
        }
    }

    if state.replay_index >= state.replay_commands.len() {
        state.replay_running = false;
        state.replay_commands.clear();
        state.replay_index = 0;
        state.add_log_entry(crate::state::LogLevel::Info, "Replay completed");
    }

    ctx.request_repaint();
}

/// Flush dirty persistence data (throttled to max once per second).
fn flush_persistence(state: &mut AppState) {
    let now = chrono::Utc::now().timestamp_millis();
    if state.terminal_dirty && now - state.terminal_last_save > 1000 {
        state.terminal_dirty = false;
        state.terminal_last_save = now;
        state.save_terminal();
    }
    if state.logs_dirty && now - state.logs_last_save > 1000 {
        state.logs_dirty = false;
        state.logs_last_save = now;
        state.save_logs();
    }
}

impl eframe::App for SerialRunApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());

        // --- Background processing ---
        sync_theme_visuals(ctx, state.theme, &mut self.current_theme);

        {
            let current_prefs = crate::state::UserPrefs::from_state(&state);
            if current_prefs != self.last_prefs {
                save_prefs(&current_prefs);
                self.last_prefs = current_prefs;
            }
        }

        poll_mcp_status(&self.mcp_handle, &mut state);
        poll_mcp_log(&self.mcp_handle, &mut state);

        // MCP changed config — force immediate GUI repaint so controls update
        if state.mcp_config_dirty {
            state.mcp_config_dirty = false;
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }

        let has_rx_data = poll_port_events(&mut state);
        if has_rx_data {
            ctx.request_repaint();
        }
        if state.is_connected || state.replay_running || state.auto_send_enabled {
            ctx.request_repaint_after(std::time::Duration::from_millis(8));
        }

        process_replay(&mut state, ctx);

        // Process STC ISP panel actions (detect/flash) — release lock before plugin execution
        {
            let action = state.stc_action.take();
            if let Some(action) = action {
                let port = state.stc_port.clone();
                let firmware = state.stc_firmware_path.clone();
                let panel_baud = state.stc_baud_rate;

                // If no port selected in STC panel, use the main app's connected port
                let port = if port.is_empty() {
                    state.selected_port.clone().unwrap_or_default()
                } else {
                    port
                };

                // Validate port selection
                if port.is_empty() {
                    state.stc_flash_running = false;
                    state.stc_log.push("[STC] Error: No serial port available. Connect a port first.".to_string());
                } else {
                    // ISP detection uses 9600 baud; flash uses the panel baud rate
                    let connect_baud = match action {
                        crate::ui::stc_panel::StcAction::Detect => 9600u32,
                        crate::ui::stc_panel::StcAction::Flash => panel_baud,
                    };

                    state.stc_flash_running = true;
                    state.stc_flash_status = match action {
                        crate::ui::stc_panel::StcAction::Detect => "Detecting...".to_string(),
                        crate::ui::stc_panel::StcAction::Flash => "Flashing...".to_string(),
                    };
                    state.stc_log.push(format!("[STC] Starting {:?}...", action));

                    // Auto-connect if not connected to the right port or wrong baud rate
                    let needs_reconnect = !state.is_connected
                        || state.selected_port.as_deref() != Some(&port)
                        || state.config.baud_rate != connect_baud;
                    // Save original connection for restoration after STC operation
                    let orig_port = if needs_reconnect { state.selected_port.clone() } else { None };
                    let orig_baud = if needs_reconnect { Some(state.config.baud_rate) } else { None };
                    if needs_reconnect {
                        // Disconnect first if connected
                        if state.is_connected {
                            let old_port = state.selected_port.clone().unwrap_or_default();
                            let old_baud = state.config.baud_rate;
                            state.stc_log.push(format!("[STC] Temporarily disconnecting ({} @ {} baud)...", old_port, old_baud));
                            state.port_owner = None;
                            state.is_connected = false;
                        }
                        state.stc_log.push(format!("[STC] Connecting to {} @ {} baud...", port, connect_baud));
                        let po = crate::port_owner::PortOwnerHandle::start();
                        po.sync_timeout(state.rx_aggregate_ms);
                        let config = serialrun_core::config::SerialConfig {
                            port_name: port.clone(),
                            baud_rate: connect_baud,
                            ..Default::default()
                        };
                        po.send(crate::port_owner::PortCommand::Open(config));
                        state.port_owner = Some(po);
                        state.selected_port = Some(port.clone());
                        state.config.baud_rate = connect_baud;
                        state.baud_rate_text = connect_baud.to_string();
                    }

                    drop(state); // Release lock before plugin execution

                    let plugins = get_loaded_plugins().clone();
                    let state_arc = self.state.clone();
                    std::thread::spawn(move || {
                        // Wait for serial connection to be established (async open)
                        for _ in 0..50 {
                            {
                                let s = state_arc.lock().unwrap_or_else(|e| e.into_inner());
                                if s.is_connected { break; }
                            }
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }

                        let cmd_name = match action {
                            crate::ui::stc_panel::StcAction::Detect => "detect",
                            crate::ui::stc_panel::StcAction::Flash => "flash",
                        };
                        // Use 9600 for detect (ISP protocol), panel baud for flash
                        let plugin_baud = match action {
                            crate::ui::stc_panel::StcAction::Detect => 9600u32,
                            crate::ui::stc_panel::StcAction::Flash => panel_baud,
                        };
                        let params = match action {
                            crate::ui::stc_panel::StcAction::Detect => {
                                serde_json::json!({"port": port, "baud_rate": plugin_baud}).to_string()
                            }
                            crate::ui::stc_panel::StcAction::Flash => {
                                serde_json::json!({"port": port, "baud_rate": plugin_baud, "firmware_path": firmware}).to_string()
                            }
                        };

                        let result = {
                            let mut plugs = plugins.lock().unwrap_or_else(|e| e.into_inner());
                            if let Some(loaded) = plugs.get_mut("serialrun-stc-isp") {
                                loaded.execute_command(cmd_name, &params)
                            } else {
                                Err(serialrun_core::plugin::PluginError::PluginError("STC plugin not loaded".to_string()))
                            }
                        };

                        // Restore original connection if we had one
                        if orig_port.is_some() {
                            let mut state = state_arc.lock().unwrap_or_else(|e| e.into_inner());
                            state.port_owner = None;
                            state.is_connected = false;

                            if let Some(orig_port) = orig_port {
                                let orig_baud = orig_baud.unwrap_or(115200);
                                state.stc_log.push(format!("[STC] Restoring connection to {} @ {} baud...", orig_port, orig_baud));
                                let po = crate::port_owner::PortOwnerHandle::start();
                                po.sync_timeout(state.rx_aggregate_ms);
                                let config = serialrun_core::config::SerialConfig {
                                    port_name: orig_port.clone(),
                                    baud_rate: orig_baud,
                                    ..Default::default()
                                };
                                po.send(crate::port_owner::PortCommand::Open(config));
                                state.port_owner = Some(po);
                                state.selected_port = Some(orig_port);
                                state.config.baud_rate = orig_baud;
                                state.baud_rate_text = orig_baud.to_string();
                            }
                        }

                        // Update state with results
                        let mut state = state_arc.lock().unwrap_or_else(|e| e.into_inner());
                        state.stc_flash_running = false;
                        match result {
                            Ok(res) => {
                                if res.success {
                                    if let Some(val) = res.result {
                                        // Plugin returns "info" key for chip info
                                        if let Some(info) = val.get("info").and_then(|v| v.as_str()) {
                                            state.stc_chip_info = info.to_string();
                                        }
                                        if let Some(family) = val.get("family").and_then(|v| v.as_str()) {
                                            state.stc_log.push(format!("[STC] Chip: {} (flash: {}KB, eeprom: {}KB)",
                                                family,
                                                val.get("flash_size").and_then(|v| v.as_u64()).unwrap_or(0),
                                                val.get("eeprom_size").and_then(|v| v.as_u64()).unwrap_or(0),
                                            ));
                                        }
                                        state.stc_flash_status = "Done".to_string();
                                        state.stc_log.push(format!("[STC] {} OK", cmd_name));
                                    }
                                } else {
                                    let err = res.error.unwrap_or_default();
                                    state.stc_flash_status = format!("Error: {}", err);
                                    state.stc_log.push(format!("[STC] {} failed: {}", cmd_name, err));
                                }
                            }
                            Err(e) => {
                                state.stc_flash_status = format!("Error: {}", e);
                                state.stc_log.push(format!("[STC] {} failed: {}", cmd_name, e));
                            }
                        }
                });
                // Re-lock state for remaining update logic
                state = self.state.lock().unwrap_or_else(|e| e.into_inner());
                } // end else (port valid)
            }
        }

        // Auto-send logic
        if state.auto_send_enabled && state.is_connected && !state.input_buffer.is_empty() {
            let now = chrono::Utc::now().timestamp_millis();
            if now - state.auto_send_last_time >= state.auto_send_interval_ms as i64 {
                state.auto_send_last_time = now;
                let saved_input = state.input_buffer.clone();
                super::ui::terminal::do_send(&mut state);
                if !state.keep_input && state.auto_send_enabled {
                    state.input_buffer = saved_input;
                }
            }
            ctx.request_repaint();
        }

        let lang = state.language;

        // --- Main layout ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui::connection::render_connection_panel(ui, &mut state, ctx);
        });
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui::status::render_status_bar(ui, &mut state);
        });
        egui::SidePanel::left("side_panel").resizable(false).default_width(200.0).show(ctx, |ui| {
            ui::connection::render_connection_controls(ui, &mut state);
            ui::settings::render_settings_panel(ui, &mut state, ctx);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui::terminal::render_terminal_panel(ui, &mut state);
        });

        // --- Floating windows (OS-level viewport, stays visible on main window focus) ---
        macro_rules! viewport_window {
            ($show:expr, $id:expr, $title:expr, $render:expr, $w:expr, $h:expr) => {
                {
                    let vid = egui::ViewportId(egui::Id::new($id));
                    let was_open = self.open_viewports.contains(&vid);
                    let is_new = $show && !was_open;

                    // Track open state
                    if $show && !was_open {
                        self.open_viewports.insert(vid);
                    }

                    // Always render if open (prevents hiding on main window focus)
                    if self.open_viewports.contains(&vid) {
                        let mut builder = egui::ViewportBuilder::default()
                            .with_title($title)
                            .with_inner_size([$w, $h])
                            .with_active(true)
                            .with_window_level(egui::WindowLevel::AlwaysOnTop);
                        // Only set position on first open — let OS manage after that
                        if is_new {
                            let main_center = ctx.input(|i| i.viewport().inner_rect)
                                .map(|r| egui::pos2(r.center().x, r.center().y))
                                .unwrap_or(egui::pos2(400.0, 300.0));
                            builder = builder.with_position(egui::pos2(main_center.x - $w / 2.0, main_center.y - $h / 2.0));
                        }
                        ctx.show_viewport_immediate(
                            vid,
                            builder,
                            |ctx, _class| {
                                if ctx.input(|i| i.viewport().close_requested()) {
                                    $show = false;
                                    self.open_viewports.remove(&vid);
                                    return;
                                }
                                egui::CentralPanel::default().show(ctx, |ui| { $render(ui, &mut state); });
                            },
                        );
                    }

                    // Sync state when user closes via $show toggle
                    if !$show && was_open {
                        self.open_viewports.remove(&vid);
                    }
                }
            };
        }

        viewport_window!(state.show_chart_window, "chart", T::data_chart(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::charts::render_chart_panel(ui, s), 400.0, 300.0);
        viewport_window!(state.show_log_window, "log", T::log_viewer(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::log_viewer::render_log_panel(ui, s), 400.0, 350.0);
        viewport_window!(state.show_help, "help", T::help(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::help::render_help_panel(ui, s), 600.0, 500.0);
        viewport_window!(state.show_modbus_window, "modbus", T::modbus_panel(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::modbus::render_modbus_panel(ui, s), 520.0, 450.0);
        viewport_window!(state.show_plc_window, "plc", T::plc_control(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::plc::render_plc_panel(ui, s), 600.0, 500.0);
        viewport_window!(state.show_bridge_window, "bridge", T::bridge(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::bridge::render_bridge_panel(ui, s), 520.0, 450.0);
        viewport_window!(state.show_simulator_window, "simulator", T::simulator(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::simulator::render_simulator_panel(ui, s), 520.0, 500.0);
        viewport_window!(state.show_checksum_window, "checksum", T::checksum(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::checksum::render_checksum_panel(ui, s), 400.0, 350.0);
        viewport_window!(state.show_file_transfer_window, "file_transfer", T::file_transfer(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::file_transfer::render_file_transfer_panel(ui, s), 420.0, 300.0);
        viewport_window!(state.show_frame_builder_window, "frame_builder", T::frame_builder(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::frame_builder::render_frame_builder_panel(ui, s), 450.0, 350.0);
        viewport_window!(state.show_data_logger_window, "data_logger", T::data_logger(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::data_logger::render_data_logger_panel(ui, s), 400.0, 250.0);
        viewport_window!(state.show_can_window, "can", T::can_analyzer(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::can_analyzer::render_can_analyzer_panel(ui, s), 550.0, 400.0);
        viewport_window!(state.show_i2c_spi_window, "i2c_spi", T::i2c_spi(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::i2c_spi::render_i2c_spi_panel(ui, s), 450.0, 380.0);
        viewport_window!(state.show_scope_window, "scope", T::oscilloscope(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::serial_scope::render_serial_scope_panel(ui, s), 600.0, 480.0);
        viewport_window!(state.show_flasher_window, "flasher", T::flasher(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::flasher::render_flasher_panel(ui, s), 420.0, 350.0);
        viewport_window!(state.show_register_editor_window, "reg_editor", T::register_editor(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::register_editor::render_register_editor_panel(ui, s), 500.0, 400.0);
        viewport_window!(state.show_plugin_window, "plugin", T::plugins(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::plugin::render_plugin_panel(ui, s), 480.0, 400.0);

        // Dynamic plugin windows — each enabled plugin with a window config gets its own independent OS viewport
        {
            let plugin_names: Vec<String> = state.plugins.iter()
                .filter(|p| p.loaded && p.enabled && p.window_config.is_some())
                .map(|p| p.manifest_name.clone())
                .collect();

            for manifest_name in plugin_names {
                let is_open = state.plugin_windows.get(&manifest_name).copied().unwrap_or(false);
                if !is_open { continue; }

                // Get window config from plugin info
                let (title, default_w, default_h) = {
                    let plugin = state.plugins.iter().find(|p| p.manifest_name == manifest_name);
                    if let Some(p) = plugin {
                        let wc = p.window_config.as_ref().unwrap();
                        (wc.title.clone(), wc.default_width, wc.default_height)
                    } else {
                        continue;
                    }
                };

                let vid = egui::ViewportId(egui::Id::new(format!("plugin_{}", manifest_name)));
                let was_open = self.open_viewports.contains(&vid);
                let is_new = !was_open;

                if !was_open {
                    self.open_viewports.insert(vid);
                }

                let mn = manifest_name.clone();
                ctx.show_viewport_immediate(
                    vid,
                    egui::ViewportBuilder::default()
                        .with_title(&title)
                        .with_inner_size([default_w, default_h])
                        .with_active(true)
                        .with_window_level(egui::WindowLevel::Normal),
                    |ctx, _class| {
                        if ctx.input(|i| i.viewport().close_requested()) {
                            state.plugin_windows.insert(mn.clone(), false);
                            self.open_viewports.remove(&vid);
                            return;
                        }
                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui::plugin::render_plugin_window_content(ui, &mut state, &mn);
                        });
                    },
                );
            }
        }
        viewport_window!(
            state.show_mcp_log_popup, "mcp_log",
            if lang == Language::Chinese { "MCP 访问日志" } else { "MCP Access Log" },
            |ui: &mut egui::Ui, s: &mut AppState| ui::mcp_log::render_mcp_log_popup(ui, s),
            560.0, 420.0
        );

        flush_persistence(&mut state);
    }
}
