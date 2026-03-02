/// Unified error type for the redash-mcp-rs server.
///
/// Covers all failure modes: configuration, networking, Redash API,
/// MCP protocol, tool dispatch, and I/O transport.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Missing or invalid environment variable.
    #[error("config error: {0}")]
    Config(String),

    /// Network-level failure (DNS, timeout, connection refused).
    #[error("network error: {0}")]
    Network(String),

    /// Non-2xx response from the Redash API.
    #[error("api error (HTTP {status}): {message}")]
    Api { status: u16, message: String },

    /// Malformed JSON-RPC message.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// Tool-level error (unknown tool, missing/invalid arguments).
    #[error("tool error: {0}")]
    Tool(String),

    /// STDIO transport I/O failure.
    #[error("transport error: {0}")]
    Transport(String),
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_config_error() {
        let err = Error::Config("REDASH_API_KEY not set".into());
        assert_eq!(err.to_string(), "config error: REDASH_API_KEY not set");
    }

    #[test]
    fn display_network_error() {
        let err = Error::Network("connection refused".into());
        assert_eq!(err.to_string(), "network error: connection refused");
    }

    #[test]
    fn display_api_error() {
        let err = Error::Api {
            status: 403,
            message: "forbidden".into(),
        };
        assert_eq!(err.to_string(), "api error (HTTP 403): forbidden");
    }

    #[test]
    fn display_protocol_error() {
        let err = Error::Protocol("missing method field".into());
        assert_eq!(err.to_string(), "protocol error: missing method field");
    }

    #[test]
    fn display_tool_error() {
        let err = Error::Tool("unknown tool: foo".into());
        assert_eq!(err.to_string(), "tool error: unknown tool: foo");
    }

    #[test]
    fn display_transport_error() {
        let err = Error::Transport("broken pipe".into());
        assert_eq!(err.to_string(), "transport error: broken pipe");
    }
}
