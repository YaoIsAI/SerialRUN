/// Pcap/Pcapng file parser and protocol decoder.
///
/// Imports packet capture files and decodes serial protocols
/// (Modbus RTU/TCP, CAN, AT commands) with Wireshark-style display.

use std::io::Cursor;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PcapError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid pcap file: {0}")]
    InvalidFormat(String),
}

pub type PcapResult<T> = Result<T, PcapError>;

// ── Core types ────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum LinkType {
    Ethernet,
    Serial,
    Raw,
    Unknown(u16),
}

impl LinkType {
    pub fn from_dlt(dlt: pcap_file::DataLink) -> Self {
        match dlt {
            pcap_file::DataLink::ETHERNET => LinkType::Ethernet,
            pcap_file::DataLink::NULL => LinkType::Raw,
            pcap_file::DataLink::RAW | pcap_file::DataLink::USER0 | pcap_file::DataLink::USER1 => LinkType::Serial,
            pcap_file::DataLink::LINUX_SLL => LinkType::Raw,
            _ => {
                // Store a hash of the debug name as diagnostic info
                let name = format!("{:?}", dlt);
                let hash = name.bytes().fold(0u16, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u16));
                LinkType::Unknown(hash)
            }
        }
    }
    pub fn name(&self) -> &str {
        match self {
            LinkType::Ethernet => "Ethernet",
            LinkType::Serial => "Serial",
            LinkType::Raw => "Raw",
            LinkType::Unknown(_) => "Unknown",
        }
    }
}

#[derive(Clone, Debug)]
pub struct PcapPacket {
    pub index: u32,
    pub timestamp_ms: i64,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct DecodedPacket {
    pub protocol: String,
    pub summary: String,
    pub details: Vec<FieldDetail>,
    pub src: String,
    pub dst: String,
}

#[derive(Clone, Debug)]
pub struct FieldDetail {
    pub name: String,
    pub value: String,
    pub offset: usize,
    pub length: usize,
}

pub struct PcapFile {
    pub link_type: LinkType,
    pub packets: Vec<PcapPacket>,
    pub filename: String,
}

impl PcapFile {
    /// Load a pcap or pcapng file.
    pub fn load(path: &Path) -> PcapResult<Self> {
        let filename = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let data = std::fs::read(path)?;

        // Try pcap format first
        let cursor = Cursor::new(data.clone());
        if let Ok(mut reader) = pcap_file::pcap::PcapReader::new(cursor) {
            let link_type = LinkType::from_dlt(reader.header().datalink);
            let mut packets = Vec::new();
            let mut index = 0u32;

            while let Some(result) = reader.next_packet() {
                match result {
                    Ok(pkt) => {
                        let ts_ms = pkt.timestamp.as_millis() as i64;
                        packets.push(PcapPacket {
                            index,
                            timestamp_ms: ts_ms,
                            data: pkt.data.to_vec(),
                        });
                        index += 1;
                    }
                    Err(_) => continue,
                }
            }

            // Return even if empty — the file format was valid
            return Ok(PcapFile { link_type, packets, filename });
        }

        // Try pcapng format
        let cursor = Cursor::new(data);
        if let Ok(mut reader) = pcap_file::pcapng::PcapNgReader::new(cursor) {
            let mut link_type = LinkType::Raw;
            let mut packets = Vec::new();
            let mut index = 0u32;

            while let Some(result) = reader.next_block() {
                match result {
                    Ok(block) => match block {
                        pcap_file::pcapng::Block::InterfaceDescription(idb) => {
                            link_type = LinkType::from_dlt(idb.linktype);
                        }
                        pcap_file::pcapng::Block::EnhancedPacket(epb) => {
                            let ts_ms = epb.timestamp.as_millis() as i64;
                            packets.push(PcapPacket {
                                index,
                                timestamp_ms: ts_ms,
                                data: epb.data.to_vec(),
                            });
                            index += 1;
                        }
                        pcap_file::pcapng::Block::SimplePacket(spb) => {
                            packets.push(PcapPacket {
                                index,
                                timestamp_ms: 0,
                                data: spb.data.to_vec(),
                            });
                            index += 1;
                        }
                        _ => {}
                    },
                    Err(_) => continue,
                }
            }

            // Return even if empty — the file format was valid
            return Ok(PcapFile { link_type, packets, filename });
        }

        Err(PcapError::InvalidFormat("Not a valid pcap or pcapng file".into()))
    }

