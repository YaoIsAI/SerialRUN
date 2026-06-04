# SerialRUN MCP API Reference

SerialRUN 内置 MCP (Model Context Protocol) 服务器，允许 AI 助手通过 TCP 远程控制串口设备。所有串口操作由 SerialRUN GUI 执行，MCP 负责转发指令和推送数据。

---

## 目录

1. [协议格式](#1-协议格式)
2. [工具一览](#2-工具一览)
3. [工具详细说明](#3-工具详细说明)
4. [事件推送](#4-事件推送)
5. [错误码](#5-错误码)
6. [注意事项](#6-注意事项)
7. [客户端示例](#7-客户端示例)

---

## 1. 协议格式

| 属性 | 值 |
|------|-----|
| 协议 | JSON-RPC 2.0 over TCP |
| 默认地址 | `127.0.0.1:9527` |
| 字符编码 | UTF-8 |
| 消息分隔符 | 换行符 `\n`（每条 JSON 占一行） |

### 请求

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "工具名",
    "arguments": { ... }
  }
}
```

### 成功响应

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "响应内容"
      }
    ]
  }
}
```

### 错误响应

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -1,
    "message": "错误描述"
  }
}
```

### 初始化

连接后先发送初始化请求：

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
```

响应会返回协议版本和服务器信息（`serialrun-mcp v0.2.0`）。初始化完成后即可调用工具。

---

## 2. 工具一览

| # | 工具名 | 功能 | 需要已连接 |
|---|--------|------|-----------|
| 1 | `list_ports` | 列出所有可用串口 | 否 |
| 2 | `connect` | 连接到指定串口 | 否 |
| 3 | `disconnect` | 断开当前串口连接 | 否 |
| 4 | `send` | 发送数据（文本或十六进制） | 是 |
| 5 | `read` | 从接收缓冲区读取数据 | 是 |
| 6 | `send_command` | 发送命令并等待响应（一问一答） | 是 |
| 7 | `modbus_read` | 读取 Modbus RTU 保持寄存器 | 是 |
| 8 | `modbus_write` | 写入 Modbus RTU 保持寄存器 | 是 |
| 9 | `plc_read` | 读取 PLC 预设寄存器 | 是 |
| 10 | `plc_write` | 写入 PLC 指定寄存器 | 是 |
| 11 | `status` | 查看连接状态和收发统计 | 否 |
| 12 | `get_config` | 读取配置项 | 是 |
| 13 | `set_config` | 修改配置项 | 是 |
| 14 | `get_access_log` | 查看 MCP 访问日志 | 否 |
| 15 | `get_device_info` | 查看设备标识信息 | 否 |
| 16 | `clear_buffers` | 清空 TX/RX 缓冲区 | 是 |
| 17 | `set_dtr` | 设置 DTR 硬件信号（立即生效） | 是 |
| 18 | `set_rts` | 设置 RTS 硬件信号（立即生效） | 是 |
| 19 | `get_config_keys` | 列出所有可用配置键及有效值 | 否 |

---

## 3. 工具详细说明

### 3.1 list_ports -- 列出可用串口

扫描系统中所有可用的串口设备并返回列表。用这个工具来发现设备，拿到端口名后再调用 `connect` 连接。

**参数**: 无

**返回示例**:

```json
{
  "content": [{
    "type": "text",
    "text": "[\n  {\n    \"name\": \"/dev/ttyUSB0\",\n    \"description\": \"USB Device\",\n    \"manufacturer\": \"FTDI\"\n  }\n]"
  }]
}
```

---

### 3.2 connect -- 连接串口

连接到指定串口。**如果当前已经连接了别的串口，会自动断开后再连接新的**，无需先手动 `disconnect`。

**参数**:

| 参数名 | 类型 | 必填 | 默认值 | 有效值 | 说明 |
|--------|------|------|--------|--------|------|
| `port` | string | 是 | - | 系统串口名 | 端口名，如 `COM1`、`/dev/ttyUSB0` |
| `baud_rate` | integer | 否 | 115200 | 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600 | 波特率 |
| `data_bits` | integer | 否 | 8 | 5, 6, 7, 8 | 数据位 |
| `stop_bits` | integer | 否 | 1 | 1, 2 | 停止位 |
| `parity` | string | 否 | "None" | None, Odd, Even | 校验方式 |
| `flow_control` | string | 否 | "None" | None, Software, Hardware | 流控方式 |

> 所有串口参数（baud_rate 等）在本次 connect 调用中立即生效。如果连接后再用 `set_config` 修改这些参数，需要 `disconnect` + `connect` 才能生效。

**示例**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"connect","arguments":{"port":"COM3","baud_rate":115200,"parity":"None"}}}
```

**成功响应**:

```json
{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"Connected to COM3 at 115200 baud"}]}}
```

---

### 3.3 disconnect -- 断开连接

断开当前串口连接并释放端口资源。断开后所有需要连接状态的工具（send、read、modbus 等）将无法使用，直到重新 `connect`。

**参数**: 无

**示例**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"disconnect","arguments":{}}}
```

