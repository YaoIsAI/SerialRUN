use crate::port_owner::PortCommand;
use crate::state::{AppState, ChecksumMode, Direction, Language, LineEnding, ScriptAction, ScriptCommand, T};
use crate::theme;
use eframe::egui;

pub fn render_terminal_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    let c = theme::get_colors(state.theme);

    // Toolbar — auto-wrapping, clear/save at end
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(T::terminal(lang)).strong().size(14.0));
        ui.separator();
        ui.checkbox(&mut state.show_timestamp, T::show_timestamp(lang));
        ui.checkbox(&mut state.auto_scroll, T::auto_scroll(lang));
        ui.separator();
        ui.label(egui::RichText::new(T::crc_label(lang)).strong()).on_hover_text(crc_hover_text(lang));
        let checksum = state.terminal_checksum_mode;
        egui::ComboBox::from_id_salt("term_crc").width(65.0).selected_text(checksum.label(lang)).show_ui(ui, |ui| {
            for &mode in ChecksumMode::all() {
                ui.selectable_value(&mut state.terminal_checksum_mode, mode, mode.label(lang));
            }
        });
        if state.is_connected {
            ui.separator();
            let old_dtr = state.dtr;
            let old_rts = state.rts;
            ui.checkbox(&mut state.dtr, "DTR");
            ui.checkbox(&mut state.rts, "RTS");
            if state.dtr != old_dtr { if let Some(ref po) = state.port_owner { po.send(PortCommand::SetDtr(state.dtr)); } }
            if state.rts != old_rts { if let Some(ref po) = state.port_owner { po.send(PortCommand::SetRts(state.rts)); } }
        }
        ui.separator();
        let auto_label = if state.auto_send_enabled { T::stop_auto(lang) } else { T::auto_send(lang) };
        if ui.small_button(auto_label).clicked() {
            state.auto_send_enabled = !state.auto_send_enabled;
            state.auto_send_last_time = chrono::Utc::now().timestamp_millis();
        }
        if state.auto_send_enabled {
            ui.add(egui::DragValue::new(&mut state.auto_send_interval_ms).range(100..=60000).suffix("ms"));
        }
        ui.separator();
        ui.checkbox(&mut state.rx_auto_aggregate, "T/O")
            .on_hover_text(if lang == Language::Chinese { "接收超时：合并碎片数据的等待时间" } else { "RX timeout: wait time to aggregate fragmented data" });
        if !state.rx_auto_aggregate {
            let old_val = state.rx_aggregate_ms;
            ui.add(egui::DragValue::new(&mut state.rx_aggregate_ms).range(10..=2000).suffix("ms"));
            if state.rx_aggregate_ms != old_val {
                if let Some(ref po) = state.port_owner {
                    po.sync_timeout(state.rx_aggregate_ms);
                }
            }
        } else {
            let baud = state.config.baud_rate;
            let calculated = if baud <= 0 { 150 } else if baud <= 4800 { 100 } else if baud <= 9600 { 50 } else if baud <= 19200 { 30 } else if baud <= 57600 { 15 } else { 10 };
            if state.rx_aggregate_ms != calculated {
                state.rx_aggregate_ms = calculated;
                if let Some(ref po) = state.port_owner {
                    po.sync_timeout(calculated);
                }
            }
            ui.label(egui::RichText::new(format!("{}ms (auto)", calculated)).color(c.text_secondary));
        }
        ui.separator();
        if ui.button(T::clear(lang)).clicked() { state.terminal_buffer.clear(); state.save_terminal(); }
        if ui.button(T::save_btn(lang)).clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("Text", &["txt"]).add_filter("All", &["*"]).save_file() {
                let mut content = String::new();
                for line in &state.terminal_buffer {
                    let ts = chrono::DateTime::from_timestamp_millis(line.timestamp).map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S%.3f").to_string()).unwrap_or_default();
                    content.push_str(&format!("[{}] {} {}\n", ts, line.direction, line.content));
                }
                let _ = std::fs::write(&path, content);
                state.add_log_entry(crate::state::LogLevel::Info, &format!("Terminal log saved to {}", path.display()));
            }
        }
    });

    ui.separator();

    // Terminal display area
    let available_height = ui.available_height() - 50.0;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(state.auto_scroll)
        .max_height(available_height)
        .show(ui, |ui| {
            for line in &state.terminal_buffer {
                let (color, content_color, prefix) = match line.direction {
                    Direction::Rx => (c.rx_color, c.rx_color, "\u{2193} RX"),
                    Direction::Tx => (c.tx_color, c.tx_color, "\u{2191} TX"),
                    Direction::System => (c.sys_color, c.sys_color, "\u{2699} SYS"),
                };
                let source_tag = if line.source.is_empty() { String::new() } else { format!("[{}] ", line.source) };
                let format_tag = if line.direction != Direction::System {
                    if line.is_hex { "[HEX] " } else { "[TEXT] " }
                } else { "" };
                let ts_color = c.timestamp_color;

                let timestamp = if state.show_timestamp {
                    let time = chrono::DateTime::from_timestamp_millis(line.timestamp)
                        .map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S%.3f").to_string())
                        .unwrap_or_default();
                    format!("[{}] ", time)
                } else {
                    String::new()
                };

                let content = if line.is_hex {
                    line.content.clone()
                } else {
                    let s = &line.content;
                    let mut out = String::with_capacity(s.len());
                    let mut chars = s.chars().peekable();
                    while let Some(ch) = chars.next() {
                        match ch {
                            '\r' => {
                                if chars.peek() == Some(&'\n') {
                                    chars.next();
                                    out.push('\n');
                                }
                            }
                            '\n' => out.push('\n'),
                            '\t' => out.push_str("    "),
                            c if c.is_control() => {}
                            c => out.push(c),
                        }
                    }
                    out
                };

                // Render row
                let row_top = ui.cursor().min.y;
                ui.horizontal_wrapped(|ui| {
                    if !timestamp.is_empty() {
                        ui.label(egui::RichText::new(&timestamp).color(ts_color).size(13.0).monospace());
                    }
                    ui.label(egui::RichText::new(prefix).color(color).size(13.0).strong());
                    if !source_tag.is_empty() {
                        ui.label(egui::RichText::new(&source_tag).color(egui::Color32::from_rgb(255, 193, 7)).size(12.0).strong());
                    }
                    if !format_tag.is_empty() {
                        let fmt_color = if line.is_hex {
                            egui::Color32::from_rgb(255, 152, 0)
                        } else {
                            egui::Color32::from_rgb(76, 175, 80)
                        };
                        ui.label(egui::RichText::new(format_tag).color(fmt_color).size(11.0).strong());
                    }
                    ui.add(egui::Label::new(egui::RichText::new(&content).color(content_color).size(14.0)).wrap());
                });
                // Add zero-height spacer to ensure cursor advances past the row
                ui.allocate_ui_with_layout(egui::vec2(0.0, 0.0), egui::Layout::left_to_right(egui::Align::Min), |_| {});
                let row_bottom = ui.cursor().min.y;

                // Right-click context menu (all line types)
                if line.direction != Direction::System && row_bottom > row_top {
                    let row_rect = egui::Rect::from_min_max(
                        egui::pos2(ui.cursor().min.x, row_top),
                        egui::pos2(ui.max_rect().max.x, row_bottom),
                    );
                    let line_content = line.content.clone();
                    let line_is_hex = line.is_hex;
                    let line_direction = line.direction;
                    let row_resp = ui.interact(row_rect, egui::Id::new(("term_row", line.timestamp)), egui::Sense::hover());
                    row_resp.context_menu(|ui| {
                        let is_tx = line_direction == Direction::Tx;

                        // Copy as-is
                        let copy_label = if lang == Language::Chinese { "复制" } else { "Copy" };
                        if ui.button(copy_label).clicked() {
                            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                let _ = clipboard.set_text(line_content.clone());
                            }
                            ui.close_menu();
                        }

                        // Copy All
                        let copy_all_label = if lang == Language::Chinese { "复制全部" } else { "Copy All" };
                        if ui.button(copy_all_label).clicked() {
                            let all_text: String = state.terminal_buffer.iter().map(|l| {
                                let ts = chrono::DateTime::from_timestamp_millis(l.timestamp)
                                    .map(|t| t.with_timezone(&chrono::Local).format("%H:%M:%S%.3f").to_string())
                                    .unwrap_or_default();
                                let dir = match l.direction {
                                    crate::state::Direction::Rx => "RX",
                                    crate::state::Direction::Tx => "TX",
                                    crate::state::Direction::System => "SYS",
                                };
                                format!("[{}] {} {}", ts, dir, l.content)
                            }).collect::<Vec<_>>().join("\n");
                            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                let _ = clipboard.set_text(all_text);
                            }
                            ui.close_menu();
                        }

                        ui.separator();

                        // Text→HEX (only for TEXT lines)
                        if !line_is_hex {
                            let convert_label = if lang == Language::Chinese { "转为 HEX" } else { "As HEX" };
                            if ui.button(convert_label).clicked() {
                                let hex_str = line_content.as_bytes()
                                    .iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                    let _ = clipboard.set_text(hex_str);
                                }
                                ui.close_menu();
                            }
                        }

                        // Resend (TX lines only)
                        if is_tx && state.is_connected {
                            ui.separator();
                            let resend_label = if lang == Language::Chinese { "重发" } else { "Resend" };
                            if ui.button(resend_label).clicked() {
                                state.input_buffer = line_content.clone();
                                state.hex_mode = line_is_hex;
                                ui.close_menu();
                            }
                        }
                    });
                }
            }
            if state.scroll_to_bottom_pending {
                state.scroll_to_bottom_pending = false;
                ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
            }
        });

    ui.separator();

    // Input area: [keep_input] [TXT/HEX] [========input========] [行尾] [发送]
    let row_height = 32.0;
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), row_height),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.add_space(8.0);
            ui.checkbox(&mut state.keep_input, T::keep_input(lang));
            ui.add_space(4.0);

            // Hex mode toggle badge
            let (mode_label, mode_color) = if state.hex_mode {
                ("HEX", egui::Color32::from_rgb(255, 152, 0))
            } else {
                ("TXT", egui::Color32::from_rgb(76, 175, 80))
            };
            let mode_btn = ui.add(egui::Button::new(
                egui::RichText::new(mode_label).color(egui::Color32::WHITE).strong().size(11.0)
            ).fill(mode_color).min_size(egui::vec2(36.0, 18.0)).rounding(3.0));
            if mode_btn.clicked() {
                state.hex_mode = !state.hex_mode;
            }
            ui.add_space(4.0);

            // Input box — fill remaining space (right controls: ~200px)
            let input_w = (ui.available_width() - 200.0).max(80.0);
            ui.add_sized(
                [input_w, row_height],
                egui::TextEdit::singleline(&mut state.input_buffer)
                    .frame(true)
                    .margin(egui::Margin::symmetric(8.0, 5.0))
            );

            ui.add_space(4.0);

            // Line ending selector
            ui.label(T::line_ending(lang));
            let le = state.line_ending;
            egui::ComboBox::from_id_salt("le_input").width(70.0).selected_text(le.label(lang)).show_ui(ui, |ui| {
                ui.selectable_value(&mut state.line_ending, LineEnding::None, LineEnding::None.label(lang));
                ui.selectable_value(&mut state.line_ending, LineEnding::CR, LineEnding::CR.label(lang));
                ui.selectable_value(&mut state.line_ending, LineEnding::LF, LineEnding::LF.label(lang));
                ui.selectable_value(&mut state.line_ending, LineEnding::CRLF, LineEnding::CRLF.label(lang));
            });

            ui.add_space(4.0);

            // Send button
            let btn_fill = if state.hex_mode {
                egui::Color32::from_rgb(255, 152, 0)
            } else {
                c.btn_send
            };
            let send_btn = ui.add(egui::Button::new(
                egui::RichText::new(T::send(lang)).color(egui::Color32::WHITE).strong().size(13.0)
            ).fill(btn_fill).min_size(egui::vec2(60.0, row_height)));
            if send_btn.clicked() && !state.input_buffer.is_empty() {
                do_send(state);
            }
        },
    );
}

