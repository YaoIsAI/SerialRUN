use crate::state::{AppState, Language, LogLevel, T};
use crate::theme;
use eframe::egui;

pub fn render_log_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    let c = theme::get_colors(state.theme);

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(T::log_viewer(lang)).strong().color(c.text_primary));
        ui.separator();
        let total = state.log_entries.len();
        let filtered = if state.log_search.is_empty() && state.log_level_filter.is_none() {
            total
        } else {
            state.log_entries.iter().filter(|e| log_matches(e, &state.log_search, state.log_level_filter)).count()
        };
        ui.label(egui::RichText::new(format!("{}/{}", filtered, total)).color(c.text_muted));
    });

    // Search and filter bar
    ui.horizontal(|ui| {
        let search_label = if lang == Language::Chinese { "搜索" } else { "Search" };
        ui.label(egui::RichText::new(search_label).color(c.text_muted).small());
        ui.add(egui::TextEdit::singleline(&mut state.log_search).desired_width(150.0).hint_text(search_label));

        ui.separator();

        let filter_label = if lang == Language::Chinese { "级别" } else { "Level" };
        ui.label(egui::RichText::new(filter_label).color(c.text_muted).small());
        egui::ComboBox::from_id_salt("log_level_filter").width(70.0).selected_text(
            match state.log_level_filter {
                None => if lang == Language::Chinese { "全部" } else { "All" },
                Some(LogLevel::Info) => "INFO",
                Some(LogLevel::Warning) => "WARN",
                Some(LogLevel::Error) => "ERROR",
            }
        ).show_ui(ui, |ui| {
            ui.selectable_value(&mut state.log_level_filter, None, if lang == Language::Chinese { "全部" } else { "All" });
            ui.selectable_value(&mut state.log_level_filter, Some(LogLevel::Info), "INFO");
            ui.selectable_value(&mut state.log_level_filter, Some(LogLevel::Warning), "WARN");
            ui.selectable_value(&mut state.log_level_filter, Some(LogLevel::Error), "ERROR");
        });

        if !state.log_search.is_empty() || state.log_level_filter.is_some() {
            let clear_label = if lang == Language::Chinese { "清除" } else { "Clear" };
            if ui.small_button(clear_label).clicked() {
                state.log_search.clear();
                state.log_level_filter = None;
            }
        }
    });

    ui.separator();

    let available_height = ui.available_height() - 40.0;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .max_height(available_height)
        .show(ui, |ui| {
            for entry in &state.log_entries {
                if !log_matches(entry, &state.log_search, state.log_level_filter) {
                    continue;
                }
                let (color, level_str) = match entry.level {
                    LogLevel::Info => (c.log_info, "INFO"),
                    LogLevel::Warning => (c.log_warning, "WARN"),
                    LogLevel::Error => (c.log_error, "ERR "),
                };

                let timestamp = chrono::DateTime::from_timestamp_millis(entry.timestamp)
                    .map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S%.3f").to_string())
                    .unwrap_or_default();

                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new(format!("[{}]", timestamp)).color(c.timestamp_color).monospace());
                    ui.label(egui::RichText::new(level_str).color(color).strong());
                    ui.label(egui::RichText::new(&entry.message).color(c.text_primary));
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

use crate::state::LogEntry;

fn log_matches(entry: &LogEntry, search: &str, level_filter: Option<LogLevel>) -> bool {
    if let Some(level) = level_filter {
        if entry.level != level { return false; }
    }
    if !search.is_empty() {
        let search_lower = search.to_lowercase();
        if !entry.message.to_lowercase().contains(&search_lower) {
            return false;
        }
    }
    true
}
