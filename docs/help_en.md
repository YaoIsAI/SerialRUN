# SerialRUN User Guide

SerialRUN is a feature-rich serial port debugging assistant supporting serial communication, Modbus debugging, PLC control, CAN bus analysis, I2C/SPI debugging, firmware flashing, and more. Built-in MCP server enables AI assistants to remotely control serial devices via TCP.

---

## Quick Start

1. Connect your serial device to the computer via USB
2. Click the refresh button `↻` in the left panel to detect ports
3. Select a serial port from the dropdown (e.g., COM3)
4. Set the baud rate (default 115200, supports custom values)
5. Click the green "Connect" button
6. Type commands in the bottom terminal input box and click "Send"

If you don't know the baud rate, click the **Auto** button to auto-detect (progress shown in status bar).

---

## Interface Layout

### Top Toolbar

The toolbar contains 15 function buttons. Click to open/close the corresponding window:

| Button | Function | Description |
|--------|----------|-------------|
| Log | Log Viewer | Display send/receive records, export to CSV, auto-persisted |
| Chart | Data Chart | Real-time data rate curve |
| PLC | PLC Controller | Supports Siemens, Mitsubishi, Delta, Omron |
| Mod | Modbus Debug | Quick register read/write, 8 function codes |
| TCP | TCP/RTU Bridge | Bridge Modbus TCP clients to serial RTU devices |
| HMI | HMI Simulator | Simulate virtual Modbus slave (TCP/RTU) |
| FT | File Transfer | XMODEM/YMODEM/ZMODEM protocols |
| FB | Frame Builder | Manual Modbus frame construction |
| DL | Data Logger | Log serial data to CSV file |
| CAN | CAN Bus | Capture and parse CAN frames (SLCAN) |
| I2C | I2C/SPI | Scan and read/write I2C/SPI devices |
| Scope | Oscilloscope | Visualize serial data waveforms |
| Flash | Flasher | STM32/ESP32 firmware flashing |
| Reg | Reg Editor | Custom device register map |
| Plug | Plugins | Load and manage extension plugins |

Three system buttons on the right side of the toolbar:
- **?** — Open this help guide
- **Dark/Light** — Switch dark/light theme (auto-saved)
- **EN/中** — Switch English/Chinese interface (auto-saved)

> **Note**: All feature panels (Log, Chart, Modbus, PLC, etc.) are now independent OS windows that can be dragged anywhere on screen and resized independently. Child windows always stay on top of the main window.

### Left Panel

- **Serial Port** — Select serial device from dropdown, click `↻` to refresh port list
- **Baud Rate** — Type custom baud rate directly, click to show common rates (300-921600)
- **Auto Button** — Auto-detect baud rate, progress shown in status bar (ASCII spinner animation)
- **Port Config** — Data bits (5-8), stop bits (1/2), parity (None/Odd/Even), flow control (None/Software/Hardware)
- **Auto Reply** — Set match pattern and reply content, auto-send reply when matching data is received
- **MCP Server** — Enable/configure MCP server, one-click copy connection info, view access logs
- **Record/Replay** — Record serial operation scripts, save and replay later

### Bottom Status Bar

- **Connection Status** — Shows current connected port and baud rate
- **Data Statistics** — RX/TX byte counters and real-time rate
- **Recording Indicator** — Red `● Recording` marker
- **AI Connection** — Yellow `AI: COM4 @ 9600 baud` indicator (MCP independent connection)
- **Baud Detection** — ASCII spinner `[-\|/] 115200...` shows current detection progress
- **Error Display** — Red `✗` + error message (auto-dismiss after 5 seconds)
- **Warning History** — Red dot + count, click to view warning/error history

### Terminal Area

Terminal displays all send/receive data with color-coded directions:
- Green `↓ RX` — Received data
- Blue `↑ TX` — Sent data
- Yellow `⚙ SYS` — System messages

Terminal top toolbar (adaptive width, auto-wraps on narrow windows):
- **Timestamp** — Display timestamp before each message (format: YYYY-MM-DD HH:MM:SS.mmm)
- **Auto Scroll** — Auto-scroll to latest message
- **CRC** — Auto-append checksum on send (hover for algorithm descriptions)
- **DTR/RTS** — Shown when connected, control Data Terminal Ready / Request To Send signals
- **Auto Send** — Repeatedly send input content at set interval
- **T/O** — RX timeout setting (hover for description), auto-calculated by baud rate or manual
- **Save/Clear** — Export terminal log / Clear terminal display

