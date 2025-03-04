use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheck {
    status: Status,
}

impl IntoResponse for HealthCheck {
    fn into_response(self) -> Response {
        let code = match self.status {
            Status::Ok | Status::Warn => StatusCode::OK,
            Status::Fail => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (code, Json(self)).into_response()
    }
}

pub async fn health_check() -> HealthCheck {
    HealthCheck { status: Status::Ok }
}
