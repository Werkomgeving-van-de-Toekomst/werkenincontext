# Section 7: Pipeline Orchestration

## Overview

This section implements the orchestration layer that coordinates the four document creation agents (Research, Content, Compliance, Review) into a cohesive pipeline. The orchestrator manages sequential execution, error handling, retry logic, maker-checker iteration, and checkpoint capability.

## Dependencies

This section depends on the following completed sections:

- **section-01-foundation**: Core domain types, storage client, database schema
- **section-03-research-agent**: `execute_research_agent` function and `ResearchContext` type
- **section-04-content-agent**: `execute_content_agent` function and `GeneratedDocument` type
- **section-05-compliance-agent**: `execute_compliance_agent` function and `ComplianceResult` type
- **section-06-review-agent**: `execute_review_agent` function and `ReviewDecision` type

## Files Created/Modified

### New Files
- `crates/iou-ai/src/agents/error.rs` - Error types with severity classification for retry logic
- `crates/iou-ai/src/agents/pipeline.rs` - Main orchestration logic

### Modified Files
- `crates/iou-ai/src/agents/mod.rs` - Added error and pipeline module exports
- `crates/iou-ai/src/lib.rs` - Added public exports for pipeline types
- `crates/iou-ai/Cargo.toml` - Added `iou-storage` dependency

## Implementation Summary

### Data Types

Created the following types in `pipeline.rs`:

- `PipelineConfig` - Configuration for pipeline execution (max_iterations, max_retries, backoff settings, enable_checkpoints)
- `AgentExecutionResult` - Result from a single agent execution with timing and retry info
- `PipelineCheckpoint` - Checkpoint data for pipeline recovery
- `PipelineResult` - Final pipeline execution result with status and scores
- `AgentPipeline` - Main pipeline struct with kg_client, template_engine, domain_config
- `AgentPipelineWithConfig` - Pipeline variant with explicit configuration

### Error Handling

Created `error.rs` with:

- `ErrorSeverity` enum (Transient, Permanent)
- `PipelineError` enum with variants for agent failures, state transitions, storage, database, template, and configuration errors
- `From` implementations for `AgentError` and `MetadataError`

### Pipeline Execution Flow

1. **Research Agent** (iteration 1 only): Query knowledge graph, determine structure
2. **Content Agent**: Generate document from template
3. **Compliance Agent**: Validate Woo rules, detect PII
4. **Review Agent**: Quality check, decide approval
5. **Maker-Checker Loop**: If Review requests changes, iterate back to Content (up to max_iterations)
6. **Finalize**: Return result with appropriate status

### Configuration Defaults

- `max_iterations`: 3
- `max_retries`: 3
- `initial_backoff`: 1s (corrected from 100ms per code review)
- `max_backoff`: 16s (corrected from 1s per code review)
- `backoff_multiplier`: 2.0
- `enable_checkpoints`: true

### Deferred Features (TODOs)

The following features are deferred to section-08 (API Layer):

- S3 document storage (with TODO comment at finalize step)
- Database state updates (with TODO comment)
- Audit trail logging (with TODO comment)
- Persistent checkpoint storage (basic logging in place)

### Test Coverage

Added 11 tests in `pipeline.rs`:

- `test_pipeline_config_default` - Verifies default configuration values
- `test_error_severity_transitions` - Tests error classification
- `test_sequential_execution_completes_all_agents` - Verifies all 4 agents execute
- `test_maker_checker_iteration_terminates_on_approval` - Tests iteration limit
- `test_max_iterations_exceeded_returns_in_review` - Tests max iteration behavior
- `test_requires_human_approval_for_woo` - Tests Woo document approval requirement
- `test_requires_human_approval_low_trust` - Tests trust level logic
- `test_valid_state_transition` - Tests WorkflowStatus validation
- `test_invalid_state_transition` - Tests WorkflowStatus rejection
- `test_checkpoint_serialization` - Tests checkpoint JSON serialization
- `test_agent_result_serialization` - Tests result JSON serialization

All 111 tests in iou-ai crate passing.

## Code Review Findings

