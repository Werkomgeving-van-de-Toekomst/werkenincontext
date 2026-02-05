//! Search endpoint

use std::sync::Arc;

use axum::{
    extract::{Extension, Query},
    Json,
};
use serde::Deserialize;

use crate::db::{Database, SearchResult};
use crate::error::ApiError;

/// Search query parameters
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i32,
}

fn default_limit() -> i32 {
    50
}

/// Response for search
#[derive(Debug, serde::Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub query: String,
    pub total: usize,
}

/// GET /search - Full-text search across information objects
pub async fn search(
    Query(params): Query<SearchQuery>,
    Extension(db): Extension<Arc<Database>>,
) -> Result<Json<SearchResponse>, ApiError> {
    if params.q.len() < 2 {
        return Err(ApiError::Validation(
            "Query must be at least 2 characters".to_string(),
        ));
    }

    let results = db.search(&params.q, params.limit)?;
    let total = results.len();

    Ok(Json(SearchResponse {
        results,
        query: params.q,
        total,
    }))
}
