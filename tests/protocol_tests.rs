use serialrun_core::protocol::modbus::*;
use serialrun_core::protocol::custom::*;

#[test]
fn test_modbus_crc_calculation() {
    let data = [0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];
    let crc = ModbusFrame::calculate_crc(&data);
    assert_eq!(crc, 0xC5CD);
}

#[test]
fn test_modbus_function_codes() {
    assert_eq!(ModbusFunction::ReadCoils.to_code(), 0x01);
    assert_eq!(ModbusFunction::ReadDiscreteInputs.to_code(), 0x02);
    assert_eq!(ModbusFunction::ReadHoldingRegisters.to_code(), 0x03);
    assert_eq!(ModbusFunction::ReadInputRegisters.to_code(), 0x04);
    assert_eq!(ModbusFunction::WriteSingleCoil.to_code(), 0x05);
    assert_eq!(ModbusFunction::WriteSingleRegister.to_code(), 0x06);
    assert_eq!(ModbusFunction::WriteMultipleCoils.to_code(), 0x0F);
    assert_eq!(ModbusFunction::WriteMultipleRegisters.to_code(), 0x10);
}

#[test]
fn test_modbus_frame_build_and_parse() {
    let frame = ModbusFrame::new(
        0x01,
        ModbusFunction::ReadHoldingRegisters,
        vec![0x00, 0x00, 0x00, 0x0A],
    );
    let bytes = frame.to_bytes();
    let parsed = ModbusFrame::parse(&bytes).unwrap();

    assert_eq!(parsed.slave_id, 0x01);
    assert_eq!(parsed.function, ModbusFunction::ReadHoldingRegisters);
    assert_eq!(parsed.data, vec![0x00, 0x00, 0x00, 0x0A]);
}

#[test]
fn test_modbus_exception_detection() {
    let frame = ModbusFrame::new(0x01, ModbusFunction::Other(0x83), vec![0x02]);
    assert!(frame.is_exception());
    assert_eq!(frame.exception_code(), Some(0x02));
}

#[test]
fn test_modbus_parser_build_read_request() {
    let frame = ModbusParser::build_read_request(
        0x01,
        ModbusFunction::ReadHoldingRegisters,
        0x0000,
        0x000A,
    );

    assert_eq!(frame.slave_id, 0x01);
    assert_eq!(frame.function, ModbusFunction::ReadHoldingRegisters);
    assert_eq!(frame.data, vec![0x00, 0x00, 0x00, 0x0A]);
}

#[test]
fn test_modbus_parser_build_write_single() {
    let frame = ModbusParser::build_write_single(0x01, 0x0001, 0x00FF);

    assert_eq!(frame.slave_id, 0x01);
    assert_eq!(frame.function, ModbusFunction::WriteSingleRegister);
    assert_eq!(frame.data, vec![0x00, 0x01, 0x00, 0xFF]);
}

#[test]
fn test_modbus_parser_format_frame() {
    let frame = ModbusFrame::new(0x01, ModbusFunction::ReadHoldingRegisters, vec![0x00, 0x00]);
    let formatted = ModbusParser::format_frame(&frame);

    assert!(formatted.contains("Slave: 1"));
    assert!(formatted.contains("Read Holding Registers"));
    assert!(formatted.contains("00 00"));
}

#[test]
fn test_modbus_frame_crc_validation() {
    let frame = ModbusFrame::new(0x01, ModbusFunction::ReadHoldingRegisters, vec![0x00, 0x00]);
    let mut bytes = frame.to_bytes();

    // Corrupt CRC
    let last = bytes.len() - 1;
    bytes[last] ^= 0xFF;

    let result = ModbusFrame::parse(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_protocol_parser_default() {
    let parser = ProtocolParser::default();
    assert!(!parser.patterns().is_empty());
}

#[test]
fn test_protocol_parser_add_pattern() {
    let mut parser = ProtocolParser::new();
    assert!(parser.add_pattern("Test", r"^TEST", "Test pattern").is_ok());
    assert_eq!(parser.patterns().len(), 1);
}

#[test]
fn test_protocol_parser_parse_at_command() {
    let parser = ProtocolParser::default();
    let data = b"AT+RST\r\n";
    let frame = parser.parse(data);
    assert!(frame.is_some());
    let frame = frame.unwrap();
    assert!(frame.parsed.unwrap().contains("AT Command"));
}

#[test]
fn test_protocol_parser_parse_json() {
    let parser = ProtocolParser::default();
    let data = b"{\"key\": \"value\"}";
    let frame = parser.parse(data);
    assert!(frame.is_some());
    let frame = frame.unwrap();
    assert!(frame.parsed.unwrap().contains("JSON"));
}

#[test]
fn test_protocol_parser_parse_hex_data() {
    let parser = ProtocolParser::default();
    let data = b"48 65 6C 6C 6F";
    let frame = parser.parse(data);
    assert!(frame.is_some());
    let frame = frame.unwrap();
    assert!(frame.parsed.unwrap().contains("Hex Data"));
}

#[test]
fn test_protocol_parser_parse_error() {
    let parser = ProtocolParser::default();
    let data = b"ERROR: something went wrong";
    let frame = parser.parse(data);
    assert!(frame.is_some());
    let frame = frame.unwrap();
    assert!(frame.parsed.unwrap().contains("Error"));
}

#[test]
fn test_protocol_parser_no_match() {
    let parser = ProtocolParser::default();
    let data = b"hello world";
    let frame = parser.parse(data);
    assert!(frame.is_none());
}

#[test]
fn test_protocol_parser_custom_parser() {
    let mut parser = ProtocolParser::new();
    let _ = parser.add_pattern_with_parser(
        "Custom",
        r"^CUSTOM",
        "Custom protocol",
        Box::new(|data| {
            let text = String::from_utf8_lossy(data);
            Some(format!("Parsed: {}", text.trim()))
        }),
    );

    let data = b"CUSTOM:123";
    let frame = parser.parse(data);
    assert!(frame.is_some());
    let frame = frame.unwrap();
    assert!(frame.parsed.unwrap().contains("Parsed: CUSTOM:123"));
}

#[test]
fn test_protocol_parser_invalid_pattern() {
    let mut parser = ProtocolParser::new();
    let result = parser.add_pattern("Invalid", r"[invalid", "Invalid regex");
    assert!(result.is_err());
}

#[test]
fn test_protocol_parser_clear() {
    let mut parser = ProtocolParser::default();
    assert!(!parser.patterns().is_empty());
    parser.clear();
    assert!(parser.patterns().is_empty());
}
