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
pub(crate) fn validate_api_key(key: &str) -> Result<String> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err(Error::Config("REDASH_API_KEY must not be empty".into()));
    }
    Ok(trimmed.to_string())
}

/// Parse and normalize an API URL: validate format and strip trailing slash.
pub(crate) fn normalize_api_url(raw: &str) -> Result<String> {
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

/// Configuration for the HTTP transport mode.
#[derive(Debug)]
pub struct HttpConfig {
    /// Redash API base URL (from `REDASH_API_URL`).
    pub api_url: String,
    /// HTTP server host (from `MCP_HOST`, default: `127.0.0.1`).
    pub host: String,
    /// HTTP server port (from `MCP_PORT`, default: `3000`).
    pub port: u16,
    /// Maximum request body size in bytes (from `MCP_MAX_BODY_SIZE`, default: 1MB).
    pub max_body_size: usize,
    /// Session timeout in seconds (from `MCP_SESSION_TIMEOUT`, default: 1800).
    pub session_timeout: u64,
    /// Rate limit: max requests per minute per IP (from `MCP_RATE_LIMIT`, default: 60).
    pub rate_limit: u64,
    /// Bearer auth tokens (from `MCP_AUTH_TOKENS`, comma-separated, required).
    pub auth_tokens: Vec<String>,
}

/// Load HTTP configuration from environment variables.
///
/// Requires `REDASH_API_URL` and `MCP_AUTH_TOKENS`.
pub fn load_http_config() -> Result<HttpConfig> {
    let raw_url = std::env::var("REDASH_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.into());
    let api_url = normalize_api_url(&raw_url)?;

    let host = std::env::var("MCP_HOST").unwrap_or_else(|_| "127.0.0.1".into());

    let port_str = std::env::var("MCP_PORT").unwrap_or_else(|_| "3000".into());
    let port = parse_port(&port_str)?;

    let max_body_size = std::env::var("MCP_MAX_BODY_SIZE")
        .unwrap_or_else(|_| "1048576".into())
        .parse::<usize>()
        .map_err(|_| Error::Config("MCP_MAX_BODY_SIZE must be a valid integer".into()))?;

    let session_timeout = std::env::var("MCP_SESSION_TIMEOUT")
        .unwrap_or_else(|_| "1800".into())
        .parse::<u64>()
        .map_err(|_| Error::Config("MCP_SESSION_TIMEOUT must be a valid integer".into()))?;

    let rate_limit = std::env::var("MCP_RATE_LIMIT")
        .unwrap_or_else(|_| "60".into())
        .parse::<u64>()
        .map_err(|_| Error::Config("MCP_RATE_LIMIT must be a valid integer".into()))?;

    let tokens_raw = std::env::var("MCP_AUTH_TOKENS")
        .map_err(|_| Error::Config("MCP_AUTH_TOKENS environment variable is not set".into()))?;
    let auth_tokens = parse_auth_tokens(&tokens_raw)?;

    Ok(HttpConfig {
        api_url,
        host,
        port,
        max_body_size,
        session_timeout,
        rate_limit,
        auth_tokens,
    })
}

/// Parse and validate a port string: must be a valid u16 >= 1024.
pub(crate) fn parse_port(raw: &str) -> Result<u16> {
    let port = raw
        .parse::<u16>()
        .map_err(|_| Error::Config("MCP_PORT must be a valid port number".into()))?;
    if port < 1024 {
        return Err(Error::Config(
            "MCP_PORT must be >= 1024 (unprivileged)".into(),
        ));
    }
    Ok(port)
}

/// Parse comma-separated auth tokens, trimming whitespace and filtering empties.
pub(crate) fn parse_auth_tokens(raw: &str) -> Result<Vec<String>> {
    let tokens: Vec<String> = raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if tokens.is_empty() {
        return Err(Error::Config(
            "MCP_AUTH_TOKENS must contain at least one token".into(),
        ));
    }
    Ok(tokens)
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

    #[test]
    fn parse_port_valid() {
        assert_eq!(parse_port("3000").unwrap(), 3000);
        assert_eq!(parse_port("1024").unwrap(), 1024);
        assert_eq!(parse_port("65535").unwrap(), 65535);
    }

    #[test]
    fn parse_port_too_low() {
        let result = parse_port("80");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("1024"));
    }

    #[test]
    fn parse_port_invalid() {
        let result = parse_port("not-a-number");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("valid port"));
    }

    #[test]
    fn parse_auth_tokens_valid() {
        let tokens = parse_auth_tokens("token1,token2,token3").unwrap();
        assert_eq!(tokens, vec!["token1", "token2", "token3"]);
    }

    #[test]
    fn parse_auth_tokens_trims_whitespace() {
        let tokens = parse_auth_tokens("  tok1 , tok2  ").unwrap();
        assert_eq!(tokens, vec!["tok1", "tok2"]);
    }

    #[test]
    fn parse_auth_tokens_empty() {
        let result = parse_auth_tokens("  ,  , ");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least one token"));
    }

    #[test]
    fn parse_auth_tokens_single() {
        let tokens = parse_auth_tokens("only-one").unwrap();
        assert_eq!(tokens, vec!["only-one"]);
    }
}
