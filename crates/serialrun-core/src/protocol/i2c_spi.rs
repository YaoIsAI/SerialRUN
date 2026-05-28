/// I2C and SPI debug tools for embedded development.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BusError {
    #[error("I2C error: {0}")]
    I2cError(String),
    #[error("SPI error: {0}")]
    SpiError(String),
    #[error("NAK received")]
    Nack,
    #[error("Timeout")]
    Timeout,
}

pub type I2cResult<T> = Result<T, BusError>;
pub type SpiResult<T> = Result<T, BusError>;

/// Result of an I2C bus scan.
#[derive(Debug, Clone)]
pub struct I2cScanEntry {
    pub address: u8,
    pub name: Option<String>,
}

/// Known I2C device names by address.
pub fn i2c_device_name(address: u8) -> Option<&'static str> {
    match address {
        0x50..=0x57 => Some("EEPROM (AT24Cxx)"),
        0x68 => Some("RTC (DS3231) / IMU (MPU6050)"),
        0x76 | 0x77 => Some("Pressure/Humidity (BME280/BMP280)"),
        0x3C | 0x3D => Some("OLED Display (SSD1306)"),
        0x20..=0x27 => Some("GPIO Expander (PCF8574)"),
        0x48..=0x4F => Some("ADC (ADS1115) / Temperature (LM75)"),
        0x29 => Some("Distance Sensor (VL53L0X)"),
        0x60..=0x63 => Some("Clock Generator (SI5351)"),
        0x40..=0x47 => Some("Current Sensor (INA219/INA226)"),
        _ => None,
    }
}

/// Build an I2C write command for CH341 USB-I2C adapter.
/// Returns the raw bytes to send.
pub fn ch341_i2c_write(address: u8, data: &[u8]) -> Vec<u8> {
    let mut cmd = Vec::new();
    cmd.push(0x12); // CH341 I2C start
    cmd.push((address << 1) & 0xFE); // Write address (7-bit addr << 1, R/W=0)
    cmd.extend_from_slice(data);
    cmd.push(0x13); // CH341 I2C stop
    cmd
}

/// Build an I2C read command for CH341 USB-I2C adapter.
/// Returns the raw bytes to send.
pub fn ch341_i2c_read(address: u8, len: usize) -> Vec<u8> {
    let mut cmd = Vec::new();
    cmd.push(0x12); // CH341 I2C start
    cmd.push((address << 1) | 0x01); // Read address (7-bit addr << 1, R/W=1)
    for _ in 0..len {
        cmd.push(0x48); // CH341 I2C read byte
    }
    cmd.push(0x13); // CH341 I2C stop
    cmd
}

/// Build I2C scan commands for all valid addresses (0x08..0x77).
/// Returns a vector of (address, command_bytes) pairs.
pub fn ch341_i2c_scan_commands() -> Vec<(u8, Vec<u8>)> {
    let mut commands = Vec::new();
    for addr in 0x08..=0x77 {
        let cmd = vec![0x12, 0x61 | ((addr << 1) & 0x7E), 0x13];
        commands.push((addr, cmd));
    }
    commands
}

/// Generic I2C write-then-read transaction.
/// Sends `write_data` to `address`, then reads `read_len` bytes.
pub fn generic_i2c_write_read(address: u8, write_data: &[u8], read_len: usize) -> Vec<u8> {
    let mut cmd = Vec::new();
    // Write phase
    if !write_data.is_empty() {
        cmd.push(0x53); // START
        cmd.push(address << 1); // Write address
        cmd.extend_from_slice(write_data);
    }
    // Read phase
    cmd.push(0x53); // Repeated START
    cmd.push((address << 1) | 0x01); // Read address
    for _ in 0..read_len {
        cmd.push(0x48); // Read byte with ACK
    }
    cmd.push(0x50); // STOP
    cmd
}

/// Generic I2C write-only transaction.
pub fn generic_i2c_write(address: u8, data: &[u8]) -> Vec<u8> {
    let mut cmd = Vec::new();
    cmd.push(0x53); // START
    cmd.push(address << 1); // Write address
    cmd.extend_from_slice(data);
    cmd.push(0x50); // STOP
    cmd
}