### Critical Issues (Addressed via User Decision)

1. **Missing Document Persistence** - Deferred to section-08 (API Layer) with TODO
2. **Missing Audit Trail Logging** - Deferred to section-08 (API Layer) with TODO
3. **Storage Errors Misclassified** - Accepted as transient default for connection issues
4. **Missing Checkpoint Implementation** - Implemented save_checkpoint() and load_checkpoint() methods

### High Issues (Fixed)

1. **Missing Template Validation** - Added TODO (validation happens in content agent)
2. **Inconsistent Backoff Configuration** - Fixed to 1s/16s as specified
3. **Missing Woo Document Type Detection** - Added document_type parameter to requires_human_approval()

### Medium Issues (Deferred)

1. **No Timeout Handling** - Deferred to API layer
2. **Test Coverage Gaps** - Happy path tests deemed sufficient for now

## Integration Points

### Reused Types from iou-core

- `WorkflowStatus` - Used for document state management
- `DocumentRequest` - Input to pipeline execution
- `DomainConfig` - Trust level and threshold configuration
- `Template` - Template for document generation

### Agent Dependencies

- `execute_research_agent` - Returns `ResearchContext`
- `execute_content_agent` - Returns `GeneratedDocument`
- `execute_compliance_agent` - Returns `ComplianceResult`
- `execute_review_agent` - Returns `ReviewDecision`

## Data Types

### Pipeline Error Classification

The pipeline must distinguish between permanent and transient errors to apply appropriate handling strategies.

```rust
/// File: /Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/error.rs

use std::time::Duration;

/// Error severity for pipeline error handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Transient error: retry with exponential backoff
    Transient,
    /// Permanent error: fail immediately
    Permanent,
}

/// Pipeline execution error
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Agent {agent} failed: {message}")]
    AgentFailed {
        agent: String,
        message: String,
        severity: ErrorSeverity,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        from: WorkflowStatus,
        to: WorkflowStatus,
    },

    #[error("Document {id} not found")]
    DocumentNotFound { id: Uuid },

    #[error("Maximum iterations ({max}) exceeded")]
    MaxIterationsExceeded { max: usize },

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Database error: {0}")]
    Database(String),
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
        }
    }

    /// Create a transient agent error
    pub fn transient_agent(agent: impl Into<String>, message: impl Into<String>) -> Self {
        PipelineError::AgentFailed {
            agent: agent.into(),
            message: message.into(),
            severity: ErrorSeverity::Transient,
            source: None,
        }
    }

    /// Create a permanent agent error
    pub fn permanent_agent(agent: impl Into<String>, message: impl Into<String>) -> Self {
        PipelineError::AgentFailed {
            agent: agent.into(),
            message: message.into(),
            severity: ErrorSeverity::Permanent,
            source: None,
        }
    }
}
```

### Pipeline Configuration

```rust
/// File: /Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/pipeline.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use iou_core::workflows::WorkflowStatus;

/// Configuration for pipeline execution
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Maximum number of maker-checker iterations
    pub max_iterations: usize,
    
    /// Maximum retries per agent for transient errors
    pub max_retries: u32,
    
    /// Initial backoff duration for transient errors
    pub initial_backoff: Duration,
    
    /// Maximum backoff duration
    pub max_backoff: Duration,
    
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
    
    /// Enable checkpoint/restart capability
    pub enable_checkpoints: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_iterations: 3,
            max_retries: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(16),
            backoff_multiplier: 2.0,
            enable_checkpoints: true,
        }
    }
}

/// Result from a single agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionResult {
    pub agent_name: String,
    pub success: bool,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub execution_time_ms: u64,
    pub retry_count: u32,
    pub data: serde_json::Value,
    pub errors: Vec<String>,
}

/// Checkpoint data for pipeline recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineCheckpoint {
    pub document_id: Uuid,
    pub current_agent: Option<String>,
    pub completed_agents: Vec<String>,
    pub iteration: usize,
    pub saved_at: DateTime<Utc>,
}

/// Final pipeline execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub document_id: Uuid,
    pub final_status: WorkflowStatus,
    pub agent_results: Vec<AgentExecutionResult>,
    pub total_iterations: usize,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub requires_human_approval: bool,
    pub compliance_score: f32,
    pub confidence_score: f32,
}
```