---

### 3.4 send -- 发送数据

向串口发送数据。支持文本和十六进制两种模式。文本模式会自动处理行尾符。

**参数**:

| 参数名 | 类型 | 必填 | 默认值 | 有效值 | 说明 |
|--------|------|------|--------|--------|------|
| `data` | string | 是 | - | 非空字符串 | 要发送的数据。hex=true 时为空格分隔的十六进制字符串，如 `"48 65 6C 6C 6F"` |
| `hex` | boolean | 否 | false | true/false | 设为 true 时，data 按十六进制解析 |
| `pause_after` | boolean | 否 | false | true/false | 设为 true 会暂停后台读取循环。适用于"先发后读"场景：发送命令后暂停读取，再用 `read` 精确读取响应。默认 false 表示发送后读取循环继续运行 |

> `pause_after` 是 `send` + `read` 配合使用的关键参数。设为 true 时，后台不再自动缓存数据，下一次 `read` 调用才能拿到设备的响应。

**示例 -- 发送文本**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"send","arguments":{"data":"AT\r\n"}}}
```

**示例 -- 发送十六进制**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"send","arguments":{"data":"01 03 00 00 00 0A C5 CD","hex":true}}}
```

**示例 -- 发送后暂停读取**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"send","arguments":{"data":"AT+RST\r\n","pause_after":true}}}
```

**成功响应**:

```json
{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"Sent 5 bytes (read loop paused)"}]}}
```

---

### 3.5 read -- 读取数据

从串口接收缓冲区读取数据。串口连接后，SerialRUN 会在后台持续接收数据并缓存，`read` 从缓冲区取数据。

**参数**:

| 参数名 | 类型 | 必填 | 默认值 | 有效值 | 说明 |
|--------|------|------|--------|--------|------|
| `timeout_ms` | integer | 否 | 1000 | 100-30000 | 等待数据的超时时间（毫秒）。如果缓冲区已有数据则立即返回 |
| `format` | string | 否 | "hex" | hex, text, raw | 输出格式：hex=空格分隔大写十六进制，text=UTF-8 文本，raw=Base64 编码 |
| `resume` | boolean | 否 | true | true/false | 读取后是否恢复后台读取循环。当配合 `send(pause_after=true)` 使用时，第一次 `read` 设 resume=false 保持暂停，后续恢复时设为 true |

**三种输出格式说明**:

- `hex`: `"4F 4B 0D 0A"` -- 空格分隔的十六进制，每个字节两位大写
- `text`: `"OK\r\n"` -- 直接按 UTF-8 解码，无法解码的字节显示为替换字符
- `raw`: `"T0tACg=="` -- Base64 编码，适合传输二进制数据

**示例 -- 常规读取**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"read","arguments":{"timeout_ms":1000,"format":"text"}}}
```