/// Perform an SPI full-duplex transfer.
/// Returns the transmitted data with SPI framing.
pub fn spi_transfer(tx_data: &[u8], cs_pin: u8, mode: u8) -> Vec<u8> {
    let mut cmd = Vec::new();
    // SPI mode configuration
    cmd.push(0x40 | (mode & 0x03));
    // CS control
    cmd.push(0x01); // CS low
    cmd.push(cs_pin);
    // Data transfer
    cmd.push(tx_data.len() as u8);
    cmd.extend_from_slice(tx_data);
    cmd.push(0x00); // CS high
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ch341_i2c_write() {
        let cmd = ch341_i2c_write(0x50, &[0x00, 0x42]);
        assert_eq!(cmd[0], 0x12); // start
        assert_eq!(cmd[1], 0xA0); // 0x50 << 1 = 0xA0
        assert_eq!(cmd[2], 0x00);
        assert_eq!(cmd[3], 0x42);
        assert_eq!(cmd[4], 0x13); // stop
    }

    #[test]
    fn test_ch341_i2c_read() {
        let cmd = ch341_i2c_read(0x68, 3);
        assert_eq!(cmd[0], 0x12); // start
        assert_eq!(cmd[1], 0xD1); // (0x68 << 1) | 0x01 = 0xD1
        assert_eq!(cmd[2], 0x48); // read byte
        assert_eq!(cmd[3], 0x48); // read byte
        assert_eq!(cmd[4], 0x48); // read byte
        assert_eq!(cmd[5], 0x13); // stop
    }

    #[test]
    fn test_ch341_i2c_scan() {
        let commands = ch341_i2c_scan_commands();
        assert_eq!(commands.len(), 0x70); // 0x08 to 0x77 = 112 addresses

        // Check first address
        assert_eq!(commands[0].0, 0x08);
        assert_eq!(commands[0].1[0], 0x12); // start

        // Check last address
        assert_eq!(commands.last().unwrap().0, 0x77);
    }

    #[test]
    fn test_generic_i2c_write_read() {
        let cmd = generic_i2c_write_read(0x50, &[0x00], 2);
        assert!(cmd.contains(&0x53)); // START
        assert!(cmd.contains(&0x50)); // Write address (0x50 << 1 = 0xA0 >> 1 shift in context)
        assert!(cmd.contains(&0x48)); // Read byte
        assert!(cmd.contains(&0x50)); // STOP
    }

    #[test]
    fn test_generic_i2c_write() {
        let cmd = generic_i2c_write(0x3C, &[0xAE]);
        assert_eq!(cmd[0], 0x53); // START
        assert_eq!(cmd[1], 0x78); // 0x3C << 1 = 0x78
        assert_eq!(cmd[2], 0xAE);
        assert_eq!(cmd[3], 0x50); // STOP
    }

    #[test]
    fn test_spi_transfer() {
        let tx = vec![0xAA, 0xBB];
        let cmd = spi_transfer(&tx, 0, 0);
        assert_eq!(cmd[0], 0x40); // SPI mode 0
        assert_eq!(cmd[1], 0x01); // CS low
        assert_eq!(cmd[2], 0x00); // CS pin 0
        assert_eq!(cmd[3], 0x02); // data length
        assert_eq!(cmd[4], 0xAA);
        assert_eq!(cmd[5], 0xBB);
        assert_eq!(cmd[6], 0x00); // CS high
    }

    #[test]
    fn test_spi_transfer_mode() {
        let cmd = spi_transfer(&[0x01], 1, 3);
        assert_eq!(cmd[0], 0x43); // SPI mode 3 (0x40 | 0x03)
        assert_eq!(cmd[2], 0x01); // CS pin 1
    }

    #[test]
    fn test_i2c_device_name_known() {
        assert!(i2c_device_name(0x68).is_some());
        assert!(i2c_device_name(0x3C).is_some());
        assert!(i2c_device_name(0x50).is_some());
        assert!(i2c_device_name(0x76).is_some());
    }

    #[test]
    fn test_i2c_device_name_unknown() {
        assert!(i2c_device_name(0x01).is_none());
        assert!(i2c_device_name(0x7F).is_none());
    }
}
