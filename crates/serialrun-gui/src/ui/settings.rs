use crate::state::{AppState, Language, T};
use crate::theme;
use eframe::egui;
use serialrun_core::config::{DataBits, FlowControl, Parity, StopBits};

pub fn render_settings_panel(ui: &mut egui::Ui, state: &mut AppState, _ctx: &egui::Context) {
    let lang = state.language;

    ui.add_space(4.0);

    // Serial config section
    ui.collapsing(T::serial_config(lang), |ui| {
        egui::Grid::new("settings_grid").show(ui, |ui| {
            ui.label(T::data_bits(lang));
            let db_text = match state.config.data_bits { DataBits::Five=>"5", DataBits::Six=>"6", DataBits::Seven=>"7", DataBits::Eight=>"8" };
            egui::ComboBox::from_id_salt("data_bits")
                .width(80.0)
                .selected_text(db_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.config.data_bits, DataBits::Five, "5");
                    ui.selectable_value(&mut state.config.data_bits, DataBits::Six, "6");
                    ui.selectable_value(&mut state.config.data_bits, DataBits::Seven, "7");
                    ui.selectable_value(&mut state.config.data_bits, DataBits::Eight, "8");
                });
            ui.end_row();

            ui.label(T::stop_bits(lang));
            let sb_text = match state.config.stop_bits { StopBits::One=>"1", StopBits::Two=>"2" };
            egui::ComboBox::from_id_salt("stop_bits")
                .width(80.0)
                .selected_text(sb_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.config.stop_bits, StopBits::One, "1");
                    ui.selectable_value(&mut state.config.stop_bits, StopBits::Two, "2");
                });
            ui.end_row();

            ui.label(T::parity(lang));
            let par_text = match state.config.parity { Parity::None=>"None", Parity::Odd=>"Odd", Parity::Even=>"Even" };
            egui::ComboBox::from_id_salt("parity")
                .width(80.0)
                .selected_text(par_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.config.parity, Parity::None, "None");
                    ui.selectable_value(&mut state.config.parity, Parity::Odd, "Odd");
                    ui.selectable_value(&mut state.config.parity, Parity::Even, "Even");
                });
            ui.end_row();

            ui.label(T::flow_control(lang));
            let fc_text = match state.config.flow_control { FlowControl::None=>"None", FlowControl::Software=>"SW", FlowControl::Hardware=>"HW" };
            egui::ComboBox::from_id_salt("flow_control")
                .width(80.0)
                .selected_text(fc_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.config.flow_control, FlowControl::None, "None");
                    ui.selectable_value(&mut state.config.flow_control, FlowControl::Software, "SW");
                    ui.selectable_value(&mut state.config.flow_control, FlowControl::Hardware, "HW");
                });
            ui.end_row();
        });
    });

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

    // MCP Server settings — always expanded
    ui.label(egui::RichText::new(T::mcp_server(lang)).strong());
    ui.add_space(2.0);
    {
        ui.horizontal(|ui| {
            if ui.checkbox(&mut state.mcp_enabled, T::mcp_enable(lang)).changed() {
                if let Some(ref cmd_tx) = state.mcp_cmd_tx {
                    if state.mcp_enabled {
                        let bind_addr = if state.mcp_bind_lan { "0.0.0.0" } else { "127.0.0.1" }.into();
                        let _ = cmd_tx.send(crate::mcp_server::McpCommand::Start { bind_addr, port: state.mcp_port });
                    } else {
                        let _ = cmd_tx.send(crate::mcp_server::McpCommand::Stop);
                    }
                }
            }
        });
        ui.add_space(4.0);

        egui::Grid::new("mcp_settings").show(ui, |ui| {
            ui.label(T::mcp_port(lang));
            let port_resp = ui.add(egui::DragValue::new(&mut state.mcp_port).range(1024..=65535));
            if port_resp.changed() && state.mcp_running {
                if let Some(ref cmd_tx) = state.mcp_cmd_tx {
                    let bind_addr = if state.mcp_bind_lan { "0.0.0.0" } else { "127.0.0.1" }.into();
                    let _ = cmd_tx.send(crate::mcp_server::McpCommand::Reconfigure { bind_addr, port: state.mcp_port });
                }
            }
            ui.end_row();

            ui.label(T::mcp_bind(lang));
            let bind_text = if state.mcp_bind_lan { T::mcp_lan(lang) } else { T::mcp_localhost(lang) };
            let bind_resp = egui::ComboBox::from_id_salt("mcp_bind").width(140.0).selected_text(bind_text).show_ui(ui, |ui| {
                ui.selectable_value(&mut state.mcp_bind_lan, false, T::mcp_localhost(lang));
                ui.selectable_value(&mut state.mcp_bind_lan, true, T::mcp_lan(lang));
            });
            if bind_resp.response.changed() && state.mcp_running {
                if let Some(ref cmd_tx) = state.mcp_cmd_tx {
                    let bind_addr = if state.mcp_bind_lan { "0.0.0.0" } else { "127.0.0.1" }.into();
                    let _ = cmd_tx.send(crate::mcp_server::McpCommand::Reconfigure { bind_addr, port: state.mcp_port });
                }
            }
            ui.end_row();

            ui.label(T::mcp_status(lang));
            let c = theme::get_colors(state.theme);
            let status_color = if state.mcp_running { c.logo_green } else { c.text_muted };
            ui.label(egui::RichText::new(if state.mcp_running { T::mcp_running(lang) } else { T::mcp_stopped(lang) }).color(status_color));
            ui.end_row();
        });

        if state.mcp_bind_lan {
            ui.add_space(4.0);
            let c = theme::get_colors(state.theme);
            ui.label(egui::RichText::new(T::mcp_warning(lang)).color(c.warning).strong().size(13.0));
        }

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let addr = if state.mcp_bind_lan { "0.0.0.0" } else { "127.0.0.1" };
            ui.label(egui::RichText::new(format!("Bind: {}:{}", addr, state.mcp_port)).monospace().size(13.0));
        });
        if state.mcp_bind_lan {
            if let Some(local_ip) = get_local_ip() {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("LAN: {}:{}", local_ip, state.mcp_port)).monospace().size(13.0).color(egui::Color32::from_rgb(80, 160, 230)).strong());
                });
            }
        }

        // Access log button
        ui.add_space(4.0);
        let log_count = state.mcp_access_log.len();
        let log_label = if lang == Language::Chinese {
            format!("\u{1F4CB} 访问日志 ({})", log_count)
        } else {
            format!("\u{1F4CB} Access Log ({})", log_count)
        };
        if ui.button(egui::RichText::new(&log_label).strong()).clicked() {
            state.show_mcp_log_popup = !state.show_mcp_log_popup;
        }
    }

    ui.separator();

    // Recording & Replay
    ui.label(egui::RichText::new(if lang == Language::Chinese { "录制 / 回放" } else { "Record / Replay" }).strong());
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        if state.recording {
            ui.label(egui::RichText::new(format!("● {}", T::recording(lang))).color(egui::Color32::from_rgb(220, 60, 60)));
            if ui.button(T::stop_recording(lang)).clicked() {
                state.recording = false;
                state.recording_last_time = 0;
                state.add_log_entry(crate::state::LogLevel::Info, &format!("Recording stopped. {} commands recorded.", state.script_commands.len()));
            }
        } else {
            if ui.button(T::start_recording(lang)).clicked() {
                state.recording = true;
                state.recording_last_time = chrono::Utc::now().timestamp_millis();
                state.script_commands.clear();
                state.add_log_entry(crate::state::LogLevel::Info, "Recording started");
            }
        }
    });

    // Script info
    if !state.script_commands.is_empty() && !state.recording {
        let send_count = state.script_commands.iter().filter(|c| c.action == crate::state::ScriptAction::Send).count();
        ui.label(egui::RichText::new(format!("{}: {} {}", if lang == Language::Chinese { "已录制" } else { "Recorded" }, send_count, if lang == Language::Chinese { "条命令" } else { "commands" })).weak().small());
    }

    // Save / Load / Replay buttons
    ui.horizontal(|ui| {
        let can_save = !state.script_commands.is_empty() && !state.recording;
        let can_load = !state.recording && !state.replay_running;
        let can_replay = !state.script_commands.is_empty() && !state.recording && !state.replay_running && state.is_connected;
        let can_stop_replay = state.replay_running;

        if ui.add_enabled(can_save, egui::Button::new(T::save_btn(lang))).clicked() {
            save_script(state);
        }
        if ui.add_enabled(can_load, egui::Button::new(T::import_btn(lang))).clicked() {
            load_script(state);
        }
        ui.separator();
        if ui.add_enabled(can_replay, egui::Button::new(
            egui::RichText::new(format!("▶ {}", if lang == Language::Chinese { "回放" } else { "Replay" })).color(egui::Color32::WHITE).strong()
        ).fill(egui::Color32::from_rgb(40, 160, 80))).clicked() {
            start_replay(state);
        }
        if ui.add_enabled(can_stop_replay, egui::Button::new(
            egui::RichText::new(format!("■ {}", if lang == Language::Chinese { "停止" } else { "Stop" })).color(egui::Color32::WHITE).strong()
        ).fill(egui::Color32::from_rgb(200, 60, 60))).clicked() {
            stop_replay(state);
        }
    });

    // Replay progress
    if state.replay_running {
        let total = state.replay_commands.len();
        let done = state.replay_index;
        let pct = if total > 0 { done as f32 / total as f32 } else { 0.0 };
        ui.add(egui::ProgressBar::new(pct).text(format!("{}/{}", done, total)));
    }
}

