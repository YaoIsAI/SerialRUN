use crate::state::{AppState, CanConnectionMode, CanFrameData, T};
use crate::async_utils::PersistentReader;
use eframe::egui;
use std::collections::HashMap;
use serialrun_core::protocol::canalyst::{self, CanalystDriver, VciCanObj};

pub fn render_can_analyzer_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    // Poll CAN TX result
    if let Some(ref rx) = state.can_tx_async {
        if let Ok(result) = rx.try_recv() {
            state.can_tx_async = None;
            if let Err(e) = result {
                state.show_error(&format!("CAN TX: {}", e));
            }
        }
    }

    // Poll persistent reader
    if let Some(ref reader) = state.can_reader {
        while let Some(frames) = reader.poll() {
            state.can_frames.extend(frames);
            if state.can_frames.len() > 100_000 {
                state.can_frames.drain(..state.can_frames.len() - 100_000);
            }
        }
    }

    // Periodic send tick
    if state.can_tx_periodic && state.can_tx_sent_count < state.can_tx_count {
        let now = chrono::Utc::now().timestamp_millis();
        if now >= state.can_tx_next_time {
            periodic_send_tick(state);
        }
    } else if state.can_tx_periodic && state.can_tx_sent_count >= state.can_tx_count {
        state.can_tx_periodic = false;
    }

    // ── Help Window ──
    if state.show_can_help {
        let mut open = state.show_can_help;
        egui::Window::new(T::can_help_title(lang))
            .open(&mut open)
            .resizable(true)
            .default_width(400.0)
            .default_height(280.0)
            .show(ui.ctx(), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading(T::can_analyzer(lang));
                    ui.add_space(4.0);
                    ui.label(T::can_help_connect(lang));
                    ui.label(T::can_help_start(lang));
                    ui.label(T::can_help_filter(lang));
                    ui.label(T::can_help_tx(lang));
                    ui.label(T::can_help_periodic(lang));
                    ui.label(T::can_help_export(lang));
                });
            });
        state.show_can_help = open;
    }

    // ═══════════════════════════════════════════════════════
    // Header: Title + Help button
    // ═══════════════════════════════════════════════════════
    ui.horizontal(|ui| {
        ui.heading(T::can_analyzer(lang));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(egui::RichText::new("?").size(14.0).strong())
                .on_hover_text(T::can_tip_header(lang))
                .clicked()
            {
                state.show_can_help = !state.show_can_help;
            }
        });
    });
    ui.separator();

    // ═══════════════════════════════════════════════════════
    // CAN Connection (Independent from terminal)
    // ═══════════════════════════════════════════════════════
    // Mode selector (only show on Windows/Linux where CANalyst is supported)
    let modes = CanConnectionMode::all();
    if modes.len() > 1 {
        ui.horizontal(|ui| {
            ui.label(T::can_mode_label(lang));
            for &mode in modes {
                let selected = state.can_connection_mode == mode;
                if ui.selectable_label(selected, mode.label(lang)).clicked() && !state.can_connected {
                    state.can_connection_mode = mode;
                }
            }
        });
        ui.add_space(2.0);
    }

    match state.can_connection_mode {
        CanConnectionMode::Slcan => render_slcan_connection(ui, state),
        CanConnectionMode::Canalyst => render_canalyst_connection(ui, state),
    }
    ui.add_space(2.0);

    // ═══════════════════════════════════════════════════════
    // TX Section - Grid layout for clean alignment
    // ═══════════════════════════════════════════════════════
    ui.label(egui::RichText::new(T::can_send_title(lang)).strong());
    ui.add_space(2.0);

    egui::Grid::new("can_tx_grid").num_columns(4).spacing([8.0, 4.0]).show(ui, |ui| {
        // Row 1: Frame Format | Frame Type | Frame ID
        ui.label(T::can_frame_format(lang));
        egui::ComboBox::from_id_salt("can_frame_fmt")
            .width(80.0)
            .selected_text(if state.can_tx_ext { T::can_ext_frame(lang) } else { T::can_std_frame(lang) })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.can_tx_ext, false, T::can_std_frame(lang));
                ui.selectable_value(&mut state.can_tx_ext, true, T::can_ext_frame(lang));
            });
        ui.label(T::can_frame_id(lang)).on_hover_text(T::can_tip_id(lang));
        ui.add(egui::TextEdit::singleline(&mut state.can_tx_id)
            .desired_width(100.0)
            .font(egui::TextStyle::Monospace)
            .hint_text(if state.can_tx_ext { "00000000" } else { "000" }));
        ui.end_row();

        // Row 2: Frame Type | Data
        ui.label(T::can_frame_type(lang));
        egui::ComboBox::from_id_salt("can_frame_type")
            .width(80.0)
            .selected_text(if state.can_tx_remote { T::can_remote_frame(lang) } else { T::can_data_frame(lang) })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.can_tx_remote, false, T::can_data_frame(lang));
                ui.selectable_value(&mut state.can_tx_remote, true, T::can_remote_frame(lang));
            });
        ui.label(T::can_data_hex(lang)).on_hover_text(T::can_tip_data(lang));
        ui.add(egui::TextEdit::singleline(&mut state.can_tx_data)
            .desired_width(180.0)
            .font(egui::TextStyle::Monospace)
            .hint_text("00 00 00 01"));
        ui.end_row();

        // Row 3: Count | Period
        ui.label(T::can_tx_total(lang));
        ui.add(egui::DragValue::new(&mut state.can_tx_count).range(1..=10000));
        ui.label(T::can_tx_period(lang));
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut state.can_tx_period_ms).range(10..=10000));
            ui.label("ms");
        });
        ui.end_row();
    });
    ui.add_space(2.0);

    // Row 4: Checkboxes + Buttons
    ui.horizontal(|ui| {
        ui.checkbox(&mut state.can_tx_id_increment, T::can_id_inc(lang));
        ui.checkbox(&mut state.can_tx_data_increment, T::can_data_inc(lang));
        ui.add_space(12.0);
        let can_ready = state.can_connected && match state.can_connection_mode {
            CanConnectionMode::Slcan => state.can_write_tx.is_some(),
            CanConnectionMode::Canalyst => state.canalyst_write_tx.is_some(),
        };
        if state.can_tx_periodic {
            ui.label(egui::RichText::new(format!("{}/{}", state.can_tx_sent_count, state.can_tx_count))
                .color(egui::Color32::from_rgb(251, 191, 36)));
            if ui.button(T::can_stop_send(lang)).clicked() {
                state.can_tx_periodic = false;
            }
        } else {
            let btn = ui.add_enabled(can_ready, egui::Button::new(
                egui::RichText::new(T::can_send_msg(lang)).strong()));
            if btn.clicked() {
                if can_ready {
                    // Single or periodic send
                    if state.can_tx_count > 1 {
                        state.can_tx_periodic = true;
                        state.can_tx_sent_count = 0;
                        state.can_tx_next_time = chrono::Utc::now().timestamp_millis();
                    } else {
                        can_transmit(state);
                    }
                } else {
                    state.show_error(T::start_first(lang));
                }
            }
        }
    });
    ui.add_space(4.0);

    // ═══════════════════════════════════════════════════════
    // Control Bar: Capture + Status + Filter
    // ═══════════════════════════════════════════════════════
    ui.separator();
    ui.horizontal(|ui| {
        // Start / Stop capture (only when connected)
        if state.can_connected {
            let label = if state.can_capturing { T::stop_capture(lang) } else { T::start_capture(lang) };
            if ui.button(egui::RichText::new(label).strong()).clicked() {
                state.can_capturing = !state.can_capturing;
                if state.can_capturing {
                    state.can_frames.clear();
                    state.can_stats = crate::state::CanStats::default();
                } else {
                    state.can_tx_periodic = false;
                }
            }
        }
        if ui.button(T::clear(lang)).clicked() {
            state.can_frames.clear();
            state.can_stats = crate::state::CanStats::default();
        }
        if ui.button(T::export_btn(lang)).clicked() {
            export_can_frames(state);
        }
        ui.separator();
        // Status indicator
        if state.can_connected {
            if state.can_capturing {
                ui.label(egui::RichText::new(format!("[{}]", T::can_running(lang)))
                    .color(egui::Color32::from_rgb(34, 197, 94)).strong());
            } else {
                ui.label(egui::RichText::new(format!("[{}]", T::can_connected(lang)))
                    .color(egui::Color32::from_rgb(59, 130, 246)).strong());
            }
        } else {
            ui.label(egui::RichText::new(format!("[{}]", T::can_disconnected(lang)))
                .color(egui::Color32::from_rgb(156, 163, 175)));
        }
        // Filter
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(egui::TextEdit::singleline(&mut state.can_filter_id)
                .desired_width(100.0)
                .font(egui::TextStyle::Monospace)
                .hint_text("Filter ID"));
            ui.label(T::filter_label(lang));
        });
    });
    ui.add_space(2.0);

    // ═══════════════════════════════════════════════════════
    // Statistics Row
    // ═══════════════════════════════════════════════════════
    let stats = compute_stats(&state.can_frames);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("R={} T={}", stats.rx_count, stats.tx_count)).monospace());
        ui.separator();
        ui.label(egui::RichText::new(format!("{}:{}", T::errors(lang), stats.error_count)).monospace());
        ui.separator();
        ui.label(egui::RichText::new(format!("{}:{}", T::id_count(lang), stats.unique_ids)).monospace());
        ui.separator();
        ui.label(egui::RichText::new(format!("{}:{:.1}%", T::bus_load(lang), stats.bus_load)).monospace());
        ui.separator();
        ui.label(egui::RichText::new(format!("{}:{:X}", T::max_id(lang), stats.max_id)).monospace());
        ui.separator();
        ui.label(egui::RichText::new(format!("{}:{}", T::frames_label(lang), state.can_frames.len())).monospace());
    });
    ui.add_space(4.0);

    // ═══════════════════════════════════════════════════════
    // Frame Table
    // ═══════════════════════════════════════════════════════
    render_frame_table(ui, state);
}

