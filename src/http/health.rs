use super::BoxBody;

/// Build a health check response.
pub fn health_response() -> hyper::Response<BoxBody> {
    hyper::Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(super::full_body(r#"{"status":"ok"}"#))
        .expect("failed to build response")
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    #[test]
    fn health_status() {
        let resp = health_response();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn health_body() {
        let resp = health_response();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
    }
}