    /// Decode a packet based on link type.
    pub fn decode_packet(&self, pkt: &PcapPacket) -> DecodedPacket {
        match &self.link_type {
            LinkType::Ethernet => decode_ethernet(&pkt.data),
            LinkType::Serial | LinkType::Raw => decode_serial_raw(&pkt.data),
            LinkType::Unknown(_) => decode_raw(&pkt.data),
        }
    }

    /// Decode raw serial data (for live capture without a pcap file).
    pub fn decode_packet_static(data: &[u8]) -> DecodedPacket {
        decode_serial_raw(data)
    }
}

// ── Ethernet decoder ──────────────────────────────────────────────────

fn decode_ethernet(data: &[u8]) -> DecodedPacket {
    if data.len() < 14 {
        return decode_raw(data);
    }

    let ethertype = u16::from_be_bytes([data[12], data[13]]);
    let payload = &data[14..];

    let src_mac = format_mac(&data[6..12]);
    let dst_mac = format_mac(&data[0..6]);

    match ethertype {
        0x0800 => {
            // IPv4
            if payload.len() >= 20 {
                let protocol = payload[9];
                let src_ip = format!("{}.{}.{}.{}", payload[12], payload[13], payload[14], payload[15]);
                let dst_ip = format!("{}.{}.{}.{}", payload[16], payload[17], payload[18], payload[19]);

                match protocol {
                    6 => {
                        // TCP
                        if payload.len() >= 40 {
                            let ip_header_len = ((payload[0] & 0x0F) as usize) * 4;
                            let tcp_header = &payload[ip_header_len..];
                            if tcp_header.len() >= 4 {
                                let src_port = u16::from_be_bytes([tcp_header[0], tcp_header[1]]);
                                let dst_port = u16::from_be_bytes([tcp_header[2], tcp_header[3]]);
                                let tcp_header_len = ((tcp_header[12] >> 4) as usize) * 4;
                                let tcp_payload = &tcp_header[tcp_header_len..];

                                if src_port == 502 || dst_port == 502 {
                                    if let Some(decoded) = decode_modbus_tcp(tcp_payload) {
                                        return DecodedPacket {
                                            src: format!("{}:{}", src_ip, src_port),
                                            dst: format!("{}:{}", dst_ip, dst_port),
                                            ..decoded
                                        };
                                    }
                                }

                                return DecodedPacket {
                                    protocol: "TCP".into(),
                                    summary: format!("{}:{} → {}:{} ({} bytes)", src_ip, src_port, dst_ip, dst_port, tcp_payload.len()),
                                    src: format!("{}:{}", src_ip, src_port),
                                    dst: format!("{}:{}", dst_ip, dst_port),
                                    details: vec![
                                        FieldDetail { name: "Source".into(), value: format!("{}:{}", src_ip, src_port), offset: 0, length: 0 },
                                        FieldDetail { name: "Destination".into(), value: format!("{}:{}", dst_ip, dst_port), offset: 0, length: 0 },
                                        FieldDetail { name: "Payload".into(), value: format!("{} bytes", tcp_payload.len()), offset: 0, length: 0 },
                                    ],
                                };
                            }
                        }
                    }
                    _ => {
                        return DecodedPacket {
                            protocol: "IPv4".into(),
                            summary: format!("{} → {} (proto={})", src_ip, dst_ip, protocol),
                            src: src_ip,
                            dst: dst_ip,
                            details: vec![
                                FieldDetail { name: "Protocol".into(), value: format!("{}", protocol), offset: 9, length: 1 },
                            ],
                        };
                    }
                }
            }
        }
        _ => {}
    }

    DecodedPacket {
        protocol: "Ethernet".into(),
        summary: format!("{} → {} (0x{:04X})", src_mac, dst_mac, ethertype),
        src: src_mac,
        dst: dst_mac,
        details: vec![
            FieldDetail { name: "EtherType".into(), value: format!("0x{:04X}", ethertype), offset: 12, length: 2 },
        ],
    }
}

// ── Serial/Raw decoder ────────────────────────────────────────────────

fn decode_serial_raw(data: &[u8]) -> DecodedPacket {
    if let Some(decoded) = decode_modbus_rtu(data) {
        return decoded;
    }
    if let Some(decoded) = decode_can_frame(data) {
        return decoded;
    }
    if let Some(decoded) = decode_at_command(data) {
        return decoded;
    }
    decode_raw(data)
}

// ── Modbus RTU decoder ────────────────────────────────────────────────

fn decode_modbus_rtu(data: &[u8]) -> Option<DecodedPacket> {
    if data.len() < 4 {
        return None;
    }

    let slave_id = data[0];
    let func_code = data[1];

    let expected_crc = modbus_crc16(&data[..data.len() - 2]);
    let actual_crc = u16::from_le_bytes([data[data.len() - 2], data[data.len() - 1]]);
    if expected_crc != actual_crc {
        return None;
    }

    let func_name = modbus_function_name(func_code);
    let is_exception = func_code & 0x80 != 0;

    let mut details = vec![
        FieldDetail { name: "Slave ID".into(), value: format!("{}", slave_id), offset: 0, length: 1 },
        FieldDetail { name: "Function".into(), value: format!("{} (0x{:02X})", func_name, func_code), offset: 1, length: 1 },
    ];

    if is_exception && data.len() >= 3 {
        let exc_code = data[2];
        let exc_name = modbus_exception_name(exc_code);
        details.push(FieldDetail { name: "Exception".into(), value: format!("{} ({})", exc_name, exc_code), offset: 2, length: 1 });
    } else if data.len() > 3 {
        let payload = &data[2..data.len() - 2];
        details.push(FieldDetail {
            name: "Data".into(),
            value: format_hex(payload),
            offset: 2,
            length: payload.len(),
        });
    }

    details.push(FieldDetail { name: "CRC".into(), value: format!("0x{:04X}", actual_crc), offset: data.len() - 2, length: 2 });

    let summary = if is_exception {
        format!("Slave {} Exception: {}", slave_id, modbus_exception_name(data.get(2).copied().unwrap_or(0)))
    } else {
        format!("Slave {} {}: {}", slave_id, func_name, format_hex_short(&data[2..data.len() - 2]))
    };

    Some(DecodedPacket {
        protocol: "Modbus RTU".into(),
        summary,
        src: format!("{}", slave_id),
        dst: "Master".into(),
        details,
    })
}

// ── Modbus TCP decoder ────────────────────────────────────────────────

fn decode_modbus_tcp(data: &[u8]) -> Option<DecodedPacket> {
    if data.len() < 7 {
        return None;
    }

    let transaction_id = u16::from_be_bytes([data[0], data[1]]);
    let protocol_id = u16::from_be_bytes([data[2], data[3]]);
    let length = u16::from_be_bytes([data[4], data[5]]);
    let unit_id = data[6];

    if protocol_id != 0 || (length as usize) + 6 > data.len() {
        return None;
    }

    let pdu = &data[7..6 + length as usize];
    if pdu.is_empty() {
        return None;
    }

    let func_code = pdu[0];
    let func_name = modbus_function_name(func_code);

    let mut details = vec![
        FieldDetail { name: "Transaction ID".into(), value: format!("{}", transaction_id), offset: 0, length: 2 },
        FieldDetail { name: "Protocol ID".into(), value: format!("{}", protocol_id), offset: 2, length: 2 },
        FieldDetail { name: "Length".into(), value: format!("{}", length), offset: 4, length: 2 },
        FieldDetail { name: "Unit ID".into(), value: format!("{}", unit_id), offset: 6, length: 1 },
        FieldDetail { name: "Function".into(), value: format!("{} (0x{:02X})", func_name, func_code), offset: 7, length: 1 },
    ];

    if pdu.len() > 1 {
        details.push(FieldDetail {
            name: "Data".into(),
            value: format_hex(&pdu[1..]),
            offset: 8,
            length: pdu.len() - 1,
        });
    }

    Some(DecodedPacket {
        protocol: "Modbus TCP".into(),
        summary: format!("Unit {} {} (txn={})", unit_id, func_name, transaction_id),
        src: format!("Unit {}", unit_id),
        dst: "Master".into(),
        details,
    })
}

// ── CAN decoder ───────────────────────────────────────────────────────

fn decode_can_frame(data: &[u8]) -> Option<DecodedPacket> {
    if data.len() < 3 {
        return None;
    }

    let id = ((data[0] as u32) << 3) | ((data[1] as u32) >> 5);
    let dlc = data[1] & 0x0F;
    if id <= 0x7FF && dlc <= 8 && data.len() >= 2 + dlc as usize {
        let can_data = &data[2..2 + dlc as usize];
        let mut details = vec![
            FieldDetail { name: "ID".into(), value: format!("0x{:03X}", id), offset: 0, length: 2 },
            FieldDetail { name: "DLC".into(), value: format!("{}", dlc), offset: 1, length: 1 },
        ];
        if !can_data.is_empty() {
            details.push(FieldDetail {
                name: "Data".into(),
                value: format_hex(can_data),
                offset: 2,
                length: can_data.len(),
            });
        }
        return Some(DecodedPacket {
            protocol: "CAN".into(),
            summary: format!("ID=0x{:03X} [{}] {}", id, dlc, format_hex_short(can_data)),
            src: format!("0x{:03X}", id),
            dst: "Bus".into(),
            details,
        });
    }
    None
}

// ── AT command decoder ────────────────────────────────────────────────

fn decode_at_command(data: &[u8]) -> Option<DecodedPacket> {
    if data.len() < 2 {
        return None;
    }

    let text = String::from_utf8_lossy(data);
    let trimmed = text.trim();

    if trimmed.starts_with("AT") || trimmed.starts_with("at") {
        return Some(DecodedPacket {
            protocol: "AT".into(),
            summary: format!("AT: {}", trimmed),
            src: "Host".into(),
            dst: "Device".into(),
            details: vec![
                FieldDetail { name: "Command".into(), value: trimmed.to_string(), offset: 0, length: data.len() },
            ],
        });
    }

    if trimmed == "OK" || trimmed.starts_with("ERROR") || trimmed.starts_with("+") {
        return Some(DecodedPacket {
            protocol: "AT".into(),
            summary: format!("AT Response: {}", trimmed),
            src: "Device".into(),
            dst: "Host".into(),
            details: vec![
                FieldDetail { name: "Response".into(), value: trimmed.to_string(), offset: 0, length: data.len() },
            ],
        });
    }

    None
}

// ── Raw hex fallback ──────────────────────────────────────────────────

fn decode_raw(data: &[u8]) -> DecodedPacket {
    DecodedPacket {
        protocol: "Raw".into(),
        summary: format!("{} bytes: {}", data.len(), format_hex_short(data)),
        src: "—".into(),
        dst: "—".into(),
        details: vec![
            FieldDetail { name: "Length".into(), value: format!("{}", data.len()), offset: 0, length: 0 },
            FieldDetail { name: "Data".into(), value: format_hex(data), offset: 0, length: data.len() },
        ],
    }
}

// ── Modbus helpers ────────────────────────────────────────────────────

fn modbus_crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

fn modbus_function_name(code: u8) -> &'static str {
    match code {
        0x01 => "Read Coils",
        0x02 => "Read Discrete Inputs",
        0x03 => "Read Holding Registers",
        0x04 => "Read Input Registers",
        0x05 => "Write Single Coil",
        0x06 => "Write Single Register",
        0x07 => "Read Exception Status",
        0x08 => "Diagnostics",
        0x0B => "Get Comm Event Counter",
        0x0C => "Get Comm Event Log",
        0x0F => "Write Multiple Coils",
        0x10 => "Write Multiple Registers",
        0x11 => "Report Server ID",
        0x16 => "Mask Write Register",
        0x17 => "Read/Write Multiple Registers",
        0x2B => "Read Device Identification",
        _ if code & 0x80 != 0 => "Exception",
        _ => "Unknown",
    }
}

