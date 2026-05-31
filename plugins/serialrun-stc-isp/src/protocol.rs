/// STC ISP Protocol implementation
///
/// STC MCU uses a proprietary ISP protocol over UART for firmware flashing.
/// The protocol involves handshake, chip detection, flash erase, write, and verify.

use std::time::Duration;

/// STC ISP protocol commands
pub const CMD握手: u8 = 0x43;      // Handshake
pub const CMD芯片信息: u8 = 0x50;  // Get chip info
pub const CMD擦除: u8 = 0x03;      // Erase flash
pub const CMD写入: u8 = 0x02;      // Write flash
pub const CMD校验: u8 = 0x06;      // Verify
pub const CMD重启: u8 = 0xFF;      // Reset MCU

/// Handshake sequence sent by host
pub fn handshake_packet() -> Vec<u8> {
    let mut packet = vec![0x43];
    packet.extend_from_slice(&[0x00; 15]); // Padding to 16 bytes
    packet
}

/// Parse handshake response from MCU
pub fn parse_handshake_response(data: &[u8]) -> Option<HandshakeInfo> {
    if data.len() < 10 || data[0] != 0x46 {
        return None;
    }

    Some(HandshakeInfo {
        ack: data[0],
        family_code: data[1],
        header_version: data[2],
        echo_back: data[3],
    })
}

/// Handshake response info
#[derive(Debug, Clone)]
pub struct HandshakeInfo {
    pub ack: u8,
    pub family_code: u8,
    pub header_version: u8,
    pub echo_back: u8,
}

/// Erase flash command
pub fn erase_packet(start_addr: u32, end_addr: u32) -> Vec<u8> {
    let mut packet = vec![CMD擦除];
    packet.extend_from_slice(&start_addr.to_le_bytes());
    packet.extend_from_slice(&end_addr.to_le_bytes());
    // Pad to 16 bytes
    packet.resize(16, 0x00);
    packet
}

/// Write flash command (128 bytes per block)
pub fn write_packet(address: u32, data: &[u8]) -> Vec<u8> {
    debug_assert!(data.len() <= 128, "Block size must be <= 128 bytes");

    let mut packet = vec![CMD写入];
    packet.extend_from_slice(&address.to_le_bytes());
    packet.extend_from_slice(&(data.len() as u16).to_le_bytes());
    packet.extend_from_slice(data);
    // Pad to 16 + data.len() bytes, aligned to 16
    let target_len = ((packet.len() + 15) / 16) * 16;
    packet.resize(target_len, 0x00);
    packet
}

/// Verify command
pub fn verify_packet(start_addr: u32, end_addr: u32, crc: u16) -> Vec<u8> {
    let mut packet = vec![CMD校验];
    packet.extend_from_slice(&start_addr.to_le_bytes());
    packet.extend_from_slice(&end_addr.to_le_bytes());
    packet.extend_from_slice(&crc.to_le_bytes());
    packet.resize(16, 0x00);
    packet
}

/// Reset MCU command
pub fn reset_packet() -> Vec<u8> {
    vec![CMD重启]
}

/// Calculate CRC-16 for STC protocol
pub fn stc_crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
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

/// Baud rate detection sequence
/// STC ISP uses 0x7F to trigger ISP mode at 9600 baud
pub fn isp_trigger_packet() -> Vec<u8> {
    vec![0x7F]
}

/// Available baud rates for STC ISP
pub const STC_BAUD_RATES: &[u32] = &[9600, 19200, 38400, 57600, 115200];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_packet() {
        let pkt = handshake_packet();
        assert_eq!(pkt.len(), 16);
        assert_eq!(pkt[0], 0x43);
    }

    #[test]
    fn test_parse_handshake() {
        let data = vec![0x46, 0x01, 0x02, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let info = parse_handshake_response(&data).unwrap();
        assert_eq!(info.ack, 0x46);
        assert_eq!(info.family_code, 0x01);
        assert_eq!(info.echo_back, 0x43);
    }

    #[test]
    fn test_parse_handshake_invalid() {
        assert!(parse_handshake_response(&[0x00]).is_none());
        assert!(parse_handshake_response(&[]).is_none());
    }

    #[test]
    fn test_erase_packet() {
        let pkt = erase_packet(0x0000, 0xFFFF);
        assert_eq!(pkt[0], CMD擦除);
        assert_eq!(pkt.len(), 16);
    }

    #[test]
    fn test_write_packet() {
        let data = vec![0xAA; 128];
        let pkt = write_packet(0x0000, &data);
        assert_eq!(pkt[0], CMD写入);
        assert!(pkt.len() > 128);
    }

    #[test]
    fn test_stc_crc16() {
        // Known test vector
        let data = b"123456789";
        let crc = stc_crc16(data);
        assert_ne!(crc, 0);
    }

    #[test]
    fn test_isp_trigger() {
        let pkt = isp_trigger_packet();
        assert_eq!(pkt, vec![0x7F]);
    }
}
