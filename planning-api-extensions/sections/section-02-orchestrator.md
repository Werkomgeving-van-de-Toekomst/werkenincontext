Now I have all the context I need. Let me generate the section content:

# Section 2: Orchestrator Integration

## Overview

This section integrates the existing `iou-orchestrator` crate into the API server to enable asynchronous document generation workflows. The orchestrator coordinates AI agent execution (Research, Content, Compliance, Review) with human-in-the-loop approval points.

**Dependencies:** Section 1 (Foundation & Configuration) must be completed first.

**Files Created:**
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/orchestrator/wrapper.rs` - Orchestrator wrapper and execution logic
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/orchestrator/types.rs` - Type mappings between orchestrator and API types

**Files Modified:**
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/orchestrator/mod.rs` - Module exports
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/documents.rs` - Integrate orchestrator into `create_document()`

## Tests First

Before implementing, write the following tests in `/Users/marc/Projecten/iou-modern/crates/iou-api/src/orchestrator/wrapper.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use iou_orchestrator::{WorkflowContext, WorkflowStateMachine};
    use iou_orchestrator::context::{DocumentRequest, DocumentType};
    use std::collections::HashMap;

    fn create_test_workflow_context() -> WorkflowContext {
        let request = DocumentRequest {
            id: Uuid::new_v4(),
            domain_id: "test_domain".to_string(),
            document_type: DocumentType::WooBesluit,
            context: HashMap::new(),
            requested_by: Uuid::new_v4(),
            requested_at: Utc::now(),
            priority: Default::default(),
        };
        WorkflowContext::new(Uuid::new_v4(), request, "1.0.0".to_string())
    }

    #[tokio::test]
    async fn test_workflow_orchestrator_starts_workflow() {
        let context = create_test_workflow_context();
        let status_tx = tokio::sync::broadcast::channel(100).0;
        let db = Arc::new(mock_database());
        
        let orchestrator = WorkflowOrchestrator::new(db, status_tx);
        
        let result = orchestrator.start_workflow(context).await;
        assert!(result.is_ok());
        
        let state_machine = result.unwrap();
        assert_eq!(state_machine.state(), &iou_orchestrator::WorkflowState::Running);
    }

    #[tokio::test]
    async fn test_workflow_orchestrator_transitions_on_agent_complete() {
        // Test that state transitions correctly when an agent completes
    }

    #[tokio::test]
    async fn test_workflow_orchestrator_applies_overall_timeout() {
        // Test that 8-minute cumulative timeout is enforced
    }

    #[tokio::test]
    async fn test_workflow_orchestrator_handles_agent_failure() {
        // Test failure handling and state transition
    }

    #[tokio::test]
    async fn test_workflow_orchestrator_retries_transient_errors() {
        // Test retry with exponential backoff
    }

    #[tokio::test]
    async fn test_idempotency_duplicate_document_id() {
        // Test that duplicate document_id returns existing document
    }

    #[tokio::test]
    async fn test_idempotency_unique_document_id() {
        // Test that unique document_id creates new document
    }

    #[tokio::test]
    async fn test_workflow_state_to_document_state_mapping() {
        // Test all WorkflowState variants map correctly
    }
}
```

## Implementation

### 1. Create `/Users/marc/Projecten/iou-modern/crates/iou-api/src/orchestrator/mod.rs`

```rust
//! Orchestrator integration module
//!
//! Integrates the iou-orchestrator crate for async document generation workflows.

pub mod wrapper;
pub mod types;

pub use wrapper::{WorkflowOrchestrator, OrchestratorConfig};
pub use types::{StatusMessage, workflow_state_to_document_state};
```

### 2. Create `/Users/marc/Projecten/iou-modern/crates/iou-api/src/orchestrator/types.rs`

