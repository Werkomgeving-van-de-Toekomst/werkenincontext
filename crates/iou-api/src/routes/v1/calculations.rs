//! Calculation endpoints (DMN-based calculations)

use iou_core::audit::{AuditAction, AuditEntry};
use iou_core::tenancy::TenantContext;
use iou_regels::DmnEvaluator;
use axum::{
    extract::{Extension, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Calculation request
#[derive(Debug, Deserialize)]
pub struct CalculationRequest {
    pub calculation_type: String,
    pub inputs: HashMap<String, serde_json::Value>,
}

/// Calculation response
#[derive(Debug, Serialize)]
pub struct CalculationResponse {
    pub calculation_id: Uuid,
    pub result: Option<HashMap<String, serde_json::Value>>,
    pub status_url: String,
}

/// Start a calculation
pub async fn start_calculation(
    Extension(tenant): Extension<TenantContext>,
    Extension(audit): Extension<iou_core::audit::SharedAuditLogger>,
    State(evaluator): State<Arc<DmnEvaluator>>,
    Json(req): Json<CalculationRequest>,
) -> Result<Json<CalculationResponse>, crate::error::ApiError> {
    let calculation_id = Uuid::new_v4();

    // Write audit first
    let audit_entry = AuditEntry::new(
        tenant.tenant_id.as_str().to_string(),
        tenant.holder_did.clone(),
        AuditAction::CalculationStarted,
        "calculation".to_string(),
        calculation_id.to_string(),
    )
    .with_context(serde_json::json!({
        "calculation_type": req.calculation_type,
        "inputs": req.inputs,
    }));
    iou_core::audit::log_shared(&audit, &audit_entry)
        .await
        .map_err(|e| crate::error::ApiError::Internal(anyhow::anyhow!("Audit failed: {}", e)))?;

    // For synchronous calculation, evaluate immediately
    // For async, would return a status URL to poll
    let mut context_inputs = HashMap::new();
    for (k, v) in req.inputs {
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
        context_inputs.insert(k, value);
    }

    let context = iou_regels::DecisionContext {
        inputs: context_inputs,
        tenant_id: Some(tenant.tenant_id.as_str().to_string()),
        context: HashMap::new(),
    };

    let result = evaluator.evaluate(&req.calculation_type, &context);

    let result = match result {
        Ok(r) => {
            // Log completion
            let audit_entry = AuditEntry::new(
                tenant.tenant_id.as_str().to_string(),
                tenant.holder_did.clone(),
                AuditAction::CalculationCompleted,
                "calculation".to_string(),
                calculation_id.to_string(),
            );
            let _ = iou_core::audit::log_shared(&audit, &audit_entry).await;

            let mut outputs = HashMap::new();
            for (k, v) in r.outputs {
                let json_value = match v {
                    iou_regels::DecisionValue::String(s) => serde_json::json!(s),
                    iou_regels::DecisionValue::Integer(i) => serde_json::json!(i),
                    iou_regels::DecisionValue::Double(f) => serde_json::json!(f),
                    iou_regels::DecisionValue::Boolean(b) => serde_json::json!(b),
                    _ => serde_json::Value::Null,
                };
                outputs.insert(k, json_value);
            }
            Some(outputs)
        }
        Err(e) => {
            // Log failure
            let audit_entry = AuditEntry::new(
                tenant.tenant_id.as_str().to_string(),
                tenant.holder_did.clone(),
                AuditAction::Custom("calculation_failed".to_string()),
                "calculation".to_string(),
                calculation_id.to_string(),
            )
            .with_outcome(iou_core::audit::models::AuditOutcome::Failed);
            let _ = iou_core::audit::log_shared(&audit, &audit_entry).await;
            return Err(crate::error::ApiError::Internal(anyhow::anyhow!(
                "Calculation failed: {}",
                e
            )));
        }
    };

    Ok(Json(CalculationResponse {
        calculation_id,
        result,
        status_url: format!("/v1/calculations/{}", calculation_id),
    }))
}