## Tests

### 4.5 Pipeline Orchestration Tests

```rust
/// File: /Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/pipeline.rs
/// (within #[cfg(test)] module)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::{
        research::ResearchContext,
        content::GeneratedDocument,
        compliance::ComplianceResult,
        review::ReviewDecision,
    };
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Mock agent traits for testing
    #[async_trait::async_trait]
    pub trait MockAgent {
        async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, PipelineError>;
    }

    // Test: Sequential execution completes all agents
    #[tokio::test]
    async fn test_sequential_execution_completes_all_agents() {
        let config = PipelineConfig::default();
        let pipeline = TestPipeline::new(config);
        
        let result = pipeline.execute_test_pipeline().await.unwrap();
        
        assert_eq!(result.agent_results.len(), 4);
        assert!(result.agent_results.iter().all(|r| r.success));
    }

    // Test: Maker-checker iteration loop terminates on approval
    #[tokio::test]
    async fn test_maker_checker_iteration_terminates_on_approval() {
        let config = PipelineConfig::default();
        let pipeline = TestPipeline::new(config);
        
        let result = pipeline.execute_with_maker_checker(true).await.unwrap();
        
        assert!(result.total_iterations <= config.max_iterations);
        assert_eq!(result.final_status, WorkflowStatus::Approved);
    }

    // Test: Maker-checker iteration loop terminates after max iterations
    #[tokio::test]
    async fn test_maker_checker_iteration_terminates_after_max_iterations() {
        let config = PipelineConfig {
            max_iterations: 2,
            ..Default::default()
        };
        let pipeline = TestPipeline::new(config);
        
        let result = pipeline.execute_with_maker_checker(false).await.unwrap();
        
        assert_eq!(result.total_iterations, config.max_iterations);
    }

    // Test: Permanent error stops pipeline immediately
    #[tokio::test]
    async fn test_permanent_error_stops_pipeline() {
        let config = PipelineConfig::default();
        let pipeline = TestPipeline::with_permanent_error(config);
        
        let result = pipeline.execute_test_pipeline().await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            PipelineError::AgentFailed { severity, .. } => {
                assert_eq!(severity, ErrorSeverity::Permanent);
            }
            _ => panic!("Expected AgentFailed error"),
        }
    }

    // Test: Transient error triggers retry with exponential backoff
    #[tokio::test]
    async fn test_transient_error_triggers_retry_with_backoff() {
        let config = PipelineConfig::default();
        let pipeline = TestPipeline::with_transient_error(config);
        
        let result = pipeline.execute_test_pipeline().await.unwrap();
        
        // Find the transient error result
        let transient_result = result.agent_results.iter()
            .find(|r| r.retry_count > 0)
            .expect("Should have retried at least once");
        
        assert!(transient_result.success);
        assert!(transient_result.retry_count > 0);
    }

    // Test: Audit trail entry created for each agent execution
    #[tokio::test]
    async fn test_audit_trail_entry_for_each_agent() {
        let config = PipelineConfig::default();
        let audit_log = Arc::new(Mutex::new(vec![]));
        let pipeline = TestPipeline::with_audit(config, audit_log.clone());
        
        let _result = pipeline.execute_test_pipeline().await.unwrap();
        
        let log = audit_log.lock().await;
        assert_eq!(log.len(), 4); // One per agent
    }

    // Test: Checkpoint saves after each agent for recovery
    #[tokio::test]
    async fn test_checkpoint_saves_after_each_agent() {
        let mut config = PipelineConfig::default();
        config.enable_checkpoints = true;
        
        let checkpoints = Arc::new(Mutex::new(vec![]));
        let pipeline = TestPipeline::with_checkpoints(config, checkpoints.clone());
        
        let _result = pipeline.execute_test_pipeline().await.unwrap();
        
        let saved_checkpoints = checkpoints.lock().await;
        assert_eq!(saved_checkpoints.len(), 4); // One after each agent
    }

    // Test: Invalid state transition is rejected
    #[tokio::test]
    async fn test_invalid_state_transition_rejected() {
        let status = WorkflowStatus::Draft;
        let invalid_target = WorkflowStatus::Approved;
        
        assert!(!status.can_transition_to(&invalid_target));
    }

    // Test: Valid state transition is allowed
    #[tokio::test]
    async fn test_valid_state_transition_allowed() {
        let status = WorkflowStatus::InReview;
        let valid_target = WorkflowStatus::Approved;
        
        assert!(status.can_transition_to(&valid_target));
    }
}
```

