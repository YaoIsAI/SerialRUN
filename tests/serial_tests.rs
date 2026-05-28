use serialrun_core::config::*;
use serialrun_core::{SerialConfig, SerialPort};

#[test]
fn test_serial_config_creation() {
    let config = SerialConfig::new("COM1")
        .with_baud_rate(9600)
        .with_data_bits(DataBits::Seven)
        .with_stop_bits(StopBits::Two)
        .with_parity(Parity::Even)
        .with_flow_control(FlowControl::Hardware);

    assert_eq!(config.port_name, "COM1");
    assert_eq!(config.baud_rate, 9600);
    assert_eq!(config.data_bits, DataBits::Seven);
    assert_eq!(config.stop_bits, StopBits::Two);
    assert_eq!(config.parity, Parity::Even);
    assert_eq!(config.flow_control, FlowControl::Hardware);
}

#[test]
fn test_serial_config_default() {
    let config = SerialConfig::default();
    assert_eq!(config.baud_rate, 115200);
    assert_eq!(config.data_bits, DataBits::Eight);
    assert_eq!(config.stop_bits, StopBits::One);
    assert_eq!(config.parity, Parity::None);
    assert_eq!(config.flow_control, FlowControl::None);
}

#[test]
fn test_serial_config_toml() {
    let config = SerialConfig::new("/dev/ttyUSB0")
        .with_baud_rate(115200)
        .with_parity(Parity::Odd);

    let toml_str = config.to_toml().unwrap();
    let restored = SerialConfig::from_toml(&toml_str).unwrap();

    assert_eq!(config.port_name, restored.port_name);
    assert_eq!(config.baud_rate, restored.baud_rate);
    assert_eq!(config.parity, restored.parity);
}

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
    assert_eq!(port.config().port_name, "COM1");
}

#[test]
fn test_port_config_access() {
    let config = SerialConfig::new("/dev/ttyS0").with_baud_rate(9600);
    let port = SerialPort::new(config);
    assert_eq!(port.config().baud_rate, 9600);
    assert_eq!(port.config().port_name, "/dev/ttyS0");
}

#[test]
fn test_data_bits_conversion() {
    let config = SerialConfig::new("test")
        .with_data_bits(DataBits::Five)
        .with_data_bits(DataBits::Six)
        .with_data_bits(DataBits::Seven)
        .with_data_bits(DataBits::Eight);

    assert_eq!(config.data_bits, DataBits::Eight);
}

#[test]
fn test_parity_conversion() {
    let config = SerialConfig::new("test")
        .with_parity(Parity::None)
        .with_parity(Parity::Odd)
        .with_parity(Parity::Even);

    assert_eq!(config.parity, Parity::Even);
}

#[test]
fn test_flow_control_conversion() {
    let config = SerialConfig::new("test")
        .with_flow_control(FlowControl::None)
        .with_flow_control(FlowControl::Software)
        .with_flow_control(FlowControl::Hardware);

    assert_eq!(config.flow_control, FlowControl::Hardware);
}
