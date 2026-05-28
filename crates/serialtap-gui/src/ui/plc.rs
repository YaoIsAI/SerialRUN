use crate::plc_presets;
use crate::state::{AppState, PlcBrand, PlcDataType, PlcRegisterDef, PlcRegisterValue, T};
use eframe::egui;
use serialtap_core::protocol::{ModbusFrame, ModbusParser};

pub fn render_plc_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    // ── Section 1: Connection & Device ──
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("PLC Control").strong().size(14.0));
        ui.separator();
        // Status LED
        let (color, status_text) = if state.is_connected {
            (egui::Color32::from_rgb(0, 200, 0), T::connected(lang))
        } else {
            (egui::Color32::from_rgb(180, 60, 60), T::disconnected(lang))
        };
        ui.label(egui::RichText::new("●").size(12.0).color(color));
        ui.label(egui::RichText::new(status_text).weak().small());
    });
    ui.add_space(2.0);

    // Device config grid
    egui::Grid::new("plc_device").num_columns(4).show(ui, |ui| {
        ui.label(T::plc_brand(lang));
        let b = state.plc.selected_brand;
        egui::ComboBox::from_id_salt("plc_b").width(100.0).selected_text(b.label(lang)).show_ui(ui, |ui| {
            for &b in PlcBrand::all() { ui.selectable_value(&mut state.plc.selected_brand, b, b.label(lang)); }
        });
        ui.label(T::plc_model(lang));
        let models = plc_presets::get_models(state.plc.selected_brand);
        if models.is_empty() { ui.label("-"); } else {
            let name = state.plc.selected_model.and_then(|i| models.get(i)).map(|m| m.model).unwrap_or(models[0].model);
            egui::ComboBox::from_id_salt("plc_m").width(100.0).selected_text(name).show_ui(ui, |ui| {
                for (i, m) in models.iter().enumerate() { ui.selectable_value(&mut state.plc.selected_model, Some(i), m.model); }
            });
        }
        ui.end_row();
        ui.label(T::slave_id(lang));
        ui.add(egui::DragValue::new(&mut state.plc.slave_id).range(1..=247).prefix("ID "));
        ui.label(T::poll_interval(lang));
        ui.add(egui::DragValue::new(&mut state.plc.poll_interval_ms).range(200..=10000).suffix(" ms"));
        ui.end_row();
    });
    ui.add_space(2.0);

    // ── Section 2: Control Bar ──
    ui.horizontal(|ui| {
        let read_label = if state.plc.polling { T::stop_monitor(lang) } else { T::read_all(lang) };
        let btn = ui.button(egui::RichText::new(read_label).strong());
        if btn.clicked() && state.is_connected {
            if state.plc.polling {
                state.plc.polling = false;
                plc_log(state, "Auto-poll stopped");
            } else {
                state.plc.polling = true;
                state.plc.last_poll_time = 0;
                plc_log(state, "Auto-poll started");
            }
        }
        if ui.button("Read Once").clicked() && state.is_connected {
            do_read_all(state);
        }
        ui.separator();
        // Write selected register
        let can_write = state.plc.selected_register.is_some();
        if can_write {
            ui.label("Write:");
            ui.add(egui::TextEdit::singleline(&mut state.plc.write_value).desired_width(80.0).hint_text("value"));
            if ui.button("Write").clicked() && state.is_connected {
                do_write_register(state);
            }
        }
    });
    ui.add_space(2.0);

    // ── Section 3: Register Table ──
    let regs = get_register_defs(state);
    if regs.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new(T::no_data(lang)).weak());
        });
    } else {
        egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
            egui::Grid::new("plc_grid").striped(true).min_col_width(40.0).show(ui, |ui| {
                // Header
                ui.label(egui::RichText::new("#").strong().small());
                ui.label(egui::RichText::new(T::address(lang)).strong().small());
                ui.label(egui::RichText::new(T::name(lang)).strong().small());
                ui.label(egui::RichText::new("Type").strong().small());
                ui.label(egui::RichText::new(T::value(lang)).strong().small());
                ui.label(egui::RichText::new(T::unit_label(lang)).strong().small());
                ui.label(egui::RichText::new("Age").strong().small());
                ui.label(egui::RichText::new("Status").strong().small());
                ui.end_row();

                for (i, reg) in regs.iter().enumerate() {
                    let val = state.plc.register_values.get(&reg.addr).cloned();
                    let is_selected = state.plc.selected_register == Some(i);

                    // Row number (clickable to select)
                    if ui.selectable_label(is_selected, format!("{}", i + 1)).clicked() {
                        state.plc.selected_register = Some(i);
                        state.plc.write_value.clear();
                    }

                    // Address
                    ui.label(egui::RichText::new(format!("0x{:04X}", reg.addr)).monospace().small());

                    // Name
                    ui.label(egui::RichText::new(&reg.name).small());

                    // Type badge
                    let type_color = match reg.data_type {
                        PlcDataType::Bool => egui::Color32::from_rgb(100, 180, 255),
                        PlcDataType::U16 | PlcDataType::I16 => egui::Color32::from_rgb(0, 200, 120),
                        PlcDataType::U32 => egui::Color32::from_rgb(200, 160, 0),
                        PlcDataType::Float32 => egui::Color32::from_rgb(200, 100, 200),
                    };
                    ui.label(egui::RichText::new(reg.data_type.label()).color(type_color).small().monospace());

                    // Value (with type-specific display)
                    match reg.data_type {
                        PlcDataType::Bool => {
                            let mut on = val.as_ref().map(|v| v.raw_u16 != 0).unwrap_or(false);
                            let on_text = if on { "ON" } else { "OFF" };
                            if ui.checkbox(&mut on, on_text).changed() {
                                write_coil(state, reg, on);
                            }
                        }
                        _ => {
                            let display = val.as_ref().map(|v| v.formatted.clone()).unwrap_or_else(|| "-".into());
                            ui.label(egui::RichText::new(&display).monospace().small());
                        }
                    }

                    // Unit
                    ui.label(egui::RichText::new(&reg.unit).weak().small());

                    // Age
                    if let Some(ref v) = val {
                        let age_ms = chrono::Utc::now().timestamp_millis() - v.last_update;
                        let age_text = if age_ms < 1000 { format!("{}ms", age_ms) }
                            else if age_ms < 60000 { format!("{}s", age_ms / 1000) }
                            else { format!("{}m", age_ms / 60000) };
                        let age_color = if age_ms < 3000 { egui::Color32::from_rgb(0, 180, 0) }
                            else if age_ms < 10000 { egui::Color32::from_rgb(180, 180, 0) }
                            else { egui::Color32::from_rgb(180, 60, 60) };
                        ui.label(egui::RichText::new(&age_text).color(age_color).small());
                    } else {
                        ui.label(egui::RichText::new("-").weak().small());
                    }

                    // Status
                    let (status_icon, status_color) = if let Some(ref v) = val {
                        let age = chrono::Utc::now().timestamp_millis() - v.last_update;
                        if age < 3000 { ("●", egui::Color32::from_rgb(0, 200, 0)) }
                        else if age < 10000 { ("●", egui::Color32::from_rgb(200, 180, 0)) }
                        else { ("●", egui::Color32::from_rgb(180, 60, 60)) }
                    } else { ("○", egui::Color32::GRAY) };
                    ui.label(egui::RichText::new(status_icon).color(status_color));
                    ui.end_row();
                }
            });
        });
    }

    // ── Section 4: Log ──
    if !state.plc.plc_log.is_empty() {
        ui.add_space(2.0);
        ui.separator();
        egui::ScrollArea::vertical().max_height(60.0).stick_to_bottom(true).show(ui, |ui| {
            for line in state.plc.plc_log.iter().rev().take(5) {
                ui.label(egui::RichText::new(line).weak().small().monospace());
            }
        });
    }

    // ── Auto-poll ──
    if state.plc.polling && state.is_connected {
        let now = chrono::Utc::now().timestamp_millis();
        if now - state.plc.last_poll_time >= state.plc.poll_interval_ms as i64 {
            do_read_all(state);
            state.plc.last_poll_time = now;
        }
    }
}