**示例 -- 配合 pause_after 使用**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"read","arguments":{"timeout_ms":2000,"resume":false,"format":"text"}}}
```

**成功响应**:

```json
{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"OK","format":"text","length":2}]}}
```

---

### 3.6 send_command -- 发送命令并等待响应

"一问一答"模式：发送一条命令，然后等待设备返回响应。**这是与 AT 命令设备交互的推荐工具**。内部会自动在命令末尾追加 `\r\n`（如果命令本身没有的话）。

**参数**:

| 参数名 | 类型 | 必填 | 默认值 | 有效值 | 说明 |
|--------|------|------|--------|--------|------|
| `command` | string | 是 | - | 非空字符串 | 要发送的命令，不需要手动加 `\r\n` |
| `timeout_ms` | integer | 否 | 1000 | 100-30000 | 等待设备响应的超时时间（毫秒） |

**与 send + read 的区别**: `send_command` 是原子操作，内部自动暂停读取循环、发送、等待响应、恢复。而 `send` + `read` 是分步操作，适合更复杂的场景。

**示例**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"send_command","arguments":{"command":"AT","timeout_ms":1000}}}
```

**成功响应**:

```json
{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"OK"}]}}
```

---

### 3.7 modbus_read -- 读取 Modbus 寄存器

发送 Modbus RTU 读保持寄存器请求（功能码 0x03）。支持可选的工程值转换（原始值 x scale + offset）。

**参数**:

| 参数名 | 类型 | 必填 | 默认值 | 有效值 | 说明 |
|--------|------|------|--------|--------|------|
| `slave_id` | integer | 否 | 1 | 1-247 | Modbus 从站地址 |
| `address` | integer | 是 | - | 0-65535 | 起始寄存器地址 |
| `quantity` | integer | 否 | 1 | 1-125 | 要读取的寄存器数量 |
| `scale` | number | 否 | 1.0 | 任意浮点数 | 比例因子。工程值 = 原始值 x scale + offset |
| `offset` | number | 否 | 0.0 | 任意浮点数 | 偏移量，加在 scale 转换之后 |
| `unit` | string | 否 | "" | 任意字符串 | 工程值的单位标签，如 `"degC"`、`"%"` |

> scale、offset、unit 中任意一个与默认值不同时，返回结果会包含 "Engineering" 工程值字段。

**示例 -- 读取 10 个寄存器**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"modbus_read","arguments":{"slave_id":1,"address":0,"quantity":10}}}
```

**示例 -- 读取温度传感器，原始值除以 10 并加偏移**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"modbus_read","arguments":{"slave_id":1,"address":100,"quantity":1,"scale":0.1,"offset":-40.0,"unit":"degC"}}}
```

**成功响应**:

```json
{
  "content": [{
    "type": "text",
    "text": "Read 10 registers from slave 1\nHEX: 01 03 14 00 01 00 02 ...\nValues: [1, 2, 3, ...]"
  }]
}
```

---

### 3.8 modbus_write -- 写入 Modbus 寄存器

发送 Modbus RTU 写单个保持寄存器请求（功能码 0x06）。

**参数**:

| 参数名 | 类型 | 必填 | 默认值 | 有效值 | 说明 |
|--------|------|------|--------|--------|------|
| `slave_id` | integer | 否 | 1 | 1-247 | Modbus 从站地址 |
| `address` | integer | 是 | - | 0-65535 | 寄存器地址 |
| `value` | integer | 是 | - | 0-65535 | 要写入的无符号 16 位值 |

