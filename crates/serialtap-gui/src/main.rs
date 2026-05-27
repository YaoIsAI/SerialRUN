#![windows_subsystem = "windows"]

mod app;
mod icon;
mod state;
mod ui;

use eframe::egui;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let icon_data = icon::generate_icon();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([700.0, 400.0])
            .with_title("SerialTap")
            .with_icon(std::sync::Arc::new(icon_data)),
        ..Default::default()
    };

    eframe::run_native(
        "SerialTap",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(app::SerialTapApp::new(cc)))
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let font_data = include_bytes!("../fonts/msyh.ttc");
    fonts.font_data.insert(
        "msyh".to_owned(),
        egui::FontData::from_static(font_data),
    );

    fonts.families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "msyh".to_owned());

    fonts.families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(0, "msyh".to_owned());

    ctx.set_fonts(fonts);
}
