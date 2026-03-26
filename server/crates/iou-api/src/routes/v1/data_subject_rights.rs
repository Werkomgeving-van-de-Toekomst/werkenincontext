//! Data Subject Rights API (AVG/GDPR Compliance)
//!
//! This module implements AVG (Algemene verordening gegevensbescherming) Articles 15, 16, and 17:
//! - Article 15: Right of access (Subject Access Request)
//! - Article 16: Right to rectification
//! - Article 17: Right to erasure ("right to be forgotten")
//!
//! These endpoints are required for legal compliance with Dutch/European data protection regulations.

use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    dsar::{DsarRepository, SarType, SarFormat, ErasureType, SubjectAccessRequestRow, RectificationRequestRow, ErasureRequestRow, PendingDsarResponse},
    error::ApiError,
    middleware::auth::{AuthContext, require_permission, Permission},
    supabase::SupabasePool,
};

// ============================================
// Request/Response Types
// ============================================

/// Subject Access Request (SAR) - AVG Article 15
#[derive(Debug, Deserialize)]
pub struct SubjectAccessRequest {
    /// Request type (default: full)
    #[serde(default = "default_sar_type")]
    pub request_type: SarType,

    /// Optional: Specific fields to include
    pub requested_fields: Option<Vec<String>>,

    /// Response format preference
    #[serde(default = "default_sar_format")]
    pub response_format: SarFormat,
}

fn default_sar_type() -> SarType {
    SarType::Full
}

fn default_sar_format() -> SarFormat {
    SarFormat::Json
}

/// SAR Response
#[derive(Debug, Serialize)]
pub struct SubjectAccessResponse {
    pub request_id: Uuid,
    pub status: SarStatus,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SarStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Expired,
}

/// Data Rectification Request - AVG Article 16
#[derive(Debug, Deserialize)]
pub struct RectificationRequest {
    /// Object ID containing the data to rectify
    pub object_id: Uuid,

    /// Field name to correct
    pub field_name: String,

    /// Current (incorrect) value
    pub old_value: Option<String>,

    /// New (correct) value
    pub new_value: String,

    /// Justification for the change
    pub justification: Option<String>,

    /// Optional supporting document references
    pub supporting_documents: Option<Vec<String>>,
}

/// Rectification Response
#[derive(Debug, Serialize)]
pub struct RectificationResponse {
    pub request_id: Uuid,
    pub status: RectificationStatus,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RectificationStatus {
    Pending,
    UnderReview,
    Approved,
    Rejected,
    Expired,
}

/// Data Erasure Request - AVG Article 17
#[derive(Debug, Deserialize)]
pub struct ErasureRequest {
    /// Object ID containing data to erase
    pub object_id: Uuid,

    /// Type of erasure
    #[serde(default = "default_erasure_type")]
    pub erasure_type: ErasureType,

    /// Legal basis for retention override (if applicable)
    pub legal_basis: Option<String>,

