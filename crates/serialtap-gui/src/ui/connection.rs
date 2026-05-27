use crate::state::{AppState, Language, T, Theme};
use eframe::egui;
use serialtap_core::SerialConfig;

pub fn render_connection_panel(ui: &mut egui::Ui, state: &mut AppState, _ctx: &egui::Context) {
    let lang = state.language;

    ui.horizontal(|ui| {
        // Logo
        ui.label(
            egui::RichText::new("S")
                .size(22.0)
                .strong()
                .color(egui::Color32::from_rgb(0, 180, 120)),
        );
        ui.label(
            egui::RichText::new("SerialTap")
                .size(16.0)
                .strong(),
        );

        ui.add_space(12.0);

        // Port selector
        ui.label(T::serial_port(lang));
        let port_names: Vec<String> = state.ports.iter().map(|p| p.name.clone()).collect();
        let selected = state.selected_port.clone().unwrap_or_default();
        egui::ComboBox::from_id_salt("port_select")
            .width(130.0)
            .selected_text(if selected.is_empty() { "—" } else { &selected })
            .show_ui(ui, |ui| {
                for name in &port_names {
                    ui.selectable_value(&mut state.selected_port, Some(name.clone()), name);
                }
            });

        if ui.button(T::refresh_ports(lang)).clicked() {
            state.refresh_ports();
        }

        ui.add_space(8.0);

        // Baud rate
        let baud_rates = [9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];
        egui::ComboBox::from_id_salt("baud_rate")
            .width(90.0)
            .selected_text(format!("{}", state.config.baud_rate))
            .show_ui(ui, |ui| {
                for &rate in &baud_rates {
                    ui.selectable_value(&mut state.config.baud_rate, rate, format!("{}", rate));
                }
            });

        ui.add_space(8.0);

        // Connect / Disconnect button
        if state.is_connected {
            if ui
                .button(egui::RichText::new(T::disconnect(lang)).color(egui::Color32::from_rgb(220, 60, 60)))
                .clicked()
            {
                if let Some(mut port) = state.port.take() {
                    let _ = port.disconnect();
                }
                state.is_connected = false;
                state.add_log_entry(crate::state::LogLevel::Info, "Disconnected");
            }
        } else if ui
            .button(egui::RichText::new(T::connect(lang)).color(egui::Color32::from_rgb(0, 180, 120)))
            .clicked()
        {
            if let Some(ref port_name) = state.selected_port {
                let config =
                    SerialConfig::new(port_name).with_baud_rate(state.config.baud_rate);
                let mut port = serialtap_core::SerialPort::new(config);
                match port.connect() {
                    Ok(()) => {
                        state.is_connected = true;
                        state.port = Some(port);
                        state.add_log_entry(
                            crate::state::LogLevel::Info,
                            &format!("Connected to {}", port_name),
                        );
                    }
                    Err(e) => {
                        state.add_log_entry(crate::state::LogLevel::Error, &e.to_string());
                    }
                }
            }
        }

        // Right side: toolbar buttons
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Help button
            let help_text = match lang {
                Language::Chinese => "使用指南",
                Language::English => "Help",
            };
            if ui.button(egui::RichText::new("?").size(14.0).strong()).on_hover_text(help_text).clicked() {
                state.show_help = !state.show_help;
            }

            ui.add_space(4.0);

            // Theme toggle - icon shows what you GET when you click
            let (theme_label, theme_hover) = match state.theme {
                Theme::Dark => ("\u{2600}", match lang {
                    Language::Chinese => "切换到浅色主题",
                    Language::English => "Switch to light theme",
                }),
                Theme::Light => ("\u{263E}", match lang {
                    Language::Chinese => "切换到深色主题",
                    Language::English => "Switch to dark theme",
                }),
            };
            if ui.button(egui::RichText::new(theme_label).size(16.0)).on_hover_text(theme_hover).clicked() {
                state.theme = match state.theme {
                    Theme::Dark => Theme::Light,
                    Theme::Light => Theme::Dark,
                };
            }

            ui.add_space(4.0);

            // Language toggle
            let lang_label = match state.language {
                Language::English => "EN",
                Language::Chinese => "中",
            };
            let lang_hover = match state.language {
                Language::English => "Switch to Chinese",
                Language::Chinese => "切换到英文",
            };
            if ui.button(egui::RichText::new(lang_label).size(14.0).strong()).on_hover_text(lang_hover).clicked() {
                state.language = match state.language {
                    Language::English => Language::Chinese,
                    Language::Chinese => Language::English,
                };
            }

            ui.add_space(4.0);

            // Log toggle
            if ui.button(T::log(lang)).on_hover_text(T::log(lang)).clicked() {
                state.show_log_window = !state.show_log_window;
            }

            ui.add_space(4.0);

            // Chart toggle
            if ui.button(T::chart(lang)).on_hover_text(T::chart(lang)).clicked() {
                state.show_chart_window = !state.show_chart_window;
            }
        });
    });
}
