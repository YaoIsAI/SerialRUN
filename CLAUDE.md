# SerialRUN — Agent Operation Guide

This document provides instructions for Claude Code agents to operate the SerialRUN serial port assistant.

## Quick Commands

### List Ports

```bash
serialrun list                    # Text format
serialrun list --format json      # JSON format
```

### Connect

```bash
serialrun connect /dev/ttyUSB0 -b 115200
serialrun connect COM1 -b 9600 -d 7 -s 2 -p odd -f hardware
```

### Send Data

```bash
serialrun send COM1 "Hello\r\n"               # Text
serialrun send COM1 "48 65 6C 6C 6F" --hex    # HEX
```

### Monitor

```bash
serialrun monitor COM1 -t                  # With timestamps
serialrun monitor COM1 -x                  # HEX mode
serialrun monitor COM1 -t -l output.log    # With logging
```

### Scripts

```bash
serialrun record COM1 -o script.txt    # Record
serialrun replay COM1 script.txt       # Replay
```

## Agent Mode (JSON Output)

### List Ports

```bash
serialrun agent list-ports
```

Output:

```json
{
  "success": true,
  "ports": [
    {
      "name": "/dev/ttyUSB0",
      "description": "USB Device 0403:6001",
      "manufacturer": "FTDI",
      "vid": 1027,
      "pid": 24577
    }
  ]
}
```

### Send Data

```bash
serialrun agent COM1 send "Hello" -b 115200
```

Output:

```json
{ "success": true, "bytes_written": 5 }
```

### Read Data

```bash
serialrun agent COM1 read --timeout 1000 --max-bytes 1024
```

Output:

```json
{
  "success": true,
  "bytes_read": 10,
  "data_hex": "48656C6C6F20576F726C64",
  "data_text": "Hello World"
}
```

### Run Script

```bash
serialrun agent COM1 run-script script.txt
```

## Common Workflows

### ESP8266/ESP32 AT Command Testing

```bash
serialrun connect COM3 -b 115200
# Then in interactive mode:
> AT
> AT+RST
> AT+CWMODE=1
> AT+CWJAP="WiFi","password"
```

### Modbus Traffic Capture

```bash
serialrun monitor /dev/ttyUSB0 -x -t -l modbus.log
serialrun send /dev/ttyUSB0 "01 03 00 00 00 0A C5 CD" --hex
```

### Automated Testing

