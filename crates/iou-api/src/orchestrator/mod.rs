//! Orchestrator integration for document workflow execution
//!
//! This module wraps the `iou-orchestrator` crate to integrate
//! workflow execution with the API layer.
//!
//! # Overview
//!
//! The orchestrator coordinates AI agent execution (Research, Content, Compliance, Review)
//! with human-in-the-loop approval points. Workflows run asynchronously with status
//! updates broadcast via WebSocket.
//!
//! # Usage
//!
//! ```no_run
//! use crate::orchestrator::WorkflowOrchestrator;
//! use tokio::sync::broadcast;
//!
//! let (status_tx, _) = broadcast::channel(100);
//! let orchestrator = WorkflowOrchestrator::new(db, status_tx);
//! orchestrator.start_workflow(context).await?;
//! ```

pub mod wrapper;
pub mod types;

pub use wrapper::{WorkflowOrchestrator, OrchestratorConfig, OrchestratorError};
pub use types::{StatusMessage, workflow_state_to_document_state, agent_display_name};