**示例 -- 向寄存器 0x0001 写入值 100**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"modbus_write","arguments":{"slave_id":1,"address":1,"value":100}}}
```

**成功响应**:

```json
{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"Wrote 100 to register 0x0001 (slave 1)\nResponse: 01 06 00 01 00 64 99 ..."}]}}
```

---

### 3.9 plc_read -- 读取 PLC 寄存器

按 PLC 品牌预设读取所有寄存器。支持 Siemens、Mitsubishi、Delta、Omron 四个品牌的默认寄存器组。

**参数**:

| 参数名 | 类型 | 必填 | 默认值 | 有效值 | 说明 |
|--------|------|------|--------|--------|------|
| `brand` | string | 否 | "Siemens" | Siemens, Mitsubishi, Delta, Omron | PLC 品牌 |
| `slave_id` | integer | 否 | 1 | 1-247 | Modbus 从站地址 |

**示例**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"plc_read","arguments":{"brand":"Siemens","slave_id":1}}}
```

**成功响应**:

```json
{
  "content": [{
    "type": "text",
    "text": "Siemens PLC (S7-200 SMART) slave 1 - 8 registers:\n[...]"
  }]
}
```

---

### 3.10 plc_write -- 写入 PLC 寄存器

向指定 PLC 品牌的某个寄存器地址写入值。

**参数**:

| 参数名 | 类型 | 必填 | 默认值 | 有效值 | 说明 |
|--------|------|------|--------|--------|------|
| `brand` | string | 否 | "Siemens" | Siemens, Mitsubishi, Delta, Omron | PLC 品牌 |
| `slave_id` | integer | 否 | 1 | 1-247 | Modbus 从站地址 |
| `address` | integer | 是 | - | 0-65535 | 寄存器地址 |
| `value` | number | 是 | - | 0-65535 | 要写入的值（内部转为 u16） |

**示例**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"plc_write","arguments":{"brand":"Mitsubishi","slave_id":1,"address":0,"value":1}}}
```

**成功响应**:

```json
{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"Wrote 1 to Mitsubishi register 0x0000 (slave 1)"}]}}
```

---

### 3.11 status -- 查看连接状态

返回当前串口连接状态、MCP 服务器信息、以及 AI 客户端的收发字节统计。不需要已连接即可调用。

**参数**: 无

**返回字段说明**:

| 字段 | 说明 |
|------|------|
| `connection.gui_connected` | GUI 是否连接了串口 |
| `connection.ai_connected` | AI (MCP) 客户端是否通过 connect 连接了串口 |
| `connection.port` | AI 连接的端口名，未连接时为 "N/A" |
| `connection.baud_rate` | AI 连接的波特率，未连接时为 0 |
| `mcp.active_clients` | 当前活跃的 MCP 客户端数量 |
| `counters.ai_tx_bytes` | AI 客户端累计发送字节数 |
| `counters.ai_rx_bytes` | AI 客户端累计接收字节数 |

**示例**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"status","arguments":{}}}
```

---

### 3.12 get_config -- 读取配置

获取 SerialRUN 的 UI/串口配置。不传 key 返回所有配置，传 key 返回单个值。

**参数**:

| 参数名 | 类型 | 必填 | 默认值 | 有效值 | 说明 |
|--------|------|------|--------|--------|------|
| `key` | string | 否 | - | 见下表 | 配置项名称。省略则返回全部配置 |

**有效 key 列表**:

| key | 类型 | 默认值 | 修改方式 | 说明 |
|-----|------|--------|---------|------|
| `baud_rate` | int | 115200 | 需重连 | 波特率 |
| `data_bits` | int | 8 | 需重连 | 数据位（5-8） |
| `stop_bits` | int | 1 | 需重连 | 停止位（1 或 2） |
| `parity` | string | "None" | 需重连 | 校验（None/Odd/Even） |
| `flow_control` | string | "None" | 需重连 | 流控（None/Software/Hardware） |
| `hex_mode` | bool | false | 立即生效 | 十六进制显示模式 |
| `show_timestamp` | bool | false | 立即生效 | 终端时间戳显示 |
| `auto_scroll` | bool | true | 立即生效 | 终端自动滚动 |
| `auto_send_enabled` | bool | false | 立即生效 | 自动发送开关 |
| `auto_send_interval_ms` | int | 1000 | 立即生效 | 自动发送间隔（毫秒） |
| `keep_input` | bool | false | 立即生效 | 保持输入框内容 |
| `line_ending` | string | "CRLF" | 立即生效 | 行尾符（None/CR/LF/CRLF） |
| `dtr` | bool | false | **立即生效** | DTR 信号电平 |
| `rts` | bool | false | **立即生效** | RTS 信号电平 |
| `auto_reply_enabled` | bool | false | 立即生效 | 自动回复开关 |
| `auto_reply_pattern` | string | "" | 立即生效 | 自动回复匹配模式 |
| `auto_reply_response` | string | "" | 立即生效 | 自动回复内容 |
| `rx_auto_aggregate` | bool | false | 立即生效 | 接收自动聚合 |
| `rx_aggregate_ms` | int | 50 | 立即生效 | 聚合等待时间（毫秒） |

