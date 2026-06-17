use crate::state::{AppState, Language, T};
use crate::theme;
use eframe::egui;
use serialrun_core::file_transfer::{FileTransfer, TransferProtocol};
use std::sync::{Arc, Mutex};

/// Compact popup content for file transfer — renders inside the terminal input area popup.
pub fn render_ft_popup_content(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    let c = theme::get_colors(state.theme);

    // Header
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("FT").size(12.0).strong().color(c.logo_green));
        ui.label(egui::RichText::new(T::file_transfer(lang)).strong().size(13.0).color(c.text_primary));
    });
    ui.separator();
    ui.add_space(4.0);

    // Poll transfer progress
    if let Some(ref rx) = state.file_transfer_progress_rx {
        while let Ok((sent, total)) = rx.try_recv() {
            state.file_transfer_progress = if total > 0 { sent as f32 / total as f32 } else { 0.0 };
        }
    }

    // Poll transfer result
    if let Some(ref rx) = state.file_transfer_thread {
        if let Ok(result) = rx.try_recv() {
            state.file_transfer_thread = None;
            state.file_transfer_progress_rx = None;
            state.file_transfer_sending = false;
            state.file_transfer_receiving = false;
            match result {
                Ok(()) => { state.file_transfer_done = true; }
                Err(e) => { state.file_transfer_error = Some(e); }
            }
            // Restart port_owner for normal terminal operation
            if state.port_owner.is_none() && state.is_connected {
                if let Some(ref pn) = state.selected_port {
                    let mut config = state.config.clone();
                    config.port_name = pn.clone();
                    let po = crate::port_owner::PortOwnerHandle::start();
                    po.send(crate::port_owner::PortCommand::Open(config));
                    state.port_owner = Some(po);
                }
            }
        }
    }

    ui.set_min_width(220.0);

    // Protocol selector + file select button
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(T::protocol(lang)).color(c.text_secondary));
        let mut current = state.file_transfer_protocol;
        egui::ComboBox::from_id_salt("ft_popup_proto")
            .width(110.0)
            .selected_text(match current {
                TransferProtocol::Xmodem => "XMODEM",
                TransferProtocol::XmodemCrc => "XMODEM-CRC",
                TransferProtocol::Ymodem => "YMODEM",
                TransferProtocol::Zmodem => "ZMODEM",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut current, TransferProtocol::Xmodem, "XMODEM");
                ui.selectable_value(&mut current, TransferProtocol::XmodemCrc, "XMODEM-CRC");
                ui.selectable_value(&mut current, TransferProtocol::Ymodem, "YMODEM");
                ui.selectable_value(&mut current, TransferProtocol::Zmodem, "ZMODEM");
            });
        state.file_transfer_protocol = current;
        // File select button — always enabled (can pick file before connecting)
        let file_label = if state.ft_selected_file.is_some() { "✓" } else { "..." };
        let file_btn = ui.add(egui::Button::new(
            egui::RichText::new(file_label).size(12.0).color(c.text_primary)
        ).fill(c.hover_bg).rounding(4.0).min_size(egui::vec2(24.0, 20.0)));
        if file_btn.clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                state.ft_selected_file = Some(path.to_string_lossy().to_string());
                state.ft_selected_path = Some(path);
            }
        }
        if let Some(ref name) = state.ft_selected_file {
            file_btn.on_hover_text(name);
        }
    });

    // Show selected file name
    if let Some(ref name) = state.ft_selected_file {
        ui.label(egui::RichText::new(name).size(10.0).color(c.text_muted));
    }

    ui.add_space(4.0);

    // Send / Receive buttons
    let can = state.is_connected && !state.file_transfer_sending && !state.file_transfer_receiving;
    ui.horizontal(|ui| {
        let send_label = if lang == Language::Chinese { "发送文件" } else { "Send File" };
        let send_btn = ui.add_enabled(can, egui::Button::new(
            egui::RichText::new(send_label).color(egui::Color32::WHITE).strong().size(12.0)
        ).fill(c.btn_send).rounding(4.0).min_size(egui::vec2(80.0, 22.0)));
        if send_btn.clicked() {
            if let Some(ref path) = state.ft_selected_path.clone() {
                start_file_transfer(state, true, path);
            } else if let Some(path) = rfd::FileDialog::new().pick_file() {
                state.ft_selected_file = Some(path.to_string_lossy().to_string());
                state.ft_selected_path = Some(path.clone());
                start_file_transfer(state, true, &path);
            }
        }

        let recv_label = if lang == Language::Chinese { "接收文件" } else { "Receive" };
        let recv_btn = ui.add_enabled(can, egui::Button::new(
            egui::RichText::new(recv_label).color(egui::Color32::WHITE).strong().size(12.0)
        ).fill(c.info).rounding(4.0).min_size(egui::vec2(80.0, 22.0)));
        if recv_btn.clicked() {
            if let Some(path) = rfd::FileDialog::new().save_file() {
                start_file_transfer(state, false, &path);
            }
        }
    });

    ui.add_space(4.0);

    // Status
    if state.file_transfer_done {
        ui.label(egui::RichText::new(T::done(lang)).color(c.success));
    } else if let Some(ref e) = state.file_transfer_error {
        ui.label(egui::RichText::new(format!("Error: {}", e)).color(c.error));
    } else if state.file_transfer_sending {
        ui.label(egui::RichText::new(T::sending(lang)).color(c.text_secondary));
    } else if state.file_transfer_receiving {
        ui.label(egui::RichText::new(T::receiving(lang)).color(c.text_secondary));
    }

    // Progress bar
    if state.file_transfer_sending || state.file_transfer_receiving {
        ui.add_space(4.0);
        ui.add(egui::ProgressBar::new(state.file_transfer_progress)
            .text(format!("{:.0}%", state.file_transfer_progress * 100.0)));
    }
}

