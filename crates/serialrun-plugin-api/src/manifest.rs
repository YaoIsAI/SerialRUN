/// Plugin manifest format (plugin.json)
///
/// Each plugin zip contains a plugin.json with metadata.
/// This allows the plugin manager to display info without loading the binary.

use serde::{Deserialize, Serialize};

/// Plugin manifest stored in plugin.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin name (unique identifier)
    pub name: String,
    /// Semantic version
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Author name
    pub author: String,
    /// Plugin license
    #[serde(default = "default_license")]
    pub license: String,
    /// Minimum SerialRUN version required
    #[serde(default = "default_min_version")]
    pub min_serialrun_version: String,
    /// Supported platforms
    #[serde(default = "default_platforms")]
    pub platforms: Vec<String>,
    /// Plugin category
    #[serde(default)]
    pub category: String,
    /// Tags for search
    #[serde(default)]
    pub tags: Vec<String>,
    /// Homepage URL
    #[serde(default)]
    pub homepage: String,
    /// Repository URL
    #[serde(default)]
    pub repository: String,
    /// Usage instructions (markdown)
    #[serde(default)]
    pub usage: String,
    /// Toolbar integration config — if present, plugin appears in the main toolbar
    #[serde(default)]
    pub toolbar: Option<ToolbarConfig>,
    /// Window config — if present, plugin opens as a standalone floating window
    #[serde(default)]
    pub window: Option<WindowConfig>,
}

/// Toolbar button configuration — declares how the plugin appears in the main toolbar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolbarConfig {
    /// Icon emoji or text for the toolbar button
    pub icon: String,
    /// Label text next to the icon
    pub label: String,
    /// Tooltip text on hover
    #[serde(default)]
    pub tooltip: String,
    /// Position in toolbar: "left", "center", "right", "plugins"
    #[serde(default = "default_toolbar_position")]
    pub position: String,
}

fn default_toolbar_position() -> String {
    "plugins".to_string()
}

/// Standalone window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window title
    pub title: String,
    /// Default width in pixels
    #[serde(default = "default_window_width")]
    pub default_width: f32,
    /// Default height in pixels
    #[serde(default = "default_window_height")]
    pub default_height: f32,
    /// Whether the window is resizable
    #[serde(default = "default_true")]
    pub resizable: bool,
    /// Minimum width
    #[serde(default)]
    pub min_width: Option<f32>,
    /// Minimum height
    #[serde(default)]
    pub min_height: Option<f32>,
}

fn default_window_width() -> f32 { 800.0 }
fn default_window_height() -> f32 { 600.0 }
fn default_true() -> bool { true }

fn default_license() -> String {
    "BSL-1.1".to_string()
}

fn default_min_version() -> String {
    "0.1.0".to_string()
}

fn default_platforms() -> Vec<String> {
    vec![
        "windows-x64".to_string(),
        "macos-arm64".to_string(),
        "linux-x64".to_string(),
    ]
}

impl PluginManifest {
    /// Create a new manifest from plugin info
    pub fn from_info(
        name: String,
        version: String,
        description: String,
        author: String,
    ) -> Self {
        Self {
            name,
            version,
            description,
            author,
            license: default_license(),
            min_serialrun_version: default_min_version(),
            platforms: default_platforms(),
            category: String::new(),
            tags: Vec::new(),
            homepage: String::new(),
            repository: String::new(),
            usage: String::new(),
            toolbar: None,
            window: None,
        }
    }

    /// Serialize manifest to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Parse manifest from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Get the current platform string
pub fn current_platform() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "windows-x64",
        ("macos", "aarch64") => "macos-arm64",
        ("macos", "x86_64") => "macos-x64",
        ("linux", "x86_64") => "linux-x64",
        ("linux", "aarch64") => "linux-arm64",
        _ => "unknown",
    }
}

/// Check if a manifest supports the current platform
pub fn supports_current_platform(manifest: &PluginManifest) -> bool {
    manifest.platforms.is_empty() || manifest.platforms.contains(&current_platform().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_roundtrip() {
        let manifest = PluginManifest::from_info(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            "A test plugin".to_string(),
            "Author".to_string(),
        );
        let json = manifest.to_json().unwrap();
        let parsed = PluginManifest::from_json(&json).unwrap();
        assert_eq!(parsed.name, "test-plugin");
        assert_eq!(parsed.version, "1.0.0");
    }

    #[test]
    fn test_current_platform() {
        let platform = current_platform();
        assert!(!platform.is_empty());
        assert!(!platform.contains("unknown"));
    }
}
