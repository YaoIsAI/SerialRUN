use crate::state::{AppState, Language, T};
use crate::theme;
use eframe::egui;

const MCP_PROMPT_ZH: &str = r#"SerialRUN MCP 服务器使用指南

SerialRUN 是一个串口调试助手，内置 MCP 服务器，允许 AI 助手通过 TCP 连接控制串口设备。
RX 数据由后台连续监听自动捕获并存入缓冲区，read 操作从缓冲区取数据，不会阻塞串口。

## 连接信息
- MCP 服务器地址：127.0.0.1
- 端口：9527
- 协议：JSON-RPC over TCP

## 可用工具（19 个）

### 1. list_ports
列出所有可用的串口设备。
```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_ports","arguments":{}}}
```

### 2. connect
连接到指定串口（支持完整串口配置）。
```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"connect","arguments":{"port":"COM3","baud_rate":115200,"data_bits":8,"stop_bits":1,"parity":"None","flow_control":"None"}}}
```

### 3. disconnect
断开当前串口连接。
```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"disconnect","arguments":{}}}
```

### 4. send
发送数据到串口（支持文本或十六进制），不等待响应。
```json
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"send","arguments":{"data":"AT\r\n","hex":false}}}
```

### 5. read
从 RX 缓冲区读取数据（后台连续监听自动捕获，非阻塞）。
```json
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"read","arguments":{"timeout_ms":2000}}}
```

### 6. send_command
发送命令并从缓冲区读取响应（写入-等待-读取模式）。推荐用于 AT 命令交互。
```json
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"send_command","arguments":{"command":"AT","timeout_ms":3000}}}
```

### 7. modbus_read
读取 Modbus RTU 保持寄存器（支持工程值转换）。
```json
{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"modbus_read","arguments":{"slave_id":1,"address":0,"quantity":10,"scale":1.0,"offset":0.0,"unit":""}}}
```

### 8. modbus_write
写入 Modbus RTU 保持寄存器（值范围 0-65535）。
```json
{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"modbus_write","arguments":{"slave_id":1,"address":0,"value":100}}}
```

### 9. plc_read
读取 PLC 预设品牌的所有寄存器（支持 Siemens/Mitsubishi/Delta/Omron）。
```json
{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"plc_read","arguments":{"brand":"Siemens","slave_id":1}}}
```

### 10. plc_write
写入 PLC 寄存器（按地址，值范围 0-65535）。
```json
{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"plc_write","arguments":{"brand":"Siemens","slave_id":1,"address":0,"value":25.0}}}
```

### 11. status
查看连接状态、字节统计、MCP 服务器信息。
```json
{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"status","arguments":{}}}
```

### 12. get_access_log
查看 MCP 访问日志（客户端 IP、工具调用记录、时间戳）。
```json
{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"get_access_log","arguments":{"limit":50}}}
```

### 13. get_config
获取所有 UI 设置或指定设置的值。不传 key 返回全部设置。
```json
{"jsonrpc":"2.0","id":13,"method":"tools/call","params":{"name":"get_config","arguments":{"key":"hex_mode"}}}
```

### 14. set_config
更新 UI 设置。修改串口参数（baud_rate 等）后需断开重连才生效。
```json
{"jsonrpc":"2.0","id":14,"method":"tools/call","params":{"name":"set_config","arguments":{"key":"hex_mode","value":true}}}
```

### 15. get_device_info
获取设备标识信息：连接状态、活跃客户端、服务器版本、协议信息。
```json
{"jsonrpc":"2.0","id":15,"method":"tools/call","params":{"name":"get_device_info","arguments":{}}}
```

### 16. clear_buffers
清空 TX 和 RX 串口缓冲区。用于清除残留数据。
```json
{"jsonrpc":"2.0","id":16,"method":"tools/call","params":{"name":"clear_buffers","arguments":{}}}
```

### 17. set_dtr
设置 DTR（数据终端就绪）硬件信号，立即生效。常用于复位 Arduino/ESP32。
```json
{"jsonrpc":"2.0","id":17,"method":"tools/call","params":{"name":"set_dtr","arguments":{"value":true}}}
```

### 18. set_rts
设置 RTS（请求发送）硬件信号，立即生效。
```json
{"jsonrpc":"2.0","id":18,"method":"tools/call","params":{"name":"set_rts","arguments":{"value":false}}}
```

### 19. get_config_keys
列出所有可用的配置键及其类型、有效值、是否需要重连。
```json
{"jsonrpc":"2.0","id":19,"method":"tools/call","params":{"name":"get_config_keys","arguments":{}}}
```

## 使用示例

