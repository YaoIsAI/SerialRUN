/// Modbus master functionality - re-exports types from modbus module.

use crate::protocol::modbus::{ModbusFrame, ModbusFunction};
use std::time::Duration;

/// Errors that can occur during Modbus master operations.
#[derive(Debug)]
pub enum ModbusMasterError {
    PortError(String),
    Timeout,
    Exception(u8),
}

impl std::fmt::Display for ModbusMasterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PortError(e) => write!(f, "Port error: {}", e),
            Self::Timeout => write!(f, "Timeout"),
            Self::Exception(code) => write!(f, "Exception: {}", code),
        }
    }
}

impl std::error::Error for ModbusMasterError {}

pub type ModbusMasterResult<T> = Result<T, ModbusMasterError>;

/// A Modbus master for sending requests and receiving responses.
///
/// This is a placeholder struct - actual implementation would use SerialPort
/// for communication.
pub struct ModbusMaster {
    slave_id: u8,
    timeout: Duration,
}

impl ModbusMaster {
    /// Create a new ModbusMaster with the given slave ID.
    pub fn new(slave_id: u8) -> Self {
        Self {
            slave_id,
            timeout: Duration::from_secs(1),
        }
    }

    /// Set the timeout for Modbus transactions.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Get the current slave ID.
    pub fn slave_id(&self) -> u8 {
        self.slave_id
    }

    /// Build a read holding registers request.
    pub fn build_read_holding_registers(
        &self,
        start_addr: u16,
        quantity: u16,
    ) -> ModbusFrame {
        ModbusFrame::new(
            self.slave_id,
            ModbusFunction::ReadHoldingRegisters,
            {
                let mut data = Vec::new();
                data.extend_from_slice(&start_addr.to_be_bytes());
                data.extend_from_slice(&quantity.to_be_bytes());
                data
            },
        )
    }

    /// Build a read input registers request.
    pub fn build_read_input_registers(
        &self,
        start_addr: u16,
        quantity: u16,
    ) -> ModbusFrame {
        ModbusFrame::new(
            self.slave_id,
            ModbusFunction::ReadInputRegisters,
            {
                let mut data = Vec::new();
                data.extend_from_slice(&start_addr.to_be_bytes());
                data.extend_from_slice(&quantity.to_be_bytes());
                data
            },
        )
    }

    /// Build a write single register request.
    pub fn build_write_single_register(&self, addr: u16, value: u16) -> ModbusFrame {
        ModbusFrame::new(
            self.slave_id,
            ModbusFunction::WriteSingleRegister,
            {
                let mut data = Vec::new();
                data.extend_from_slice(&addr.to_be_bytes());
                data.extend_from_slice(&value.to_be_bytes());
                data
            },
        )
    }

    /// Build a write multiple registers request.
    pub fn build_write_multiple_registers(
        &self,
        start_addr: u16,
        values: &[u16],
    ) -> ModbusFrame {
        let mut data = Vec::new();
        data.extend_from_slice(&start_addr.to_be_bytes());
        data.push(values.len() as u8);
        for &val in values {
            data.extend_from_slice(&val.to_be_bytes());
        }
        ModbusFrame::new(self.slave_id, ModbusFunction::WriteMultipleRegisters, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modbus_master_new() {
        let master = ModbusMaster::new(0x01);
        assert_eq!(master.slave_id(), 0x01);
    }

    #[test]
    fn test_modbus_master_timeout() {
        let master = ModbusMaster::new(0x01).with_timeout(Duration::from_secs(5));
        assert_eq!(master.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_build_read_holding_registers() {
        let master = ModbusMaster::new(0x01);
        let frame = master.build_read_holding_registers(0x0000, 0x000A);

        assert_eq!(frame.slave_id, 0x01);
        assert_eq!(frame.function, ModbusFunction::ReadHoldingRegisters);
        assert_eq!(frame.data, vec![0x00, 0x00, 0x00, 0x0A]);
    }

    #[test]
    fn test_build_read_input_registers() {
        let master = ModbusMaster::new(0x02);
        let frame = master.build_read_input_registers(0x0100, 0x0005);

        assert_eq!(frame.slave_id, 0x02);
        assert_eq!(frame.function, ModbusFunction::ReadInputRegisters);
    }

    #[test]
    fn test_build_write_single_register() {
        let master = ModbusMaster::new(0x01);
        let frame = master.build_write_single_register(0x0010, 0x1234);

        assert_eq!(frame.slave_id, 0x01);
        assert_eq!(frame.function, ModbusFunction::WriteSingleRegister);
        assert_eq!(frame.data, vec![0x00, 0x10, 0x12, 0x34]);
    }

    #[test]
    fn test_build_write_multiple_registers() {
        let master = ModbusMaster::new(0x01);
        let frame = master.build_write_multiple_registers(0x0000, &[0x1234, 0x5678]);

        assert_eq!(frame.slave_id, 0x01);
        assert_eq!(frame.function, ModbusFunction::WriteMultipleRegisters);
        // start_addr(2) + count(1) + values(4) = 7 bytes
        assert_eq!(frame.data.len(), 7);
    }

    #[test]
    fn test_error_display() {
        let e = ModbusMasterError::PortError("test".to_string());
        assert!(format!("{}", e).contains("test"));

        let e = ModbusMasterError::Timeout;
        assert_eq!(format!("{}", e), "Timeout");

        let e = ModbusMasterError::Exception(0x02);
        assert!(format!("{}", e).contains("2"));
    }
}
