//! AI-powered workflow analysis and assistance agents
//!
//! This module provides specialized agents for:
//! - Natural language to workflow configuration generation
//! - Approval decision assistance
//! - Workflow optimization recommendations

pub mod config_generator;
pub mod approval_assistant;
pub mod optimizer;

pub use config_generator::{WorkflowConfigGenerator, ConfigGenerationResult};
pub use approval_assistant::{ApprovalAssistant, ApprovalSuggestion, SuggestionContext};
pub use optimizer::{WorkflowOptimizer, OptimizationReport, OptimizationSuggestion};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Common context for workflow AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    pub workflow_id: Option<Uuid>,
    pub domain_id: String,
    pub document_type: String,
    pub analysis_date: DateTime<Utc>,
}

impl WorkflowContext {
    pub fn new(domain_id: String, document_type: String) -> Self {
        Self {
            workflow_id: None,
            domain_id,
            document_type,
            analysis_date: Utc::now(),
        }
    }

    pub fn with_workflow_id(mut self, workflow_id: Uuid) -> Self {
        self.workflow_id = Some(workflow_id);
        self
    }
}

/// Error type for workflow AI operations
#[derive(Debug, thiserror::Error)]
pub enum WorkflowAIError {
    #[error("LLM error: {0}")]
    LLM(String),

    #[error("Invalid workflow configuration: {0}")]
    InvalidConfig(String),

    #[error("Insufficient data for analysis")]
    InsufficientData,

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for WorkflowAIError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}
