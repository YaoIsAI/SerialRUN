use crate::state::{AppState, Language, Theme, T};
use crate::ui;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct SerialTapApp {
    state: Arc<Mutex<AppState>>,
    current_theme: Theme,
}

impl SerialTapApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut state = AppState::new();
        state.language = Language::Chinese;
        state.theme = Theme::Dark;

        // Apply dark theme immediately
        let mut visuals = egui::Visuals::dark();
        visuals.window_rounding = egui::Rounding::same(8.0);
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(6.0);
        visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
        visuals.widgets.active.rounding = egui::Rounding::same(6.0);
        cc.egui_ctx.set_visuals(visuals);

        Self {
            state: Arc::new(Mutex::new(state)),
            current_theme: Theme::Dark,
        }
    }
}

impl eframe::App for SerialTapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut state = self.state.lock().unwrap();
        let lang = state.language;

        // Apply theme every frame to ensure it sticks
        if state.theme != self.current_theme {
            let mut visuals = match state.theme {
                Theme::Dark => egui::Visuals::dark(),
                Theme::Light => egui::Visuals::light(),
            };
            visuals.window_rounding = egui::Rounding::same(8.0);
            visuals.widgets.noninteractive.rounding = egui::Rounding::same(6.0);
            visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
            visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
            visuals.widgets.active.rounding = egui::Rounding::same(6.0);
            ctx.set_visuals(visuals);
            self.current_theme = state.theme;
        }

        // Top bar: title + connection + lang/theme buttons
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui::connection::render_connection_panel(ui, &mut state, ctx);
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui::status::render_status_bar(ui, &state);
        });

        // Left panel: collapsible settings + help
        egui::SidePanel::left("side_panel")
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui::settings::render_settings_panel(ui, &mut state, ctx);
            });

        // Center: terminal
        egui::CentralPanel::default().show(ctx, |ui| {
            ui::terminal::render_terminal_panel(ui, &mut state);
        });

        // Floating windows
        let mut show_chart = state.show_chart_window;
        if show_chart {
            egui::Window::new(T::data_chart(lang))
                .open(&mut show_chart)
                .show(ctx, |ui| {
                    ui::charts::render_chart_panel(ui, &state);
                });
        }
        state.show_chart_window = show_chart;

        let mut show_log = state.show_log_window;
        if show_log {
            egui::Window::new(T::log_viewer(lang))
                .open(&mut show_log)
                .show(ctx, |ui| {
                    ui::log_viewer::render_log_panel(ui, &state);
                });
        }
        state.show_log_window = show_log;

        let mut show_help = state.show_help;
        if show_help {
            egui::Window::new(T::help(lang))
                .open(&mut show_help)
                .default_width(500.0)
                .default_height(400.0)
                .show(ctx, |ui| {
                    ui::help::render_help_panel(ui, lang);
                });
        }
        state.show_help = show_help;
    }
}
