/// Serial flasher for STM32 and ESP32 microcontrollers.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlashError {
    #[error("Flash error: {0}")]
    FlashError(String),
    #[error("Timeout")]
    Timeout,
    #[error("Checksum mismatch: expected {expected:#04X}, got {got:#04X}")]
    ChecksumMismatch { expected: u16, got: u16 },
    #[error("Invalid hex format: {0}")]
    InvalidIhex(String),
    #[error("Invalid binary: {0}")]
    InvalidBinary(String),
}

pub type FlashResult<T> = Result<T, FlashError>;

/// Parsed Intel HEX record.
#[derive(Debug, Clone)]
pub struct IhexRecord {
    pub byte_count: u8,
    pub address: u16,
    pub record_type: u8,
    pub data: Vec<u8>,
}

/// STM32 flasher using the STM32 bootloader protocol.
pub struct Stm32Flasher {
    buffer: Vec<u8>,
}

impl Default for Stm32Flasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Stm32Flasher {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    /// Build a GET command (0x00) to read the bootloader version.
    pub fn get_command(&self) -> Vec<u8> {
        vec![0x7F] // SYNC + GET
    }

    /// Build an ACK response check.
    pub fn ack(&self) -> u8 {
        0x79
    }

    /// Build a NAK response check.
    pub fn nack(&self) -> u8 {
        0x1F
    }

    /// Build an erase memory command.
    /// Pages: list of page numbers to erase.
    pub fn erase_memory(&self, pages: &[u16]) -> Vec<u8> {
        let mut cmd = Vec::new();
        cmd.push(0x43); // Erase command
        cmd.push(0x44); // Complement

        let page_count = pages.len() as u8;
        cmd.push(page_count.wrapping_sub(1));

        for &page in pages {
            cmd.extend_from_slice(&page.to_be_bytes());
        }

        // XOR checksum of payload
        let mut checksum = 0x43u8;
        checksum ^= 0x44;
        checksum ^= page_count.wrapping_sub(1);
        for &page in pages {
            checksum ^= (page >> 8) as u8;
            checksum ^= (page & 0xFF) as u8;
        }
        cmd.push(checksum);

        cmd
    }

    /// Build a write memory command.
    /// address: starting memory address
    /// data: bytes to write
    pub fn write_memory(&self, address: u32, data: &[u8]) -> Vec<u8> {
        let mut cmd = Vec::new();
        cmd.push(0x31); // Write memory command
        cmd.push(0xCE); // Complement

        let addr_bytes = address.to_be_bytes();
        cmd.extend_from_slice(&addr_bytes);
        cmd.push(address.to_be_bytes().iter().fold(0u8, |acc, &b| acc ^ b));

        let len = (data.len() - 1) as u8;
        cmd.push(len);

        let mut checksum = len;
        for &b in data {
            checksum ^= b;
        }
        for &b in &addr_bytes {
            checksum ^= b;
        }
        cmd.extend_from_slice(data);
        cmd.push(checksum);

        cmd
    }

    /// Build a read memory command.
    pub fn read_memory(&self, address: u32, len: u8) -> Vec<u8> {
        let mut cmd = Vec::new();
        cmd.push(0x11); // Read memory command
        cmd.push(0xEE); // Complement

        let addr_bytes = address.to_be_bytes();
        cmd.extend_from_slice(&addr_bytes);
        cmd.push(addr_bytes.iter().fold(0u8, |acc, &b| acc ^ b));

        cmd.push(len.wrapping_sub(1));
        cmd.push((len - 1) ^ 0xFF);

        cmd
    }

    /// Build a GO (jump) command.
    pub fn go(&self, address: u32) -> Vec<u8> {
        let mut cmd = Vec::new();
        cmd.push(0x21); // Go command
        cmd.push(0xDE); // Complement

        let addr_bytes = address.to_be_bytes();
        cmd.extend_from_slice(&addr_bytes);
        cmd.push(addr_bytes.iter().fold(0u8, |acc, &b| acc ^ b));

        cmd
    }

