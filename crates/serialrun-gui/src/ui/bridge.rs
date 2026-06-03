use crate::state::{AppState, T};
use eframe::egui;

pub fn render_bridge_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    ui.horizontal(|ui| {
        ui.heading(T::bridge(lang));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.button(egui::RichText::new("?").size(14.0).strong())
                .on_hover_text(T::bridge_tip_header(lang));
        });
    });
    ui.separator();

    poll_bridge_logs(state);

    // ── Configuration ──
    ui.collapsing(T::serial_config(lang), |ui| {
        egui::Grid::new("bridge_config").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
            ui.label(T::tcp_port(lang));
            ui.add(egui::DragValue::new(&mut state.bridge.tcp_port).range(1..=65535));
            ui.end_row();

            ui.label(T::serial_port(lang));
            egui::ComboBox::from_id_salt("bridge_port")
                .width(120.0)
                .selected_text(if state.bridge.serial_port_name.is_empty() { "--" } else { &state.bridge.serial_port_name })
                .show_ui(ui, |ui| {
                    for p in &state.ports {
                        ui.selectable_value(&mut state.bridge.serial_port_name, p.name.clone(), &p.name);
                    }
                });
            ui.end_row();

            ui.label(T::baud_rate(lang));
            ui.add(egui::DragValue::new(&mut state.bridge.baud_rate).range(300..=4000000));
            ui.end_row();

            ui.label(T::timeout_ms(lang));
            ui.add(egui::DragValue::new(&mut state.bridge.timeout_ms).range(100..=5000));
            ui.end_row();
        });
    });

    ui.add_space(4.0);

    // ── Control ──
    ui.horizontal(|ui| {
        if state.bridge.running {
            if ui.button(T::stop_bridge(lang)).clicked() {
                if let Some(stop) = state.bridge_stop.take() {
                    stop.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                state.bridge.running = false;
            }
            ui.label(egui::RichText::new(T::mcp_running(lang)).color(super::status::LOGO_GREEN).strong());
        } else {
            if ui.button(T::start_bridge(lang)).clicked() {
                let config = serialrun_core::protocol::BridgeConfig {
                    tcp_port: state.bridge.tcp_port,
                    serial_port_name: state.bridge.serial_port_name.clone(),
                    baud_rate: state.bridge.baud_rate,
                    timeout_ms: state.bridge.timeout_ms,
                };
                match serialrun_core::protocol::start_bridge(config) {
                    Ok((stop, log_rx, err_rx)) => {
                        state.bridge_stop = Some(stop);
                        state.bridge_log_rx = Some(log_rx);
                        state.bridge_err_rx = Some(err_rx);
                        state.bridge.running = true;
                        state.bridge.log.clear();
                    }
                    Err(e) => {
                        state.bridge.status_msg = Some(e.clone());
                        state.show_error(&e);
                    }
                }
            }
        }
    });

    if let Some(ref msg) = state.bridge.status_msg {
        let display_msg = msg.replace("0.0.0.0", &get_local_ip().unwrap_or_else(|| "127.0.0.1".into()));
        ui.label(egui::RichText::new(&display_msg).color(egui::Color32::from_rgb(0, 150, 0)).strong());
        ui.label(egui::RichText::new(T::bridge_hint(lang)).weak().small());
    }

    // ── Log ──
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(T::bridge_log(lang)).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button(T::clear(lang)).clicked() {
                state.bridge.log.clear();
            }
        });
    });
    egui::ScrollArea::vertical().max_height(180.0).stick_to_bottom(true).show(ui, |ui| {
        for entry in state.bridge.log.iter().rev() {
            let ts = chrono::DateTime::from_timestamp_millis(entry.timestamp)
                .map(|t| t.with_timezone(&chrono::Local).format("%H:%M:%S%.3f").to_string())
                .unwrap_or_default();
            let color = if entry.success { egui::Color32::GREEN } else { egui::Color32::RED };
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("[{}]", ts)).weak().monospace());
                ui.label(egui::RichText::new(&entry.direction).color(color).strong());
                ui.label(egui::RichText::new(&entry.client_addr).weak());
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("TX:").weak().monospace());
                ui.label(egui::RichText::new(&entry.request_hex).monospace());
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("RX:").weak().monospace());
                ui.label(egui::RichText::new(&entry.response_hex).monospace().color(color));
            });
            ui.separator();
        }
    });
}

fn poll_bridge_logs(state: &mut AppState) {
    if let Some(rx) = &state.bridge_err_rx {
        while let Ok(msg) = rx.try_recv() {
            state.bridge.status_msg = Some(msg);
        }
    }
    if let Some(rx) = &state.bridge_log_rx {
        let mut bridge_entries = Vec::new();
        while let Ok(entry) = rx.try_recv() {
            state.bridge.log.push_back(crate::state::BridgeLogEntry {
                timestamp: entry.timestamp,
                client_addr: entry.client_addr.clone(),
                direction: entry.direction.clone(),
                request_hex: entry.request_hex.clone(),
                response_hex: entry.response_hex.clone(),
                success: entry.success,
            });
            bridge_entries.push(format!("[TCP/Bridge] {} from {}: TX {} | RX {}",
                entry.direction, entry.client_addr, entry.request_hex, entry.response_hex));
            if state.bridge.log.len() > 1000 {
                state.bridge.log.pop_front();
            }
        }
        for msg in bridge_entries {
            state.add_log_entry(crate::state::LogLevel::Info, &msg);
        }
    }
    if state.bridge.running && state.bridge_stop.is_none() {
        state.bridge.running = false;
    }
}

fn get_local_ip() -> Option<String> {
    crate::util::get_local_ip()
}
