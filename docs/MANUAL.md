# SerialRUN User Manual

[中文版](MANUAL_CN.md)

---

## Table of Contents

- [Introduction](#introduction)
- [Installation](#installation)
- [CLI Reference](#cli-reference)
- [GUI Guide](#gui-guide)
- [Troubleshooting](#troubleshooting)

## Introduction

SerialRUN is a cross-platform serial port assistant designed for embedded developers. It provides both CLI and GUI interfaces for serial communication, protocol analysis, and automation.

## Installation

```bash
git clone https://github.com/yourusername/SerialRUN.git
cd SerialRUN
cargo build --release
```

Binaries will be in `target/release/`.

## CLI Reference

### `serialrun list`

List all available serial ports.

```bash
serialrun list                # Text output
serialrun list --format json  # JSON output
```

### `serialrun connect <port> [options]`

Connect to a serial port in interactive mode.

| Option | Default | Description |
|--------|---------|-------------|
| `-b, --baud` | 115200 | Baud rate |
| `-d, --data-bits` | 8 | Data bits (5/6/7/8) |
| `-s, --stop-bits` | 1 | Stop bits (1/2) |
| `-p, --parity` | none | Parity (none/odd/even) |
| `-f, --flow` | none | Flow control (none/software/hardware) |

### `serialrun send <port> <data> [options]`

```bash
serialrun send COM1 "Hello\r\n"              # Send text
serialrun send COM1 "48 65 6C 6C 6F" --hex   # Send HEX
```

### `serialrun monitor <port> [options]`

```bash
serialrun monitor COM1 -t                  # Timestamps
serialrun monitor COM1 -x                  # HEX mode
serialrun monitor COM1 -t -l output.log    # Log to file
```

### `serialrun record <port> [options]`

```bash
serialrun record COM1 -o script.txt
```

### `serialrun replay <port> <script>`

```bash
serialrun replay COM1 script.txt
```

### `serialrun agent [port] <action>`

JSON output mode for automation. See [CLAUDE.md](../CLAUDE.md) for details.

## GUI Guide

```bash
serialrun-gui
```

### Interface Overview

- **Top bar**: Tool buttons (Log, Chart, PLC, Modbus, File Transfer, Frame Builder, Data Logger, CAN, I2C/SPI, Oscilloscope, Flasher, Register Editor, Plugins), Theme/Language/Help
- **Left panel**: Port selector, baud rate, Auto-detect, Connect/Disconnect
- **Center**: Terminal display with TX/RX/SYS indicators
- **Bottom**: Status bar with connection state and byte counts

### GUI Panels

| Panel | Description |
|-------|-------------|
| Terminal | Serial TX/RX with HEX mode, timestamps, CRC checksums |
| Modbus | RTU monitor with function code parsing and register display |
| PLC Control | Modbus register polling with brand presets (Siemens, Mitsubishi, Delta, etc.) |
| CAN Bus | SLCAN frame capture, ID filtering, per-ID statistics |
| I2C/SPI | Register read/write debug tool with address and data width config |
| Oscilloscope | Real-time waveform display with trigger and cursor measurement |
| File Transfer | XMODEM / YMODEM / ZMODEM protocol transfer |
| Frame Builder | Visual Modbus frame construction with live hex preview |
| Flasher | STM32 ISP and ESP32 serial flashing |
| Data Logger | Continuous CSV recording with timestamp |
| Register Editor | CSV/JSON import/export, alarm threshold monitoring |
| Chart | Multi-series real-time data visualization |
| Plugin Manager | Dynamic plugin discovery and loading |
| Log Viewer | Application log with filter and export |

### Language and Theme

- Click **中** / **EN** to switch language
- Click **Dark** / **Light** to switch theme
- Click **?** to open the help guide

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Port not found | Click Refresh, check device connection |
| Permission denied (Linux) | `sudo usermod -a -G dialout $USER` |
| Garbled text | Check baud rate matches device setting |
| No data | Verify cable, check flow control settings |