> The toolbar uses `horizontal_wrapped` layout — controls automatically wrap to the next line when the window is too narrow.

Right-click context menu:
- **Copy** — Copy current line content
- **Copy All** — Copy all terminal data (with timestamps and direction markers)
- **Convert to HEX** — Convert text line to hexadecimal display (TEXT lines only)
- **Resend** — Re-send the line data (TX lines only, requires connection)

Terminal auto-detects data format: text commands display as text, binary data (e.g., Modbus RTU frames) display as HEX. Supports Chinese, Japanese, English, Russian, and other languages.

---

## Feature Details

### Log Viewer

Records all system events including connect/disconnect, data send/receive, errors, etc.
- Three log levels: INFO/WARN/ERR with color coding
- Displays complete send/receive data (HEX + text)
- Auto-persisted to `~/.serialrun/logs.json`, survives restart
- **Clear** — Clear all logs
- **Export** — Export to CSV file (timestamp, level, message)

### Data Chart

Real-time display of data send/receive rate (bytes/sec) with auto-scaling Y-axis and grid lines.
- Bottom shows cumulative RX/TX byte counters
- Max-value label shows current peak rate

### Modbus Debug Tool

Three collapsible sections:

**Quick Request**
- Slave ID: 0-247
- Function codes: 01 Read Coils / 02 Read Discrete Inputs / 03 Read Holding Registers / 04 Read Input Registers / 05 Write Single Coil / 06 Write Single Register / 15 Write Multiple Coils / 16 Write Multiple Registers
- Start address and quantity/write value
- **Response Timeout** — Configurable Modbus response wait time (50-5000ms, default 500ms), increase for slow devices
- TX/RX data is also displayed in the terminal panel
- Display last request and response hex data

**Register Monitor**
- Configurable poll interval (100-10000ms)
- Real-time display of register address, raw value, formatted value, and last update time

**Frame Log**
- Records all Modbus frame history (up to 200 entries)
- Shows timestamp, request/response hex, and decoded content

### PLC Controller

Professional minimalist PLC monitoring panel:
- **Brands**: Siemens S7-1200, Mitsubishi FX3U, Delta DVP, Omron CP1H, Custom
- **Data Types**: BOOL, UINT16, INT16, UINT32, FLOAT32
- **Batch Read**: Contiguous registers are coalesced into single Modbus requests for faster polling
- **Inline Write**: Click a register row to edit and write values directly
- **Scale Factor**: Values are automatically scaled on read; writes apply inverse scaling
- **Per-register Status**: Green/yellow/red dot indicates data freshness (<3s / <10s / stale)
- **Auto-poll**: Configurable interval (100-10000ms)

### TCP/RTU Bridge

Bridges Modbus TCP clients to serial RTU devices. External SCADA/HMI software can communicate with serial devices via TCP.

- **TCP Port** — Listen port for TCP connections (default 502)
- **Serial Port** — Target serial port connected to RTU device
- **Baud Rate** — Serial communication speed
- **Timeout** — Response timeout in milliseconds
- **Start/Stop** — Toggle bridge operation
- **Bridge Log** — Shows all bridged requests/responses with timestamps

### HMI Simulator

Simulates a virtual Modbus slave device. Useful for testing PLC programs, SCADA systems, or Modbus master software without physical hardware.

- **Mode** — TCP Server (listen for TCP connections) or RTU Slave (respond on serial port)
- **TCP Port** — Listen port (TCP mode)
- **Serial Port / Baud Rate** — Serial settings (RTU mode)
- **Slave ID** — Modbus slave address (0-247)
- **Holding Registers** — Edit register values (0-65535), add new registers
- **Coils** — Toggle coil on/off states, add new coils
- **Simulator Log** — Shows all received requests and responses

### File Transfer

Transfer files via serial port with these protocols:
- **XMODEM** — Basic checksum
- **XMODEM-CRC** — CRC checksum, more reliable
- **YMODEM** — Supports file name and size info
- **ZMODEM** — Advanced protocol with resume support

### Frame Builder

Manually construct Modbus frames:
1. Set slave ID, function code, address, value
2. Click "Build" to generate hex frame
3. Click "Send" to transmit via serial port

### Data Logger

