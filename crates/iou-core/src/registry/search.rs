//! Registry Search Functionality
//!
//! Comprehensive search capabilities for the registry including
//! full-text search, filtering, sorting, and advanced query operators.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{DataEntity, EntityType, RegistrySearch};

/// Search query for the registry
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchQuery {
    /// Text search query (supports operators)
    pub query: Option<String>,

    /// Entity type filter
    pub entity_type: Option<EntityType>,

    /// Source ID filter
    pub source_id: Option<Uuid>,

    /// Organization filter
    pub organization_id: Option<Uuid>,

    /// Quality score filter
    pub quality_filter: Option<QualityFilter>,

    /// Date range filter
    pub date_filter: Option<DateFilter>,

    /// Metadata filters
    pub metadata_filters: Vec<MetadataFilter>,

    /// Sort specification
    pub sort: Option<SortSpec>,

    /// Pagination
    pub pagination: Option<Pagination>,

    /// Search options
    pub options: SearchOptions,
}

/// Quality score filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityFilter {
    /// Minimum quality score (0-1)
    pub min_score: Option<f32>,

    /// Maximum quality score (0-1)
    pub max_score: Option<f32>,

    /// Quality rating filter
    pub rating: Option<QualityRating>,
}

/// Quality rating classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityRating {
    Excellent,
    Good,
    Acceptable,
    Poor,
}

/// Date range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateFilter {
    /// Field to filter on
    pub field: DateField,

    /// Start date (inclusive)
    pub from: Option<DateTime<Utc>>,

    /// End date (inclusive)
    pub to: Option<DateTime<Utc>>,

    /// Relative date offset
    pub relative: Option<RelativeDate>,
}

/// Date fields available for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DateField {
    CreatedAt,
    UpdatedAt,
    LastSyncedAt,
}

/// Relative date offset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelativeDate {
    LastHour,
    Last24Hours,
    Last7Days,
    Last30Days,
    Last90Days,
    LastYear,
    Today,
    Yesterday,
    ThisWeek,
    ThisMonth,
    ThisYear,
}

/// Metadata filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataFilter {
    /// Field path (supports dot notation for nested fields)
    pub field: String,

    /// Comparison operator
    pub operator: FilterOperator,

    /// Value to compare against
    pub value: serde_json::Value,
}

/// Filter operator for metadata queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    In,
    NotIn,
    Exists,
    NotExists,
    Regex,
}

/// Sort specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortSpec {
    /// Field to sort by
    pub field: SortField,

    /// Sort direction
    pub direction: SortDirection,
}

/// Fields available for sorting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    Name,
    CreatedAt,
    UpdatedAt,
    LastSyncedAt,
    QualityScore,
    EntityType,
    SourceId,
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Pagination specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Page number (0-indexed)
    pub page: usize,

    /// Items per page
    pub per_page: usize,
}

/// Search options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchOptions {
    /// Whether to include inactive entities
    pub include_inactive: bool,

    /// Whether to include metadata in results
    pub include_metadata: bool,

    /// Whether to return total count
    pub include_total: bool,

    /// Search timeout in milliseconds
    pub timeout_ms: Option<u64>,

    /// Whether to highlight matching text
    pub highlight_matches: bool,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Matching entities
    pub entities: Vec<SearchResultEntity>,

    /// Total matching entities (if requested)
    pub total_count: Option<usize>,

    /// Page information (if paginated)
    pub page_info: Option<PageInfo>,

    /// Search execution time
    pub execution_time_ms: u64,

    /// Search timestamp
    pub searched_at: DateTime<Utc>,

    /// Faceted results (if requested)
    pub facets: Option<SearchFacets>,
}