fn get_register_defs(state: &AppState) -> Vec<PlcRegisterDef> {
    if state.plc.selected_brand == PlcBrand::Custom {
        state.plc.custom_registers.clone()
    } else {
        let models = plc_presets::get_models(state.plc.selected_brand);
        models.get(state.plc.selected_model.unwrap_or(0))
            .map(|m| m.registers.clone())
            .unwrap_or_default()
    }
}

fn plc_log(state: &mut AppState, msg: &str) {
    state.plc.plc_log.push_back(format!("{} {}", chrono::Utc::now().format("%H:%M:%S"), msg));
    if state.plc.plc_log.len() > 50 { state.plc.plc_log.pop_front(); }
}

fn do_read_all(state: &mut AppState) {
    let regs = get_register_defs(state);
    for reg in &regs {
        let qty = match reg.data_type {
            PlcDataType::U32 | PlcDataType::Float32 => 2,
            _ => 1,
        };
        let frame = ModbusParser::build_read_request(
            state.plc.slave_id,
            serialtap_core::protocol::ModbusFunction::ReadHoldingRegisters,
            reg.addr,
            qty,
        );
        let req = frame.to_bytes();
        let mut buf = [0u8; 256];
        let result = (|| -> Result<Vec<u8>, String> {
            let p = state.port.as_mut().ok_or("Not connected")?;
            p.write(&req).map_err(|e| e.to_string())?;
            std::thread::sleep(std::time::Duration::from_millis(50));
            let n = p.read(&mut buf).map_err(|e| e.to_string())?;
            if n < 4 { return Err("No response".into()); }
            Ok(buf[..n].to_vec())
        })();
        if let Ok(resp) = result {
            if let Ok(f) = ModbusFrame::parse(&resp) {
                let data = &f.data;
                let formatted = match reg.data_type {
                    PlcDataType::Bool => {
                        let raw = data.get(1).copied().unwrap_or(0);
                        let on = raw != 0;
                        if on { "ON".into() } else { "OFF".into() }
                    }
                    PlcDataType::U16 => {
                        let raw = data.get(1..3).map(|d| u16::from_be_bytes([d[0], d[1]])).unwrap_or(0);
                        if reg.scale_factor != 1.0 { format!("{:.2}", raw as f64 * reg.scale_factor) } else { format!("{}", raw) }
                    }
                    PlcDataType::I16 => {
                        let raw = data.get(1..3).map(|d| u16::from_be_bytes([d[0], d[1]])).unwrap_or(0) as i16;
                        if reg.scale_factor != 1.0 { format!("{:.2}", raw as f64 * reg.scale_factor) } else { format!("{}", raw) }
                    }
                    PlcDataType::U32 => {
                        let raw = data.get(1..5).map(|d| u32::from_be_bytes([d[0], d[1], d[2], d[3]])).unwrap_or(0);
                        if reg.scale_factor != 1.0 { format!("{:.2}", raw as f64 * reg.scale_factor) } else { format!("{}", raw) }
                    }
                    PlcDataType::Float32 => {
                        let raw = data.get(1..5).map(|d| u32::from_be_bytes([d[0], d[1], d[2], d[3]])).unwrap_or(0);
                        let f = f32::from_bits(raw);
                        if reg.scale_factor != 1.0 { format!("{:.3}", f as f64 * reg.scale_factor) } else { format!("{:.3}", f) }
                    }
                };
                let raw_bytes = data.get(1..).unwrap_or(&[]).to_vec();
                state.plc.register_values.insert(reg.addr, PlcRegisterValue {
                    raw_u16: data.get(1..3).map(|d| u16::from_be_bytes([d[0], d[1]])).unwrap_or(0),
                    formatted,
                    last_update: chrono::Utc::now().timestamp_millis(),
                    raw_bytes,
                });
            }
        }
    }
}