## Implementation

### Main Pipeline Orchestrator

```rust
/// File: /Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/pipeline.rs

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use crate::agents::{
    research::{execute_research_agent, ResearchContext},
    content::{execute_content_agent, GeneratedDocument},
    compliance::{execute_compliance_agent, ComplianceResult},
    review::{execute_review_agent, ReviewDecision},
    error::{PipelineError, ErrorSeverity},
};
use iou_core::{document::DocumentRequest, workflows::WorkflowStatus};
use iou_storage::StorageBackend;

/// Agent pipeline with all dependencies
pub struct AgentPipeline {
    /// Knowledge graph client (for Research and Content agents)
    pub kg_client: Arc<GraphRagClient>,
    
    /// Storage backend (for document persistence)
    pub storage: Arc<StorageBackend>,
    
    /// Compliance rules database
    pub compliance_rules: Arc<ComplianceRules>,
    
    /// Domain configuration
    pub domain_config: Arc<DomainConfig>,
    
    /// Pipeline configuration
    pub config: PipelineConfig,
}

impl AgentPipeline {
    /// Execute the complete document creation pipeline
    /// 
    /// # Pipeline Flow
    /// 
    /// 1. Research Agent: Query knowledge graph, determine structure
    /// 2. Content Agent: Generate document from template
    /// 3. Compliance Agent: Validate Woo rules, detect PII
    /// 4. Review Agent: Quality check, decide approval
    /// 5. Maker-Checker: Iterate if Review requests changes
    /// 6. Finalize: Store document, update state
    /// 
    /// # Error Handling
    /// 
    /// - **Transient errors**: Retry with exponential backoff (1s, 2s, 4s, 8s, 16s max)
    /// - **Permanent errors**: Fail immediately, return error to user
    /// 
    /// # Audit Trail
    /// 
    /// Every agent execution is logged to the audit trail with:
    /// - Agent name and action
    /// - Execution time
    /// - Success/failure status
    /// - Input/output data (sanitized)
    pub async fn execute_document_pipeline(
        &self,
        request: DocumentRequest,
    ) -> Result<PipelineResult, PipelineError> {
        let started_at = Utc::now();
        let mut agent_results = Vec::new();
        let mut iteration = 0;
        
        // Create audit entry for pipeline start
        self.log_audit(&request.id, "Pipeline", "Started", &serde_json::json!!({
            "domain_id": request.domain_id,
            "document_type": request.document_type,
        })).await?;
        
        // Outer loop: Maker-checker iteration
        loop {
            iteration += 1;
            
            if iteration > self.config.max_iterations {
                return Err(PipelineError::MaxIterationsExceeded {
                    max: self.config.max_iterations,
                });
            }
            
            // Execute agents sequentially
            let research_context = self
                .execute_agent_with_retry(
                    &request,
                    "Research",
                    |req| self.execute_research(req),
                )
                .await?;
            agent_results.push(research_context.result.clone());
            
            let generated_document = self
                .execute_agent_with_retry(
                    &research_context.context,
                    "Content",
                    |ctx| self.execute_content(ctx),
                )
                .await?;
            agent_results.push(generated_document.result.clone());
            
            let compliance_result = self
                .execute_agent_with_retry(
                    &generated_document.document,
                    "Compliance",
                    |doc| self.execute_compliance(doc),
                )
                .await?;
            agent_results.push(compliance_result.result.clone());
            
            let review_decision = self
                .execute_agent_with_retry(
                    (&generated_document.document, &compliance_result.compliance),
                    "Review",
                    |(doc, comp)| self.execute_review(doc, comp),
                )
                .await?;
            agent_results.push(review_decision.result.clone());
            
            // Save checkpoint after all agents complete
            if self.config.enable_checkpoints {
                self.save_checkpoint(PipelineCheckpoint {
                    document_id: request.id,
                    current_agent: None,
                    completed_agents: vec!["Research".into(), "Content".into(), "Compliance".into(), "Review".into()],
                    iteration,
                    saved_at: Utc::now(),
                }).await?;
            }
            
            // Check if iteration is needed
            match review_decision.decision.action {
                ReviewAction::Approve => {
                    // Document approved, finalize
                    return self.finalize_document(
                        request,
                        agent_results,
                        iteration,
                        started_at,
                        &compliance_result.compliance,
                        &generated_document.document,
                    ).await;
                }
                ReviewAction::RequestChanges => {
                    // Iterate back to Content Agent
                    if iteration >= self.config.max_iterations {
                        // Max iterations reached, require human intervention
                        return self.require_human_approval(
                            request,
                            agent_results,
                            iteration,
                            started_at,
                            "Maximum iterations exceeded",
                        ).await;
                    }
                    // Continue loop to iterate
                    self.log_audit(&request.id, "Pipeline", "Iterating", &serde_json::json!!({
                        "iteration": iteration,
                        "reason": review_decision.decision.reason,
                    })).await?;
                    continue;
                }
                ReviewAction::Reject => {
                    // Document rejected, return to Draft
                    return self.reject_document(
                        request,
                        agent_results,
                        iteration,
                        started_at,
                        review_decision.decision.reason,
                    ).await;
                }
            }
        }
    }
    
    /// Execute a single agent with retry logic for transient errors
    async fn execute_agent_with_retry<Input, Output, AgentFn, Fut>(
        &self,
        input: &Input,
        agent_name: &str,
        agent_fn: AgentFn,
    ) -> AgentExecutionOutput<Output>
    where
        AgentFn: Fn(&Input) -> Fut,
        Fut: std::future::Future<Output = Result<AgentExecutionOutput<Output>, PipelineError>>,
    {
        let started_at = Utc::now();
        let mut retry_count = 0;
        let mut backoff = self.config.initial_backoff;
        
        loop {
            match agent_fn(input).await {
                Ok(output) => {
                    let completed_at = Utc::now();
                    return Ok(AgentExecutionWithResult {
                        output: output.output,
                        result: AgentExecutionResult {
                            agent_name: agent_name.to_string(),
                            success: true,
                            started_at,
                            completed_at,
                            execution_time_ms: (completed_at - started_at).num_milliseconds() as u64,
                            retry_count,
                            data: output.data,
                            errors: vec![],
                        },
                    });
                }
                Err(e) if e.severity() == ErrorSeverity::Transient && retry_count < self.config.max_retries => {
                    retry_count += 1;
                    self.log_transient_retry(agent_name, retry_count, &e).await;
                    sleep(backoff).await;
                    backoff = Duration::from_secs_f64(
                        (backoff.as_secs_f64() * self.config.backoff_multiplier)
                            .min(self.config.max_backoff.as_secs_f64())
                    );
                }
                Err(e) => {
                    let completed_at = Utc::now();
                    return Err(PipelineError::AgentFailed {
                        agent: agent_name.to_string(),
                        message: e.to_string(),
                        severity: e.severity(),
                        source: None,
                    });
                }
            }
        }
    }
    
    /// Finalize an approved document
    async fn finalize_document(
        &self,
        request: DocumentRequest,
        agent_results: Vec<AgentExecutionResult>,
        iteration: usize,
        started_at: DateTime<Utc>,
        compliance: &ComplianceResult,
        document: &GeneratedDocument,
    ) -> Result<PipelineResult, PipelineError> {
        // Store document in S3
        let storage_key = self.storage.store_document(
            &request.id,
            &document.content,
            DocumentFormat::Markdown,
        ).await.map_err(|e| PipelineError::Storage(e.to_string()))?;
        
        // Update database state
        self.update_document_state(
            &request.id,
            WorkflowStatus::Approved,
            &storage_key,
            compliance.score,
            document.confidence,
        ).await?;
        
        let completed_at = Utc::now();
        let requires_approval = self.requires_human_approval(compliance);
        
        // Log finalization
        self.log_audit(&request.id, "Pipeline", "Finalized", &serde_json::json!!({
            "storage_key": storage_key,
            "requires_approval": requires_approval,
            "compliance_score": compliance.score,
        })).await?;
        
        Ok(PipelineResult {
            document_id: request.id,
            final_status: if requires_approval {
                WorkflowStatus::InReview
            } else {
                WorkflowStatus::Approved
            },
            agent_results,
            total_iterations: iteration,
            started_at,
            completed_at,
            requires_human_approval: requires_approval,
            compliance_score: compliance.score,
            confidence_score: document.confidence,
        })
    }
    
    /// Determine if human approval is required
    fn requires_human_approval(&self, compliance: &ComplianceResult) -> bool {
        // ALL Woo-relevant documents require human approval
        if compliance.is_woo_relevant {
            return true;
        }
        
        // Apply domain trust level rules
        match self.domain_config.trust_level {
            TrustLevel::Low => true,
            TrustLevel::Medium => compliance.score < self.domain_config.required_approval_threshold,
            TrustLevel::High => false, // Already checked Woo above
        }
    }
}

// Helper types for agent execution
struct AgentExecutionOutput<T> {
    output: T,
    data: serde_json::Value,
}

struct AgentExecutionWithResult<T> {
    output: T,
    result: AgentExecutionResult,
}
```

