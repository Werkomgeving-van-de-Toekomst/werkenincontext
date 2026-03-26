//! Workflow orchestrator wrapper
//!
//! This module wraps the `iou-orchestrator` crate to integrate workflow execution
//! with the API layer. Workflows run asynchronously with status updates broadcast
//! via WebSocket.
//!
//! # Architecture
//!
//! - Workflows execute asynchronously in spawned tasks
//! - 8-minute cumulative timeout enforced across all agents
//! - Status updates broadcast via channels for WebSocket clients
//! - Database updated after each agent completes
//!
//! # Usage
//!
//! ```no_run
//! use crate::orchestrator::WorkflowOrchestrator;
//! use tokio::sync::broadcast;
//!
//! # async fn example(db: std::sync::Arc<crate::db::Database>) -> Result<(), Box<dyn std::error::Error>> {
//! let (status_tx, _) = broadcast::channel(100);
//! let (doc_tx, _) = broadcast::channel(100);
//! let orchestrator = WorkflowOrchestrator::new(db, status_tx, doc_tx);
//! // Use orchestrator to start workflows...
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::Instant;
use uuid::Uuid;

use crate::db::Database;
use crate::orchestrator::types::{StatusMessage, workflow_state_to_document_state, agent_display_name};
use crate::websockets::types::DocumentStatus;
use iou_core::workflows::WorkflowStatus;
use iou_orchestrator::{
    WorkflowStateMachine, WorkflowContext, WorkflowState,
    AgentType, RetryPolicy,
};

/// Overall timeout for the entire workflow (8 minutes cumulative)
const OVERALL_TIMEOUT_MS: u64 = 8 * 60 * 1000;

/// Orchestrator configuration.
///
/// Use [`OrchestratorConfigBuilder`] to construct with custom settings.
#[derive(Clone, Debug)]
pub struct OrchestratorConfig {
    /// Overall timeout for all agents combined (milliseconds)
    pub overall_timeout_ms: u64,
    /// Retry policy for transient errors
    pub retry_policy: RetryPolicy,
    /// Per-agent timeout fallback (milliseconds)
    pub default_agent_timeout_ms: u64,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            overall_timeout_ms: OVERALL_TIMEOUT_MS,
            retry_policy: RetryPolicy::default(),
            default_agent_timeout_ms: 300_000, // 5 minutes per agent
        }
    }
}

/// Builder for [`OrchestratorConfig`].
///
/// # Example
///
/// ```no_run
/// use crate::orchestrator::wrapper::OrchestratorConfig;
///
/// let config = OrchestratorConfig::builder()
///     .with_overall_timeout_ms(600_000)
///     .build();
/// ```
#[derive(Debug)]
pub struct OrchestratorConfigBuilder {
    config: OrchestratorConfig,
}

impl Default for OrchestratorConfigBuilder {
    fn default() -> Self {
        Self {
            config: OrchestratorConfig::default(),
        }
    }
}

impl OrchestratorConfigBuilder {
    /// Create a new builder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the overall timeout for all agents combined.
    pub fn with_overall_timeout_ms(mut self, millis: u64) -> Self {
        self.config.overall_timeout_ms = millis;
        self
    }

    /// Set the retry policy for transient errors.
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.config.retry_policy = policy;
        self
    }

    /// Build the configuration.
    pub fn build(self) -> OrchestratorConfig {
        self.config
    }
}

impl OrchestratorConfig {
    /// Create a new builder for this configuration.
    pub fn builder() -> OrchestratorConfigBuilder {
        OrchestratorConfigBuilder::new()
    }
}

/// Workflow orchestrator.
///
/// Wraps the `iou-orchestrator` state machine to provide async execution
/// with timeout handling and status broadcasting.
///
/// # Errors
///
/// All public methods that can fail return [`Result<T, OrchestratorError>`].
pub struct WorkflowOrchestrator {
    db: Arc<Database>,
    status_tx: broadcast::Sender<StatusMessage>,
    doc_status_tx: broadcast::Sender<DocumentStatus>,
    config: OrchestratorConfig,
}