> **"需重连"** 表示 `set_config` 修改后不会立即作用于已打开的串口，需要先 `disconnect` 再 `connect`。**"立即生效"** 表示修改后立刻生效，无需重连。其中 `dtr` 和 `rts` 是通过串口控制线信号直接控制的，响应最快。

**示例 -- 读取全部配置**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_config","arguments":{}}}
```

**示例 -- 读取单个配置**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_config","arguments":{"key":"baud_rate"}}}
```

---

### 3.13 set_config -- 修改配置

修改 SerialRUN 的配置项。修改后 GUI 会同步更新。**部分配置需要重连串口才能生效**，详见 `get_config` 的 key 列表。

**参数**:

| 参数名 | 类型 | 必填 | 默认值 | 有效值 | 说明 |
|--------|------|------|--------|--------|------|
| `key` | string | 是 | - | 见 get_config 有效 key 列表 | 要修改的配置项 |
| `value` | any | 是 | - | 对应 key 的有效值 | 新值。bool 类型传 true/false，int 类型传数字，string 类型传字符串 |

**示例 -- 开启十六进制显示**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"set_config","arguments":{"key":"hex_mode","value":true}}}
```

**示例 -- 切换波特率并重连**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"set_config","arguments":{"key":"baud_rate","value":9600}}}
```

此时波特率已修改，但需要手动执行 disconnect + connect 才能生效：

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"disconnect","arguments":{}}}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"connect","arguments":{"port":"COM3"}}}
```

或者更简便的方式：直接用 `connect` 指定新波特率（connect 会自动断开再重连）：

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"connect","arguments":{"port":"COM3","baud_rate":9600}}}
```

**示例 -- 拉高 DTR 信号（立即生效）**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"set_config","arguments":{"key":"dtr","value":true}}}
```

---

### 3.14 get_access_log -- 查看访问日志

查看 MCP 服务器的访问日志，包括客户端 IP、调用的工具名、时间戳等。用于审计和调试。

**参数**:

| 参数名 | 类型 | 必填 | 默认值 | 有效值 | 说明 |
|--------|------|------|--------|--------|------|
| `limit` | integer | 否 | 50 | 1-500 | 返回的日志条目上限 |

**示例**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_access_log","arguments":{"limit":10}}}
```

**成功响应**:

```json
{
  "content": [{
    "type": "text",
    "text": "Active clients: 1\n\nAccess Log (last 3):\n[\n  {\n    \"time\": \"2026-06-04 10:00:00.123\",\n    \"ip\": \"127.0.0.1\",\n    \"action\": \"CALL\",\n    \"detail\": \"connect({\"port\":\"COM3\"})\"\n  }\n]"
  }]
}
```

---

### 3.15 get_device_info -- 查看设备信息

获取当前设备标识信息，包括连接状态、MCP 服务器版本、协议信息等。不需要已连接。

**参数**: 无

**示例**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_device_info","arguments":{}}}
```

**返回内容包含**:

- 设备名称: `SerialRUN`
- 连接状态: `Connected` / `Disconnected`
- 活跃客户端数
- 访问日志条目总数
- MCP 服务器版本: `v0.2.0`
- 协议: `JSON-RPC over TCP`

### 3.16 clear_buffers -- 清空缓冲区

清空串口的 TX 和 RX 缓冲区。在开始新的测量序列前使用，清除残留数据。

**参数**: 无

**示例**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"clear_buffers","arguments":{}}}
```