1. 列出端口：`list_ports`
2. 连接设备：`connect` 到 COM3，波特率 115200
3. 发送 AT 命令：`send_command` "AT"
4. 读取响应：`read`（从缓冲区取数据）
5. Modbus 读取：`modbus_read` 地址 0，数量 10
6. 查看设置：`get_config`（返回所有 UI 设置）
7. 修改设置：`set_config` key="hex_mode" value=true
8. 查看状态：`status`
9. 断开连接：`disconnect`

## 可配置项（get_config / set_config）

**需要重连才生效的串口参数：**

| 键 | 类型 | 说明 |
|---|------|------|
| `baud_rate` | integer | 波特率 (9600-921600) |
| `data_bits` | integer | 数据位 (5-8) |
| `stop_bits` | integer | 停止位 (1-2) |
| `parity` | string | 校验 (None/Odd/Even) |
| `flow_control` | string | 流控 (None/Software/Hardware) |

**立即生效的设置：**

| 键 | 类型 | 说明 |
|---|------|------|
| `hex_mode` | bool | HEX 输入模式 |
| `show_timestamp` | bool | 显示时间戳 |
| `auto_scroll` | bool | 自动滚动 |
| `auto_send_enabled` | bool | 自动发送开关 |
| `auto_send_interval_ms` | integer | 自动发送间隔 (ms) |
| `keep_input` | bool | 保留输入 |
| `line_ending` | string | 行尾符 (None/CR/LF/CRLF) |
| `dtr` | bool | DTR 信号 |
| `rts` | bool | RTS 信号 |
| `auto_reply_enabled` | bool | 自动回复开关 |
| `auto_reply_pattern` | string | 自动回复匹配模式 |
| `auto_reply_response` | string | 自动回复内容 |
| `rx_auto_aggregate` | bool | 自动聚合接收数据 |
| `rx_aggregate_ms` | integer | 聚合等待时间 (ms) |

## 注意事项
- RX 数据由后台连续监听自动捕获，read 从缓冲区取数据，不会阻塞串口
- send_command 发送后自动等待响应，推荐用于 AT 命令交互
- 发送文本数据时会自动添加 \r\n 结束符
- 十六进制数据用空格分隔，如 "48 65 6C 6C 0F"
- Modbus/PLC 写入值范围 0-65535，超出会返回错误
- Modbus 读取数量限制 1-125
- AI 助手可使用 tools/list 获取所有可用工具及其参数说明
- 使用 get_config_keys 可查看所有配置键及其有效值
- set_dtr/set_rts 立即生效，无需重连
- clear_buffers 用于清除缓冲区残留数据
- 快捷指令：输入框右侧点击 + 可保存常用指令，点击 ▶ 展开快捷指令栏，右键可删除"#;

const MCP_PROMPT_EN: &str = r#"SerialRUN MCP Server Guide

SerialRUN is a serial port debugging assistant with a built-in MCP server, allowing AI assistants to control serial devices via TCP connection.
RX data is automatically captured by background continuous monitoring and stored in a buffer. The read operation retrieves data from the buffer without blocking the serial port.

## Connection Info
- MCP Server Address: 127.0.0.1
- Port: 9527
- Protocol: JSON-RPC over TCP

## Available Tools (19)

### 1. list_ports
List all available serial ports.
```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_ports","arguments":{}}}
```

### 2. connect
Connect to a serial port (full serial config supported).
```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"connect","arguments":{"port":"COM3","baud_rate":115200,"data_bits":8,"stop_bits":1,"parity":"None","flow_control":"None"}}}
```

### 3. disconnect
Disconnect from current serial port.
```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"disconnect","arguments":{}}}
```

### 4. send
Send data to serial port (text or hex), no response wait.
```json
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"send","arguments":{"data":"AT\r\n","hex":false}}}
```

### 5. read
Read data from RX buffer (auto-captured by background monitor, non-blocking).
```json
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"read","arguments":{"timeout_ms":2000}}}
```

### 6. send_command
Send command and read response from buffer (write-wait-read mode). Recommended for AT command interaction.
```json
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"send_command","arguments":{"command":"AT","timeout_ms":3000}}}
```

### 7. modbus_read
Read Modbus RTU holding registers (with engineering value conversion).
```json
{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"modbus_read","arguments":{"slave_id":1,"address":0,"quantity":10,"scale":1.0,"offset":0.0,"unit":""}}}
```

### 8. modbus_write
Write a Modbus RTU holding register (value range 0-65535).
```json
{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"modbus_write","arguments":{"slave_id":1,"address":0,"value":100}}}
```

### 9. plc_read
Read all registers from a PLC preset (Siemens/Mitsubishi/Delta/Omron).
```json
{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"plc_read","arguments":{"brand":"Siemens","slave_id":1}}}
```