fn modbus_exception_name(code: u8) -> &'static str {
    match code {
        0x01 => "Illegal Function",
        0x02 => "Illegal Data Address",
        0x03 => "Illegal Data Value",
        0x04 => "Slave Device Failure",
        0x05 => "Acknowledge",
        0x06 => "Slave Device Busy",
        0x08 => "Memory Parity Error",
        0x0A => "Gateway Path Unavailable",
        0x0B => "Gateway Target Failed",
        _ => "Unknown",
    }
}

// ── Formatting helpers ────────────────────────────────────────────────

fn format_mac(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(":")
}

fn format_hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
}

fn format_hex_short(data: &[u8]) -> String {
    if data.len() <= 8 {
        format_hex(data)
    } else {
        format!("{}...", format_hex(&data[..8]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modbus_crc16() {
        let data = [0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];
        let crc = modbus_crc16(&data);
        assert_eq!(crc, 0xC5CD);
    }

    #[test]
    fn test_decode_modbus_rtu() {
        let data = [0x01, 0x03, 0x00, 0x00, 0x00, 0x0A, 0xC5, 0xCD];
        let decoded = decode_modbus_rtu(&data);
        assert!(decoded.is_some());
        let d = decoded.unwrap();
        assert_eq!(d.protocol, "Modbus RTU");
        assert!(d.summary.contains("Read Holding Registers"));
    }

    #[test]
    fn test_decode_modbus_rtu_invalid_crc() {
        let data = [0x01, 0x03, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00];
        let decoded = decode_modbus_rtu(&data);
        assert!(decoded.is_none());
    }
}
