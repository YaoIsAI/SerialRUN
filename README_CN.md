<div align="center">

# SerialRUN

**面向嵌入式开发者的跨平台串口助手**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux%20%7C%20iOS%20%7C%20Android-blue.svg)]()

[English](README.md) | [中文](#功能特性)

</div>

---

## 功能特性

- **跨平台** — Windows、macOS、Linux、iOS、Android
- **CLI & GUI** — 命令行用于自动化，桌面客户端用于交互式使用
- **协议支持** — Modbus RTU/TCP 解析，自定义协议模式匹配
- **数据可视化** — 实时图表和统计信息
- **脚本录制** — 录制和回放串口通信会话
- **文件传输** — 支持 XMODEM / YMODEM / ZMODEM
- **CAN 总线分析** — SLCAN 协议解析、帧过滤、按 ID 统计
- **I2C/SPI 调试** — 寄存器读写，支持地址和数据宽度配置
- **串口示波器** — 实时波形显示，支持触发和光标测量
- **烧录器** — STM32 ISP 和 ESP32 串口烧录
- **寄存器编辑器** — CSV/JSON 导入导出，报警阈值监控
- **数据记录器** — 持续 CSV 记录，带时间戳
- **帧生成器** — 可视化 Modbus 帧构造，实时十六进制预览
- **PLC 控制** — Modbus 寄存器轮询，内置品牌预设（西门子、三菱等）
- **插件系统** — 可扩展架构，支持动态加载插件
- **MCP 服务器** — 内置 TCP 服务器，支持 AI 助手集成
- **十六进制模式** — 以十六进制格式收发数据
- **自动回复** — 自动响应匹配的模式
- **双语界面** — 英文/中文语言切换，深色/浅色主题

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
│   ├── serialrun-gui/        # 桌面客户端 (egui)
│   ├── serialrun-mcp/        # MCP 服务器（AI 集成）
│   └── serialrun-plugin-api/ # 插件 API 定义
├── plugins/
│   └── example-plugin/       # 插件示例 (C FFI)
├── assets/                   # 嵌入式图片（二维码）
├── docs/                     # 文档
├── tests/                    # 集成测试
└── benches/                  # 性能测试
```

## GUI 面板

| 面板 | 说明 |
|------|------|
| 终端 | 串口收发，支持 HEX 模式、时间戳、CRC |
| Modbus | RTU 监听，解析功能码 |
| PLC 控制 | 寄存器轮询，内置品牌预设 |
| CAN 总线 | SLCAN 帧捕获和分析 |
| I2C/SPI | 寄存器读写调试工具 |
| 示波器 | 实时波形显示 |
| 文件传输 | XMODEM/YMODEM/ZMODEM |
| 帧生成器 | 可视化 Modbus 帧构造 |
| 烧录器 | STM32 ISP / ESP32 串口烧录 |
| 数据记录器 | CSV 记录，带时间戳 |
| 寄存器编辑器 | 导入导出寄存器映射 |
| 图表 | 多系列实时数据可视化 |
| 插件管理 | 动态插件发现和加载 |
| 日志查看 | 应用日志，支持过滤和导出 |

## 跨平台构建

| 平台 | 命令 |
|------|------|
| Windows (MSVC) | `cargo build --target x86_64-pc-windows-msvc --release` |
| macOS (Apple Silicon) | `cargo build --target aarch64-apple-darwin --release` |
| macOS (Intel) | `cargo build --target x86_64-apple-darwin --release` |
| Linux | `cargo build --target x86_64-unknown-linux-gnu --release` |

详见 [docs/BUILD_CN.md](docs/BUILD_CN.md) 了解 Android、iOS 构建及 .app 打包。

## Agent 模式 (自动化)

```bash
serialrun agent list-ports                # 列出端口 (JSON)
serialrun agent COM1 send "AT+RST"        # 发送数据
serialrun agent COM1 read --timeout 1000  # 读取数据
serialrun agent COM1 run-script test.txt  # 运行脚本
```

## MCP 服务器

SerialRUN 内置 MCP 服务器，支持 AI 助手集成。

```bash
# 启动 MCP 服务器（默认：127.0.0.1:9527）
serialrun-mcp
```

可用工具：`list_ports`、`connect`、`disconnect`、`send`、`read`、`send_command`。

## 插件开发

```rust
#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char { /* ... */ }

#[no_mangle]
pub extern "C" fn plugin_execute(command: *const c_char, params: *const c_char) -> *mut c_char { /* ... */ }
```

完整示例见 [plugins/example-plugin/](plugins/example-plugin/)。

## 文档

| 文档 | 说明 |
|------|------|
| [docs/MANUAL_CN.md](docs/MANUAL_CN.md) | 用户手册 |
| [docs/SKILL_CN.md](docs/SKILL_CN.md) | 技能参考 |
| [docs/BUILD_CN.md](docs/BUILD_CN.md) | 构建指南 |
| [CLAUDE_CN.md](CLAUDE_CN.md) | Agent 操作指南 |

## 开发

```bash
cargo build       # 构建所有 crate
cargo test        # 运行测试
cargo bench       # 运行性能测试
```

## 许可证

[MIT 许可证](LICENSE)

---

<div align="center">

**为嵌入式开发者用心打造**

</div>