### 10. plc_write
Write to a PLC register by address (value range 0-65535).
```json
{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"plc_write","arguments":{"brand":"Siemens","slave_id":1,"address":0,"value":25.0}}}
```

### 11. status
View connection status, byte counters, MCP server info.
```json
{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"status","arguments":{}}}
```

### 12. get_access_log
View MCP access log (client IPs, tool calls, timestamps).
```json
{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"get_access_log","arguments":{"limit":50}}}
```

### 13. get_config
Get all UI settings or a specific setting value. Omit key to return all settings.
```json
{"jsonrpc":"2.0","id":13,"method":"tools/call","params":{"name":"get_config","arguments":{"key":"hex_mode"}}}
```

### 14. set_config
Update a UI setting. Changing serial params requires disconnect+connect to take effect.
```json
{"jsonrpc":"2.0","id":14,"method":"tools/call","params":{"name":"set_config","arguments":{"key":"hex_mode","value":true}}}
```

### 15. get_device_info
Get device identification: connection status, active clients, server version, protocol info.
```json
{"jsonrpc":"2.0","id":15,"method":"tools/call","params":{"name":"get_device_info","arguments":{}}}
```

### 16. clear_buffers
Flush both TX and RX serial buffers. Useful before starting a new measurement sequence.
```json
{"jsonrpc":"2.0","id":16,"method":"tools/call","params":{"name":"clear_buffers","arguments":{}}}
```

### 17. set_dtr
Set DTR (Data Terminal Ready) hardware signal. Takes effect immediately. Common use: toggle DTR to reset Arduino/ESP32.
```json
{"jsonrpc":"2.0","id":17,"method":"tools/call","params":{"name":"set_dtr","arguments":{"value":true}}}
```

### 18. set_rts
Set RTS (Request To Send) hardware signal. Takes effect immediately.
```json
{"jsonrpc":"2.0","id":18,"method":"tools/call","params":{"name":"set_rts","arguments":{"value":false}}}
```

### 19. get_config_keys
List all available configuration keys with their types, valid values, and whether they require reconnect.
```json
{"jsonrpc":"2.0","id":19,"method":"tools/call","params":{"name":"get_config_keys","arguments":{}}}
```

## Usage Examples

1. List ports: `list_ports`
2. Connect device: `connect` to COM3 at 115200 baud
3. Send AT command: `send_command` "AT"
4. Read response: `read` (from buffer)
5. Modbus read: `modbus_read` address 0, quantity 10
6. Get settings: `get_config` (returns all UI settings)
7. Change setting: `set_config` key="hex_mode" value=true
8. Check status: `status`
9. Disconnect: `disconnect`

## Configurable Settings (get_config / set_config)

**Serial params (require reconnect to take effect):**

| Key | Type | Description |
|-----|------|-------------|
| `baud_rate` | integer | Baud rate (9600-921600) |
| `data_bits` | integer | Data bits (5-8) |
| `stop_bits` | integer | Stop bits (1-2) |
| `parity` | string | Parity (None/Odd/Even) |
| `flow_control` | string | Flow control (None/Software/Hardware) |

**Immediate effect settings:**

| Key | Type | Description |
|-----|------|-------------|
| `hex_mode` | bool | HEX input mode |
| `show_timestamp` | bool | Show timestamps |
| `auto_scroll` | bool | Auto scroll |
| `auto_send_enabled` | bool | Auto send on/off |
| `auto_send_interval_ms` | integer | Auto send interval (ms) |
| `keep_input` | bool | Keep input after send |
| `line_ending` | string | Line ending (None/CR/LF/CRLF) |
| `dtr` | bool | DTR signal |
| `rts` | bool | RTS signal |
| `auto_reply_enabled` | bool | Auto reply on/off |
| `auto_reply_pattern` | string | Auto reply match pattern |
| `auto_reply_response` | string | Auto reply response |
| `rx_auto_aggregate` | bool | Auto-aggregate received data |
| `rx_aggregate_ms` | integer | Aggregate wait time (ms) |

## Notes
- RX data is auto-captured by background monitor; read retrieves from buffer without blocking
- send_command sends then waits for response — recommended for AT command interaction
- Text data automatically appends \r\n terminator
- Hex data uses space separator, e.g., "48 65 6C 6C 0F"
- Modbus/PLC write value range 0-65535, out of range returns error
- Modbus read quantity limited to 1-125
- AI assistants can use tools/list to discover all available tools and their parameters
- Use get_config_keys to list all config keys with valid values
- set_dtr/set_rts take effect immediately, no reconnect needed
- clear_buffers flushes stale data from TX and RX buffers
- Quick Commands: click + next to input to save, click ▶ to expand, right-click to delete"#;

