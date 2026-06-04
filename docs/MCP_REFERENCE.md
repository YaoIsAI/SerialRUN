# SerialRUN MCP 参数速查表

> 版本: 0.2.0 | 协议: JSON-RPC 2.0 over TCP

---

## 1. set_config 完整 Key-Value 参考

通过 `set_config` 工具可动态修改 GUI 配置。所有配置修改**立即生效**（无需重启），部分参数（如 `dtr`、`rts`）会立即发送到硬件。

| # | key 名称 | 值类型 | 有效值范围 | 默认值 | 立即生效 | 说明 |
|---|---------|--------|-----------|--------|---------|------|
| 1 | `baud_rate` | integer | 300 - 4000000 | `115200` | 否（需重连） | 波特率。修改后需重新连接端口才生效 |
| 2 | `data_bits` | integer | 5, 6, 7, 8 | `8` | 否（需重连） | 数据位。修改后需重新连接端口才生效 |
| 3 | `stop_bits` | integer | 1, 2 | `1` | 否（需重连） | 停止位。修改后需重新连接端口才生效 |
| 4 | `parity` | string | `"None"`, `"Odd"`, `"Even"` | `"None"` | 否（需重连） | 校验位。修改后需重新连接端口才生效 |
| 5 | `flow_control` | string | `"None"`, `"Software"`, `"Hardware"` | `"None"` | 否（需重连） | 流控。修改后需重新连接端口才生效 |
| 6 | `hex_mode` | boolean | true / false | `false` | 是 | 终端十六进制显示模式 |
| 7 | `show_timestamp` | boolean | true / false | `true` | 是 | 终端是否显示时间戳 |
| 8 | `auto_scroll` | boolean | true / false | `true` | 是 | 终端自动滚动 |
| 9 | `auto_send_enabled` | boolean | true / false | `false` | 是 | 启用自动发送 |
| 10 | `auto_send_interval_ms` | integer | 100 - 3600000 | `1000` | 是 | 自动发送间隔（毫秒） |
| 11 | `keep_input` | boolean | true / false | `false` | 是 | 发送后保留输入框内容 |
| 12 | `line_ending` | string | `"None"`, `"CR"`, `"LF"`, `"CRLF"` | `"None"` | 是 | 行尾符追加模式 |
| 13 | `dtr` | boolean | true / false | `true` | **硬件即时** | DTR 信号。立即写入硬件，无需重连 |
| 14 | `rts` | boolean | true / false | `false` | **硬件即时** | RTS 信号。立即写入硬件，无需重连 |
| 15 | `auto_reply_enabled` | boolean | true / false | `false` | 是 | 启用自动回复 |
| 16 | `auto_reply_pattern` | string | 任意字符串 | `""` | 是 | 自动回复匹配模式 |
| 17 | `auto_reply_response` | string | 任意字符串 | `""` | 是 | 自动回复内容 |
| 18 | `rx_auto_aggregate` | boolean | true / false | `true` | 是 | 自动聚合接收数据（减少闪烁） |
| 19 | `rx_aggregate_ms` | integer | 10 - 10000 | `150` | 是 | 接收数据聚合间隔（毫秒） |

