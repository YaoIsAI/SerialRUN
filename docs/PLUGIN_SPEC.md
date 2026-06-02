# SerialRUN Plugin Development Specification

## Overview

SerialRUN plugins are shared libraries (.dll/.so/.dylib) that extend the application's functionality. Each plugin communicates with the host through defined FFI interfaces.

## Architecture

```
┌─────────────────────────────────────────────────┐
│  SerialRUN Host                                 │
│  ┌─────────────────────────────────────────────┐│
│  │ Plugin Manager                              ││
│  │  ├─ Load .dll/.so/.dylib via libloading     ││
│  │  ├─ Call plugin_init(callbacks)             ││
│  │  ├─ Forward commands to plugin_execute()    ││
│  │  └─ Manage lifecycle (init → execute → drop)││
│  └─────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────┐│
│  │ Host Callbacks (provided to plugin)         ││
│  │  ├─ serial_read / serial_write              ││
│  │  ├─ progress_set / progress_set_status      ││
│  │  ├─ file_open_dialog / file_save_dialog     ││
│  │  └─ log_info / log_warn / log_error         ││
│  └─────────────────────────────────────────────┘│
└─────────────────────────────────────────────────┘
          │ FFI (C ABI)
          ▼
┌─────────────────────────────────────────────────┐
│  Plugin (.dll/.so/.dylib)                       │
│  ┌─────────────────────────────────────────────┐│
│  │ Required FFI Functions                      ││
│  │  ├─ plugin_get_info() → PluginInfo JSON     ││
│  │  ├─ plugin_get_commands() → Commands JSON   ││
│  │  ├─ plugin_execute(cmd, params) → Result    ││
│  │  └─ plugin_free_string(s)                   ││
│  └─────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────┐│
│  │ Optional FFI Functions                      ││
│  │  ├─ plugin_get_capabilities() → Caps JSON   ││
│  │  ├─ plugin_init(callbacks) → bool           ││
│  │  └─ plugin_cleanup()                        ││
│  └─────────────────────────────────────────────┘│
└─────────────────────────────────────────────────┘
```

## Required FFI Functions

Every plugin MUST export these 4 functions:

### plugin_get_info

```rust
#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char
```

Returns JSON-serialized plugin info:
```json
{
    "name": "my-plugin",
    "version": "1.0.0",
    "description": "Plugin description",
    "author": "Author Name"
}
```

### plugin_get_commands

```rust
#[no_mangle]
pub extern "C" fn plugin_get_commands() -> *mut c_char
```

Returns JSON array of commands:
```json
[
    {
        "name": "flash",
        "description": "Flash firmware to device",
        "parameters": [
            {"name": "firmware_path", "description": "Path to firmware file", "required": true, "param_type": "string"},
            {"name": "baud_rate", "description": "Baud rate", "required": false, "param_type": "number"}
        ]
    }
]
```

### plugin_execute

```rust
#[no_mangle]
pub extern "C" fn plugin_execute(command: *const c_char, params: *const c_char) -> *mut c_char
```

- `command`: JSON string of the command name
- `params`: JSON string of parameters
- Returns: JSON-serialized result

```json
// Success
{"success": true, "result": {"data": "..."}}
// Error
{"success": false, "error": "Something went wrong"}
```

### plugin_free_string

```rust
#[no_mangle]
pub extern "C" fn plugin_free_string(s: *mut c_char)
```

Frees a string allocated by the plugin. Required for memory management.

## Optional FFI Functions

### plugin_get_capabilities

```rust
#[no_mangle]
pub extern "C" fn plugin_get_capabilities() -> *mut c_char
```

Returns JSON array of capabilities:
```json
["serial_port", "progress", "logging"]
```

Available capabilities:
| Capability | Description |
|------------|-------------|
| `serial_port` | Plugin needs serial port read/write access |
| `ui_panel` | Plugin provides a custom UI panel |
| `file_dialog` | Plugin needs file open/save dialogs |
| `progress` | Plugin reports progress during operations |
| `logging` | Plugin uses host logging |

### plugin_init

```rust
#[no_mangle]
pub extern "C" fn plugin_init(callbacks: *const PluginCallbacks) -> bool
```

Called once after loading. Receives host callbacks for serial port, file dialogs, progress, and logging.

### plugin_cleanup

```rust
#[no_mangle]
pub extern "C" fn plugin_cleanup()
```

Called before unloading. Plugin should release all resources.

