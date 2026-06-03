use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

const MAX_MCP_CLIENTS: usize = 10;
use base64::Engine as _;

// ── MCP Lifecycle ──

/// Serial command from MCP server to GUI — the GUI executes all serial
/// operations through its own port_owner so state stays in sync.
pub enum McpSerialRequest {
    Connect {
        port_name: String,
        baud_rate: u32,
        data_bits: u8,
        stop_bits: u8,
        parity: String,
        flow_control: String,
        resp: mpsc::Sender<Result<String, String>>,
    },
    Disconnect {
        resp: mpsc::Sender<Result<String, String>>,
    },
    Send {
        data: Vec<u8>,
        pause_after: bool,
        resp: mpsc::Sender<Result<usize, String>>,
    },
    Read {
        timeout_ms: u64,
        resume: bool,
        resp: mpsc::Sender<Result<Vec<u8>, String>>,
    },
    ReadWait {
        timeout_ms: u64,
        resp: mpsc::Sender<Result<Vec<u8>, String>>,
    },
    /// Write data then read response (exclusive — pauses read loop)
    SendRead {
        data: Vec<u8>,
        timeout_ms: u64,
        resp: mpsc::Sender<Result<Vec<u8>, String>>,
    },
    /// Subscribe to serial port events (returns a receiver for real-time data push)
    SubscribeEvents {
        resp: mpsc::Sender<Option<mpsc::Receiver<crate::port_owner::PortEvent>>>,
    },
    IsConnected {
        resp: mpsc::Sender<bool>,
    },
    GetConfig {
        key: Option<String>,
        resp: mpsc::Sender<serde_json::Value>,
    },
    SetConfig {
        key: String,
        value: serde_json::Value,
        resp: mpsc::Sender<Result<String, String>>,
    },
}

pub enum McpCommand {
    Start { bind_addr: String, port: u16 },
    Stop,
    Reconfigure { bind_addr: String, port: u16 },
    /// Set the channel for MCP to send serial commands to GUI
    SetSerialRequestTx(Option<mpsc::Sender<McpSerialRequest>>),
}

/// Access log entry sent from MCP server to GUI
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct McpAccessLogEntry {
    pub timestamp: String,
    pub client_ip: String,
    pub action: String,
    pub detail: String,
    #[serde(default)]
    pub device_info: String,
}

pub enum McpStatus {
    Running { addr: String },
    Stopped,
    Error(String),
}

/// Handle to control the MCP server from the GUI.
pub struct McpHandle {
    cmd_tx: mpsc::Sender<McpCommand>,
    status_rx: mpsc::Receiver<McpStatus>,
    log_rx: mpsc::Receiver<McpAccessLogEntry>,
}

impl McpHandle {
    pub fn start() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (status_tx, status_rx) = mpsc::channel();
        let (log_tx, log_rx) = mpsc::channel();
        std::thread::spawn(move || mcp_manager(cmd_rx, status_tx, log_tx));
        Self { cmd_tx, status_rx, log_rx }
    }

    pub fn send(&self, cmd: McpCommand) {
        let _ = self.cmd_tx.send(cmd);
    }

    pub fn poll_status(&self) -> Option<McpStatus> {
        self.status_rx.try_recv().ok()
    }

    pub fn poll_log(&self) -> Option<McpAccessLogEntry> {
        self.log_rx.try_recv().ok()
    }

    pub fn cmd_tx(&self) -> mpsc::Sender<McpCommand> {
        self.cmd_tx.clone()
    }
}

/// Shared state between MCP manager and handler threads
struct McpShared {
    /// Channel to send serial commands to GUI (GUI executes all serial ops)
    serial_req_tx: Option<mpsc::Sender<McpSerialRequest>>,
    /// Whether the port is actually connected (from GUI's perspective)
    connected: AtomicBool,
    /// Active client count for concurrency control
    active_clients: AtomicUsize,
    /// Access log entries (IP, time, action)
    access_log: Mutex<Vec<AccessLogEntry>>,
    /// AI connection info (for status tool)
    ai_port_name: Mutex<String>,
    ai_baud_rate: std::sync::atomic::AtomicU32,
    ai_tx_count: std::sync::atomic::AtomicU64,
    ai_rx_count: std::sync::atomic::AtomicU64,
}

use std::sync::atomic::AtomicU32;

#[derive(Clone, serde::Serialize)]
struct AccessLogEntry {
    timestamp: String,
    client_ip: String,
    action: String,
    detail: String,
    device_info: String,
}

fn mcp_manager(cmd_rx: mpsc::Receiver<McpCommand>, status_tx: mpsc::Sender<McpStatus>, log_tx: mpsc::Sender<McpAccessLogEntry>) {
    let mut running = false;
    let mut stop_flag = Arc::new(AtomicBool::new(false));
    let mut current_thread: Option<std::thread::JoinHandle<()>> = None;
    let shared = Arc::new(Mutex::new(McpShared {
        serial_req_tx: None,
        connected: AtomicBool::new(false),
        active_clients: AtomicUsize::new(0),
        access_log: Mutex::new(Vec::new()),
        ai_port_name: Mutex::new(String::new()),
        ai_baud_rate: AtomicU32::new(0),
        ai_tx_count: std::sync::atomic::AtomicU64::new(0),
        ai_rx_count: std::sync::atomic::AtomicU64::new(0),
    }));

    loop {
        match cmd_rx.recv() {
            Ok(McpCommand::Start { bind_addr, port }) => {
                if running { continue; }
                stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
                let sf = stop_flag.clone();
                let stx = status_tx.clone();
                let sh = shared.clone();
                let ltx = log_tx.clone();
                let handle = std::thread::spawn(move || {
                    run_mcp_listener(&bind_addr, port, sf, stx, sh, ltx);
                });
                current_thread = Some(handle);
                running = true;
            }
            Ok(McpCommand::Stop) => {
                if !running { continue; }
                stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                if let Some(h) = current_thread.take() { let _ = h.join(); }
                running = false;
                let _ = status_tx.send(McpStatus::Stopped);
            }
            Ok(McpCommand::Reconfigure { bind_addr, port }) => {
                if running {
                    stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                    if let Some(h) = current_thread.take() { let _ = h.join(); }
                    running = false;
                    let _ = status_tx.send(McpStatus::Stopped);
                }
                stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
                let sf = stop_flag.clone();
                let stx = status_tx.clone();
                let sh = shared.clone();
                let ltx = log_tx.clone();
                let handle = std::thread::spawn(move || {
                    run_mcp_listener(&bind_addr, port, sf, stx, sh, ltx);
                });
                current_thread = Some(handle);
                running = true;
            }
            Ok(McpCommand::SetSerialRequestTx(tx)) => {
                if let Ok(mut sh) = shared.lock() {
                    sh.serial_req_tx = tx;
                }
            }
            Err(_) => break,
        }
    }

    stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    if let Some(h) = current_thread.take() { let _ = h.join(); }
}

