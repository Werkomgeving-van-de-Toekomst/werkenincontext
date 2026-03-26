//! Woo Publication API (Wet open overheid Compliance)
//!
//! This module implements the Woo (Wet open overheid) publication workflow,
//! which replaced the Wob (Wet openbaarheid van bestuur) in 2022.
//! Woo requires government information to be proactively published.

use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    dsar::{
        WooRepository, WooPublicationRow, WooRequestRow, WooStatistics, WooDeadlineSummary,
        PublicationPlatform, WooRequestType, RequesterType, WooPriority,
        WooRefusalGround, Redaction,
    },
    error::ApiError,
    middleware::auth::{AuthContext, require_permission, Permission},
    supabase::SupabasePool,
};

/// Woo Publication Request
#[derive(Debug, Deserialize)]
pub struct WooPublicationRequest {
    /// Object ID to publish
    pub object_id: Uuid,

    /// Publication platform
    #[serde(default = "default_platform")]
    pub publication_platform: PublicationPlatform,

    /// Publication categories
    pub category_ids: Option<Vec<Uuid>>,

    /// Legal basis for publication
    pub legal_basis: Option<String>,

    /// Publication summary (description)
    pub publication_summary: Option<String>,

    /// Consultation required for stakeholders
    pub consultation_required: Option<bool>,

    /// Redactions needed (sensitive information)
    pub redactions: Option<Vec<Redaction>>,
}

fn default_platform() -> PublicationPlatform {
    PublicationPlatform::Rijksoverheid
}

/// Woo Publication Response
#[derive(Debug, Serialize)]
pub struct WooPublicationResponse {
    pub id: Uuid,
    pub object_id: Uuid,
    pub publication_status: WooPublicationStatus,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WooPublicationStatus {
    Pending,
    Assessment,
    RedactionRequired,
    Approved,
    Rejected,
    Published,
    Withdrawn,
    Scheduled,
}

/// Woo Publication Approval Request
#[derive(Debug, Deserialize)]
pub struct WooApprovalRequest {
    /// Approve or reject
    pub approved: bool,

    /// Rejection ground (if rejecting)
    pub refusal_ground: Option<WooRefusalGround>,

    /// Required redactions
    pub redactions: Option<Vec<Redaction>>,

    /// Review notes
    pub notes: Option<String>,
}

/// Woo Publication Publish Request
#[derive(Debug, Deserialize)]
pub struct WooPublishRequest {
    /// Publication URL after publishing
    pub publication_url: Option<String>,

    /// DOI (Digital Object Identifier)
    pub doi: Option<String>,

    /// Imposition reference
    pub imposition_reference: Option<String>,
}

/// Woo Request (Active Woo verzoek)
#[derive(Debug, Deserialize)]
pub struct WooRequest {
    /// Requester name
    pub requester_name: String,

    /// Requester email
    pub requester_email: String,

    /// Requester address (optional)
    pub requester_address: Option<String>,

    /// Requester type
    #[serde(default = "default_requester_type")]
    pub requester_type: RequesterType,

    /// Title/subject of request
    pub title: String,

    /// Description of requested information
    pub description: String,

    /// Specific information being requested
    pub requested_information: Option<Vec<String>>,

    /// Priority level
    #[serde(default = "default_priority")]
    pub priority: WooPriority,
}

fn default_requester_type() -> RequesterType {
    RequesterType::Citizen
}

fn default_priority() -> WooPriority {
    WooPriority::Normal
}

/// Query parameters for listing publications
#[derive(Debug, Deserialize)]
pub struct WooListParams {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default = "default_list_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
}

fn default_list_limit() -> i32 {
    50
}

// ============================================
// Woo Publication Handlers
// ============================================

/// POST /api/v1/documents/:id/request-woo-publication
/// Request Woo publication for a document
pub async fn request_woo_publication(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Path(object_id): Path<Uuid>,
    Json(req): Json<WooPublicationRequest>,
) -> Result<Json<WooPublicationResponse>, ApiError> {
    require_permission(&auth, Permission::ObjectClassify)?;

    let pool = pool.as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable("Woo functionality requires Supabase connection".to_string()))?;

    let repo = crate::dsar::WooRepository::new(pool.inner().clone());
    let publication_id = Uuid::new_v4();
    let now = Utc::now();

    repo.create_publication(
        publication_id,
        object_id,
        req.publication_platform,
        req.category_ids,
        req.legal_basis,
        req.publication_summary,
        req.consultation_required.unwrap_or(false),
        req.redactions,
        now,
    ).await?;

    tracing::info!(
        "Created Woo publication request: id={}, object_id={}, platform={:?}",
        publication_id, object_id, req.publication_platform
    );

    Ok(Json(WooPublicationResponse {
        id: publication_id,
        object_id,
        publication_status: WooPublicationStatus::Pending,
        message: "Woo-publicatieverzoek ingediend. Het verzoek wordt beoordeeld.".to_string(),
        created_at: now,
    }))
}

