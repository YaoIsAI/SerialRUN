#![windows_subsystem = "windows"]

mod app;
mod async_utils;
mod icon;
mod mcp_server;
mod plc_presets;
mod plugin_callbacks;
mod port_owner;
mod state;
pub mod theme;
mod ui;
pub mod util;

use eframe::egui;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Start MCP manager (does not bind yet — waits for Start command)
    let mcp_handle = mcp_server::McpHandle::start();
    // Send initial start command
    mcp_handle.send(mcp_server::McpCommand::Start {
        bind_addr: "127.0.0.1".into(),
        port: 9527,
    });

    let icon_data = icon::generate_icon().map(|d| std::sync::Arc::new(d));
    let options = eframe::NativeOptions {
        viewport: {
            let mut vb = egui::ViewportBuilder::default()
                .with_inner_size([900.0, 600.0])
                .with_min_inner_size([700.0, 500.0])
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
            Ok(Box::new(app::SerialRunApp::new(cc, mcp_handle)))
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Microsoft YaHei - best CJK rendering quality (Chinese/Japanese)
    let font_data = include_bytes!("../fonts/msyh.ttc");
    fonts.font_data.insert(
        "msyh".to_owned(),
        egui::FontData::from_static(font_data),
    );

    // NotoSansSC - covers scripts msyh misses (Korean, Thai, etc.)
    if let Ok(noto_data) = std::fs::read("C:\\Windows\\Fonts\\NotoSansSC-VF.ttf") {
        fonts.font_data.insert(
            "noto".to_owned(),
            egui::FontData::from_owned(noto_data),
        );
        // NotoSansSC as fallback for glyphs msyh doesn't have
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.insert(0, "msyh".to_owned());
            family.push("noto".to_owned());
        }
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            family.insert(0, "msyh".to_owned());
            family.push("noto".to_owned());
        }
    } else {
        // Fallback: msyh only
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.insert(0, "msyh".to_owned());
        }
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            family.insert(0, "msyh".to_owned());
        }
    }

    ctx.set_fonts(fonts);
}
