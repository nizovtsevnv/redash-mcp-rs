use crate::error::{Error, Result};

const DEFAULT_API_URL: &str = "http://localhost:5000/api";

/// Configuration for the STDIO transport mode.
pub struct StdioConfig {
    /// Redash API key (from `REDASH_API_KEY`).
    pub api_key: String,
    /// Redash API base URL (from `REDASH_API_URL`).
    pub api_url: String,
}

/// Load STDIO configuration from environment variables.
///
/// Requires `REDASH_API_KEY`. Uses `REDASH_API_URL` with a default
/// of `http://localhost:5000/api` if not set.
pub fn load_stdio_config() -> Result<StdioConfig> {
    let api_key = std::env::var("REDASH_API_KEY")
        .map_err(|_| Error::Config("REDASH_API_KEY environment variable is not set".into()))?;
    let api_key = validate_api_key(&api_key)?;

    let raw_url = std::env::var("REDASH_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.into());
    let api_url = normalize_api_url(&raw_url)?;

    Ok(StdioConfig { api_key, api_url })
}

/// Validate that an API key is not empty or whitespace-only.
fn validate_api_key(key: &str) -> Result<String> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err(Error::Config("REDASH_API_KEY must not be empty".into()));
    }
    Ok(trimmed.to_string())
}

/// Parse and normalize an API URL: validate format and strip trailing slash.
fn normalize_api_url(raw: &str) -> Result<String> {
    let parsed =
        url::Url::parse(raw).map_err(|e| Error::Config(format!("invalid REDASH_API_URL: {e}")))?;

    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(Error::Config(format!(
            "REDASH_API_URL must use http or https scheme, got: {}",
            parsed.scheme()
        )));
    }

    let url_str = parsed.as_str().trim_end_matches('/');
    Ok(url_str.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_url_strips_trailing_slash() {
        let result = normalize_api_url("http://localhost:5000/api/").unwrap();
        assert_eq!(result, "http://localhost:5000/api");
    }

    #[test]
    fn normalize_url_preserves_clean_url() {
        let result = normalize_api_url("https://redash.example.com/api").unwrap();
        assert_eq!(result, "https://redash.example.com/api");
    }

    #[test]
    fn normalize_url_rejects_invalid() {
        let result = normalize_api_url("not-a-url");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid REDASH_API_URL"));
    }

    #[test]
    fn normalize_url_rejects_non_http_scheme() {
        let result = normalize_api_url("ftp://example.com/api");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("http or https"));
    }

    #[test]
    fn validate_key_rejects_empty() {
        let result = validate_api_key("");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must not be empty"));
    }

    #[test]
    fn validate_key_rejects_whitespace() {
        let result = validate_api_key("   \t\n  ");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must not be empty"));
    }

    #[test]
    fn validate_key_trims_and_accepts() {
        let result = validate_api_key("  my-key-123  ").unwrap();
        assert_eq!(result, "my-key-123");
    }
}
