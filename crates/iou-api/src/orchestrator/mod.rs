//! Orchestrator integration for document workflow execution
//!
//! This module wraps the `iou-orchestrator` crate to integrate
//! workflow execution with the API layer.

pub mod wrapper;
pub mod types;

pub use wrapper::WorkflowOrchestrator;
