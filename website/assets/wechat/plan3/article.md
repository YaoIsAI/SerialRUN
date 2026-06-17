# 深度使用 SerialRUN MCP：让 AI 助手成为你的串口调试搭档

> 当嵌入式开发者遇到 AI，串口调试从此进入对话时代。

---

## 一、为什么你需要 SerialRUN + MCP？

作为嵌入式开发者，你一定经历过这些场景：

- 深夜调试 AT 指令，反复输入 `AT+CSQ` 检查信号，手动记录每次返回值
- Modbus 寄存器监控，需要反复切换 HEX/TEXT 模式，逐个读取地址
- PLC 调试时，面对一堆寄存器地址，每次都要翻文档查定义

**这些重复性工作，AI 可以帮你完成。**

SerialRUN 是一款开源免费的专业串口调试助手，内置 MCP（Model Context Protocol）服务器。MCP 是 AI 助手与外部工具通信的标准协议——Claude、ChatGPT 等 AI 助手可以通过 MCP 直接控制你的串口设备。

**简单说：你用自然语言告诉 AI 要做什么，AI 调用 SerialRUN 的工具去执行。**

---

## 二、3 分钟上手

### Step 1：启用 MCP 服务器

打开 SerialRUN → 左侧面板 → MCP 服务器 → 勾选「启用 MCP 服务器」

默认配置：地址 `127.0.0.1`，端口 `9527`。如果需要局域网访问，勾选「绑定地址 → 局域网」。

### Step 2：复制 MCP 配置

点击 SerialRUN 左侧面板底部的「复制 MCP 说明」按钮。这份文档包含：
- 19 个工具的完整说明
- 每个工具的 JSON-RPC 调用格式
- 使用示例和注意事项

### Step 3：配置 AI 助手

打开 Claude / ChatGPT / Cursor，将复制的内容粘贴到对话中。AI 会自动识别这些工具。

### Step 4：开始对话

现在你可以直接用自然语言操作串口了：

```
你：帮我扫描串口，连接 COM3，波特率 115200
AI：[调用 list_ports] → [调用 connect] → 已连接 COM3 @ 115200
```

---

## 三、19 个工具全覆盖

SerialRUN 的 MCP 提供 19 个内置工具，覆盖串口调试的全部场景：

### 基础串口操作（7 个）

| 工具 | 功能 | 对话示例 |
|------|------|---------|
| `list_ports` | 扫描串口 | "当前有哪些串口？" |
| `connect` | 打开串口 | "连接 COM3，波特率 115200" |
| `disconnect` | 关闭串口 | "断开连接" |
| `send` | 发送数据 | "发送 Hello World" |
| `read` | 读取数据 | "读取串口数据" |
| `send_command` | 发送+等待响应 | "发送 AT 指令" |
| `clear_buffers` | 清空缓冲区 | "清空收发缓冲" |

### Modbus 调试（2 个）

| 工具 | 功能 | 对话示例 |
|------|------|---------|
| `modbus_read` | 读寄存器 | "读取从站 1 地址 0 的 10 个寄存器" |
| `modbus_write` | 写寄存器 | "写入从站 1 地址 100 值为 25" |

### PLC 控制（2 个）

| 工具 | 功能 | 对话示例 |
|------|------|---------|
| `plc_read` | 读全部寄存器 | "读取西门子 S7-1200 全部寄存器" |
| `plc_write` | 写寄存器 | "写入 PLC 地址 0 值为 25" |

### 硬件信号（2 个）

| 工具 | 功能 | 对话示例 |
|------|------|---------|
| `set_dtr` | DTR 信号 | "拉高 DTR 复位设备" |
| `set_rts` | RTS 信号 | "拉低 RTS" |

### 配置管理（6 个）

| 工具 | 功能 | 对话示例 |
|------|------|---------|
| `status` | 连接状态 | "查看当前状态" |
| `get_config` | 读设置 | "查看所有设置" |
| `set_config` | 改设置 | "关闭时间戳显示" |
| `get_config_keys` | 查配置项 | "列出所有可配置项" |
| `get_device_info` | 设备信息 | "查看设备信息" |
| `get_access_log` | 访问日志 | "查看 MCP 访问记录" |

---

## 四、实战场景深度演示

### 场景 1：AT 命令调试（最常用）

**传统方式：** 手动输入 AT → 等待 → 复制返回值 → 记录到文档 → 反复操作

**MCP 方式：**

