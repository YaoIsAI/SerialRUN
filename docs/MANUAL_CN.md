# SerialRUN 用户手册

[English](MANUAL.md)

---

## 目录

- [简介](#简介)
- [安装](#安装)
- [CLI 参考](#cli-参考)
- [GUI 指南](#gui-指南)
- [故障排除](#故障排除)

## 简介

SerialRUN 是面向嵌入式开发者的跨平台串口助手，提供 CLI 和 GUI 两种界面，支持串口通信、协议分析和自动化。

## 安装

```bash
git clone https://github.com/yourusername/SerialRUN.git
cd SerialRUN
cargo build --release
```

生成的二进制文件在 `target/release/` 目录下。

## CLI 参考

### `serialrun list`

列出所有可用串口。

```bash
serialrun list                # 文本输出
serialrun list --format json  # JSON 输出
```

### `serialrun connect <端口> [选项]`

连接串口进入交互模式。

| 选项 | 默认值 | 说明 |
|------|--------|------|
| `-b, --baud` | 115200 | 波特率 |
| `-d, --data-bits` | 8 | 数据位 (5/6/7/8) |
| `-s, --stop-bits` | 1 | 停止位 (1/2) |
| `-p, --parity` | none | 校验位 (none/odd/even) |
| `-f, --flow` | none | 流控 (none/software/hardware) |

### `serialrun send <端口> <数据> [选项]`

```bash
serialrun send COM1 "Hello\r\n"              # 发送文本
serialrun send COM1 "48 65 6C 6C 6F" --hex   # 发送十六进制
```

### `serialrun monitor <端口> [选项]`

```bash
serialrun monitor COM1 -t                  # 带时间戳
serialrun monitor COM1 -x                  # 十六进制模式
serialrun monitor COM1 -t -l output.log    # 记录到文件
```

### `serialrun record <端口> [选项]`

```bash
serialrun record COM1 -o script.txt
```

### `serialrun replay <端口> <脚本>`

```bash
serialrun replay COM1 script.txt
```

### `serialrun agent [端口] <动作>`

JSON 输出模式，用于自动化。详见 [CLAUDE_CN.md](../CLAUDE_CN.md)。

## GUI 指南

```bash
serialrun-gui
```

### 界面说明

- **顶部栏**: 工具按钮（日志、图表、PLC、Modbus、桥接、模拟器、文件传输、帧生成器、数据记录、CAN、I2C/SPI、示波器、烧录器、寄存器编辑、插件）、主题/语言/帮助
- **左侧面板**: 端口选择、波特率、自动检测、连接/断开
- **中央区域**: 终端显示，带 TX/RX/SYS 指示，工具栏自动换行
- **底部**: 状态栏，显示连接状态和字节计数
- **浮动面板**: 所有功能面板为独立 OS 窗口，可自由拖拽和缩放。子窗口始终在主窗口前面。

### GUI 面板

| 面板 | 说明 |
|------|------|
| 终端 | 串口收发，支持 HEX 模式、时间戳、CRC 校验，工具栏自动换行 |
| Modbus | RTU 监听，解析功能码，显示寄存器值，可配置响应超时（50-5000ms），TX/RX 同步显示在终端 |
| PLC 控制 | Modbus 寄存器轮询，内置品牌预设（西门子、三菱、台达等），TX 同步显示在终端 |
| TCP/RTU 桥接 | 将 Modbus TCP 客户端桥接到串口 RTU 设备 |
| HMI 模拟器 | 虚拟 Modbus 从站，可配置寄存器和线圈 |
| CAN 总线 | SLCAN 帧捕获，ID 过滤，按 ID 统计 |
| I2C/SPI | 寄存器读写调试工具，支持地址和数据宽度配置，TX 同步显示在终端 |
| 示波器 | 实时波形显示，支持触发和光标测量 |
| 文件传输 | XMODEM / YMODEM / ZMODEM 协议传输 |
| 帧生成器 | 可视化 Modbus 帧构造，实时十六进制预览 |
| 烧录器 | STM32 ISP 和 ESP32 串口烧录 |
| 数据记录器 | 持续 CSV 记录，带时间戳 |
| 寄存器编辑器 | CSV/JSON 导入导出，报警阈值监控 |
| 图表 | 多系列实时数据可视化 |
| 插件管理 | 动态插件发现和加载 |
| 日志查看 | 应用日志，支持过滤和导出 |

### 语言和主题

- 点击 **中** / **EN** 切换语言
- 点击 **Dark** / **Light** 切换主题
- 点击 **?** 打开使用指南

## 故障排除

| 问题 | 解决方案 |
|------|----------|
| 端口未找到 | 点击刷新，检查设备连接 |
| 权限不足 (Linux) | `sudo usermod -a -G dialout $USER` |
| 文本乱码 | 确认波特率与设备设置一致 |
| 无数据 | 检查线缆，确认流控设置 |
