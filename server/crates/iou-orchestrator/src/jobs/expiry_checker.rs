//! Scheduled expiry checker job
//!
//! Periodically checks for approval stages approaching or past deadline
//! and triggers escalations. Runs on a configurable interval.

use crate::error::OrchestratorError;
use chrono::{DateTime, Utc};
use iou_core::{
    escalation::{EscalationService, ExpiryAction, NotificationChannel, StageDeadlineInfo},
    realtime::{RealtimeClient, RealtimeConfig},
    sla::SlaCalculator,
};
use std::time::Duration;
use tokio::time::interval;
use uuid::Uuid;

/// Configuration for the expiry checker
#[derive(Debug, Clone)]
pub struct ExpiryCheckerConfig {
    /// Check interval in seconds (default: 300 = 5 minutes)
    pub check_interval_secs: u64,

    /// Notification channels to use for escalations
    pub notification_channels: Vec<NotificationChannel>,

    /// Whether to execute expiry actions or just notify
    pub execute_actions: bool,
}

impl Default for ExpiryCheckerConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 300,
            notification_channels: vec![
                NotificationChannel::WebSocket,
                NotificationChannel::UiIndicator,
            ],
            execute_actions: true,
        }
    }
}

/// Background job that checks for expiring approval stages
pub struct ExpiryChecker {
    config: ExpiryCheckerConfig,
    escalation_service: EscalationService,
    // Database client would be injected here
}

impl ExpiryChecker {
    /// Create a new expiry checker
    pub fn new(
        config: ExpiryCheckerConfig,
        escalation_service: EscalationService,
    ) -> Self {
        Self {
            config,
            escalation_service,
        }
    }

    /// Create with default configuration
    pub fn with_defaults(escalation_service: EscalationService) -> Self {
        Self::new(ExpiryCheckerConfig::default(), escalation_service)
    }

    /// Get the checker's configuration
    pub fn config(&self) -> &ExpiryCheckerConfig {
        &self.config
    }

    /// Run the expiry checker loop
    ///
    /// This method runs indefinitely, checking for expiring stages
    /// at the configured interval. Should be run in a dedicated task.
    pub async fn run(&self) -> Result<(), OrchestratorError> {
        let mut timer = interval(Duration::from_secs(self.config.check_interval_secs));

        tracing::info!(
            "Expiry checker started, checking every {} seconds",
            self.config.check_interval_secs
        );

        loop {
            timer.tick().await;

            if let Err(e) = self.check_and_escalate().await {
                tracing::error!("Error in expiry check: {}", e);
                // Continue running despite errors
            }
        }
    }

    /// Perform a single check and escalate if needed
    pub async fn check_and_escalate(&self) -> Result<(), OrchestratorError> {
        tracing::debug!("Running expiry check for all stages");

        // TODO: Query database for all InProgress stages
        let in_progress_stages = self.fetch_in_progress_stages().await?;

        if in_progress_stages.is_empty() {
            tracing::trace!("No in-progress stages to check");
            return Ok(());
        }

        tracing::debug!("Checking {} in-progress stages", in_progress_stages.len());

        // Find stages needing escalation
        let pending_escalations = self
            .escalation_service
            .check_overdue_stages(in_progress_stages)
            .await;

        if pending_escalations.is_empty() {
            tracing::trace!("No stages require escalation at this time");
            return Ok(());
        }

        tracing::info!(
            "Found {} stages requiring escalation",
            pending_escalations.len()
        );

        // Process each escalation
        for escalation in &pending_escalations {
            self.process_escalation(escalation).await?;
        }

        Ok(())
    }

    /// Process a single escalation
    async fn process_escalation(
        &self,
        escalation: &iou_core::escalation::PendingEscalation,
    ) -> Result<(), OrchestratorError> {
        tracing::info!(
            "Processing escalation for document {} stage '{}': {:?}",
            escalation.stage_info.document_id,
            escalation.stage_info.stage_name,
            escalation.escalation_type
        );

        // Send notifications
        match self
            .escalation_service
            .send_escalation(escalation, &self.config.notification_channels)
            .await
        {
            Ok(records) => {
                tracing::info!(
                    "Sent {} notifications for document {}",
                    records.len(),
                    escalation.stage_info.document_id
                );
            }
            Err(e) => {
                tracing::error!(
                    "Failed to send escalation for document {}: {}",
                    escalation.stage_info.document_id,
                    e
                );
            }
        }

        // Execute expiry action if deadline exceeded
        if escalation.escalation_type == iou_core::escalation::EscalationType::DeadlineExceeded
            && self.config.execute_actions
        {
            self.execute_expiry_action(escalation).await?;
        }

        Ok(())
    }

