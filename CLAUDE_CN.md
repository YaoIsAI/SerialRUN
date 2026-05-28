# SerialRUN — Agent 操作指南

本文档为 Claude Code agent 提供 SerialRUN 串口助手的操作指南。

## 快速命令

### 列出端口

```bash
serialrun list                    # 文本格式
serialrun list --format json      # JSON 格式
```

### 连接

```bash
serialrun connect /dev/ttyUSB0 -b 115200
serialrun connect COM1 -b 9600 -d 7 -s 2 -p odd -f hardware
```

### 发送数据

```bash
serialrun send COM1 "Hello\r\n"               # 文本
serialrun send COM1 "48 65 6C 6C 6F" --hex    # 十六进制
```

### 监听

```bash
serialrun monitor COM1 -t                  # 带时间戳
serialrun monitor COM1 -x                  # 十六进制模式
serialrun monitor COM1 -t -l output.log    # 带日志
```

### 脚本

```bash
serialrun record COM1 -o script.txt    # 录制
serialrun replay COM1 script.txt       # 回放
```

## Agent 模式 (JSON 输出)

### 列出端口

```bash
serialrun agent list-ports
```

输出:

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

### 发送数据

```bash
serialrun agent COM1 send "Hello" -b 115200
```

输出:

```json
{ "success": true, "bytes_written": 5 }
```

### 读取数据

```bash
serialrun agent COM1 read --timeout 1000 --max-bytes 1024
```

输出:

```json
{
  "success": true,
  "bytes_read": 10,
  "data_hex": "48656C6C6F20576F726C64",
  "data_text": "Hello World"
}
```

### 运行脚本

```bash
serialrun agent COM1 run-script script.txt
```

## 常用工作流

### ESP8266/ESP32 AT 指令测试

```bash
serialrun connect COM3 -b 115200
# 在交互模式中:
> AT
> AT+RST
> AT+CWMODE=1
> AT+CWJAP="WiFi","password"
```

### Modbus 抓包

```bash
serialrun monitor /dev/ttyUSB0 -x -t -l modbus.log
serialrun send /dev/ttyUSB0 "01 03 00 00 00 0A C5 CD" --hex
```

### 自动化测试

```bash
serialrun record COM1 -o test.txt
serialrun replay COM1 test.txt
```

## 故障排除

| 问题 | 解决方案 |
|------|----------|
| 端口未找到 | `serialrun list` 检查端口列表 |
| 权限不足 | `sudo usermod -a -G dialout $USER` (Linux) |
| 连接失败 | 确认波特率与设备一致 |
| 无数据接收 | 检查线缆和流控设置 |
