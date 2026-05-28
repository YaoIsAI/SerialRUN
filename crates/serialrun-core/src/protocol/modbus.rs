use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModbusError {
    #[error("Invalid frame: {0}")]
    InvalidFrame(String),
    #[error("CRC error: expected {expected:#06x}, got {got:#06x}")]
    CrcError { expected: u16, got: u16 },
    #[error("Unsupported function code: {0:#04x}")]
    UnsupportedFunction(u8),
}

pub type ModbusResult<T> = Result<T, ModbusError>;

#[derive(Debug, Clone, PartialEq)]
pub enum ModbusFunction {
    ReadCoils,
    ReadDiscreteInputs,
    ReadHoldingRegisters,
    ReadInputRegisters,
    WriteSingleCoil,
    WriteSingleRegister,
    WriteMultipleCoils,
    WriteMultipleRegisters,
    Other(u8),
}

impl ModbusFunction {
    pub fn from_code(code: u8) -> Self {
        match code {
            0x01 => Self::ReadCoils,
            0x02 => Self::ReadDiscreteInputs,
            0x03 => Self::ReadHoldingRegisters,
            0x04 => Self::ReadInputRegisters,
            0x05 => Self::WriteSingleCoil,
            0x06 => Self::WriteSingleRegister,
            0x0F => Self::WriteMultipleCoils,
            0x10 => Self::WriteMultipleRegisters,
            _ => Self::Other(code),
        }
    }