impl WorkflowOrchestrator {
    /// Create a new workflow orchestrator.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection for persisting state
    /// * `status_tx` - Broadcast channel for orchestrator status messages
    /// * `doc_status_tx` - Broadcast channel for WebSocket document status
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use tokio::sync::broadcast;
    /// # use crate::orchestrator::wrapper::WorkflowOrchestrator;
    /// # use crate::db::Database;
    /// # async fn example(db: Arc<Database>) {
    /// let (status_tx, _) = broadcast::channel(100);
    /// let (doc_tx, _) = broadcast::channel(100);
    /// let orchestrator = WorkflowOrchestrator::new(db, status_tx, doc_tx);
    /// # }
    /// ```
    pub fn new(
        db: Arc<Database>,
        status_tx: broadcast::Sender<StatusMessage>,
        doc_status_tx: broadcast::Sender<DocumentStatus>,
    ) -> Self {
        Self {
            db,
            status_tx,
            doc_status_tx,
            config: OrchestratorConfig::default(),
        }
    }

    /// Create a new orchestrator with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection for persisting state
    /// * `status_tx` - Broadcast channel for orchestrator status messages
    /// * `doc_status_tx` - Broadcast channel for WebSocket document status
    /// * `config` - Custom orchestrator configuration
    pub fn with_config(
        db: Arc<Database>,
        status_tx: broadcast::Sender<StatusMessage>,
        doc_status_tx: broadcast::Sender<DocumentStatus>,
        config: OrchestratorConfig,
    ) -> Self {
        Self {
            db,
            status_tx,
            doc_status_tx,
            config,
        }
    }

