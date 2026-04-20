//! Natural language to workflow configuration generator
//!
//! Converts user descriptions in Dutch or English into structured
//! workflow configurations using LLM assistance.

use super::{WorkflowContext, WorkflowAIError};
use iou_core::ai::ollama::OllamaClient;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Workflow configuration generator
pub struct WorkflowConfigGenerator {
    ollama: OllamaClient,
}

impl WorkflowConfigGenerator {
    pub fn new(ollama: OllamaClient) -> Self {
        Self { ollama }
    }

    /// Generate workflow configuration from natural language description
    pub async fn generate_from_description(
        &self,
        description: &str,
        context: &WorkflowContext,
    ) -> Result<ConfigGenerationResult, WorkflowAIError> {
        let language = self.detect_language(description);
        let prompt = self.build_generation_prompt(description, context, language);

        let response = self
            .ollama
            .generate(&prompt)
            .await
            .map_err(|e| WorkflowAIError::LLM(e.to_string()))?;

        let config: WorkflowConfig = serde_json::from_str(&response)
            .map_err(|e| WorkflowAIError::Serialization(format!("Invalid JSON: {}", e)))?;

        let warnings = self.validate_config(&config);

        Ok(ConfigGenerationResult {
            config,
            language,
            warnings,
            generated_at: Utc::now(),
        })
    }

    /// Suggest improvements to an existing workflow configuration
    pub async fn suggest_improvements(
        &self,
        current_config: &WorkflowConfig,
        performance_data: &PerformanceSnapshot,
    ) -> Result<Vec<ConfigSuggestion>, WorkflowAIError> {
        let prompt = self.build_optimization_prompt(current_config, performance_data);

        let response = self
            .ollama
            .generate(&prompt)
            .await
            .map_err(|e| WorkflowAIError::LLM(e.to_string()))?;

        let suggestions: Vec<ConfigSuggestion> = serde_json::from_str(&response)
            .map_err(|e| WorkflowAIError::Serialization(format!("Invalid JSON: {}", e)))?;

        Ok(suggestions)
    }

    /// Detect language from description
    fn detect_language(&self, text: &str) -> Language {
        let dutch_markers = ["goedkeuring", "fase", "workflow", "stap", "delegatie"];
        let lowercase = text.to_lowercase();

        if dutch_markers.iter().any(|m| lowercase.contains(m)) {
            Language::Dutch
        } else {
            Language::English
        }
    }

    /// Build prompt for workflow generation
    fn build_generation_prompt(
        &self,
        description: &str,
        context: &WorkflowContext,
        language: Language,
    ) -> String {
        let system_prompt = match language {
            Language::Dutch => r#"Je bent een expert in workflow configuratie voor Nederlandse overheidsdocumenten.
Genereer een JSON workflow configuratie op basis van de beschrijving.

Regels:
- Stage names moeten duidelijk en professioneel zijn
- SLA in uren (standaard 24-48 uur voor goedgekeuring)
- Approval types: sequential, parallel_any, parallel_all, parallel_majority
- Expiry actions: notify_only, return_to_draft, auto_approve, escalate_to"#,
            Language::English => r#"You are an expert in workflow configuration for government documents.
Generate a JSON workflow configuration based on the description.

Rules:
- Stage names must be clear and professional
- SLA in hours (default 24-48 hours for approvals)
- Approval types: sequential, parallel_any, parallel_all, parallel_majority
- Expiry actions: notify_only, return_to_draft, auto_approve, escalate_to"#,
        };

        let schema = r#"
Expected JSON format:
{
  "workflow_name": "string",
  "description": "string",
  "stages": [
    {
      "stage_id": "unique_id",
      "stage_name": "string",
      "stage_order": 1,
      "approval_type": "sequential|parallel_any|parallel_all|parallel_majority",
      "approvers": [{"role": "string"}],
      "sla_hours": 48,
      "expiry_action": "notify_only|return_to_draft|auto_approve",
      "is_optional": false
    }
  ]
}"#;

        format!(
            "{}\n\n{}\n\nContext:\n- Domain: {}\n- Document Type: {}\n\nUser Description:\n{}\n\nGenerate only valid JSON, no explanations.",
            system_prompt, schema, context.domain_id, context.document_type, description
        )
    }