```bash
serialrun record COM1 -o test.txt
serialrun replay COM1 test.txt
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Port not found | `serialrun list` to check |
| Permission denied | `sudo usermod -a -G dialout $USER` (Linux) |
| Connection failed | Verify baud rate matches device |
| No data received | Check cable and flow control |

---

## Project Context (for cross-machine continuity)

### Architecture
- **Core crates** (`serialrun-core`, `serialrun-plugin-api`) — open source, in git
- **GUI crate** (`serialrun-gui`) — **proprietary, NOT in git**, must be copied separately
- **Plugins** (`plugins/`) — in git, each is independent cdylib crate

### Plugin System (v0.3.0)
- Plugins only depend on `serialrun-plugin-api`, never on gui/core
- Community plugins hosted in `YaoIsAI/serialrun-plugins` GitHub repo
- Community search reads `plugins/*/plugin.json` from that repo via GitHub Contents API
- Install downloads ZIP from Releases → extracts to `~/.serialrun/plugins/`
- Key constant in `plugin_registry.rs`: `PLUGINS_REPO = "YaoIsAI/serialrun-plugins"`

### Build & Test Flow
```bash
# Windows (platform-separated output)
taskkill //F //IM serialrun.exe  # Close running app
cargo build --target x86_64-pc-windows-msvc --release -p serialrun-gui
./target/x86_64-pc-windows-msvc/release/serialrun.exe   # Launch for testing

# macOS (use Makefile)
make app       # Build .app bundle
make install   # Install to /Applications
```

### Git Remotes
- **GitHub:** `https://github.com/YaoIsAI/SerialRUN.git`
- **Plugins repo:** `https://github.com/YaoIsAI/serialrun-plugins.git`

### Key Files
| File | Purpose |
|------|---------|
| `crates/serialrun-plugin-api/src/lib.rs` | Plugin FFI types, callbacks, capabilities |
| `crates/serialrun-plugin-api/src/manifest.rs` | plugin.json parser, ToolbarConfig, WindowConfig |
| `crates/serialrun-core/src/plugin.rs` | LoadedPlugin, DLL loading via libloading |
| `crates/serialrun-core/src/plugin_install.rs` | PluginManager, install/uninstall/enable/disable |
| `crates/serialrun-core/src/plugin_registry.rs` | GitHub community search and download |
| `plugins/serialrun-mpy-ide/` | MicroPython IDE plugin (REPL, editor, file browser) |
| `plugins/serialrun-example-plugin/` | **Template plugin** — copy this to start new plugin |
| `docs/PLUGIN_DEVELOPMENT.md` | Full plugin development handbook |
| `docs/PLUGIN_CHEATSHEET.md` | Quick reference for agents and developers |
| `docs/PLUGIN_SCHEMA.json` | Machine-readable plugin.json schema |

### Plugin Development (Agent Guide)

**To create a new plugin:**
1. Copy `plugins/serialrun-example-plugin/` as template
2. Edit `plugin.json` (name, version, description, author)
3. Edit `src/lib.rs` — implement commands in `plugin_execute()`
4. Add to workspace `Cargo.toml` members list
5. Build & install: `serialrun plugin build plugins/<name>`
6. Test: `serialrun plugin test plugins/<name>`
7. Package: `serialrun plugin package plugins/<name>`
8. Validate: `serialrun plugin validate ~/.serialrun/plugins/<name>/`

**FFI contract (4 required + 4 optional functions):**
- Required: `plugin_get_info`, `plugin_get_commands`, `plugin_execute`, `plugin_free_string`
- Optional: `plugin_get_capabilities`, `plugin_init`, `plugin_cleanup`, `plugin_get_ui_layout`
- All return `*mut c_char` containing JSON strings
- Thread safety: use `OnceLock<Mutex<Option<PluginCallbacks>>>` for callbacks

**Plugin CLI commands:**
```bash
serialrun plugin validate <dir>   # Check plugin.json format
serialrun plugin info <dir>       # Show plugin details
serialrun plugin list             # List installed plugins
serialrun plugin build <dir>      # Build + install to local plugins dir
serialrun plugin test <dir>       # Run plugin unit tests
serialrun plugin package <dir>    # Package into distributable ZIP
```

### Known Bug Patterns
- `repo_name` in RegistryPlugin must be plugin name (not repo name)
- Local plugins have `"local/"` prefix — strip before matching
- After community install, add to `community_installed` HashSet for immediate UI update
- Installed tab: check for duplicate rendering of capabilities/actions blocks

### Mac Migration
1. Clone from GitHub: `git clone https://github.com/YaoIsAI/SerialRUN.git`
2. Copy GUI crate separately (proprietary, not in git)
3. Install Rust toolchain: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
4. Build: `make app` (macOS) or `cargo build --target x86_64-pc-windows-msvc --release -p serialrun-gui` (Windows)
5. Plugin install dir: `~/.serialrun/plugins/`

### Website Deployment
- **Location:** `website/` directory (in `.gitignore`, not in git)
- **Platform:** Cloudflare Pages
- **Deploy:** `npx wrangler pages deploy website --project-name=serialrun`
- **URLs:** `https://serialrun.com` (custom domain), `https://serialrun.pages.dev`
- **Production branch:** must be set to `master`
- **Structure:** index.html (landing), guide.html, license.html, plugins.html, downloads.html
- **i18n:** `i18n.js` for Chinese/English translations
- **WeChat images:** `website/assets/wechat/` (2 plans with 6 images each)

### Push Preferences
- **GitHub:** Public code, releases, community plugins repo
- **Private backup:** Separate internal repo (not on GitHub)
- **Sync script:** `./scripts/sync-remotes.sh` handles multi-remote sync