    /// Start a workflow for document creation.
    ///
    /// This method starts the workflow asynchronously and returns immediately.
    /// The workflow runs in a spawned task, with status updates broadcast via
    /// the configured channels.
    ///
    /// # Arguments
    ///
    /// * `context` - The workflow context containing document request details
    ///
    /// # Returns
    ///
    /// Returns the document ID if the workflow was started successfully.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError::StateError`] if the workflow cannot be started.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use crate::orchestrator::wrapper::WorkflowOrchestrator;
    /// # use iou_orchestrator::WorkflowContext;
    /// # async fn example(orchestrator: WorkflowOrchestrator, ctx: WorkflowContext) -> Result<(), Box<dyn std::error::Error>> {
    /// let document_id = orchestrator.start_workflow(ctx).await?;
    /// println!("Workflow started for document: {}", document_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_workflow(
        &self,
        context: WorkflowContext,
    ) -> Result<Uuid, OrchestratorError> {
        let document_id = context.id;
        let mut state_machine = WorkflowStateMachine::new(context);

        // Start the workflow
        state_machine.start()
            .map_err(|e| OrchestratorError::StateError(e.to_string()))?;

        // Broadcast started status
        let timestamp = chrono::Utc::now().timestamp();
        let status = StatusMessage::Started {
            document_id,
            agent: "workflow".to_string(),
            timestamp,
        };
        let _ = self.status_tx.send(status.clone());
        let _ = self.doc_status_tx.send(status.to_document_status());

        // Spawn async execution
        let orchestrator = self.clone();
        tokio::spawn(async move {
            orchestrator.execute_workflow(state_machine).await;
        });

        Ok(document_id)
    }

    /// Execute the workflow asynchronously.
    ///
    /// This runs the main workflow loop, executing each agent in sequence
    /// with timeout handling and retry logic.
    async fn execute_workflow(&self, mut state_machine: WorkflowStateMachine) {
        let document_id = state_machine.context.id;
        let started_at = Instant::now();
        const TOTAL_AGENTS: u8 = 4; // Research, Content, Compliance, Review

        loop {
            // Check overall timeout
            let elapsed = started_at.elapsed().as_millis() as u64;
            if elapsed >= self.config.overall_timeout_ms {
                self.fail_workflow(
                    document_id,
                    &format!("Workflow exceeded overall timeout of {}ms", self.config.overall_timeout_ms),
                ).await;
                return;
            }

            // Check if workflow is terminal
            if state_machine.is_terminal() {
                self.handle_terminal_state(&state_machine).await;
                return;
            }

            // Check if awaiting human input
            if state_machine.is_awaiting_human() {
                // Waiting for approval - pause polling
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }

            // Get next agent to execute
            let next_agent = match state_machine.context.next_agent() {
                Some(agent) => agent,
                None => {
                    // All agents complete, transition to completed
                    let _ = state_machine.all_agents_complete();
                    continue;
                }
            };

            // Execute agent with timeout
            match self.execute_agent(&mut state_machine, next_agent).await {
                Ok(_) => {
                    // Broadcast progress
                    let completed_count = state_machine.context.completed_agents.len() as u8;
                    let percent = (completed_count * 100) / TOTAL_AGENTS;

                    let timestamp = chrono::Utc::now().timestamp();
                    let status = StatusMessage::Progress {
                        document_id,
                        agent: agent_display_name(&next_agent).to_string(),
                        percent,
                        timestamp,
                    };

                    let _ = self.status_tx.send(status.clone());
                    let _ = self.doc_status_tx.send(status.to_document_status());

                    // Update database
                    self.update_document_progress(document_id).await;
                }
                Err(e) if e.is_transient() => {
                    // Retry with backoff
                    let retry_count = state_machine.context.get_retry_count(&next_agent);
                    if self.config.retry_policy.can_retry(retry_count) {
                        let backoff = self.config.retry_policy.backoff(retry_count);
                        tokio::time::sleep(backoff).await;
                        state_machine.context.record_agent_failure(next_agent);
                        let _ = state_machine.retry_attempt();
                        continue;
                    } else {
                        // Max retries exceeded
                        self.fail_workflow(
                            document_id,
                            &format!("Agent {:?} failed after {} retries", next_agent, retry_count),
                        ).await;
                        return;
                    }
                }
                Err(e) => {
                    // Permanent error, fail the workflow
                    self.fail_workflow(
                        document_id,
                        &format!("Agent {:?} failed: {}", next_agent, e),
                    ).await;
                    return;
                }
            }
        }
    }

    /// Execute a single agent.
    ///
    /// Currently simulates agent execution. In production, this will call
    /// the actual AI agent services from `iou-ai`.
    async fn execute_agent(
        &self,
        state_machine: &mut WorkflowStateMachine,
        agent: AgentType,
    ) -> Result<(), AgentError> {
        // TODO: Integrate with actual agent execution in future section
        // For now, simulate agent work with a short delay
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Record successful completion
        let result = iou_orchestrator::context::AgentResult::success(
            agent,
            serde_json::json!({ "output": "simulated" }),
            100,
        );
        state_machine.context_mut().record_agent_result(result.clone());

        // Transition state
        let _ = state_machine.agent_complete(agent);

        Ok(())
    }

    /// Handle terminal state (completed/failed/cancelled).
    async fn handle_terminal_state(&self, state_machine: &WorkflowStateMachine) {
        let document_id = state_machine.context.id;
        let timestamp = chrono::Utc::now().timestamp();

        match state_machine.state() {
            WorkflowState::Completed => {
                let status = StatusMessage::Completed { document_id, timestamp };
                let _ = self.status_tx.send(status.clone());
                let _ = self.doc_status_tx.send(status.to_document_status());
                self.update_document_state(document_id, WorkflowStatus::Approved).await;
            }
            WorkflowState::Failed | WorkflowState::Cancelled => {
                let status = StatusMessage::Failed {
                    document_id,
                    error: "Workflow failed or was cancelled".to_string(),
                    timestamp,
                };
                let _ = self.status_tx.send(status.clone());
                let _ = self.doc_status_tx.send(status.to_document_status());
                self.update_document_state(document_id, WorkflowStatus::Rejected).await;
            }
            _ => {}
        }
    }

    /// Fail a workflow with an error message.
    async fn fail_workflow(&self, document_id: Uuid, error: &str) {
        let timestamp = chrono::Utc::now().timestamp();
        let status = StatusMessage::Failed {
            document_id,
            error: error.to_string(),
            timestamp,
        };
        let _ = self.status_tx.send(status.clone());
        let _ = self.doc_status_tx.send(status.to_document_status());
        self.update_document_state(document_id, WorkflowStatus::Rejected).await;
    }

    /// Update document state in database.
    async fn update_document_state(&self, document_id: Uuid, state: WorkflowStatus) {
        if let Ok(Some(mut doc)) = self.db.get_document_async(document_id).await {
            doc.state = state;
            doc.updated_at = chrono::Utc::now();
            let _ = self.db.update_document_async(doc).await;
        }
    }

    /// Update document progress timestamp in database.
    async fn update_document_progress(&self, document_id: Uuid) {
        if let Ok(Some(mut doc)) = self.db.get_document_async(document_id).await {
            doc.updated_at = chrono::Utc::now();
            let _ = self.db.update_document_async(doc).await;
        }
    }
}

