//! Registry API Request/Response Types
//!
//! Types for the registry service API endpoints.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::registry::{
    quality::{QualityDimension, QualityScore, QualityThreshold},
    source::SourceType,
    EntityId, EntityType,
};

// ============================================
// SOURCE ENDPOINTS
// ============================================

/// Request for POST /api/v1/registry/sources
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateSourceRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub source_type: SourceType,
    pub connection: SourceConnectionRequest,
    pub owner_id: Uuid,
    pub organization_id: Uuid,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Connection details for creating a data source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SourceConnectionRequest {
    #[serde(rename = "database")]
    Database {
        host: String,
        port: u16,
        database: String,
        username: String,
        password_ref: String,
    },
    #[serde(rename = "api")]
    Api {
        base_url: String,
        api_key_ref: Option<String>,
        auth_type: String,
    },
    #[serde(rename = "file")]
    File { path: String, format: String },
    #[serde(rename = "legacy")]
    Legacy {
        endpoint: String,
        protocol: String,
        credentials_ref: String,
    },
}

/// Request for PUT /api/v1/registry/sources/{id}
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateSourceRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    pub connection: Option<SourceConnectionRequest>,
    pub is_active: Option<bool>,
}

/// Response for GET /api/v1/registry/sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSourcesResponse {
    pub sources: Vec<SourceDetails>,
    pub total_count: usize,
    pub page: usize,
    pub per_page: usize,
}

/// Response for GET /api/v1/registry/sources/{id}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDetailsResponse {
    pub source: SourceDetails,
}

/// Detailed source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDetails {
    pub id: Uuid,
    pub name: String,
    pub source_type: SourceType,
    pub owner_id: Uuid,
    pub organization_id: Uuid,
    pub status: String,
    pub entity_count: usize,
    pub is_active: bool,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub next_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for POST /api/v1/registry/sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSourceResponse {
    pub source_id: Uuid,
    pub status: String,
    pub message: String,
}

// ============================================
// ENTITY ENDPOINTS
// ============================================

/// Request for GET /api/v1/registry/entities
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub struct SearchEntitiesRequest {
    pub entity_type: Option<EntityType>,
    pub source_id: Option<Uuid>,
    #[validate(length(max = 500))]
    pub query: Option<String>,
    pub min_quality: Option<f32>,
    #[serde(default = "default_search_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

fn default_search_limit() -> usize {
    50
}

/// Response for GET /api/v1/registry/entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEntitiesResponse {
    pub entities: Vec<EntityDetails>,
    pub total_count: usize,
    pub execution_time_ms: u64,
}

/// Response for GET /api/v1/registry/entities/{id}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDetailsResponse {
    pub entity: EntityDetails,
}

/// Detailed entity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDetails {
    pub id: EntityId,
    pub canonical_id: Uuid,
    pub source_id: Uuid,
    pub external_id: String,
    pub entity_type: EntityType,
    pub name: String,
    pub metadata: serde_json::Value,
    pub quality_score: QualityScoreDetails,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Quality score details for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScoreDetails {
    pub overall: f32,
    pub completeness: f32,
    pub accuracy: f32,
    pub consistency: f32,
    pub timeliness: f32,
    pub rating: QualityRating,
}

impl From<QualityScore> for QualityScoreDetails {
    fn from(score: QualityScore) -> Self {
        let overall = score.overall();
        Self {
            overall,
            completeness: score.completeness(),
            accuracy: score.accuracy(),
            consistency: score.consistency(),
            timeliness: score.timeliness(),
            rating: QualityRating::from_score(overall),
        }
    }
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

/// Request for GET /api/v1/registry/sources/{id}/entities
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub struct ListSourceEntitiesRequest {
    #[serde(default = "default_search_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
    pub min_quality: Option<f32>,
}

/// Response for GET /api/v1/registry/sources/{id}/entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSourceEntitiesResponse {
    pub entities: Vec<EntityDetails>,
    pub total_count: usize,
    pub source_id: Uuid,
}

// ============================================
// QUALITY ENDPOINTS
// ============================================

/// Request for GET /api/v1/registry/quality
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GetQualityMetricsRequest {
    pub entity_id: Option<Uuid>,
    pub source_id: Option<Uuid>,
    pub dimension: Option<QualityDimension>,
}