    /// Build prompt for optimization suggestions
    fn build_optimization_prompt(
        &self,
        config: &WorkflowConfig,
        performance: &PerformanceSnapshot,
    ) -> String {
        format!(
            r#"Analyze this workflow configuration and suggest improvements based on performance data.

Current Workflow:
{}

Performance Data:
- Avg completion hours: {:.1}
- SLA compliance: {:.1}%
- Common bottleneck: {}
- Total executions: {}

Provide 2-4 specific suggestions in this JSON format:
[
  {{
    "type": "sla_adjustment|approval_type_change|stage_reorder|approver_change",
    "stage_id": "affected_stage_id",
    "description": "Clear explanation of the issue",
    "suggestion": "Specific recommendation",
    "expected_impact": "Expected improvement (e.g., 'Reduces avg time by 12 hours')"
  }}
]"#,
            serde_json::to_string_pretty(config).unwrap_or("{}".to_string()),
            performance.avg_completion_hours.unwrap_or(0.0),
            performance.sla_compliance_pct,
            performance.bottleneck_stage.as_deref().unwrap_or("none"),
            performance.total_executions
        )
    }

    /// Validate generated configuration
    fn validate_config(&self, config: &WorkflowConfig) -> Vec<String> {
        let mut warnings = Vec::new();

        if config.stages.is_empty() {
            warnings.push("Workflow has no stages defined".to_string());
        }

        for (i, stage) in config.stages.iter().enumerate() {
            if stage.stage_order != (i as i32 + 1) {
                warnings.push(format!(
                    "Stage '{}' has order {} but is in position {}",
                    stage.stage_name, stage.stage_order, i + 1
                ));
            }
            if stage.sla_hours < 1 {
                warnings.push(format!(
                    "Stage '{}' has SLA less than 1 hour ({})",
                    stage.stage_name, stage.sla_hours
                ));
            }
            if stage.approvers.is_empty() {
                warnings.push(format!(
                    "Stage '{}' has no approvers defined",
                    stage.stage_name
                ));
            }
        }

        warnings
    }
}

// ============================================================================
// Types
// ============================================================================

/// Result of configuration generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigGenerationResult {
    pub config: WorkflowConfig,
    pub language: Language,
    pub warnings: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

/// Complete workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub workflow_name: String,
    pub description: String,
    pub stages: Vec<StageConfig>,
}

/// Individual stage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageConfig {
    pub stage_id: String,
    pub stage_name: String,
    pub stage_order: i32,
    pub approval_type: ApprovalType,
    pub approvers: Vec<ApproverConfig>,
    pub sla_hours: i32,
    pub expiry_action: ExpiryActionConfig,
    pub is_optional: bool,
}

/// Approval type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalType {
    Sequential,
    ParallelAny,
    ParallelAll,
    ParallelMajority,
}

/// Approver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproverConfig {
    pub role: String,
}

/// Expiry action configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpiryActionConfig {
    NotifyOnly,
    ReturnToDraft,
    AutoApprove,
    EscalateTo { target: String },
}

/// Configuration improvement suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSuggestion {
    pub suggestion_type: SuggestionType,
    pub stage_id: Option<String>,
    pub description: String,
    pub suggestion: String,
    pub expected_impact: String,
}

/// Type of configuration suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionType {
    SlaAdjustment,
    ApprovalTypeChange,
    StageReorder,
    ApproverChange,
}

/// Language detected from input
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Language {
    Dutch,
    English,
}

/// Performance snapshot for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub avg_completion_hours: Option<f64>,
    pub sla_compliance_pct: f64,
    pub bottleneck_stage: Option<String>,
    pub total_executions: i32,
}
