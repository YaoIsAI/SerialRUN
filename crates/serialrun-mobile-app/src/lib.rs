//! SerialRUN Mobile App
//!
//! Android serial port terminal with egui interface and MCP support.

pub mod app;
pub mod mcp;

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    use eframe::egui;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("com.serialrun.mobile")
            .with_title("SerialRUN")
            .with_fullscreen(true),
        ..Default::default()
    };

    eframe::run_native(
        "SerialRUN",
        options,
        Box::new(|cc| Ok(Box::new(app::MobileApp::new(cc)))),
    )
    .unwrap();
}

#[cfg(not(target_os = "android"))]
pub fn run_desktop() {
    use eframe::egui;

    // Load app icon
    let icon_data = include_bytes!("../../../scripts/icon_preview.png");
    let icon = image::load_from_memory(icon_data)
        .ok()
        .map(|img| {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let pixels = rgba.into_raw();
            egui::IconData {
                rgba: pixels,
                width: size[0] as u32,
                height: size[1] as u32,
            }
        });

    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([390.0, 844.0])
            .with_min_inner_size([320.0, 500.0])
            .with_title("SerialRUN Mobile"),
        ..Default::default()
    };

    if let Some(icon) = icon {
        options.viewport = options.viewport.with_icon(icon);
    }

    eframe::run_native(
        "SerialRUN",
        options,
        Box::new(|cc| Ok(Box::new(app::MobileApp::new(cc)))),
    )
    .unwrap();
}
