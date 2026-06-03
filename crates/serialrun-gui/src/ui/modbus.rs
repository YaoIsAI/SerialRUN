use crate::state::{AppState, Language, ModbusFrameLogEntry, ModbusFunctionCode, MonitorEntry, T};
use eframe::egui;
use serialrun_core::protocol::{ModbusFrame, ModbusParser};

/// Map Modbus exception code to human-readable description
fn modbus_exception_name(code: u8) -> &'static str {
    match code {
        0x01 => "Illegal Function",
        0x02 => "Illegal Data Address",
        0x03 => "Illegal Data Value",
        0x04 => "Slave Device Failure",
        0x05 => "Acknowledge",
        0x06 => "Slave Device Busy",
        0x08 => "Memory Parity Error",
        0x0A => "Gateway Path Unavailable",
        0x0B => "Gateway Target Failed to Respond",
        _ => "Unknown Exception",
    }
}

pub fn render_modbus_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    ui.horizontal(|ui| {
        ui.heading(T::modbus_panel(lang));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.button(egui::RichText::new("?").size(14.0).strong())
                .on_hover_text(T::modbus_tip_header(lang));
        });
    });
    ui.separator();

    // Poll async Modbus result
    if let Some(rx) = &state.modbus_async_receiver {
        if let Ok(result) = rx.try_recv() {
            state.modbus_async_receiver = None;
            match result {
                Ok(resp) => {
                    let resp_hex = hex_str(&resp);
                    state.add_log_entry(crate::state::LogLevel::Info, &format!("[Modbus] RX: {}", resp_hex));
                    state.add_terminal_line(crate::state::Direction::Rx, format!("[Modbus] {}", resp_hex), true);
                    if let Ok(f) = ModbusFrame::parse(&resp) {
                        state.modbus.last_response_hex = resp_hex.clone();
                        let is_err = f.is_exception();
                        let decoded = if is_err {
                            let code = f.exception_code().unwrap_or(0);
                            let name = modbus_exception_name(code);
                            format!("Exception 0x{:02X}: {}", code, name)
                        } else {
                            ModbusParser::format_frame(&f)
                        };
                        state.modbus.frame_log.push_back(ModbusFrameLogEntry {
                            timestamp: chrono::Utc::now().timestamp_millis(),
                            request_hex: state.modbus.last_request_hex.clone(),
                            response_hex: resp_hex,
                            decoded,
                            is_error: is_err,
                        });
                        if is_err {
                            let code = f.exception_code().unwrap_or(0);
                            state.modbus.last_error = Some(format!("Exception 0x{:02X}: {}", code, modbus_exception_name(code)));
                        }
                        if state.modbus.frame_log.len() > 1000 { state.modbus.frame_log.pop_front(); }
                    } else {
                        state.modbus.last_error = Some("Response parse error — check baud rate and wiring".into());
                    }
                }
                Err(e) => { state.modbus.last_error = Some(e.clone()); state.add_log_entry(crate::state::LogLevel::Error, &format!("[Modbus] Error: {}", e)); state.show_error(&e); }
            }
        }
    }

    ui.collapsing(T::quick_request(lang), |ui| { render_quick_request(ui, state); });
    ui.separator();
    ui.collapsing(T::register_monitor(lang), |ui| { render_register_monitor(ui, state); });
    ui.separator();
    ui.collapsing(T::frame_log(lang), |ui| { render_frame_log(ui, state); });
}

fn render_quick_request(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    ui.horizontal(|ui| {
        ui.label(T::slave_id(lang));
        ui.add(egui::DragValue::new(&mut state.modbus.slave_id).range(0..=247).prefix(" "));
        ui.label(T::function_code(lang));
        let fc = state.modbus.function_code;
        egui::ComboBox::from_id_salt("modbus_fc").width(80.0).selected_text(fc.label(lang)).show_ui(ui, |ui| {
            for &f in ModbusFunctionCode::all() { ui.selectable_value(&mut state.modbus.function_code, f, f.label(lang)); }
        });
        ui.label(T::start_address(lang));
        ui.add(egui::TextEdit::singleline(&mut state.modbus.start_addr).desired_width(50.0));
        if state.modbus.function_code.is_read() {
            ui.label(T::quantity(lang));
            ui.add(egui::TextEdit::singleline(&mut state.modbus.quantity).desired_width(40.0));
        } else {
            ui.label(T::write_value(lang));
            ui.add(egui::TextEdit::singleline(&mut state.modbus.write_value).desired_width(60.0));
        }
    });
    ui.horizontal(|ui| {
        ui.label(if lang == Language::Chinese { "响应超时" } else { "Timeout" });
        ui.add(egui::DragValue::new(&mut state.modbus.response_timeout_ms).range(50..=5000).suffix("ms"));
    });
    ui.add_space(4.0);
    if ui.button(T::send_request(lang)).clicked() { do_modbus_request(state); }
    if let Some(ref err) = state.modbus.last_error { ui.colored_label(egui::Color32::RED, err.as_str()); }
    if !state.modbus.last_request_hex.is_empty() {
        ui.separator();
        ui.label(egui::RichText::new(T::last_request(lang)).strong());
        ui.label(egui::RichText::new(&state.modbus.last_request_hex).monospace());
        ui.label(egui::RichText::new(T::last_response(lang)).strong());
        ui.label(egui::RichText::new(&state.modbus.last_response_hex).monospace());
    }
}

