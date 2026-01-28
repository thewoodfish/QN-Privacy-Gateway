//! Lightweight in-process metrics tracking.

use serde_json::json;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

pub struct Metrics {
    requests_total: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    unique_hashes: RwLock<HashSet<String>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            requests_total: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            unique_hashes: RwLock::new(HashSet::new()),
        }
    }

    pub async fn record_request(&self, hash: String) {
        // Track total requests and approximate uniqueness by hash.
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        let mut guard = self.unique_hashes.write().await;
        guard.insert(hash);
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn snapshot(&self) -> Value {
        // Snapshot is intentionally lightweight for the dashboard polling.
        let unique_request_hashes = self.unique_hashes.read().await.len();

        json!({
            "requests_total": self.requests_total.load(Ordering::Relaxed),
            "cache_hits": self.cache_hits.load(Ordering::Relaxed),
            "cache_misses": self.cache_misses.load(Ordering::Relaxed),
            "unique_request_hashes": unique_request_hashes
        })
    }
}
