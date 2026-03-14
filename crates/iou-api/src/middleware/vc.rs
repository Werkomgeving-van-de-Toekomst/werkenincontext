//! Verifiable Credential authentication middleware

use crate::error::ApiError;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use iou_core::ssi::{PresentationValidator, VerifiablePresentation};
use iou_core::tenancy::TenantContext;
use std::sync::Arc;

/// VC validation middleware state
pub struct VcState {
    pub validator: Arc<PresentationValidator>,
    pub audit_logger: iou_core::audit::SharedAuditLogger,
}

/// VC validation middleware
pub async fn vc_middleware(
    State(state): State<VcState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract VP from Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

    if !auth_header.starts_with("VC ") {
        return Err(ApiError::Unauthorized(
            "Invalid authorization format (expected 'VC <presentation>')".to_string()
        ));
    }

    let vp_data = &auth_header[3..];

    // Parse and validate VP
    let vp: VerifiablePresentation =
        serde_json::from_str(vp_data)
            .map_err(|e| ApiError::Unauthorized(format!("Invalid VP format: {}", e)))?;

    let validated = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(state.validator.validate(&vp))
    })
        .map_err(|e| ApiError::Unauthorized(format!("VP validation failed: {}", e)))?;

    // Convert VC claims to TenantContext
    let tenant_context = TenantContext::from_vc_claims(&validated.claims)
        .map_err(|e| ApiError::Unauthorized(format!("Invalid claims: {}", e)))?;

    // Log VC presentation
    let audit_entry = iou_core::audit::AuditEntry::new(
        tenant_context.tenant_id.as_str().to_string(),
        tenant_context.holder_did.clone(),
        iou_core::audit::AuditAction::VCPresented,
        "authentication",
        "vc_presented",
    );
    let _ = iou_core::audit::log_shared(&state.audit_logger, &audit_entry).await;

    // Store validated claims in request extensions
    request.extensions_mut().insert(validated);
    request.extensions_mut().insert(tenant_context);
    request.extensions_mut().insert(state.audit_logger.clone());

    Ok(next.run(request).await)
}