pub fn do_send(state: &mut AppState) {
    // Check connection first
    if state.port_owner.is_none() {
        let msg = if state.language == Language::Chinese {
            "未连接串口，请先连接设备"
        } else {
            "Not connected. Please connect to a serial port first."
        };
        state.show_error(msg);
        return;
    }

    let data = if state.keep_input {
        state.input_buffer.clone()
    } else {
        std::mem::take(&mut state.input_buffer)
    };
    let hex_mode = state.hex_mode;
    let checksum_mode = state.terminal_checksum_mode;
    let line_ending = state.line_ending;
    let mut bytes = if hex_mode {
        match parse_hex(&data) {
            Some(b) => b,
            None => {
                let msg = if state.language == Language::Chinese {
                    "HEX 格式错误：只允许 0-9, A-F, a-f, 空格"
                } else {
                    "Invalid HEX: only 0-9, A-F, a-f, spaces allowed"
                };
                state.show_error(msg);
                state.input_buffer = data;
                return;
            }
        }
    } else {
        let mut b = data.as_bytes().to_vec();
        b.extend_from_slice(line_ending.suffix());
        b
    };

    bytes = checksum_mode.append_checksum(&bytes);

    // Show actual data sent (including CRC if selected) in terminal
    let display = if checksum_mode != crate::state::ChecksumMode::None {
        // CRC appended — show full frame including checksum
        if hex_mode {
            format_hex_bytes(&bytes)
        } else {
            // Show hex of full frame (including CRC bytes)
            format_hex_bytes(&bytes)
        }
    } else if hex_mode {
        data.clone()
    } else {
        data.replace("\r", "\\r").replace("\n", "\\n")
    };

    state.tx_count += bytes.len() as u64;
    state.add_chart_data(bytes.len() as f64);
    state.add_terminal_line(Direction::Tx, display, hex_mode);
    let hex_preview = format_hex_bytes(&bytes);
    let text_preview = String::from_utf8_lossy(&bytes).to_string();
    state.add_log_entry(crate::state::LogLevel::Info, &format!("TX {} bytes: {} | {}", bytes.len(), hex_preview, text_preview));
    super::data_logger::log_data(state, "TX", &bytes);
    if state.recording {
        let now = chrono::Utc::now().timestamp_millis();
        let delay = if state.recording_last_time > 0 {
            (now - state.recording_last_time).max(0) as u64
        } else {
            0
        };
        // Record wait if delay > 50ms (avoid noise)
        if delay > 50 {
            state.script_commands.push(ScriptCommand {
                delay_ms: delay,
                action: ScriptAction::Wait,
                data: None,
            });
        }
        state.script_commands.push(ScriptCommand {
            delay_ms: 0,
            action: ScriptAction::Send,
            data: Some(data.clone()),
        });
        state.recording_last_time = now;
    }

    // Write through port owner (connection already verified at top of function)
    if let Some(ref po) = state.port_owner {
        po.send(PortCommand::Write(bytes));
    }
}

