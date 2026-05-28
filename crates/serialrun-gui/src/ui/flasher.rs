use crate::state::{AppState, McuType, T};
use eframe::egui;

pub fn render_flasher_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    // Poll async flasher result
    if let Some(rx) = &state.flasher_async_receiver {
        if let Ok(result) = rx.try_recv() {
            state.flasher_async_receiver = None;
            match result {
                Ok(msg) => {
                    flasher_log(state, &msg);
                    if msg == "Flash complete!" {
                        state.flasher_progress = 1.0;
                    }
                }
                Err(e) => { flasher_log(state, &e); }
            }
        }
    }

    ui.label(egui::RichText::new("Serial Flasher").strong());
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("MCU:");
        for mcu in &[McuType::Stm32, McuType::Esp32] {
            if ui.selectable_label(state.flasher_mcu == *mcu, mcu.label()).clicked() {
                state.flasher_mcu = *mcu;
            }
        }
    });
    ui.add_space(4.0);

    // File selection
    ui.horizontal(|ui| {
        ui.label("Firmware:");
        if ui.button("...").clicked() {
            let filter = match state.flasher_mcu {
                McuType::Stm32 => ("Firmware", &["hex", "bin"] as &[&str]),
                McuType::Esp32 => ("Firmware", &["bin", "elf"] as &[&str]),
            };
            if let Some(path) = rfd::FileDialog::new().add_filter(filter.0, filter.1).pick_file() {
                state.flasher_file = path.display().to_string();
            }
        }
        ui.label(&state.flasher_file);
    });
    ui.add_space(4.0);

    // Progress
    if state.flasher_progress > 0.0 && state.flasher_progress < 1.0 {
        ui.add(egui::ProgressBar::new(state.flasher_progress).text(format!("{}%", (state.flasher_progress * 100.0) as u32)));
    }
    ui.add_space(4.0);

    // Actions
    ui.horizontal(|ui| {
        if ui.button(T::connect(lang)).clicked() {
            flasher_connect(state);
        }
        if ui.button(T::erase(lang)).clicked() {
            flasher_erase(state);
        }
        if ui.button(T::flash(lang)).clicked() {
            flasher_flash(state);
        }
    });
    ui.add_space(4.0);

    // Log
    if !state.flasher_log.is_empty() {
        ui.separator();
        egui::ScrollArea::vertical().max_height(120.0).stick_to_bottom(true).show(ui, |ui| {
            for line in &state.flasher_log {
                ui.label(egui::RichText::new(line).monospace().small());
            }
        });
    }
}

fn flasher_log(state: &mut AppState, msg: &str) {
    state.flasher_log.push_back(msg.to_string());
    if state.flasher_log.len() > 100 { state.flasher_log.pop_front(); }
    state.add_log_entry(crate::state::LogLevel::Info, &format!("Flasher: {}", msg));
}

fn flasher_connect(state: &mut AppState) {
    if state.flasher_async_receiver.is_some() {
        return;
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let port_name = state.selected_port.clone().unwrap_or_default();
    let baud_rate = state.config.baud_rate;
    let mcu = state.flasher_mcu;
    state.flasher_async_receiver = Some(rx);

    if mcu == McuType::Stm32 {
        flasher_log(state, "STM32: Sending init sequence (0x7F)...");
    } else {
        flasher_log(state, "ESP32: Sending sync sequence...");
    }

    std::thread::spawn(move || {
        let config = serialrun_core::config::SerialConfig {
            port_name,
            baud_rate,
            ..Default::default()
        };
        let mut port = serialrun_core::SerialPort::new(config);
        if port.connect().is_err() {
            let _ = tx.send(Err("Connect failed".into()));
            return;
        }

        let result = if mcu == McuType::Stm32 {
            if port.write(&[0x7F]).is_ok() {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let mut buf = [0u8; 32];
                match port.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        let hex = buf[..n].iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                        Ok(format!("STM32: ACK received: {}", hex))
                    }
                    _ => Ok("STM32: No ACK received. Check BOOT0 pin.".into()),
                }
            } else {
                Err("Write failed".into())
            }
        } else {
            let sync = [0xC0, 0x08, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x07, 0x12, 0x20, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0xC0];
            if port.write(&sync).is_ok() {
                std::thread::sleep(std::time::Duration::from_millis(200));
                let mut buf = [0u8; 64];
                match port.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        let hex = buf[..n].iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                        Ok(format!("ESP32: Response: {}", hex))
                    }
                    _ => Ok("ESP32: No response. Hold BOOT and reset.".into()),
                }
            } else {
                Err("Write failed".into())
            }
        };
        let _ = port.disconnect();
        let _ = tx.send(result);
    });
}

