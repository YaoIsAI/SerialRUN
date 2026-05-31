use serialrun_core::config::SerialConfig;
use serialrun_core::protocol::ModbusFunction;
use serialrun_core::{SerialPort, SerialPortInfo};
use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;

/// Persisted user preferences (theme, language, serial config, etc.)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UserPrefs {
    // Theme & Language
    pub theme: Theme,
    pub language: Language,
    // Serial config
    #[serde(default = "default_baud_rate")]
    pub baud_rate: u32,
    #[serde(default = "default_data_bits")]
    pub data_bits: String,
    #[serde(default = "default_stop_bits")]
    pub stop_bits: String,
    #[serde(default = "default_parity")]
    pub parity: String,
    #[serde(default = "default_flow_control")]
    pub flow_control: String,
    // Terminal display
    #[serde(default)]
    pub hex_mode: bool,
    #[serde(default = "default_true")]
    pub show_timestamp: bool,
    #[serde(default = "default_true")]
    pub auto_scroll: bool,
    #[serde(default)]
    pub keep_input: bool,
    #[serde(default)]
    pub line_ending: String,
    #[serde(default)]
    pub terminal_checksum_mode: String,
    // Auto-send
    #[serde(default)]
    pub auto_send_enabled: bool,
    #[serde(default = "default_auto_send_interval")]
    pub auto_send_interval_ms: u64,
    // DTR/RTS
    #[serde(default = "default_true")]
    pub dtr: bool,
    #[serde(default)]
    pub rts: bool,
    // Auto-reply
    #[serde(default)]
    pub auto_reply_enabled: bool,
    #[serde(default)]
    pub auto_reply_pattern: String,
    #[serde(default)]
    pub auto_reply_response: String,
    // MCP
    #[serde(default = "default_true")]
    pub mcp_enabled: bool,
    #[serde(default = "default_mcp_port")]
    pub mcp_port: u16,
    #[serde(default)]
    pub mcp_bind_lan: bool,
    // RX aggregation
    #[serde(default = "default_rx_aggregate_ms")]
    pub rx_aggregate_ms: u64,
    #[serde(default = "default_true")]
    pub rx_auto_aggregate: bool,
}

fn default_baud_rate() -> u32 { 115200 }
fn default_data_bits() -> String { "8".into() }
fn default_stop_bits() -> String { "1".into() }
fn default_parity() -> String { "None".into() }
fn default_flow_control() -> String { "None".into() }
fn default_true() -> bool { true }
fn default_auto_send_interval() -> u64 { 1000 }
fn default_mcp_port() -> u16 { 9527 }
fn default_rx_aggregate_ms() -> u64 { 150 }

impl Default for UserPrefs {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            language: Language::Chinese,
            baud_rate: 115200,
            data_bits: "8".into(),
            stop_bits: "1".into(),
            parity: "None".into(),
            flow_control: "None".into(),
            hex_mode: false,
            show_timestamp: true,
            auto_scroll: true,
            keep_input: false,
            line_ending: "None".into(),
            terminal_checksum_mode: "None".into(),
            auto_send_enabled: false,
            auto_send_interval_ms: 1000,
            dtr: true,
            rts: false,
            auto_reply_enabled: false,
            auto_reply_pattern: String::new(),
            auto_reply_response: String::new(),
            mcp_enabled: true,
            mcp_port: 9527,
            mcp_bind_lan: false,
            rx_aggregate_ms: 150,
            rx_auto_aggregate: true,
        }
    }
}

impl UserPrefs {
    /// Build UserPrefs from current AppState for saving
    pub fn from_state(state: &AppState) -> Self {
        Self {
            theme: state.theme,
            language: state.language,
            baud_rate: state.config.baud_rate,
            data_bits: match state.config.data_bits {
                serialrun_core::config::DataBits::Five => "5".into(),
                serialrun_core::config::DataBits::Six => "6".into(),
                serialrun_core::config::DataBits::Seven => "7".into(),
                serialrun_core::config::DataBits::Eight => "8".into(),
            },
            stop_bits: match state.config.stop_bits {
                serialrun_core::config::StopBits::Two => "2".into(),
                _ => "1".into(),
            },
            parity: match state.config.parity {
                serialrun_core::config::Parity::Odd => "Odd".into(),
                serialrun_core::config::Parity::Even => "Even".into(),
                _ => "None".into(),
            },
            flow_control: match state.config.flow_control {
                serialrun_core::config::FlowControl::Software => "Software".into(),
                serialrun_core::config::FlowControl::Hardware => "Hardware".into(),
                _ => "None".into(),
            },
            hex_mode: state.hex_mode,
            show_timestamp: state.show_timestamp,
            auto_scroll: state.auto_scroll,
            keep_input: state.keep_input,
            line_ending: match state.line_ending {
                LineEnding::CR => "CR".into(),
                LineEnding::LF => "LF".into(),
                LineEnding::CRLF => "CRLF".into(),
                _ => "None".into(),
            },
            terminal_checksum_mode: match state.terminal_checksum_mode {
                ChecksumMode::Crc16Modbus => "Crc16Modbus".into(),
                ChecksumMode::Crc16Ccitt => "Crc16Ccitt".into(),
                ChecksumMode::Crc16Xmodem => "Crc16Xmodem".into(),
                ChecksumMode::Crc32 => "Crc32".into(),
                ChecksumMode::Lrc => "Lrc".into(),
                ChecksumMode::Checksum8 => "Checksum8".into(),
                ChecksumMode::Checksum16 => "Checksum16".into(),
                _ => "None".into(),
            },
            auto_send_enabled: state.auto_send_enabled,
            auto_send_interval_ms: state.auto_send_interval_ms,
            dtr: state.dtr,
            rts: state.rts,
            auto_reply_enabled: state.auto_reply_enabled,
            auto_reply_pattern: state.auto_reply_pattern.clone(),
            auto_reply_response: state.auto_reply_response.clone(),
            mcp_enabled: state.mcp_enabled,
            mcp_port: state.mcp_port,
            mcp_bind_lan: state.mcp_bind_lan,
            rx_aggregate_ms: state.rx_aggregate_ms,
            rx_auto_aggregate: state.rx_auto_aggregate,
        }
    }

