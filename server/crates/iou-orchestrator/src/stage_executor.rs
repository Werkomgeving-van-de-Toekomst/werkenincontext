//! Stage execution engine
//!
//! Handles the lifecycle of approval stages: initialization, starting,
//! recording approvals, and completion detection.

use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use iou_core::workflows::multi_stage::{
    StageInstance, StageStatus, ApprovalType, ApprovalDecision,
    ApprovalResponse,
};
use iou_core::config::workflow::{WorkflowConfig, StageConfig, ApproverConfig};
use crate::state_machine::multi_stage::{StageCompletionStatus, evaluate_stage_completion};
use async_trait::async_trait;

/// Context for stage execution
pub struct StageExecutorContext {
    pub db: Arc<dyn Database>,
    pub delegation_resolver: Arc<dyn DelegationResolver>,
    pub sla_calculator: Arc<dyn SlaCalculator>,
    pub notification_service: Arc<dyn NotificationService>,
}

/// Database trait for stage persistence
#[async_trait]
pub trait Database: Send + Sync {
    async fn create_stage_instance(&self, stage: &StageInstance) -> std::result::Result<(), DbError>;
    async fn update_stage_instance(&self, stage: &StageInstance) -> std::result::Result<(), DbError>;
    async fn get_stage_instances(&self, document_id: Uuid) -> std::result::Result<Vec<StageInstance>, DbError>;
    async fn get_stage_instance(&self, id: Uuid) -> std::result::Result<Option<StageInstance>, DbError>;
    async fn create_approval_record(&self, approval: &ApprovalResponse) -> std::result::Result<(), DbError>;
}

/// Extension trait for downcasting Database
pub trait DatabaseAny {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: Database + 'static> DatabaseAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Delegation resolver trait
#[async_trait]
pub trait DelegationResolver: Send + Sync {
    async fn resolve_approver(&self, user_id: Uuid, document_type: &str, document_id: Option<Uuid>) -> std::result::Result<Uuid, DbError>;
}

/// SLA calculator trait
pub trait SlaCalculator: Send + Sync {
    fn calculate_deadline(&self, from: DateTime<Utc>, sla_hours: i32) -> DateTime<Utc>;
}

/// Notification service trait
#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn notify_stage_started(&self, approver_id: Uuid, stage: &StageInstance) -> std::result::Result<(), StageExecutorError>;
}

/// Error type for stage execution
#[derive(Debug, thiserror::Error)]
pub enum StageExecutorError {
    #[error("Database error: {0}")]
    Database(#[from] DbError),
    #[error("Invalid stage configuration: {0}")]
    InvalidConfig(String),
    #[error("Stage not found: {0}")]
    StageNotFound(Uuid),
    #[error("Approver not authorized for this stage")]
    UnauthorizedApprover,
    #[error("Approval already recorded for this approver")]
    DuplicateApproval,
    #[error("Notification error: {0}")]
    NotificationError(String),
}

/// Database error type
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Database operation failed: {0}")]
    QueryFailed(String),
}

pub type Result<T> = std::result::Result<T, StageExecutorError>;

/// Stage execution engine
pub struct StageExecutor {
    ctx: StageExecutorContext,
    // Store a test config for testing purposes
    test_config: WorkflowConfig,
}

impl StageExecutor {
    pub fn new(ctx: StageExecutorContext) -> Self {
        // Create a default test config
        let test_config = WorkflowConfig {
            approval_stages: vec![
                StageConfig {
                    stage_id: "manager_review".to_string(),
                    stage_name: "Manager Review".to_string(),
                    stage_order: 1,
                    approval_type: iou_core::config::workflow::ApprovalTypeConfig::Sequential,
                    approvers: vec![
                        ApproverConfig {
                            user_id: Some(Uuid::new_v4().to_string()),
                            role: None,
                        },
                    ],
                    sla_hours: 72,
                    expiry_action: "notify_only".to_string(),
                    is_optional: false,
                    condition: None,
                },
                StageConfig {
                    stage_id: "director_approval".to_string(),
                    stage_name: "Director Approval".to_string(),
                    stage_order: 2,
                    approval_type: iou_core::config::workflow::ApprovalTypeConfig::ParallelAny,
                    approvers: vec![
                        ApproverConfig {
                            user_id: Some(Uuid::new_v4().to_string()),
                            role: None,
                        },
                    ],
                    sla_hours: 48,
                    expiry_action: "escalate_to:finance".to_string(),
                    is_optional: true,
                    condition: Some("amount > 10000".to_string()),
                },
            ],
            version_storage: iou_core::config::workflow::VersionStorageConfig::default(),
            sla: iou_core::config::workflow::SlaConfig {
                weekend_days: vec!["Saturday".to_string(), "Sunday".to_string()],
                escalation_hours: vec![24, 8, 1],
            },
        };

        Self { ctx, test_config }
    }

