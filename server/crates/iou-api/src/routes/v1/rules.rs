//! Business rules evaluation endpoints

use iou_core::audit::{AuditAction, AuditEntry};
use iou_core::tenancy::TenantContext;
use iou_regels::DmnEvaluator;
use axum::{
    extract::{Extension, Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// List available rules for tenant
pub async fn list_rules(
    Extension(tenant): Extension<TenantContext>,
    Extension(audit): Extension<iou_core::audit::SharedAuditLogger>,
    State(evaluator): State<Arc<DmnEvaluator>>,
) -> Result<Json<Vec<RuleInfo>>, crate::error::ApiError> {
    // Write audit first
    let audit_entry = AuditEntry::new(
        tenant.tenant_id.as_str().to_string(),
        tenant.holder_did.clone(),
        AuditAction::Custom("rules_listed".to_string()),
        "rules".to_string(),
        "list".to_string(),
    );
    let _ = iou_core::audit::log_shared(&audit, &audit_entry).await;

    // Discover DMN rules from Open Regels
    let decisions = evaluator
        .discover_dmn_decisions(None)
        .await
        .map_err(|e| crate::error::ApiError::Internal(anyhow::anyhow!("Failed to discover rules: {}", e)))?;

    let rules: Vec<RuleInfo> = decisions.into_iter().map(|r| RuleInfo {
        id: r.uri.clone(),
        name: r.label.unwrap_or(r.uri.split('/').last().unwrap_or("unknown").to_string()),
        description: r.beschrijving,
        dmn_version: "1.4".to_string(),
    }).collect();

    Ok(Json(rules))
}

#[derive(Debug, Serialize)]
pub struct RuleInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub dmn_version: String,
}

/// Rule evaluation request
#[derive(Debug, Deserialize)]
pub struct RuleEvaluationRequest {
    pub inputs: HashMap<String, serde_json::Value>,
}

/// Rule evaluation response
#[derive(Debug, Serialize)]
pub struct RuleEvaluationResponse {
    pub rule_id: String,
    pub outputs: HashMap<String, serde_json::Value>,
    pub matched: bool,
    pub matched_rule_id: Option<String>,
    pub evaluation_time_us: u64,
}

/// Evaluate a business rule
pub async fn evaluate_rule(
    Extension(tenant): Extension<TenantContext>,
    Extension(audit): Extension<iou_core::audit::SharedAuditLogger>,
    State(evaluator): State<Arc<DmnEvaluator>>,
    Path(rule_id): Path<String>,
    Json(req): Json<RuleEvaluationRequest>,
) -> Result<Json<RuleEvaluationResponse>, crate::error::ApiError> {
    // Write audit first
    let audit_entry = AuditEntry::new(
        tenant.tenant_id.as_str().to_string(),
        tenant.holder_did.clone(),
        AuditAction::RuleEvaluated,
        "rule".to_string(),
        rule_id.clone(),
    )
    .with_context(serde_json::json!({
        "inputs": req.inputs,
    }));
    iou_core::audit::log_shared(&audit, &audit_entry)
        .await
        .map_err(|e| crate::error::ApiError::Internal(anyhow::anyhow!("Audit failed: {}", e)))?;

    // Build decision context
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
            _ => return Err(crate::error::ApiError::Validation("Unsupported input type".into())),
        };
        context_inputs.insert(k, value);
    }

    let context = iou_regels::DecisionContext {
        inputs: context_inputs,
        tenant_id: Some(tenant.tenant_id.as_str().to_string()),
        context: HashMap::new(),
    };

    // Evaluate rule
    let result = evaluator
        .evaluate(&rule_id, &context)
        .map_err(|e| crate::error::ApiError::Internal(anyhow::anyhow!("Evaluation failed: {}", e)))?;

    // Convert outputs to JSON
    let mut outputs = HashMap::new();
    for (k, v) in result.outputs {
        let json_value = match v {
            iou_regels::DecisionValue::String(s) => serde_json::json!(s),
            iou_regels::DecisionValue::Integer(i) => serde_json::json!(i),
            iou_regels::DecisionValue::Double(f) => serde_json::json!(f),
            iou_regels::DecisionValue::Boolean(b) => serde_json::json!(b),
            _ => serde_json::Value::Null,
        };
        outputs.insert(k, json_value);
    }

    Ok(Json(RuleEvaluationResponse {
        rule_id,
        outputs,
        matched: result.matched,
        matched_rule_id: result.metadata.matched_rule,
        evaluation_time_us: result.metadata.evaluation_time_us,
    }))
}

/// Get Open Regels rule details
pub async fn get_open_regels_rule(
    Extension(tenant): Extension<TenantContext>,
    Extension(audit): Extension<iou_core::audit::SharedAuditLogger>,
    State(client): State<Arc<iou_regels::OpenRegelsClient>>,
    Path(rule_uri): Path<String>,
) -> Result<Json<RegelDetail>, crate::error::ApiError> {
    // Write audit
    let audit_entry = AuditEntry::new(
        tenant.tenant_id.as_str().to_string(),
        tenant.holder_did.clone(),
        AuditAction::Custom("rule_fetched".to_string()),
        "regel".to_string(),
        rule_uri.clone(),
    );
    let _ = iou_core::audit::log_shared(&audit, &audit_entry).await;

    // Fetch from Open Regels
    let json_ld = client.select(&format!(
        "SELECT * WHERE {{ <{}> ?p ?o }} LIMIT 100",
        rule_uri
    ))
    .await
    .map_err(|e| crate::error::ApiError::Internal(anyhow::anyhow!("Failed to fetch rule: {}", e)))?;

    // Convert to JSON value
    let bindings: Vec<serde_json::Value> = json_ld.into_iter().map(|mut b| {
        let mut obj = serde_json::Map::new();
        for (k, v) in b {
            obj.insert(k, serde_json::json!({
                "value": v.value,
                "type": v.kind
            }));
        }
        serde_json::Value::Object(obj)
    }).collect();

    Ok(Json(RegelDetail {
        uri: rule_uri.clone(),
        json_ld: serde_json::json!(bindings),
        fetched_at: chrono::Utc::now(),
    }))
}

#[derive(Debug, Serialize)]
pub struct RegelDetail {
    pub uri: String,
    pub json_ld: serde_json::Value,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}
