use super::BoxBody;

/// Build a 200 OK response with a JSON body.
///
/// Optionally includes the `Mcp-Session-Id` header.
pub fn ok_json(body: &str, session_id: Option<&str>) -> hyper::Response<BoxBody> {
    let mut builder = hyper::Response::builder()
        .status(200)
        .header("Content-Type", "application/json");

    if let Some(id) = session_id {
        builder = builder.header("Mcp-Session-Id", id);
    }

    builder
        .body(super::full_body(body.to_string()))
        .expect("failed to build response")
}

/// Build a 202 Accepted response (for notifications).
pub fn accepted(session_id: Option<&str>) -> hyper::Response<BoxBody> {
    let mut builder = hyper::Response::builder().status(202);

    if let Some(id) = session_id {
        builder = builder.header("Mcp-Session-Id", id);
    }

    builder
        .body(super::empty_body())
        .expect("failed to build response")
}

/// Build a 400 Bad Request response with a message.
pub fn bad_request(message: &str) -> hyper::Response<BoxBody> {
    hyper::Response::builder()
        .status(400)
        .header("Content-Type", "application/json")
        .body(super::full_body(format!(r#"{{"error":"{message}"}}"#)))
        .expect("failed to build response")
}

/// Build a 401 Unauthorized response.
pub fn unauthorized() -> hyper::Response<BoxBody> {
    hyper::Response::builder()
        .status(401)
        .header("Content-Type", "application/json")
        .body(super::full_body(r#"{"error":"unauthorized"}"#))
        .expect("failed to build response")
}

/// Build a 404 Not Found response.
pub fn not_found() -> hyper::Response<BoxBody> {
    hyper::Response::builder()
        .status(404)
        .header("Content-Type", "application/json")
        .body(super::full_body(r#"{"error":"not found"}"#))
        .expect("failed to build response")
}

/// Build a 405 Method Not Allowed response.
pub fn method_not_allowed() -> hyper::Response<BoxBody> {
    hyper::Response::builder()
        .status(405)
        .header("Content-Type", "application/json")
        .body(super::full_body(r#"{"error":"method not allowed"}"#))
        .expect("failed to build response")
}

/// Build a 429 Too Many Requests response.
pub fn too_many_requests() -> hyper::Response<BoxBody> {
    hyper::Response::builder()
        .status(429)
        .header("Content-Type", "application/json")
        .body(super::full_body(r#"{"error":"too many requests"}"#))
        .expect("failed to build response")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_json_status() {
        let resp = ok_json(r#"{"ok":true}"#, None);
        assert_eq!(resp.status(), 200);
    }

    #[test]
    fn ok_json_with_session_id() {
        let resp = ok_json(r#"{"ok":true}"#, Some("sess-123"));
        assert_eq!(resp.headers().get("Mcp-Session-Id").unwrap(), "sess-123");
    }

    #[test]
    fn accepted_status() {
        let resp = accepted(None);
        assert_eq!(resp.status(), 202);
    }

    #[test]
    fn bad_request_status() {
        let resp = bad_request("invalid");
        assert_eq!(resp.status(), 400);
    }

    #[test]
    fn unauthorized_status() {
        let resp = unauthorized();
        assert_eq!(resp.status(), 401);
    }

    #[test]
    fn not_found_status() {
        let resp = not_found();
        assert_eq!(resp.status(), 404);
    }

    #[test]
    fn method_not_allowed_status() {
        let resp = method_not_allowed();
        assert_eq!(resp.status(), 405);
    }

    #[test]
    fn too_many_requests_status() {
        let resp = too_many_requests();
        assert_eq!(resp.status(), 429);
    }
}
