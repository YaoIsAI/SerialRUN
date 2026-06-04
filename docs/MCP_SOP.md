# SerialRUN MCP 操作指南 (SOP)

> 本文档面向 AI agent，说明如何通过 SerialRUN MCP 工具完成常见串口任务。
> MCP 服务器地址：`127.0.0.1:9527`，协议：JSON-RPC over TCP，以 `\n` 为分隔符。

## 重要原则

- **直接调用 MCP 工具**，不要写 Python/JS 脚本去操作串口
- **不要用 pyserial、socket、serialport 等库**，MCP 工具已经封装了所有串口操作
- 一个任务 = 多个工具调用的组合，按顺序执行即可
- 工具调用之间共享同一个串口连接，不需要重复 connect
- HEX 数据格式：空格分隔，例如 `"01 03 00 00 00 0A"`

---

## 场景 1: 扫描并连接串口

**步骤**：`list_ports` → `connect`

**说明**：先扫描可用端口，再选择目标端口连接。

**对话示例**：

```
用户: "帮我连接串口"
AI 思考:
  1. 调用 list_ports，获取可用端口列表
  2. 从结果中选择目标端口
  3. 调用 connect(port="COM3", baud_rate=115200)

调用序列:
  tools/call list_ports()
  tools/call connect(port="COM3", baud_rate=115200)
```

---

## 场景 2: 发送 AT 指令并读取响应

**步骤**：`connect` → `send_command`

**说明**：`send_command` 会自动暂停读循环，发送命令后等待响应，最适合 AT 指令交互。

**对话示例**：

```
用户: "发 AT 指令测试 WiFi 模块"
AI 思考:
  1. 调用 send_command，发送 AT 指令并等待响应

调用序列:
  tools/call send_command(command="AT", timeout_ms=2000)
  tools/call send_command(command="AT+CWMODE=1", timeout_ms=2000)
  tools/call send_command(command="AT+CWJAP=\"WiFi\",\"password\"", timeout_ms=5000)
```

**参数建议**：
- 普通 AT 指令：`timeout_ms=2000`
- WiFi 连接等慢操作：`timeout_ms=5000` 甚至更长

---

## 场景 3: 发送自定义数据并读取

**步骤**：`send(pause_after=true)` → `read(resume=false)`

**说明**：分步发送和读取，适合需要精确控制的场景。`pause_after=true` 确保发送后暂停读循环，`read` 时用 `resume=false` 保持暂停状态。

**对话示例**：

```
用户: "发送十六进制数据 01 03 00 00 00 0A，然后读取响应"
AI 思考:
  1. 先发送数据，设置 pause_after 暂停读循环
  2. 再读取响应

调用序列:
  tools/call send(data="01 03 00 00 00 0A", hex=true, pause_after=true)
  tools/call read(format="hex", resume=false, timeout_ms=2000)
```

---

## 场景 4: Modbus 读取寄存器

**步骤**：`connect` → `modbus_read`

**说明**：支持工程值转换（scale/offset/unit）。`modbus_read` 自动构建 Modbus RTU 请求帧并解析响应。

**对话示例**：

```
用户: "读取从站 1 的温度值，地址 100，量程 0-100 度对应 0-4095"
AI 思考:
  1. scale = 100.0 / 4095.0，offset = 0，单位 °C
  2. 调用 modbus_read 并传入工程值转换参数

调用序列:
  tools/call modbus_read(slave_id=1, address=100, quantity=1, scale=0.02442, unit="°C")
```

**批量读取**：

```
用户: "读取从站 2 地址 0-9 共 10 个寄存器"
调用序列:
  tools/call modbus_read(slave_id=2, address=0, quantity=10)
```

---

## 场景 5: Modbus 写入寄存器

**步骤**：`connect` → `modbus_write`

**对话示例**：

```
用户: "设置从站 1 的地址 200 值为 500"
AI 思考:
  1. 调用 modbus_write 写入单个寄存器

调用序列:
  tools/call modbus_write(slave_id=1, address=200, value=500)
```

---

## 场景 6: PLC 全量读取

**步骤**：`connect` → `plc_read`

**说明**：`plc_read` 按品牌预设自动读取该品牌的所有寄存器。支持品牌：Siemens、Mitsubishi、Delta、Omron。

**对话示例**：

```
用户: "读取西门子 S7-1200 的所有寄存器"
AI 思考:
  1. 指定品牌为 Siemens，调用 plc_read

调用序列:
  tools/call plc_read(brand="Siemens", slave_id=1)
```

**PLC 写入**：

```
用户: "写入三菱 PLC 地址 D100 值为 1234"
调用序列:
  tools/call plc_write(brand="Mitsubishi", slave_id=1, address=100, value=1234)
```

---

## 场景 7: 修改串口参数（需重连）

**步骤**：`disconnect` → `connect(新参数)`

**说明**：波特率、数据位、停止位、校验位、流控等参数修改后**必须重连**才能生效。

**对话示例**：

```
用户: "把波特率改成 9600"
AI 思考:
  1. 串口参数修改需要先断开再重连
  2. 记住当前连接的端口名

调用序列:
  tools/call disconnect()
  tools/call connect(port="COM3", baud_rate=9600)
```

**修改多个参数**：

```
用户: "改成 9600 波特率，偶校验，2 个停止位"
调用序列:
  tools/call disconnect()
  tools/call connect(port="COM3", baud_rate=9600, parity="Even", stop_bits=2)
```

---

## 场景 8: 修改界面设置

**步骤**：`set_config`

**说明**：界面类设置（时间戳、自动滚动、行尾符等）修改后**立即生效**，不需要重连。

