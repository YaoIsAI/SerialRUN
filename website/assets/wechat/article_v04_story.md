# 6 天，4 个版本：一个嵌入式开发者的工具进化史

> 从一个串口调试助手，到 AI 驱动的嵌入式开发平台。SerialRUN 从 v0.1.0 到 v0.4.0 的完整故事。

---

## 第一章：起源——为什么要做 SerialRUN？

### 一个嵌入式开发者的日常

每个嵌入式开发者都经历过这样的场景：

深夜，你盯着屏幕上的串口终端，手动输入 `AT+CSQ`，等待返回，记录结果。然后输入 `AT+COPS?`，等待，记录。再输入 `AT+GMR`，等待，记录。

你的桌面上开着三个窗口：一个是串口终端（minicom），一个是 Modbus 调试工具（Modbus Poll），一个是 PLC 编程软件（TIA Portal）。每个工具都有自己的操作方式、自己的快捷键、自己的数据格式。

**工具碎片化**是嵌入式开发的最大痛点之一。

2026 年 5 月底，我决定用 Rust 写一个串口调试助手，把常用功能整合到一个工具里。这就是 SerialRUN 的起点。

### 技术选型：为什么是 Rust？

- **安全性**：没有空指针、没有数据竞争、没有缓冲区溢出。串口操作涉及硬件，安全很重要。
- **性能**：即时编译，运行时零开销。串口数据处理需要低延迟。
- **跨平台**：Windows、macOS、Linux 一套代码。嵌入式开发者用什么系统都有。
- **GUI 框架**：选择 egui（即时模式渲染），开发速度快，不需要 Web 技术栈。

---

## 第二章：v0.1.0——初始发布（2026-05-31）

### 核心功能

v0.1.0 是一个功能完整的串口调试助手：

**串口通信**
- HEX/TEXT 双模式显示
- 时间戳、序号标记
- CRC 校验（CRC16-Modbus、CRC16-CCITT、CRC32、LRC、SUM8）
- 自动发送（可配置间隔）
- DTR/RTS 硬件信号控制
- 自动波特率检测

**协议支持**
- Modbus RTU/TCP：8 种功能码（FC01-FC08），寄存器监控
- PLC 控制：西门子 S7-1200、三菱 FX3U、台达 DVP、欧姆龙 CP1H 预设
- TCP/RTU 桥接：SCADA/HMI 通过网络连接串口设备
- CAN 总线分析：SLCAN 协议，帧捕获与解析
- I2C/SPI 调试：设备扫描、数据读写

**固件烧录**
- STM32 ISP：串口烧录
- ESP32 串口烧录
- XMODEM 文件传输

**MCP 服务器**
- 内置 MCP 服务器，支持 AI 助手远程控制串口
- 15 个内置工具

**界面**
- 多窗口独立运行（OS 级窗口）
- 中英文双语
- 数据持久化（配置、日志、终端历史）

### 发布当天

v0.1.0 在 2026 年 5 月 31 日发布。发布时的状态：
- Windows 已构建并测试
- macOS 和 Linux 需要源码编译
- 没有用户反馈（刚发布）
- MCP 服务器基础功能可用，但文档不完善

---

## 第三章：v0.2.0——插件系统 + MCP 全面升级（2026-06-02）

### 两天后的反思

v0.1.0 发布两天后，我开始收到用户反馈：

1. 「能不能扩展功能？比如加一个 STC 烧录插件」
2. 「MCP 工具太少了，能不能加更多？」
3. 「AI 助手怎么连接 SerialRUN？文档不清楚」

这些问题指向两个方向：**可扩展性**和**AI 集成**。

### 插件系统

v0.2.0 的核心是插件系统。设计目标：

- 用户可以自己开发插件，扩展 SerialRUN 的功能
- 插件是 Rust 动态库（cdylib），通过 `serialrun-plugin-api` 与宿主通信
- 插件可以添加自定义面板、协议支持、工具按钮
- 社区插件通过 GitHub 仓库搜索和安装

