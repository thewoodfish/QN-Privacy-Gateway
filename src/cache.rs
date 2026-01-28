//! In-memory TTL cache for safe RPC responses.

use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub struct Cache {
    ttl: Duration,
    inner: RwLock<HashMap<String, CacheEntry>>,
}

struct CacheEntry {
    value: Value,
    expires_at: Instant,
}

impl Cache {
    pub fn new(ttl: Duration) -> Self {
        // TTL applies uniformly to all cached responses.
        Self {
            ttl,
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Value> {
        // Opportunistically evict expired entries on access.
        let mut guard = self.inner.write().await;
        if let Some(entry) = guard.get(key) {
            if Instant::now() < entry.expires_at {
                return Some(entry.value.clone());
            }
        }

        guard.remove(key);
        None
    }

    pub async fn insert(&self, key: String, value: Value) {
        let entry = CacheEntry {
            value,
            expires_at: Instant::now() + self.ttl,
        };

        self.inner.write().await.insert(key, entry);
    }
}
