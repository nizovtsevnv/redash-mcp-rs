use crate::error::Result;
use crate::redash::RedashClient;
use crate::{prompts, resources, tools};
use serde_json::Value;

const PROTOCOL_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = env!("CARGO_PKG_NAME");
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

// JSON-RPC 2.0 error codes
const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;

/// Process a single JSON-RPC message and return a response (or None for notifications).
pub async fn handle_message(request: &str, client: &RedashClient) -> Result<Option<String>> {
    let parsed = match serde_json::from_str::<Value>(request) {
        Ok(v) => v,
        Err(_) => {
            let resp = error_response(Value::Null, PARSE_ERROR, "Parse error");
            return Ok(Some(serde_json::to_string(&resp).unwrap_or_default()));
        }
    };

    let (id, method, params) = match parse_request(&parsed) {
        Ok(v) => v,
        Err(msg) => {
            let req_id = parsed.get("id").cloned().unwrap_or(Value::Null);
            let resp = error_response(req_id, INVALID_REQUEST, &msg);
            return Ok(Some(serde_json::to_string(&resp).unwrap_or_default()));
        }
    };

    // Notifications (no id) get no response
    if id.is_null() {
        return Ok(None);
    }

    let result = dispatch(&method, &params, client).await;

    let resp = match result {
        Ok(value) => success_response(id, value),
        Err((code, msg)) => error_response(id, code, &msg),
    };

    Ok(Some(serde_json::to_string(&resp).unwrap_or_default()))
}

/// Extract id, method, and params from a JSON-RPC request object.
fn parse_request(parsed: &Value) -> std::result::Result<(Value, String, Value), String> {
    let method = parsed
        .get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing or invalid method field".to_string())?;

    let id = parsed.get("id").cloned().unwrap_or(Value::Null);
    let params = parsed
        .get("params")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    Ok((id, method.to_string(), params))
}

/// Route a method call to the appropriate handler.
async fn dispatch(
    method: &str,
    params: &Value,
    client: &RedashClient,
) -> std::result::Result<Value, (i64, String)> {
    match method {
        "initialize" => Ok(initialize_result()),
        "notifications/initialized" => {
            // Should not reach here (handled as notification), but return empty just in case
            Ok(Value::Null)
        }
        "tools/list" => Ok(serde_json::json!({ "tools": tools::tool_definitions() })),
        "tools/call" => handle_tool_call(params, client).await,
        "resources/list" => Ok(serde_json::json!({
            "resources": resources::resource_list(),
            "resourceTemplates": resources::resource_templates()
        })),
        "resources/read" => handle_resource_read(params, client).await,
        "prompts/list" => Ok(serde_json::json!({ "prompts": prompts::prompt_list() })),
        "prompts/get" => handle_prompt_get(params),
        _ => Err((METHOD_NOT_FOUND, format!("method not found: {method}"))),
    }
}

/// Handle a tools/call request.
async fn handle_tool_call(
    params: &Value,
    client: &RedashClient,
) -> std::result::Result<Value, (i64, String)> {
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (INVALID_REQUEST, "missing tool name".to_string()))?;

    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    match tools::call_tool(name, &args, client).await {
        Ok(result) => Ok(result),
        Err(e) => Ok(tools::format_tool_error(&e.to_string())),
    }
}

/// Handle a prompts/get request.
fn handle_prompt_get(params: &Value) -> std::result::Result<Value, (i64, String)> {
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (INVALID_REQUEST, "missing prompt name".to_string()))?;

    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    prompts::get_prompt(name, &args).map_err(|e| (INVALID_REQUEST, e.to_string()))
}

/// Handle a resources/read request.
async fn handle_resource_read(
    params: &Value,
    client: &RedashClient,
) -> std::result::Result<Value, (i64, String)> {
    let uri = params
        .get("uri")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (INVALID_REQUEST, "missing resource URI".to_string()))?;

    resources::read_resource(uri, client)
        .await
        .map_err(|e| (INVALID_REQUEST, e.to_string()))
}

/// Build the initialize response with server capabilities.
fn initialize_result() -> Value {
    serde_json::json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "tools": {},
            "resources": {},
            "prompts": {}
        },
        "serverInfo": {
            "name": SERVER_NAME,
            "version": SERVER_VERSION
        }
    })
}

/// Build a JSON-RPC success response.
fn success_response(id: Value, result: Value) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

/// Build a JSON-RPC error response.
fn error_response(id: Value, code: i64, message: &str) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_request() {
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        });
        let (id, method, _params) = parse_request(&req).unwrap();
        assert_eq!(id, 1);
        assert_eq!(method, "initialize");
    }

    #[test]
    fn parse_request_missing_method() {
        let req = serde_json::json!({ "jsonrpc": "2.0", "id": 1 });
        let err = parse_request(&req).unwrap_err();
        assert!(err.contains("method"));
    }

    #[test]
    fn parse_notification_has_null_id() {
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        let (id, method, _) = parse_request(&req).unwrap();
        assert!(id.is_null());
        assert_eq!(method, "notifications/initialized");
    }

    #[test]
    fn parse_request_default_params() {
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        });
        let (_, _, params) = parse_request(&req).unwrap();
        assert!(params.is_object());
    }

    #[test]
    fn success_response_structure() {
        let resp = success_response(serde_json::json!(1), serde_json::json!({"ok": true}));
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 1);
        assert_eq!(resp["result"]["ok"], true);
        assert!(resp.get("error").is_none());
    }

    #[test]
    fn error_response_structure() {
        let resp = error_response(serde_json::json!(1), METHOD_NOT_FOUND, "not found");
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 1);
        assert_eq!(resp["error"]["code"], METHOD_NOT_FOUND);
        assert_eq!(resp["error"]["message"], "not found");
        assert!(resp.get("result").is_none());
    }

    #[test]
    fn initialize_result_structure() {
        let result = initialize_result();
        assert_eq!(result["protocolVersion"], PROTOCOL_VERSION);
        assert!(result["capabilities"]["tools"].is_object());
        assert!(result["serverInfo"]["name"].is_string());
        assert!(result["serverInfo"]["version"].is_string());
    }

    #[tokio::test]
    async fn handle_malformed_json() {
        let client = RedashClient::new("http://test".into(), "key".into(), 30, 0);
        let resp = handle_message("not json{", &client).await.unwrap().unwrap();
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(parsed["error"]["code"], PARSE_ERROR);
    }

    #[tokio::test]
    async fn handle_unknown_method() {
        let client = RedashClient::new("http://test".into(), "key".into(), 30, 0);
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"foo/bar"}"#;
        let resp = handle_message(req, &client).await.unwrap().unwrap();
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(parsed["error"]["code"], METHOD_NOT_FOUND);
    }

    #[tokio::test]
    async fn handle_notification_returns_none() {
        let client = RedashClient::new("http://test".into(), "key".into(), 30, 0);
        let req = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let result = handle_message(req, &client).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn handle_initialize() {
        let client = RedashClient::new("http://test".into(), "key".into(), 30, 0);
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let resp = handle_message(req, &client).await.unwrap().unwrap();
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(parsed["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert!(parsed["result"]["capabilities"]["tools"].is_object());
    }

    #[tokio::test]
    async fn handle_tools_list() {
        let client = RedashClient::new("http://test".into(), "key".into(), 30, 0);
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
        let resp = handle_message(req, &client).await.unwrap().unwrap();
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        let tools = parsed["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 40);
    }
}
