//! Integration tests for the MCP protocol layer.
//!
//! These tests exercise the public API without a real Redash server,
//! verifying JSON-RPC message handling and protocol compliance.

use redash_mcp_rs::mcp::handle_message;
use redash_mcp_rs::redash::RedashClient;

fn test_client() -> RedashClient {
    RedashClient::new("http://localhost:5000/api".into(), "test-key".into(), 30, 0)
}

#[tokio::test]
async fn initialize_handshake() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["id"], 1);
    assert_eq!(parsed["result"]["protocolVersion"], "2024-11-05");
    assert!(parsed["result"]["capabilities"]["tools"].is_object());
    assert!(parsed["result"]["serverInfo"]["name"].is_string());
    assert!(parsed["result"]["serverInfo"]["version"].is_string());
}

#[tokio::test]
async fn tools_list_returns_all_tools() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    let tools = parsed["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 42);

    for tool in tools {
        assert!(tool["name"].is_string(), "tool missing name");
        assert!(tool["description"].is_string(), "tool missing description");
        assert!(tool["inputSchema"].is_object(), "tool missing inputSchema");
    }
}

#[tokio::test]
async fn notification_returns_none() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let result = handle_message(req, &client).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn unknown_method_returns_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":3,"method":"unknown/method","params":{}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["error"]["code"], -32601);
    assert!(parsed["error"]["message"]
        .as_str()
        .unwrap()
        .contains("method not found"));
}

#[tokio::test]
async fn malformed_json_returns_parse_error() {
    let client = test_client();
    let resp = handle_message("{not json", &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["error"]["code"], -32700);
}

#[tokio::test]
async fn tools_call_unknown_tool_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"nonexistent","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    // Tool errors are returned as results with isError, not JSON-RPC errors
    assert!(parsed.get("error").is_none());
    let result = &parsed["result"];
    assert_eq!(result["isError"], true);
    assert!(result["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("unknown tool"));
}

#[tokio::test]
async fn create_query_missing_args_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"create_query","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn update_query_missing_id_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"update_query","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn archive_query_missing_id_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"archive_query","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn execute_query_missing_args_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"execute_query","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn create_dashboard_missing_name_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"create_dashboard","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn get_user_missing_id_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"get_user","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn create_visualization_missing_args_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"create_visualization","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn delete_visualization_missing_id_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"delete_visualization","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn add_widget_missing_dashboard_id_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":13,"method":"tools/call","params":{"name":"add_widget","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn remove_widget_missing_id_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":14,"method":"tools/call","params":{"name":"remove_widget","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn update_dashboard_missing_id_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":15,"method":"tools/call","params":{"name":"update_dashboard","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn archive_dashboard_missing_id_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":16,"method":"tools/call","params":{"name":"archive_dashboard","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn create_alert_missing_args_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":17,"method":"tools/call","params":{"name":"create_alert","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn get_alert_missing_id_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":18,"method":"tools/call","params":{"name":"get_alert","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn delete_alert_missing_id_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":19,"method":"tools/call","params":{"name":"delete_alert","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn refresh_query_missing_id_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":20,"method":"tools/call","params":{"name":"refresh_query","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn fork_query_missing_id_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":21,"method":"tools/call","params":{"name":"fork_query","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn create_query_snippet_missing_args_returns_is_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":22,"method":"tools/call","params":{"name":"create_query_snippet","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["result"]["isError"], true);
    assert!(parsed["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("missing required argument"));
}

#[tokio::test]
async fn resources_list_returns_templates() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":23,"method":"resources/list","params":{}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    let templates = parsed["result"]["resourceTemplates"].as_array().unwrap();
    assert_eq!(templates.len(), 1);
    assert!(templates[0]["uriTemplate"]
        .as_str()
        .unwrap()
        .contains("datasource"));

    let resources = parsed["result"]["resources"].as_array().unwrap();
    assert!(resources.is_empty());
}

#[tokio::test]
async fn resources_read_missing_uri_returns_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":24,"method":"resources/read","params":{}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["error"]["code"], -32600);
    assert!(parsed["error"]["message"]
        .as_str()
        .unwrap()
        .contains("missing resource URI"));
}

#[tokio::test]
async fn resources_read_invalid_uri_returns_error() {
    let client = test_client();
    let req =
        r#"{"jsonrpc":"2.0","id":25,"method":"resources/read","params":{"uri":"http://invalid"}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["error"]["code"], -32600);
    assert!(parsed["error"]["message"]
        .as_str()
        .unwrap()
        .contains("unsupported resource URI"));
}

#[tokio::test]
async fn initialize_capabilities_include_resources() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":26,"method":"initialize","params":{}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert!(parsed["result"]["capabilities"]["resources"].is_object());
}

#[tokio::test]
async fn prompts_list_returns_prompts() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":27,"method":"prompts/list","params":{}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    let prompts = parsed["result"]["prompts"].as_array().unwrap();
    assert_eq!(prompts.len(), 3);
    for prompt in prompts {
        assert!(prompt["name"].is_string());
        assert!(prompt["description"].is_string());
    }
}

#[tokio::test]
async fn prompts_get_explore_data() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":28,"method":"prompts/get","params":{"name":"explore_data","arguments":{"data_source_id":"1"}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert!(parsed["result"]["description"].is_string());
    let messages = parsed["result"]["messages"].as_array().unwrap();
    assert!(!messages.is_empty());
    assert_eq!(messages[0]["role"], "user");
}

#[tokio::test]
async fn prompts_get_missing_name_returns_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":29,"method":"prompts/get","params":{}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["error"]["code"], -32600);
    assert!(parsed["error"]["message"]
        .as_str()
        .unwrap()
        .contains("missing prompt name"));
}

#[tokio::test]
async fn prompts_get_unknown_prompt_returns_error() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":30,"method":"prompts/get","params":{"name":"nonexistent","arguments":{}}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert_eq!(parsed["error"]["code"], -32600);
    assert!(parsed["error"]["message"]
        .as_str()
        .unwrap()
        .contains("unknown prompt"));
}

#[tokio::test]
async fn initialize_capabilities_include_prompts() {
    let client = test_client();
    let req = r#"{"jsonrpc":"2.0","id":31,"method":"initialize","params":{}}"#;
    let resp = handle_message(req, &client).await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();

    assert!(parsed["result"]["capabilities"]["prompts"].is_object());
}
