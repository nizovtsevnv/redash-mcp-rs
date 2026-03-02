use super::BoxBody;

/// Add CORS headers to a response.
pub fn add_cors_headers(resp: &mut hyper::Response<BoxBody>) {
    let headers = resp.headers_mut();
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert(
        "Access-Control-Allow-Methods",
        "GET, POST, DELETE, OPTIONS".parse().unwrap(),
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization, Mcp-Session-Id, X-Redash-API-Key"
            .parse()
            .unwrap(),
    );
    headers.insert(
        "Access-Control-Expose-Headers",
        "Mcp-Session-Id".parse().unwrap(),
    );
}

/// Build a 204 No Content preflight response with CORS headers.
pub fn preflight() -> hyper::Response<BoxBody> {
    let mut resp = hyper::Response::builder()
        .status(204)
        .body(super::empty_body())
        .expect("failed to build response");
    add_cors_headers(&mut resp);
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preflight_status() {
        let resp = preflight();
        assert_eq!(resp.status(), 204);
    }

    #[test]
    fn preflight_has_cors_headers() {
        let resp = preflight();
        assert_eq!(
            resp.headers().get("Access-Control-Allow-Origin").unwrap(),
            "*"
        );
        assert!(resp
            .headers()
            .get("Access-Control-Allow-Methods")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("POST"));
        assert!(resp
            .headers()
            .get("Access-Control-Allow-Headers")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("Authorization"));
        assert!(resp
            .headers()
            .get("Access-Control-Expose-Headers")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("Mcp-Session-Id"));
    }

    #[test]
    fn add_cors_to_existing_response() {
        let mut resp = hyper::Response::builder()
            .status(200)
            .body(super::super::full_body("test"))
            .unwrap();
        add_cors_headers(&mut resp);
        assert_eq!(
            resp.headers().get("Access-Control-Allow-Origin").unwrap(),
            "*"
        );
    }
}