pub fn render_help_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    egui::ScrollArea::vertical().max_height(500.0).show(ui, |ui| {
        let c = theme::get_colors(state.theme);

        ui.add_space(4.0);
        if let Some(handle) = get_logo_texture(ui.ctx()) {
            let tex_size = handle.size_vec2();
            let max_width = 420.0;
            let scale = (max_width / tex_size.x).min(1.0);
            let desired = egui::vec2(tex_size.x * scale, tex_size.y * scale);
            ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(handle.id(), desired)));
        }
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        let md = match lang {
            Language::Chinese => &state.help_content_zh,
            Language::English => &state.help_content_en,
        };
        render_markdown(ui, md);

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // MCP section with interactive copy button
        ui.heading("MCP 服务器 / MCP Server");
        ui.add_space(4.0);
        ui.label(if lang == Language::Chinese {
            "SerialRUN 内置 MCP 服务器（19 个工具），支持 AI 助手通过 TCP 控制串口设备。RX 数据由后台连续监听自动捕获，send_command 发送后响应自动在缓冲区中。"
        } else {
            "SerialRUN includes a built-in MCP server (19 tools) for AI assistants to control serial devices via TCP. RX data is auto-captured by background monitor; send_command responses are available in the buffer immediately."
        });
        ui.add_space(4.0);

        let mcp_addr = if state.mcp_bind_lan {
            let local_ip = get_local_ip_for_help().unwrap_or_else(|| "0.0.0.0".into());
            format!("{}:{}", local_ip, state.mcp_port)
        } else {
            format!("127.0.0.1:{}", state.mcp_port)
        };
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("地址/Address:").strong());
            ui.label(egui::RichText::new(&mcp_addr).monospace().color(egui::Color32::from_rgb(80, 160, 230)).strong());
        });
        ui.add_space(8.0);

        let copy_text = if lang == Language::Chinese { MCP_PROMPT_ZH } else { MCP_PROMPT_EN };

        // Reset copied state after 2 seconds
        let now = chrono::Utc::now().timestamp_millis();
        if state.copied && now - state.copied_time > 2000 {
            state.copied = false;
        }

        let c = theme::get_colors(state.theme);
        let btn_color = if state.copied { c.btn_mcp_copied } else { c.btn_mcp_copy };
        let copy_label = if state.copied { T::copied(lang) } else { T::copy_mcp_guide(lang) };
        let btn = ui.add(egui::Button::new(
            egui::RichText::new(copy_label).color(egui::Color32::WHITE).strong().size(14.0)
        ).fill(btn_color).min_size(egui::vec2(300.0, 36.0)));
        if btn.clicked() {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(copy_text.to_string());
                state.copied = true;
                state.copied_time = now;
            }
        }

        ui.add_space(4.0);
        ui.label(egui::RichText::new(T::copy_hint(lang)).weak().small());

        // ── Buy me a coffee ──
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        let coffee_text = if lang == Language::Chinese {
            "如果 SerialRUN 对你有帮助，请作者喝杯咖啡吧！"
        } else {
            "If SerialRUN helps you, buy the author a coffee!"
        };
        ui.label(egui::RichText::new(coffee_text).strong().size(14.0));
        ui.add_space(4.0);

        ui.label(egui::RichText::new("Author: Yao").size(13.0));
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("GitHub: ").size(13.0));
            ui.label(egui::RichText::new("YaoIsAI").size(13.0).color(egui::Color32::from_rgb(80, 160, 255)).strong());
        });
        ui.add_space(2.0);
        ui.label(egui::RichText::new("WeChat Pay / \u{5FAE}\u{4FE1}\u{652F}\u{4ED8}").size(13.0));
        ui.add_space(6.0);

        // QR code image
        if let Some(handle) = get_qr_texture(ui.ctx()) {
            let max_width = 180.0;
            let tex_size = handle.size_vec2();
            let scale = max_width / tex_size.x;
            let desired = egui::vec2(tex_size.x * scale, tex_size.y * scale);
            ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(handle.id(), desired)));
        }
    });
}

static QR_IMAGE_BYTES: &[u8] = include_bytes!("../../../../assets/wechat_pay_qr.jpg");

use std::sync::OnceLock;
static QR_HANDLE: OnceLock<Option<egui::TextureHandle>> = OnceLock::new();

fn get_qr_texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let entry = QR_HANDLE.get_or_init(|| {
        let img = image::load_from_memory(QR_IMAGE_BYTES).ok()?;
        let rgba = img.to_rgba8();
        let w = rgba.width() as usize;
        let h = rgba.height() as usize;
        let pixels = rgba.into_raw();
        let color_image = egui::ColorImage::from_rgba_unmultiplied([w, h], &pixels);
        Some(ctx.load_texture("wechat_pay_qr", color_image, egui::TextureOptions::default()))
    });

    entry.as_ref().cloned()
}

