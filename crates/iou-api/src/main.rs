//! IOU-Modern API Server
//!
//! REST API voor de Informatie Ondersteunde Werkomgeving,
//! gebouwd met Axum en DuckDB.

use std::sync::Arc;

use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod error;
mod routes;

use db::Database;

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

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(routes::health::health_check))
        // Context endpoints
        .route("/context/:id", get(routes::context::get_context))
        .route("/domains", get(routes::context::list_domains))
        .route("/domains", post(routes::context::create_domain))
        // Object endpoints
        .route("/objects", post(routes::objects::create_object))
        .route("/objects/:id", get(routes::objects::get_object))
        // Search
        .route("/search", get(routes::search::search))
        // Apps
        .route("/apps/recommended", get(routes::apps::get_recommended_apps))
        // GraphRAG
        .route(
            "/graphrag/relations/:domain_id",
            get(routes::graphrag::get_relations),
        )
        .route("/graphrag/entities", get(routes::graphrag::list_entities))
        .route(
            "/graphrag/communities",
            get(routes::graphrag::list_communities),
        )
        // CORS layer
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        // Database extension
        .layer(Extension(Arc::new(db)));

    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
