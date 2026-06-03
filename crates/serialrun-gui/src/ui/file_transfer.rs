use crate::state::{AppState, T};
use eframe::egui;
use serialrun_core::file_transfer::{FileTransfer, TransferProtocol};
use std::sync::{Arc, Mutex};

pub fn render_file_transfer_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    ui.horizontal(|ui| {
        ui.heading(T::file_transfer(lang));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.button(egui::RichText::new("?").size(14.0).strong())
                .on_hover_text(T::file_transfer_tip_header(lang));
        });
    });
    ui.separator();

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

    ui.label(T::protocol(lang));
    ui.horizontal(|ui| {
        let mut current = state.file_transfer_protocol;
        egui::ComboBox::from_id_salt("ft_proto").selected_text(match current { TransferProtocol::Xmodem => "XMODEM", TransferProtocol::XmodemCrc => "XMODEM-CRC", TransferProtocol::Ymodem => "YMODEM", TransferProtocol::Zmodem => "ZMODEM" }).show_ui(ui, |ui| {
            ui.selectable_value(&mut current, TransferProtocol::Xmodem, "XMODEM");
            ui.selectable_value(&mut current, TransferProtocol::XmodemCrc, "XMODEM-CRC");
            ui.selectable_value(&mut current, TransferProtocol::Ymodem, "YMODEM");
            ui.selectable_value(&mut current, TransferProtocol::Zmodem, "ZMODEM");
        });
        state.file_transfer_protocol = current;
    });
    ui.add_space(8.0);

    // Progress bar
    if state.file_transfer_sending || state.file_transfer_receiving {
        ui.add(egui::ProgressBar::new(state.file_transfer_progress).text(format!("{:.0}%", state.file_transfer_progress * 100.0)));
        ui.add_space(4.0);
    }

    if state.file_transfer_done { ui.label(egui::RichText::new(T::done(lang)).color(egui::Color32::GREEN)); }
    else if let Some(ref e) = state.file_transfer_error { ui.label(egui::RichText::new(format!("Error: {}", e)).color(egui::Color32::RED)); }
    else if state.file_transfer_sending { ui.label(T::sending(lang)); }
    else if state.file_transfer_receiving { ui.label(T::receiving(lang)); }
    else { ui.label(T::ready(lang)); }
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let can = state.is_connected && !state.file_transfer_sending && !state.file_transfer_receiving;
        if ui.add_enabled(can, egui::Button::new(T::send_file(lang))).clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                start_file_transfer(state, true, &path);
            }
        }
        if ui.add_enabled(can, egui::Button::new(T::receive_file(lang))).clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("All", &["*"]).save_file() {
                start_file_transfer(state, false, &path);
            }
        }
    });
}

fn start_file_transfer(state: &mut AppState, send: bool, path: &std::path::Path) {
    // Validate port is selected
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

    // Stop port_owner and wait for port release before opening exclusive file transfer
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
