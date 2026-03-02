use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::RwLock;

/// Validate a Bearer token against the list of valid tokens.
///
/// Expects the header value in the form `Bearer <token>`.
pub fn validate_bearer_token(header_value: Option<&str>, valid_tokens: &[String]) -> bool {
    let value = match header_value {
        Some(v) => v,
        None => return false,
    };
    let token = match value.strip_prefix("Bearer ") {
        Some(t) => t.trim(),
        None => return false,
    };
    valid_tokens.iter().any(|t| t == token)
}

/// Sliding-window rate limiter keyed by IP address.
pub struct RateLimiter {
    requests: RwLock<HashMap<String, Vec<Instant>>>,
    max_per_minute: u64,
}

impl RateLimiter {
    /// Create a new rate limiter with the given per-minute limit.
    pub fn new(max_per_minute: u64) -> Self {
        Self {
            requests: RwLock::new(HashMap::new()),
            max_per_minute,
        }
    }

    /// Check whether a request from the given IP is allowed.
    ///
    /// Returns `true` if the request is within the rate limit.
    pub async fn check(&self, ip: &str) -> bool {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(60);
        let mut requests = self.requests.write().await;
        let entry = requests.entry(ip.to_string()).or_default();

        // Remove entries outside the sliding window
        entry.retain(|t| now.duration_since(*t) < window);

        if entry.len() as u64 >= self.max_per_minute {
            return false;
        }

        entry.push(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens() -> Vec<String> {
        vec!["valid-token-1".into(), "valid-token-2".into()]
    }

    #[test]
    fn valid_bearer_token() {
        assert!(validate_bearer_token(
            Some("Bearer valid-token-1"),
            &tokens()
        ));
    }

    #[test]
    fn valid_second_token() {
        assert!(validate_bearer_token(
            Some("Bearer valid-token-2"),
            &tokens()
        ));
    }

    #[test]
    fn invalid_token() {
        assert!(!validate_bearer_token(
            Some("Bearer wrong-token"),
            &tokens()
        ));
    }

    #[test]
    fn missing_bearer_prefix() {
        assert!(!validate_bearer_token(Some("valid-token-1"), &tokens()));
    }

    #[test]
    fn none_header() {
        assert!(!validate_bearer_token(None, &tokens()));
    }

    #[test]
    fn empty_bearer() {
        assert!(!validate_bearer_token(Some("Bearer "), &tokens()));
    }

    #[tokio::test]
    async fn rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(3);
        assert!(limiter.check("1.2.3.4").await);
        assert!(limiter.check("1.2.3.4").await);
        assert!(limiter.check("1.2.3.4").await);
    }

    #[tokio::test]
    async fn rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(2);
        assert!(limiter.check("1.2.3.4").await);
        assert!(limiter.check("1.2.3.4").await);
        assert!(!limiter.check("1.2.3.4").await);
    }

    #[tokio::test]
    async fn rate_limiter_per_ip() {
        let limiter = RateLimiter::new(1);
        assert!(limiter.check("1.1.1.1").await);
        assert!(limiter.check("2.2.2.2").await);
        assert!(!limiter.check("1.1.1.1").await);
    }
}
