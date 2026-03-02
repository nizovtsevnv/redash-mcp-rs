use bytes::Bytes;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;

/// A streaming body for Server-Sent Events (SSE).
///
/// Wraps an `mpsc::Receiver<Bytes>` to produce SSE-formatted frames.
pub struct SseBody {
    rx: mpsc::Receiver<Bytes>,
}

impl SseBody {
    /// Create a new SSE body from a receiver channel.
    pub fn new(rx: mpsc::Receiver<Bytes>) -> Self {
        Self { rx }
    }
}

impl hyper::body::Body for SseBody {
    type Data = Bytes;
    type Error = std::convert::Infallible;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        match self.rx.poll_recv(cx) {
            Poll::Ready(Some(data)) => Poll::Ready(Some(Ok(hyper::body::Frame::data(data)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Format a JSON-RPC response as an SSE `data:` event.
pub fn format_sse_event(json: &str) -> Bytes {
    Bytes::from(format!("data: {json}\n\n"))
}

/// Build an SSE response with the appropriate headers.
///
/// Returns the response and a sender for pushing SSE events.
pub fn sse_response(
    session_id: Option<&str>,
) -> (hyper::Response<super::BoxBody>, mpsc::Sender<Bytes>) {
    use http_body_util::BodyExt;

    let (tx, rx) = mpsc::channel(32);
    let body = SseBody::new(rx);

    let mut builder = hyper::Response::builder()
        .status(200)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive");

    if let Some(id) = session_id {
        builder = builder.header("Mcp-Session-Id", id);
    }

    let resp = builder
        .body(body.map_err(|never| match never {}).boxed())
        .expect("failed to build SSE response");

    (resp, tx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_sse_event_structure() {
        let event = format_sse_event(r#"{"jsonrpc":"2.0","id":1}"#);
        let s = std::str::from_utf8(&event).unwrap();
        assert!(s.starts_with("data: "));
        assert!(s.ends_with("\n\n"));
        assert!(s.contains(r#""jsonrpc":"2.0""#));
    }

    #[tokio::test]
    async fn sse_body_receives_data() {
        use http_body_util::BodyExt;

        let (tx, rx) = mpsc::channel(1);
        let mut body = SseBody::new(rx);

        tx.send(Bytes::from("data: test\n\n")).await.unwrap();
        drop(tx);

        let frame = body.frame().await.unwrap().unwrap();
        assert_eq!(frame.into_data().unwrap(), "data: test\n\n");
    }

    #[test]
    fn sse_response_headers() {
        let (resp, _tx) = sse_response(Some("sess-42"));
        assert_eq!(resp.status(), 200);
        assert_eq!(
            resp.headers().get("Content-Type").unwrap(),
            "text/event-stream"
        );
        assert_eq!(resp.headers().get("Mcp-Session-Id").unwrap(), "sess-42");
    }
}
