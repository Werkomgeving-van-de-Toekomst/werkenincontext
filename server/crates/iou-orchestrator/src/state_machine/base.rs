//! Workflow state machine definition
//!
//! Defines the states, events, and transitions for the document creation workflow.
//!
//! Note: Using hand-rolled state machine for better control and maintainability.
//! The smlang DSL was evaluated but a custom implementation provides clearer
//! error handling and better integration with our async workflow.

use crate::context::{AgentType, WorkflowContext};
use crate::error::StateError;
use std::fmt;

// ============================================
// State Machine Definition
// ============================================

/// All possible workflow states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkflowState {
    /// Workflow created but not started
    Created,
    /// Agents are executing
    Running,
    /// Waiting for human approval
    AwaitingApproval,
    /// Approval timeout - waiting for escalation
    AwaitingEscalation,
    /// All agents completed successfully
    Completed,
    /// Workflow failed
    Failed,
    /// Workflow was cancelled
    Cancelled,
    /// Retrying a failed agent
    Retrying,
    /// Workflow archived
    Archived,
}

/// All possible events that can trigger state transitions
#[derive(Debug, Clone)]
pub enum WorkflowEvent {
    /// Start the workflow
    Start,
    /// Agent completed without approval needed
    AgentComplete(AgentType),
    /// Agent completed and needs approval
    AgentCompletePending(AgentType),
    /// All agents completed
    AllAgentsComplete,
    /// Agent failed
    AgentFailed,
    /// Retry attempt
    RetryAttempt,
    /// Max retries exceeded
    MaxRetriesExceeded,
    /// Approval granted
    Approved,
    /// Approval with modifications
    Modified,
    /// Approval rejected
    Rejected,
    /// Approval timeout - escalate
    TimeoutEscalated,
    /// Cancel workflow
    Cancel,
    /// Archive completed workflow
    Archive,
}

impl fmt::Display for WorkflowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowState::Created => write!(f, "Created"),
            WorkflowState::Running => write!(f, "Running"),
            WorkflowState::AwaitingApproval => write!(f, "AwaitingApproval"),
            WorkflowState::AwaitingEscalation => write!(f, "AwaitingEscalation"),
            WorkflowState::Completed => write!(f, "Completed"),
            WorkflowState::Failed => write!(f, "Failed"),
            WorkflowState::Cancelled => write!(f, "Cancelled"),
            WorkflowState::Retrying => write!(f, "Retrying"),
            WorkflowState::Archived => write!(f, "Archived"),
        }
    }
}

impl WorkflowState {
    /// Convert state to string for storage
    pub fn to_str(&self) -> &'static str {
        match self {
            WorkflowState::Created => "Created",
            WorkflowState::Running => "Running",
            WorkflowState::AwaitingApproval => "AwaitingApproval",
            WorkflowState::AwaitingEscalation => "AwaitingEscalation",
            WorkflowState::Completed => "Completed",
            WorkflowState::Failed => "Failed",
            WorkflowState::Cancelled => "Cancelled",
            WorkflowState::Retrying => "Retrying",
            WorkflowState::Archived => "Archived",
        }
    }

    /// Parse state from string
    pub fn from_str(s: &str) -> Result<Self, StateError> {
        match s {
            "Created" => Ok(WorkflowState::Created),
            "Running" => Ok(WorkflowState::Running),
            "AwaitingApproval" => Ok(WorkflowState::AwaitingApproval),
            "AwaitingEscalation" => Ok(WorkflowState::AwaitingEscalation),
            "Completed" => Ok(WorkflowState::Completed),
            "Failed" => Ok(WorkflowState::Failed),
            "Cancelled" => Ok(WorkflowState::Cancelled),
            "Retrying" => Ok(WorkflowState::Retrying),
            "Archived" => Ok(WorkflowState::Archived),
            _ => Err(StateError::InvalidState(s.to_string())),
        }
    }

    /// Check if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            WorkflowState::Completed
                | WorkflowState::Failed
                | WorkflowState::Cancelled
                | WorkflowState::Archived
        )
    }

    /// Check if this state requires human input
    pub fn requires_human(&self) -> bool {
        matches!(
            self,
            WorkflowState::AwaitingApproval | WorkflowState::AwaitingEscalation
        )
    }
}

