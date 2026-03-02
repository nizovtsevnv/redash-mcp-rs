use super::server::AppState;
use super::{auth, cors, health, request, response, router, sse, BoxBody};
use crate::{config, mcp, redash};
use std::sync::Arc;

/// Handle an incoming HTTP request through the full pipeline.
pub async fn handle_request(
    req: hyper::Request<hyper::body::Incoming>,
    state: Arc<AppState>,
) -> hyper::Response<BoxBody> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let route = router::resolve(&method, &path);

    let mut resp = match route {
        router::Route::Preflight => cors::preflight(),
        router::Route::Health => health::health_response(),
        router::Route::NotFound => response::not_found(),
        router::Route::McpPost | router::Route::McpGet | router::Route::McpDelete => {
            handle_mcp(req, &route, state).await
        }
    };

    cors::add_cors_headers(&mut resp);
    resp
}

/// Handle MCP-specific routes (POST/GET/DELETE /mcp) with auth and rate limiting.
async fn handle_mcp(
    req: hyper::Request<hyper::body::Incoming>,
    route: &router::Route,
    state: Arc<AppState>,
) -> hyper::Response<BoxBody> {
    // Authenticate
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());
    if !auth::validate_bearer_token(auth_header, &state.config.auth_tokens) {
        return response::unauthorized();
    }

    // Rate limit by peer address (use header or fallback)
    let ip = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    if !state.rate_limiter.check(&ip).await {
        return response::too_many_requests();
    }

    match route {
        router::Route::McpPost => handle_mcp_post(req, state).await,
        router::Route::McpGet => handle_mcp_get(&req, state).await,
        router::Route::McpDelete => handle_mcp_delete(&req, state).await,
        _ => response::not_found(),
    }
}

/// Handle POST /mcp — receive JSON-RPC message.
async fn handle_mcp_post(
    req: hyper::Request<hyper::body::Incoming>,
    state: Arc<AppState>,
) -> hyper::Response<BoxBody> {
    // Validate content type
    let content_type = req
        .headers()
        .get("Content-Type")
        .and_then(|v| v.to_str().ok());
    if !request::is_json_content_type(content_type) {
        return response::bad_request("Content-Type must be application/json");
    }

    // Check for existing session
    let session_id = req
        .headers()
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Get API key from session or header
    let api_key = if let Some(ref sid) = session_id {
        match state.sessions.get(sid).await {
            Some(key) => key,
            None => return response::bad_request("invalid or expired session"),
        }
    } else {
        // New session — require X-Redash-API-Key header
        match req
            .headers()
            .get("X-Redash-API-Key")
            .and_then(|v| v.to_str().ok())
        {
            Some(key) => match config::validate_api_key(key) {
                Ok(k) => k,
                Err(_) => return response::bad_request("invalid X-Redash-API-Key"),
            },
            None => return response::bad_request("missing X-Redash-API-Key header"),
        }
    };

    // Read and parse body
    let body_str = match request::read_body(req.into_body(), state.config.max_body_size).await {
        Ok(b) => b,
        Err(e) => return response::bad_request(&e.to_string()),
    };

    // Build per-request RedashClient with shared connection pool
    let client = redash::RedashClient::with_shared_client(
        state.shared_client.clone(),
        state.config.api_url.clone(),
        api_key.clone(),
    );

    // Dispatch to MCP handler
    match mcp::handle_message(&body_str, &client).await {
        Ok(Some(resp_body)) => {
            // Check if this is an initialize response — create session
            let actual_session_id = if session_id.is_none() {
                if is_initialize_response(&body_str) {
                    Some(state.sessions.create(api_key).await)
                } else {
                    None
                }
            } else {
                session_id
            };
            response::ok_json(&resp_body, actual_session_id.as_deref())
        }
        Ok(None) => {
            // Notification — no response body
            response::accepted(session_id.as_deref())
        }
        Err(e) => response::bad_request(&e.to_string()),
    }
}

/// Handle GET /mcp — SSE endpoint.
async fn handle_mcp_get(
    req: &hyper::Request<hyper::body::Incoming>,
    state: Arc<AppState>,
) -> hyper::Response<BoxBody> {
    let session_id = req
        .headers()
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok());

    // Require existing session
    let sid = match session_id {
        Some(id) => {
            if state.sessions.get(id).await.is_none() {
                return response::bad_request("invalid or expired session");
            }
            id.to_string()
        }
        None => return response::bad_request("missing Mcp-Session-Id header"),
    };

    // Return SSE stream (the sender can be used to push server-initiated messages)
    let (resp, _tx) = sse::sse_response(Some(&sid));
    resp
}

/// Handle DELETE /mcp — close session.
async fn handle_mcp_delete(
    req: &hyper::Request<hyper::body::Incoming>,
    state: Arc<AppState>,
) -> hyper::Response<BoxBody> {
    let session_id = req
        .headers()
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok());

    match session_id {
        Some(id) => {
            state.sessions.remove(id).await;
            response::accepted(None)
        }
        None => response::bad_request("missing Mcp-Session-Id header"),
    }
}

/// Check if a request body contains an initialize method call.
fn is_initialize_response(body: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("method").and_then(|m| m.as_str()).map(String::from))
        .map(|m| m == "initialize")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_initialize_request() {
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        assert!(is_initialize_response(body));
    }

    #[test]
    fn detect_non_initialize_request() {
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
        assert!(!is_initialize_response(body));
    }

    #[test]
    fn detect_invalid_json() {
        assert!(!is_initialize_response("not json"));
    }
}