// ── Periodic Send Tick ──
fn periodic_send_tick(state: &mut AppState) {
    if state.can_tx_sent_count >= state.can_tx_count {
        state.can_tx_periodic = false;
        return;
    }
    if state.can_tx_id_increment {
        if let Some(id) = parse_hex_id(&state.can_tx_id) {
            let new_id = if state.can_tx_ext { id.wrapping_add(1) & 0x1FFFFFFF } else { id.wrapping_add(1) & 0x7FF };
            state.can_tx_id = format!("{:X}", new_id);
        }
    }
    if state.can_tx_data_increment {
        if let Some(mut data) = parse_hex_data(&state.can_tx_data) {
            if let Some(last) = data.last_mut() {
                *last = last.wrapping_add(1);
            }
            state.can_tx_data = data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
        }
    }
    can_transmit(state);
    state.can_tx_sent_count += 1;
    state.can_tx_next_time = chrono::Utc::now().timestamp_millis() + state.can_tx_period_ms as i64;
}

// ── Frame Table ──
fn render_frame_table(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    let filter = parse_hex_id(&state.can_filter_id);

    // Table header
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("#").strong().monospace());
        ui.add_space(24.0);
        ui.label(egui::RichText::new(T::can_col_time(lang)).strong().monospace());
        ui.add_space(8.0);
        ui.label(egui::RichText::new(T::can_col_channel(lang)).strong().monospace());
        ui.add_space(4.0);
        ui.label(egui::RichText::new(T::can_col_dir(lang)).strong().monospace());
        ui.add_space(4.0);
        ui.label(egui::RichText::new(T::can_col_id(lang)).strong().monospace());
        ui.add_space(16.0);
        ui.label(egui::RichText::new(T::can_col_type(lang)).strong().monospace());
        ui.add_space(4.0);
        ui.label(egui::RichText::new("DLC").strong().monospace());
        ui.add_space(4.0);
        ui.label(egui::RichText::new(T::can_col_data(lang)).strong().monospace());
    });
    ui.separator();

    // Frame rows
    egui::ScrollArea::vertical().max_height(280.0).stick_to_bottom(true).show(ui, |ui| {
        let mut idx = 0u32;
        let mut prev_ts: Option<i64> = None;
        for frame in &state.can_frames {
            if let Some(filt) = filter {
                if frame.id != filt { continue; }
            }
            idx += 1;
            let ts = chrono::DateTime::from_timestamp_millis(frame.timestamp)
                .map(|t| t.with_timezone(&chrono::Local).format("%H:%M:%S%.3f").to_string())
                .unwrap_or_default();
            let delta = prev_ts.map(|p| frame.timestamp - p).filter(|&d| d >= 0);
            prev_ts = Some(frame.timestamp);

            let id_str = if frame.is_ext { format!("0x{:08X}", frame.id) } else { format!("0x{:03X}", frame.id) };
            let data_str = frame.data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
            let id_color = get_id_color(frame.id);

            ui.horizontal(|ui| {
                // Index
                ui.label(egui::RichText::new(format!("{:04}", idx)).monospace());
                // Time + delta
                ui.label(egui::RichText::new(&ts).monospace());
                if let Some(d) = delta {
                    let dc = if d < 10 { egui::Color32::from_rgb(0, 180, 0) }
                        else if d < 100 { egui::Color32::from_rgb(180, 180, 0) }
                        else { egui::Color32::from_rgb(180, 60, 60) };
                    ui.label(egui::RichText::new(format!("+{}ms", d)).color(dc).monospace());
                }
                // Channel
                ui.label(egui::RichText::new(format!("ch{}", frame.channel)).monospace());
                // Direction dot + label
                let (dir_text, dir_color) = if frame.is_error {
                    ("ERR", egui::Color32::RED)
                } else if frame.is_tx {
                    (T::can_dir_tx(lang), egui::Color32::from_rgb(251, 191, 36))
                } else {
                    (T::can_dir_rx(lang), egui::Color32::from_rgb(34, 197, 94))
                };
                let (rect, _) = ui.allocate_exact_size(egui::vec2(6.0, 6.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 3.0, dir_color);
                ui.label(egui::RichText::new(dir_text).color(dir_color).monospace());
                // ID
                ui.label(egui::RichText::new(&id_str).color(id_color).monospace().strong());
                // Type
                let type_str = if frame.is_error { "ERR" }
                    else if frame.is_ext { T::can_ext_frame(lang) }
                    else { T::can_std_frame(lang) };
                ui.label(egui::RichText::new(type_str).monospace());
                // DLC
                ui.label(egui::RichText::new(format!("0x{:02X}", frame.dlc)).monospace());
                // Data
                ui.label(egui::RichText::new(&data_str).monospace());
            });
        }
        if idx == 0 {
            ui.label(egui::RichText::new("No data").weak().monospace());
        }
    });
}