// ============================================
// State Machine
// ============================================

/// The workflow state machine with context
#[derive(Debug, Clone)]
pub struct WorkflowStateMachine {
    /// Current state
    state: WorkflowState,
    /// The workflow context
    pub context: WorkflowContext,
}

impl WorkflowStateMachine {
    /// Create a new workflow state machine
    pub fn new(context: WorkflowContext) -> Self {
        Self {
            state: WorkflowState::Created,
            context,
        }
    }

    /// Get the current state
    pub fn state(&self) -> &WorkflowState {
        &self.state
    }

    /// Get the workflow context
    pub fn context(&self) -> &WorkflowContext {
        &self.context
    }

    /// Get mutable workflow context
    pub fn context_mut(&mut self) -> &mut WorkflowContext {
        &mut self.context
    }

    /// Start the workflow
    pub fn start(&mut self) -> Result<&WorkflowState, StateError> {
        self.transition(&WorkflowEvent::Start)
    }

    /// Record an agent completion (without approval needed)
    pub fn agent_complete(&mut self, agent: AgentType) -> Result<&WorkflowState, StateError> {
        self.transition(&WorkflowEvent::AgentComplete(agent))
    }

    /// Record an agent completion that requires approval
    pub fn agent_complete_pending(&mut self, agent: AgentType) -> Result<&WorkflowState, StateError> {
        self.transition(&WorkflowEvent::AgentCompletePending(agent))
    }

    /// Record that all agents are complete
    pub fn all_agents_complete(&mut self) -> Result<&WorkflowState, StateError> {
        self.transition(&WorkflowEvent::AllAgentsComplete)
    }

    /// Record an agent failure
    pub fn agent_failed(&mut self) -> Result<&WorkflowState, StateError> {
        self.transition(&WorkflowEvent::AgentFailed)
    }

    /// Retry a failed agent
    pub fn retry_attempt(&mut self) -> Result<&WorkflowState, StateError> {
        self.transition(&WorkflowEvent::RetryAttempt)
    }

    /// Mark retries as exceeded
    pub fn max_retries_exceeded(&mut self) -> Result<&WorkflowState, StateError> {
        self.transition(&WorkflowEvent::MaxRetriesExceeded)
    }

    /// Approve the current pending approval
    pub fn approve(&mut self) -> Result<&WorkflowState, StateError> {
        self.transition(&WorkflowEvent::Approved)
    }

    /// Apply modifications and continue
    pub fn modified(&mut self) -> Result<&WorkflowState, StateError> {
        self.transition(&WorkflowEvent::Modified)
    }

    /// Reject the workflow
    pub fn reject(&mut self) -> Result<&WorkflowState, StateError> {
        self.transition(&WorkflowEvent::Rejected)
    }

    /// Handle approval timeout with escalation
    pub fn timeout_escalated(&mut self) -> Result<&WorkflowState, StateError> {
        self.transition(&WorkflowEvent::TimeoutEscalated)
    }

    /// Cancel the workflow
    pub fn cancel(&mut self) -> Result<&WorkflowState, StateError> {
        self.transition(&WorkflowEvent::Cancel)
    }

    /// Archive a completed workflow
    pub fn archive(&mut self) -> Result<&WorkflowState, StateError> {
        self.transition(&WorkflowEvent::Archive)
    }

