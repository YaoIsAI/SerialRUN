/// STC ISP Flasher - Dedicated plugin panel
///
/// Left-right tab layout:
/// Left: Serial Port + Firmware configuration
/// Right: Chip Info + Actions + Log

use crate::state::{AppState, T};
use eframe::egui;

pub fn render_stc_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    let available = ui.available_size();
    let left_width = available.x * 0.45;
    let right_width = available.x - left_width - 8.0;

    ui.horizontal(|ui| {
        // ═══════════════════════════════════════
        // Left: Configuration
        // ═══════════════════════════════════════
        ui.vertical(|ui| {
            ui.set_min_width(left_width);

            // Serial Port
            ui.label(egui::RichText::new(T::stc_serial_port(lang)).strong());
            ui.add_space(2.0);
            egui::Grid::new("stc_serial").num_columns(2).spacing([4.0, 4.0]).show(ui, |ui| {
                ui.label(T::stc_port_label(lang));
                egui::ComboBox::from_id_salt("stc_port")
                    .width(140.0)
                    .selected_text(if state.stc_port.is_empty() { "—" } else { &state.stc_port })
                    .show_ui(ui, |ui| {
                        for port in &state.ports {
                            let name = port.name.clone();
                            ui.selectable_value(&mut state.stc_port, name.clone(), name);
                        }
                    });
                ui.end_row();

                ui.label(T::stc_baud_label(lang));
                ui.add(egui::DragValue::new(&mut state.stc_baud_rate).range(2400..=115200));
                ui.end_row();
            });
            ui.add_space(8.0);

            // Firmware
            ui.label(egui::RichText::new(T::stc_firmware(lang)).strong());
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                let display = if state.stc_firmware_path.is_empty() {
                    "—".to_string()
                } else {
                    let p = std::path::Path::new(&state.stc_firmware_path);
                    p.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default()
                };
                ui.label(egui::RichText::new(display).monospace());
                if ui.button(T::stc_browse(lang)).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title(T::stc_select_firmware(lang))
                        .add_filter("Firmware", &["hex", "bin"])
                        .add_filter("All Files", &["*"])
                        .pick_file()
                    {
                        state.stc_firmware_path = path.display().to_string();
                    }
                }
            });
            if !state.stc_firmware_path.is_empty() {
                let path = std::path::Path::new(&state.stc_firmware_path);
                if path.exists() {
                    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                    let size_str = if size >= 1024 {
                        format!("{:.1} KB", size as f64 / 1024.0)
                    } else {
                        format!("{} B", size)
                    };
                    ui.label(egui::RichText::new(size_str).weak().small());
                } else {
                    ui.label(egui::RichText::new(T::stc_file_not_found(lang)).color(egui::Color32::RED).small());
                }
            }
        });

        ui.separator();

        // ═══════════════════════════════════════
        // Right: Info + Actions + Log
        // ═══════════════════════════════════════
        ui.vertical(|ui| {
            ui.set_min_width(right_width);

            // Chip Info
            if !state.stc_chip_info.is_empty() {
                ui.label(egui::RichText::new(T::stc_chip_info(lang)).strong());
                ui.add_space(2.0);
                egui::ScrollArea::vertical().max_height(60.0).show(ui, |ui| {
                    ui.label(egui::RichText::new(&state.stc_chip_info).monospace().small());
                });
                ui.add_space(4.0);
            }

            // Action buttons
            ui.horizontal(|ui| {
                let busy = state.stc_flash_running;
                if ui.add_enabled(!busy, egui::Button::new(T::stc_detect(lang))).clicked() {
                    state.stc_action = Some(StcAction::Detect);
                }
                let fw_ok = !state.stc_firmware_path.is_empty() && std::path::Path::new(&state.stc_firmware_path).exists();
                if ui.add_enabled(!busy && fw_ok, egui::Button::new(T::stc_flash(lang))).clicked() {
                    state.stc_action = Some(StcAction::Flash);
                }
            });

            // Progress
            if state.stc_flash_running {
                ui.horizontal(|ui| {
                    crate::ui::spinner::spinner_inline(ui, egui::Color32::from_rgb(59, 130, 246));
                    ui.label(&state.stc_flash_status);
                });
            }
            ui.add_space(4.0);

            // Log
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(T::stc_log(lang)).strong().small());
                if !state.stc_log.is_empty() {
                    if ui.small_button(T::clear(lang)).clicked() {
                        state.stc_log.clear();
                    }
                }
            });
            egui::ScrollArea::vertical().max_height(100.0).stick_to_bottom(true).show(ui, |ui| {
                if state.stc_log.is_empty() {
                    ui.label(egui::RichText::new(T::stc_no_log(lang)).weak().small());
                } else {
                    for entry in state.stc_log.iter().rev().take(30) {
                        let color = if entry.contains("Error") || entry.contains("failed") {
                            egui::Color32::from_rgb(239, 68, 68)
                        } else if entry.contains("OK") || entry.contains("Detected") {
                            egui::Color32::from_rgb(34, 197, 94)
                        } else {
                            egui::Color32::from_rgb(156, 163, 175)
                        };
                        ui.label(egui::RichText::new(entry).small().monospace().color(color));
                    }
                }
            });
        });
    });
}

#[derive(Debug, Clone)]
pub enum StcAction {
    Detect,
    Flash,
}