    /// Build a read UID (unique ID) command.
    pub fn read_uid(&self) -> Vec<u8> {
        vec![0x27, 0xD8] // Get UID command + complement
    }

    /// Build a mass erase command (0xFF pages).
    pub fn mass_erase(&self) -> Vec<u8> {
        self.erase_memory(&[0xFFFF])
    }

    /// Calculate a simple checksum of data bytes.
    pub fn checksum(&self, data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &b| acc ^ b)
    }
}

/// ESP32 flasher using the ESP32 bootloader stub protocol.
pub struct Esp32Flasher {
    buffer: Vec<u8>,
}

impl Default for Esp32Flasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Esp32Flasher {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    /// Build a sync command to synchronize with the bootloader.
    pub fn sync(&self) -> Vec<u8> {
        let mut cmd = Vec::new();
        // SLIP framing
        cmd.push(0xC0); // Packet start
        cmd.push(0x08); // Command: sync
        cmd.extend_from_slice(&0u16.to_le_bytes()); // Direction: request
        cmd.extend_from_slice(&0u32.to_le_bytes()); // Size
        cmd.extend_from_slice(&0u32.to_le_bytes()); // Checksum

        // Sync data
        let sync_str = b"SYNC\x0D\x0A\x0D\x0A";
        cmd.extend_from_slice(sync_str);

        cmd.push(0xC0); // Packet end
        cmd
    }

    /// Build a chip ID command.
    pub fn chip_id(&self) -> Vec<u8> {
        let mut cmd = Vec::new();
        cmd.push(0xC0);
        cmd.push(0x0B); // Command: read_mac / chip_id
        cmd.extend_from_slice(&0u16.to_le_bytes());
        cmd.extend_from_slice(&0u32.to_le_bytes());
        cmd.extend_from_slice(&0u32.to_le_bytes());
        cmd.push(0xC0);
        cmd
    }

    /// Build a version command.
    pub fn version(&self) -> Vec<u8> {
        let mut cmd = Vec::new();
        cmd.push(0xC0);
        cmd.push(0x08); // Command: version
        cmd.extend_from_slice(&0u16.to_le_bytes());
        cmd.extend_from_slice(&0u32.to_le_bytes());
        cmd.extend_from_slice(&0u32.to_le_bytes());
        cmd.push(0xC0);
        cmd
    }

    /// Build a flash begin command.
    /// erase_size: size of each sector
    /// num_sectors: number of sectors to erase
    /// block_size: write block size
    pub fn flash_begin(&self, erase_size: u32, num_sectors: u32) -> Vec<u8> {
        let mut cmd = Vec::new();
        cmd.push(0xC0);
        cmd.push(0x02); // Command: flash_begin
        cmd.extend_from_slice(&0u16.to_le_bytes());
        let size = 12u32;
        cmd.extend_from_slice(&size.to_le_bytes());

        let mut payload = Vec::new();
        payload.extend_from_slice(&erase_size.to_le_bytes());
        payload.extend_from_slice(&num_sectors.to_le_bytes());
        let block_size = 4096u32;
        payload.extend_from_slice(&block_size.to_le_bytes());

        let checksum = payload.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        cmd.extend_from_slice(&checksum.to_le_bytes());
        cmd.extend_from_slice(&payload);
        cmd.push(0xC0);
        cmd
    }

    /// Build a flash finish command.
    pub fn flash_finish(&self) -> Vec<u8> {
        let mut cmd = Vec::new();
        cmd.push(0xC0);
        cmd.push(0x04); // Command: flash_finish
        cmd.extend_from_slice(&0u16.to_le_bytes());
        cmd.extend_from_slice(&4u32.to_le_bytes());
        cmd.extend_from_slice(&0u32.to_le_bytes()); // checksum
        cmd.extend_from_slice(&0u32.to_le_bytes()); // reboot = false
        cmd.push(0xC0);
        cmd
    }

