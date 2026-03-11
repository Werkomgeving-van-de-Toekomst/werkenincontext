//! Document creation and management API routes
//!
//! This module provides REST endpoints for:
//! - Creating new documents via the agent pipeline
//! - Querying document status
//! - Approving/rejecting documents
//! - Accessing audit trails
//! - Downloading generated documents

use axum::{
    extract::{Extension, Path, Query},
    response::{IntoResponse, Json},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::Database;
use crate::error::ApiError;
use crate::middleware::auth::AuthContext;
use iou_core::document::{DocumentMetadata, AuditEntry, DocumentFormat};
use iou_core::workflows::WorkflowStatus;

// ============================================
// Request/Response Types
// ============================================

/// Request payload for document creation
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateDocumentRequest {
    pub domain_id: String,
    pub document_type: String,
    pub context: HashMap<String, String>,
}

/// Response for successful document creation
#[derive(Debug, Serialize)]
pub struct CreateDocumentResponse {
    pub document_id: Uuid,
    pub state: String,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Document status response
#[derive(Debug, Serialize)]
pub struct DocumentStatusResponse {
    pub document_id: Uuid,
    pub state: String,
    pub current_agent: Option<String>,
    pub compliance_score: f32,
    pub confidence_score: f32,
    pub requires_approval: bool,
    pub errors: Vec<String>,
}

/// Approval request
#[derive(Debug, Deserialize, Serialize)]
pub struct ApprovalRequest {
    pub approved: bool,
    pub comments: Option<String>,
}

/// Approval response
#[derive(Debug, Serialize)]
pub struct ApprovalResponse {
    pub document_id: Uuid,
    pub state: String,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<String>,
}

/// Audit trail response
#[derive(Debug, Serialize)]
pub struct AuditTrailResponse {
    pub document_id: Uuid,
    pub audit_trail: Vec<AuditEntryDto>,
}

/// Audit entry DTO
#[derive(Debug, Serialize)]
pub struct AuditEntryDto {
    pub agent: String,
    pub action: String,
    pub timestamp: DateTime<Utc>,
    pub details: serde_json::Value,
}

/// Download parameters
#[derive(Debug, Deserialize)]
pub struct DownloadParams {
    pub format: Option<DocumentFormat>,
}

// ============================================
// Route Handlers
// ============================================

/// POST /api/documents/create
pub async fn create_document(
    Extension(db): Extension<Arc<Database>>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<CreateDocumentResponse>, ApiError> {
    // Validate template exists
    let _template = db
        .get_active_template_async(req.domain_id.clone(), req.document_type.clone())
        .await?
        .ok_or_else(|| ApiError::Validation(format!(
            "No template found for type '{}' in domain '{}'",
            req.document_type, req.domain_id
        )))?;

    let document_id = Uuid::new_v4();
    let now = Utc::now();

    // Create document record
    let document = DocumentMetadata {
        id: document_id,
        domain_id: req.domain_id.clone(),
        document_type: req.document_type.clone(),
        state: WorkflowStatus::Draft,
        current_version_key: String::new(),
        previous_version_key: None,
        compliance_score: 0.0,
        confidence_score: 0.0,
        created_at: now,
        updated_at: now,
    };

    db.create_document_async(document).await?;

    tracing::info!(
        domain_id = %req.domain_id,
        document_type = %req.document_type,
        document_id = %document_id,
        "Document creation requested"
    );

    // TODO: Invoke pipeline executor asynchronously

    Ok(Json(CreateDocumentResponse {
        document_id,
        state: "draft".to_string(),
        estimated_completion: None,
    }))
}

/// GET /api/documents/{id}/status
pub async fn get_status(
    Extension(db): Extension<Arc<Database>>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentStatusResponse>, ApiError> {
    let document = db
        .get_document_async(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("No document with ID {}", id)))?;

    // Determine if approval is required based on domain trust level
    // TODO: Load domain config and check trust level
    let requires_approval = match document.state {
        WorkflowStatus::Submitted | WorkflowStatus::InReview => true,
        _ => false,
    };

    Ok(Json(DocumentStatusResponse {
        document_id: id,
        state: format!("{:?}", document.state).to_lowercase(),
        current_agent: None,
        compliance_score: document.compliance_score,
        confidence_score: document.confidence_score,
        requires_approval,
        errors: vec![],
    }))
}

/// POST /api/documents/{id}/approve
pub async fn approve_document(
    Extension(db): Extension<Arc<Database>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApprovalRequest>,
) -> Result<Json<ApprovalResponse>, ApiError> {
    // TODO: Check for object_approver role from auth context

    let mut document = db
        .get_document_async(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("No document with ID {}", id)))?;

    // Verify document is in approvable state
    if !matches!(document.state, WorkflowStatus::Submitted | WorkflowStatus::InReview) {
        return Err(ApiError::Validation(format!(
            "Document is in {:?} state, cannot approve",
            document.state
        )));
    }

    // Process approval or rejection
    let new_state = if req.approved {
        WorkflowStatus::Approved
    } else {
        WorkflowStatus::Draft
    };

    document.state = new_state;
    document.updated_at = Utc::now();

    db.update_document_async(document).await?;

    tracing::info!(
        document_id = %id,
        approved = %req.approved,
        "Document approval processed"
    );

    // TODO: If approved, trigger final processing (storage, publication)

    Ok(Json(ApprovalResponse {
        document_id: id,
        state: format!("{:?}", new_state).to_lowercase(),
        approved_at: if req.approved { Some(Utc::now()) } else { None },
        approved_by: None, // TODO: Get from auth context
    }))
}

/// GET /api/documents/{id}/audit
pub async fn get_audit_trail(
    Extension(db): Extension<Arc<Database>>,
    Path(id): Path<Uuid>,
) -> Result<Json<AuditTrailResponse>, ApiError> {
    // Verify document exists
    let _document = db
        .get_document_async(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("No document with ID {}", id)))?;

    // Fetch audit entries ordered by timestamp DESC
    let audit_entries = db.get_audit_trail_async(id).await?;

    let audit_trail: Vec<AuditEntryDto> = audit_entries
        .into_iter()
        .map(|entry| AuditEntryDto {
            agent: entry.agent_name,
            action: entry.action,
            timestamp: entry.timestamp,
            details: entry.details,
        })
        .collect();

    Ok(Json(AuditTrailResponse {
        document_id: id,
        audit_trail,
    }))
}

/// GET /api/documents/{id}/download?format=odf|pdf|md
pub async fn download_document(
    Extension(db): Extension<Arc<Database>>,
    Path(id): Path<Uuid>,
    Query(params): Query<DownloadParams>,
) -> Result<impl IntoResponse, ApiError> {
    let document = db
        .get_document_async(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("No document with ID {}", id)))?;

    // Only published documents can be downloaded
    if document.state != WorkflowStatus::Published {
        return Err(ApiError::Validation(
            "Document must be published before download".to_string(),
        ));
    }

    let format = params.format.unwrap_or(DocumentFormat::Markdown);

    // TODO: Retrieve from S3 storage based on format
    // For now, return a placeholder message
    let content = format!("Document content for {} in {:?} format", id, format);
    let content_type: String = format.content_type().to_string();

    tracing::info!(
        document_id = %id,
        format = ?format,
        "Document downloaded"
    );

    Ok((
        [(axum::http::header::CONTENT_TYPE, content_type)],
        content,
    ))
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_document_request_serialization() {
        let mut context = HashMap::new();
        context.insert("reference".to_string(), "REF-001".to_string());

        let req = CreateDocumentRequest {
            domain_id: "test_domain".to_string(),
            document_type: "woo_besluit".to_string(),
            context,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("REF-001"));
    }

    #[test]
    fn test_approval_request_serialization() {
        let req = ApprovalRequest {
            approved: true,
            comments: Some("Looks good".to_string()),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Looks good"));
    }

    #[test]
    fn test_document_format_content_type() {
        assert_eq!(DocumentFormat::Markdown.content_type(), "text/markdown");
        assert_eq!(
            DocumentFormat::ODF.content_type(),
            "application/vnd.oasis.opendocument.text"
        );
        assert_eq!(DocumentFormat::PDF.content_type(), "application/pdf");
    }
}
