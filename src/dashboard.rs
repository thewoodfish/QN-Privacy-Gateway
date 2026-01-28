//! Dashboard UI handlers and SSE stream.

use crate::log_events::LogEvent;
use crate::server::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response, Sse};
use axum::routing::get;
use axum::Router;
use futures_util::stream::{self, StreamExt};
use serde_json::json;
use tokio_stream::wrappers::BroadcastStream;

pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(dashboard_handler))
        .route("/assets/dashboard.css", get(css_handler))
        .route("/assets/dashboard.js", get(js_handler))
        .route("/events", get(events_handler))
}

async fn dashboard_handler(State(state): State<AppState>) -> Html<String> {
    // Inject runtime privacy mode into the static template.
    let template = include_str!("../assets/dashboard.html");
    let html = template.replace("{{PRIVACY_MODE}}", &state.config.privacy_mode.to_string());
    Html(html)
}

async fn css_handler() -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/css; charset=utf-8"),
    );
    (headers, include_str!("../assets/dashboard.css")).into_response()
}

async fn js_handler() -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/javascript; charset=utf-8"),
    );
    (headers, include_str!("../assets/dashboard.js")).into_response()
}

async fn events_handler(
    State(state): State<AppState>,
) -> Result<
    Sse<impl futures_util::Stream<Item = Result<axum::response::sse::Event, axum::Error>>>,
    StatusCode,
> {
    // Replay recent events first to populate the UI.
    let history = state.log_state.recent(100).await;
    let history_stream = stream::iter(history.into_iter().map(|event| Ok(to_sse_event(event))));

    // Stream new events as they occur.
    let receiver = state.log_state.subscribe();
    let live_stream = BroadcastStream::new(receiver).filter_map(|msg| async move {
        match msg {
            Ok(event) => Some(Ok(to_sse_event(event))),
            Err(_) => None,
        }
    });

    let stream = history_stream.chain(live_stream);

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(10))
            .text("keepalive"),
    ))
}

fn to_sse_event(event: LogEvent) -> axum::response::sse::Event {
    let data = serde_json::to_string(&event).unwrap_or_else(|_| {
        json!({
            "ts": "",
            "level": "ERROR",
            "event": "ERR",
            "note": "failed to serialize log event"
        })
        .to_string()
    });
    axum::response::sse::Event::default()
        .event("log")
        .data(data)
}
