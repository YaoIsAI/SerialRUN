# Changelog

## v0.2.4 - 2026-06-01

### CAN Bus Analyzer & Plugin System Overhaul

#### CAN Bus Panel
- **Independent Connection**: CAN analyzer uses its own serial port (separate from terminal)
- **Port/Baud Selection**: Independent CAN port and baud rate (100K-1000K)
- **Connect/Disconnect**: Clear connection lifecycle with status indicators
- **Periodic Send**: Configurable count + period with ID/data auto-increment
- **Frame Table**: USB-CAN Tool style table with index, time, channel, direction, ID, type, DLC, data
- **TX/RX Direction**: Color-coded direction dots (green=RX, yellow=TX, red=ERR)
- **Port Conflict Warning**: Alerts when CAN port is shared with terminal
- **Tooltip System**: Hover labels on Frame ID and Data fields for guidance
- **Bus Load Fix**: Corrected bus load calculation formula
- **Line Parsing**: Supports both `\r` and `\n` line terminators
- **Async Import**: Plugin import runs in background thread with ASCII spinner

#### Plugin Panel
- **Auto-Discover**: Plugins discovered on startup (no manual refresh needed)
- **ZIP Import (Rust Native)**: Replaced PowerShell/tar/unzip with Rust `zip` crate (cross-platform)
- **Help Icon (?)**: Per-card usage documentation on hover
- **Expand/Collapse**: Toggle card to show/hide commands panel
- **i18n Support**: All buttons and labels follow language switching
- **Usage Documentation**: Moved to plugin.json `usage` field

#### STC ISP Flasher
- **Left-Right Layout**: Config on left, info/actions/log on right
- **Full i18n**: All labels and buttons support Chinese/English

#### Sub-Windows
- **Always Visible**: Sub-windows never hide when main window gains focus
- **Always On Top**: Sub-windows stay above main window
- **Position Stable**: Sub-windows don't reset position on re-render

#### Bug Fixes
- Fixed periodic send never starting (`can_tx_periodic` never set to true)
- Fixed `can_connected` set before actual connection confirmed
- Fixed bus load calculation overcounting by factor of N
- Fixed disconnect not always resetting periodic send
- Fixed phantom TX frames added when channel dead
- Fixed CAN port/baud rate not persisted across sessions
- Fixed help file not found (now reads from docs/ directory)
- Fixed cross-filesystem rename failure in plugin install

#### Files Changed
| File | Description |
|------|-------------|
| `crates/serialrun-core/Cargo.toml` | Added `zip` crate dependency |
| `crates/serialrun-core/src/plugin_install.rs` | Rust native zip extraction, cross-filesystem copy fallback |
| `docs/PLUGIN_DEVELOPMENT.md` | Plugin development guide |
| `docs/PLUGIN_VISUAL_SPEC.md` | Plugin panel visual specification |

---

## v0.2.3 - 2026-06-01

### Plugin System Enhancement

#### New Features
- **Plugin Development Specification**: Complete spec document for plugin authors
- **STC ISP Dedicated Panel**: Professional UI with chip detection, firmware selection, flash/erase buttons
- **Plugin Commands Cached**: Commands stored in state at load time (no re-loading)
- **Import Animation**: ASCII spinner during plugin import
- **Plugin Card UI**: Each plugin displayed as a card with capabilities, enable/disable, Open/Uninstall buttons
- **Command Execution UI**: Generic command panel for plugins without dedicated UI

#### Files Added
| File | Description |
|------|-------------|
| `docs/PLUGIN_SPEC.md` | Plugin development specification |
| `crates/serialrun-gui/src/ui/stc_panel.rs` | STC ISP Flasher dedicated panel |

---

## v0.2.2 - 2026-05-31

### Plugin Management (Phase 3A)

#### New Features
- **Plugin Manifest**: `plugin.json` format with metadata (name, version, author, platforms, tags)
- **Plugin Manager**: Install, uninstall, enable, disable plugins
- **Zip Import**: Install plugins from zip files via system unzip/tar
- **Plugin State**: Persisted to `~/.serialrun/plugin_state.json`
- **Platform Checking**: Auto-detect platform compatibility
- **Auto-Discover**: Scan plugins directory for installed plugins
- **GUI Integration**: Import ZIP button, Uninstall button, enable/disable toggle

#### Files Added
| File | Description |
|------|-------------|
| `crates/serialrun-plugin-api/src/manifest.rs` | Plugin manifest format |
| `crates/serialrun-core/src/plugin_install.rs` | Plugin installation manager |

---

## v0.2.1 - 2026-05-31

### STC ISP Flashing Plugin (Phase 2)

#### New Features
- **STC ISP Plugin**: Flash STC series MCUs via ISP protocol
- **Supported Chips**: STC89, STC12, STC15, STC8, STC8G, STC8H
- **Firmware Formats**: Intel HEX and raw binary
- **ISP Protocol**: Handshake, erase, write (128-byte blocks), verify (CRC-16), reset
- **Plugin Capabilities**: SerialPort, Progress, Logging

#### Bug Fixes (Phase 2 Review)
- **Critical**: Binary firmware files now load correctly (was always reading as UTF-8)
- **Critical**: HEX record checksums now validated
- **High**: HEX parser bounds check prevents panic on malformed byte_count
- **Medium**: All error paths in handle_flash now set Status::Error
- **Medium**: Empty firmware rejected before erase/flash sequence
- **Medium**: Erase range capped to chip flash size
- **Low**: `assert!` → `debug_assert!` in write_packet (UB fix for cdylib)
- **Low**: `CString::new().unwrap()` → `if let Ok()` (panic prevention)
- **Low**: Added `log_error` helper for error logging

#### Files Added
| File | Description |
|------|-------------|
| `plugins/serialrun-stc-isp/Cargo.toml` | Plugin dependencies |
| `plugins/serialrun-stc-isp/src/lib.rs` | Plugin entry + FFI + command handlers |
| `plugins/serialrun-stc-isp/src/protocol.rs` | STC ISP protocol implementation |
| `plugins/serialrun-stc-isp/src/chip.rs` | Chip identification + HEX file parser |

---

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
