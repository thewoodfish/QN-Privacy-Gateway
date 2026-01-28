//! Structured log events, ring buffer, and broadcaster for the dashboard.

use chrono::Utc;
use serde::Serialize;
use std::collections::VecDeque;
use tokio::sync::{broadcast, RwLock};

#[derive(Clone, Debug, Serialize)]
pub struct LogEvent {
    pub ts: String,
    pub level: String,
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl LogEvent {
    pub fn new(level: &str, event: &str) -> Self {
        Self {
            ts: Utc::now().to_rfc3339(),
            level: level.to_string(),
            event: event.to_string(),
            request_hash: None,
            method: None,
            latency_ms: None,
            note: None,
        }
    }

    pub fn with_hash(mut self, hash: String) -> Self {
        self.request_hash = Some(hash);
        self
    }

    pub fn with_method(mut self, method: String) -> Self {
        self.method = Some(method);
        self
    }

    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}

pub struct LogState {
    capacity: usize,
    buffer: RwLock<VecDeque<LogEvent>>,
    sender: broadcast::Sender<LogEvent>,
}

impl LogState {
    pub fn new(capacity: usize, broadcast_capacity: usize) -> Self {
        // Capacity bounds memory usage for the dashboard ring buffer.
        let (sender, _) = broadcast::channel(broadcast_capacity);
        Self {
            capacity,
            buffer: RwLock::new(VecDeque::with_capacity(capacity)),
            sender,
        }
    }

    pub async fn record(&self, event: LogEvent) {
        // Push into ring buffer and fan out to all subscribers.
        {
            let mut guard = self.buffer.write().await;
            if guard.len() >= self.capacity {
                guard.pop_front();
            }
            guard.push_back(event.clone());
        }

        let _ = self.sender.send(event);
    }

    pub async fn recent(&self, limit: usize) -> Vec<LogEvent> {
        // Return a snapshot of the latest events.
        let guard = self.buffer.read().await;
        let start = guard.len().saturating_sub(limit);
        guard.iter().skip(start).cloned().collect()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LogEvent> {
        self.sender.subscribe()
    }
}
