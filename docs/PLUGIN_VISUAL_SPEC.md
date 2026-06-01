# SerialRUN Plugin Panel Visual Specification

## Plugin Card Layout

```
┌─────────────────────────────────────────────────┐
│ ● Plugin Name           v1.0.0  by Author  ?  ▼ │
│   Serial  UI  Progress                          │
│   ☐ Enabled                          [Uninstall]│
├─────────────────────────────────────────────────┤
│ (expanded content when ▼ is clicked)            │
│ Usage documentation...                          │
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

1. **Usage Documentation** - Full text from plugin.json `usage` field, scrollable (max 120px)
2. **Separator line**
3. **Commands Panel** (if plugin has commands):
   - Command selector (ComboBox)
   - Command description
   - JSON parameter editor
   - Run button
   - Result display

### Special Panels

Some plugins have custom panels instead of the generic command panel:
- `serialrun-stc-isp` → Dedicated STC ISP Flasher panel

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

### ASCII Spinner Animation

Characters: ⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
Color: #FFC800 (amber)
Speed: 8 frames/second

## Panel Header

```
Plugins              [Import ZIP]
─────────────────────────────────
```

## Empty State

```
No plugins installed. Click Import ZIP to install.
```

## Plugin List Scroll

- Max height: 400px
- Scrollable when content exceeds height
- Stick to bottom not needed (static list)

## Sub-Window Behavior

- All sub-windows are in-app `egui::Window` (not OS-level viewports)
- Sub-windows never hide when main window gains focus
- Sub-windows have close button (X) in title bar
- Sub-windows are resizable and collapsible
- Sub-windows open at center of main window
