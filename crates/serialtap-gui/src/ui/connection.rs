use crate::state::{AppState, Language, T, Theme};
use eframe::egui;

/// Top bar: Logo + Tool buttons + System buttons
pub fn render_connection_panel(ui: &mut egui::Ui, state: &mut AppState, _ctx: &egui::Context) {
    let lang = state.language;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("S").size(22.0).strong().color(egui::Color32::from_rgb(0, 180, 120)));
        ui.label(egui::RichText::new("SerialTap").size(16.0).strong());
        ui.add_space(8.0);

        let mut toggled: Option<usize> = None;
        let buttons: [(&str, &str, &str); 13] = [
            ("Log", "日志", "Log"), ("Chart", "图表", "Chart"),
            ("PLC", "PLC 控制", "PLC"), ("Mod", "Modbus", "Modbus"),
            ("FT", "文件传输", "File Transfer"), ("FB", "帧生成器", "Frame Builder"), ("DL", "数据记录", "Data Logger"),
            ("CAN", "CAN 总线", "CAN Bus"), ("I2C", "I2C/SPI", "I2C/SPI"),
            ("Scope", "示波器", "Oscilloscope"), ("Flash", "烧录器", "Flasher"), ("Reg", "寄存器编辑", "Reg Editor"),
            ("Plug", "插件", "Plugins"),
        ];
        for (i, (label, zh, en)) in buttons.iter().enumerate() {
            if i == 2 || i == 7 || i == 12 { ui.separator(); }
            let tooltip = match lang { Language::Chinese => *zh, Language::English => *en };
            if ui.small_button(*label).on_hover_text(tooltip).clicked() { toggled = Some(i); }
        }
        if let Some(i) = toggled {
            match i {
                0 => state.show_log_window = !state.show_log_window, 1 => state.show_chart_window = !state.show_chart_window,
                2 => state.show_plc_window = !state.show_plc_window, 3 => state.show_modbus_window = !state.show_modbus_window,
                4 => state.show_file_transfer_window = !state.show_file_transfer_window, 5 => state.show_frame_builder_window = !state.show_frame_builder_window,
                6 => state.show_data_logger_window = !state.show_data_logger_window, 7 => state.show_can_window = !state.show_can_window,
                8 => state.show_i2c_spi_window = !state.show_i2c_spi_window, 9 => state.show_scope_window = !state.show_scope_window,
                10 => state.show_flasher_window = !state.show_flasher_window, 11 => state.show_register_editor_window = !state.show_register_editor_window,
                12 => state.show_plugin_window = !state.show_plugin_window, _ => {}
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(egui::RichText::new("?").size(14.0).strong()).on_hover_text(if lang == Language::Chinese { "使用指南" } else { "Help" }).clicked() { state.show_help = !state.show_help; }
            ui.add_space(2.0);
            let (tl, th) = match state.theme { Theme::Dark => ("\u{2600}", if lang==Language::Chinese{"浅色"}else{"Light"}), Theme::Light => ("\u{263E}", if lang==Language::Chinese{"深色"}else{"Dark"}) };
            if ui.button(egui::RichText::new(tl).size(16.0)).on_hover_text(th).clicked() { state.theme = match state.theme { Theme::Dark => Theme::Light, Theme::Light => Theme::Dark }; }
            ui.add_space(2.0);
            let ll = if lang==Language::English {"EN"} else {"中"};
            let lh = if lang==Language::English {"Switch to Chinese"} else {"切换到英文"};
            if ui.button(egui::RichText::new(ll).size(14.0).strong()).on_hover_text(lh).clicked() { state.language = match state.language { Language::English => Language::Chinese, Language::Chinese => Language::English }; }
        });
    });
}

/// Left panel: Port + Baud + Connect
pub fn render_connection_controls(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    let port_names: Vec<String> = state.ports.iter().map(|p| p.name.clone()).collect();
    let selected = state.selected_port.clone().unwrap_or_default();
    ui.horizontal(|ui| {
        ui.label(T::serial_port(lang));
        egui::ComboBox::from_id_salt("port_select").width(90.0).selected_text(if selected.is_empty() { "—" } else { &selected }).show_ui(ui, |ui| {
            for name in &port_names { ui.selectable_value(&mut state.selected_port, Some(name.clone()), name); }
        });
        if ui.small_button("\u{21BB}").on_hover_text(T::refresh_ports(lang)).clicked() { state.refresh_ports(); }
    });
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        let baud_rates = [9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];
        egui::ComboBox::from_id_salt("baud_rate").width(80.0).selected_text(format!("{}", state.config.baud_rate)).show_ui(ui, |ui| {
            for &rate in &baud_rates { ui.selectable_value(&mut state.config.baud_rate, rate, format!("{}", rate)); }
        });
        if !state.is_connected {
            if ui.small_button("Auto").on_hover_text(T::auto_detect(lang)).clicked() {
                if let Some(ref pn) = state.selected_port {
                    let pn = pn.clone();
                    if let Some(baud) = auto_detect_baud(&pn) {
                        state.config.baud_rate = baud;
                        state.add_log_entry(crate::state::LogLevel::Info, &format!("Auto-detected: {}", baud));
                    } else {
                        state.add_log_entry(crate::state::LogLevel::Warning, "Auto-detect: no data received");
                    }
                }
            }
        }
        if state.is_connected {
            if ui.button(egui::RichText::new(T::disconnect(lang)).color(egui::Color32::from_rgb(220, 60, 60))).clicked() {
                if let Some(mut port) = state.port.take() { let _ = port.disconnect(); }
                state.is_connected = false;
                state.add_log_entry(crate::state::LogLevel::Info, "Disconnected");
            }
        } else if ui.button(egui::RichText::new(T::connect(lang)).color(egui::Color32::from_rgb(0, 180, 120))).clicked() {
            if let Some(ref pn) = state.selected_port {
                let config = serialtap_core::config::SerialConfig { port_name: pn.clone(), baud_rate: state.config.baud_rate, ..Default::default() };
                let mut port = serialtap_core::SerialPort::new(config);
                match port.connect() {
                    Ok(()) => { state.is_connected = true; state.port = Some(port); state.add_log_entry(crate::state::LogLevel::Info, &format!("Connected to {}", pn)); }
                    Err(e) => { state.add_log_entry(crate::state::LogLevel::Error, &e.to_string()); }
                }
            }
        }
    });
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);
}

fn auto_detect_baud(port_name: &str) -> Option<u32> {
    for &baud in &[9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600] {
        let config = serialtap_core::config::SerialConfig { port_name: port_name.to_string(), baud_rate: baud, ..Default::default() };
        let mut port = serialtap_core::SerialPort::new(config);
        if port.connect().is_err() { continue; }
        let _ = port.clear_buffer(serialtap_core::port::ClearBuffer::All);
        std::thread::sleep(std::time::Duration::from_millis(100));
        let mut buf = [0u8; 256];
        let n = port.read(&mut buf).unwrap_or(0);
        let _ = port.disconnect();
        if n > 0 { return Some(baud); }
    }
    None
}
