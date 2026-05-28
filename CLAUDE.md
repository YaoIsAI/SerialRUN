# SerialRUN — Agent Operation Guide

This document provides instructions for Claude Code agents to operate the SerialRUN serial port assistant.

## Quick Commands

### List Ports

```bash
serialrun list                    # Text format
serialrun list --format json      # JSON format
```

### Connect

```bash
serialrun connect /dev/ttyUSB0 -b 115200
serialrun connect COM1 -b 9600 -d 7 -s 2 -p odd -f hardware
```

### Send Data

```bash
serialrun send COM1 "Hello\r\n"               # Text
serialrun send COM1 "48 65 6C 6C 6F" --hex    # HEX
```

### Monitor

```bash
serialrun monitor COM1 -t                  # With timestamps
serialrun monitor COM1 -x                  # HEX mode
serialrun monitor COM1 -t -l output.log    # With logging
```

### Scripts

```bash
serialrun record COM1 -o script.txt    # Record
serialrun replay COM1 script.txt       # Replay
```

## Agent Mode (JSON Output)

### List Ports

```bash
serialrun agent list-ports
```

Output:

```json
{
  "success": true,
  "ports": [
    {
      "name": "/dev/ttyUSB0",
      "description": "USB Device 0403:6001",
      "manufacturer": "FTDI",
      "vid": 1027,
      "pid": 24577
    }
  ]
}
```

### Send Data

```bash
serialrun agent COM1 send "Hello" -b 115200
```

Output:

```json
{ "success": true, "bytes_written": 5 }
```

### Read Data

```bash
serialrun agent COM1 read --timeout 1000 --max-bytes 1024
```

Output:

```json
{
  "success": true,
  "bytes_read": 10,
  "data_hex": "48656C6C6F20576F726C64",
  "data_text": "Hello World"
}
```

### Run Script

```bash
serialrun agent COM1 run-script script.txt
```

## Common Workflows

### ESP8266/ESP32 AT Command Testing

```bash
serialrun connect COM3 -b 115200
# Then in interactive mode:
> AT
> AT+RST
> AT+CWMODE=1
> AT+CWJAP="WiFi","password"
```

### Modbus Traffic Capture

```bash
serialrun monitor /dev/ttyUSB0 -x -t -l modbus.log
serialrun send /dev/ttyUSB0 "01 03 00 00 00 0A C5 CD" --hex
```

### Automated Testing

```bash
serialrun record COM1 -o test.txt
serialrun replay COM1 test.txt
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Port not found | `serialrun list` to check |
| Permission denied | `sudo usermod -a -G dialout $USER` (Linux) |
| Connection failed | Verify baud rate matches device |
| No data received | Check cable and flow control |