Log serial send/receive data to CSV files:
- Records timestamp, direction (TX/RX), data (text and hex), byte count
- Shows current buffer record count
- Start/stop recording at any time

### CAN Bus Analyzer

CAN bus analysis tool with two connection modes:

**SLCAN (Serial)** — For SLCAN-compatible adapters connected via serial port:
- Select port and baud rate (100K/125K/250K/500K/1M)
- Uses independent serial connection, does not affect terminal

**CANalyst-II (USB)** — For CANalyst-II / USBCAN-2A adapters (Windows/Linux):
- Scan and auto-detect connected USB CAN devices
- Dual channel support (CAN1 + CAN2)
- Work mode: Normal / Listen-only / Loopback self-test
- Full baud rate range: 10K to 1Mbps (17 standard rates)
- Device info display (hardware version, serial number)
- Requires `ControlCAN.dll` in the application directory

**Common features (both modes):**
- **Capture** — Start/stop CAN frame capture
- **Statistics** — Total frames, error frames, unique IDs, bus load estimate
- **Frame List** — Timestamp, direction, ID (standard/extended), DLC, data
- **Transmit** — Enter ID and data to send CAN frames (single or periodic burst)
- **Filter** — Filter display by ID
- CSV export supported, up to 100,000 frames buffer

### I2C/SPI Debug

Two switchable modes:

**I2C Mode**
- Set device address (default 0x68) and register address
- **Scan** — Scan for I2C devices in range 0x08-0x77
- **Read** — Read data from specified register
- **Write** — Write data to specified register

**SPI Mode**
- Enter MOSI data (hex)
- **Transfer** — Send data and receive response

### Oscilloscope

Visualize serial data as waveform plots:
- Configurable timebase (1-5000ms)
- 10x16 grid, auto-scaling Y-axis
- Hover tooltip shows exact time and value
- Buffer up to 10000 data points

### Firmware Flasher

Supports firmware flashing for two MCU types:

**STM32**
- Sends 0x7F init sequence on connect
- Supports HEX and BIN firmware formats
- Chunked write (128 bytes/block)

**ESP32**
- Sends sync sequence on connect
- Supports BIN and ELF firmware formats
- Chunked write (128 bytes/block)

### Register Editor

Custom device register map table:
- Add/remove register entries
- Set address, name, data type, value, description
- Import from CSV/JSON, export to CSV
- Optional alarm feature with threshold setting

### Plugin Manager

Manages extension plugins with two tabs:

**Installed Tab**
- Displays plugin name, version, author, and load status
- Enable/disable plugins with checkbox
- Expand/collapse to show plugin commands panel
- Help icon (?) shows usage documentation on hover
- Import ZIP button for local plugin installation

**Community Tab**
- Search plugins on GitHub (tagged with `serialrun-plugin`)
- One-click install from GitHub Releases
- Shows plugin description, stars, and capabilities
- Auto-discovers popular plugins on first visit

---

## Tips

- When debugging a new device, start with **115200** baud rate
- If you don't know the baud rate, click the **Auto** button (progress shown in status bar)
- You can type custom baud rates directly in the baud rate input field (e.g., 74880, 250000)
- In HEX mode, sending data automatically strips spaces and `0x` prefixes
- You can append **CRC checksums** when sending text for data integrity
- **Auto Reply** is useful for protocol debugging that requires responses
- Use **Record/Replay** to quickly repeat test sequences
- Right-click terminal data lines for quick copy, copy all, convert to HEX, or resend
- Terminal auto-detects text/HEX format: AT commands display as text, Modbus frames as HEX
- Multi-language terminal display: Chinese, Japanese, English, Russian, Arabic, etc.
- **RX Timeout (T/O)** auto-calculates optimal merge time by baud rate, or set manually
- For Modbus debugging, use "Quick Request" first to verify connectivity, then "Register Monitor" for continuous observation
- PLC controller supports custom brands — import your own register definitions
- All configuration and logs auto-save to `~/.serialrun/` directory, survives restart
- All feature panels (Log, Chart, Modbus, PLC, etc.) are independent OS windows — drag freely and resize
- Child windows always stay on top of the main window
- Data sent from Modbus/PLC/I2C panels is also displayed in the terminal
- Modbus panel supports configurable response timeout — increase for slow devices

---

## Recording & Replay

SerialRUN supports recording serial operations and replaying them for repeated test sequences.

### Recording