/// GET /api/v1/woo-publications
/// List all Woo publication requests
pub async fn list_woo_publications(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Query(params): Query<WooListParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Users can view published publications freely
    // Pending/assessment requires compliance role
    if let Some(status) = &params.status {
        if status != "published" {
            require_permission(&auth, Permission::ComplianceAssess)?;
        }
    }

    let Some(pool) = pool.as_ref() else {
        return Ok(Json(serde_json::json!({
            "publications": [],
            "total": 0,
            "limit": params.limit,
            "offset": params.offset,
        })));
    };

    let repo = crate::dsar::WooRepository::new(pool.inner().clone());
    let (publications, total) = repo.list_publications(
        params.status.as_deref(),
        params.limit,
        params.offset,
    ).await?;

    Ok(Json(serde_json::json!({
        "publications": publications,
        "total": total,
        "limit": params.limit,
        "offset": params.offset,
    })))
}

/// GET /api/v1/woo-publications/:id
/// Get details of a Woo publication request
pub async fn get_woo_publication(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let Some(pool) = pool.as_ref() else {
        return Err(ApiError::ServiceUnavailable("Woo functionality requires Supabase connection".to_string()));
    };

    let repo = crate::dsar::WooRepository::new(pool.inner().clone());
    let publication = repo.get_publication(id).await?
        .ok_or_else(|| ApiError::NotFound("Woo publication not found".to_string()))?;

    // If published, public access
    // If pending/assessment, require compliance role
    if publication.publication_status != "published" {
        require_permission(&auth, Permission::ComplianceAssess)?;
    }

    Ok(Json(serde_json::json!(publication)))
}

/// POST /api/v1/woo-publications/:id/approve
/// Approve or reject a Woo publication request (compliance officer only)
pub async fn approve_woo_publication(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Path(id): Path<Uuid>,
    Json(req): Json<WooApprovalRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_permission(&auth, Permission::WooPublish)?;

    let Some(pool) = pool.as_ref() else {
        return Err(ApiError::ServiceUnavailable("Woo functionality requires Supabase connection".to_string()));
    };

    let repo = crate::dsar::WooRepository::new(pool.inner().clone());

    let notes = req.notes.clone();
    repo.approve_publication(
        id,
        auth.user_id,
        req.approved,
        req.refusal_ground,
        notes,
    ).await?;

    let now = Utc::now();
    if req.approved {
        Ok(Json(serde_json::json!({
            "id": id,
            "status": "approved",
            "approved_by": auth.user_id,
            "approved_at": now,
            "message": "Woo-publicatie goedgekeurd."
        })))
    } else {
        Ok(Json(serde_json::json!({
            "id": id,
            "status": "rejected",
            "approved_by": auth.user_id,
            "approved_at": now,
            "refusal_ground": req.refusal_ground,
            "message": "Woo-publicatie afgewezen."
        })))
    }
}

/// POST /api/v1/woo-publications/:id/publish
/// Mark a Woo publication as published
pub async fn publish_woo_publication(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Path(id): Path<Uuid>,
    Json(req): Json<WooPublishRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_permission(&auth, Permission::WooPublish)?;

    let Some(pool) = pool.as_ref() else {
        return Err(ApiError::ServiceUnavailable("Woo functionality requires Supabase connection".to_string()));
    };

    let repo = crate::dsar::WooRepository::new(pool.inner().clone());

    repo.publish_publication(
        id,
        req.publication_url.clone(),
        req.doi.clone(),
        req.imposition_reference.clone(),
    ).await?;

    let now = Utc::now();
    Ok(Json(serde_json::json!({
        "id": id,
        "status": "published",
        "published_at": now,
        "publication_url": req.publication_url,
        "doi": req.doi,
        "message": "Woo-publicatie gepubliceerd."
    })))
}

/// POST /api/v1/woo-publications/:id/withdraw
/// Withdraw a previously published Woo document
pub async fn withdraw_woo_publication(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Path(id): Path<Uuid>,
    Json(req): Json<WithdrawalRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_permission(&auth, Permission::WooPublish)?;

    let Some(pool) = pool.as_ref() else {
        return Err(ApiError::ServiceUnavailable("Woo functionality requires Supabase connection".to_string()));
    };

    let repo = crate::dsar::WooRepository::new(pool.inner().clone());

    repo.withdraw_publication(id, req.reason.clone()).await?;

    let now = Utc::now();
    Ok(Json(serde_json::json!({
        "id": id,
        "status": "withdrawn",
        "withdrawn_at": now,
        "reason": req.reason,
        "message": "Woo-publicatie ingetrokken."
    })))
}

#[derive(Debug, Deserialize)]
pub struct WithdrawalRequest {
    pub reason: String,
}

// ============================================
// Woo Request (Active Woo verzoeken) Handlers
// ============================================

