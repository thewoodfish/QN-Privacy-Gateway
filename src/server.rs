//! HTTP routing and request handlers.

use crate::cache::Cache;
use crate::config::Config;
use crate::dashboard::dashboard_routes;
use crate::log_events::LogState;
use crate::metrics::Metrics;
use crate::proxy::handle_rpc_request;
use axum::extract::ws::WebSocketUpgrade;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub cache: Arc<Cache>,
    pub metrics: Arc<Metrics>,
    pub log_state: Arc<LogState>,
    pub client: Client,
}

pub fn build_router(config: Arc<Config>, metrics: Arc<Metrics>) -> Router {
    // One shared HTTP client for upstream requests to reuse connections.
    let client = Client::builder()
        .timeout(config.request_timeout)
        .build()
        .expect("failed to build http client");

    // In-memory cache keyed by normalized request hash.
    let cache = Arc::new(Cache::new(config.cache_ttl));
    // Log buffer + broadcaster for dashboard SSE.
    let log_state = Arc::new(LogState::new(1500, 1024));

    let state = AppState {
        config,
        cache,
        metrics,
        log_state,
        client,
    };

    // Main API routes plus optional dashboard assets.
    Router::new()
        .route("/", post(rpc_handler))
        .route("/ws", get(ws_handler))
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .merge(dashboard_routes())
        .with_state(state)
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

async fn metrics_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(state.metrics.snapshot().await)
}

async fn rpc_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    handle_rpc_request(state, payload)
        .await
        .map(Json)
        .map_err(|err| (StatusCode::BAD_GATEWAY, Json(json!({ "error": err }))))
}

async fn ws_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    if state.config.quicknode_ws_url.is_none() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "QUICKNODE_WS_URL not configured" })),
        ));
    }

    Ok(ws.on_upgrade(move |socket| crate::proxy::handle_ws_proxy(state, socket)))
}
