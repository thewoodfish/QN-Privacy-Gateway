//! Proxy logic for forwarding requests and applying privacy features.

use crate::log_events::LogEvent;
use crate::normalize::{normalize_for_mode, normalize_rpc_request};
use crate::server::AppState;
use axum::extract::ws::{Message as AxumMessage, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

const BASE_BACKOFF_MS: u64 = 100;

pub async fn handle_rpc_request(state: AppState, payload: Value) -> Result<Value, String> {
    let start = Instant::now();

    // Pull method early for routing, caching, and logging.
    let method = payload
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let mode = state.config.privacy_mode;

    // Normalize for deterministic hashing, separate from outbound normalization.
    let normalized_for_hash = normalize_for_mode(mode, payload.clone());
    let request_hash = hash_value(&normalized_for_hash)?;

    state.metrics.record_request(request_hash.clone()).await;
    tracing::info!(method = %method, hash = %request_hash, "incoming request");
    state
        .log_state
        .record(LogEvent::new("INFO", "REQ_IN").with_method(method.clone()))
        .await;

    // Cache lookup only for safe read methods.
    if mode.should_cache(&method) {
        if let Some(cached) = state.cache.get(&request_hash).await {
            state.metrics.record_cache_hit();
            tracing::info!(method = %method, hash = %request_hash, "cache hit");
            state
                .log_state
                .record(
                    LogEvent::new("INFO", "CACHE_HIT")
                        .with_hash(request_hash.clone())
                        .with_method(method.clone()),
                )
                .await;
            return Ok(cached);
        }

        state.metrics.record_cache_miss();
        tracing::info!(method = %method, hash = %request_hash, "cache miss");
        state
            .log_state
            .record(
                LogEvent::new("INFO", "CACHE_MISS")
                    .with_hash(request_hash.clone())
                    .with_method(method.clone()),
            )
            .await;
    }

    // Normalize outbound request body when privacy mode allows.
    let outbound_payload = if mode.should_normalize_outbound() {
        normalize_rpc_request(payload)
    } else {
        payload
    };

    state
        .log_state
        .record(
            LogEvent::new("INFO", "NORMALIZED")
                .with_hash(request_hash.clone())
                .with_method(method.clone()),
        )
        .await;

    tracing::info!(method = %method, hash = %request_hash, "forwarding request");
    state
        .log_state
        .record(
            LogEvent::new("INFO", "FORWARDED")
                .with_hash(request_hash.clone())
                .with_method(method.clone()),
        )
        .await;

    // Forward upstream with bounded retries and backoff.
    let response = match send_with_retries(
        &state.client,
        &state.config.quicknode_url,
        outbound_payload,
        state.config.retry_attempts,
    )
    .await
    {
        Ok(value) => value,
        Err(err) => {
            state
                .log_state
                .record(
                    LogEvent::new("ERROR", "ERR")
                        .with_hash(request_hash.clone())
                        .with_method(method.clone())
                        .with_note(err.clone()),
                )
                .await;
            return Err(err);
        }
    };

    // Populate cache on successful responses only.
    if mode.should_cache(&method) {
        state
            .cache
            .insert(request_hash.clone(), response.clone())
            .await;
    }

    let elapsed = start.elapsed();
    tracing::info!(method = %method, hash = %request_hash, elapsed_ms = elapsed.as_millis(), "response completed");
    state
        .log_state
        .record(
            LogEvent::new("INFO", "RESP_OUT")
                .with_hash(request_hash)
                .with_method(method.clone())
                .with_latency(elapsed.as_millis() as u64),
        )
        .await;

    Ok(response)
}

pub async fn handle_ws_proxy(state: AppState, socket: WebSocket) {
    let ws_url = match state.config.quicknode_ws_url.clone() {
        Some(url) => url,
        None => {
            tracing::warn!("websocket upgrade attempted without QUICKNODE_WS_URL");
            return;
        }
    };

    let upstream = connect_async(&ws_url).await;
    let (upstream_ws, _) = match upstream {
        Ok(result) => result,
        Err(err) => {
            tracing::error!(error = %err, "failed to connect to upstream websocket");
            return;
        }
    };

    let (mut client_tx, mut client_rx) = socket.split();
    let (mut upstream_tx, mut upstream_rx) = upstream_ws.split();

    let client_to_upstream = async {
        while let Some(msg) = client_rx.next().await {
            match msg {
                Ok(message) => {
                    if let Some(mapped) = map_to_upstream(message) {
                        if upstream_tx.send(mapped).await.is_err() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                Err(err) => {
                    tracing::warn!(error = %err, "websocket client error");
                    break;
                }
            }
        }
    };

    let upstream_to_client = async {
        while let Some(msg) = upstream_rx.next().await {
            match msg {
                Ok(message) => {
                    if let Some(mapped) = map_from_upstream(message) {
                        if client_tx.send(mapped).await.is_err() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                Err(err) => {
                    tracing::warn!(error = %err, "websocket upstream error");
                    break;
                }
            }
        }
    };

    tokio::select! {
        _ = client_to_upstream => {},
        _ = upstream_to_client => {},
    }
}

fn map_to_upstream(message: AxumMessage) -> Option<TungsteniteMessage> {
    match message {
        AxumMessage::Text(text) => Some(TungsteniteMessage::Text(text)),
        AxumMessage::Binary(bin) => Some(TungsteniteMessage::Binary(bin)),
        AxumMessage::Ping(payload) => Some(TungsteniteMessage::Ping(payload)),
        AxumMessage::Pong(payload) => Some(TungsteniteMessage::Pong(payload)),
        AxumMessage::Close(_) => Some(TungsteniteMessage::Close(None)),
    }
}

fn map_from_upstream(message: TungsteniteMessage) -> Option<AxumMessage> {
    match message {
        TungsteniteMessage::Text(text) => Some(AxumMessage::Text(text)),
        TungsteniteMessage::Binary(bin) => Some(AxumMessage::Binary(bin)),
        TungsteniteMessage::Ping(payload) => Some(AxumMessage::Ping(payload)),
        TungsteniteMessage::Pong(payload) => Some(AxumMessage::Pong(payload)),
        TungsteniteMessage::Close(_) => Some(AxumMessage::Close(None)),
        TungsteniteMessage::Frame(_) => None,
    }
}

async fn send_with_retries(
    client: &reqwest::Client,
    url: &str,
    payload: Value,
    attempts: usize,
) -> Result<Value, String> {
    let mut last_err = None;

    for attempt in 0..attempts.max(1) {
        // Each attempt is a fresh POST with the same payload.
        let response = client.post(url).json(&payload).send().await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                if status.is_server_error() {
                    last_err = Some(format!("upstream server error: {}", status));
                } else if status.is_client_error() {
                    return Err(format!("upstream client error: {}", status));
                } else {
                    return resp.json::<Value>().await.map_err(|err| err.to_string());
                }
            }
            Err(err) => {
                last_err = Some(err.to_string());
            }
        }

        // Linear backoff to reduce upstream pressure.
        let backoff = Duration::from_millis(BASE_BACKOFF_MS * (attempt as u64 + 1));
        sleep(backoff).await;
    }

    Err(last_err.unwrap_or_else(|| "request failed".to_string()))
}

fn hash_value(value: &Value) -> Result<String, String> {
    // Hash the canonical JSON string for cache key stability.
    let payload = serde_json::to_string(value).map_err(|err| err.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    Ok(hex::encode(hasher.finalize()))
}