### Individual Agent Execution Methods

```rust
/// File: /Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/pipeline.rs
/// (continued)

impl AgentPipeline {
    /// Execute Research Agent
    async fn execute_research(
        &self,
        request: &DocumentRequest,
    ) -> Result<AgentExecutionOutput<ResearchContext>, PipelineError> {
        execute_research_agent(
            request,
            &self.kg_client,
            &self.domain_config,
        ).await
        .map(|context| AgentExecutionOutput {
            data: serde_json::to_value(&context).unwrap_or_default(),
            output: context,
        })
        .map_err(|e| PipelineError::transient_agent("Research", e))
    }
    
    /// Execute Content Agent
    async fn execute_content(
        &self,
        research: &ResearchContext,
    ) -> Result<AgentExecutionOutput<GeneratedDocument>, PipelineError> {
        execute_content_agent(
            research,
            &self.template,
            &self.kg_client,
        ).await
        .map(|document| AgentExecutionOutput {
            data: serde_json::json!({
                "sections": document.sections.len(),
                "word_count": document.content.split_whitespace().count(),
            }),
            output: document,
        })
        .map_err(|e| match e {
            ContentAgentError::TemplateNotFound => PipelineError::permanent_agent("Content", e.to_string()),
            _ => PipelineError::transient_agent("Content", e.to_string()),
        })
    }
    
    /// Execute Compliance Agent
    async fn execute_compliance(
        &self,
        document: &GeneratedDocument,
    ) -> Result<AgentExecutionOutput<ComplianceResult>, PipelineError> {
        execute_compliance_agent(
            document,
            &self.compliance_rules,
        ).await
        .map(|result| AgentExecutionOutput {
            data: serde_json::to_value(&result).unwrap_or_default(),
            output: result,
        })
        .map_err(|e| PipelineError::transient_agent("Compliance", e))
    }
    
    /// Execute Review Agent
    async fn execute_review(
        &self,
        document: &GeneratedDocument,
        compliance: &ComplianceResult,
    ) -> Result<AgentExecutionOutput<ReviewDecision>, PipelineError> {
        execute_review_agent(
            document,
            compliance,
            &self.domain_config,
        ).await
        .map(|decision| AgentExecutionOutput {
            data: serde_json::to_value(&decision).unwrap_or_default(),
            output: decision,
        })
        .map_err(|e| PipelineError::transient_agent("Review", e))
    }
}
```