    /// Justification for erasure
    pub justification: Option<String>,
}

fn default_erasure_type() -> ErasureType {
    ErasureType::Anonymization
}

/// Erasure Response
#[derive(Debug, Serialize)]
pub struct ErasureResponse {
    pub request_id: Uuid,
    pub status: ErasureStatus,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ErasureStatus {
    Pending,
    LegalReview,
    ComplianceRequired,
    Approved,
    Rejected,
    Completed,
    Expired,
}

/// Query parameters for listing requests
#[derive(Debug, Deserialize)]
pub struct DsarListParams {
    #[serde(default = "default_list_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
    #[serde(default)]
    pub status: Option<String>,
}

fn default_list_limit() -> i32 {
    50
}

/// DSAR Request List Response
#[derive(Debug, Serialize)]
pub struct DsarListResponse {
    pub sar: Vec<SarSummary>,
    pub rectifications: Vec<RectificationSummary>,
    pub erasures: Vec<ErasureSummary>,
}

#[derive(Debug, Serialize)]
pub struct SarSummary {
    pub id: Uuid,
    pub status: SarStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RectificationSummary {
    pub id: Uuid,
    pub object_id: Uuid,
    pub field_name: String,
    pub status: RectificationStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ErasureSummary {
    pub id: Uuid,
    pub object_id: Uuid,
    pub erasure_type: ErasureType,
    pub status: ErasureStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalRequest {
    pub approved: bool,
    pub notes: Option<String>,
}

// ============================================
// Helper functions
// ============================================

fn parse_sar_status(s: &str) -> SarStatus {
    match s.to_lowercase().as_str() {
        "pending" => SarStatus::Pending,
        "processing" => SarStatus::Processing,
        "completed" => SarStatus::Completed,
        "failed" => SarStatus::Failed,
        "expired" => SarStatus::Expired,
        _ => SarStatus::Pending,
    }
}

fn parse_rectification_status(s: &str) -> RectificationStatus {
    match s.to_lowercase().as_str() {
        "pending" => RectificationStatus::Pending,
        "under_review" => RectificationStatus::UnderReview,
        "approved" => RectificationStatus::Approved,
        "rejected" => RectificationStatus::Rejected,
        "expired" => RectificationStatus::Expired,
        _ => RectificationStatus::Pending,
    }
}

fn parse_erasure_status(s: &str) -> ErasureStatus {
    match s.to_lowercase().as_str() {
        "pending" => ErasureStatus::Pending,
        "legal_review" => ErasureStatus::LegalReview,
        "compliance_required" => ErasureStatus::ComplianceRequired,
        "approved" => ErasureStatus::Approved,
        "rejected" => ErasureStatus::Rejected,
        "completed" => ErasureStatus::Completed,
        "expired" => ErasureStatus::Expired,
        _ => ErasureStatus::Pending,
    }
}

fn parse_erasure_type(s: &str) -> ErasureType {
    match s.to_lowercase().as_str() {
        "anonymization" => ErasureType::Anonymization,
        "deletion" => ErasureType::Deletion,
        "pseudonymization" => ErasureType::Pseudonymization,
        _ => ErasureType::Anonymization,
    }
}

// ============================================
// SAR Handlers (AVG Article 15)
// ============================================

/// POST /api/v1/subject-access-request
/// Create a new Subject Access Request
pub async fn create_sar(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Json(req): Json<SubjectAccessRequest>,
) -> Result<Json<SubjectAccessResponse>, ApiError> {
    let pool = pool.as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable("DSAR functionality requires Supabase connection".to_string()))?;

    let repo = DsarRepository::new(pool.inner().clone());
    let request_id = Uuid::new_v4();
    let now = Utc::now();
    let expires_at = now + Duration::days(30);

    repo.create_sar(
        request_id,
        auth.user_id,
        req.request_type,
        req.requested_fields,
        req.response_format,
        expires_at,
        now,
    ).await?;

    // Log audit trail
    repo.log_audit(
        "sar",
        request_id,
        auth.user_id,
        "create",
        Some(format!("request_type={:?}", req.request_type)),
        None, // IP would come from request extensions
    ).await?;

    Ok(Json(SubjectAccessResponse {
        request_id,
        status: SarStatus::Pending,
        expires_at,
        created_at: now,
        message: "Uw verzoek tot inzage is ontvangen. U ontvangt bericht zodra de verwerking is voltooid.".to_string(),
    }))
}

/// GET /api/v1/subject-access-request/:id
/// Get status of a SAR
pub async fn get_sar(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = pool.as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable("DSAR functionality requires Supabase connection".to_string()))?;

    let repo = DsarRepository::new(pool.inner().clone());
    let sar = repo.get_sar(id).await?
        .ok_or_else(|| ApiError::NotFound("Subject Access Request not found".to_string()))?;

    // Verify ownership
    if sar.requesting_user_id != auth.user_id {
        // Check if user is compliance officer
        require_permission(&auth, Permission::ComplianceAssess)?;
    }

    Ok(Json(serde_json::json!({
        "id": sar.id,
        "status": sar.status,
        "request_type": sar.request_type,
        "response_data": sar.response_data,
        "error_message": sar.error_message,
        "expires_at": sar.expires_at,
        "created_at": sar.created_at,
        "completed_at": sar.completed_at,
    })))
}

/// GET /api/v1/my-data-requests
/// List all data subject rights requests for the current user
pub async fn list_my_dsar(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Query(params): Query<DsarListParams>,
) -> Result<Json<DsarListResponse>, ApiError> {
    let pool = pool.as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable("DSAR functionality requires Supabase connection".to_string()))?;

    let repo = DsarRepository::new(pool.inner().clone());
    let (sar, rectifications, erasures) = (
        repo.list_user_sar(auth.user_id, params.status.as_deref(), params.limit, params.offset).await?,
        repo.list_user_rectifications(auth.user_id, params.status.as_deref(), params.limit, params.offset).await?,
        repo.list_user_erasures(auth.user_id, params.status.as_deref(), params.limit, params.offset).await?,
    );

