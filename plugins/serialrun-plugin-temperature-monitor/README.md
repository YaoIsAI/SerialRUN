# SerialRUN Example Plugin

This is the **recommended template** for creating new SerialRUN plugins.

It demonstrates ALL plugin API features:

| Feature | Command | Description |
|---------|---------|-------------|
| Basic params | `echo` | Parse string parameters |
| Serial port | `serial_send` | Send/receive data via host callbacks |
| Number params | `add` | Parse number parameters |
| Progress | `demo_progress` | Show progress bar in host UI |
| File dialog | `open_file` | Open file dialog and read file |
| Config storage | `get_setting` / `set_setting` | Persistent key-value storage |
| UI layout | `plugin_get_ui_layout` | Declarative UI with splits and panels |
| Logging | `plugin_init` | Log messages via host |
| Capabilities | `plugin_get_capabilities` | Declare required host features |

## How to Use as Template

```bash
# 1. Copy the directory
cp -r plugins/serialrun-example-plugin plugins/my-plugin

# 2. Rename in Cargo.toml
# Change: name = "serialrun-example-plugin" → name = "my-plugin"

# 3. Rename in plugin.json
# Change: "name": "serialrun-example-plugin" → "name": "my-plugin"

# 4. Modify src/lib.rs
# - Change plugin_get_info() to return your plugin name
# - Modify plugin_get_commands() to define your commands
# - Implement command logic in plugin_execute()

# 5. Add to workspace Cargo.toml
# Add "plugins/my-plugin" to the members list

# 6. Build
cargo build --release -p my-plugin

# 7. Install
mkdir -p ~/.serialrun/plugins/my-plugin
cp target/release/libmy_plugin.dylib ~/.serialrun/plugins/my-plugin/  # macOS
cp plugin.json ~/.serialrun/plugins/my-plugin/

# 8. Validate
serialrun plugin validate ~/.serialrun/plugins/my-plugin

# 9. Restart SerialRUN — your plugin appears in Plug menu
```

## FFI Functions Checklist

- [ ] `plugin_get_info()` — Return plugin name/version/description/author
- [ ] `plugin_get_commands()` — Return list of commands with parameters
- [ ] `plugin_execute(cmd, params)` — Execute command, return PluginResult
- [ ] `plugin_free_string(s)` — Free allocated strings
- [ ] `plugin_get_capabilities()` — Declare host features you need (optional)
- [ ] `plugin_init(callbacks)` — Store host callbacks (optional)
- [ ] `plugin_cleanup()` — Release resources (optional)
- [ ] `plugin_get_ui_layout()` — Return UI layout JSON (optional)

## Thread Safety

All plugins MUST use `OnceLock<Mutex<Option<PluginCallbacks>>>` for callback storage:

```rust
static CALLBACKS: OnceLock<Mutex<Option<PluginCallbacks>>> = OnceLock::new();

fn get_callbacks() -> Option<PluginCallbacks> {
    CALLBACKS.get()?.lock().ok()?.clone()
}
```

## Build Output

| Platform | Binary Name | Location |
|----------|-------------|----------|
| macOS | `lib<name>.dylib` | `target/release/` |
| Windows | `<name>.dll` | `target/release/` |
| Linux | `lib<name>.so` | `target/release/` |
