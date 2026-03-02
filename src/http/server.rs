use super::auth::RateLimiter;
use super::handler;
use super::session::SessionStore;
use crate::config::HttpConfig;
use crate::error::{Error, Result};
use hyper_util::rt::TokioIo;
use std::sync::Arc;

/// Shared application state for the HTTP server.
pub struct AppState {
    /// HTTP configuration.
    pub config: HttpConfig,
    /// Session store.
    pub sessions: SessionStore,
    /// Rate limiter.
    pub rate_limiter: RateLimiter,
    /// Shared reqwest client for connection pooling.
    pub shared_client: reqwest::Client,
}

/// Run the HTTP server.
pub async fn run(config: HttpConfig) -> Result<()> {
    let addr = format!("{}:{}", config.host, config.port);

    let state = Arc::new(AppState {
        sessions: SessionStore::new(config.session_timeout),
        rate_limiter: RateLimiter::new(config.rate_limit),
        shared_client: reqwest::Client::new(),
        config,
    });

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| Error::Transport(format!("failed to bind to {addr}: {e}")))?;

    tracing::info!("MCP HTTP server listening on {addr}");

    // Spawn session cleanup task
    let cleanup_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            cleanup_state.sessions.cleanup().await;
        }
    });

    // Accept connections
    loop {
        let (stream, peer_addr) = listener
            .accept()
            .await
            .map_err(|e| Error::Transport(format!("accept error: {e}")))?;

        let io = TokioIo::new(stream);
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            let service = hyper::service::service_fn(move |req| {
                let state = Arc::clone(&state);
                async move {
                    Ok::<_, std::convert::Infallible>(handler::handle_request(req, state).await)
                }
            });

            if let Err(e) = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .await
            {
                tracing::warn!("connection error from {peer_addr}: {e}");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_state_creation() {
        let config = HttpConfig {
            api_url: "http://localhost:5000/api".into(),
            host: "127.0.0.1".into(),
            port: 3000,
            max_body_size: 1048576,
            session_timeout: 1800,
            rate_limit: 60,
            auth_tokens: vec!["test-token".into()],
        };

        let state = AppState {
            sessions: SessionStore::new(config.session_timeout),
            rate_limiter: RateLimiter::new(config.rate_limit),
            shared_client: reqwest::Client::new(),
            config,
        };

        assert_eq!(state.config.port, 3000);
        assert_eq!(state.config.auth_tokens.len(), 1);
    }
}
