use eframe::egui;
use egui::{Color32, RichText, Rounding, Vec2};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq)]
enum Theme { Dark, Light }

#[derive(Clone, Copy)]
struct C {
    bg: Color32, card: Color32, hover: Color32, text: Color32, dim: Color32,
    muted: Color32, accent: Color32, ok: Color32, err: Color32, warn: Color32, input: Color32,
}
impl C {
    fn dark() -> Self {
        Self {
            bg: Color32::from_rgb(18, 18, 18), card: Color32::from_rgb(28, 28, 30),
            hover: Color32::from_rgb(38, 38, 40), text: Color32::from_rgb(255, 255, 255),
            dim: Color32::from_rgb(142, 142, 147), muted: Color32::from_rgb(99, 99, 102),
            accent: Color32::from_rgb(0, 122, 255), ok: Color32::from_rgb(52, 199, 89),
            err: Color32::from_rgb(255, 59, 48), warn: Color32::from_rgb(255, 149, 0),
            input: Color32::from_rgb(44, 44, 46),
        }
    }
    fn light() -> Self {
        Self {
            bg: Color32::from_rgb(242, 242, 247), card: Color32::from_rgb(255, 255, 255),
            hover: Color32::from_rgb(230, 230, 235), text: Color32::from_rgb(0, 0, 0),
            dim: Color32::from_rgb(100, 100, 100), muted: Color32::from_rgb(140, 140, 140),
            accent: Color32::from_rgb(0, 122, 255), ok: Color32::from_rgb(52, 199, 89),
            err: Color32::from_rgb(255, 59, 48), warn: Color32::from_rgb(255, 149, 0),
            input: Color32::from_rgb(230, 230, 235),
        }
    }
}

pub struct MobileApp {
    theme: Theme,
    connected: bool,
    port_name: String,
    baud_rate: u32,
    data_bits: u8,
    stop_bits: u8,
    parity: String,
    flow_control: String,
    timeout_ms: u64,
    ports: Vec<String>,
    port_drop: bool,
    baud_drop: bool,
    settings_open: bool,
    shared_port: Arc<Mutex<Option<serialrun_core::SerialPort>>>,
    lines: Vec<Line>,
    input: String,
    hex_mode: bool,
    auto_scroll: bool,
    show_timestamp: bool,
    mcp_on: bool,
    mcp_port: u16,
    mcp_log: Arc<Mutex<Vec<String>>>,
    modbus_on: bool,
    mb_func: u8,
    mb_slave: u8,
    mb_addr: String,
    mb_val: String,
    mb_qty: u16,
    status: String,
}

#[derive(Clone)]
struct Line {
    dir: String,
    text: String,
    time: String,
}

impl Default for MobileApp {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            connected: false,
            port_name: String::new(),
            baud_rate: 115200,
            data_bits: 8,
            stop_bits: 1,
            parity: "None".into(),
            flow_control: "None".into(),
            timeout_ms: 1000,
            ports: Vec::new(),
            port_drop: false,
            baud_drop: false,
            settings_open: false,
            shared_port: Arc::new(Mutex::new(None)),
            lines: Vec::new(),
            input: String::new(),
            hex_mode: false,
            auto_scroll: true,
            show_timestamp: true,
            mcp_on: false,
            mcp_port: 9527,
            mcp_log: Arc::new(Mutex::new(Vec::new())),
            modbus_on: false,
            mb_func: 3,
            mb_slave: 1,
            mb_addr: "0".into(),
            mb_val: String::new(),
            mb_qty: 1,
            status: "Ready".into(),
        }
    }
}