pub fn parse_hex(hex_str: &str) -> Option<Vec<u8>> {
    // Strip spaces and 0x/0X prefixes per byte
    let hex_str: String = hex_str.split_whitespace()
        .filter_map(|token| {
            let t = token.strip_prefix("0x").or_else(|| token.strip_prefix("0X")).unwrap_or(token);
            if t.is_empty() { return None; }
            // Validate each character is hex
            if t.chars().all(|c| c.is_ascii_hexdigit()) {
                Some(t.to_string())
            } else {
                None
            }
        })
        .collect();
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

/// Format raw bytes as space-separated hex string
pub fn format_hex_bytes(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
}

fn crc_hover_text(lang: Language) -> String {
    if lang == Language::Chinese {
        "CRC16/MODBUS: Modbus RTU 标准校验 (0xA001)\nCRC16/CCITT: CCITT 标准 (0x1021)\nCRC32: 32位循环冗余校验\nLRC: 纵向冗余校验 (Modbus ASCII)\nSUM8: 8位累加和校验".into()
    } else {
        "CRC16/MODBUS: Modbus RTU standard (0xA001)\nCRC16/CCITT: CCITT standard (0x1021)\nCRC32: 32-bit CRC\nLRC: Longitudinal Redundancy Check (Modbus ASCII)\nSUM8: 8-bit additive checksum".into()
    }
}
