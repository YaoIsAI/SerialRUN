use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialConfig {
    pub port_name: String,
    pub baud_rate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
    pub flow_control: FlowControl,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataBits {
    Five,
    Six,
    Seven,
    Eight,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StopBits {
    One,
    Two,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Parity {
    None,
    Odd,
    Even,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FlowControl {
    None,
    Software,
    Hardware,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub serial: SerialConfig,
    pub log_dir: PathBuf,
    pub auto_reconnect: bool,
    pub hex_mode: bool,
    pub timestamp_logs: bool,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            port_name: String::new(),
            baud_rate: 115200,
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            parity: Parity::None,
            flow_control: FlowControl::None,
            timeout_ms: 1000,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            serial: SerialConfig::default(),
            log_dir: PathBuf::from("logs"),
            auto_reconnect: true,
            hex_mode: false,
            timestamp_logs: true,
        }
    }
}

impl SerialConfig {
    pub fn new(port_name: impl Into<String>) -> Self {
        Self {
            port_name: port_name.into(),
            ..Default::default()
        }
    }

    pub fn with_baud_rate(mut self, baud_rate: u32) -> Self {
        self.baud_rate = baud_rate;
        self
    }

    pub fn with_data_bits(mut self, data_bits: DataBits) -> Self {
        self.data_bits = data_bits;
        self
    }

    pub fn with_stop_bits(mut self, stop_bits: StopBits) -> Self {
        self.stop_bits = stop_bits;
        self
    }

    pub fn with_parity(mut self, parity: Parity) -> Self {
        self.parity = parity;
        self
    }

    pub fn with_flow_control(mut self, flow_control: FlowControl) -> Self {
        self.flow_control = flow_control;
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn to_toml(&self) -> anyhow::Result<String> {
        Ok(toml::to_string_pretty(self)?)
    }

    pub fn from_toml(s: &str) -> anyhow::Result<Self> {
        Ok(toml::from_str(s)?)
    }
}

impl AppConfig {
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_serial_config() {
        let config = SerialConfig::default();
        assert_eq!(config.baud_rate, 115200);
        assert_eq!(config.data_bits, DataBits::Eight);
        assert_eq!(config.stop_bits, StopBits::One);
        assert_eq!(config.parity, Parity::None);
        assert_eq!(config.flow_control, FlowControl::None);
    }

    #[test]
    fn test_serial_config_builder() {
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
    fn test_serial_config_toml_roundtrip() {
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
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.serial.baud_rate, 115200);
        assert!(config.auto_reconnect);
        assert!(!config.hex_mode);
        assert!(config.timestamp_logs);
    }
}