impl MobileApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        if let Ok(d) = std::fs::read("C:\\Windows\\Fonts\\msyh.ttc") {
            fonts.font_data.insert("msyh".into(), egui::FontData::from_owned(d));
            if let Some(f) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                f.insert(0, "msyh".into());
            }
            if let Some(f) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                f.insert(0, "msyh".into());
            }
        }
        cc.egui_ctx.set_fonts(fonts);
        let mut app = Self::default();
        let mcp = Arc::new(Mutex::new(crate::mcp::McpState::new(
            app.shared_port.clone(),
            app.mcp_log.clone(),
        )));
        crate::mcp::start_mcp_server(mcp, app.mcp_port);
        app.mcp_on = true;
        app.status = format!("MCP :{}", app.mcp_port);
        app
    }

    fn c(&self) -> C {
        match self.theme {
            Theme::Dark => C::dark(),
            Theme::Light => C::light(),
        }
    }

    fn push(&mut self, dir: &str, text: &str) {
        let t = if self.show_timestamp {
            chrono::Local::now().format("%H:%M:%S").to_string()
        } else {
            String::new()
        };
        self.lines.push(Line {
            dir: dir.into(),
            text: text.into(),
            time: t,
        });
        if self.lines.len() > 1000 {
            self.lines.remove(0);
        }
    }

    fn push_raw(&mut self, dir: &str, data: &[u8]) {
        let display = if self.hex_mode {
            data.iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            match std::str::from_utf8(data) {
                Ok(s) => s.to_string(),
                Err(_) => data
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" "),
            }
        };
        self.push(dir, &display);
    }

    fn scan(&mut self) {
        self.status = "Scanning...".into();
        match serialrun_core::SerialPort::list_ports() {
            Ok(p) => {
                self.ports = p
                    .iter()
                    .map(|x| {
                        let d = x.description.as_deref().unwrap_or("");
                        if d.is_empty() {
                            x.name.clone()
                        } else {
                            format!("{} - {}", x.name, d)
                        }
                    })
                    .collect();
                self.status = format!("{} port(s)", self.ports.len());
            }
            Err(e) => {
                self.status = format!("Error: {}", e);
            }
        }
    }

    fn connect(&mut self) {
        if self.port_name.is_empty() {
            return;
        }
        let pn = self
            .port_name
            .split(" - ")
            .next()
            .unwrap_or(&self.port_name)
            .to_string();
        let parity = match self.parity.as_str() {
            "Odd" => serialrun_core::config::Parity::Odd,
            "Even" => serialrun_core::config::Parity::Even,
            _ => serialrun_core::config::Parity::None,
        };
        let db = match self.data_bits {
            5 => serialrun_core::config::DataBits::Five,
            6 => serialrun_core::config::DataBits::Six,
            7 => serialrun_core::config::DataBits::Seven,
            _ => serialrun_core::config::DataBits::Eight,
        };
        let sb = match self.stop_bits {
            2 => serialrun_core::config::StopBits::Two,
            _ => serialrun_core::config::StopBits::One,
        };
        let fc = match self.flow_control.as_str() {
            "Hardware" => serialrun_core::config::FlowControl::Hardware,
            "Software" => serialrun_core::config::FlowControl::Software,
            _ => serialrun_core::config::FlowControl::None,
        };
        let cfg = serialrun_core::config::SerialConfig {
            port_name: pn,
            baud_rate: self.baud_rate,
            data_bits: db,
            stop_bits: sb,
            parity,
            flow_control: fc,
            timeout_ms: self.timeout_ms,
        };
        let mut port = serialrun_core::SerialPort::new(cfg);
        match port.connect() {
            Ok(()) => {
                *self.shared_port.lock().unwrap() = Some(port);
                self.connected = true;
                self.status = format!("Connected @ {}", self.baud_rate);
                self.push("SYS", &format!("Connected @ {} baud", self.baud_rate));
            }
            Err(e) => {
                self.status = format!("Error: {}", e);
                self.push("SYS", &format!("Error: {}", e));
            }
        }
    }

    fn disconnect(&mut self) {
        if let Ok(mut g) = self.shared_port.lock() {
            if let Some(mut p) = g.take() {
                let _ = p.disconnect();
            }
        }
        self.connected = false;
        self.status = "Disconnected".into();
        self.push("SYS", "Disconnected");
    }

    fn send(&mut self) {
        if self.input.is_empty() {
            return;
        }
        let data = self.input.clone();
        self.input.clear();
        let result = if let Ok(mut g) = self.shared_port.lock() {
            if let Some(ref mut p) = *g {
                p.write(data.as_bytes())
                    .map(|n| Ok(n))
                    .unwrap_or_else(|e| Err(e.to_string()))
            } else {
                Err("Not connected".into())
            }
        } else {
            Err("Lock error".into())
        };
        match result {
            Ok(n) => {
                self.push("TX", &data);
                self.status = format!("Sent {} bytes", n);
            }
            Err(e) => {
                self.push("SYS", &e);
            }
        }
    }

    fn read_port(&mut self) {
        if let Ok(g) = self.shared_port.try_lock() {
            self.connected = g.is_some();
        }
        if !self.connected {
            return;
        }
        let mut rx = Vec::new();
        if let Ok(mut g) = self.shared_port.try_lock() {
            if let Some(ref mut p) = *g {
                if let Ok(n) = p.bytes_to_read() {
                    if n > 0 {
                        let mut buf = [0u8; 1024];
                        if let Ok(r) = p.read(&mut buf[..(n as usize).min(1024)]) {
                            if r > 0 {
                                rx = buf[..r].to_vec();
                            }
                        }
                    }
                }
            }
        }
        if !rx.is_empty() {
            self.push_raw("RX", &rx);
        }
    }

    fn mb_read(&mut self) {
        if !self.connected {
            self.push("SYS", "Not connected");
            return;
        }
        let addr: u16 = self.mb_addr.parse().unwrap_or(0);
        let slave = self.mb_slave;
        let qty = self.mb_qty;
        use serialrun_core::protocol::{ModbusFrame, ModbusFunction, ModbusParser};
        let frame =
            ModbusParser::build_read_request(slave, ModbusFunction::ReadHoldingRegisters, addr, qty);
        let req = frame.to_bytes();
        let result = if let Ok(mut g) = self.shared_port.lock() {
            if let Some(ref mut p) = *g {
                if let Err(e) = p.write(&req) {
                    Err(e.to_string())
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    let mut buf = [0u8; 256];
                    match p.read(&mut buf) {
                        Ok(n) if n >= 4 => match ModbusFrame::parse(&buf[..n]) {
                            Ok(f) => {
                                let mut vals = Vec::new();
                                let data = &f.data;
                                let mut i = 1;
                                while i + 1 < data.len() {
                                    vals.push(u16::from_be_bytes([data[i], data[i + 1]]));
                                    i += 2;
                                }
                                Ok(format!(
                                    "Read {} registers from slave {}: {:?}",
                                    qty, slave, vals
                                ))
                            }
                            Err(e) => Err(format!("Parse: {}", e)),
                        },
                        _ => Err("No response".into()),
                    }
                }
            } else {
                Err("Not connected".into())
            }
        } else {
            Err("Lock error".into())
        };
        match result {
            Ok(msg) => self.push("SYS", &msg),
            Err(e) => self.push("SYS", &format!("Error: {}", e)),
        }
    }

    fn mb_write(&mut self) {
        if !self.connected {
            self.push("SYS", "Not connected");
            return;
        }
        let addr: u16 = self.mb_addr.parse().unwrap_or(0);
        let val: u16 = self.mb_val.parse().unwrap_or(0);
        let slave = self.mb_slave;
        use serialrun_core::protocol::ModbusParser;
        let frame = ModbusParser::build_write_single(slave, addr, val);
        let req = frame.to_bytes();
        let result = if let Ok(mut g) = self.shared_port.lock() {
            if let Some(ref mut p) = *g {
                if let Err(e) = p.write(&req) {
                    Err(e.to_string())
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    let mut buf = [0u8; 256];
                    match p.read(&mut buf) {
                        Ok(_) => Ok(format!(
                            "Wrote {} to register 0x{:04X} (slave {})",
                            val, addr, slave
                        )),
                        Err(e) => Err(e.to_string()),
                    }
                }
            } else {
                Err("Not connected".into())
            }
        } else {
            Err("Lock error".into())
        };
        match result {
            Ok(msg) => self.push("SYS", &msg),
            Err(e) => self.push("SYS", &format!("Error: {}", e)),
        }
    }
}

