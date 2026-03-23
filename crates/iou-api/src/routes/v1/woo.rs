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
    error::ApiError,
    middleware::auth::{AuthContext, require_permission, Permission},
};

// ============================================
// Request/Response Types
// ============================================

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

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PublicationPlatform {
    Rijksoverheid,
    OverheidNl,
    Gemeente,
    Provincie,
    Waterschap,
    Custom,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Redaction {
    pub field_name: String,
    pub reason: String,
    pub position: Option<(usize, usize)>, // start, end for text redaction
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

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WooRefusalGround {
    Privacy,
    NationalSecurity,
    CommercialConfidence,
    OngoingInvestigation,
    InternationalRelations,
    None,
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

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RequesterType {
    Citizen,
    Organization,
    Journalist,
    Government,
    Other,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WooPriority {
    Low,
    Normal,
    High,
    Urgent,
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

/// Woo Statistics Response
#[derive(Debug, Serialize)]
pub struct WooStatistics {
    pub total_requests: i64,
    pub pending_publication: i64,
    pub published_count: i64,
    pub avg_processing_days: f64,
    pub overdue_requests: i64,
    pub upcoming_deadlines: Vec<WooDeadlineSummary>,
}

#[derive(Debug, Serialize)]
pub struct WooDeadlineSummary {
    pub request_id: Uuid,
    pub reference_number: String,
    pub title: String,
    pub days_until_due: i64,
}

// ============================================
// Woo Publication Handlers
// ============================================

/// POST /api/v1/documents/:id/request-woo-publication
/// Request Woo publication for a document
pub async fn request_woo_publication(
    Extension(auth): Extension<AuthContext>,
    Path(object_id): Path<Uuid>,
    Json(req): Json<WooPublicationRequest>,
) -> Result<Json<WooPublicationResponse>, ApiError> {
    require_permission(&auth, Permission::ObjectClassify)?;

    let publication_id = Uuid::new_v4();
    let now = Utc::now();

    // Create publication request
    // TODO: Implement with Supabase client
    tracing::info!(
        "Creating Woo publication request: id={}, object_id={}, platform={:?}",
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
    Query(params): Query<WooListParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Users can view published publications freely
    // Pending/assessment requires compliance role
    if let Some(status) = &params.status {
        if status != "published" {
            require_permission(&auth, Permission::ComplianceAssess)?;
        }
    }

    // TODO: Implement with Supabase client
    Ok(Json(serde_json::json!({
        "publications": [],
        "total": 0,
        "limit": params.limit,
        "offset": params.offset,
    })))
}

/// GET /api/v1/woo-publications/:id
/// Get details of a Woo publication request
pub async fn get_woo_publication(
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Implement with Supabase client
    // If published, public access
    // If pending/assessment, require compliance role
    Ok(Json(serde_json::json!({
        "id": id,
        "status": "pending",
        "message": "Not yet implemented"
    })))
}

/// POST /api/v1/woo-publications/:id/approve
/// Approve or reject a Woo publication request (compliance officer only)
pub async fn approve_woo_publication(
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<WooApprovalRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_permission(&auth, Permission::WooPublish)?;

    let now = Utc::now();

    // TODO: Implement with Supabase client
    if req.approved {
        // Generate publicatie_nr
        // Update status to 'approved'
        // Set approved_by and approved_at
        Ok(Json(serde_json::json!({
            "id": id,
            "status": "approved",
            "approved_by": auth.user_id,
            "approved_at": now,
            "message": "Woo-publicatie goedgekeurd."
        })))
    } else {
        // Update status to 'rejected'
        // Set refusal_ground
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
    Path(id): Path<Uuid>,
    Json(req): Json<WooPublishRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_permission(&auth, Permission::WooPublish)?;

    let now = Utc::now();

    // TODO: Implement with Supabase client
    // Update status to 'published'
    // Set published_at, publication_url, doi
    // Set woo_publication_date on information_objects

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
    Path(id): Path<Uuid>,
    Json(req): Json<WithdrawalRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_permission(&auth, Permission::WooPublish)?;

    let now = Utc::now();

    // TODO: Implement with Supabase client
    // Update status to 'withdrawn'
    // Log reason

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
    Json(req): Json<WooRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let request_id = Uuid::new_v4();
    let simple_str = Uuid::new_v4().simple().to_string();
    let reference_number = format!("WOO-{}", &simple_str[..8].to_uppercase());
    let now = Utc::now();
    let decision_due_date = now.date_naive() + chrono::Days::new(56); // 8 weeks

    // TODO: Implement with Supabase client
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
    Query(params): Query<WooListParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Woo officers can see all
    require_permission(&auth, Permission::ComplianceAssess)?;

    // TODO: Implement with Supabase client
    Ok(Json(serde_json::json!({
        "requests": [],
        "total": 0,
        "limit": params.limit,
        "offset": params.offset,
    })))
}

/// GET /api/v1/woo-requests/:id
/// Get details of a Woo request
pub async fn get_woo_request(
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_permission(&auth, Permission::ComplianceAssess)?;

    // TODO: Implement with Supabase client
    Ok(Json(serde_json::json!({
        "id": id,
        "status": "received",
        "message": "Not yet implemented"
    })))
}

/// GET /api/v1/woo/statistics
/// Get Woo publication statistics
pub async fn get_woo_statistics(
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<WooStatistics>, ApiError> {
    require_permission(&auth, Permission::ComplianceAssess)?;

    // TODO: Implement with Supabase client
    Ok(Json(WooStatistics {
        total_requests: 0,
        pending_publication: 0,
        published_count: 0,
        avg_processing_days: 0.0,
        overdue_requests: 0,
        upcoming_deadlines: vec![],
    }))
}

/// GET /api/v1/woo/upcoming-deadlines
/// Get upcoming Woo request deadlines
pub async fn get_woo_deadlines(
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Vec<WooDeadlineSummary>>, ApiError> {
    require_permission(&auth, Permission::ComplianceAssess)?;

    // TODO: Implement with Supabase client
    Ok(Json(vec![]))
}

/// GET /api/v1/woo/published-documents
/// Get list of published Woo documents (public endpoint)
pub async fn get_published_woo_documents(
    Query(params): Query<WooListParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Public endpoint - no auth required
    // TODO: Implement with Supabase client
    Ok(Json(serde_json::json!({
        "documents": [],
        "total": 0,
        "limit": params.limit,
        "offset": params.offset,
    })))
}

/// POST /api/v1/woo-publications/:id/consultation-complete
/// Mark stakeholder consultation as complete
pub async fn mark_consultation_complete(
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_permission(&auth, Permission::WooPublish)?;

    // TODO: Implement with Supabase client
    Ok(Json(serde_json::json!({
        "id": id,
        "consultation_completed_at": Utc::now(),
        "message": "Consultatie afgerond."
    })))
}
