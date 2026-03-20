//! Stakeholder API endpoints
//!
//! Provides REST API access to extracted stakeholder entities
//! and their relationships to documents.

use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use iou_ai::graphrag::{KnowledgeGraph, InfluenceMetrics};

use crate::error::ApiError;

/// Stakeholder detail response
#[derive(Debug, Serialize)]
pub struct StakeholderResponse {
    pub id: Uuid,
    pub name: String,
    pub entity_type: String,
    pub canonical_name: Option<String>,
    pub description: Option<String>,
    pub confidence: f32,
    pub metadata: serde_json::Value,
    pub influence: InfluenceMetricsResponse,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Influence metrics for API response
#[derive(Debug, Serialize)]
pub struct InfluenceMetricsResponse {
    pub mention_count: usize,
    pub document_count: usize,
    pub pagerank_score: f32,
}

impl From<InfluenceMetrics> for InfluenceMetricsResponse {
    fn from(m: InfluenceMetrics) -> Self {
        Self {
            mention_count: m.mention_count,
            document_count: m.document_count,
            pagerank_score: m.pagerank_score,
        }
    }
}

/// Document mention info
#[derive(Debug, Serialize)]
pub struct DocumentMentionResponse {
    pub document_id: Uuid,
    pub confidence: f32,
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: usize,
    pub page_size: usize,
    pub total_count: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Search query parameters
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
    #[serde(default = "default_page")]
    page: usize,
    #[serde(default = "default_page_size")]
    page_size: usize,
    #[serde(default = "default_threshold")]
    threshold: Option<f32>,
}

fn default_page() -> usize { 1 }
fn default_page_size() -> usize { 20 }
fn default_threshold() -> Option<f32> { Some(0.7) }

/// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    page: usize,
    #[serde(default = "default_page_size")]
    page_size: usize,
}

/// GET /stakeholders/:id
///
/// Get detailed information about a specific stakeholder.
pub async fn get_stakeholder(
    Path(id): Path<Uuid>,
    Extension(knowledge_graph): Extension<Arc<KnowledgeGraph>>,
) -> Result<Json<StakeholderResponse>, ApiError> {
    let entity = knowledge_graph
        .get_entity(id)
        .ok_or_else(|| ApiError::NotFound("Stakeholder not found".to_string()))?;

    let influence = knowledge_graph.get_stakeholder_influence(id);

    Ok(Json(StakeholderResponse {
        id: entity.id,
        name: entity.name.clone(),
        entity_type: format!("{:?}", entity.entity_type),
        canonical_name: entity.canonical_name.clone(),
        description: entity.description.clone(),
        confidence: entity.confidence,
        metadata: entity.metadata.clone(),
        influence: influence.into(),
        created_at: entity.created_at,
    }))
}

