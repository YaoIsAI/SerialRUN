/// STC ISP Protocol implementation
///
/// Based on STC ISP bootloader protocol specification.
/// Reference: stcgal (https://github.com/grigorig/stcgal)
///
/// Packet format:
///   [0x46] [0xB9] [LEN_H] [LEN_L] [CMD] [PAYLOAD...] [CHECKSUM]
///   Header  Magic   Length (2 bytes)   Command  Data        Sum of all preceding bytes
///
/// Baud rate detection: send 0x7F repeatedly, MCU responds after power cycle.

use std::time::Duration;

// ============================================================================
// Protocol Constants
// ============================================================================

/// Packet header magic bytes
pub const HEADER_MAGIC: [u8; 2] = [0x46, 0xB9];

/// ISP trigger byte (sent repeatedly to trigger ISP mode)
pub const ISP_TRIGGER: u8 = 0x7F;

/// Command codes
pub const CMD_INFO: u8 = 0x50;        // Get chip info
pub const CMD_ERASE: u8 = 0x03;       // Erase flash
pub const CMD_WRITE: u8 = 0x02;       // Write flash
pub const CMD_VERIFY: u8 = 0x06;      // Verify flash
pub const CMD_RESET: u8 = 0xFF;       // Reset MCU
pub const CMD_SET_BAUD: u8 = 0x0F;    // Set baud rate

/// Baud rates to try during detection
pub const DETECT_BAUD_RATES: &[u32] = &[2400, 4800, 9600, 19200, 38400, 57600, 115200];

// ============================================================================
// Packet Building
// ============================================================================

/// Build a packet with proper framing
fn build_packet(cmd: u8, payload: &[u8]) -> Vec<u8> {
    let len = (payload.len() + 1) as u16; // +1 for command byte
    let mut packet = Vec::with_capacity(5 + payload.len() + 1);
    packet.push(HEADER_MAGIC[0]);  // 0x46
    packet.push(HEADER_MAGIC[1]);  // 0xB9
    packet.push((len >> 8) as u8);  // Length high
    packet.push((len & 0xFF) as u8); // Length low
    packet.push(cmd);
    packet.extend_from_slice(payload);

    // Calculate checksum: two's complement of sum of all preceding bytes
    let sum: u8 = packet.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    let checksum = sum.wrapping_neg();
    packet.push(checksum);

    packet
}

/// ISP trigger packet (0x7F sent repeatedly)
pub fn isp_trigger_packet() -> Vec<u8> {
    vec![ISP_TRIGGER]
}

/// Erase flash command
pub fn erase_packet(start_addr: u32, end_addr: u32) -> Vec<u8> {
    let mut payload = Vec::with_capacity(8);
    payload.extend_from_slice(&start_addr.to_le_bytes());
    payload.extend_from_slice(&end_addr.to_le_bytes());
    build_packet(CMD_ERASE, &payload)
}

/// Write flash command (128 bytes per block)
pub fn write_packet(address: u32, data: &[u8]) -> Vec<u8> {
    debug_assert!(data.len() <= 128, "Block size must be <= 128 bytes");

    let mut payload = Vec::with_capacity(2 + 2 + data.len());
    payload.extend_from_slice(&address.to_le_bytes());
    payload.extend_from_slice(&(data.len() as u16).to_le_bytes());
    payload.extend_from_slice(data);
    build_packet(CMD_WRITE, &payload)
}

/// Verify command
pub fn verify_packet(start_addr: u32, end_addr: u32, crc: u16) -> Vec<u8> {
    let mut payload = Vec::with_capacity(6);
    payload.extend_from_slice(&start_addr.to_le_bytes());
    payload.extend_from_slice(&end_addr.to_le_bytes());
    payload.extend_from_slice(&crc.to_le_bytes());
    build_packet(CMD_VERIFY, &payload)
}

/// Reset MCU command
pub fn reset_packet() -> Vec<u8> {
    build_packet(CMD_RESET, &[])
}

/// Get chip info command
pub fn info_packet() -> Vec<u8> {
    build_packet(CMD_INFO, &[])
}

// ============================================================================
// Packet Parsing
// ============================================================================

/// Parsed packet from MCU
#[derive(Debug, Clone)]
pub struct McuPacket {
    pub cmd: u8,
    pub payload: Vec<u8>,
    pub valid: bool,
}

/// Parse a packet from MCU response
pub fn parse_packet(data: &[u8]) -> Option<McuPacket> {
    // Minimum packet: header(2) + len(2) + cmd(1) + checksum(1) = 6 bytes
    if data.len() < 6 {
        return None;
    }

    // Check header magic
    if data[0] != HEADER_MAGIC[0] || data[1] != HEADER_MAGIC[1] {
        return None;
    }

    let len = ((data[2] as u16) << 8) | (data[3] as u16) as u16;
    let cmd = data[4];

    // Verify checksum
    let checksum_sum: u8 = data[..data.len() - 1].iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    let expected_checksum = data[data.len() - 1];
    let valid = checksum_sum.wrapping_add(expected_checksum) == 0;

    // Extract payload (between cmd and checksum)
    let payload_end = std::cmp::min(4 + len as usize, data.len() - 1);
    let payload = if payload_end > 5 {
        data[5..payload_end].to_vec()
    } else {
        Vec::new()
    };

    Some(McuPacket { cmd, payload, valid })
}