    pub fn to_code(&self) -> u8 {
        match self {
            Self::ReadCoils => 0x01,
            Self::ReadDiscreteInputs => 0x02,
            Self::ReadHoldingRegisters => 0x03,
            Self::ReadInputRegisters => 0x04,
            Self::WriteSingleCoil => 0x05,
            Self::WriteSingleRegister => 0x06,
            Self::WriteMultipleCoils => 0x0F,
            Self::WriteMultipleRegisters => 0x10,
            Self::Other(code) => *code,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::ReadCoils => "Read Coils",
            Self::ReadDiscreteInputs => "Read Discrete Inputs",
            Self::ReadHoldingRegisters => "Read Holding Registers",
            Self::ReadInputRegisters => "Read Input Registers",
            Self::WriteSingleCoil => "Write Single Coil",
            Self::WriteSingleRegister => "Write Single Register",
            Self::WriteMultipleCoils => "Write Multiple Coils",
            Self::WriteMultipleRegisters => "Write Multiple Registers",
            Self::Other(_) => "Other",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModbusFrame {
    pub slave_id: u8,
    pub function: ModbusFunction,
    pub data: Vec<u8>,
    pub crc: u16,
}

impl ModbusFrame {
    pub fn new(slave_id: u8, function: ModbusFunction, data: Vec<u8>) -> Self {
        let crc = Self::calculate_crc(&Self::build_frame(slave_id, function.to_code(), &data));
        Self {
            slave_id,
            function,
            data,
            crc,
        }
    }

    pub fn parse(frame: &[u8]) -> ModbusResult<Self> {
        if frame.len() < 4 {
            return Err(ModbusError::InvalidFrame(
                "Frame too short".to_string(),
            ));
        }

        let slave_id = frame[0];
        let function_code = frame[1];
        let data = frame[2..frame.len() - 2].to_vec();
        let received_crc = u16::from_le_bytes([frame[frame.len() - 2], frame[frame.len() - 1]]);

        let frame_without_crc = &frame[..frame.len() - 2];
        let calculated_crc = Self::calculate_crc(frame_without_crc);

        if received_crc != calculated_crc {
            return Err(ModbusError::CrcError {
                expected: calculated_crc,
                got: received_crc,
            });
        }

        Ok(Self {
            slave_id,
            function: ModbusFunction::from_code(function_code),
            data,
            crc: received_crc,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut frame = vec![self.slave_id, self.function.to_code()];
        frame.extend_from_slice(&self.data);
        frame.extend_from_slice(&self.crc.to_le_bytes());
        frame
    }

    fn build_frame(slave_id: u8, function_code: u8, data: &[u8]) -> Vec<u8> {
        let mut frame = vec![slave_id, function_code];
        frame.extend_from_slice(data);
        frame
    }

    pub fn calculate_crc(data: &[u8]) -> u16 {
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

    pub fn is_exception(&self) -> bool {
        self.function.to_code() & 0x80 != 0
    }

    pub fn exception_code(&self) -> Option<u8> {
        if self.is_exception() && !self.data.is_empty() {
            Some(self.data[0])
        } else {
            None
        }
    }
}

pub struct ModbusParser;

impl ModbusParser {
    pub fn parse_request(data: &[u8]) -> ModbusResult<ModbusFrame> {
        ModbusFrame::parse(data)
    }

    pub fn parse_response(data: &[u8]) -> ModbusResult<ModbusFrame> {
        ModbusFrame::parse(data)
    }

    pub fn build_read_request(
        slave_id: u8,
        function: ModbusFunction,
        start_addr: u16,
        quantity: u16,
    ) -> ModbusFrame {
        let mut data = Vec::new();
        data.extend_from_slice(&start_addr.to_be_bytes());
        data.extend_from_slice(&quantity.to_be_bytes());
        ModbusFrame::new(slave_id, function, data)
    }

    pub fn build_write_single(
        slave_id: u8,
        addr: u16,
        value: u16,
    ) -> ModbusFrame {
        let mut data = Vec::new();
        data.extend_from_slice(&addr.to_be_bytes());
        data.extend_from_slice(&value.to_be_bytes());
        ModbusFrame::new(slave_id, ModbusFunction::WriteSingleRegister, data)
    }

    pub fn format_frame(frame: &ModbusFrame) -> String {
        let hex_data: Vec<String> = frame.data.iter().map(|b| format!("{:02X}", b)).collect();
        format!(
            "Slave: {} | Function: {} ({:02X}) | Data: [{}] | CRC: {:04X}",
            frame.slave_id,
            frame.function.name(),
            frame.function.to_code(),
            hex_data.join(" "),
            frame.crc
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc_calculation() {
        let data = [0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];
        let crc = ModbusFrame::calculate_crc(&data);
        assert_ne!(crc, 0);
        assert_eq!(crc, ModbusFrame::calculate_crc(&data));
    }

    #[test]
    fn test_function_codes() {
        assert_eq!(ModbusFunction::ReadCoils.to_code(), 0x01);
        assert_eq!(ModbusFunction::ReadHoldingRegisters.to_code(), 0x03);
        assert_eq!(ModbusFunction::WriteSingleRegister.to_code(), 0x06);
        assert_eq!(ModbusFunction::WriteMultipleRegisters.to_code(), 0x10);
    }

    #[test]
    fn test_frame_build_and_parse() {
        let frame = ModbusFrame::new(0x01, ModbusFunction::ReadHoldingRegisters, vec![0x00, 0x00, 0x00, 0x0A]);
        let bytes = frame.to_bytes();
        let parsed = ModbusFrame::parse(&bytes).unwrap();

        assert_eq!(parsed.slave_id, 0x01);
        assert_eq!(parsed.function, ModbusFunction::ReadHoldingRegisters);
        assert_eq!(parsed.data, vec![0x00, 0x00, 0x00, 0x0A]);
    }

    #[test]
    fn test_exception_detection() {
        let frame = ModbusFrame::new(0x01, ModbusFunction::Other(0x83), vec![0x02]);
        assert!(frame.is_exception());
        assert_eq!(frame.exception_code(), Some(0x02));
    }

    #[test]
    fn test_format_frame() {
        let frame = ModbusFrame::new(0x01, ModbusFunction::ReadHoldingRegisters, vec![0x00, 0x00]);
        let formatted = ModbusParser::format_frame(&frame);
        assert!(formatted.contains("Slave: 1"));
        assert!(formatted.contains("Read Holding Registers"));
    }
}
