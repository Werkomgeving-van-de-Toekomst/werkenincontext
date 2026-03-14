//! BPMN process endpoints

use iou_core::audit::{AuditAction, AuditEntry};
use iou_core::tenancy::TenantContext;
use iou_regels::{BpmnProcessEngine, ProcessInstanceState};
use axum::{
    extract::{Extension, State, Path},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Process start request
#[derive(Debug, Deserialize)]
pub struct ProcessRequest {
    pub process_definition_id: String,
    pub variables: HashMap<String, serde_json::Value>,
}

/// Process response
#[derive(Debug, Serialize)]
pub struct ProcessResponse {
    pub process_instance_id: Uuid,
    pub status: String,
    pub status_url: String,
}

/// Process instance details
#[derive(Debug, Serialize)]
pub struct ProcessInstanceResponse {
    pub id: Uuid,
    pub process_definition_id: String,
    pub status: String,
    pub variables: HashMap<String, serde_json::Value>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Start a BPMN process instance
pub async fn start_process(
    Extension(tenant): Extension<TenantContext>,
    Extension(audit): Extension<iou_core::audit::SharedAuditLogger>,
    Extension(engine): State<Arc<BpmnProcessEngine>>,
    Json(req): Json<ProcessRequest>,
) -> Result<Json<ProcessResponse>, crate::error::ApiError> {
    let instance_id = Uuid::new_v4();

    // Write audit first
    let audit_entry = AuditEntry::new(
        tenant.tenant_id.as_str().to_string(),
        tenant.holder_did.clone(),
        AuditAction::ProcessStarted,
        "process",
        instance_id.to_string(),
    )
    .with_context(serde_json::json!({
        "process_definition_id": req.process_definition_id,
        "variables": req.variables,
    }));
    iou_core::audit::log_shared(&audit, &audit_entry).await
        .map_err(|e| crate::error::ApiError::Internal(format!("Audit failed: {}", e)))?;

    // Convert variables
    let mut vars = HashMap::new();
    for (k, v) in req.variables {
        let value = match v {
            serde_json::Value::String(s) => iou_regels::DecisionValue::String(s),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    iou_regels::DecisionValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    iou_regels::DecisionValue::Double(f)
                } else {
                    return Err(crate::error::ApiError::Validation("Invalid number".into()));
                }
            }
            serde_json::Value::Bool(b) => iou_regels::DecisionValue::Boolean(b),
            _ => return Err(crate::error::ApiError::Validation("Unsupported type".into())),
        };
        vars.insert(k, value);
    }

    // Start process
    let instance = engine.start_process(&req.process_definition_id, vars).await
        .map_err(|e| crate::error::ApiError::Internal(format!("Failed to start process: {}", e)))?;

    Ok(Json(ProcessResponse {
        process_instance_id: instance.id,
        status: format!("{:?}", instance.state),
        status_url: format!("/v1/processes/{}", instance.id),
    }))
}

/// Get process instance status
pub async fn get_process(
    Extension(tenant): Extension<TenantContext>,
    Extension(audit): Extension<iou_core::audit::SharedAuditLogger>,
    Path(id): Path<Uuid>,
    State(_engine): State<Arc<BpmnProcessEngine>>,
) -> Result<Json<ProcessInstanceResponse>, crate::error::ApiError> {
    // Write audit
    let audit_entry = AuditEntry::new(
        tenant.tenant_id.as_str().to_string(),
        tenant.holder_did.clone(),
        AuditAction::Custom("process_status".to_string()),
        "process",
        id.to_string(),
    );
    let _ = iou_core::audit::log_shared(&audit, &audit_entry).await;

    // TODO: Query process instance from engine
    // For now, return a mock response
    Ok(Json(ProcessInstanceResponse {
        id,
        process_definition_id: "unknown".to_string(),
        status: "running".to_string(),
        variables: HashMap::new(),
        started_at: chrono::Utc::now(),
        completed_at: None,
    }))
}