/// Single entity in search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultEntity {
    /// Entity ID
    pub id: Uuid,

    /// Canonical ID
    pub canonical_id: Uuid,

    /// Entity type
    pub entity_type: EntityType,

    /// Entity name
    pub name: String,

    /// External ID
    pub external_id: String,

    /// Source ID
    pub source_id: Uuid,

    /// Quality score
    pub quality_score: f32,

    /// Quality rating
    pub quality_rating: QualityRating,

    /// Match highlights (if enabled)
    pub highlights: Option<Vec<MatchHighlight>>,

    /// Metadata (if requested)
    pub metadata: Option<serde_json::Value>,

    /// When entity was last synced
    pub last_synced_at: Option<DateTime<Utc>>,

    /// When entity was created
    pub created_at: DateTime<Utc>,
}

/// Highlight for matching text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchHighlight {
    /// Field that contains the match
    pub field: String,

    /// Matching text snippet
    pub snippet: String,

    /// Highlight markers
    pub highlights: Vec<(usize, usize)>,
}

/// Page information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    /// Current page (0-indexed)
    pub page: usize,

    /// Items per page
    pub per_page: usize,

    /// Total pages
    pub total_pages: usize,

    /// Total items
    pub total_items: usize,

    /// Whether there's a next page
    pub has_next: bool,

    /// Whether there's a previous page
    pub has_previous: bool,
}

/// Faceted search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacets {
    /// Entity type distribution
    pub entity_types: HashMap<EntityType, usize>,

    /// Source distribution
    pub sources: HashMap<Uuid, String>,

    /// Quality rating distribution
    pub quality_ratings: HashMap<QualityRating, usize>,

    /// Date ranges (e.g., last synced buckets)
    pub date_ranges: HashMap<String, usize>,
}

/// Advanced search query with operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSearchQuery {
    /// Query clauses combined with AND/OR
    pub clauses: Vec<SearchClause>,

    /// Logical operator between clauses
    pub logic: LogicOperator,

    /// Filters (same as basic search)
    pub filters: SearchFilters,
}

/// Search clause with operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchClause {
    /// Field to search (or "*" for all fields)
    pub field: String,

    /// Search operator
    pub operator: SearchOperator,

    /// Value(s) to search for
    pub value: SearchValue,
}

/// Search operator for clauses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    LessThan,
    GreaterThan,
    Between,
    In,
    NotIn,
    Exists,
    Fuzzy,
}

/// Search value (single or multiple)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SearchValue {
    Single(String),
    Multiple(Vec<String>),
    Range((String, String)),
}

/// Logic operator for combining clauses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "uppercase")]
pub enum LogicOperator {
    And,
    Or,
}

/// Combined filters for advanced search
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchFilters {
    pub entity_types: Vec<EntityType>,
    pub source_ids: Vec<Uuid>,
    pub organization_ids: Vec<Uuid>,
    pub quality_filter: Option<QualityFilter>,
    pub date_filter: Option<DateFilter>,
}

/// Suggestion for autocomplete/typeahead
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSuggestion {
    /// Suggested text
    pub text: String,

    /// Suggestion type
    pub suggestion_type: SuggestionType,

    /// Additional context (e.g., entity type)
    pub context: Option<String>,

    /// Relevance score
    pub score: f32,
}

/// Type of search suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionType {
    EntityName,
    EntityType,
    FieldName,
    Operator,
    Value,
}

/// Saved search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSearch {
    /// Saved search ID
    pub id: Uuid,

    /// Search name
    pub name: String,

    /// Search description
    pub description: Option<String>,

    /// Query (serialized)
    pub query: SearchQuery,

    /// Owner user ID
    pub owner_id: Uuid,

    /// Whether search is public
    pub is_public: bool,

    /// Usage count
    pub usage_count: usize,

    /// Created at
    pub created_at: DateTime<Utc>,

    /// Last used at
    pub last_used_at: Option<DateTime<Utc>>,
}

// ============================================
// SEARCH SERVICE
// ============================================

/// Registry search service
pub struct SearchService {
    // In a real implementation, this would have a reference to
    // the registry storage and potentially a search index like
    // Tantivy, Meilisearch, or Elasticsearch
}