// ── CAN Reader (Independent connection) ──
fn start_can_reader(state: &mut AppState) {
    if state.can_reader.is_some() { return; }
    // Use CAN-specific port and baud rate (independent from terminal)
    let port_name = state.can_port.clone();
    let baud_rate = state.can_baud_rate;
    if port_name.is_empty() {
        state.show_error("CAN: no port selected");
        state.can_capturing = false;
        return;
    }
    // CAN has its own serial connection — does NOT take over terminal's port_owner
    let (write_tx, write_rx) = std::sync::mpsc::channel::<Vec<u8>>();
    state.can_write_tx = Some(write_tx);
    let reader = PersistentReader::start(move |stop, tx| {
        let config = serialrun_core::config::SerialConfig {
            port_name, baud_rate, ..Default::default()
        };
        let mut port = serialrun_core::SerialPort::new(config);
        if port.connect().is_err() { return; }
        let _ = port.set_timeout(std::time::Duration::from_millis(50));
        let mut line_buf = String::new();
        let mut buf = [0u8; 1024];
        while !stop.load(std::sync::atomic::Ordering::Relaxed) {
            while let Ok(data) = write_rx.try_recv() {
                let _ = port.write(&data);
            }
            match port.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let text = String::from_utf8_lossy(&buf[..n]);
                    line_buf.push_str(&text);
                    let mut frames = Vec::new();
                    while let Some(pos) = line_buf.find(|c| c == '\r' || c == '\n') {
                        let line = line_buf[..pos].to_string();
                        let next = if pos + 1 < line_buf.len() {
                            let next_ch = line_buf.as_bytes()[pos + 1];
                            if (line_buf.as_bytes()[pos] == b'\r' && next_ch == b'\n')
                                || (line_buf.as_bytes()[pos] == b'\n' && next_ch == b'\r')
                            { pos + 2 } else { pos + 1 }
                        } else { pos + 1 };
                        line_buf = line_buf[next..].to_string();
                        if let Some(frame) = parse_slcan_line(&line) {
                            frames.push(frame);
                        }
                    }
                    if !frames.is_empty() { let _ = tx.send(frames); }
                }
                _ => { std::thread::sleep(std::time::Duration::from_millis(5)); }
            }
        }
        let _ = port.disconnect();
    });
    state.can_reader = Some(reader);
}