fn get_local_ip_for_help() -> Option<String> {
    crate::util::get_local_ip()
}

static LOGO_HANDLE: OnceLock<Option<egui::TextureHandle>> = OnceLock::new();

fn get_logo_texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let entry = LOGO_HANDLE.get_or_init(|| {
        const S: u32 = 10; // bigger pixel blocks
        const COLS: usize = 7;
        const ROWS: usize = 10;

        // Bold letter bitmaps (10 rows x 7 cols)
        const S_L: [[u8; COLS]; ROWS] = [
            [0,1,1,1,1,1,0],[1,1,0,0,0,0,0],[1,1,0,0,0,0,0],[1,1,0,0,0,0,0],
            [0,1,1,1,1,0,0],[0,0,0,0,1,1,0],[0,0,0,0,1,1,0],[0,0,0,0,1,1,0],
            [1,1,0,0,1,1,0],[0,1,1,1,1,0,0],
        ];
        const E_L: [[u8; COLS]; ROWS] = [
            [1,1,1,1,1,1,0],[1,1,0,0,0,0,0],[1,1,0,0,0,0,0],[1,1,0,0,0,0,0],
            [1,1,1,1,1,0,0],[1,1,0,0,0,0,0],[1,1,0,0,0,0,0],[1,1,0,0,0,0,0],
            [1,1,0,0,0,0,0],[1,1,1,1,1,1,0],
        ];
        const R_L: [[u8; COLS]; ROWS] = [
            [1,1,1,1,1,1,0],[1,1,0,0,0,1,1],[1,1,0,0,0,1,1],[1,1,0,0,0,1,1],
            [1,1,1,1,1,1,0],[1,1,0,1,1,0,0],[1,1,0,0,1,1,0],[1,1,0,0,0,1,1],
            [1,1,0,0,0,1,1],[1,1,0,0,0,0,1],
        ];
        const I_L: [[u8; COLS]; ROWS] = [
            [0,1,1,1,1,1,0],[0,0,1,1,0,0,0],[0,0,1,1,0,0,0],[0,0,1,1,0,0,0],
            [0,0,1,1,0,0,0],[0,0,1,1,0,0,0],[0,0,1,1,0,0,0],[0,0,1,1,0,0,0],
            [0,0,1,1,0,0,0],[0,1,1,1,1,1,0],
        ];
        const A_L: [[u8; COLS]; ROWS] = [
            [0,0,1,1,1,0,0],[0,1,1,0,1,1,0],[1,1,0,0,0,1,1],[1,1,0,0,0,1,1],
            [1,1,0,0,0,1,1],[1,1,1,1,1,1,1],[1,1,0,0,0,1,1],[1,1,0,0,0,1,1],
            [1,1,0,0,0,1,1],[1,1,0,0,0,1,1],
        ];
        const L_L: [[u8; COLS]; ROWS] = [
            [1,1,0,0,0,0,0],[1,1,0,0,0,0,0],[1,1,0,0,0,0,0],[1,1,0,0,0,0,0],
            [1,1,0,0,0,0,0],[1,1,0,0,0,0,0],[1,1,0,0,0,0,0],[1,1,0,0,0,0,0],
            [1,1,0,0,0,0,0],[1,1,1,1,1,1,0],
        ];
        const U_L: [[u8; COLS]; ROWS] = [
            [1,1,0,0,0,1,1],[1,1,0,0,0,1,1],[1,1,0,0,0,1,1],[1,1,0,0,0,1,1],
            [1,1,0,0,0,1,1],[1,1,0,0,0,1,1],[1,1,0,0,0,1,1],[1,1,0,0,0,1,1],
            [1,1,0,0,0,1,1],[0,1,1,1,1,1,0],
        ];
        const N_L: [[u8; COLS]; ROWS] = [
            [1,1,0,0,0,1,1],[1,1,1,0,0,1,1],[1,1,1,1,0,1,1],[1,1,0,1,1,1,1],
            [1,1,0,0,1,1,1],[1,1,0,0,0,1,1],[1,1,0,0,0,1,1],[1,1,0,0,0,1,1],
            [1,1,0,0,0,1,1],[1,1,0,0,0,1,1],
        ];

        // Sub-text: "Pure Rust Driven" (5 rows per char)
        const CHARS: [[[u8; 4]; 5]; 16] = [
            [[1,1,1,0],[1,0,0,1],[1,1,1,0],[1,0,0,0],[1,0,0,0]], // P
            [[0,0,0,0],[1,0,0,1],[1,0,0,1],[1,0,0,1],[0,1,1,0]], // u
            [[0,0,0,0],[1,0,1,0],[1,1,0,0],[1,0,0,0],[1,0,0,0]], // r
            [[0,0,0,0],[0,1,1,0],[1,0,0,1],[1,1,1,0],[0,1,1,1]], // e
            [[0,0,0,0],[0,0,0,0],[0,0,0,0],[0,0,0,0],[0,0,0,0]], // space
            [[1,1,1,0],[1,0,0,1],[1,1,1,0],[1,0,1,0],[1,0,0,1]], // R
            [[0,0,0,0],[1,0,0,1],[1,0,0,1],[1,0,0,1],[0,1,1,0]], // u
            [[0,0,0,0],[0,1,1,1],[1,1,0,0],[0,0,1,0],[1,1,1,0]], // s
            [[0,1,0,0],[1,1,1,0],[0,1,0,0],[0,1,0,0],[0,0,1,0]], // t
            [[0,0,0,0],[0,0,0,0],[0,0,0,0],[0,0,0,0],[0,0,0,0]], // space
            [[1,1,1,0],[1,0,0,1],[1,0,0,1],[1,0,0,1],[1,1,1,0]], // D
            [[0,0,0,0],[1,0,1,0],[1,1,0,0],[1,0,0,0],[1,0,0,0]], // r
            [[0,1,0,0],[0,0,0,0],[0,1,0,0],[0,1,0,0],[0,1,0,0]], // i
            [[0,0,0,0],[1,0,0,1],[1,0,0,1],[0,1,1,0],[0,1,1,0]], // v
            [[0,0,0,0],[0,1,1,0],[1,0,0,1],[1,1,1,0],[0,1,1,1]], // e
            [[0,0,0,0],[1,0,1,0],[1,1,0,1],[1,0,0,1],[1,0,0,1]], // n
        ];

        let letters: [&[[u8; COLS]; ROWS]; 9] = [
            &S_L, &E_L, &R_L, &I_L, &A_L, &L_L, &R_L, &U_L, &N_L,
        ];
        let gap: u32 = 3;
        let space_gap: u32 = S * 2;
        let text_w = COLS as u32 * S * 9 + gap * 8 + space_gap;
        let letter_h: u32 = ROWS as u32 * S;

        // REVERSE shear: shift LEFT as row increases = top leans right, bottom leans left
        // This creates a \ shape (top-right to bottom-left lean)
        let shear: i32 = S as i32; // full block per row

        let pad: u32 = 40;
        let bottom_text_h: u32 = 24;

        // Image size: need extra space on LEFT for the shear
        let shear_extra = (ROWS as u32 - 1) * S;
        let img_w = text_w + shear_extra + pad * 2;
        let img_h = letter_h + bottom_text_h + pad * 2;

        let mut img = image::RgbaImage::new(img_w, img_h);
        for p in img.pixels_mut() { *p = image::Rgba([0, 0, 0, 0]); }

        // Bold green gradient: thicker feel with brighter colors
        let colors: [image::Rgba<u8>; ROWS] = [
            image::Rgba([200,255,200,255]), image::Rgba([160,230,160,255]),
            image::Rgba([130,210,130,255]), image::Rgba([100,195,100,255]),
            image::Rgba([76,175,80,255]),  image::Rgba([60,155,65,255]),
            image::Rgba([45,130,50,255]),  image::Rgba([35,110,40,255]),
            image::Rgba([28,90,32,255]),   image::Rgba([20,70,24,255]),
        ];

        let sub_color = image::Rgba([100, 180, 100, 200]);

        let mut put_rect = |x0: i32, y0: i32, rw: u32, rh: u32, color: image::Rgba<u8>| {
            for dy in 0..rh { for dx in 0..rw {
                let px = x0 + dx as i32;
                let py = y0 + dy as i32;
                if px >= 0 && py >= 0 && (px as u32) < img_w && (py as u32) < img_h {
                    img.put_pixel(px as u32, py as u32, color);
                }
            }}
        };

        // Draw main letters with REVERSE 45-degree shear (\ direction)
        // Top row gets maximum right offset, bottom row gets zero offset
        let base_x = pad as i32 + shear_extra as i32; // start from right side
        let mut cx = base_x;
        for (li, &letter) in letters.iter().enumerate() {
            if li == 6 { cx += space_gap as i32; }
            for row in 0..ROWS {
                // Reverse: top rows (row=0) get max offset, bottom rows (row=9) get 0
                let shear_off = ((ROWS as i32 - 1 - row as i32) * shear) as i32;
                for col in 0..COLS {
                    if letter[row][col] == 1 {
                        let px = cx + col as i32 * S as i32 + shear_off;
                        let py = pad as i32 + row as i32 * S as i32;
                        put_rect(px, py, S, S, colors[row]);
                    }
                }
            }
            cx += COLS as i32 * S as i32 + gap as i32;
        }

        // Bottom text "Pure Rust Driven" centered
        let char_w: i32 = 4;
        let char_gap: i32 = 1;
        let sub_scale: i32 = 3;
        let bottom_total = CHARS.len() as i32 * (char_w + char_gap) * sub_scale;
        let mut bx = ((img_w as i32 - bottom_total) / 2) as i32;
        let by = (pad + letter_h + 10) as i32;

        for &ch in CHARS.iter() {
            for row in 0..5i32 {
                for col in 0..4i32 {
                    if ch[row as usize][col as usize] == 1 {
                        put_rect(bx + col * sub_scale, by + row * sub_scale, sub_scale as u32, sub_scale as u32, sub_color);
                    }
                }
            }
            bx += (char_w + char_gap) * sub_scale;
        }

        let rgba = img.into_raw();
        let color_image = egui::ColorImage::from_rgba_unmultiplied([img_w as usize, img_h as usize], &rgba);
        Some(ctx.load_texture("serialrun_logo", color_image, egui::TextureOptions::default()))
    });
    entry.as_ref().cloned()
}