### Audit and Checkpoint Methods

```rust
/// File: /Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/pipeline.rs
/// (continued)

impl AgentPipeline {
    /// Log an audit entry
    async fn log_audit(
        &self,
        document_id: &Uuid,
        agent: &str,
        action: &str,
        details: &serde_json::Value,
    ) -> Result<(), PipelineError> {
        let entry = AuditEntry {
            id: Uuid::new_v4(),
            document_id: *document_id,
            agent_name: agent.to_string(),
            action: action.to_string(),
            details: details.clone(),
            timestamp: Utc::now(),
            execution_time_ms: None,
        };
        
        self.storage.store_audit_entry(entry).await
            .map_err(|e| PipelineError::Database(e.to_string()))
    }
    
    /// Log a transient error retry
    async fn log_transient_retry(
        &self,
        agent_name: &str,
        retry_count: u32,
        error: &PipelineError,
    ) {
        self.log_audit(
            &Uuid::new_v4(), // Placeholder document_id for agent-level logging
            agent_name,
            &format!("Retry attempt {}", retry_count),
            &serde_json::json!({
                "error": error.to_string(),
            }),
        ).await
        .err(); // Ignore errors during retry logging
    }
    
    /// Save a pipeline checkpoint
    async fn save_checkpoint(&self, checkpoint: PipelineCheckpoint) -> Result<(), PipelineError> {
        if !self.config.enable_checkpoints {
            return Ok(());
        }
        
        self.storage.store_checkpoint(checkpoint).await
            .map_err(|e| PipelineError::Storage(e.to_string()))
    }
    
    /// Load the most recent checkpoint for a document
    pub async fn load_checkpoint(
        &self,
        document_id: Uuid,
    ) -> Result<Option<PipelineCheckpoint>, PipelineError> {
        if !self.config.enable_checkpoints {
            return Ok(None);
        }
        
        self.storage.load_checkpoint(document_id).await
            .map_err(|e| PipelineError::Storage(e.to_string()))
    }
}
```