fn stop_can_reader(state: &mut AppState) {
    if let Some(mut reader) = state.can_reader.take() {
        reader.stop();
    }
    state.can_write_tx = None;
    // CAN has its own connection — no need to restart terminal's port_owner
}

// ── SLCAN Connection UI ──
fn render_slcan_connection(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    ui.horizontal(|ui| {
        ui.label(T::can_port(lang));
        let terminal_port = state.selected_port.clone().unwrap_or_default();
        let can_port_empty = state.can_port.is_empty();
        let port_conflict = !can_port_empty && state.can_port == terminal_port && state.is_connected;

        egui::ComboBox::from_id_salt("can_port")
            .width(140.0)
            .selected_text(if can_port_empty { "—" } else { &state.can_port })
            .show_ui(ui, |ui| {
                for port in &state.ports {
                    let name = port.name.clone();
                    let is_terminal = name == terminal_port && state.is_connected;
                    let is_can_active = name == state.can_port && state.can_connected;
                    let label = if is_terminal && is_can_active {
                        format!("{} ({})", name, T::can_port_both(lang))
                    } else if is_terminal {
                        format!("{} ({})", name, T::can_port_terminal(lang))
                    } else if is_can_active {
                        format!("{} ({})", name, T::can_port_can(lang))
                    } else {
                        name.clone()
                    };
                    ui.selectable_value(&mut state.can_port, name, label);
                }
            });

        // Connect / Disconnect button
        if state.can_connected {
            if ui.button(egui::RichText::new(T::can_disconnect(lang)).strong()).clicked() {
                state.can_capturing = false;
                state.can_tx_periodic = false;
                stop_can_reader(state);
                state.can_connected = false;
            }
        } else {
            let can_port_ready = !state.can_port.is_empty();
            if ui.add_enabled(can_port_ready, egui::Button::new(
                egui::RichText::new(T::can_connect(lang)).strong())).clicked() {
                start_can_reader(state);
                if state.can_reader.is_some() {
                    state.can_connected = true;
                    state.can_capturing = true;
                    state.can_frames.clear();
                    state.can_stats = crate::state::CanStats::default();
                } else {
                    state.show_error("CAN: failed to open port");
                }
            }
        }

        if port_conflict {
            ui.label(egui::RichText::new(format!("⚠ {}", T::can_port_conflict(lang)))
                .color(egui::Color32::from_rgb(251, 191, 36)));
        }

        ui.separator();
        ui.label(T::can_baud(lang));
        egui::ComboBox::from_id_salt("can_baud")
            .width(80.0)
            .selected_text(baud_display(state.can_baud_rate))
            .show_ui(ui, |ui| {
                for &rate in &[100_000, 125_000, 250_000, 500_000, 1_000_000] {
                    ui.selectable_value(&mut state.can_baud_rate, rate, baud_display(rate));
                }
            });

        if ui.small_button("↻").on_hover_text(T::refresh_ports(lang)).clicked() {
            state.refresh_ports();
        }
    });
}