impl SearchService {
    /// Create a new search service
    pub fn new() -> Self {
        Self
    }

    /// Execute a search query
    pub fn search(&self, query: SearchQuery, entities: &[DataEntity]) -> SearchResult {
        let start = std::time::Instant::now();

        // Filter entities
        let mut filtered: Vec<_> = entities
            .iter()
            .filter(|e| query.options.include_inactive || e.is_active)
            .filter(|e| {
                if let Some(entity_type) = query.entity_type {
                    if e.entity_type != entity_type {
                        return false;
                    }
                }
                if let Some(source_id) = query.source_id {
                    if e.source_id != source_id {
                        return false;
                    }
                }
                if let Some(ref quality_filter) = query.quality_filter {
                    let score = e.quality_score.overall();
                    if let Some(min) = quality_filter.min_score {
                        if score < min {
                            return false;
                        }
                    }
                    if let Some(max) = quality_filter.max_score {
                        if score > max {
                            return false;
                        }
                    }
                    if let Some(rating) = quality_filter.rating {
                        let entity_rating = QualityRating::from_score(score);
                        if entity_rating != rating {
                            return false;
                        }
                    }
                }
                // Date filter
                if let Some(ref date_filter) = query.date_filter {
                    let date_value = match date_filter.field {
                        DateField::CreatedAt => e.created_at,
                        DateField::UpdatedAt => e.updated_at,
                        DateField::LastSyncedAt => e.last_synced_at.unwrap_or(e.updated_at),
                    };

                    if let Some(from) = date_filter.from {
                        if date_value < from {
                            return false;
                        }
                    }
                    if let Some(to) = date_filter.to {
                        if date_value > to {
                            return false;
                        }
                    }
                }
                true
            })
            .collect();

        // Text search
        if let Some(ref text_query) = query.query {
            let query_lower = text_query.to_lowercase();
            filtered.retain(|e| {
                e.name.to_lowercase().contains(&query_lower)
                    || e.external_id.to_lowercase().contains(&query_lower)
            });
        }

        // Metadata filters
        for filter in &query.metadata_filters {
            filtered.retain(|e| self.apply_metadata_filter(e, filter));
        }

        // Sort
        if let Some(ref sort) = query.sort {
            self.sort_results(&mut filtered, sort);
        }

        let total_count = filtered.len();

        // Paginate
        let (page_entities, page_info) = if let Some(ref pagination) = query.pagination {
            let start = pagination.page * pagination.per_page;
            let end = start + pagination.per_page;
            let page_entities: Vec<_> = filtered
                .into_iter()
                .skip(start)
                .take(pagination.per_page)
                .collect();

            let total_pages = (total_count + pagination.per_page - 1) / pagination.per_page;

            let page_info = PageInfo {
                page: pagination.page,
                per_page: pagination.per_page,
                total_pages,
                total_items: total_count,
                has_next: pagination.page + 1 < total_pages,
                has_previous: pagination.page > 0,
            };

            (page_entities, Some(page_info))
        } else {
            (filtered.into_iter().collect(), None)
        };

        // Build facets (if requested)
        let facets = if query.options.include_total {
            Some(self.build_facets(entities))
        } else {
            None
        };

        // Convert to search result entities
        let result_entities: Vec<SearchResultEntity> = page_entities
            .into_iter()
            .map(|e| SearchResultEntity {
                id: e.id,
                canonical_id: e.canonical_id,
                entity_type: e.entity_type,
                name: e.name.clone(),
                external_id: e.external_id.clone(),
                source_id: e.source_id,
                quality_score: e.quality_score.overall(),
                quality_rating: QualityRating::from_score(e.quality_score.overall()),
                highlights: None, // TODO: implement highlighting
                metadata: if query.options.include_metadata {
                    Some(e.metadata.clone())
                } else {
                    None
                },
                last_synced_at: e.last_synced_at,
                created_at: e.created_at,
            })
            .collect();

        SearchResult {
            entities: result_entities,
            total_count: if query.options.include_total {
                Some(total_count)
            } else {
                None
            },
            page_info,
            execution_time_ms: start.elapsed().as_millis() as u64,
            searched_at: Utc::now(),
            facets,
        }
    }