**对话示例**：

```
用户: "关闭终端时间戳"
调用序列:
  tools/call set_config(key="show_timestamp", value=false)
```

**查看当前设置**：

```
用户: "现在有哪些配置？"
调用序列:
  tools/call get_config()
```

---

## 场景 9: 控制 DTR/RTS 信号

**步骤**：`set_config(key="dtr" | "rts", value=true | false)`

**说明**：DTR/RTS 信号修改立即生效，常用于嵌入式设备复位、进入 bootloader 等场景。

**对话示例**：

```
用户: "拉高 DTR 信号"
调用序列:
  tools/call set_config(key="dtr", value=true)
```

**典型用法 - ESP32 进入下载模式**：

```
用户: "重启 ESP32 进入下载模式"
调用序列:
  tools/call set_config(key="rts", value=true)
  tools/call set_config(key="dtr", value=false)
  tools/call set_config(key="rts", value=false)
```

---

## 场景 10: 录制串口数据

**步骤**：`send` + `read` 循环

**说明**：定期调用 `read` 获取数据并保存。适合抓取一段时间内的串口通信。

**对话示例**：

```
用户: "录制串口数据 10 秒"
AI 思考:
  1. 循环调用 read，每次读取后将数据拼接
  2. 10 秒后停止

调用序列（伪逻辑）:
  while 未超过 10 秒:
    result = tools/call read(timeout_ms=1000, format="text")
    将 result 保存到结果列表
```

---

## 场景 11: 检查连接状态

**步骤**：`status`

**对话示例**：

```
用户: "现在串口什么状态"
调用序列:
  tools/call status()
```

**获取设备信息**：

```
用户: "显示设备信息"
调用序列:
  tools/call get_device_info()
```

---

## 场景 12: 查看访问日志

**步骤**：`get_access_log`

**说明**：所有 MCP 操作自动记录客户端 IP 和操作详情，用于审计和排查问题。

**对话示例**：

```
用户: "查看最近的操作日志"
调用序列:
  tools/call get_access_log(limit=20)
```

---

## 常见错误处理

| 错误信息 | 原因 | 解决方案 |
|---------|------|---------|
| "Port not found" | 串口名不存在 | 调用 `list_ports` 确认端口名 |
| "Connection failed" | 波特率不匹配或设备未上电 | 检查波特率，确认设备已供电 |
| "Timeout" | 设备未响应 | 增加 `timeout_ms`，检查接线和协议 |
| "Permission denied" | 无串口权限 | Linux: `sudo usermod -a -G dialout $USER` 后重新登录 |
| "Port busy" | 端口被其他程序占用 | 关闭其他串口工具后重试 |
| "Unknown method" | 工具名拼写错误 | 检查工具名是否在 15 个工具列表中 |

---

## 参数修改生效规则

| 参数类型 | 修改方式 | 是否需要重连 |
|---------|---------|:-----------:|
| 波特率 (baud_rate) | disconnect → connect | 需要重连 |
| 数据位 (data_bits) | disconnect → connect | 需要重连 |
| 停止位 (stop_bits) | disconnect → connect | 需要重连 |
| 校验位 (parity) | disconnect → connect | 需要重连 |
| 流控 (flow_control) | disconnect → connect | 需要重连 |
| DTR 信号 | set_config(key="dtr") | 立即生效 |
| RTS 信号 | set_config(key="rts") | 立即生效 |
| 时间戳显示 (show_timestamp) | set_config | 立即生效 |
| 自动滚动 (auto_scroll) | set_config | 立即生效 |
| 行尾符 (line_ending) | set_config | 立即生效 |
| 自动发送 (auto_send) | set_config | 立即生效 |
| 自动回复 (auto_reply) | set_config | 立即生效 |

---

## MCP 工具速查表

| # | 工具名 | 功能 | 关键参数 |
|---|--------|------|---------|
| 1 | `list_ports` | 扫描可用串口 | 无 |
| 2 | `connect` | 连接串口 | port, baud_rate, data_bits, stop_bits, parity, flow_control |
| 3 | `disconnect` | 断开连接 | 无 |
| 4 | `send` | 发送数据 | data, hex, pause_after |
| 5 | `read` | 读取数据 | timeout_ms, max_bytes, resume, format |
| 6 | `send_command` | 发送命令并读响应 | command, timeout_ms |
| 7 | `modbus_read` | Modbus 读寄存器 | slave_id, address, quantity, scale, offset, unit |
| 8 | `modbus_write` | Modbus 写寄存器 | slave_id, address, value |
| 9 | `plc_read` | PLC 全量读取 | brand, slave_id |
| 10 | `plc_write` | PLC 写入 | brand, slave_id, address, value |
| 11 | `status` | 查看连接状态 | 无 |
| 12 | `get_config` | 获取设置 | key (可选) |
| 13 | `set_config` | 修改设置 | key, value |
| 14 | `get_access_log` | 查看访问日志 | limit |
| 15 | `get_device_info` | 获取设备信息 | 无 |

---

## 注意事项

1. **端口排他性**：Windows 上串口是独占资源，同一时间只能有一个连接
2. **超时设置**：`send_command` 建议 timeout_ms 为 500-2000ms，慢速设备适当增加
3. **HEX 格式**：十六进制数据用空格分隔，例如 `"48 65 6C 6C 6F"`
4. **文本行尾**：文本发送自动追加 `\r\n`，不需要手动添加
5. **并发控制**：多个客户端可同时连接 MCP 服务器，但串口操作会排队执行
6. **工具选择**：简单收发用 `send` + `read`，AT 指令交互用 `send_command` 更方便