**调用示例：**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "set_config",
    "arguments": {
      "key": "baud_rate",
      "value": 9600
    }
  }
}
```

**注意：** `baud_rate`、`data_bits`、`stop_bits`、`parity`、`flow_control` 属于串口连接参数，修改后需要重新调用 `connect` 或 `disconnect` + `connect` 才能生效。

---

## 2. 常用波特率列表

| 波特率 | 典型使用场景 |
|--------|------------|
| 9600 | 低速设备：GPS 模块、工业传感器、老式 MCU（如 8051）、Modbus RTU 从站 |
| 19200 | 中低速工业设备、PLC 通信（某些旧型号）、条码扫描器 |
| 38400 | 中速设备、某些 PLC 通信（如三菱 FX 系列） |
| 57600 | 中速设备、蓝牙 SPP 串口、某些嵌入式模块 |
| **115200**（默认） | 最常用：Arduino、ESP8266/ESP32、STM32、树莓派 GPIO、大多数嵌入式开发 |
| 230400 | 高速数据传输：ESP32 高速串口、固件烧录 |
| 460800 | 高速数据传输：固件刷写、大文件传输 |
| 921600 | 最高速：ESP32 OTA、高速数据采集、固件烧录 |

**说明：**
- 波特率必须与设备端设置完全一致，否则数据乱码
- Modbus RTU 常用 9600 或 19200
- 嵌入式开发调试常用 115200
- 更高波特率（如 1500000、2000000）部分硬件也支持，通过 `baud_rate` 参数可自由设置

---

## 3. Modbus 功能码对照表

| 功能码 | 十六进制 | 名称 | MCP 工具 | 说明 |
|--------|---------|------|---------|------|
| FC01 | 0x01 | Read Coils | - | 读线圈状态（位操作） |
| FC02 | 0x02 | Read Discrete Inputs | - | 读离散输入（位操作） |
| FC03 | 0x03 | Read Holding Registers | `modbus_read` | 读保持寄存器（16位） |
| FC04 | 0x04 | Read Input Registers | - | 读输入寄存器（16位） |
| FC05 | 0x05 | Write Single Coil | - | 写单个线圈 |
| FC06 | 0x06 | Write Single Register | `modbus_write` | 写单个保持寄存器 |
| FC15 | 0x0F | Write Multiple Coils | - | 写多个线圈 |
| FC16 | 0x10 | Write Multiple Registers | - | 写多个保持寄存器 |

**MCP 工具参数说明：**

### modbus_read

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `slave_id` | integer | 1 | 从站地址（1-247） |
| `address` | integer | （必填） | 起始寄存器地址 |
| `quantity` | integer | 1 | 读取寄存器数量（1-125） |
| `scale` | number | 1.0 | 缩放因子：value = raw * scale + offset |
| `offset` | number | 0.0 | 偏移量（缩放后加） |
| `unit` | string | "" | 工程单位标签（如 "°C"、"rpm"） |

### modbus_write

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `slave_id` | integer | 1 | 从站地址（1-247） |
| `address` | integer | （必填） | 寄存器地址 |
| `value` | integer | （必填） | 写入值（0-65535） |

**调用示例（读取温度传感器）：**

```json
{
  "name": "modbus_read",
  "arguments": {
    "slave_id": 1,
    "address": 0,
    "quantity": 2,
    "scale": 0.1,
    "unit": "°C"
  }
}
```

**返回格式：** 当设置 `scale`/`offset`/`unit` 时，返回包含 `raw`（原始值）和 `value`（工程值）的结构化数据。

---

## 4. PLC 品牌预设寄存器表

### Siemens（西门子）- S7-1200

| 地址 | 名称 | 数据类型 | 缩放因子 | 单位 | 说明 |
|------|------|---------|---------|------|------|
| 0 | Temperature SP | Float32 | 0.1 | °C | 温度设定值 |
| 2 | Temperature PV | Float32 | 0.1 | °C | 温度过程值 |
| 4 | Pressure | Float32 | 0.01 | bar | 系统压力 |
| 8 | Speed SP | U16 | 1.0 | rpm | 电机转速设定值 |
| 9 | Speed PV | U16 | 1.0 | rpm | 电机转速实际值 |
| 10 | Motor Status | U16 | 1.0 | - | 电机状态位 |
| 11 | Alarm Code | U16 | 1.0 | - | 报警代码 |

**支持的数据类型：** Float32, U16, I16, U32, BOOL

**典型应用：** 工业温控系统、压力监测、电机控制、过程自动化

---

### Mitsubishi（三菱）- FX3U

| 地址 | 名称 | 数据类型 | 缩放因子 | 单位 | 说明 |
|------|------|---------|---------|------|------|
| 0 | D0 - General | I16 | 1.0 | - | 通用数据寄存器 D0 |
| 1 | D1 - General | I16 | 1.0 | - | 通用数据寄存器 D1 |
| 4 | D4 - Counter | U16 | 1.0 | - | 计数器值 |
| 5 | D5 - Timer | U16 | 0.01 | s | 定时器值 |
| 10 | Speed | U16 | 1.0 | rpm | 电机转速 |

**支持的数据类型：** I16, U16, Bool, U32, Float32

**典型应用：** 运动控制、CNC 加工、包装机械、纺织设备

---

### Delta（台达）- DVP

| 地址 | 名称 | 数据类型 | 缩放因子 | 单位 | 说明 |
|------|------|---------|---------|------|------|
| 0 | D0 | I16 | 1.0 | - | 数据寄存器 D0 |
| 4 | Temperature | U16 | 0.1 | °C | 温度读数 |
| 5 | Pressure | U16 | 0.01 | MPa | 压力读数 |

**支持的数据类型：** I16, U16, Bool, U32, Float32

**典型应用：** 小型自动化设备、HVAC 控制、水处理、包装机

---

### Omron（欧姆龙）- CP1H

| 地址 | 名称 | 数据类型 | 缩放因子 | 单位 | 说明 |
|------|------|---------|---------|------|------|
| 0 | D0 | U16 | 1.0 | - | DM 区域 D0 |
| 4 | Temperature | U16 | 0.1 | °C | 温度输入 |
| 5 | Setpoint | U16 | 0.1 | °C | 温度设定值 |
| 6 | Output | U16 | 0.1 | % | 控制输出 |

**支持的数据类型：** U16, I16, Bool, U32, Float32

**典型应用：** 过程控制、温度调节、自动化生产线、食品机械

---

### PLC 工具调用示例

**plc_read - 读取所有寄存器：**

```json
{
  "name": "plc_read",
  "arguments": {
    "brand": "Siemens",
    "slave_id": 1
  }
}
```

**plc_write - 写入寄存器：**

```json
{
  "name": "plc_write",
  "arguments": {
    "brand": "Siemens",
    "slave_id": 1,
    "address": 8,
    "value": 1500
  }
}
```

**支持的品牌名称：** `Siemens`（西门子）、`Mitsubishi`（三菱）、`Delta`（台达）、`Omron`（欧姆龙）

---

## 5. 错误码参考

| 错误码 | JSON-RPC 含义 | 常见原因 | 解决方案 |
|--------|-------------|---------|---------|
| -1 | 服务器内部错误 | 连接失败、超时、GUI 不可用 | 检查串口连接状态；确认 GUI 已启动；检查串口参数 |
| -32601 | 方法不存在 | 工具名拼写错误 | 参考 tools/list 返回的工具名列表 |
| -32602 | 参数无效 | 参数类型错误、缺少必填参数、值超出范围 | 检查参数类型和取值范围 |
| -32700 | 解析错误 | JSON 格式不正确 | 检查请求 JSON 格式 |

**具体错误信息示例：**

| 错误信息 | 说明 |
|---------|------|
| `"Port name is required"` | connect 工具缺少 port 参数 |
| `"Data is required"` | send 工具缺少 data 参数 |
| `"Command is required"` | send_command 工具缺少 command 参数 |
| `"address is required"` | modbus_read/modbus_write 缺少 address 参数 |
| `"value is required"` | modbus_write 缺少 value 参数 |
| `"quantity must be 1-125"` | modbus_read quantity 超出范围 |
| `"value must be 0-65535"` | modbus_write value 超出范围 |
| `"key is required"` | set_config 缺少 key 参数 |
| `"Unknown key: xxx"` | set_config 使用了不存在的 key |
| `"Unknown brand: xxx"` | plc_read/plc_write 使用了不支持的品牌 |
| `"Not connected"` | 操作前未建立串口连接 |
| `"GUI not available. Start SerialRUN first."` | MCP 服务器未启动或未连接 GUI |
| `"Timeout waiting for GUI"` | GUI 响应超时（通常 5 秒） |
| `"No response"` | Modbus 从站未响应 |

---

## 6. 事件推送格式

MCP 服务器通过 JSON-RPC notification 向客户端推送串口事件。客户端无需主动轮询，连接后即可接收。

### notifications/serial_data - 串口数据推送

```json
{
  "jsonrpc": "2.0",
  "method": "notifications/serial_data",
  "params": {
    "data": "SGVsbG8gV29ybGQ=",
    "length": 11,
    "timestamp": "2026-06-04 14:30:25.123"
  }
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `data` | string | Base64 编码的串口数据 |
| `length` | integer | 原始数据字节数 |
| `timestamp` | string | 接收时间戳（毫秒精度） |

### notifications/serial_event - 串口事件推送

```json
{
  "jsonrpc": "2.0",
  "method": "notifications/serial_event",
  "params": {
    "event": "opened",
    "success": true,
    "message": "Connected to COM1 at 115200 baud",
    "client_ip": "127.0.0.1"
  }
}
```

**事件类型：**

| event 值 | 说明 | 附加字段 |
|----------|------|---------|
| `opened` | 串口已打开 | `success` (bool), `message` (string) |
| `closed` | 串口已关闭 | - |
| `error` | 串口错误 | `message` (string) |

**通用字段：**

| 字段 | 类型 | 说明 |
|------|------|------|
| `client_ip` | string | 触发事件的客户端 IP |

**使用建议：** 客户端在收到 `serial_data` 事件后，解码 Base64 字段即可获得原始串口数据。适合实时数据采集和监控场景。

---

## 7. MCP 工具调用频率限制

### 通用规则

- **无硬性频率限制**，但建议每次调用间隔 **> 50ms**（20 次/秒）
- MCP 服务器最多支持 **10 个并发客户端**（`MAX_MCP_CLIENTS = 10`）
- 超出限制时新连接将被拒绝

### 特定场景建议

| 场景 | 建议间隔 | 原因 |
|------|---------|------|
| 普通串口读写 | > 50ms | 避免缓冲区溢出 |
| Modbus RTU 通信 | > 200ms | 从站响应需要时间（典型 100-500ms） |
| PLC 轮询（plc_read） | > 500ms | 避免覆盖 GUI 的轮询 |
| 高速数据采集 | > 10ms | 仅适用于专用场景，需评估设备能力 |

### 超时与响应时间

| 操作 | 内部超时 | 说明 |
|------|---------|------|
| connect | 5 秒 | 连接操作的总超时 |
| disconnect | 5 秒 | 断开操作的总超时 |
| send | 5 秒 | 发送操作超时 |
| read | 6 秒 | 读取操作超时（默认等待 1 秒，额外 buffer） |
| send_command | 6 秒 | 发送+读取总超时 |
| modbus_read/write | 5 秒 | Modbus 请求+响应总超时 |
| set_config / get_config | 2 秒 | 配置操作超时 |

### 防护机制

- **连接数限制：** 最多 10 个并发 MCP 客户端
- **日志上限：** 访问日志最多保留 500 条，超出后自动清理旧条目
- **Modbus quantity 限制：** 每次最多读取 125 个寄存器
- **Modbus value 限制：** 写入值范围 0-65535（u16）

---

## 附录：完整 MCP 工具列表

| 工具名 | 描述 | 必填参数 |
|--------|------|---------|
| `list_ports` | 列出可用串口 | 无 |
| `connect` | 连接串口 | `port` |
| `disconnect` | 断开串口 | 无 |
| `send` | 发送数据 | `data` |
| `read` | 读取数据 | 无 |
| `send_command` | 发送命令并等待响应 | `command` |
| `modbus_read` | 读取 Modbus 保持寄存器 | `address` |
| `modbus_write` | 写入 Modbus 保持寄存器 | `address`, `value` |
| `plc_read` | 读取 PLC 预设寄存器 | 无（brand 可选） |
| `plc_write` | 写入 PLC 寄存器 | `address`, `value` |
| `get_access_log` | 获取访问日志 | 无 |
| `get_device_info` | 获取设备信息 | 无 |
| `status` | 获取连接状态和计数器 | 无 |
| `get_config` | 获取配置 | 无（key 可选） |
| `set_config` | 修改配置 | `key`, `value` |
