use serialtap_core::config::SerialConfig;
use serialtap_core::protocol::ModbusFunction;
use serialtap_core::{SerialPort, SerialPortInfo};
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Copy, PartialEq)]
pub enum Language {
    English,
    Chinese,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Theme {
    Dark,
    Light,
}

impl Language {
    pub fn label(&self) -> &str {
        match self {
            Language::English => "English",
            Language::Chinese => "中文",
        }
    }
}

impl Theme {
    pub fn label(&self, lang: Language) -> &str {
        match (self, lang) {
            (Theme::Dark, Language::English) => "Dark",
            (Theme::Dark, Language::Chinese) => "深色",
            (Theme::Light, Language::English) => "Light",
            (Theme::Light, Language::Chinese) => "浅色",
        }
    }
}

pub struct T;

impl T {
    pub fn app_title(lang: Language) -> &'static str {
        match lang {
            Language::English => "SerialTap - Serial Port Assistant",
            Language::Chinese => "SerialTap - 串口助手",
        }
    }

    pub fn connect(lang: Language) -> &'static str {
        match lang {
            Language::English => "Connect",
            Language::Chinese => "连接",
        }
    }

    pub fn disconnect(lang: Language) -> &'static str {
        match lang {
            Language::English => "Disconnect",
            Language::Chinese => "断开",
        }
    }

    pub fn refresh_ports(lang: Language) -> &'static str {
        match lang {
            Language::English => "Refresh Ports",
            Language::Chinese => "刷新端口",
        }
    }

    pub fn send(lang: Language) -> &'static str {
        match lang {
            Language::English => "Send",
            Language::Chinese => "发送",
        }
    }

    pub fn clear(lang: Language) -> &'static str {
        match lang {
            Language::English => "Clear",
            Language::Chinese => "清空",
        }
    }

    pub fn terminal(lang: Language) -> &'static str {
        match lang {
            Language::English => "Terminal",
            Language::Chinese => "终端",
        }
    }

    pub fn settings(lang: Language) -> &'static str {
        match lang {
            Language::English => "Settings",
            Language::Chinese => "设置",
        }
    }

    pub fn connected(lang: Language) -> &'static str {
        match lang {
            Language::English => "Connected",
            Language::Chinese => "已连接",
        }
    }

    pub fn disconnected(lang: Language) -> &'static str {
        match lang {
            Language::English => "Disconnected",
            Language::Chinese => "未连接",
        }
    }

    pub fn baud_rate(lang: Language) -> &'static str {
        match lang {
            Language::English => "Baud Rate",
            Language::Chinese => "波特率",
        }
    }

    pub fn serial_port(lang: Language) -> &'static str {
        match lang {
            Language::English => "Port",
            Language::Chinese => "端口",
        }
    }

    pub fn data_bits(lang: Language) -> &'static str {
        match lang {
            Language::English => "Data Bits",
            Language::Chinese => "数据位",
        }
    }

    pub fn stop_bits(lang: Language) -> &'static str {
        match lang {
            Language::English => "Stop Bits",
            Language::Chinese => "停止位",
        }
    }

    pub fn parity(lang: Language) -> &'static str {
        match lang {
            Language::English => "Parity",
            Language::Chinese => "校验位",
        }
    }

    pub fn flow_control(lang: Language) -> &'static str {
        match lang {
            Language::English => "Flow Control",
            Language::Chinese => "流控",
        }
    }

    pub fn hex_mode(lang: Language) -> &'static str {
        match lang {
            Language::English => "HEX Mode",
            Language::Chinese => "十六进制模式",
        }
    }

    pub fn show_timestamp(lang: Language) -> &'static str {
        match lang {
            Language::English => "Show Timestamp",
            Language::Chinese => "显示时间戳",
        }
    }

    pub fn auto_scroll(lang: Language) -> &'static str {
        match lang {
            Language::English => "Auto Scroll",
            Language::Chinese => "自动滚动",
        }
    }

    pub fn language(lang: Language) -> &'static str {
        match lang {
            Language::English => "Language",
            Language::Chinese => "语言",
        }
    }

    pub fn theme(lang: Language) -> &'static str {
        match lang {
            Language::English => "Theme",
            Language::Chinese => "主题",
        }
    }

    pub fn chart(lang: Language) -> &'static str {
        match lang {
            Language::English => "Chart",
            Language::Chinese => "图表",
        }
    }

    pub fn log(lang: Language) -> &'static str {
        match lang {
            Language::English => "Log",
            Language::Chinese => "日志",
        }
    }

    pub fn recording(lang: Language) -> &'static str {
        match lang {
            Language::English => "Recording",
            Language::Chinese => "录制中",
        }
    }

    pub fn auto_reply(lang: Language) -> &'static str {
        match lang {
            Language::English => "Auto Reply",
            Language::Chinese => "自动回复",
        }
    }

    pub fn pattern(lang: Language) -> &'static str {
        match lang {
            Language::English => "Pattern",
            Language::Chinese => "匹配模式",
        }
    }

    pub fn response(lang: Language) -> &'static str {
        match lang {
            Language::English => "Response",
            Language::Chinese => "回复内容",
        }
    }

    pub fn start_recording(lang: Language) -> &'static str {
        match lang {
            Language::English => "Start Recording",
            Language::Chinese => "开始录制",
        }
    }

    pub fn stop_recording(lang: Language) -> &'static str {
        match lang {
            Language::English => "Stop Recording",
            Language::Chinese => "停止录制",
        }
    }

    pub fn clear_logs(lang: Language) -> &'static str {
        match lang {
            Language::English => "Clear Logs",
            Language::Chinese => "清空日志",
        }
    }

    pub fn export_logs(lang: Language) -> &'static str {
        match lang {
            Language::English => "Export Logs",
            Language::Chinese => "导出日志",
        }
    }

    pub fn log_viewer(lang: Language) -> &'static str {
        match lang {
            Language::English => "Log Viewer",
            Language::Chinese => "日志查看器",
        }
    }

    pub fn data_chart(lang: Language) -> &'static str {
        match lang {
            Language::English => "Data Chart",
            Language::Chinese => "数据图表",
        }
    }

    pub fn no_data(lang: Language) -> &'static str {
        match lang {
            Language::English => "No data yet",
            Language::Chinese => "暂无数据",
        }
    }

    pub fn sent(lang: Language) -> &'static str {
        match lang {
            Language::English => "Sent",
            Language::Chinese => "已发送",
        }
    }

    pub fn bytes(lang: Language) -> &'static str {
        match lang {
            Language::English => "bytes",
            Language::Chinese => "字节",
        }
    }

    pub fn display(lang: Language) -> &'static str {
        match lang {
            Language::English => "Display",
            Language::Chinese => "显示",
        }
    }

    pub fn serial_config(lang: Language) -> &'static str {
        match lang {
            Language::English => "Serial Port Configuration",
            Language::Chinese => "串口配置",
        }
    }

    pub fn help(lang: Language) -> &'static str {
        match lang {
            Language::English => "Help",
            Language::Chinese => "使用指南",
        }
    }

    pub fn quick_start(lang: Language) -> &'static str {
        match lang {
            Language::English => "Quick Start",
            Language::Chinese => "快速开始",
        }
    }

    pub fn step1(lang: Language) -> &'static str {
        match lang {
            Language::English => "1. Connect your serial device via USB",
            Language::Chinese => "1. 通过 USB 连接串口设备",
        }
    }

    pub fn step2(lang: Language) -> &'static str {
        match lang {
            Language::English => "2. Click \"Refresh\" to detect the port",
            Language::Chinese => "2. 点击「刷新」检测端口",
        }
    }

    pub fn step3(lang: Language) -> &'static str {
        match lang {
            Language::English => "3. Select port and baud rate, click \"Connect\"",
            Language::Chinese => "3. 选择端口和波特率，点击「连接」",
        }
    }

    pub fn step4(lang: Language) -> &'static str {
        match lang {
            Language::English => "4. Type commands in the input box and press Enter",
            Language::Chinese => "4. 在输入框输入命令，按回车发送",
        }
    }

    pub fn features(lang: Language) -> &'static str {
        match lang {
            Language::English => "Features",
            Language::Chinese => "功能介绍",
        }
    }

    pub fn feature_send(lang: Language) -> &'static str {
        match lang {
            Language::English => "Send/Receive text or HEX data",
            Language::Chinese => "收发文本或十六进制数据",
        }
    }

    pub fn feature_log(lang: Language) -> &'static str {
        match lang {
            Language::English => "Real-time log viewer with export",
            Language::Chinese => "实时日志查看与导出",
        }
    }

    pub fn feature_chart(lang: Language) -> &'static str {
        match lang {
            Language::English => "Data rate visualization",
            Language::Chinese => "数据速率可视化",
        }
    }

    pub fn feature_auto_reply(lang: Language) -> &'static str {
        match lang {
            Language::English => "Auto reply to matched patterns",
            Language::Chinese => "自动回复匹配的模式",
        }
    }

    pub fn feature_record(lang: Language) -> &'static str {
        match lang {
            Language::English => "Record and replay scripts",
            Language::Chinese => "录制和回放脚本",
        }
    }

    pub fn tips(lang: Language) -> &'static str {
        match lang {
            Language::English => "Tips",
            Language::Chinese => "小贴士",
        }
    }

    pub fn tip1(lang: Language) -> &'static str {
        match lang {
            Language::English => "Common baud rates: 9600, 115200",
            Language::Chinese => "常用波特率：9600、115200",
        }
    }

    pub fn tip2(lang: Language) -> &'static str {
        match lang {
            Language::English => "8N1 = 8 data bits, No parity, 1 stop bit",
            Language::Chinese => "8N1 = 8数据位, 无校验, 1停止位",
        }
    }

    pub fn tip3(lang: Language) -> &'static str {
        match lang {
            Language::English => "HEX mode for binary protocols (Modbus, etc.)",
            Language::Chinese => "十六进制模式适用于二进制协议 (Modbus等)",
        }
    }

    pub fn modbus_panel(l: Language) -> &'static str { match l { Language::English => "Modbus", Language::Chinese => "Modbus" } }
    pub fn quick_request(l: Language) -> &'static str { match l { Language::English => "Quick Request", Language::Chinese => "快速请求" } }
    pub fn slave_id(l: Language) -> &'static str { match l { Language::English => "Slave ID", Language::Chinese => "从站地址" } }
    pub fn function_code(l: Language) -> &'static str { match l { Language::English => "Function", Language::Chinese => "功能码" } }
    pub fn start_address(l: Language) -> &'static str { match l { Language::English => "Start Address", Language::Chinese => "起始地址" } }
    pub fn quantity(l: Language) -> &'static str { match l { Language::English => "Quantity", Language::Chinese => "数量" } }
    pub fn write_value(l: Language) -> &'static str { match l { Language::English => "Value", Language::Chinese => "写入值" } }
    pub fn send_request(l: Language) -> &'static str { match l { Language::English => "Send", Language::Chinese => "发送" } }
    pub fn register_monitor(l: Language) -> &'static str { match l { Language::English => "Register Monitor", Language::Chinese => "寄存器监控" } }
    pub fn poll_interval(l: Language) -> &'static str { match l { Language::English => "Interval (ms)", Language::Chinese => "间隔 (ms)" } }
    pub fn start_monitor(l: Language) -> &'static str { match l { Language::English => "Start", Language::Chinese => "开始" } }
    pub fn stop_monitor(l: Language) -> &'static str { match l { Language::English => "Stop", Language::Chinese => "停止" } }
    pub fn frame_log(l: Language) -> &'static str { match l { Language::English => "Frame Log", Language::Chinese => "帧日志" } }
    pub fn clear_frame_log(l: Language) -> &'static str { match l { Language::English => "Clear", Language::Chinese => "清空" } }
    pub fn last_request(l: Language) -> &'static str { match l { Language::English => "Request", Language::Chinese => "请求" } }
    pub fn last_response(l: Language) -> &'static str { match l { Language::English => "Response", Language::Chinese => "响应" } }
    pub fn plc_control(l: Language) -> &'static str { match l { Language::English => "PLC", Language::Chinese => "PLC 控制" } }
    pub fn plc_brand(l: Language) -> &'static str { match l { Language::English => "Brand", Language::Chinese => "品牌" } }
    pub fn plc_model(l: Language) -> &'static str { match l { Language::English => "Model", Language::Chinese => "型号" } }
    pub fn read_all(l: Language) -> &'static str { match l { Language::English => "Read All", Language::Chinese => "全部读取" } }
    pub fn address(l: Language) -> &'static str { match l { Language::English => "Address", Language::Chinese => "地址" } }
    pub fn name(l: Language) -> &'static str { match l { Language::English => "Name", Language::Chinese => "名称" } }
    pub fn value(l: Language) -> &'static str { match l { Language::English => "Value", Language::Chinese => "值" } }
    pub fn unit_label(l: Language) -> &'static str { match l { Language::English => "Unit", Language::Chinese => "单位" } }
    pub fn description(l: Language) -> &'static str { match l { Language::English => "Description", Language::Chinese => "说明" } }
    pub fn status(l: Language) -> &'static str { match l { Language::English => "Status", Language::Chinese => "状态" } }
    pub fn checksum(l: Language) -> &'static str { match l { Language::English => "Checksum", Language::Chinese => "校验码" } }
    pub fn input_data(l: Language) -> &'static str { match l { Language::English => "Input Data (HEX)", Language::Chinese => "输入数据 (HEX)" } }
    pub fn file_transfer(l: Language) -> &'static str { match l { Language::English => "File Transfer", Language::Chinese => "文件传输" } }
    pub fn send_file(l: Language) -> &'static str { match l { Language::English => "Send File", Language::Chinese => "发送文件" } }
    pub fn receive_file(l: Language) -> &'static str { match l { Language::English => "Receive File", Language::Chinese => "接收文件" } }
    pub fn protocol(l: Language) -> &'static str { match l { Language::English => "Protocol", Language::Chinese => "协议" } }
    pub fn frame_builder(l: Language) -> &'static str { match l { Language::English => "Frame Builder", Language::Chinese => "帧生成器" } }
    pub fn frame_hex(l: Language) -> &'static str { match l { Language::English => "Frame (HEX)", Language::Chinese => "帧 (HEX)" } }
    pub fn data_logger(l: Language) -> &'static str { match l { Language::English => "Data Logger", Language::Chinese => "数据记录" } }
    pub fn can_analyzer(l: Language) -> &'static str { match l { Language::English => "CAN Bus", Language::Chinese => "CAN 总线" } }
    pub fn i2c_spi(l: Language) -> &'static str { match l { Language::English => "I2C/SPI", Language::Chinese => "I2C/SPI" } }
    pub fn oscilloscope(l: Language) -> &'static str { match l { Language::English => "Scope", Language::Chinese => "示波器" } }
    pub fn flasher(l: Language) -> &'static str { match l { Language::English => "Flasher", Language::Chinese => "烧录器" } }
    pub fn register_editor(l: Language) -> &'static str { match l { Language::English => "Reg Editor", Language::Chinese => "寄存器编辑" } }
    pub fn plugins(l: Language) -> &'static str { match l { Language::English => "Plugins", Language::Chinese => "插件" } }
    pub fn auto_detect(l: Language) -> &'static str { match l { Language::English => "Auto Detect", Language::Chinese => "自动检测" } }
    pub fn import_btn(l: Language) -> &'static str { match l { Language::English => "Import", Language::Chinese => "导入" } }
    pub fn export_btn(l: Language) -> &'static str { match l { Language::English => "Export", Language::Chinese => "导出" } }
    pub fn erase(l: Language) -> &'static str { match l { Language::English => "Erase", Language::Chinese => "擦除" } }
    pub fn flash(l: Language) -> &'static str { match l { Language::English => "Flash", Language::Chinese => "烧录" } }
    pub fn scan(l: Language) -> &'static str { match l { Language::English => "Scan", Language::Chinese => "扫描" } }
    pub fn capture(l: Language) -> &'static str { match l { Language::English => "Capture", Language::Chinese => "采集" } }
}