**插件 API 设计：**
- `PluginManifest`：描述插件元数据（名称、版本、作者、能力）
- `PluginCapabilities`：声明插件需要的宿主能力（串口访问、文件对话框、进度条）
- `PluginCallbacks`：宿主回调（串口读写、文件操作、日志记录）

**第一个社区插件：STC ISP**

STC 是国产 MCU 品牌，STC89/12/15/8 系列广泛用于教学和工业。STC ISP 协议是专有的串口烧录协议，需要特定的时序和握手。

插件实现了完整的 STC ISP 流程：
1. 检测 MCU 型号（通过 ISP 握手）
2. 擦除 Flash
3. 编程（逐块写入）
4. 校验（CRC 对比）
5. 复位运行

### MCP 升级

MCP 从 15 个工具升级到完整文档化：

**新增能力：**
- 每个工具的 JSON-RPC 调用格式
- 参数说明和有效值范围
- 使用示例和注意事项
- 一键复制功能（在帮助面板中）

**MCP 架构优化：**
- 专用读缓冲区：MCP 和 GUI 终端独立读取，不互相干扰
- 实时同步：MCP `set_config` 修改后 GUI 立即更新
- 终端显示：MCP 的 TX/RX 数据在终端中可见（带 [MCP] 标签）

### CAN 总线分析器

独立的 CAN 总线分析面板：
- 独立串口连接（与终端分开）
- USB-CAN Tool 风格的帧表格（索引、时间、通道、方向、ID、类型、DLC、数据）
- 周期发送（可配置次数和周期，ID/数据自动递增）
- 端口冲突警告

### 自动化测试

43 个自动化测试，覆盖：
- 串口连接/断开/重连
- Modbus 读写（各种功能码）
- PLC 寄存器读取（4 个品牌）
- MCP 工具调用
- 插件加载/卸载/启用/禁用

所有测试基于真实 MCU 硬件，不是模拟器。

---

## 第四章：v0.3.0——快捷指令 + 用户体验优化（2026-06-04）

### 用户痛点

v0.2.0 发布后，用户反馈集中在两个方面：

1. **重复操作**：调试 AT 指令时，总是要反复输入 `AT+CSQ`、`AT+COPS?`、`AT+GMR`
2. **主题切换 bug**：切换浅色/深色模式后，重启应用又回到之前的主题

### 快捷指令功能

**设计思路：** 用户保存常用指令，一键发送。

**实现细节：**
- `QuickCommand` 结构体：`name`（显示名称）、`data`（指令内容）、`is_hex`（HEX 模式）、`line_ending`（行尾符）
- 存储在 `UserPrefs` 中，自动持久化到 `~/.serialrun/config.toml`
- UI 布局：输入行上方，可折叠（▶/▲ 切换）
- `+` 按钮：输入框右侧，绿色文字，点击保存当前输入为快捷指令
- 重复检测：保存时检查 data + is_hex 是否已存在
- 右键菜单：删除快捷指令

**用户体验：**
- 展开时显示"快捷指令：[按钮1] [按钮2]..."
- 点击按钮直接发送
- 鼠标悬停显示完整指令内容
- 颜色跟随主题（深色/浅色）

### MCP 从 15 到 19

新增 4 个工具：
- `clear_buffers`：清空 TX/RX 缓冲区
- `set_dtr`：设置 DTR 硬件信号（立即生效）
- `set_rts`：设置 RTS 硬件信号（立即生效）
- `get_config_keys`：列出所有可用配置键

**文档升级：**
- 配置表分为两类：需要重连 vs 立即生效
- 客户端帮助面板新增一键复制功能
- 官网指南更新到 19 个工具

### 主题切换修复

**根因分析：**
1. `current_theme` 初始化为硬编码的 `Theme::Light`，但 config.toml 保存的是 `Dark`
2. eframe 默认使用 Dark visuals，但 `sync_theme_visuals` 认为"已经同步了"所以跳过
3. 第一帧渲染时使用 eframe 默认 visuals，与 config 不一致

