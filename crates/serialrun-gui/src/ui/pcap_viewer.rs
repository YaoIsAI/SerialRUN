use crate::state::{AppState, T, Theme};
use eframe::egui;
use serialrun_core::protocol::pcap::PcapFile;

pub fn render_pcap_viewer(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;
    let is_dark = state.theme == Theme::Dark;

    // ── Help window ──
    if state.show_pcap_help {
        let mut open = state.show_pcap_help;
        egui::Window::new(T::pcap_help_title(lang))
            .open(&mut open)
            .resizable(true)
            .default_width(480.0)
            .default_height(400.0)
            .show(ui.ctx(), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if lang == crate::state::Language::Chinese {
                        ui.heading("抓包分析器使用说明");
                        ui.add_space(8.0);
                        ui.label("抓包分析器可以导入 pcap/pcapng 文件进行协议分析，也可以实时捕获串口数据。");
                        ui.add_space(8.0);

                        ui.strong("两种模式：");
                        ui.label("1. 文件导入：点击「打开文件」选择 .pcap 或 .pcapng 文件");
                        ui.label("2. 实时抓包：先连接串口，再点击「⏺ 开始抓包」");
                        ui.add_space(8.0);

                        ui.strong("支持的协议（自动识别）：");
                        ui.label("• Modbus RTU — 串口原始 Modbus 帧，自动校验 CRC");
                        ui.label("• Modbus TCP — 以太网封装的 Modbus 协议（端口 502）");
                        ui.label("• CAN 总线 — 标准帧（11位 ID）自动解码");
                        ui.label("• AT 指令 — 识别 AT 命令和响应");
                        ui.label("• Raw — 无法识别的数据显示为原始十六进制");
                        ui.add_space(8.0);

                        ui.strong("三栏视图：");
                        ui.label("• 上方：数据包列表（序号、时间、协议、源、目标、摘要）");
                        ui.label("• 中间：选中包的协议详情（可展开字段）");
                        ui.label("• 下方：十六进制转储（偏移量 + HEX + ASCII）");
                        ui.add_space(8.0);

                        ui.strong("过滤：");
                        ui.label("在过滤框输入关键字，可按协议名（Modbus、CAN、AT）或内容过滤。");
                        ui.add_space(8.0);

                        ui.strong("颜色说明：");
                        ui.horizontal(|ui| {
                            ui.label("•");
                            ui.label(egui::RichText::new("绿色").color(egui::Color32::from_rgb(46, 204, 113)));
                            ui.label(" = Modbus RTU/TCP");
                        });
                        ui.horizontal(|ui| {
                            ui.label("•");
                            ui.label(egui::RichText::new("橙色").color(egui::Color32::from_rgb(230, 160, 50)));
                            ui.label(" = CAN 总线");
                        });
                        ui.horizontal(|ui| {
                            ui.label("•");
                            ui.label(egui::RichText::new("紫色").color(egui::Color32::from_rgb(180, 100, 230)));
                            ui.label(" = AT 指令");
                        });
                        ui.horizontal(|ui| {
                            ui.label("•");
                            ui.label(egui::RichText::new("灰色").color(egui::Color32::from_rgb(128, 128, 140)));
                            ui.label(" = 原始数据");
                        });
                    } else {
                        heading_en(ui);
                    }
                });
            });
        state.show_pcap_help = open;
    }

    // Theme-aware colors
    let text_color = if is_dark { egui::Color32::from_rgb(220, 220, 230) } else { egui::Color32::from_rgb(30, 30, 50) };
    let muted_color = if is_dark { egui::Color32::from_rgb(128, 128, 140) } else { egui::Color32::from_rgb(120, 120, 130) };
    let header_bg = if is_dark { egui::Color32::from_rgb(40, 42, 54) } else { egui::Color32::from_rgb(235, 237, 242) };
    let selected_bg = egui::Color32::from_rgb(67, 97, 238);
    let selected_text = egui::Color32::WHITE;

    // ── Toolbar ──
    ui.horizontal(|ui| {
        if ui.button(T::pcap_open(lang)).clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Pcap files", &["pcap", "pcapng", "cap"])
                .pick_file()
            {
                match PcapFile::load(&path) {
                    Ok(pcap) => {
                        state.pcap_decoded = pcap.packets.iter().map(|p| pcap.decode_packet(p)).collect();
                        state.pcap_link_type = pcap.link_type.name().to_string();
                        state.pcap_packets = pcap.packets;
                        state.pcap_filename = pcap.filename;
                        state.pcap_selected = None;
                        state.pcap_filter.clear();
                    }
                    Err(e) => {
                        state.show_error(&format!("Pcap: {}", e));
                    }
                }
            }
        }

        if ui.button(T::pcap_clear(lang)).clicked() {
            state.pcap_packets.clear();
            state.pcap_decoded.clear();
            state.pcap_selected = None;
            state.pcap_filename.clear();
            state.pcap_filter.clear();
            state.pcap_capturing = false;
        }

        // Live capture toggle
        ui.separator();
        if state.pcap_capturing {
            let btn = ui.button(egui::RichText::new("⏹ Stop Capture").color(egui::Color32::from_rgb(255, 80, 80)));
            if btn.clicked() {
                state.pcap_capturing = false;
            }
        } else {
            let can_capture = state.is_connected;
            let btn = ui.add_enabled(can_capture, egui::Button::new(
                egui::RichText::new("⏺ Start Capture").color(egui::Color32::from_rgb(80, 200, 80))
            ));
            if btn.clicked() {
                state.pcap_capturing = true;
                state.pcap_filename = "Live Capture".into();
                state.pcap_link_type = "Serial".into();
            }
            if !can_capture {
                btn.on_hover_text("Connect to a serial port first");
            }
        }

        ui.separator();
        ui.label(T::pcap_filter_label(lang));
        ui.add(egui::TextEdit::singleline(&mut state.pcap_filter)
            .desired_width(200.0)
            .hint_text("Modbus / CAN / AT / hex"));

        if !state.pcap_filename.is_empty() {
            ui.separator();
            ui.label(egui::RichText::new(&state.pcap_filename).color(egui::Color32::from_rgb(100, 160, 230)));
            ui.label(egui::RichText::new(format!("({})", state.pcap_link_type)).color(muted_color));
        }

        // Help button (right-aligned)
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(egui::RichText::new("?").size(14.0).strong())
                .on_hover_text(T::pcap_help_hint(lang))
                .clicked()
            {
                state.show_pcap_help = !state.show_pcap_help;
            }
        });
    });

    ui.separator();

    if state.pcap_packets.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.label(egui::RichText::new(T::pcap_no_file(lang)).size(16.0).color(muted_color));
        });
        return;
    }

    // ── Filter packets ──
    let filter = state.pcap_filter.to_lowercase();
    let filtered_indices: Vec<usize> = state.pcap_decoded.iter().enumerate()
        .filter(|(_, d)| {
            if filter.is_empty() { return true; }
            d.protocol.to_lowercase().contains(&filter)
                || d.summary.to_lowercase().contains(&filter)
                || d.src.to_lowercase().contains(&filter)
                || d.dst.to_lowercase().contains(&filter)
        })
        .map(|(i, _)| i)
        .collect();

    // ── Status bar ──
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("{} {}", T::pcap_total(lang), state.pcap_packets.len())).color(muted_color));
        ui.separator();
        ui.label(egui::RichText::new(format!("{} {}", T::pcap_shown(lang), filtered_indices.len())).color(muted_color));
    });

    ui.add_space(2.0);

    // ── Three-pane layout ──
    let available = ui.available_height();
    let packet_list_h = available * 0.45;
    let details_h = available * 0.25;
    let hex_h = available * 0.25;

    // ── Pane 1: Packet list ──
    ui.label(egui::RichText::new(T::pcap_title(lang)).strong().color(text_color));

    egui::ScrollArea::vertical()
        .max_height(packet_list_h)
        .stick_to_bottom(false)
        .show(ui, |ui| {
            egui::Grid::new("pcap_packet_list")
                .striped(true)
                .num_columns(6)
                .min_col_width(40.0)
                .show(ui, |ui| {
                    // Header row
                    for h in [T::pcap_col_no(lang), T::pcap_col_time(lang), T::pcap_col_proto(lang),
                              T::pcap_col_src(lang), T::pcap_col_dst(lang), T::pcap_col_info(lang)] {
                        ui.label(egui::RichText::new(h).strong().size(11.0).color(text_color));
                    }
                    ui.end_row();

                    for &idx in &filtered_indices {
                        let pkt = &state.pcap_packets[idx];
                        let decoded = &state.pcap_decoded[idx];
                        let selected = state.pcap_selected == Some(idx);

                        let proto_color = match decoded.protocol.as_str() {
                            "Modbus RTU" | "Modbus TCP" => egui::Color32::from_rgb(46, 204, 113),
                            "CAN" => egui::Color32::from_rgb(230, 160, 50),
                            "AT" => egui::Color32::from_rgb(180, 100, 230),
                            _ => muted_color,
                        };

                        // Selected row: paint background, use high-contrast text
                        let row_text = if selected {
                            if is_dark { egui::Color32::WHITE } else { egui::Color32::from_rgb(20, 20, 40) }
                        } else {
                            text_color
                        };
                        let row_proto = if selected {
                            if is_dark { egui::Color32::from_rgb(150, 255, 180) } else { egui::Color32::from_rgb(0, 130, 60) }
                        } else {
                            proto_color
                        };

                        let time_str = format_timestamp(pkt.timestamp_ms);

                        let no_resp = ui.add(egui::Label::new(
                            egui::RichText::new(format!("{}", pkt.index + 1)).size(11.0).color(row_text)
                        ).sense(egui::Sense::click()));
                        // Paint selected row background
                        if selected {
                            let rect = no_resp.rect;
                            let row_rect = egui::Rect::from_min_max(
                                egui::pos2(ui.min_rect().left(), rect.min.y),
                                egui::pos2(ui.min_rect().right(), rect.max.y),
                            );
                            let sel_color = if is_dark {
                                egui::Color32::from_rgba_premultiplied(67, 97, 238, 80)
                            } else {
                                egui::Color32::from_rgba_premultiplied(67, 97, 238, 40)
                            };
                            ui.painter().rect_filled(row_rect, 0.0, sel_color);
                        }
                        ui.label(egui::RichText::new(&time_str).size(11.0).color(row_text));
                        ui.label(egui::RichText::new(&decoded.protocol).size(11.0).color(row_proto).strong());
                        ui.label(egui::RichText::new(&decoded.src).size(11.0).color(row_text));
                        ui.label(egui::RichText::new(&decoded.dst).size(11.0).color(row_text));
                        let info_resp = ui.add(egui::Label::new(
                            egui::RichText::new(&decoded.summary).size(11.0).color(row_text)
                        ).sense(egui::Sense::click()));
                        ui.end_row();

                        if no_resp.clicked() || info_resp.clicked() {
                            state.pcap_selected = Some(idx);
                        }
                    }
                });
        });

    ui.separator();

    // ── Pane 2: Packet details ──
    if let Some(sel) = state.pcap_selected {
        if sel < state.pcap_decoded.len() {
            let decoded = &state.pcap_decoded[sel];

            ui.label(egui::RichText::new(T::pcap_details(lang)).strong().color(text_color));
            egui::ScrollArea::vertical()
                .max_height(details_h)
                .show(ui, |ui| {
                    egui::CollapsingHeader::new(
                        egui::RichText::new(format!("{} — {}", decoded.protocol, decoded.summary)).strong().color(text_color)
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        for field in &decoded.details {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&field.name).strong().size(12.0).color(text_color));
                                ui.label(egui::RichText::new(&field.value).size(12.0).monospace().color(egui::Color32::from_rgb(100, 180, 255)));
                                if field.length > 0 {
                                    ui.label(egui::RichText::new(
                                        format!("[{}:{}]", field.offset, field.offset + field.length)
                                    ).size(10.0).color(muted_color));
                                }
                            });
                        }
                    });
                });

            ui.separator();

            // ── Pane 3: Hex dump ──
            ui.label(egui::RichText::new(T::pcap_hex_dump(lang)).strong().color(text_color));
            let pkt = &state.pcap_packets[sel];
            egui::ScrollArea::vertical()
                .max_height(hex_h)
                .show(ui, |ui| {
                    render_hex_dump(ui, &pkt.data, is_dark);
                });
        }
    } else {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(egui::RichText::new("← Select a packet to view details")
                .size(14.0).color(muted_color));
        });
    }
}

