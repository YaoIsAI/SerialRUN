# SerialRUN 插件开发手册

> **版本**: API v0.3.0 | **最后更新**: 2026-06-03

---

## 目录

1. [架构概览](#1-架构概览)
2. [快速开始](#2-快速开始)
3. [插件清单 plugin.json](#3-插件清单)
4. [FFI 接口规范](#4-ffi-接口规范)
5. [宿主回调 API](#5-宿主回调-api)
6. [UI 布局系统](#6-ui-布局系统)
7. [工具栏与窗口集成](#7-工具栏与窗口集成)
8. [本地开发环境搭建](#8-本地开发环境搭建)
9. [完整示例](#9-完整示例)
10. [测试与调试](#10-测试与调试)
11. [打包与发布](#11-打包与发布)
12. [插件社区生态](#12-插件社区生态)
13. [API 参考](#13-api-参考)
14. [常见问题](#14-常见问题)

---

## 1. 架构概览

### 系统架构

```
┌──────────────────────────────────────────────────────┐
│ SerialRUN 宿主（完全不知道插件的存在）                  │
│                                                        │
│ 工具栏: [终端] [Modbus] [PLC] [Plug(2)] [中] [Dark]   │
│          └─ 悬停显示插件列表，点击打开独立窗口           │
│                                                        │
│ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐   │
│ │ 终端窗口      │ │ MicroPython  │ │ STC ISP      │   │
│ │ (原生)        │ │ IDE (插件)   │ │ (插件)       │   │
│ └──────────────┘ └──────────────┘ └──────────────┘   │
│   ↑ 独立 OS 窗口   ↑ 独立 OS 窗口   ↑ 独立 OS 窗口    │
└──────────────────────────────────────────────────────┘
         ↕ FFI 调用（插件 ↔ 宿主）
┌──────────────────────────────────────────────────────┐
│ 插件 DLL（独立编译，独立发布）                          │
│ 只依赖 serialrun-plugin-api crate                    │
│ 不依赖 serialrun-gui / serialrun-core                │
└──────────────────────────────────────────────────────┘
```

### 核心原则

| 原则 | 说明 |
|------|------|
| **宿主零知识** | 宿主代码中不能出现任何插件名称 |
| **声明式集成** | 插件通过 `plugin.json` 声明工具栏和窗口配置 |
| **独立编译** | 插件只依赖 `serialrun-plugin-api` crate |
| **动态加载** | 工具栏按钮、窗口都是运行时从 manifest 生成的 |
| **独立窗口** | 插件功能在独立 OS 窗口中运行 |

---

## 2. 快速开始

### 2.1 创建插件项目

```bash
# 在 SerialRUN 仓库根目录
cargo new --lib plugins/my-plugin
cd plugins/my-plugin
```

### 2.2 配置 Cargo.toml

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"
description = "My SerialRUN plugin"
license = "BSL-1.1"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
serialrun-plugin-api = { path = "../../crates/serialrun-plugin-api" }
```

### 2.3 创建 plugin.json

```json
{
  "name": "my-plugin",
  "version": "0.1.0",
  "description": "My awesome plugin",
  "author": "Your Name",
  "license": "BSL-1.1",
  "min_serialrun_version": "0.2.0",
  "platforms": ["windows-x64", "macos-arm64", "linux-x64"],
  "category": "tool",
  "tags": ["example"],
  "toolbar": {
    "icon": "🔧",
    "label": "My Tool",
    "tooltip": "My awesome tool"
  },
  "window": {
    "title": "My Plugin Window",
    "default_width": 800,
    "default_height": 600,
    "resizable": true
  }
}
```

### 2.4 实现插件

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use serialrun_plugin_api::*;

// 存储宿主回调
static mut CALLBACKS: Option<PluginCallbacks> = None;

// 必需：返回插件信息
#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char {
    let info = PluginInfo {
        name: "my-plugin".to_string(),
        version: "0.1.0".to_string(),
        description: "My awesome plugin".to_string(),
        author: "Your Name".to_string(),
    };
    CString::new(serde_json::to_string(&info).unwrap()).unwrap().into_raw()
}

// 必需：返回命令列表
#[no_mangle]
pub extern "C" fn plugin_get_commands() -> *mut c_char {
    let commands = vec![
        PluginCommand {
            name: "hello".to_string(),
            description: "Say hello".to_string(),
            parameters: vec![PluginParameter {
                name: "name".to_string(),
                description: "Your name".to_string(),
                required: true,
                param_type: "string".to_string(),
            }],
        },
    ];
    CString::new(serde_json::to_string(&commands).unwrap()).unwrap().into_raw()
}

// 必需：执行命令
#[no_mangle]
pub extern "C" fn plugin_execute(command: *const c_char, params: *const c_char) -> *mut c_char {
    let cmd = unsafe { CStr::from_ptr(command).to_string_lossy() };
    let result = match cmd.as_ref() {
        "hello" => PluginResult::success(serde_json::json!({"message": "Hello from my-plugin!"})),
        _ => PluginResult::error(format!("Unknown command: {}", cmd)),
    };
    CString::new(serde_json::to_string(&result).unwrap()).unwrap().into_raw()
}

// 必需：释放字符串
#[no_mangle]
pub extern "C" fn plugin_free_string(s: *mut c_char) {
    if !s.is_null() { unsafe { let _ = CString::from_raw(s); } }
}

// 可选：声明能力
#[no_mangle]
pub extern "C" fn plugin_get_capabilities() -> *mut c_char {
    let caps = vec![PluginCapability::SerialPort, PluginCapability::Logging];
    CString::new(serialize_capabilities(&caps).unwrap()).unwrap().into_raw()
}

// 可选：初始化
#[no_mangle]
pub extern "C" fn plugin_init(callbacks: *const PluginCallbacks) -> bool {
    if callbacks.is_null() { return false; }
    unsafe { CALLBACKS = Some(*callbacks); }
    true
}

// 可选：清理
#[no_mangle]
pub extern "C" fn plugin_cleanup() {
    unsafe { CALLBACKS = None; }
}
```

### 2.5 添加到工作区

在 SerialRUN 根目录 `Cargo.toml` 的 `members` 中添加：

```toml
members = [
    # ... 其他 crate ...
    "plugins/my-plugin",
]
```

### 2.6 构建与测试

```bash
cargo build --release -p my-plugin

# 安装到本地测试
mkdir -p ~/.serialrun/plugins/my-plugin
cp target/release/my_plugin.dll ~/.serialrun/plugins/my-plugin/  # Windows
cp plugin.json ~/.serialrun/plugins/my-plugin/

# 重启 SerialRUN，插件自动出现在 Plug(1) 菜单中
```

---

## 3. 插件清单

`plugin.json` 是插件的元数据文件，宿主在加载前读取它。

### 基础字段（必需）

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | string | 插件唯一标识符 |
| `version` | string | 语义化版本号 |
| `description` | string | 人类可读描述 |
| `author` | string | 作者名 |

### 平台与兼容性

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `license` | string | `"BSL-1.1"` | SPDX 许可证 |
| `min_serialrun_version` | string | `"0.1.0"` | 最低宿主版本 |
| `platforms` | string[] | 全部 | 支持的平台 |

### 分类与搜索

| 字段 | 类型 | 说明 |
|------|------|------|
| `category` | string | 分类：`ide`, `firmware-flash`, `tool`, `protocol` |
| `tags` | string[] | 搜索标签 |
| `homepage` | string | 主页 URL |
| `repository` | string | GitHub 仓库 URL |
| `usage` | string | Markdown 使用说明 |

### 工具栏集成

```json
"toolbar": {
  "icon": "🔌",
  "label": "MicroPython",
  "tooltip": "MicroPython IDE"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `toolbar.icon` | string | 按钮图标（emoji） |
| `toolbar.label` | string | 按钮文字 |
| `toolbar.tooltip` | string | 鼠标悬停提示 |

### 独立窗口配置

```json
"window": {
  "title": "MicroPython IDE",
  "default_width": 800,
  "default_height": 600,
  "resizable": true,
  "min_width": 600,
  "min_height": 400
}
```

---

## 4. FFI 接口规范

### 必需函数（4个）

| 函数 | 签名 | 说明 |
|------|------|------|
| `plugin_get_info` | `fn() -> *mut c_char` | 返回 JSON PluginInfo |
| `plugin_get_commands` | `fn() -> *mut c_char` | 返回 JSON Command 数组 |
| `plugin_execute` | `fn(cmd, params) -> *mut c_char` | 执行命令，返回 PluginResult |
| `plugin_free_string` | `fn(s: *mut c_char)` | 释放字符串 |

### 可选函数（7个）

| 函数 | 签名 | 说明 |
|------|------|------|
| `plugin_get_capabilities` | `fn() -> *mut c_char` | 声明能力 |
| `plugin_init` | `fn(callbacks) -> bool` | 初始化 |
| `plugin_cleanup` | `fn()` | 清理 |
| `plugin_get_ui_layout` | `fn() -> *mut c_char` | UI 布局 JSON |

### 能力声明

```rust
PluginCapability::SerialPort        // 串口读写
PluginCapability::UiPanel           // UI 面板
PluginCapability::FileDialog        // 文件对话框
PluginCapability::Progress          // 进度报告
PluginCapability::Logging           // 日志
PluginCapability::FileSystem        // 设备文件系统
PluginCapability::EventSubscription // 事件订阅
PluginCapability::ConfigStorage     // 配置存储
PluginCapability::UiLayout          // 声明式 UI
```

---

## 5. 宿主回调 API

### 串口访问

```rust
serial_read(buf, len, timeout_ms) -> i32    // 读取
serial_write(data, len) -> i32              // 写入
serial_set_baud(baud) -> bool               // 设置波特率
serial_is_connected() -> bool               // 连接状态
```

### 文件操作

```rust
file_open_dialog(filter) -> *mut c_char     // 打开文件
file_save_dialog(filter) -> *mut c_char     // 保存文件
file_read(path) -> *mut c_char              // 读取文件
```

### 设备文件系统

```rust
fs_list_dir(path) -> *mut c_char            // 列目录
fs_read_file(path) -> *mut c_char           // 读文件
fs_write_file(path, data) -> bool           // 写文件
fs_delete_file(path) -> bool                // 删文件
fs_mkdir(path) -> bool                      // 建目录
fs_exists(path) -> bool                     // 检查存在
```

### 事件系统

```rust
on_serial_data(callback)                    // 串口数据回调
on_connection_changed(callback)             // 连接状态回调
```

### 配置存储

```rust
config_get(key) -> *mut c_char              // 获取配置
config_set(key, value) -> bool              // 设置配置
```

### 日志

```rust
log_info(msg)
log_warn(msg)
log_error(msg)
```

---

## 6. UI 布局系统

### JSON 布局声明

```json
{
  "type": "split_horizontal",
  "ratio": 0.3,
  "children": [
    {
      "type": "panel",
      "id": "file_browser",
      "title": "📁 Files",
      "content": { "type": "tree_view" }
    },
    {
      "type": "split_vertical",
      "ratio": 0.6,
      "children": [
        {
          "type": "panel",
          "id": "editor",
          "title": "📝 Editor",
          "content": { "type": "code_editor", "language": "python" }
        },
        {
          "type": "panel",
          "id": "repl",
          "title": "💬 REPL",
          "content": { "type": "terminal" }
        }
      ]
    }
  ]
}
```

### 内容类型

| 类型 | 说明 |
|------|------|
| `tree_view` | 树形文件浏览器 |
| `code_editor` | 代码编辑器 |
| `terminal` | 终端/控制台 |
| `text` | 纯文本 |
| `html` | HTML 内容 |

---

## 7. 工具栏与窗口集成

### 工作流程

1. 用户悬停 "Plug" 按钮 → 显示已安装插件列表
2. 点击插件名称 → 打开独立 OS 窗口
3. 窗口中渲染 `plugin_get_ui_layout()` 的 UI
4. 用户关闭窗口 → 状态保存

### 插件通信

```
用户操作 → 宿主 UI → plugin_execute() → 插件处理 → 返回结果 → 宿主渲染
```

---

## 8. 本地开发环境搭建

### 前置条件

- Rust 工具链（`rustup`）
- SerialRUN 源码仓库

### 开发流程

```bash
# 1. 克隆仓库
git clone https://github.com/YaoIsAI/SerialRUN.git
cd SerialRUN

# 2. 创建插件
cargo new --lib plugins/my-plugin
cd plugins/my-plugin

# 3. 配置 Cargo.toml（见快速开始）

# 4. 实现插件

# 5. 构建
cargo build --release -p my-plugin

# 6. 安装到本地
mkdir -p ~/.serialrun/plugins/my-plugin
cp target/release/my_plugin.dll ~/.serialrun/plugins/my-plugin/  # Windows
cp plugin.json ~/.serialrun/plugins/my-plugin/

# 7. 重启 SerialRUN 测试
```

### 热重载

```bash
# 修改代码后
cargo build --release -p my-plugin
cp target/release/my_plugin.dll ~/.serialrun/plugins/my-plugin/

# 在 SerialRUN 中：Plug → 管理 → 禁用 → 启用
```

---

## 9. 完整示例

### 示例：串口回显插件

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use serialrun_plugin_api::*;

static mut CALLBACKS: Option<PluginCallbacks> = None;

fn get_callbacks() -> Option<PluginCallbacks> {
    unsafe { CALLBACKS }
}

#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char {
    let info = PluginInfo {
        name: "echo-plugin".to_string(),
        version: "0.1.0".to_string(),
        description: "Serial port echo plugin".to_string(),
        author: "Example".to_string(),
    };
    CString::new(serde_json::to_string(&info).unwrap()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_get_commands() -> *mut c_char {
    let commands = vec![
        PluginCommand {
            name: "echo".to_string(),
            description: "Send data and read response".to_string(),
            parameters: vec![PluginParameter {
                name: "data".to_string(),
                description: "Data to send (hex)".to_string(),
                required: true,
                param_type: "string".to_string(),
            }],
        },
    ];
    CString::new(serde_json::to_string(&commands).unwrap()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_execute(command: *const c_char, params: *const c_char) -> *mut c_char {
    let cmd = unsafe { CStr::from_ptr(command).to_string_lossy() };
    let params_str = unsafe { CStr::from_ptr(params).to_string_lossy() };
    let params: serde_json::Value = serde_json::from_str(&params_str).unwrap_or_default();

    let result = match cmd.as_ref() {
        "echo" => {
            let cb = get_callbacks().unwrap();
            let write = cb.serial_write.unwrap();
            let read = cb.serial_read.unwrap();

            if let Some(data_hex) = params.get("data").and_then(|v| v.as_str()) {
                let data = hex::decode(data_hex).unwrap_or_default();
                write(data.as_ptr(), data.len() as u32);

                let mut buf = [0u8; 1024];
                let n = read(buf.as_mut_ptr(), buf.len() as u32, 1000);
                let response = if n > 0 {
                    hex::encode(&buf[..n as usize])
                } else {
                    "timeout".to_string()
                };

                PluginResult::success(serde_json::json!({"response": response}))
            } else {
                PluginResult::error("Missing 'data' parameter")
            }
        }
        _ => PluginResult::error(format!("Unknown command: {}", cmd)),
    };

    CString::new(serde_json::to_string(&result).unwrap()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_free_string(s: *mut c_char) {
    if !s.is_null() { unsafe { let _ = CString::from_raw(s); } }
}

#[no_mangle]
pub extern "C" fn plugin_get_capabilities() -> *mut c_char {
    let caps = vec![PluginCapability::SerialPort, PluginCapability::Logging];
    CString::new(serialize_capabilities(&caps).unwrap()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_init(callbacks: *const PluginCallbacks) -> bool {
    if callbacks.is_null() { return false; }
    unsafe { CALLBACKS = Some(*callbacks); }
    true
}

#[no_mangle]
pub extern "C" fn plugin_cleanup() {
    unsafe { CALLBACKS = None; }
}
```

---

## 10. 测试与调试

### 测试清单

| 测试项 | 验证内容 |
|--------|----------|
| 插件加载 | 日志显示 "Loaded: my-plugin v0.1.0" |
| 工具栏 | Plug 菜单中显示插件 |
| 窗口打开 | 点击后打开独立窗口 |
| 窗口拖拽 | 可自由拖拽和调整大小 |
| 功能执行 | 按钮和命令正常工作 |
| 关闭窗口 | 关闭后不崩溃 |
| 卸载插件 | 目录被删除，列表刷新 |
| 重新安装 | 卸载后重新安装正常 |

### 调试方法

- 使用 `log_info/warn/error` 输出日志
- 在 SerialRUN 的 Log 面板中查看
- 使用 `println!` 输出到 stderr

---

## 11. 打包与发布

### 插件包结构

```
my-plugin.zip
├── plugin.json
├── windows-x64/
│   └── my_plugin.dll
├── macos-arm64/
│   └── libmy_plugin.dylib
└── linux-x64/
    └── libmy_plugin.so
```

### 跨平台编译

```bash
# Windows
cargo build --release -p my-plugin

# macOS
cargo build --release -p my-plugin --target aarch64-apple-darwin

# Linux
cargo build --release -p my-plugin --target x86_64-unknown-linux-gnu
```

### 打包

```bash
# Windows
powershell -Command "Compress-Archive -Path 'plugin.json', 'windows-x64', 'macos-arm64', 'linux-x64' -DestinationPath 'my-plugin.zip'"

# macOS/Linux
zip -r my-plugin.zip plugin.json windows-x64/ macos-arm64/ linux-x64/
```

---

## 12. 插件社区生态

### 发布到社区仓库

所有社区插件统一发布到 `YaoIsAI/serialrun-plugins` 仓库：

1. 在 `serialrun-plugins` 仓库的 `plugins/` 目录下创建插件子目录
2. 将 `plugin.json` 放入该子目录
3. 编译插件并打包成 ZIP（包含 `plugin.json` + 平台二进制）
4. 创建 GitHub Release，上传 ZIP 作为附件

### 社区搜索

SerialRUN 的社区标签页从 `YaoIsAI/serialrun-plugins` 仓库搜索：
- 读取 `plugins/*/plugin.json` 获取插件列表
- 从 Releases 中匹配 ZIP 文件供下载安装

### 用户安装

1. 打开 SerialRUN → Plug → 社区
2. 搜索插件名
3. 点击"安装" → 自动下载 ZIP → 解压安装到 `~/.serialrun/plugins/`

---

## 13. API 参考

### PluginInfo

```rust
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}
```

### PluginCommand

```rust
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub parameters: Vec<PluginParameter>,
}
```

### PluginParameter

```rust
pub struct PluginParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub param_type: String,
}
```

### PluginResult

```rust
pub struct PluginResult {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}
```

### PluginCallbacks

```rust
pub struct PluginCallbacks {
    // 串口
    pub serial_read: Option<fn(buf, len, timeout_ms) -> i32>,
    pub serial_write: Option<fn(data, len) -> i32>,
    pub serial_set_baud: Option<fn(baud) -> bool>,
    pub serial_is_connected: Option<fn() -> bool>,
    // 进度
    pub progress_set: Option<fn(percent, message)>,
    pub progress_set_status: Option<fn(status)>,
    pub progress_is_cancelled: Option<fn() -> bool>,
    // 文件
    pub file_open_dialog: Option<fn(filter) -> *mut c_char>,
    pub file_save_dialog: Option<fn(filter) -> *mut c_char>,
    pub file_read: Option<fn(path) -> *mut c_char>,
    // 设备文件系统
    pub fs_list_dir: Option<fn(path) -> *mut c_char>,
    pub fs_read_file: Option<fn(path) -> *mut c_char>,
    pub fs_write_file: Option<fn(path, data) -> bool>,
    pub fs_delete_file: Option<fn(path) -> bool>,
    pub fs_mkdir: Option<fn(path) -> bool>,
    pub fs_exists: Option<fn(path) -> bool>,
    // 事件
    pub on_serial_data: Option<fn(callback)>,
    pub on_connection_changed: Option<fn(callback)>,
    // 配置
    pub config_get: Option<fn(key) -> *mut c_char>,
    pub config_set: Option<fn(key, value) -> bool>,
    // 日志
    pub log_info: Option<fn(msg)>,
    pub log_warn: Option<fn(msg)>,
    pub log_error: Option<fn(msg)>,
    // 异步
    pub execute_async: Option<fn(cmd, params, callback)>,
    // 内存
    pub free_string: Option<fn(s: *mut c_char)>,
}
```

---

## 14. 常见问题

### Q: 插件加载失败？

检查：
1. 二进制文件在正确目录
2. `plugin.json` 格式正确
3. 查看 SerialRUN Log 面板

### Q: 窗口打不开？

检查 `plugin.json` 中的 `window` 配置，确保插件已启用。

### Q: 如何调试？

使用 `log_info/warn/error` 输出日志，在 Log 面板查看。

### Q: 插件可以访问网络吗？

可以，插件是完整 Rust 代码。但建议通过宿主回调访问串口。

### Q: 如何支持多平台？

在 `plugin.json` 的 `platforms` 中列出，为每个平台编译对应二进制。
