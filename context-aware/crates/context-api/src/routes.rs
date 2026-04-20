// =============================================================================
// Route Definitions
// =============================================================================

use axum::{
    Router,
    routing::{get, post, put, delete},
};

use super::{handlers, AppState};

/// Context API routes
pub fn context_routes() -> Router<AppState> {
    Router::new()
        // Context CRUD
        .route("/context", get(handlers::list_contexts).post(handlers::create_context))
        .route("/context/:id", get(handlers::get_context).put(handlers::update_context).delete(handlers::delete_context))
        .route("/context/object/:object_id", get(handlers::get_contexts_by_object))
        .route("/context/org/:org_id", get(handlers::get_contexts_by_org))

        // Context Records (history)
        .route("/context/:id/history", get(handlers::get_context_history))
        .route("/context/:id/version/:version", get(handlers::get_context_version))

        // Context Types
        .route("/context-types", get(handlers::list_context_types).post(handlers::create_context_type))
        .route("/context-types/:id", get(handlers::get_context_type).put(handlers::update_context_type))

        // Inference
        .route("/inference/analyze", post(handlers::analyze_context))

        // Quality
        .route("/quality/report", get(handlers::get_quality_report))
        .route("/quality/score/:context_id", get(handlers::get_quality_score))

        // Health
        .route("/health", get(handlers::health_check))
}