1. Find the "Record / Replay" section in the left panel
2. Click **Start Recording** — status bar shows red `● Recording`
3. Use the terminal normally to send data (all sent commands are recorded, including timing intervals)
4. Click **Stop Recording** — shows the number of recorded commands

### Saving Scripts

After recording, click the **Save** button:
- Supports `.txt` and `.srs` formats
- Script files are plain text, one command per line:
  - `SEND data content` — send command
  - `WAIT milliseconds` — wait for specified time

### Loading Scripts

Click the **Import** button to load a previously saved script file.

### Replaying

1. Ensure serial device is connected
2. Click the green **▶ Replay** button
3. Script executes automatically with recorded timing intervals
4. Progress bar shows current progress
5. Click red **■ Stop** at any time to interrupt replay

### Script File Example

```
# SerialRUN Script
SEND AT
WAIT 500
SEND AT+RST
WAIT 1000
SEND AT+CWMODE=1
```

---

## Terminal Advanced Features

### HEX Mode

The terminal supports independent TX/RX HEX modes:
- **TXT/HEX toggle** — Switch on the left of the input box, input parsed accordingly
- In HEX mode, sending data automatically strips spaces and `0x` prefixes
- Supports `0x` prefixes (e.g., `0x48 0x65`), auto-strips spaces

### Line Endings

Automatically append line endings when sending text:
- **None** — No characters appended
- **CR (\r)** — Carriage return, common for serial terminals
- **LF (\n)** — Line feed, common for network protocols
- **CRLF (\r\n)** — CR+LF, HTTP/Modbus ASCII standard

### Auto Send

Click **Auto Send** to repeatedly send the input content at a set interval:
- Configurable interval (100ms - 60s)
- Useful for heartbeat packets, polling requests, etc.
- Click **Stop Auto** to cancel

### DTR/RTS Control

When connected, DTR and RTS checkboxes appear in the terminal toolbar:
- **DTR (Data Terminal Ready)** — Data terminal ready signal
- **RTS (Request To Send)** — Request to send signal
- Some devices (e.g., Arduino) require DTR to be asserted for communication

### Keep Input

The **Keep input** checkbox on the left of the terminal input:
- When checked, sent data is not cleared from the input box
- Useful for repeatedly sending the same command during debugging
- Default: off (input clears after sending)

### RX Timeout (T/O)

Controls the wait time for merging fragmented data:
- **Auto mode** — Check T/O to auto-calculate optimal timeout by baud rate (displays as `XXms (auto)`)
  - ≤4800 baud: 100ms | ≤9600: 50ms | ≤19200: 30ms | ≤57600: 15ms | >57600: 10ms
- **Manual mode** — Uncheck to set custom value (10-2000ms), changes take effect immediately
- Auto mode syncs calculated value directly to the serial read thread without overwriting manual settings
- For devices with inter-chunk gaps >10ms (e.g., MCU responses), switch to manual mode and increase the timeout

### Checksum

Automatically append checksums when sending. Hover over CRC label for algorithm descriptions. Supports 8 algorithms:
- **CRC-16/MODBUS** — Modbus RTU standard (polynomial 0xA001)
- **CRC-16/CCITT** — CCITT standard (polynomial 0x1021)
- **CRC-16/XMODEM** — XMODEM file transfer protocol
- **CRC-32** — 32-bit CRC (ZIP/Ethernet)
- **LRC** — Longitudinal Redundancy Check (Modbus ASCII)
- **Checksum-8** — 8-bit additive
- **Checksum-16** — 16-bit additive

---

## Error Notification System

SerialRUN has a unified error notification system:

### Bottom Status Bar

- All errors/warnings display in the bottom status bar
- Red `✗` icon + error message
- Auto-dismiss after 5 seconds
- Also recorded to log and warning history

### Warning History

- Red dot + count displayed on the right side of status bar (e.g., `● 3`)
- Click to open warning/error history window
- Shows last 50 entries with timestamps and details
- Support clearing history
- History auto-saved to `~/.serialrun/warnings.json`

---

## MCP Server

SerialRUN includes a built-in MCP (Model Context Protocol) server, allowing AI assistants to remotely control serial devices via TCP.
RX data is automatically captured by background continuous monitoring and stored in a buffer. The read operation retrieves data from the buffer without blocking the serial port.

### Setup