// ── CANalyst-II Connection UI ──
fn render_canalyst_connection(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    // Check DLL availability
    let driver = canalyst::get_driver();
    if driver.is_none() {
        ui.colored_label(egui::Color32::from_rgb(220, 50, 50), T::can_dll_not_found(lang));
        return;
    }

    // Scan devices button
    ui.horizontal(|ui| {
        if ui.button(T::can_scan_devices(lang)).clicked() {
            if let Some(ref drv) = driver {
                let devices = drv.find_devices();
                state.canalyst_device_list = devices.iter().map(|d| d.serial_number()).collect();
                if state.canalyst_device_list.is_empty() {
                    state.show_error(T::can_no_device(lang));
                } else if state.canalyst_device_index as usize >= state.canalyst_device_list.len() {
                    state.canalyst_device_index = 0;
                }
            }
        }

        // Device selector
        ui.label(T::can_device_label(lang));
        let device_count = state.canalyst_device_list.len();
        let device_text = if device_count == 0 {
            "—".to_string()
        } else {
            state.canalyst_device_list
                .get(state.canalyst_device_index as usize)
                .cloned()
                .unwrap_or_else(|| format!("#{}", state.canalyst_device_index))
        };
        egui::ComboBox::from_id_salt("canalyst_device")
            .width(120.0)
            .selected_text(&device_text)
            .show_ui(ui, |ui| {
                for (i, serial) in state.canalyst_device_list.iter().enumerate() {
                    ui.selectable_value(&mut state.canalyst_device_index, i as u32, serial);
                }
            });

        // Channel selector
        ui.label(T::can_channel(lang));
        let ch_text = if state.canalyst_channel == 0 { "CAN1" } else { "CAN2" };
        egui::ComboBox::from_id_salt("canalyst_channel")
            .width(60.0)
            .selected_text(ch_text)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.canalyst_channel, 0, "CAN1");
                ui.selectable_value(&mut state.canalyst_channel, 1, "CAN2");
            });

        // Baud rate
        ui.label(T::can_baud(lang));
        egui::ComboBox::from_id_salt("canalyst_baud")
            .width(80.0)
            .selected_text(baud_display(state.can_baud_rate))
            .show_ui(ui, |ui| {
                for &rate in canalyst::SUPPORTED_BAUD_RATES {
                    ui.selectable_value(&mut state.can_baud_rate, rate, baud_display(rate));
                }
            });
    });

    // Work mode + Board info
    ui.horizontal(|ui| {
        ui.label(T::can_work_mode(lang));
        let mode_text = match state.canalyst_work_mode {
            0 => T::can_normal_mode(lang),
            1 => T::can_listen_mode(lang),
            2 => T::can_loopback_mode(lang),
            _ => T::can_normal_mode(lang),
        };
        egui::ComboBox::from_id_salt("canalyst_mode")
            .width(80.0)
            .selected_text(mode_text)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.canalyst_work_mode, 0, T::can_normal_mode(lang));
                ui.selectable_value(&mut state.canalyst_work_mode, 1, T::can_listen_mode(lang));
                ui.selectable_value(&mut state.canalyst_work_mode, 2, T::can_loopback_mode(lang));
            });

        if let Some(ref info) = state.canalyst_board_info {
            ui.separator();
            ui.label(egui::RichText::new(format!("{} {}", T::can_board_info(lang), info))
                .color(egui::Color32::from_rgb(100, 160, 230)));
        }
    });

    // Connect / Disconnect
    ui.horizontal(|ui| {
        if state.can_connected {
            if ui.button(egui::RichText::new(T::can_disconnect(lang)).strong()).clicked() {
                state.can_capturing = false;
                state.can_tx_periodic = false;
                stop_canalyst_reader(state);
                state.can_connected = false;
                state.canalyst_board_info = None;
            }
        } else {
            let has_device = !state.canalyst_device_list.is_empty();
            if ui.add_enabled(has_device, egui::Button::new(
                egui::RichText::new(T::can_connect(lang)).strong())).clicked() {
                start_canalyst_reader(state);
                if state.can_reader.is_some() {
                    state.can_connected = true;
                    state.can_capturing = true;
                    state.can_frames.clear();
                    state.can_stats = crate::state::CanStats::default();
                } else {
                    state.show_error("CANalyst-II: connection failed");
                }
            }
        }
    });
}

