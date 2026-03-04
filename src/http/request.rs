use crate::error::{Error, Result};
use http_body_util::BodyExt;

/// Read the full request body, enforcing a maximum size.
pub async fn read_body(body: hyper::body::Incoming, max_size: usize) -> Result<String> {
    let collected = body
        .collect()
        .await
        .map_err(|e| Error::Transport(format!("failed to read request body: {e}")))?;

    let bytes = collected.to_bytes();
    if bytes.len() > max_size {
        return Err(Error::Transport(format!(
            "request body too large: {} bytes (max {})",
            bytes.len(),
            max_size
        )));
    }

    String::from_utf8(bytes.to_vec())
        .map_err(|e| Error::Transport(format!("invalid UTF-8 in request body: {e}")))
}

/// Check if the Accept header includes `text/event-stream`.
pub fn accepts_sse(accept: Option<&str>) -> bool {
    match accept {
        Some(a) => a.split(',').any(|part| {
            let media = part.split(';').next().unwrap_or("");
            media.trim() == "text/event-stream"
        }),
        None => false,
    }
}

/// Check that a request has the expected JSON content type.
pub fn is_json_content_type(content_type: Option<&str>) -> bool {
    match content_type {
        Some(ct) => ct.starts_with("application/json"),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_content_type_valid() {
        assert!(is_json_content_type(Some("application/json")));
        assert!(is_json_content_type(Some(
            "application/json; charset=utf-8"
        )));
    }

    #[test]
    fn json_content_type_invalid() {
        assert!(!is_json_content_type(Some("text/plain")));
        assert!(!is_json_content_type(Some("text/html")));
    }

    #[test]
    fn json_content_type_none() {
        assert!(!is_json_content_type(None));
    }

    #[test]
    fn accepts_sse_present() {
        assert!(accepts_sse(Some("text/event-stream")));
        assert!(accepts_sse(Some("text/event-stream, application/json")));
        assert!(accepts_sse(Some("application/json, text/event-stream")));
    }

    #[test]
    fn accepts_sse_with_params() {
        assert!(accepts_sse(Some("text/event-stream; q=0.9")));
        assert!(accepts_sse(Some(
            "application/json, text/event-stream; q=0.8"
        )));
    }

    #[test]
    fn accepts_sse_absent() {
        assert!(!accepts_sse(Some("application/json")));
        assert!(!accepts_sse(Some("text/plain")));
        assert!(!accepts_sse(None));
    }
}
