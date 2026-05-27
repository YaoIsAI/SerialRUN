use crate::state::{AppState, Direction, T};
use eframe::egui;
use std::time::Duration;

pub fn render_terminal_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    // Toolbar
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(T::terminal(lang)).strong().size(14.0));
        ui.separator();
        ui.checkbox(&mut state.hex_mode, "HEX");
        ui.checkbox(&mut state.show_timestamp, T::show_timestamp(lang));
        ui.checkbox(&mut state.auto_scroll, T::auto_scroll(lang));

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
                    Direction::Rx => (egui::Color32::from_rgb(0, 200, 80), "\u{2193} RX"),
                    Direction::Tx => (egui::Color32::from_rgb(80, 160, 255), "\u{2191} TX"),
                    Direction::System => (egui::Color32::from_rgb(180, 160, 60), "\u{2699}"),
                };

                let timestamp = if state.show_timestamp {
                    let time = chrono::DateTime::from_timestamp_millis(line.timestamp)
                        .map(|t| t.format("%H:%M:%S%.3f").to_string())
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
                    ui.label(
                        egui::RichText::new(format!("{} {}", timestamp, prefix))
                            .color(color)
                            .small(),
                    );
                    ui.label(&content);
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

    let bytes = if hex_mode {
        parse_hex(&data).unwrap_or_default()
    } else {
        data.as_bytes().to_vec()
    };

    let display = if hex_mode {
        data.clone()
    } else {
        data.replace("\r", "\\r").replace("\n", "\\n")
    };

    // Write and read without holding port borrow across state mutations
    let write_result = if let Some(ref mut port) = state.port {
        port.write(&bytes).map(|n| {
            state.tx_count += n as u64;
            n
        })
    } else {
        return;
    };

    match write_result {
        Ok(n) => {
            state.add_terminal_line(Direction::Tx, display, false);
            state.add_log_entry(crate::state::LogLevel::Info, &format!("Sent {} bytes", n));
        }
        Err(e) => {
            state.add_terminal_line(Direction::System, format!("Send error: {}", e), false);
            state.add_log_entry(crate::state::LogLevel::Error, &e.to_string());
            return;
        }
    }

    // Try to read response
    let mut buf = [0u8; 1024];
    std::thread::sleep(Duration::from_millis(50));
    if let Some(ref mut port) = state.port {
        match port.read(&mut buf) {
            Ok(n) if n > 0 => {
                state.rx_count += n as u64;
                let received = String::from_utf8_lossy(&buf[..n]).to_string();
                state.add_terminal_line(Direction::Rx, received, false);
                state.add_log_entry(crate::state::LogLevel::Info, &format!("Received {} bytes", n));
            }
            _ => {}
        }
    }
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
