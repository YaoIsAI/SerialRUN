use crate::state::{AppState, Language, T};
use crate::theme;
use eframe::egui;

/// Logo green — consistent across status bar, MCP, and other indicators
pub const LOGO_GREEN: egui::Color32 = egui::Color32::from_rgb(76, 175, 80);

fn format_rate(rate: f64) -> String {
    if rate >= 1024.0 * 1024.0 {
        format!("{:.1} M", rate / (1024.0 * 1024.0))
    } else if rate >= 1024.0 {
        format!("{:.1} K", rate / 1024.0)
    } else {
        format!("{:.0}", rate)
    }
}

pub fn render_status_bar(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    let c = theme::get_colors(state.theme);

    // Auto-expire error display
    state.clear_error_if_expired();

    ui.horizontal(|ui| {
        let status_color = if state.is_connected { c.success } else { c.error };

        let status_text = if state.is_connected {
            format!("{}: {}", T::connected(lang), state.selected_port.as_deref().unwrap_or("N/A"))
        } else {
            T::disconnected(lang).to_string()
        };

        ui.label(egui::RichText::new(status_text).color(status_color));

        ui.separator();

        ui.label(egui::RichText::new(format!("{}: {}", T::baud_rate(lang), state.config.baud_rate)).color(c.text_secondary));

        ui.separator();

        // Calculate data rate every second
        let now = chrono::Utc::now().timestamp_millis();
        if now - state.rate_last_check >= 1000 {
            let dt = (now - state.rate_last_check) as f64 / 1000.0;
            if dt > 0.0 {
                state.rx_rate = (state.rx_count - state.rate_last_rx) as f64 / dt;
                state.tx_rate = (state.tx_count - state.rate_last_tx) as f64 / dt;
            }
            state.rate_last_rx = state.rx_count;
            state.rate_last_tx = state.tx_count;
            state.rate_last_check = now;
        }

        let rate_label = if lang == Language::Chinese { "速率" } else { "Rate" };
        ui.label(egui::RichText::new(format!("RX: {} {} ({} {}/s)", state.rx_count, T::bytes(lang), format_rate(state.rx_rate), rate_label)).color(c.text_secondary));
        ui.label(egui::RichText::new(format!("TX: {} {} ({} {}/s)", state.tx_count, T::bytes(lang), format_rate(state.tx_rate), rate_label)).color(c.text_secondary));

        if state.recording {
            ui.separator();
            ui.label(egui::RichText::new(format!("● {}", T::recording(lang))).color(c.error));
        }

        // AI connection indicator
        if state.ai_connected {
            ui.separator();
            let ai_text = format!("AI: {} @ {} baud", state.ai_port_name, state.ai_baud_rate);
            ui.label(egui::RichText::new(ai_text).color(egui::Color32::from_rgb(255, 193, 7)).strong());
        }

        // Auto-detect baud rate spinner
        if state.auto_detect_running {
            ui.ctx().request_repaint();
            let spinner_chars = ['-', '\\', '|', '/'];
            let t = ui.ctx().input(|i| i.time);
            let idx = (t * 6.0) as usize % spinner_chars.len();
            let spinner = spinner_chars[idx];
            let progress_text = match state.auto_detect_progress {
                Some(baud) => format!("{} {}...", spinner, baud),
                None => {
                    let base = if lang == Language::Chinese { "检测波特率" } else { "Detecting baud" };
                    format!("{} {}...", spinner, base)
                }
            };
            ui.separator();
            ui.label(egui::RichText::new(progress_text).color(egui::Color32::from_rgb(100, 180, 255)).strong());
        }

        // Show auto-detect result for 3 seconds after completion
        if let Some(baud) = state.auto_detect_result {
            let now = chrono::Utc::now().timestamp_millis();
            if now - state.auto_detect_result_time < 3000 {
                ui.ctx().request_repaint();
                ui.separator();
                let msg = if lang == Language::Chinese {
                    format!("\u{2714} 检测到波特率: {}", baud)
                } else {
                    format!("\u{2714} Detected baud: {}", baud)
                };
                ui.label(egui::RichText::new(msg).color(egui::Color32::from_rgb(34, 197, 94)).strong());
            } else {
                state.auto_detect_result = None;
            }
        }

        // Show current error/warning message inline (red for errors)
        if let Some(ref err) = state.global_error {
            ui.separator();
            ui.label(egui::RichText::new("\u{2716}").color(c.error).size(13.0));
            ui.label(egui::RichText::new(err.as_str()).color(c.error).strong());
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Version
            ui.label(egui::RichText::new("SerialRUN v0.4.0").color(c.text_muted));

            // Warning history: red dot + count
            let warning_count = state.warning_history.len();
            if warning_count > 0 {
                ui.separator();
                let dot_label = format!("\u{25CF} {}", warning_count);
                if ui.add(egui::Button::new(
                    egui::RichText::new(&dot_label).color(c.error).strong().size(12.0)
                ).fill(egui::Color32::TRANSPARENT).frame(false)).on_hover_text(
                    if lang == Language::Chinese { "点击查看警告/错误历史" } else { "Click to view warning/error history" }
                ).clicked() {
                    state.show_warning_popup = !state.show_warning_popup;
                }
            }
        });
    });

    // Warning history popup window
    if state.show_warning_popup {
        let popup_title = if lang == Language::Chinese { "⚠ 警告 / 错误历史" } else { "⚠ Warning / Error History" };
        let mut open = state.show_warning_popup;
        egui::Window::new(popup_title)
            .open(&mut open)
            .default_width(420.0)
            .default_height(350.0)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                let warning_count = state.warning_history.len();
                ui.label(egui::RichText::new(
                    format!("{} {}", warning_count, if lang == Language::Chinese { "条记录" } else { "entries" })
                ).color(c.text_muted));
                ui.separator();

                egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                    for entry in state.warning_history.iter().rev().take(50) {
                        let ts = chrono::DateTime::from_timestamp_millis(entry.timestamp)
                            .map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S%.3f").to_string())
                            .unwrap_or_default();
                        ui.horizontal_wrapped(|ui| {
                            ui.label(egui::RichText::new(format!("[{}]", ts)).color(c.timestamp_color).monospace());
                            ui.label(egui::RichText::new(&entry.message).color(c.error));
                        });
                    }
                });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button(egui::RichText::new(
                        if lang == Language::Chinese { "清空历史" } else { "Clear History" }
                    ).color(c.error)).clicked() {
                        state.warning_history.clear();
                        state.save_warnings();
                        state.show_warning_popup = false;
                    }
                });
            });
        state.show_warning_popup = open;
    }
}
