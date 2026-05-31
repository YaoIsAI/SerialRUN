use crate::state::{AppState, ModbusFunctionCode, T};
use eframe::egui;
use serialrun_core::protocol::ModbusParser;

pub fn render_frame_builder_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    // Poll async write result
    if let Some(ref rx) = state.fb_write_async {
        if let Ok(result) = rx.try_recv() {
            state.fb_write_async = None;
            if let Err(e) = result {
                state.frame_builder_error = Some(format!("Frame send: {}", e));
                state.show_error(&format!("Frame send: {}", e));
            }
        }
    }

    egui::Grid::new("fb_grid").num_columns(2).show(ui, |ui| {
        ui.label(T::slave_id(lang)); ui.add(egui::DragValue::new(&mut state.frame_builder_slave_id).range(0..=247)); ui.end_row();
        ui.label(T::function_code(lang));
        let fc = state.frame_builder_fc;
        egui::ComboBox::from_id_salt("fb_fc").selected_text(fc.label(lang)).show_ui(ui, |ui| { for &f in ModbusFunctionCode::all() { ui.selectable_value(&mut state.frame_builder_fc, f, f.label(lang)); } });
        ui.end_row();
        ui.label(T::start_address(lang)); ui.text_edit_singleline(&mut state.frame_builder_addr); ui.end_row();
        if fc.is_read() { ui.label(T::quantity(lang)); } else { ui.label(T::write_value(lang)); }
        ui.text_edit_singleline(&mut state.frame_builder_value); ui.end_row();
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui.button(T::build_btn(lang)).clicked() {
            state.frame_builder_error = None;
            let addr: u16 = match state.frame_builder_addr.parse() { Ok(v) => v, Err(_) => { let m = "Invalid address".to_string(); state.frame_builder_error = Some(m.clone()); state.show_error(&m); return; } };
            let fc = state.frame_builder_fc;
            let frame = if fc.is_read() {
                let qty: u16 = match state.frame_builder_value.parse() { Ok(v) => v, Err(_) => { let m = "Invalid quantity".to_string(); state.frame_builder_error = Some(m.clone()); state.show_error(&m); return; } };
                ModbusParser::build_read_request(state.frame_builder_slave_id, fc.to_core_function(), addr, qty)
            } else {
                let val: u16 = match state.frame_builder_value.parse() { Ok(v) => v, Err(_) => { let m = "Invalid value".to_string(); state.frame_builder_error = Some(m.clone()); state.show_error(&m); return; } };
                ModbusParser::build_write_single(state.frame_builder_slave_id, addr, val)
            };
            state.frame_builder_hex = frame.to_bytes().iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
        }
        if ui.button(T::send(lang)).clicked() && !state.frame_builder_hex.is_empty() && state.is_connected && state.fb_write_async.is_none() {
            if let Some(bytes) = parse_hex(&state.frame_builder_hex) {
                state.add_terminal_line(crate::state::Direction::Tx, crate::ui::terminal::format_hex_bytes(&bytes), true);
                if let Some(ref po) = state.port_owner {
                    po.send(crate::port_owner::PortCommand::Write(bytes));
                }
            }
        }
    });
    if let Some(ref e) = state.frame_builder_error { ui.colored_label(egui::Color32::RED, e.as_str()); }
    if !state.frame_builder_hex.is_empty() {
        ui.separator();
        ui.label(egui::RichText::new(T::frame_hex(lang)).strong());
        ui.label(egui::RichText::new(&state.frame_builder_hex).monospace());
    }
}

fn parse_hex(s: &str) -> Option<Vec<u8>> {
    let s = s.replace(' ', "");
    if s.is_empty() || s.len() % 2 != 0 { return None; }
    (0..s.len()).step_by(2).filter_map(|i| u8::from_str_radix(&s[i..i+2], 16).ok()).collect::<Vec<_>>().into()
}
