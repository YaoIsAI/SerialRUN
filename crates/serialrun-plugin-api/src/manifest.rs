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
}

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