**修复方案：**
1. `current_theme` 从 `prefs.theme` 读取（不硬编码）
2. 在 `main.rs` 中用 `cc.egui_ctx.set_visuals()` 在第一帧之前设置正确 visuals
3. `sync_theme_visuals` 添加 `ctx.request_repaint()` 确保切换后立即重绘

### PLC 标签修复

终端中的 PLC 数据没有显示 `[PLC]` 源标签。原因是 `add_terminal_line_tagged` 设置了 `tag` 字段，但终端渲染代码只检查 `source` 字段。修复：`source_tag` 优先检查 `source`，然后回退到 `extract_line_tag()`（检查 `tag` 字段）。

---

## 第五章：v0.4.0——CLI 交互模式（2026-06-05）

### 用户需求

v0.3.0 发布后，一个重要的用户反馈：

> 「能不能不开 GUI 就用 SerialRUN？我在 CI/CD 流程中需要测试串口设备，不可能每次都打开一个 GUI 窗口。」

这个需求指向一个架构问题：SerialRUN 目前只有 GUI 交互方式。AI 通过 MCP 连接，但人类只能通过 GUI 操作。

### 架构决策

三个方案：

**方案 A：CLI 独立安装**
- 单独的 serialrun-cli 二进制文件
- 优点：架构清晰
- 缺点：两个安装包，版本同步问题

**方案 B：CLI 集成到 GUI（最终选择）**
- 同一个 serialrun 二进制，检测子命令走 CLI 路径
- 优点：一次安装，GUI 和 CLI 共享代码
- 缺点：需要重构 main.rs

**方案 C：CLI 通过 MCP 通信**
- CLI 是 MCP 的命令行客户端
- 优点：架构统一
- 缺点：需要 GUI 运行

选择方案 B，因为用户体验最好：一个文件搞定。

### 实现细节

**命令解析：** 使用 `clap` 库，定义 CLI 子命令：

```rust
#[derive(Subcommand)]
pub enum Commands {
    Interactive { port: String, baud: u32 },
    ListPorts,
    Connect { port: String, baud: u32, ... },
    Disconnect,
    Send { data: String, hex: bool },
    Read { timeout: u64, format: String },
    SendCommand { command: String, timeout: u64 },
    ModbusRead { slave: u8, address: u16, quantity: u16, ... },
    ModbusWrite { slave: u8, address: u16, value: u16 },
    Status,
}
```

**模式检测：** 在 `main()` 开头检查命令行参数：

```rust
// 无参数 → 启动 GUI
// serialrun list-ports → CLI 模式
// serialrun interactive → 交互模式
```

**连接持久化：** CLI 命令是独立进程，无法跨进程共享状态。解决方案：`~/.serialrun/cli_port` 文件保存最后的连接参数（端口 + 波特率），后续命令自动读取并重新打开串口。

**交互模式（minicom 风格）：**

```
serialrun> cmd AT
← AT
> 
serialrun> send 你好
→ Sent 6 bytes
serialrun> cmd AT+CSQ
← ERR
serialrun> modbus-r 0 5
← [1, 2, 3, 4, 5] (HEX: 00 01 00 02 00 03 00 04 00 05)
serialrun> status
Connected: /dev/ttyUSB0 @ 115200 baud
serialrun> exit
Bye! Disconnected.
```

串口在整个会话期间保持打开，命令直接执行。这和 minicom 的体验一致。

### CLI 测试报告

| 命令 | 结果 | 说明 |
|------|------|------|
| `list-ports` | ✅ | 正确列出串口 |
| `connect` | ✅ | 连接成功，保存配置 |
| `send "你好"` | ✅ | 发送 6 字节 |
| `send "41 54" --hex` | ✅ | HEX 发送 |
| `send-command "AT"` | ✅ | 收到 AT 回复 |
| `read` | ⚠️ | 独立进程限制（数据丢失） |
| `modbus-read` | ✅ | Modbus 读取成功 |
| `modbus-write` | ✅ | Modbus 写入成功 |
| `status` | ✅ | 显示连接状态 |
| `interactive` | ✅ | 交互模式正常 |
| `disconnect` | ✅ | 断开成功 |

