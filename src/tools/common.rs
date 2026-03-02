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

/// Extract an optional string argument from tool arguments.
pub fn optional_string(args: &Value, name: &str) -> Option<String> {
    args.get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Extract an optional JSON value argument from tool arguments.
pub fn optional_json(args: &Value, name: &str) -> Option<Value> {
    args.get(name).cloned()
}

/// Truncate query result data to a maximum number of rows.
///
/// Extracts `query_result.data.rows` and `query_result.data.columns`,
/// slices rows to `max_rows`, and returns a flat structure with `_metadata`.
/// If the structure is unexpected, returns the original data unchanged.
pub fn truncate_query_result(data: &Value, max_rows: u64) -> Value {
    let columns = match data
        .get("query_result")
        .and_then(|qr| qr.get("data"))
        .and_then(|d| d.get("columns"))
    {
        Some(c) => c.clone(),
        None => return data.clone(),
    };

    let rows = match data
        .get("query_result")
        .and_then(|qr| qr.get("data"))
        .and_then(|d| d.get("rows"))
        .and_then(|r| r.as_array())
    {
        Some(r) => r,
        None => return data.clone(),
    };

    let total_rows = rows.len() as u64;
    let returned_rows = total_rows.min(max_rows);
    let truncated = total_rows > max_rows;
    let column_count = columns.as_array().map_or(0, |c| c.len());

    let sliced_rows: Vec<Value> = rows.iter().take(max_rows as usize).cloned().collect();

    serde_json::json!({
        "columns": columns,
        "rows": sliced_rows,
        "_metadata": {
            "total_rows": total_rows,
            "returned_rows": returned_rows,
            "truncated": truncated,
            "column_count": column_count
        }
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

    #[test]
    fn optional_string_present() {
        let args = json!({"name": "test query"});
        assert_eq!(
            optional_string(&args, "name"),
            Some("test query".to_string())
        );
    }

    #[test]
    fn optional_string_missing() {
        let args = json!({});
        assert_eq!(optional_string(&args, "name"), None);
    }

    #[test]
    fn optional_string_wrong_type() {
        let args = json!({"name": 42});
        assert_eq!(optional_string(&args, "name"), None);
    }

    #[test]
    fn optional_json_present() {
        let args = json!({"options": {"limit": 10}});
        assert_eq!(optional_json(&args, "options"), Some(json!({"limit": 10})));
    }

    #[test]
    fn optional_json_missing() {
        let args = json!({});
        assert_eq!(optional_json(&args, "options"), None);
    }

    #[test]
    fn optional_json_object() {
        let args = json!({"params": {"key": "value"}});
        let result = optional_json(&args, "params").unwrap();
        assert_eq!(result["key"], "value");
    }

    #[test]
    fn truncate_no_truncation_needed() {
        let data = json!({
            "query_result": {
                "data": {
                    "columns": [{"name": "id"}, {"name": "name"}],
                    "rows": [{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]
                }
            }
        });
        let result = truncate_query_result(&data, 100);
        assert_eq!(result["_metadata"]["total_rows"], 2);
        assert_eq!(result["_metadata"]["returned_rows"], 2);
        assert_eq!(result["_metadata"]["truncated"], false);
        assert_eq!(result["_metadata"]["column_count"], 2);
        assert_eq!(result["rows"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn truncate_limits_rows() {
        let rows: Vec<Value> = (0..10).map(|i| json!({"id": i})).collect();
        let data = json!({
            "query_result": {
                "data": {
                    "columns": [{"name": "id"}],
                    "rows": rows
                }
            }
        });
        let result = truncate_query_result(&data, 3);
        assert_eq!(result["_metadata"]["total_rows"], 10);
        assert_eq!(result["_metadata"]["returned_rows"], 3);
        assert_eq!(result["_metadata"]["truncated"], true);
        assert_eq!(result["rows"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn truncate_exact_boundary() {
        let rows: Vec<Value> = (0..5).map(|i| json!({"id": i})).collect();
        let data = json!({
            "query_result": {
                "data": {
                    "columns": [{"name": "id"}],
                    "rows": rows
                }
            }
        });
        let result = truncate_query_result(&data, 5);
        assert_eq!(result["_metadata"]["truncated"], false);
        assert_eq!(result["_metadata"]["total_rows"], 5);
        assert_eq!(result["_metadata"]["returned_rows"], 5);
    }

    #[test]
    fn truncate_zero_rows() {
        let data = json!({
            "query_result": {
                "data": {
                    "columns": [{"name": "id"}],
                    "rows": []
                }
            }
        });
        let result = truncate_query_result(&data, 100);
        assert_eq!(result["_metadata"]["total_rows"], 0);
        assert_eq!(result["_metadata"]["returned_rows"], 0);
        assert_eq!(result["_metadata"]["truncated"], false);
        assert_eq!(result["rows"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn truncate_unknown_structure() {
        let data = json!({"something": "else"});
        let result = truncate_query_result(&data, 100);
        assert_eq!(result, data);
    }

    #[test]
    fn truncate_missing_rows_key() {
        let data = json!({
            "query_result": {
                "data": {
                    "columns": [{"name": "id"}]
                }
            }
        });
        let result = truncate_query_result(&data, 100);
        assert_eq!(result, data);
    }
}
