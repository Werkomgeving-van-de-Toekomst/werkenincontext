//! Multi-agent document creation system.
//!
//! This module implements a pipeline of specialized agents for automated
//! document generation in the IOU-Modern system.

pub mod config;
pub mod research;

use thiserror::Error;

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
}

// Re-export agent types
pub use research::{ResearchContext, execute_research_agent};
pub use config::{AgentConfig, ResearchAgentConfig};
