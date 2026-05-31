use crate::state::{AppState, SimMode, T};
use eframe::egui;

pub fn render_simulator_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    poll_sim_logs(state);

    // ── Connection Config ──
    ui.collapsing(T::serial_config(lang), |ui| {
        egui::Grid::new("sim_config").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
            ui.label(T::sim_mode(lang));
            egui::ComboBox::from_id_salt("sim_mode").width(120.0)
                .selected_text(state.simulator.mode.label(lang))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.simulator.mode, SimMode::TcpServer, SimMode::TcpServer.label(lang));
                    ui.selectable_value(&mut state.simulator.mode, SimMode::RtuSlave, SimMode::RtuSlave.label(lang));
                });
            ui.end_row();

            ui.label(T::tcp_port(lang));
            ui.add(egui::DragValue::new(&mut state.simulator.tcp_port).range(1..=65535));
            ui.end_row();

            if state.simulator.mode == SimMode::RtuSlave {
                ui.label(T::serial_port(lang));
                egui::ComboBox::from_id_salt("sim_port").width(120.0)
                    .selected_text(if state.simulator.serial_port_name.is_empty() { "--" } else { &state.simulator.serial_port_name })
                    .show_ui(ui, |ui| {
                        for p in &state.ports {
                            ui.selectable_value(&mut state.simulator.serial_port_name, p.name.clone(), &p.name);
                        }
                    });
                ui.end_row();

                ui.label(T::baud_rate(lang));
                ui.add(egui::DragValue::new(&mut state.simulator.baud_rate).range(300..=4000000));
                ui.end_row();
            }

            ui.label(T::slave_id(lang));
            ui.add(egui::DragValue::new(&mut state.simulator.slave_id).range(0..=247));
            ui.end_row();
        });
    });

    ui.add_space(4.0);

    // ── Control ──
    ui.horizontal(|ui| {
        if state.simulator.running {
            if ui.button(T::stop_sim(lang)).clicked() {
                if let Some(stop) = state.sim_stop.take() {
                    stop.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                state.simulator.running = false;
            }
            ui.label(egui::RichText::new(T::mcp_running(lang)).color(super::status::LOGO_GREEN).strong());
        } else {
            if ui.button(T::start_sim(lang)).clicked() {
                let cfg = serialrun_core::protocol::SimulatorConfig {
                    mode: match state.simulator.mode {
                        SimMode::TcpServer => serialrun_core::protocol::SimulatorMode::TcpServer,
                        SimMode::RtuSlave => serialrun_core::protocol::SimulatorMode::RtuSlave,
                    },
                    tcp_port: state.simulator.tcp_port,
                    serial_port_name: state.simulator.serial_port_name.clone(),
                    baud_rate: state.simulator.baud_rate,
                    slave_id: state.simulator.slave_id,
                    holding_registers: state.simulator.holding_registers.clone(),
                    input_registers: state.simulator.input_registers.clone(),
                    coils: state.simulator.coils.clone(),
                    discrete_inputs: state.simulator.discrete_inputs.clone(),
                };
                match serialrun_core::protocol::start_simulator(cfg) {
                    Ok((stop, log_rx, err_rx)) => {
                        state.sim_stop = Some(stop);
                        state.sim_log_rx = Some(log_rx);
                        state.sim_err_rx = Some(err_rx);
                        state.simulator.running = true;
                        state.simulator.log.clear();
                    }
                    Err(e) => {
                        state.simulator.status_msg = Some(e.clone());
                        state.show_error(&e);
                    }
                }
            }
        }
    });

    if let Some(ref msg) = state.simulator.status_msg {
        let display_msg = msg.replace("0.0.0.0", &get_local_ip().unwrap_or_else(|| "127.0.0.1".into()));
        ui.label(egui::RichText::new(&display_msg).color(egui::Color32::from_rgb(0, 150, 0)).strong());
        ui.label(egui::RichText::new(T::listening_hint(lang)).weak().small());
    }

    ui.add_space(4.0);

    // ── Data Editors ──
    ui.collapsing(T::holding_registers(lang), |ui| {
        render_holding_registers(ui, state);
    });
    ui.collapsing(T::coils(lang), |ui| {
        render_coils(ui, state);
    });

    // ── Log ──
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(T::sim_log(lang)).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button(T::clear(lang)).clicked() {
                state.simulator.log.clear();
            }
        });
    });
    egui::ScrollArea::vertical().max_height(140.0).stick_to_bottom(true).show(ui, |ui| {
        for entry in state.simulator.log.iter().rev() {
            let ts = chrono::DateTime::from_timestamp_millis(entry.timestamp)
                .map(|t| t.with_timezone(&chrono::Local).format("%H:%M:%S%.3f").to_string())
                .unwrap_or_default();
            let color = if entry.success { egui::Color32::GREEN } else { egui::Color32::RED };
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("[{}]", ts)).weak().monospace());
                ui.label(egui::RichText::new(&entry.direction).color(color).strong());
                ui.label(egui::RichText::new(&entry.decoded).weak());
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&entry.hex).monospace().weak());
            });
            ui.separator();
        }
    });
}

