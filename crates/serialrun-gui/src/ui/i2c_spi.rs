use crate::state::{AppState, I2cMode, T};
use eframe::egui;

pub fn render_i2c_spi_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    // Poll async I2C/SPI result
    if let Some(rx) = &state.i2c_async_receiver {
        if let Ok(result) = rx.try_recv() {
            state.i2c_async_receiver = None;
            match result {
                Ok(text) => { state.i2c_result = text; }
                Err(e) => { state.i2c_result = e; }
            }
        }
    }

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(T::i2c_spi(lang)).strong());
        ui.separator();
        for mode in &[I2cMode::I2C, I2cMode::SPI] {
            if ui.selectable_label(state.i2c_mode == *mode, mode.label()).clicked() {
                state.i2c_mode = *mode;
            }
        }
    });
    ui.add_space(4.0);

    match state.i2c_mode {
        I2cMode::I2C => render_i2c(ui, state),
        I2cMode::SPI => render_spi(ui, state),
    }
}

fn render_i2c(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    egui::Grid::new("i2c_grid").num_columns(2).show(ui, |ui| {
        ui.label("Address (hex):"); ui.text_edit_singleline(&mut state.i2c_address); ui.end_row();
        ui.label("Register (hex):"); ui.text_edit_singleline(&mut state.i2c_register); ui.end_row();
        ui.label("Data (hex):"); ui.text_edit_singleline(&mut state.i2c_data); ui.end_row();
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui.button(T::scan(lang)).clicked() { i2c_scan(state); }
        if ui.button("Read").clicked() { i2c_read(state); }
        if ui.button("Write").clicked() { i2c_write(state); }
    });
    ui.add_space(4.0);
    if !state.i2c_result.is_empty() {
        ui.separator();
        ui.label(egui::RichText::new("Result:").strong());
        egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
            ui.label(egui::RichText::new(&state.i2c_result).monospace());
        });
    }
}

fn render_spi(ui: &mut egui::Ui, state: &mut AppState) {
    egui::Grid::new("spi_grid").num_columns(2).show(ui, |ui| {
        ui.label("MOSI (hex):"); ui.text_edit_singleline(&mut state.i2c_data); ui.end_row();
    });
    ui.add_space(4.0);
    if ui.button("Transfer").clicked() { spi_transfer(state); }
    ui.add_space(4.0);
    if !state.i2c_result.is_empty() {
        ui.separator();
        ui.label(egui::RichText::new("Result:").strong());
        egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
            ui.label(egui::RichText::new(&state.i2c_result).monospace());
        });
    }
}

fn parse_hex_bytes(s: &str) -> Option<Vec<u8>> {
    let s = s.replace(' ', "").replace("0x", "").replace("0X", "");
    if s.is_empty() || s.len() % 2 != 0 { return None; }
    (0..s.len()).step_by(2).filter_map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect::<Vec<_>>().into()
}

fn i2c_scan(state: &mut AppState) {
    if state.i2c_async_receiver.is_some() {
        return;
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let port_name = state.selected_port.clone().unwrap_or_default();
    let baud_rate = state.config.baud_rate;
    state.i2c_async_receiver = Some(rx);
    state.i2c_result = "Scanning...".into();

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

        let mut found = Vec::new();
        let mut buf = [0u8; 32];
        for scan_addr in 0x08..=0x77u8 {
            let cmd = [scan_addr << 1, 0x01];
            if port.write(&cmd).is_ok() {
                std::thread::sleep(std::time::Duration::from_millis(10));
                if let Ok(n) = port.read(&mut buf) {
                    if n > 0 { found.push(format!("0x{:02X}", scan_addr)); }
                }
            }
        }
        let _ = port.disconnect();
        let result = if found.is_empty() { "No devices found".into() } else {
            format!("Found {} device(s): {}", found.len(), found.join(", "))
        };
        let _ = tx.send(Ok(result));
    });
}