fn render_hex_dump(ui: &mut egui::Ui, data: &[u8], is_dark: bool) {
    let offset_color = if is_dark { egui::Color32::from_rgb(100, 120, 180) } else { egui::Color32::from_rgb(80, 100, 160) };
    let hex_color = if is_dark { egui::Color32::from_rgb(200, 200, 220) } else { egui::Color32::from_rgb(30, 30, 50) };
    let ascii_color = if is_dark { egui::Color32::from_rgb(80, 200, 120) } else { egui::Color32::from_rgb(0, 140, 60) };

    for (row_idx, chunk) in data.chunks(16).enumerate() {
        let offset = row_idx * 16;

        let mut hex_str = String::new();
        let mut ascii_str = String::new();

        for (i, &byte) in chunk.iter().enumerate() {
            if i > 0 { hex_str.push(' '); }
            hex_str.push_str(&format!("{:02X}", byte));
            ascii_str.push(if byte.is_ascii_graphic() || byte == b' ' {
                byte as char
            } else {
                '.'
            });
        }

        if chunk.len() < 16 {
            for _ in chunk.len()..16 {
                hex_str.push_str("   ");
            }
        }

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("{:08X}", offset)).size(12.0).monospace().color(offset_color));
            ui.label(egui::RichText::new(hex_str).size(12.0).monospace().color(hex_color));
            ui.label(egui::RichText::new(format!("|{}|", ascii_str)).size(12.0).monospace().color(ascii_color));
        });
    }
}