fn do_modbus_request(state: &mut AppState) {
    state.modbus.last_error = None;
    let addr: u16 = match state.modbus.start_addr.parse() { Ok(v) => v, Err(_) => { let m = "Invalid address".to_string(); state.modbus.last_error = Some(m.clone()); state.show_error(&m); return; } };
    let frame = if state.modbus.function_code.is_read() {
        let qty: u16 = match state.modbus.quantity.parse() { Ok(v) => v, Err(_) => { let m = "Invalid quantity".to_string(); state.modbus.last_error = Some(m.clone()); state.show_error(&m); return; } };
        ModbusParser::build_read_request(state.modbus.slave_id, state.modbus.function_code.to_core_function(), addr, qty)
    } else {
        let val: u16 = match state.modbus.write_value.parse() { Ok(v) => v, Err(_) => { let m = "Invalid value".to_string(); state.modbus.last_error = Some(m.clone()); state.show_error(&m); return; } };
        ModbusParser::build_write_single(state.modbus.slave_id, addr, val)
    };
    let req_bytes = frame.to_bytes();
    let req_hex = hex_str(&req_bytes);
    state.modbus.last_request_hex = req_hex.clone();
    state.add_log_entry(crate::state::LogLevel::Info, &format!("[Modbus] TX: {}", req_hex));
    state.add_terminal_line(crate::state::Direction::Tx, format!("[Modbus] {}", req_hex), true);

    // Start async request via port owner
    if state.modbus_async_receiver.is_none() {
        let (tx, rx) = std::sync::mpsc::channel();
        let po = state.port_owner.as_ref().map(|p| p.cmd_tx());
        let timeout_ms = state.modbus.response_timeout_ms;
        state.modbus_async_receiver = Some(rx);
        std::thread::spawn(move || {
            let Some(cmd_tx) = po else { let _ = tx.send(Err("Not connected".into())); return; };
            let (resp_tx, resp_rx) = std::sync::mpsc::channel();
            let _ = cmd_tx.send(crate::port_owner::PortCommand::ReadExclusive { data: req_bytes, timeout_ms, resp_tx });
            let result = resp_rx.recv().unwrap_or_else(|e| Err(format!("Channel closed: {}", e)));
            let _ = tx.send(result.and_then(|data| {
                if data.len() >= 4 { Ok(data) } else { Err("Response too short".into()) }
            }));
        });
    }
}