    /// Initialize stage instances for a newly submitted document
    pub async fn initialize_stages(
        &self,
        document_id: Uuid,
        _domain_id: &str,
        _document_type: &str,
        submitted_at: DateTime<Utc>,
    ) -> Result<Vec<StageInstance>> {
        // Load workflow configuration
        let config = &self.test_config;

        let mut stage_instances = Vec::new();

        for stage_config in &config.approval_stages {
            // Check if optional stage should be skipped
            if stage_config.is_optional {
                if !self.evaluate_optional_condition(stage_config, document_id).await? {
                    continue; // Skip this stage
                }
            }

            // Resolve approvers
            let resolved_approvers = self.resolve_approvers(
                &stage_config.approvers,
                _document_type,
                Some(document_id),
            ).await?;

            // Calculate deadline
            let deadline = self.ctx.sla_calculator
                .calculate_deadline(submitted_at, stage_config.sla_hours);

            // Create stage instance
            let instance = StageInstance {
                id: Uuid::new_v4(),
                document_id,
                stage_id: stage_config.stage_id.clone(),
                status: StageStatus::Pending,
                started_at: None,
                completed_at: None,
                deadline: Some(deadline),
                approvers: resolved_approvers,
                approvals_received: Vec::new(),
            };

            // Persist to database
            self.ctx.db.create_stage_instance(&instance).await?;

            stage_instances.push(instance);
        }

        Ok(stage_instances)
    }

    /// Start a stage (transition to InProgress and notify approvers)
    pub async fn start_stage(&self, stage: &StageInstance) -> Result<()> {
        let mut updated = stage.clone();
        updated.status = StageStatus::InProgress;
        updated.started_at = Some(Utc::now());

        self.ctx.db.update_stage_instance(&updated).await?;

        // Send WebSocket notifications to all approvers
        for approver_id in &updated.approvers {
            self.ctx.notification_service
                .notify_stage_started(*approver_id, &updated)
                .await?;
        }

        Ok(())
    }

    /// Record an approval decision for a stage
    pub async fn record_approval(
        &self,
        stage_id: Uuid,
        approver_id: Uuid,
        decision: ApprovalDecision,
        comment: Option<String>,
        approval_type: &ApprovalType,
    ) -> Result<StageCompletionStatus> {
        // Load the stage
        let mut stage = self.ctx.db.get_stage_instance(stage_id).await?
            .ok_or(StageExecutorError::StageNotFound(stage_id))?;

        // Verify approver is authorized
        if !stage.approvers.contains(&approver_id) {
            return Err(StageExecutorError::UnauthorizedApprover);
        }

        // Check for duplicate approval
        if stage.approvals_received.iter().any(|a| a.approver_id == approver_id) {
            return Err(StageExecutorError::DuplicateApproval);
        }

        // Create approval record
        let approval = ApprovalResponse {
            approver_id,
            delegated_from: None,
            decision: decision.clone(),
            comment,
            responded_at: Utc::now(),
        };

        // Update stage first (adds approval to in-memory list)
        stage.approvals_received.push(approval.clone());

        // Check stage completion
        let completion_status = evaluate_stage_completion(&stage, approval_type);

        // If complete, update stage status
        if matches!(completion_status, StageCompletionStatus::Complete) {
            stage.status = StageStatus::Completed;
            stage.completed_at = Some(Utc::now());
        }

        // Persist the updated stage (with new approval and status)
        self.ctx.db.update_stage_instance(&stage).await?;

        // Then persist approval record for audit trail
        self.ctx.db.create_approval_record(&approval).await?;

        Ok(completion_status)
    }

    /// Check if a stage meets its quorum requirements
    pub fn meets_quorum(&self, stage: &StageInstance, approval_type: &ApprovalType) -> bool {
        stage.is_complete(*approval_type)
    }

    // Private helper methods

    async fn evaluate_optional_condition(
        &self,
        _stage_config: &StageConfig,
        _document_id: Uuid,
    ) -> Result<bool> {
        // For now, return true (include the stage)
        // In a real implementation, this would evaluate the condition expression
        Ok(true)
    }

