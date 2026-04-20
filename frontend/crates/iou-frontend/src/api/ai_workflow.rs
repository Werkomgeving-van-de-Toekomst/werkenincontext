//! AI Workflow API Client
//!
//! Frontend API client for AI workflow services

use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const AI_WORKFLOW_BASE_URL: &str = "/api/workflows";

/// Analyze workflow performance
pub async fn analyze_workflow(workflow_id: Uuid) -> Result<WorkflowAnalysis, String> {
    let url = format!("{}/analyze", AI_WORKFLOW_BASE_URL);
    let request = AnalyzeWorkflowRequest { workflow_id };

    // In production, use actual fetch
    // let response = fetch(&url, request).await?;
    mock_analyze_workflow(workflow_id).await
}

/// Analyze all workflows in a domain
pub async fn analyze_domain(domain_id: &str) -> Result<DomainAnalysis, String> {
    let url = format!("{}/analyze-domain", AI_WORKFLOW_BASE_URL);
    let request = AnalyzeDomainRequest { domain_id: domain_id.to_string() };

    mock_analyze_domain(domain_id).await
}

/// Generate workflow configuration from description
pub async fn generate_workflow_config(
    description: String,
    domain_id: String,
    document_type: String,
) -> Result<ConfigGenerationResult, String> {
    let url = format!("{}/generate", AI_WORKFLOW_BASE_URL);
    let request = GenerateConfigRequest { description, domain_id, document_type };

    mock_generate_config(description, domain_id, document_type).await
}

/// Get AI approval suggestion
pub async fn get_approval_suggestion(
    document_id: Uuid,
    stage_id: String,
) -> Result<ApprovalSuggestion, String> {
    let url = format!("{}/suggest-decision", AI_WORKFLOW_BASE_URL);
    let request = SuggestDecisionRequest { document_id, stage_id };

    mock_approval_suggestion(document_id, stage_id).await
}

/// Get workflow optimization report
pub async fn optimize_workflow(workflow_id: Uuid) -> Result<OptimizationReport, String> {
    let url = format!("{}/optimize", AI_WORKFLOW_BASE_URL);
    let request = OptimizeWorkflowRequest { workflow_id };

    mock_optimize_workflow(workflow_id).await
}

/// Get workflow analytics for dashboard
pub async fn get_workflow_analytics(workflow_id: Uuid) -> Result<WorkflowDashboardData, String> {
    let url = format!("{}/{}/analytics", AI_WORKFLOW_BASE_URL, workflow_id);

    mock_workflow_analytics(workflow_id).await
}

// ============================================================================
// Request Types
// ============================================================================

#[derive(Debug, Serialize)]
struct AnalyzeWorkflowRequest {
    workflow_id: Uuid,
}

#[derive(Debug, Serialize)]
struct AnalyzeDomainRequest {
    domain_id: String,
}

#[derive(Debug, Serialize)]
struct GenerateConfigRequest {
    description: String,
    domain_id: String,
    document_type: String,
}

#[derive(Debug, Serialize)]
struct SuggestImprovementsRequest {
    workflow_id: Uuid,
}

#[derive(Debug, Serialize)]
struct SuggestDecisionRequest {
    document_id: Uuid,
    stage_id: String,
}