    /// Build an SPI flash write command.
    /// address: flash address
    /// data: data to write
    pub fn spi_flash_write(&self, address: u32, data: &[u8]) -> Vec<u8> {
        let mut cmd = Vec::new();
        cmd.push(0xC0);
        cmd.push(0x03); // Command: flash_data
        cmd.extend_from_slice(&0u16.to_le_bytes());
        let size = (8 + data.len()) as u32;
        cmd.extend_from_slice(&size.to_le_bytes());

        let checksum = data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        cmd.extend_from_slice(&checksum.to_le_bytes());
        cmd.extend_from_slice(&address.to_le_bytes());
        cmd.extend_from_slice(data);
        cmd.push(0xC0);
        cmd
    }
}

/// Parse an Intel HEX file into a list of records.
pub fn parse_ihex(content: &str) -> FlashResult<Vec<IhexRecord>> {
    let mut records = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if !line.starts_with(':') {
            return Err(FlashError::InvalidIhex(format!(
                "Line does not start with ':' : {}",
                line
            )));
        }

        let hex = &line[1..];

        if hex.len() < 10 {
            return Err(FlashError::InvalidIhex(format!(
                "Line too short: {}",
                line
            )));
        }

        let byte_count = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|e| FlashError::InvalidIhex(format!("Invalid byte count: {}", e)))?;

        let address = u16::from_str_radix(&hex[2..6], 16)
            .map_err(|e| FlashError::InvalidIhex(format!("Invalid address: {}", e)))?;

        let record_type = u8::from_str_radix(&hex[6..8], 16)
            .map_err(|e| FlashError::InvalidIhex(format!("Invalid record type: {}", e)))?;

        let mut data = Vec::new();
        for i in (8..8 + byte_count as usize * 2).step_by(2) {
            if i + 2 <= hex.len() {
                let byte = u8::from_str_radix(&hex[i..i + 2], 16)
                    .map_err(|e| FlashError::InvalidIhex(format!("Invalid data byte: {}", e)))?;
                data.push(byte);
            }
        }

        // Verify checksum
        let mut sum = 0u8;
        sum = sum.wrapping_add(byte_count);
        sum = sum.wrapping_add((address >> 8) as u8);
        sum = sum.wrapping_add((address & 0xFF) as u8);
        sum = sum.wrapping_add(record_type);
        for &b in &data {
            sum = sum.wrapping_add(b);
        }

        if let Some(checksum) = hex.get(8 + byte_count as usize * 2..8 + byte_count as usize * 2 + 2) {
            let expected = u8::from_str_radix(checksum, 16)
                .map_err(|e| FlashError::InvalidIhex(format!("Invalid checksum: {}", e)))?;
            if sum.wrapping_add(expected) != 0 {
                return Err(FlashError::InvalidIhex(format!(
                    "Checksum mismatch at address {:#06X}",
                    address
                )));
            }
        }

        records.push(IhexRecord {
            byte_count,
            address,
            record_type,
            data,
        });

        // Stop at end-of-file record
        if record_type == 0x01 {
            break;
        }
    }

    Ok(records)
}

