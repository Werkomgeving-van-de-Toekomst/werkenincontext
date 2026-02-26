//! IOU-Modern API Server
//!
//! REST API voor de Informatie Ondersteunde Werkomgeving,
//! gebouwd met Axum en DuckDB.

use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    routing::{get, post, put},
    Router,
    middleware as axum_middleware,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod error;
mod middleware;
mod routes;
mod workflows;

use db::Database;
use workflows::WorkflowEngine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting IOU-Modern API server...");

    // Load configuration
    dotenvy::dotenv().ok();
    let config = config::Config::from_env()?;

    // Initialize DuckDB database
    let db = Database::new(&config.database_path)?;
    db.initialize_schema()?;

    tracing::info!("DuckDB initialized at: {}", config.database_path);

    // Initialize workflow engine
    let db_arc = Arc::new(db);
    let workflow_engine = Arc::new(WorkflowEngine::new(db_arc.clone()));

    // Build API router
    let api = Router::new()
        // Health check (no auth required)
        .route("/health", get(routes::health::health_check))
        // Authentication endpoints
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/refresh", post(routes::auth::refresh_token))
        .route("/auth/logout", post(routes::auth::logout))
        // Context endpoints
        .route("/context/{id}", get(routes::context::get_context))
        .route("/domains", get(routes::context::list_domains))
        .route("/domains", post(routes::context::create_domain))
        // Object endpoints
        .route("/objects", post(routes::objects::create_object))
        .route("/objects/{id}", get(routes::objects::get_object))
        // Search (advanced search)
        .route("/search", get(routes::search::search))
        .route("/search/advanced", get(routes::search::search_advanced))
        .route("/search/suggest", get(routes::search::search_suggest))
        .route("/search/similar", get(routes::search::find_similar))
        .route("/search/reindex", post(routes::search::reindex_search))
        // Compliance endpoints
        .route("/compliance/overview", get(routes::compliance::get_compliance_overview))
        .route("/compliance/alerts", get(routes::compliance::get_compliance_alerts))
        .route("/compliance/trends", get(routes::compliance::get_compliance_trends))
        .route("/compliance/assessment/{id}", get(routes::compliance::get_object_assessment))
        .route("/compliance/assess", post(routes::compliance::trigger_assessment))
        .route("/compliance/resolve/{alert_id}", put(routes::compliance::resolve_alert))
        // Apps
        .route("/apps/recommended", get(routes::apps::get_recommended_apps))
        // GraphRAG
        .route(
            "/graphrag/relations/{domain_id}",
            get(routes::graphrag::get_relations),
        )
        .route("/graphrag/entities", get(routes::graphrag::list_entities))
        .route(
            "/graphrag/communities",
            get(routes::graphrag::list_communities),
        )
        // Workflow endpoints
        .route("/workflows", get(workflows::list_workflows).post(workflows::create_workflow))
        .route("/workflows/stats", get(workflows::get_workflow_stats))
        .route("/workflows/{id}", get(workflows::get_workflow))
        .route("/workflows/start", post(workflows::start_workflow))
        .route("/workflows/{id}/transition", post(workflows::transition_workflow))
        .route("/workflows/history/{document_id}", get(workflows::get_workflow_history))
        .route("/workflows/actions/pending", get(workflows::get_pending_actions))
        .without_v07_checks();

    // Combine API with static file serving
    let app = Router::new()
        .nest("/api", api)
        // Serve static frontend files
        .fallback_service(ServeDir::new("static").append_index_html_on_directories(true))
        // CORS layer
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        // Optional auth middleware (adds auth context if token present)
        .layer(axum_middleware::from_fn(middleware::optional_auth_middleware))
        // Extensions
        .layer(Extension(db_arc))
        .layer(Extension(workflow_engine));

    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