```rust
//! Type mappings between orchestrator and API types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use iou_core::workflows::WorkflowStatus as DocumentState;
use iou_orchestrator::{WorkflowState, AgentType};

/// Status message for WebSocket broadcast
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum StatusMessage {
    #[serde(rename = "started")]
    Started { document_id: Uuid, agent: String },
    
    #[serde(rename = "progress")]
    Progress { document_id: Uuid, agent: String, percent: u8 },
    
    #[serde(rename = "completed")]
    Completed { document_id: Uuid },
    
    #[serde(rename = "failed")]
    Failed { document_id: Uuid, error: String },
}

impl StatusMessage {
    pub fn document_id(&self) -> Uuid {
        match self {
            StatusMessage::Started { document_id, .. } => *document_id,
            StatusMessage::Progress { document_id, .. } => *document_id,
            StatusMessage::Completed { document_id } => *document_id,
            StatusMessage::Failed { document_id, .. } => *document_id,
        }
    }
}

/// Convert orchestrator WorkflowState to DocumentState
pub fn workflow_state_to_document_state(workflow_state: &WorkflowState) -> DocumentState {
    match workflow_state {
        WorkflowState::Created => DocumentState::Draft,
        WorkflowState::Running => DocumentState::InReview,
        WorkflowState::AwaitingApproval => DocumentState::InReview,
        WorkflowState::AwaitingEscalation => DocumentState::InReview,
        WorkflowState::Completed => DocumentState::Approved,
        WorkflowState::Failed => DocumentState::Rejected,
        WorkflowState::Cancelled => DocumentState::Rejected,
        WorkflowState::Retrying => DocumentState::InReview,
        WorkflowState::Archived => DocumentState::Archived,
    }
}

/// Agent display name for status messages
pub fn agent_display_name(agent: &AgentType) -> &'static str {
    match agent {
        AgentType::Research => "onderzoeksagent",
        AgentType::Content => "contentagent",
        AgentType::Compliance => "complianceagent",
        AgentType::Review => "reviewagent",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_state_mapping() {
        assert_eq!(
            workflow_state_to_document_state(&WorkflowState::Created),
            DocumentState::Draft
        );
        assert_eq!(
            workflow_state_to_document_state(&WorkflowState::Running),
            DocumentState::InReview
        );
        assert_eq!(
            workflow_state_to_document_state(&WorkflowState::Completed),
            DocumentState::Approved
        );
        assert_eq!(
            workflow_state_to_document_state(&WorkflowState::Failed),
            DocumentState::Rejected
        );
    }

    #[test]
    fn test_status_message_document_id() {
        let id = Uuid::new_v4();
        
        let msg = StatusMessage::Started { document_id: id, agent: "test".to_string() };
        assert_eq!(msg.document_id(), id);
        
        let msg = StatusMessage::Progress { document_id: id, agent: "test".to_string(), percent: 50 };
        assert_eq!(msg.document_id(), id);
        
        let msg = StatusMessage::Completed { document_id: id };
        assert_eq!(msg.document_id(), id);
        
        let msg = StatusMessage::Failed { document_id: id, error: "test".to_string() };
        assert_eq!(msg.document_id(), id);
    }
}
```

### 3. Create `/Users/marc/Projecten/iou-modern/crates/iou-api/src/orchestrator/wrapper.rs`

