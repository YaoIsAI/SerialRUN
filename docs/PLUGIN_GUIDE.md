# SerialRUN 插件开发指南

## 概述

SerialRUN 支持通过插件扩展功能。插件是动态链接库（.dll/.so/.dylib），通过 C FFI 接口与主程序通信。

## 快速开始

### 1. 创建插件项目

```bash
cargo new my-plugin --lib
cd my-plugin
```

### 2. 配置 Cargo.toml

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serialrun-plugin-api = "0.2.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### 3. 实现插件

```rust
use serialrun_plugin_api::{PluginInfo, PluginCommand, PluginParameter, PluginResult};
use std::ffi::{c_char, CStr, CString};

// 必须：返回插件信息
#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char {
    let info = PluginInfo {
        name: "My Plugin".to_string(),
        version: "0.1.0".to_string(),
        description: "我的第一个插件".to_string(),
        author: "Your Name".to_string(),
    };
    CString::new(serde_json::to_string(&info).unwrap()).unwrap().into_raw()
}

// 必须：返回可用命令列表
#[no_mangle]
pub extern "C" fn plugin_get_commands() -> *mut c_char {
    let commands = vec![
        PluginCommand {
            name: "hello".to_string(),
            description: "Say hello".to_string(),
            parameters: vec![PluginParameter {
                name: "name".to_string(),
                description: "Name to greet".to_string(),
                required: true,
                param_type: "string".to_string(),
            }],
        },
    ];
    CString::new(serde_json::to_string(&commands).unwrap()).unwrap().into_raw()
}

// 必须：执行命令
#[no_mangle]
pub extern "C" fn plugin_execute(command: *const c_char, params: *const c_char) -> *mut c_char {
    let cmd = unsafe { CStr::from_ptr(command).to_string_lossy() };
    let params: serde_json::Value = unsafe {
        if params.is_null() {serde_json::json!({})}
        else { serde_json::from_str(&CStr::from_ptr(params).to_string_lossy()).unwrap_or_default() }
    };

    let result = match cmd.as_ref() {
        "hello" => {
            let name = params["name"].as_str().unwrap_or("World");
            PluginResult::success(serde_json::json!(format!("Hello, {}!", name)))
        }
        _ => PluginResult::error(format!("Unknown command: {}", cmd)),
    };

    CString::new(serde_json::to_string(&result).unwrap()).unwrap().into_raw()
}

// 必须：释放字符串
#[no_mangle]
pub extern "C" fn plugin_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}
```

### 4. 编译

```bash
cargo build --release
```

产出文件在 `target/release/` 目录：
- Windows: `my_plugin.dll`
- macOS: `libmy_plugin.dylib`
- Linux: `libmy_plugin.so`

### 5. 打包插件

```bash
# Windows
7z a my-plugin-1.0.0-windows-x64.zip target/release/my_plugin.dll plugin.json

# Linux
zip my-plugin-1.0.0-linux-x64.zip target/release/libmy_plugin.so plugin.json

# macOS
zip my-plugin-1.0.0-macos-arm64.zip target/release/libmy_plugin.dylib plugin.json
```

### 6. 安装插件

**方式一：ZIP 导入**
1. 在 SerialRUN 中点击 **导入 ZIP**
2. 选择打包好的 .zip 文件
3. 插件自动安装并加载

**方式二：社区安装（推荐发布方式）**
1. 将代码推送到 GitHub 仓库
2. 给仓库添加 `serialrun-plugin` topic 标签
3. 创建 Release，上传 ZIP 文件
4. 用户在 SerialRUN 的 **社区** 标签页搜索并安装

---

## 必须的 FFI 函数

每个插件必须导出这 4 个函数：

| 函数 | 签名 | 说明 |
|------|------|------|
| `plugin_get_info` | `fn() -> *mut c_char` | 返回 JSON 格式的插件信息 |
| `plugin_get_commands` | `fn() -> *mut c_char` | 返回 JSON 格式的命令列表 |
| `plugin_execute` | `fn(command, params) -> *mut c_char` | 执行命令，返回 JSON 结果 |
| `plugin_free_string` | `fn(s: *mut c_char)` | 释放分配的字符串 |

---

## 可选的 FFI 函数

### plugin_get_capabilities

声明插件需要的宿主能力：

```rust
use serialrun_plugin_api::{PluginCapability, serialize_capabilities};

#[no_mangle]
pub extern "C" fn plugin_get_capabilities() -> *mut c_char {
    let caps = vec![PluginCapability::Logging, PluginCapability::FileDialog];
    CString::new(serialize_capabilities(&caps).unwrap()).unwrap().into_raw()
}
```

可用的能力：

| 能力 | 说明 |
|------|------|
| `SerialPort` | 需要串口读写访问 |
| `UiPanel` | 提供自定义 UI 面板 |
| `FileDialog` | 需要文件打开/保存对话框 |
| `Progress` | 需要进度条回调 |
| `Logging` | 使用宿主日志功能 |

### plugin_init

初始化插件，接收宿主回调：

