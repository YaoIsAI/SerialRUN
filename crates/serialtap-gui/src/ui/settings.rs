use crate::state::{AppState, T};
use eframe::egui;
use serialtap_core::config::{DataBits, FlowControl, Parity, StopBits};

pub fn render_settings_panel(ui: &mut egui::Ui, state: &mut AppState, _ctx: &egui::Context) {
    let lang = state.language;

    ui.add_space(4.0);

    // Serial config section
    ui.collapsing(T::serial_config(lang), |ui| {
        egui::Grid::new("settings_grid").show(ui, |ui| {
            ui.label(T::data_bits(lang));
            egui::ComboBox::from_id_salt("data_bits")
                .width(80.0)
                .selected_text(format!("{:?}", state.config.data_bits))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.config.data_bits, DataBits::Five, "5");
                    ui.selectable_value(&mut state.config.data_bits, DataBits::Six, "6");
                    ui.selectable_value(&mut state.config.data_bits, DataBits::Seven, "7");
                    ui.selectable_value(&mut state.config.data_bits, DataBits::Eight, "8");
                });
            ui.end_row();

            ui.label(T::stop_bits(lang));
            egui::ComboBox::from_id_salt("stop_bits")
                .width(80.0)
                .selected_text(format!("{:?}", state.config.stop_bits))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.config.stop_bits, StopBits::One, "1");
                    ui.selectable_value(&mut state.config.stop_bits, StopBits::Two, "2");
                });
            ui.end_row();

            ui.label(T::parity(lang));
            egui::ComboBox::from_id_salt("parity")
                .width(80.0)
                .selected_text(format!("{:?}", state.config.parity))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.config.parity, Parity::None, "None");
                    ui.selectable_value(&mut state.config.parity, Parity::Odd, "Odd");
                    ui.selectable_value(&mut state.config.parity, Parity::Even, "Even");
                });
            ui.end_row();

            ui.label(T::flow_control(lang));
            egui::ComboBox::from_id_salt("flow_control")
                .width(80.0)
                .selected_text(format!("{:?}", state.config.flow_control))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.config.flow_control, FlowControl::None, "None");
                    ui.selectable_value(&mut state.config.flow_control, FlowControl::Software, "SW");
                    ui.selectable_value(&mut state.config.flow_control, FlowControl::Hardware, "HW");
                });
            ui.end_row();
        });
    });

    ui.separator();

    // Display settings
    ui.label(T::display(lang));
    ui.add_space(4.0);
    ui.checkbox(&mut state.hex_mode, T::hex_mode(lang));
    ui.checkbox(&mut state.show_timestamp, T::show_timestamp(lang));
    ui.checkbox(&mut state.auto_scroll, T::auto_scroll(lang));

    ui.separator();

    // Auto reply
    ui.label(T::auto_reply(lang));
    ui.add_space(4.0);
    ui.checkbox(&mut state.auto_reply_enabled, T::auto_reply(lang));
    if state.auto_reply_enabled {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label(format!("{}:", T::pattern(lang)));
            ui.text_edit_singleline(&mut state.auto_reply_pattern);
        });
        ui.horizontal(|ui| {
            ui.label(format!("{}:", T::response(lang)));
            ui.text_edit_singleline(&mut state.auto_reply_response);
        });
    }

    ui.separator();

    // Recording
    if state.recording {
        ui.label(egui::RichText::new(format!("● {}", T::recording(lang))).color(egui::Color32::from_rgb(220, 60, 60)));
        if ui.button(T::stop_recording(lang)).clicked() {
            state.recording = false;
            state.add_log_entry(crate::state::LogLevel::Info, "Recording stopped");
        }
    } else {
        if ui.button(T::start_recording(lang)).clicked() {
            state.recording = true;
            state.script_commands.clear();
            state.add_log_entry(crate::state::LogLevel::Info, "Recording started");
        }
    }

    // Bottom: help hint
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(format!("? {}", T::help(lang)))
                .small()
                .color(egui::Color32::GRAY),
        );
    });
}