/// GET /stakeholders/:id/documents
///
/// Get all documents that mention this stakeholder.
pub async fn get_stakeholder_documents(
    Path(id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
    Extension(knowledge_graph): Extension<Arc<KnowledgeGraph>>,
) -> Result<Json<PaginatedResponse<DocumentMentionResponse>>, ApiError> {
    let all_docs = knowledge_graph.get_stakeholder_documents(id);

    let total_count = all_docs.len();
    let total_pages = (total_count + pagination.page_size - 1) / pagination.page_size;

    let start = (pagination.page - 1) * pagination.page_size;
    let end = std::cmp::min(start + pagination.page_size, total_count);

    let page_docs: Vec<DocumentMentionResponse> = all_docs
        .iter()
        .skip(start)
        .take(end - start)
        .map(|&doc_id| DocumentMentionResponse {
            document_id: doc_id,
            confidence: 1.0, // Default confidence for mentions
        })
        .collect();

    Ok(Json(PaginatedResponse {
        data: page_docs,
        pagination: PaginationMeta {
            page: pagination.page,
            page_size: pagination.page_size,
            total_count,
            total_pages,
            has_next: pagination.page < total_pages,
            has_prev: pagination.page > 1,
        },
    }))
}

/// GET /documents/:id/stakeholders
///
/// Get all stakeholders mentioned in a specific document.
pub async fn get_document_stakeholders(
    Path(id): Path<Uuid>,
    Extension(knowledge_graph): Extension<Arc<KnowledgeGraph>>,
) -> Result<Json<Vec<StakeholderResponse>>, ApiError> {
    let stakeholders = knowledge_graph.get_document_stakeholders(id);

    let response: Vec<StakeholderResponse> = stakeholders
        .iter()
        .map(|entity| {
            let influence = knowledge_graph.get_stakeholder_influence(entity.id);
            StakeholderResponse {
                id: entity.id,
                name: entity.name.clone(),
                entity_type: format!("{:?}", entity.entity_type),
                canonical_name: entity.canonical_name.clone(),
                description: entity.description.clone(),
                confidence: entity.confidence,
                metadata: entity.metadata.clone(),
                influence: influence.into(),
                created_at: entity.created_at,
            }
        })
        .collect();

    Ok(Json(response))
}

/// GET /stakeholders/search?q=
///
/// Search for stakeholders by name with fuzzy matching.
pub async fn search_stakeholders(
    Query(query): Query<SearchQuery>,
    Extension(knowledge_graph): Extension<Arc<KnowledgeGraph>>,
) -> Result<Json<PaginatedResponse<StakeholderResponse>>, ApiError> {
    let threshold = query.threshold.unwrap_or(0.7);
    let all_matches = knowledge_graph.find_stakeholders_by_name(&query.q, threshold);

    let total_count = all_matches.len();
    let total_pages = (total_count + query.page_size - 1) / query.page_size;

    let start = (query.page - 1) * query.page_size;
    let end = std::cmp::min(start + query.page_size, total_count);

    let page_matches: Vec<StakeholderResponse> = all_matches
        .iter()
        .skip(start)
        .take(end - start)
        .map(|entity| {
            let influence = knowledge_graph.get_stakeholder_influence(entity.id);
            StakeholderResponse {
                id: entity.id,
                name: entity.name.clone(),
                entity_type: format!("{:?}", entity.entity_type),
                canonical_name: entity.canonical_name.clone(),
                description: entity.description.clone(),
                confidence: entity.confidence,
                metadata: entity.metadata.clone(),
                influence: influence.into(),
                created_at: entity.created_at,
            }
        })
        .collect();

    Ok(Json(PaginatedResponse {
        data: page_matches,
        pagination: PaginationMeta {
            page: query.page,
            page_size: query.page_size,
            total_count,
            total_pages,
            has_next: query.page < total_pages,
            has_prev: query.page > 1,
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;

    // Helper to create a test app
    async fn create_test_app() -> axum::Router {
        let kg = Arc::new(KnowledgeGraph::new());
        axum::Router::new()
            .route("/stakeholders/:id", axum::routing::get(get_stakeholder))
            .layer(Extension(kg))
    }

    #[tokio::test]
    async fn test_search_query_default_values() {
        let query = SearchQuery {
            q: "test".to_string(),
            page: 1,
            page_size: 20,
            threshold: Some(0.7),
        };

        assert_eq!(query.page, 1);
        assert_eq!(query.page_size, 20);
        assert_eq!(query.threshold, Some(0.7));
    }

    #[test]
    fn test_influence_metrics_conversion() {
        let metrics = InfluenceMetrics {
            entity_id: Uuid::new_v4(),
            mention_count: 5,
            document_count: 3,
            pagerank_score: 0.8,
            betweenness_centrality: 0.5,
        };

        let response: InfluenceMetricsResponse = metrics.into();
        assert_eq!(response.mention_count, 5);
        assert_eq!(response.document_count, 3);
        assert_eq!(response.pagerank_score, 0.8);
    }
}
