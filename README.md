<div align="center">

# SerialRUN

**A cross-platform serial port assistant for embedded developers**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux%200%7C%20iOS%20%7C%20Android-blue.svg)]()

[English](#features) | [中文](README_CN.md)

</div>

---

## Features

- **Cross-platform** — Windows, macOS, Linux, iOS, Android
- **CLI & GUI** — Command-line for automation, desktop app for interactive use
- **Protocol Support** — Modbus RTU/TCP parsing, custom protocol patterns
- **Data Visualization** — Real-time charts and statistics
- **Script Recording** — Record and replay serial communication sessions
- **File Transfer** — XMODEM / YMODEM / ZMODEM support
- **CAN Bus Analyzer** — SLCAN protocol parsing, frame filtering, per-ID statistics
- **I2C/SPI Debug** — Register read/write with address and data width config
- **Serial Oscilloscope** — Real-time waveform display with trigger and cursor measurement
- **Flasher** — STM32 ISP and ESP32 serial flashing
- **Register Editor** — CSV/JSON import/export, alarm threshold monitoring
- **Data Logger** — Continuous CSV recording with timestamp
- **Frame Builder** — Visual Modbus frame construction with live hex preview
- **PLC Control** — Modbus register polling with brand presets (Siemens, Mitsubishi, etc.)
- **Plugin System** — Extensible architecture with dynamic plugin loading
- **MCP Server** — Built-in TCP server for AI assistant integration
- **HEX Mode** — Send and receive data in hexadecimal format
- **Auto Reply** — Automatically respond to matched patterns
- **Bilingual UI** — English / Chinese language switching, Dark / Light themes

## Quick Start

### Install

```bash
git clone https://github.com/YaoIsAI/SerialRUN.git
cd SerialRUN
cargo build --release
```

### CLI Usage

```bash
# List available ports
serialrun list

# Connect to a port
serialrun connect COM1 -b 115200

# Send data
serialrun send COM1 "Hello\r\n"

# Monitor with timestamps
serialrun monitor COM1 -t -l output.log

# Record a script
serialrun record COM1 -o script.txt

# Replay a script
serialrun replay COM1 script.txt
```

### GUI Usage

```bash
serialrun-gui
```

### GUI Quick Start

1. Connect your serial device via USB
2. Click **Refresh** to detect the port
3. Select port and baud rate, click **Connect**
4. Type commands in the input box and press Enter

## Project Structure

```
SerialRUN/
├── crates/
│   ├── serialrun-core/       # Core library (port, protocol, checksum, data logger)
│   ├── serialrun-cli/        # CLI application
│   ├── serialrun-gui/        # GUI application (egui)
│   ├── serialrun-mcp/        # MCP server for AI integration
│   └── serialrun-plugin-api/ # Plugin API definitions
├── plugins/
│   └── example-plugin/       # Plugin example (C FFI)
├── assets/                   # Embedded images (QR code)
├── docs/                     # Documentation
├── tests/                    # Integration tests
└── benches/                  # Benchmarks
```

## GUI Panels

| Panel | Description |
|-------|-------------|
| Terminal | Serial TX/RX with HEX mode, timestamps, CRC |
| Modbus | RTU monitor with function code parsing |
| PLC Control | Register polling with brand presets |
| CAN Bus | SLCAN frame capture and analysis |
| I2C/SPI | Register read/write debug tool |
| Oscilloscope | Real-time waveform display |
| File Transfer | XMODEM/YMODEM/ZMODEM |
| Frame Builder | Visual Modbus frame construction |
| Flasher | STM32 ISP / ESP32 serial flashing |
| Data Logger | CSV recording with timestamp |
| Register Editor | Import/export register maps |
| Chart | Multi-series real-time data visualization |
| Plugin Manager | Dynamic plugin discovery and loading |
| Log Viewer | Application log with filter and export |

## Build for Different Platforms

| Platform | Command |
|----------|---------|
| Windows (MSVC) | `cargo build --target x86_64-pc-windows-msvc --release` |
| macOS (Apple Silicon) | `cargo build --target aarch64-apple-darwin --release` |
| macOS (Intel) | `cargo build --target x86_64-apple-darwin --release` |
| Linux | `cargo build --target x86_64-unknown-linux-gnu --release` |

See [docs/BUILD.md](docs/BUILD.md) for detailed instructions including Android, iOS, and .app bundling.

## Agent Mode (Automation)

```bash
serialrun agent list-ports                # List ports (JSON)
serialrun agent COM1 send "AT+RST"        # Send data
serialrun agent COM1 read --timeout 1000  # Read data
serialrun agent COM1 run-script test.txt  # Run script
```

## MCP Server

SerialRUN includes a built-in MCP server for AI assistant integration.

```bash
# Start MCP server (default: 127.0.0.1:9527)
serialrun-mcp
```

Available tools: `list_ports`, `connect`, `disconnect`, `send`, `read`, `send_command`.

## Plugin Development

```rust
#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char { /* ... */ }

#[no_mangle]
pub extern "C" fn plugin_execute(command: *const c_char, params: *const c_char) -> *mut c_char { /* ... */ }
```

See [plugins/example-plugin/](plugins/example-plugin/) for a complete example.

## Documentation

| Document | Description |
|----------|-------------|
| [docs/MANUAL.md](docs/MANUAL.md) | User manual |
| [docs/SKILL.md](docs/SKILL.md) | Skill reference |
| [docs/BUILD.md](docs/BUILD.md) | Build guide |
| [CLAUDE.md](CLAUDE.md) | Agent operation guide |

## Development

```bash
cargo build       # Build all crates
cargo test        # Run tests
cargo bench       # Run benchmarks
```

## License

[MIT License](LICENSE)

---

<div align="center">

**Made with ❤️ for embedded developers**

</div>
