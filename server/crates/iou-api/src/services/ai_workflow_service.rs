//! AI Workflow Service
//!
//! Provides AI-powered workflow analysis, configuration generation,
//! and optimization recommendations.

use crate::error::ApiError;
use iou_ai::agents::workflow::{
    WorkflowConfigGenerator, ApprovalAssistant, WorkflowOptimizer,
    WorkflowContext, WorkflowAIError, SuggestionContext, ConfigGenerationResult,
    ApprovalSuggestion, OptimizationReport,
};
use iou_ai::agents::workflow::config_generator::{PerformanceSnapshot, WorkflowConfig};
use iou_ai::agents::workflow::optimizer::{WorkflowOptimizationData, BottleneckInfo, StageInsight, RiskLevel, BottleneckType};
use iou_core::ai::ollama::OllamaClient;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// AI Workflow Service
pub struct AiWorkflowService {
    db: PgPool,
    generator: Arc<WorkflowConfigGenerator>,
    assistant: Arc<ApprovalAssistant>,
    optimizer: Arc<WorkflowOptimizer>,
}

impl AiWorkflowService {
    pub fn new(db: PgPool, ollama_base_url: String) -> Result<Self, ApiError> {
        let ollama = OllamaClient::new(ollama_base_url);

        Ok(Self {
            db: db.clone(),
            generator: Arc::new(WorkflowConfigGenerator::new(ollama.clone())),
            assistant: Arc::new(ApprovalAssistant::new(ollama.clone())),
            optimizer: Arc::new(WorkflowOptimizer::new(ollama)),
        })
    }

    /// Analyze workflow performance
    pub async fn analyze_workflow(
        &self,
        workflow_id: Uuid,
    ) -> Result<WorkflowAnalysis, ApiError> {
        let stages = self.get_workflow_stages(workflow_id).await?;
        let metrics = self.get_stage_metrics(workflow_id).await?;
        let analytics = self.get_workflow_analytics(workflow_id).await?;

        let bottleneck = self.identify_bottleneck(&stages, &metrics).await;
        let sla_compliance = self.calculate_sla_compliance(&metrics).await;
        let stage_insights = self.build_stage_insights(&stages, &metrics).await;

        Ok(WorkflowAnalysis {
            workflow_id,
            total_stages: stages.len(),
            avg_completion_hours: analytics.avg_completion_hours,
            sla_compliance_pct: sla_compliance,
            bottleneck,
            stage_insights,
            total_executions: analytics.total_executions,
            analyzed_at: Utc::now(),
        })
    }

    /// Generate workflow configuration from natural language
    pub async fn generate_config(
        &self,
        description: String,
        domain_id: String,
        document_type: String,
    ) -> Result<ConfigGenerationResult, ApiError> {
        let context = WorkflowContext::new(domain_id, document_type);
        self.generator.generate_from_description(&description, &context).await
            .map_err(|e| ApiError::Internal(format!("Config generation failed: {}", e)))
    }

    /// Suggest improvements to existing configuration
    pub async fn suggest_config_improvements(
        &self,
        workflow_id: Uuid,
    ) -> Result<Vec<iou_ai::agents::workflow::config_generator::ConfigSuggestion>, ApiError> {
        let analysis = self.analyze_workflow(workflow_id).await?;
        let snapshot = PerformanceSnapshot {
            avg_completion_hours: analysis.avg_completion_hours,
            sla_compliance_pct: analysis.sla_compliance_pct,
            bottleneck_stage: analysis.bottleneck.as_ref().map(|b| b.stage_name.clone()),
            total_executions: analysis.total_executions,
        };

        let config = WorkflowConfig {
            workflow_name: "Current Workflow".to_string(),
            description: String::new(),
            stages: vec![],
        };

        self.generator.suggest_improvements(&config, &snapshot).await
            .map_err(|e| ApiError::Internal(format!("Suggestion generation failed: {}", e)))
    }

    /// Get AI approval suggestion for a document stage
    pub async fn suggest_decision(
        &self,
        document_id: Uuid,
        stage_id: String,
    ) -> Result<ApprovalSuggestion, ApiError> {
        let (doc_title, doc_type, compliance_score, stage_name, approver_role) =
            self.fetch_approval_context(document_id, &stage_id).await?;

        let context = SuggestionContext {
            document_id,
            document_title: doc_title.clone(),
            document_type: doc_type.clone(),
            stage_id: stage_id.clone(),
            stage_name: stage_name.clone(),
            compliance_score,
            issues: vec![],
            pii_detected: false,
            refusal_grounds: vec![],
            deadline: None,
            approver_role,
        };

        self.assistant.suggest_decision(&context).await
            .map_err(|e| ApiError::Internal(format!("Decision suggestion failed: {}", e)))
    }

