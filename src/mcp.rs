use crate::error::Result;
use crate::logging::{self, LogLevel, McpLogLevel};
use crate::progress;
use crate::redash::RedashClient;
use crate::{prompts, resources, tools};
use serde_json::Value;

/// Channel for sending server-initiated notifications (progress, log) to the transport layer.
/// `None` means no streaming — notifications are silently discarded.
pub type NotificationSender = Option<tokio::sync::mpsc::Sender<Value>>;

const PROTOCOL_VERSION: &str = "2025-03-26";
const SERVER_NAME: &str = env!("CARGO_PKG_NAME");
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

// JSON-RPC 2.0 error codes
const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;

/// Process a single JSON-RPC message and return a response (or None for notifications).
pub async fn handle_message(
    request: &str,
    client: &RedashClient,
    log_level: &McpLogLevel,
    notification_tx: &NotificationSender,
) -> Result<Option<String>> {
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

    let result = dispatch(&method, &params, client, log_level, notification_tx).await;

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
    log_level: &McpLogLevel,
    notification_tx: &NotificationSender,
) -> std::result::Result<Value, (i64, String)> {
    match method {
        "initialize" => Ok(initialize_result()),
        "ping" => Ok(serde_json::json!({})),
        "notifications/initialized" | "notifications/cancelled" => {
            // Should not reach here (handled as notification), but return empty just in case
            Ok(Value::Null)
        }
        "logging/setLevel" => handle_set_log_level(params, log_level),
        "tools/list" => Ok(serde_json::json!({ "tools": tools::tool_definitions() })),
        "tools/call" => handle_tool_call(params, client, notification_tx, log_level).await,
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
    notification_tx: &NotificationSender,
    log_level: &McpLogLevel,
) -> std::result::Result<Value, (i64, String)> {
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (INVALID_REQUEST, "missing tool name".to_string()))?;

    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    let progress_token = progress::extract_progress_token(params);

    // Send log notification before tool call
    if let Some(tx) = notification_tx {
        if log_level.should_log(LogLevel::Info) {
            let notif =
                logging::log_notification(LogLevel::Info, "tools", &format!("calling {name}"));
            let _ = tx.send(notif).await;
        }
    }

    match tools::call_tool(
        name,
        &args,
        client,
        notification_tx,
        progress_token.as_ref(),
    )
    .await
    {
        Ok(result) => Ok(result),
        Err(e) => {
            let error_msg = e.to_string();
            if let Some(tx) = notification_tx {
                if log_level.should_log(LogLevel::Error) {
                    let notif = logging::log_notification(LogLevel::Error, "tools", &error_msg);
                    let _ = tx.send(notif).await;
                }
            }
            Ok(tools::format_tool_error(&error_msg))
        }
    }
}

