//! Error handling for the API

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use iou_core::storage::S3Error;

/// API error type
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Too many requests: {0}")]
    TooManyRequests(String),

    #[error("Payload too large: {0}")]
    PayloadTooLarge(String),

    #[error("Database error: {0}")]
    Database(#[from] duckdb::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Convert S3Error to ApiError
impl From<S3Error> for ApiError {
    fn from(err: S3Error) -> Self {
        match err {
            S3Error::NotFound(msg) => ApiError::NotFound(msg),
            S3Error::PayloadTooLarge { .. } => ApiError::PayloadTooLarge(err.to_string()),
            S3Error::ConnectionFailed(msg) => ApiError::Internal(anyhow::anyhow!("S3 connection failed: {}", msg)),
            S3Error::UploadFailed(msg) => ApiError::Internal(anyhow::anyhow!("Upload failed: {}", msg)),
            S3Error::DownloadFailed(msg) => ApiError::Internal(anyhow::anyhow!("Download failed: {}", msg)),
            S3Error::InvalidConfig(msg) => ApiError::Internal(anyhow::anyhow!("Invalid S3 config: {}", msg)),
            S3Error::S3Error(msg) => ApiError::Internal(anyhow::anyhow!("S3 error: {}", msg)),
            S3Error::HttpError { code, message } => ApiError::Internal(anyhow::anyhow!("S3 HTTP {}: {}", code, message)),
            S3Error::MissingEnvVar(var) => ApiError::Internal(anyhow::anyhow!("Missing env var: {}", var)),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg.clone()),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg.clone()),
            ApiError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, "TOO_MANY_REQUESTS", msg.clone()),
            ApiError::PayloadTooLarge(msg) => (StatusCode::PAYLOAD_TOO_LARGE, "PAYLOAD_TOO_LARGE", msg.clone()),
            ApiError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                format!("Database error: {}", e),
            ),
            ApiError::Internal(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                format!("Internal error: {}", e),
            ),
        };

        let body = Json(json!({
            "error": {
                "code": code,
                "message": message
            }
        }));

        (status, body).into_response()
    }
}