### MCP vs CLI vs GUI

| | GUI | MCP | CLI |
|---|-----|-----|-----|
| 交互方式 | 鼠标点击 | 自然语言 | 命令行 |
| 需要运行 | ✅ | ✅ | ❌ |
| 适合场景 | 日常调试 | AI 对话 | 自动化/脚本 |
| 连接保持 | ✅ | ✅ | ✅（交互模式） |
| 安装方式 | 下载即用 | 内置 | 同一二进制 |

### 帮助面板升级

客户端帮助面板新增两个一键复制区域：

1. **MCP 服务器指南** — 19 个工具的完整文档
2. **CLI 操作手册** — 交互模式 + 单次命令 + Modbus 示例

用户点击按钮即可复制完整文档，粘贴给 AI 助手或放入脚本。

---

## 第六章：Bug 调试故事

### Bug 1：主题切换需要点两次

**现象：** 从 Dark 切换到 Light，第一次点击没反应，第二次才切换。

**调试过程：**
1. 检查 `sync_theme_visuals` — 逻辑正确
2. 检查 `current_theme` 初始化 — 硬编码为 `Theme::Light`！
3. 根因：config.toml 保存 Dark，但 `current_theme` 初始化为 Light → `sync_theme_visuals` 认为"已同步" → 跳过
4. 修复：`current_theme: prefs.theme`

但修复后还是有问题。继续排查：
5. eframe 默认使用 Dark visuals → 第一帧渲染用 Dark
6. `sync_theme_visuals` 在 UI 渲染之前执行 → 设置 visuals 后 UI 用新 visuals
7. 但 `set_visuals()` 不会立即影响已经渲染的 widget → 需要 `ctx.request_repaint()`
8. 最终修复：在 `main.rs` 的 `run_native` 回调中，用 `cc.egui_ctx.set_visuals()` 在第一帧之前设置正确 visuals

**教训：** eframe 的 visuals 不是即时生效的，需要在正确的时间点设置。

### Bug 2：PLC 标签不显示

**现象：** 终端中的 PLC 数据没有 `[PLC]` 标签，但过滤栏里有 PLC 选项。

**调试过程：**
1. PLC 面板用 `add_terminal_line_tagged(..., "PLC")` 添加数据
2. 终端渲染代码检查 `line.source` 字段显示标签
3. `add_terminal_line_tagged` 设置的是 `tag` 字段，不是 `source` 字段！
4. `extract_line_tag()` 函数已经会检查 `tag` 字段
5. 但 `source_tag` 的显示逻辑只用了 `line.source`
6. 修复：`source_tag` 优先检查 `source`，然后回退到 `extract_line_tag()`

### Bug 3：CLI 首次运行无法启动 GUI

**现象：** `serialrun`（无参数）没有启动 GUI，直接退出。

**调试过程：**
1. 检查 main.rs 的 CLI 检测逻辑
2. 原始代码：无参数时进入交互模式
3. 用户期望：无参数时启动 GUI
4. 修复：无参数时不进入 CLI 模式，继续执行 GUI 启动逻辑

### Bug 4：CLI 命令无法保持连接

**现象：** `serialrun connect COM3` 连接后，`serialrun send "AT"` 显示"未连接"。

**调试过程：**
1. 每个 CLI 命令是独立进程
2. `ACTIVE_PORT` Mutex 只在同一进程内有效
3. 进程结束后串口关闭，数据丢失
4. 解决方案：`~/.serialrun/cli_port` 文件保存连接参数，后续命令自动重新打开

### Bug 5：二进制文件不一致

