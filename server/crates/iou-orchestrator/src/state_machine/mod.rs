//! Workflow state machine modules

pub mod base;
pub mod multi_stage;

// Re-exports from base state machine
pub use base::{WorkflowStateMachine, WorkflowState, WorkflowEvent};

// Re-exports from multi-stage
pub use multi_stage::{
    StageCompletionStatus, WorkflowTransition,
    transition_to_next_stage, evaluate_stage_completion, is_valid_transition,
};
