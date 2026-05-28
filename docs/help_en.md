# SerialTap User Guide

SerialTap is a feature-rich serial port debugging assistant supporting serial communication, Modbus debugging, PLC control, CAN bus analysis, I2C/SPI debugging, firmware flashing, and more. Built-in MCP server enables AI assistants to remotely control serial devices via TCP.

---

## Quick Start

1. Connect your serial device to the computer via USB
2. Click the refresh button `↻` in the left panel to detect ports
3. Select a serial port from the dropdown (e.g., COM3)
4. Set the baud rate (default 115200, works for most devices)
5. Click the green "Connect" button
6. Type commands in the bottom terminal input box and press Enter or click "Send"

If you don't know the baud rate, click the **Auto** button to auto-detect.

---

## Interface Layout

### Top Toolbar

The toolbar contains 13 function buttons. Click to open/close the corresponding window:

| Button | Function | Description |
|--------|----------|-------------|
| Log | Log Viewer | Display send/receive records, export to CSV |
| Chart | Data Chart | Real-time data rate curve |
| PLC | PLC Controller | Supports Siemens, Mitsubishi, Delta, Omron |
| Mod | Modbus Debug | Quick register read/write, 8 function codes |
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
- **Dark/Light** — Switch dark/light theme
- **EN/中** — Switch English/Chinese interface

### Left Panel

- **Serial Port** — Select serial device from dropdown, click `↻` to refresh port list
- **Baud Rate** — Select communication speed: 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600
- **Auto Button** — Auto-detect baud rate by trying each speed until data is received
- **Port Config** — Data bits (5-8), stop bits (1/2), parity (None/Odd/Even), flow control (None/Software/Hardware)
- **Display Settings** — Hex mode, timestamp display, auto-scroll
- **Auto Reply** — Set match pattern and reply content, auto-send reply when matching data is received
- **Record/Replay** — Record serial operation scripts, save and replay later

### Terminal Area

Terminal displays all send/receive data with color-coded directions:
- Green `↓ RX` — Received data
- Blue `↑ TX` — Sent data
- Yellow `⚙ SYS` — System messages

Top toolbar options:
- **HEX** — Toggle hex input mode
- **Show Timestamp** — Display timestamp before each message
- **Auto Scroll** — Auto-scroll to latest message
- **CRC** — Auto-append checksum on send (CRC16/MODBUS, CRC16/CCITT, CRC32, LRC, SUM8)

---

## Feature Details

### Log Viewer

Records all system events including connect/disconnect, data send/receive, errors, etc.
- Three log levels: INFO/WARN/ERR with color coding
- Displays current entry count
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
- Display last request and response hex data

**Register Monitor**
- Configurable poll interval (100-10000ms)
- Real-time display of register address, raw value, formatted value, and last update time

**Frame Log**
- Records all Modbus frame history (up to 200 entries)
- Shows timestamp, request/response hex, and decoded content

### PLC Controller

Supports Modbus communication with major PLC brands:
- **Brands**: Siemens, Mitsubishi, Delta, Omron, Custom
- **Data Types**: BOOL, UINT16, INT16, UINT32, FLOAT
- Register table shows address, name, type, value, scale factor, unit
- BOOL registers can be toggled via checkbox
- Supports read-all and single-read modes
- Auto-poll mode with configurable interval (200-10000ms)

### File Transfer

Transfer files via serial port with these protocols:
- **XMODEM** — Basic checksum
- **XMODEM-CRC** — CRC checksum, more reliable
- **YMODEM** — Supports file name and size info
- **ZMODEM** — Advanced protocol with resume support

Workflow: Select protocol → Click send/receive → Choose file → Wait for completion

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

CAN bus analysis tool based on SLCAN protocol:
- **Capture** — Start/stop CAN frame capture
- **Statistics** — Total frames, error frames, unique IDs, bus load estimate
- **ID Statistics** — Per-ID frame count, frequency, time intervals
- **Frame List** — Timestamp, direction, ID (standard/extended), DLC, data
- **Transmit** — Enter ID and data to send CAN frames
- **Filter** — Filter display by ID
- CSV export supported, up to 2000 frames buffer

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

Workflow: Select MCU type → Connect → Select firmware file → Erase (optional) → Flash

### Register Editor

Custom device register map table:
- Add/remove register entries
- Set address, name, data type, value, description
- Import from CSV/JSON, export to CSV
- Optional alarm feature with threshold setting

### Plugin Manager

Scans `plugins/` directory for extension plugins:
- Displays plugin name, version, author, and load status
- Supports Windows (.dll), Linux (.so), macOS (.dylib) platforms
- Click refresh button to re-scan

---

## Tips

- When debugging a new device, start with **115200** baud rate
- If you don't know the baud rate, use the **Auto** button
- In HEX mode, sending data automatically strips spaces and `0x` prefixes
- You can append **CRC checksums** when sending text for data integrity
- **Auto Reply** is useful for protocol debugging that requires responses
- Use **Record/Replay** to quickly repeat test sequences
- For Modbus debugging, use "Quick Request" first to verify connectivity, then "Register Monitor" for continuous observation
- PLC controller supports custom brands — import your own register definitions

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

**Q: File transfer fails?**
A: Ensure both sides use the same protocol, check serial connection stability, try lowering the baud rate.