    /// Check if the workflow is in a terminal state
    pub fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }

    /// Check if the workflow is waiting for human input
    pub fn is_awaiting_human(&self) -> bool {
        self.state.requires_human()
    }

    /// Check if the workflow can execute agents
    pub fn is_executing(&self) -> bool {
        matches!(self.state, WorkflowState::Running | WorkflowState::Retrying)
    }

    /// Reconstruct from a saved state
    pub fn from_state(state: WorkflowState, context: WorkflowContext) -> Result<Self, StateError> {
        Ok(Self { state, context })
    }

    /// Internal: Process a state transition
    fn transition(&mut self, event: &WorkflowEvent) -> Result<&WorkflowState, StateError> {
        let new_state = self.get_next_state(&self.state, event)?;
        let from = self.state.clone();
        self.state = new_state;
        self.context.last_activity = chrono::Utc::now();
        tracing::debug!(
            "Workflow {}: {:?} -> {:?} (event: {:?})",
            self.context.id,
            from,
            self.state,
            event
        );
        Ok(&self.state)
    }

    /// Get the next state given current state and event
    fn get_next_state(&self, state: &WorkflowState, event: &WorkflowEvent) -> Result<WorkflowState, StateError> {
        match (state, event) {
            // From Created
            (WorkflowState::Created, WorkflowEvent::Start) => Ok(WorkflowState::Running),
            (WorkflowState::Created, WorkflowEvent::Cancel) => Ok(WorkflowState::Cancelled),

            // From Running
            (WorkflowState::Running, WorkflowEvent::AgentComplete(_)) => Ok(WorkflowState::Running),
            (WorkflowState::Running, WorkflowEvent::AgentCompletePending(_)) => Ok(WorkflowState::AwaitingApproval),
            (WorkflowState::Running, WorkflowEvent::AllAgentsComplete) => Ok(WorkflowState::Completed),
            (WorkflowState::Running, WorkflowEvent::AgentFailed) => Ok(WorkflowState::Retrying),
            (WorkflowState::Running, WorkflowEvent::Cancel) => Ok(WorkflowState::Cancelled),

            // From AwaitingApproval
            (WorkflowState::AwaitingApproval, WorkflowEvent::Approved) => Ok(WorkflowState::Running),
            (WorkflowState::AwaitingApproval, WorkflowEvent::Modified) => Ok(WorkflowState::Running),
            (WorkflowState::AwaitingApproval, WorkflowEvent::Rejected) => Ok(WorkflowState::Failed),
            (WorkflowState::AwaitingApproval, WorkflowEvent::TimeoutEscalated) => Ok(WorkflowState::AwaitingEscalation),
            (WorkflowState::AwaitingApproval, WorkflowEvent::Cancel) => Ok(WorkflowState::Cancelled),

            // From AwaitingEscalation
            (WorkflowState::AwaitingEscalation, WorkflowEvent::Approved) => Ok(WorkflowState::Running),
            (WorkflowState::AwaitingEscalation, WorkflowEvent::Rejected) => Ok(WorkflowState::Failed),

            // From Retrying
            (WorkflowState::Retrying, WorkflowEvent::RetryAttempt) => Ok(WorkflowState::Running),
            (WorkflowState::Retrying, WorkflowEvent::MaxRetriesExceeded) => Ok(WorkflowState::Failed),
            (WorkflowState::Retrying, WorkflowEvent::Cancel) => Ok(WorkflowState::Cancelled),

            // From Completed
            (WorkflowState::Completed, WorkflowEvent::Archive) => Ok(WorkflowState::Archived),

            // Invalid transitions
            (state, event) => Err(StateError::InvalidTransition {
                from: format!("{:?}", state),
                to: format!("{:?}", event),
            }),
        }
    }
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{DocumentRequest, DocumentType};
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_context() -> WorkflowContext {
        let request = DocumentRequest {
            id: Uuid::new_v4(),
            domain_id: "test".to_string(),
            document_type: DocumentType::WooBesluit,
            context: HashMap::new(),
            requested_by: Uuid::new_v4(),
            requested_at: chrono::Utc::now(),
            priority: Default::default(),
        };

        WorkflowContext::new(Uuid::new_v4(), request, "1.0.0".to_string())
    }

    #[test]
    fn test_state_machine_creation() {
        let context = create_test_context();
        let sm = WorkflowStateMachine::new(context);
        assert_eq!(sm.state(), &WorkflowState::Created);
    }

    #[test]
    fn test_state_machine_start() {
        let context = create_test_context();
        let mut sm = WorkflowStateMachine::new(context);
        let result = sm.start();
        assert!(result.is_ok());
        assert_eq!(sm.state(), &WorkflowState::Running);
    }

    #[test]
    fn test_agent_complete_without_approval() {
        let context = create_test_context();
        let mut sm = WorkflowStateMachine::new(context);
        sm.start().unwrap();
        let result = sm.agent_complete(AgentType::Research);
        assert!(result.is_ok());
        assert_eq!(sm.state(), &WorkflowState::Running);
    }

    #[test]
    fn test_agent_complete_pending_approval() {
        let context = create_test_context();
        let mut sm = WorkflowStateMachine::new(context);
        sm.start().unwrap();
        let result = sm.agent_complete_pending(AgentType::Research);
        assert!(result.is_ok());
        assert_eq!(sm.state(), &WorkflowState::AwaitingApproval);
        assert!(sm.is_awaiting_human());
    }

    #[test]
    fn test_workflow_cancellation() {
        let context = create_test_context();
        let mut sm = WorkflowStateMachine::new(context);
        let result = sm.cancel();
        assert!(result.is_ok());
        assert_eq!(sm.state(), &WorkflowState::Cancelled);
        assert!(sm.is_terminal());
    }

    #[test]
    fn test_approval_flow() {
        let context = create_test_context();
        let mut sm = WorkflowStateMachine::new(context);
        sm.start().unwrap();
        sm.agent_complete_pending(AgentType::Research).unwrap();

        let result = sm.approve();
        assert!(result.is_ok());
        assert_eq!(sm.state(), &WorkflowState::Running);
    }

    #[test]
    fn test_rejection_flow() {
        let context = create_test_context();
        let mut sm = WorkflowStateMachine::new(context);
        sm.start().unwrap();
        sm.agent_complete_pending(AgentType::Research).unwrap();

        let result = sm.reject();
        assert!(result.is_ok());
        assert_eq!(sm.state(), &WorkflowState::Failed);
        assert!(sm.is_terminal());
    }

    #[test]
    fn test_retry_flow() {
        let context = create_test_context();
        let mut sm = WorkflowStateMachine::new(context);
        sm.start().unwrap();
        let result = sm.agent_failed();
        assert!(result.is_ok());
        assert_eq!(sm.state(), &WorkflowState::Retrying);

        let result = sm.retry_attempt();
        assert!(result.is_ok());
        assert_eq!(sm.state(), &WorkflowState::Running);
    }

    #[test]
    fn test_exceeded_retries() {
        let context = create_test_context();
        let mut sm = WorkflowStateMachine::new(context);
        sm.start().unwrap();
        sm.agent_failed().unwrap(); // Now in Retrying

        let result = sm.max_retries_exceeded();
        assert!(result.is_ok());
        assert_eq!(sm.state(), &WorkflowState::Failed);
    }

    #[test]
    fn test_state_serialization() {
        assert_eq!(WorkflowState::Running.to_str(), "Running");
        assert_eq!(WorkflowState::AwaitingApproval.to_str(), "AwaitingApproval");

        assert_eq!(
            WorkflowState::from_str("Running").unwrap(),
            WorkflowState::Running
        );
        assert_eq!(
            WorkflowState::from_str("AwaitingApproval").unwrap(),
            WorkflowState::AwaitingApproval
        );
    }

    #[test]
    fn test_invalid_state_deserialization() {
        let result = WorkflowState::from_str("InvalidState");
        assert!(matches!(result, Err(StateError::InvalidState(_))));
    }

    #[test]
    fn test_state_properties() {
        assert!(WorkflowState::Completed.is_terminal());
        assert!(WorkflowState::Failed.is_terminal());
        assert!(WorkflowState::Cancelled.is_terminal());
        assert!(!WorkflowState::Running.is_terminal());

        assert!(WorkflowState::AwaitingApproval.requires_human());
        assert!(WorkflowState::AwaitingEscalation.requires_human());
        assert!(!WorkflowState::Running.requires_human());
    }

    #[test]
    fn test_reconstruction_from_state() {
        let context = create_test_context();
        let sm = WorkflowStateMachine::from_state(WorkflowState::Running, context);
        assert!(sm.is_ok());
        let sm = sm.unwrap();
        assert_eq!(sm.state(), &WorkflowState::Running);
    }

    #[test]
    fn test_is_awaiting_human() {
        let context = create_test_context();
        let sm = WorkflowStateMachine::new(context);
        assert!(!sm.is_awaiting_human());

        let context = create_test_context();
        let sm = WorkflowStateMachine::from_state(WorkflowState::AwaitingApproval, context).unwrap();
        assert!(sm.is_awaiting_human());

        let context = create_test_context();
        let sm = WorkflowStateMachine::from_state(WorkflowState::AwaitingEscalation, context).unwrap();
        assert!(sm.is_awaiting_human());
    }

    #[test]
    fn test_is_executing() {
        let context = create_test_context();
        let sm = WorkflowStateMachine::new(context);
        assert!(!sm.is_executing());

        let context = create_test_context();
        let sm = WorkflowStateMachine::from_state(WorkflowState::Running, context).unwrap();
        assert!(sm.is_executing());

        let context = create_test_context();
        let sm = WorkflowStateMachine::from_state(WorkflowState::Retrying, context).unwrap();
        assert!(sm.is_executing());

        let context = create_test_context();
        let sm = WorkflowStateMachine::from_state(WorkflowState::AwaitingApproval, context).unwrap();
        assert!(!sm.is_executing());
    }

    #[test]
    fn test_invalid_transition() {
        let context = create_test_context();
        let mut sm = WorkflowStateMachine::new(context);
        // Can't complete before starting
        let result = sm.agent_complete(AgentType::Research);
        assert!(result.is_err());
    }

    #[test]
    fn test_escalation_flow() {
        let context = create_test_context();
        let mut sm = WorkflowStateMachine::new(context);
        sm.start().unwrap();
        sm.agent_complete_pending(AgentType::Research).unwrap();

        // Timeout and escalate
        sm.timeout_escalated().unwrap();
        assert_eq!(sm.state(), &WorkflowState::AwaitingEscalation);

        // Approve from escalation
        sm.approve().unwrap();
        assert_eq!(sm.state(), &WorkflowState::Running);
    }

    #[test]
    fn test_full_workflow() {
        let context = create_test_context();
        let mut sm = WorkflowStateMachine::new(context);

        // Start workflow
        sm.start().unwrap();
        assert_eq!(sm.state(), &WorkflowState::Running);

        // Complete first agent
        sm.agent_complete(AgentType::Research).unwrap();
        assert_eq!(sm.state(), &WorkflowState::Running);

        // Second agent needs approval
        sm.agent_complete_pending(AgentType::Content).unwrap();
        assert_eq!(sm.state(), &WorkflowState::AwaitingApproval);

        // Approve
        sm.approve().unwrap();
        assert_eq!(sm.state(), &WorkflowState::Running);

        // Third agent
        sm.agent_complete(AgentType::Compliance).unwrap();
        assert_eq!(sm.state(), &WorkflowState::Running);

        // All complete
        sm.all_agents_complete().unwrap();
        assert_eq!(sm.state(), &WorkflowState::Completed);
        assert!(sm.is_terminal());
    }
}
