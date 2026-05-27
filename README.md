<div align="center">

# SerialTap

**A cross-platform serial port assistant for embedded developers**

**面向嵌入式开发者的跨平台串口助手**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux%20%7C%20iOS%20%7C%20Android-blue.svg)]()

[English](#features) | [中文](#功能特性)

</div>

---

## Features

- **Cross-platform** — Windows, macOS, Linux, iOS, Android
- **CLI & GUI** — Command-line for automation, desktop app for interactive use
- **Protocol Support** — Modbus RTU/TCP parsing, custom protocol patterns
- **Data Visualization** — Real-time charts and statistics
- **Script Recording** — Record and replay serial communication sessions
- **File Transfer** — XMODEM / YMODEM / ZMODEM support
- **Plugin System** — Extensible architecture with dynamic plugin loading
- **HEX Mode** — Send and receive data in hexadecimal format
- **Auto Reply** — Automatically respond to matched patterns
- **Bilingual UI** — English / 中文 language switching, Dark / Light themes

## 功能特性

- **跨平台** — Windows、macOS、Linux、iOS、Android
- **CLI & GUI** — 命令行用于自动化，桌面客户端用于交互式使用
- **协议支持** — Modbus RTU/TCP 解析，自定义协议模式匹配
- **数据可视化** — 实时图表和统计信息
- **脚本录制** — 录制和回放串口通信会话
- **文件传输** — 支持 XMODEM / YMODEM / ZMODEM
- **插件系统** — 可扩展架构，支持动态加载插件
- **十六进制模式** — 以十六进制格式收发数据
- **自动回复** — 自动响应匹配的模式
- **双语界面** — 英文/中文语言切换，深色/浅色主题

---

## Quick Start

### Install

```bash
git clone https://github.com/yourusername/SerialTap.git
cd SerialTap
cargo build --release
```

### CLI Usage

```bash
# List available ports / 列出可用串口
serialtap list

# Connect to a port / 连接串口
serialtap connect COM1 -b 115200

# Send data / 发送数据
serialtap send COM1 "Hello\r\n"

# Monitor with timestamps / 带时间戳监听
serialtap monitor COM1 -t -l output.log

# Record a script / 录制脚本
serialtap record COM1 -o script.txt

# Replay a script / 回放脚本
serialtap replay COM1 script.txt
```

### GUI Usage

```bash
# Launch desktop app / 启动桌面客户端
serialtap-gui
```

### Quick Start Guide (GUI)

1. Connect your serial device via USB
2. Click **Refresh** to detect the port
3. Select port and baud rate, click **Connect**
4. Type commands in the input box and press Enter

### 快速开始 (GUI)

1. 通过 USB 连接串口设备
2. 点击「刷新」检测端口
3. 选择端口和波特率，点击「连接」
4. 在输入框输入命令，按回车发送

---

## Project Structure

```
SerialTap/
├── crates/
│   ├── serialtap-core/       # Core library / 核心库
│   ├── serialtap-cli/        # CLI application / 命令行工具
│   └── serialtap-gui/        # GUI application / 桌面客户端
├── plugins/
│   └── example-plugin/       # Example plugin / 插件示例
├── docs/                     # Documentation / 文档
├── tests/                    # Integration tests / 集成测试
└── benches/                  # Benchmarks / 性能测试
```

## 项目结构

```
SerialTap/
├── crates/
│   ├── serialtap-core/       # 核心串口逻辑库
│   ├── serialtap-cli/        # 命令行工具
│   └── serialtap-gui/        # 桌面客户端 (egui)
├── plugins/
│   └── example-plugin/       # 插件示例 (C FFI)
├── docs/                     # 文档
├── tests/                    # 集成测试
└── benches/                  # 性能测试
```

---

## Build for Different Platforms

| Platform | Command |
|----------|---------|
| Windows (MSVC) | `cargo build --target x86_64-pc-windows-msvc --release` |
| macOS (Apple Silicon) | `cargo build --target aarch64-apple-darwin --release` |
| macOS (Intel) | `cargo build --target x86_64-apple-darwin --release` |
| Linux | `cargo build --target x86_64-unknown-linux-gnu --release` |

See [docs/BUILD.md](docs/BUILD.md) for detailed build instructions including Android, iOS, and .app bundling.

## 跨平台构建

| 平台 | 命令 |
|------|------|
| Windows (MSVC) | `cargo build --target x86_64-pc-windows-msvc --release` |
| macOS (Apple Silicon) | `cargo build --target aarch64-apple-darwin --release` |
| macOS (Intel) | `cargo build --target x86_64-apple-darwin --release` |
| Linux | `cargo build --target x86_64-unknown-linux-gnu --release` |

详见 [docs/BUILD.md](docs/BUILD.md) 了解 Android、iOS 构建及 .app 打包。

---

## Agent Mode (Automation)

SerialTap CLI supports JSON output for programmatic access:

```bash
# List ports (JSON) / 列出端口 (JSON)
serialtap agent list-ports

# Send data / 发送数据
serialtap agent COM1 send "AT+RST" -b 115200

# Read data / 读取数据
serialtap agent COM1 read --timeout 1000

# Run script / 运行脚本
serialtap agent COM1 run-script script.txt
```

## Agent 模式 (自动化)

SerialTap CLI 支持 JSON 输出，便于程序化调用：

```bash
# 列出端口
serialtap agent list-ports

# 发送数据
serialtap agent COM1 send "AT+RST" -b 115200

# 读取数据
serialtap agent COM1 read --timeout 1000

# 运行脚本
serialtap agent COM1 run-script script.txt
```

---

## Plugin Development

Create plugins with C FFI interface:

```rust
#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char { /* ... */ }

#[no_mangle]
pub extern "C" fn plugin_execute(command: *const c_char, params: *const c_char) -> *mut c_char { /* ... */ }
```

See [plugins/example-plugin/](plugins/example-plugin/) for a complete example.

## 插件开发

使用 C FFI 接口创建插件：

```rust
#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char { /* ... */ }

#[no_mangle]
pub extern "C" fn plugin_execute(command: *const c_char, params: *const c_char) -> *mut c_char { /* ... */ }
```

完整示例见 [plugins/example-plugin/](plugins/example-plugin/)。

---

## Documentation

| Document | Description |
|----------|-------------|
| [docs/MANUAL.md](docs/MANUAL.md) | User manual / 用户手册 |
| [docs/SKILL.md](docs/SKILL.md) | Skill reference / 技能参考 |
| [docs/BUILD.md](docs/BUILD.md) | Build guide / 构建指南 |
| [CLAUDE.md](CLAUDE.md) | Agent operation guide / Agent 操作指南 |

## 文档

| 文档 | 说明 |
|------|------|
| [docs/MANUAL.md](docs/MANUAL.md) | 用户手册 |
| [docs/SKILL.md](docs/SKILL.md) | 技能参考 |
| [docs/BUILD.md](docs/BUILD.md) | 构建指南 |
| [CLAUDE.md](CLAUDE.md) | Agent 操作指南 |

---

## Development

```bash
# Build all crates / 构建所有 crate
cargo build

# Run tests / 运行测试
cargo test

# Run benchmarks / 运行性能测试
cargo bench
```

## 开发

```bash
# 构建所有 crate
cargo build

# 运行测试
cargo test

# 运行性能测试
cargo bench
```

---

## License

[MIT License](LICENSE)

---

<div align="center">

**Made with ❤️ for embedded developers**

**为嵌入式开发者用心打造**

</div>