fn run_mcp_listener(
    bind_addr: &str,
    port: u16,
    stop_flag: Arc<std::sync::atomic::AtomicBool>,
    status_tx: mpsc::Sender<McpStatus>,
    shared: Arc<Mutex<McpShared>>,
    log_tx: mpsc::Sender<McpAccessLogEntry>,
) {
    let addr = format!("{}:{}", bind_addr, port);
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            let _ = status_tx.send(McpStatus::Error(format!("Failed to bind: {}", e)));
            return;
        }
    };
    listener.set_nonblocking(true).ok();

    let _ = status_tx.send(McpStatus::Running { addr: addr.clone() });
    eprintln!("MCP server listening on {}", addr);

    loop {
        if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
        match listener.accept() {
            Ok((stream, addr)) => {
                // Connection limit check
                {
                    let sh = shared.lock().unwrap();
                    if sh.active_clients.load(Ordering::Relaxed) >= MAX_MCP_CLIENTS {
                        eprintln!("[MCP] Connection rejected from {}: max clients ({}) reached", addr, MAX_MCP_CLIENTS);
                        drop(stream);
                        continue;
                    }
                }
                // Accept inherits non-blocking from listener; reset to blocking for read/write
                let _ = stream.set_nonblocking(false);
                let client_ip = addr.ip().to_string();
                let shared = shared.clone();
                let log_tx = log_tx.clone();
                std::thread::spawn(move || {
                    handle_client(stream, shared, client_ip, log_tx);
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
        }
    }

    let _ = status_tx.send(McpStatus::Stopped);
    eprintln!("MCP server stopped");
}

#[derive(Serialize, Deserialize)]
struct McpRequest {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct McpResponse {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
}

#[derive(Serialize, Deserialize)]
struct McpError {
    code: i32,
    message: String,
}

impl McpResponse {
    fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: Some(result), error: None }
    }
    fn error(id: Option<serde_json::Value>, code: i32, message: String) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: None, error: Some(McpError { code, message }) }
    }
}

