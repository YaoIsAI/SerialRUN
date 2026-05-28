use crate::state::{AppState, Language, Theme, T};
use crate::ui;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct SerialRunApp {
    state: Arc<Mutex<AppState>>,
    current_theme: Theme,
}

impl SerialRunApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut state = AppState::new();
        state.language = Language::Chinese;
        state.theme = Theme::Dark;
        let mut visuals = egui::Visuals::dark();
        visuals.window_rounding = egui::Rounding::same(8.0);
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(6.0);
        visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
        visuals.widgets.active.rounding = egui::Rounding::same(6.0);
        cc.egui_ctx.set_visuals(visuals);
        Self { state: Arc::new(Mutex::new(state)), current_theme: Theme::Dark }
    }
}

impl eframe::App for SerialRunApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let lang = state.language;

        if state.theme != self.current_theme {
            let mut visuals = match state.theme {
                Theme::Dark => egui::Visuals::dark(),
                Theme::Light => {
                    let mut v = egui::Visuals::light();
                    v.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(230, 230, 235);
                    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(30, 30, 30));
                    v.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(200, 200, 210);
                    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 0, 0));
                    v.widgets.active.weak_bg_fill = egui::Color32::from_rgb(170, 170, 185);
                    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 0, 0));
                    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 50));
                    v
                }
            };
            visuals.window_rounding = egui::Rounding::same(8.0);
            visuals.widgets.noninteractive.rounding = egui::Rounding::same(6.0);
            visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
            visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
            visuals.widgets.active.rounding = egui::Rounding::same(6.0);
            ctx.set_visuals(visuals);
            self.current_theme = state.theme;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui::connection::render_connection_panel(ui, &mut state, ctx);
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui::status::render_status_bar(ui, &state);
        });

        egui::SidePanel::left("side_panel").resizable(false).default_width(200.0).show(ctx, |ui| {
            ui::connection::render_connection_controls(ui, &mut state);
            ui::settings::render_settings_panel(ui, &mut state, ctx);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui::terminal::render_terminal_panel(ui, &mut state);
        });

        // Floating windows
        macro_rules! toggle_window {
            ($show:expr, $title:expr, $render:expr, $w:expr, $h:expr) => {
                let mut show = $show;
                if show {
                    egui::Window::new($title).open(&mut show).default_width($w).default_height($h).show(ctx, |ui| { $render(ui, &mut state); });
                }
                $show = show;
            };
        }

        toggle_window!(state.show_chart_window, T::data_chart(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::charts::render_chart_panel(ui, s), 400.0, 300.0);
        toggle_window!(state.show_log_window, T::log_viewer(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::log_viewer::render_log_panel(ui, s), 400.0, 350.0);
        toggle_window!(state.show_help, T::help(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::help::render_help_panel(ui, s), 600.0, 500.0);
        toggle_window!(state.show_modbus_window, T::modbus_panel(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::modbus::render_modbus_panel(ui, s), 520.0, 450.0);
        toggle_window!(state.show_plc_window, T::plc_control(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::plc::render_plc_panel(ui, s), 600.0, 500.0);
        toggle_window!(state.show_checksum_window, T::checksum(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::checksum::render_checksum_panel(ui, s), 400.0, 350.0);
        toggle_window!(state.show_file_transfer_window, T::file_transfer(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::file_transfer::render_file_transfer_panel(ui, s), 420.0, 300.0);
        toggle_window!(state.show_frame_builder_window, T::frame_builder(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::frame_builder::render_frame_builder_panel(ui, s), 450.0, 350.0);
        toggle_window!(state.show_data_logger_window, T::data_logger(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::data_logger::render_data_logger_panel(ui, s), 400.0, 250.0);
        toggle_window!(state.show_can_window, T::can_analyzer(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::can_analyzer::render_can_analyzer_panel(ui, s), 550.0, 400.0);
        toggle_window!(state.show_i2c_spi_window, T::i2c_spi(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::i2c_spi::render_i2c_spi_panel(ui, s), 450.0, 380.0);
        toggle_window!(state.show_scope_window, T::oscilloscope(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::serial_scope::render_serial_scope_panel(ui, s), 600.0, 480.0);
        toggle_window!(state.show_flasher_window, T::flasher(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::flasher::render_flasher_panel(ui, s), 420.0, 350.0);
        toggle_window!(state.show_register_editor_window, T::register_editor(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::register_editor::render_register_editor_panel(ui, s), 500.0, 400.0);
        toggle_window!(state.show_plugin_window, T::plugins(lang), |ui: &mut egui::Ui, s: &mut AppState| ui::plugin::render_plugin_panel(ui, s), 480.0, 400.0);
    }
}