// ── Modbus types ──

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ModbusFunctionCode {
    ReadCoils, ReadDiscreteInputs, ReadHoldingRegisters, ReadInputRegisters,
    WriteSingleCoil, WriteSingleRegister, WriteMultipleCoils, WriteMultipleRegisters,
}

impl ModbusFunctionCode {
    pub fn label(&self, l: Language) -> &'static str {
        match (self, l) {
            (Self::ReadCoils, Language::English) => "01 - Read Coils", (Self::ReadCoils, Language::Chinese) => "01 - 读线圈",
            (Self::ReadDiscreteInputs, Language::English) => "02 - Read Discrete Inputs", (Self::ReadDiscreteInputs, Language::Chinese) => "02 - 读离散输入",
            (Self::ReadHoldingRegisters, Language::English) => "03 - Read Holding Registers", (Self::ReadHoldingRegisters, Language::Chinese) => "03 - 读保持寄存器",
            (Self::ReadInputRegisters, Language::English) => "04 - Read Input Registers", (Self::ReadInputRegisters, Language::Chinese) => "04 - 读输入寄存器",
            (Self::WriteSingleCoil, Language::English) => "05 - Write Single Coil", (Self::WriteSingleCoil, Language::Chinese) => "05 - 写单个线圈",
            (Self::WriteSingleRegister, Language::English) => "06 - Write Single Register", (Self::WriteSingleRegister, Language::Chinese) => "06 - 写单个寄存器",
            (Self::WriteMultipleCoils, Language::English) => "15 - Write Multiple Coils", (Self::WriteMultipleCoils, Language::Chinese) => "15 - 写多个线圈",
            (Self::WriteMultipleRegisters, Language::English) => "16 - Write Multiple Registers", (Self::WriteMultipleRegisters, Language::Chinese) => "16 - 写多个寄存器",
        }
    }
    pub fn code(&self) -> u8 { match self { Self::ReadCoils=>0x01, Self::ReadDiscreteInputs=>0x02, Self::ReadHoldingRegisters=>0x03, Self::ReadInputRegisters=>0x04, Self::WriteSingleCoil=>0x05, Self::WriteSingleRegister=>0x06, Self::WriteMultipleCoils=>0x0F, Self::WriteMultipleRegisters=>0x10 } }
    pub fn is_read(&self) -> bool { matches!(self, Self::ReadCoils | Self::ReadDiscreteInputs | Self::ReadHoldingRegisters | Self::ReadInputRegisters) }
    pub fn to_core_function(&self) -> ModbusFunction { match self { Self::ReadCoils=>ModbusFunction::ReadCoils, Self::ReadDiscreteInputs=>ModbusFunction::ReadDiscreteInputs, Self::ReadHoldingRegisters=>ModbusFunction::ReadHoldingRegisters, Self::ReadInputRegisters=>ModbusFunction::ReadInputRegisters, Self::WriteSingleCoil=>ModbusFunction::WriteSingleCoil, Self::WriteSingleRegister=>ModbusFunction::WriteSingleRegister, Self::WriteMultipleCoils=>ModbusFunction::WriteMultipleCoils, Self::WriteMultipleRegisters=>ModbusFunction::WriteMultipleRegisters } }
    pub fn all() -> &'static [Self] { &[Self::ReadCoils, Self::ReadDiscreteInputs, Self::ReadHoldingRegisters, Self::ReadInputRegisters, Self::WriteSingleCoil, Self::WriteSingleRegister, Self::WriteMultipleCoils, Self::WriteMultipleRegisters] }
}

