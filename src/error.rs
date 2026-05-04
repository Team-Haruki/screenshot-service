use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Screenshot(anyhow::Error),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn screenshot(error: impl Into<anyhow::Error>) -> Self {
        Self::Screenshot(error.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::Screenshot(error) => {
                tracing::error!(?error, "screenshot request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("screenshot failed: {error}"),
                )
            }
        };

        (status, Json(ErrorResponse { error })).into_response()
    }
}
