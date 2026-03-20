//! Interne endpoints voor Zeebe job workers (`X-Camunda-Worker-Token`).

use axum::{extract::Extension, http::HeaderMap, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::camunda::DuckdbPipelineCheckpointStore;
use crate::db::{Database, DocumentPipelineInputRow};
use crate::error::ApiError;
use crate::orchestrator::types::StatusMessage;
use crate::websockets::types::DocumentStatus;
use iou_ai::{
    AgentPipelineWithConfig, PipelineConfig, PipelineError,
};
use iou_core::document::{DocumentRequest, DomainConfig, Template};
use iou_core::document::AuditEntry;
use iou_core::storage::S3Client;
use iou_core::workflows::WorkflowStatus;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunPipelineJobRequest {
    pub document_id: Uuid,
    pub zeebe_job_key: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunPipelineJobResponse {
    pub requires_human_approval: bool,
    pub compliance_score: f64,
    pub confidence_score: f64,
    pub final_status: String,
    pub document_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeepAgentBridgeRequest {
    pub document_id: Uuid,
    #[serde(default)]
    pub prompt_hint: Option<String>,
    /// Zeebe job key voor idempotente afronding (Camunda-retries).
    #[serde(default)]
    pub zeebe_job_key: Option<i64>,
}

fn verify_worker_token(headers: &HeaderMap) -> Result<(), ApiError> {
    let expected = std::env::var("CAMUNDA_WORKER_TOKEN").unwrap_or_default();
    if expected.is_empty() {
        return Err(ApiError::Internal(anyhow::anyhow!(
            "CAMUNDA_WORKER_TOKEN is niet gezet"
        )));
    }
    let got = headers
        .get("x-camunda-worker-token")
        .and_then(|v| v.to_str().ok());
    if got != Some(expected.as_str()) {
        return Err(ApiError::Unauthorized(
            "ongeldige Camunda worker token".to_string(),
        ));
    }
    Ok(())
}

fn workflow_status_api_string(s: WorkflowStatus) -> String {
    match s {
        WorkflowStatus::Draft => "draft",
        WorkflowStatus::Submitted => "submitted",
        WorkflowStatus::InReview => "in_review",
        WorkflowStatus::ChangesRequested => "changes_requested",
        WorkflowStatus::Approved => "approved",
        WorkflowStatus::Published => "published",
        WorkflowStatus::Rejected => "rejected",
        WorkflowStatus::Archived => "archived",
    }
    .to_string()
}

/// POST /api/internal/camunda/run-pipeline
pub async fn run_pipeline_job(
    headers: HeaderMap,
    Extension(db): Extension<Arc<Database>>,
    Extension(kg): Extension<Arc<iou_ai::graphrag::KnowledgeGraph>>,
    Extension(status_tx): Extension<broadcast::Sender<StatusMessage>>,
    Extension(doc_status_tx): Extension<broadcast::Sender<DocumentStatus>>,
    Extension(s3): Extension<Arc<S3Client>>,
    Json(req): Json<RunPipelineJobRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    verify_worker_token(&headers)?;

    let db = db.clone();
    let job_key = req.zeebe_job_key;
    let document_id = req.document_id;

    let cached: Option<serde_json::Value> = tokio::task::spawn_blocking({
        let db = db.clone();
        move || db.get_camunda_job_result_json(job_key)
    })
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?
    .map_err(ApiError::Internal)?;

    if let Some(v) = cached {
        return Ok(Json(v));
    }

    let row: Option<DocumentPipelineInputRow> = tokio::task::spawn_blocking({
        let db = db.clone();
        move || db.get_document_pipeline_input(document_id)
    })
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?
    .map_err(ApiError::Internal)?;

    let row = row.ok_or_else(|| {
        ApiError::NotFound(format!("document_pipeline_inputs ontbreekt voor {document_id}"))
    })?;

    let template: Option<Template> = tokio::task::spawn_blocking({
        let db = db.clone();
        let domain_id = row.domain_id.clone();
        let document_type = row.document_type.clone();
        move || db.get_active_template(&domain_id, &document_type)
    })
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?
    .map_err(ApiError::Internal)?;

    let template = template.ok_or_else(|| {
        ApiError::Validation(format!(
            "Geen actieve template voor domain {} type {}",
            row.domain_id, row.document_type
        ))
    })?;

    let engine = iou_ai::TemplateEngine::new().map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;
    engine
        .register_template(&template.id, &template.content)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;

    let doc_request = DocumentRequest {
        id: document_id,
        domain_id: row.domain_id.clone(),
        document_type: row.document_type.clone(),
        context: row.context.clone(),
        requested_at: Utc::now(),
    };

    let domain_config = Arc::new(DomainConfig {
        domain_id: row.domain_id.clone(),
        trust_level: iou_core::document::TrustLevel::Medium,
        required_approval_threshold: 0.8,
        auto_approval_threshold: 0.95,
    });

    let checkpoint_store = Arc::new(DuckdbPipelineCheckpointStore(db.clone()));
    let mut pipeline_config = PipelineConfig::default();
    pipeline_config.enable_checkpoints = true;

    let pipeline = AgentPipelineWithConfig {
        kg_client: kg.clone(),
        template_engine: Arc::new(engine),
        domain_config,
        config: pipeline_config,
        stakeholder_extractor: None,
        checkpoint_store: Some(checkpoint_store),
    };

    let ts = chrono::Utc::now().timestamp();
    let _ = status_tx.send(StatusMessage::Started {
        document_id,
        agent: "camunda_pipeline".to_string(),
        timestamp: ts,
    });
    let _ = doc_status_tx.send(
        StatusMessage::Started {
            document_id,
            agent: "camunda_pipeline".to_string(),
            timestamp: ts,
        }
        .to_document_status(),
    );

    let pipeline_result = pipeline
        .execute_document_pipeline(&doc_request, &template)
        .await
        .map_err(|e: PipelineError| ApiError::Internal(anyhow::anyhow!(e.to_string())))?;

    finalize_after_pipeline(
        &db,
        &s3,
        document_id,
        &pipeline_result.agent_results,
        pipeline_result.final_status,
        pipeline_result.compliance_score,
        pipeline_result.confidence_score,
        pipeline_result.requires_human_approval,
    )
    .await?;

    let out = RunPipelineJobResponse {
        requires_human_approval: pipeline_result.requires_human_approval,
        compliance_score: pipeline_result.compliance_score as f64,
        confidence_score: pipeline_result.confidence_score as f64,
        final_status: workflow_status_api_string(pipeline_result.final_status),
        document_id: document_id.to_string(),
    };

    let out_value = serde_json::to_value(&out).map_err(ApiError::Internal)?;

    let inserted = tokio::task::spawn_blocking({
        let db = db.clone();
        let out_value = out_value.clone();
        move || {
            db.try_record_camunda_job_completion(
                job_key,
                document_id,
                "iou-run-pipeline",
                &out_value,
            )
        }
    })
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?
    .map_err(ApiError::Internal)?;

    if !inserted {
        let v = tokio::task::spawn_blocking(move || db.get_camunda_job_result_json(job_key))
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?
            .map_err(ApiError::Internal)?
            .unwrap_or(out_value);
        return Ok(Json(v));
    }

    let ts = chrono::Utc::now().timestamp();
    let _ = status_tx.send(StatusMessage::Progress {
        document_id,
        agent: "camunda_pipeline".to_string(),
        percent: 100,
        timestamp: ts,
    });
    let _ = doc_status_tx.send(
        StatusMessage::Progress {
            document_id,
            agent: "camunda_pipeline".to_string(),
            percent: 100,
            timestamp: ts,
        }
        .to_document_status(),
    );

    Ok(Json(out_value))
}

async fn finalize_after_pipeline(
    db: &Arc<Database>,
    s3: &Arc<S3Client>,
    document_id: Uuid,
    agent_results: &[iou_ai::AgentExecutionResult],
    final_status: WorkflowStatus,
    compliance_score: f32,
    confidence_score: f32,
    requires_human_approval: bool,
) -> Result<(), ApiError> {
    let summary = serde_json::json!({
        "agent_results": agent_results,
        "requires_human_approval": requires_human_approval,
    });
    let key = format!("documents/{}/v1.pipeline.json", document_id);
    s3.upload(
        &key,
        serde_json::to_vec(&summary).map_err(ApiError::Internal)?,
        "application/json",
    )
    .await?;

    let mut doc = db
        .get_document_async(document_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(document_id.to_string()))?;

    doc.state = final_status;
    doc.compliance_score = compliance_score;
    doc.confidence_score = confidence_score;
    doc.current_version_key = key.clone();
    doc.updated_at = Utc::now();
    db.update_document_async(doc).await?;

    let finalize_audit = AuditEntry::new(
        document_id,
        "CamundaPipeline".to_string(),
        "pipeline_finalize".to_string(),
        serde_json::json!({
            "s3_key": key,
            "final_status": workflow_status_api_string(final_status),
            "compliance_score": compliance_score,
            "confidence_score": confidence_score,
            "requires_human_approval": requires_human_approval,
        }),
    );
    db.add_audit_entry_async(finalize_audit).await?;

    for r in agent_results {
        let entry = AuditEntry {
            id: Uuid::new_v4(),
            document_id,
            agent_name: r.agent_name.clone(),
            action: if r.success { "completed".into() } else { "failed".into() },
            details: serde_json::json!({
                "data": r.data,
                "errors": r.errors,
                "execution_time_ms": r.execution_time_ms,
            }),
            timestamp: r.completed_at,
            execution_time_ms: Some(r.execution_time_ms as i32),
        };
        db.add_audit_entry_async(entry).await?;
    }

    Ok(())
}

/// POST /api/internal/camunda/deep-agent — HTTP-bridge naar de Python Deep Agents service.
pub async fn deep_agent_bridge(
    headers: HeaderMap,
    Extension(db): Extension<Arc<Database>>,
    Json(req): Json<DeepAgentBridgeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    verify_worker_token(&headers)?;

    let document_id = req.document_id;
    let zeebe_job_key = req.zeebe_job_key;
    let db = db.clone();
    if let Some(job_key) = zeebe_job_key {
        let cached: Option<serde_json::Value> = tokio::task::spawn_blocking({
            let db = db.clone();
            move || db.get_camunda_job_result_json(job_key)
        })
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?
        .map_err(ApiError::Internal)?;
        if let Some(v) = cached {
            return Ok(Json(v));
        }
    }

    let base = std::env::var("DEEP_AGENT_SERVICE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8091".to_string());

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(ApiError::Internal)?;

    let token = std::env::var("CAMUNDA_WORKER_TOKEN").unwrap_or_default();
    let url = format!("{}/internal/run", base.trim_end_matches('/'));

    let py_body = serde_json::json!({
        "documentId": document_id,
        "promptHint": req.prompt_hint,
    });

    let resp = client
        .post(url)
        .header("X-Camunda-Worker-Token", token)
        .json(&py_body)
        .send()
        .await
        .map_err(ApiError::Internal)?;

    if !resp.status().is_success() {
        let txt = resp.text().await.unwrap_or_default();
        return Err(ApiError::Internal(anyhow::anyhow!(
            "deep-agent service: {}",
            txt
        )));
    }

    let v: serde_json::Value = resp.json().await.map_err(ApiError::Internal)?;

    if let Some(job_key) = zeebe_job_key {
        let inserted = tokio::task::spawn_blocking({
            let db = db.clone();
            let v = v.clone();
            move || {
                db.try_record_camunda_job_completion(
                    job_key,
                    document_id,
                    "iou-deep-agent",
                    &v,
                )
            }
        })
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?
        .map_err(ApiError::Internal)?;
        if !inserted {
            let v = tokio::task::spawn_blocking(move || db.get_camunda_job_result_json(job_key))
                .await
                .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?
                .map_err(ApiError::Internal)?
                .unwrap_or(v);
            return Ok(Json(v));
        }
    }

    Ok(Json(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_status_maps_lowercase() {
        assert_eq!(
            workflow_status_api_string(WorkflowStatus::InReview),
            "in_review"
        );
    }
}