    /// Get search suggestions for autocomplete
    pub fn suggestions(&self, prefix: &str, entities: &[DataEntity]) -> Vec<SearchSuggestion> {
        let prefix_lower = prefix.to_lowercase();
        let mut suggestions = Vec::new();

        // Entity name suggestions
        for entity in entities.iter().take(100) {
            if entity.name.to_lowercase().starts_with(&prefix_lower) {
                suggestions.push(SearchSuggestion {
                    text: entity.name.clone(),
                    suggestion_type: SuggestionType::EntityName,
                    context: Some(format!("{:?}", entity.entity_type)),
                    score: 1.0,
                });
            }
        }

        // Limit and sort by relevance
        suggestions.truncate(10);
        suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        suggestions
    }

    /// Build facet information
    fn build_facets(&self, entities: &[DataEntity]) -> SearchFacets {
        let mut entity_types: HashMap<EntityType, usize> = HashMap::new();
        let mut sources: HashMap<Uuid, String> = HashMap::new();
        let mut quality_ratings: HashMap<QualityRating, usize> = HashMap::new();
        let mut date_ranges: HashMap<String, usize> = HashMap::new();

        for entity in entities {
            // Entity type facet
            *entity_types.entry(entity.entity_type).or_insert(0) += 1;

            // Source facet (just store ID for now)
            sources.entry(entity.source_id).or_insert_with(|| {
                format!("Source {}", entity.source_id)
            });

            // Quality rating facet
            let rating = QualityRating::from_score(entity.quality_score.overall());
            *quality_ratings.entry(rating).or_insert(0) += 1;

            // Date range facet (based on last synced)
            if let Some(last_synced) = entity.last_synced_at {
                let days_old = (Utc::now() - last_synced).num_days();
                let range = if days_old <= 1 {
                    "Last 24 hours".to_string()
                } else if days_old <= 7 {
                    "Last 7 days".to_string()
                } else if days_old <= 30 {
                    "Last 30 days".to_string()
                } else if days_old <= 90 {
                    "Last 90 days".to_string()
                } else {
                    "Older than 90 days".to_string()
                };
                *date_ranges.entry(range).or_insert(0) += 1;
            }
        }

        SearchFacets {
            entity_types,
            sources,
            quality_ratings,
            date_ranges,
        }
    }

    /// Apply a metadata filter to an entity
    fn apply_metadata_filter(&self, entity: &DataEntity, filter: &MetadataFilter) -> bool {
        let value = self.get_metadata_value(entity, &filter.field);

        match filter.operator {
            FilterOperator::Exists => value.is_some(),
            FilterOperator::NotExists => value.is_none(),
            FilterOperator::Equals => {
                value.map(|v| v == filter.value).unwrap_or(false)
            }
            FilterOperator::NotEquals => {
                value.map(|v| v != filter.value).unwrap_or(true)
            }
            FilterOperator::Contains => {
                value.and_then(|v| v.as_str())
                    .map(|s| filter.value.as_str().map_or(false, |fv| s.contains(fv)))
                    .unwrap_or(false)
            }
            FilterOperator::In => {
                if let Some(arr) = filter.value.as_array() {
                    value.map(|v| arr.contains(&v)).unwrap_or(false)
                } else {
                    false
                }
            }
            _ => true, // Other operators not implemented for demo
        }
    }

    /// Get metadata value by field path (supports dot notation)
    fn get_metadata_value(&self, entity: &DataEntity, path: &str) -> Option<serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = Some(&entity.metadata);

