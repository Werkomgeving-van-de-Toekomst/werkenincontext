//! Error types for the workflow orchestrator

use thiserror::Error;

/// Main orchestrator error type
#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("State machine error: {0}")]
    State(#[from] StateError),

    #[error("Checkpoint error: {0}")]
    Checkpoint(#[from] CheckpointError),

    #[error("Agent execution failed: {agent} - {message}")]
    AgentFailed {
        agent: String,
        message: String,
        severity: ErrorSeverity,
    },

    #[error("Agent timeout: {agent} did not complete within {timeout_ms}ms")]
    AgentTimeout { agent: String, timeout_ms: u64 },

    #[error("Agent cancelled: {reason}")]
    AgentCancelled { reason: String },

    #[error("Approval required but not provided")]
    ApprovalRequired,

    #[error("Approval timeout: workflow {workflow_id} timed out after {hours}h")]
    ApprovalTimeout { workflow_id: String, hours: u32 },

    #[error("Invalid configuration: {0}")]
    Configuration(String),

    #[error("Compatibility error: {0}")]
    Compatibility(#[from] CompatibilityError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Severity of an error, determines retry behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Transient error - retry with backoff
    Transient,
    /// Permanent error - fail immediately
    Permanent,
}

/// State machine related errors
#[derive(Debug, Error)]
pub enum StateError {
    #[error("Invalid state transition: {from:?} -> {to:?}")]
    InvalidTransition {
        from: String,
        to: String,
    },

    #[error("Invalid state string: {0}")]
    InvalidState(String),

    #[error("State reconstruction failed: {0}")]
    ReconstructionFailed(String),

    #[error("Guard condition failed: {0}")]
    GuardFailed(String),
}

/// Checkpoint related errors
#[derive(Debug, Error)]
pub enum CheckpointError {
    #[error("Invalid JSON in checkpoint: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("Invalid state in checkpoint: {0}")]
    InvalidState(String),

    #[error("Checkpoint not found for workflow: {0}")]
    NotFound(String),

    #[error("Checkpoint validation failed: {0}")]
    ValidationFailed(String),

    #[error("Storage error: {0}")]
    Storage(String),
}

/// Version compatibility errors
#[derive(Debug, Error)]
pub enum CompatibilityError {
    #[error("Version mismatch: checkpoint has {checkpoint}, current is {current}")]
    VersionMismatch {
        checkpoint: String,
        current: String,
    },

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(String),

    #[error("Migration required from {from} to {to} but not supported")]
    MigrationNotSupported { from: String, to: String },
}
