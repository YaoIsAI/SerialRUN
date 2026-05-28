/// CRC and checksum utility functions for serial protocols.

/// CRC-16/MODBUS (poly 0xA001, reflected, init 0xFFFF)
pub fn crc16_modbus(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 0x0001 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

/// CRC-16/CCITT (poly 0x1021, init 0x0000)
pub fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0x0000;
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

/// CRC-16/XMODEM (poly 0x8408, reflected, init 0x0000)
pub fn crc16_xmodem(data: &[u8]) -> u16 {
    let mut crc: u16 = 0x0000;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 0x0001 != 0 {
                crc = (crc >> 1) ^ 0x8408;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

/// CRC-32 (poly 0xEDB88320 reflected, init 0xFFFFFFFF, final XOR 0xFFFFFFFF)
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xFFFFFFFF
}

/// Longitudinal Redundancy Check - byte sum wrapping, then negate
pub fn lrc(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b)).wrapping_neg()
}

/// Simple 8-bit checksum (byte sum)
pub fn checksum8(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
}

/// Simple 16-bit checksum (byte sum)
pub fn checksum16(data: &[u8]) -> u16 {
    data.iter().fold(0u16, |acc, &b| acc.wrapping_add(b as u16))
}

/// Results of running all checksum algorithms on a data slice.
#[derive(Debug, Clone)]
pub struct ChecksumResults {
    pub crc16_modbus: u16,
    pub crc16_ccitt: u16,
    pub crc16_xmodem: u16,
    pub crc32: u32,
    pub lrc: u8,
    pub checksum8: u8,
    pub checksum16: u16,
}

impl ChecksumResults {
    /// Calculate all checksums for the given data.
    pub fn calculate_all(data: &[u8]) -> Self {
        Self {
            crc16_modbus: crc16_modbus(data),
            crc16_ccitt: crc16_ccitt(data),
            crc16_xmodem: crc16_xmodem(data),
            crc32: crc32(data),
            lrc: lrc(data),
            checksum8: checksum8(data),
            checksum16: checksum16(data),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc16_modbus_empty() {
        assert_eq!(crc16_modbus(&[]), 0xFFFF);
    }

    #[test]
    fn test_crc16_modbus_known() {
        // "123456789" is a standard test vector
        let data = b"123456789";
        let crc = crc16_modbus(data);
        assert_eq!(crc, 0x4B37);
    }

    #[test]
    fn test_crc16_modbus_consistency() {
        let data = [0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];
        let crc1 = crc16_modbus(&data);
        let crc2 = crc16_modbus(&data);
        assert_eq!(crc1, crc2);
        assert_ne!(crc1, 0);
    }

    #[test]
    fn test_crc16_ccitt_empty() {
        assert_eq!(crc16_ccitt(&[]), 0x0000);
    }

    #[test]
    fn test_crc16_ccitt_known() {
        let data = b"123456789";
        let crc = crc16_ccitt(data);
        // CRC-16/CCITT with init=0x0000, MSB-first, poly=0x1021
        assert_eq!(crc, 0x31C3);
    }

    #[test]
    fn test_crc16_xmodem_empty() {
        assert_eq!(crc16_xmodem(&[]), 0x0000);
    }

    #[test]
    fn test_crc16_xmodem_known() {
        // CRC-16/XMODEM with reflected poly 0x8408, init 0x0000
        let data = b"123456789";
        let crc = crc16_xmodem(data);
        assert_eq!(crc, 0x2189);
    }

    #[test]
    fn test_crc32_empty() {
        assert_eq!(crc32(&[]), 0x00000000);
    }

    #[test]
    fn test_crc32_known() {
        let data = b"123456789";
        let result = crc32(data);
        assert_eq!(result, 0xCBF43926);
    }

    #[test]
    fn test_lrc_empty() {
        assert_eq!(lrc(&[]), 0);
    }

    #[test]
    fn test_lrc_single_byte() {
        assert_eq!(lrc(&[0x42]), 0xBE); // ~0x42 = 0xBE
    }

    #[test]
    fn test_lrc_multiple() {
        let data = [0x01, 0x02, 0x03];
        // sum = 6, neg = 0xFA (wrapping)
        assert_eq!(lrc(&data), 0xFA);
    }

    #[test]
    fn test_checksum8() {
        assert_eq!(checksum8(&[]), 0);
        assert_eq!(checksum8(&[1, 2, 3]), 6);
        assert_eq!(checksum8(&[0xFF, 0x01]), 0x00); // wrapping
    }

    #[test]
    fn test_checksum16() {
        assert_eq!(checksum16(&[]), 0);
        assert_eq!(checksum16(&[1, 2, 3]), 6);
        assert_eq!(checksum16(&[0xFF, 0x01]), 0x0100);
    }

    #[test]
    fn test_calculate_all() {
        let data = b"test data";
        let results = ChecksumResults::calculate_all(data);

        assert_eq!(results.crc16_modbus, crc16_modbus(data));
        assert_eq!(results.crc16_ccitt, crc16_ccitt(data));
        assert_eq!(results.crc16_xmodem, crc16_xmodem(data));
        assert_eq!(results.crc32, crc32(data));
        assert_eq!(results.lrc, lrc(data));
        assert_eq!(results.checksum8, checksum8(data));
        assert_eq!(results.checksum16, checksum16(data));
    }

    #[test]
    fn test_different_algos_different_results() {
        let data = b"hello world";
        let results = ChecksumResults::calculate_all(data);
        // Not all must differ, but at least some should
        assert_ne!(results.crc16_modbus, results.crc16_ccitt);
        assert_ne!(results.crc16_modbus, results.crc16_xmodem);
    }
}