        for part in &parts {
            match current {
                Some(serde_json::Value::Object(map)) => {
                    current = map.get(*part);
                }
                Some(serde_json::Value::Array(arr)) => {
                    if let Ok(index) = part.parse::<usize>() {
                        current = arr.get(index);
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        current.cloned()
    }

    /// Sort results by sort specification
    fn sort_results(&self, results: &mut Vec<&DataEntity>, sort: &SortSpec) {
        results.sort_by(|a, b| {
            let comparison = match sort.field {
                SortField::Name => a.name.cmp(&b.name),
                SortField::CreatedAt => a.created_at.cmp(&b.created_at),
                SortField::UpdatedAt => a.updated_at.cmp(&b.updated_at),
                SortField::LastSyncedAt => {
                    a.last_synced_at
                        .cmp(&b.last_synced_at)
                        .then_with(|| a.created_at.cmp(&b.created_at))
                }
                SortField::QualityScore => {
                    a.quality_score
                        .overall()
                        .partial_cmp(&b.quality_score.overall())
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
                SortField::EntityType => a.entity_type.cmp(&b.entity_type),
                SortField::SourceId => a.source_id.cmp(&b.source_id),
            };

            match sort.direction {
                SortDirection::Asc => comparison,
                SortDirection::Desc => comparison.reverse(),
            }
        });
    }
}

impl Default for SearchService {
    fn default() -> Self {
        Self::new()
    }
}

impl QualityRating {
    pub fn from_score(score: f32) -> Self {
        if score >= 0.9 {
            Self::Excellent
        } else if score >= 0.75 {
            Self::Good
        } else if score >= 0.5 {
            Self::Acceptable
        } else {
            Self::Poor
        }
    }
}

// ============================================
// QUERY BUILDERS
// ============================================

/// Builder for constructing search queries
pub struct SearchQueryBuilder {
    query: SearchQuery,
}

impl SearchQueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            query: SearchQuery::default(),
        }
    }

    /// Set text query
    pub fn text(mut self, query: impl Into<String>) -> Self {
        self.query.query = Some(query.into());
        self
    }

    /// Filter by entity type
    pub fn entity_type(mut self, entity_type: EntityType) -> Self {
        self.query.entity_type = Some(entity_type);
        self
    }

    /// Filter by source ID
    pub fn source_id(mut self, source_id: Uuid) -> Self {
        self.query.source_id = Some(source_id);
        self
    }

    /// Filter by quality
    pub fn quality_min(mut self, min_score: f32) -> Self {
        self.query.quality_filter = self.query.quality_filter.take().or_else(|| Some(QualityFilter {
            min_score: None,
            max_score: None,
            rating: None,
        }));
        if let Some(ref mut qf) = self.query.quality_filter {
            qf.min_score = Some(min_score);
        }
        self
    }

    /// Filter by quality rating
    pub fn quality_rating(mut self, rating: QualityRating) -> Self {
        self.query.quality_filter = self.query.quality_filter.take().or_else(|| Some(QualityFilter {
            min_score: None,
            max_score: None,
            rating: None,
        }));
        if let Some(ref mut qf) = self.query.quality_filter {
            qf.rating = Some(rating);
        }
        self
    }

    /// Add metadata filter
    pub fn metadata_filter(mut self, field: impl Into<String>, operator: FilterOperator, value: serde_json::Value) -> Self {
        self.query.metadata_filters.push(MetadataFilter {
            field: field.into(),
            operator,
            value,
        });
        self
    }

    /// Sort results
    pub fn sort(mut self, field: SortField, direction: SortDirection) -> Self {
        self.query.sort = Some(SortSpec { field, direction });
        self
    }

    /// Paginate results
    pub fn paginate(mut self, page: usize, per_page: usize) -> Self {
        self.query.pagination = Some(Pagination { page, per_page });
        self
    }

    /// Include inactive entities
    pub fn include_inactive(mut self, include: bool) -> Self {
        self.query.options.include_inactive = include;
        self
    }

    /// Include metadata in results
    pub fn include_metadata(mut self, include: bool) -> Self {
        self.query.options.include_metadata = include;
        self
    }

    /// Include total count
    pub fn include_total(mut self, include: bool) -> Self {
        self.query.options.include_total = include;
        self
    }

    /// Build the query
    pub fn build(self) -> SearchQuery {
        self.query
    }
}

impl Default for SearchQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::quality::QualityScore;

