//! Workflow stage API endpoints
//!
//! Provides endpoints for managing multi-stage approval workflows on documents.

use axum::{
    extract::{Extension, Path, Query},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::db::Database;
use crate::error::ApiError;
use crate::middleware::auth::AuthContext;

/// Summary view of an approval stage
#[derive(Debug, Clone, Serialize)]
pub struct StageView {
    pub stage_id: String,
    pub id: Uuid,
    pub document_id: Uuid,
    pub stage_name: String,
    pub stage_order: i32,
    pub status: String,
    pub approval_type: String,
    pub approvers: Vec<ApproverView>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub deadline: Option<DateTime<Utc>>,
    pub sla_hours: i32,
}

/// Approver information for display
#[derive(Debug, Clone, Serialize)]
pub struct ApproverView {
    pub user_id: Uuid,
    pub user_name: Option<String>,
    pub status: String,
    pub responded_at: Option<chrono::DateTime<chrono::Utc>>,
    pub delegated_from: Option<Uuid>,
}

/// Detailed stage information with all approvals
#[derive(Debug, Clone, Serialize)]
pub struct StageDetailView {
    #[serde(flatten)]
    pub base: StageView,
    pub approvals_received: Vec<ApprovalResponseView>,
    pub escalation_count: i32,
    pub current_escalation_level: Option<i32>,
}

/// Individual approval response for display
#[derive(Debug, Clone, Serialize)]
pub struct ApprovalResponseView {
    pub approver_id: Uuid,
    pub delegated_from: Option<Uuid>,
    pub decision: String,
    pub comment: Option<String>,
    pub responded_at: DateTime<Utc>,
}

/// Request to approve a stage
#[derive(Debug, Deserialize)]
pub struct ApproveStageRequest {
    pub comment: Option<String>,
}

/// Request to reject a stage
#[derive(Debug, Deserialize)]
pub struct RejectStageRequest {
    pub reason: String,
    pub comment: Option<String>,
}

/// Request to delegate approval to another user
#[derive(Debug, Deserialize)]
pub struct DelegateStageRequest {
    pub to_user_id: Uuid,
    pub reason: Option<String>,
}

/// Query parameters for listing stages
#[derive(Debug, Deserialize)]
pub struct ListStagesQuery {
    pub include_completed: Option<bool>,
    pub include_pending: Option<bool>,
}

pub fn workflow_stages_router() -> Router {
    Router::new()
        .route("/api/documents/:id/stages", get(get_stages))
        .route("/api/documents/:id/stages/:stage_id", get(get_stage))
        .route(
            "/api/documents/:id/stages/:stage_id/approve",
            post(approve_stage),
        )
        .route(
            "/api/documents/:id/stages/:stage_id/reject",
            post(reject_stage),
        )
        .route(
            "/api/documents/:id/stages/:stage_id/delegate",
            post(delegate_stage),
        )
}

/// GET /api/documents/:id/stages
///
/// Returns all approval stages for a document, ordered by stage_order.
async fn get_stages(
    Extension(_db): Extension<Arc<Database>>,
    Path(id): Path<Uuid>,
    Extension(_auth): Extension<AuthContext>,
    Query(_params): Query<ListStagesQuery>,
) -> Result<Json<Vec<StageView>>, ApiError> {
    // TODO: Implement stage listing from database
    // For now, return empty list to ensure API compiles
    tracing::warn!("get_stages for document {} - placeholder implementation", id);
    Ok(Json(vec![]))
}

/// GET /api/documents/:id/stages/:stage_id
///
/// Returns detailed information about a specific stage including all approvals.
async fn get_stage(
    Extension(_db): Extension<Arc<Database>>,
    Path((_id, stage_id)): Path<(Uuid, String)>,
    Extension(_auth): Extension<AuthContext>,
) -> Result<Json<StageDetailView>, ApiError> {
    // TODO: Implement stage detail retrieval
    tracing::warn!("get_stage for {} - placeholder implementation", stage_id);
    Err(ApiError::NotFound(format!(
        "Stage {} not found - not yet implemented",
        stage_id
    )))
}

/// POST /api/documents/:id/stages/:stage_id/approve
///
/// Records an approval for the current stage.
async fn approve_stage(
    Extension(_db): Extension<Arc<Database>>,
    Path((_id, stage_id)): Path<(Uuid, String)>,
    Extension(auth): Extension<AuthContext>,
    Json(_req): Json<ApproveStageRequest>,
) -> Result<Json<StageView>, ApiError> {
    // TODO: Implement approval recording via StageExecutor
    tracing::warn!(
        "approve_stage for {} by user {} - placeholder implementation",
        stage_id,
        auth.user_id
    );
    Err(ApiError::Validation(
        "Stage approval not yet implemented".to_string(),
    ))
}

/// POST /api/documents/:id/stages/:stage_id/reject
///
/// Records a rejection for the current stage.
async fn reject_stage(
    Extension(_db): Extension<Arc<Database>>,
    Path((_id, stage_id)): Path<(Uuid, String)>,
    Extension(_auth): Extension<AuthContext>,
    Json(_req): Json<RejectStageRequest>,
) -> Result<Json<StageView>, ApiError> {
    // TODO: Implement rejection recording
    tracing::warn!("reject_stage for {} - placeholder implementation", stage_id);
    Err(ApiError::Validation(
        "Stage rejection not yet implemented".to_string(),
    ))
}

/// POST /api/documents/:id/stages/:stage_id/delegate
///
/// Delegates approval authority to another user.
async fn delegate_stage(
    Extension(_db): Extension<Arc<Database>>,
    Path((_id, stage_id)): Path<(Uuid, String)>,
    Extension(_auth): Extension<AuthContext>,
    Json(_req): Json<DelegateStageRequest>,
) -> Result<Json<StageView>, ApiError> {
    // TODO: Implement delegation via DelegationService
    tracing::warn!("delegate_stage for {} - placeholder implementation", stage_id);
    Err(ApiError::Validation(
        "Stage delegation not yet implemented".to_string(),
    ))
}
