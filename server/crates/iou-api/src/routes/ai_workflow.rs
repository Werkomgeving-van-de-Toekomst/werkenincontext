//! AI Workflow API Routes
//!
//! Endpoints for AI-powered workflow analysis, configuration,
//! and optimization.

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::ai_workflow_service::AiWorkflowService;

/// Analyze workflow performance
///
/// POST /api/workflows/analyze
pub async fn analyze_workflow(
    State(service): State<AiWorkflowService>,
    Json(request): Json<AnalyzeWorkflowRequest>,
) -> impl IntoResponse {
    match service.analyze_workflow(request.workflow_id).await {
        Ok(analysis) => (StatusCode::OK, Json(analysis)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Analyze all workflows in a domain
///
/// POST /api/workflows/analyze-domain
pub async fn analyze_domain(
    State(service): State<AiWorkflowService>,
    Json(request): Json<AnalyzeDomainRequest>,
) -> impl IntoResponse {
    match service.analyze_domain(&request.domain_id).await {
        Ok(analysis) => (StatusCode::OK, Json(analysis)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Generate workflow configuration from description
///
/// POST /api/workflows/generate
pub async fn generate_config(
    State(service): State<AiWorkflowService>,
    Json(request): Json<GenerateConfigRequest>,
) -> impl IntoResponse {
    match service.generate_config(
        request.description,
        request.domain_id,
        request.document_type,
    ).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Suggest improvements to workflow configuration
///
/// POST /api/workflows/suggest-improvements
pub async fn suggest_improvements(
    State(service): State<AiWorkflowService>,
    Json(request): Json<SuggestImprovementsRequest>,
) -> impl IntoResponse {
    match service.suggest_config_improvements(request.workflow_id).await {
        Ok(suggestions) => (StatusCode::OK, Json(suggestions)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get AI suggestion for approval decision
///
/// POST /api/workflows/suggest-decision
pub async fn suggest_decision(
    State(service): State<AiWorkflowService>,
    Json(request): Json<SuggestDecisionRequest>,
) -> impl IntoResponse {
    match service.suggest_decision(request.document_id, request.stage_id).await {
        Ok(suggestion) => (StatusCode::OK, Json(suggestion)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Generate document summary for approval
///
/// GET /api/documents/:id/summary
pub async fn document_summary(
    State(service): State<AiWorkflowService>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match service.generate_summary(id).await {
        Ok(summary) => (StatusCode::OK, Json(DocumentSummaryResponse { summary })).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get workflow optimization report
///
/// POST /api/workflows/optimize
pub async fn optimize_workflow(
    State(service): State<AiWorkflowService>,
    Json(request): Json<OptimizeWorkflowRequest>,
) -> impl IntoResponse {
    match service.optimize_workflow(request.workflow_id).await {
        Ok(report) => (StatusCode::OK, Json(report)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get analytics data for workflow dashboard
///
/// GET /api/workflows/:id/analytics
pub async fn workflow_analytics(
    State(service): State<AiWorkflowService>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match service.analyze_workflow(id).await {
        Ok(analysis) => {
            let dashboard = WorkflowDashboardData {
                workflow_id: analysis.workflow_id,
                total_stages: analysis.total_stages,
                avg_completion_hours: analysis.avg_completion_hours,
                sla_compliance_pct: analysis.sla_compliance_pct,
                bottleneck: analysis.bottleneck,
                stage_insights: analysis.stage_insights,
            };
            (StatusCode::OK, Json(dashboard)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AnalyzeWorkflowRequest {
    pub workflow_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct AnalyzeDomainRequest {
    pub domain_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GenerateConfigRequest {
    pub description: String,
    pub domain_id: String,
    pub document_type: String,
}

#[derive(Debug, Deserialize)]
pub struct SuggestImprovementsRequest {
    pub workflow_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct SuggestDecisionRequest {
    pub document_id: Uuid,
    pub stage_id: String,
}

#[derive(Debug, Deserialize)]
pub struct OptimizeWorkflowRequest {
    pub workflow_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct DocumentSummaryResponse {
    pub summary: String,
}

#[derive(Debug, Serialize)]
pub struct WorkflowDashboardData {
    pub workflow_id: Uuid,
    pub total_stages: usize,
    pub avg_completion_hours: Option<f64>,
    pub sla_compliance_pct: f64,
    pub bottleneck: Option<iou_ai::agents::workflow::BottleneckInfo>,
    pub stage_insights: Vec<iou_ai::agents::workflow::analyzer::StageInsight>,
}
