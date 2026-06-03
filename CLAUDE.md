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
taskkill //F //IM serialrun.exe  # Windows: close running app
cargo build --release -p serialrun-gui
./target/release/serialrun.exe   # Launch for testing
```

### Git Remotes
- **GitHub:** `https://github.com/YaoIsAI/SerialRUN.git`
- **Gitea (local):** `http://192.168.31.85:38633/yao/serialrun.git`
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
| `docs/PLUGIN_DEVELOPMENT.md` | Plugin development handbook |

### Known Bug Patterns
- `repo_name` in RegistryPlugin must be plugin name (not repo name)
- Local plugins have `"local/"` prefix — strip before matching
- After community install, add to `community_installed` HashSet for immediate UI update
- Installed tab: check for duplicate rendering of capabilities/actions blocks

### Mac Migration
1. Clone from GitHub: `git clone https://github.com/YaoIsAI/SerialRUN.git`
2. Copy GUI crate separately (proprietary, not in git)
3. Install Rust toolchain: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
4. Build: `cargo build --release -p serialrun-gui`
5. Plugin install dir: `~/.serialrun/plugins/`