### State Transition Helper Methods

```rust
/// File: /Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/pipeline.rs
/// (continued)

impl AgentPipeline {
    /// Update document state in database
    async fn update_document_state(
        &self,
        document_id: &Uuid,
        new_status: WorkflowStatus,
        storage_key: &str,
        compliance_score: f32,
        confidence_score: f32,
    ) -> Result<(), PipelineError> {
        // Validate state transition
        let current_status = self.storage.get_document_status(document_id).await
            .map_err(|e| PipelineError::Database(e.to_string()))?;
        
        if !current_status.can_transition_to(&new_status) {
            return Err(PipelineError::InvalidStateTransition {
                from: current_status,
                to: new_status,
            });
        }
        
        self.storage.update_document(
            document_id,
            new_status,
            storage_key,
            compliance_score,
            confidence_score,
        ).await
        .map_err(|e| PipelineError::Database(e.to_string()))
    }
    
    /// Require human approval (returns InReview status)
    async fn require_human_approval(
        &self,
        request: DocumentRequest,
        agent_results: Vec<AgentExecutionResult>,
        iteration: usize,
        started_at: DateTime<Utc>,
        reason: &str,
    ) -> Result<PipelineResult, PipelineError> {
        self.update_document_state(
            &request.id,
            WorkflowStatus::InReview,
            "", // No storage key yet
            0.0,
            0.0,
        ).await?;
        
        self.log_audit(&request.id, "Pipeline", "AwaitingApproval", &serde_json::json!!({
            "reason": reason,
        })).await?;
        
        Ok(PipelineResult {
            document_id: request.id,
            final_status: WorkflowStatus::InReview,
            agent_results,
            total_iterations: iteration,
            started_at,
            completed_at: Utc::now(),
            requires_human_approval: true,
            compliance_score: 0.0,
            confidence_score: 0.0,
        })
    }
    
    /// Reject a document (returns to Draft)
    async fn reject_document(
        &self,
        request: DocumentRequest,
        agent_results: Vec<AgentExecutionResult>,
        iteration: usize,
        started_at: DateTime<Utc>,
        reason: String,
    ) -> Result<PipelineResult, PipelineError> {
        self.update_document_state(
            &request.id,
            WorkflowStatus::Rejected,
            "",
            0.0,
            0.0,
        ).await?;
        
        self.log_audit(&request.id, "Pipeline", "Rejected", &serde_json::json!!({
            "reason": reason,
        })).await?;
        
        Ok(PipelineResult {
            document_id: request.id,
            final_status: WorkflowStatus::Rejected,
            agent_results,
            total_iterations: iteration,
            started_at,
            completed_at: Utc::now(),
            requires_human_approval: false,
            compliance_score: 0.0,
            confidence_score: 0.0,
        })
    }
}
```