fn flasher_erase(state: &mut AppState) {
    if state.flasher_async_receiver.is_some() {
        return;
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let port_name = state.selected_port.clone().unwrap_or_default();
    let baud_rate = state.config.baud_rate;
    let mcu = state.flasher_mcu;
    state.flasher_async_receiver = Some(rx);

    if mcu == McuType::Stm32 {
        flasher_log(state, "STM32: Mass erase command...");
    } else {
        flasher_log(state, "ESP32: Erase flash...");
    }

    std::thread::spawn(move || {
        let config = serialrun_core::config::SerialConfig {
            port_name,
            baud_rate,
            ..Default::default()
        };
        let mut port = serialrun_core::SerialPort::new(config);
        if port.connect().is_err() {
            let _ = tx.send(Err("Connect failed".into()));
            return;
        }

        let result = if mcu == McuType::Stm32 {
            let erase_cmd = [0x01, 0xFF, 0x00, 0x00];
            if port.write(&erase_cmd).is_ok() {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let mut buf = [0u8; 16];
                match port.read(&mut buf) {
                    Ok(n) if n > 0 => Ok(format!("STM32: Erase done. {} byte(s) response", n)),
                    _ => Ok("STM32: Erase timeout".into()),
                }
            } else {
                Err("Write failed".into())
            }
        } else {
            let erase = [0xC0, 0x00, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0];
            if port.write(&erase).is_ok() {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let mut buf = [0u8; 64];
                match port.read(&mut buf) {
                    Ok(n) if n > 0 => Ok(format!("ESP32: Erase response: {} byte(s)", n)),
                    _ => Ok("ESP32: Erase timeout".into()),
                }
            } else {
                Err("Write failed".into())
            }
        };
        let _ = port.disconnect();
        let _ = tx.send(result);
    });
}

fn flasher_flash(state: &mut AppState) {
    if state.flasher_file.is_empty() {
        flasher_log(state, "No firmware file selected");
        return;
    }

    if state.flasher_async_receiver.is_some() {
        return;
    }

    let firmware_path = state.flasher_file.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    let port_name = state.selected_port.clone().unwrap_or_default();
    let baud_rate = state.config.baud_rate;
    state.flasher_async_receiver = Some(rx);
    state.flasher_progress = 0.0;
    flasher_log(state, &format!("Reading firmware from {}...", firmware_path));

    std::thread::spawn(move || {
        // Read file in background thread
        let path = match std::fs::read(&firmware_path) {
            Ok(data) => data,
            Err(e) => {
                let _ = tx.send(Err(format!("Read error: {}", e)));
                return;
            }
        };

        let config = serialrun_core::config::SerialConfig {
            port_name,
            baud_rate,
            ..Default::default()
        };
        let mut port = serialrun_core::SerialPort::new(config);
        if port.connect().is_err() {
            let _ = tx.send(Err("Connect failed".into()));
            return;
        }

        let chunk_size = 128;
        let total = path.len();
        let mut offset = 0;
        while offset < total {
            let end = (offset + chunk_size).min(total);
            if port.write(&path[offset..end]).is_err() {
                let _ = port.disconnect();
                let _ = tx.send(Err("Write failed".into()));
                return;
            }
            offset = end;
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let _ = port.disconnect();
        let _ = tx.send(Ok(format!("Flash complete! {} bytes written.", total)));
    });
}
