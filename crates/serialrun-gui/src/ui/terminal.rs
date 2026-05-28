use crate::state::{AppState, ChecksumMode, Direction, ScriptAction, ScriptCommand, T};
use eframe::egui;
use std::time::Duration;

pub fn render_terminal_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    let is_dark = state.theme == crate::state::Theme::Dark;

    // Poll async read result
    if let Some(rx) = &state.terminal_async_receiver {
        if let Ok(result) = rx.try_recv() {
            state.terminal_async_receiver = None;
            if let Ok(data) = result {
                if !data.is_empty() {
                    state.rx_count += data.len() as u64;
                    let received = String::from_utf8_lossy(&data).to_string();
                    state.add_terminal_line(Direction::Rx, received.clone(), false);
                    state.add_log_entry(crate::state::LogLevel::Info, &format!("Received {} bytes", data.len()));
                    // Data logger
                    super::data_logger::log_data(state, "RX", &data);
                    // Auto-reply (async)
                    if state.auto_reply_enabled && !state.auto_reply_pattern.is_empty() && !state.auto_reply_response.is_empty() {
                        if received.contains(&state.auto_reply_pattern) && state.auto_reply_async.is_none() {
                            let reply = state.auto_reply_response.clone();
                            let reply_bytes = reply.as_bytes().to_vec();
                            let port_name = state.selected_port.clone().unwrap_or_default();
                            let baud_rate = state.config.baud_rate;
                            state.auto_reply_async = Some(crate::async_utils::spawn_serial_write(port_name, baud_rate, reply_bytes));
                            state.add_terminal_line(Direction::Tx, reply, false);
                            state.add_log_entry(crate::state::LogLevel::Info, &format!("Auto-reply sent: {}", state.auto_reply_response));
                        }
                    }
                }
            }
        }
    }

    // Poll auto-reply write result
    if let Some(ref rx) = state.auto_reply_async {
        if let Ok(result) = rx.try_recv() {
            state.auto_reply_async = None;
            if let Err(e) = result {
                state.add_log_entry(crate::state::LogLevel::Error, &format!("Auto-reply error: {}", e));
            }
        }
    }

    // Poll async write result (for chained read-after-write)
    if let Some(ref rx) = state.terminal_async_receiver {
        // Already handled above
    }

    // Toolbar
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(T::terminal(lang)).strong().size(14.0));
        ui.separator();
        ui.checkbox(&mut state.hex_mode, "HEX");
        ui.checkbox(&mut state.show_timestamp, T::show_timestamp(lang));
        ui.checkbox(&mut state.auto_scroll, T::auto_scroll(lang));

        ui.separator();
        ui.label("CRC:");
        let checksum = state.terminal_checksum_mode;
        egui::ComboBox::from_id_salt("term_crc").width(100.0).selected_text(checksum.label(lang)).show_ui(ui, |ui| {
            for &mode in ChecksumMode::all() {
                ui.selectable_value(&mut state.terminal_checksum_mode, mode, mode.label(lang));
            }
        });

        ui.add_space(8.0);

        if ui.button(T::clear(lang)).clicked() {
            state.terminal_buffer.clear();
        }
    });

    ui.separator();

    // Terminal display area
    let available_height = ui.available_height() - 40.0;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(state.auto_scroll)
        .max_height(available_height)
        .show(ui, |ui| {
            for line in &state.terminal_buffer {
                let (color, prefix) = match line.direction {
                    Direction::Rx => (egui::Color32::from_rgb(0, 220, 100), "\u{2193} RX"),
                    Direction::Tx => (egui::Color32::from_rgb(100, 180, 255), "\u{2191} TX"),
                    Direction::System => (egui::Color32::from_rgb(200, 180, 80), "\u{2699} SYS"),
                };
                let content_color = if is_dark { egui::Color32::WHITE } else { egui::Color32::BLACK };
                let ts_color = if is_dark { egui::Color32::from_rgb(150, 150, 150) } else { egui::Color32::from_rgb(100, 100, 100) };

                let timestamp = if state.show_timestamp {
                    let time = chrono::DateTime::from_timestamp_millis(line.timestamp)
                        .map(|t| t.with_timezone(&chrono::Local).format("%H:%M:%S%.3f").to_string())
                        .unwrap_or_default();
                    format!("[{}] ", time)
                } else {
                    String::new()
                };

                let content = if line.is_hex {
                    line.content.clone()
                } else {
                    line.content
                        .replace("\r\n", "\u{21B5}\n")
                        .replace("\r", "\u{21B5}")
                        .replace("\n", "\u{21B5}\n")
                };

                ui.horizontal(|ui| {
                    if !timestamp.is_empty() {
                        ui.label(
                            egui::RichText::new(&timestamp)
                                .color(ts_color)
                                .size(13.0)
                                .monospace(),
                        );
                    }
                    ui.label(
                        egui::RichText::new(prefix)
                            .color(color)
                            .size(13.0)
                            .strong(),
                    );
                    ui.label(egui::RichText::new(&content).color(content_color).size(14.0));
                });
            }
        });

    ui.separator();

    // Input area
    ui.horizontal(|ui| {
        let response = ui.text_edit_singleline(&mut state.input_buffer);

        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if !state.input_buffer.is_empty() && state.is_connected {
                do_send(state);
            }
        }

        let send_btn = ui.button(egui::RichText::new(T::send(lang)).strong());
        if send_btn.clicked() && !state.input_buffer.is_empty() && state.is_connected {
            do_send(state);
        }
    });
}