```
你：帮我检查 ESP32 模块状态
AI：好的，我来帮你检查。
    → [connect] COM3 @ 115200
    → [send_command] "AT"
    → [send_command] "AT+CSQ"
    → [send_command] "AT+COPS?"
    
    检查结果：
    - AT 回复：OK
    - 信号强度：CSQ 25（良好）
    - 网络注册：COPS? 0,0,"CMCC"（已注册中国移动）
    - 建议：模块正常，可以进行下一步测试
```

### 场景 2：Modbus 寄存器监控

```
你：读取从站 1 的温度和压力，每 2 秒读一次
AI：好的，开始监控。
    → [modbus_read] slave=1, addr=0, qty=2
    
    第 1 次：温度 25.3°C，压力 1.2bar
    第 2 次：温度 25.4°C，压力 1.2bar
    第 3 次：温度 25.3°C，压力 1.3bar
    
    趋势分析：温度稳定在 25.3-25.4°C，压力略有波动（±0.1bar），属正常范围。
```

### 场景 3：PLC 全面诊断

```
你：帮我诊断这台西门子 PLC
AI：好的，正在读取 S7-1200 全部寄存器...
    → [connect] COM5 @ 9600
    → [plc_read] brand=Siemens
    
    诊断结果：
    - Temperature SP: 80.0°C（设定值）
    - Temperature PV: 78.5°C（实际值，偏差 1.5°C）
    - Pressure: 2.1bar（正常范围）
    - Motor Status: Running（运行中）
    - Alarm Code: 0（无报警）
    
    结论：PLC 运行正常，温度控制在设定值附近，无报警。
```

### 场景 4：固件刷写辅助

```
你：准备刷写 ESP32，帮我复位芯片
AI：好的，通过 DTR 信号复位 ESP32。
    → [set_dtr] value=false
    → [set_dtr] value=true
    → [set_dtr] value=false
    
    芯片已复位，进入下载模式。现在可以用 esptool 刷写固件。
```

---

## 五、进阶技巧

### 1. 组合操作一次完成

不需要分步告诉 AI，一句话就行：

```
你：连接 COM3，发送 AT 指令检查模块，然后每 5 秒读取信号强度，持续 1 分钟
AI：→ connect → send_command("AT") → read → 循环 modbus_read × 12 次
```

### 2. 利用配置管理优化体验

```
你：把终端时间戳关掉，然后关闭自动滚动
AI：→ set_config(key="show_timestamp", value=false)
    → set_config(key="auto_scroll", value=false)
    已关闭。
```

### 3. 利用 clear_buffers 开始干净的测试

```
你：清空缓冲区，然后发送 AT 指令
AI：→ clear_buffers → send_command("AT")
    缓冲区已清空，收到回复：OK
```

### 4. 利用 get_config_keys 发现隐藏功能

```
你：SerialRUN 还能配置什么？
AI：→ get_config_keys
    
    可配置项包括：
    - 串口参数（需重连）：baud_rate, data_bits, stop_bits, parity, flow_control
    - 显示设置（立即生效）：hex_mode, show_timestamp, auto_scroll
    - 发送设置：auto_send_enabled, auto_send_interval_ms, line_ending
    - 硬件信号：dtr, rts
    - 接收设置：rx_auto_aggregate, rx_aggregate_ms
```

---

## 六、注意事项

1. **先连接再操作**：所有串口操作都需要先用 `connect` 打开串口
2. **send_command 最常用**：对于 AT 命令等请求-响应模式，优先用 `send_command` 而不是 `send` + `read`
3. **Modbus 超时**：默认 200ms，慢设备可增大 `modbus_timeout_ms` 参数
4. **set_config 需重连**：修改串口参数（波特率等）后需要 `disconnect` + `connect` 才生效
5. **set_dtr/set_rts 立即生效**：硬件信号修改不需要重连
6. **查看访问日志**：用 `get_access_log` 可以回溯 AI 的每一步操作

---

## 七、获取更多资源

- 🌐 **官网下载**：www.serialrun.com/downloads.html
- 📖 **MCP 使用指南**：www.serialrun.com/guide.html#mcp
- 📋 **MCP API 文档**：www.serialrun.com（GitHub 仓库 docs/MCP_API.md）
- ⭐ **GitHub**：github.com/YaoIsAI/SerialRUN

**当前版本：v0.3.0**（19 个 MCP 工具，含快捷指令功能）

SerialRUN 是开源项目（BSL 1.1 许可证），欢迎 Star、Fork、提交 Issue！

---

*本文所有对话示例均为 SerialRUN MCP 的实际使用效果。*