```rust
//! Workflow orchestrator wrapper
//!
//! Wraps the iou-orchestrator crate for use in the API server.
//! Handles async execution, timeouts, and status broadcasting.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::{timeout, Instant};
use uuid::Uuid;

use crate::db::Database;
use crate::orchestrator::types::{StatusMessage, workflow_state_to_document_state, agent_display_name};
use iou_core::workflows::WorkflowStatus;
use iou_orchestrator::{
    WorkflowStateMachine, WorkflowContext, WorkflowState, WorkflowEvent,
    AgentType, OrchestratorConfig, RetryPolicy,
};

/// Overall timeout for the entire workflow (8 minutes cumulative)
const OVERALL_TIMEOUT_MS: u64 = 8 * 60 * 1000;

/// Orchestrator configuration
#[derive(Clone)]
pub struct OrchestratorConfig {
    /// Overall timeout for all agents combined
    pub overall_timeout_ms: u64,
    /// Retry policy for transient errors
    pub retry_policy: RetryPolicy,
    /// Per-agent timeout (fallback)
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

/// Active workflow tracking
#[derive(Debug)]
struct ActiveWorkflow {
    document_id: Uuid,
    started_at: Instant,
    state_machine: WorkflowStateMachine,
}

/// Workflow orchestrator
pub struct WorkflowOrchestrator {
    db: Arc<Database>,
    status_tx: broadcast::Sender<StatusMessage>,
    config: OrchestratorConfig,
}

impl WorkflowOrchestrator {
    /// Create a new workflow orchestrator
    pub fn new(
        db: Arc<Database>,
        status_tx: broadcast::Sender<StatusMessage>,
    ) -> Self {
        Self {
            db,
            status_tx,
            config: OrchestratorConfig::default(),
        }
    }

    /// Start a workflow for document creation
    pub async fn start_workflow(
        &self,
        context: WorkflowContext,
    ) -> Result<WorkflowStateMachine, OrchestratorError> {
        let document_id = context.id;
        let mut state_machine = WorkflowStateMachine::new(context);
        
        // Start the workflow
        state_machine.start()
            .map_err(|e| OrchestratorError::StateError(e.to_string()))?;
        
        // Broadcast started status
        let _ = self.status_tx.send(StatusMessage::Started {
            document_id,
            agent: "workflow".to_string(),
        });
        
        // Spawn async execution
        let orchestrator = self.clone();
        tokio::spawn(async move {
            orchestrator.execute_workflow(state_machine).await;
        });
        
        Ok(state_machine)
    }

    /// Execute the workflow asynchronously
    async fn execute_workflow(&self, mut state_machine: WorkflowStateMachine) {
        let document_id = state_machine.context.id;
        let started_at = Instant::now();
        let mut cumulative_time = 0;
        
        loop {
            // Check overall timeout
            let elapsed = started_at.elapsed().as_millis() as u64;
            if elapsed >= self.config.overall_timeout_ms {
                let _ = state_machine.cancel();
                let _ = self.status_tx.send(StatusMessage::Failed {
                    document_id,
                    error: format!("Workflow exceeded overall timeout of {}ms", self.config.overall_timeout_ms),
                });
                self.update_document_state(document_id, WorkflowStatus::Rejected).await;
                return;
            }
            
            // Check if workflow is complete
            if state_machine.is_terminal() {
                match state_machine.state() {
                    WorkflowState::Completed => {
                        let _ = self.status_tx.send(StatusMessage::Completed { document_id });
                        self.update_document_state(document_id, WorkflowStatus::Approved).await;
                    }
                    WorkflowState::Failed | WorkflowState::Cancelled => {
                        let _ = self.status_tx.send(StatusMessage::Failed {
                            document_id,
                            error: "Workflow failed or was cancelled".to_string(),
                        });
                        self.update_document_state(document_id, WorkflowStatus::Rejected).await;
                    }
                    _ => {}
                }
                return;
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
            let agent_start = Instant::now();
            match self.execute_agent(&mut state_machine, next_agent).await {
                Ok(_) => {
                    let elapsed_ms = agent_start.elapsed().as_millis() as u64;
                    cumulative_time += elapsed_ms;
                    
                    // Broadcast progress
                    let completed_count = state_machine.context.completed_agents.len() as u8;
                    let total_agents = 4; // Research, Content, Compliance, Review
                    let percent = (completed_count * 100) / total_agents;
                    
                    let _ = self.status_tx.send(StatusMessage::Progress {
                        document_id,
                        agent: agent_display_name(&next_agent).to_string(),
                        percent,
                    });
                    
                    // Update database with progress
                    self.update_document_progress(
                        document_id,
                        workflow_state_to_document_state(state_machine.state()),
                    ).await;
                }
                Err(e) if e.is_transient() => {
                    // Retry with backoff
                    let retry_count = state_machine.context.get_retry_count(&next_agent);
                    if self.config.retry_policy.can_retry(retry_count) {
                        let backoff = self.config.retry_policy.backoff(retry_count);
                        tokio::time::sleep(backoff).await;
                        let _ = state_machine.retry_attempt();
                        continue;
                    } else {
                        // Max retries exceeded
                        let _ = state_machine.max_retries_exceeded();
                        let _ = self.status_tx.send(StatusMessage::Failed {
                            document_id,
                            error: format!("Agent {:?} failed after {} retries", next_agent, retry_count),
                        });
                        return;
                    }
                }
                Err(e) => {
                    // Permanent error, fail the workflow
                    let _ = state_machine.agent_failed();
                    let _ = self.status_tx.send(StatusMessage::Failed {
                        document_id,
                        error: format!("Agent {:?} failed: {}", next_agent, e),
                    });
                    return;
                }
            }
        }
    }

    /// Execute a single agent
    async fn execute_agent(
        &self,
        state_machine: &mut WorkflowStateMachine,
        agent: AgentType,
    ) -> Result<(), AgentError> {
        // TODO: Integrate with actual agent execution
        // For now, this is a placeholder that simulates agent execution
        
        // In the full implementation, this would:
        // 1. Call the appropriate AI agent service
        // 2. Handle the agent's response
        // 3. Record the result in the workflow context
        // 4. Transition the state machine based on the result
        
        // Simulate agent work
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

    /// Update document state in database
    async fn update_document_state(&self, document_id: Uuid, state: WorkflowStatus) {
        if let Ok(Some(mut doc)) = self.db.get_document_async(document_id).await {
            doc.state = state;
            doc.updated_at = chrono::Utc::now();
            let _ = self.db.update_document_async(doc).await;
        }
    }

    /// Update document progress in database
    async fn update_document_progress(&self, document_id: Uuid, state: WorkflowStatus) {
        if let Ok(Some(mut doc)) = self.db.get_document_async(document_id).await {
            doc.state = state;
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
            config: self.config.clone(),
        }
    }
}

/// Orchestrator error type
#[derive(Debug, thiserror::Error)]
pub enum OrchestratorError {
    #[error("State machine error: {0}")]
    StateError(String),
    
    #[error("Agent execution failed: {0}")]
    AgentError(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Agent execution error
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Transient error: {0}")]
    Transient(String),
    
    #[error("Permanent error: {0}")]
    Permanent(String),
}

impl AgentError {
    fn is_transient(&self) -> bool {
        matches!(self, AgentError::Transient(_))
    }
}
```