fn do_write_register(state: &mut AppState) {
    let Some(idx) = state.plc.selected_register else { return };
    let regs = get_register_defs(state);
    let Some(reg) = regs.get(idx) else { return };

    match reg.data_type {
        PlcDataType::Bool => {
            let on = state.plc.write_value.trim() == "1" || state.plc.write_value.trim().eq_ignore_ascii_case("on") || state.plc.write_value.trim().eq_ignore_ascii_case("true");
            write_coil(state, reg, on);
        }
        PlcDataType::U16 | PlcDataType::I16 => {
            let val: u16 = match state.plc.write_value.trim().parse() {
                Ok(v) => v,
                Err(_) => { plc_log(state, &format!("Invalid value for {}", reg.name)); return; }
            };
            let data = vec![(reg.addr >> 8) as u8, reg.addr as u8, (val >> 8) as u8, val as u8];
            let frame = ModbusFrame::new(state.plc.slave_id, serialtap_core::protocol::ModbusFunction::WriteSingleRegister, data);
            if let Some(ref mut port) = state.port {
                let _ = port.write(&frame.to_bytes());
                plc_log(state, &format!("Wrote {} = {} to 0x{:04X}", reg.name, val, reg.addr));
            }
        }
        PlcDataType::U32 => {
            let val: u32 = match state.plc.write_value.trim().parse() {
                Ok(v) => v,
                Err(_) => { plc_log(state, &format!("Invalid value for {}", reg.name)); return; }
            };
            let bytes = val.to_be_bytes();
            let data = vec![
                (reg.addr >> 8) as u8, reg.addr as u8,
                bytes[0], bytes[1],
                ((reg.addr + 1) >> 8) as u8, (reg.addr + 1) as u8,
                bytes[2], bytes[3],
            ];
            let frame = ModbusFrame::new(state.plc.slave_id, serialtap_core::protocol::ModbusFunction::WriteMultipleRegisters, data);
            if let Some(ref mut port) = state.port {
                let _ = port.write(&frame.to_bytes());
                plc_log(state, &format!("Wrote {} = {} to 0x{:04X}", reg.name, val, reg.addr));
            }
        }
        PlcDataType::Float32 => {
            let val: f32 = match state.plc.write_value.trim().parse() {
                Ok(v) => v,
                Err(_) => { plc_log(state, &format!("Invalid value for {}", reg.name)); return; }
            };
            let bits = val.to_bits();
            let bytes = bits.to_be_bytes();
            let data = vec![
                (reg.addr >> 8) as u8, reg.addr as u8,
                bytes[0], bytes[1],
                ((reg.addr + 1) >> 8) as u8, (reg.addr + 1) as u8,
                bytes[2], bytes[3],
            ];
            let frame = ModbusFrame::new(state.plc.slave_id, serialtap_core::protocol::ModbusFunction::WriteMultipleRegisters, data);
            if let Some(ref mut port) = state.port {
                let _ = port.write(&frame.to_bytes());
                plc_log(state, &format!("Wrote {} = {} to 0x{:04X}", reg.name, val, reg.addr));
            }
        }
    }
}

fn write_coil(state: &mut AppState, reg: &PlcRegisterDef, on: bool) {
    let data = if on {
        vec![(reg.addr >> 8) as u8, reg.addr as u8, 0xFF, 0x00]
    } else {
        vec![(reg.addr >> 8) as u8, reg.addr as u8, 0x00, 0x00]
    };
    let frame = ModbusFrame::new(state.plc.slave_id, serialtap_core::protocol::ModbusFunction::WriteSingleCoil, data);
    if let Some(ref mut port) = state.port {
        let _ = port.write(&frame.to_bytes());
        plc_log(state, &format!("Coil {} => {}", reg.name, if on { "ON" } else { "OFF" }));
    }
}