// ── CANalyst-II Reader ──
fn start_canalyst_reader(state: &mut AppState) {
    if state.can_reader.is_some() { return; }
    let driver = match canalyst::get_driver() {
        Some(d) => d,
        None => { state.show_error("CANalyst-II: DLL not loaded"); return; }
    };

    let dev_index = state.canalyst_device_index;
    let can_index = state.canalyst_channel as u32;
    let baud_rate = state.can_baud_rate;
    let work_mode = state.canalyst_work_mode;

    // Open device and initialize
    if let Err(e) = driver.open_device(dev_index) {
        state.show_error(&format!("CANalyst-II: {}", e));
        return;
    }
    if let Err(e) = driver.init_can_with_mode(dev_index, can_index, baud_rate, work_mode) {
        let _ = driver.close_device(dev_index);
        state.show_error(&format!("CANalyst-II init: {}", e));
        return;
    }
    if let Err(e) = driver.clear_buffer(dev_index, can_index) {
        log::warn!("CANalyst-II clear_buffer: {}", e);
    }
    if let Err(e) = driver.start_can(dev_index, can_index) {
        let _ = driver.close_device(dev_index);
        state.show_error(&format!("CANalyst-II start: {}", e));
        return;
    }

    // Read board info
    match driver.read_board_info(dev_index) {
        Ok(info) => {
            state.canalyst_board_info = Some(format!("{} SN:{}", info.hw_type(), info.serial_number()));
        }
        Err(e) => { log::warn!("CANalyst-II board info: {}", e); }
    }

    // Create write channel for TX
    let (write_tx, write_rx) = std::sync::mpsc::channel::<CanFrameData>();
    state.can_write_tx = None; // SLCAN channel not used
    state.canalyst_write_tx = Some(write_tx);

    let driver_clone = driver.clone();
    let reader = PersistentReader::start(move |stop, tx| {
        let mut buf = [VciCanObj::default(); 2500];
        while !stop.load(std::sync::atomic::Ordering::Relaxed) {
            // TX: drain write channel
            while let Ok(frame) = write_rx.try_recv() {
                let vci_obj = canalyst_frame_to_vci(&frame);
                let _ = driver_clone.transmit(dev_index, can_index, &[vci_obj]);
            }
            // RX: poll frames
            match driver_clone.receive(dev_index, can_index, 2500) {
                Ok(frames) if !frames.is_empty() => {
                    let parsed: Vec<CanFrameData> = frames.iter()
                        .map(|o| vci_to_can_frame(o, can_index as u8 + 1))
                        .collect();
                    let _ = tx.send(parsed);
                }
                Err(_) => {
                    // Device disconnected
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                _ => {}
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        driver_clone.disconnect(dev_index, can_index);
    });
    state.can_reader = Some(reader);
}

fn stop_canalyst_reader(state: &mut AppState) {
    if let Some(mut reader) = state.can_reader.take() {
        reader.stop();
    }
    state.canalyst_write_tx = None;
}

// ── VCI <-> CanFrameData conversion ──
fn vci_to_can_frame(obj: &VciCanObj, channel: u8) -> CanFrameData {
    CanFrameData {
        timestamp: chrono::Utc::now().timestamp_millis(),
        id: obj.id,
        is_ext: obj.extern_flag != 0,
        dlc: obj.data_len,
        data: obj.data[..obj.data_len as usize].to_vec(),
        is_error: false,
        is_tx: false,
        channel,
    }
}

fn canalyst_frame_to_vci(frame: &CanFrameData) -> VciCanObj {
    let mut obj = VciCanObj {
        id: frame.id,
        timestamp: 0,
        time_flag: 0,
        send_type: 0, // normal send with auto-retry
        remote_flag: 0,
        extern_flag: if frame.is_ext { 1 } else { 0 },
        data_len: frame.dlc,
        data: [0u8; 8],
        reserved: [0u8; 3],
    };
    let len = frame.data.len().min(8);
    obj.data[..len].copy_from_slice(&frame.data[..len]);
    obj
}

// ── Statistics ──
struct CanStats { error_count: u64, rx_count: u64, tx_count: u64, unique_ids: usize, max_id: u32, bus_load: f64 }

fn compute_stats(frames: &[CanFrameData]) -> CanStats {
    let error_count = frames.iter().filter(|f| f.is_error).count() as u64;
    let rx_count = frames.iter().filter(|f| !f.is_tx && !f.is_error).count() as u64;
    let tx_count = frames.iter().filter(|f| f.is_tx && !f.is_error).count() as u64;
    let mut ids = std::collections::HashSet::new();
    let mut max_id = 0u32;
    for f in frames { ids.insert(f.id); if f.id > max_id { max_id = f.id; } }
    let time_span = if frames.len() >= 2 {
        (frames.last().unwrap().timestamp - frames.first().unwrap().timestamp) as f64
    } else { 0.0 };
    // CAN frame overhead: SOF(1) + ID(11/29) + RTR(1) + DLC(4) + CRC(15) + ACK(2) + EOF(7) = 47 or 67
    // Plus stuff bits (worst case ~20% overhead), use 20 as approximation
    let n = frames.len() as u64;
    let total_data_bits: u64 = frames.iter().map(|f| 8 * f.dlc as u64).sum();
    let total_bits = n * 47 + total_data_bits + n * 20; // overhead + data + stuff
    let bus_load = if time_span > 0.0 {
        (total_bits as f64 / 500_000.0) / (time_span / 1000.0) * 100.0
    } else { 0.0 };
    CanStats { error_count, rx_count, tx_count, unique_ids: ids.len(), max_id, bus_load: bus_load.min(100.0) }
}

fn get_id_color(id: u32) -> egui::Color32 {
    let hash = id.wrapping_mul(2654435761);
    let r = ((hash >> 0) & 0xFF) as u8;
    let g = ((hash >> 8) & 0xFF) as u8;
    let b = ((hash >> 16) & 0xFF) as u8;
    let r = (r as u16 * 170 / 255 + 80) as u8;
    let g = (g as u16 * 170 / 255 + 80) as u8;
    let b = (b as u16 * 170 / 255 + 80) as u8;
    egui::Color32::from_rgb(r, g, b)
}

// ── Transmit ──
fn can_transmit(state: &mut AppState) {
    let id: u32 = match parse_hex_id(&state.can_tx_id) {
        Some(v) => v,
        None => { state.show_error("CAN TX: invalid ID"); return; }
    };
    let data = match parse_hex_data(&state.can_tx_data) {
        Some(d) => d,
        None => { state.show_error("CAN TX: invalid data"); return; }
    };
    if data.len() > 8 {
        state.show_error("CAN TX: data too long (max 8 bytes)");
        return;
    }
    let is_ext = state.can_tx_ext;

    match state.can_connection_mode {
        CanConnectionMode::Slcan => {
            // SLCAN: send text command through serial write channel
            if state.can_write_tx.is_none() {
                state.show_error(T::start_first(state.language));
                return;
            }
            let cmd = if state.can_tx_remote {
                if is_ext { format!("R{:08X}{}\r", id, data.len()) }
                else { format!("r{:03X}{}\r", id, data.len()) }
            } else if is_ext {
                format!("T{:08X}{}\r", id, data.iter().map(|b| format!("{:02X}", b)).collect::<String>())
            } else {
                format!("t{:03X}{}\r", id, data.iter().map(|b| format!("{:02X}", b)).collect::<String>())
            };
            let tx_data = cmd.into_bytes();
            let send_ok = if let Some(ref write_tx) = state.can_write_tx {
                write_tx.send(tx_data).is_ok()
            } else { false };
            if !send_ok {
                state.show_error("CAN TX: send failed (channel closed)");
                return;
            }
        }
        CanConnectionMode::Canalyst => {
            // CANalyst-II: send VciCanObj through dedicated channel
            if state.canalyst_write_tx.is_none() {
                state.show_error(T::start_first(state.language));
                return;
            }
            let frame = CanFrameData {
                timestamp: 0, id, is_ext,
                dlc: data.len() as u8, data: data.clone(),
                is_error: false, is_tx: true,
                channel: state.canalyst_channel + 1,
            };
            let send_ok = if let Some(ref write_tx) = state.canalyst_write_tx {
                write_tx.send(frame).is_ok()
            } else { false };
            if !send_ok {
                state.show_error("CAN TX: send failed (channel closed)");
                return;
            }
        }
    }

    state.can_frames.push(CanFrameData {
        timestamp: chrono::Utc::now().timestamp_millis(),
        id, is_ext, dlc: data.len() as u8, data, is_error: false, is_tx: true, channel: state.can_tx_channel,
    });
    state.add_log_entry(crate::state::LogLevel::Info, &format!("CAN TX: ID={:X} Data={}", id, state.can_tx_data));
}

// ── Export ──
fn export_can_frames(state: &mut AppState) {
    if state.can_frames.is_empty() { return; }
    if let Some(path) = rfd::FileDialog::new().add_filter("CSV", &["csv"]).save_file() {
        let mut content = String::from("index,timestamp,channel,direction,id,ext,dlc,data\n");
        for (i, f) in state.can_frames.iter().enumerate() {
            let ts = chrono::DateTime::from_timestamp_millis(f.timestamp)
                .map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S%.3f").to_string())
                .unwrap_or_default();
            let data_str = f.data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
            let dir = if f.is_tx { "TX" } else { "RX" };
            content.push_str(&format!("{},{},ch{},{},{:X},{},{},{}\n",
                i + 1, ts, f.channel, dir, f.id, f.is_ext, f.dlc, data_str));
        }
        if let Err(e) = std::fs::write(&path, content) {
            state.add_log_entry(crate::state::LogLevel::Error, &format!("Export failed: {}", e));
        } else {
            state.add_log_entry(crate::state::LogLevel::Info, &format!("Exported {} frames to {}", state.can_frames.len(), path.display()));
        }
    }
}

// ── SLCAN Parser ──
fn parse_slcan_line(line: &str) -> Option<CanFrameData> {
    let line = line.trim();
    if line.is_empty() { return None; }
    let (is_ext, is_error, rest) = if let Some(r) = line.strip_prefix('T') {
        (true, false, r)
    } else if let Some(r) = line.strip_prefix('t') {
        (false, false, r)
    } else if let Some(r) = line.strip_prefix('E') {
        (false, true, r)
    } else {
        return None;
    };
    let id_len = if is_ext { 8 } else { 3 };
    if rest.len() < id_len + 1 { return None; }
    let id_hex = &rest[..id_len];
    let id = u32::from_str_radix(id_hex, 16).ok()?;
    let dlc_char = rest.as_bytes()[id_len] as char;
    let dlc = dlc_char.to_digit(10)? as u8;
    let data_str = &rest[id_len + 1..];
    let mut data = Vec::new();
    for i in (0..data_str.len()).step_by(2) {
        if i + 2 <= data_str.len() {
            if let Ok(b) = u8::from_str_radix(&data_str[i..i + 2], 16) {
                data.push(b);
            }
        }
    }
    Some(CanFrameData {
        timestamp: chrono::Utc::now().timestamp_millis(),
        id, is_ext, dlc, data, is_error, is_tx: false, channel: 1,
    })
}

fn parse_hex_id(s: &str) -> Option<u32> {
    let s = s.trim().replace(' ', "").replace("0x", "").replace("0X", "");
    if s.is_empty() { return None; }
    u32::from_str_radix(&s, 16).ok()
}

fn parse_hex_data(s: &str) -> Option<Vec<u8>> {
    let s = s.trim().replace(' ', "").replace("0x", "").replace("0X", "");
    if s.is_empty() { return Some(Vec::new()); }
    if s.len() % 2 != 0 { return None; }
    (0..s.len()).step_by(2).filter_map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect::<Vec<_>>().into()
}

/// Format baud rate as human-readable: 500K, 1000K, etc.
fn baud_display(rate: u32) -> String {
    if rate >= 1_000_000 {
        format!("{}M", rate / 1_000_000)
    } else if rate >= 1_000 {
        format!("{}K", rate / 1_000)
    } else {
        format!("{}", rate)
    }
}