### 4. Update `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/documents.rs`

Modify the `create_document` function to integrate the orchestrator:

```rust
use crate::orchestrator::{WorkflowOrchestrator, OrchestratorConfig};
use tokio::sync::broadcast;
use iou_orchestrator::{
    WorkflowContext, AgentType,
    context::{DocumentRequest, DocumentType, RequestPriority},
};

// In create_document function:
pub async fn create_document(
    Extension(db): Extension<Arc<Database>>,
    Extension(status_tx): Extension<broadcast::Sender<StatusMessage>>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<CreateDocumentResponse>, ApiError> {
    // Validate template exists
    let _template = db
        .get_active_template_async(req.domain_id.clone(), req.document_type.clone())
        .await?
        .ok_or_else(|| ApiError::Validation(format!(
            "No template found for type '{}' in domain '{}'",
            req.document_type, req.domain_id
        )))?;

    let document_id = Uuid::new_v4();
    let now = Utc::now();

    // Check for idempotency - if document_id was provided in request
    // and already exists, return the existing document
    // (For now, we always generate new IDs)

    // Create document request for orchestrator
    let document_request = DocumentRequest {
        id: document_id,
        domain_id: req.domain_id.clone(),
        document_type: parse_document_type(&req.document_type)?,
        context: req.context,
        requested_by: Uuid::new_v4(), // TODO: Get from auth context
        requested_at: now,
        priority: RequestPriority::Normal,
    };

    // Create workflow context
    let workflow_context = WorkflowContext::new(
        document_id,
        document_request,
        "1.0.0".to_string(),
    );

    // Create document record in database
    let document = DocumentMetadata {
        id: document_id,
        domain_id: req.domain_id.clone(),
        document_type: req.document_type.clone(),
        state: WorkflowStatus::Draft,
        current_version_key: String::new(),
        previous_version_key: None,
        compliance_score: 0.0,
        confidence_score: 0.0,
        created_at: now,
        updated_at: now,
    };

    db.create_document_async(document).await?;

    // Create and start workflow orchestrator
    let orchestrator = WorkflowOrchestrator::new(db.clone(), status_tx);
    orchestrator.start_workflow(workflow_context).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to start workflow: {}", e)))?;

    tracing::info!(
        domain_id = %req.domain_id,
        document_type = %req.document_type,
        document_id = %document_id,
        "Document creation requested and workflow started"
    );

    // Calculate estimated completion time (8 minutes max)
    let estimated_completion = now + chrono::Duration::minutes(8);

    Ok(Json(CreateDocumentResponse {
        document_id,
        state: "draft".to_string(),
        estimated_completion: Some(estimated_completion),
    }))
}

/// Helper to parse document type from string
fn parse_document_type(s: &str) -> Result<DocumentType, ApiError> {
    match s.to_lowercase().as_str() {
        "woo_besluit" => Ok(DocumentType::WooBesluit),
        "woo_informatie" => Ok(DocumentType::WooInformatie),
        "woo_besluit_beroep" => Ok(DocumentType::WooBesluitBeroep),
        "woo_informatie_beroep" => Ok(DocumentType::WooInformatieBeroep),
        "beleidsnotitie" => Ok(DocumentType::Beleidsnotitie),
        "raadsvoorstel" => Ok(DocumentType::Raadsvoorstel),
        "ambtsbericht" => Ok(DocumentType::Ambtsbericht),
        "persbericht" => Ok(DocumentType::Persbericht),
        "interne_memo" => Ok(DocumentType::InterneMemo),
        _ => Ok(DocumentType::Custom(s.to_string())),
    }
}
```

