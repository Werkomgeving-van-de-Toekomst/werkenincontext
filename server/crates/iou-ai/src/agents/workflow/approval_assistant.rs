//! Approval assistant agent
//!
//! Helps approvers make informed decisions by analyzing:
//! - Document content and compliance status
//! - Historical approval patterns
//! - Risk assessment
//! - Recommended decision with rationale

use super::{WorkflowContext, WorkflowAIError};
use iou_core::ai::ollama::{OllamaClient, OllamaModel};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Approval assistant for decision support
pub struct ApprovalAssistant {
    ollama: OllamaClient,
}

impl ApprovalAssistant {
    pub fn new(ollama: OllamaClient) -> Self {
        Self { ollama }
    }

    /// Generate an approval suggestion for a document at a specific stage
    pub async fn suggest_decision(
        &self,
        context: &SuggestionContext,
    ) -> Result<ApprovalSuggestion, WorkflowAIError> {
        let prompt = self.build_decision_prompt(context);

        let response = self
            .ollama
            .generate(&prompt, OllamaModel::Llama3_2)
            .await
            .map_err(|e| WorkflowAIError::LLM(e.to_string()))?;

        let suggestion: ApprovalSuggestion = serde_json::from_str(&response)
            .map_err(|e| WorkflowAIError::Serialization(format!("Invalid JSON: {}", e)))?;

        Ok(suggestion)
    }

    /// Generate a quick summary of what needs approval
    pub async fn generate_summary(
        &self,
        document_title: &str,
        document_type: &str,
        key_points: &[String],
    ) -> Result<String, WorkflowAIError> {
        let prompt = format!(
            r#"Summarize the following document for approval review.

Document Type: {}
Title: {}

Key Points:
{}

Provide a 2-3 sentence summary in Dutch focusing on:
1. What this document is about
2. Any potential concerns for approval
3. Recommended action (approve/reject/review)"#,
            document_type, document_title, key_points.join("\n- ")
        );

        self
            .ollama
            .generate(&prompt, OllamaModel::Llama3_2)
            .await
            .map_err(|e| WorkflowAIError::LLM(e.to_string()))
    }

    /// Build prompt for approval decision
    fn build_decision_prompt(&self, context: &SuggestionContext) -> String {
        let system_prompt = r#"You are an expert in Dutch government document approval processes.
Analyze the provided context and recommend an approval decision.

Respond with valid JSON only:
{
  "recommended_decision": "approve|reject|delegate",
  "confidence": 0.0-1.0,
  "rationale": "Clear explanation in Dutch",
  "risk_factors": ["list of concerns"],
  "positive_factors": ["list of strong points"],
  "conditions": ["conditions if approval is conditional"],
  "delegate_to": "role to delegate to if decision is delegate"
}

Decision Guidelines:
- Approve: Document is compliant, complete, and meets all requirements
- Reject: Document has critical issues (non-compliance, missing info, policy violations)
- Delegate: Send to a more appropriate approver (subject matter expert, legal, etc.)"#;

        format!(
            "{}\n\nDocument: {}\nType: {}\nStage: {}\n\nCompliance Score: {:.1}/1.0\nIssues: {}\n\nPII Detected: {}\nRefusal Grounds: {}\n\nProvide recommendation JSON.",
            system_prompt,
            context.document_title,
            context.document_type,
            context.stage_name,
            context.compliance_score,
            context.issues.join(", "),
            context.pii_detected,
            context.refusal_grounds.join(", ")
        )
    }
}

// ============================================================================
// Types
// ============================================================================

/// Context for generating approval suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionContext {
    pub document_id: Uuid,
    pub document_title: String,
    pub document_type: String,
    pub stage_id: String,
    pub stage_name: String,
    pub compliance_score: f32,
    pub issues: Vec<String>,
    pub pii_detected: bool,
    pub refusal_grounds: Vec<String>,
    pub deadline: Option<DateTime<Utc>>,
    pub approver_role: String,
}

/// Approval suggestion from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalSuggestion {
    pub recommended_decision: RecommendedDecision,
    pub confidence: f32,
    pub rationale: String,
    pub risk_factors: Vec<String>,
    pub positive_factors: Vec<String>,
    pub conditions: Vec<String>,
    pub delegate_to: Option<String>,
    pub generated_at: DateTime<Utc>,
}

/// Recommended decision type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendedDecision {
    Approve,
    Reject,
    Delegate,
}