/// Parse binary data into a simple record (all data at offset 0).
pub fn parse_binary(data: &[u8]) -> FlashResult<Vec<IhexRecord>> {
    if data.is_empty() {
        return Err(FlashError::InvalidBinary("Empty binary data".to_string()));
    }

    Ok(vec![IhexRecord {
        byte_count: data.len().min(255) as u8,
        address: 0,
        record_type: 0x00,
        data: data.to_vec(),
    }])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stm32_get_command() {
        let flasher = Stm32Flasher::new();
        let cmd = flasher.get_command();
        assert_eq!(cmd, vec![0x7F]);
    }

    #[test]
    fn test_stm32_ack_nack() {
        let flasher = Stm32Flasher::new();
        assert_eq!(flasher.ack(), 0x79);
        assert_eq!(flasher.nack(), 0x1F);
    }

    #[test]
    fn test_stm32_erase_memory() {
        let flasher = Stm32Flasher::new();
        let cmd = flasher.erase_memory(&[0, 1]);
        assert_eq!(cmd[0], 0x43);
        assert_eq!(cmd[1], 0x44);
    }

    #[test]
    fn test_stm32_mass_erase() {
        let flasher = Stm32Flasher::new();
        let cmd = flasher.mass_erase();
        assert!(!cmd.is_empty());
    }

    #[test]
    fn test_stm32_write_memory() {
        let flasher = Stm32Flasher::new();
        let data = vec![0xAA, 0xBB, 0xCC];
        let cmd = flasher.write_memory(0x08000000, &data);
        assert_eq!(cmd[0], 0x31);
        assert_eq!(cmd[1], 0xCE);
    }

    #[test]
    fn test_stm32_read_memory() {
        let flasher = Stm32Flasher::new();
        let cmd = flasher.read_memory(0x08000000, 16);
        assert_eq!(cmd[0], 0x11);
        assert_eq!(cmd[1], 0xEE);
    }

    #[test]
    fn test_stm32_go() {
        let flasher = Stm32Flasher::new();
        let cmd = flasher.go(0x08000000);
        assert_eq!(cmd[0], 0x21);
        assert_eq!(cmd[1], 0xDE);
    }

    #[test]
    fn test_stm32_read_uid() {
        let flasher = Stm32Flasher::new();
        let cmd = flasher.read_uid();
        assert_eq!(cmd, vec![0x27, 0xD8]);
    }

    #[test]
    fn test_stm32_checksum() {
        let flasher = Stm32Flasher::new();
        let data = [0x01, 0x02, 0x03];
        assert_eq!(flasher.checksum(&data), 0x01 ^ 0x02 ^ 0x03);
    }

    #[test]
    fn test_esp32_sync() {
        let flasher = Esp32Flasher::new();
        let cmd = flasher.sync();
        assert!(cmd.contains(&0xC0));
        assert!(cmd.contains(&0x08));
    }

    #[test]
    fn test_esp32_chip_id() {
        let flasher = Esp32Flasher::new();
        let cmd = flasher.chip_id();
        assert!(cmd.contains(&0x0B));
    }

    #[test]
    fn test_esp32_version() {
        let flasher = Esp32Flasher::new();
        let cmd = flasher.version();
        assert!(cmd.contains(&0x08));
    }

    #[test]
    fn test_esp32_flash_begin() {
        let flasher = Esp32Flasher::new();
        let cmd = flasher.flash_begin(4096, 100);
        assert!(cmd.contains(&0x02));
    }

    #[test]
    fn test_esp32_flash_finish() {
        let flasher = Esp32Flasher::new();
        let cmd = flasher.flash_finish();
        assert!(cmd.contains(&0x04));
    }

    #[test]
    fn test_esp32_spi_flash_write() {
        let flasher = Esp32Flasher::new();
        let data = vec![0xAA, 0xBB];
        let cmd = flasher.spi_flash_write(0x00000000, &data);
        assert!(cmd.contains(&0x03));
    }

    #[test]
    fn test_parse_ihex_valid() {
        // Proper Intel HEX: :LLAAAATT[DD...]CC
        // :0400000001020304F2 - 4 data bytes at addr 0x0000
        // :00000001FF - EOF record
        let ihex = ":0400000001020304F2\n:00000001FF\n";
        let records = parse_ihex(ihex).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].byte_count, 4);
        assert_eq!(records[0].data, vec![0x01, 0x02, 0x03, 0x04]);
        assert_eq!(records[0].record_type, 0x00);
        assert_eq!(records[1].record_type, 0x01); // EOF
    }

    #[test]
    fn test_parse_ihex_invalid() {
        let result = parse_ihex("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_binary() {
        let data = [0x01, 0x02, 0x03];
        let records = parse_binary(&data).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].data, vec![0x01, 0x02, 0x03]);
        assert_eq!(records[0].address, 0);
    }

    #[test]
    fn test_parse_binary_empty() {
        let result = parse_binary(&[]);
        assert!(result.is_err());
    }
}