### 5. Update `/Users/marc/Projecten/iou-modern/crates/iou-api/src/main.rs`

Add the orchestrator state to the application:

```rust
use tokio::sync::broadcast;

// In main function, after initializing the database:
let db_arc = Arc::new(db);
let workflow_engine = Arc::new(WorkflowEngine::new(db_arc.clone()));

// Create broadcast channel for status updates
let (status_tx, _rx) = broadcast::channel(100);

// Extend the app with the status channel
let app = Router::new()
    // ... existing routes ...
    .layer(Extension(db_arc))
    .layer(Extension(workflow_engine))
    .layer(Extension(status_tx));  // NEW
```

## Key Implementation Notes

1. **Overall Timeout (8 minutes)**: The cumulative execution time of all agents is tracked. If the total exceeds 8 minutes, the workflow is cancelled and a failure status is broadcast.

2. **Idempotency**: When a document is created, check if a document with the same ID already exists in the database. If so, return the existing document without starting a new workflow.

3. **State Mapping**: The `workflow_state_to_document_state()` function maps orchestrator states to document states:
   - `Created` -> `Draft`
   - `Running/Retrying` -> `InReview`
   - `AwaitingApproval/AwaitingEscalation` -> `InReview`
   - `Completed` -> `Approved`
   - `Failed/Cancelled` -> `Rejected`
   - `Archived` -> `Archived`

4. **Broadcast Channel**: Status updates are sent via a bounded broadcast channel (capacity 100). WebSocket clients can subscribe to receive real-time updates (implemented in Section 5).

5. **Database Updates**: After each agent completes, the document state is updated in the database to reflect current progress.

6. **Async Execution**: The workflow runs asynchronously in a spawned task. The API returns immediately after starting the workflow.

7. **Agent Execution**: The current implementation simulates agent execution. In production, this will integrate with the actual AI agent services from `iou-ai`.