fn handle_request(
    request: McpRequest,
    shared: &Mutex<McpShared>,
) -> McpResponse {
    match request.method.as_str() {
        "initialize" => {
            let result = serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "serialrun-mcp", "version": "0.2.0" }
            });
            McpResponse::success(request.id, result)
        }
        "tools/list" => {
            let tools = serde_json::json!({
                "tools": [
                    {
                        "name": "list_ports",
                        "description": "List available serial ports",
                        "inputSchema": { "type": "object", "properties": {} }
                    },
                    {
                        "name": "connect",
                        "description": "Connect to a serial port (independently or via GUI)",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "port": { "type": "string", "description": "Port name (e.g., COM1, /dev/ttyUSB0)" },
                                "baud_rate": { "type": "integer", "description": "Baud rate (default: 115200)" },
                                "data_bits": { "type": "integer", "description": "Data bits: 5, 6, 7, 8 (default: 8)" },
                                "stop_bits": { "type": "integer", "description": "Stop bits: 1, 2 (default: 1)" },
                                "parity": { "type": "string", "description": "Parity: None, Odd, Even (default: None)" },
                                "flow_control": { "type": "string", "description": "Flow control: None, Software, Hardware (default: None)" }
                            },
                            "required": ["port"]
                        }
                    },
                    {
                        "name": "disconnect",
                        "description": "Disconnect from current serial port",
                        "inputSchema": { "type": "object", "properties": {} }
                    },
                    {
                        "name": "send",
                        "description": "Send data to serial port (text or hex). Use pause_after=true to keep reading paused for a subsequent read call.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "data": { "type": "string", "description": "Data to send (text or hex string)" },
                                "hex": { "type": "boolean", "description": "If true, data is interpreted as hex" },
                                "pause_after": { "type": "boolean", "description": "If true, pauses the read loop after sending so next read() can receive the response" }
                            },
                            "required": ["data"]
                        }
                    },
                    {
                        "name": "read",
                        "description": "Read data from serial port. After send(pause_after=true), use resume=false to keep reading paused.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "timeout_ms": { "type": "integer", "description": "Read timeout in ms (default: 1000)" },
                                "max_bytes": { "type": "integer", "description": "Maximum bytes to read (default: 1024)" },
                                "resume": { "type": "boolean", "description": "If true (default), resumes the read loop after reading. Set to false when reading after send(pause_after=true)." },
                                "format": { "type": "string", "description": "Output format: 'hex' (space-separated hex), 'text' (UTF-8), 'raw' (base64). Default: 'hex'", "enum": ["hex", "text", "raw"] }
                            }
                        }
                    },
                    {
                        "name": "send_command",
                        "description": "Send a command and wait for response (write-read)",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "command": { "type": "string", "description": "Command to send" },
                                "timeout_ms": { "type": "integer", "description": "Response timeout in ms (default: 1000)" }
                            },
                            "required": ["command"]
                        }
                    },
                    {
                        "name": "modbus_read",
                        "description": "Read Modbus RTU holding registers with optional engineering value conversion",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "slave_id": { "type": "integer", "description": "Slave ID (1-247)", "default": 1 },
                                "address": { "type": "integer", "description": "Start register address" },
                                "quantity": { "type": "integer", "description": "Number of registers (default: 1)" },
                                "scale": { "type": "number", "description": "Scale factor: value = raw * scale + offset (default: 1.0)" },
                                "offset": { "type": "number", "description": "Offset added after scaling (default: 0.0)" },
                                "unit": { "type": "string", "description": "Unit label for engineering values (default: '')" }
                            },
                            "required": ["address"]
                        }
                    },
                    {
                        "name": "modbus_write",
                        "description": "Write a Modbus RTU holding register",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "slave_id": { "type": "integer", "description": "Slave ID (1-247)", "default": 1 },
                                "address": { "type": "integer", "description": "Register address" },
                                "value": { "type": "integer", "description": "Value to write (u16)" }
                            },
                            "required": ["address", "value"]
                        }
                    },
                    {
                        "name": "plc_read",
                        "description": "Read all registers from a PLC preset",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "brand": { "type": "string", "description": "PLC brand: Siemens, Mitsubishi, Delta, Omron", "default": "Siemens" },
                                "slave_id": { "type": "integer", "description": "Slave ID (1-247)", "default": 1 }
                            }
                        }
                    },
                    {
                        "name": "plc_write",
                        "description": "Write to a PLC register by address",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "brand": { "type": "string", "description": "PLC brand: Siemens, Mitsubishi, Delta, Omron", "default": "Siemens" },
                                "slave_id": { "type": "integer", "description": "Slave ID (1-247)", "default": 1 },
                                "address": { "type": "integer", "description": "Register address" },
                                "value": { "type": "number", "description": "Value to write" }
                            },
                            "required": ["address", "value"]
                        }
                    },
                    {
                        "name": "get_access_log",
                        "description": "Get MCP access log (client IPs, tool calls, timestamps, device info)",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "limit": { "type": "integer", "description": "Max entries to return (default: 50)" }
                            }
                        }
                    },
                    {
                        "name": "get_device_info",
                        "description": "Get current device identification info (port, baud rate, connection status)",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "status",
                        "description": "Get serial port status, connection info, and byte counters",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "get_config",
                        "description": "Get all UI settings or specific setting values",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "key": { "type": "string", "description": "Setting key (optional, returns all if omitted)" }
                            }
                        }
                    },
                    {
                        "name": "set_config",
                        "description": "Update a UI setting (syncs to GUI immediately)",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "key": { "type": "string", "description": "Setting key" },
                                "value": { "description": "New value" }
                            },
                            "required": ["key", "value"]
                        }
                    }
                ]
            });
            McpResponse::success(request.id, tools)
        }
        "tools/call" => {
            let tool_name = request.params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = request.params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

            // Sync McpShared.connected with GUI's actual state (Bug 4 fix)
            if tool_name != "list_ports" && tool_name != "get_access_log" {
                if let Some(ref tx) = { let sh = shared.lock().unwrap(); sh.serial_req_tx.clone() } {
                    let (stx, srx) = mpsc::channel();
                    let _ = tx.send(McpSerialRequest::IsConnected { resp: stx });
                    if let Ok(gui_connected) = srx.recv_timeout(Duration::from_secs(1)) {
                        if let Ok(sh) = shared.lock() {
                            sh.connected.store(gui_connected, Ordering::Relaxed);
                        }
                    }
                }
            }

            match tool_name {
                "list_ports" => {
                    match serialrun_core::SerialPort::list_ports() {
                        Ok(ports) => {
                            let ports_json: Vec<serde_json::Value> = ports.iter().map(|p| {
                                serde_json::json!({
                                    "name": p.name,
                                    "description": p.description,
                                    "manufacturer": p.manufacturer,
                                })
                            }).collect();
                            McpResponse::success(request.id, serde_json::json!({
                                "content": [{ "type": "text", "text": serde_json::to_string_pretty(&ports_json).unwrap() }]
                            }))
                        }
                        Err(e) => McpResponse::error(request.id, -1, e.to_string())
                    }
                }
                "connect" => {
                    let port_name = arguments.get("port").and_then(|v| v.as_str()).unwrap_or("");
                    let baud_rate = arguments.get("baud_rate").and_then(|v| v.as_u64()).unwrap_or(115200) as u32;
                    let data_bits = arguments.get("data_bits").and_then(|v| v.as_u64()).unwrap_or(8) as u8;
                    let stop_bits = arguments.get("stop_bits").and_then(|v| v.as_u64()).unwrap_or(1) as u8;
                    let parity = arguments.get("parity").and_then(|v| v.as_str()).unwrap_or("None").to_string();
                    let flow_control = arguments.get("flow_control").and_then(|v| v.as_str()).unwrap_or("None").to_string();

                    if port_name.is_empty() {
                        return McpResponse::error(request.id, -32602, "Port name is required".into());
                    }

                    let tx = {
                        let sh = match shared.lock() {
                            Ok(sh) => sh,
                            Err(_) => return McpResponse::error(request.id, -1, "Internal error".into()),
                        };
                        match sh.serial_req_tx.clone() {
                            Some(tx) => tx,
                            None => return McpResponse::error(request.id, -1, "GUI not available. Start SerialRUN first.".into()),
                        }
                    };

                    let (resp_tx, resp_rx) = mpsc::channel();
                    let _ = tx.send(McpSerialRequest::Connect {
                        port_name: port_name.to_string(),
                        baud_rate, data_bits, stop_bits, parity, flow_control,
                        resp: resp_tx,
                    });
                    match resp_rx.recv_timeout(Duration::from_secs(5)) {
                        Ok(Ok(msg)) => {
                            if let Ok(sh) = shared.lock() {
                                sh.connected.store(true, Ordering::Relaxed);
                                *sh.ai_port_name.lock().unwrap() = port_name.to_string();
                                sh.ai_baud_rate.store(baud_rate, Ordering::Relaxed);
                            }
                            McpResponse::success(request.id, serde_json::json!({
                                "content": [{ "type": "text", "text": msg }]
                            }))
                        }
                        Ok(Err(e)) => {
                            if let Ok(sh) = shared.lock() {
                                sh.connected.store(false, Ordering::Relaxed);
                            }
                            McpResponse::error(request.id, -1, e)
                        }
                        Err(_) => {
                            if let Ok(sh) = shared.lock() {
                                sh.connected.store(false, Ordering::Relaxed);
                            }
                            McpResponse::error(request.id, -1, "Timeout waiting for GUI".into())
                        }
                    }
                }
                "disconnect" => {
                    let tx = {
                        let sh = match shared.lock() {
                            Ok(sh) => sh,
                            Err(_) => return McpResponse::error(request.id, -1, "Internal error".into()),
                        };
                        match sh.serial_req_tx.clone() {
                            Some(tx) => tx,
                            None => return McpResponse::error(request.id, -1, "GUI not available".into()),
                        }
                    };

                    let (resp_tx, resp_rx) = mpsc::channel();
                    let _ = tx.send(McpSerialRequest::Disconnect { resp: resp_tx });
                    match resp_rx.recv_timeout(Duration::from_secs(5)) {
                        Ok(Ok(msg)) => {
                            if let Ok(sh) = shared.lock() {
                                sh.connected.store(false, Ordering::Relaxed);
                                *sh.ai_port_name.lock().unwrap() = String::new();
                                sh.ai_baud_rate.store(0, Ordering::Relaxed);
                            }
                            McpResponse::success(request.id, serde_json::json!({
                                "content": [{ "type": "text", "text": msg }]
                            }))
                        }
                        Ok(Err(e)) => McpResponse::error(request.id, -1, e),
                        Err(_) => McpResponse::error(request.id, -1, "Timeout waiting for GUI".into()),
                    }
                }
                "send" => {
                    let data = arguments.get("data").and_then(|v| v.as_str()).unwrap_or("");
                    let hex = arguments.get("hex").and_then(|v| v.as_bool()).unwrap_or(false);
                    let pause_after = arguments.get("pause_after").and_then(|v| v.as_bool()).unwrap_or(false);

                    if data.is_empty() {
                        return McpResponse::error(request.id, -32602, "Data is required".into());
                    }

                    let bytes = if hex {
                        data.split_whitespace()
                            .filter_map(|s| u8::from_str_radix(s, 16).ok())
                            .collect::<Vec<_>>()
                    } else {
                        data.as_bytes().to_vec()
                    };
                    let _len = bytes.len();

                    let tx = {
                        let sh = match shared.lock() {
                            Ok(sh) => sh,
                            Err(_) => return McpResponse::error(request.id, -1, "Internal error".into()),
                        };
                        match sh.serial_req_tx.clone() {
                            Some(tx) => tx,
                            None => return McpResponse::error(request.id, -1, "Not connected".into()),
                        }
                    };

                    let (resp_tx, resp_rx) = mpsc::channel();
                    let _ = tx.send(McpSerialRequest::Send { data: bytes, pause_after, resp: resp_tx });
                    match resp_rx.recv_timeout(Duration::from_secs(5)) {
                        Ok(Ok(n)) => {
                            if let Ok(sh) = shared.lock() {
                                sh.ai_tx_count.fetch_add(n as u64, Ordering::Relaxed);
                            }
                            McpResponse::success(request.id, serde_json::json!({
                                "content": [{ "type": "text", "text": format!("Sent {} bytes{}", n, if pause_after { " (read loop paused)" } else { "" }) }]
                            }))
                        }
                        Ok(Err(e)) => McpResponse::error(request.id, -1, e),
                        Err(_) => McpResponse::error(request.id, -1, "Timeout".into()),
                    }
                }
                "read" => {
                    let timeout_ms = arguments.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(1000);
                    let resume = arguments.get("resume").and_then(|v| v.as_bool()).unwrap_or(true);
                    let format = arguments.get("format").and_then(|v| v.as_str()).unwrap_or("hex");

                    let tx = {
                        let sh = match shared.lock() {
                            Ok(sh) => sh,
                            Err(_) => return McpResponse::error(request.id, -1, "Internal error".into()),
                        };
                        match sh.serial_req_tx.clone() {
                            Some(tx) => tx,
                            None => return McpResponse::error(request.id, -1, "Not connected".into()),
                        }
                    };

                    let (resp_tx, resp_rx) = mpsc::channel();
                    let _ = tx.send(McpSerialRequest::Read { timeout_ms, resume, resp: resp_tx });
                    match resp_rx.recv_timeout(Duration::from_secs(6)) {
                        Ok(Ok(data)) => {
                            if !data.is_empty() {
                                if let Ok(sh) = shared.lock() {
                                    sh.ai_rx_count.fetch_add(data.len() as u64, Ordering::Relaxed);
                                }
                            }
                            let formatted = match format {
                                "text" => String::from_utf8_lossy(&data).to_string(),
                                "raw" => base64::engine::general_purpose::STANDARD.encode(&*data),
                                _ => data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "),
                            };
                            McpResponse::success(request.id, serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": formatted,
                                    "format": format,
                                    "length": data.len()
                                }]
                            }))
                        }
                        Ok(Err(e)) => McpResponse::error(request.id, -1, e),
                        Err(_) => McpResponse::error(request.id, -1, "Timeout".into()),
                    }
                }
                "send_command" => {
                    let command = arguments.get("command").and_then(|v| v.as_str()).unwrap_or("");
                    let timeout_ms = arguments.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(1000);

                    if command.is_empty() {
                        return McpResponse::error(request.id, -32602, "Command is required".into());
                    }

                    let mut cmd_bytes = command.as_bytes().to_vec();
                    if !command.ends_with("\r\n") && !command.ends_with('\n') && !command.ends_with('\r') {
                        cmd_bytes.extend_from_slice(b"\r\n");
                    }

                    let tx = {
                        let sh = match shared.lock() {
                            Ok(sh) => sh,
                            Err(_) => return McpResponse::error(request.id, -1, "Internal error".into()),
                        };
                        match sh.serial_req_tx.clone() {
                            Some(tx) => tx,
                            None => return McpResponse::error(request.id, -1, "Not connected".into()),
                        }
                    };

                    let (resp_tx, resp_rx) = mpsc::channel();
                    let _ = tx.send(McpSerialRequest::SendRead { data: cmd_bytes, timeout_ms, resp: resp_tx });
                    match resp_rx.recv_timeout(Duration::from_secs(6)) {
                        Ok(Ok(data)) => {
                            let response = String::from_utf8_lossy(&data).to_string();
                            McpResponse::success(request.id, serde_json::json!({
                                "content": [{ "type": "text", "text": response }]
                            }))
                        }
                        Ok(Err(e)) => McpResponse::error(request.id, -1, e),
                        Err(_) => McpResponse::error(request.id, -1, "Timeout".into()),
                    }
                }
                "modbus_read" => {
                    let slave_id = arguments.get("slave_id").and_then(|v| v.as_u64()).unwrap_or(1) as u8;
                    let address = match arguments.get("address").and_then(|v| v.as_u64()) {
                        Some(a) => a as u16,
                        None => return McpResponse::error(request.id, -32602, "address is required".into()),
                    };
                    let quantity = arguments.get("quantity").and_then(|v| v.as_u64()).unwrap_or(1) as u16;
                    if quantity == 0 || quantity > 125 {
                        return McpResponse::error(request.id, -32602, "quantity must be 1-125".into());
                    }
                    let scale = arguments.get("scale").and_then(|v| v.as_f64()).unwrap_or(1.0);
                    let offset = arguments.get("offset").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let unit = arguments.get("unit").and_then(|v| v.as_str()).unwrap_or("");

                    use serialrun_core::protocol::{ModbusFrame, ModbusParser, ModbusFunction};
                    let frame = ModbusParser::build_read_request(slave_id, ModbusFunction::ReadHoldingRegisters, address, quantity);
                    let req = frame.to_bytes();

                    let tx = {
                        let sh = match shared.lock() {
                            Ok(sh) => sh,
                            Err(_) => return McpResponse::error(request.id, -1, "Internal error".into()),
                        };
                        match sh.serial_req_tx.clone() {
                            Some(tx) => tx,
                            None => return McpResponse::error(request.id, -1, "Not connected".into()),
                        }
                    };

                    let (resp_tx, resp_rx) = mpsc::channel();
                    let _ = tx.send(McpSerialRequest::SendRead { data: req.clone(), timeout_ms: 200, resp: resp_tx });
                    match resp_rx.recv_timeout(Duration::from_secs(5)) {
                        Ok(Ok(resp)) if resp.len() >= 4 => {
                            if let Ok(sh) = shared.lock() {
                                sh.ai_tx_count.fetch_add(req.len() as u64, Ordering::Relaxed);
                                sh.ai_rx_count.fetch_add(resp.len() as u64, Ordering::Relaxed);
                            }
                            match ModbusFrame::parse(&resp) {
                                Ok(f) => {
                                    let hex = resp.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                                    let mut values = Vec::new();
                                    let data = &f.data;
                                    let mut i = 1;
                                    while i + 1 < data.len() {
                                        values.push(u16::from_be_bytes([data[i], data[i+1]]));
                                        i += 2;
                                    }
                                    let use_conversion = (scale - 1.0).abs() > f64::EPSILON || offset.abs() > f64::EPSILON || !unit.is_empty();
                                    let result_text = if use_conversion {
                                        let engineering: Vec<serde_json::Value> = values.iter().map(|v| {
                                            let eng = *v as f64 * scale + offset;
                                            if unit.is_empty() {
                                                serde_json::json!({"raw": v, "value": format!("{:.3}", eng)})
                                            } else {
                                                serde_json::json!({"raw": v, "value": format!("{:.3}", eng), "unit": unit})
                                            }
                                        }).collect();
                                        format!("Read {} registers from slave {}\nHEX: {}\nRaw: {:?}\nEngineering: {}",
                                            quantity, slave_id, hex, values,
                                            serde_json::to_string_pretty(&engineering).unwrap())
                                    } else {
                                        format!("Read {} registers from slave {}\nHEX: {}\nValues: {:?}", quantity, slave_id, hex, values)
                                    };
                                    McpResponse::success(request.id, serde_json::json!({
                                        "content": [{ "type": "text", "text": result_text }]
                                    }))
                                }
                                Err(e) => McpResponse::error(request.id, -1, format!("Parse error: {}", e))
                            }
                        }
                        _ => McpResponse::error(request.id, -1, "No response".into())
                    }
                }
                "modbus_write" => {
                    let slave_id = arguments.get("slave_id").and_then(|v| v.as_u64()).unwrap_or(1) as u8;
                    let address = match arguments.get("address").and_then(|v| v.as_u64()) {
                        Some(a) => a as u16,
                        None => return McpResponse::error(request.id, -32602, "address is required".into()),
                    };
                    let value = match arguments.get("value").and_then(|v| v.as_u64()) {
                        Some(v) => {
                            if v > 65535 {
                                return McpResponse::error(request.id, -32602, "value must be 0-65535".into());
                            }
                            v as u16
                        }
                        None => return McpResponse::error(request.id, -32602, "value is required".into()),
                    };

                    use serialrun_core::protocol::ModbusParser;
                    let frame = ModbusParser::build_write_single(slave_id, address, value);
                    let req = frame.to_bytes();

                    let tx = {
                        let sh = match shared.lock() {
                            Ok(sh) => sh,
                            Err(_) => return McpResponse::error(request.id, -1, "Internal error".into()),
                        };
                        match sh.serial_req_tx.clone() {
                            Some(tx) => tx,
                            None => return McpResponse::error(request.id, -1, "Not connected".into()),
                        }
                    };

                    let (resp_tx, resp_rx) = mpsc::channel();
                    let _ = tx.send(McpSerialRequest::SendRead { data: req.clone(), timeout_ms: 200, resp: resp_tx });
                    match resp_rx.recv_timeout(Duration::from_secs(5)) {
                        Ok(Ok(resp)) if resp.len() >= 4 => {
                            if let Ok(sh) = shared.lock() {
                                sh.ai_tx_count.fetch_add(req.len() as u64, Ordering::Relaxed);
                                sh.ai_rx_count.fetch_add(resp.len() as u64, Ordering::Relaxed);
                            }
                            let hex = resp.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                            McpResponse::success(request.id, serde_json::json!({
                                "content": [{ "type": "text", "text": format!("Wrote {} to register 0x{:04X} (slave {})\nResponse: {}", value, address, slave_id, hex) }]
                            }))
                        }
                        _ => McpResponse::error(request.id, -1, "No response".into())
                    }
                }
                "plc_read" => {
                    let brand_name = arguments.get("brand").and_then(|v| v.as_str()).unwrap_or("Siemens");
                    let slave_id = arguments.get("slave_id").and_then(|v| v.as_u64()).unwrap_or(1) as u8;

                    let brand = match brand_name {
                        "Siemens" | "siemens" => crate::state::PlcBrand::Siemens,
                        "Mitsubishi" | "mitsubishi" => crate::state::PlcBrand::Mitsubishi,
                        "Delta" | "delta" => crate::state::PlcBrand::Delta,
                        "Omron" | "omron" => crate::state::PlcBrand::Omron,
                        _ => return McpResponse::error(request.id, -32602, format!("Unknown brand: {}. Use Siemens, Mitsubishi, Delta, Omron", brand_name)),
                    };

                    let models = crate::plc_presets::get_models(brand);
                    let regs = models.first().map(|m| m.registers.clone()).unwrap_or_default();
                    if regs.is_empty() {
                        return McpResponse::error(request.id, -1, "No registers defined for this brand".into());
                    }

                    let tx = {
                        let sh = match shared.lock() {
                            Ok(sh) => sh,
                            Err(_) => return McpResponse::error(request.id, -1, "Internal error".into()),
                        };
                        match sh.serial_req_tx.clone() {
                            Some(tx) => tx,
                            None => return McpResponse::error(request.id, -1, "Not connected".into()),
                        }
                    };

                    use serialrun_core::protocol::{ModbusFrame, ModbusParser, ModbusFunction};
                    let mut results = Vec::new();
                    for reg in &regs {
                        let qty = match reg.data_type {
                            crate::state::PlcDataType::U32 | crate::state::PlcDataType::Float32 => 2,
                            _ => 1,
                        };
                        let frame = ModbusParser::build_read_request(slave_id, ModbusFunction::ReadHoldingRegisters, reg.addr, qty);
                        let req = frame.to_bytes();
                        let (resp_tx, resp_rx) = mpsc::channel();
                        let _ = tx.send(McpSerialRequest::SendRead { data: req.clone(), timeout_ms: 200, resp: resp_tx });
                        match resp_rx.recv_timeout(Duration::from_secs(5)) {
                            Ok(Ok(resp)) if resp.len() >= 4 => {
                                if let Ok(f) = ModbusFrame::parse(&resp) {
                                    let data = &f.data;
                                    let val_str = match reg.data_type {
                                        crate::state::PlcDataType::Bool => {
                                            let raw = data.get(1).copied().unwrap_or(0);
                                            if raw != 0 { "ON".into() } else { "OFF".into() }
                                        }
                                        crate::state::PlcDataType::U16 => {
                                            let raw = data.get(1..3).map(|d| u16::from_be_bytes([d[0], d[1]])).unwrap_or(0);
                                            if reg.scale_factor != 1.0 { format!("{:.2}", raw as f64 * reg.scale_factor) } else { format!("{}", raw) }
                                        }
                                        crate::state::PlcDataType::I16 => {
                                            let raw = data.get(1..3).map(|d| u16::from_be_bytes([d[0], d[1]])).unwrap_or(0) as i16;
                                            if reg.scale_factor != 1.0 { format!("{:.2}", raw as f64 * reg.scale_factor) } else { format!("{}", raw) }
                                        }
                                        crate::state::PlcDataType::U32 => {
                                            let raw = data.get(1..5).map(|d| u32::from_be_bytes([d[0], d[1], d[2], d[3]])).unwrap_or(0);
                                            if reg.scale_factor != 1.0 { format!("{:.2}", raw as f64 * reg.scale_factor) } else { format!("{}", raw) }
                                        }
                                        crate::state::PlcDataType::Float32 => {
                                            let raw = data.get(1..5).map(|d| u32::from_be_bytes([d[0], d[1], d[2], d[3]])).unwrap_or(0);
                                            let f = f32::from_bits(raw);
                                            if reg.scale_factor != 1.0 { format!("{:.3}", f as f64 * reg.scale_factor) } else { format!("{:.3}", f) }
                                        }
                                    };
                                    results.push(serde_json::json!({"addr": reg.addr, "name": reg.name, "type": reg.data_type.label(), "value": val_str, "unit": reg.unit}));
                                } else {
                                    results.push(serde_json::json!({"addr": reg.addr, "name": reg.name, "error": "parse error"}));
                                }
                            }
                            _ => {
                                results.push(serde_json::json!({"addr": reg.addr, "name": reg.name, "error": "no response"}));
                            }
                        }
                    }

                    McpResponse::success(request.id, serde_json::json!({
                        "content": [{ "type": "text", "text": format!("{} PLC ({}) slave {} - {} registers:\n{}", brand_name, models.first().map(|m| m.model).unwrap_or("?"), slave_id, results.len(), serde_json::to_string_pretty(&results).unwrap()) }]
                    }))
                }
                "plc_write" => {
                    let brand_name = arguments.get("brand").and_then(|v| v.as_str()).unwrap_or("Siemens");
                    let slave_id = arguments.get("slave_id").and_then(|v| v.as_u64()).unwrap_or(1) as u8;
                    let address = match arguments.get("address").and_then(|v| v.as_u64()) {
                        Some(a) => a as u16,
                        None => return McpResponse::error(request.id, -32602, "address is required".into()),
                    };
                    let value = match arguments.get("value") {
                        Some(v) => v.as_f64().unwrap_or(0.0),
                        None => return McpResponse::error(request.id, -32602, "value is required".into()),
                    };

                    let _brand = match brand_name {
                        "Siemens" | "siemens" => crate::state::PlcBrand::Siemens,
                        "Mitsubishi" | "mitsubishi" => crate::state::PlcBrand::Mitsubishi,
                        "Delta" | "delta" => crate::state::PlcBrand::Delta,
                        "Omron" | "omron" => crate::state::PlcBrand::Omron,
                        _ => return McpResponse::error(request.id, -32602, format!("Unknown brand: {}", brand_name)),
                    };

                    use serialrun_core::protocol::ModbusParser;
                    if value < 0.0 || value > 65535.0 {
                        return McpResponse::error(request.id, -32602, "value must be 0-65535".into());
                    }
                    let raw_val = value as u16;
                    let frame = ModbusParser::build_write_single(slave_id, address, raw_val);
                    let req = frame.to_bytes();

                    let tx = {
                        let sh = match shared.lock() {
                            Ok(sh) => sh,
                            Err(_) => return McpResponse::error(request.id, -1, "Internal error".into()),
                        };
                        match sh.serial_req_tx.clone() {
                            Some(tx) => tx,
                            None => return McpResponse::error(request.id, -1, "Not connected".into()),
                        }
                    };

                    let (resp_tx, resp_rx) = mpsc::channel();
                    let _ = tx.send(McpSerialRequest::SendRead { data: req.clone(), timeout_ms: 200, resp: resp_tx });
                    match resp_rx.recv_timeout(Duration::from_secs(5)) {
                        Ok(Ok(resp)) if resp.len() >= 4 => {
                            if let Ok(sh) = shared.lock() {
                                sh.ai_tx_count.fetch_add(req.len() as u64, Ordering::Relaxed);
                                sh.ai_rx_count.fetch_add(resp.len() as u64, Ordering::Relaxed);
                            }
                            McpResponse::success(request.id, serde_json::json!({
                                "content": [{ "type": "text", "text": format!("Wrote {} to {} register 0x{:04X} (slave {})", value, brand_name, address, slave_id) }]
                            }))
                        }
                        _ => McpResponse::error(request.id, -1, "No response".into())
                    }
                }
                "get_access_log" => {
                    let limit = arguments.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
                    let sh = match shared.lock() {
                        Ok(sh) => sh,
                        Err(_) => return McpResponse::error(request.id, -1, "Internal error".into()),
                    };
                    let active = sh.active_clients.load(std::sync::atomic::Ordering::Relaxed);
                    let log_entries = match sh.access_log.lock() {
                        Ok(log) => {
                            let start = if log.len() > limit { log.len() - limit } else { 0 };
                            log[start..].iter().map(|e| {
                                serde_json::json!({
                                    "time": e.timestamp,
                                    "ip": e.client_ip,
                                    "action": e.action,
                                    "detail": e.detail,
                                })
                            }).collect::<Vec<_>>()
                        }
                        Err(_) => vec![],
                    };
                    McpResponse::success(request.id, serde_json::json!({
                        "content": [{ "type": "text", "text": format!("Active clients: {}\n\nAccess Log (last {}):\n{}", active, log_entries.len(), serde_json::to_string_pretty(&log_entries).unwrap()) }]
                    }))
                }
                "get_device_info" => {
                    let sh = match shared.lock() {
                        Ok(sh) => sh,
                        Err(_) => return McpResponse::error(request.id, -1, "Internal error".into()),
                    };
                    let connected = sh.connected.load(std::sync::atomic::Ordering::Relaxed);
                    let active = sh.active_clients.load(std::sync::atomic::Ordering::Relaxed);
                    let log_count = match sh.access_log.lock() {
                        Ok(log) => log.len(),
                        Err(_) => 0,
                    };
                    McpResponse::success(request.id, serde_json::json!({
                        "content": [{ "type": "text", "text": format!(
                            "Device: SerialRUN\nStatus: {}\nActive clients: {}\nTotal access log entries: {}\nServer: MCP v0.2.0\nProtocol: JSON-RPC over TCP",
                            if connected { "Connected" } else { "Disconnected" },
                            active,
                            log_count
                        ) }]
                    }))
                }
                "status" => {
                    let sh = match shared.lock() {
                        Ok(sh) => sh,
                        Err(_) => return McpResponse::error(request.id, -1, "Internal error".into()),
                    };
                    let connected = sh.connected.load(Ordering::Relaxed);
                    let has_serial_tx = sh.serial_req_tx.is_some();
                    let ai_port = sh.ai_port_name.lock().unwrap().clone();
                    let gui_connected = connected && has_serial_tx && ai_port.is_empty();
                    let ai_connected = connected && has_serial_tx && !ai_port.is_empty();
                    let active_clients = sh.active_clients.load(Ordering::Relaxed);
                    let ai_baud = sh.ai_baud_rate.load(Ordering::Relaxed);
                    let ai_tx = sh.ai_tx_count.load(Ordering::Relaxed);
                    let ai_rx = sh.ai_rx_count.load(Ordering::Relaxed);

                    let status = serde_json::json!({
                        "connection": {
                            "gui_connected": gui_connected,
                            "ai_connected": ai_connected,
                            "port": if ai_connected { &ai_port } else { "N/A" },
                            "baud_rate": if ai_connected { ai_baud } else { 0 },
                        },
                        "mcp": {
                            "active_clients": active_clients,
                            "server_version": "0.2.0",
                        },
                        "counters": {
                            "ai_tx_bytes": ai_tx,
                            "ai_rx_bytes": ai_rx,
                        }
                    });
                    McpResponse::success(request.id, serde_json::json!({
                        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&status).unwrap() }]
                    }))
                }
                "get_config" => {
                    let key = arguments.get("key").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let (resp_tx, resp_rx) = mpsc::channel();
                    let tx = { let sh = shared.lock().unwrap(); sh.serial_req_tx.clone() };
                    if let Some(tx) = tx {
                        let _ = tx.send(McpSerialRequest::GetConfig { key, resp: resp_tx });
                        match resp_rx.recv_timeout(Duration::from_secs(2)) {
                            Ok(val) => McpResponse::success(request.id, serde_json::json!({
                                "content": [{ "type": "text", "text": serde_json::to_string_pretty(&val).unwrap() }]
                            })),
                            Err(_) => McpResponse::error(request.id, -1, "Timeout".into()),
                        }
                    } else {
                        McpResponse::error(request.id, -1, "Not connected".into())
                    }
                }
                "set_config" => {
                    let key = arguments.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let value = arguments.get("value").cloned().unwrap_or(serde_json::Value::Null);
                    if key.is_empty() {
                        McpResponse::error(request.id, -32602, "key is required".into())
                    } else {
                        let (resp_tx, resp_rx) = mpsc::channel();
                        let tx = { let sh = shared.lock().unwrap(); sh.serial_req_tx.clone() };
                        if let Some(tx) = tx {
                            let _ = tx.send(McpSerialRequest::SetConfig { key, value, resp: resp_tx });
                            match resp_rx.recv_timeout(Duration::from_secs(2)) {
                                Ok(Ok(msg)) => McpResponse::success(request.id, serde_json::json!({
                                    "content": [{ "type": "text", "text": msg }]
                                })),
                                Ok(Err(e)) => McpResponse::error(request.id, -32602, e),
                                Err(_) => McpResponse::error(request.id, -1, "Timeout".into()),
                            }
                        } else {
                            McpResponse::error(request.id, -1, "Not connected".into())
                        }
                    }
                }
                _ => McpResponse::error(request.id, -32601, format!("Unknown tool: {}", tool_name))
            }
        }
        "notifications/initialized" => {
            McpResponse::success(request.id, serde_json::json!({}))
        }
        _ => McpResponse::error(request.id, -32601, format!("Unknown method: {}", request.method))
    }
}

