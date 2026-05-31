use crate::port_owner::{PortCommand, PortOwnerHandle};
use crate::state::{AppState, AutoDetectMsg, Language, Theme, T};
use crate::theme;
use eframe::egui;

/// Top bar: Logo + Tool buttons + System buttons
pub fn render_connection_panel(ui: &mut egui::Ui, state: &mut AppState, ctx: &egui::Context) {
    let lang = state.language;
    ui.horizontal(|ui| {
        let icon_texture_id = {
            let icon_bytes = include_bytes!("../../icon_embedded.png");
            image::load_from_memory(icon_bytes).ok().map(|img| {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                let texture = ctx.load_texture("app_icon", color_image, egui::TextureOptions::default());
                texture.id()
            })
        };
        if let Some(texture_id) = icon_texture_id {
            ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(texture_id, egui::vec2(20.0, 20.0))));
        }
        ui.label(egui::RichText::new("SerialRUN").size(16.0).strong());
        ui.add_space(8.0);

        let mut toggled: Option<usize> = None;
        let buttons: [(&str, &str, &str); 15] = [
            ("Log", "日志", "Log"), ("Chart", "图表", "Chart"),
            ("PLC", "PLC 控制", "PLC"), ("Mod", "Modbus", "Modbus"),
            ("TCP", "TCP 桥接", "TCP Bridge"), ("HMI", "HMI 模拟器", "HMI Sim"),
            ("FT", "文件传输", "File Transfer"), ("FB", "帧生成器", "Frame Builder"), ("DL", "数据记录", "Data Logger"),
            ("CAN", "CAN 总线", "CAN Bus"), ("I2C", "I2C/SPI", "I2C/SPI"),
            ("Scope", "示波器", "Oscilloscope"), ("Flash", "烧录器", "Flasher"), ("Reg", "寄存器编辑", "Reg Editor"),
            ("Plug", "插件", "Plugins"),
        ];
        for (i, (label, zh, en)) in buttons.iter().enumerate() {
            if i == 2 || i == 9 || i == 14 { ui.separator(); }
            let tooltip = match lang { Language::Chinese => *zh, Language::English => *en };
            if ui.small_button(*label).on_hover_text(tooltip).clicked() { toggled = Some(i); }
        }
        if let Some(i) = toggled {
            match i {
                0 => state.show_log_window = !state.show_log_window, 1 => state.show_chart_window = !state.show_chart_window,
                2 => state.show_plc_window = !state.show_plc_window, 3 => state.show_modbus_window = !state.show_modbus_window,
                4 => state.show_bridge_window = !state.show_bridge_window, 5 => state.show_simulator_window = !state.show_simulator_window,
                6 => state.show_file_transfer_window = !state.show_file_transfer_window, 7 => state.show_frame_builder_window = !state.show_frame_builder_window,
                8 => state.show_data_logger_window = !state.show_data_logger_window, 9 => state.show_can_window = !state.show_can_window,
                10 => state.show_i2c_spi_window = !state.show_i2c_spi_window, 11 => state.show_scope_window = !state.show_scope_window,
                12 => state.show_flasher_window = !state.show_flasher_window, 13 => state.show_register_editor_window = !state.show_register_editor_window,
                14 => state.show_plugin_window = !state.show_plugin_window, _ => {}
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(egui::RichText::new("?").size(14.0).strong()).on_hover_text(if lang == Language::Chinese { "使用指南" } else { "Help" }).clicked() { state.show_help = !state.show_help; }
            ui.add_space(2.0);
            // Theme button
            let (tl, th) = match state.theme {
                Theme::Dark => ("Dark", if lang==Language::Chinese{"切换到浅色"}else{"Switch to Light"}),
                Theme::Light => ("Light", if lang==Language::Chinese{"切换到深色"}else{"Switch to Dark"})
            };
            if ui.button(egui::RichText::new(tl).size(12.0).strong()).on_hover_text(th).clicked() {
                state.theme = match state.theme { Theme::Dark => Theme::Light, Theme::Light => Theme::Dark };
            }
            ui.add_space(2.0);
            let ll = if lang==Language::English {"EN"} else {"中"};
            let lh = if lang==Language::English {"Switch to Chinese"} else {"切换到英文"};
            if ui.button(egui::RichText::new(ll).size(14.0).strong()).on_hover_text(lh).clicked() {
                state.language = match state.language { Language::English => Language::Chinese, Language::Chinese => Language::English };
            }
        });
    });
}

