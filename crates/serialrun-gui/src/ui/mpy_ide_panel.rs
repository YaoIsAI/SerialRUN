/// MicroPython IDE Panel - standalone floating window
/// Provides a Thonny-like IDE experience for MicroPython devices

use crate::state::AppState;
use crate::app::get_loaded_plugins;
use eframe::egui::{self, Color32, RichText, Stroke, Vec2, Margin, Rounding};
use serialrun_plugin_api::PluginResult;

/// Render the MicroPython IDE as a standalone floating window
pub fn render_mpy_ide_window(ctx: &egui::Context, state: &mut AppState) {
    if !state.show_mpy_ide_window {
        return;
    }

    let mut open = state.show_mpy_ide_window;
    egui::Window::new("MicroPython IDE")
        .id(egui::Id::new("mpy_ide_window"))
        .open(&mut open)
        .default_size([1200.0, 800.0])
        .min_size([800.0, 600.0])
        .resizable(true)
        .collapsible(true)
        .title_bar(true)
        .show(ctx, |ui| {
            render_ide_content(ui, state);
        });

    state.show_mpy_ide_window = open;
}

fn render_ide_content(ui: &mut egui::Ui, state: &mut AppState) {
    let available = ui.available_size();

    // Toolbar
    ui.horizontal(|ui| {
        // Connection status
        let connected = state.is_connected;
        let status_color = if connected { Color32::from_rgb(34, 197, 94) } else { Color32::from_rgb(239, 68, 68) };
        let status_text = if connected { "● Connected" } else { "○ Disconnected" };
        ui.label(RichText::new(status_text).color(status_color).strong());

        ui.separator();

        // Device info
        if let Some(ref info) = state.mpy_device_info {
            ui.label(RichText::new(info).color(Color32::from_rgb(150, 150, 160)).small());
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("❌ Close").clicked() {
                state.show_mpy_ide_window = false;
            }
        });
    });

    ui.separator();

    // Main content: left panel (files) + right panel (editor + REPL)
    let left_width = 250.0;
    let right_width = available.x - left_width - 16.0;

    ui.horizontal(|ui| {
        // Left panel: File browser
        ui.allocate_ui(Vec2::new(left_width, available.y - 40.0), |ui| {
            render_file_browser(ui, state);
        });

        ui.separator();

        // Right panel: Editor + REPL
        ui.allocate_ui(Vec2::new(right_width, available.y - 40.0), |ui| {
            ui.vertical(|ui| {
                // Code editor (60% height)
                let editor_height = (available.y - 40.0) * 0.55;
                ui.allocate_ui(Vec2::new(right_width, editor_height), |ui| {
                    render_code_editor(ui, state);
                });

                ui.separator();

                // REPL terminal (40% height)
                let repl_height = available.y - 40.0 - editor_height - 20.0;
                ui.allocate_ui(Vec2::new(right_width, repl_height), |ui| {
                    render_repl_terminal(ui, state);
                });
            });
        });
    });
}

// ============================================================================
// File Browser
// ============================================================================

fn render_file_browser(ui: &mut egui::Ui, state: &mut AppState) {
    egui::Frame::none()
        .fill(Color32::from_rgb(25, 25, 35))
        .stroke(Stroke::new(1.0, Color32::from_rgb(50, 50, 60)))
        .rounding(Rounding::same(4.0))
        .inner_margin(Margin::same(8.0))
        .show(ui, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(RichText::new("📁 Files").strong().color(Color32::from_rgb(200, 200, 210)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("🔄").on_hover_text("Refresh").clicked() {
                        // Refresh file list
                        execute_plugin_command(state, "list_dir", r#"{"path": "/"}"#);
                    }
                    if ui.small_button("📁").on_hover_text("New Folder").clicked() {
                        // TODO: New folder dialog
                    }
                    if ui.small_button("📄").on_hover_text("New File").clicked() {
                        // TODO: New file dialog
                    }
                });
            });
            ui.separator();

            // File tree
            let has_entries = state.plugin_ui_file_tree
                .get("serialrun-mpy-ide")
                .map(|e| !e.is_empty())
                .unwrap_or(false);

            if !has_entries {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(RichText::new("📁 No files loaded").color(Color32::from_rgb(100, 100, 110)));
                    ui.label(RichText::new("Connect to a device and click Refresh").color(Color32::from_rgb(80, 80, 90)).small());
                });
            } else {
                // Clone entries to avoid borrow issues
                let mut entries = state.plugin_ui_file_tree
                    .get("serialrun-mpy-ide")
                    .cloned()
                    .unwrap_or_default();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_file_entries(ui, &mut entries, 0, state);
                });

                // Write back entries
                state.plugin_ui_file_tree.insert("serialrun-mpy-ide".to_string(), entries);
            }
        });
}

