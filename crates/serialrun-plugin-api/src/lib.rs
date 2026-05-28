/// SerialRUN Plugin API - shared types for plugin development.

use serde::{Deserialize, Serialize};

/// Current plugin API version.
pub const PLUGIN_API_VERSION: &str = "0.1.0";

/// Information about a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

/// A command exposed by a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub parameters: Vec<PluginParameter>,
}

/// A parameter for a plugin command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub param_type: String,
}

/// Result of executing a plugin command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl PluginResult {
    pub fn success(result: serde_json::Value) -> Self {
        Self {
            success: true,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            result: None,
            error: Some(error.into()),
        }
    }
}

/// Parse plugin info from a JSON string.
pub fn parse_plugin_info(json: &str) -> Result<PluginInfo, serde_json::Error> {
    serde_json::from_str(json)
}

/// Parse plugin commands from a JSON string.
pub fn parse_plugin_commands(json: &str) -> Result<Vec<PluginCommand>, serde_json::Error> {
    serde_json::from_str(json)
}

/// Parse a plugin result from a JSON string.
pub fn parse_plugin_result(json: &str) -> Result<PluginResult, serde_json::Error> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_api_version() {
        assert_eq!(PLUGIN_API_VERSION, "0.1.0");
    }

    #[test]
    fn test_plugin_info_serde() {
        let info = PluginInfo {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        let parsed: PluginInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.version, "1.0.0");
    }

    #[test]
    fn test_plugin_command_serde() {
        let cmd = PluginCommand {
            name: "echo".to_string(),
            description: "Echo input".to_string(),
            parameters: vec![PluginParameter {
                name: "data".to_string(),
                description: "Data to echo".to_string(),
                required: true,
                param_type: "string".to_string(),
            }],
        };

        let json = serde_json::to_string(&cmd).unwrap();
        let parsed: PluginCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "echo");
        assert_eq!(parsed.parameters.len(), 1);
        assert!(parsed.parameters[0].required);
    }

    #[test]
    fn test_plugin_result_success() {
        let result = PluginResult::success(serde_json::json!({"value": 42}));
        assert!(result.success);
        assert!(result.result.is_some());
        assert!(result.error.is_none());

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("42"));
    }

    #[test]
    fn test_plugin_result_error() {
        let result = PluginResult::error("Something went wrong");
        assert!(!result.success);
        assert!(result.result.is_none());
        assert!(result.error.is_some());

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Something went wrong"));
    }

    #[test]
    fn test_parse_plugin_info() {
        let json = r#"{"name":"test","version":"1.0","description":"desc","author":"auth"}"#;
        let info = parse_plugin_info(json).unwrap();
        assert_eq!(info.name, "test");
    }

    #[test]
    fn test_parse_plugin_commands() {
        let json = r#"[{"name":"cmd","description":"desc","parameters":[]}]"#;
        let cmds = parse_plugin_commands(json).unwrap();
        assert_eq!(cmds.len(), 1);
    }

    #[test]
    fn test_parse_plugin_result() {
        let json = r#"{"success":true,"result":null}"#;
        let result = parse_plugin_result(json).unwrap();
        assert!(result.success);
    }
}
