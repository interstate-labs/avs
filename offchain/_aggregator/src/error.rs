use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub struct AppError(eyre::Report);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self.0);
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        (status, Json(json!({ "error": self.0.to_string() }))).into_response()
    }
}

impl From<eyre::Report> for AppError {
    fn from(error: eyre::Report) -> Self {
        Self(error)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
