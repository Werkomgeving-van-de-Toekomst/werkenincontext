//! Pipeline error types with severity classification for retry logic.

use thiserror::Error;

/// Error severity for pipeline error handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Transient error: retry with exponential backoff
    Transient,
    /// Permanent error: fail immediately
    Permanent,
}

/// Pipeline execution error
#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Agent {agent} failed: {message}")]
    AgentFailed {
        agent: String,
        message: String,
        severity: ErrorSeverity,
    },

    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        from: iou_core::workflows::WorkflowStatus,
        to: iou_core::workflows::WorkflowStatus,
    },

    #[error("Document {id} not found")]
    DocumentNotFound { id: uuid::Uuid },

    #[error("Maximum iterations ({max}) exceeded")]
    MaxIterationsExceeded { max: usize },

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

impl PipelineError {
    /// Classify an error as transient or permanent based on the error type
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Network/timeout related errors are transient
            PipelineError::AgentFailed { severity, .. } => *severity,

            // State and data errors are permanent
            PipelineError::InvalidStateTransition { .. } => ErrorSeverity::Permanent,
            PipelineError::DocumentNotFound { .. } => ErrorSeverity::Permanent,
            PipelineError::MaxIterationsExceeded { .. } => ErrorSeverity::Permanent,

            // Storage/db errors context-dependent - default to transient
            PipelineError::Storage(_) | PipelineError::Database(_) => ErrorSeverity::Transient,

            // Template and configuration errors are permanent
            PipelineError::Template(_) | PipelineError::Configuration(_) => ErrorSeverity::Permanent,
        }
    }

    /// Create a transient agent error
    pub fn transient_agent(agent: impl Into<String>, message: impl Into<String>) -> Self {
        PipelineError::AgentFailed {
            agent: agent.into(),
            message: message.into(),
            severity: ErrorSeverity::Transient,
        }
    }

    /// Create a permanent agent error
    pub fn permanent_agent(agent: impl Into<String>, message: impl Into<String>) -> Self {
        PipelineError::AgentFailed {
            agent: agent.into(),
            message: message.into(),
            severity: ErrorSeverity::Permanent,
        }
    }
}

impl From<crate::agents::AgentError> for PipelineError {
    fn from(err: crate::agents::AgentError) -> Self {
        match err {
            crate::agents::AgentError::TransientError(msg) => {
                PipelineError::AgentFailed {
                    agent: "Unknown".to_string(),
                    message: msg,
                    severity: ErrorSeverity::Transient,
                }
            }
            crate::agents::AgentError::StorageError(msg) => {
                PipelineError::Storage(msg)
            }
            crate::agents::AgentError::TemplateError(msg) => {
                PipelineError::Template(msg)
            }
            _ => PipelineError::AgentFailed {
                agent: "Unknown".to_string(),
                message: err.to_string(),
                severity: ErrorSeverity::Permanent,
            }
        }
    }
}

impl From<iou_storage::metadata::MetadataError> for PipelineError {
    fn from(err: iou_storage::metadata::MetadataError) -> Self {
        match err {
            iou_storage::metadata::MetadataError::NotFound(_) => {
                PipelineError::Storage(err.to_string())
            }
            iou_storage::metadata::MetadataError::InvalidState(_) => {
                PipelineError::Database(err.to_string())
            }
            _ => PipelineError::Database(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity_transient() {
        let err = PipelineError::transient_agent("TestAgent", "Network timeout");
        assert_eq!(err.severity(), ErrorSeverity::Transient);
    }

    #[test]
    fn test_error_severity_permanent() {
        let err = PipelineError::permanent_agent("TestAgent", "Invalid input");
        assert_eq!(err.severity(), ErrorSeverity::Permanent);
    }

    #[test]
    fn test_error_severity_document_not_found() {
        let err = PipelineError::DocumentNotFound { id: uuid::Uuid::new_v4() };
        assert_eq!(err.severity(), ErrorSeverity::Permanent);
    }

    #[test]
    fn test_error_severity_max_iterations() {
        let err = PipelineError::MaxIterationsExceeded { max: 3 };
        assert_eq!(err.severity(), ErrorSeverity::Permanent);
    }

    #[test]
    fn test_error_severity_storage_defaults_to_transient() {
        let err = PipelineError::Storage("Connection failed".to_string());
        assert_eq!(err.severity(), ErrorSeverity::Transient);
    }

    #[test]
    fn test_error_severity_template_is_permanent() {
        let err = PipelineError::Template("Template not found".to_string());
        assert_eq!(err.severity(), ErrorSeverity::Permanent);
    }
}