    Ok(Json(DsarListResponse {
        sar: sar.into_iter().map(|s| SarSummary {
            id: s.id,
            status: parse_sar_status(&s.status),
            created_at: s.created_at,
            expires_at: s.expires_at,
        }).collect(),
        rectifications: rectifications.into_iter().map(|r| RectificationSummary {
            id: r.id,
            object_id: r.object_id,
            field_name: r.field_name,
            status: parse_rectification_status(&r.status),
            created_at: r.created_at,
        }).collect(),
        erasures: erasures.into_iter().map(|e| ErasureSummary {
            id: e.id,
            object_id: e.object_id,
            erasure_type: parse_erasure_type(&e.erasure_type),
            status: parse_erasure_status(&e.status),
            created_at: e.created_at,
        }).collect(),
    }))
}

// ============================================
// Rectification Handlers (AVG Article 16)
// ============================================

/// POST /api/v1/data-rectification
/// Create a new Data Rectification Request
pub async fn create_rectification(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Json(req): Json<RectificationRequest>,
) -> Result<Json<RectificationResponse>, ApiError> {
    let pool = pool.as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable("DSAR functionality requires Supabase connection".to_string()))?;

    let repo = DsarRepository::new(pool.inner().clone());
    let request_id = Uuid::new_v4();
    let now = Utc::now();
    let expires_at = now + Duration::days(30);

    // Verify the object exists
    let _object = repo.get_object(req.object_id).await?
        .ok_or_else(|| ApiError::NotFound("Object not found".to_string()))?;

    repo.create_rectification(
        request_id,
        auth.user_id,
        req.object_id,
        &req.field_name,
        req.old_value,
        &req.new_value,
        req.justification,
        req.supporting_documents,
        expires_at,
        now,
    ).await?;

    // Log audit trail
    repo.log_audit(
        "rectification",
        request_id,
        auth.user_id,
        "create",
        Some(format!("object_id={}, field={}", req.object_id, req.field_name)),
        None,
    ).await?;

    Ok(Json(RectificationResponse {
        request_id,
        status: RectificationStatus::Pending,
        expires_at,
        created_at: now,
        message: "Uw verzoek tot rectificatie is ontvangen en wordt beoordeeld.".to_string(),
    }))
}

/// GET /api/v1/data-rectification/:id
/// Get status of a rectification request
pub async fn get_rectification(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = pool.as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable("DSAR functionality requires Supabase connection".to_string()))?;

    let repo = DsarRepository::new(pool.inner().clone());
    let rect = repo.get_rectification(id).await?
        .ok_or_else(|| ApiError::NotFound("Rectification request not found".to_string()))?;

    // Verify ownership or compliance role
    if rect.requesting_user_id != auth.user_id {
        require_permission(&auth, Permission::ComplianceAssess)?;
    }

    Ok(Json(serde_json::json!({
        "id": rect.id,
        "object_id": rect.object_id,
        "field_name": rect.field_name,
        "old_value": rect.old_value,
        "new_value": rect.new_value,
        "justification": rect.justification,
        "status": rect.status,
        "reviewed_by": rect.reviewed_by,
        "reviewed_at": rect.reviewed_at,
        "created_at": rect.created_at,
        "expires_at": rect.expires_at,
    })))
}

/// PUT /api/v1/data-rectification/:id/approve
/// Approve a rectification request (compliance officer only)
pub async fn approve_rectification(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApprovalRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_permission(&auth, Permission::ComplianceApprove)?;

    let pool = pool.as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable("DSAR functionality requires Supabase connection".to_string()))?;

    let repo = DsarRepository::new(pool.inner().clone());

    let notes_clone = req.notes.clone();
    repo.approve_rectification(id, auth.user_id, req.approved, notes_clone).await?;

    // Log audit trail
    repo.log_audit(
        "rectification",
        id,
        auth.user_id,
        if req.approved { "approve" } else { "reject" },
        req.notes,
        None,
    ).await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "request_id": id,
        "approved": req.approved,
        "reviewed_by": auth.user_id,
        "reviewed_at": Utc::now(),
    })))
}

// ============================================
// Erasure Handlers (AVG Article 17)
// ============================================