fn render_register_monitor(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    ui.horizontal(|ui| {
        ui.label(T::slave_id(lang));
        ui.add(egui::DragValue::new(&mut state.modbus.monitor_slave_id).range(0..=247).prefix(" "));
        ui.label(T::start_address(lang));
        ui.add(egui::TextEdit::singleline(&mut state.modbus.monitor_start_addr).desired_width(50.0));
        ui.label(T::quantity(lang));
        ui.add(egui::TextEdit::singleline(&mut state.modbus.monitor_quantity).desired_width(40.0));
        ui.label(T::poll_interval(lang));
        ui.add(egui::DragValue::new(&mut state.modbus.monitor_interval_ms).range(100..=10000).suffix("ms"));
    });
    ui.add_space(4.0);

    // Poll async monitor result
    if let Some(ref rx) = state.modbus_monitor_async {
        if let Ok(result) = rx.try_recv() {
            state.modbus_monitor_async = None;
            match result {
                Ok(resp) => {
                    if let Ok(f) = ModbusFrame::parse(&resp) {
                        let data = &f.data;
                        let addr: u16 = state.modbus.monitor_start_addr.parse().unwrap_or(0);
                        if data.len() >= 2 {
                            state.modbus.monitor_entries.clear();
                            let mut i = 1;
                            while i + 1 < data.len() {
                                let val = u16::from_be_bytes([data[i], data[i + 1]]);
                                state.modbus.monitor_entries.push(MonitorEntry { addr: addr + (state.modbus.monitor_entries.len() as u16), raw_value: val, display_value: format!("{}", val), last_update: chrono::Utc::now().timestamp_millis(), error: None });
                                i += 2;
                            }
                            // Output to terminal — show raw response only
                            let hex_preview = resp.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                            state.add_terminal_line(crate::state::Direction::Rx, format!("[Modbus Mon] {}", hex_preview), true);
                            state.add_log_entry(crate::state::LogLevel::Info, &format!("[Modbus Mon] Poll OK: {} regs", state.modbus.monitor_entries.len()));
                        }
                    }
                }
                Err(e) => {
                    state.add_log_entry(crate::state::LogLevel::Error, &format!("[Modbus Mon] Error: {}", e));
                }
            }
        }
    }

    let label = if state.modbus.monitor_polling { T::stop_monitor(lang) } else { T::start_monitor(lang) };
    if ui.button(label).clicked() {
        if state.modbus.monitor_polling { state.modbus.monitor_polling = false; }
        else if state.is_connected { state.modbus.monitor_polling = true; state.modbus.last_poll_time = 0; }
    }
    if state.modbus.monitor_polling && state.is_connected && state.modbus_monitor_async.is_none() {
        let now = chrono::Utc::now().timestamp_millis();
        if now - state.modbus.last_poll_time >= state.modbus.monitor_interval_ms as i64 {
            do_monitor_poll(state);
            state.modbus.last_poll_time = now;
        }
    }
    if !state.modbus.monitor_entries.is_empty() {
        ui.separator();
        // Freeze age display when monitoring is stopped (use last update time + 0ms)
        let now_ms = if state.modbus.monitor_polling {
            chrono::Utc::now().timestamp_millis()
        } else {
            state.modbus.monitor_entries.iter().map(|e| e.last_update).max().unwrap_or(0)
        };
        egui::Grid::new("modbus_mon_table").striped(true).spacing([12.0, 4.0]).show(ui, |ui| {
            // Header
            ui.label(egui::RichText::new(T::plc_addr_label(lang)).strong());
            ui.label(egui::RichText::new("HEX").strong());
            ui.label(egui::RichText::new(T::plc_value_label(lang)).strong());
            ui.label(egui::RichText::new("Age").strong());
            ui.end_row();

            for entry in &state.modbus.monitor_entries {
                // Address
                ui.label(egui::RichText::new(format!("0x{:04X}", entry.addr)).monospace());
                // Raw HEX
                ui.label(egui::RichText::new(format!("0x{:04X}", entry.raw_value)).monospace().color(egui::Color32::from_rgb(100, 180, 255)));
                // Value with age-based color
                let age = now_ms - entry.last_update;
                let color = if age < 3000 { egui::Color32::from_rgb(0, 200, 0) }
                    else if age < 10000 { egui::Color32::from_rgb(200, 180, 0) }
                    else { egui::Color32::from_rgb(180, 60, 60) };
                ui.label(egui::RichText::new(&entry.display_value).monospace().color(color));
                // Age
                let age_text = if age < 1000 { format!("{}ms", age) } else { format!("{:.1}s", age as f64 / 1000.0) };
                ui.label(egui::RichText::new(age_text).weak());
                ui.end_row();
            }
        });
    }
}

fn do_monitor_poll(state: &mut AppState) {
    let addr: u16 = match state.modbus.monitor_start_addr.parse() { Ok(v) => v, Err(_) => return };
    let qty: u16 = match state.modbus.monitor_quantity.parse() { Ok(v) => v, Err(_) => return };
    let timeout_ms = state.modbus.response_timeout_ms;
    let frame = ModbusParser::build_read_request(state.modbus.monitor_slave_id, state.modbus.monitor_function.to_core_function(), addr, qty);
    let req = frame.to_bytes();
    let po = match state.port_owner.as_ref().map(|p| p.cmd_tx()) {
        Some(tx) => tx,
        None => return,
    };
    let (tx, rx) = std::sync::mpsc::channel();
    state.modbus_monitor_async = Some(rx);
    std::thread::spawn(move || {
        let (resp_tx, resp_rx) = std::sync::mpsc::channel();
        let _ = po.send(crate::port_owner::PortCommand::ReadExclusive { data: req, timeout_ms, resp_tx });
        let result = resp_rx.recv().unwrap_or_else(|e| Err(format!("Channel closed: {}", e)));
        let _ = tx.send(result);
    });
}

fn render_frame_log(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    let available = ui.available_height().max(100.0).min(400.0);
    egui::ScrollArea::vertical().max_height(available).stick_to_bottom(true).show(ui, |ui| {
        for entry in state.modbus.frame_log.iter().rev() {
            let ts = chrono::DateTime::from_timestamp_millis(entry.timestamp).map(|t| t.with_timezone(&chrono::Local).format("%H:%M:%S%.3f").to_string()).unwrap_or_default();
            let color = if entry.is_error { egui::Color32::RED } else { egui::Color32::GREEN };
            ui.label(egui::RichText::new(format!("[{}] {}", ts, entry.decoded)).color(color).monospace());
            ui.label(egui::RichText::new(format!("TX: {}", entry.request_hex)).monospace().weak());
            ui.label(egui::RichText::new(format!("RX: {}", entry.response_hex)).monospace().color(color));
            ui.separator();
        }
    });
    if ui.button(T::clear_frame_log(lang)).clicked() { state.modbus.frame_log.clear(); }
}

fn hex_str(bytes: &[u8]) -> String { crate::util::format_hex(bytes) }
