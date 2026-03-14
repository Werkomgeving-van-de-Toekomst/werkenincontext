//! Shared types for search functionality
//!
//! These types are used by both db.rs and routes/search.rs
//! to avoid circular dependencies.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

pub fn default_limit() -> i32 {
    50
}

pub fn default_offset() -> i32 {
    0
}

pub fn default_semantic() -> bool {
    false
}

pub fn default_search_mode() -> SearchMode {
    SearchMode::Text
}

pub fn default_min_score() -> f32 {
    0.0
}

pub fn default_sort() -> SortOrder {
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
