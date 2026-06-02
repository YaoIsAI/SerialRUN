# SerialRUN Plugin Development Guide

## Overview

SerialRUN plugins are C-ABI dynamic libraries (.dll/.so/.dylib) that extend the application with custom functionality. Plugins can access serial ports, show UI panels, display progress, and more.

## Quick Start

### 1. Create a new plugin project

```bash
cargo new --lib my-plugin
cd my-plugin
```

### 2. Add dependencies to Cargo.toml

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
serialrun-plugin-api = { git = "https://github.com/nicedoc/openclaw", path = "crates/serialrun-plugin-api" }
```

### 3. Implement the plugin

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use serialrun_plugin_api::*;

#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char {
    let info = serde_json::json!({
        "name": "my-plugin",
        "version": "1.0.0",
        "description": "What this plugin does",
        "author": "Your Name"
    });
    CString::new(info.to_string()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_get_commands() -> *mut c_char {
    let cmds = serde_json::json!([
        {
            "name": "do_something",
            "description": "Does something useful",
            "parameters": [
                {"name": "input", "description": "Input text", "required": true, "param_type": "string"}
            ]
        }
    ]);
    CString::new(cmds.to_string()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_execute(
    command: *const c_char,
    params: *const c_char,
) -> *mut c_char {
    let cmd = unsafe { CStr::from_ptr(command).to_string_lossy() };
    let _params: serde_json::Value = serde_json::from_str(
        &unsafe { CStr::from_ptr(params).to_string_lossy() }
    ).unwrap_or_default();

    let result = match cmd.as_ref() {
        "do_something" => serde_json::json!({
            "success": true,
            "result": {"message": "Done!"}
        }),
        _ => serde_json::json!({
            "success": false,
            "error": format!("Unknown command: {}", cmd)
        }),
    };

    CString::new(result.to_string()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}
```

### 4. Create plugin.json

```json
{
    "name": "my-plugin",
    "version": "1.0.0",
    "description": "What this plugin does",
    "author": "Your Name",
    "license": "MIT",
    "min_serialrun_version": "0.2.4",
    "platforms": ["windows-x64", "linux-x64", "macos-arm64"],
    "category": "firmware-flash",
    "tags": ["stc", "isp", "flash"],
    "homepage": "https://github.com/yourname/my-plugin",
    "repository": "https://github.com/yourname/my-plugin",
    "usage": "## My Plugin\n\nThis plugin does X, Y, Z.\n\n### Commands\n- `do_something`: Does something useful"
}
```

### 5. Build and test

```bash
cargo build --release
# Test locally: copy .dll + plugin.json to ~/.serialrun/plugins/my-plugin/
```

## Required FFI Functions

| Function | Required | Description |
|----------|----------|-------------|
| `plugin_get_info` | Yes | Returns plugin metadata as JSON string |
| `plugin_get_commands` | Yes | Returns command list as JSON string |
| `plugin_execute` | Yes | Executes a command, returns result as JSON string |
| `plugin_free_string` | Yes | Frees strings allocated by the plugin |
| `plugin_init` | No | Called once when loaded, receives `PluginCallbacks` pointer |
| `plugin_cleanup` | No | Called when unloaded, release resources |
| `plugin_get_capabilities` | No | Returns capability list as JSON string |

## plugin.json Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `name` | Yes | — | Unique identifier (used as directory name) |
| `version` | Yes | — | Semantic version (e.g. `1.0.0`) |
| `description` | Yes | — | Short description shown in plugin list |
| `author` | Yes | — | Author name |
| `license` | No | `BSL-1.1` | License identifier (SPDX) |
| `min_serialrun_version` | No | `0.1.0` | Minimum SerialRUN version required |
| `platforms` | No | all | Supported platforms: `windows-x64`, `macos-arm64`, `linux-x64`, etc. |
| `category` | No | — | Category for filtering (e.g. `firmware-flash`, `debug`, `protocol`) |
| `tags` | No | `[]` | Search tags for discoverability |
| `homepage` | No | — | Project homepage URL |
| `repository` | No | — | GitHub repository URL (used for update checking) |
| `usage` | No | — | Usage instructions shown as tooltip in UI (supports markdown) |

## Capabilities

Declare what your plugin needs by implementing `plugin_get_capabilities()`:

```rust
#[no_mangle]
pub extern "C" fn plugin_get_capabilities() -> *mut c_char {
    let caps = serde_json::json!(["serial_port", "progress", "logging"]);
    CString::new(caps.to_string()).unwrap().into_raw()
}
```

