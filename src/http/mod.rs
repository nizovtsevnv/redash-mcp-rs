pub mod auth;
pub mod cors;
pub mod health;
pub mod request;
pub mod response;
pub mod router;
pub mod session;
pub mod sse;

use bytes::Bytes;
use http_body_util::BodyExt;

/// Boxed HTTP body type used across the HTTP transport.
pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, std::convert::Infallible>;

/// Wrap data into a complete BoxBody.
pub fn full_body(data: impl Into<Bytes>) -> BoxBody {
    http_body_util::Full::new(data.into()).boxed()
}

/// Create an empty BoxBody.
pub fn empty_body() -> BoxBody {
    http_body_util::Empty::<Bytes>::new().boxed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn full_body_has_data() {
        let body = full_body("hello");
        let collected = body.collect().await.unwrap();
        assert_eq!(collected.to_bytes(), "hello");
    }

    #[tokio::test]
    async fn empty_body_has_no_data() {
        let body = empty_body();
        let collected = body.collect().await.unwrap();
        assert!(collected.to_bytes().is_empty());
    }
}