#[derive(Clone)]
pub struct ModbusState {
    pub slave_id: u8, pub function_code: ModbusFunctionCode, pub start_addr: String, pub quantity: String, pub write_value: String,
    pub last_request_hex: String, pub last_response_hex: String, pub last_error: Option<String>,
    pub monitor_entries: Vec<MonitorEntry>, pub monitor_polling: bool, pub monitor_interval_ms: u64, pub last_poll_time: i64,
    pub monitor_slave_id: u8, pub monitor_start_addr: String, pub monitor_quantity: String, pub monitor_function: ModbusFunctionCode,
    pub frame_log: VecDeque<ModbusFrameLogEntry>,
}

#[derive(Clone)]
pub struct MonitorEntry { pub addr: u16, pub raw_value: u16, pub display_value: String, pub last_update: i64, pub error: Option<String> }

#[derive(Clone)]
pub struct ModbusFrameLogEntry { pub timestamp: i64, pub request_hex: String, pub response_hex: String, pub decoded: String, pub is_error: bool }

// ── PLC types ──

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlcBrand { Siemens, Mitsubishi, Delta, Omron, Custom }
impl PlcBrand {
    pub fn label(&self, l: Language) -> &'static str { match (self, l) { (Self::Siemens, Language::English)=>"Siemens", (Self::Siemens, Language::Chinese)=>"西门子", (Self::Mitsubishi, Language::English)=>"Mitsubishi", (Self::Mitsubishi, Language::Chinese)=>"三菱", (Self::Delta, Language::English)=>"Delta", (Self::Delta, Language::Chinese)=>"台达", (Self::Omron, Language::English)=>"Omron", (Self::Omron, Language::Chinese)=>"欧姆龙", (Self::Custom, Language::English)=>"Custom", (Self::Custom, Language::Chinese)=>"自定义" } }
    pub fn all() -> &'static [Self] { &[Self::Siemens, Self::Mitsubishi, Self::Delta, Self::Omron, Self::Custom] }
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PlcDataType { Bool, U16, I16, U32, Float32 }
impl PlcDataType { pub fn label(&self) -> &'static str { match self { Self::Bool=>"BOOL", Self::U16=>"UINT16", Self::I16=>"INT16", Self::U32=>"UINT32", Self::Float32=>"FLOAT" } } }

#[derive(Clone)]
pub struct PlcState {
    pub selected_brand: PlcBrand,
    pub selected_model: Option<usize>,
    pub slave_id: u8,
    pub register_values: HashMap<u16, PlcRegisterValue>,
    pub custom_registers: Vec<PlcRegisterDef>,
    pub polling: bool,
    pub poll_interval_ms: u64,
    pub last_poll_time: i64,
    pub selected_register: Option<usize>,
    pub write_value: String,
    pub plc_log: VecDeque<String>,
}

#[derive(Clone)]
pub struct PlcRegisterValue { pub raw_u16: u16, pub formatted: String, pub last_update: i64, pub raw_bytes: Vec<u8> }

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PlcRegisterDef { pub addr: u16, pub name: String, pub data_type: PlcDataType, pub scale_factor: f64, pub unit: String, pub description: String }

// ── Checksum mode ──

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ChecksumMode { None, Crc16Modbus, Crc16Ccitt, Crc32, Lrc, Checksum8 }
impl ChecksumMode {
    pub fn label(&self, l: Language) -> &'static str { match (self, l) { (Self::None, Language::English)=>"None", (Self::None, Language::Chinese)=>"无", (Self::Crc16Modbus, _)=>"CRC16/MODBUS", (Self::Crc16Ccitt, _)=>"CRC16/CCITT", (Self::Crc32, _)=>"CRC32", (Self::Lrc, _)=>"LRC", (Self::Checksum8, _)=>"SUM8" } }
    pub fn all() -> &'static [Self] { &[Self::None, Self::Crc16Modbus, Self::Crc16Ccitt, Self::Crc32, Self::Lrc, Self::Checksum8] }
    pub fn append_checksum(&self, data: &[u8]) -> Vec<u8> {
        let mut r = data.to_vec();
        match self { Self::None => return data.to_vec(), Self::Crc16Modbus => { let c = serialtap_core::checksum::crc16_modbus(data); r.extend_from_slice(&c.to_le_bytes()); } Self::Crc16Ccitt => { let c = serialtap_core::checksum::crc16_ccitt(data); r.extend_from_slice(&c.to_be_bytes()); } Self::Crc32 => { let c = serialtap_core::checksum::crc32(data); r.extend_from_slice(&c.to_le_bytes()); } Self::Lrc => r.push(serialtap_core::checksum::lrc(data)), Self::Checksum8 => r.push(serialtap_core::checksum::checksum8(data)), }
        r
    }
}

// ── CAN types ──
#[derive(Clone)]
pub struct CanFrameData { pub timestamp: i64, pub id: u32, pub is_ext: bool, pub dlc: u8, pub data: Vec<u8>, pub is_error: bool }
#[derive(Clone, Default)]
pub struct CanStats { pub total_frames: u64, pub error_frames: u64, pub max_id: u32, pub ids_seen: std::collections::HashSet<u32> }

// ── I2C/SPI types ──
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum I2cMode { I2C, SPI }
impl I2cMode { pub fn label(&self) -> &'static str { match self { Self::I2C=>"I2C", Self::SPI=>"SPI" } } }

// ── Scope types ──
#[derive(Clone)]
pub struct ScopeDataPoint { pub time_ms: f64, pub value: f64 }

// ── Flasher types ──
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum McuType { Stm32, Esp32 }
impl McuType { pub fn label(&self) -> &'static str { match self { Self::Stm32=>"STM32", Self::Esp32=>"ESP32" } } }

// ── Register Editor types ──
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct RegMapEntry { pub addr: u16, pub name: String, pub data_type: String, pub value: String, pub description: String }

// ── Plugin types ──
#[derive(Clone)]
pub struct PluginInfo { pub name: String, pub version: String, pub author: String, pub loaded: bool }

pub struct AppState {
    pub ports: Vec<SerialPortInfo>,
    pub selected_port: Option<String>,
    pub config: SerialConfig,
    pub is_connected: bool,
    pub port: Option<SerialPort>,
    pub terminal_buffer: VecDeque<TerminalLine>,
    pub input_buffer: String,
    pub hex_mode: bool,
    pub auto_scroll: bool,
    pub show_timestamp: bool,
    pub terminal_checksum_mode: ChecksumMode,
    pub show_chart_window: bool,
    pub show_log_window: bool,
    pub rx_count: u64,
    pub tx_count: u64,
    pub chart_data: Vec<f64>,
    pub log_entries: Vec<LogEntry>,
    pub auto_reply_enabled: bool,
    pub auto_reply_pattern: String,
    pub auto_reply_response: String,
    pub recording: bool,
    pub script_commands: Vec<ScriptCommand>,
    pub language: Language,
    pub theme: Theme,
    pub show_help: bool,
    pub show_modbus_window: bool,
    pub modbus: ModbusState,
    pub show_plc_window: bool,
    pub plc: PlcState,
    pub show_checksum_window: bool,
    pub show_file_transfer_window: bool,
    pub file_transfer_protocol: serialtap_core::file_transfer::TransferProtocol,
    pub file_transfer_sending: bool,
    pub file_transfer_receiving: bool,
    pub file_transfer_done: bool,
    pub file_transfer_error: Option<String>,
    pub file_transfer_progress: f32,
    pub show_frame_builder_window: bool,
    pub frame_builder_slave_id: u8,
    pub frame_builder_fc: ModbusFunctionCode,
    pub frame_builder_addr: String,
    pub frame_builder_value: String,
    pub frame_builder_hex: String,
    pub frame_builder_error: Option<String>,
    pub show_data_logger_window: bool,
    pub data_logger_recording: bool,
    pub data_logger_path: String,
    pub data_logger_buffered: usize,
    pub show_can_window: bool,
    pub show_i2c_spi_window: bool,
    pub show_scope_window: bool,
    pub show_flasher_window: bool,
    pub show_register_editor_window: bool,
    pub show_plugin_window: bool,
    pub checksum_mode: ChecksumMode,
    pub checksum_input: String,
    // CAN
    pub can_capturing: bool,
    pub can_frames: Vec<CanFrameData>,
    pub can_filter_id: String,
    pub can_stats: CanStats,
    pub can_show_stats: bool,
    pub can_tx_id: String,
    pub can_tx_data: String,
    // I2C/SPI
    pub i2c_mode: I2cMode,
    pub i2c_address: String,
    pub i2c_register: String,
    pub i2c_data: String,
    pub i2c_result: String,
    // Scope
    pub scope_capturing: bool,
    pub scope_data: Vec<ScopeDataPoint>,
    pub scope_timebase_ms: f64,
    // Flasher
    pub flasher_mcu: McuType,
    pub flasher_file: String,
    pub flasher_progress: f32,
    pub flasher_log: VecDeque<String>,
    // Register Editor
    pub reg_map: Vec<RegMapEntry>,
    pub reg_selected: Option<usize>,
    pub reg_alarm_enabled: bool,
    pub reg_alarm_threshold: String,
    // Plugins
    pub plugins: Vec<PluginInfo>,
    // Help guide (external markdown)
    pub help_content_zh: String,
    pub help_content_en: String,
    // Auto-detect
    pub auto_detect_receiver: Option<std::sync::mpsc::Receiver<Option<u32>>>,
    // Async operation states
    pub modbus_async_receiver: Option<std::sync::mpsc::Receiver<Result<Vec<u8>, String>>>,
    pub plc_async_receiver: Option<std::sync::mpsc::Receiver<Result<Vec<(u16, std::result::Result<Vec<u8>, String>)>, String>>>,
    pub i2c_async_receiver: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    pub flasher_async_receiver: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    pub terminal_async_receiver: Option<std::sync::mpsc::Receiver<Result<Vec<u8>, String>>>,
}

#[derive(Clone)]
pub struct TerminalLine {
    pub timestamp: i64,
    pub direction: Direction,
    pub content: String,
    pub is_hex: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    Rx,
    Tx,
    System,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Rx => write!(f, "RX"),
            Direction::Tx => write!(f, "TX"),
            Direction::System => write!(f, "SYS"),
        }
    }
}

#[derive(Clone)]
pub struct LogEntry {
    pub timestamp: i64,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Clone)]
pub struct ScriptCommand {
    pub delay_ms: u64,
    pub action: ScriptAction,
    pub data: Option<String>,
}

#[derive(Clone, PartialEq)]
pub enum ScriptAction {
    Send,
    Wait,
    Read,
}

fn load_help_file(filename: &str) -> String {
    // 1. Current working directory (works when running from terminal in project root)
    let cwd_path = std::path::PathBuf::from(format!("docs/{}", filename));
    if let Ok(content) = std::fs::read_to_string(&cwd_path) {
        return content;
    }
    // 2. Relative to executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            // Dev build: <exe_dir>/docs/filename
            let p = exe_dir.join(format!("docs/{}", filename));
            if let Ok(content) = std::fs::read_to_string(&p) {
                return content;
            }
            // macOS .app bundle: Contents/Resources/docs/filename
            let p = exe_dir.join(format!("../Resources/docs/{}", filename));
            if let Ok(content) = std::fs::read_to_string(&p) {
                return content;
            }
        }
    }
    format!("# {}\n\nHelp file not found.", filename.trim_end_matches(".md").replace('_', " "))
}