impl eframe::App for MobileApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let c = self.c();
        self.read_port();

        ctx.style_mut(|s| {
            s.visuals = match self.theme {
                Theme::Dark => egui::Visuals::dark(),
                Theme::Light => egui::Visuals::light(),
            };
            s.visuals.window_fill = c.bg;
            s.visuals.panel_fill = c.bg;
            s.visuals.widgets.noninteractive.bg_fill = c.input;
            s.visuals.widgets.inactive.bg_fill = c.input;
            s.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, c.text);
            s.visuals.widgets.hovered.bg_fill = c.hover;
            s.visuals.widgets.active.bg_fill = c.accent;
        });

        // Process MCP log
        let msgs: Vec<String> = self.mcp_log.lock().unwrap().drain(..).collect();
        for m in msgs {
            if m.starts_with("TX:") {
                self.push("TX", &m[3..]);
            } else if m.starts_with("RX:") {
                self.push("RX", &m[3..]);
            } else {
                self.push("SYS", &m);
            }
        }

        // ── Top Bar ──
        egui::TopBottomPanel::top("top")
            .frame(
                egui::Frame::none()
                    .fill(c.card)
                    .inner_margin(egui::Margin::symmetric(12.0, 8.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(rect.center(), 5.0, if self.connected { c.ok } else { c.err });
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("SerialRUN")
                            .size(15.0)
                            .strong()
                            .color(c.text),
                    );
                    let (theme_label, theme_tip) = match self.theme {
                        Theme::Dark => ("\u{1F319}", "Light"),
                        Theme::Light => ("\u{2600}\u{FE0F}", "Dark"),
                    };
                    if ui
                        .add(
                            egui::Button::new(RichText::new(theme_label).size(14.0))
                                .fill(Color32::TRANSPARENT)
                                .frame(false),
                        )
                        .on_hover_text(theme_tip)
                        .clicked()
                    {
                        self.theme = match self.theme {
                            Theme::Dark => Theme::Light,
                            Theme::Light => Theme::Dark,
                        };
                    }
                    ui.add_space(4.0);
                    if self.connected {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Disconnect").color(c.err).size(11.0),
                                )
                                .fill(Color32::TRANSPARENT),
                            )
                            .clicked()
                        {
                            self.disconnect();
                        }
                    } else if ui
                        .add(
                            egui::Button::new(
                                RichText::new("Connect")
                                    .color(Color32::WHITE)
                                    .strong()
                                    .size(11.0),
                            )
                            .fill(c.accent)
                            .rounding(Rounding::same(6.0)),
                        )
                        .clicked()
                    {
                        self.connect();
                    }
                });
            });

        // ── Main Content ──
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(c.bg)
                    .inner_margin(egui::Margin::same(8.0)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - 80.0)
                    .show(ui, |ui| {
                        // Connection Card
                        card(ui, c, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Connection").size(11.0).color(c.dim));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new("Scan")
                                                        .color(c.accent)
                                                        .size(10.0),
                                                )
                                                .fill(Color32::TRANSPARENT),
                                            )
                                            .clicked()
                                        {
                                            self.scan();
                                        }
                                    },
                                );
                            });
                            ui.add_space(4.0);

                            // Port + Baud on same row using fixed columns
                            ui.allocate_ui(Vec2::new(ui.available_width(), 60.0), |ui| {
                                ui.columns(2, |cols| {
                                    // Port column
                                    cols[0].vertical(|ui| {
                                        ui.label(RichText::new("Port").size(9.0).color(c.muted));
                                        let plbl = if self.port_name.is_empty() {
                                            "Select...".into()
                                        } else {
                                            self.port_name.clone()
                                        };
                                        let pbtn = ui.add(
                                            egui::Button::new(
                                                RichText::new(&plbl)
                                                    .color(if self.port_name.is_empty() {
                                                        c.muted
                                                    } else {
                                                        c.text
                                                    })
                                                    .size(10.0),
                                            )
                                            .fill(c.input)
                                            .rounding(Rounding::same(6.0))
                                            .min_size(Vec2::new(ui.available_width(), 28.0)),
                                        );
                                        if pbtn.clicked() {
                                            self.port_drop = !self.port_drop;
                                            self.baud_drop = false;
                                        }
                                        if self.port_drop && !self.ports.is_empty() {
                                            egui::Area::new(egui::Id::new("pd"))
                                                .fixed_pos(
                                                    pbtn.rect.left_bottom() + Vec2::new(0.0, 4.0),
                                                )
                                                .order(egui::Order::Foreground)
                                                .show(ui.ctx(), |ui| {
                                                    egui::Frame::none()
                                                        .fill(c.hover)
                                                        .rounding(Rounding::same(6.0))
                                                        .inner_margin(egui::Margin::same(4.0))
                                                        .show(ui, |ui| {
                                                            ui.set_min_width(180.0);
                                                            for p in &self.ports {
                                                                let sel = self.port_name == *p;
                                                                if ui.add(
                                                                    egui::Button::new(
                                                                        RichText::new(p)
                                                                            .size(10.0)
                                                                            .color(if sel {
                                                                                c.accent
                                                                            } else {
                                                                                c.text
                                                                            }),
                                                                    )
                                                                    .fill(if sel {
                                                                        c.accent.linear_multiply(0.2)
                                                                    } else {
                                                                        Color32::TRANSPARENT
                                                                    })
                                                                    .rounding(Rounding::same(4.0))
                                                                    .min_size(Vec2::new(
                                                                        ui.available_width(),
                                                                        24.0,
                                                                    )),
                                                                ).clicked() {
                                                                    self.port_name = p.clone();
                                                                    self.port_drop = false;
                                                                }
                                                            }
                                                        });
                                                });
                                        } else if !self.port_drop {
                                            // keep state in sync
                                        }
                                    });
                                    // Baud column
                                    cols[1].vertical(|ui| {
                                        ui.label(RichText::new("Baud").size(9.0).color(c.muted));
                                        let bbtn = ui.add(
                                            egui::Button::new(
                                                RichText::new(format!("{}", self.baud_rate))
                                                    .size(10.0)
                                                    .color(c.text),
                                            )
                                            .fill(c.input)
                                            .rounding(Rounding::same(6.0))
                                            .min_size(Vec2::new(ui.available_width(), 28.0)),
                                        );
                                        if bbtn.clicked() {
                                            self.baud_drop = !self.baud_drop;
                                            self.port_drop = false;
                                        }
                                        if self.baud_drop {
                                            egui::Area::new(egui::Id::new("bd"))
                                                .fixed_pos(
                                                    bbtn.rect.left_bottom() + Vec2::new(0.0, 4.0),
                                                )
                                                .order(egui::Order::Foreground)
                                                .show(ui.ctx(), |ui| {
                                                    egui::Frame::none()
                                                        .fill(c.hover)
                                                        .rounding(Rounding::same(6.0))
                                                        .inner_margin(egui::Margin::same(4.0))
                                                        .show(ui, |ui| {
                                                            ui.set_min_width(100.0);
                                                            for &r in &[
                                                                9600, 19200, 38400, 57600, 115200,
                                                                230400, 460800, 921600,
                                                            ] {
                                                                let sel = self.baud_rate == r;
                                                                if ui.add(
                                                                    egui::Button::new(
                                                                        RichText::new(format!(
                                                                                "{}", r
                                                                            ))
                                                                            .size(10.0)
                                                                            .color(if sel {
                                                                                c.accent
                                                                            } else {
                                                                                c.text
                                                                            }),
                                                                    )
                                                                    .fill(if sel {
                                                                        c.accent.linear_multiply(0.2)
                                                                    } else {
                                                                        Color32::TRANSPARENT
                                                                    })
                                                                    .rounding(Rounding::same(4.0))
                                                                    .min_size(Vec2::new(
                                                                        ui.available_width(),
                                                                        20.0,
                                                                    )),
                                                                ).clicked() {
                                                                    self.baud_rate = r;
                                                                    self.baud_drop = false;
                                                                }
                                                            }
                                                        });
                                                });
                                        }
                                    });
                                });
                            });

                            // Settings
                            ui.add_space(4.0);
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(if self.settings_open {
                                            "\u{25BC} Settings"
                                        } else {
                                            "\u{25B6} Settings"
                                        })
                                        .size(10.0)
                                        .color(c.dim),
                                    )
                                    .fill(Color32::TRANSPARENT),
                                )
                                .clicked()
                            {
                                self.settings_open = !self.settings_open;
                            }
                            if self.settings_open {
                                ui.add_space(4.0);
                                ui.separator();
                                ui.add_space(4.0);
                                row(ui, "Data Bits", c, |ui| {
                                    for &d in &[5, 6, 7, 8] {
                                        pill(ui, &format!("{}", d), self.data_bits == d, c, || {
                                            self.data_bits = d
                                        });
                                    }
                                });
                                row(ui, "Stop Bits", c, |ui| {
                                    for &s in &[1, 2] {
                                        pill(ui, &format!("{}", s), self.stop_bits == s, c, || {
                                            self.stop_bits = s
                                        });
                                    }
                                });
                                row(ui, "Parity", c, |ui| {
                                    for p in &["None", "Odd", "Even"] {
                                        pill(ui, p, self.parity == *p, c, || {
                                            self.parity = p.to_string()
                                        });
                                    }
                                });
                                row(ui, "Flow", c, |ui| {
                                    for f in &["None", "Hardware", "Software"] {
                                        pill(
                                            ui,
                                            f,
                                            self.flow_control == *f,
                                            c,
                                            || self.flow_control = f.to_string(),
                                        );
                                    }
                                });
                                row(ui, "Timeout", c, |ui| {
                                    ui.add(
                                        egui::DragValue::new(&mut self.timeout_ms)
                                            .speed(100)
                                            .range(10..=10000)
                                            .suffix(" ms"),
                                    );
                                });
                                ui.add_space(4.0);
                                ui.separator();
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new("Terminal Settings")
                                        .size(10.0)
                                        .color(c.dim),
                                );
                                ui.checkbox(
                                    &mut self.show_timestamp,
                                    RichText::new("Show Timestamps").size(10.0).color(c.text),
                                );
                                ui.checkbox(
                                    &mut self.auto_scroll,
                                    RichText::new("Auto Scroll").size(10.0).color(c.text),
                                );
                                ui.checkbox(
                                    &mut self.hex_mode,
                                    RichText::new("HEX Mode").size(10.0).color(c.text),
                                );
                            }
                        });
                        ui.add_space(4.0);

                        // Terminal
                        card(ui, c, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Terminal")
                                        .size(11.0)
                                        .color(c.dim),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new("Clear")
                                                        .color(c.err)
                                                        .size(9.0),
                                                )
                                                .fill(Color32::TRANSPARENT),
                                            )
                                            .clicked()
                                        {
                                            self.lines.clear();
                                        }
                                    },
                                );
                            });
                            ui.add_space(4.0);
                            let h = (ui.available_height() * 0.4).max(150.0);
                            egui::ScrollArea::vertical()
                                .max_height(h)
                                .stick_to_bottom(self.auto_scroll)
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    if self.lines.is_empty() {
                                        ui.vertical_centered(|ui| {
                                            ui.add_space(20.0);
                                            ui.label(
                                                RichText::new("No data")
                                                    .color(c.muted)
                                                    .size(12.0),
                                            );
                                        });
                                    } else {
                                        for l in &self.lines {
                                            let (dc, dbg) = match l.dir.as_str() {
                                                "TX" => (c.ok, c.ok.linear_multiply(0.1)),
                                                "RX" => (
                                                    c.accent,
                                                    c.accent.linear_multiply(0.1),
                                                ),
                                                _ => (c.warn, c.warn.linear_multiply(0.1)),
                                            };
                                            ui.horizontal_wrapped(|ui| {
                                                egui::Frame::none()
                                                    .fill(dbg)
                                                    .rounding(Rounding::same(3.0))
                                                    .inner_margin(egui::Margin::symmetric(
                                                        4.0, 1.0,
                                                    ))
                                                    .show(ui, |ui| {
                                                        ui.label(
                                                            RichText::new(&l.dir)
                                                                .size(8.0)
                                                                .color(dc)
                                                                .strong(),
                                                        );
                                                    });
                                                if self.show_timestamp {
                                                    ui.add_space(3.0);
                                                    ui.label(
                                                        RichText::new(&l.time)
                                                            .size(8.0)
                                                            .color(c.muted),
                                                    );
                                                }
                                                ui.add_space(3.0);
                                                ui.label(
                                                    RichText::new(&l.text)
                                                        .size(10.0)
                                                        .color(c.text),
                                                );
                                            });
                                        }
                                    }
                                });
                        });
                        ui.add_space(4.0);

                        // MCP
                        card(ui, c, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("MCP Server")
                                        .size(11.0)
                                        .color(c.dim),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let bc = if self.mcp_on { c.err } else { c.ok };
                                        let bt = if self.mcp_on { "Stop" } else { "Start" };
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new(bt)
                                                        .color(Color32::WHITE)
                                                        .size(10.0)
                                                        .strong(),
                                                )
                                                .fill(bc)
                                                .rounding(Rounding::same(6.0))
                                                .min_size(Vec2::new(50.0, 24.0)),
                                            )
                                            .clicked()
                                        {
                                            if self.mcp_on {
                                                self.mcp_on = false;
                                                self.push("SYS", "MCP stopped");
                                            } else {
                                                let s = Arc::new(Mutex::new(
                                                    crate::mcp::McpState::new(
                                                        self.shared_port.clone(),
                                                        self.mcp_log.clone(),
                                                    ),
                                                ));
                                                crate::mcp::start_mcp_server(s, self.mcp_port);
                                                self.mcp_on = true;
                                                self.push(
                                                    "SYS",
                                                    &format!("MCP :{}", self.mcp_port),
                                                );
                                            }
                                        }
                                    },
                                );
                            });
                            if self.mcp_on {
                                ui.label(
                                    RichText::new(format!(
                                        "127.0.0.1:{}",
                                        self.mcp_port
                                    ))
                                    .size(9.0)
                                    .color(c.accent),
                                );
                            }
                        });
                        ui.add_space(4.0);

                        // Modbus
                        card(ui, c, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Modbus")
                                        .size(11.0)
                                        .color(c.dim),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui
                                            .add(
                                                egui::Button::new(RichText::new(
                                                    if self.modbus_on {
                                                        "\u{25BC}"
                                                    } else {
                                                        "\u{25B6}"
                                                    },
                                                )
                                                .size(10.0)
                                                .color(c.dim))
                                                .fill(Color32::TRANSPARENT),
                                            )
                                            .clicked()
                                        {
                                            self.modbus_on = !self.modbus_on;
                                        }
                                    },
                                );
                            });
                            if self.modbus_on {
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("Slave").size(9.0).color(c.muted),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut self.mb_slave)
                                            .range(1..=247),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("Func").size(9.0).color(c.muted),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut self.mb_func)
                                            .range(1..=4),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("Addr").size(9.0).color(c.muted),
                                    );
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.mb_addr)
                                            .desired_width(80.0),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("Qty").size(9.0).color(c.muted),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut self.mb_qty)
                                            .range(1..=125),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("Val").size(9.0).color(c.muted),
                                    );
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.mb_val)
                                            .desired_width(80.0),
                                    );
                                });
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new("Read")
                                                    .color(Color32::WHITE)
                                                    .size(10.0),
                                            )
                                            .fill(c.accent)
                                            .rounding(Rounding::same(6.0))
                                            .min_size(Vec2::new(60.0, 24.0)),
                                        )
                                        .clicked()
                                    {
                                        self.mb_read();
                                    }
                                    ui.add_space(4.0);
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new("Write")
                                                    .color(Color32::WHITE)
                                                    .size(10.0),
                                            )
                                            .fill(c.ok)
                                            .rounding(Rounding::same(6.0))
                                            .min_size(Vec2::new(60.0, 24.0)),
                                        )
                                        .clicked()
                                    {
                                        self.mb_write();
                                    }
                                });
                            }
                        });
                        ui.add_space(4.0);

                        // Status
                        egui::Frame::none()
                            .fill(c.card)
                            .rounding(Rounding::same(6.0))
                            .inner_margin(egui::Margin::symmetric(8.0, 5.0))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(&self.status)
                                        .size(9.0)
                                        .color(c.muted),
                                );
                            });
                    }); // end ScrollArea

                    // Input bar
                    ui.add_space(4.0);
                    egui::Frame::none()
                        .fill(c.card)
                        .rounding(Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(6.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let (tf, xf) = if !self.hex_mode {
                                    (c.accent, c.input)
                                } else {
                                    (c.input, c.accent)
                                };
                                let (tc, xc) = if !self.hex_mode {
                                    (Color32::WHITE, c.dim)
                                } else {
                                    (c.dim, Color32::WHITE)
                                };
                                ui.add(
                                    egui::Button::new(
                                        RichText::new("TXT").size(10.0).color(tc),
                                    )
                                    .fill(tf)
                                    .rounding(Rounding::same(10.0))
                                    .min_size(Vec2::new(32.0, 22.0)),
                                );
                                ui.add(
                                    egui::Button::new(
                                        RichText::new("HEX").size(10.0).color(xc),
                                    )
                                    .fill(xf)
                                    .rounding(Rounding::same(10.0))
                                    .min_size(Vec2::new(32.0, 22.0)),
                                );
                                ui.add_space(4.0);
                                // Allocate exact width for input field
                                let send_w = 54.0_f32;
                                let input_w = ui.available_width() - send_w - 4.0;
                                let _r = ui.allocate_ui(Vec2::new(input_w, 30.0), |ui| {
                                    egui::Frame::none()
                                        .fill(c.input)
                                        .rounding(Rounding::same(6.0))
                                        .inner_margin(egui::Margin::same(0.0))
                                        .show(ui, |ui| {
                                            let r = ui.add(
                                                egui::TextEdit::singleline(&mut self.input)
                                                    .hint_text(
                                                        RichText::new("Enter data...")
                                                            .color(c.muted),
                                                    )
                                                    .desired_width(ui.available_width())
                                                    .text_color(c.text)
                                                    .margin(egui::Margin::symmetric(8.0, 6.0)),
                                            );
                                            if r.lost_focus()
                                                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                            {
                                                self.send();
                                            }
                                        });
                                });
                                ui.add_space(4.0);
                                let sb = ui.add_enabled(
                                    self.connected,
                                    egui::Button::new(
                                        RichText::new("Send")
                                            .size(11.0)
                                            .color(Color32::WHITE)
                                            .strong(),
                                    )
                                    .fill(if self.connected {
                                        c.accent
                                    } else {
                                        c.input
                                    })
                                    .rounding(Rounding::same(6.0))
                                    .min_size(Vec2::new(send_w, 30.0)),
                                );
                                if sb.clicked() {
                                    self.send();
                                }
                            });
                        });
                }); // end CentralPanel
            } // end update
} // end impl eframe::App

fn card(ui: &mut egui::Ui, c: C, f: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none()
        .fill(c.card)
        .rounding(Rounding::same(10.0))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, f);
}

fn row(ui: &mut egui::Ui, label: &str, c: C, f: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(9.0).color(c.muted));
        f(ui);
    });
}

fn pill(
    ui: &mut egui::Ui,
    label: &str,
    active: bool,
    c: C,
    on_click: impl FnOnce(),
) {
    let (bg, tc) = if active {
        (c.accent, Color32::WHITE)
    } else {
        (c.input, c.dim)
    };
    if ui
        .add(
            egui::Button::new(RichText::new(label).size(9.0).color(tc))
                .fill(bg)
                .rounding(Rounding::same(10.0))
                .min_size(Vec2::new(32.0, 20.0)),
        )
        .clicked()
    {
        on_click();
    }
}
