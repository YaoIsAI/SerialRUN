use crate::plc_presets;
use crate::state::{AppState, PlcBrand, PlcDataType, PlcRegisterDef, PlcRegisterValue, T};
use eframe::egui;
use serialrun_core::protocol::{ModbusFrame, ModbusParser};

pub fn render_plc_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    poll_async_results(state);
    poll_plc_port_events(state);

    // Force repaint when polling — egui only repaints on user input by default
    if state.plc.polling || state.plc_async_receiver.is_some() {
        ui.ctx().request_repaint();
    }

    // ── Row 1: Connection (Port | Baud | Connect/Disconnect) ──
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        let plc_color = if state.plc_port_connected { egui::Color32::from_rgb(0, 200, 0) } else { egui::Color32::from_rgb(180, 60, 60) };
        ui.label(egui::RichText::new("\u{25CF}").size(10.0).color(plc_color));

        let port_text = state.plc_selected_port.as_deref().unwrap_or("—");
        egui::ComboBox::from_id_salt("plc_port").width(110.0).selected_text(port_text).show_ui(ui, |ui| {
            // Auto-refresh ports when dropdown is opened
            refresh_plc_ports(state);
            for port in &state.plc_port_list {
                let name = port.name.clone();
                let is_terminal = name == state.selected_port.as_deref().unwrap_or("") && state.is_connected;
                let is_plc = name == state.plc_selected_port.as_deref().unwrap_or("") && state.plc_port_connected;
                let label = if is_terminal && is_plc {
                    format!("{} ({})", name, T::port_both(lang))
                } else if is_terminal {
                    format!("{} ({})", name, T::port_terminal(lang))
                } else if is_plc {
                    format!("{} ({})", name, T::port_plc(lang))
                } else {
                    name.clone()
                };
                ui.selectable_value(&mut state.plc_selected_port, Some(name), label);
            }
        });

        let baud_text = format!("{}", state.plc_baud_rate);
        egui::ComboBox::from_id_salt("plc_baud").width(75.0).selected_text(&baud_text).show_ui(ui, |ui| {
            for &rate in &[9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600] {
                ui.selectable_value(&mut state.plc_baud_rate, rate, format!("{}", rate));
            }
        });

        if state.plc_port_connected {
            if ui.button(egui::RichText::new(T::disconnect(lang)).strong().color(egui::Color32::from_rgb(239, 68, 68))).clicked() {
                do_plc_disconnect(state);
            }
        } else {
            let can_connect = state.plc_selected_port.is_some();
            if ui.add_enabled(can_connect, egui::Button::new(egui::RichText::new(T::connect(lang)).strong())).clicked() {
                do_plc_connect(state);
            }
        }
    });

    // ── Row 2: PLC Protocol (Brand | Model | SlaveID | Interval | Timeout | Poll/Once | ?) ──
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let b = state.plc.selected_brand;
        egui::ComboBox::from_id_salt("plc_b").width(85.0).selected_text(b.label(lang)).show_ui(ui, |ui| {
            for &b in PlcBrand::all() {
                if ui.selectable_value(&mut state.plc.selected_brand, b, b.label(lang)).changed() {
                    state.plc.selected_model = None;
                    state.plc.selected_register = None;
                }
            }
        });

        let models = plc_presets::get_models(state.plc.selected_brand);
        if !models.is_empty() {
            let name = state.plc.selected_model.and_then(|i| models.get(i)).map(|m| m.model).unwrap_or(models[0].model);
            egui::ComboBox::from_id_salt("plc_m").width(80.0).selected_text(name).show_ui(ui, |ui| {
                for (i, m) in models.iter().enumerate() { ui.selectable_value(&mut state.plc.selected_model, Some(i), m.model); }
            });
        }

        ui.label(egui::RichText::new(T::plc_slave_id(lang)).weak().small());
        ui.add(egui::DragValue::new(&mut state.plc.slave_id).range(1..=247).prefix(" "));

        ui.label(egui::RichText::new("\u{21BB}").weak().small());
        ui.add(egui::DragValue::new(&mut state.plc.poll_interval_ms).range(100..=10000).suffix("ms"));

        ui.label(egui::RichText::new(T::plc_response_timeout(lang)).weak().small());
        ui.add(egui::DragValue::new(&mut state.plc.plc_response_timeout_ms).range(10..=5000).suffix("ms"));

        ui.separator();

        let read_label = if state.plc.polling { format!("\u{25A0} {}", T::stop_btn(lang)) } else { format!("\u{25B6} {}", T::poll_btn(lang)) };
        if ui.button(egui::RichText::new(read_label).strong()).clicked() && state.plc_port_connected {
            state.plc.polling = !state.plc.polling;
            if state.plc.polling { state.plc.last_poll_time = 0; }
        }
        if ui.button(format!("\u{21BB} {}", T::once_btn(lang))).clicked() && state.plc_port_connected {
            do_read_all(state);
        }

        ui.button(egui::RichText::new("?").size(14.0).strong())
            .on_hover_text(T::plc_tip_header(lang));
    });

    ui.add_space(4.0);

    // ── Custom brand: inline register editor ──
    if state.plc.selected_brand == PlcBrand::Custom {
        render_custom_register_editor(ui, state);
        ui.add_space(4.0);
    }

    // ── Register Table ──
    let regs = get_register_defs(state);
    if regs.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new(T::no_data(lang)).weak());
        });
    } else {
        let row_height = 22.0;
        let table_rows = regs.len();
        egui::ScrollArea::vertical().max_height((row_height * table_rows as f32 + 30.0).min(400.0)).show(ui, |ui| {
            egui::Grid::new("plc_grid").striped(true).spacing([8.0, 2.0]).show(ui, |ui| {
                // Header
                header_cell(ui, &T::plc_addr_label(lang));
                header_cell(ui, &T::plc_name_label(lang));
                header_cell(ui, &T::plc_type_label(lang));
                header_cell(ui, &T::plc_value_label(lang));
                header_cell(ui, &T::plc_unit_label(lang));
                ui.end_row();

                let now_ms = chrono::Utc::now().timestamp_millis();

                for (i, reg) in regs.iter().enumerate() {
                    let val = state.plc.register_values.get(&reg.addr).cloned();
                    let is_selected = state.plc.selected_register == Some(i);

                    // Address — tooltip shows decimal
                    ui.label(egui::RichText::new(format!("0x{:04X}", reg.addr)).monospace().size(12.0))
                        .on_hover_text(format!("{} ({})", reg.addr, reg.description));

                    // Name — tooltip shows description
                    ui.label(egui::RichText::new(&reg.name).size(12.0))
                        .on_hover_text(&reg.description);

                    // Type badge
                    let tc = match reg.data_type {
                        PlcDataType::Bool => egui::Color32::from_rgb(100, 180, 255),
                        PlcDataType::U16 | PlcDataType::I16 => egui::Color32::from_rgb(0, 200, 120),
                        PlcDataType::U32 => egui::Color32::from_rgb(200, 160, 0),
                        PlcDataType::Float32 => egui::Color32::from_rgb(200, 100, 200),
                    };
                    ui.label(egui::RichText::new(reg.data_type.label()).color(tc).size(11.0).monospace());

                    // Value — click to toggle edit mode
                    if is_selected {
                        // Inline write row with cancel
                        ui.horizontal(|ui| {
                            match reg.data_type {
                                PlcDataType::Bool => {
                                    let on = val.as_ref().map(|v| v.raw_u16 != 0).unwrap_or(false);
                                    let on_text = if on { T::plc_on(lang) } else { T::plc_off(lang) };
                                    if ui.small_button(on_text).clicked() {
                                        write_coil(state, reg, !on);
                                        state.plc.selected_register = None;
                                    }
                                }
                                _ => {
                                    let hint = match reg.data_type {
                                        PlcDataType::U16 | PlcDataType::I16 => "0-65535",
                                        PlcDataType::U32 => "0-4294967295",
                                        PlcDataType::Float32 => "25.0",
                                        _ => "value",
                                    };
                                    ui.add(egui::TextEdit::singleline(&mut state.plc.write_value).desired_width(80.0).hint_text(hint));
                                    if ui.small_button(T::plc_write(lang)).on_hover_text(T::plc_tip_register(lang)).clicked() && state.plc_port_connected {
                                        do_write_register(state);
                                        state.plc.selected_register = None;
                                    }
                                }
                            }
                            if ui.small_button(T::cancel_label(lang)).clicked() {
                                state.plc.selected_register = None;
                            }
                        });
                    } else {
                        // Display value — clickable to enter edit mode
                        let display = val.as_ref().map(|v| v.formatted.clone()).unwrap_or_else(|| "-".into());
                        let age_color = val.as_ref().map(|v| {
                            let age = now_ms - v.last_update;
                            if age < 3000 { egui::Color32::from_rgb(0, 200, 0) }
                            else if age < 10000 { egui::Color32::from_rgb(200, 180, 0) }
                            else { egui::Color32::from_rgb(180, 60, 60) }
                        }).unwrap_or(egui::Color32::GRAY);

                        let rt = egui::RichText::new(&display).monospace().size(12.0).color(age_color);
                        let tooltip = val.as_ref().map(|v| {
                            let raw_hex = v.raw_bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                            let age_s = (now_ms - v.last_update) as f64 / 1000.0;
                            format!("Raw: {}\nLast: {:.1}s ago", raw_hex, age_s)
                        }).unwrap_or_default();
                        if ui.selectable_label(false, rt).on_hover_text(tooltip).clicked() {
                            // Toggle: clicking same register deselects it
                            if state.plc.selected_register == Some(i) {
                                state.plc.selected_register = None;
                            } else {
                                state.plc.selected_register = Some(i);
                                state.plc.write_value.clear();
                            }
                        }
                    }

                    // Unit
                    ui.label(egui::RichText::new(&reg.unit).weak().size(11.0));

                    ui.end_row();
                }
            });
        });
    }

    // ── TX/RX Display (copyable) ──
    if !state.plc.plc_last_tx.is_empty() || !state.plc.plc_last_rx.is_empty() {
        ui.add_space(4.0);
        ui.separator();
        ui.label(egui::RichText::new("TX/RX").strong().small());
        if !state.plc.plc_last_tx.is_empty() {
            let tx_text = format!("TX: {}", state.plc.plc_last_tx);
            ui.add(egui::TextEdit::singleline(&mut tx_text.as_str()).font(egui::FontId::monospace(11.0)).interactive(false));
        }
        if !state.plc.plc_last_rx.is_empty() {
            let byte_count = state.plc.plc_last_rx.split_whitespace().count();
            let rx_text = format!("RX ({} bytes): {}", byte_count, state.plc.plc_last_rx);
            ui.add(egui::TextEdit::singleline(&mut rx_text.as_str()).font(egui::FontId::monospace(11.0)).interactive(false));
        }
    }

    // ── Log ──
    if !state.plc.plc_log.is_empty() {
        ui.add_space(2.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Log").strong().small());
            if ui.small_button(T::clear(lang)).clicked() {
                state.plc.plc_log.clear();
            }
        });
        egui::ScrollArea::vertical().max_height(100.0).stick_to_bottom(true).show(ui, |ui| {
            for line in state.plc.plc_log.iter().rev().take(5) {
                let color = if line.contains("ERR") || line.contains("error") {
                    egui::Color32::from_rgb(239, 68, 68)
                } else if line.contains("OK") || line.contains("=>") {
                    egui::Color32::from_rgb(34, 197, 94)
                } else {
                    egui::Color32::from_rgb(156, 163, 175)
                };
                ui.label(egui::RichText::new(line).weak().small().monospace().color(color));
            }
        });
    }

    // Auto-poll
    if state.plc.polling && state.plc_port_connected {
        let now = chrono::Utc::now().timestamp_millis();
        if now - state.plc.last_poll_time >= state.plc.poll_interval_ms as i64 {
            do_read_all(state);
            state.plc.last_poll_time = now;
        }
    }
}

