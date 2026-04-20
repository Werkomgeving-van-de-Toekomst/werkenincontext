//! Registry Service
//!
//! Business logic for the single source of truth registry.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::api::{
    AssessQualityRequest, AssessQualityResponse, CreateSourceRequest, CreateSourceResponse,
    EntityDetails, EntityDetailsResponse, GetQualityMetricsRequest, GetSyncStatusRequest,
    QualityMetricsResponse, QualityScoreDetails, SearchEntitiesRequest, SearchEntitiesResponse,
    ListSourceEntitiesRequest, ListSourceEntitiesResponse, ListSourcesResponse,
    SearchEntitiesResponse as SearchResponse, QualityIssueDetails, QualityMetricDetails,
    QualityRating, RegistryError, SourceDetails, SourceDetailsResponse, SyncHistoryItem,
    SyncStatusResponse, TriggerSyncRequest, TriggerSyncResponse, UpdateSourceRequest,
};
use super::entity::DataEntity;
use super::quality::{QualityAssessment, QualityIssue, QualityScore};
use super::source::{DataSource, SourceConnection, SourceSyncResult, SourceSyncStatus, SourceStatus};
use crate::registry::{EntityType, RegistryConfig, RegistrySearch};

/// Error type for registry operations
#[derive(Debug, Clone)]
pub enum RegistryServiceError {
    NotFound(String),
    ValidationError(String),
    Conflict(String),
    SourceUnavailable(Uuid),
    InternalError(String),
}

impl From<RegistryServiceError> for RegistryError {
    fn from(err: RegistryServiceError) -> Self {
        match err {
            RegistryServiceError::NotFound(msg) => RegistryError::not_found(&msg),
            RegistryServiceError::ValidationError(msg) => RegistryError::validation_error(&msg),
            RegistryServiceError::Conflict(msg) => RegistryError::conflict(&msg),
            RegistryServiceError::SourceUnavailable(id) => RegistryError::source_unavailable(id),
            RegistryServiceError::InternalError(msg) => RegistryError::internal_error(&msg),
        }
    }
}

/// Result type for registry operations
pub type Result<T> = std::result::Result<T, RegistryServiceError>;

/// In-memory registry storage (for demonstration)
/// In production, this would be replaced with a database backend
#[derive(Clone)]
pub struct RegistryStorage {
    sources: Arc<RwLock<HashMap<Uuid, DataSource>>>,
    entities: Arc<RwLock<HashMap<Uuid, DataEntity>>>,
    /// Maps (source_id, external_id) to canonical entity_id
    entity_index: Arc<RwLock<HashMap<(Uuid, String), Uuid>>>,
    sync_history: Arc<RwLock<Vec<SyncHistoryEntry>>>,
    config: RegistryConfig,
}

struct SyncHistoryEntry {
    sync_id: Uuid,
    source_id: Uuid,
    status: String,
    entities_synced: usize,
    duration_ms: u64,
    started_at: DateTime<Utc>,
}

impl RegistryStorage {
    pub fn new(config: RegistryConfig) -> Self {
        Self {
            sources: Arc::new(RwLock::new(HashMap::new())),
            entities: Arc::new(RwLock::new(HashMap::new())),
            entity_index: Arc::new(RwLock::new(HashMap::new())),
            sync_history: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }
}

/// Registry service
#[derive(Clone)]
pub struct RegistryService {
    storage: RegistryStorage,
}

impl RegistryService {
    /// Create a new registry service
    pub fn new(config: RegistryConfig) -> Self {
        Self {
            storage: RegistryStorage::new(config),
        }
    }

    /// Create a new registry service with default config
    pub fn default() -> Self {
        Self::new(RegistryConfig::default())
    }

    // ============================================
    // SOURCE OPERATIONS
    // ============================================