## PluginCallbacks Structure

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PluginCallbacks {
    pub serial_read: Option<extern "C" fn(buf: *mut u8, len: u32, timeout_ms: u32) -> i32>,
    pub serial_write: Option<extern "C" fn(data: *const u8, len: u32) -> i32>,
    pub serial_set_baud: Option<extern "C" fn(baud: u32) -> bool>,
    pub serial_is_connected: Option<extern "C" fn() -> bool>,
    pub progress_set: Option<extern "C" fn(percent: f32, message: *const c_char)>,
    pub progress_set_status: Option<extern "C" fn(status: PluginStatus)>,
    pub progress_is_cancelled: Option<extern "C" fn() -> bool>,
    pub file_open_dialog: Option<extern "C" fn(filter: *const c_char) -> *mut c_char>,
    pub file_save_dialog: Option<extern "C" fn(filter: *const c_char) -> *mut c_char>,
    pub file_read: Option<extern "C" fn(path: *const c_char) -> *mut c_char>,
    pub free_string: Option<extern "C" fn(s: *mut c_char)>,
    pub log_info: Option<extern "C" fn(msg: *const c_char)>,
    pub log_warn: Option<extern "C" fn(msg: *const c_char)>,
    pub log_error: Option<extern "C" fn(msg: *const c_char)>,
}
```

## Plugin Manifest Format

### Directory Structure

```
my-plugin/
├── plugin.json          # Manifest
├── my_plugin.dll        # Windows binary
├── libmy_plugin.so      # Linux binary
└── libmy_plugin.dylib   # macOS binary
```

### plugin.json

```json
{
    "name": "my-plugin",
    "version": "1.0.0",
    "description": "Plugin description",
    "author": "Author Name",
    "license": "BSL-1.1",
    "min_serialrun_version": "0.1.0",
    "platforms": ["windows-x64", "macos-arm64", "linux-x64"],
    "category": "firmware-flash",
    "tags": ["stc", "isp", "flash"],
    "homepage": "https://github.com/user/project",
    "repository": "https://github.com/user/project",
    "usage": "Detailed usage instructions shown as tooltip"
}
```

### Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | String | (required) | Unique identifier, used as directory name |
| `version` | String | (required) | Semantic version |
| `description` | String | (required) | Human-readable description |
| `author` | String | (required) | Author name |
| `license` | String | `"BSL-1.1"` | SPDX license identifier |
| `min_serialrun_version` | String | `"0.1.0"` | Minimum app version required |
| `platforms` | Vec<String> | all | Supported platforms |
| `category` | String | `""` | Plugin category for filtering |
| `tags` | Vec<String> | `[]` | Search tags for discoverability |
| `homepage` | String | `""` | Project homepage URL |
| `repository` | String | `""` | GitHub repo URL (used for update checking) |
| `usage` | String | `""` | Usage instructions (shown as tooltip in UI) |

### Platform Strings

| Value | Platform |
|-------|----------|
| `windows-x64` | Windows x86_64 |
| `macos-arm64` | macOS Apple Silicon |
| `macos-x64` | macOS Intel |
| `linux-x64` | Linux x86_64 |
| `linux-arm64` | Linux ARM64 |

## Plugin Lifecycle

```
1. User imports ZIP or installs from community
2. PluginManager extracts to ~/.serialrun/plugins/<name>/
3. Plugin state saved to plugin_state.json
4. On app start, PluginManager.discover() scans plugins/
5. LoadedPlugin::load() opens shared library via libloading
6. plugin_get_info() called → metadata displayed
7. plugin_get_commands() called → commands listed
8. plugin_init(callbacks) called → plugin receives host callbacks
9. User runs commands → plugin_execute() called
10. On app exit or unload, plugin_cleanup() called
```

## Security Model

- Plugins run in the same process as the host
- Plugins can access serial port only through host callbacks
- Plugins cannot access filesystem directly (except through file callbacks)
- No sandbox isolation (considered for future versions)

## Publishing to Community

See [PLUGIN_DEVELOPMENT.md](PLUGIN_DEVELOPMENT.md#publishing-to-plugin-community) for the full guide on publishing plugins to the community.

Quick summary:
1. Add `serialrun-plugin` topic to your GitHub repo
2. Ensure `plugin.json` is in the repo root
3. Create a GitHub Release with platform-specific ZIP files
4. Your plugin will be discoverable in SerialRUN's Community tab

## Example: STC ISP Plugin

See `plugins/serialrun-stc-isp/` for a complete example implementing:
- Serial port communication via callbacks
- Progress reporting
- Custom commands (flash, detect)
- Intel HEX file parsing
- STC ISP protocol implementation
