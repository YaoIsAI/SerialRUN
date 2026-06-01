# SerialRUN Plugin Development Guide

## Overview

SerialRUN plugins are C-ABI dynamic libraries (.dll/.so/.dylib) that extend the application with custom functionality. Plugins can access serial ports, show UI panels, display progress, and more.

## Quick Start

### 1. Create a new plugin project

```bash
cargo new --lib my-plugin
```

### 2. Add dependencies to Cargo.toml

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
serialrun-plugin-api = "0.1"
```

### 3. Implement the plugin

```rust
use serialrun_plugin_api::*;

#[no_mangle]
pub extern "C" fn plugin_get_info() -> *const PluginInfo {
    // Return plugin metadata
}

#[no_mangle]
pub extern "C" fn plugin_get_commands() -> *const PluginCommands {
    // Return available commands
}

#[no_mangle]
pub extern "C" fn plugin_execute(
    command: *const c_char,
    params: *const c_char,
    callbacks: *const PluginCallbacks,
) -> PluginResult {
    // Execute a command
}

#[no_mangle]
pub extern "C" fn plugin_free_string(s: *mut c_char) {
    // Free a string returned by the plugin
}
```

### 4. Build and package

```bash
cargo build --release
# Create a zip with: my_plugin.dll (or .so/.dylib) + plugin.json
```

## Required FFI Functions

| Function | Required | Description |
|----------|----------|-------------|
| `plugin_get_info` | Yes | Returns plugin name, version, author |
| `plugin_get_commands` | Yes | Returns list of commands with descriptions |
| `plugin_execute` | Yes | Executes a command with JSON params |
| `plugin_free_string` | Yes | Frees strings allocated by the plugin |
| `plugin_init` | No | Called once when plugin is loaded, receives callbacks |
| `plugin_cleanup` | No | Called when plugin is unloaded |
| `plugin_get_capabilities` | No | Returns plugin capabilities (serial_port, ui_panel, etc.) |

## Plugin Manifest (plugin.json)

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "What this plugin does",
  "usage": "Detailed usage instructions shown in the UI",
  "platforms": ["windows", "linux", "macos"]
}
```

## Available Capabilities

- `serial_port` - Can read/write serial ports
- `ui_panel` - Provides a custom UI panel
- `file_dialog` - Can open file open/save dialogs
- `progress` - Can show progress bars
- `logging` - Can write to the application log

## Callbacks

Plugins receive a `PluginCallbacks` struct with function pointers:

```rust
pub struct PluginCallbacks {
    pub serial_read: Option<fn(buf: *mut u8, len: u32, timeout_ms: u32) -> i32>,
    pub serial_write: Option<fn(data: *const u8, len: u32) -> i32>,
    pub serial_set_baud: Option<fn(baud: u32) -> bool>,
    pub serial_is_connected: Option<fn() -> bool>,
    pub progress_set: Option<fn(percent: f32, message: *const c_char)>,
    pub progress_set_status: Option<fn(status: PluginStatus)>,
    pub progress_is_cancelled: Option<fn() -> bool>,
    pub file_open_dialog: Option<fn(filter: *const c_char) -> *mut c_char>,
    pub file_save_dialog: Option<fn(filter: *const c_char) -> *mut c_char>,
    pub file_read: Option<fn(path: *const c_char) -> *mut c_char>,
    pub free_string: Option<fn(s: *mut c_char)>,
    pub log_info: Option<fn(msg: *const c_char)>,
    pub log_warn: Option<fn(msg: *const c_char)>,
    pub log_error: Option<fn(msg: *const c_char)>,
}
```

## Installation

### Method 1: ZIP Import (Recommended)
1. Package your plugin as a ZIP containing: `my_plugin.dll` + `plugin.json`
2. In SerialRUN: Plugins > Import ZIP
3. Select your ZIP file

### Method 2: Manual Install
1. Create directory: `~/.serialrun/plugins/my-plugin/`
2. Copy `my_plugin.dll` and `plugin.json` into it
3. Restart SerialRUN or click Refresh

## Thread Safety

- Plugin `execute` is called on a background thread
- Callbacks are safe to call from any thread
- Do NOT block for long periods in `execute` - use callbacks for progress updates
- Serial port access is serialized through the host

## Error Handling

Return errors as `PluginResult` with `success: false` and an error message. The host will display the error to the user.

## Example: STC ISP Flasher Plugin

See `plugins/serialrun-stc-isp/` for a complete working example that demonstrates:
- Serial port communication via callbacks
- Progress reporting
- MCU detection and flashing
- Custom UI panel
