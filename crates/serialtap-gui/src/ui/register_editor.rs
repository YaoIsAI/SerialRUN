use crate::state::{AppState, RegMapEntry, T};
use eframe::egui;
use std::fs;

pub fn render_register_editor_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Register Map Editor").strong());
        ui.separator();
        if ui.button(T::import_btn(lang)).clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("CSV", &["csv"]).add_filter("JSON", &["json"]).pick_file() {
                import_register_map(state, &path);
            }
        }
        if ui.button(T::export_btn(lang)).clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("CSV", &["csv"]).save_file() {
                export_register_map(state, &path);
            }
        }
        if ui.button("Add").clicked() {
            state.reg_map.push(RegMapEntry {
                addr: state.reg_map.len() as u16 * 10,
                name: format!("Reg{}", state.reg_map.len()),
                data_type: "UINT16".into(),
                value: String::new(),
                description: String::new(),
            });
        }
    });
    ui.add_space(4.0);

    // Register table
    if state.reg_map.is_empty() {
        ui.label(egui::RichText::new("No registers loaded. Import a register map or add registers manually.").weak());
    } else {
        egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
            egui::Grid::new("reg_grid").num_columns(5).striped(true).show(ui, |ui| {
                ui.label(egui::RichText::new("Addr").strong());
                ui.label(egui::RichText::new("Name").strong());
                ui.label(egui::RichText::new("Type").strong());
                ui.label(egui::RichText::new("Value").strong());
                ui.label(egui::RichText::new("Desc").strong());
                ui.end_row();
                let mut remove_idx = None;
                for (i, entry) in state.reg_map.iter_mut().enumerate() {
                    let mut addr_str = format!("0x{:04X}", entry.addr);
                    ui.text_edit_singleline(&mut addr_str);
                    if let Ok(a) = u16::from_str_radix(addr_str.trim_start_matches("0x"), 16) { entry.addr = a; }
                    ui.text_edit_singleline(&mut entry.name);
                    ui.text_edit_singleline(&mut entry.data_type);
                    ui.text_edit_singleline(&mut entry.value);
                    ui.text_edit_singleline(&mut entry.description);
                    if ui.small_button("X").clicked() { remove_idx = Some(i); }
                    ui.end_row();
                }
                if let Some(idx) = remove_idx { state.reg_map.remove(idx); }
            });
        });
    }

    // Alarm
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.checkbox(&mut state.reg_alarm_enabled, "Alarm");
        if state.reg_alarm_enabled {
            ui.label("Threshold:");
            ui.text_edit_singleline(&mut state.reg_alarm_threshold);
        }
    });
}

fn import_register_map(state: &mut AppState, path: &std::path::Path) {
    match fs::read_to_string(path) {
        Ok(content) => {
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                match serde_json::from_str::<Vec<RegMapEntry>>(&content) {
                    Ok(entries) => {
                        state.reg_map = entries;
                        state.add_log_entry(crate::state::LogLevel::Info, &format!("Imported {} registers from JSON", state.reg_map.len()));
                    }
                    Err(e) => { state.add_log_entry(crate::state::LogLevel::Error, &format!("JSON parse error: {}", e)); }
                }
            } else {
                // CSV: addr,name,type,value,description
                state.reg_map.clear();
                for (i, line) in content.lines().enumerate() {
                    if i == 0 && line.starts_with("addr") { continue; } // skip header
                    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 2 {
                        let trimmed = parts[0].trim_start_matches("0x").trim_start_matches("0X");
                        let addr = match u16::from_str_radix(trimmed, 16) {
                            Ok(a) => a,
                            Err(_) => match parts[0].parse::<u16>() {
                                Ok(a) => a,
                                Err(_) => continue,
                            },
                        };
                        state.reg_map.push(RegMapEntry {
                            addr,
                            name: parts.get(1).unwrap_or(&"").to_string(),
                            data_type: parts.get(2).unwrap_or(&"UINT16").to_string(),
                            value: parts.get(3).unwrap_or(&"").to_string(),
                            description: parts.get(4).unwrap_or(&"").to_string(),
                        });
                    }
                }
                state.add_log_entry(crate::state::LogLevel::Info, &format!("Imported {} registers from CSV", state.reg_map.len()));
            }
        }
        Err(e) => { state.add_log_entry(crate::state::LogLevel::Error, &format!("File read error: {}", e)); }
    }
}

fn export_register_map(state: &mut AppState, path: &std::path::Path) {
    let mut content = String::from("addr,name,type,value,description\n");
    for entry in &state.reg_map {
        content.push_str(&format!("0x{:04X},{},{},{},{}\n", entry.addr, entry.name, entry.data_type, entry.value, entry.description));
    }
    match fs::write(path, content) {
        Ok(()) => { state.add_log_entry(crate::state::LogLevel::Info, &format!("Exported {} registers to {}", state.reg_map.len(), path.display())); }
        Err(e) => { state.add_log_entry(crate::state::LogLevel::Error, &format!("Export error: {}", e)); }
    }
}
