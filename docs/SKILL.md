# SerialRUN Skill Reference

[中文版](SKILL_CN.md)

---

## Overview

SerialRUN provides serial port communication capabilities for embedded development workflows. It supports both CLI automation and interactive GUI usage.

## Core Capabilities

### 1. Port Management

- Enumerate all available serial ports
- Configure baud rate, data bits, stop bits, parity, flow control
- Connect / disconnect / auto-reconnect

### 2. Data Communication

- **Text mode**: UTF-8 encoded send/receive
- **HEX mode**: Hexadecimal data display and transmission
- **Mixed mode**: Simultaneous text and HEX display

### 3. Protocol Analysis

- Built-in Modbus RTU parser (CRC-16, frame decode)
- Custom protocol patterns with regex matching
- Protocol logging and traffic analysis

### 4. Script Operations

- Record serial sessions to JSON or text scripts
- Replay scripts with timing preserved
- Script editing and parameterization

### 5. File Transfer

- XMODEM (128-byte blocks)
- YMODEM (1024-byte blocks with batch)
- ZMODEM (with resume support)

### 6. Plugin System

- Dynamic loading (.so / .dll / .dylib)
- C FFI interface
- Custom command execution

## Integration Points

### CLI Pipeline

```bash
serialrun list --format json | jq '.ports[0].name'
serialrun send COM1 "test" && serialrun monitor COM1 -t -l output.log
```

### Agent JSON API

```bash
serialrun agent list-ports           # List ports
serialrun agent COM1 send "data"     # Send
serialrun agent COM1 read            # Read
serialrun agent COM1 run-script.txt  # Execute script
```

### Plugin API

```rust
#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char;

#[no_mangle]
pub extern "C" fn plugin_execute(
    command: *const c_char,
    params: *const c_char
) -> *mut c_char;
```

## Best Practices

### Port Configuration

- Always match baud rate between device and assistant
- Use 8N1 (8 data bits, No parity, 1 stop bit) as default
- Enable flow control for high-throughput applications

### Data Handling

- Use HEX mode for binary protocols (Modbus, custom)
- Enable timestamps for debugging sessions
- Log important sessions for later analysis

### Script Development

- Start with manual recording
- Add appropriate delays between commands
- Test scripts thoroughly before automation
