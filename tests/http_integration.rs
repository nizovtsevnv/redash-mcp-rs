//! Integration tests for the HTTP transport layer.
//!
//! These tests start a real HTTP server and exercise the full request
//! pipeline including auth, rate limiting, session management, and MCP dispatch.

use std::net::TcpListener;

use redash_mcp_rs::config::HttpConfig;
use redash_mcp_rs::http::server;

/// Find a free port by binding to port 0 and reading the assigned port.
fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

/// Create a test HTTP config with a random port.
fn test_config() -> HttpConfig {
    HttpConfig {
        api_url: "http://localhost:5000/api".into(),
        host: "127.0.0.1".into(),
        port: free_port(),
        max_body_size: 1048576,
        session_timeout: 1800,
        rate_limit: 60,
        auth_tokens: vec!["test-token".into()],
        timeout: 30,
        max_retries: 0,
    }
}

/// Start the server in the background and return the base URL.
async fn start_server(config: HttpConfig) -> String {
    let url = format!("http://{}:{}", config.host, config.port);
    tokio::spawn(async move {
        server::run(config).await.unwrap();
    });
    // Wait for the server to start
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    url
}

#[tokio::test]
async fn health_endpoint() {
    let config = test_config();
    let url = start_server(config).await;

    let client = reqwest::Client::new();
    let resp = client.get(format!("{url}/health")).send().await.unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn cors_preflight() {
    let config = test_config();
    let url = start_server(config).await;

    let client = reqwest::Client::new();
    let resp = client
        .request(reqwest::Method::OPTIONS, format!("{url}/mcp"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 204);
    assert_eq!(
        resp.headers()
            .get("Access-Control-Allow-Origin")
            .unwrap()
            .to_str()
            .unwrap(),
        "*"
    );
}

#[tokio::test]
async fn mcp_post_without_auth_returns_401() {
    let config = test_config();
    let url = start_server(config).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{url}/mcp"))
        .header("Content-Type", "application/json")
        .body(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn mcp_post_with_wrong_token_returns_401() {
    let config = test_config();
    let url = start_server(config).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{url}/mcp"))
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer wrong-token")
        .body(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn mcp_post_without_content_type_returns_400() {
    let config = test_config();
    let url = start_server(config).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{url}/mcp"))
        .header("Authorization", "Bearer test-token")
        .body(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn mcp_post_without_api_key_returns_400() {
    let config = test_config();
    let url = start_server(config).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{url}/mcp"))
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer test-token")
        .body(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("X-Redash-API-Key"));
}

#[tokio::test]
async fn mcp_initialize_creates_session() {
    let config = test_config();
    let url = start_server(config).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{url}/mcp"))
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer test-token")
        .header("X-Redash-API-Key", "my-redash-key")
        .body(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    // Should have a session ID header
    let session_id = resp
        .headers()
        .get("Mcp-Session-Id")
        .expect("missing Mcp-Session-Id header")
        .to_str()
        .unwrap()
        .to_string();
    assert!(!session_id.is_empty());

    // Response should be a valid JSON-RPC initialize response
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["id"], 1);
    assert!(body["result"]["protocolVersion"].is_string());
    assert!(body["result"]["capabilities"]["tools"].is_object());
}

#[tokio::test]
async fn mcp_session_flow() {
    let config = test_config();
    let url = start_server(config).await;

    let client = reqwest::Client::new();

    // 1. Initialize — get session
    let resp = client
        .post(format!("{url}/mcp"))
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer test-token")
        .header("X-Redash-API-Key", "my-redash-key")
        .body(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let session_id = resp
        .headers()
        .get("Mcp-Session-Id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // 2. Use session to list tools
    let resp = client
        .post(format!("{url}/mcp"))
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer test-token")
        .header("Mcp-Session-Id", &session_id)
        .body(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let tools = body["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 51);

    // 3. Delete session
    let resp = client
        .delete(format!("{url}/mcp"))
        .header("Authorization", "Bearer test-token")
        .header("Mcp-Session-Id", &session_id)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 202);

    // 4. Try to use deleted session — should fail
    let resp = client
        .post(format!("{url}/mcp"))
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer test-token")
        .header("Mcp-Session-Id", &session_id)
        .body(r#"{"jsonrpc":"2.0","id":3,"method":"tools/list","params":{}}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn not_found_returns_404() {
    let config = test_config();
    let url = start_server(config).await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{url}/nonexistent"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn cors_headers_on_all_responses() {
    let config = test_config();
    let url = start_server(config).await;

    let client = reqwest::Client::new();

    // Health endpoint should have CORS headers
    let resp = client.get(format!("{url}/health")).send().await.unwrap();
    assert!(resp.headers().get("Access-Control-Allow-Origin").is_some());

    // 404 should also have CORS headers
    let resp = client.get(format!("{url}/notfound")).send().await.unwrap();
    assert!(resp.headers().get("Access-Control-Allow-Origin").is_some());
}
