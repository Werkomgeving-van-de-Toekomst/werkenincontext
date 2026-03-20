//! Keuze van document-workflow: interne Rust-orchestrator vs. volledig Camunda (Zeebe).

use std::sync::Arc;

use crate::camunda::CamundaGateway;

/// `IOU_DOCUMENT_WORKFLOW=camunda` — alleen Zeebe + workers; geen [`WorkflowOrchestrator`](crate::orchestrator::WorkflowOrchestrator).
/// Andere waarde of unset: klassieke Rust-workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentWorkflowDriver {
    Rust,
    Camunda,
}

impl DocumentWorkflowDriver {
    pub fn from_env() -> Self {
        match std::env::var("IOU_DOCUMENT_WORKFLOW")
            .map(|s| s.to_lowercase())
            .as_deref()
        {
            Ok("camunda") | Ok("c8") | Ok("zeebe") => DocumentWorkflowDriver::Camunda,
            _ => DocumentWorkflowDriver::Rust,
        }
    }
}

/// Gedeelde configuratie voor document-routes (`/documents/*`).
#[derive(Clone)]
pub struct DocumentWorkflowRuntime {
    pub driver: DocumentWorkflowDriver,
    pub camunda: Option<Arc<CamundaGateway>>,
}