fn start_file_transfer(state: &mut AppState, send: bool, path: &std::path::Path) {
    let port_name = match state.selected_port.clone() {
        Some(p) if !p.is_empty() => p,
        _ => {
            state.file_transfer_error = Some("No serial port selected".into());
            return;
        }
    };

    state.file_transfer_sending = send;
    state.file_transfer_receiving = !send;
    state.file_transfer_done = false;
    state.file_transfer_error = None;
    state.file_transfer_progress = 0.0;
    let baud_rate = state.config.baud_rate;
    let proto = state.file_transfer_protocol;
    let file_path = path.to_path_buf();

    if let Some(po) = state.port_owner.take() {
        po.wait_for_release();
    }

    let (result_tx, result_rx) = std::sync::mpsc::channel();
    let (progress_tx, progress_rx) = std::sync::mpsc::channel();
    state.file_transfer_thread = Some(result_rx);
    state.file_transfer_progress_rx = Some(progress_rx);

    std::thread::spawn(move || {
        let config = serialrun_core::config::SerialConfig {
            port_name,
            baud_rate,
            ..Default::default()
        };
        let mut port = serialrun_core::SerialPort::new(config);
        if port.connect().is_err() {
            let _ = result_tx.send(Err("Connect failed".into()));
            return;
        }

        let shared = Arc::new(Mutex::new(port));
        let transfer = FileTransfer::new(proto);

        let p = shared.clone();
        let wf = move |d: &[u8]| -> Result<(), serialrun_core::file_transfer::TransferError> {
            let mut port = p.lock().unwrap();
            port.write(d).map_err(|e| serialrun_core::file_transfer::TransferError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
            Ok(())
        };
        let p = shared.clone();
        let rf = || -> Result<u8, serialrun_core::file_transfer::TransferError> {
            let mut port = p.lock().unwrap();
            let mut b = [0u8; 1];
            match port.read(&mut b) {
                Ok(1) => Ok(b[0]),
                _ => Ok(0),
            }
        };
        let pt = progress_tx;

        let result = if send {
            transfer.send_file(&file_path, wf, rf, |p| { let _ = pt.send((p.bytes_transferred, p.total_bytes)); })
        } else {
            transfer.receive_file(&file_path, wf, rf, |p| { let _ = pt.send((p.bytes_transferred, p.total_bytes)); })
        };

        let _ = shared.lock().unwrap().disconnect();
        let _ = result_tx.send(result.map_err(|e| e.to_string()));
    });
}