/// Handle a logging/setLevel request.
fn handle_set_log_level(
    params: &Value,
    log_level: &McpLogLevel,
) -> std::result::Result<Value, (i64, String)> {
    let level_str = params
        .get("level")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (INVALID_REQUEST, "missing level parameter".to_string()))?;

    let level = LogLevel::parse(level_str)
        .ok_or_else(|| (INVALID_REQUEST, format!("invalid log level: {level_str}")))?;

    log_level.set(level);
    tracing::debug!("MCP log level set to {}", level.as_str());

    Ok(serde_json::json!({}))
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
            "prompts": {},
            "logging": {}
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

    fn test_log_level() -> McpLogLevel {
        McpLogLevel::default()
    }

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
        let resp = handle_message("not json{", &client, &test_log_level(), &None)
            .await
            .unwrap()
            .unwrap();
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(parsed["error"]["code"], PARSE_ERROR);
    }

    #[tokio::test]
    async fn handle_unknown_method() {
        let client = RedashClient::new("http://test".into(), "key".into(), 30, 0);
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"foo/bar"}"#;
        let resp = handle_message(req, &client, &test_log_level(), &None)
            .await
            .unwrap()
            .unwrap();
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(parsed["error"]["code"], METHOD_NOT_FOUND);
    }

    #[tokio::test]
    async fn handle_notification_returns_none() {
        let client = RedashClient::new("http://test".into(), "key".into(), 30, 0);
        let req = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let result = handle_message(req, &client, &test_log_level(), &None)
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn handle_initialize() {
        let client = RedashClient::new("http://test".into(), "key".into(), 30, 0);
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let resp = handle_message(req, &client, &test_log_level(), &None)
            .await
            .unwrap()
            .unwrap();
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(parsed["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert!(parsed["result"]["capabilities"]["tools"].is_object());
    }

    #[tokio::test]
    async fn ping_returns_empty_object() {
        let client = RedashClient::new("http://test".into(), "key".into(), 30, 0);
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"ping","params":{}}"#;
        let resp = handle_message(req, &client, &test_log_level(), &None)
            .await
            .unwrap()
            .unwrap();
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(parsed["result"], serde_json::json!({}));
    }

    #[tokio::test]
    async fn handle_tools_list() {
        let client = RedashClient::new("http://test".into(), "key".into(), 30, 0);
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
        let resp = handle_message(req, &client, &test_log_level(), &None)
            .await
            .unwrap()
            .unwrap();
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        let tools = parsed["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 60);
    }

    #[tokio::test]
    async fn handle_set_log_level_valid() {
        let client = RedashClient::new("http://test".into(), "key".into(), 30, 0);
        let log_level = McpLogLevel::default();
        let req =
            r#"{"jsonrpc":"2.0","id":1,"method":"logging/setLevel","params":{"level":"debug"}}"#;
        let resp = handle_message(req, &client, &log_level, &None)
            .await
            .unwrap()
            .unwrap();
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(parsed["result"], serde_json::json!({}));
        assert_eq!(log_level.get(), LogLevel::Debug);
    }

    #[tokio::test]
    async fn handle_set_log_level_invalid() {
        let client = RedashClient::new("http://test".into(), "key".into(), 30, 0);
        let log_level = McpLogLevel::default();
        let req =
            r#"{"jsonrpc":"2.0","id":1,"method":"logging/setLevel","params":{"level":"bogus"}}"#;
        let resp = handle_message(req, &client, &log_level, &None)
            .await
            .unwrap()
            .unwrap();
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        assert!(parsed.get("error").is_some());
    }

    #[test]
    fn initialize_includes_logging_capability() {
        let result = initialize_result();
        assert!(result["capabilities"]["logging"].is_object());
    }

    #[tokio::test]
    async fn cancelled_notification_returns_none() {
        let client = RedashClient::new("http://test".into(), "key".into(), 30, 0);
        let req = r#"{"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":1,"reason":"user cancelled"}}"#;
        let result = handle_message(req, &client, &test_log_level(), &None)
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn tool_call_sends_log_notification_when_level_allows() {
        let client = RedashClient::new("http://test".into(), "key".into(), 30, 0);
        let log_level = McpLogLevel::new(LogLevel::Info);
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Value>(32);
        let notification_tx: NotificationSender = Some(tx);

        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"nonexistent","arguments":{}}}"#;
        let _ = handle_message(req, &client, &log_level, &notification_tx).await;

        // Should receive at least the "calling" info notification
        let notif = rx.recv().await.unwrap();
        assert_eq!(notif["method"], "notifications/message");
        assert_eq!(notif["params"]["level"], "info");
        assert!(notif["params"]["data"]
            .as_str()
            .unwrap()
            .contains("calling nonexistent"));
    }

    #[tokio::test]
    async fn tool_call_sends_error_log_on_failure() {
        let client = RedashClient::new("http://test".into(), "key".into(), 30, 0);
        let log_level = McpLogLevel::new(LogLevel::Error);
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Value>(32);
        let notification_tx: NotificationSender = Some(tx);

        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"nonexistent","arguments":{}}}"#;
        let _ = handle_message(req, &client, &log_level, &notification_tx).await;

        // With Error level, we should NOT get the Info "calling" notification
        // but should get the Error notification for unknown tool
        let notif = rx.recv().await.unwrap();
        assert_eq!(notif["params"]["level"], "error");
        assert!(notif["params"]["data"]
            .as_str()
            .unwrap()
            .contains("unknown tool"));
    }

    #[tokio::test]
    async fn tool_call_no_log_when_level_suppresses() {
        let client = RedashClient::new("http://test".into(), "key".into(), 30, 0);
        let log_level = McpLogLevel::new(LogLevel::Emergency);
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Value>(32);
        let notification_tx: NotificationSender = Some(tx);

        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"nonexistent","arguments":{}}}"#;
        let _ = handle_message(req, &client, &log_level, &notification_tx).await;

        // With Emergency level, no Info or Error notifications should be sent
        drop(notification_tx);
        assert!(rx.recv().await.is_none());
    }
}