```rust
use serialrun_plugin_api::PluginCallbacks;
use std::sync::{Mutex, OnceLock};
use std::ffi::CStr;

static CALLBACKS: OnceLock<Mutex<Option<PluginCallbacks>>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn plugin_init(callbacks: *const PluginCallbacks) -> bool {
    if callbacks.is_null() { return false; }
    let cbs = unsafe { *callbacks };
    let store = CALLBACKS.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = store.lock() {
        *guard = Some(cbs);
        // 使用日志回调
        if let Some(log) = cbs.log_info {
            let msg = std::ffi::CString::new("Plugin initialized!").unwrap();
            log(msg.as_ptr());
        }
    }
    true
}
```

### plugin_cleanup

清理插件资源：

```rust
#[no_mangle]
pub extern "C" fn plugin_cleanup() {
    if let Some(store) = CALLBACKS.get() {
        if let Ok(mut guard) = store.lock() {
            *guard = None;
        }
    }
}
```

---

## 宿主回调 API

通过 `PluginCallbacks` 结构体，插件可以调用宿主提供的功能：

### 串口访问

```rust
// 读取串口数据
let cbs = CALLBACKS.get().unwrap().lock().unwrap();
if let Some(ref cbs) = *cbs {
    if let Some(read) = cbs.serial_read {
        let mut buf = [0u8; 1024];
        let n = read(buf.as_mut_ptr(), buf.len() as u32, 1000);
        if n > 0 {
            let data = &buf[..n as usize];
            // 处理数据
        }
    }

    // 写入串口数据
    if let Some(write) = cbs.serial_write {
        let data = b"AT+RST\r\n";
        write(data.as_ptr(), data.len() as u32);
    }

    // 设置波特率
    if let Some(set_baud) = cbs.serial_set_baud {
        set_baud(115200);
    }

    // 检查连接状态
    if let Some(is_connected) = cbs.serial_is_connected {
        if is_connected() {
            // 已连接
        }
    }
}
```

### 进度报告

```rust
if let Some(ref cbs) = *cbs {
    if let Some(progress) = cbs.progress_set {
        let msg = CString::new("正在写入...").unwrap();
        progress(50.0, msg.as_ptr()); // 50%
    }

    if let Some(status) = cbs.progress_set_status {
        status(PluginStatus::Running); // 或 Success, Error, Idle
    }

    if let Some(cancelled) = cbs.progress_is_cancelled {
        if cancelled() {
            // 用户取消了操作
            return;
        }
    }
}
```

### 文件操作

```rust
if let Some(ref cbs) = *cbs {
    // 打开文件对话框
    if let Some(open) = cbs.file_open_dialog {
        let filter = CString::new("Firmware").unwrap();
        let path_ptr = open(filter.as_ptr());
        if !path_ptr.is_null() {
            let path = CStr::from_ptr(path_ptr).to_string_lossy().to_string();
            if let Some(free) = cbs.free_string {
                free(path_ptr);
            }
            // 使用 path
        }
    }

    // 读取文件（返回 base64）
    if let Some(read) = cbs.file_read {
        let path = CString::new("/path/to/file.hex").unwrap();
        let data_ptr = read(path.as_ptr());
        if !data_ptr.is_null() {
            let b64 = CStr::from_ptr(data_ptr).to_string_lossy().to_string();
            if let Some(free) = cbs.free_string {
                free(data_ptr);
            }
            // 解码 base64 使用
        }
    }
}
```

### 日志

```rust
if let Some(ref cbs) = *cbs {
    if let Some(info) = cbs.log_info {
        let msg = CString::new("Info message").unwrap();
        info(msg.as_ptr());
    }
    if let Some(warn) = cbs.log_warn {
        let msg = CString::new("Warning message").unwrap();
        warn(msg.as_ptr());
    }
    if let Some(error) = cbs.log_error {
        let msg = CString::new("Error message").unwrap();
        error(msg.as_ptr());
    }
}
```

---

## 命令参数格式

命令参数通过 JSON 字符串传递：

```json
{
    "name": "value",
    "number": 42,
    "flag": true,
    "list": [1, 2, 3]
}
```

返回结果：

```rust
// 成功
PluginResult::success(serde_json::json!({"data": "result"}))

// 失败
PluginResult::error("Something went wrong")
```

---

## 插件管理

### 安装

将 `.dll` / `.so` / `.dylib` 文件放入 `plugins/` 目录。

### 卸载

从 `plugins/` 目录删除文件，重启 SerialRUN。

### 启用/禁用

在 SerialRUN 的插件管理器中，勾选/取消勾选启用开关。

---

## 示例插件

`plugins/serialrun-example-plugin/` 包含完整的示例：

- 3 个命令：echo、timestamp、add
- 使用 Logging 能力
- 完整的单元测试

参考此插件学习如何开发自己的插件。

---

## 常见问题

**Q: 插件加载失败？**
A: 检查：
1. 文件扩展名是否正确（.dll/.so/.dylib）
2. 是否导出了 4 个必须的 FFI 函数
3. JSON 格式是否正确
4. 依赖库是否完整

**Q: 插件能访问串口吗？**
A: 可以。在 `plugin_get_capabilities` 中声明 `SerialPort`，然后通过 `PluginCallbacks` 的 `serial_read`/`serial_write` 回调访问。

**Q: 插件能显示 UI 吗？**
A: 当前版本支持专用 UI 面板（如 STC ISP 插件）。其他插件通过命令面板交互。自定义 UI 面板支持在后续版本中扩展。

**Q: 插件是线程安全的吗？**
A: 插件的 FFI 函数可能从不同线程调用。如果插件有共享状态，需要自己加锁。
