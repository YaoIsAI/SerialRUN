use crate::state::{AppState, CanFrameData, T};
use crate::async_utils::PersistentReader;
use eframe::egui;
use std::collections::HashMap;

pub fn render_can_analyzer_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    // Poll CAN TX result
    if let Some(ref rx) = state.can_tx_async {
        if let Ok(result) = rx.try_recv() {
            state.can_tx_async = None;
            if let Err(e) = result {
                state.add_log_entry(crate::state::LogLevel::Error, &format!("CAN TX error: {}", e));
            }
        }
    }

    // Poll persistent reader
    if let Some(ref reader) = state.can_reader {
        while let Some(frames) = reader.poll() {
            state.can_frames.extend(frames);
            if state.can_frames.len() > 2000 {
                state.can_frames.drain(..state.can_frames.len() - 2000);
            }
        }
    }

    // ── Header ──
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("CAN Bus Analyzer").strong().size(14.0));
        ui.separator();
        let label = if state.can_capturing { T::stop_monitor(lang) } else { T::start_monitor(lang) };
        if ui.button(label).clicked() {
            state.can_capturing = !state.can_capturing;
            if state.can_capturing {
                state.can_frames.clear();
                state.can_stats = crate::state::CanStats::default();
                // Start persistent reader
                start_can_reader(state);
            } else {
                // Stop persistent reader
                stop_can_reader(state);
            }
        }
        if ui.button(T::clear(lang)).clicked() {
            state.can_frames.clear();
            state.can_stats = crate::state::CanStats::default();
        }
        if ui.button("Export").clicked() {
            export_can_frames(state);
        }
    });
    ui.add_space(2.0);

    // ── Stats Bar ──
    let stats = compute_stats(&state.can_frames);
    ui.horizontal(|ui| {
        ui.label(format!("Frames: {}", state.can_frames.len()));
        ui.separator();
        ui.label(format!("Errors: {}", stats.error_count));
        ui.separator();
        ui.label(format!("IDs: {}", stats.unique_ids));
        ui.separator();
        ui.label(format!("Bus Load: {:.1}%", stats.bus_load));
        ui.separator();
        ui.label(format!("Max ID: {:X}", stats.max_id));
    });
    ui.add_space(2.0);

    // ── Filter & Transmit ──
    ui.horizontal(|ui| {
        ui.label("Filter:");
        ui.add(egui::TextEdit::singleline(&mut state.can_filter_id).desired_width(80.0));
        ui.separator();
        ui.label("TX ID:");
        ui.add(egui::TextEdit::singleline(&mut state.can_tx_id).desired_width(70.0));
        ui.label("Data:");
        ui.add(egui::TextEdit::singleline(&mut state.can_tx_data).desired_width(120.0));
        if ui.button("Send").clicked() && state.is_connected {
            can_transmit(state);
        }
    });
    ui.add_space(2.0);

    // ── Tabs: Frames / Statistics ──
    egui::ComboBox::from_id_salt("can_tab").width(100.0)
        .selected_text(if state.can_show_stats { "Statistics" } else { "Frames" })
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.can_show_stats, false, "Frames");
            ui.selectable_value(&mut state.can_show_stats, true, "Statistics");
        });
    ui.add_space(2.0);

    if state.can_show_stats {
        render_statistics(ui, &state.can_frames);
    } else {
        render_frame_list(ui, state);
    }
}

