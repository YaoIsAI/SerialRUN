<div align="center">

# SerialRUN

**专业串口调试助手 — 面向嵌入式开发者**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-BSL%201.1-blue.svg)](https://spdx.org/licenses/BSL-1.1.html)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-blue.svg)]()

[English](README.md) | [中文](#功能特性)

</div>

---

<p align="center">
  <img src="assets/screenshot_zh.png" alt="SerialRUN 截图" width="800">
</p>

<p align="center">
  <em>Modbus RTU 调试 — 实时寄存器监控，TX/RX 终端同步显示</em>
</p>

---

## 下载

| 平台 | 链接 |
|------|------|
| Windows (x64) | [SerialRUN-v0.1.0-windows-x64.zip](http://192.168.31.85:38633/yao/serialrun/releases/tag/v0.1.0) |
| macOS | 从源码编译（见下方） |
| Linux | 从源码编译（见下方） |

或从源码编译：

```bash
git clone https://github.com/YaoIsAI/SerialRUN.git
cd SerialRUN
cargo build --release
# 二进制文件：target/release/serialrun.exe (Windows)
#             target/release/serialrun (macOS/Linux)
```

---

## 功能特性

- **跨平台** — Windows、macOS、Linux
- **CLI & GUI** — 命令行用于自动化，桌面客户端用于交互式使用
- **多窗口界面** — 所有面板作为独立 OS 窗口运行，自由拖拽和缩放，始终在最前
- **协议支持** — Modbus RTU/TCP 解析，自定义协议模式匹配
- **数据可视化** — 实时图表和统计信息
- **脚本录制** — 录制和回放串口通信会话，保留时间间隔
- **文件传输** — 支持 XMODEM / YMODEM / ZMODEM
- **CAN 总线分析** — SLCAN 协议解析、帧过滤、按 ID 统计
- **I2C/SPI 调试** — 寄存器读写，支持地址和数据宽度配置
- **串口示波器** — 实时波形显示，支持触发和光标测量
- **烧录器** — STM32 ISP 和 ESP32 串口烧录
- **寄存器编辑器** — CSV/JSON 导入导出，报警阈值监控
- **数据记录器** — 持续 CSV 记录，带时间戳
- **帧生成器** — 可视化 Modbus 帧构造，实时十六进制预览
- **PLC 控制** — Modbus 寄存器轮询，内置品牌预设（西门子、三菱、台达、欧姆龙）
- **TCP/RTU 桥接** — 将 Modbus TCP 客户端桥接到串口 RTU 设备
- **HMI 模拟器** — 虚拟 Modbus 从站模拟器（TCP/RTU）
- **插件系统** — 可扩展架构，支持动态加载插件
- **MCP 服务器** — 内置 TCP 服务器，15 个工具支持 AI 助手集成
- **访问日志** — 所有 MCP 操作自动记录客户端 IP，支持追溯
- **十六进制模式** — 以十六进制格式收发数据
- **自动回复** — 自动响应匹配的模式
- **自动换行工具栏** — 终端控件自适应窗口大小，不会被裁剪
- **双语界面** — 英文/中文语言切换，深色/浅色主题
- **数据持久化** — 配置、日志、终端历史、警告自动保存到 `~/.serialrun/`
- **全局错误系统** — 统一错误提示，状态栏显示，支持历史查看

## 快速开始

### 安装

```bash
git clone https://github.com/YaoIsAI/SerialRUN.git
cd SerialRUN
cargo build --release
```

### 命令行使用

```bash
# 列出可用串口
serialrun list

# 连接串口
serialrun connect COM1 -b 115200

# 发送数据
serialrun send COM1 "Hello\r\n"

# 带时间戳监听
serialrun monitor COM1 -t -l output.log

# 录制脚本
serialrun record COM1 -o script.txt

# 回放脚本
serialrun replay COM1 script.txt
```

### 桌面客户端使用

```bash
serialrun-gui
```

### GUI 快速开始

1. 通过 USB 连接串口设备
2. 点击「刷新」检测端口
3. 选择端口和波特率，点击「连接」
4. 在输入框输入命令，按回车发送

## 项目结构

```
SerialRUN/
├── crates/
│   ├── serialrun-core/       # 核心库（端口、协议、校验、数据记录）
│   ├── serialrun-cli/        # 命令行工具
│   ├── serialrun-gui/        # 桌面客户端 (egui/eframe)
│   ├── serialrun-mcp/        # MCP 服务器（AI 集成）
│   └── serialrun-plugin-api/ # 插件 API 定义
├── plugins/
│   └── serialrun-example-plugin/  # 插件示例 (C FFI)
├── assets/                   # 嵌入式图片和图标
├── docs/                     # 文档
├── tests/                    # 集成测试
└── benches/                  # 性能测试
```

## GUI 面板

所有面板作为独立 OS 窗口运行——自由拖拽、缩放和排列。子窗口始终在主窗口前面。

| 面板 | 说明 |
|------|------|
| 终端 | 串口收发，支持 HEX 模式、时间戳、CRC 校验，工具栏自动换行 |
| Modbus | RTU 监听，8 种功能码，可配置响应超时（50-5000ms），TX/RX 同步显示在终端 |
| PLC 控制 | 寄存器轮询，内置品牌预设（西门子、三菱、台达、欧姆龙），TX 同步显示在终端 |
| TCP/RTU 桥接 | 将 Modbus TCP 客户端桥接到串口 RTU 设备 |
| HMI 模拟器 | 虚拟 Modbus 从站，可配置寄存器和线圈 |
| CAN 总线 | SLCAN 帧捕获，ID 过滤，按 ID 统计 |
| I2C/SPI | 寄存器读写调试工具，TX 同步显示在终端 |
| 示波器 | 实时波形显示，支持触发和光标测量 |
| 文件传输 | XMODEM / YMODEM / ZMODEM 协议传输 |
| 帧生成器 | 可视化 Modbus 帧构造，实时十六进制预览 |
| 烧录器 | STM32 ISP 和 ESP32 串口烧录 |
| 数据记录器 | 持续 CSV 记录，带时间戳 |
| 寄存器编辑器 | CSV/JSON 导入导出，报警阈值监控 |
| 图表 | 多系列实时数据可视化 |
| 插件管理 | 动态插件发现和加载 |
| 日志查看 | 应用日志，支持过滤、导出和持久化 |

## 跨平台构建

| 平台 | 命令 |
|------|------|
| Windows (MSVC) | `cargo build --target x86_64-pc-windows-msvc --release` |
| macOS (Apple Silicon) | `cargo build --target aarch64-apple-darwin --release` |
| macOS (Intel) | `cargo build --target x86_64-apple-darwin --release` |
| Linux | `cargo build --target x86_64-unknown-linux-gnu --release` |

详见 [docs/BUILD_CN.md](docs/BUILD_CN.md) 了解构建详情。

## Agent 模式 (自动化)

```bash
serialrun agent list-ports                # 列出端口 (JSON)
serialrun agent COM1 send "AT+RST"        # 发送数据
serialrun agent COM1 read --timeout 1000  # 读取数据
serialrun agent COM1 run-script test.txt  # 运行脚本
```

## MCP 服务器

SerialRUN 内置 MCP 服务器，15 个工具支持 AI 助手集成。所有串口操作通过 GUI 的端口管理器路由。

### 可用工具

| 工具 | 说明 |
|------|------|
| `list_ports` | 列出所有可用串口设备 |
| `connect` | 连接串口（支持波特率、数据位、停止位、校验位、流控） |
| `disconnect` | 断开当前连接 |
| `send` | 发送数据（支持文本和十六进制），不等待响应 |
| `read` | 从 RX 缓冲区读取数据（非阻塞，后台自动捕获） |
| `send_command` | 发送命令并从缓冲区读取响应（推荐用于 AT 命令） |
| `modbus_read` | 读取 Modbus RTU 保持寄存器（支持工程值转换） |
| `modbus_write` | 写入 Modbus RTU 保持寄存器 |
| `plc_read` | 读取 PLC 预设品牌的所有寄存器 |
| `plc_write` | 按地址写入 PLC 寄存器 |
| `status` | 查看连接状态、字节统计、MCP 服务器信息 |
| `get_config` | 读取 UI 设置（支持读取全部或单个键值） |
| `set_config` | 修改 UI 设置（立即同步到 GUI） |
| `get_access_log` | 查看访问日志（客户端 IP、操作记录、时间戳） |

### 特性

- 所有操作自动记录客户端 IP，支持追溯
- 支持多客户端并发连接
- 支持本机和局域网模式
- 访问日志在 GUI 设置面板中可见
- 一键复制 MCP 连接信息

详见 [docs/MCP_API.md](docs/MCP_API.md) 了解完整 API 参考和 JSON-RPC 示例。

## 数据持久化

SerialRUN 自动保存以下数据到 `~/.serialrun/` 目录：

| 文件 | 内容 |
|------|------|
| `config.toml` | 主题、语言、波特率等配置 |
| `logs.json` | 日志记录（最多 2000 条） |
| `terminal.json` | 终端收发记录（最多 5000 条） |
| `warnings.json` | 警告/错误历史（最多 1000 条） |
| `mcp_access_log.json` | MCP 访问日志（最多 1000 条） |

## 插件开发

```rust
#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char { /* ... */ }

#[no_mangle]
pub extern "C" fn plugin_execute(command: *const c_char, params: *const c_char) -> *mut c_char { /* ... */ }
```

完整示例见 [plugins/serialrun-example-plugin/](plugins/serialrun-example-plugin/)。

## 文档

| 文档 | 说明 |
|------|------|
| [docs/help_en.md](docs/help_en.md) | 英文使用指南 |
| [docs/help_zh.md](docs/help_zh.md) | 中文使用指南 |
| [docs/MANUAL.md](docs/MANUAL.md) | 用户手册 |
| [docs/MCP_API.md](docs/MCP_API.md) | MCP API 参考 |
| [docs/BUILD_CN.md](docs/BUILD_CN.md) | 构建指南 |
| [CLAUDE_CN.md](CLAUDE_CN.md) | Agent 操作指南 |

## 开发

```bash
cargo build       # 构建所有 crate
cargo test        # 运行测试
cargo bench       # 运行性能测试
```

## 许可证

SerialRUN 采用 [Business Source License 1.1](https://spdx.org/licenses/BSL-1.1.html) 和 [MIT 许可证](LICENSE) 双重许可。详见 [LICENSE](LICENSE)。

---

<div align="center">

**为嵌入式开发者用心打造**

</div>
