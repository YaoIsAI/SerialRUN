//! Embedded MCP server — shares serial port and log with UI

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize)]
pub struct McpRequest { pub jsonrpc: String, pub id: Option<serde_json::Value>, pub method: String, #[serde(default)] pub params: serde_json::Value }
#[derive(Serialize, Deserialize)]
pub struct McpResponse { pub jsonrpc: String, pub id: Option<serde_json::Value>, #[serde(skip_serializing_if = "Option::is_none")] pub result: Option<serde_json::Value>, #[serde(skip_serializing_if = "Option::is_none")] pub error: Option<McpError> }
#[derive(Serialize, Deserialize)]
pub struct McpError { pub code: i32, pub message: String }
impl McpResponse {
    fn ok(id: Option<serde_json::Value>, r: serde_json::Value) -> Self { Self { jsonrpc: "2.0".into(), id, result: Some(r), error: None } }
    fn err(id: Option<serde_json::Value>, c: i32, m: String) -> Self { Self { jsonrpc: "2.0".into(), id, result: None, error: Some(McpError { code: c, message: m }) } }
}

pub struct McpState {
    pub shared_port: Arc<Mutex<Option<serialrun_core::SerialPort>>>,
    pub connected_port: Option<String>,
    pub log: Arc<Mutex<Vec<String>>>,
}

impl McpState {
    pub fn new(shared_port: Arc<Mutex<Option<serialrun_core::SerialPort>>>, log: Arc<Mutex<Vec<String>>>) -> Self {
        Self { shared_port, connected_port: None, log }
    }
    fn notify(&self, msg: &str) { if let Ok(mut l) = self.log.lock() { l.push(msg.to_string()); } }