    /// Generate document summary for approval
    pub async fn generate_summary(
        &self,
        document_id: Uuid,
    ) -> Result<String, ApiError> {
        let (title, doc_type, key_points) = self.fetch_document_summary_context(document_id).await?;

        self.assistant.generate_summary(&title, &doc_type, &key_points).await
            .map_err(|e| ApiError::Internal(format!("Summary generation failed: {}", e)))
    }

    /// Get optimization report for a workflow
    pub async fn optimize_workflow(
        &self,
        workflow_id: Uuid,
    ) -> Result<OptimizationReport, ApiError> {
        let analysis = self.analyze_workflow(workflow_id).await?;

        let data = WorkflowOptimizationData {
            workflow_id: analysis.workflow_id,
            total_stages: analysis.total_stages,
            avg_completion_hours: analysis.avg_completion_hours,
            sla_compliance_pct: analysis.sla_compliance_pct,
            bottleneck: analysis.bottleneck.clone(),
            stage_insights: analysis.stage_insights.clone(),
            total_executions: analysis.total_executions,
        };

        self.optimizer.create_report(&data).await
            .map_err(|e| ApiError::Internal(format!("Optimization failed: {}", e)))
    }

    /// Fetch context for approval suggestion
    async fn fetch_approval_context(
        &self,
        document_id: Uuid,
        _stage_id: &str,
    ) -> Result<(String, String, f32, String, String), ApiError> {
        let row = sqlx::query(
            r#"
            SELECT
                d.title,
                d.document_type,
                COALESCE(cr.score, 1.0) as compliance_score,
                COALESCE(as_def.stage_name, 'Review') as stage_name,
                COALESCE(as_def.approvers->0->>'role', 'Approver') as approver_role
            FROM documents d
            LEFT JOIN compliance_results cr ON cr.document_id = d.id
            LEFT JOIN document_approval_stages das ON das.document_id = d.id
            LEFT JOIN approval_stages as_def ON as_def.stage_id = das.stage_id
            WHERE d.id = $1
            LIMIT 1
            "#
        )
        .bind(document_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

        row.map(|r| {
            (
                r.get("title"),
                r.get("document_type"),
                r.get("compliance_score"),
                r.get("stage_name"),
                r.get("approver_role"),
            )
        }).ok_or_else(|| ApiError::NotFound("Document not found".to_string()))
    }

    /// Fetch context for document summary
    async fn fetch_document_summary_context(
        &self,
        document_id: Uuid,
    ) -> Result<(String, String, Vec<String>), ApiError> {
        let row = sqlx::query(
            r#"
            SELECT title, document_type
            FROM documents
            WHERE id = $1
            "#
        )
        .bind(document_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

        row.map(|r| {
            let title: String = r.get("title");
            let doc_type: String = r.get("document_type");

            let key_points = vec![
                format!("Document type: {}", doc_type),
                format!("Title: {}", title),
                "Requires review and approval".to_string(),
            ];

            (title, doc_type, key_points)
        }).ok_or_else(|| ApiError::NotFound("Document not found".to_string()))
    }

    /// Get workflow stage definitions
    async fn get_workflow_stages(
        &self,
        workflow_id: Uuid,
    ) -> Result<Vec<StageInfo>, ApiError> {
        let rows = sqlx::query(
            r#"
            SELECT stage_id, stage_name, stage_order, approval_type, sla_hours
            FROM approval_stages
            WHERE workflow_id = $1
            ORDER BY stage_order
            "#
        )
        .bind(workflow_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

        Ok(rows.into_iter().map(|row| StageInfo {
            stage_id: row.get("stage_id"),
            stage_name: row.get("stage_name"),
            stage_order: row.get("stage_order"),
            approval_type: row.get("approval_type"),
            sla_hours: row.get("sla_hours"),
        }).collect())
    }

    /// Get completion metrics for workflow stages
    async fn get_stage_metrics(
        &self,
        _workflow_id: Uuid,
    ) -> Result<Vec<StageMetrics>, ApiError> {
        // Return mock data for now - in production, query actual metrics
        Ok(vec![])
    }

    /// Get workflow analytics record
    async fn get_workflow_analytics(
        &self,
        workflow_id: Uuid,
    ) -> Result<WorkflowAnalyticsRecord, ApiError> {
        let row = sqlx::query_as::<_, WorkflowAnalyticsRecord>(
            "SELECT * FROM workflow_analytics WHERE workflow_id = $1"
        )
        .bind(workflow_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

        Ok(row.unwrap_or_else(|| WorkflowAnalyticsRecord {
            id: Uuid::new_v4(),
            workflow_id,
            domain_id: String::new(),
            document_type: String::new(),
            avg_completion_hours: None,
            sla_compliance_pct: None,
            bottleneck_stage_id: None,
            total_executions: 0,
            last_analyzed_at: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }))
    }

    /// Identify the bottleneck stage
    async fn identify_bottleneck(
        &self,
        stages: &[StageInfo],
        metrics: &[StageMetrics],
    ) -> Option<BottleneckInfo> {
        metrics.iter()
            .filter(|m| m.execution_count > 0)
            .max_by(|a, b| {
                a.avg_duration_hours
                    .partial_cmp(&b.avg_duration_hours)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .and_then(|m| {
                let stage = stages.iter().find(|s| s.stage_id == m.stage_id)?;
                Some(BottleneckInfo {
                    stage_id: m.stage_id.clone(),
                    stage_name: stage.stage_name.clone(),
                    avg_duration_hours: m.avg_duration_hours,
                    sla_compliance_pct: m.sla_compliance_pct,
                    issue_type: if m.sla_compliance_pct < 50.0 {
                        BottleneckType::CriticalSLABreach
                    } else if m.avg_duration_hours > 24.0 {
                        BottleneckType::LongDelay
                    } else {
                        BottleneckType::SlowApproval
                    },
                })
            })
    }

    /// Calculate overall SLA compliance
    async fn calculate_sla_compliance(&self, metrics: &[StageMetrics]) -> f64 {
        if metrics.is_empty() {
            return 100.0;
        }

        let total_executions: f64 = metrics.iter().map(|m| m.execution_count as f64).sum();
        if total_executions == 0.0 {
            return 100.0;
        }

        let weighted_sla: f64 = metrics.iter()
            .map(|m| m.sla_compliance_pct * m.execution_count as f64)
            .sum();

        weighted_sla / total_executions
    }

    /// Build insights for each stage
    async fn build_stage_insights(
        &self,
        stages: &[StageInfo],
        metrics: &[StageMetrics],
    ) -> Vec<StageInsight> {
        stages.iter().map(|stage| {
            let stage_metrics = metrics.iter()
                .find(|m| m.stage_id == stage.stage_id);

            match stage_metrics {
                Some(m) if m.execution_count > 0 => {
                    let risk_level = if m.sla_compliance_pct < 50.0 {
                        RiskLevel::Critical
                    } else if m.sla_compliance_pct < 75.0 {
                        RiskLevel::High
                    } else if m.sla_compliance_pct < 90.0 {
                        RiskLevel::Medium
                    } else {
                        RiskLevel::Low
                    };

                    StageInsight {
                        stage_id: stage.stage_id.clone(),
                        stage_name: stage.stage_name.clone(),
                        avg_duration_hours: Some(m.avg_duration_hours),
                        sla_compliance_pct: Some(m.sla_compliance_pct),
                        risk_level,
                        recommendation: format!("Stage performs at {:.1}% SLA compliance", m.sla_compliance_pct),
                    }
                }
                _ => StageInsight {
                    stage_id: stage.stage_id.clone(),
                    stage_name: stage.stage_name.clone(),
                    avg_duration_hours: None,
                    sla_compliance_pct: None,
                    risk_level: RiskLevel::Unknown,
                    recommendation: "Insufficient data for analysis".to_string(),
                }
            }
        }).collect()
    }
}

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowAnalysis {
    pub workflow_id: Uuid,
    pub total_stages: usize,
    pub avg_completion_hours: Option<f64>,
    pub sla_compliance_pct: f64,
    pub bottleneck: Option<BottleneckInfo>,
    pub stage_insights: Vec<StageInsight>,
    pub total_executions: i32,
    pub analyzed_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct StageInfo {
    stage_id: String,
    stage_name: String,
    stage_order: i32,
    approval_type: String,
    sla_hours: i32,
}

#[derive(Debug, Clone)]
struct StageMetrics {
    stage_id: String,
    execution_count: usize,
    avg_duration_hours: f64,
    sla_compliance_pct: f64,
    total_delegations: usize,
    total_rejections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowAnalyticsRecord {
    id: Uuid,
    workflow_id: Uuid,
    domain_id: String,
    document_type: String,
    avg_completion_hours: Option<f64>,
    sla_compliance_pct: Option<f64>,
    bottleneck_stage_id: Option<Uuid>,
    total_executions: i32,
    last_analyzed_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