/// Parse handshake response from MCU
///
/// The MCU responds to 0x7F trigger with a packet containing chip info.
/// After power-cycling the MCU, it sends a handshake response.
pub fn parse_handshake_response(data: &[u8]) -> Option<HandshakeInfo> {
    // Try to parse as a framed packet first
    if let Some(pkt) = parse_packet(data) {
        if pkt.cmd == CMD_INFO && pkt.valid && pkt.payload.len() >= 8 {
            return Some(HandshakeInfo {
                family_code: pkt.payload[0],
                header_version: pkt.payload[1],
                mcu_id: ((pkt.payload[2] as u16) << 8) | (pkt.payload[3] as u16),
                flash_size_kb: ((pkt.payload[4] as u16) << 8) | (pkt.payload[5] as u16),
                eeprom_size_kb: ((pkt.payload[6] as u16) << 8) | (pkt.payload[7] as u16),
            });
        }
    }

    // Fallback: parse raw response (some MCU versions don't use framing)
    if data.len() >= 10 && data[0] == 0x46 {
        return Some(HandshakeInfo {
            family_code: data[1],
            header_version: data[2],
            mcu_id: ((data[4] as u16) << 8) | (data[5] as u16),
            flash_size_kb: ((data[6] as u16) << 8) | (data[7] as u16),
            eeprom_size_kb: ((data[8] as u16) << 8) | (data[9] as u16),
        });
    }

    None
}

/// Handshake response info
#[derive(Debug, Clone)]
pub struct HandshakeInfo {
    pub family_code: u8,
    pub header_version: u8,
    pub mcu_id: u16,
    pub flash_size_kb: u16,
    pub eeprom_size_kb: u16,
}

/// Check if response indicates success (ACK)
pub fn is_ack(data: &[u8]) -> bool {
    if let Some(pkt) = parse_packet(data) {
        pkt.valid && pkt.cmd == 0x46
    } else {
        !data.is_empty() && data[0] == 0x46
    }
}

// ============================================================================
// CRC-16
// ============================================================================

/// Calculate CRC-16 for STC protocol
/// Uses CRC-16/CCITT with initial value 0xFFFF
pub fn stc_crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF; // Standard CCITT initial value
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_packet() {
        let pkt = build_packet(CMD_ERASE, &[0x00, 0x00, 0xFF, 0xFF]);
        eprintln!("Packet ({} bytes): {}", pkt.len(), pkt.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "));
        // Manually verify checksum
        let mut manual_sum: u8 = 0;
        for i in 0..pkt.len()-1 {
            manual_sum = manual_sum.wrapping_add(pkt[i]);
            eprintln!("  byte[{}] = {:02X}, running sum = {:02X}", i, pkt[i], manual_sum);
        }
        eprintln!("  checksum byte = {:02X}", pkt[pkt.len()-1]);
        eprintln!("  total sum = {:02X}", manual_sum.wrapping_add(pkt[pkt.len()-1]));
        assert_eq!(pkt[0], 0x46);
        assert_eq!(pkt[1], 0xB9);
        assert_eq!(pkt[4], CMD_ERASE);
        let sum: u8 = pkt.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_parse_packet() {
        let pkt = build_packet(CMD_INFO, &[0x25, 0x00, 0x12, 0x34, 0x00, 0x40, 0x00, 0x20]);
        let parsed = parse_packet(&pkt).unwrap();
        assert!(parsed.valid);
        assert_eq!(parsed.cmd, CMD_INFO);
        assert_eq!(parsed.payload.len(), 8);
    }

    #[test]
    fn test_parse_handshake_framed() {
        let pkt = build_packet(CMD_INFO, &[0x25, 0x00, 0x12, 0x34, 0x00, 0x40, 0x00, 0x20]);
        let info = parse_handshake_response(&pkt).unwrap();
        assert_eq!(info.family_code, 0x25);
        assert_eq!(info.mcu_id, 0x1234);
        assert_eq!(info.flash_size_kb, 64);
        assert_eq!(info.eeprom_size_kb, 32);
    }

    #[test]
    fn test_parse_handshake_raw() {
        let data = vec![0x46, 0x25, 0x00, 0x43, 0x12, 0x34, 0x00, 0x40, 0x00, 0x20];
        let info = parse_handshake_response(&data).unwrap();
        assert_eq!(info.family_code, 0x25);
        assert_eq!(info.mcu_id, 0x1234);
        assert_eq!(info.flash_size_kb, 64);
    }

    #[test]
    fn test_is_ack() {
        let pkt = build_packet(0x46, &[]);
        assert!(is_ack(&pkt));

        let bad = vec![0x00, 0x00];
        assert!(!is_ack(&bad));
    }

    #[test]
    fn test_erase_packet() {
        let pkt = erase_packet(0x0000, 0xFFFF);
        assert_eq!(pkt[0], 0x46);
        assert_eq!(pkt[1], 0xB9);
        assert_eq!(pkt[4], CMD_ERASE);
    }

    #[test]
    fn test_write_packet() {
        let data = vec![0xAA; 128];
        let pkt = write_packet(0x0000, &data);
        assert_eq!(pkt[4], CMD_WRITE);
        assert!(pkt.len() > 128);
    }

    #[test]
    fn test_stc_crc16() {
        // Known test vector for CRC-16/CCITT
        let data = b"123456789";
        let crc = stc_crc16(data);
        assert_eq!(crc, 0x29B1);
    }

    #[test]
    fn test_isp_trigger() {
        let pkt = isp_trigger_packet();
        assert_eq!(pkt, vec![0x7F]);
    }
}