    pub fn handle_request(&mut self, req: McpRequest) -> McpResponse {
        match req.method.as_str() {
            "initialize" => McpResponse::ok(req.id, serde_json::json!({ "protocolVersion": "2024-11-05", "capabilities": { "tools": {} }, "serverInfo": { "name": "serialrun-mobile-mcp", "version": env!("CARGO_PKG_VERSION") } })),
            "tools/list" => McpResponse::ok(req.id, serde_json::json!({ "tools": [
                { "name": "list_ports", "description": "List serial ports", "inputSchema": { "type": "object", "properties": {} } },
                { "name": "connect", "description": "Connect to serial port", "inputSchema": { "type": "object", "properties": { "port": { "type": "string" }, "baud_rate": { "type": "integer" } }, "required": ["port"] } },
                { "name": "disconnect", "description": "Disconnect", "inputSchema": { "type": "object", "properties": {} } },
                { "name": "send", "description": "Send data", "inputSchema": { "type": "object", "properties": { "data": { "type": "string" }, "hex": { "type": "boolean" } }, "required": ["data"] } },
                { "name": "read", "description": "Read data", "inputSchema": { "type": "object", "properties": { "timeout_ms": { "type": "integer" } } } },
                { "name": "status", "description": "Connection status", "inputSchema": { "type": "object", "properties": {} } }
            ] })),
            "tools/call" => {
                let tool = req.params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = req.params.get("arguments").cloned().unwrap_or(serde_json::json!({}));
                match tool {
                    "list_ports" => match serialrun_core::SerialPort::list_ports() {
                        Ok(p) => McpResponse::ok(req.id, serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&p.iter().map(|x| serde_json::json!({"name":x.name,"desc":x.description})).collect::<Vec<_>>()).unwrap() }] })),
                        Err(e) => McpResponse::err(req.id, -1, e.to_string()),
                    },
                    "connect" => {
                        let pn = args.get("port").and_then(|v| v.as_str()).unwrap_or("");
                        let br = args.get("baud_rate").and_then(|v| v.as_u64()).unwrap_or(115200) as u32;
                        if pn.is_empty() { return McpResponse::err(req.id, -32602, "port required".into()); }
                        if let Ok(mut g) = self.shared_port.lock() {
                            if let Some(mut old) = g.take() { let _ = old.disconnect(); }
                            let cfg = serialrun_core::config::SerialConfig { port_name: pn.to_string(), baud_rate: br, timeout_ms: 100, ..Default::default() };
                            let mut port = serialrun_core::SerialPort::new(cfg);
                            match port.connect() {
                                Ok(()) => { *g = Some(port); self.connected_port = Some(pn.to_string()); self.notify(&format!("MCP connected: {} @ {} baud", pn, br)); McpResponse::ok(req.id, serde_json::json!({ "content": [{ "type": "text", "text": format!("Connected to {} at {} baud", pn, br) }] })) }
                                Err(e) => McpResponse::err(req.id, -1, e.to_string()),
                            }
                        } else { McpResponse::err(req.id, -1, "lock".into()) }
                    }
                    "disconnect" => {
                        if let Ok(mut g) = self.shared_port.lock() { if let Some(mut p) = g.take() { let _ = p.disconnect(); } }
                        self.connected_port = None; self.notify("MCP disconnected"); McpResponse::ok(req.id, serde_json::json!({ "content": [{ "type": "text", "text": "Disconnected" }] }))
                    }
                    "send" => {
                        let data = args.get("data").and_then(|v| v.as_str()).unwrap_or("");
                        let hex = args.get("hex").and_then(|v| v.as_bool()).unwrap_or(false);
                        if data.is_empty() { return McpResponse::err(req.id, -32602, "data required".into()); }
                        if let Ok(mut g) = self.shared_port.lock() {
                            if let Some(ref mut p) = *g {
                                let bytes = if hex { data.split_whitespace().filter_map(|s| u8::from_str_radix(s, 16).ok()).collect() } else { data.as_bytes().to_vec() };
                                match p.write(&bytes) { Ok(n) => { self.notify(&format!("TX: {} bytes: {}", n, data)); McpResponse::ok(req.id, serde_json::json!({ "content": [{ "type": "text", "text": format!("Sent {} bytes", n) }] })) } Err(e) => McpResponse::err(req.id, -1, e.to_string()) }
                            } else { McpResponse::err(req.id, -1, "not connected".into()) }
                        } else { McpResponse::err(req.id, -1, "lock".into()) }
                    }
                    "read" => {
                        let timeout = args.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(1000) as u64;
                        if let Ok(mut g) = self.shared_port.lock() {
                            if let Some(ref mut p) = *g {
                                let _ = p.set_timeout(std::time::Duration::from_millis(timeout));
                                let mut buf = vec![0u8; 4096];
                                match p.read(&mut buf) {
                                    Ok(n) => { let text = String::from_utf8_lossy(&buf[..n]).to_string(); self.notify(&format!("RX: {}", text)); McpResponse::ok(req.id, serde_json::json!({ "content": [{ "type": "text", "text": format!("{} bytes: {}", n, text) }] })) }
                                    Err(e) => McpResponse::err(req.id, -1, e.to_string()),
                                }
                            } else { McpResponse::err(req.id, -1, "not connected".into()) }
                        } else { McpResponse::err(req.id, -1, "lock".into()) }
                    }
                    "status" => { let s = self.connected_port.as_deref().unwrap_or("Disconnected"); McpResponse::ok(req.id, serde_json::json!({ "content": [{ "type": "text", "text": s }] })) }
                    _ => McpResponse::err(req.id, -32601, format!("unknown: {}", tool)),
                }
            }
            "notifications/initialized" => McpResponse::ok(req.id, serde_json::json!({})),
            _ => McpResponse::err(req.id, -32601, format!("unknown: {}", req.method)),
        }
    }
}

fn handle_client(stream: TcpStream, state: Arc<Mutex<McpState>>) {
    let reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = stream;
    for line in reader.lines() {
        let line = match line { Ok(l) => l, Err(_) => break };
        let line = line.trim().to_string();
        if line.is_empty() { continue; }
        let req: McpRequest = match serde_json::from_str(&line) { Ok(r) => r, Err(e) => { let _ = writeln!(writer, "{}", serde_json::to_string(&McpResponse::err(None, -32700, format!("parse: {}", e))).unwrap()); let _ = writer.flush(); continue; } };
        let mut s = state.lock().unwrap();
        let resp = s.handle_request(req);
        drop(s);
        let _ = writeln!(writer, "{}", serde_json::to_string(&resp).unwrap());
        let _ = writer.flush();
    }
}

pub fn start_mcp_server(state: Arc<Mutex<McpState>>, port: u16) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let listener = match TcpListener::bind(format!("127.0.0.1:{}", port)) { Ok(l) => l, Err(e) => { eprintln!("MCP bind error: {}", e); return; } };
        eprintln!("MCP server on :{}", port);
        for stream in listener.incoming() { if let Ok(stream) = stream { let state = state.clone(); std::thread::spawn(move || handle_client(stream, state)); } }
    })
}
