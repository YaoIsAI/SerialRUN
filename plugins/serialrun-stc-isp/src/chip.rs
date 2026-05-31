/// STC chip identification and info

/// Known STC chip families
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StcFamily {
    STC89,
    STC12,
    STC15,
    STC8,
    STC8G,
    STC8H,
    Unknown(u8),
}

impl StcFamily {
    pub fn from_family_code(code: u8) -> Self {
        match code {
            0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07 | 0x08 | 0x09 | 0x0A => StcFamily::STC89,
            0x0B | 0x0C | 0x0D | 0x0E | 0x0F | 0x10 | 0x11 => StcFamily::STC12,
            0x12 | 0x13 | 0x14 | 0x15 | 0x16 | 0x17 | 0x18 | 0x19 | 0x1A => StcFamily::STC15,
            0x21 | 0x22 | 0x23 | 0x24 | 0x25 | 0x26 | 0x27 | 0x28 | 0x29 | 0x2A | 0x2B | 0x2C | 0x2D | 0x2E | 0x2F | 0x30 => StcFamily::STC8,
            0x41 | 0x42 | 0x43 | 0x44 | 0x45 | 0x46 | 0x47 | 0x48 | 0x49 | 0x4A => StcFamily::STC8G,
            0x51 | 0x52 | 0x53 | 0x54 | 0x55 | 0x56 | 0x57 | 0x58 | 0x59 | 0x5A => StcFamily::STC8H,
            _ => StcFamily::Unknown(code),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            StcFamily::STC89 => "STC89",
            StcFamily::STC12 => "STC12",
            StcFamily::STC15 => "STC15",
            StcFamily::STC8 => "STC8",
            StcFamily::STC8G => "STC8G",
            StcFamily::STC8H => "STC8H",
            StcFamily::Unknown(_) => "Unknown",
        }
    }
}

impl std::fmt::Display for StcFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Chip information from ISP handshake
#[derive(Debug, Clone)]
pub struct ChipInfo {
    pub family: StcFamily,
    pub family_code: u8,
    pub flash_size: u32,       // bytes
    pub eeprom_size: u32,      // bytes
    pub info_message: String,
}

impl ChipInfo {
    pub fn from_handshake(family_code: u8, _header_version: u8) -> Self {
        let family = StcFamily::from_family_code(family_code);

        // Flash sizes based on family (typical defaults)
        let (flash_size, eeprom_size) = match family {
            StcFamily::STC89 => (64 * 1024, 8 * 1024),      // 64KB Flash, 8KB EEPROM
            StcFamily::STC12 => (8 * 1024, 4 * 1024),       // 8KB Flash, 4KB EEPROM
            StcFamily::STC15 => (16 * 1024, 8 * 1024),      // 16KB Flash, 8KB EEPROM
            StcFamily::STC8 => (32 * 1024, 8 * 1024),       // 32KB Flash, 8KB EEPROM
            StcFamily::STC8G => (64 * 1024, 32 * 1024),     // 64KB Flash, 32KB EEPROM
            StcFamily::STC8H => (64 * 1024, 32 * 1024),     // 64KB Flash, 32KB EEPROM
            StcFamily::Unknown(_) => (32 * 1024, 8 * 1024), // Default
        };

        let info_message = format!(
            "{} (family code: 0x{:02X}), Flash: {}KB, EEPROM: {}KB",
            family.name(),
            family_code,
            flash_size / 1024,
            eeprom_size / 1024
        );

        Self {
            family,
            family_code,
            flash_size,
            eeprom_size,
            info_message,
        }
    }
}

/// Parse HEX file format (Intel HEX)
pub fn parse_hex_file(content: &str) -> Result<Vec<u8>, String> {
    let mut binary = Vec::new();
    let mut base_addr: u32 = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with(':') {
            continue;
        }

        let bytes = hex_to_bytes(&line[1..]).map_err(|e| format!("Hex parse error: {}", e))?;
        if bytes.len() < 5 {
            return Err("Invalid HEX record".to_string());
        }

        let byte_count = bytes[0] as u32;
        let addr = ((bytes[1] as u32) << 8) | (bytes[2] as u32);
        let record_type = bytes[3];

        match record_type {
            0x00 => {
                // Data record
                let start = (base_addr + addr) as usize;
                let data = &bytes[4..(4 + byte_count as usize)];
                if start + data.len() > binary.len() {
                    binary.resize(start + data.len(), 0xFF);
                }
                binary[start..start + data.len()].copy_from_slice(data);
            }
            0x01 => {
                // End of file
                break;
            }
            0x02 => {
                // Extended segment address
                if bytes.len() >= 6 {
                    base_addr = ((bytes[4] as u32) << 8 | (bytes[5] as u32)) << 4;
                }
            }
            0x04 => {
                // Extended linear address
                if bytes.len() >= 6 {
                    base_addr = ((bytes[4] as u32) << 8 | (bytes[5] as u32)) << 16;
                }
            }
            _ => {
                // Skip unknown record types
            }
        }
    }

    Ok(binary)
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Odd number of hex characters".to_string());
    }

    let mut bytes = Vec::new();
    for chunk in hex.as_bytes().chunks(2) {
        let high = hex_digit(chunk[0])?;
        let low = hex_digit(chunk[1])?;
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

fn hex_digit(b: u8) -> Result<u8, String> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(format!("Invalid hex char: {}", b as char)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stc_family() {
        assert_eq!(StcFamily::from_family_code(0x01), StcFamily::STC89);
        assert_eq!(StcFamily::from_family_code(0x25), StcFamily::STC8);
        assert_eq!(StcFamily::from_family_code(0x52), StcFamily::STC8H);
        assert!(matches!(StcFamily::from_family_code(0xFF), StcFamily::Unknown(_)));
    }

    #[test]
    fn test_chip_info() {
        let info = ChipInfo::from_handshake(0x25, 0x00);
        assert_eq!(info.family, StcFamily::STC8);
        assert_eq!(info.flash_size, 32 * 1024);
    }

    #[test]
    fn test_parse_hex_file() {
        // Minimal Intel HEX: one data record + end record
        let hex = ":03000000010203F2\n:00000001FF\n";
        let binary = parse_hex_file(hex).unwrap();
        assert_eq!(binary, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_hex_to_bytes() {
        assert_eq!(hex_to_bytes("48656C6C6F").unwrap(), b"Hello");
        assert!(hex_to_bytes("1").is_err());
    }
}
