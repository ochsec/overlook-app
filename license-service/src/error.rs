use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("License not found")]
    LicenseNotFound,

    #[error("License has been revoked")]
    LicenseRevoked,

    #[error("Maximum number of devices reached")]
    MaxDevicesReached,

    #[error("License has expired")]
    LicenseExpired,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::LicenseNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::LicenseRevoked => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::MaxDevicesReached => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::LicenseExpired => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
