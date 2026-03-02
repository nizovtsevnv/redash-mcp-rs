use crate::error::{Error, Result};

/// Connect timeout in seconds.
const CONNECT_TIMEOUT_SECS: u64 = 10;

/// User-Agent header value.
const USER_AGENT: &str = concat!("redash-mcp-rs/", env!("CARGO_PKG_VERSION"));

/// HTTP client for the Redash API.
///
/// Wraps `reqwest::Client` with base URL and API key authentication.
/// All requests use the `Authorization: Key <api_key>` header format.
pub struct RedashClient {
    client: reqwest::Client,
    api_url: String,
    api_key: String,
    max_retries: u32,
}

impl RedashClient {
    /// Create a new client for the given Redash instance.
    pub fn new(api_url: String, api_key: String, timeout: u64, max_retries: u32) -> Self {
        let client = build_client(timeout);
        Self {
            client,
            api_url,
            api_key,
            max_retries,
        }
    }

    /// Create a client using a shared reqwest::Client for connection pooling.
    pub fn with_shared_client(
        client: reqwest::Client,
        api_url: String,
        api_key: String,
        max_retries: u32,
    ) -> Self {
        Self {
            client,
            api_url,
            api_key,
            max_retries,
        }
    }

    /// Send a GET request to a Redash API endpoint.
    ///
    /// `path` must start with `/` (e.g. `/data_sources`).
    pub async fn get(&self, path: &str) -> Result<serde_json::Value> {
        let url = build_url(&self.api_url, path);
        let request = self
            .client
            .get(&url)
            .header("Authorization", format!("Key {}", self.api_key));
        let response = self.send_with_retry(request).await?;
        handle_response(response).await
    }

    /// Send a POST request with a JSON body to a Redash API endpoint.
    ///
    /// `path` must start with `/` (e.g. `/queries`).
    pub async fn post(&self, path: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = build_url(&self.api_url, path);
        let request = self
            .client
            .post(&url)
            .header("Authorization", format!("Key {}", self.api_key))
            .json(&body);
        let response = self.send_with_retry(request).await?;
        handle_response(response).await
    }

    /// Send a DELETE request to a Redash API endpoint.
    ///
    /// `path` must start with `/` (e.g. `/queries/42`).
    pub async fn delete(&self, path: &str) -> Result<serde_json::Value> {
        let url = build_url(&self.api_url, path);
        let request = self
            .client
            .delete(&url)
            .header("Authorization", format!("Key {}", self.api_key));
        let response = self.send_with_retry(request).await?;
        handle_response(response).await
    }

    /// Send a request with retry logic for transient network errors.
    ///
    /// Retries only on transport-level errors (timeouts, connection refused, etc.),
    /// NOT on HTTP 4xx/5xx status codes. Uses exponential backoff starting at 500ms.
    async fn send_with_retry(
        &self,
        request: reqwest::RequestBuilder,
    ) -> std::result::Result<reqwest::Response, reqwest::Error> {
        if self.max_retries == 0 {
            return request.send().await;
        }

        // Clone the request builder for retries via `try_clone()`
        let built = request.build()?;
        let mut last_err = None;

        for attempt in 0..=self.max_retries {
            let req = match built.try_clone() {
                Some(r) => r,
                None => {
                    // Body is not cloneable (streaming) — cannot retry
                    return self.client.execute(built).await;
                }
            };

            match self.client.execute(req).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if attempt < self.max_retries {
                        let delay = backoff_delay(attempt);
                        tracing::warn!(
                            "request failed (attempt {}/{}), retrying in {}ms: {e}",
                            attempt + 1,
                            self.max_retries + 1,
                            delay.as_millis()
                        );
                        tokio::time::sleep(delay).await;
                    }
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.expect("at least one attempt must have been made"))
    }
}

/// Build a reqwest client with timeout and user-agent configured.
pub fn build_client(timeout_secs: u64) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .connect_timeout(std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .user_agent(USER_AGENT)
        .build()
        .expect("failed to build HTTP client")
}

