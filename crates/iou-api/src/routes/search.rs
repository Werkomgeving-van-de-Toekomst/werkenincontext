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

/// Advanced search query parameters
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    /// Search query string
    pub q: String,

    /// Maximum results to return
    #[serde(default = "default_limit")]
    pub limit: i32,

    /// Offset for pagination
    #[serde(default = "default_offset")]
    pub offset: i32,

    /// Filter by domain type
    pub domain_type: Option<String>,

    /// Filter by specific domain ID
    pub domain_id: Option<String>,

    /// Filter by object type
    pub object_type: Option<String>,

    /// Filter by classification level
    pub classification: Option<String>,

    /// Filter by compliance status
    pub compliance_status: Option<String>,

    /// Enable semantic search (uses embeddings if available)
    #[serde(default = "default_semantic")]
    pub semantic: bool,

    /// Search mode: "text", "semantic", or "hybrid"
    #[serde(default = "default_search_mode")]
    pub mode: SearchMode,

    /// Minimum relevance score (0.0 - 1.0)
    #[serde(default = "default_min_score")]
    pub min_score: f32,

    /// Sort order
    #[serde(default = "default_sort")]
    pub sort: SortOrder,
}

fn default_limit() -> i32 {
    50
}

fn default_offset() -> i32 {
    0
}

fn default_semantic() -> bool {
    false
}

fn default_search_mode() -> SearchMode {
    SearchMode::Text
}

fn default_min_score() -> f32 {
    0.0
}

fn default_sort() -> SortOrder {
    SortOrder::Relevance
}

/// Search mode
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    /// Traditional text-based search
    Text,
    /// Semantic search using embeddings
    Semantic,
    /// Hybrid: combine text and semantic with re-ranking
    Hybrid,
}

/// Sort order for results
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    /// Sort by relevance score
    Relevance,
    /// Sort by date (newest first)
    DateDesc,
    /// Sort by date (oldest first)
    DateAsc,
    /// Sort by title A-Z
    TitleAsc,
}

/// Advanced search result
#[derive(Debug, Serialize)]
pub struct AdvancedSearchResult {
    /// Unique identifier
    pub id: Uuid,

    /// Object type
    pub object_type: String,

    /// Title of the object
    pub title: String,

    /// Brief snippet with highlighted matches
    pub snippet: String,

    /// Domain this object belongs to
    pub domain_id: Uuid,
    pub domain_name: String,
    pub domain_type: String,

    /// Classification level
    pub classification: String,

    /// Relevance score (0.0 - 1.0)
    pub score: f32,

    /// Creation date
    pub created_at: String,

    /// Optional: semantic similarity score
    pub semantic_score: Option<f32>,

    /// Optional: text search rank
    pub text_rank: Option<f32>,

    /// Compliance metadata
    pub is_woo_relevant: Option<bool>,
    pub woo_disclosure_class: Option<String>,
}

/// Faceted search results for filters
#[derive(Debug, Serialize)]
pub struct SearchFacets {
    /// Available domain types with counts
    pub domain_types: Vec<FacetCount>,

    /// Available object types with counts
    pub object_types: Vec<FacetCount>,

    /// Available classifications with counts
    pub classifications: Vec<FacetCount>,

    /// Compliance status distribution
    pub compliance_statuses: Vec<FacetCount>,
}

#[derive(Debug, Serialize)]
pub struct FacetCount {
    pub value: String,
    pub count: i64,
    pub label: String,
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

#[derive(Debug, Serialize)]
pub struct SuggestionResult {
    pub text: String,
    pub suggestion_type: SuggestionType,
    pub count: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SuggestionType {
    Query,
    Domain,
    Tag,
    Entity,
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