impl AppState {
    pub fn new() -> Self {
        Self {
            ports: Vec::new(),
            selected_port: None,
            config: SerialConfig::default(),
            is_connected: false,
            port: None,
            terminal_buffer: VecDeque::new(),
            input_buffer: String::new(),
            hex_mode: false,
            auto_scroll: true,
            show_timestamp: true,
            terminal_checksum_mode: ChecksumMode::None,
            show_chart_window: false,
            show_log_window: false,
            rx_count: 0,
            tx_count: 0,
            chart_data: Vec::new(),
            log_entries: Vec::new(),
            auto_reply_enabled: false,
            auto_reply_pattern: String::new(),
            auto_reply_response: String::new(),
            recording: false,
            script_commands: Vec::new(),
            language: Language::Chinese,
            theme: Theme::Dark,
            show_help: false,
            show_modbus_window: false,
            modbus: ModbusState { slave_id: 1, function_code: ModbusFunctionCode::ReadHoldingRegisters, start_addr: "0".into(), quantity: "10".into(), write_value: String::new(), last_request_hex: String::new(), last_response_hex: String::new(), last_error: None, monitor_entries: Vec::new(), monitor_polling: false, monitor_interval_ms: 1000, last_poll_time: 0, monitor_slave_id: 1, monitor_start_addr: "0".into(), monitor_quantity: "10".into(), monitor_function: ModbusFunctionCode::ReadHoldingRegisters, frame_log: VecDeque::new() },
            show_plc_window: false,
            plc: PlcState {
                selected_brand: PlcBrand::Siemens, selected_model: None, slave_id: 1,
                register_values: HashMap::new(), custom_registers: Vec::new(),
                polling: false, poll_interval_ms: 1000, last_poll_time: 0,
                selected_register: None, write_value: String::new(), plc_log: VecDeque::new(),
            },
            show_checksum_window: false,
            show_file_transfer_window: false,
            file_transfer_protocol: serialtap_core::file_transfer::TransferProtocol::Xmodem,
            file_transfer_sending: false, file_transfer_receiving: false, file_transfer_done: false, file_transfer_error: None, file_transfer_progress: 0.0,
            show_frame_builder_window: false,
            frame_builder_slave_id: 1, frame_builder_fc: ModbusFunctionCode::ReadHoldingRegisters, frame_builder_addr: "0".into(), frame_builder_value: "1".into(), frame_builder_hex: String::new(), frame_builder_error: None,
            show_data_logger_window: false, data_logger_recording: false, data_logger_path: String::new(), data_logger_buffered: 0,
            show_can_window: false, show_i2c_spi_window: false, show_scope_window: false, show_flasher_window: false,
            show_register_editor_window: false, show_plugin_window: false,
            checksum_mode: ChecksumMode::None, checksum_input: String::new(),
            can_capturing: false, can_frames: Vec::new(), can_filter_id: String::new(), can_stats: CanStats::default(),
            can_show_stats: false, can_tx_id: String::new(), can_tx_data: String::new(),
            i2c_mode: I2cMode::I2C, i2c_address: "68".into(), i2c_register: "00".into(), i2c_data: String::new(), i2c_result: String::new(),
            scope_capturing: false, scope_data: Vec::new(), scope_timebase_ms: 100.0,
            flasher_mcu: McuType::Stm32, flasher_file: String::new(), flasher_progress: 0.0, flasher_log: VecDeque::new(),
            reg_map: Vec::new(), reg_selected: None, reg_alarm_enabled: false, reg_alarm_threshold: "100".into(),
            plugins: Vec::new(),
            help_content_zh: load_help_file("help_zh.md"),
            help_content_en: load_help_file("help_en.md"),
            auto_detect_receiver: None,
            modbus_async_receiver: None,
            plc_async_receiver: None,
            i2c_async_receiver: None,
            flasher_async_receiver: None,
            terminal_async_receiver: None,
        }
    }

    pub fn refresh_ports(&mut self) {
        self.ports = SerialPort::list_ports().unwrap_or_default();
    }

    pub fn add_terminal_line(&mut self, direction: Direction, content: String, is_hex: bool) {
        let line = TerminalLine {
            timestamp: chrono::Utc::now().timestamp_millis(),
            direction,
            content,
            is_hex,
        };
        self.terminal_buffer.push_back(line);
        if self.terminal_buffer.len() > 1000 {
            self.terminal_buffer.pop_front();
        }
    }

    pub fn add_log_entry(&mut self, level: LogLevel, message: &str) {
        let entry = LogEntry {
            timestamp: chrono::Utc::now().timestamp_millis(),
            level,
            message: message.to_string(),
        };
        self.log_entries.push(entry);
        if self.log_entries.len() > 500 {
            self.log_entries.remove(0);
        }
    }

    pub fn add_chart_data(&mut self, value: f64) {
        self.chart_data.push(value);
        if self.chart_data.len() > 200 {
            self.chart_data.remove(0);
        }
    }
}
