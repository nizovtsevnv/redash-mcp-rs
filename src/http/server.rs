use super::auth::RateLimiter;
use super::handler;
use super::session::SessionStore;
use crate::config::HttpConfig;
use crate::error::{Error, Result};
use crate::redash;
use hyper_util::rt::TokioIo;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Maximum time to wait for active connections to drain during shutdown.
const DRAIN_TIMEOUT_SECS: u64 = 10;

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
    /// Number of active connections.
    pub active_connections: AtomicUsize,
}

/// Run the HTTP server.
pub async fn run(config: HttpConfig) -> Result<()> {
    let addr = format!("{}:{}", config.host, config.port);

    let state = Arc::new(AppState {
        sessions: SessionStore::new(config.session_timeout),
        rate_limiter: RateLimiter::new(config.rate_limit),
        shared_client: redash::build_client(config.timeout),
        config,
        active_connections: AtomicUsize::new(0),
    });

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| Error::Transport(format!("failed to bind to {addr}: {e}")))?;

    tracing::info!("MCP HTTP server listening on {addr}");

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // Spawn session cleanup task
    let cleanup_state = Arc::clone(&state);
    let mut cleanup_rx = shutdown_tx.subscribe();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    cleanup_state.sessions.cleanup().await;
                }
                _ = cleanup_rx.changed() => {
                    break;
                }
            }
        }
    });

    // Accept connections until shutdown signal
    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, peer_addr) = result
                    .map_err(|e| Error::Transport(format!("accept error: {e}")))?;

                let io = TokioIo::new(stream);
                let state = Arc::clone(&state);

                state.active_connections.fetch_add(1, Ordering::Relaxed);

                let conn_state = Arc::clone(&state);
                let counter_state = Arc::clone(&state);
                tokio::spawn(async move {
                    let service = hyper::service::service_fn(move |req| {
                        let state = Arc::clone(&conn_state);
                        async move {
                            Ok::<_, std::convert::Infallible>(
                                handler::handle_request(req, state).await,
                            )
                        }
                    });

                    if let Err(e) = hyper::server::conn::http1::Builder::new()
                        .serve_connection(io, service)
                        .await
                    {
                        tracing::warn!("connection error from {peer_addr}: {e}");
                    }

                    counter_state
                        .active_connections
                        .fetch_sub(1, Ordering::Relaxed);
                });
            }
            _ = shutdown_signal() => {
                tracing::info!("shutdown signal received, stopping accept loop");
                break;
            }
        }
    }

    // Signal background tasks to stop
    let _ = shutdown_tx.send(true);
    drop(shutdown_rx);

    // Wait for active connections to drain
    let drain_deadline =
        tokio::time::Instant::now() + std::time::Duration::from_secs(DRAIN_TIMEOUT_SECS);
    loop {
        let active = state.active_connections.load(Ordering::Relaxed);
        if active == 0 {
            tracing::info!("all connections drained, shutting down");
            break;
        }
        if tokio::time::Instant::now() >= drain_deadline {
            tracing::warn!(
                "drain timeout reached with {active} active connections, forcing shutdown"
            );
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    Ok(())
}

/// Wait for a shutdown signal (Ctrl+C or SIGTERM on Unix).
async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to register SIGTERM handler");
        tokio::select! {
            _ = ctrl_c => {}
            _ = sigterm.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        ctrl_c.await.ok();
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
            timeout: 30,
            max_retries: 2,
        };

        let state = AppState {
            sessions: SessionStore::new(config.session_timeout),
            rate_limiter: RateLimiter::new(config.rate_limit),
            shared_client: redash::build_client(config.timeout),
            config,
            active_connections: AtomicUsize::new(0),
        };

        assert_eq!(state.config.port, 3000);
        assert_eq!(state.config.auth_tokens.len(), 1);
    }

    #[test]
    fn connection_counter_increment_decrement() {
        let counter = AtomicUsize::new(0);
        counter.fetch_add(1, Ordering::Relaxed);
        counter.fetch_add(1, Ordering::Relaxed);
        assert_eq!(counter.load(Ordering::Relaxed), 2);
        counter.fetch_sub(1, Ordering::Relaxed);
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn drain_timeout_constant() {
        assert_eq!(DRAIN_TIMEOUT_SECS, 10);
    }
}