### 3.17 set_dtr -- 设置 DTR 信号

直接设置 DTR (Data Terminal Ready) 硬件信号，**立即生效，无需重连**。常见用途：通过 DTR 信号复位 Arduino/ESP32。

**参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| value | boolean | 是 | true=高电平, false=低电平 |

**示例**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"set_dtr","arguments":{"value":true}}}
```

### 3.18 set_rts -- 设置 RTS 信号

直接设置 RTS (Request To Send) 硬件信号，**立即生效，无需重连**。

**参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| value | boolean | 是 | true=高电平, false=低电平 |

**示例**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"set_rts","arguments":{"value":false}}}
```

### 3.19 get_config_keys -- 列出所有配置键

列出所有可用的配置键、类型、有效值和默认值。用于发现 `set_config` 可以修改哪些参数。

**参数**: 无

**返回内容包含**:

- `serial_params_need_reconnect`: 修改后需要重连才生效的参数（baud_rate, data_bits, stop_bits, parity, flow_control）
- `immediate_effect`: 修改后立即生效的参数（dtr, rts, hex_mode, show_timestamp 等）

---

## 4. 事件推送

SerialRUN MCP 服务器支持向客户端实时推送串口事件。当客户端连接时，服务器会自动订阅串口事件，无需额外操作。

推送消息是 JSON-RPC notification 格式（没有 `id` 字段），直接混在普通响应的同一个 TCP 连接中。客户端在读取响应时需要区分：有 `id` 的是响应，没有 `id` 的是事件推送。

### 4.1 notifications/serial_data -- 串口数据推送

串口收到数据时实时推送。数据以 Base64 编码传输。

**推送格式**:

```json
{
  "jsonrpc": "2.0",
  "method": "notifications/serial_data",
  "params": {
    "data": "SEVMTE8=",,
    "length": 5,
    "timestamp": "2026-06-04 10:30:00.123"
  }
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `data` | string | Base64 编码的串口原始数据 |
| `length` | integer | 原始数据的字节长度 |
| `timestamp` | string | 接收时间，格式 `YYYY-MM-DD HH:MM:SS.mmm` |

**示例 -- Python 解码**:

```python
import base64
raw = base64.b64decode(notification["params"]["data"])
text = raw.decode("utf-8", errors="replace")
```

### 4.2 notifications/serial_event -- 串口状态事件

串口状态变化时推送，包括打开、关闭、错误。

**推送格式**:

```json
{
  "jsonrpc": "2.0",
  "method": "notifications/serial_event",
  "params": {
    "event": "opened",
    "success": true,
    "message": "Connected to COM3",
    "client_ip": "127.0.0.1"
  }
}
```

**事件类型**:

| event | 说明 | 附带字段 |
|-------|------|---------|
| `opened` | 串口已打开 | `success` (bool), `message` (string) |
| `closed` | 串口已关闭 | `client_ip` |
| `error` | 串口错误 | `message` (string), `client_ip` |

---

## 5. 错误码

| 错误码 | 含义 | 常见原因 |
|--------|------|---------|
| -1 | 通用错误 | 未连接、GUI 不可用、超时、设备无响应 |
| -32601 | 未知方法 | 调用了不存在的工具名或方法名 |
| -32602 | 参数错误 | 必填参数缺失、参数值超出有效范围 |
| -32700 | 解析错误 | JSON 格式不正确 |

---

## 6. 注意事项

### 6.1 配置修改与重连

通过 `set_config` 修改波特率、数据位、停止位、校验、流控这些串口底层参数后，**必须先 `disconnect` 再 `connect` 才能生效**。这些参数写入了配置但不会自动应用到已打开的串口。

最快的方式是直接调用 `connect` 重新连接（connect 会自动断开旧连接）。

`dtr` 和 `rts` 是例外：这两个控制线信号修改后立即生效，不需要重连。

### 6.2 connect 的自动重连

当已经通过 MCP 连接了某个串口时，再次调用 `connect` 指定另一个端口（或相同端口不同参数），会**自动断开当前连接后重新连接**。无需手动先调用 `disconnect`。

### 6.3 Modbus 超时

`modbus_read`、`modbus_write`、`plc_read`、`plc_write` 的底层通信超时**硬编码为 200ms**。如果设备响应较慢，可能返回 "No response" 错误。这是 Modbus RTU 的标准响应时间要求，不建议修改。

### 6.4 send_command 自动追加行尾

`send_command` 会自动在命令末尾追加 `\r\n`（如果命令本身没有以 `\r\n`、`\n` 或 `\r` 结尾）。不需要手动添加行尾符。

### 6.5 端口独占

Windows 系统下串口是独占资源，同一时间只能被一个程序打开。如果 SerialRUN GUI 已经打开了某个串口，MCP 的 `connect` 会通过 GUI 转发操作，可以正常使用。但外部程序无法同时打开同一个串口。

### 6.6 客户端并发

MCP 服务器最多支持 **10 个** 同时连接的客户端。串口操作在同一时间只能被一个客户端执行，其他客户端的请求会排队等待。

### 6.7 多客户端读取冲突

多个客户端同时调用 `read` 时，数据会被第一个读取到的客户端消费。建议由一个客户端负责读取，其他客户端通过 `notifications/serial_data` 事件推送获取数据。

---

## 7. 客户端示例

### 7.1 Python -- 持久连接模式

```python
import socket
import json
import threading