// ── Markdown renderer ──

enum MdBlock<'a> {
    Heading(u8, &'a str),       // level, text
    Bullet(&'a str),            // text after "- "
    Numbered(&'a str),          // full line like "1. xxx"
    Paragraph(&'a str),
    CodeBlock(Vec<&'a str>),    // lines inside ```
    Table(Vec<Vec<&'a str>>),   // rows → cells
    Hr,                         // ---
}

fn parse_md(text: &str) -> Vec<MdBlock<'_>> {
    let lines: Vec<&str> = text.lines().collect();
    let mut blocks = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Fenced code block
        if trimmed.starts_with("```") {
            i += 1;
            let mut code_lines = Vec::new();
            while i < lines.len() && !lines[i].trim().starts_with("```") {
                code_lines.push(lines[i]);
                i += 1;
            }
            blocks.push(MdBlock::CodeBlock(code_lines));
            i += 1; // skip closing ```
            continue;
        }

        // Horizontal rule
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            blocks.push(MdBlock::Hr);
            i += 1;
            continue;
        }

        // Empty line
        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        // Table: detect by | prefix
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            let mut table_rows = Vec::new();
            while i < lines.len() {
                let t = lines[i].trim();
                if !t.starts_with('|') || !t.ends_with('|') {
                    break;
                }
                // Skip separator rows like |---|---|
                let inner = &t[1..t.len()-1];
                let cells: Vec<&str> = inner.split('|').map(|c| c.trim()).collect();
                let is_separator = cells.iter().all(|c| c.chars().all(|ch| ch == '-' || ch == ':'));
                if !is_separator {
                    table_rows.push(cells);
                }
                i += 1;
            }
            if !table_rows.is_empty() {
                blocks.push(MdBlock::Table(table_rows));
            }
            continue;
        }

        // Heading
        if trimmed.starts_with("### ") {
            blocks.push(MdBlock::Heading(3, &trimmed[4..]));
            i += 1;
        } else if trimmed.starts_with("## ") {
            blocks.push(MdBlock::Heading(2, &trimmed[3..]));
            i += 1;
        } else if trimmed.starts_with("# ") {
            blocks.push(MdBlock::Heading(1, &trimmed[2..]));
            i += 1;
        } else if trimmed.starts_with("- ") {
            blocks.push(MdBlock::Bullet(&trimmed[2..]));
            i += 1;
        } else if trimmed.starts_with(|c: char| c.is_ascii_digit()) && trimmed.len() > 2 && trimmed.as_bytes()[1] == b'.' {
            blocks.push(MdBlock::Numbered(trimmed));
            i += 1;
        } else {
            blocks.push(MdBlock::Paragraph(trimmed));
            i += 1;
        }
    }

    blocks
}

fn render_markdown(ui: &mut egui::Ui, text: &str) {
    let blocks = parse_md(text);

    for block in &blocks {
        match block {
            MdBlock::Heading(level, text) => {
                let (size, extra_space) = match level {
                    1 => (18.0, 10.0),
                    2 => (15.0, 8.0),
                    _ => (13.5, 5.0),
                };
                ui.add_space(extra_space);
                ui.label(egui::RichText::new(*text).strong().size(size));
                ui.add_space(3.0);
            }
            MdBlock::Bullet(text) => {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.label(egui::RichText::new("\u{2022}").size(13.0).color(egui::Color32::from_rgb(0, 160, 100)));
                    ui.add_space(6.0);
                    render_inline(ui, text);
                });
            }
            MdBlock::Numbered(text) => {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    render_inline(ui, text);
                });
            }
            MdBlock::Paragraph(text) => {
                ui.add_space(2.0);
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(4.0);
                    render_inline(ui, text);
                });
            }
            MdBlock::CodeBlock(lines) => {
                ui.add_space(4.0);
                let code_text = lines.join("\n");
                let frame = egui::Frame::none()
                    .fill(egui::Color32::from_rgb(30, 30, 30))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)))
                    .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                    .rounding(4.0);
                frame.show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(code_text)
                            .monospace()
                            .size(12.0)
                            .color(egui::Color32::from_rgb(200, 200, 200)),
                    );
                });
                ui.add_space(4.0);
            }
            MdBlock::Table(rows) => {
                ui.add_space(4.0);
                render_table(ui, rows);
                ui.add_space(4.0);
            }
            MdBlock::Hr => {
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);
            }
        }
    }
}

