use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

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

struct SerialRunMcp {
    port: Option<serialrun_core::SerialPort>,
    connected_port: Option<String>,
}

impl SerialRunMcp {
    fn new() -> Self {
        Self { port: None, connected_port: None }
    }

    fn handle_request(&mut self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "initialize" => {
                let result = serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "serialrun-mcp",
                        "version": "0.1.0"
                    }
                });
                McpResponse::success(request.id, result)
            }
            "tools/list" => {
                let tools = serde_json::json!({
                    "tools": [
                        {
                            "name": "list_ports",
                            "description": "List available serial ports",
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        },
                        {
                            "name": "connect",
                            "description": "Connect to a serial port",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "port": { "type": "string", "description": "Port name (e.g., COM1, /dev/ttyUSB0)" },
                                    "baud_rate": { "type": "integer", "description": "Baud rate (default: 115200)" }
                                },
                                "required": ["port"]
                            }
                        },
                        {
                            "name": "disconnect",
                            "description": "Disconnect from current serial port",
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        },
                        {
                            "name": "send",
                            "description": "Send data to serial port",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "data": { "type": "string", "description": "Data to send (text or hex string)" },
                                    "hex": { "type": "boolean", "description": "If true, data is interpreted as hex" }
                                },
                                "required": ["data"]
                            }
                        },
                        {
                            "name": "read",
                            "description": "Read data from serial port",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "timeout_ms": { "type": "integer", "description": "Read timeout in milliseconds (default: 1000)" },
                                    "max_bytes": { "type": "integer", "description": "Maximum bytes to read (default: 1024)" }
                                }
                            }
                        },
                        {
                            "name": "send_command",
                            "description": "Send a command and wait for response",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "command": { "type": "string", "description": "Command to send" },
                                    "timeout_ms": { "type": "integer", "description": "Response timeout in milliseconds (default: 1000)" }
                                },
                                "required": ["command"]
                            }
                        }
                    ]
                });
                McpResponse::success(request.id, tools)
            }
            "tools/call" => {
                let tool_name = request.params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let arguments = request.params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

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

                        if port_name.is_empty() {
                            return McpResponse::error(request.id, -32602, "Port name is required".into());
                        }

                        let config = serialrun_core::config::SerialConfig {
                            port_name: port_name.to_string(),
                            baud_rate,
                            ..Default::default()
                        };
                        let mut port = serialrun_core::SerialPort::new(config);
                        match port.connect() {
                            Ok(()) => {
                                self.port = Some(port);
                                self.connected_port = Some(port_name.to_string());
                                McpResponse::success(request.id, serde_json::json!({
                                    "content": [{ "type": "text", "text": format!("Connected to {} at {} baud", port_name, baud_rate) }]
                                }))
                            }
                            Err(e) => McpResponse::error(request.id, -1, e.to_string())
                        }
                    }
                    "disconnect" => {
                        if let Some(mut port) = self.port.take() {
                            let _ = port.disconnect();
                        }
                        self.connected_port = None;
                        McpResponse::success(request.id, serde_json::json!({
                            "content": [{ "type": "text", "text": "Disconnected" }]
                        }))
                    }
                    "send" => {
                        let data = arguments.get("data").and_then(|v| v.as_str()).unwrap_or("");
                        let hex = arguments.get("hex").and_then(|v| v.as_bool()).unwrap_or(false);

                        if data.is_empty() {
                            return McpResponse::error(request.id, -32602, "Data is required".into());
                        }

                        let Some(ref mut port) = self.port else {
                            return McpResponse::error(request.id, -1, "Not connected".into());
                        };

                        let bytes = if hex {
                            data.split_whitespace()
                                .filter_map(|s| u8::from_str_radix(s, 16).ok())
                                .collect::<Vec<_>>()
                        } else {
                            data.as_bytes().to_vec()
                        };

                        match port.write(&bytes) {
                            Ok(n) => McpResponse::success(request.id, serde_json::json!({
                                "content": [{ "type": "text", "text": format!("Sent {} bytes", n) }]
                            })),
                            Err(e) => McpResponse::error(request.id, -1, e.to_string())
                        }
                    }
                    "read" => {
                        let _timeout_ms = arguments.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(1000) as u64;
                        let max_bytes = arguments.get("max_bytes").and_then(|v| v.as_u64()).unwrap_or(1024) as usize;

                        let Some(ref mut port) = self.port else {
                            return McpResponse::error(request.id, -1, "Not connected".into());
                        };

                        let mut buf = vec![0u8; max_bytes];
                        match port.read(&mut buf) {
                            Ok(n) => {
                                let data_hex = buf[..n].iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                                let data_text = String::from_utf8_lossy(&buf[..n]).to_string();
                                McpResponse::success(request.id, serde_json::json!({
                                    "content": [{ "type": "text", "text": format!("Read {} bytes\nHEX: {}\nText: {}", n, data_hex, data_text) }]
                                }))
                            }
                            Err(e) => McpResponse::error(request.id, -1, e.to_string())
                        }
                    }
                    "send_command" => {
                        let command = arguments.get("command").and_then(|v| v.as_str()).unwrap_or("");
                        let _timeout_ms = arguments.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(1000) as u64;

                        if command.is_empty() {
                            return McpResponse::error(request.id, -32602, "Command is required".into());
                        }

                        let Some(ref mut port) = self.port else {
                            return McpResponse::error(request.id, -1, "Not connected".into());
                        };

                        // Send command
                        let mut cmd_bytes = command.as_bytes().to_vec();
                        if !command.ends_with("\r\n") && !command.ends_with('\n') && !command.ends_with('\r') {
                            cmd_bytes.extend_from_slice(b"\r\n");
                        }

                        if let Err(e) = port.write(&cmd_bytes) {
                            return McpResponse::error(request.id, -1, e.to_string());
                        }

                        // Wait and read response
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        let mut buf = vec![0u8; 4096];
                        match port.read(&mut buf) {
                            Ok(n) => {
                                let response = String::from_utf8_lossy(&buf[..n]).to_string();
                                McpResponse::success(request.id, serde_json::json!({
                                    "content": [{ "type": "text", "text": response }]
                                }))
                            }
                            Err(e) => McpResponse::error(request.id, -1, e.to_string())
                        }
                    }
                    _ => McpResponse::error(request.id, -32601, format!("Unknown tool: {}", tool_name))
                }
            }
            "notifications/initialized" => {
                // Client notification, no response needed
                McpResponse::success(request.id, serde_json::json!({}))
            }
            _ => McpResponse::error(request.id, -32601, format!("Unknown method: {}", request.method))
        }
    }
}

pub fn run_mcp_server() {
    let mcp = Arc::new(Mutex::new(SerialRunMcp::new()));

    // Try to listen on port 9527
    let listener = match TcpListener::bind("127.0.0.1:9527") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("MCP server: Failed to bind to port 9527: {}", e);
            return;
        }
    };

    eprintln!("MCP server listening on 127.0.0.1:9527");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mcp = mcp.clone();
                std::thread::spawn(move || {
                    handle_client(stream, mcp);
                });
            }
            Err(e) => {
                eprintln!("MCP server: Connection failed: {}", e);
            }
        }
    }
}

fn handle_client(stream: TcpStream, mcp: Arc<Mutex<SerialRunMcp>>) {
    let reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = stream;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let request: McpRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                let response = McpResponse::error(None, -32700, format!("Parse error: {}", e));
                let _ = writeln!(writer, "{}", serde_json::to_string(&response).unwrap());
                let _ = writer.flush();
                continue;
            }
        };

        let mut mcp = mcp.lock().unwrap();
        let response = mcp.handle_request(request);

        let _ = writeln!(writer, "{}", serde_json::to_string(&response).unwrap());
        let _ = writer.flush();
    }
}