fn i2c_read(state: &mut AppState) {
    let addr: u8 = match u8::from_str_radix(&state.i2c_address.replace("0x", "").replace("0X", ""), 16) {
        Ok(v) => v, Err(_) => { state.i2c_result = "Invalid address".into(); return; }
    };
    let reg: u8 = match u8::from_str_radix(&state.i2c_register.replace("0x", "").replace("0X", ""), 16) {
        Ok(v) => v, Err(_) => { state.i2c_result = "Invalid register".into(); return; }
    };

    if state.i2c_async_receiver.is_some() {
        return;
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let port_name = state.selected_port.clone().unwrap_or_default();
    let baud_rate = state.config.baud_rate;
    state.i2c_async_receiver = Some(rx);
    state.i2c_result = "Reading...".into();

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

        let cmd = [addr << 1 | 0x01, reg];
        let result = if port.write(&cmd).is_ok() {
            std::thread::sleep(std::time::Duration::from_millis(50));
            let mut buf = [0u8; 32];
            match port.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let hex = buf[..n].iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                    Ok(format!("Read {} byte(s) from 0x{:02X}:\n{}", n, addr, hex))
                }
                _ => Ok("No response".into()),
            }
        } else {
            Err("Write failed".into())
        };
        let _ = port.disconnect();
        let _ = tx.send(result);
    });
}

fn i2c_write(state: &mut AppState) {
    if state.i2c_async_receiver.is_some() { return; }
    let addr: u8 = match u8::from_str_radix(&state.i2c_address.replace("0x", "").replace("0X", ""), 16) {
        Ok(v) => v, Err(_) => { state.i2c_result = "Invalid address".into(); return; }
    };
    let reg: u8 = match u8::from_str_radix(&state.i2c_register.replace("0x", "").replace("0X", ""), 16) {
        Ok(v) => v, Err(_) => { state.i2c_result = "Invalid register".into(); return; }
    };
    let data = match parse_hex_bytes(&state.i2c_data) {
        Some(d) => d, None => { state.i2c_result = "Invalid data hex".into(); return; }
    };
    let mut cmd = vec![addr << 1, reg];
    cmd.extend_from_slice(&data);
    let port_name = state.selected_port.clone().unwrap_or_default();
    let baud_rate = state.config.baud_rate;
    let data_len = data.len();
    let (tx, rx) = std::sync::mpsc::channel();
    state.i2c_async_receiver = Some(rx);
    std::thread::spawn(move || {
        let config = serialrun_core::config::SerialConfig { port_name, baud_rate, ..Default::default() };
        let mut port = serialrun_core::SerialPort::new(config);
        if port.connect().is_err() {
            let _ = tx.send(Err("Connect failed".into()));
            return;
        }
        match port.write(&cmd) {
            Ok(_) => { let _ = tx.send(Ok(format!("Written {} byte(s) to 0x{:02X}", data_len, addr))); }
            Err(e) => { let _ = tx.send(Err(e.to_string())); }
        }
        let _ = port.disconnect();
    });
}

fn spi_transfer(state: &mut AppState) {
    let data = match parse_hex_bytes(&state.i2c_data) {
        Some(d) => d, None => { state.i2c_result = "Invalid data hex".into(); return; }
    };

    if state.i2c_async_receiver.is_some() {
        return;
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let port_name = state.selected_port.clone().unwrap_or_default();
    let baud_rate = state.config.baud_rate;
    state.i2c_async_receiver = Some(rx);
    state.i2c_result = "Transferring...".into();

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

        let result = match port.write(&data) {
            Ok(_) => {
                std::thread::sleep(std::time::Duration::from_millis(50));
                let mut buf = [0u8; 256];
                match port.read(&mut buf) {
                    Ok(n) => {
                        let hex = buf[..n].iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                        Ok(format!("SPI: Sent {} byte(s), received {}:\n{}", data.len(), n, hex))
                    }
                    Err(_) => Ok(format!("Sent {} byte(s), no response", data.len())),
                }
            }
            Err(e) => Err(format!("SPI error: {}", e)),
        };
        let _ = port.disconnect();
        let _ = tx.send(result);
    });
}