/// Left panel: Port + Baud + Connect
pub fn render_connection_controls(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    let selected = state.selected_port.clone().unwrap_or_default();
    ui.horizontal(|ui| {
        ui.label(T::serial_port(lang));
        egui::ComboBox::from_id_salt("port_select").width(120.0).selected_text(if selected.is_empty() { "—" } else { &selected }).show_ui(ui, |ui| {
            // Auto-refresh port list when dropdown is opened
            state.refresh_ports();
            for port in &state.ports {
                let label = match (&port.description, &port.manufacturer) {
                    (Some(desc), Some(mfr)) if !desc.is_empty() && !mfr.is_empty() => format!("{} - {} ({})", port.name, desc, mfr),
                    (Some(desc), _) if !desc.is_empty() => format!("{} - {}", port.name, desc),
                    _ => port.name.clone(),
                };
                ui.selectable_value(&mut state.selected_port, Some(port.name.clone()), label);
            }
        });
        if ui.small_button("\u{21BB}").on_hover_text(T::refresh_ports(lang)).clicked() { state.refresh_ports(); }
    });
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        let baud_rates = [300, 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];
        // Editable baud rate: type to input, click to dropdown
        let text_resp = ui.add(egui::TextEdit::singleline(&mut state.baud_rate_text).desired_width(80.0));
        if text_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Ok(rate) = state.baud_rate_text.trim().parse::<u32>() {
                if rate > 0 { state.config.baud_rate = rate; }
            } else {
                state.baud_rate_text = state.config.baud_rate.to_string();
            }
        }
        // Show popup on click
        if text_resp.clicked() {
            ui.memory_mut(|m| m.open_popup(egui::Id::new("baud_rate_popup")));
        }
        egui::popup_below_widget(ui, egui::Id::new("baud_rate_popup"), &text_resp, egui::popup::PopupCloseBehavior::CloseOnClick, |ui: &mut egui::Ui| {
            for &rate in &baud_rates {
                if ui.selectable_label(rate == state.config.baud_rate, format!("{}", rate)).clicked() {
                    state.config.baud_rate = rate;
                    state.baud_rate_text = rate.to_string();
                    ui.close_menu();
                }
            }
        });

        // Auto-detect button - disabled while detection is running or port is connected
        let auto_enabled = !state.is_connected && !state.auto_detect_running && !state.mcp_connect_in_progress;
        if ui.add_enabled(auto_enabled, egui::Button::new("Auto")).on_hover_text(T::auto_detect(lang)).clicked() {
            if let Some(ref pn) = state.selected_port {
                let pn = pn.clone();
                let (tx, rx) = std::sync::mpsc::channel();
                state.auto_detect_running = true;
                state.auto_detect_progress = None;
                std::thread::spawn(move || {
                    let result = auto_detect_baud(&pn, &tx);
                    let _ = tx.send(AutoDetectMsg::Done(result));
                });
                state.auto_detect_receiver = Some(rx);
                state.add_log_entry(crate::state::LogLevel::Info, "Auto-detecting baud rate...");
            } else {
                let msg = if lang == Language::Chinese { "请先选择端口" } else { "Please select a port first" };
                state.show_warning(msg);
            }
        }

        // Check for auto-detect result
        if let Some(ref rx) = state.auto_detect_receiver {
            // Drain all messages, keep last progress and check for final result
            let mut final_result: Option<Option<u32>> = None;
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    AutoDetectMsg::Progress(baud) => { state.auto_detect_progress = Some(baud); }
                    AutoDetectMsg::Done(result) => { final_result = Some(result); }
                }
            }
            if let Some(result) = final_result {
                state.auto_detect_receiver = None;
                state.auto_detect_running = false;
                state.auto_detect_progress = None;
                match result {
                    Some(baud) => {
                        state.config.baud_rate = baud;
                        state.baud_rate_text = baud.to_string();
                        state.add_log_entry(crate::state::LogLevel::Info, &format!("Auto-detected: {}", baud));
                    }
                    None => {
                        let msg = if state.language == crate::state::Language::Chinese {
                            "自动检测：未收到数据，请检查设备是否正在发送"
                        } else {
                            "Auto-detect: no data received. Check if device is sending."
                        };
                        state.show_warning(msg);
                    }
                }
            }
        }

        // Disconnect button
        let c = theme::get_colors(state.theme);
        if state.is_connected {
            // Show who is connected
            if state.connected_by == "MCP" {
                ui.label(egui::RichText::new("\u{25CF}").color(egui::Color32::from_rgb(255, 193, 7)).size(10.0));
                ui.label(egui::RichText::new("MCP").color(egui::Color32::from_rgb(255, 193, 7)).size(11.0));
            }
            if ui.button(egui::RichText::new(T::disconnect(lang)).color(c.error)).clicked() {
                // Drop the handle entirely — Drop impl sends Close and joins the thread
                state.port_owner = None;
                state.is_connected = false;
                state.connected_by.clear();
                state.ai_connected = false;
                state.ai_port_name.clear();
                state.ai_baud_rate = 0;
                state.add_log_entry(crate::state::LogLevel::Info, "Disconnected");
            }
        // Connect button - disabled while auto-detect is running or MCP is connecting
        } else if !state.auto_detect_running && !state.mcp_connect_in_progress && ui.button(egui::RichText::new(T::connect(lang)).color(c.success)).clicked() {
            if let Some(ref pn) = state.selected_port {
                let mut config = state.config.clone();
                config.port_name = pn.clone();
                // Drop old handle first to ensure old thread is shut down
                state.port_owner = None;
                let po = PortOwnerHandle::start();
                po.sync_timeout(state.rx_aggregate_ms);
                po.send(PortCommand::Open(config));
                state.port_owner = Some(po);
                state.connected_by.clear(); // GUI-initiated connection
                // Note: is_connected will be set to true when PortEvent::Opened(true) arrives
                state.add_log_entry(crate::state::LogLevel::Info, &format!("Connecting to {}...", pn));
            }
        }
    });
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);
}

pub fn auto_detect_baud(port_name: &str, progress_tx: &std::sync::mpsc::Sender<AutoDetectMsg>) -> Option<u32> {
    for &baud in &[9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600] {
        let _ = progress_tx.send(AutoDetectMsg::Progress(baud));
        let config = serialrun_core::config::SerialConfig { port_name: port_name.to_string(), baud_rate: baud, ..Default::default() };
        let mut port = serialrun_core::SerialPort::new(config);
        if port.connect().is_err() { continue; }
        let _ = port.clear_buffer(serialrun_core::port::ClearBuffer::All);
        std::thread::sleep(std::time::Duration::from_millis(100));
        let mut buf = [0u8; 256];
        let n = port.read(&mut buf).unwrap_or(0);
        let _ = port.disconnect();
        std::thread::sleep(std::time::Duration::from_millis(50));
        if n > 0 { return Some(baud); }
    }
    None
}