fn format_timestamp(ms: i64) -> String {
    if ms == 0 {
        return "00:00.000".into();
    }
    let total_secs = ms / 1000;
    let millis = ms % 1000;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}.{:03}", mins, secs, millis)
}

fn heading_en(ui: &mut egui::Ui) {
    ui.heading("Packet Capture Viewer Help");
    ui.add_space(8.0);
    ui.label("The Packet Capture Viewer can import pcap/pcapng files for protocol analysis, or capture live serial data in real-time.");
    ui.add_space(8.0);

    ui.strong("Two Modes:");
    ui.label("1. File Import: Click 'Open File' to load a .pcap or .pcapng file");
    ui.label("2. Live Capture: Connect to a serial port first, then click '⏺ Start Capture'");
    ui.add_space(8.0);

    ui.strong("Supported Protocols (auto-detected):");
    ui.label("• Modbus RTU — Raw serial Modbus frames with CRC validation");
    ui.label("• Modbus TCP — Ethernet-encapsulated Modbus protocol (port 502)");
    ui.label("• CAN Bus — Standard frames (11-bit ID) auto-decoded");
    ui.label("• AT Commands — Recognizes AT commands and responses");
    ui.label("• Raw — Unrecognized data shown as hex dump");
    ui.add_space(8.0);

    ui.strong("Three-Pane View:");
    ui.label("• Top: Packet list (No., Time, Protocol, Source, Destination, Summary)");
    ui.label("• Middle: Selected packet's protocol details (expandable fields)");
    ui.label("• Bottom: Hex dump (offset + HEX + ASCII sidebar)");
    ui.add_space(8.0);

    ui.strong("Filter:");
    ui.label("Enter keywords in the filter box to filter by protocol name (Modbus, CAN, AT) or content.");
    ui.add_space(8.0);

    ui.strong("Color Legend:");
    ui.horizontal(|ui| {
        ui.label("•");
        ui.label(egui::RichText::new("Green").color(egui::Color32::from_rgb(46, 204, 113)));
        ui.label(" = Modbus RTU/TCP");
    });
    ui.horizontal(|ui| {
        ui.label("•");
        ui.label(egui::RichText::new("Orange").color(egui::Color32::from_rgb(230, 160, 50)));
        ui.label(" = CAN Bus");
    });
    ui.horizontal(|ui| {
        ui.label("•");
        ui.label(egui::RichText::new("Purple").color(egui::Color32::from_rgb(180, 100, 230)));
        ui.label(" = AT Commands");
    });
    ui.horizontal(|ui| {
        ui.label("•");
        ui.label(egui::RichText::new("Gray").color(egui::Color32::from_rgb(128, 128, 140)));
        ui.label(" = Raw Data");
    });
}
