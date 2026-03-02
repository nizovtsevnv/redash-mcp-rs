use serde_json::Value;

/// Extract the progress token from a JSON-RPC params object.
///
/// MCP clients may send `_meta.progressToken` in tool call params
/// to receive progress notifications during long-running operations.
pub fn extract_progress_token(params: &Value) -> Option<Value> {
    params
        .get("_meta")
        .and_then(|meta| meta.get("progressToken"))
        .cloned()
}

/// Format a JSON-RPC progress notification.
///
/// Returns a `notifications/progress` message with the given token,
/// progress count, and optional total.
pub fn format_progress(token: &Value, progress: u64, total: Option<u64>) -> Value {
    let mut params = serde_json::json!({
        "progressToken": token,
        "progress": progress
    });

    if let Some(t) = total {
        params["total"] = serde_json::json!(t);
    }

    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/progress",
        "params": params
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn format_progress_with_total() {
        let token = json!("tok-123");
        let notif = format_progress(&token, 5, Some(10));
        assert_eq!(notif["jsonrpc"], "2.0");
        assert_eq!(notif["method"], "notifications/progress");
        assert_eq!(notif["params"]["progressToken"], "tok-123");
        assert_eq!(notif["params"]["progress"], 5);
        assert_eq!(notif["params"]["total"], 10);
    }

    #[test]
    fn format_progress_without_total() {
        let token = json!(42);
        let notif = format_progress(&token, 3, None);
        assert_eq!(notif["params"]["progressToken"], 42);
        assert_eq!(notif["params"]["progress"], 3);
        assert!(notif["params"].get("total").is_none());
    }

    #[test]
    fn extract_token_present() {
        let params = json!({
            "name": "execute_query",
            "arguments": {},
            "_meta": { "progressToken": "abc-def" }
        });
        let token = extract_progress_token(&params);
        assert_eq!(token, Some(json!("abc-def")));
    }

    #[test]
    fn extract_token_missing() {
        let params = json!({
            "name": "execute_query",
            "arguments": {}
        });
        assert!(extract_progress_token(&params).is_none());
    }
}
