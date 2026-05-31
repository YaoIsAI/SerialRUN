use crate::state::{AppState, Language};
use crate::theme;
use eframe::egui;

pub fn render_mcp_log_popup(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    let c = theme::get_colors(state.theme);
    let title = if lang == Language::Chinese { "MCP 访问日志" } else { "MCP Access Log" };
    let count = state.mcp_access_log.len();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(title).strong().color(c.text_primary));
        ui.separator();
        ui.label(egui::RichText::new(
            format!("{} {}", count, if lang == Language::Chinese { "条记录" } else { "entries" })
        ).color(c.text_muted));
    });
    ui.separator();

    let available_height = ui.available_height() - 40.0;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .max_height(available_height)
        .show(ui, |ui| {
            for entry in state.mcp_access_log.iter().rev() {
                let (color, level_str) = match entry.action.as_str() {
                    "CONNECT" => (c.mcp_connect, "CONN"),
                    "DISCONNECT" => (c.mcp_disconnect, "DISC"),
                    "CALL" => (c.mcp_call, "CALL"),
                    _ => (c.text_muted, "????"),
                };
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new(format!("[{}]", entry.timestamp)).color(c.timestamp_color).monospace());
                    ui.label(egui::RichText::new(level_str).color(color).strong());
                    ui.label(egui::RichText::new(&entry.client_ip).color(c.text_secondary).strong());
                    ui.label(egui::RichText::new(&entry.detail).color(c.text_primary));
                });
            }
        });

    ui.separator();
    ui.horizontal(|ui| {
        if ui.button(egui::RichText::new(
            if lang == Language::Chinese { "清空日志" } else { "Clear Log" }
        ).color(c.error)).clicked() {
            state.mcp_access_log.clear();
            state.save_mcp_log();
        }
    });
}