    /// Register a new data source
    pub async fn create_source(&self, request: CreateSourceRequest) -> Result<CreateSourceResponse> {
        let connection = self.convert_connection_request(request.connection)?;

        let mut source = DataSource::new(
            request.name.clone(),
            request.source_type,
            connection,
            request.owner_id,
            request.organization_id,
        );

        // Check if source with same name exists
        {
            let sources = self.storage.sources.read().await;
            if sources.values().any(|s| s.name == request.name) {
                return Err(RegistryServiceError::Conflict(format!(
                    "Data source with name '{}' already exists",
                    request.name
                )));
            }
        }

        let source_id = source.id;
        source.mark_connected();

        {
            let mut sources = self.storage.sources.write().await;
            sources.insert(source_id, source);
        }

        Ok(CreateSourceResponse {
            source_id,
            status: "pending".to_string(),
            message: "Data source registered successfully".to_string(),
        })
    }

    /// Get source details
    pub async fn get_source(&self, source_id: Uuid) -> Result<SourceDetailsResponse> {
        let sources = self.storage.sources.read().await;
        let source = sources
            .get(&source_id)
            .ok_or_else(|| RegistryServiceError::NotFound(format!("Data source {}", source_id)))?;

        Ok(SourceDetailsResponse {
            source: self.source_to_details(source).await,
        })
    }

    /// List all data sources
    pub async fn list_sources(
        &self,
        organization_id: Option<Uuid>,
        page: usize,
        per_page: usize,
    ) -> Result<ListSourcesResponse> {
        let sources = self.storage.sources.read().await;

        let mut filtered: Vec<_> = sources
            .values()
            .filter(|s| match organization_id {
                Some(org_id) => s.organization_id == org_id,
                None => true,
            })
            .collect();

        filtered.sort_by_key(|s| s.created_at);

        let total_count = filtered.len();
        let start = page * per_page;
        let page_sources: Vec<_> = filtered
            .into_iter()
            .skip(start)
            .take(per_page)
            .map(|s| async { self.source_to_details(s).await })
            .collect::<futures::future::JoinAll<_>>()
            .await;

        Ok(ListSourcesResponse {
            sources: page_sources,
            total_count,
            page,
            per_page,
        })
    }

    /// Update a data source
    pub async fn update_source(
        &self,
        source_id: Uuid,
        request: UpdateSourceRequest,
    ) -> Result<SourceDetailsResponse> {
        let mut sources = self.storage.sources.write().await;
        let source = sources
            .get_mut(&source_id)
            .ok_or_else(|| RegistryServiceError::NotFound(format!("Data source {}", source_id)))?;

        if let Some(name) = request.name {
            source.name = name;
        }

        if let Some(is_active) = request.is_active {
            source.is_active = is_active;
        }

        // Note: Connection update would require re-validation
        // For now, we don't support connection updates via this method

        let response = SourceDetailsResponse {
            source: self.source_to_details(source).await,
        };

        Ok(response)
    }

    /// Deactivate a data source
    pub async fn delete_source(&self, source_id: Uuid) -> Result<()> {
        let mut sources = self.storage.sources.write().await;
        let mut source = sources
            .get_mut(&source_id)
            .ok_or_else(|| RegistryServiceError::NotFound(format!("Data source {}", source_id)))?;

        source.is_active = false;

        Ok(())
    }

    // ============================================
    // ENTITY OPERATIONS
    // ============================================