    /// Apply persisted preferences to AppState (inverse of from_state).
    pub fn apply_to(&self, state: &mut AppState) {
        state.language = self.language;
        state.theme = self.theme;
        state.config.baud_rate = self.baud_rate;
        state.baud_rate_text = self.baud_rate.to_string();
        state.config.data_bits = match self.data_bits.as_str() {
            "5" => serialrun_core::config::DataBits::Five,
            "6" => serialrun_core::config::DataBits::Six,
            "7" => serialrun_core::config::DataBits::Seven,
            _ => serialrun_core::config::DataBits::Eight,
        };
        state.config.stop_bits = match self.stop_bits.as_str() {
            "2" => serialrun_core::config::StopBits::Two,
            _ => serialrun_core::config::StopBits::One,
        };
        state.config.parity = match self.parity.as_str() {
            "Odd" => serialrun_core::config::Parity::Odd,
            "Even" => serialrun_core::config::Parity::Even,
            _ => serialrun_core::config::Parity::None,
        };
        state.config.flow_control = match self.flow_control.as_str() {
            "Software" => serialrun_core::config::FlowControl::Software,
            "Hardware" => serialrun_core::config::FlowControl::Hardware,
            _ => serialrun_core::config::FlowControl::None,
        };
        state.hex_mode = self.hex_mode;
        state.show_timestamp = self.show_timestamp;
        state.auto_scroll = self.auto_scroll;
        state.keep_input = self.keep_input;
        state.line_ending = match self.line_ending.as_str() {
            "CR" => LineEnding::CR,
            "LF" => LineEnding::LF,
            "CRLF" => LineEnding::CRLF,
            _ => LineEnding::None,
        };
        state.terminal_checksum_mode = match self.terminal_checksum_mode.as_str() {
            "Crc16Modbus" => ChecksumMode::Crc16Modbus,
            "Crc16Ccitt" => ChecksumMode::Crc16Ccitt,
            "Crc16Xmodem" => ChecksumMode::Crc16Xmodem,
            "Crc32" => ChecksumMode::Crc32,
            "Lrc" => ChecksumMode::Lrc,
            "Checksum8" => ChecksumMode::Checksum8,
            "Checksum16" => ChecksumMode::Checksum16,
            _ => ChecksumMode::None,
        };
        state.auto_send_enabled = self.auto_send_enabled;
        state.auto_send_interval_ms = self.auto_send_interval_ms;
        state.dtr = self.dtr;
        state.rts = self.rts;
        state.auto_reply_enabled = self.auto_reply_enabled;
        state.auto_reply_pattern = self.auto_reply_pattern.clone();
        state.auto_reply_response = self.auto_reply_response.clone();
        state.mcp_enabled = self.mcp_enabled;
        state.mcp_port = self.mcp_port;
        state.mcp_bind_lan = self.mcp_bind_lan;
        state.rx_aggregate_ms = self.rx_aggregate_ms;
        state.rx_auto_aggregate = self.rx_auto_aggregate;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Language {
    English,
    Chinese,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineEnding {
    None,
    CR,
    LF,
    CRLF,
}
impl LineEnding {
    pub fn suffix(&self) -> &'static [u8] {
        match self {
            LineEnding::None => b"",
            LineEnding::CR => b"\r",
            LineEnding::LF => b"\n",
            LineEnding::CRLF => b"\r\n",
        }
    }
    pub fn label(&self, lang: Language) -> &'static str {
        match self {
            LineEnding::None => if lang == Language::Chinese { "无" } else { "None" },
            LineEnding::CR => "CR (\\r)",
            LineEnding::LF => "LF (\\n)",
            LineEnding::CRLF => "CRLF (\\r\\n)",
        }
    }
}

pub struct T;

impl T {
    pub fn app_title(lang: Language) -> &'static str {
        match lang {
            Language::English => "SerialRUN - Serial Port Assistant",
            Language::Chinese => "SerialRUN - 串口助手",
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
    // Bridge
    pub fn bridge(l: Language) -> &'static str { match l { Language::English => "TCP/RTU Bridge", Language::Chinese => "TCP/RTU 桥接" } }
    pub fn tcp_port(l: Language) -> &'static str { match l { Language::English => "TCP Port", Language::Chinese => "TCP 端口" } }
    pub fn start_bridge(l: Language) -> &'static str { match l { Language::English => "Start Bridge", Language::Chinese => "启动桥接" } }
    pub fn stop_bridge(l: Language) -> &'static str { match l { Language::English => "Stop Bridge", Language::Chinese => "停止桥接" } }
    pub fn timeout_ms(l: Language) -> &'static str { match l { Language::English => "Timeout (ms)", Language::Chinese => "超时 (ms)" } }
    pub fn bridge_log(l: Language) -> &'static str { match l { Language::English => "Bridge Log", Language::Chinese => "桥接日志" } }
    // Simulator
    pub fn simulator(l: Language) -> &'static str { match l { Language::English => "HMI Simulator", Language::Chinese => "HMI 模拟器" } }
    pub fn sim_mode(l: Language) -> &'static str { match l { Language::English => "Mode", Language::Chinese => "模式" } }
    pub fn start_sim(l: Language) -> &'static str { match l { Language::English => "Start Simulator", Language::Chinese => "启动模拟器" } }
    pub fn stop_sim(l: Language) -> &'static str { match l { Language::English => "Stop Simulator", Language::Chinese => "停止模拟器" } }
    pub fn holding_registers(l: Language) -> &'static str { match l { Language::English => "Holding Registers", Language::Chinese => "保持寄存器" } }
    pub fn coils(l: Language) -> &'static str { match l { Language::English => "Coils", Language::Chinese => "线圈" } }
    pub fn set_value(l: Language) -> &'static str { match l { Language::English => "Set", Language::Chinese => "设置" } }
    pub fn sim_log(l: Language) -> &'static str { match l { Language::English => "Simulator Log", Language::Chinese => "模拟器日志" } }
    // MCP
    pub fn mcp_server(l: Language) -> &'static str { match l { Language::English => "MCP Server", Language::Chinese => "MCP 服务器" } }
    pub fn mcp_port(l: Language) -> &'static str { match l { Language::English => "Port", Language::Chinese => "端口" } }
    pub fn mcp_bind(l: Language) -> &'static str { match l { Language::English => "Bind Address", Language::Chinese => "绑定地址" } }
    pub fn mcp_localhost(l: Language) -> &'static str { match l { Language::English => "Localhost only", Language::Chinese => "仅本机" } }
    pub fn mcp_lan(l: Language) -> &'static str { match l { Language::English => "All interfaces (LAN)", Language::Chinese => "所有接口（局域网）" } }
    pub fn mcp_enable(l: Language) -> &'static str { match l { Language::English => "Enable MCP Server", Language::Chinese => "启用 MCP 服务器" } }
    pub fn mcp_status(l: Language) -> &'static str { match l { Language::English => "Status", Language::Chinese => "状态" } }
    pub fn mcp_running(l: Language) -> &'static str { match l { Language::English => "Running", Language::Chinese => "运行中" } }
    pub fn mcp_stopped(l: Language) -> &'static str { match l { Language::English => "Stopped", Language::Chinese => "已停止" } }
    pub fn mcp_warning(l: Language) -> &'static str { match l { Language::English => "LAN mode: anyone on the network can control serial ports. Use with caution.", Language::Chinese => "局域网模式：网络中任何人都可以控制串口端口，请谨慎使用。" } }

    // ── Terminal ──
    pub fn crc_label(l: Language) -> &'static str { match l { Language::English => "CRC:", Language::Chinese => "CRC:" } }
    pub fn line_ending(l: Language) -> &'static str { match l { Language::English => "End:", Language::Chinese => "行尾:" } }
    pub fn auto_send(l: Language) -> &'static str { match l { Language::English => "Auto", Language::Chinese => "自动发送" } }
    pub fn stop_auto(l: Language) -> &'static str { match l { Language::English => "Stop Auto", Language::Chinese => "停止发送" } }
    pub fn save_btn(l: Language) -> &'static str { match l { Language::English => "Save", Language::Chinese => "保存" } }

    // ── Checksum ──
    pub fn algorithm(l: Language) -> &'static str { match l { Language::English => "Algorithm", Language::Chinese => "算法" } }
    pub fn result_label(l: Language) -> &'static str { match l { Language::English => "Result", Language::Chinese => "结果" } }

    // ── I2C/SPI ──
    pub fn address_hex(l: Language) -> &'static str { match l { Language::English => "Address (hex):", Language::Chinese => "地址 (hex):" } }
    pub fn register_hex(l: Language) -> &'static str { match l { Language::English => "Register (hex):", Language::Chinese => "寄存器 (hex):" } }
    pub fn data_hex(l: Language) -> &'static str { match l { Language::English => "Data (hex):", Language::Chinese => "数据 (hex):" } }
    pub fn read_btn(l: Language) -> &'static str { match l { Language::English => "Read", Language::Chinese => "读取" } }
    pub fn write_btn(l: Language) -> &'static str { match l { Language::English => "Write", Language::Chinese => "写入" } }
    pub fn transfer_btn(l: Language) -> &'static str { match l { Language::English => "Transfer", Language::Chinese => "传输" } }
    pub fn mosi(l: Language) -> &'static str { match l { Language::English => "MOSI (hex):", Language::Chinese => "MOSI (hex):" } }
    pub fn result_colon(l: Language) -> &'static str { match l { Language::English => "Result:", Language::Chinese => "结果:" } }

    // ── Flasher ──
    pub fn serial_flasher(l: Language) -> &'static str { match l { Language::English => "Serial Flasher", Language::Chinese => "串口烧录器" } }
    pub fn mcu_label(l: Language) -> &'static str { match l { Language::English => "MCU:", Language::Chinese => "MCU:" } }
    pub fn firmware(l: Language) -> &'static str { match l { Language::English => "Firmware:", Language::Chinese => "固件:" } }

    // ── Register Editor ──
    pub fn register_map_editor(l: Language) -> &'static str { match l { Language::English => "Register Map Editor", Language::Chinese => "寄存器映射编辑" } }
    pub fn add_btn(l: Language) -> &'static str { match l { Language::English => "Add", Language::Chinese => "添加" } }
    pub fn alarm(l: Language) -> &'static str { match l { Language::English => "Alarm", Language::Chinese => "报警" } }
    pub fn threshold(l: Language) -> &'static str { match l { Language::English => "Threshold:", Language::Chinese => "阈值:" } }

    // ── Plugin ──
    pub fn refresh_btn(l: Language) -> &'static str { match l { Language::English => "Refresh", Language::Chinese => "刷新" } }
    pub fn no_plugins(l: Language) -> &'static str { match l { Language::English => "No plugins found.", Language::Chinese => "未找到插件。" } }
    pub fn plugin_hint(l: Language) -> &'static str { match l { Language::English => "Place .dylib/.so/.dll plugins in the plugins/ directory next to the executable.", Language::Chinese => "将 .dylib/.so/.dll 插件放在可执行文件旁的 plugins/ 目录中。" } }

    // ── Frame Builder ──
    pub fn build_btn(l: Language) -> &'static str { match l { Language::English => "Build", Language::Chinese => "构建" } }

    // ── File Transfer ──
    pub fn done(l: Language) -> &'static str { match l { Language::English => "Done", Language::Chinese => "完成" } }
    pub fn sending(l: Language) -> &'static str { match l { Language::English => "Sending...", Language::Chinese => "发送中..." } }
    pub fn receiving(l: Language) -> &'static str { match l { Language::English => "Receiving...", Language::Chinese => "接收中..." } }
    pub fn ready(l: Language) -> &'static str { match l { Language::English => "Ready", Language::Chinese => "就绪" } }

    // ── Data Logger ──
    pub fn log_label(l: Language) -> &'static str { match l { Language::English => "Log:", Language::Chinese => "日志:" } }
    pub fn data_rate(l: Language) -> &'static str { match l { Language::English => "Data Rate (bytes/s)", Language::Chinese => "数据速率 (bytes/s)" } }

    // ── Scope ──
    pub fn timebase(l: Language) -> &'static str { match l { Language::English => "Timebase (ms):", Language::Chinese => "时基 (ms):" } }

    // ── CAN ──
    pub fn statistics(l: Language) -> &'static str { match l { Language::English => "Statistics", Language::Chinese => "统计" } }
    pub fn frames_label(l: Language) -> &'static str { match l { Language::English => "Frames", Language::Chinese => "帧" } }
    pub fn filter_label(l: Language) -> &'static str { match l { Language::English => "Filter:", Language::Chinese => "过滤:" } }
    pub fn tx_id(l: Language) -> &'static str { match l { Language::English => "TX ID:", Language::Chinese => "发送 ID:" } }
    pub fn data_label(l: Language) -> &'static str { match l { Language::English => "Data:", Language::Chinese => "数据:" } }
    pub fn bus_load(l: Language) -> &'static str { match l { Language::English => "Bus Load:", Language::Chinese => "总线负载:" } }
    pub fn max_id(l: Language) -> &'static str { match l { Language::English => "Max ID:", Language::Chinese => "最大 ID:" } }
    pub fn errors(l: Language) -> &'static str { match l { Language::English => "Errors:", Language::Chinese => "错误:" } }
    pub fn id_count(l: Language) -> &'static str { match l { Language::English => "IDs:", Language::Chinese => "ID数:" } }

    // ── PLC ──
    pub fn once_btn(l: Language) -> &'static str { match l { Language::English => "Once", Language::Chinese => "单次" } }
    pub fn poll_btn(l: Language) -> &'static str { match l { Language::English => "Poll", Language::Chinese => "轮询" } }
    pub fn stop_btn(l: Language) -> &'static str { match l { Language::English => "Stop", Language::Chinese => "停止" } }

    // ── Help / Copy ──
    pub fn copy_mcp_guide(l: Language) -> &'static str { match l { Language::English => "Copy MCP Guide (for AI assistant)", Language::Chinese => "复制 MCP 说明（发给 AI 助手）" } }
    pub fn copied(l: Language) -> &'static str { match l { Language::English => "Copied!", Language::Chinese => "已复制!" } }
    pub fn copy_hint(l: Language) -> &'static str { match l { Language::English => "Click to copy the full MCP guide. Paste into your AI assistant for serial port control.", Language::Chinese => "点击复制完整 MCP 说明，粘贴到 AI 助手中即可控制串口。" } }

    // ── Status messages ──
    pub fn listening_hint(l: Language) -> &'static str { match l { Language::English => "Waiting for Modbus TCP client connections.", Language::Chinese => "等待 Modbus TCP 客户端连接。" } }
    pub fn bridge_hint(l: Language) -> &'static str { match l { Language::English => "TCP clients can now connect to relay serial data.", Language::Chinese => "TCP 客户端可连接此地址进行串口中转。" } }
    pub fn keep_input(l: Language) -> &'static str { match l { Language::English => "Keep input", Language::Chinese => "保留输入" } }
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
    pub response_timeout_ms: u64,
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
pub enum ChecksumMode { None, Crc16Modbus, Crc16Ccitt, Crc16Xmodem, Crc32, Lrc, Checksum8, Checksum16 }
impl ChecksumMode {
    pub fn label(&self, l: Language) -> &'static str { match (self, l) { (Self::None, Language::English)=>"None", (Self::None, Language::Chinese)=>"无", (Self::Crc16Modbus, _)=>"CRC16/MODBUS", (Self::Crc16Ccitt, _)=>"CRC16/CCITT", (Self::Crc16Xmodem, _)=>"CRC16/XMODEM", (Self::Crc32, _)=>"CRC32", (Self::Lrc, _)=>"LRC", (Self::Checksum8, _)=>"SUM8", (Self::Checksum16, _)=>"SUM16" } }
    pub fn all() -> &'static [Self] { &[Self::None, Self::Crc16Modbus, Self::Crc16Ccitt, Self::Crc16Xmodem, Self::Crc32, Self::Lrc, Self::Checksum8, Self::Checksum16] }
    pub fn append_checksum(&self, data: &[u8]) -> Vec<u8> {
        let mut r = data.to_vec();
        match self { Self::None => return data.to_vec(), Self::Crc16Modbus => { let c = serialrun_core::checksum::crc16_modbus(data); r.extend_from_slice(&c.to_le_bytes()); } Self::Crc16Ccitt => { let c = serialrun_core::checksum::crc16_ccitt(data); r.extend_from_slice(&c.to_be_bytes()); } Self::Crc16Xmodem => { let c = serialrun_core::checksum::crc16_xmodem(data); r.extend_from_slice(&c.to_be_bytes()); } Self::Crc32 => { let c = serialrun_core::checksum::crc32(data); r.extend_from_slice(&c.to_le_bytes()); } Self::Lrc => r.push(serialrun_core::checksum::lrc(data)), Self::Checksum8 => r.push(serialrun_core::checksum::checksum8(data)), Self::Checksum16 => { let c = serialrun_core::checksum::checksum16(data); r.extend_from_slice(&c.to_be_bytes()); } }
        r
    }
}

// ── Bridge types ──
#[derive(Clone)]
pub struct BridgeState {
    pub running: bool,
    pub tcp_port: u16,
    pub serial_port_name: String,
    pub baud_rate: u32,
    pub timeout_ms: u64,
    pub log: VecDeque<BridgeLogEntry>,
    pub status_msg: Option<String>,
}

#[derive(Clone)]
pub struct BridgeLogEntry { pub timestamp: i64, pub client_addr: String, pub direction: String, pub request_hex: String, pub response_hex: String, pub success: bool }

// ── Simulator types ──
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SimMode { TcpServer, RtuSlave }
impl SimMode { pub fn label(&self, l: Language) -> &'static str { match (self, l) { (Self::TcpServer, Language::English)=>"TCP Server", (Self::TcpServer, Language::Chinese)=>"TCP 服务器", (Self::RtuSlave, Language::English)=>"RTU Slave", (Self::RtuSlave, Language::Chinese)=>"RTU 从站" } } }

#[derive(Clone)]
pub struct SimulatorState {
    pub running: bool,
    pub mode: SimMode,
    pub tcp_port: u16,
    pub serial_port_name: String,
    pub baud_rate: u32,
    pub slave_id: u8,
    pub holding_registers: HashMap<u16, u16>,
    pub input_registers: HashMap<u16, u16>,
    pub coils: HashMap<u16, bool>,
    pub discrete_inputs: HashMap<u16, bool>,
    pub edit_addr: String,
    pub edit_value: String,
    pub log: VecDeque<SimulatorLogEntry>,
    pub status_msg: Option<String>,
}

#[derive(Clone)]
pub struct SimulatorLogEntry { pub timestamp: i64, pub direction: String, pub hex: String, pub decoded: String, pub success: bool }

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

// ── Auto-detect types ──

pub enum AutoDetectMsg {
    Progress(u32),
    Done(Option<u32>),
}

// ── Plugin types ──
#[derive(Clone)]
pub struct PluginInfo { pub name: String, pub version: String, pub author: String, pub loaded: bool }

pub struct AppState {
    pub ports: Vec<SerialPortInfo>,
    pub selected_port: Option<String>,
    pub config: SerialConfig,
    pub baud_rate_text: String,
    pub is_connected: bool,
    pub terminal_buffer: VecDeque<TerminalLine>,
    pub input_buffer: String,
    pub hex_mode: bool,
    pub auto_scroll: bool,
    pub scroll_to_bottom_pending: bool,
    pub terminal_dirty: bool,
    pub logs_dirty: bool,
    pub terminal_last_save: i64,
    pub logs_last_save: i64,
    pub show_timestamp: bool,
    // SSCOM-like features
    pub line_ending: LineEnding,
    pub dtr: bool,
    pub rts: bool,
    pub auto_send_enabled: bool,
    pub auto_send_interval_ms: u64,
    pub auto_send_last_time: i64,
    pub keep_input: bool,
    pub terminal_checksum_mode: ChecksumMode,
    pub show_chart_window: bool,
    pub show_log_window: bool,
    pub rx_count: u64,
    pub tx_count: u64,
    pub rx_rate: f64,
    pub tx_rate: f64,
    pub rate_last_check: i64,
    pub rate_last_rx: u64,
    pub rate_last_tx: u64,
    pub chart_data: Vec<f64>,
    pub log_entries: Vec<LogEntry>,
    pub auto_reply_enabled: bool,
    pub auto_reply_pattern: String,
    pub auto_reply_response: String,
    pub recording: bool,
    pub recording_last_time: i64,
    pub script_commands: Vec<ScriptCommand>,
    // Replay
    pub replay_running: bool,
    pub replay_commands: Vec<ScriptCommand>,
    pub replay_index: usize,
    pub replay_start_time: i64,
    pub language: Language,
    pub theme: Theme,
    pub show_help: bool,
    pub show_modbus_window: bool,
    pub modbus: ModbusState,
    pub show_plc_window: bool,
    pub plc: PlcState,
    pub show_checksum_window: bool,
    pub show_file_transfer_window: bool,
    pub file_transfer_protocol: serialrun_core::file_transfer::TransferProtocol,
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
    pub show_bridge_window: bool,
    pub show_simulator_window: bool,
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
    // Copy button state (help panel)
    pub copied: bool,
    pub copied_time: i64,
    pub rx_aggregate_ms: u64,
    pub rx_auto_aggregate: bool,
    // Auto-detect
    pub auto_detect_receiver: Option<std::sync::mpsc::Receiver<AutoDetectMsg>>,
    pub auto_detect_running: bool,
    pub auto_detect_progress: Option<u32>,
    // Port owner (persistent reader/writer thread)
    pub port_owner: Option<crate::port_owner::PortOwnerHandle>,
    // Global error notification
    pub global_error: Option<String>,
    pub global_error_time: i64,
    pub warning_history: VecDeque<WarningEntry>,
    pub show_warning_popup: bool,
    // Async operation states
    pub modbus_async_receiver: Option<std::sync::mpsc::Receiver<Result<Vec<u8>, String>>>,
    pub plc_async_receiver: Option<std::sync::mpsc::Receiver<Result<Vec<(u16, std::result::Result<Vec<u8>, String>)>, String>>>,
    pub i2c_async_receiver: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    pub flasher_async_receiver: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    pub terminal_async_receiver: Option<std::sync::mpsc::Receiver<Result<Vec<u8>, String>>>,
    // Async write operations
    pub can_tx_async: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
    pub plc_write_async: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
    pub modbus_monitor_async: Option<std::sync::mpsc::Receiver<Result<Vec<u8>, String>>>,
    pub auto_reply_async: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
    pub fb_write_async: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
    // Persistent readers (continuous capture)
    pub can_reader: Option<crate::async_utils::PersistentReader<Vec<CanFrameData>>>,
    pub can_write_tx: Option<std::sync::mpsc::Sender<Vec<u8>>>,
    pub scope_reader: Option<crate::async_utils::PersistentReader<Vec<ScopeDataPoint>>>,
    pub scope_write_tx: Option<std::sync::mpsc::Sender<Vec<u8>>>,
    // File transfer async
    pub file_transfer_thread: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
    pub file_transfer_progress_rx: Option<std::sync::mpsc::Receiver<(u64, u64)>>,
    // Bridge & Simulator
    pub bridge: BridgeState,
    pub simulator: SimulatorState,
    pub bridge_stop: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    pub bridge_log_rx: Option<std::sync::mpsc::Receiver<serialrun_core::protocol::BridgeLogEntry>>,
    pub bridge_err_rx: Option<std::sync::mpsc::Receiver<String>>,
    pub sim_stop: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    pub sim_log_rx: Option<std::sync::mpsc::Receiver<serialrun_core::protocol::SimulatorLogEntry>>,
    pub sim_err_rx: Option<std::sync::mpsc::Receiver<String>>,
    pub sim_registers: Option<std::sync::Arc<std::sync::Mutex<serialrun_core::protocol::SimulatorState>>>,
    // MCP server config
    pub mcp_enabled: bool,
    pub mcp_port: u16,
    pub mcp_bind_lan: bool,
    pub mcp_running: bool,
    pub mcp_status: String,
    pub mcp_cmd_tx: Option<mpsc::Sender<crate::mcp_server::McpCommand>>,
    // MCP access log (for GUI display)
    pub mcp_access_log: VecDeque<crate::mcp_server::McpAccessLogEntry>,
    pub show_mcp_log_popup: bool,
    // Log viewer search
    pub log_search: String,
    pub log_level_filter: Option<LogLevel>,
    // AI connection status (from MCP independent connect)
    pub ai_connected: bool,
    pub ai_port_name: String,
    pub ai_baud_rate: u32,
    pub ai_tx_count: u64,
    pub ai_rx_count: u64,
    /// Who initiated the current connection: empty = GUI, "MCP" = AI
    pub connected_by: String,
    /// MCP connect operation in progress — GUI connect button disabled
    pub mcp_connect_in_progress: bool,
    // Device identification for traceability
    pub device_id: String,
    pub device_model: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct TerminalLine {
    pub timestamp: i64,
    pub direction: Direction,
    pub content: String,
    pub is_hex: bool,
    /// Source of the data: empty for manual UI, "MCP" for AI-initiated
    #[serde(default)]
    pub source: String,
}

#[derive(Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    pub timestamp: i64,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct WarningEntry {
    pub timestamp: i64,
    pub message: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ScriptCommand {
    pub delay_ms: u64,
    pub action: ScriptAction,
    pub data: Option<String>,
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ScriptAction {
    Send,
    Wait,
}

impl ScriptCommand {
    /// Serialize to text line: "SEND data" or "WAIT 500"
    pub fn to_text_line(&self) -> String {
        match self.action {
            ScriptAction::Send => {
                let data = self.data.as_deref().unwrap_or("");
                format!("SEND {}", data)
            }
            ScriptAction::Wait => format!("WAIT {}", self.delay_ms),
        }
    }

    /// Parse from text line
    pub fn from_text_line(line: &str) -> Option<Self> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        if let Some(data) = line.strip_prefix("SEND ") {
            Some(ScriptCommand {
                delay_ms: 0,
                action: ScriptAction::Send,
                data: Some(data.to_string()),
            })
        } else if let Some(ms_str) = line.strip_prefix("WAIT ") {
            let delay_ms = ms_str.parse::<u64>().unwrap_or(100);
            Some(ScriptCommand {
                delay_ms,
                action: ScriptAction::Wait,
                data: None,
            })
        } else {
            None
        }
    }
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
        let mut state = Self {
            ports: Vec::new(),
            selected_port: None,
            config: SerialConfig::default(),
            baud_rate_text: "115200".into(),
            is_connected: false,
            terminal_buffer: VecDeque::new(),
            input_buffer: String::new(),
            hex_mode: false,
            auto_scroll: true,
            scroll_to_bottom_pending: false,
            terminal_dirty: false,
            logs_dirty: false,
            terminal_last_save: 0,
            logs_last_save: 0,
            show_timestamp: true,
            line_ending: LineEnding::None,
            dtr: true,
            rts: false,
            auto_send_enabled: false,
            auto_send_interval_ms: 1000,
            auto_send_last_time: chrono::Utc::now().timestamp_millis(),
            keep_input: false,
            terminal_checksum_mode: ChecksumMode::None,
            show_chart_window: false,
            show_log_window: false,
            rx_count: 0,
            tx_count: 0,
            rx_rate: 0.0,
            tx_rate: 0.0,
            rate_last_check: chrono::Utc::now().timestamp_millis(),
            rate_last_rx: 0,
            rate_last_tx: 0,
            chart_data: Vec::new(),
            log_entries: Vec::new(),
            auto_reply_enabled: false,
            auto_reply_pattern: String::new(),
            auto_reply_response: String::new(),
            recording: false,
            recording_last_time: 0,
            script_commands: Vec::new(),
            replay_running: false,
            replay_commands: Vec::new(),
            replay_index: 0,
            replay_start_time: 0,
            language: Language::Chinese,
            theme: Theme::Dark,
            show_help: false,
            show_modbus_window: false,
            modbus: ModbusState { slave_id: 1, function_code: ModbusFunctionCode::ReadHoldingRegisters, start_addr: "0".into(), quantity: "10".into(), write_value: String::new(), last_request_hex: String::new(), last_response_hex: String::new(), last_error: None, monitor_entries: Vec::new(), monitor_polling: false, monitor_interval_ms: 1000, last_poll_time: 0, monitor_slave_id: 1, monitor_start_addr: "0".into(), monitor_quantity: "10".into(), monitor_function: ModbusFunctionCode::ReadHoldingRegisters, frame_log: VecDeque::new(), response_timeout_ms: 500 },
            show_plc_window: false,
            plc: PlcState {
                selected_brand: PlcBrand::Siemens, selected_model: None, slave_id: 1,
                register_values: HashMap::new(), custom_registers: Vec::new(),
                polling: false, poll_interval_ms: 1000, last_poll_time: 0,
                selected_register: None, write_value: String::new(), plc_log: VecDeque::new(),
            },
            show_checksum_window: false,
            show_file_transfer_window: false,
            file_transfer_protocol: serialrun_core::file_transfer::TransferProtocol::Xmodem,
            file_transfer_sending: false, file_transfer_receiving: false, file_transfer_done: false, file_transfer_error: None, file_transfer_progress: 0.0,
            show_frame_builder_window: false,
            frame_builder_slave_id: 1, frame_builder_fc: ModbusFunctionCode::ReadHoldingRegisters, frame_builder_addr: "0".into(), frame_builder_value: "1".into(), frame_builder_hex: String::new(), frame_builder_error: None,
            show_data_logger_window: false, data_logger_recording: false, data_logger_path: String::new(), data_logger_buffered: 0,
            show_can_window: false, show_i2c_spi_window: false, show_scope_window: false, show_flasher_window: false,
            show_register_editor_window: false, show_plugin_window: false,
            show_bridge_window: false, show_simulator_window: false,
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
            copied: false,
            copied_time: 0,
            rx_aggregate_ms: 150,
            rx_auto_aggregate: true,
            auto_detect_receiver: None,
            auto_detect_running: false,
            auto_detect_progress: None,
            port_owner: None,
            global_error: None,
            global_error_time: 0,
            warning_history: VecDeque::new(),
            show_warning_popup: false,
            modbus_async_receiver: None,
            plc_async_receiver: None,
            i2c_async_receiver: None,
            flasher_async_receiver: None,
            terminal_async_receiver: None,
            can_tx_async: None,
            plc_write_async: None,
            modbus_monitor_async: None,
            auto_reply_async: None,
            fb_write_async: None,
            can_reader: None,
            can_write_tx: None,
            scope_reader: None,
            scope_write_tx: None,
            file_transfer_thread: None,
            file_transfer_progress_rx: None,
            bridge: BridgeState {
                running: false, tcp_port: 502, serial_port_name: String::new(), baud_rate: 9600, timeout_ms: 500,
                log: VecDeque::new(), status_msg: None,
            },
            simulator: SimulatorState {
                running: false, mode: SimMode::TcpServer, tcp_port: 502, serial_port_name: String::new(), baud_rate: 9600,
                slave_id: 1, holding_registers: (0..10).map(|i| (i, 0u16)).collect(),
                input_registers: HashMap::new(), coils: (0..16).map(|i| (i, false)).collect(),
                discrete_inputs: HashMap::new(), edit_addr: "0".into(), edit_value: "0".into(),
                log: VecDeque::new(), status_msg: None,
            },
            bridge_stop: None, bridge_log_rx: None, bridge_err_rx: None,
            sim_stop: None, sim_log_rx: None, sim_err_rx: None, sim_registers: None,
            mcp_enabled: true, mcp_port: 9527, mcp_bind_lan: false, mcp_running: false, mcp_status: String::new(),
            mcp_cmd_tx: None,
            mcp_access_log: VecDeque::new(),
            show_mcp_log_popup: false,
            log_search: String::new(),
            log_level_filter: None,
            ai_connected: false,
            ai_port_name: String::new(),
            ai_baud_rate: 0,
            ai_tx_count: 0,
            ai_rx_count: 0,
            connected_by: String::new(),
            mcp_connect_in_progress: false,
            device_id: String::new(),
            device_model: String::new(),
        };
        // Load persisted data from previous session
        state.load_logs();
        state.load_terminal();
        state.load_warnings();
        state.load_mcp_log();
        state
    }

    pub fn refresh_ports(&mut self) {
        self.ports = SerialPort::list_ports().unwrap_or_default();
    }

    pub fn add_terminal_line(&mut self, direction: Direction, content: String, is_hex: bool) {
        self.add_terminal_line_source(direction, content, is_hex, "");
    }

    pub fn add_terminal_line_source(&mut self, direction: Direction, content: String, is_hex: bool, source: &str) {
        let line = TerminalLine {
            timestamp: chrono::Utc::now().timestamp_millis(),
            direction,
            content,
            is_hex,
            source: source.to_string(),
        };
        self.terminal_buffer.push_back(line);
        if self.auto_scroll {
            self.scroll_to_bottom_pending = true;
        }
        if self.terminal_buffer.len() > 5000 {
            self.terminal_buffer.pop_front();
        }
        self.terminal_dirty = true;
    }

    pub fn add_log_entry(&mut self, level: LogLevel, message: &str) {
        let entry = LogEntry {
            timestamp: chrono::Utc::now().timestamp_millis(),
            level,
            message: message.to_string(),
        };
        self.log_entries.push(entry);
        if self.log_entries.len() > 2000 {
            self.log_entries.remove(0);
        }
        self.logs_dirty = true;
    }

    pub fn save_logs(&self) {
        if let Some(path) = log_file_path() {
            if let Ok(content) = serde_json::to_string(&self.log_entries) {
                let _ = std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")));
                let _ = std::fs::write(&path, content);
            }
        }
    }

    pub fn load_logs(&mut self) {
        if let Some(path) = log_file_path() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(entries) = serde_json::from_str::<Vec<LogEntry>>(&content) {
                    self.log_entries = entries.into();
                }
            }
        }
    }

    pub fn save_terminal(&self) {
        if let Some(path) = terminal_file_path() {
            if let Ok(content) = serde_json::to_string(&self.terminal_buffer) {
                let _ = std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")));
                let _ = std::fs::write(&path, content);
            }
        }
    }

    pub fn load_terminal(&mut self) {
        if let Some(path) = terminal_file_path() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(entries) = serde_json::from_str::<VecDeque<TerminalLine>>(&content) {
                    self.terminal_buffer = entries;
                }
            }
        }
    }