/// POST /api/v1/data-erasure
/// Create a new Data Erasure Request ("Right to be Forgotten")
pub async fn create_erasure(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Json(req): Json<ErasureRequest>,
) -> Result<Json<ErasureResponse>, ApiError> {
    let pool = pool.as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable("DSAR functionality requires Supabase connection".to_string()))?;

    let repo = DsarRepository::new(pool.inner().clone());
    let request_id = Uuid::new_v4();
    let now = Utc::now();
    let expires_at = now + Duration::days(30);

    // Verify the object exists
    let object = repo.get_object(req.object_id).await?
        .ok_or_else(|| ApiError::NotFound("Object not found".to_string()))?;

    // Check for legal retention holds
    let retention_check = object.retention_period.is_some();

    repo.create_erasure(
        request_id,
        auth.user_id,
        req.object_id,
        req.erasure_type,
        req.legal_basis,
        req.justification,
        retention_check,
        expires_at,
        now,
    ).await?;

    // Log audit trail
    repo.log_audit(
        "erasure",
        request_id,
        auth.user_id,
        "create",
        Some(format!("object_id={}, type={:?}", req.object_id, req.erasure_type)),
        None,
    ).await?;

    let message = if retention_check {
        "Uw verzoek tot verwijdering is ontvangen. Er wordt gecontroleerd of er wettelijke bewaartermijnen van toepassing zijn.".to_string()
    } else {
        "Uw verzoek tot verwijdering is ontvangen en wordt verwerkt.".to_string()
    };

    Ok(Json(ErasureResponse {
        request_id,
        status: if retention_check { ErasureStatus::LegalReview } else { ErasureStatus::Pending },
        expires_at,
        created_at: now,
        message,
    }))
}

/// GET /api/v1/data-erasure/:id
/// Get status of an erasure request
pub async fn get_erasure(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = pool.as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable("DSAR functionality requires Supabase connection".to_string()))?;

    let repo = DsarRepository::new(pool.inner().clone());
    let erasure = repo.get_erasure(id).await?
        .ok_or_else(|| ApiError::NotFound("Erasure request not found".to_string()))?;

    // Verify ownership or compliance role
    if erasure.requesting_user_id != auth.user_id {
        require_permission(&auth, Permission::ComplianceAssess)?;
    }

    Ok(Json(serde_json::json!({
        "id": erasure.id,
        "object_id": erasure.object_id,
        "erasure_type": erasure.erasure_type,
        "status": erasure.status,
        "retention_check": erasure.retention_check,
        "legal_basis": erasure.legal_basis,
        "reviewed_by": erasure.reviewed_by,
        "reviewed_at": erasure.reviewed_at,
        "completed_at": erasure.completed_at,
        "created_at": erasure.created_at,
        "expires_at": erasure.expires_at,
    })))
}

/// PUT /api/v1/data-erasure/:id/approve
/// Approve an erasure request (compliance officer only)
pub async fn approve_erasure(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApprovalRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_permission(&auth, Permission::ComplianceApprove)?;

    let pool = pool.as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable("DSAR functionality requires Supabase connection".to_string()))?;

    let repo = DsarRepository::new(pool.inner().clone());

    let notes_clone = req.notes.clone();
    let completed = if req.approved {
        // Perform the erasure
        // Note: execute_erasure is called inside approve_erasure
        true
    } else {
        false
    };

    repo.approve_erasure(id, auth.user_id, req.approved, notes_clone, completed).await?;

    // Log audit trail
    repo.log_audit(
        "erasure",
        id,
        auth.user_id,
        if req.approved { "approve" } else { "reject" },
        req.notes,
        None,
    ).await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "request_id": id,
        "approved": req.approved,
        "completed": completed,
        "reviewed_by": auth.user_id,
        "reviewed_at": Utc::now(),
    })))
}

// ============================================
// Admin/Compliance Officer Endpoints
// ============================================

/// GET /api/v1/admin/dsar/pending
/// List all pending DSAR requests (compliance officers only)
pub async fn list_pending_dsar(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_permission(&auth, Permission::ComplianceAssess)?;

    let pool = pool.as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable("DSAR functionality requires Supabase connection".to_string()))?;

    let repo = DsarRepository::new(pool.inner().clone());
    let pending = repo.get_pending_dsar().await?;

    Ok(Json(serde_json::json!({
        "pending_sar": pending.sar,
        "pending_rectifications": pending.rectifications,
        "pending_erasures": pending.erasures,
    })))
}

/// GET /api/v1/subject-access-request/:id/data
/// Get the actual data for a completed SAR (user only)
pub async fn get_sar_data(
    Extension(auth): Extension<AuthContext>,
    Extension(pool): Extension<Option<Arc<SupabasePool>>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = pool.as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable("DSAR functionality requires Supabase connection".to_string()))?;

    let repo = DsarRepository::new(pool.inner().clone());

    // Verify ownership
    let sar = repo.get_sar(id).await?
        .ok_or_else(|| ApiError::NotFound("Subject Access Request not found".to_string()))?;

    if sar.requesting_user_id != auth.user_id {
        return Err(ApiError::Forbidden("You can only access your own data".to_string()));
    }

    // Get user's personal data
    let user_data = repo.get_user_data(auth.user_id).await?;

    Ok(Json(user_data))
}
