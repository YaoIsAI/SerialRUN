use serialrun_plugin_api::PluginInfo as ApiPluginInfo;
use std::ffi::{c_char, CStr, CString};

#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char {
    let info = ApiPluginInfo {
        name: "Example Plugin".to_string(),
        version: "0.1.0".to_string(),
        description: "An example plugin for SerialRUN".to_string(),
        author: "SerialRUN Team".to_string(),
    };

    let json = serde_json::to_string(&info).unwrap();
    CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_get_commands() -> *mut c_char {
    let commands = vec![
        serialrun_plugin_api::PluginCommand {
            name: "echo".to_string(),
            description: "Echo back the input data".to_string(),
            parameters: vec![serialrun_plugin_api::PluginParameter {
                name: "data".to_string(),
                description: "Data to echo".to_string(),
                required: true,
                param_type: "string".to_string(),
            }],
        },
        serialrun_plugin_api::PluginCommand {
            name: "timestamp".to_string(),
            description: "Get current timestamp".to_string(),
            parameters: vec![],
        },
        serialrun_plugin_api::PluginCommand {
            name: "add".to_string(),
            description: "Add two numbers".to_string(),
            parameters: vec![
                serialrun_plugin_api::PluginParameter {
                    name: "a".to_string(),
                    description: "First number".to_string(),
                    required: true,
                    param_type: "number".to_string(),
                },
                serialrun_plugin_api::PluginParameter {
                    name: "b".to_string(),
                    description: "Second number".to_string(),
                    required: true,
                    param_type: "number".to_string(),
                },
            ],
        },
    ];

    let json = serde_json::to_string(&commands).unwrap();
    CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_execute(command: *const c_char, params: *const c_char) -> *mut c_char {
    let command = unsafe {
        if command.is_null() {
            return CString::new(r#"{"success":false,"error":"Null command"}"#).unwrap().into_raw();
        }
        CStr::from_ptr(command).to_string_lossy().to_string()
    };

    let params = unsafe {
        if params.is_null() {
            "{}".to_string()
        } else {
            CStr::from_ptr(params).to_string_lossy().to_string()
        }
    };

    let result = match command.as_str() {
        "echo" => {
            let parsed: serde_json::Value = serde_json::from_str(&params).unwrap_or_default();
            let data = parsed["data"].as_str().unwrap_or("No data");
            serialrun_plugin_api::PluginResult::success(serde_json::json!(data))
        }
        "timestamp" => {
            let now = chrono::Utc::now();
            serialrun_plugin_api::PluginResult::success(serde_json::json!({
                "timestamp": now.to_rfc3339()
            }))
        }
        "add" => {
            let parsed: serde_json::Value = serde_json::from_str(&params).unwrap_or_default();
            let a = parsed["a"].as_f64().unwrap_or(0.0);
            let b = parsed["b"].as_f64().unwrap_or(0.0);
            serialrun_plugin_api::PluginResult::success(serde_json::json!(a + b))
        }
        _ => {
            serialrun_plugin_api::PluginResult::error(format!("Unknown command: {}", command))
        }
    };

    let json = serde_json::to_string(&result).unwrap();
    CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_info() {
        let ptr = plugin_get_info();
        assert!(!ptr.is_null());

        let info_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let info: ApiPluginInfo = serde_json::from_str(&info_str).unwrap();

        assert_eq!(info.name, "Example Plugin");
        assert_eq!(info.version, "0.1.0");

        plugin_free_string(ptr);
    }

    #[test]
    fn test_plugin_commands() {
        let ptr = plugin_get_commands();
        assert!(!ptr.is_null());

        let commands_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let commands: Vec<serialrun_plugin_api::PluginCommand> =
            serde_json::from_str(&commands_str).unwrap();

        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0].name, "echo");
        assert_eq!(commands[2].name, "add");

        plugin_free_string(ptr);
    }

    #[test]
    fn test_plugin_execute_echo() {
        let command = CString::new("echo").unwrap();
        let params = CString::new(r#"{"data": "hello"}"#).unwrap();

        let ptr = plugin_execute(command.as_ptr(), params.as_ptr());
        assert!(!ptr.is_null());

        let result_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let result: serialrun_plugin_api::PluginResult =
            serde_json::from_str(&result_str).unwrap();

        assert!(result.success);
        assert_eq!(result.result.unwrap(), serde_json::json!("hello"));

        plugin_free_string(ptr);
    }

    #[test]
    fn test_plugin_execute_add() {
        let command = CString::new("add").unwrap();
        let params = CString::new(r#"{"a": 3.0, "b": 4.0}"#).unwrap();

        let ptr = plugin_execute(command.as_ptr(), params.as_ptr());
        assert!(!ptr.is_null());

        let result_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let result: serialrun_plugin_api::PluginResult =
            serde_json::from_str(&result_str).unwrap();

        assert!(result.success);
        assert_eq!(result.result.unwrap(), serde_json::json!(7.0));

        plugin_free_string(ptr);
    }

    #[test]
    fn test_plugin_execute_unknown() {
        let command = CString::new("unknown").unwrap();
        let params = CString::new("{}").unwrap();

        let ptr = plugin_execute(command.as_ptr(), params.as_ptr());
        assert!(!ptr.is_null());

        let result_str = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
        let result: serialrun_plugin_api::PluginResult =
            serde_json::from_str(&result_str).unwrap();

        assert!(!result.success);

        plugin_free_string(ptr);
    }
}