/// POST /api/v1/woo-requests
/// Create a new Woo request (citizen/government request)
pub async fn create_woo_request(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Json(req): Json<WooRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let Some(pool) = pool.as_ref() else {
        return Err(ApiError::ServiceUnavailable("Woo functionality requires Supabase connection".to_string()));
    };

    let repo = crate::dsar::WooRepository::new(pool.inner().clone());
    let request_id = Uuid::new_v4();
    let simple_str = Uuid::new_v4().simple().to_string();
    let reference_number = format!("WOO-{}", &simple_str[..8].to_uppercase());
    let now = Utc::now();
    let decision_due_date = now.date_naive() + chrono::Days::new(56); // 8 weeks

    repo.create_request(
        request_id,
        reference_number.clone(),
        req.requester_name.clone(),
        req.requester_email.clone(),
        req.requester_address.clone(),
        req.requester_type,
        WooRequestType::Information,
        req.title.clone(),
        req.description.clone(),
        req.requested_information.clone(),
        req.priority,
        decision_due_date,
        now,
    ).await?;

    Ok(Json(serde_json::json!({
        "id": request_id,
        "reference_number": reference_number,
        "status": "received",
        "decision_due_date": decision_due_date,
        "message": "Uw Woo-verzoek is ontvangen. U ontvangt binnen 5 werkdagen een bevestiging.",
        "created_at": now,
    })))
}

/// GET /api/v1/woo-requests
/// List Woo requests
pub async fn list_woo_requests(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Query(params): Query<WooListParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_permission(&auth, Permission::ComplianceAssess)?;

    let Some(pool) = pool.as_ref() else {
        return Ok(Json(serde_json::json!({
            "requests": [],
            "total": 0,
            "limit": params.limit,
            "offset": params.offset,
        })));
    };

    let repo = crate::dsar::WooRepository::new(pool.inner().clone());
    let (requests, total) = repo.list_requests(
        params.status.as_deref(),
        params.limit,
        params.offset,
    ).await?;

    Ok(Json(serde_json::json!({
        "requests": requests,
        "total": total,
        "limit": params.limit,
        "offset": params.offset,
    })))
}

/// GET /api/v1/woo-requests/:id
/// Get details of a Woo request
pub async fn get_woo_request(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_permission(&auth, Permission::ComplianceAssess)?;

    let Some(pool) = pool.as_ref() else {
        return Err(ApiError::ServiceUnavailable("Woo functionality requires Supabase connection".to_string()));
    };

    let repo = crate::dsar::WooRepository::new(pool.inner().clone());
    let request = repo.get_request(id).await?
        .ok_or_else(|| ApiError::NotFound("Woo request not found".to_string()))?;

    Ok(Json(serde_json::json!(request)))
}

/// GET /api/v1/woo/statistics
/// Get Woo publication statistics
pub async fn get_woo_statistics(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
) -> Result<Json<WooStatistics>, ApiError> {
    require_permission(&auth, Permission::ComplianceAssess)?;

    let Some(pool) = pool.as_ref() else {
        return Ok(Json(WooStatistics {
            total_requests: 0,
            pending_publication: 0,
            published_count: 0,
            avg_processing_days: 0.0,
            overdue_requests: 0,
            upcoming_deadlines: vec![],
        }));
    };

    let repo = crate::dsar::WooRepository::new(pool.inner().clone());
    let stats = repo.get_statistics().await?;

    Ok(Json(stats))
}

/// GET /api/v1/woo/upcoming-deadlines
/// Get upcoming Woo request deadlines
pub async fn get_woo_deadlines(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
) -> Result<Json<Vec<WooDeadlineSummary>>, ApiError> {
    require_permission(&auth, Permission::ComplianceAssess)?;

    let Some(pool) = pool.as_ref() else {
        return Ok(Json(vec![]));
    };

    let repo = crate::dsar::WooRepository::new(pool.inner().clone());
    let deadlines = repo.get_upcoming_deadlines().await?;

    Ok(Json(deadlines))
}

/// GET /api/v1/woo/published-documents
/// Get list of published Woo documents (public endpoint)
pub async fn get_published_woo_documents(
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Query(params): Query<WooListParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Public endpoint - no auth required
    let Some(pool) = pool.as_ref() else {
        return Ok(Json(serde_json::json!({
            "documents": [],
            "total": 0,
            "limit": params.limit,
            "offset": params.offset,
        })));
    };

    let repo = crate::dsar::WooRepository::new(pool.inner().clone());
    let (documents, total) = repo.get_published_documents(
        params.limit,
        params.offset,
    ).await?;

    Ok(Json(serde_json::json!({
        "documents": documents,
        "total": total,
        "limit": params.limit,
        "offset": params.offset,
    })))
}

/// POST /api/v1/woo-publications/:id/consultation-complete
/// Mark stakeholder consultation as complete
pub async fn mark_consultation_complete(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_permission(&auth, Permission::WooPublish)?;

    let Some(pool) = pool.as_ref() else {
        return Err(ApiError::ServiceUnavailable("Woo functionality requires Supabase connection".to_string()));
    };

    let repo = crate::dsar::WooRepository::new(pool.inner().clone());
    repo.mark_consultation_complete(id).await?;

    Ok(Json(serde_json::json!({
        "id": id,
        "consultation_completed_at": Utc::now(),
        "message": "Consultatie afgerond."
    })))
}