fn render_table(ui: &mut egui::Ui, rows: &[Vec<&str>]) {
    if rows.is_empty() {
        return;
    }

    let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if max_cols == 0 {
        return;
    }

    let frame = egui::Frame::none()
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)))
        .inner_margin(egui::Margin::symmetric(4.0, 2.0))
        .rounding(2.0);
    frame.show(ui, |ui| {
        for (row_idx, row) in rows.iter().enumerate() {
            let is_header = row_idx == 0;
            ui.horizontal(|ui| {
                for col_idx in 0..max_cols {
                    let cell_text = row.get(col_idx).copied().unwrap_or("");
                    let _cell_width = match max_cols {
                        2 => 200.0,
                        3 => 150.0,
                        _ => 120.0,
                    };
                    let rt = egui::RichText::new(cell_text).size(12.0);
                    let rt = if is_header {
                        rt.strong().color(egui::Color32::from_rgb(100, 200, 255))
                    } else {
                        rt
                    };
                    ui.add(egui::Label::new(rt).sense(egui::Sense::hover()));
                    if col_idx < max_cols - 1 {
                        ui.separator();
                    }
                }
            });
            if is_header {
                ui.add(egui::Separator::default().horizontal());
            }
        }
    });
}

fn render_inline(ui: &mut egui::Ui, text: &str) {
    // Parse inline markdown into segments: Normal, Bold, Code
    enum Segment<'a> {
        Normal(&'a str),
        Bold(&'a str),
        Code(&'a str),
    }

    let mut segments = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        // Find the earliest inline marker
        let next_bold = remaining.find("**");
        let next_code = remaining.find('`');

        let next = match (next_bold, next_code) {
            (Some(b), Some(c)) => Some(b.min(c)),
            (Some(b), None) => Some(b),
            (None, Some(c)) => Some(c),
            (None, None) => None,
        };

        match next {
            None => {
                segments.push(Segment::Normal(remaining));
                break;
            }
            Some(pos) => {
                if pos > 0 {
                    segments.push(Segment::Normal(&remaining[..pos]));
                }
                remaining = &remaining[pos..];
                if remaining.starts_with("**") {
                    // Find closing **
                    if let Some(end) = remaining[2..].find("**") {
                        segments.push(Segment::Bold(&remaining[2..2+end]));
                        remaining = &remaining[2+end+2..];
                    } else {
                        segments.push(Segment::Normal("**"));
                        remaining = &remaining[2..];
                    }
                } else if remaining.starts_with('`') {
                    // Find closing `
                    if let Some(end) = remaining[1..].find('`') {
                        segments.push(Segment::Code(&remaining[1..1+end]));
                        remaining = &remaining[1+end+1..];
                    } else {
                        segments.push(Segment::Normal("`"));
                        remaining = &remaining[1..];
                    }
                }
            }
        }
    }

    // Render all segments in a single horizontal layout so they stay on one line
    ui.horizontal_wrapped(|ui| {
        for seg in &segments {
            match seg {
                Segment::Normal(t) => {
                    ui.label(egui::RichText::new(*t).size(13.0));
                }
                Segment::Bold(t) => {
                    ui.label(egui::RichText::new(*t).strong().size(13.0));
                }
                Segment::Code(t) => {
                    let frame = egui::Frame::none()
                        .fill(egui::Color32::from_rgb(40, 40, 40))
                        .inner_margin(egui::Margin::symmetric(4.0, 1.0))
                        .rounding(2.0);
                    frame.show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(*t)
                                .monospace()
                                .size(12.0)
                                .color(egui::Color32::from_rgb(220, 140, 80)),
                        );
                    });
                }
            }
        }
    });
}