fn handle_client(stream: TcpStream, shared: Arc<Mutex<McpShared>>, client_ip: String, log_tx: mpsc::Sender<McpAccessLogEntry>) {
    // Wrap stream in Arc<Mutex> for thread-safe writes (Bug 2 fix)
    let stream = Arc::new(Mutex::new(stream));

    // Log client connect with device info
    let connect_detail = {
        let sh = shared.lock().unwrap();
        if sh.connected.load(Ordering::Relaxed) {
            let port = sh.ai_port_name.lock().unwrap().clone();
            if port.is_empty() {
                "GUI connected".to_string()
            } else {
                format!("AI: {}", port)
            }
        } else {
            "No port open".to_string()
        }
    };
    log_access(&shared, &client_ip, "CONNECT", &connect_detail, &log_tx);
    eprintln!("[MCP] Client connected: {}", client_ip);

    // Increment active client count
    {
        let sh = shared.lock().unwrap();
        sh.active_clients.fetch_add(1, Ordering::Relaxed);
    }

    // Subscribe to serial port events for real-time push notifications
    let push_stop = Arc::new(AtomicBool::new(false));
    let push_handle = {
        let tx = {
            let sh = shared.lock().unwrap();
            sh.serial_req_tx.clone()
        };
        if let Some(tx) = tx {
            let (sub_tx, sub_rx) = mpsc::channel();
            let _ = tx.send(McpSerialRequest::SubscribeEvents { resp: sub_tx });
            match sub_rx.recv_timeout(Duration::from_secs(2)) {
                Ok(Some(evt_rx)) => {
                    let write_stream = stream.clone();
                    let stop = push_stop.clone();
                    let ip = client_ip.clone();
                    Some(std::thread::spawn(move || {
                        event_push_thread(write_stream, evt_rx, stop, ip);
                    }))
                }
                _ => None,
            }
        } else { None }
    };

    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];

    loop {
        // Read from the stream (lock briefly for read)
        let n = {
            let mut s = stream.lock().unwrap();
            match s.read(&mut tmp) {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            }
        };
        buf.extend_from_slice(&tmp[..n]);

        // Process all complete lines in buffer
        while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let line_bytes: Vec<u8> = buf.drain(..=pos).collect();
            let line = String::from_utf8_lossy(&line_bytes).trim().to_string();
            if line.is_empty() { continue; }

            let request: McpRequest = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    let response = McpResponse::error(None, -32700, format!("Parse error: {}", e));
                    let mut s = stream.lock().unwrap();
                    let _ = write!(s, "{}\n", serde_json::to_string(&response).unwrap());
                    let _ = s.flush();
                    continue;
                }
            };

            // Log tool call with arguments
            let tool_name = request.params.get("name").and_then(|v| v.as_str()).unwrap_or(&request.method).to_string();
            let call_detail = {
                let args = request.params.get("arguments");
                match args {
                    Some(a) if !a.is_object() || a.as_object().map_or(false, |m| !m.is_empty()) => {
                        let args_str = serde_json::to_string(a).unwrap_or_default();
                        // Truncate long args for readability
                        if args_str.len() > 120 {
                            format!("{}({}...)", tool_name, &args_str[..120])
                        } else {
                            format!("{}({})", tool_name, args_str)
                        }
                    }
                    _ => tool_name.clone(),
                }
            };
            log_access(&shared, &client_ip, "CALL", &call_detail, &log_tx);

            let response = handle_request(request, &shared);
            let resp_bytes = serde_json::to_string(&response).unwrap();
            let resp_line = format!("{}\n", resp_bytes);
            {
                let mut s = stream.lock().unwrap();
                let _ = s.write_all(resp_line.as_bytes());
                let _ = s.flush();
            }

            // For disconnect, wait for the response to be fully delivered
            // before the client closes the connection
            if tool_name == "disconnect" {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }

    // Signal push thread to stop and join it
    push_stop.store(true, Ordering::Relaxed);
    if let Some(h) = push_handle {
        let _ = h.join();
    }

    // Decrement active client count and log disconnect
    let disconnect_detail = {
        let sh = shared.lock().unwrap();
        let remaining = sh.active_clients.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
            v.checked_sub(1)
        }).unwrap_or(0);
        format!("{} clients remaining", remaining)
    };
    log_access(&shared, &client_ip, "DISCONNECT", &disconnect_detail, &log_tx);
    eprintln!("[MCP] Client disconnected: {}", client_ip);
}

