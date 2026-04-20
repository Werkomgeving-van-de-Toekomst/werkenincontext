// =============================================================================
// Middleware for Context API
// =============================================================================

use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use tracing::{info_span, Instrument};

/// Request ID header
const X_REQUEST_ID: &str = "x-request-id";

/// Logging middleware
pub async fn logging_middleware(
    req: Request,
    next: Next,
) -> Response {
    let request_id = req
        .headers()
        .get(X_REQUEST_ID)
        .and_then(|h| h.to_str().ok())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
        .to_string();

    let uri = req.uri().clone();
    let method = req.method().clone();

    let span = info_span!(
        "request",
        request_id = %request_id,
        method = %method,
        uri = %uri,
    );

    let response = next.run(req).instrument(span).await;

    response
}

/// Authentication middleware (stub - integrate with your auth system)
pub async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // TODO: Implement JWT validation
    // For now, just pass through
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, Router};

    #[tokio::test]
    async fn test_logging_middleware() {
        let app = Router::new()
            .route("/", axum::routing::get(|| async { "Hello" }))
            .layer(axum::middleware::from_fn(logging_middleware));

        // Test request
    }
}
