//! Runtime configuration sourced from environment variables.

use crate::privacy_mode::PrivacyMode;
use std::env;
use std::time::Duration;

#[derive(Clone)]
pub struct Config {
    pub quicknode_url: String,
    pub quicknode_ws_url: Option<String>,
    pub privacy_mode: PrivacyMode,
    pub cache_ttl: Duration,
    pub request_timeout: Duration,
    pub retry_attempts: usize,
    pub bind_addr: String,
}

impl Config {
    pub fn from_env() -> Self {
        let quicknode_url = env::var("QUICKNODE_RPC_URL").expect("QUICKNODE_RPC_URL must be set");

        let privacy_mode = env::var("PRIVACY_MODE")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(PrivacyMode::Balanced);

        let quicknode_ws_url = env::var("QUICKNODE_WS_URL").ok();

        let cache_ttl_seconds: u64 = env::var("CACHE_TTL_SECONDS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(5);

        let request_timeout_ms: u64 = env::var("REQUEST_TIMEOUT_MS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(8_000);

        let retry_attempts: usize = env::var("RETRY_ATTEMPTS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(3);

        let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

        Self {
            quicknode_url,
            quicknode_ws_url,
            privacy_mode,
            cache_ttl: Duration::from_secs(cache_ttl_seconds),
            request_timeout: Duration::from_millis(request_timeout_ms),
            retry_attempts,
            bind_addr,
        }
    }
}
