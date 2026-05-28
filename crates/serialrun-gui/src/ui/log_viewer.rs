use crate::state::{AppState, LogLevel, T};
use eframe::egui;

pub fn render_log_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    let is_dark = state.theme == crate::state::Theme::Dark;

    ui.horizontal(|ui| {
        ui.label(T::log_viewer(lang));
        ui.separator();
        ui.label(format!("Entries: {}", state.log_entries.len()));
    });

    ui.separator();

    let available_height = ui.available_height() - 40.0;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .max_height(available_height)
        .show(ui, |ui| {
            for entry in &state.log_entries {
                let color = match entry.level {
                    LogLevel::Info => if is_dark { egui::Color32::WHITE } else { egui::Color32::BLACK },
                    LogLevel::Warning => egui::Color32::YELLOW,
                    LogLevel::Error => egui::Color32::RED,
                };

                let level_str = match entry.level {
                    LogLevel::Info => "INFO",
                    LogLevel::Warning => "WARN",
                    LogLevel::Error => "ERR ",
                };

                let timestamp = chrono::DateTime::from_timestamp_millis(entry.timestamp)
                    .map(|t| t.with_timezone(&chrono::Local).format("%H:%M:%S%.3f").to_string())
                    .unwrap_or_default();

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("[{}]", timestamp)).weak());
                    ui.label(egui::RichText::new(level_str).color(color).strong());
                    ui.label(&entry.message);
                });
            }
        });

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button(T::clear_logs(lang)).clicked() {
            state.log_entries.clear();
        }

        if ui.button(T::export_logs(lang)).clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("CSV", &["csv"]).save_file() {
                let mut content = String::from("timestamp,level,message\n");
                for entry in &state.log_entries {
                    let ts = chrono::DateTime::from_timestamp_millis(entry.timestamp)
                        .map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S%.3f").to_string())
                        .unwrap_or_default();
                    let level = match entry.level {
                        LogLevel::Info => "INFO",
                        LogLevel::Warning => "WARN",
                        LogLevel::Error => "ERROR",
                    };
                    content.push_str(&format!("{},{},\"{}\"\n", ts, level, entry.message.replace('"', "\"\"")));
                }
                if let Err(e) = std::fs::write(&path, content) {
                    state.add_log_entry(LogLevel::Error, &format!("Export failed: {}", e));
                } else {
                    state.add_log_entry(LogLevel::Info, &format!("Logs exported to {}", path.display()));
                }
            }
        }
    });
}
