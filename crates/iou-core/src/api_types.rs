//! API request en response types
//!
//! Gedeelde types voor communicatie tussen frontend en backend.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::compliance::{Classification, ComplianceStatus};
use crate::domain::{Case, DomainType, InformationDomain, Project};
use crate::graphrag::DomainRelation;
use crate::objects::{InformationObject, ObjectReference, ObjectType};

// ============================================
// CONTEXT ENDPOINTS
// ============================================

/// Response voor GET /context/{id}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextResponse {
    pub current_domain: InformationDomain,
    pub domain_details: Option<DomainDetails>,
    pub related_domains: Vec<RelatedDomainInfo>,
    pub recent_objects: Vec<ObjectReference>,
    pub recommended_apps: Vec<AppRecommendation>,
    pub stakeholders: Vec<StakeholderInfo>,
    pub compliance_status: Option<ComplianceStatus>,
}

/// Details specifiek voor domeintype (zaak, project, beleid)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DomainDetails {
    #[serde(rename = "zaak")]
    Zaak(Case),
    #[serde(rename = "project")]
    Project(Project),
    // Beleid en Expertise kunnen later toegevoegd worden
}

/// Gerelateerd domein met relatie-informatie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedDomainInfo {
    pub domain: InformationDomain,
    pub relation_type: String,
    pub strength: f32,
    pub shared_entities: Vec<String>,
}

/// Request voor POST /domains
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDomainRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub domain_type: DomainType,
    pub description: Option<String>,
    pub organization_id: Uuid,
    pub owner_user_id: Option<Uuid>,
    pub parent_domain_id: Option<Uuid>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Response voor POST /domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDomainResponse {
    pub domain: InformationDomain,
    pub ai_suggestions: Vec<MetadataSuggestion>,
}

// ============================================
// OBJECT ENDPOINTS
// ============================================

/// Request voor POST /objects
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateObjectRequest {
    pub domain_id: Uuid,
    pub object_type: ObjectType,
    #[validate(length(min = 1, max = 500))]
    pub title: String,
    pub description: Option<String>,
    pub content_location: String,
    pub content_text: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,

    // Optionele compliance overrides
    pub classification: Option<Classification>,
    pub is_woo_relevant: Option<bool>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Response voor POST /objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateObjectResponse {
    pub object: InformationObject,
    pub applied_rules: Vec<AppliedRule>,
    pub ai_suggestions: Vec<MetadataSuggestion>,
}

/// Automatisch toegepaste business rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedRule {
    pub rule_name: String,
    pub description: String,
    pub field_modified: String,
    pub new_value: serde_json::Value,
}

// ============================================
// SEARCH ENDPOINTS
// ============================================

/// Request voor GET /search
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SearchRequest {
    #[validate(length(min = 2, max = 500))]
    pub query: String,
    pub domain_id: Option<Uuid>,
    pub object_types: Option<Vec<ObjectType>>,
    pub classification: Option<Classification>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
}

fn default_limit() -> i32 {
    50
}

/// Zoekresultaat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub object_type: ObjectType,
    pub title: String,
    pub snippet: String,
    pub domain_id: Uuid,
    pub domain_name: String,
    pub classification: Classification,
    pub relevance_score: f32,
    pub created_at: DateTime<Utc>,
}

/// Response voor GET /search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_count: i64,
    pub query: String,
    pub took_ms: u64,
}

// ============================================
// APP RECOMMENDATIONS
// ============================================

/// Aanbevolen app voor de huidige context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRecommendation {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub app_type: String,
    pub icon_url: Option<String>,
    pub endpoint_url: String,
    pub relevance_score: f32,
    pub reason: String,
    pub badge: Option<AppBadge>,
}

/// Badge voor een app (nieuw, aanbevolen, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppBadge {
    pub label: String,
    pub color: String,
}

// ============================================
// STAKEHOLDERS
// ============================================

/// Stakeholder informatie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeholderInfo {
    pub id: Uuid,
    pub name: String,
    pub stakeholder_type: String,
    pub role_in_domain: String,
    pub contact_email: Option<String>,
    pub organization: Option<String>,
}

// ============================================
// AI SUGGESTIONS
// ============================================

/// AI metadata suggestie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataSuggestion {
    pub id: Uuid,
    pub field: String,
    pub suggested_value: serde_json::Value,
    pub confidence: f32,
    pub reasoning: String,
    pub source: SuggestionSource,
}

/// Bron van de suggestie
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionSource {
    /// Named Entity Recognition
    Ner,
    /// Text classificatie
    Classification,
    /// Regelgebaseerd
    RuleBased,
    /// Pattern matching
    PatternMatching,
    /// Embedding similarity
    SemanticSimilarity,
}

/// Feedback op AI suggestie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionFeedback {
    pub suggestion_id: Uuid,
    pub accepted: bool,
    pub modified_value: Option<serde_json::Value>,
    pub feedback_text: Option<String>,
}

// ============================================
// GRAPHRAG ENDPOINTS
// ============================================

/// Response voor GET /graphrag/relations/{domain_id}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRelationsResponse {
    pub domain_id: Uuid,
    pub relations: Vec<DomainRelation>,
    pub entities: Vec<EntityInfo>,
    pub communities: Vec<CommunityInfo>,
}

/// Entiteit info voor frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInfo {
    pub id: Uuid,
    pub name: String,
    pub entity_type: String,
    pub domain_count: i32,
}

/// Community info voor frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityInfo {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub member_count: i32,
    pub keywords: Vec<String>,
}

// ============================================
// ERROR TYPES
// ============================================

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn not_found(message: &str) -> Self {
        Self {
            code: "NOT_FOUND".to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn validation_error(message: &str, details: serde_json::Value) -> Self {
        Self {
            code: "VALIDATION_ERROR".to_string(),
            message: message.to_string(),
            details: Some(details),
        }
    }

    pub fn unauthorized(message: &str) -> Self {
        Self {
            code: "UNAUTHORIZED".to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn internal_error(message: &str) -> Self {
        Self {
            code: "INTERNAL_ERROR".to_string(),
            message: message.to_string(),
            details: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_domain_request_validation() {
        let valid_request = CreateDomainRequest {
            name: "Test Domain".to_string(),
            domain_type: DomainType::Zaak,
            description: None,
            organization_id: Uuid::new_v4(),
            owner_user_id: None,
            parent_domain_id: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = CreateDomainRequest {
            name: "".to_string(), // Te kort
            ..valid_request
        };

        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_api_error_serialization() {
        let error = ApiError::not_found("Domain niet gevonden");
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("NOT_FOUND"));
    }
}
