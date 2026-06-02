# SerialRUN Plugin Panel Visual Specification

## Tab Layout

```
[Installed] [Community]           [Import ZIP]
─────────────────────────────────────────────
```

- Tab bar at top with selectable labels
- "Import ZIP" button only visible on Installed tab
- Community tab auto-loads popular plugins on first visit

## Installed Plugin Card

```
┌─────────────────────────────────────────────────┐
│ ● Plugin Name           v1.0.0  by Author  ?  ▼ │
│   Serial  UI  Progress                          │
│   ☐ Enabled                          [Uninstall]│
├─────────────────────────────────────────────────┤
│ (expanded content when ▼ is clicked)            │
│ Command panel...                                │
└─────────────────────────────────────────────────┘
```

### Card Components

| Element | Position | Style | Description |
|---------|----------|-------|-------------|
| Status dot | Left | ● green / ○ gray | Shows if plugin DLL is loaded |
| Name | Left | Bold | Plugin display name |
| Version | Left | Weak, small | v1.0.0 format |
| Author | Left | Weak, small | by Author format |
| Help (?) | Right | Blue, bold | Hover shows full usage documentation |
| Expand (▼/▲) | Right | Small button | Toggle expanded content |
| Capabilities | Below header | Colored tags | Serial=green, UI=blue, File=purple, Progress=amber |
| Enabled checkbox | Bottom left | Standard | Enable/disable plugin |
| Uninstall | Bottom right | Small button | Remove plugin |

### Capability Colors

| Capability | Label | Color |
|-----------|-------|-------|
| serial_port | Serial | #22C55E (green) |
| ui_panel | UI | #3B82F6 (blue) |
| file_dialog | File | #A855F7 (purple) |
| progress | Progress | #F59E0B (amber) |
| logging | Log | #6B7280 (gray) |

### Expanded Content

When the expand button (▼) is clicked:

1. **Separator line**
2. **Commands Panel** (if plugin has commands):
   - Command selector (ComboBox)
   - Command description
   - JSON parameter editor
   - Run button
   - Result display

### Special Panels

Some plugins have custom panels instead of the generic command panel:
- `serialrun-stc-isp` → Dedicated STC ISP Flasher panel

## Community Plugin Card

```
┌─────────────────────────────────────────────────┐
│ 📦 owner/repo            v1.0.0  by Author      │
│ Plugin description text here                     │
│ ★ 12  Serial  Progress  File                    │
│                              [Install]           │
└─────────────────────────────────────────────────┘
```

### Community Card Components

| Element | Position | Style | Description |
|---------|----------|-------|-------------|
| Package icon | Left | 📦 | Indicates community plugin |
| Repo name | Left | Bold | GitHub owner/repo |
| Version | Left | Weak, small | From plugin.json |
| Author | Left | Weak, small | By Author format |
| Description | Below header | Weak, small | Plugin description |
| Stars | Below | Amber | ★ count |
| Capabilities | Below | Colored tags | Same colors as installed |
| Install button | Bottom right | Button | One-click install |
| Installed label | Bottom right | Green text | Shown if already installed |
| Downloading | Bottom right | Weak text | Shown during download |

### Community Search Bar

```
Search: [________________] [Search]
```

- Text input with placeholder "Search plugins..."
- Search button triggers GitHub API query
- Enter key also triggers search
- Loading spinner shown during search

## Import Flow

```
[Import ZIP] clicked
  → File dialog opens (native OS)
  → User selects .zip file
  → Spinner animation starts: ⠋ Installing plugin...
  → Background thread extracts and installs
  → On success: plugin list refreshes automatically
  → On error: error message in log
```

## Community Install Flow

```
[Install] clicked
  → Download spinner: ⠋ Downloading: owner/repo
  → Background thread downloads release ZIP
  → Installs via PluginManager
  → On success: plugin list refreshes automatically
  → On error: error message in log
```

### ASCII Spinner Animation

Characters: ⠋ ⠙ ⠹ ⠸ ⠼ ⠦ ⠧ ⠇ ⠏
- Import color: #FFC800 (amber)
- Community color: #3B82F6 (blue)
- Speed: 8 frames/second

## Empty States

### Installed tab (no plugins)
```
No plugins installed. Import a ZIP or browse the Community tab.
```

### Community tab (no search yet)
```
Search for plugins or browse popular extensions.
```

### Community tab (no results)
```
No plugins found. Try a different search.
```

## Plugin List Scroll

- Max height: 400px
- Scrollable when content exceeds height
- Stick to bottom not needed (static list)

## Sub-Window Behavior

- All sub-windows are OS-level viewports (`show_viewport_immediate`)
- Sub-windows never hide when main window gains focus
- Sub-windows stay above main window (`WindowLevel::AlwaysOnTop`)
- Position stable: only set on first open (no position reset on re-render)
