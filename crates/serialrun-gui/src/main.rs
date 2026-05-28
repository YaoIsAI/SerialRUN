#![windows_subsystem = "windows"]

mod app;
mod async_utils;
mod icon;
mod mcp_server;
mod plc_presets;
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

    // Start MCP server in background thread
    std::thread::spawn(|| {
        mcp_server::run_mcp_server();
    });

    let icon_data = icon::generate_icon().map(|d| std::sync::Arc::new(d));
    let options = eframe::NativeOptions {
        viewport: {
            let mut vb = egui::ViewportBuilder::default()
                .with_inner_size([900.0, 600.0])
                .with_min_inner_size([700.0, 400.0])
                .with_title("SerialRUN");
            if let Some(icon) = icon_data {
                vb = vb.with_icon(icon);
            }
            vb
        },
        ..Default::default()
    };

    eframe::run_native(
        "SerialRUN",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(app::SerialRunApp::new(cc)))
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

    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        family.insert(0, "msyh".to_owned());
    }
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        family.insert(0, "msyh".to_owned());
    }

    ctx.set_fonts(fonts);
}
