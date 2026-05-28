use crate::async_utils::PersistentReader;
use crate::state::{AppState, ScopeDataPoint, T};
use eframe::egui;

pub fn render_serial_scope_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    // Poll persistent reader
    if let Some(ref reader) = state.scope_reader {
        while let Some(points) = reader.poll() {
            let t0 = state.scope_data.first().map(|p| p.time_ms).unwrap_or(0.0);
            for p in points {
                state.scope_data.push(ScopeDataPoint {
                    time_ms: if t0 == 0.0 { p.time_ms } else { p.time_ms },
                    value: p.value,
                });
            }
            let max_points = 10000;
            if state.scope_data.len() > max_points {
                state.scope_data.drain(..state.scope_data.len() - max_points);
            }
        }
    }

    egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(T::oscilloscope(lang)).strong());
            ui.separator();
            let label = if state.scope_capturing { T::stop_monitor(lang) } else { T::capture(lang) };
            if ui.button(label).clicked() {
                state.scope_capturing = !state.scope_capturing;
                if state.scope_capturing {
                    state.scope_data.clear();
                    start_scope_reader(state);
                } else {
                    stop_scope_reader(state);
                }
            }
            if ui.button(T::clear(lang)).clicked() { state.scope_data.clear(); }
        });
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Timebase (ms):");
            ui.add(egui::DragValue::new(&mut state.scope_timebase_ms).range(1.0..=5000.0).speed(10.0));
        });
        ui.add_space(4.0);

        let available = ui.available_size();
        let height = available.y.max(200.0).min(350.0);
        let (response, painter) = ui.allocate_painter(egui::vec2(available.x, height), egui::Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(15, 15, 25));

        for i in 0..=10 {
            let y = rect.top() + rect.height() * i as f32 / 10.0;
            painter.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], egui::Stroke::new(0.5, egui::Color32::from_rgb(40, 40, 50)));
        }
        for i in 0..=16 {
            let x = rect.left() + rect.width() * i as f32 / 16.0;
            painter.line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], egui::Stroke::new(0.5, egui::Color32::from_rgb(40, 40, 50)));
        }

        if state.scope_data.len() >= 2 {
            let min_val = state.scope_data.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);
            let max_val = state.scope_data.iter().map(|p| p.value).fold(f64::NEG_INFINITY, f64::max);
            let range = (max_val - min_val).max(1.0);
            let first_t = state.scope_data.first().unwrap().time_ms;
            let last_t = state.scope_data.last().unwrap().time_ms;
            let time_range = (last_t - first_t).max(1.0);

            let points: Vec<egui::Pos2> = state.scope_data.iter().map(|p| {
                let x = rect.left() + rect.width() * ((p.time_ms - first_t) / time_range) as f32;
                let y = rect.bottom() - rect.height() * ((p.value - min_val) / range) as f32;
                egui::pos2(x, y)
            }).collect();

            for w in points.windows(2) {
                painter.line_segment([w[0], w[1]], egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 200, 100)));
            }

            painter.text(egui::pos2(rect.left() + 4.0, rect.top() + 4.0), egui::Align2::LEFT_TOP, format!("{:.1}", max_val), egui::FontId::monospace(10.0), egui::Color32::GRAY);
            painter.text(egui::pos2(rect.left() + 4.0, rect.bottom() - 14.0), egui::Align2::LEFT_TOP, format!("{:.1}", min_val), egui::FontId::monospace(10.0), egui::Color32::GRAY);

            if let Some(hover_pos) = response.hover_pos() {
                let frac = ((hover_pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                let hover_t = first_t + frac as f64 * time_range;
                if let Some(closest) = state.scope_data.iter().min_by(|a, b| {
                    (a.time_ms - hover_t).abs().partial_cmp(&(b.time_ms - hover_t).abs()).unwrap_or(std::cmp::Ordering::Equal)
                }) {
                    painter.text(hover_pos + egui::vec2(8.0, -16.0), egui::Align2::LEFT_TOP, format!("T={:.3}ms V={:.1}", closest.time_ms, closest.value), egui::FontId::monospace(11.0), egui::Color32::WHITE);
                }
            }
        } else {
            painter.text(rect.center(), egui::Align2::CENTER_CENTER, "No data", egui::FontId::proportional(14.0), egui::Color32::GRAY);
        }

        ui.horizontal(|ui| { ui.label(format!("Points: {}", state.scope_data.len())); });
    });
}

fn start_scope_reader(state: &mut AppState) {
    if state.scope_reader.is_some() { return; }
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
        let mut buf = [0u8; 1024];
        while !stop.load(std::sync::atomic::Ordering::Relaxed) {
            match port.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let now = chrono::Utc::now().timestamp_millis() as f64;
                    let points: Vec<ScopeDataPoint> = buf[..n].iter().map(|&byte| {
                        ScopeDataPoint { time_ms: now, value: byte as f64 }
                    }).collect();
                    let _ = tx.send(points);
                }
                _ => {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }
        }
        let _ = port.disconnect();
    });
    state.scope_reader = Some(reader);
}

fn stop_scope_reader(state: &mut AppState) {
    if let Some(mut reader) = state.scope_reader.take() {
        reader.stop();
    }
}
