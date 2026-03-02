use crate::error::{Error, Result};

/// HTTP client for the Redash API.
///
/// Wraps `reqwest::Client` with base URL and API key authentication.
/// All requests use the `Authorization: Key <api_key>` header format.
pub struct RedashClient {
    client: reqwest::Client,
    api_url: String,
    api_key: String,
}

impl RedashClient {
    /// Create a new client for the given Redash instance.
    pub fn new(api_url: String, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_url,
            api_key,
        }
    }

    /// Create a client using a shared reqwest::Client for connection pooling.
    pub fn with_shared_client(client: reqwest::Client, api_url: String, api_key: String) -> Self {
        Self {
            client,
            api_url,
            api_key,
        }
    }

    /// Send a GET request to a Redash API endpoint.
    ///
    /// `path` must start with `/` (e.g. `/data_sources`).
    pub async fn get(&self, path: &str) -> Result<serde_json::Value> {
        let url = build_url(&self.api_url, path);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Key {}", self.api_key))
            .send()
            .await?;

        handle_response(response).await
    }

    /// Send a POST request with a JSON body to a Redash API endpoint.
    ///
    /// `path` must start with `/` (e.g. `/queries`).
    pub async fn post(&self, path: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = build_url(&self.api_url, path);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Key {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        handle_response(response).await
    }

    /// Send a DELETE request to a Redash API endpoint.
    ///
    /// `path` must start with `/` (e.g. `/queries/42`).
    pub async fn delete(&self, path: &str) -> Result<serde_json::Value> {
        let url = build_url(&self.api_url, path);
        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Key {}", self.api_key))
            .send()
            .await?;

        handle_response(response).await
    }
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
}
