use bytes::Bytes;
use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

/// An active MCP session.
struct Session {
    api_key: String,
    last_active: Instant,
}

/// Thread-safe session store with timeout-based expiry.
pub struct SessionStore {
    sessions: RwLock<HashMap<String, Session>>,
    sse_senders: RwLock<HashMap<String, Vec<mpsc::Sender<Bytes>>>>,
    timeout_secs: u64,
}

impl SessionStore {
    /// Create a new session store with the given timeout in seconds.
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            sse_senders: RwLock::new(HashMap::new()),
            timeout_secs,
        }
    }

    /// Create a new session, returning its unique ID.
    pub async fn create(&self, api_key: String) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let session = Session {
            api_key,
            last_active: Instant::now(),
        };
        self.sessions.write().await.insert(id.clone(), session);
        id
    }

    /// Look up a session by ID, updating its last-active timestamp.
    /// Returns the associated API key if found and not expired.
    pub async fn get(&self, id: &str) -> Option<String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(id)?;
        if session.last_active.elapsed() > std::time::Duration::from_secs(self.timeout_secs) {
            sessions.remove(id);
            return None;
        }
        session.last_active = Instant::now();
        Some(session.api_key.clone())
    }

    /// Remove a session by ID and its associated SSE senders.
    pub async fn remove(&self, id: &str) {
        self.sessions.write().await.remove(id);
        self.sse_senders.write().await.remove(id);
    }

    /// Register an SSE sender for a session (GET /mcp).
    pub async fn register_sse(&self, session_id: &str, sender: mpsc::Sender<Bytes>) {
        self.sse_senders
            .write()
            .await
            .entry(session_id.to_string())
            .or_default()
            .push(sender);
    }

    /// Remove all expired sessions and clean up closed SSE senders.
    pub async fn cleanup(&self) {
        let timeout = self.timeout_secs;
        let expired: Vec<String> = {
            let sessions = self.sessions.read().await;
            sessions
                .iter()
                .filter(|(_, s)| s.last_active.elapsed() > std::time::Duration::from_secs(timeout))
                .map(|(k, _)| k.clone())
                .collect()
        };

        if !expired.is_empty() {
            let mut sessions = self.sessions.write().await;
            let mut senders = self.sse_senders.write().await;
            for id in &expired {
                sessions.remove(id);
                senders.remove(id);
            }
        }

        // Clean up closed SSE channels for active sessions
        let mut senders = self.sse_senders.write().await;
        senders.retain(|_, v| {
            v.retain(|tx| !tx.is_closed());
            !v.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_and_get_session() {
        let store = SessionStore::new(1800);
        let id = store.create("test-key".into()).await;
        let key = store.get(&id).await;
        assert_eq!(key, Some("test-key".to_string()));
    }

    #[tokio::test]
    async fn get_nonexistent_returns_none() {
        let store = SessionStore::new(1800);
        assert!(store.get("nonexistent").await.is_none());
    }

    #[tokio::test]
    async fn remove_session() {
        let store = SessionStore::new(1800);
        let id = store.create("test-key".into()).await;
        store.remove(&id).await;
        assert!(store.get(&id).await.is_none());
    }

    #[tokio::test]
    async fn expired_session_returns_none() {
        let store = SessionStore::new(0);
        let id = store.create("test-key".into()).await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        assert!(store.get(&id).await.is_none());
    }

    #[tokio::test]
    async fn cleanup_removes_expired() {
        let store = SessionStore::new(0);
        store.create("key1".into()).await;
        store.create("key2".into()).await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        store.cleanup().await;
        assert_eq!(store.sessions.read().await.len(), 0);
    }

    #[tokio::test]
    async fn unique_session_ids() {
        let store = SessionStore::new(1800);
        let id1 = store.create("key".into()).await;
        let id2 = store.create("key".into()).await;
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn register_sse_sender() {
        let store = SessionStore::new(1800);
        let id = store.create("key".into()).await;
        let (tx, _rx) = mpsc::channel(1);
        store.register_sse(&id, tx).await;
        assert_eq!(store.sse_senders.read().await.get(&id).unwrap().len(), 1);
    }

    #[tokio::test]
    async fn remove_session_cleans_sse_senders() {
        let store = SessionStore::new(1800);
        let id = store.create("key".into()).await;
        let (tx, _rx) = mpsc::channel(1);
        store.register_sse(&id, tx).await;
        store.remove(&id).await;
        assert!(store.sse_senders.read().await.get(&id).is_none());
    }

    #[tokio::test]
    async fn cleanup_removes_closed_sse_senders() {
        let store = SessionStore::new(1800);
        let id = store.create("key".into()).await;
        let (tx, rx) = mpsc::channel(1);
        store.register_sse(&id, tx).await;
        drop(rx); // Close the channel
        store.cleanup().await;
        assert!(store.sse_senders.read().await.get(&id).is_none());
    }

    #[tokio::test]
    async fn cleanup_keeps_open_sse_senders() {
        let store = SessionStore::new(1800);
        let id = store.create("key".into()).await;
        let (tx, _rx) = mpsc::channel(1);
        store.register_sse(&id, tx).await;
        store.cleanup().await;
        assert_eq!(store.sse_senders.read().await.get(&id).unwrap().len(), 1);
    }
}
