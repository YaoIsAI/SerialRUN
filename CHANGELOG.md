# Changelog

## v0.2.0 - 2026-06-02

### Major Release: Plugin System, CAN Analyzer, MCP Server

#### Plugin System (Complete)
- **Plugin API v0.2**: Capabilities, lifecycle, host callbacks (SerialPort, FileDialog, Progress, Logging)
- **Plugin Manager**: Install, uninstall, enable, disable, ZIP import
- **Plugin Community**: Online search on GitHub, one-click install from Releases
- **Plugin Manifest**: `plugin.json` format with metadata, platform checking, version compatibility
- **STC ISP Plugin**: Flash STC series MCUs (STC89/12/15/8/8G/8H) via ISP protocol
- **12 Bug Fixes**: Memory leaks, race conditions, dead code, cross-filesystem issues

#### CAN Bus Analyzer
- **Independent Connection**: Uses its own serial port (separate from terminal)
- **Frame Table**: USB-CAN Tool style with index, time, channel, direction, ID, type, DLC, data
- **Periodic Send**: Configurable count + period with ID/data auto-increment
- **Port Conflict Warning**: Alerts when CAN port is shared with terminal

#### MCP Server (15 Tools)
- **Serial Operations**: connect, disconnect, send, read, send_command
- **Modbus RTU**: modbus_read (FC03 with engineering conversion), modbus_write (FC06)
- **PLC Control**: plc_read (Siemens/Mitsubishi/Delta/Omron), plc_write
- **Configuration**: get_config, set_config (real-time GUI sync)
- **Monitoring**: status, get_device_info, get_access_log
- **Dedicated Read Buffer**: MCP and GUI terminal read independently (no data race)
- **Terminal Display**: All MCP TX/RX visible in terminal with [MCP] tag
- **Config Sync**: MCP set_config changes reflect in GUI immediately
- **Comprehensive Test Suite**: 43 automated tests against real MCU

#### Terminal Improvements
- **Line Numbers**: Badge-style line numbers when timestamps are disabled
- **Left Padding**: Fixed text clipping at scroll area edge
- **MCP RX Display**: MCP responses now visible in terminal

#### Documentation
- **Plugin Development Guide**: Complete rewrite with community publishing
- **Plugin Spec**: Updated to match implementation
- **MCP API Reference**: Full tool documentation
- **Help Files**: Updated for all new features

---

## v0.1.0 - 2026-05-31

### Initial Release

- Serial communication (HEX/TEXT, timestamps, CRC, auto-send, DTR/RTS)
- Modbus RTU/TCP debugging (8 function codes, register monitor)
- PLC control (Siemens, Mitsubishi, Delta, Omron presets)
- TCP/RTU bridge and HMI simulator
- CAN bus analysis (SLCAN)
- I2C/SPI debug
- Firmware flash (STM32 ISP, ESP32 serial)
- MCP server with 15 tools for AI integration
- Multi-window interface (independent OS windows)
- Data persistence (config, logs, terminal history)
- Bilingual UI (English/Chinese)
- Cross-platform (Windows, macOS, Linux)