class SerialRUNClient:
    def __init__(self, host="127.0.0.1", port=9527):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.settimeout(10)
        self.sock.connect((host, port))
        self._id = 0
        self._lock = threading.Lock()
        self._pending = {}  # id -> (event, response)
        self._buffer = b""
        # 启动后台接收线程
        self._running = True
        self._recv_thread = threading.Thread(target=self._recv_loop, daemon=True)
        self._recv_thread.start()
        # 初始化
        self._call("initialize", {})

    def _call(self, method, params=None):
        with self._lock:
            self._id += 1
            req_id = self._id
        msg = {"jsonrpc": "2.0", "id": req_id, "method": method}
        if params:
            msg["params"] = params
        event = threading.Event()
        self._pending[req_id] = (event, None)
        self.sock.sendall((json.dumps(msg) + "\n").encode())
        event.wait(timeout=10)
        resp = self._pending.pop(req_id, (None, None))[1]
        return resp

    def _recv_loop(self):
        while self._running:
            try:
                chunk = self.sock.recv(4096)
                if not chunk:
                    break
                self._buffer += chunk
                while b"\n" in self._buffer:
                    line, self._buffer = self._buffer.split(b"\n", 1)
                    if not line.strip():
                        continue
                    msg = json.loads(line.decode())
                    if "id" in msg and msg["id"] is not None:
                        # 普通响应
                        self._pending[msg["id"]] = (None, msg)
                        if msg["id"] in self._pending:
                            # 唤醒等待线程
                            pass  # 简化示例，实际用 Event
                    elif "method" in msg:
                        # 事件推送
                        self._on_notification(msg)
            except Exception:
                break

    def _on_notification(self, msg):
        """处理事件推送，子类可覆写"""
        method = msg.get("method", "")
        params = msg.get("params", {})
        if method == "notifications/serial_data":
            import base64
            data = base64.b64decode(params["data"])
            print(f"[RX {params['timestamp']}] {data}")
        elif method == "notifications/serial_event":
            print(f"[EVENT] {params['event']}: {params.get('message', '')}")

    def call_tool(self, name, arguments=None):
        params = {"name": name}
        if arguments:
            params["arguments"] = arguments
        return self._call("tools/call", params)

    def close(self):
        self._running = False
        self.sock.close()