// ============================================================================
// Custom Register Editor (BUG 1 FIX)
// ============================================================================

fn render_custom_register_editor(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    ui.group(|ui| {
        ui.label(egui::RichText::new(T::plc_custom_regs(lang)).strong());

        // Existing registers list
        let mut remove_idx = None;
        for (i, reg) in state.plc.custom_registers.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("0x{:04X}", reg.addr)).monospace().small());
                ui.label(egui::RichText::new(&reg.name).small());
                ui.label(egui::RichText::new(reg.data_type.label()).small().color(egui::Color32::GRAY));
                if !reg.unit.is_empty() {
                    ui.label(egui::RichText::new(&reg.unit).small().weak());
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("\u{2715}").on_hover_text(T::plc_delete(lang)).clicked() {
                        remove_idx = Some(i);
                    }
                });
            });
        }
        if let Some(idx) = remove_idx {
            state.plc.custom_registers.remove(idx);
        }

        // Add new register form
        if state.plc.adding_custom_register {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(T::plc_addr_label(lang)).small());
                ui.add(egui::TextEdit::singleline(&mut state.plc.new_reg_addr).desired_width(60.0).hint_text("0x0000"));
                ui.label(egui::RichText::new(T::plc_name_label(lang)).small());
                ui.add(egui::TextEdit::singleline(&mut state.plc.new_reg_name).desired_width(80.0).hint_text("Name"));
            });
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("new_reg_type").width(80.0).selected_text(state.plc.new_reg_type.label()).show_ui(ui, |ui| {
                    for dt in [PlcDataType::Bool, PlcDataType::U16, PlcDataType::I16, PlcDataType::U32, PlcDataType::Float32] {
                        ui.selectable_value(&mut state.plc.new_reg_type, dt, dt.label());
                    }
                });
                ui.label(egui::RichText::new("Scale").small());
                ui.add(egui::TextEdit::singleline(&mut state.plc.new_reg_scale).desired_width(50.0).hint_text("1.0"));
                ui.label(egui::RichText::new(T::plc_unit_label(lang)).small());
                ui.add(egui::TextEdit::singleline(&mut state.plc.new_reg_unit).desired_width(50.0));
            });
            ui.horizontal(|ui| {
                if ui.button(T::plc_add_register(lang)).clicked() {
                    if let Ok(addr) = parse_hex_addr(&state.plc.new_reg_addr) {
                        let scale = state.plc.new_reg_scale.parse::<f64>().unwrap_or(1.0);
                        state.plc.custom_registers.push(PlcRegisterDef {
                            addr,
                            name: state.plc.new_reg_name.clone(),
                            data_type: state.plc.new_reg_type,
                            scale_factor: scale,
                            unit: state.plc.new_reg_unit.clone(),
                            description: String::new(),
                        });
                        state.plc.new_reg_addr.clear();
                        state.plc.new_reg_name.clear();
                        state.plc.new_reg_scale = "1.0".into();
                        state.plc.new_reg_unit.clear();
                        state.plc.adding_custom_register = false;
                    }
                }
                if ui.button(T::plc_cancel(lang)).clicked() {
                    state.plc.adding_custom_register = false;
                }
            });
        } else {
            if ui.button(format!("+ {}", T::plc_add_register(lang))).clicked() {
                state.plc.adding_custom_register = true;
            }
        }
    });
}

