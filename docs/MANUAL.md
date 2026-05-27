# SerialTap User Manual / SerialTap 用户手册

English | [中文](#中文用户手册)

---

## Table of Contents

- [Introduction](#introduction)
- [Installation](#installation)
- [CLI Reference](#cli-reference)
- [GUI Guide](#gui-guide)
- [Troubleshooting](#troubleshooting)

## Introduction

SerialTap is a cross-platform serial port assistant designed for embedded developers. It provides both CLI and GUI interfaces for serial communication, protocol analysis, and automation.

## Installation

### From Source

```bash
git clone https://github.com/yourusername/SerialTap.git
cd SerialTap
cargo build --release
```

Binaries will be in `target/release/`.

## CLI Reference

### `serialtap list`

List all available serial ports.

```bash
serialtap list                # Text output
serialtap list --format json  # JSON output
```

### `serialtap connect <port> [options]`

Connect to a serial port in interactive mode.

| Option | Default | Description |
|--------|---------|-------------|
| `-b, --baud` | 115200 | Baud rate |
| `-d, --data-bits` | 8 | Data bits (5/6/7/8) |
| `-s, --stop-bits` | 1 | Stop bits (1/2) |
| `-p, --parity` | none | Parity (none/odd/even) |
| `-f, --flow` | none | Flow control (none/software/hardware) |

### `serialtap send <port> <data> [options]`

```bash
serialtap send COM1 "Hello\r\n"              # Send text
serialtap send COM1 "48 65 6C 6C 6F" --hex   # Send HEX
```

### `serialtap monitor <port> [options]`

```bash
serialtap monitor COM1 -t                  # Timestamps
serialtap monitor COM1 -x                  # HEX mode
serialtap monitor COM1 -t -l output.log    # Log to file
```

### `serialtap record <port> [options]`

```bash
serialtap record COM1 -o script.txt
```

### `serialtap replay <port> <script>`

```bash
serialtap replay COM1 script.txt
```

### `serialtap agent [port] <action>`

JSON output mode for automation. See [CLAUDE.md](../CLAUDE.md) for details.

## GUI Guide

```bash
serialtap-gui
```

### Interface Overview

- **Top bar**: Port selector, baud rate, Connect/Disconnect, Chart/Log/Language/Theme/Help buttons
- **Left panel**: Serial config (collapsible), Display options, Auto reply, Recording
- **Center**: Terminal display with TX/RX/SYS indicators
- **Bottom**: Status bar with connection state and byte counts

### Language & Theme

- Click **中** / **EN** to switch language
- Click **☀** / **☾** to switch theme
- Click **?** to open the help guide

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Port not found | Click Refresh, check device connection |
| Permission denied (Linux) | `sudo usermod -a -G dialout $USER` |
| Garbled text | Check baud rate matches device setting |
| No data | Verify cable, check flow control settings |

---

## 中文用户手册

### 目录

- [简介](#简介)
- [安装](#安装)
- [CLI 参考](#cli-参考)
- [GUI 指南](#gui-指南)
- [故障排除](#故障排除)

### 简介

SerialTap 是面向嵌入式开发者的跨平台串口助手，提供 CLI 和 GUI 两种界面，支持串口通信、协议分析和自动化。

### 安装

```bash
git clone https://github.com/yourusername/SerialTap.git
cd SerialTap
cargo build --release
```

生成的二进制文件在 `target/release/` 目录下。

### CLI 参考

#### `serialtap list`

列出所有可用串口。

```bash
serialtap list                # 文本输出
serialtap list --format json  # JSON 输出
```

#### `serialtap connect <端口> [选项]`

连接串口进入交互模式。

| 选项 | 默认值 | 说明 |
|------|--------|------|
| `-b, --baud` | 115200 | 波特率 |
| `-d, --data-bits` | 8 | 数据位 (5/6/7/8) |
| `-s, --stop-bits` | 1 | 停止位 (1/2) |
| `-p, --parity` | none | 校验位 (none/odd/even) |
| `-f, --flow` | none | 流控 (none/software/hardware) |

#### `serialtap send <端口> <数据> [选项]`

```bash
serialtap send COM1 "Hello\r\n"              # 发送文本
serialtap send COM1 "48 65 6C 6C 6F" --hex   # 发送十六进制
```

#### `serialtap monitor <端口> [选项]`

```bash
serialtap monitor COM1 -t                  # 带时间戳
serialtap monitor COM1 -x                  # 十六进制模式
serialtap monitor COM1 -t -l output.log    # 记录到文件
```

#### `serialtap record <端口> [选项]`

```bash
serialtap record COM1 -o script.txt
```

#### `serialtap replay <端口> <脚本>`

```bash
serialtap replay COM1 script.txt
```

#### `serialtap agent [端口] <动作>`

JSON 输出模式，用于自动化。详见 [CLAUDE.md](../CLAUDE.md)。

### GUI 指南

```bash
serialtap-gui
```

#### 界面说明

- **顶部栏**: 端口选择、波特率、连接/断开、图表/日志/语言/主题/帮助按钮
- **左侧面板**: 串口配置（可折叠）、显示选项、自动回复、录制
- **中央区域**: 终端显示，带 TX/RX/SYS 指示
- **底部**: 状态栏，显示连接状态和字节计数

#### 语言和主题

- 点击 **中** / **EN** 切换语言
- 点击 **☀** / **☾** 切换主题
- 点击 **?** 打开使用指南

### 故障排除

| 问题 | 解决方案 |
|------|----------|
| 端口未找到 | 点击刷新，检查设备连接 |
| 权限不足 (Linux) | `sudo usermod -a -G dialout $USER` |
| 文本乱码 | 确认波特率与设备设置一致 |
| 无数据 | 检查线缆，确认流控设置 |