# 使用示例
client = SerialRUNClient()

# 列出端口
print(client.call_tool("list_ports"))

# 连接串口
print(client.call_tool("connect", {"port": "COM3", "baud_rate": 115200}))

# 发送 AT 命令
print(client.call_tool("send_command", {"command": "AT", "timeout_ms": 1000}))

# Modbus 读取
print(client.call_tool("modbus_read", {"slave_id": 1, "address": 0, "quantity": 5}))

# 断开
print(client.call_tool("disconnect"))
client.close()
```

### 7.2 JavaScript/Node.js -- 持久连接模式

```javascript
const net = require("net");

class SerialRUNClient {
  constructor(host = "127.0.0.1", port = 9527) {
    this.host = host;
    this.port = port;
    this.id = 0;
    this.pending = new Map(); // id -> { resolve, reject }
    this.buffer = "";
    this.socket = null;
    this.onNotification = null; // 回调函数
  }

  async connect() {
    return new Promise((resolve, reject) => {
      this.socket = net.createConnection(
        { host: this.host, port: this.port },
        () => {
          this.socket.on("data", (chunk) => this._onData(chunk));
          this.socket.on("error", reject);
          this._initialize().then(resolve).catch(reject);
        }
      );
    });
  }

  _onData(chunk) {
    this.buffer += chunk.toString();
    while (this.buffer.includes("\n")) {
      const idx = this.buffer.indexOf("\n");
      const line = this.buffer.slice(0, idx).trim();
      this.buffer = this.buffer.slice(idx + 1);
      if (!line) continue;

      const msg = JSON.parse(line);
      if (msg.id != null && this.pending.has(msg.id)) {
        // 普通响应
        this.pending.get(msg.id).resolve(msg);
        this.pending.delete(msg.id);
      } else if (msg.method) {
        // 事件推送
        if (this.onNotification) this.onNotification(msg);
      }
    }
  }

  _initialize() {
    return this.call("initialize", {});
  }

  call(method, params) {
    return new Promise((resolve, reject) => {
      const id = ++this.id;
      this.pending.set(id, { resolve, reject });
      const msg = { jsonrpc: "2.0", id, method };
      if (params) msg.params = params;
      this.socket.write(JSON.stringify(msg) + "\n");
      // 超时保护
      setTimeout(() => {
        if (this.pending.has(id)) {
          this.pending.delete(id);
          reject(new Error("Timeout"));
        }
      }, 10000);
    });
  }

  tool(name, args = {}) {
    return this.call("tools/call", { name, arguments: args });
  }

  close() {
    this.socket.end();
  }
}

// 使用示例
(async () => {
  const client = new SerialRUNClient();
  await client.connect();

  // 处理事件推送
  client.onNotification = (msg) => {
    if (msg.method === "notifications/serial_data") {
      const data = Buffer.from(msg.params.data, "base64").toString();
      console.log(`[RX] ${data}`);
    }
  };

  // 列出端口
  console.log(await client.tool("list_ports"));

  // 连接
  console.log(await client.tool("connect", { port: "COM3", baud_rate: 115200 }));

  // 发送 AT 命令
  console.log(await client.tool("send_command", { command: "AT" }));

  // Modbus 读取
  console.log(await client.tool("modbus_read", { slave_id: 1, address: 0, quantity: 5 }));

  // 断开
  console.log(await client.tool("disconnect"));
  client.close();
})();
```

### 7.3 bash/netcat -- 快速测试

```bash
# 连接 MCP 服务器
nc 127.0.0.1 9527

# 初始化
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}

# 列出工具
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}

# 连接串口
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"connect","arguments":{"port":"COM3","baud_rate":115200}}}

# 发送数据
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"send","arguments":{"data":"Hello\r\n"}}}

# 读取数据
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"read","arguments":{"timeout_ms":2000,"format":"text"}}}

# 断开
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"disconnect","arguments":{}}}
```
