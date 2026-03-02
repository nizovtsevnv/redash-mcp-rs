use hyper::Method;

/// Resolved route for an incoming HTTP request.
#[derive(Debug, PartialEq)]
pub enum Route {
    /// POST /mcp — JSON-RPC message.
    McpPost,
    /// GET /mcp — SSE stream for server-initiated messages.
    McpGet,
    /// DELETE /mcp — close a session.
    McpDelete,
    /// GET /health — health check.
    Health,
    /// OPTIONS (any path) — CORS preflight.
    Preflight,
    /// Anything else.
    NotFound,
}

/// Resolve an HTTP method + path into a route.
pub fn resolve(method: &Method, path: &str) -> Route {
    if method == Method::OPTIONS {
        return Route::Preflight;
    }

    match (method, path) {
        (&Method::POST, "/mcp") => Route::McpPost,
        (&Method::GET, "/mcp") => Route::McpGet,
        (&Method::DELETE, "/mcp") => Route::McpDelete,
        (&Method::GET, "/health") => Route::Health,
        _ => Route::NotFound,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_mcp() {
        assert_eq!(resolve(&Method::POST, "/mcp"), Route::McpPost);
    }

    #[test]
    fn get_mcp() {
        assert_eq!(resolve(&Method::GET, "/mcp"), Route::McpGet);
    }

    #[test]
    fn delete_mcp() {
        assert_eq!(resolve(&Method::DELETE, "/mcp"), Route::McpDelete);
    }

    #[test]
    fn get_health() {
        assert_eq!(resolve(&Method::GET, "/health"), Route::Health);
    }

    #[test]
    fn options_any_path() {
        assert_eq!(resolve(&Method::OPTIONS, "/mcp"), Route::Preflight);
        assert_eq!(resolve(&Method::OPTIONS, "/health"), Route::Preflight);
        assert_eq!(resolve(&Method::OPTIONS, "/foo"), Route::Preflight);
    }

    #[test]
    fn unknown_path() {
        assert_eq!(resolve(&Method::GET, "/unknown"), Route::NotFound);
    }

    #[test]
    fn wrong_method_for_health() {
        assert_eq!(resolve(&Method::POST, "/health"), Route::NotFound);
    }
}