impl Clone for WorkflowOrchestrator {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
            status_tx: self.status_tx.clone(),
            doc_status_tx: self.doc_status_tx.clone(),
            config: self.config.clone(),
        }
    }
}

/// Orchestrator error type.
///
/// Represents errors that can occur during workflow orchestration.
#[derive(Debug, thiserror::Error)]
pub enum OrchestratorError {
    /// State machine error.
    #[error("State machine error: {0}")]
    StateError(String),

    /// Agent execution failed.
    #[error("Agent execution failed: {0}")]
    AgentError(String),

    /// Operation timed out.
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Database operation failed.
    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Agent execution error.
///
/// Distinguishes between transient (retryable) and permanent errors.
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    /// Transient error that may be retried.
    #[error("Transient error: {0}")]
    Transient(String),

    /// Permanent error that should not be retried.
    #[error("Permanent error: {0}")]
    Permanent(String),
}

impl AgentError {
    /// Check if this error is transient (retryable).
    fn is_transient(&self) -> bool {
        matches!(self, AgentError::Transient(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use iou_orchestrator::context::{DocumentRequest, DocumentType};

    /// Create a test workflow context.
    fn create_test_context() -> WorkflowContext {
        let request = DocumentRequest {
            id: Uuid::new_v4(),
            domain_id: "test_domain".to_string(),
            document_type: DocumentType::WooBesluit,
            context: HashMap::new(),
            requested_by: Uuid::new_v4(),
            requested_at: chrono::Utc::now(),
            priority: Default::default(),
        };
        WorkflowContext::new(Uuid::new_v4(), request, "1.0.0".to_string())
    }

    #[test]
    fn test_orchestrator_config_default() {
        let config = OrchestratorConfig::default();
        assert_eq!(config.overall_timeout_ms, OVERALL_TIMEOUT_MS);
        assert_eq!(config.default_agent_timeout_ms, 300_000);
    }

    #[test]
    fn test_orchestrator_config_builder() {
        let config = OrchestratorConfig::builder()
            .with_overall_timeout_ms(600_000)
            .build();

        assert_eq!(config.overall_timeout_ms, 600_000);
    }

    #[test]
    fn test_agent_error_transient() {
        let transient = AgentError::Transient("network error".to_string());
        assert!(transient.is_transient());

        let permanent = AgentError::Permanent("invalid data".to_string());
        assert!(!permanent.is_transient());
    }

    #[test]
    fn test_orchestrator_error_display() {
        let err = OrchestratorError::StateError("test".to_string());
        assert!(err.to_string().contains("State machine error"));

        let err = OrchestratorError::AgentError("test".to_string());
        assert!(err.to_string().contains("Agent execution failed"));

        let err = OrchestratorError::Timeout("test".to_string());
        assert!(err.to_string().contains("Timeout"));

        let err = OrchestratorError::DatabaseError("test".to_string());
        assert!(err.to_string().contains("Database error"));
    }
}
