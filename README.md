<div align="center">

# SerialRUN

**Professional Serial Port Debugging Assistant for Embedded Developers**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-BSL%201.1-blue.svg)](https://mariadb.com/bsl11/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-blue.svg)]()
[![Website](https://img.shields.io/badge/Website-serialrun.com-brightgreen)](https://www.serialrun.com)

[English](#features) | [中文](README_CN.md)

</div>

---

<p align="center">
  <img src="assets/screenshot_en.png" alt="SerialRUN Screenshot" width="800">
</p>

<p align="center">
  <em>Modbus RTU debugging — real-time register monitoring with TX/RX terminal display</em>
</p>

### Website

<p align="center">
  <a href="https://www.serialrun.com" target="_blank">
    <img src="assets/screenshot_website_en.png" alt="SerialRUN Website" width="800">
  </a>
</p>

<p align="center">
  <em><a href="https://www.serialrun.com" target="_blank">www.serialrun.com</a> — Features, download, user guide, and documentation</em>
</p>

---

## Download

| Platform | Link |
|----------|------|
| Windows (x64) | [SerialRUN-v0.3.0-windows-x64.zip](https://github.com/YaoIsAI/SerialRUN/releases/download/v0.3.0/serialrun-0.3.0-windows-x64.zip) |
| macOS (Apple Silicon / Intel) | [SerialRUN-v0.3.0-macos-universal.zip](https://github.com/YaoIsAI/SerialRUN/releases/download/v0.3.0/serialrun-0.3.0-macos-universal.zip) |
| Linux (x86_64 / aarch64) | Build from source |

### Build from Source

```bash
git clone https://github.com/YaoIsAI/SerialRUN.git
cd SerialRUN
cargo build --release

# Windows:  target/release/serialrun.exe
# macOS:    target/release/serialrun
# Linux:    target/release/serialrun

# macOS .app bundle:
make app
```

See [docs/BUILD.md](docs/BUILD.md) for detailed platform-specific instructions.

## Features

- **Cross-platform** — Windows, macOS, Linux
- **CLI & GUI** — Command-line for automation, desktop app for interactive use
- **Multi-Window Interface** — All panels run as independent OS windows, drag and resize freely, always on top
- **Protocol Support** — Modbus RTU/TCP parsing, custom protocol patterns
- **Data Visualization** — Real-time charts and statistics
- **Script Recording** — Record and replay serial communication sessions with timing
- **File Transfer** — XMODEM / YMODEM / ZMODEM support
- **CAN Bus Analyzer** — SLCAN protocol parsing, frame filtering, per-ID statistics
- **I2C/SPI Debug** — Register read/write with address and data width config
- **Serial Oscilloscope** — Real-time waveform display with trigger and cursor measurement
- **Flasher** — STM32 ISP and ESP32 serial flashing
- **Register Editor** — CSV/JSON import/export, alarm threshold monitoring
- **Data Logger** — Continuous CSV recording with timestamp
- **Frame Builder** — Visual Modbus frame construction with live hex preview
- **PLC Control** — Modbus register polling with brand presets (Siemens, Mitsubishi, Delta, Omron)
- **TCP/RTU Bridge** — Bridge Modbus TCP clients to serial RTU devices
- **HMI Simulator** — Virtual Modbus slave simulator (TCP/RTU)
- **Plugin System** — Extensible architecture with dynamic plugin loading
- **MCP Server** — Built-in TCP server with 15 tools for AI assistant integration
- **Access Logging** — All MCP operations logged with client IP for traceability
- **HEX Mode** — Send and receive data in hexadecimal format
- **Auto Reply** — Automatically respond to matched patterns
- **Auto-Wrapping Toolbar** — Terminal controls adapt to window size, nothing gets clipped
- **Bilingual UI** — English / Chinese language switching, Dark / Light themes
- **Data Persistence** — Configuration, logs, terminal history, and warnings auto-saved to `~/.serialrun/`
- **Global Error System** — Unified error notifications in status bar with history

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
4. Type commands in the input box and click **Send**

## Project Structure

```
SerialRUN/
├── crates/
│   ├── serialrun-core/       # Core library (port, protocol, checksum, data logger)
│   ├── serialrun-cli/        # CLI application
│   ├── serialrun-gui/        # GUI application (egui/eframe)
│   ├── serialrun-mcp/        # MCP server for AI integration
│   └── serialrun-plugin-api/ # Plugin API definitions
├── plugins/
│   └── serialrun-example-plugin/  # Plugin example (C FFI)
├── assets/                   # Embedded images and icons
├── docs/                     # Documentation
├── tests/                    # Integration tests
└── benches/                  # Benchmarks
```

## GUI Panels

All panels run as independent OS windows — drag, resize, and arrange freely. Child windows always stay on top of the main window.

| Panel | Description |
|-------|-------------|
| Terminal | Serial TX/RX with HEX mode, timestamps, CRC checksums, auto-wrapping toolbar |
| Modbus | RTU monitor with 8 function codes, configurable response timeout (50-5000ms), TX/RX in terminal |
| PLC Control | Register polling with brand presets (Siemens, Mitsubishi, Delta, Omron), TX in terminal |
| TCP/RTU Bridge | Bridge Modbus TCP clients to serial RTU devices |
| HMI Simulator | Virtual Modbus slave with configurable registers and coils |
| CAN Bus | SLCAN frame capture, ID filtering, per-ID statistics |
| I2C/SPI | Register read/write debug tool, TX in terminal |
| Oscilloscope | Real-time waveform display with trigger and cursor measurement |
| File Transfer | XMODEM / YMODEM / ZMODEM protocol transfer |
| Frame Builder | Visual Modbus frame construction with live hex preview |
| Flasher | STM32 ISP and ESP32 serial flashing |
| Data Logger | Continuous CSV recording with timestamp |
| Register Editor | CSV/JSON import/export, alarm threshold monitoring |
| Chart | Multi-series real-time data visualization |
| Plugin Manager | Dynamic plugin discovery and loading |
| Log Viewer | Application log with filter, export, and persistence |

## Build for Different Platforms

| Platform | Command |
|----------|---------|
| Windows (MSVC) | `cargo build --target x86_64-pc-windows-msvc --release` |
| macOS (Apple Silicon) | `cargo build --target aarch64-apple-darwin --release` |
| macOS (Intel) | `cargo build --target x86_64-apple-darwin --release` |
| Linux | `cargo build --target x86_64-unknown-linux-gnu --release` |

See [docs/BUILD.md](docs/BUILD.md) for detailed instructions.

## Agent Mode (Automation)

```bash
serialrun agent list-ports                # List ports (JSON)
serialrun agent COM1 send "AT+RST"        # Send data
serialrun agent COM1 read --timeout 1000  # Read data
serialrun agent COM1 run-script test.txt  # Run script
```

## MCP Server

SerialRUN includes a built-in MCP server with 15 tools for AI assistant integration. All serial operations are routed through the GUI's port manager.

### Available Tools

| Tool | Description |
|------|-------------|
| `list_ports` | List all available serial ports |
| `connect` | Connect to serial port (baud rate, data bits, stop bits, parity, flow control) |
| `disconnect` | Disconnect from current connection |
| `send` | Send data (text or hex), no response wait |
| `read` | Read data from RX buffer (non-blocking, auto-captured by background monitor) |
| `send_command` | Send command and read response from buffer (recommended for AT commands) |
| `modbus_read` | Read Modbus RTU holding registers (with engineering value conversion) |
| `modbus_write` | Write Modbus RTU holding register |
| `plc_read` | Read all registers from a PLC preset brand |
| `plc_write` | Write to a PLC register by address |
| `status` | View connection status, byte counters, MCP server info |
| `get_config` | Read UI settings (supports all or single key) |
| `set_config` | Update UI setting (syncs to GUI immediately) |
| `get_access_log` | View access log with client IPs |

### Features

- All operations logged with client IP for traceability
- Supports multiple concurrent clients
- Localhost or LAN mode
- Access log visible in GUI settings panel
- Copy MCP connection info with one click

See [docs/MCP_API.md](docs/MCP_API.md) for the full API reference with JSON-RPC examples.

## Data Persistence

SerialRUN automatically saves data to `~/.serialrun/` directory:

| File | Content |
|------|---------|
| `config.toml` | Theme, language, baud rate settings |
| `logs.json` | Application logs (max 2000) |
| `terminal.json` | Terminal send/receive history (max 5000) |
| `warnings.json` | Warning/error history (max 1000) |
| `mcp_access_log.json` | MCP access log (max 1000) |

## Plugin Development

```rust
#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char { /* ... */ }

#[no_mangle]
pub extern "C" fn plugin_execute(command: *const c_char, params: *const c_char) -> *mut c_char { /* ... */ }
```

See [plugins/serialrun-example-plugin/](plugins/serialrun-example-plugin/) for a complete example.

## Documentation

| Document | Description |
|----------|-------------|
| [docs/help_en.md](docs/help_en.md) | English user guide |
| [docs/help_zh.md](docs/help_zh.md) | Chinese user guide |
| [docs/MANUAL.md](docs/MANUAL.md) | User manual |
| [docs/MCP_API.md](docs/MCP_API.md) | MCP API reference |
| [docs/BUILD.md](docs/BUILD.md) | Build guide |
| [CLAUDE.md](CLAUDE.md) | Agent operation guide |

## Development

```bash
cargo build       # Build all crates
cargo test        # Run tests
cargo bench       # Run benchmarks
```

## Support

If SerialRUN helps your work, consider buying me a coffee!

<p align="center">
  <img src="assets/wechat_pay_qr.jpg" alt="WeChat Pay" width="200">
</p>

<p align="center">
  <em>WeChat Pay</em>
</p>

## License

SerialRUN is dual-licensed under the [Business Source License 1.1](https://mariadb.com/bsl11/) and the [MIT License](LICENSE). See [LICENSE](LICENSE) for details.

---

<div align="center">

**Made with ❤️ for embedded developers**

</div>