| Capability | Description |
|------------|-------------|
| `serial_port` | Plugin needs serial port read/write access |
| `ui_panel` | Plugin provides a custom UI panel |
| `file_dialog` | Plugin needs file open/save dialogs |
| `progress` | Plugin reports progress during operations |
| `logging` | Plugin uses host logging |

## Host Callbacks

When `plugin_init(callbacks)` is called, your plugin receives a `PluginCallbacks` struct:

```rust
#[repr(C)]
pub struct PluginCallbacks {
    // Serial port
    pub serial_read: Option<extern "C" fn(buf: *mut u8, len: u32, timeout_ms: u32) -> i32>,
    pub serial_write: Option<extern "C" fn(data: *const u8, len: u32) -> i32>,
    pub serial_set_baud: Option<extern "C" fn(baud: u32) -> bool>,
    pub serial_is_connected: Option<extern "C" fn() -> bool>,
    // Progress
    pub progress_set: Option<extern "C" fn(percent: f32, message: *const c_char)>,
    pub progress_set_status: Option<extern "C" fn(status: PluginStatus)>,
    pub progress_is_cancelled: Option<extern "C" fn() -> bool>,
    // File operations
    pub file_open_dialog: Option<extern "C" fn(filter: *const c_char) -> *mut c_char>,
    pub file_save_dialog: Option<extern "C" fn(filter: *const c_char) -> *mut c_char>,
    pub file_read: Option<extern "C" fn(path: *const c_char) -> *mut c_char>,
    pub free_string: Option<extern "C" fn(s: *mut c_char)>,
    // Logging
    pub log_info: Option<extern "C" fn(msg: *const c_char)>,
    pub log_warn: Option<extern "C" fn(msg: *const c_char)>,
    pub log_error: Option<extern "C" fn(msg: *const c_char)>,
}
```

### Filter format for file dialogs

Pass filters as `"Name|ext1,ext2"` format:
```
"Firmware|hex,bin,elf"
"HEX Files|hex"
```

## Thread Safety

- `plugin_execute` runs on a background thread
- Callbacks are safe to call from any thread
- Do NOT block for long periods — use `progress_set` for progress updates
- Serial port access is serialized through the host

## Error Handling

Return errors with `success: false`:
```json
{"success": false, "error": "Something went wrong"}
```

---

## Publishing to Plugin Community

### Step 1: Add `serialrun-plugin` topic to your GitHub repo

Go to your repo on GitHub → Settings → General → Topics → add `serialrun-plugin`

### Step 2: Create a Release

```bash
# Build for all platforms (or just the ones you support)
cargo build --release

# Create ZIP with plugin binary + plugin.json
# Windows:
7z a my-plugin-1.0.0-windows-x64.zip target/release/my_plugin.dll plugin.json

# Linux:
zip my-plugin-1.0.0-linux-x64.zip target/release/libmy_plugin.so plugin.json

# macOS:
zip my-plugin-1.0.0-macos-arm64.zip target/release/libmy_plugin.dylib plugin.json
```

### Step 3: Upload to GitHub Releases

1. Go to your repo → Releases → Create a new release
2. Tag: `v1.0.0`
3. Title: `v1.0.0`
4. Upload the ZIP files as release assets
5. Publish

### Step 4: Verify

Your plugin will appear in SerialRUN's Community tab within minutes. Users can:
1. Open SerialRUN → Plugins → Community tab
2. Search for your plugin by name or tags
3. Click Install — done!

### Requirements for Community listing

- [ ] Repo has `serialrun-plugin` topic
- [ ] `plugin.json` in repo root
- [ ] Release ZIP contains: binary (.dll/.so/.dylib) + `plugin.json`
- [ ] Binary is compiled for the target platform
- [ ] `repository` field in plugin.json points to the GitHub repo

### ZIP naming convention

```
{plugin-name}-{version}-{platform}.zip
```

Examples:
```
my-plugin-1.0.0-windows-x64.zip
my-plugin-1.0.0-linux-x64.zip
my-plugin-1.0.0-macos-arm64.zip
```

### Updating your plugin

1. Bump `version` in plugin.json and Cargo.toml
2. Rebuild and create new Release ZIP
3. Upload to a new GitHub Release
4. Users will see the update in the Community tab

---

## Example: STC ISP Flasher Plugin

See `plugins/serialrun-stc-isp/` for a complete working example that demonstrates:
- Serial port communication via callbacks
- Progress reporting
- MCU detection and flashing
- Custom UI panel
- Full plugin.json manifest
