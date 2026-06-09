# SerialRUN Plugin Development — Quick Reference

> For agents and developers. Full docs: [PLUGIN_DEVELOPMENT.md](PLUGIN_DEVELOPMENT.md)

## Architecture

```
Plugin (cdylib) ←→ Host (serialrun-gui)
     FFI (C ABI, JSON over strings)
```

- Plugin ONLY depends on `serialrun-plugin-api` crate
- Plugin does NOT depend on `serialrun-gui` or `serialrun-core`
- Binary naming: `lib<name>.dylib` (macOS), `<name>.dll` (Windows), `lib<name>.so` (Linux)

## Required FFI Functions (4)

```rust
// 1. Return plugin info as JSON
#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char;

// 2. Return command list as JSON
#[no_mangle]
pub extern "C" fn plugin_get_commands() -> *mut c_char;

// 3. Execute command, return PluginResult as JSON
#[no_mangle]
pub extern "C" fn plugin_execute(command: *const c_char, params: *const c_char) -> *mut c_char;

// 4. Free allocated strings
#[no_mangle]
pub extern "C" fn plugin_free_string(s: *mut c_char);
```

## Optional FFI Functions (4)

```rust
#[no_mangle] pub extern "C" fn plugin_get_capabilities() -> *mut c_char;  // Declare abilities
#[no_mangle] pub extern "C" fn plugin_init(callbacks: *const PluginCallbacks) -> bool;  // Init
#[no_mangle] pub extern "C" fn plugin_cleanup();  // Cleanup
#[no_mangle] pub extern "C" fn plugin_get_ui_layout() -> *mut c_char;  // UI layout JSON
```

## JSON Formats

### plugin_get_info → PluginInfo
```json
{"name": "my-plugin", "version": "0.1.0", "description": "...", "author": "..."}
```

### plugin_get_commands → Vec<PluginCommand>
```json
[{"name": "cmd", "description": "...", "parameters": [
  {"name": "param", "description": "...", "required": true, "param_type": "string"}
]}]
```

### plugin_execute → PluginResult
```json
{"success": true, "result": {...}}   // or
{"success": false, "error": "..."}
```

### plugin_get_capabilities → Vec<PluginCapability>
```json
["serial_port", "logging", "progress", "file_dialog", "ui_panel", "file_system", "ui_layout"]
```

## Thread Safety Pattern

```rust
use std::sync::{Mutex, OnceLock};
static CALLBACKS: OnceLock<Mutex<Option<PluginCallbacks>>> = OnceLock::new();

fn get_callbacks() -> Option<PluginCallbacks> {
    CALLBACKS.get()?.lock().ok()?.clone()
}
```

## Cargo.toml Template

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serialrun-plugin-api = { path = "../../crates/serialrun-plugin-api" }
serde = { workspace = true }
serde_json = { workspace = true }
```

## Build & Install

```bash
# Build
cargo build --release -p my-plugin

# Install (macOS)
mkdir -p ~/.serialrun/plugins/my-plugin
cp target/release/libmy_plugin.dylib ~/.serialrun/plugins/my-plugin/
cp plugin.json ~/.serialrun/plugins/my-plugin/

# Validate
serialrun plugin validate ~/.serialrun/plugins/my-plugin
serialrun plugin info ~/.serialrun/plugins/my-plugin
serialrun plugin list
```

## plugin.json Minimal

```json
{
  "name": "my-plugin",
  "version": "0.1.0",
  "description": "My plugin",
  "author": "Me"
}
```

## Capabilities

| Capability | Use When |
|------------|----------|
| `SerialPort` | Plugin reads/writes serial port |
| `UiPanel` | Plugin provides custom UI |
| `FileDialog` | Plugin needs open/save dialogs |
| `Progress` | Plugin shows progress bars |
| `Logging` | Plugin uses host log system |
| `FileSystem` | Plugin accesses device filesystem |
| `EventSubscription` | Plugin subscribes to serial events |
| `ConfigStorage` | Plugin stores persistent config |
| `UiLayout` | Plugin declares JSON-based UI |

## Platform Strings

`windows-x64`, `macos-arm64`, `macos-x64`, `linux-x64`, `linux-arm64`

## Lifecycle

```
Load DLL → plugin_get_info() → plugin_get_commands() → plugin_get_capabilities()
  → plugin_init(callbacks) → plugin_execute(...) [repeated] → plugin_cleanup() → Unload
```

## Reference Plugins

| Plugin | Path | Features |
|--------|------|----------|
| example | `plugins/serialrun-example-plugin/` | Template, 3 commands, logging |
| mpy-ide | `plugins/serialrun-mpy-ide/` | REPL, file browser, UI layout |
| stc-isp | `plugins/serialrun-stc-isp/` | Flash, detect, progress, protocol |