fn do_send(state: &mut AppState) {
    let data = std::mem::take(&mut state.input_buffer);
    let hex_mode = state.hex_mode;
    let checksum_mode = state.terminal_checksum_mode;

    let mut bytes = if hex_mode {
        parse_hex(&data).unwrap_or_default()
    } else {
        data.as_bytes().to_vec()
    };

    bytes = checksum_mode.append_checksum(&bytes);

    let display = if hex_mode {
        data.clone()
    } else {
        data.replace("\r", "\\r").replace("\n", "\\n")
    };

    let port_name = state.selected_port.clone().unwrap_or_default();
    let baud_rate = state.config.baud_rate;
    state.tx_count += bytes.len() as u64;
    state.add_terminal_line(Direction::Tx, display, false);
    state.add_log_entry(crate::state::LogLevel::Info, &format!("Sent {} bytes", bytes.len()));
    // Data logger
    super::data_logger::log_data(state, "TX", &bytes);
    // Recording
    if state.recording {
        state.script_commands.push(ScriptCommand {
            delay_ms: 0,
            action: ScriptAction::Send,
            data: Some(data.clone()),
        });
    }

    // Async write, then async read
    let write_bytes = bytes.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    state.terminal_async_receiver = Some(rx);
    std::thread::spawn(move || {
        let config = serialrun_core::config::SerialConfig {
            port_name: port_name.clone(),
            baud_rate,
            ..Default::default()
        };
        let mut port = serialrun_core::SerialPort::new(config);
        if port.connect().is_err() {
            let _ = tx.send(Err("Connect failed".into()));
            return;
        }
        if port.write(&write_bytes).is_err() {
            let _ = port.disconnect();
            let _ = tx.send(Err("Write failed".into()));
            return;
        }
        // Wait for response
        std::thread::sleep(Duration::from_millis(50));
        let mut buf = [0u8; 1024];
        match port.read(&mut buf) {
            Ok(n) if n > 0 => {
                let _ = tx.send(Ok(buf[..n].to_vec()));
            }
            _ => {
                let _ = tx.send(Ok(Vec::new()));
            }
        }
        let _ = port.disconnect();
    });
}

fn parse_hex(hex_str: &str) -> Option<Vec<u8>> {
    let hex_str = hex_str.replace(" ", "").replace("0x", "").replace("0X", "");
    if hex_str.is_empty() || hex_str.len() % 2 != 0 {
        return None;
    }

    let mut bytes = Vec::new();
    for i in (0..hex_str.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex_str[i..i + 2], 16).ok()?;
        bytes.push(byte);
    }

    Some(bytes)
}