fn render_file_entries(
    ui: &mut egui::Ui,
    entries: &mut Vec<crate::ui::plugin_ui::FileEntry>,
    depth: usize,
    state: &mut AppState,
) {
    let indent = depth as f32 * 16.0;

    for entry in entries.iter_mut() {
        ui.horizontal(|ui| {
            ui.add_space(indent);

            let icon = if entry.is_dir {
                if entry.expanded { "📂" } else { "📁" }
            } else {
                if entry.name.ends_with(".py") { "🐍" }
                else if entry.name.ends_with(".json") { "📋" }
                else if entry.name.ends_with(".txt") { "📝" }
                else { "📄" }
            };

            let label = format!("{} {}", icon, entry.name);
            let response = ui.label(RichText::new(label).color(Color32::from_rgb(200, 200, 210)));

            if entry.is_dir && response.clicked() {
                entry.expanded = !entry.expanded;
                if entry.expanded {
                    let path = entry.path.clone();
                    execute_plugin_command(state, "list_dir", &format!(r#"{{"path": "{}"}}"#, path));
                }
            } else if !entry.is_dir && response.clicked() {
                let path = entry.path.clone();
                let file_name = entry.name.clone();
                let content = execute_plugin_command(state, "read_file", &format!(r#"{{"path": "{}"}}"#, path));
                if let Some(c) = content {
                    state.plugin_ui_editor_content.insert("serialrun-mpy-ide".to_string(), c);
                    state.plugin_ui_editor_file.insert("serialrun-mpy-ide".to_string(), Some(file_name));
                }
            }
        });
    }
}

// ============================================================================
// Code Editor
// ============================================================================

fn render_code_editor(ui: &mut egui::Ui, state: &mut AppState) {
    egui::Frame::none()
        .fill(Color32::from_rgb(30, 30, 40))
        .stroke(Stroke::new(1.0, Color32::from_rgb(50, 50, 60)))
        .rounding(Rounding::same(4.0))
        .inner_margin(Margin::same(8.0))
        .show(ui, |ui| {
            // Toolbar
            ui.horizontal(|ui| {
                let file_name = state.plugin_ui_editor_file
                    .get("serialrun-mpy-ide")
                    .and_then(|f| f.clone())
                    .unwrap_or_else(|| " untitled".to_string());

                ui.label(RichText::new(format!("📝 {}", file_name)).color(Color32::from_rgb(180, 180, 190)));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("▶ Run").color(Color32::from_rgb(34, 197, 94))).clicked() {
                        // Run current file
                        let content = state.plugin_ui_editor_content
                            .get("serialrun-mpy-ide")
                            .cloned()
                            .unwrap_or_default();
                        if !content.is_empty() {
                            execute_plugin_command(state, "execute", &format!(r#"{{"code": "{}"}}"#, content.replace('"', "\\\"").replace('\n', "\\n")));
                        }
                    }
                    if ui.button(RichText::new("💾 Save").color(Color32::from_rgb(59, 130, 246))).clicked() {
                        // Save to device
                        let content = state.plugin_ui_editor_content
                            .get("serialrun-mpy-ide")
                            .cloned()
                            .unwrap_or_default();
                        let file_name = state.plugin_ui_editor_file
                            .get("serialrun-mpy-ide")
                            .and_then(|f| f.as_ref())
                            .cloned()
                            .unwrap_or_else(|| "/main.py".to_string());
                        let path = if file_name.starts_with('/') { file_name } else { format!("/{}", file_name) };
                        execute_plugin_command(state, "write_file", &format!(r#"{{"path": "{}", "content": "{}"}}"#, path, content.replace('"', "\\\"").replace('\n', "\\n")));
                    }
                    if ui.button(RichText::new("⏹ Stop").color(Color32::from_rgb(239, 68, 68))).clicked() {
                        execute_plugin_command(state, "interrupt", "{}");
                    }
                });
            });
            ui.separator();

            // Code editor
            let content = state.plugin_ui_editor_content
                .entry("serialrun-mpy-ide".to_string())
                .or_default();

            let available = ui.available_size();
            egui::ScrollArea::both().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(content)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(available.x - 20.0)
                        .desired_rows(20)
                        .code_editor()
                );
            });
        });
}

// ============================================================================
// REPL Terminal
// ============================================================================

fn render_repl_terminal(ui: &mut egui::Ui, state: &mut AppState) {
    egui::Frame::none()
        .fill(Color32::from_rgb(20, 20, 30))
        .stroke(Stroke::new(1.0, Color32::from_rgb(50, 50, 60)))
        .rounding(Rounding::same(4.0))
        .inner_margin(Margin::same(8.0))
        .show(ui, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(RichText::new("💬 REPL").strong().color(Color32::from_rgb(200, 200, 210)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("🗑 Clear").clicked() {
                        state.plugin_ui_repl_content.insert("serialrun-mpy-ide".to_string(), String::new());
                    }
                    if ui.small_button("⏹ Interrupt").clicked() {
                        execute_plugin_command(state, "interrupt", "{}");
                    }
                    if ui.small_button("🔄 Reset").clicked() {
                        execute_plugin_command(state, "reset", "{}");
                    }
                });
            });
            ui.separator();

            // REPL output
            let available = ui.available_size();
            let content = state.plugin_ui_repl_content
                .entry("serialrun-mpy-ide".to_string())
                .or_default();

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_height(available.y - 50.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(content)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(available.x - 20.0)
                            .desired_rows(10)
                            .interactive(false)
                    );
                });

            // Input line
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new(">>>").color(Color32::from_rgb(80, 200, 120)).monospace());

                // Clone input to avoid borrow issues
                let current_input = state.plugin_ui_repl_input
                    .get("serialrun-mpy-ide")
                    .cloned()
                    .unwrap_or_default();

                let mut new_input = current_input.clone();
                let response = ui.add(
                    egui::TextEdit::singleline(&mut new_input)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(available.x - 80.0)
                        .hint_text("Type Python code...")
                );

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if !new_input.is_empty() {
                        let cmd = new_input.clone();
                        let output = execute_plugin_command(state, "execute", &format!(r#"{{"code": "{}"}}"#, cmd.replace('"', "\\\"")));

                        // Add to REPL history
                        let repl = state.plugin_ui_repl_content
                            .entry("serialrun-mpy-ide".to_string())
                            .or_default();
                        repl.push_str(&format!(">>> {}\n", cmd));
                        if let Some(out) = output {
                            repl.push_str(&out);
                            repl.push('\n');
                        }

                        new_input.clear();
                    }
                    response.request_focus();
                }

                // Update state
                state.plugin_ui_repl_input.insert("serialrun-mpy-ide".to_string(), new_input);
            });
        });
}

