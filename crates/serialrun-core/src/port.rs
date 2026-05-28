use crate::config::{DataBits, FlowControl, Parity, SerialConfig, StopBits};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PortError {
    #[error("Port not found: {0}")]
    PortNotFound(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Read error: {0}")]
    ReadError(String),
    #[error("Write error: {0}")]
    WriteError(String),
    #[error("Port is not connected")]
    NotConnected,
}

pub type PortResult<T> = Result<T, PortError>;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SerialPortInfo {
    pub name: String,
    pub description: Option<String>,
    pub manufacturer: Option<String>,
    pub serial_number: Option<String>,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
}

pub struct SerialPort {
    port: Option<Box<dyn serialport::SerialPort>>,
    config: SerialConfig,
    is_connected: bool,
}

impl SerialPort {
    pub fn new(config: SerialConfig) -> Self {
        Self {
            port: None,
            config,
            is_connected: false,
        }
    }

    pub fn list_ports() -> PortResult<Vec<SerialPortInfo>> {
        let ports = serialport::available_ports().map_err(|e| {
            PortError::ConnectionFailed(format!("Failed to list ports: {}", e))
        })?;

        Ok(ports
            .into_iter()
            .map(|p| {
                let (vid, pid, manufacturer, serial_number, description) = match &p.port_type {
                    serialport::SerialPortType::UsbPort(info) => (
                        Some(info.vid),
                        Some(info.pid),
                        info.manufacturer.clone(),
                        info.serial_number.clone(),
                        Some(format!("USB Device {:04X}:{:04X}", info.vid, info.pid)),
                    ),
                    _ => (None, None, None, None, None),
                };

                SerialPortInfo {
                    name: p.port_name,
                    description,
                    manufacturer,
                    serial_number,
                    vid,
                    pid,
                }
            })
            .collect())
    }

    pub fn connect(&mut self) -> PortResult<()> {
        if self.is_connected {
            return Ok(());
        }

        let builder = serialport::new(&self.config.port_name, self.config.baud_rate)
            .data_bits(match self.config.data_bits {
                DataBits::Five => serialport::DataBits::Five,
                DataBits::Six => serialport::DataBits::Six,
                DataBits::Seven => serialport::DataBits::Seven,
                DataBits::Eight => serialport::DataBits::Eight,
            })
            .stop_bits(match self.config.stop_bits {
                StopBits::One => serialport::StopBits::One,
                StopBits::Two => serialport::StopBits::Two,
            })
            .parity(match self.config.parity {
                Parity::None => serialport::Parity::None,
                Parity::Odd => serialport::Parity::Odd,
                Parity::Even => serialport::Parity::Even,
            })
            .flow_control(match self.config.flow_control {
                FlowControl::None => serialport::FlowControl::None,
                FlowControl::Software => serialport::FlowControl::Software,
                FlowControl::Hardware => serialport::FlowControl::Hardware,
            })
            .timeout(Duration::from_millis(self.config.timeout_ms));

        let port = builder.open().map_err(|e| {
            PortError::ConnectionFailed(format!(
                "Failed to open {}: {}",
                self.config.port_name, e
            ))
        })?;

        self.port = Some(port);
        self.is_connected = true;
        Ok(())
    }

    pub fn disconnect(&mut self) -> PortResult<()> {
        self.port = None;
        self.is_connected = false;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    pub fn read(&mut self, buf: &mut [u8]) -> PortResult<usize> {
        let port = self.port.as_mut().ok_or(PortError::NotConnected)?;
        port.read(buf).map_err(|e| PortError::ReadError(e.to_string()))
    }

    pub fn write(&mut self, buf: &[u8]) -> PortResult<usize> {
        let port = self.port.as_mut().ok_or(PortError::NotConnected)?;
        port.write(buf).map_err(|e| PortError::WriteError(e.to_string()))
    }

    pub fn write_string(&mut self, s: &str) -> PortResult<usize> {
        self.write(s.as_bytes())
    }

    pub fn bytes_to_read(&self) -> PortResult<u32> {
        let port = self.port.as_ref().ok_or(PortError::NotConnected)?;
        Ok(port.bytes_to_read().unwrap_or(0))
    }

    pub fn bytes_to_write(&self) -> PortResult<u32> {
        let port = self.port.as_ref().ok_or(PortError::NotConnected)?;
        Ok(port.bytes_to_write().unwrap_or(0))
    }

    pub fn clear_buffer(&self, buffer: ClearBuffer) -> PortResult<()> {
        let port = self.port.as_ref().ok_or(PortError::NotConnected)?;
        let buf = match buffer {
            ClearBuffer::Input => serialport::ClearBuffer::Input,
            ClearBuffer::Output => serialport::ClearBuffer::Output,
            ClearBuffer::All => serialport::ClearBuffer::All,
        };
        port.clear(buf).map_err(|e| PortError::WriteError(e.to_string()))
    }

    pub fn config(&self) -> &SerialConfig {
        &self.config
    }

    pub fn set_timeout(&mut self, timeout: Duration) -> PortResult<()> {
        let port = self.port.as_mut().ok_or(PortError::NotConnected)?;
        port.set_timeout(timeout)
            .map_err(|e| PortError::WriteError(e.to_string()))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ClearBuffer {
    Input,
    Output,
    All,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_ports() {
        let ports = SerialPort::list_ports();
        assert!(ports.is_ok());
    }

    #[test]
    fn test_new_port() {
        let config = SerialConfig::new("COM1");
        let port = SerialPort::new(config);
        assert!(!port.is_connected());
    }

    #[test]
    fn test_port_config() {
        let config = SerialConfig::new("/dev/ttyUSB0").with_baud_rate(9600);
        let port = SerialPort::new(config);
        assert_eq!(port.config().port_name, "/dev/ttyUSB0");
        assert_eq!(port.config().baud_rate, 9600);
    }
}