In the left settings panel, find "MCP Server":
1. Check "Enable MCP Server"
2. Set the port number (default 9527)
3. Choose bind address:
   - **Localhost only** — Only local AI assistants can connect (recommended)
   - **All interfaces (LAN)** — Allow LAN AI assistants to connect
4. Click **Copy MCP Info** button to one-click copy connection info and tool descriptions for your AI assistant

### Available Tools (15)

| Tool | Description |
|------|-------------|
| `list_ports` | List all available serial ports |
| `connect` | Connect to serial port (supports baud rate, data bits, stop bits, parity, flow control) |
| `disconnect` | Disconnect from current connection |
| `send` | Send data (supports text and hex), no response wait |
| `read` | Read data from RX buffer (auto-captured by background monitor, non-blocking) |
| `send_command` | Send command and read response from buffer (recommended for AT commands) |
| `modbus_read` | Read Modbus RTU holding registers (with engineering value conversion) |
| `modbus_write` | Write Modbus RTU holding register |
| `plc_read` | Read all registers from a PLC preset brand |
| `plc_write` | Write to a PLC register by address |
| `status` | View connection status, byte counters, MCP server info |
| `get_config` | Read UI settings (supports all or single key) |
| `set_config` | Update UI setting (syncs to GUI immediately) |
| `get_access_log` | View access log (client IPs, tool calls, timestamps) |
| `get_device_info` | Get current device identification info (port, baud rate, connection status) |

### Usage

1. Open the Help panel (click `?` button)
2. In the MCP Server section, click **Copy MCP Guide**
3. Paste the copied content to any MCP-capable AI assistant
4. The AI assistant can then control your serial devices via TCP

Or in the left settings panel, click **Copy MCP Info** button to quickly copy connection info.

AI assistants can use the `tools/list` method to discover all available tools and their parameters.

### Access Log

- All MCP operations automatically log client IP addresses
- Left settings panel shows access record count, click to view detailed log
- Color coded: 🟢 CONNECT, 🔴 DISCONNECT, 🔵 CALL
- Query full log via `get_access_log` tool

### LAN Mode

When LAN mode is enabled:
- Shows your local IP address and port (e.g., `192.168.1.100:9527`)
- AI assistants on other LAN devices can connect
- All operations are automatically logged with client IP (use `get_access_log` tool to view)
- ⚠️ Use with caution — anyone on the network can control serial ports

---

## Data Persistence

SerialRUN automatically saves the following data to `~/.serialrun/` directory:

| File | Content | Save Trigger |
|------|---------|--------------|
| `config.toml` | Theme, language, baud rate, etc. | Auto-save on settings change |
| `terminal.json` | Terminal records (max 5000) | Saved each frame when new data arrives |
| `logs.json` | Log records (max 2000) | Saved each frame when new entries arrive |
| `warnings.json` | Warning/error history (max 1000) | Saved on each new entry |
| `mcp_access_log.json` | MCP access log (max 1000) | Saved on each new entry |

All data automatically restores on app restart.

---

## Shortcuts

- **Enter** — Send data in terminal input box
- **Ctrl+C** — Copy selected text in terminal

---

## Troubleshooting

**Q: Port list is empty?**
A: Click the refresh button `↻`. If still empty, check if the device is connected and drivers are installed. Linux users may need 
`sudo usermod -a -G dialout $USER` for serial port permissions.

**Q: No data received after connecting?**
A: Check that the baud rate matches the device settings, verify wiring (TX/RX cross-connected), and check flow control settings.

**Q: Auto baud rate detection fails?**
A: Ensure the device is sending data. Auto-detection requires data output during the connection attempt.

**Q: Modbus communication no response?**
A: Check slave address, verify function code and address range, validate CRC checksum.

**Q: PLC register shows ERR?**
A: The register read failed. Check wiring, slave ID, and ensure the register address exists on the PLC.

**Q: How to use MCP server?**
A: The MCP server is enabled by default on port 9527 (localhost). Configure in Settings > MCP Server:
- Change port number if 9527 is occupied
- Enable "All interfaces (LAN)" to allow remote AI assistants to connect
- AI assistants can control serial ports via MCP protocol (see Help > MCP Guide)

**Q: File transfer fails?**
A: Ensure both sides use the same protocol, check serial connection stability, try lowering the baud rate.

**Q: Data lost after restart?**
A: All data is auto-saved to `~/.serialrun/` directory. Check if the directory exists and has write permissions.