    async fn resolve_approvers(
        &self,
        approver_configs: &[ApproverConfig],
        document_type: &str,
        document_id: Option<Uuid>,
    ) -> Result<Vec<Uuid>> {
        let mut resolved = Vec::new();

        for config in approver_configs {
            if let Some(user_id_str) = &config.user_id {
                // Parse UUID
                let user_id = Uuid::parse_str(user_id_str)
                    .map_err(|_| StageExecutorError::InvalidConfig(
                        format!("Invalid UUID: {}", user_id_str)
                    ))?;

                // Check for delegations
                let final_approver = self.ctx.delegation_resolver
                    .resolve_approver(user_id, document_type, document_id)
                    .await?;
                resolved.push(final_approver);
            } else if let Some(_role) = &config.role {
                // Resolve role to users - placeholder
                // In real implementation, query users by role
            }
        }

        Ok(resolved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    // Mock implementations for testing

    struct MockDb;

    #[async_trait]
    impl Database for MockDb {
        async fn create_stage_instance(&self, _stage: &StageInstance) -> std::result::Result<(), DbError> {
            Ok(())
        }
        async fn update_stage_instance(&self, _stage: &StageInstance) -> std::result::Result<(), DbError> {
            Ok(())
        }
        async fn get_stage_instances(&self, _document_id: Uuid) -> std::result::Result<Vec<StageInstance>, DbError> {
            Ok(vec![])
        }
        async fn get_stage_instance(&self, _id: Uuid) -> std::result::Result<Option<StageInstance>, DbError> {
            Ok(None)
        }
        async fn create_approval_record(&self, _approval: &ApprovalResponse) -> std::result::Result<(), DbError> {
            Ok(())
        }
    }

    struct MockDelegationResolver;

    #[async_trait]
    impl DelegationResolver for MockDelegationResolver {
        async fn resolve_approver(&self, user_id: Uuid, _document_type: &str, _document_id: Option<Uuid>) -> std::result::Result<Uuid, DbError> {
            Ok(user_id)
        }
    }

    struct MockSlaCalculator;

    impl SlaCalculator for MockSlaCalculator {
        fn calculate_deadline(&self, from: DateTime<Utc>, sla_hours: i32) -> DateTime<Utc> {
            from + Duration::hours(sla_hours as i64)
        }
    }

    struct MockNotificationService;

    #[async_trait]
    impl NotificationService for MockNotificationService {
        async fn notify_stage_started(&self, _approver_id: Uuid, _stage: &StageInstance) -> std::result::Result<(), StageExecutorError> {
            Ok(())
        }
    }

    fn create_test_context() -> StageExecutorContext {
        StageExecutorContext {
            db: Arc::new(MockDb),
            delegation_resolver: Arc::new(MockDelegationResolver),
            sla_calculator: Arc::new(MockSlaCalculator),
            notification_service: Arc::new(MockNotificationService),
        }
    }

    #[test]
    fn test_meets_quorum_parallel_any() {
        let ctx = create_test_context();
        let executor = StageExecutor::new(ctx);

        let approvers = vec![Uuid::new_v4(), Uuid::new_v4()];
        let stage = StageInstance {
            id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            stage_id: "review".to_string(),
            status: StageStatus::InProgress,
            started_at: Some(Utc::now()),
            completed_at: None,
            deadline: None,
            approvers: approvers.clone(),
            approvals_received: vec![
                ApprovalResponse::new(approvers[0], ApprovalDecision::Approved, None),
            ],
        };

        assert!(executor.meets_quorum(&stage, &ApprovalType::ParallelAny));
    }

    #[test]
    fn test_meets_quorum_parallel_all() {
        let ctx = create_test_context();
        let executor = StageExecutor::new(ctx);

        let approvers = vec![Uuid::new_v4(), Uuid::new_v4()];
        let stage = StageInstance {
            id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            stage_id: "review".to_string(),
            status: StageStatus::InProgress,
            started_at: Some(Utc::now()),
            completed_at: None,
            deadline: None,
            approvers: approvers.clone(),
            approvals_received: vec![
                ApprovalResponse::new(approvers[0], ApprovalDecision::Approved, None),
            ],
        };

        assert!(!executor.meets_quorum(&stage, &ApprovalType::ParallelAll));
    }

    #[test]
    fn test_meets_quorum_parallel_majority() {
        let ctx = create_test_context();
        let executor = StageExecutor::new(ctx);

        let approvers: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();
        let stage = StageInstance {
            id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            stage_id: "review".to_string(),
            status: StageStatus::InProgress,
            started_at: Some(Utc::now()),
            completed_at: None,
            deadline: None,
            approvers: approvers.clone(),
            approvals_received: vec![
                ApprovalResponse::new(approvers[0], ApprovalDecision::Approved, None),
                ApprovalResponse::new(approvers[1], ApprovalDecision::Approved, None),
            ],
        };

        // 2 out of 5 is not majority
        assert!(!executor.meets_quorum(&stage, &ApprovalType::ParallelMajority));
    }
}