    pub fn save_mcp_log(&self) {
        if let Some(path) = mcp_log_file_path() {
            if let Ok(content) = serde_json::to_string(&self.mcp_access_log) {
                let _ = std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")));
                let _ = std::fs::write(&path, content);
            }
        }
    }

    pub fn load_mcp_log(&mut self) {
        if let Some(path) = mcp_log_file_path() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(entries) = serde_json::from_str::<VecDeque<crate::mcp_server::McpAccessLogEntry>>(&content) {
                    self.mcp_access_log = entries;
                }
            }
        }
    }

    pub fn show_error(&mut self, msg: &str) {
        self.global_error = Some(msg.to_string());
        self.global_error_time = chrono::Utc::now().timestamp_millis();
        self.add_log_entry(LogLevel::Error, msg);
        self.add_warning_entry(msg);
    }

    pub fn show_warning(&mut self, msg: &str) {
        self.global_error = Some(msg.to_string());
        self.global_error_time = chrono::Utc::now().timestamp_millis();
        self.add_log_entry(LogLevel::Warning, msg);
        self.add_warning_entry(msg);
    }

    fn add_warning_entry(&mut self, msg: &str) {
        self.warning_history.push_back(WarningEntry {
            timestamp: chrono::Utc::now().timestamp_millis(),
            message: msg.to_string(),
        });
        if self.warning_history.len() > 1000 {
            self.warning_history.pop_front();
        }
        self.save_warnings();
    }

    pub fn clear_error_if_expired(&mut self) {
        if self.global_error.is_some() {
            let now = chrono::Utc::now().timestamp_millis();
            if now - self.global_error_time > 5000 {
                self.global_error = None;
            }
        }
    }

    pub fn save_warnings(&self) {
        if let Some(path) = warning_file_path() {
            if let Ok(content) = serde_json::to_string(&self.warning_history) {
                let _ = std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")));
                let _ = std::fs::write(&path, content);
            }
        }
    }

    pub fn load_warnings(&mut self) {
        if let Some(path) = warning_file_path() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(entries) = serde_json::from_str::<VecDeque<WarningEntry>>(&content) {
                    self.warning_history = entries;
                }
            }
        }
    }

    pub fn add_chart_data(&mut self, value: f64) {
        self.chart_data.push(value);
        if self.chart_data.len() > 200 {
            self.chart_data.remove(0);
        }
    }
}

fn log_file_path() -> Option<std::path::PathBuf> {
    let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).ok()?;
    Some(std::path::PathBuf::from(home).join(".serialrun").join("logs.json"))
}

fn terminal_file_path() -> Option<std::path::PathBuf> {
    let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).ok()?;
    Some(std::path::PathBuf::from(home).join(".serialrun").join("terminal.json"))
}

fn warning_file_path() -> Option<std::path::PathBuf> {
    let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).ok()?;
    Some(std::path::PathBuf::from(home).join(".serialrun").join("warnings.json"))
}

fn mcp_log_file_path() -> Option<std::path::PathBuf> {
    let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).ok()?;
    Some(std::path::PathBuf::from(home).join(".serialrun").join("mcp_access_log.json"))
}