fn parse_hex_addr(s: &str) -> Result<u16, ()> {
    let s = s.trim().trim_start_matches("0x").trim_start_matches("0X");
    u16::from_str_radix(s, 16).map_err(|_| ())
}

// ============================================================================
// UI Helpers
// ============================================================================

fn header_cell(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).strong().size(11.0));
}

// ============================================================================
// Async Results
// ============================================================================

fn poll_async_results(state: &mut AppState) {
    let lang = state.language;

    // Poll read results
    if let Some(rx) = &state.plc_async_receiver {
        if let Ok(result) = rx.try_recv() {
            state.plc_async_receiver = None;
            match result {
                Ok(results) => {
                    // Capture first response for PLC panel RX display
                    for (_, resp_result) in &results {
                        match resp_result {
                            Ok(resp) => {
                                state.plc.plc_last_rx = resp.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                                break;
                            }
                            Err(e) => {
                                if state.plc.plc_last_rx.is_empty() {
                                    state.plc.plc_last_rx = format!("ERR: {}", e);
                                }
                            }
                        }
                    }
                    let regs = get_register_defs(state);
                    for (addr, resp_result) in results {
                        match resp_result {
                            Ok(data) => {
                                // `data` is already extracted by do_read_all: [byte_count, reg_bytes...]
                                // Do NOT call ModbusFrame::parse() again — it's not a full Modbus frame
                                state.add_log_entry(crate::state::LogLevel::Info,
                                    &format!("[PLC] Reg 0x{:04X}: data {} bytes: {}", addr, data.len(),
                                        data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")));
                                if let Some(reg) = regs.iter().find(|r| r.addr == addr) {
                                    let formatted = format_value(reg, &data);
                                    let raw_bytes = data.get(1..).unwrap_or(&[]).to_vec();
                                    let raw_u16 = data.get(1..3).map(|d| u16::from_be_bytes([d[0], d[1]])).unwrap_or(0);
                                    state.add_log_entry(crate::state::LogLevel::Info,
                                        &format!("[PLC] Reg 0x{:04X} ({}): value='{}', raw={}", addr, reg.name, formatted, raw_u16));
                                    state.plc.register_values.insert(addr, PlcRegisterValue {
                                        raw_u16, formatted,
                                        last_update: chrono::Utc::now().timestamp_millis(),
                                        raw_bytes,
                                    });
                                } else {
                                    state.add_log_entry(crate::state::LogLevel::Warning,
                                        &format!("[PLC] Reg 0x{:04X}: no matching register definition", addr));
                                }
                            }
                            Err(e) => {
                                state.add_log_entry(crate::state::LogLevel::Error,
                                    &format!("[PLC] Reg 0x{:04X}: read error: {}", addr, e));
                                if let Some(_reg) = regs.iter().find(|r| r.addr == addr) {
                                    state.plc.register_values.insert(addr, PlcRegisterValue {
                                        raw_u16: 0,
                                        formatted: format!("ERR: {}", e),
                                        last_update: chrono::Utc::now().timestamp_millis() - 30000,
                                        raw_bytes: Vec::new(),
                                    });
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    state.plc.plc_last_rx = format!("ERR: {}", e);
                }
            }
        }
    }

    // Poll write results
    if let Some(ref rx) = state.plc_write_async {
        if let Ok(result) = rx.try_recv() {
            state.plc_write_async = None;
            match result {
                Ok(resp) => {
                    // Display write response as RX
                    let rx_hex = resp.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                    state.plc.plc_last_rx = rx_hex.clone();
                    state.add_terminal_line_tagged(crate::state::Direction::Rx, rx_hex.clone(), true, "PLC");
                    plc_log(state, &format!("Write response: {} bytes", resp.len()));
                }
                Err(e) => {
                    state.plc.plc_last_rx = format!("ERR: {}", e);
                    plc_log(state, &format!("{}: {}", T::plc_write_error(lang), e));
                }
            }
        }
    }

    // Poll raw Modbus responses for terminal display
    if let Some(ref rx) = state.plc.plc_raw_response_rx {
        let raw_resps: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
        for resp in raw_resps {
            let rx_hex = resp.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
            state.add_terminal_line_tagged(crate::state::Direction::Rx, rx_hex.clone(), true, "PLC");
        }
    }
}

// ============================================================================
// Value Formatting
// ============================================================================

fn format_value(reg: &PlcRegisterDef, data: &[u8]) -> String {
    match reg.data_type {
        PlcDataType::Bool => {
            let raw = data.get(1).copied().unwrap_or(0);
            if raw != 0 { "ON".into() } else { "OFF".into() }
        }
        PlcDataType::U16 => {
            let raw = data.get(1..3).map(|d| u16::from_be_bytes([d[0], d[1]])).unwrap_or(0);
            let scaled = raw as f64 * reg.scale_factor;
            if reg.scale_factor != 1.0 { format!("{:.2}", scaled) } else { format!("{}", raw) }
        }
        PlcDataType::I16 => {
            let raw = data.get(1..3).map(|d| u16::from_be_bytes([d[0], d[1]])).unwrap_or(0) as i16;
            let scaled = raw as f64 * reg.scale_factor;
            if reg.scale_factor != 1.0 { format!("{:.2}", scaled) } else { format!("{}", raw) }
        }
        PlcDataType::U32 => {
            let raw = data.get(1..5).map(|d| u32::from_be_bytes([d[0], d[1], d[2], d[3]])).unwrap_or(0);
            let scaled = raw as f64 * reg.scale_factor;
            if reg.scale_factor != 1.0 { format!("{:.2}", scaled) } else { format!("{}", raw) }
        }
        PlcDataType::Float32 => {
            let raw = data.get(1..5).map(|d| u32::from_be_bytes([d[0], d[1], d[2], d[3]])).unwrap_or(0);
            let f = f32::from_bits(raw);
            let scaled = f as f64 * reg.scale_factor;
            if reg.scale_factor != 1.0 { format!("{:.3}", scaled) } else { format!("{:.3}", f) }
        }
    }
}

// ============================================================================
// Register Definitions
// ============================================================================

fn get_register_defs(state: &AppState) -> Vec<PlcRegisterDef> {
    if state.plc.selected_brand == PlcBrand::Custom {
        state.plc.custom_registers.clone()
    } else {
        let models = plc_presets::get_models(state.plc.selected_brand);
        let idx = state.plc.selected_model.unwrap_or(0);
        models.get(idx)
            .map(|m| m.registers.clone())
            .unwrap_or_default()
    }
}

// ============================================================================
// Logging
// ============================================================================

fn plc_log(state: &mut AppState, msg: &str) {
    state.plc.plc_log.push_back(format!("{} {}", chrono::Local::now().format("%H:%M:%S"), msg));
    if state.plc.plc_log.len() > 500 { state.plc.plc_log.pop_front(); }
    state.add_log_entry(crate::state::LogLevel::Info, &format!("[PLC] {}", msg));
}

// ============================================================================
// Read All (batched)
// ============================================================================

fn do_read_all(state: &mut AppState) {
    if state.plc_async_receiver.is_some() { return; }

    let regs = get_register_defs(state);
    if regs.is_empty() { return; }

    let slave_id = state.plc.slave_id;
    let timeout_ms = state.plc.plc_response_timeout_ms;
    let po = state.plc_port_owner.as_ref().map(|p| p.cmd_tx());

    let batches = build_read_batches(&regs);

    // Capture TX hex for display (show first batch request)
    if let Some(first_batch) = batches.first() {
        let frame = ModbusParser::build_read_request(slave_id, serialrun_core::protocol::ModbusFunction::ReadHoldingRegisters, first_batch.start_addr, first_batch.quantity);
        let tx_hex = frame.to_bytes().iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
        state.plc.plc_last_tx = if batches.len() > 1 {
            format!("{} (+{} more)", tx_hex, batches.len() - 1)
        } else {
            tx_hex.clone()
        };
        state.add_terminal_line_tagged(crate::state::Direction::Tx, tx_hex, true, "PLC");
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let (raw_tx, raw_rx) = std::sync::mpsc::channel::<Vec<u8>>();
    state.plc_async_receiver = Some(rx);
    state.plc.plc_raw_response_rx = Some(raw_rx);

    std::thread::spawn(move || {
        let po = match po {
            Some(p) => p,
            None => { let _ = tx.send(Err("Not connected".into())); return; }
        };

        let mut all_results: Vec<(u16, Result<Vec<u8>, String>)> = Vec::new();

        for batch in &batches {
            let frame = ModbusParser::build_read_request(
                slave_id,
                serialrun_core::protocol::ModbusFunction::ReadHoldingRegisters,
                batch.start_addr,
                batch.quantity,
            );
            let req = frame.to_bytes();
            log::info!("[PLC] Batch: addr=0x{:04X}, qty={}, regs={}, TX: {}",
                batch.start_addr, batch.quantity, batch.regs.len(),
                req.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "));
            let (resp_tx, resp_rx) = std::sync::mpsc::channel();
            let _ = po.send(crate::port_owner::PortCommand::ReadExclusive { data: req, timeout_ms, resp_tx });
            let result = resp_rx.recv().unwrap_or_else(|e| Err(format!("Channel closed: {}", e)));
            match result {
                Ok(resp) => {
                    // Send full response to terminal
                    let _ = raw_tx.send(resp.clone());
                    log::info!("[PLC] Batch 0x{:04X}: RX {} bytes: {}",
                        batch.start_addr, resp.len(),
                        resp.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "));
                    if resp.len() < 4 {
                        log::error!("[PLC] Batch 0x{:04X}: response too short ({} bytes)", batch.start_addr, resp.len());
                        for reg in &batch.regs {
                            all_results.push((reg.addr, Err(format!("Response too short: {} bytes", resp.len()))));
                        }
                    } else if let Ok(f) = ModbusFrame::parse(&resp) {
                        log::info!("[PLC] Batch 0x{:04X}: parse OK, data={} bytes", batch.start_addr, f.data.len());
                        for reg in &batch.regs {
                            let offset = (reg.addr - batch.start_addr) as usize;
                            let bytes_per_reg = 2;
                            let byte_offset = 1 + offset * bytes_per_reg;
                            let needed: Vec<u8> = match reg.data_type {
                                PlcDataType::U32 | PlcDataType::Float32 => {
                                    let end = (byte_offset + 4).min(f.data.len());
                                    std::iter::once(f.data[0])
                                        .chain(f.data[byte_offset..end].iter().copied())
                                        .collect()
                                }
                                _ => {
                                    let end = (byte_offset + 2).min(f.data.len());
                                    std::iter::once(f.data[0])
                                        .chain(f.data[byte_offset..end].iter().copied())
                                        .collect()
                                }
                            };
                            log::info!("[PLC] Reg 0x{:04X}: offset={}, byte_offset={}, needed={} bytes: {}",
                                reg.addr, offset, byte_offset, needed.len(),
                                needed.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "));
                            all_results.push((reg.addr, Ok(needed)));
                        }
                    } else {
                        log::error!("[PLC] Batch 0x{:04X}: ModbusFrame::parse FAILED", batch.start_addr);
                        for reg in &batch.regs {
                            all_results.push((reg.addr, Err("Parse error".into())));
                        }
                    }
                }
                _ => {
                    for reg in &batch.regs {
                        all_results.push((reg.addr, Err("No response".into())));
                    }
                }
            }
        }

        let _ = tx.send(Ok(all_results));
    });
}

struct ReadBatch {
    start_addr: u16,
    quantity: u16,
    regs: Vec<PlcRegisterDef>,
}

fn build_read_batches(regs: &[PlcRegisterDef]) -> Vec<ReadBatch> {
    if regs.is_empty() { return vec![]; }

    let mut sorted = regs.to_vec();
    sorted.sort_by_key(|r| r.addr);

    let mut batches = Vec::new();
    let mut current = ReadBatch {
        start_addr: sorted[0].addr,
        quantity: match sorted[0].data_type {
            PlcDataType::U32 | PlcDataType::Float32 => 2,
            _ => 1,
        },
        regs: vec![sorted[0].clone()],
    };

    for reg in sorted.iter().skip(1) {
        let prev_end = current.start_addr + current.quantity;
        let needed = match reg.data_type {
            PlcDataType::U32 | PlcDataType::Float32 => 2,
            _ => 1,
        };

        // Merge if contiguous or has small gap (<=2 addresses)
        if reg.addr <= prev_end + 2 {
            let new_end = reg.addr + needed;
            current.quantity = new_end - current.start_addr;
            current.regs.push(reg.clone());
        } else {
            batches.push(current);
            current = ReadBatch {
                start_addr: reg.addr,
                quantity: needed,
                regs: vec![reg.clone()],
            };
        }
    }
    batches.push(current);
    batches
}

// ============================================================================
// Write Register
// ============================================================================

fn do_write_register(state: &mut AppState) {
    let lang = state.language;
    if state.plc_write_async.is_some() { return; }
    let Some(idx) = state.plc.selected_register else { return };
    let regs = get_register_defs(state);
    let Some(reg) = regs.get(idx) else { return };

    let frame_bytes = match reg.data_type {
        PlcDataType::Bool => {
            let on = state.plc.write_value.trim() == "1"
                || state.plc.write_value.trim().eq_ignore_ascii_case("on")
                || state.plc.write_value.trim().eq_ignore_ascii_case("true");
            let data = if on {
                vec![(reg.addr >> 8) as u8, reg.addr as u8, 0xFF, 0x00]
            } else {
                vec![(reg.addr >> 8) as u8, reg.addr as u8, 0x00, 0x00]
            };
            ModbusFrame::new(state.plc.slave_id, serialrun_core::protocol::ModbusFunction::WriteSingleCoil, data).to_bytes()
        }
        PlcDataType::U16 | PlcDataType::I16 => {
            let user_val: f64 = match state.plc.write_value.trim().parse() {
                Ok(v) => v,
                Err(_) => { plc_log(state, &format!("{}: {}", T::plc_invalid_value(lang), reg.name)); return; }
            };
            let raw = if reg.scale_factor != 1.0 { (user_val / reg.scale_factor).round() as u16 } else { user_val as u16 };
            let data = vec![(reg.addr >> 8) as u8, reg.addr as u8, (raw >> 8) as u8, raw as u8];
            ModbusFrame::new(state.plc.slave_id, serialrun_core::protocol::ModbusFunction::WriteSingleRegister, data).to_bytes()
        }
        PlcDataType::U32 => {
            let user_val: f64 = match state.plc.write_value.trim().parse() {
                Ok(v) => v,
                Err(_) => { plc_log(state, &format!("{}: {}", T::plc_invalid_value(lang), reg.name)); return; }
            };
            let raw = if reg.scale_factor != 1.0 { (user_val / reg.scale_factor).round() as u32 } else { user_val as u32 };
            let bytes = raw.to_be_bytes();
            // FC16: [start_addr(2)] [quantity(2)] [byte_count(1)] [data(N)]
            let data = vec![
                (reg.addr >> 8) as u8, reg.addr as u8,
                0x00, 0x02,  // quantity = 2 registers
                0x04,        // byte_count = 4 bytes
                bytes[0], bytes[1], bytes[2], bytes[3],
            ];
            ModbusFrame::new(state.plc.slave_id, serialrun_core::protocol::ModbusFunction::WriteMultipleRegisters, data).to_bytes()
        }
        PlcDataType::Float32 => {
            let user_val: f64 = match state.plc.write_value.trim().parse() {
                Ok(v) => v,
                Err(_) => { plc_log(state, &format!("{}: {}", T::plc_invalid_value(lang), reg.name)); return; }
            };
            let raw_f = if reg.scale_factor != 1.0 { user_val / reg.scale_factor } else { user_val };
            let bits = (raw_f as f32).to_bits();
            let bytes = bits.to_be_bytes();
            // FC16: [start_addr(2)] [quantity(2)] [byte_count(1)] [data(N)]
            let data = vec![
                (reg.addr >> 8) as u8, reg.addr as u8,
                0x00, 0x02,  // quantity = 2 registers
                0x04,        // byte_count = 4 bytes
                bytes[0], bytes[1], bytes[2], bytes[3],
            ];
            ModbusFrame::new(state.plc.slave_id, serialrun_core::protocol::ModbusFunction::WriteMultipleRegisters, data).to_bytes()
        }
    };

    state.add_terminal_line_tagged(crate::state::Direction::Tx, crate::ui::terminal::format_hex_bytes(&frame_bytes), true, "PLC");
    if let Some(po) = state.plc_port_owner.as_ref().map(|p| p.cmd_tx()) {
        let timeout_ms = state.plc.plc_response_timeout_ms;
        let (resp_tx, resp_rx): (std::sync::mpsc::Sender<Result<Vec<u8>, String>>, _) = std::sync::mpsc::channel();
        state.plc_write_async = Some(resp_rx);
        std::thread::spawn(move || {
            let _ = po.send(crate::port_owner::PortCommand::ReadExclusive { data: frame_bytes, timeout_ms, resp_tx });
        });
    }
    plc_log(state, &format!("W {} (0x{:04X})", reg.name, reg.addr));
}

// ============================================================================
// Write Coil
// ============================================================================

fn write_coil(state: &mut AppState, reg: &PlcRegisterDef, on: bool) {
    let data = if on {
        vec![(reg.addr >> 8) as u8, reg.addr as u8, 0xFF, 0x00]
    } else {
        vec![(reg.addr >> 8) as u8, reg.addr as u8, 0x00, 0x00]
    };
    let frame = ModbusFrame::new(state.plc.slave_id, serialrun_core::protocol::ModbusFunction::WriteSingleCoil, data);
    let frame_bytes = frame.to_bytes();
    state.add_terminal_line_tagged(crate::state::Direction::Tx, crate::ui::terminal::format_hex_bytes(&frame_bytes), true, "PLC");
    if let Some(po) = state.plc_port_owner.as_ref().map(|p| p.cmd_tx()) {
        let timeout_ms = state.plc.plc_response_timeout_ms;
        let (resp_tx, resp_rx): (std::sync::mpsc::Sender<Result<Vec<u8>, String>>, _) = std::sync::mpsc::channel();
        state.plc_write_async = Some(resp_rx);
        std::thread::spawn(move || {
            let _ = po.send(crate::port_owner::PortCommand::ReadExclusive { data: frame_bytes, timeout_ms, resp_tx });
        });
    }
    let lang = state.language;
    plc_log(state, &format!("{} => {}", reg.name, if on { T::plc_on(lang) } else { T::plc_off(lang) }));
}

// ============================================================================
// PLC Independent Port Connection
// ============================================================================

fn refresh_plc_ports(state: &mut AppState) {
    state.plc_port_list = serialrun_core::SerialPort::list_ports().unwrap_or_default();
}

fn do_plc_connect(state: &mut AppState) {
    let Some(port_name) = state.plc_selected_port.clone() else { return };
    let lang = state.language;

    let po = crate::port_owner::PortOwnerHandle::start();
    let config = serialrun_core::config::SerialConfig {
        port_name: port_name.clone(),
        baud_rate: state.plc_baud_rate,
        ..Default::default()
    };
    po.send(crate::port_owner::PortCommand::Open(config));
    state.plc_port_owner = Some(po);
    state.plc_port_connected = true;
    state.add_log_entry(crate::state::LogLevel::Info,
        &format!("[PLC] Connected to {} @ {} baud", port_name, state.plc_baud_rate));
    plc_log(state, &format!("{} {} @ {}", T::connected(lang), port_name, state.plc_baud_rate));
}

fn do_plc_disconnect(state: &mut AppState) {
    state.plc.polling = false;
    if let Some(po) = state.plc_port_owner.take() {
        po.send(crate::port_owner::PortCommand::Close);
        drop(po);
    }
    state.plc_port_connected = false;
    state.add_log_entry(crate::state::LogLevel::Info, "[PLC] Disconnected");
    let lang = state.language;
    plc_log(state, &format!("{}", T::disconnected(lang)));
}

fn poll_plc_port_events(state: &mut AppState) {
    // Collect events first to avoid borrow conflict
    let events: Vec<_> = if let Some(ref po) = state.plc_port_owner {
        std::iter::from_fn(|| po.poll()).collect()
    } else {
        return;
    };
    for evt in events {
        match evt {
            crate::port_owner::PortEvent::Opened(ok, msg) => {
                if ok {
                    state.plc_port_connected = true;
                    state.add_log_entry(crate::state::LogLevel::Info, &format!("[PLC] Port opened: {}", msg));
                } else {
                    state.plc_port_connected = false;
                    state.add_log_entry(crate::state::LogLevel::Error, &format!("[PLC] Port open failed: {}", msg));
                }
            }
            crate::port_owner::PortEvent::Closed => {
                state.plc_port_connected = false;
                state.plc.polling = false;
                state.add_log_entry(crate::state::LogLevel::Info, "[PLC] Port closed");
            }
            crate::port_owner::PortEvent::Error(e) => {
                state.add_log_entry(crate::state::LogLevel::Error, &format!("[PLC] Port error: {}", e));
            }
            _ => {}
        }
    }
}