    #[test]
    fn test_search_query_builder() {
        let query = SearchQueryBuilder::new()
            .text("test")
            .entity_type(EntityType::Citizen)
            .quality_min(0.8)
            .sort(SortField::Name, SortDirection::Asc)
            .paginate(0, 20)
            .build();

        assert!(query.query.is_some());
        assert_eq!(query.query.unwrap(), "test");
        assert_eq!(query.entity_type, Some(EntityType::Citizen));
        assert!(query.quality_filter.is_some());
        assert!(query.sort.is_some());
        assert!(query.pagination.is_some());
    }

    #[test]
    fn test_quality_rating_from_score() {
        assert_eq!(QualityRating::from_score(0.95), QualityRating::Excellent);
        assert_eq!(QualityRating::from_score(0.8), QualityRating::Good);
        assert_eq!(QualityRating::from_score(0.6), QualityRating::Acceptable);
        assert_eq!(QualityRating::from_score(0.3), QualityRating::Poor);
    }

    #[test]
    fn test_search_service_empty() {
        let service = SearchService::new();
        let query = SearchQuery::default();
        let result = service.search(query, &[]);

        assert_eq!(result.entities.len(), 0);
        assert_eq!(result.total_count, Some(0));
    }

    #[test]
    fn test_search_service_with_entities() {
        let service = SearchService::new();
        let entities = vec![
            DataEntity::new(
                Uuid::new_v4(),
                "EXT-1".to_string(),
                EntityType::Citizen,
                "John Doe".to_string(),
            ),
            DataEntity::new(
                Uuid::new_v4(),
                "EXT-2".to_string(),
                EntityType::Organization,
                "Acme Corp".to_string(),
            ),
        ];

        let query = SearchQuery::default();
        let result = service.search(query, &entities);

        assert_eq!(result.entities.len(), 2);
    }

    #[test]
    fn test_search_with_entity_type_filter() {
        let service = SearchService::new();
        let source_id = Uuid::new_v4();
        let entities = vec![
            DataEntity::new(source_id, "EXT-1".to_string(), EntityType::Citizen, "John".to_string()),
            DataEntity::new(source_id, "EXT-2".to_string(), EntityType::Organization, "Acme".to_string()),
        ];

        let query = SearchQueryBuilder::new()
            .entity_type(EntityType::Citizen)
            .build();

        let result = service.search(query, &entities);

        assert_eq!(result.entities.len(), 1);
        assert_eq!(result.entities[0].entity_type, EntityType::Citizen);
    }

    #[test]
    fn test_search_sorting() {
        let service = SearchService::new();
        let source_id = Uuid::new_v4();
        let entities = vec![
            DataEntity::new(source_id, "EXT-1".to_string(), EntityType::Citizen, "Zoe".to_string()),
            DataEntity::new(source_id, "EXT-2".to_string(), EntityType::Citizen, "Alice".to_string()),
        ];

        let query = SearchQueryBuilder::new()
            .sort(SortField::Name, SortDirection::Asc)
            .build();

        let result = service.search(query, &entities);

        assert_eq!(result.entities.len(), 2);
        assert_eq!(result.entities[0].name, "Alice");
        assert_eq!(result.entities[1].name, "Zoe");
    }

    #[test]
    fn test_suggestions() {
        let service = SearchService::new();
        let source_id = Uuid::new_v4();
        let entities = vec![
            DataEntity::new(source_id, "EXT-1".to_string(), EntityType::Citizen, "John Doe".to_string()),
            DataEntity::new(source_id, "EXT-2".to_string(), EntityType::Citizen, "Jane Smith".to_string()),
        ];

        let suggestions = service.suggestions("J", &entities);

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.text.starts_with("J")));
    }
}