/// Push serial port events to MCP client as JSON-RPC notifications
fn event_push_thread(
    write_stream: Arc<Mutex<TcpStream>>,
    evt_rx: mpsc::Receiver<crate::port_owner::PortEvent>,
    stop: Arc<AtomicBool>,
    client_ip: String,
) {
    use crate::port_owner::PortEvent;
    loop {
        if stop.load(Ordering::Relaxed) { break; }
        match evt_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(PortEvent::Data(data)) => {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                let notification = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "notifications/serial_data",
                    "params": {
                        "data": b64,
                        "length": data.len(),
                        "timestamp": chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
                    }
                });
                let msg = format!("{}\n", serde_json::to_string(&notification).unwrap());
                let mut s = write_stream.lock().unwrap();
                if s.write_all(msg.as_bytes()).is_err() { break; }
                let _ = s.flush();
            }
            Ok(PortEvent::Opened(ok, msg)) => {
                let notification = serde_json::json!({
                    "jsonrpc": "2.0", "method": "notifications/serial_event",
                    "params": { "event": "opened", "success": ok, "message": msg, "client_ip": client_ip }
                });
                let msg = format!("{}\n", serde_json::to_string(&notification).unwrap());
                let mut s = write_stream.lock().unwrap();
                if s.write_all(msg.as_bytes()).is_err() { break; }
                let _ = s.flush();
            }
            Ok(PortEvent::Closed) => {
                let notification = serde_json::json!({
                    "jsonrpc": "2.0", "method": "notifications/serial_event",
                    "params": { "event": "closed", "client_ip": client_ip }
                });
                let msg = format!("{}\n", serde_json::to_string(&notification).unwrap());
                let mut s = write_stream.lock().unwrap();
                if s.write_all(msg.as_bytes()).is_err() { break; }
                let _ = s.flush();
            }
            Ok(PortEvent::Error(e)) => {
                let notification = serde_json::json!({
                    "jsonrpc": "2.0", "method": "notifications/serial_event",
                    "params": { "event": "error", "message": e, "client_ip": client_ip }
                });
                let msg = format!("{}\n", serde_json::to_string(&notification).unwrap());
                let mut s = write_stream.lock().unwrap();
                if s.write_all(msg.as_bytes()).is_err() { break; }
                let _ = s.flush();
            }
            Ok(_) => {}
            Err(mpsc::RecvTimeoutError::Timeout) => { continue; }
            Err(mpsc::RecvTimeoutError::Disconnected) => { break; }
        }
    }
    eprintln!("[MCP] Event push thread stopped for {}", client_ip);
}

fn log_access(shared: &Arc<Mutex<McpShared>>, client_ip: &str, action: &str, detail: &str, log_tx: &mpsc::Sender<McpAccessLogEntry>) {
    let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();

    let device_info = {
        let sh = shared.lock().unwrap();
        if sh.connected.load(Ordering::Relaxed) {
            format!("SerialRUN@{}", client_ip)
        } else {
            "No device connected".to_string()
        }
    };

    // Send to GUI via channel
    let gui_entry = McpAccessLogEntry {
        timestamp: ts.clone(),
        client_ip: client_ip.to_string(),
        action: action.to_string(),
        detail: detail.to_string(),
        device_info: device_info.clone(),
    };
    let _ = log_tx.send(gui_entry);

    // Also store in shared log for get_access_log tool
    let entry = AccessLogEntry {
        timestamp: ts,
        client_ip: client_ip.to_string(),
        action: action.to_string(),
        detail: detail.to_string(),
        device_info,
    };
    if let Ok(sh) = shared.lock() {
        if let Ok(mut log) = sh.access_log.lock() {
            log.push(entry);
            if log.len() > 500 {
                log.drain(0..100);
            }
        }
    }
}
