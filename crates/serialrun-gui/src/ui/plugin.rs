use crate::state::{AppState, PluginInfo, T};
use eframe::egui;

pub fn render_plugin_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(T::plugins(lang)).strong());
        ui.separator();
        if ui.button("Refresh").clicked() {
            discover_plugins(state);
        }
    });
    ui.add_space(4.0);

    ui.label("Place .dylib/.so/.dll plugins in the plugins/ directory next to the executable.");
    ui.add_space(4.0);

    if state.plugins.is_empty() {
        ui.label(egui::RichText::new("No plugins found.").weak());
    } else {
        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            for plugin in &mut state.plugins {
                ui.horizontal(|ui| {
                    let status_color = if plugin.loaded { egui::Color32::GREEN } else { egui::Color32::GRAY };
                    ui.label(egui::RichText::new(if plugin.loaded { "●" } else { "○" }).color(status_color));
                    ui.label(egui::RichText::new(&plugin.name).strong());
                    ui.label(egui::RichText::new(format!("v{}", plugin.version)).weak());
                    ui.label(egui::RichText::new(&plugin.author).weak());
                });
                ui.separator();
            }
        });
    }
}

fn discover_plugins(state: &mut AppState) {
    state.plugins.clear();
    // Look for plugins directory relative to executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let plugins_dir = exe_dir.join("plugins");
            if plugins_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                        let valid_ext = match std::env::consts::OS {
                            "macos" => ext == "dylib",
                            "windows" => ext == "dll",
                            _ => ext == "so",
                        };
                        if valid_ext {
                            let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
                            state.plugins.push(PluginInfo {
                                name,
                                version: "0.1.0".into(),
                                author: "Unknown".into(),
                                loaded: false,
                            });
                        }
                    }
                }
            }
        }
    }
    if state.plugins.is_empty() {
        state.add_log_entry(crate::state::LogLevel::Info, "No plugins found in plugins/ directory");
    } else {
        state.add_log_entry(crate::state::LogLevel::Info, &format!("Found {} plugin(s)", state.plugins.len()));
    }
}
