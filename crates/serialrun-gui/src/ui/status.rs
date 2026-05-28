use crate::state::{AppState, T};
use eframe::egui;

pub fn render_status_bar(ui: &mut egui::Ui, state: &AppState) {
    let lang = state.language;
    ui.horizontal(|ui| {
        let status_color = if state.is_connected {
            egui::Color32::GREEN
        } else {
            egui::Color32::RED
        };

        let status_text = if state.is_connected {
            format!("{}: {}", T::connected(lang), state.selected_port.as_deref().unwrap_or("N/A"))
        } else {
            T::disconnected(lang).to_string()
        };

        ui.label(egui::RichText::new(status_text).color(status_color));

        ui.separator();

        ui.label(format!("{}: {}", T::baud_rate(lang), state.config.baud_rate));

        ui.separator();

        ui.label(format!("RX: {} {}", state.rx_count, T::bytes(lang)));
        ui.label(format!("TX: {} {}", state.tx_count, T::bytes(lang)));

        if state.recording {
            ui.separator();
            ui.label(egui::RichText::new(format!("● {}", T::recording(lang))).color(egui::Color32::RED));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label("SerialRUN v0.1.0");
        });
    });
}