#[derive(Debug, Serialize)]
struct OptimizeWorkflowRequest {
    workflow_id: Uuid,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowAnalysis {
    pub workflow_id: Uuid,
    pub total_stages: usize,
    pub avg_completion_hours: Option<f64>,
    pub sla_compliance_pct: f64,
    pub bottleneck: Option<BottleneckInfo>,
    pub stage_insights: Vec<StageInsight>,
    pub total_executions: i32,
    pub analyzed_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DomainAnalysis {
    pub domain_id: String,
    pub total_workflows: usize,
    pub workflow_analyses: Vec<WorkflowAnalysis>,
    pub avg_completion_hours: f64,
    pub overall_sla_compliance_pct: f64,
    pub common_bottlenecks: Vec<CommonBottleneck>,
    pub analyzed_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigGenerationResult {
    pub config: WorkflowConfig,
    pub language: String,
    pub warnings: Vec<String>,
    pub generated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowConfig {
    pub workflow_name: String,
    pub description: String,
    pub stages: Vec<StageConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StageConfig {
    pub stage_id: String,
    pub stage_name: String,
    pub stage_order: i32,
    pub approval_type: String,
    pub approvers: Vec<ApproverConfig>,
    pub sla_hours: i32,
    pub expiry_action: String,
    pub is_optional: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApproverConfig {
    pub role: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApprovalSuggestion {
    pub recommended_decision: String,
    pub confidence: f32,
    pub rationale: String,
    pub risk_factors: Vec<String>,
    pub positive_factors: Vec<String>,
    pub conditions: Vec<String>,
    pub delegate_to: Option<String>,
    pub generated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OptimizationReport {
    pub workflow_id: Uuid,
    pub current_sla_compliance: f64,
    pub current_avg_hours: f64,
    pub target_sla_compliance: f64,
    pub target_avg_hours: f64,
    pub priority: String,
    pub suggestions: Vec<OptimizationSuggestion>,
    pub estimated_impact: ImpactEstimate,
    pub generated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OptimizationSuggestion {
    pub suggestion_type: String,
    pub title: String,
    pub description: String,
    pub actions: Vec<ActionItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActionItem {
    pub action: String,
    pub details: String,
    pub effort: String,
    pub impact: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImpactEstimate {
    pub current_sla_compliance_pct: f64,
    pub projected_sla_compliance_pct: f64,
    pub current_avg_hours: f64,
    pub projected_avg_hours: f64,
    pub time_to_implement_weeks: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowDashboardData {
    pub workflow_id: Uuid,
    pub total_stages: usize,
    pub avg_completion_hours: Option<f64>,
    pub sla_compliance_pct: f64,
    pub bottleneck: Option<BottleneckInfo>,
    pub stage_insights: Vec<StageInsight>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BottleneckInfo {
    pub stage_id: String,
    pub stage_name: String,
    pub avg_duration_hours: f64,
    pub sla_compliance_pct: f64,
    pub issue_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StageInsight {
    pub stage_id: String,
    pub stage_name: String,
    pub avg_duration_hours: Option<f64>,
    pub sla_compliance_pct: Option<f64>,
    pub risk_level: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommonBottleneck {
    pub stage_name: String,
    pub affected_workflows: usize,
    pub avg_duration_hours: f64,
}

// ============================================================================
// Mock Functions (for development)
// ============================================================================

async fn mock_analyze_workflow(workflow_id: Uuid) -> Result<WorkflowAnalysis, String> {
    Ok(WorkflowAnalysis {
        workflow_id,
        total_stages: 3,
        avg_completion_hours: Some(48.2),
        sla_compliance_pct: 78.5,
        bottleneck: Some(BottleneckInfo {
            stage_id: "stage_3".to_string(),
            stage_name: "Finale Goedkeuring".to_string(),
            avg_duration_hours: 18.1,
            sla_compliance_pct: 58.0,
            issue_type: "SlowApproval".to_string(),
        }),
        stage_insights: vec![
            StageInsight {
                stage_id: "stage_1".to_string(),
                stage_name: "Juridische Check".to_string(),
                avg_duration_hours: Some(6.2),
                sla_compliance_pct: Some(94.0),
                risk_level: "Low".to_string(),
                recommendation: "Stage performs well within SLA".to_string(),
            },
            StageInsight {
                stage_id: "stage_2".to_string(),
                stage_name: "Management Review".to_string(),
                avg_duration_hours: Some(12.4),
                sla_compliance_pct: Some(72.0),
                risk_level: "Medium".to_string(),
                recommendation: "Consider adding parallel approvers".to_string(),
            },
            StageInsight {
                stage_id: "stage_3".to_string(),
                stage_name: "Finale Goedkeuring".to_string(),
                avg_duration_hours: Some(18.1),
                sla_compliance_pct: Some(58.0),
                risk_level: "High".to_string(),
                recommendation: "Critical SLA breaches detected".to_string(),
            },
        ],
        total_executions: 234,
        analyzed_at: chrono::Utc::now().to_rfc3339(),
    })
}

async fn mock_analyze_domain(domain_id: &str) -> Result<DomainAnalysis, String> {
    Ok(DomainAnalysis {
        domain_id: domain_id.to_string(),
        total_workflows: 5,
        workflow_analyses: vec![],
        avg_completion_hours: 52.3,
        overall_sla_compliance_pct: 75.2,
        common_bottlenecks: vec![
            CommonBottleneck {
                stage_name: "Finale Goedkeuring".to_string(),
                affected_workflows: 3,
                avg_duration_hours: 19.5,
            },
        ],
        analyzed_at: chrono::Utc::now().to_rfc3339(),
    })
}

async fn mock_generate_config(
    description: String,
    domain_id: String,
    document_type: String,
) -> Result<ConfigGenerationResult, String> {
    Ok(ConfigGenerationResult {
        config: WorkflowConfig {
            workflow_name: format!("{} Workflow", document_type),
            description,
            stages: vec![
                StageConfig {
                    stage_id: "stage_1".to_string(),
                    stage_name: "Juridische Check".to_string(),
                    stage_order: 1,
                    approval_type: "parallel_any".to_string(),
                    approvers: vec![ApproverConfig { role: "Jurist".to_string() }],
                    sla_hours: 24,
                    expiry_action: "notify_only".to_string(),
                    is_optional: false,
                },
                StageConfig {
                    stage_id: "stage_2".to_string(),
                    stage_name: "Management Goedkeuring".to_string(),
                    stage_order: 2,
                    approval_type: "sequential".to_string(),
                    approvers: vec![
                        ApproverConfig { role: "Manager".to_string() },
                        ApproverConfig { role: "Team Lead".to_string() },
                    ],
                    sla_hours: 48,
                    expiry_action: "escalate_to".to_string(),
                    is_optional: false,
                },
            ],
        },
        language: "Dutch".to_string(),
        warnings: vec![],
        generated_at: chrono::Utc::now().to_rfc3339(),
    })
}

async fn mock_approval_suggestion(
    document_id: Uuid,
    stage_id: String,
) -> Result<ApprovalSuggestion, String> {
    Ok(ApprovalSuggestion {
        recommended_decision: "approve".to_string(),
        confidence: 0.92,
        rationale: "Document voldoet aan alle Woo vereisten. Geen weigeringsgronden aanwezig.".to_string(),
        risk_factors: vec![],
        positive_factors: vec![
            "Volledige documentatie aanwezig".to_string(),
            "Geen PII detecties".to_string(),
            "Alle verplichte secties compleet".to_string(),
        ],
        conditions: vec![],
        delegate_to: None,
        generated_at: chrono::Utc::now().to_rfc3339(),
    })
}

async fn mock_optimize_workflow(workflow_id: Uuid) -> Result<OptimizationReport, String> {
    Ok(OptimizationReport {
        workflow_id,
        current_sla_compliance: 78.5,
        current_avg_hours: 48.2,
        target_sla_compliance: 90.0,
        target_avg_hours: 38.5,
        priority: "Medium".to_string(),
        suggestions: vec![],
        estimated_impact: ImpactEstimate {
            current_sla_compliance_pct: 78.5,
            projected_sla_compliance_pct: 90.0,
            current_avg_hours: 48.2,
            projected_avg_hours: 38.5,
            time_to_implement_weeks: 4,
        },
        generated_at: chrono::Utc::now().to_rfc3339(),
    })
}

async fn mock_workflow_analytics(workflow_id: Uuid) -> Result<WorkflowDashboardData, String> {
    let analysis = mock_analyze_workflow(workflow_id).await?;
    Ok(WorkflowDashboardData {
        workflow_id: analysis.workflow_id,
        total_stages: analysis.total_stages,
        avg_completion_hours: analysis.avg_completion_hours,
        sla_compliance_pct: analysis.sla_compliance_pct,
        bottleneck: analysis.bottleneck,
        stage_insights: analysis.stage_insights,
    })
}