fn start_can_reader(state: &mut AppState) {
    if state.can_reader.is_some() { return; }
    let port_name = state.selected_port.clone().unwrap_or_default();
    let baud_rate = state.config.baud_rate;
    let reader = PersistentReader::start(move |stop, tx| {
        let config = serialrun_core::config::SerialConfig {
            port_name,
            baud_rate,
            ..Default::default()
        };
        let mut port = serialrun_core::SerialPort::new(config);
        if port.connect().is_err() { return; }
        let mut line_buf = String::new();
        let mut buf = [0u8; 1024];
        while !stop.load(std::sync::atomic::Ordering::Relaxed) {
            match port.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let text = String::from_utf8_lossy(&buf[..n]);
                    line_buf.push_str(&text);
                    let mut frames = Vec::new();
                    while let Some(pos) = line_buf.find('\r') {
                        let line = line_buf[..pos].to_string();
                        line_buf = line_buf[pos + 1..].to_string();
                        if let Some(frame) = parse_slcan_line(&line) {
                            frames.push(frame);
                        }
                    }
                    if !frames.is_empty() {
                        let _ = tx.send(frames);
                    }
                }
                _ => {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
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
}

fn render_frame_list(ui: &mut egui::Ui, state: &mut AppState) {
    let filter = parse_hex_id(&state.can_filter_id);
    egui::ScrollArea::vertical().max_height(220.0).stick_to_bottom(true).show(ui, |ui| {
        let mut prev_ts: Option<i64> = None;
        for frame in &state.can_frames {
            if let Some(filt) = filter {
                if frame.id != filt { continue; }
            }
            let id_str = if frame.is_ext { format!("{:08X}", frame.id) } else { format!("{:03X}", frame.id) };
            let data_str = frame.data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
            let id_color = get_id_color(frame.id);
            let ts = chrono::DateTime::from_timestamp_millis(frame.timestamp)
                .map(|t| t.with_timezone(&chrono::Local).format("%H:%M:%S%.3f").to_string())
                .unwrap_or_default();
            let delta = prev_ts.map(|p| frame.timestamp - p).filter(|&d| d >= 0);
            prev_ts = Some(frame.timestamp);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("[{}]", ts)).weak().monospace().small());
                if let Some(d) = delta {
                    let delta_color = if d < 10 { egui::Color32::from_rgb(0, 180, 0) }
                        else if d < 100 { egui::Color32::from_rgb(180, 180, 0) }
                        else { egui::Color32::from_rgb(180, 60, 60) };
                    ui.label(egui::RichText::new(format!("+{}ms", d)).color(delta_color).monospace().small());
                } else {
                    ui.label(egui::RichText::new("      ").monospace().small());
                }
                let dir = if frame.is_error { "ERR" } else { "RX " };
                let dir_color = if frame.is_error { egui::Color32::RED } else { egui::Color32::LIGHT_GREEN };
                ui.label(egui::RichText::new(dir).color(dir_color).monospace().small());
                ui.label(egui::RichText::new(&id_str).color(id_color).monospace().small().strong());
                let ext_text = if frame.is_ext { "EXT" } else { "   " };
                let ext_color = if frame.is_ext { egui::Color32::from_rgb(150, 150, 255) } else { egui::Color32::TRANSPARENT };
                ui.label(egui::RichText::new(ext_text).color(ext_color).monospace().small());
                ui.label(egui::RichText::new(format!("DLC={}", frame.dlc)).monospace().small());
                ui.label(egui::RichText::new(&data_str).monospace().small());
            });
        }
    });
}

fn render_statistics(ui: &mut egui::Ui, frames: &[CanFrameData]) {
    let stats = compute_per_id_stats(frames);
    if stats.is_empty() {
        ui.label(egui::RichText::new("No data").weak());
        return;
    }

    egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
        egui::Grid::new("can_stats_grid").striped(true).show(ui, |ui| {
            ui.label(egui::RichText::new("ID").strong().small());
            ui.label(egui::RichText::new("Count").strong().small());
            ui.label(egui::RichText::new("Freq").strong().small());
            ui.label(egui::RichText::new("Min Δ").strong().small());
            ui.label(egui::RichText::new("Max Δ").strong().small());
            ui.label(egui::RichText::new("Avg Δ").strong().small());
            ui.label(egui::RichText::new("Last Data").strong().small());
            ui.end_row();

            for (id, stat) in stats.iter().rev() {
                let id_str = if *id > 0x7FF { format!("{:08X}", id) } else { format!("{:03X}", id) };
                let color = get_id_color(*id);
                ui.label(egui::RichText::new(&id_str).color(color).monospace().small().strong());
                ui.label(egui::RichText::new(format!("{}", stat.count)).monospace().small());
                let freq = if stat.time_span_ms > 0.0 { stat.count as f64 / (stat.time_span_ms / 1000.0) } else { 0.0 };
                ui.label(egui::RichText::new(format!("{:.1} Hz", freq)).monospace().small());
                ui.label(egui::RichText::new(format!("{:.1}ms", stat.min_delta)).monospace().small());
                ui.label(egui::RichText::new(format!("{:.1}ms", stat.max_delta)).monospace().small());
                ui.label(egui::RichText::new(format!("{:.1}ms", stat.avg_delta)).monospace().small());
                let data_str = stat.last_data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                ui.label(egui::RichText::new(&data_str).monospace().small());
                ui.end_row();
            }
        });
    });
}

struct CanStats {
    error_count: u64,
    unique_ids: usize,
    max_id: u32,
    bus_load: f64,
}

struct PerIdStat {
    count: u64,
    last_data: Vec<u8>,
    min_delta: f64,
    max_delta: f64,
    avg_delta: f64,
    time_span_ms: f64,
}

