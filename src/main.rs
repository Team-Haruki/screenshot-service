mod error;
mod request;
mod screenshot;

use std::{env, net::SocketAddr};

use axum::{
    Json, Router,
    body::Body,
    extract::{
        DefaultBodyLimit, Json as JsonExtractor, Query,
        rejection::{JsonRejection, QueryRejection},
    },
    http::{
        HeaderValue, StatusCode,
        header::{CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE},
    },
    response::{IntoResponse, Response},
    routing::get,
};
use serde_json::json;
use tokio::{net::TcpListener, signal};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    error::AppError,
    request::{ScreenshotQuery, ScreenshotRequest},
    screenshot::take_screenshot,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let app = Router::new()
        .route("/health", get(health))
        .route(
            "/screenshot",
            get(handle_screenshot_get).post(handle_screenshot_post),
        )
        .layer(DefaultBodyLimit::max(8 * 1024 * 1024))
        .layer(TraceLayer::new_for_http());

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{port}").parse()?;
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(%addr, "server starting");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

async fn handle_screenshot_get(
    query: Result<Query<ScreenshotQuery>, QueryRejection>,
) -> Result<Response, AppError> {
    let Query(query) = query.map_err(|error| AppError::bad_request(error.to_string()))?;
    let request = ScreenshotRequest::from_query(query).map_err(AppError::bad_request)?;
    process_screenshot(request).await
}

async fn handle_screenshot_post(
    payload: Result<JsonExtractor<ScreenshotRequest>, JsonRejection>,
) -> Result<Response, AppError> {
    let JsonExtractor(request) =
        payload.map_err(|error| AppError::bad_request(error.to_string()))?;
    process_screenshot(request).await
}

async fn process_screenshot(mut request: ScreenshotRequest) -> Result<Response, AppError> {
    request.apply_defaults();
    request
        .validate()
        .map_err(|error| AppError::bad_request(error.to_string()))?;

    let content_type = request.content_type();
    let filename = request.filename();
    let data = take_screenshot(&request)
        .await
        .map_err(AppError::screenshot)?;
    let content_length = data.len();

    let mut response = Body::from(data).into_response();
    let headers = response.headers_mut();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
    headers.insert(
        CONTENT_LENGTH,
        HeaderValue::from_str(&content_length.to_string())
            .map_err(|error| AppError::screenshot(anyhow::Error::new(error)))?,
    );
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=86400"),
    );
    headers.insert(
        CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("inline; filename=\"{filename}\""))
            .map_err(|error| AppError::screenshot(anyhow::Error::new(error)))?,
    );

    Ok(response)
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("screenshot_service=info,tower_http=info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
