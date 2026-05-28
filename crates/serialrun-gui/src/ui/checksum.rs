use crate::state::{AppState, T};
use eframe::egui;

pub fn render_checksum_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    ui.label(T::input_data(lang));
    ui.text_edit_multiline(&mut state.checksum_input);
    ui.add_space(4.0);
    ui.separator();
    let hex = state.checksum_input.replace(' ', "").replace("0x", "").replace("0X", "");
    if !hex.is_empty() && hex.len() % 2 == 0 {
        let data: Result<Vec<u8>, _> = (0..hex.len()).step_by(2).map(|i| u8::from_str_radix(&hex[i..i+2], 16)).collect();
        if let Ok(data) = data {
            let r = serialrun_core::checksum::ChecksumResults::calculate_all(&data);
            egui::Grid::new("crc_grid").show(ui, |ui| {
                ui.label(egui::RichText::new("Algorithm").strong());
                ui.label(egui::RichText::new("Result").strong());
                ui.end_row();
                ui.label("CRC-16/MODBUS"); ui.label(format!("0x{:04X}", r.crc16_modbus)); ui.end_row();
                ui.label("CRC-16/CCITT"); ui.label(format!("0x{:04X}", r.crc16_ccitt)); ui.end_row();
                ui.label("CRC-16/XMODEM"); ui.label(format!("0x{:04X}", r.crc16_xmodem)); ui.end_row();
                ui.label("CRC-32"); ui.label(format!("0x{:08X}", r.crc32)); ui.end_row();
                ui.label("LRC"); ui.label(format!("0x{:02X}", r.lrc)); ui.end_row();
                ui.label("Checksum-8"); ui.label(format!("0x{:02X}", r.checksum8)); ui.end_row();
                ui.label("Checksum-16"); ui.label(format!("0x{:04X}", r.checksum16)); ui.end_row();
            });
        }
    }
}
