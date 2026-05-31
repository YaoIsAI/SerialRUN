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

    ui.label(egui::RichText::new(T::serial_flasher(lang)).strong());
    ui.separator();

    ui.horizontal(|ui| {
        ui.label(T::mcu_label(lang));
        for mcu in &[McuType::Stm32, McuType::Esp32] {
            if ui.selectable_label(state.flasher_mcu == *mcu, mcu.label()).clicked() {
                state.flasher_mcu = *mcu;
            }
        }
    });
    ui.add_space(4.0);

    // File selection
    ui.horizontal(|ui| {
        ui.label(T::firmware(lang));
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
    if state.flasher_log.len() > 500 { state.flasher_log.pop_front(); }
    state.add_log_entry(crate::state::LogLevel::Info, &format!("Flasher: {}", msg));
}

fn flasher_connect(state: &mut AppState) {
    if state.flasher_async_receiver.is_some() { return; }
    let po = match state.port_owner.as_ref() { Some(p) => p.cmd_tx(), None => return };
    let mcu = state.flasher_mcu;
    let (tx, rx) = std::sync::mpsc::channel();
    state.flasher_async_receiver = Some(rx);

    if mcu == McuType::Stm32 {
        flasher_log(state, "STM32: Sending init sequence (0x7F)...");
    } else {
        flasher_log(state, "ESP32: Sending sync sequence...");
    }

    std::thread::spawn(move || {
        let result = if mcu == McuType::Stm32 {
            let (resp_tx, resp_rx) = std::sync::mpsc::channel();
            let _ = po.send(crate::port_owner::PortCommand::WriteRead { data: vec![0x7F], timeout_ms: 200, resp_tx });
            match resp_rx.recv() {
                Ok(Ok(buf)) if !buf.is_empty() => {
                    let hex = buf.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                    Ok(format!("STM32: ACK received: {}", hex))
                }
                _ => Ok("STM32: No ACK received. Check BOOT0 pin.".into()),
            }
        } else {
            let sync = vec![0xC0, 0x08, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x07, 0x12, 0x20, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0xC0];
            let (resp_tx, resp_rx) = std::sync::mpsc::channel();
            let _ = po.send(crate::port_owner::PortCommand::WriteRead { data: sync, timeout_ms: 300, resp_tx });
            match resp_rx.recv() {
                Ok(Ok(buf)) if !buf.is_empty() => {
                    let hex = buf.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                    Ok(format!("ESP32: Response: {}", hex))
                }
                _ => Ok("ESP32: No response. Hold BOOT and reset.".into()),
            }
        };
        let _ = tx.send(result);
    });
}

fn flasher_erase(state: &mut AppState) {
    if state.flasher_async_receiver.is_some() { return; }
    let po = match state.port_owner.as_ref() { Some(p) => p.cmd_tx(), None => return };
    let mcu = state.flasher_mcu;
    let (tx, rx) = std::sync::mpsc::channel();
    state.flasher_async_receiver = Some(rx);

    if mcu == McuType::Stm32 {
        flasher_log(state, "STM32: Mass erase command...");
    } else {
        flasher_log(state, "ESP32: Erase flash...");
    }

    std::thread::spawn(move || {
        let result = if mcu == McuType::Stm32 {
            let (resp_tx, resp_rx) = std::sync::mpsc::channel();
            let _ = po.send(crate::port_owner::PortCommand::WriteRead { data: vec![0x01, 0xFF, 0x00, 0x00], timeout_ms: 1000, resp_tx });
            match resp_rx.recv() {
                Ok(Ok(buf)) if !buf.is_empty() => Ok(format!("STM32: Erase done. {} byte(s) response", buf.len())),
                _ => Ok("STM32: Erase timeout".into()),
            }
        } else {
            let (resp_tx, resp_rx) = std::sync::mpsc::channel();
            let _ = po.send(crate::port_owner::PortCommand::WriteRead { data: vec![0xC0, 0x00, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0], timeout_ms: 1000, resp_tx });
            match resp_rx.recv() {
                Ok(Ok(buf)) if !buf.is_empty() => Ok(format!("ESP32: Erase response: {} byte(s)", buf.len())),
                _ => Ok("ESP32: Erase timeout".into()),
            }
        };
        let _ = tx.send(result);
    });
}

fn flasher_flash(state: &mut AppState) {
    if state.flasher_file.is_empty() {
        flasher_log(state, "No firmware file selected");
        return;
    }
    if state.flasher_async_receiver.is_some() { return; }
    let po = match state.port_owner.as_ref() { Some(p) => p.cmd_tx(), None => return };

    let firmware_path = state.flasher_file.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    state.flasher_async_receiver = Some(rx);
    state.flasher_progress = 0.0;
    flasher_log(state, &format!("Reading firmware from {}...", firmware_path));

    std::thread::spawn(move || {
        let firmware = match std::fs::read(&firmware_path) {
            Ok(data) => data,
            Err(e) => { let _ = tx.send(Err(format!("Read error: {}", e))); return; }
        };

        let flasher = serialrun_core::protocol::flasher::Stm32Flasher::new();

        // Send init sequence
        let (resp_tx, resp_rx) = std::sync::mpsc::channel();
        let _ = po.send(crate::port_owner::PortCommand::WriteRead { data: vec![0x7F], timeout_ms: 200, resp_tx });
        match resp_rx.recv() {
            Ok(Ok(buf)) if !buf.is_empty() && buf[0] == 0x79 => {}
            _ => { let _ = tx.send(Err("No ACK from bootloader. Check BOOT0 pin.".into())); return; }
        }

        // Flash in 128-byte chunks
        let chunk_size = 128;
        let total = firmware.len();
        let mut offset = 0;
        let mut addr: u32 = 0x08000000;

        while offset < total {
            let end = (offset + chunk_size).min(total);
            let chunk = &firmware[offset..end];
            let cmd = flasher.write_memory(addr, chunk);

            let (resp_tx, resp_rx) = std::sync::mpsc::channel();
            let _ = po.send(crate::port_owner::PortCommand::WriteRead { data: cmd, timeout_ms: 200, resp_tx });
            match resp_rx.recv() {
                Ok(Ok(ack)) if !ack.is_empty() && ack[0] == 0x79 => {}
                _ => { let _ = tx.send(Err(format!("No ACK at offset 0x{:X}", offset))); return; }
            }

            offset = end;
            addr += chunk.len() as u32;
        }

        let _ = tx.send(Ok(format!("Flash complete! {} bytes written.", total)));
    });
}
