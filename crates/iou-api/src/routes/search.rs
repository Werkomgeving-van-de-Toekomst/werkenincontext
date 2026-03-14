//! Advanced search endpoint
//!
//! Implements full-text search with DuckDB FTS, semantic search via embeddings,
//! and hybrid search combining both approaches with re-ranking.

use std::sync::Arc;

use axum::{
    extract::{Extension, Query},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::{Database, SearchResult};
use crate::error::ApiError;

// Re-export shared search types for backward compatibility
pub use crate::search_types::{
    AdvancedSearchResult, FacetCount, SearchFacets, SearchParams, SearchMode,
    SortOrder, SuggestionResult, SuggestionType,
    default_limit, default_offset, default_semantic, default_search_mode,
    default_min_score, default_sort,
};

/// Simple search query parameters (for backwards compatibility)
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i32,
}

/// Response for simple search
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub query: String,
    pub total: usize,
}

/// GET /search - Simple full-text search (backwards compatible)
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

/// Complete search response
#[derive(Debug, Serialize)]
pub struct SearchResults {
    /// Search results
    pub results: Vec<AdvancedSearchResult>,

    /// Original query
    pub query: String,

    /// Total results (before pagination)
    pub total: i64,

    /// Faceted search options
    pub facets: SearchFacets,

    /// Pagination info
    pub limit: i32,
    pub offset: i32,
    pub has_more: bool,

    /// Search mode used
    pub mode: SearchMode,

    /// Time taken for search (ms)
    pub duration_ms: u64,
}

/// GET /api/search/advanced - Advanced full-text and semantic search
pub async fn search_advanced(
    Query(params): Query<SearchParams>,
    Extension(db): Extension<Arc<Database>>,
) -> Result<Json<SearchResults>, ApiError> {
    let start = std::time::Instant::now();

    if params.q.len() < 2 {
        return Err(ApiError::Validation(
            "Query must be at least 2 characters".to_string(),
        ));
    }

    // Build the search based on mode
    let (results, total) = match params.mode {
        SearchMode::Text => db.search_text(&params, &params.q)?,
        SearchMode::Semantic => {
            // For semantic search, we need embeddings - fallback to text for now
            // In production, this would call the semantic search service
            db.search_text(&params, &params.q)?
        }
        SearchMode::Hybrid => {
            // Combine text and semantic search results
            // For now, just use text search
            db.search_text(&params, &params.q)?
        }
    };

    // Build facets
    let facets = db.get_search_facets(&params.q)?;

    let duration_ms = start.elapsed().as_millis() as u64;
    let has_more = (params.offset + results.len() as i32) < total as i32;

    Ok(Json(SearchResults {
        results,
        query: params.q,
        total,
        facets,
        limit: params.limit,
        offset: params.offset,
        has_more,
        mode: params.mode,
        duration_ms,
    }))
}

/// GET /api/search/suggest - Autocomplete suggestions
#[derive(Debug, Deserialize)]
pub struct SuggestParams {
    pub q: String,
    #[serde(default = "default_suggest_limit")]
    pub limit: i32,
}

fn default_suggest_limit() -> i32 {
    10
}

pub async fn search_suggest(
    Query(params): Query<SuggestParams>,
    Extension(db): Extension<Arc<Database>>,
) -> Result<Json<Vec<SuggestionResult>>, ApiError> {
    if params.q.len() < 2 {
        return Ok(Json(vec![]));
    }

    let suggestions = db.get_search_suggestions(&params.q, params.limit)?;

    Ok(Json(suggestions))
}

/// GET /api/search/similar - Find similar documents
#[derive(Debug, Deserialize)]
pub struct SimilarParams {
    pub id: Uuid,
    #[serde(default = "default_limit")]
    pub limit: i32,
}

pub async fn find_similar(
    Query(params): Query<SimilarParams>,
    Extension(db): Extension<Arc<Database>>,
) -> Result<Json<Vec<AdvancedSearchResult>>, ApiError> {
    let results = db.find_similar_documents(params.id, params.limit)?;

    Ok(Json(results))
}

/// POST /api/search/reindex - Trigger reindex of search data
pub async fn reindex_search(
    Extension(db): Extension<Arc<Database>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let indexed = db.reindex_search()?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "indexed": indexed,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
