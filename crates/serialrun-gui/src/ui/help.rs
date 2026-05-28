use crate::state::{AppState, Language};
use eframe::egui;

const MCP_PROMPT_ZH: &str = r#"SerialRUN MCP 服务器使用指南

SerialRUN 是一个串口调试助手，内置 MCP 服务器，允许 AI 助手通过 TCP 连接控制串口设备。

## 连接信息
- MCP 服务器地址：127.0.0.1
- 端口：9527
- 协议：JSON-RPC over TCP

## 可用工具

### 1. list_ports
列出所有可用的串口设备。
```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_ports","arguments":{}}}
```

### 2. connect
连接到指定串口。
```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"connect","arguments":{"port":"COM3","baud_rate":115200}}}
```

### 3. disconnect
断开当前串口连接。
```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"disconnect","arguments":{}}}
```

### 4. send
发送数据到串口（支持文本或十六进制）。
```json
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"send","arguments":{"data":"AT\r\n","hex":false}}}
```

### 5. read
从串口读取数据。
```json
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"read","arguments":{"timeout_ms":1000,"max_bytes":1024}}}
```

### 6. send_command
发送命令并等待响应。
```json
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"send_command","arguments":{"command":"AT","timeout_ms":1000}}}
```

## 使用示例

1. 先列出端口：`list_ports`
2. 连接设备：`connect` 到 COM3，波特率 115200
3. 发送命令：`send_command` "AT"
4. 读取响应：`read`
5. 完成后断开：`disconnect`

## 注意事项
- 确保 SerialRUN 客户端已启动
- 确保目标串口未被其他程序占用
- 发送文本数据时会自动添加 \r\n 结束符
- 十六进制数据用空格分隔，如 "48 65 6C 6C 6F""#;

const MCP_PROMPT_EN: &str = r#"SerialRUN MCP Server Guide

SerialRUN is a serial port debugging assistant with a built-in MCP server, allowing AI assistants to control serial devices via TCP connection.

## Connection Info
- MCP Server Address: 127.0.0.1
- Port: 9527
- Protocol: JSON-RPC over TCP

## Available Tools

### 1. list_ports
List all available serial ports.
```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_ports","arguments":{}}}
```

### 2. connect
Connect to a serial port.
```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"connect","arguments":{"port":"COM3","baud_rate":115200}}}
```

### 3. disconnect
Disconnect from current serial port.
```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"disconnect","arguments":{}}}
```

### 4. send
Send data to serial port (text or hex).
```json
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"send","arguments":{"data":"AT\r\n","hex":false}}}
```

### 5. read
Read data from serial port.
```json
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"read","arguments":{"timeout_ms":1000,"max_bytes":1024}}}
```

### 6. send_command
Send command and wait for response.
```json
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"send_command","arguments":{"command":"AT","timeout_ms":1000}}}
```

## Usage Examples

1. List ports: `list_ports`
2. Connect device: `connect` to COM3 at 115200 baud
3. Send command: `send_command` "AT"
4. Read response: `read`
5. Disconnect when done: `disconnect`

## Notes
- Ensure SerialRUN client is running
- Ensure target serial port is not occupied by other programs
- Text data will automatically append \r\n terminator
- Hex data uses space separator, e.g., "48 65 6C 6C 6F""#;

pub fn render_help_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    egui::ScrollArea::vertical().max_height(500.0).show(ui, |ui| {
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
            "SerialRUN 内置 MCP 服务器，支持 AI 助手通过 TCP 控制串口设备。"
        } else {
            "SerialRUN includes a built-in MCP server for AI assistants to control serial devices via TCP."
        });
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("地址/Address:").strong());
            ui.label("127.0.0.1:9527");
        });
        ui.add_space(8.0);

        let copy_text = if lang == Language::Chinese { MCP_PROMPT_ZH } else { MCP_PROMPT_EN };
        let copy_label = if lang == Language::Chinese { "一键复制 MCP 说明（发给 AI 助手）" } else { "Copy MCP Guide (send to AI assistant)" };

        if ui.button(egui::RichText::new(copy_label).strong()).clicked() {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(copy_text.to_string());
            }
        }

        ui.add_space(4.0);
        ui.label(egui::RichText::new(if lang == Language::Chinese {
            "点击上方按钮复制完整说明，粘贴给任何 AI 助手即可让它理解如何使用 SerialRUN。"
        } else {
            "Click the button above to copy the full guide. Paste it to any AI assistant to let it understand how to use SerialRUN."
        }).weak().small());

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
