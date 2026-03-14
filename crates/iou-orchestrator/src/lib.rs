//! IOU-Orchestrator: Workflow orchestration with human-in-the-loop
//!
//! This crate provides a state machine driven workflow system for coordinating
//! AI agents with human approval points at each step.
//!
//! # Architecture
//!
//! - [`state_machine`]: Smlang-based state machine definition
//! - [`context`]: Workflow context and related types
//! - [`executor`]: Agent execution with timeout and retry
//! - [`checkpoint`]: Checkpoint management for crash recovery
//! - [`events`]: Event types and bus for real-time updates
//! - [`approval`]: Human-in-the-loop approval workflow
//! - [`timeout`]: Timeout handling with escalation
//! - [`version`]: Workflow versioning for compatibility

pub mod state_machine;
pub mod context;
pub mod version;
pub mod config;
pub mod error;

// Re-exports for convenience
pub use state_machine::{WorkflowStateMachine, WorkflowState, WorkflowEvent};
pub use context::{
    WorkflowContext, AgentType, AgentResult, ApprovalRequest, ApprovalDecision,
    HumanModification, AuditEntry, DocumentRequest, DocumentType,
};
pub use version::{WorkflowVersion, VersionCompatibility};
pub use config::{OrchestratorConfig, RetryPolicy, TimeoutConfig};
pub use error::{OrchestratorError, CheckpointError, StateError, CompatibilityError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_imports() {
        // Basic smoke test to ensure library structure is valid
        let _state = WorkflowState::Created;
    }
}