fn save_script(state: &mut AppState) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Script", &["txt", "srs"])
        .add_filter("All", &["*"])
        .save_file()
    {
        let mut content = String::from("# SerialRUN Script\n");
        for cmd in &state.script_commands {
            content.push_str(&format!("{}\n", cmd.to_text_line()));
        }
        match std::fs::write(&path, content) {
            Ok(()) => {
                state.add_log_entry(crate::state::LogLevel::Info, &format!("Script saved: {} ({} commands)", path.display(), state.script_commands.len()));
            }
            Err(e) => {
                state.show_error(&format!("Save failed: {}", e));
            }
        }
    }
}

fn load_script(state: &mut AppState) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Script", &["txt", "srs"])
        .add_filter("All", &["*"])
        .pick_file()
    {
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let commands: Vec<crate::state::ScriptCommand> = content.lines()
                    .filter_map(crate::state::ScriptCommand::from_text_line)
                    .collect();
                let count = commands.iter().filter(|c| c.action == crate::state::ScriptAction::Send).count();
                state.script_commands = commands;
                state.add_log_entry(crate::state::LogLevel::Info, &format!("Script loaded: {} ({} commands)", path.display(), count));
            }
            Err(e) => {
                state.show_error(&format!("Load failed: {}", e));
            }
        }
    }
}

fn start_replay(state: &mut AppState) {
    if state.script_commands.is_empty() || !state.is_connected {
        return;
    }
    state.replay_commands = state.script_commands.clone();
    state.replay_index = 0;
    state.replay_start_time = chrono::Utc::now().timestamp_millis();
    state.replay_running = true;
    state.add_log_entry(crate::state::LogLevel::Info, &format!("Replay started ({} commands)", state.replay_commands.len()));
}

fn stop_replay(state: &mut AppState) {
    state.replay_running = false;
    state.replay_index = 0;
    state.replay_commands.clear();
    state.add_log_entry(crate::state::LogLevel::Info, "Replay stopped");
}

fn get_local_ip() -> Option<String> {
    crate::util::get_local_ip()
}
