//! IOU-Orchestrator: Workflow orchestration with human-in-the-loop
//!
//! This crate provides a state machine driven workflow system for coordinating
//! AI agents with human approval points at each step.
//!
//! # Architecture
//!
//! - [`state_machine`]: State machine definition and multi-stage transitions
//! - [`stage_executor`]: Stage execution engine with approval lifecycle management
//! - [`context`]: Workflow context and related types
//! - [`version`]: Workflow versioning for compatibility
//! - [`config`]: Orchestrator configuration
//! - [`error`]: Error types

pub mod state_machine;
pub mod stage_executor;
pub mod context;
pub mod version;
pub mod config;
pub mod error;
pub mod jobs;

// Re-exports for convenience
pub use state_machine::{
    WorkflowStateMachine, WorkflowState, WorkflowEvent,
    StageCompletionStatus, WorkflowTransition,
    transition_to_next_stage, evaluate_stage_completion, is_valid_transition,
};
pub use stage_executor::{
    StageExecutor, StageExecutorContext, StageExecutorError,
    Database, DelegationResolver, SlaCalculator, NotificationService, DbError, Result,
};
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