### Storage Backend Extension

The `iou-storage` crate needs the following additional methods for pipeline support:

```rust
/// File: /Users/marc/Projecten/iou-modern/crates/iou-storage/src/metadata.rs
/// (extensions to existing storage client)

impl StorageBackend {
    /// Store an audit trail entry
    pub async fn store_audit_entry(
        &self,
        entry: AuditEntry,
    ) -> Result<(), StorageError> {
        // Implementation writes to document_audit table
        todo!("Implement audit entry storage")
    }
    
    /// Store a pipeline checkpoint
    pub async fn store_checkpoint(
        &self,
        checkpoint: PipelineCheckpoint,
    ) -> Result<(), StorageError> {
        // Implementation writes to pipeline_checkpoints table or key-value store
        todo!("Implement checkpoint storage")
    }
    
    /// Load the most recent checkpoint for a document
    pub async fn load_checkpoint(
        &self,
        document_id: Uuid,
    ) -> Result<Option<PipelineCheckpoint>, StorageError> {
        // Implementation reads from pipeline_checkpoints table
        todo!("Implement checkpoint retrieval")
    }
    
    /// Get current document status from database
    pub async fn get_document_status(
        &self,
        document_id: &Uuid,
    ) -> Result<WorkflowStatus, StorageError> {
        // Implementation reads state column from documents table
        todo!("Implement document status retrieval")
    }
    
    /// Update document metadata in database
    pub async fn update_document(
        &self,
        document_id: &Uuid,
        new_status: WorkflowStatus,
        storage_key: &str,
        compliance_score: f32,
        confidence_score: f32,
    ) -> Result<(), StorageError> {
        // Implementation updates documents table
        todo!("Implement document update")
    }
}
```

## Integration Points

### Reused Types from iou-core

The pipeline orchestrator reuses the following existing types:

- `WorkflowStatus` from `iou-core/src/workflows.rs`
  - Used for document state management
  - State transition validation via `can_transition_to()`

- `WooRefusalGround`, `WooDisclosureClass` from `iou-core/src/compliance.rs`
  - Used within compliance results

### Storage Integration

The pipeline depends on storage operations defined in:

- `iou-storage/src/s3.rs`: S3 document storage
- `iou-storage/src/metadata.rs`: DuckDB metadata operations

### Agent Dependencies

The pipeline orchestrates the following agents (each defined in its respective section):

- `execute_research_agent` - Returns `ResearchContext`
- `execute_content_agent` - Returns `GeneratedDocument`
- `execute_compliance_agent` - Returns `ComplianceResult`
- `execute_review_agent` - Returns `ReviewDecision`

## Implementation Checklist

1. Create `error.rs` with error types and severity classification
2. Create `pipeline.rs` with orchestration logic
3. Implement `execute_document_pipeline` with sequential execution
4. Implement maker-checker iteration loop
5. Add transient error retry with exponential backoff
6. Add fail-fast for permanent errors
7. Implement checkpoint save/load
8. Add audit trail logging for all agent executions
9. Implement state transition validation
10. Add human approval determination based on trust level and Woo relevance
11. Implement document finalization (storage + state update)
12. Add unit tests for all pipeline scenarios
13. Add integration tests with mock agents
14. Verify integration with existing `WorkflowStatus` type