/// Response for GET /api/v1/registry/quality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetricsResponse {
    pub overall_score: f32,
    pub rating: QualityRating,
    pub metrics: Vec<QualityMetricDetails>,
    pub entity_count: usize,
    pub calculated_at: DateTime<Utc>,
}

/// Quality metric details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetricDetails {
    pub name: String,
    pub value: f32,
    pub threshold_type: String,
    pub threshold_value: Option<f32>,
    pub passes: bool,
}

/// Request for POST /api/v1/registry/quality/assess
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AssessQualityRequest {
    pub entity_id: Uuid,
    #[serde(default)]
    pub dimensions: Vec<QualityDimension>,
}

/// Response for POST /api/v1/registry/quality/assess
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessQualityResponse {
    pub entity_id: Uuid,
    pub score: QualityScoreDetails,
    pub issues: Vec<QualityIssueDetails>,
    pub recommendations: Vec<String>,
}

/// Quality issue details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssueDetails {
    pub severity: String,
    pub description: String,
    pub affected_field: Option<String>,
    pub suggestion: Option<String>,
}

// ============================================
// SYNC ENDPOINTS
// ============================================

/// Request for POST /api/v1/registry/sync/{source_id}
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TriggerSyncRequest {
    #[serde(default)]
    pub full_sync: bool,
    pub entity_ids: Option<Vec<Uuid>>,
}

/// Response for POST /api/v1/registry/sync/{source_id}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerSyncResponse {
    pub sync_id: Uuid,
    pub source_id: Uuid,
    pub status: String,
    pub message: String,
}

/// Request for GET /api/v1/registry/sync/{source_id}/status
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetSyncStatusRequest {
    pub limit: Option<usize>,
}

/// Response for GET /api/v1/registry/sync/{source_id}/status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusResponse {
    pub source_id: Uuid,
    pub status: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub next_sync_at: Option<DateTime<Utc>>,
    pub entities_synced: usize,
    pub error_message: Option<String>,
    pub recent_syncs: Vec<SyncHistoryItem>,
}

/// Sync history item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHistoryItem {
    pub sync_id: Uuid,
    pub status: String,
    pub entities_synced: usize,
    pub duration_ms: u64,
    pub started_at: DateTime<Utc>,
}

// ============================================
// ERROR RESPONSES
// ============================================

/// Registry API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl RegistryError {
    pub fn not_found(what: &str) -> Self {
        Self {
            code: "NOT_FOUND".to_string(),
            message: format!("{} niet gevonden", what),
            details: None,
        }
    }

    pub fn validation_error(msg: &str) -> Self {
        Self {
            code: "VALIDATION_ERROR".to_string(),
            message: msg.to_string(),
            details: None,
        }
    }

    pub fn conflict(msg: &str) -> Self {
        Self {
            code: "CONFLICT".to_string(),
            message: msg.to_string(),
            details: None,
        }
    }

    pub fn internal_error(msg: &str) -> Self {
        Self {
            code: "INTERNAL_ERROR".to_string(),
            message: msg.to_string(),
            details: None,
        }
    }

    pub fn source_unavailable(source_id: Uuid) -> Self {
        Self {
            code: "SOURCE_UNAVAILABLE".to_string(),
            message: format!("Data source {} is momenteel niet beschikbaar", source_id),
            details: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_source_request_validation() {
        let connection = SourceConnectionRequest::Api {
            base_url: "https://api.example.com".to_string(),
            api_key_ref: None,
            auth_type: "Bearer".to_string(),
        };

        let request = CreateSourceRequest {
            name: "Test API".to_string(),
            source_type: SourceType::Api,
            connection,
            owner_id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            metadata: serde_json::json!({}),
        };

        assert!(request.validate().is_ok());

        let invalid_request = CreateSourceRequest {
            name: "".to_string(),
            ..request
        };

        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_quality_rating_from_score() {
        assert_eq!(QualityRating::from_score(0.95), QualityRating::Excellent);
        assert_eq!(QualityRating::from_score(0.8), QualityRating::Good);
        assert_eq!(QualityRating::from_score(0.6), QualityRating::Acceptable);
        assert_eq!(QualityRating::from_score(0.3), QualityRating::Poor);
    }

    #[test]
    fn test_registry_error_serialization() {
        let error = RegistryError::not_found("Data source");
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("NOT_FOUND"));
    }

    #[test]
    fn test_search_entities_default_limit() {
        let request = SearchEntitiesRequest::default();
        assert_eq!(request.limit, 50);
    }
}
