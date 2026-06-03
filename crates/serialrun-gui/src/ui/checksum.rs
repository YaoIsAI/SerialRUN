use crate::state::{AppState, Language, T};
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
                ui.label(egui::RichText::new(T::algorithm(lang)).strong());
                ui.label(egui::RichText::new(T::result_label(lang)).strong());
                ui.end_row();
                crc_row(ui, "CRC-16/MODBUS", format!("0x{:04X}", r.crc16_modbus), lang);
                crc_row(ui, "CRC-16/CCITT", format!("0x{:04X}", r.crc16_ccitt), lang);
                crc_row(ui, "CRC-16/XMODEM", format!("0x{:04X}", r.crc16_xmodem), lang);
                crc_row(ui, "CRC-32", format!("0x{:08X}", r.crc32), lang);
                crc_row(ui, "LRC", format!("0x{:02X}", r.lrc), lang);
                crc_row(ui, "Checksum-8", format!("0x{:02X}", r.checksum8), lang);
                crc_row(ui, "Checksum-16", format!("0x{:04X}", r.checksum16), lang);
            });
        }
    }
}

fn crc_row(ui: &mut egui::Ui, algo: &str, result: String, lang: Language) {
    ui.horizontal(|ui| {
        ui.label(algo);
        ui.label(egui::RichText::new("?").color(egui::Color32::from_rgb(100, 150, 220)).strong())
            .on_hover_text(crc_tooltip(algo, lang));
    });
    ui.label(result);
    ui.end_row();
}

fn crc_tooltip(algo: &str, lang: Language) -> &'static str {
    match (algo, lang) {
        ("CRC-16/MODBUS", Language::Chinese) => "Modbus RTU 标准校验，多项式 0xA001",
        ("CRC-16/MODBUS", Language::English) => "Modbus RTU standard, polynomial 0xA001",
        ("CRC-16/CCITT", Language::Chinese) => "CCITT 标准，多项式 0x1021，常用于 X.25/HDLC",
        ("CRC-16/CCITT", Language::English) => "CCITT standard, poly 0x1021, used in X.25/HDLC",
        ("CRC-16/XMODEM", Language::Chinese) => "XMODEM 文件传输协议校验",
        ("CRC-16/XMODEM", Language::English) => "XMODEM file transfer protocol checksum",
        ("CRC-32", Language::Chinese) => "32位循环冗余校验，用于 ZIP/以太网等",
        ("CRC-32", Language::English) => "32-bit CRC, used in ZIP/Ethernet",
        ("LRC", Language::Chinese) => "纵向冗余校验，Modbus ASCII 模式使用",
        ("LRC", Language::English) => "Longitudinal Redundancy Check, Modbus ASCII",
        ("Checksum-8", Language::Chinese) => "8位累加和校验",
        ("Checksum-8", Language::English) => "8-bit additive checksum",
        ("Checksum-16", Language::Chinese) => "16位累加和校验",
        ("Checksum-16", Language::English) => "16-bit additive checksum",
        _ => "",
    }
}
