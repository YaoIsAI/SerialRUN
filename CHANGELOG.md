# Changelog

## v0.4.7 - 2026-10-28

### In-App Updates & Automatic Upgrade

#### Updater
- **Check for updates from GitHub**: Settings → Software Update queries the latest release and shows the new version, release notes, asset name and size
- **Automatic startup check** with a 24-hour interval; a status-bar badge appears when a new version is available. Can be disabled in settings
- **One-click upgrade**: downloads the correct package for the current OS/architecture, shows progress and download speed, verifies SHA-256 when available, then installs and restarts
- **Native installation per platform**:
  - Windows: runs the Inno Setup installer silently, then relaunches SerialRUN
  - macOS: replaces the `.app` bundle atomically with a backup, clears the quarantine flag and reopens
  - Linux: replaces the binary in place when writable, otherwise opens the `.deb` with the system package manager
- **Dismissed versions** are remembered per release, while manual checks remain available at any time

#### Release Engineering
- Release workflow publishes `SHA256SUMS` next to the installers for download verification
- The status-bar version is now taken from the Cargo package version at compile time, removing the manual sync step

#### Documentation
- README (EN/CN) documents the updater in the feature list

---

## v0.4.6 - 2026-10-28

### Installers, Official Website Refresh & Smoother Downloads

#### Release Engineering
- **Windows installer (.exe)**: Inno Setup build with Start Menu shortcuts, optional desktop icon and a proper uninstaller. The portable ZIP remains available
- **Debian package (.deb)**: installs `serialrun`, `serialrun-cli` and `serialrun-mcp` under `/usr/bin`, with a desktop entry, hicolor icon and automatic dependency resolution via `apt install ./serialrun_*.deb`
- **Release pipeline**: Android-style one-click releases now publish 8 assets (Windows setup + ZIP, macOS arm64/x64 DMG + ZIP, Linux DEB + tar.gz)

#### Official Website
- **Complete redesign** in a modern open-source project style: dark hero, clean feature grid, framed screenshot, quick start and a dedicated downloads page
- **Automatic OS detection**: the primary download button adapts to the visitor — Windows gets the `.exe` installer, Apple Silicon gets the arm64 DMG, Intel Macs get the x64 DMG, Linux gets the `.deb`
- **Direct installer downloads**: no more detours through ZIP archives; platform cards highlight the recommended package
- **All 9 languages preserved** and updated with the new copy

---

## v0.4.5 - 2026-10-28

### Multi-Platform Releases & User Feedback Fixes

#### Release Engineering
- **GitHub Actions release pipeline**: Windows x64, macOS Apple Silicon, macOS Intel, Linux x86_64
- **Automated version bump**: one-click releases sync Cargo manifests, Cargo.lock, status bar, Info.plist and README download links
- **Consistent MCP version**: MCP servers now report the actual build version instead of a hardcoded value
- **Private assets stay private**: the donation QR code is only bundled in local builds; CI and release builds no longer depend on ignored files
- **Verified packages**: each platform ships a ready-to-run archive (ZIP / DMG / tar.gz) attached to GitHub Releases

#### Bug Fixes (from GitHub issues)
- **Custom baud rate (#12)**: typed values were only applied when pressing Enter; clicking Connect silently kept the old rate. The typed rate is now applied on focus loss and on Connect
- **High baud rate presets**: added 1000000, 1500000, 2000000 and 3000000
- **macOS (#11)**: the previous "macos-universal" package was actually ARM64-only, so Intel Macs could not launch it. Releases now ship separate Intel (x64) and Apple Silicon (arm64) builds, with correct bundle versions, icon, and a minimum macOS 11 target

#### Documentation
- Updated download table, MCP tool list (23 tools), baud rate reference
- Added release SOP (`docs/RELEASE_SOP.md`)
- Updated build documentation with the CI release flow

---

## v0.4.3 - 2026-06-18

- Wireshark-style packet capture viewer with live capture from serial ports
- pcap MCP tools (load_pcap, query_packets, get_packet, pcap_stats)
- File transfer integration, dynamic UI, keyboard shortcuts
- Complete plugin system with community plugins
- Sample Modbus RTU/TCP capture files

---

## v0.2.0 - 2026-06-02

### Major Release: Plugin System, CAN Analyzer, MCP Server

#### Plugin System (Complete)
- **Plugin API v0.2**: Capabilities, lifecycle, host callbacks (SerialPort, FileDialog, Progress, Logging)
- **Plugin Manager**: Install, uninstall, enable, disable, ZIP import
- **Plugin Community**: Online search on GitHub, one-click install from Releases (local import ready, community search coming soon)
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