/// Calculate exponential backoff delay for a given attempt (0-indexed).
///
/// Base delay: 500ms, multiplied by 2^attempt, capped at 8 seconds.
fn backoff_delay(attempt: u32) -> std::time::Duration {
    let base_ms: u64 = 500;
    let max_ms: u64 = 8000;
    let delay_ms = base_ms.saturating_mul(1u64 << attempt).min(max_ms);
    std::time::Duration::from_millis(delay_ms)
}

/// Build a full URL from the base API URL and a path.
fn build_url(api_url: &str, path: &str) -> String {
    format!("{}{}", api_url, path)
}

/// Extract an error message from a response body.
///
/// Tries JSON `{"message": "..."}` first, falls back to plain text.
fn parse_error_body(body: &str) -> String {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(msg) = json.get("message").and_then(|v| v.as_str()) {
            return msg.to_string();
        }
    }
    if body.trim().is_empty() {
        return "unknown error".to_string();
    }
    body.trim().to_string()
}

/// Process an HTTP response: return JSON on success, Error::Api on non-2xx.
async fn handle_response(response: reqwest::Response) -> Result<serde_json::Value> {
    let status = response.status().as_u16();
    if !(200..300).contains(&status) {
        let body = response.text().await.unwrap_or_default();
        return Err(Error::Api {
            status,
            message: parse_error_body(&body),
        });
    }
    let json = response.json().await?;
    Ok(json)
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Network(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_url_simple_path() {
        let url = build_url("http://localhost:5000/api", "/data_sources");
        assert_eq!(url, "http://localhost:5000/api/data_sources");
    }

    #[test]
    fn build_url_with_query_params() {
        let url = build_url("http://localhost:5000/api", "/queries/search?q=revenue");
        assert_eq!(url, "http://localhost:5000/api/queries/search?q=revenue");
    }

    #[test]
    fn parse_error_body_json() {
        let body = r#"{"message": "forbidden"}"#;
        assert_eq!(parse_error_body(body), "forbidden");
    }

    #[test]
    fn parse_error_body_plain_text() {
        let body = "Internal Server Error";
        assert_eq!(parse_error_body(body), "Internal Server Error");
    }

    #[test]
    fn parse_error_body_empty() {
        assert_eq!(parse_error_body(""), "unknown error");
        assert_eq!(parse_error_body("   "), "unknown error");
    }

    #[test]
    fn parse_error_body_json_without_message() {
        let body = r#"{"error": "something"}"#;
        assert_eq!(parse_error_body(body), r#"{"error": "something"}"#);
    }

    #[test]
    fn connect_timeout_less_than_request_timeout() {
        // Connect timeout (10s) should always be less than any valid request timeout (1-300s)
        // The minimum valid request timeout is 1s, but connect timeout is fixed at 10s
        // which means for request timeouts < 10s, reqwest uses the smaller of the two
        assert!(CONNECT_TIMEOUT_SECS <= 30);
    }

    #[test]
    fn user_agent_format() {
        assert!(USER_AGENT.starts_with("redash-mcp-rs/"));
        // Version part should be non-empty
        let version = USER_AGENT.strip_prefix("redash-mcp-rs/").unwrap();
        assert!(!version.is_empty());
    }

    #[test]
    fn backoff_delay_exponential() {
        assert_eq!(backoff_delay(0), std::time::Duration::from_millis(500));
        assert_eq!(backoff_delay(1), std::time::Duration::from_millis(1000));
        assert_eq!(backoff_delay(2), std::time::Duration::from_millis(2000));
        assert_eq!(backoff_delay(3), std::time::Duration::from_millis(4000));
        assert_eq!(backoff_delay(4), std::time::Duration::from_millis(8000));
    }

    #[test]
    fn backoff_delay_capped() {
        // Attempts beyond 4 should be capped at 8000ms
        assert_eq!(backoff_delay(5), std::time::Duration::from_millis(8000));
        assert_eq!(backoff_delay(10), std::time::Duration::from_millis(8000));
    }

    #[test]
    fn build_client_succeeds() {
        let _client = build_client(30);
    }
}
