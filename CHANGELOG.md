# Changelog

## v0.2.0 - 2026-05-31

### Plugin System Enhancement (Phase 1)

#### New Features
- **Plugin Capabilities**: Plugins can now declare capabilities (SerialPort, UiPanel, FileDialog, Progress, Logging) via `plugin_get_capabilities()` FFI function
- **Host Callbacks**: Plugins receive a `PluginCallbacks` struct with access to serial port, file dialogs, progress reporting, and logging
- **Plugin Lifecycle**: New `plugin_init(callbacks)` and `plugin_cleanup()` optional FFI functions
- **PluginManager::init_all()**: Initialize all discovered plugins with host callbacks
- **Plugin Capabilities Display**: Plugin manager UI shows each plugin's capabilities as tags
- **Enable/Disable Toggle**: Plugins can be enabled or disabled from the UI

#### Bug Fixes (Phase 1 Review)
- **Critical**: `init()` was never called after plugin discovery — added `PluginManager::init_all()`
- **Critical**: `LoadedPlugin` had no `Drop` impl — cleanup was never invoked on removal
- **Critical**: `static mut CALLBACKS` in example plugin was UB — replaced with `OnceLock<Mutex>`
- **Critical**: `execute_command` ran on uninitialized plugins — now checks `is_initialized`
- **Safety**: `PluginCallbacks` function pointers now use `extern "C"` ABI (was Rust ABI)
- **Safety**: `PluginCallbacks` derives `Clone+Copy` for safe FFI transfer
- **Safety**: Added `free_string` callback for memory management of returned strings
- **Compatibility**: Added `PluginCapability::Unknown` variant for forward compatibility
- **Logging**: Capability parsing errors now logged instead of silently swallowed
- **Type Safety**: `progress_set_status` uses `PluginStatus` type instead of `c_int`

#### Plugin API Changes
- **Version**: `0.1.0` → `0.2.0`
- **New Types**: `PluginCapability`, `PluginStatus`, `PluginCallbacks`
- **New Functions**: `parse_capabilities()`, `serialize_capabilities()`
- **New FFI**: `plugin_get_capabilities`, `plugin_init`, `plugin_cleanup` (all optional)
- **Backward Compatible**: Existing 4 FFI functions unchanged

#### Files Changed
| File | Change |
|------|--------|
| `crates/serialrun-plugin-api/src/lib.rs` | New types, version bump, Copy derive |
| `crates/serialrun-core/src/plugin.rs` | Capabilities detection, init/cleanup, Drop impl |
| `crates/serialrun-gui/src/plugin_callbacks.rs` | Host callback adapter (new file) |
| `crates/serialrun-gui/src/ui/plugin.rs` | Capabilities display, enable/disable |
| `crates/serialrun-gui/src/state.rs` | PluginInfo: add capabilities, enabled fields |
| `plugins/serialrun-example-plugin/src/lib.rs` | New FFI functions, OnceLock fix |

---

## v0.1.0 - 2026-05-31

### Initial Release

- Serial communication (HEX/TEXT, timestamps, CRC, auto-send, DTR/RTS)
- Modbus RTU/TCP debugging (8 function codes, register monitor)
- PLC control (Siemens, Mitsubishi, Delta, Omron presets)
- TCP/RTU bridge and HMI simulator
- CAN bus analysis (SLCAN)
- I2C/SPI debug
- Firmware flash (STM32 ISP, ESP32 serial)
- MCP server with 15 tools for AI integration
- Multi-window interface (independent OS windows)
- Data persistence (config, logs, terminal history)
- Bilingual UI (English/Chinese)
- Cross-platform (Windows, macOS, Linux)