fn render_holding_registers(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    ui.horizontal(|ui| {
        ui.label(T::address(lang));
        ui.add(egui::TextEdit::singleline(&mut state.simulator.edit_addr).desired_width(60.0));
        ui.label(T::value(lang));
        ui.add(egui::TextEdit::singleline(&mut state.simulator.edit_value).desired_width(80.0));
        if ui.button(T::set_value(lang)).clicked() {
            if let (Ok(addr), Ok(val)) = (
                state.simulator.edit_addr.parse::<u16>(),
                state.simulator.edit_value.parse::<u16>(),
            ) {
                state.simulator.holding_registers.insert(addr, val);
                if let Some(ref regs) = state.sim_registers {
                    serialrun_core::protocol::update_holding_register(regs, addr, val);
                }
            }
        }
    });
    ui.add_space(4.0);
    egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
        let addrs: Vec<u16> = state.simulator.holding_registers.keys().copied().collect();
        for &addr in addrs.iter() {
            let val = state.simulator.holding_registers.get(&addr).copied().unwrap_or(0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("0x{:04X}", addr)).monospace());
                let mut v = val;
                ui.add(egui::DragValue::new(&mut v).range(0..=65535));
                if v != val {
                    state.simulator.holding_registers.insert(addr, v);
                    if let Some(ref regs) = state.sim_registers {
                        serialrun_core::protocol::update_holding_register(regs, addr, v);
                    }
                }
            });
        }
    });
    ui.horizontal(|ui| {
        if ui.button("+").clicked() {
            let max_addr = state.simulator.holding_registers.keys().max().copied().unwrap_or(0);
            state.simulator.holding_registers.insert(max_addr + 1, 0);
        }
    });
}

fn render_coils(ui: &mut egui::Ui, state: &mut AppState) {
    let addrs: Vec<u16> = state.simulator.coils.keys().copied().collect();
    egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
        for &addr in addrs.iter() {
            let val = state.simulator.coils.get(&addr).copied().unwrap_or(false);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("0x{:04X}", addr)).monospace());
                let mut v = val;
                ui.checkbox(&mut v, "");
                if v != val {
                    state.simulator.coils.insert(addr, v);
                    if let Some(ref regs) = state.sim_registers {
                        serialrun_core::protocol::update_coil(regs, addr, v);
                    }
                }
            });
        }
    });
    ui.horizontal(|ui| {
        if ui.button("+").clicked() {
            let max_addr = state.simulator.coils.keys().max().copied().unwrap_or(0);
            state.simulator.coils.insert(max_addr + 1, false);
        }
    });
}

fn poll_sim_logs(state: &mut AppState) {
    if let Some(rx) = &state.sim_err_rx {
        while let Ok(msg) = rx.try_recv() {
            state.simulator.status_msg = Some(msg);
        }
    }
    if let Some(rx) = &state.sim_log_rx {
        let mut sim_entries = Vec::new();
        while let Ok(entry) = rx.try_recv() {
            state.simulator.log.push_back(crate::state::SimulatorLogEntry {
                timestamp: entry.timestamp,
                direction: entry.direction.clone(),
                hex: entry.hex.clone(),
                decoded: entry.decoded.clone(),
                success: entry.success,
            });
            sim_entries.push(format!("[Simulator] {} {} | {}", entry.direction, entry.hex, entry.decoded));
            if state.simulator.log.len() > 1000 {
                state.simulator.log.pop_front();
            }
        }
        for msg in sim_entries {
            state.add_log_entry(crate::state::LogLevel::Info, &msg);
        }
    }
    if state.simulator.running && state.sim_stop.is_none() {
        state.simulator.running = false;
    }
}

fn get_local_ip() -> Option<String> {
    crate::util::get_local_ip()
}
