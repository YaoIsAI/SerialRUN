# SerialTap — Agent Operation Guide / Agent 操作指南

This document provides instructions for Claude Code agents to operate the SerialTap serial port assistant.

本文档为 Claude Code agent 提供 SerialTap 串口助手的操作指南。

---

## Quick Commands / 快速命令

### List Ports / 列出端口

```bash
serialtap list                    # Text format / 文本格式
serialtap list --format json      # JSON format / JSON 格式
```

### Connect / 连接

```bash
serialtap connect /dev/ttyUSB0 -b 115200
serialtap connect COM1 -b 9600 -d 7 -s 2 -p odd -f hardware
```

### Send Data / 发送数据

```bash
serialtap send COM1 "Hello\r\n"               # Text / 文本
serialtap send COM1 "48 65 6C 6C 6F" --hex    # HEX / 十六进制
```

### Monitor / 监听

```bash
serialtap monitor COM1 -t                  # With timestamps / 带时间戳
serialtap monitor COM1 -x                  # HEX mode / 十六进制模式
serialtap monitor COM1 -t -l output.log    # With logging / 带日志
```

### Scripts / 脚本

```bash
serialtap record COM1 -o script.txt    # Record / 录制
serialtap replay COM1 script.txt       # Replay / 回放
```

---

## Agent Mode (JSON Output) / Agent 模式 (JSON 输出)

### List Ports / 列出端口

```bash
serialtap agent list-ports
```

Output / 输出:
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

### Send Data / 发送数据

```bash
serialtap agent COM1 send "Hello" -b 115200
```

Output / 输出:
```json
{ "success": true, "bytes_written": 5 }
```

### Read Data / 读取数据

```bash
serialtap agent COM1 read --timeout 1000 --max-bytes 1024
```

Output / 输出:
```json
{
  "success": true,
  "bytes_read": 10,
  "data_hex": "48656C6C6F20576F726C64",
  "data_text": "Hello World"
}
```

### Run Script / 运行脚本

```bash
serialtap agent COM1 run-script script.txt
```

---

## Common Workflows / 常用工作流

### ESP8266/ESP32 AT Command Testing / AT 指令测试

```bash
serialtap connect COM3 -b 115200
# Then in interactive mode / 交互模式中:
> AT
> AT+RST
> AT+CWMODE=1
> AT+CWJAP="WiFi","password"
```

### Modbus Traffic Capture / Modbus 抓包

```bash
serialtap monitor /dev/ttyUSB0 -x -t -l modbus.log
serialtap send /dev/ttyUSB0 "01 03 00 00 00 0A C5 CD" --hex
```

### Automated Testing / 自动化测试

```bash
# Record manual test / 录制手动测试
serialtap record COM1 -o test.txt

# Replay in CI / 在 CI 中回放
serialtap replay COM1 test.txt
```

---

## Troubleshooting / 故障排除

| Problem / 问题 | Solution / 解决方案 |
|----------------|---------------------|
| Port not found / 端口未找到 | `serialtap list` to check / 检查端口列表 |
| Permission denied / 权限不足 | `sudo usermod -a -G dialout $USER` (Linux) |
| Connection failed / 连接失败 | Verify baud rate matches device / 确认波特率匹配 |
| No data received / 无数据接收 | Check cable and flow control / 检查线缆和流控 |
