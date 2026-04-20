//! Single Source of Truth Registry
//!
//! Implementeert IHH02: Central register voor alle informatieobjecten
//! om duplicaten en inconsistenties te voorkomen.

mod api;
mod entity;
mod quality;
mod quality_engine;
mod search;
mod service;
mod source;
mod source_reader;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use api::*;
pub use entity::{DataEntity, EntityId, EntityType};
pub use quality::{DataQualityMetric, QualityScore, QualityThreshold};
pub use quality_engine::{
    AggregationType, ComparisonOperator, DefaultRuleSets, DimensionScore, IssueSeverity,
    QualityAggregation, QualityCalculation, QualityEngine, QualityIssueResult, QualityRule,
    QualityRuleSet, RuleCondition, ScoreDistribution, TrendDirection, TrendSubjectType,
};
pub use search::{
    AdvancedSearchQuery, DateField, DateFilter, FilterOperator, LogicOperator, MatchHighlight,
    MetadataFilter, PageInfo, Pagination, QualityFilter, QualityRating, SavedSearch, SearchClause,
    SearchFacets, SearchFilters, SearchOperator, SearchOptions, SearchQueryBuilder, SearchQuery,
    SearchResult, SearchResultEntity, SearchService, SearchSuggestion, SearchValue, SortDirection,
    SortField, SortSpec, SuggestionType,
};
pub use service::{RegistryService, RegistryServiceError, Result};
pub use source::{DataSource, SourceConnection, SourceStatus, SourceType};
pub use source_reader::{
    ApiParam, ApiTemplate, ApiTemplateBuilder, DatabaseQueryType, DatabaseTemplate,
    DatabaseTemplateBuilder, FieldMapping, FileTemplate, FileTemplateBuilder, GraphQLTemplate,
    LegacyTemplate, OutputParser, PaginationConfig, PaginationType, ParamValue,
    ReadError, ReadErrorType, ReadOptions, ReadPhase, ReadProgress, ReadTemplate,
    ReadTemplateBuilder, SoapTemplate, SourceReadConfig, SourceReadResult, TemplatePresets,
    Transform,
};

/// Registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Maximum number of entities per source
    pub max_entities_per_source: usize,

    /// Default sync interval in seconds
    pub default_sync_interval_seconds: u64,

    /// Quality threshold for entity acceptance
    pub min_quality_threshold: f32,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            max_entities_per_source: 1_000_000,
            default_sync_interval_seconds: 3600, // 1 hour
            min_quality_threshold: 0.5,
        }
    }
}

/// Registry search filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegistrySearch {
    /// Entity type filter
    pub entity_type: Option<EntityType>,

    /// Source ID filter
    pub source_id: Option<Uuid>,

    /// Text search in name/metadata
    pub query: Option<String>,

    /// Minimum quality score
    pub min_quality: Option<f32>,

    /// Limit results
    pub limit: Option<usize>,

    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Registry search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySearchResults {
    /// Total matching entities
    pub total_count: usize,

    /// Returned entities
    pub entities: Vec<DataEntity>,

    /// Search execution time
    pub execution_time_ms: u64,
}

/// Sync status for a data source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    /// Source ID
    pub source_id: Uuid,

    /// Last sync timestamp
    pub last_synced_at: Option<DateTime<Utc>>,

    /// Last sync status
    pub last_sync_status: SyncStatusType,

    /// Next scheduled sync
    pub next_sync_at: Option<DateTime<Utc>>,

    /// Entities synced count
    pub entities_synced: usize,

    /// Error message if sync failed
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatusType {
    Pending,
    Running,
    Success,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_config_default() {
        let config = RegistryConfig::default();
        assert_eq!(config.max_entities_per_source, 1_000_000);
        assert_eq!(config.default_sync_interval_seconds, 3600);
        assert_eq!(config.min_quality_threshold, 0.5);
    }

    #[test]
    fn test_registry_search_default() {
        let search = RegistrySearch::default();
        assert!(search.entity_type.is_none());
        assert!(search.query.is_none());
        assert!(search.limit.is_none());
    }

    #[test]
    fn test_registry_search_with_filters() {
        let search = RegistrySearch {
            entity_type: Some(EntityType::Citizen),
            min_quality: Some(0.8),
            limit: Some(100),
            ..Default::default()
        };

        assert!(search.entity_type.is_some());
        assert_eq!(search.limit, Some(100));
    }
}
