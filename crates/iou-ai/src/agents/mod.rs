//! Multi-agent document creation system.
//!
//! This module implements a pipeline of specialized agents for automated
//! document generation in the IOU-Modern system.

pub mod config;
pub mod research;
pub mod content;
pub mod compliance;
pub mod review;
pub mod error;
pub mod pipeline;

pub use error::{PipelineError, ErrorSeverity};
pub use pipeline::{
    AgentPipeline, AgentPipelineWithConfig, PipelineConfig,
    AgentExecutionResult, PipelineCheckpoint, PipelineResult,
};

use thiserror::Error;
use crate::llm::LlmError;

/// Common error type for all agents
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("GraphRAG query failed: {0}")]
    GraphRagQueryFailed(String),

    #[error("No similar documents found and no default template available")]
    NoSimilarDocuments,

    #[error("Invalid document type: {0}")]
    InvalidDocumentType(String),

    #[error("Domain configuration not found: {0}")]
    DomainNotFound(String),

    #[error("AI provider error: {0}")]
    AiProviderError(String),

    #[error("Transient error (will retry): {0}")]
    TransientError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Compliance check failed: {0}")]
    ComplianceError(String),

    #[error("Review check failed: {0}")]
    ReviewError(String),
}

impl From<LlmError> for AgentError {
    fn from(err: LlmError) -> Self {
        AgentError::AiProviderError(err.to_string())
    }
}

// Re-export agent types
pub use research::{ResearchContext, execute_research_agent};
pub use content::{GeneratedDocument, execute_content_agent, ContentAgentConfig, EntityLink, SectionMetadata};
pub use compliance::{
    ComplianceResult, execute_compliance_agent, ComplianceConfig,
    PiiLocation, PiiType, AccessibilityIssue, AccessibilityLevel
};
pub use review::{
    ReviewDecision, ReviewAction, execute_review_agent, ReviewConfig,
    QualityIssue, QualityIssueCategory,
};
pub use config::{AgentConfig, ResearchAgentConfig};
