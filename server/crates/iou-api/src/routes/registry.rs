//! Registry API endpoints
//!
//! HTTP handlers for the Single Source of Truth Registry (IHH02)

use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use iou_core::registry::{
    AssessQualityRequest, AssessQualityResponse, CreateSourceRequest, CreateSourceResponse,
    EntityDetails, EntityDetailsResponse, GetQualityMetricsRequest, GetSyncStatusRequest,
    ListSourceEntitiesRequest, ListSourceEntitiesResponse, ListSourcesResponse,
    QualityMetricsResponse, RegistryError, RegistryService, SearchEntitiesRequest,
    SearchEntitiesResponse as SearchResponse, SourceDetails, SourceDetailsResponse,
    SyncStatusResponse, TriggerSyncRequest, TriggerSyncResponse, UpdateSourceRequest,
};

/// Registry service state (shared via Axum Extension)
#[derive(Clone)]
pub struct RegistryState {
    pub service: Arc<RegistryService>,
}

impl RegistryState {
    pub fn new(service: RegistryService) -> Self {
        Self {
            service: Arc::new(service),
        }
    }
}

// ============================================
// SOURCE ENDPOINTS
// ============================================

/// GET /api/v1/registry/sources - List all data sources
pub async fn list_sources(
    Query(params): Query<ListSourcesParams>,
    Extension(state): Extension<Arc<RegistryState>>,
) -> Result<Json<ListSourcesResponse>, ApiError> {
    let result = state
        .service
        .list_sources(
            params.organization_id,
            params.page.unwrap_or(0),
            params.per_page.unwrap_or(50),
        )
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct ListSourcesParams {
    pub organization_id: Option<Uuid>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

/// POST /api/v1/registry/sources - Register new data source
pub async fn create_source(
    Extension(state): Extension<Arc<RegistryState>>,
    Json(request): Json<CreateSourceRequest>,
) -> Result<Json<CreateSourceResponse>, ApiError> {
    let result = state
        .service
        .create_source(request)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    Ok(Json(result))
}

/// GET /api/v1/registry/sources/:id - Get source details
pub async fn get_source(
    Path(source_id): Path<Uuid>,
    Extension(state): Extension<Arc<RegistryState>>,
) -> Result<Json<SourceDetailsResponse>, ApiError> {
    let result = state
        .service
        .get_source(source_id)
        .await
        .map_err(|e| match e {
            RegistryServiceError::NotFound(msg) => ApiError::NotFound(msg),
            _ => ApiError::Internal(e.into()),
        })?;

    Ok(Json(result))
}

/// PUT /api/v1/registry/sources/:id - Update source
pub async fn update_source(
    Path(source_id): Path<Uuid>,
    Extension(state): Extension<Arc<RegistryState>>,
    Json(request): Json<UpdateSourceRequest>,
) -> Result<Json<SourceDetailsResponse>, ApiError> {
    let result = state
        .service
        .update_source(source_id, request)
        .await
        .map_err(|e| match e {
            RegistryServiceError::NotFound(msg) => ApiError::NotFound(msg),
            _ => ApiError::Internal(e.into()),
        })?;

    Ok(Json(result))
}

/// DELETE /api/v1/registry/sources/:id - Deactivate source
pub async fn delete_source(
    Path(source_id): Path<Uuid>,
    Extension(state): Extension<Arc<RegistryState>>,
) -> Result<Json<DeletedResponse>, ApiError> {
    state
        .service
        .delete_source(source_id)
        .await
        .map_err(|e| match e {
            RegistryServiceError::NotFound(msg) => ApiError::NotFound(msg),
            _ => ApiError::Internal(e.into()),
        })?;

    Ok(Json(DeletedResponse {
        deleted: source_id,
        message: "Data source deactivated".to_string(),
    }))
}

#[derive(Debug, Serialize)]
pub struct DeletedResponse {
    pub deleted: Uuid,
    pub message: String,
}

// ============================================
// ENTITY ENDPOINTS
// ============================================

/// GET /api/v1/registry/entities - Search entities
pub async fn search_entities(
    Query(params): Query<SearchEntitiesParams>,
    Extension(state): Extension<Arc<RegistryState>>,
) -> Result<Json<SearchResponse>, ApiError> {
    let request = SearchEntitiesRequest {
        entity_type: params.entity_type,
        source_id: params.source_id,
        query: params.query,
        min_quality: params.min_quality,
        limit: params.limit.unwrap_or(50),
        offset: params.offset.unwrap_or(0),
    };

    let result = state
        .service
        .search_entities(request)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct SearchEntitiesParams {
    pub entity_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub query: Option<String>,
    pub min_quality: Option<f32>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// GET /api/v1/registry/entities/:id - Get entity by canonical ID
pub async fn get_entity(
    Path(entity_id): Path<Uuid>,
    Extension(state): Extension<Arc<RegistryState>>,
) -> Result<Json<EntityDetailsResponse>, ApiError> {
    let result = state
        .service
        .get_entity(entity_id)
        .await
        .map_err(|e| match e {
            RegistryServiceError::NotFound(msg) => ApiError::NotFound(msg),
            _ => ApiError::Internal(e.into()),
        })?;

    Ok(Json(result))
}

/// GET /api/v1/registry/sources/:id/entities - Get entities from source
pub async fn list_source_entities(
    Path(source_id): Path<Uuid>,
    Query(params): Query<ListSourceEntitiesParams>,
    Extension(state): Extension<Arc<RegistryState>>,
) -> Result<Json<ListSourceEntitiesResponse>, ApiError> {
    let request = ListSourceEntitiesRequest {
        limit: params.limit.unwrap_or(50),
        offset: params.offset.unwrap_or(0),
        min_quality: params.min_quality,
    };

    let result = state
        .service
        .list_source_entities(source_id, request)
        .await
        .map_err(|e| match e {
            RegistryServiceError::NotFound(msg) => ApiError::NotFound(msg),
            _ => ApiError::Internal(e.into()),
        })?;

    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct ListSourceEntitiesParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub min_quality: Option<f32>,
}

// ============================================
// QUALITY ENDPOINTS
// ============================================

/// GET /api/v1/registry/quality - Get quality metrics
pub async fn get_quality_metrics(
    Query(params): Query<QualityMetricsParams>,
    Extension(state): Extension<Arc<RegistryState>>,
) -> Result<Json<QualityMetricsResponse>, ApiError> {
    let request = GetQualityMetricsRequest {
        entity_id: params.entity_id,
        source_id: params.source_id,
        dimension: None, // TODO: parse from params
    };

    let result = state
        .service
        .get_quality_metrics(request)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct QualityMetricsParams {
    pub entity_id: Option<Uuid>,
    pub source_id: Option<Uuid>,
}

/// POST /api/v1/registry/quality/assess - Assess entity quality
pub async fn assess_quality(
    Json(request): Json<AssessQualityRequest>,
    Extension(state): Extension<Arc<RegistryState>>,
) -> Result<Json<AssessQualityResponse>, ApiError> {
    let result = state
        .service
        .assess_quality(request)
        .await
        .map_err(|e| match e {
            RegistryServiceError::NotFound(msg) => ApiError::NotFound(msg),
            _ => ApiError::Internal(e.into()),
        })?;

    Ok(Json(result))
}

// ============================================
// SYNC ENDPOINTS
// ============================================

/// POST /api/v1/registry/sync/:source_id - Trigger sync from source
pub async fn trigger_sync(
    Path(source_id): Path<Uuid>,
    Extension(state): Extension<Arc<RegistryState>>,
    Json(request): Json<TriggerSyncRequest>,
) -> Result<Json<TriggerSyncResponse>, ApiError> {
    let result = state
        .service
        .trigger_sync(source_id, request)
        .await
        .map_err(|e| match e {
            RegistryServiceError::NotFound(msg) => ApiError::NotFound(msg),
            RegistryServiceError::SourceUnavailable(id) => {
                ApiError::ServiceUnavailable(format!("Source {} is unavailable", id))
            }
            _ => ApiError::Internal(e.into()),
        })?;

    Ok(Json(result))
}

/// GET /api/v1/registry/sync/:source_id/status - Get sync status
pub async fn get_sync_status(
    Path(source_id): Path<Uuid>,
    Query(params): Query<SyncStatusParams>,
    Extension(state): Extension<Arc<RegistryState>>,
) -> Result<Json<SyncStatusResponse>, ApiError> {
    let request = GetSyncStatusRequest {
        limit: params.limit,
    };

    let result = state
        .service
        .get_sync_status(source_id, request)
        .await
        .map_err(|e| match e {
            RegistryServiceError::NotFound(msg) => ApiError::NotFound(msg),
            _ => ApiError::Internal(e.into()),
        })?;

    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct SyncStatusParams {
    pub limit: Option<usize>,
}

// ============================================
// HEALTH ENDPOINT
// ============================================

/// GET /api/v1/registry/health - Registry health check
pub async fn registry_health(
    Extension(state): Extension<Arc<RegistryState>>,
) -> Result<Json<RegistryHealthResponse>, ApiError> {
    // Check if service is accessible by attempting a simple operation
    let sources = state
        .service
        .list_sources(None, 0, 1)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    let entity_count = sources.total_count;

    Ok(Json(RegistryHealthResponse {
        status: "healthy".to_string(),
        service: "iou-registry".to_string(),
        version: env!("CARGO_PKG_VERSION"),
        sources_count: entity_count,
        uptime_seconds: get_system_uptime(),
    }))
}

#[derive(Debug, Serialize)]
pub struct RegistryHealthResponse {
    pub status: String,
    pub service: String,
    pub version: &'static str,
    pub sources_count: usize,
    pub uptime_seconds: u64,
}

fn get_system_uptime() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ============================================
// BATCH OPERATIONS
// ============================================

/// POST /api/v1/registry/entities/batch - Batch upsert entities
pub async fn batch_upsert_entities(
    Extension(state): Extension<Arc<RegistryState>>,
    Json(request): Json<BatchUpsertRequest>,
) -> Result<Json<BatchUpsertResponse>, ApiError> {
    let mut results = Vec::new();
    let mut failed = 0;

    for entity_data in request.entities {
        match state.service.upsert_entity(entity_data).await {
            Ok(entity) => results.push(entity),
            Err(_) => failed += 1,
        }
    }

    Ok(Json(BatchUpsertResponse {
        created: results.len(),
        failed,
        entities: results,
    }))
}

#[derive(Debug, Deserialize)]
pub struct BatchUpsertRequest {
    pub entities: Vec<EntityData>,
}

#[derive(Debug, Serialize)]
pub struct BatchUpsertResponse {
    pub created: usize,
    pub failed: usize,
    pub entities: Vec<EntityDetails>,
}

/// Simplified entity data for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    pub source_id: Uuid,
    pub external_id: String,
    pub entity_type: String,
    pub name: String,
    pub metadata: serde_json::Value,
}

// Need to import the service error type
use iou_core::registry::RegistryServiceError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deleted_response_serialization() {
        let response = DeletedResponse {
            deleted: Uuid::new_v4(),
            message: "Test".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("deleted"));
        assert!(json.contains("message"));
    }
}