fn compute_stats(frames: &[CanFrameData]) -> CanStats {
    let error_count = frames.iter().filter(|f| f.is_error).count() as u64;
    let mut ids = std::collections::HashSet::new();
    let mut max_id = 0u32;
    for f in frames {
        ids.insert(f.id);
        if f.id > max_id { max_id = f.id; }
    }
    let time_span = if frames.len() >= 2 {
        (frames.last().unwrap().timestamp - frames.first().unwrap().timestamp) as f64
    } else { 0.0 };
    let bits_per_frame = 47 + 20 + frames.iter().map(|f| 8 * f.dlc as u64).sum::<u64>();
    let total_bits = bits_per_frame * frames.len() as u64;
    let bus_load = if time_span > 0.0 {
        (total_bits as f64 / 500_000.0) / (time_span / 1000.0) * 100.0
    } else { 0.0 };

    CanStats { error_count, unique_ids: ids.len(), max_id, bus_load: bus_load.min(100.0) }
}

fn compute_per_id_stats(frames: &[CanFrameData]) -> Vec<(u32, PerIdStat)> {
    let mut map: HashMap<u32, Vec<&CanFrameData>> = HashMap::new();
    for f in frames { map.entry(f.id).or_default().push(f); }

    let mut result: Vec<(u32, PerIdStat)> = map.iter().map(|(&id, frs)| {
        let count = frs.len() as u64;
        let last_data = frs.last().map(|f| f.data.clone()).unwrap_or_default();
        let mut deltas = Vec::new();
        for w in frs.windows(2) {
            deltas.push((w[1].timestamp - w[0].timestamp) as f64);
        }
        let min_delta = deltas.iter().copied().fold(f64::INFINITY, f64::min);
        let max_delta = deltas.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let avg_delta = if !deltas.is_empty() { deltas.iter().sum::<f64>() / deltas.len() as f64 } else { 0.0 };
        let time_span = if frs.len() >= 2 {
            (frs.last().unwrap().timestamp - frs.first().unwrap().timestamp) as f64
        } else { 0.0 };

        (id, PerIdStat { count, last_data, min_delta, max_delta, avg_delta, time_span_ms: time_span })
    }).collect();
    result.sort_by(|a, b| b.1.count.cmp(&a.1.count));
    result
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

fn can_transmit(state: &mut AppState) {
    let id: u32 = match parse_hex_id(&state.can_tx_id) {
        Some(v) => v,
        None => { state.add_log_entry(crate::state::LogLevel::Error, "CAN TX: invalid ID"); return; }
    };
    let data = match parse_hex_data(&state.can_tx_data) {
        Some(d) => d,
        None => { state.add_log_entry(crate::state::LogLevel::Error, "CAN TX: invalid data"); return; }
    };
    if data.len() > 8 {
        state.add_log_entry(crate::state::LogLevel::Error, "CAN TX: data too long (max 8 bytes)");
        return;
    }
    let is_ext = id > 0x7FF;
    let cmd = if is_ext {
        format!("T{:08X}{}\r", id, data.iter().map(|b| format!("{:02X}", b)).collect::<String>())
    } else {
        format!("t{:03X}{}\r", id, data.iter().map(|b| format!("{:02X}", b)).collect::<String>())
    };
    let port_name = state.selected_port.clone().unwrap_or_default();
    let baud_rate = state.config.baud_rate;
    let tx_data = cmd.into_bytes();
    state.can_tx_async = Some(crate::async_utils::spawn_serial_write(port_name, baud_rate, tx_data));
    // Log as TX immediately (optimistic)
    state.can_frames.push(CanFrameData {
        timestamp: chrono::Utc::now().timestamp_millis(),
        id, is_ext, dlc: data.len() as u8, data, is_error: false,
    });
    state.add_log_entry(crate::state::LogLevel::Info, &format!("CAN TX: ID={:X} Data={}", id, state.can_tx_data));
}

fn export_can_frames(state: &mut AppState) {
    if state.can_frames.is_empty() { return; }
    if let Some(path) = rfd::FileDialog::new().add_filter("CSV", &["csv"]).save_file() {
        let mut content = String::from("timestamp,id,ext,dlc,data,error\n");
        for f in &state.can_frames {
            let ts = chrono::DateTime::from_timestamp_millis(f.timestamp)
                .map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S%.3f").to_string())
                .unwrap_or_default();
            let data_str = f.data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
            content.push_str(&format!("{},{:X},{},{},{},\"{}\"\n",
                ts, f.id, f.is_ext, f.dlc, data_str, f.is_error));
        }
        if let Err(e) = std::fs::write(&path, content) {
            state.add_log_entry(crate::state::LogLevel::Error, &format!("Export failed: {}", e));
        } else {
            state.add_log_entry(crate::state::LogLevel::Info, &format!("Exported {} frames to {}", state.can_frames.len(), path.display()));
        }
    }
}

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
        id, is_ext, dlc, data, is_error,
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
