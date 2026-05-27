use serialtap_core::config::SerialConfig;
use serialtap_core::{SerialPort, SerialPortInfo};
use std::collections::VecDeque;

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
}

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
