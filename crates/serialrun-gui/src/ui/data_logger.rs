use crate::state::{AppState, T};
use eframe::egui;
use std::fs::File;
use std::io::Write;

pub fn render_data_logger_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    ui.horizontal(|ui| {
        ui.label(T::log_label(lang));
        if ui.button("...").clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("CSV", &["csv"]).save_file() {
                state.data_logger_path = path.display().to_string();
                state.add_log_entry(crate::state::LogLevel::Info, &format!("Log path: {}", path.display()));
            }
        }
        ui.label(&state.data_logger_path);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.button(egui::RichText::new("?").size(14.0).strong())
                .on_hover_text(T::data_logger_tip_header(lang));
        });
    });
    ui.add_space(4.0);

    let label = if state.data_logger_recording { T::stop_recording(lang) } else { T::start_recording(lang) };
    if ui.button(label).clicked() {
        state.data_logger_recording = !state.data_logger_recording;
        if state.data_logger_recording && !state.data_logger_path.is_empty() {
            if let Ok(mut f) = File::create(&state.data_logger_path) {
                if writeln!(f, "timestamp,source,direction,raw_hex,raw_bytes").is_err() {
                    state.add_log_entry(crate::state::LogLevel::Error, "Failed to write CSV header");
                }
            }
            state.data_logger_buffered = 0;
            state.add_log_entry(crate::state::LogLevel::Info, "Data logging started");
        } else if !state.data_logger_recording {
            state.add_log_entry(crate::state::LogLevel::Info, &format!("Data logging stopped. {} records in this session.", state.data_logger_buffered));
        }
    }

    ui.add_space(4.0);
    if state.data_logger_recording {
        ui.label(egui::RichText::new(format!("Recording... {} records", state.data_logger_buffered)).color(egui::Color32::RED));
        ui.label(egui::RichText::new("All RX/TX data will be logged to CSV").weak());
    } else if !state.data_logger_path.is_empty() {
        ui.label(egui::RichText::new(format!("Total records: {}", state.data_logger_buffered)).weak());
    }
}

pub fn log_data(state: &mut AppState, direction: &str, data: &[u8]) {
    if !state.data_logger_recording || state.data_logger_path.is_empty() { return; }
    let ts = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    let hex = data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
    if let Ok(mut f) = File::options().create(true).append(true).open(&state.data_logger_path) {
        let _ = writeln!(f, "{},{},{},\"{}\",{}", ts, "terminal", direction, hex, data.len());
    }
    state.data_logger_buffered += 1;
}
