use crate::error::{Error, Result};
use serde_json::Value;

/// Extract a required string argument from tool arguments.
pub fn required_string(args: &Value, name: &str) -> Result<String> {
    args.get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| Error::Tool(format!("missing required argument: {name}")))
}

/// Extract a required unsigned integer argument from tool arguments.
pub fn required_u64(args: &Value, name: &str) -> Result<u64> {
    args.get(name)
        .and_then(|v| v.as_u64())
        .ok_or_else(|| Error::Tool(format!("missing required argument: {name}")))
}

/// Extract an optional unsigned integer argument, returning a default if absent.
pub fn optional_u64(args: &Value, name: &str, default: u64) -> u64 {
    args.get(name).and_then(|v| v.as_u64()).unwrap_or(default)
}

/// Wrap a successful tool result in the MCP content format.
pub fn format_tool_result(data: &Value) -> Value {
    let text = serde_json::to_string_pretty(data).unwrap_or_else(|_| data.to_string());
    serde_json::json!({
        "content": [{
            "type": "text",
            "text": text
        }]
    })
}

/// Wrap an error message in the MCP content format with `isError` flag.
pub fn format_tool_error(msg: &str) -> Value {
    serde_json::json!({
        "content": [{
            "type": "text",
            "text": msg
        }],
        "isError": true
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn required_string_present() {
        let args = json!({"q": "revenue"});
        assert_eq!(required_string(&args, "q").unwrap(), "revenue");
    }

    #[test]
    fn required_string_missing() {
        let args = json!({});
        let err = required_string(&args, "q").unwrap_err();
        assert!(err.to_string().contains("missing required argument: q"));
    }

    #[test]
    fn required_string_wrong_type() {
        let args = json!({"q": 42});
        let err = required_string(&args, "q").unwrap_err();
        assert!(err.to_string().contains("missing required argument: q"));
    }

    #[test]
    fn required_u64_present() {
        let args = json!({"id": 5});
        assert_eq!(required_u64(&args, "id").unwrap(), 5);
    }

    #[test]
    fn required_u64_missing() {
        let args = json!({});
        let err = required_u64(&args, "id").unwrap_err();
        assert!(err.to_string().contains("missing required argument: id"));
    }

    #[test]
    fn required_u64_wrong_type() {
        let args = json!({"id": "abc"});
        let err = required_u64(&args, "id").unwrap_err();
        assert!(err.to_string().contains("missing required argument: id"));
    }

    #[test]
    fn optional_u64_present() {
        let args = json!({"page": 3});
        assert_eq!(optional_u64(&args, "page", 1), 3);
    }

    #[test]
    fn optional_u64_missing_returns_default() {
        let args = json!({});
        assert_eq!(optional_u64(&args, "page", 1), 1);
    }

    #[test]
    fn format_tool_result_structure() {
        let data = json!({"name": "test"});
        let result = format_tool_result(&data);
        let content = result["content"].as_array().unwrap();
        assert_eq!(content.len(), 1);
        assert_eq!(content[0]["type"], "text");
        assert!(content[0]["text"].as_str().unwrap().contains("test"));
        assert!(result.get("isError").is_none());
    }

    #[test]
    fn format_tool_error_structure() {
        let result = format_tool_error("something failed");
        let content = result["content"].as_array().unwrap();
        assert_eq!(content.len(), 1);
        assert_eq!(content[0]["type"], "text");
        assert_eq!(content[0]["text"], "something failed");
        assert_eq!(result["isError"], true);
    }
}
