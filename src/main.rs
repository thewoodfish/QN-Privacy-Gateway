//! Service entry point and runtime setup.

mod cache;
mod config;
mod dashboard;
mod log_events;
mod metrics;
mod normalize;
mod privacy_mode;
mod proxy;
mod server;

use crate::config::Config;
use crate::metrics::Metrics;
use crate::server::build_router;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Load .env file if it exists
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Config::from_env();
    let metrics = Metrics::new();

    let bind_addr = config
        .bind_addr
        .parse::<SocketAddr>()
        .expect("invalid bind address");

    let app: Router = build_router(Arc::new(config), Arc::new(metrics));

    tracing::info!(%bind_addr, "starting server");

    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .expect("failed to bind");

    axum::serve(listener, app).await.expect("server error");
}
