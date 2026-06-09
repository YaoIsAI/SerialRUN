#![windows_subsystem = "windows"]

mod app;
mod async_utils;
pub mod cli;
mod icon;
mod mcp_server;
mod plc_presets;
mod plugin_callbacks;
mod port_owner;
mod state;
pub mod theme;
mod ui;
pub mod util;

use clap::Parser;
use eframe::egui;

fn main() -> eframe::Result<()> {
    // Check if CLI arguments are provided
    let args: Vec<String> = std::env::args().collect();

    // Known CLI subcommands
    let cli_commands = [
        "interactive", "list-ports", "connect", "disconnect",
        "send", "read", "send-command", "modbus-read", "modbus-write",
        "status", "plugin",
    ];

    // Check if first arg is a known subcommand (not starting with -)
    let is_cli = args.len() > 1
        && !args[1].starts_with('-')
        && cli_commands.contains(&args[1].as_str());

    if is_cli {
        // CLI mode
        let cli = cli::Cli::parse();
        cli::run_cli(cli, None);
        return Ok(());
    }

    // GUI mode
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
            // Apply saved theme visuals before first frame (eframe defaults to Dark)
            let prefs: crate::state::UserPrefs = {
                let path = if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
                    std::path::PathBuf::from(home).join(".serialrun").join("config.toml")
                } else {
                    std::path::PathBuf::from(".serialrun").join("config.toml")
                };
                std::fs::read_to_string(&path).ok().and_then(|c| toml::from_str(&c).ok()).unwrap_or_default()
            };
            let mut visuals = match prefs.theme {
                crate::state::Theme::Dark => egui::Visuals::dark(),
                crate::state::Theme::Light => {
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
            cc.egui_ctx.set_visuals(visuals);
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