// ============================================================================
// Helper: Execute Plugin Command
// ============================================================================

fn execute_plugin_command(state: &mut AppState, command: &str, params: &str) -> Option<String> {
    let plugins = get_loaded_plugins();
    let mut plugins = plugins.lock().unwrap_or_else(|e| e.into_inner());

    if let Some(loaded) = plugins.get_mut("serialrun-mpy-ide") {
        match loaded.execute_command(command, params) {
            Ok(result) => {
                if result.success {
                    result.result.and_then(|v| {
                        // Extract string from JSON value
                        match v {
                            serde_json::Value::String(s) => Some(s),
                            serde_json::Value::Object(map) => {
                                // Try to find a "content", "output", or "entries" field
                                map.get("content")
                                    .or_else(|| map.get("output"))
                                    .or_else(|| map.get("entries"))
                                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                            }
                            _ => Some(serde_json::to_string(&v).unwrap_or_default()),
                        }
                    })
                } else {
                    let error = result.error.unwrap_or_else(|| "Unknown error".to_string());
                    state.add_log_entry(crate::state::LogLevel::Error, &format!("[MPY IDE] {}: {}", command, error));
                    None
                }
            }
            Err(e) => {
                state.add_log_entry(crate::state::LogLevel::Error, &format!("[MPY IDE] {} failed: {}", command, e));
                None
            }
        }
    } else {
        state.add_log_entry(crate::state::LogLevel::Error, "[MPY IDE] Plugin not loaded");
        None
    }
}