    /// Search entities
    pub async fn search_entities(
        &self,
        request: SearchEntitiesRequest,
    ) -> Result<SearchEntitiesResponse> {
        let start = std::time::Instant::now();

        let entities = self.storage.entities.read().await;
        let mut filtered: Vec<_> = entities
            .values()
            .filter(|e| {
                if !e.is_active {
                    return false;
                }
                if let Some(entity_type) = request.entity_type {
                    if e.entity_type != entity_type {
                        return false;
                    }
                }
                if let Some(source_id) = request.source_id {
                    if e.source_id != source_id {
                        return false;
                    }
                }
                if let Some(min_quality) = request.min_quality {
                    if !e.meets_quality_threshold(min_quality) {
                        return false;
                    }
                }
                if let Some(ref query) = request.query {
                    let query_lower = query.to_lowercase();
                    if !e.name.to_lowercase().contains(&query_lower)
                        && !e.external_id.to_lowercase().contains(&query_lower)
                    {
                        return false;
                    }
                }
                true
            })
            .collect();

        // Sort by quality score (descending)
        filtered.sort_by(|a, b| {
            b.quality_score
                .overall()
                .partial_cmp(&a.quality_score.overall())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let total_count = filtered.len();
        let page_entities: Vec<_> = filtered
            .into_iter()
            .skip(request.offset)
            .take(request.limit)
            .map(|e| self.entity_to_details(e.clone()))
            .collect();

        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(SearchEntitiesResponse {
            entities: page_entities,
            total_count,
            execution_time_ms,
        })
    }

    /// Get entity by ID
    pub async fn get_entity(&self, entity_id: Uuid) -> Result<EntityDetailsResponse> {
        let entities = self.storage.entities.read().await;
        let entity = entities
            .get(&entity_id)
            .ok_or_else(|| RegistryServiceError::NotFound(format!("Entity {}", entity_id)))?;

        Ok(EntityDetailsResponse {
            entity: self.entity_to_details(entity.clone()),
        })
    }

    /// List entities from a specific source
    pub async fn list_source_entities(
        &self,
        source_id: Uuid,
        request: ListSourceEntitiesRequest,
    ) -> Result<ListSourceEntitiesResponse> {
        // Verify source exists
        {
            let sources = self.storage.sources.read().await;
            if !sources.contains_key(&source_id) {
                return Err(RegistryServiceError::NotFound(format!("Data source {}", source_id)));
            }
        }

        let entities = self.storage.entities.read().await;
        let mut filtered: Vec<_> = entities
            .values()
            .filter(|e| e.source_id == source_id && e.is_active)
            .collect();

        let total_count = filtered.len();
        let page_entities: Vec<_> = filtered
            .into_iter()
            .filter(|e| {
                if let Some(min_quality) = request.min_quality {
                    e.meets_quality_threshold(min_quality)
                } else {
                    true
                }
            })
            .skip(request.offset)
            .take(request.limit)
            .map(|e| self.entity_to_details(e.clone()))
            .collect();

        Ok(ListSourceEntitiesResponse {
            entities: page_entities,
            total_count,
            source_id,
        })
    }

    /// Register or update an entity from a source
    pub async fn upsert_entity(&self, entity: DataEntity) -> Result<EntityDetails> {
        let mut entities = self.storage.entities.write().await;
        let mut entity_index = self.storage.entity_index.write().await;

        let key = (entity.source_id, entity.external_id.clone());

        if let Some(&existing_id) = entity_index.get(&key) {
            // Update existing entity
            if let Some(existing) = entities.get_mut(&existing_id) {
                existing.name = entity.name;
                existing.metadata = entity.metadata;
                existing.quality_score = entity.quality_score;
                existing.updated_at = Utc::now();
                existing.mark_synced();

                return Ok(self.entity_to_details(existing.clone()));
            }
        }

        // Insert new entity
        let entity_id = entity.id;
        entities.insert(entity_id, entity.clone());
        entity_index.insert(key, entity_id);

        Ok(self.entity_to_details(entity))
    }

    // ============================================
    // QUALITY OPERATIONS
    // ============================================

    /// Get quality metrics
    pub async fn get_quality_metrics(
        &self,
        request: GetQualityMetricsRequest,
    ) -> Result<QualityMetricsResponse> {
        let entities = self.storage.entities.read().await;

        let mut filtered: Vec<_> = entities.values().filter(|e| e.is_active).collect();

        if let Some(source_id) = request.source_id {
            filtered.retain(|e| e.source_id == source_id);
        }

        if let Some(entity_id) = request.entity_id {
            filtered.retain(|e| e.id == entity_id);
        }

        if filtered.is_empty() {
            return Ok(QualityMetricsResponse {
                overall_score: 0.0,
                rating: QualityRating::Poor,
                metrics: vec![],
                entity_count: 0,
                calculated_at: Utc::now(),
            });
        }

        // Calculate aggregate scores
        let count = filtered.len() as f32;
        let avg_completeness: f32 = filtered.iter().map(|e| e.quality_score.completeness()).sum::<f32>() / count;
        let avg_accuracy: f32 = filtered.iter().map(|e| e.quality_score.accuracy()).sum::<f32>() / count;
        let avg_consistency: f32 = filtered.iter().map(|e| e.quality_score.consistency()).sum::<f32>() / count;
        let avg_timeliness: f32 = filtered.iter().map(|e| e.quality_score.timeliness()).sum::<f32>() / count;

        let overall = (avg_completeness * 0.3
            + avg_accuracy * 0.3
            + avg_consistency * 0.2
            + avg_timeliness * 0.2);

        let metrics = vec![
            QualityMetricDetails {
                name: "Completeness".to_string(),
                value: avg_completeness,
                threshold_type: "minimum".to_string(),
                threshold_value: Some(0.8),
                passes: avg_completeness >= 0.8,
            },
            QualityMetricDetails {
                name: "Accuracy".to_string(),
                value: avg_accuracy,
                threshold_type: "minimum".to_string(),
                threshold_value: Some(0.9),
                passes: avg_accuracy >= 0.9,
            },
            QualityMetricDetails {
                name: "Consistency".to_string(),
                value: avg_consistency,
                threshold_type: "minimum".to_string(),
                threshold_value: Some(0.7),
                passes: avg_consistency >= 0.7,
            },
            QualityMetricDetails {
                name: "Timeliness".to_string(),
                value: avg_timeliness,
                threshold_type: "minimum".to_string(),
                threshold_value: Some(0.6),
                passes: avg_timeliness >= 0.6,
            },
        ];

        Ok(QualityMetricsResponse {
            overall_score: overall,
            rating: QualityRating::from_score(overall),
            metrics,
            entity_count: filtered.len(),
            calculated_at: Utc::now(),
        })
    }

    /// Assess quality of a specific entity
    pub async fn assess_quality(
        &self,
        request: AssessQualityRequest,
    ) -> Result<AssessQualityResponse> {
        let entities = self.storage.entities.read().await;
        let entity = entities
            .get(&request.entity_id)
            .ok_or_else(|| RegistryServiceError::NotFound(format!("Entity {}", request.entity_id)))?;

        // Convert quality score to details
        let score_details = QualityScoreDetails::from(entity.quality_score);

        // Generate quality issues based on score
        let issues = self.generate_quality_issues(&entity, &entity.quality_score);

        // Generate recommendations
        let recommendations = self.generate_quality_recommendations(&entity);

        Ok(AssessQualityResponse {
            entity_id: entity.id,
            score: score_details,
            issues,
            recommendations,
        })
    }

    // ============================================
    // SYNC OPERATIONS
    // ============================================

    /// Trigger a sync from a source
    pub async fn trigger_sync(
        &self,
        source_id: Uuid,
        _request: TriggerSyncRequest,
    ) -> Result<TriggerSyncResponse> {
        let mut sources = self.storage.sources.write().await;
        let source = sources
            .get_mut(&source_id)
            .ok_or_else(|| RegistryServiceError::NotFound(format!("Data source {}", source_id)))?;

        if !source.is_healthy() {
            return Err(RegistryServiceError::SourceUnavailable(source_id));
        }

        // Update source status
        source.status = SourceStatus::Syncing {
            started_at: Utc::now(),
        };

        let sync_id = Uuid::new_v4();

        // In a real implementation, this would trigger a background job
        // For now, we'll mark it as immediately successful
        source.mark_synced();
        source.mark_connected();

        Ok(TriggerSyncResponse {
            sync_id,
            source_id,
            status: "started".to_string(),
            message: "Sync job started".to_string(),
        })
    }

    /// Get sync status for a source
    pub async fn get_sync_status(
        &self,
        source_id: Uuid,
        _request: GetSyncStatusRequest,
    ) -> Result<SyncStatusResponse> {
        let sources = self.storage.sources.read().await;
        let source = sources
            .get(&source_id)
            .ok_or_else(|| RegistryServiceError::NotFound(format!("Data source {}", source_id)))?;

        let sync_history = self.storage.sync_history.read().await;
        let recent_syncs: Vec<_> = sync_history
            .iter()
            .filter(|s| s.source_id == source_id)
            .rev()
            .take(10)
            .map(|s| SyncHistoryItem {
                sync_id: s.sync_id,
                status: s.status.clone(),
                entities_synced: s.entities_synced,
                duration_ms: s.duration_ms,
                started_at: s.started_at,
            })
            .collect();

        let status = match &source.status {
            SourceStatus::Syncing { .. } => "syncing".to_string(),
            SourceStatus::Failed { .. } => "failed".to_string(),
            SourceStatus::Disabled => "disabled".to_string(),
            SourceStatus::Unreachable { .. } => "unreachable".to_string(),
            _ => "idle".to_string(),
        };

        Ok(SyncStatusResponse {
            source_id,
            status,
            last_synced_at: source.last_synced_at,
            next_sync_at: source.next_sync_at,
            entities_synced: source.entity_count,
            error_message: match &source.status {
                SourceStatus::Failed { error } => Some(error.clone()),
                _ => None,
            },
            recent_syncs,
        })
    }

    // ============================================
    // HELPER METHODS
    // ============================================

    async fn source_to_details(&self, source: &DataSource) -> SourceDetails {
        let status = match &source.status {
            SourceStatus::Pending => "pending".to_string(),
            SourceStatus::Connected => "connected".to_string(),
            SourceStatus::Syncing { .. } => "syncing".to_string(),
            SourceStatus::Failed { .. } => "failed".to_string(),
            SourceStatus::Disabled => "disabled".to_string(),
            SourceStatus::Unreachable { .. } => "unreachable".to_string(),
        };

        SourceDetails {
            id: source.id,
            name: source.name.clone(),
            source_type: source.source_type,
            owner_id: source.owner_id,
            organization_id: source.organization_id,
            status,
            entity_count: source.entity_count,
            is_active: source.is_active,
            last_synced_at: source.last_synced_at,
            next_sync_at: source.next_sync_at,
            created_at: source.created_at,
            updated_at: source.updated_at,
        }
    }

    fn entity_to_details(&self, entity: DataEntity) -> EntityDetails {
        EntityDetails {
            id: entity.id,
            canonical_id: entity.canonical_id,
            source_id: entity.source_id,
            external_id: entity.external_id,
            entity_type: entity.entity_type,
            name: entity.name,
            metadata: entity.metadata,
            quality_score: QualityScoreDetails::from(entity.quality_score),
            last_synced_at: entity.last_synced_at,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            is_active: entity.is_active,
        }
    }

    fn convert_connection_request(
        &self,
        request: super::api::SourceConnectionRequest,
    ) -> Result<SourceConnection> {
        Ok(match request {
            super::api::SourceConnectionRequest::Database {
                host,
                port,
                database,
                username,
                password_ref,
            } => SourceConnection::Database {
                host,
                port,
                database,
                username,
                password_ref,
            },
            super::api::SourceConnectionRequest::Api {
                base_url,
                api_key_ref,
                auth_type,
            } => {
                let auth = match auth_type.as_str() {
                    "none" | "None" => super::source::ApiAuthType::None,
                    "basic" | "Basic" => super::source::ApiAuthType::Basic,
                    "bearer" | "Bearer" => super::source::ApiAuthType::Bearer,
                    "api_key" | "ApiKey" => super::source::ApiAuthType::ApiKey,
                    "oauth2" | "OAuth2" => super::source::ApiAuthType::OAuth2,
                    _ => super::source::ApiAuthType::Bearer,
                };
                SourceConnection::Api {
                    base_url,
                    api_key_ref,
                    auth_type: auth,
                }
            }
            super::api::SourceConnectionRequest::File { path, format } => {
                let file_format = match format.to_lowercase().as_str() {
                    "csv" => super::source::FileFormat::Csv,
                    "json" => super::source::FileFormat::Json,
                    "xml" => super::source::FileFormat::Xml,
                    "excel" => super::source::FileFormat::Excel,
                    "parquet" => super::source::FileFormat::Parquet,
                    _ => super::source::FileFormat::Csv,
                };
                SourceConnection::File {
                    path,
                    format: file_format,
                }
            }
            super::api::SourceConnectionRequest::Legacy {
                endpoint,
                protocol,
                credentials_ref,
            } => SourceConnection::Legacy {
                endpoint,
                protocol,
                credentials_ref,
            },
        })
    }

    fn generate_quality_issues(&self, entity: &DataEntity, score: &QualityScore) -> Vec<QualityIssueDetails> {
        let mut issues = Vec::new();

        // Check completeness
        if score.completeness() < 0.8 {
            issues.push(QualityIssueDetails {
                severity: "medium".to_string(),
                description: "Low completeness score".to_string(),
                affected_field: Some("metadata".to_string()),
                suggestion: Some("Ensure all required fields are populated".to_string()),
            });
        }

        // Check accuracy
        if score.accuracy() < 0.9 {
            issues.push(QualityIssueDetails {
                severity: "high".to_string(),
                description: "Data validation issues detected".to_string(),
                affected_field: Some("validation".to_string()),
                suggestion: Some("Review validation rules for this entity type".to_string()),
            });
        }

        // Check if entity is stale
        if let Some(last_synced) = entity.last_synced_at {
            let days_since_sync = (Utc::now() - last_synced).num_days();
            if days_since_sync > 30 {
                issues.push(QualityIssueDetails {
                    severity: "low".to_string(),
                    description: format!("Entity not synced for {} days", days_since_sync),
                    affected_field: Some("last_synced_at".to_string()),
                    suggestion: Some("Trigger a sync from the source system".to_string()),
                });
            }
        }

        issues
    }

    fn generate_quality_recommendations(&self, entity: &DataEntity) -> Vec<String> {
        let mut recommendations = Vec::new();

        if entity.quality_score.overall() < 0.7 {
            recommendations.push("Consider reviewing data entry workflows for this entity type".to_string());
        }

        if entity.metadata.is_null() || entity.metadata.as_object().map_or(true, |m| m.is_empty()) {
            recommendations.push("Add enriching metadata to improve searchability".to_string());
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::source::SourceType;

    #[tokio::test]
    async fn test_create_source() {
        let service = RegistryService::default();

        let request = CreateSourceRequest {
            name: "Test API".to_string(),
            source_type: SourceType::Api,
            connection: super::super::api::SourceConnectionRequest::Api {
                base_url: "https://api.example.com".to_string(),
                api_key_ref: None,
                auth_type: "Bearer".to_string(),
            },
            owner_id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            metadata: serde_json::json!({}),
        };

        let result = service.create_source(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status, "pending");
    }

    #[tokio::test]
    async fn test_get_source() {
        let service = RegistryService::default();

        let request = CreateSourceRequest {
            name: "Test API".to_string(),
            source_type: SourceType::Api,
            connection: super::super::api::SourceConnectionRequest::Api {
                base_url: "https://api.example.com".to_string(),
                api_key_ref: None,
                auth_type: "Bearer".to_string(),
            },
            owner_id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            metadata: serde_json::json!({}),
        };

        let create_response = service.create_source(request).await.unwrap();
        let result = service.get_source(create_response.source_id).await;

        assert!(result.is_ok());
        let details = result.unwrap();
        assert_eq!(details.source.name, "Test API");
    }

    #[tokio::test]
    async fn test_search_entities_empty() {
        let service = RegistryService::default();

        let request = SearchEntitiesRequest::default();
        let result = service.search_entities(request).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.entities.len(), 0);
        assert_eq!(response.total_count, 0);
    }

    #[tokio::test]
    async fn test_upsert_entity() {
        let service = RegistryService::default();
        let source_id = Uuid::new_v4();

        let entity = DataEntity::new(
            source_id,
            "EXT-123".to_string(),
            EntityType::Citizen,
            "John Doe".to_string(),
        );

        let result = service.upsert_entity(entity).await;
        assert!(result.is_ok());

        let details = result.unwrap();
        assert_eq!(details.external_id, "EXT-123");
        assert_eq!(details.name, "John Doe");
    }

    #[tokio::test]
    async fn test_get_quality_metrics() {
        let service = RegistryService::default();

        let request = GetQualityMetricsRequest {
            entity_id: None,
            source_id: None,
            dimension: None,
        };

        let result = service.get_quality_metrics(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.entity_count, 0);
        assert_eq!(response.rating, QualityRating::Poor);
    }
}
