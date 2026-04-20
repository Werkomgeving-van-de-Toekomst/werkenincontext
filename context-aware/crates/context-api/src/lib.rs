// =============================================================================
// Context-API: Axum REST API for Context-Aware Data
// =============================================================================

pub mod handlers;
pub mod routes;
pub mod middleware;
pub mod openapi;

use axum::{Router, http::StatusCode};
use context_store::{ConnectionPool, StoreConfig};
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::Level;

use context_core::Context;

pub use handlers::*;
pub use routes::*;

/// Context API state
#[derive(Clone)]
pub struct AppState {
    pub pool: ConnectionPool,
}

/// Create the Axum router for the Context API
pub fn router() -> Router {
    let config = StoreConfig::default();
    let pool = ConnectionPool::new(config)
        .expect("Failed to create connection pool");

    let state = AppState { pool };

    Router::new()
        .nest("/api/v1", routes::context_routes())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &http::Request<_>| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        status = http::StatusCode::OK.as_u16(),
                    )
                }),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Start the API server
pub async fn run(addr: &str) -> anyhow::Result<()> {
    let app = router();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Context API listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// API Error type
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Context not found: {0}")]
    ContextNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Database error: {0}")]
    Database(#[from] context_store::StoreError),

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl ApiError {
    pub fn status(&self) -> StatusCode {
        match self {
            ApiError::ContextNotFound(_) => StatusCode::NOT_FOUND,
            ApiError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status();
        let body = serde_json::json!({
            "error": self.to_string(),
            "status": status.as_u16(),
        });

        (status, axum::Json(body)).into_response()
    }
}

/// Type alias for API results
pub type ApiResult<T> = Result<T, ApiError>;
