use crate::state::{AppState, T};
use eframe::egui;
use serialrun_core::protocol::pcap::{PcapFile, DecodedPacket};

pub fn render_pcap_viewer(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

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
        }

        ui.separator();
        ui.label(T::pcap_filter_label(lang));
        ui.add(egui::TextEdit::singleline(&mut state.pcap_filter)
            .desired_width(200.0)
            .hint_text("Modbus / CAN / AT / hex"));

        if !state.pcap_filename.is_empty() {
            ui.separator();
            ui.label(egui::RichText::new(&state.pcap_filename).color(egui::Color32::from_rgb(100, 160, 230)));
            ui.label(format!("({})", state.pcap_link_type));
        }
    });

    ui.separator();

    if state.pcap_packets.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.label(egui::RichText::new(T::pcap_no_file(lang)).size(16.0).color(egui::Color32::from_rgb(128, 128, 128)));
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
        ui.label(format!("{} {}", T::pcap_total(lang), state.pcap_packets.len()));
        ui.separator();
        ui.label(format!("{} {}", T::pcap_shown(lang), filtered_indices.len()));
    });

    ui.add_space(2.0);

    // ── Three-pane layout ──
    let available = ui.available_height();
    let packet_list_h = available * 0.45;
    let details_h = available * 0.25;
    let hex_h = available * 0.25;

    // ── Pane 1: Packet list ──
    ui.label(egui::RichText::new(T::pcap_title(lang)).strong());
    let header_color = egui::Color32::from_rgb(230, 230, 240);
    let header_text = egui::Color32::from_rgb(40, 40, 60);

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
                    ui.label(egui::RichText::new(T::pcap_col_no(lang)).color(header_text).strong().size(11.0));
                    ui.label(egui::RichText::new(T::pcap_col_time(lang)).color(header_text).strong().size(11.0));
                    ui.label(egui::RichText::new(T::pcap_col_proto(lang)).color(header_text).strong().size(11.0));
                    ui.label(egui::RichText::new(T::pcap_col_src(lang)).color(header_text).strong().size(11.0));
                    ui.label(egui::RichText::new(T::pcap_col_dst(lang)).color(header_text).strong().size(11.0));
                    ui.label(egui::RichText::new(T::pcap_col_info(lang)).color(header_text).strong().size(11.0));
                    ui.end_row();

                    for &idx in &filtered_indices {
                        let pkt = &state.pcap_packets[idx];
                        let decoded = &state.pcap_decoded[idx];
                        let selected = state.pcap_selected == Some(idx);

                        let row_bg = if selected {
                            egui::Color32::from_rgb(67, 97, 238)
                        } else {
                            let proto_color = match decoded.protocol.as_str() {
                                "Modbus RTU" | "Modbus TCP" => egui::Color32::from_rgb(0, 180, 0),
                                "CAN" => egui::Color32::from_rgb(200, 120, 0),
                                "AT" => egui::Color32::from_rgb(150, 50, 200),
                                _ => egui::Color32::from_rgb(128, 128, 128),
                            };
                            proto_color
                        };

                        let text_color = if selected {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::from_rgb(40, 40, 60)
                        };

                        let time_str = format_timestamp(pkt.timestamp_ms);
                        let no_response = ui.add(egui::Label::new(
                            egui::RichText::new(format!("{}", pkt.index + 1)).size(11.0).color(text_color)
                        ).sense(egui::Sense::click()));
                        ui.label(egui::RichText::new(&time_str).size(11.0).color(text_color));
                        ui.label(egui::RichText::new(&decoded.protocol).size(11.0).color(row_bg).strong());
                        ui.label(egui::RichText::new(&decoded.src).size(11.0).color(text_color));
                        ui.label(egui::RichText::new(&decoded.dst).size(11.0).color(text_color));
                        let info_response = ui.add(egui::Label::new(
                            egui::RichText::new(&decoded.summary).size(11.0).color(text_color)
                        ).sense(egui::Sense::click()));
                        ui.end_row();

                        if no_response.clicked() || info_response.clicked() {
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

            ui.label(egui::RichText::new(T::pcap_details(lang)).strong());
            egui::ScrollArea::vertical()
                .max_height(details_h)
                .show(ui, |ui| {
                    egui::CollapsingHeader::new(
                        egui::RichText::new(format!("{} — {}", decoded.protocol, decoded.summary)).strong()
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        for field in &decoded.details {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&field.name).strong().size(12.0));
                                ui.label(egui::RichText::new(&field.value).size(12.0).monospace());
                                if field.length > 0 {
                                    ui.label(egui::RichText::new(
                                        format!("[{}:{}]", field.offset, field.offset + field.length)
                                    ).size(10.0).color(egui::Color32::from_rgb(128, 128, 128)));
                                }
                            });
                        }
                    });
                });

            ui.separator();

            // ── Pane 3: Hex dump ──
            ui.label(egui::RichText::new(T::pcap_hex_dump(lang)).strong());
            let pkt = &state.pcap_packets[sel];
            egui::ScrollArea::vertical()
                .max_height(hex_h)
                .show(ui, |ui| {
                    render_hex_dump(ui, &pkt.data);
                });
        }
    } else {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(egui::RichText::new("← Select a packet to view details")
                .size(14.0).color(egui::Color32::from_rgb(128, 128, 128)));
        });
    }
}

fn render_hex_dump(ui: &mut egui::Ui, data: &[u8]) {
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
            ui.label(egui::RichText::new(format!("{:08X}", offset)).size(12.0).color(egui::Color32::from_rgb(100, 100, 150)).monospace());
            ui.label(egui::RichText::new(hex_str).size(12.0).monospace());
            ui.label(egui::RichText::new(format!("|{}|", ascii_str)).size(12.0).monospace().color(egui::Color32::from_rgb(0, 150, 0)));
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