    /// Execute the configured expiry action for a stage
    async fn execute_expiry_action(
        &self,
        escalation: &iou_core::escalation::PendingEscalation,
    ) -> Result<(), OrchestratorError> {
        let action = self.escalation_service.expiry_action();

        match action {
            ExpiryAction::NotifyOnly => {
                tracing::debug!(
                    "NotifyOnly action: no state change for document {}",
                    escalation.stage_info.document_id
                );
            }
            ExpiryAction::ReturnToDraft => {
                tracing::info!(
                    "Returning document {} to draft due to expiry",
                    escalation.stage_info.document_id
                );
                self.return_document_to_draft(escalation.stage_info.document_id)
                    .await?;
            }
            ExpiryAction::AutoApprove => {
                tracing::info!(
                    "Auto-approving stage {} for document {}",
                    escalation.stage_info.stage_id,
                    escalation.stage_info.document_id
                );
                self.auto_approve_stage(
                    escalation.stage_info.stage_instance_id,
                    escalation.stage_info.approvers.clone(),
                )
                .await?;
            }
            ExpiryAction::EscalateTo { target } => {
                tracing::info!(
                    "Escalating document {} to target '{}'",
                    escalation.stage_info.document_id,
                    target
                );
                // Additional escalation to the target
                // This would notify the target user/role
            }
        }

        Ok(())
    }

    /// Fetch all in-progress stages from the database
    async fn fetch_in_progress_stages(&self) -> Result<Vec<StageDeadlineInfo>, OrchestratorError> {
        // TODO: Implement database query
        // SELECT das.id, das.document_id, das.stage_id, as.stage_name,
        //        das.deadline, das.approvers
        // FROM document_approval_stages das
        // JOIN approval_stages as ON das.stage_id = as.id
        // WHERE das.stage_status = 'in_progress'
        //   AND das.deadline IS NOT NULL

        // For now, return empty vector
        Ok(vec![])
    }

    /// Return a document to draft status
    async fn return_document_to_draft(&self, document_id: Uuid) -> Result<(), OrchestratorError> {
        // TODO: Update document status to Draft
        // UPDATE documents SET status = 'draft' WHERE id = $1
        tracing::info!("Document {} returned to draft", document_id);
        Ok(())
    }

    /// Auto-approve a stage
    async fn auto_approve_stage(
        &self,
        stage_instance_id: Uuid,
        approvers: Vec<Uuid>,
    ) -> Result<(), OrchestratorError> {
        // TODO: Record automatic approval
        // This would create an approval record with a system user as approver
        tracing::info!(
            "Stage {} auto-approved with {} approvers",
            stage_instance_id,
            approvers.len()
        );
        Ok(())
    }

    /// Run a single check (for testing or manual invocation)
    pub async fn run_once(&self) -> Result<(), OrchestratorError> {
        self.check_and_escalate().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iou_core::realtime::{RealtimeClient, RealtimeConfig};
    use std::sync::Arc;

    #[test]
    fn test_expiry_checker_config_default() {
        let config = ExpiryCheckerConfig::default();
        assert_eq!(config.check_interval_secs, 300);
        assert_eq!(config.notification_channels.len(), 2);
        assert!(config.execute_actions);
    }

    #[tokio::test]
    async fn test_expiry_checker_creation() {
        let sla_calculator = SlaCalculator::new();

        let rt_config = RealtimeConfig {
            websocket_url: "ws://localhost:4000/socket/websocket".to_string(),
            ..Default::default()
        };
        let rt_client = RealtimeClient::new(rt_config);
        let escalation_service = EscalationService::new(
            sla_calculator,
            Arc::new(rt_client),
        );

        let checker = ExpiryChecker::with_defaults(escalation_service);

        // Verify checker was created
        assert_eq!(checker.config.check_interval_secs, 300);
    }

    #[tokio::test]
    async fn test_run_once_with_no_stages() {
        let sla_calculator = SlaCalculator::new();

        let rt_config = RealtimeConfig {
            websocket_url: "ws://localhost:4000/socket/websocket".to_string(),
            ..Default::default()
        };
        let rt_client = RealtimeClient::new(rt_config);
        let escalation_service = EscalationService::new(
            sla_calculator,
            Arc::new(rt_client),
        );

        let checker = ExpiryChecker::with_defaults(escalation_service);

        // Should not error even with no stages
        let result = checker.run_once().await;
        assert!(result.is_ok());
    }
}