**现象：** `make install` 后 `/Applications/SerialRUN.app` 里的二进制和 `target/release/serialrun` 不同。

**调试过程：**
1. MD5 对比发现不同
2. `make app` 的 `cargo build` 有增量编译缓存
3. `icon_embedded.png` 被 `gen_icon.py` 更新后触发重编译
4. 但 `make install` 复制的是旧的 bundle
5. 解决方案：`rm -rf` 后重新 `cp -R`

---

## 第七章：使用指南

### 快速开始

**下载安装：**
- Windows：下载 zip → 解压 → 运行 `serialrun.exe`
- macOS：下载 zip → 拖到 Applications → 双击运行
- Linux：源码编译 `cargo build --release`

### GUI 操作

1. **连接串口**：左侧面板选择端口和波特率 → 点击"连接"
2. **发送数据**：底部输入框输入内容 → 点击"发送"
3. **快捷指令**：输入常用指令 → 点击 `+` 保存 → 以后一键发送
4. **Modbus 调试**：顶部工具栏切换到 Modbus 面板
5. **PLC 控制**：顶部工具栏切换到 PLC 面板
6. **MCP 服务器**：左侧面板启用 → AI 助手通过 TCP 连接

### CLI 操作

**交互模式（推荐）：**
```bash
serialrun interactive /dev/ttyUSB0 --baud 115200
# 进入交互模式
serialrun> cmd AT
serialrun> send-command "AT+CSQ" --timeout 3000
serialrun> modbus-r 0 10 --slave 1
serialrun> exit
```

**单次命令：**
```bash
serialrun list-ports
serialrun connect COM3 --baud 115200
serialrun send-command "AT" --timeout 3000
serialrun disconnect
```

### MCP 操作

1. 启用 MCP 服务器（默认端口 9527）
2. 复制 MCP 配置（帮助面板 → 复制 MCP 说明）
3. 粘贴给 AI 助手
4. AI 用自然语言操作：

```
你：帮我连接串口，发送 AT 指令检查模块
AI：[connect] → [send_command] "AT" → [read]
    模块正常，信号强度 -65dBm
```

### 19 个 MCP 工具

| 类别 | 工具 |
|------|------|
| 基础串口 | list_ports, connect, disconnect, send, read, send_command, clear_buffers |
| Modbus | modbus_read, modbus_write |
| PLC | plc_read, plc_write |
| 硬件信号 | set_dtr, set_rts |
| 配置管理 | status, get_config, set_config, get_config_keys, get_device_info, get_access_log |

---

## 第八章：感谢与展望

SerialRUN 的快速迭代离不开用户反馈。每一个 bug 报告、每一个功能请求，都让这个工具变得更好。

**感谢：**
- 所有在 GitHub 上 Star、Fork、提 Issue 的开发者
- AGI Builder（WaytoAGI + 红杉）的认可和孵化
- 嵌入式开发社区的支持

**展望：**
- v0.5.0：更多 Modbus 功能码（FC01/FC02/FC04/FC16）
- v0.6.0：社区插件市场
- v0.7.0：AI 深度集成（理解设备协议）

---

## 获取 SerialRUN

- 🌐 **官网下载**: www.serialrun.com/downloads.html
- 📖 **使用指南**: www.serialrun.com/guide.html
- 📖 **MCP 指南**: www.serialrun.com/guide.html#mcp
- ⭐ **GitHub**: github.com/YaoIsAI/SerialRUN

**当前版本：v0.4.0**
- 19 个 MCP 工具
- CLI 交互模式（minicom 风格）
- 快捷指令功能
- 跨平台支持

**下载地址：**
- macOS：www.serialrun.com/downloads.html（v0.4.0 Universal）
- Windows：www.serialrun.com/downloads.html（v0.3.0）
- Linux：源码编译

---

*SerialRUN 是开源项目（BSL 1.1），欢迎 Star、Fork、提交 Issue！*

*如果你也在做嵌入式开发相关的工具，欢迎交流。*
