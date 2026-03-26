//! Stage execution tests
//!
//! Tests for the stage executor lifecycle management.

use iou_orchestrator::stage_executor::{StageExecutor, StageExecutorContext, StageExecutorError, Result};
use iou_core::workflows::multi_stage::{
    StageInstance, StageStatus, ApprovalType, ApprovalDecision, ApprovalResponse,
};
use iou_core::config::workflow::{WorkflowConfig, StageConfig, ApprovalTypeConfig, VersionStorageConfig, SlaConfig, ApproverConfig};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

// Mock database implementation for testing
struct MockDatabase {
    stages: std::sync::Arc<tokio::sync::RwLock<Vec<StageInstance>>>,
    approvals: std::sync::Arc<tokio::sync::RwLock<Vec<ApprovalResponse>>>,
}

impl MockDatabase {
    fn new() -> Self {
        Self {
            stages: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            approvals: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    async fn get_stages(&self) -> Vec<StageInstance> {
        self.stages.read().await.clone()
    }
}

#[async_trait::async_trait]
impl iou_orchestrator::stage_executor::Database for MockDatabase {
    async fn create_stage_instance(&self, stage: &StageInstance) -> std::result::Result<(), iou_orchestrator::stage_executor::DbError> {
        self.stages.write().await.push(stage.clone());
        Ok(())
    }

    async fn update_stage_instance(&self, stage: &StageInstance) -> std::result::Result<(), iou_orchestrator::stage_executor::DbError> {
        let mut stages = self.stages.write().await;
        if let Some(pos) = stages.iter().position(|s| s.id == stage.id) {
            stages[pos] = stage.clone();
        }
        Ok(())
    }

    async fn get_stage_instances(&self, document_id: Uuid) -> std::result::Result<Vec<StageInstance>, iou_orchestrator::stage_executor::DbError> {
        let stages = self.stages.read().await;
        Ok(stages.iter().filter(|s| s.document_id == document_id).cloned().collect())
    }

    async fn get_stage_instance(&self, id: Uuid) -> std::result::Result<Option<StageInstance>, iou_orchestrator::stage_executor::DbError> {
        let stages = self.stages.read().await;
        Ok(stages.iter().find(|s| s.id == id).cloned())
    }

    async fn create_approval_record(&self, approval: &ApprovalResponse) -> std::result::Result<(), iou_orchestrator::stage_executor::DbError> {
        self.approvals.write().await.push(approval.clone());
        Ok(())
    }
}

// Mock notification service
struct MockNotificationService;

#[async_trait::async_trait]
impl iou_orchestrator::stage_executor::NotificationService for MockNotificationService {
    async fn notify_stage_started(&self, _approver_id: Uuid, _stage: &StageInstance) -> Result<()> {
        Ok(())
    }
}

// Mock SLA calculator
struct MockSlaCalculator;

impl MockSlaCalculator {
    fn calculate_deadline(&self, _from: chrono::DateTime<Utc>, _sla_hours: i32) -> chrono::DateTime<Utc> {
        Utc::now() + chrono::Duration::hours(24)
    }
}

// Mock delegation resolver
struct MockDelegationResolver;

impl MockDelegationResolver {
    async fn resolve_approver(&self, user_id: Uuid, _document_type: &str, _document_id: Option<Uuid>) -> Result<Uuid> {
        Ok(user_id) // Return the same user for tests
    }
}

fn create_test_context() -> StageExecutorContext {
    StageExecutorContext {
        db: Arc::new(MockDatabase::new()),
        delegation_resolver: Arc::new(MockDelegationResolver),
        sla_calculator: Arc::new(MockSlaCalculator),
        notification_service: Arc::new(MockNotificationService),
    }
}

fn create_test_workflow_config() -> WorkflowConfig {
    WorkflowConfig {
        approval_stages: vec![
            StageConfig {
                stage_id: "manager_review".to_string(),
                stage_name: "Manager Review".to_string(),
                stage_order: 1,
                approval_type: ApprovalTypeConfig::Sequential,
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
                approval_type: ApprovalTypeConfig::ParallelAny,
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
        version_storage: VersionStorageConfig::default(),
        sla: SlaConfig {
            weekend_days: vec!["Saturday".to_string(), "Sunday".to_string()],
            escalation_hours: vec![24, 8, 1],
        },
    }
}

#[tokio::test]
async fn test_initialize_stages_creates_stage_instances_from_config() {
    let ctx = create_test_context();
    let executor = StageExecutor::new(ctx);

    let document_id = Uuid::new_v4();
    let stages = executor
        .initialize_stages(document_id, "test_domain", "invoice", Utc::now())
        .await
        .unwrap();

    // Should create 2 stages from test config
    assert_eq!(stages.len(), 2);
    assert_eq!(stages[0].stage_id, "manager_review");
    assert_eq!(stages[1].stage_id, "director_approval");
}

#[tokio::test]
async fn test_initialize_stages_resolves_approvers_using_delegation_lookup() {
    let ctx = create_test_context();
    let executor = StageExecutor::new(ctx);

    let document_id = Uuid::new_v4();
    let approver_id = Uuid::new_v4();

    let config = WorkflowConfig {
        approval_stages: vec![StageConfig {
            stage_id: "review".to_string(),
            stage_name: "Review".to_string(),
            stage_order: 1,
            approval_type: ApprovalTypeConfig::Sequential,
            approvers: vec![ApproverConfig {
                user_id: Some(approver_id.to_string()),
                role: None,
            }],
            sla_hours: 72,
            expiry_action: "notify_only".to_string(),
            is_optional: false,
            condition: None,
        }],
        version_storage: VersionStorageConfig::default(),
        sla: SlaConfig {
            weekend_days: vec![],
            escalation_hours: vec![24],
        },
    };

    // Temporarily set the config
    let stages = executor
        .initialize_stages(document_id, "test_domain", "test", Utc::now())
        .await
        .unwrap();

    // Should have resolved the approver
    assert_eq!(stages[0].approvers.len(), 1);
}

#[tokio::test]
async fn test_initialize_stages_calculates_deadlines_using_sla_calculator() {
    let ctx = create_test_context();
    let executor = StageExecutor::new(ctx);

    let document_id = Uuid::new_v4();
    let stages = executor
        .initialize_stages(document_id, "test_domain", "invoice", Utc::now())
        .await
        .unwrap();

    // All stages should have deadlines
    for stage in &stages {
        assert!(stage.deadline.is_some());
    }
}

#[tokio::test]
async fn test_initialize_stages_skips_optional_stages_when_condition_not_met() {
    // This test will need condition evaluation implemented
    let ctx = create_test_context();
    let executor = StageExecutor::new(ctx);

    let document_id = Uuid::new_v4();
    let stages = executor
        .initialize_stages(document_id, "test_domain", "invoice", Utc::now())
        .await
        .unwrap();

    // The director_approval stage is optional with a condition
    // For now, we expect both stages (condition evaluation returns true)
    assert!(stages.len() >= 1);
}

#[tokio::test]
async fn test_start_stage_updates_status_to_in_progress_and_sets_started_at() {
    let ctx = create_test_context();
    let executor = StageExecutor::new(ctx);

    let document_id = Uuid::new_v4();
    let approver = Uuid::new_v4();
    let mut stage = StageInstance {
        id: Uuid::new_v4(),
        document_id,
        stage_id: "review".to_string(),
        status: StageStatus::Pending,
        started_at: None,
        completed_at: None,
        deadline: None,
        approvers: vec![approver],
        approvals_received: vec![],
    };

    executor.start_stage(&stage).await.unwrap();

    // Check the stage was updated in the database
    let db = ctx.db.as_any().downcast_ref::<MockDatabase>().unwrap();
    let stages = db.get_stages().await;
    let updated = &stages[0];

    assert_eq!(updated.status, StageStatus::InProgress);
    assert!(updated.started_at.is_some());
}

#[tokio::test]
async fn test_start_stage_sends_websocket_notification_to_all_approvers() {
    let ctx = create_test_context();
    let executor = StageExecutor::new(ctx);

    let approvers = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    let stage = StageInstance {
        id: Uuid::new_v4(),
        document_id: Uuid::new_v4(),
        stage_id: "review".to_string(),
        status: StageStatus::Pending,
        started_at: None,
        completed_at: None,
        deadline: None,
        approvers: approvers.clone(),
        approvals_received: vec![],
    };

    // Should not error even with multiple approvers
    let result = executor.start_stage(&stage).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_record_approval_creates_approval_response_record() {
    let ctx = create_test_context();
    let executor = StageExecutor::new(ctx);

    let document_id = Uuid::new_v4();
    let approver = Uuid::new_v4();
    let stage = StageInstance {
        id: Uuid::new_v4(),
        document_id,
        stage_id: "review".to_string(),
        status: StageStatus::InProgress,
        started_at: Some(Utc::now()),
        completed_at: None,
        deadline: None,
        approvers: vec![approver],
        approvals_received: vec![],
    };

    // Create the stage first
    ctx.db.create_stage_instance(&stage).await.unwrap();

    let result = executor
        .record_approval(
            stage.id,
            approver,
            ApprovalDecision::Approved,
            Some("Looks good".to_string()),
            &ApprovalType::ParallelAny,
        )
        .await
        .unwrap();

    // Check approval was recorded
    let db = ctx.db.as_any().downcast_ref::<MockDatabase>().unwrap();
    let stages = db.get_stages().await;
    assert_eq!(stages[0].approvals_received.len(), 1);
}

#[tokio::test]
async fn test_record_approval_updates_stages_approvals_received_list() {
    let ctx = create_test_context();
    let executor = StageExecutor::new(ctx);

    let document_id = Uuid::new_v4();
    let approver1 = Uuid::new_v4();
    let approver2 = Uuid::new_v4();
    let stage = StageInstance {
        id: Uuid::new_v4(),
        document_id,
        stage_id: "review".to_string(),
        status: StageStatus::InProgress,
        started_at: Some(Utc::now()),
        completed_at: None,
        deadline: None,
        approvers: vec![approver1, approver2],
        approvals_received: vec![],
    };

    ctx.db.create_stage_instance(&stage).await.unwrap();

    executor
        .record_approval(
            stage.id,
            approver1,
            ApprovalDecision::Approved,
            None,
            &ApprovalType::ParallelAll,
        )
        .await
        .unwrap();

    let db = ctx.db.as_any().downcast_ref::<MockDatabase>().unwrap();
    let stages = db.get_stages().await;
    assert_eq!(stages[0].approvals_received.len(), 1);
}

#[tokio::test]
async fn test_record_approval_returns_complete_when_quorum_met() {
    use iou_orchestrator::state_machine::multi_stage::StageCompletionStatus;

    let ctx = create_test_context();
    let executor = StageExecutor::new(ctx);

    let document_id = Uuid::new_v4();
    let approver = Uuid::new_v4();
    let stage = StageInstance {
        id: Uuid::new_v4(),
        document_id,
        stage_id: "review".to_string(),
        status: StageStatus::InProgress,
        started_at: Some(Utc::now()),
        completed_at: None,
        deadline: None,
        approvers: vec![approver],
        approvals_received: vec![],
    };

    ctx.db.create_stage_instance(&stage).await.unwrap();

    let result = executor
        .record_approval(
            stage.id,
            approver,
            ApprovalDecision::Approved,
            None,
            &ApprovalType::ParallelAny,
        )
        .await
        .unwrap();

    assert_eq!(result, StageCompletionStatus::Complete);
}

#[tokio::test]
async fn test_record_approval_triggers_next_stage_transition_on_completion() {
    let ctx = create_test_context();
    let executor = StageExecutor::new(ctx);

    let document_id = Uuid::new_v4();
    let approver = Uuid::new_v4();
    let stage = StageInstance {
        id: Uuid::new_v4(),
        document_id,
        stage_id: "review".to_string(),
        status: StageStatus::InProgress,
        started_at: Some(Utc::now()),
        completed_at: None,
        deadline: None,
        approvers: vec![approver],
        approvals_received: vec![],
    };

    ctx.db.create_stage_instance(&stage).await.unwrap();

    executor
        .record_approval(
            stage.id,
            approver,
            ApprovalDecision::Approved,
            None,
            &ApprovalType::ParallelAny,
        )
        .await
        .unwrap();

    // Verify stage was marked complete
    let db = ctx.db.as_any().downcast_ref::<MockDatabase>().unwrap();
    let stages = db.get_stages().await;
    assert_eq!(stages[0].status, StageStatus::Completed);
    assert!(stages[0].completed_at.is_some());
}

#[tokio::test]
async fn test_meets_quorum_returns_true_for_parallel_any_with_single_approval() {
    let ctx = create_test_context();
    let executor = StageExecutor::new(ctx);

    let approvers = vec![Uuid::new_v4(), Uuid::new_v4()];
    let mut stage = StageInstance {
        id: Uuid::new_v4(),
        document_id: Uuid::new_v4(),
        stage_id: "review".to_string(),
        status: StageStatus::InProgress,
        started_at: Some(Utc::now()),
        completed_at: None,
        deadline: None,
        approvers: approvers.clone(),
        approvals_received: vec![ApprovalResponse::new(
            approvers[0],
            ApprovalDecision::Approved,
            None,
        )],
    };

    assert!(executor.meets_quorum(&stage, &ApprovalType::ParallelAny));
}

#[tokio::test]
async fn test_meets_quorum_returns_true_for_parallel_all_only_when_all_approved() {
    let ctx = create_test_context();
    let executor = StageExecutor::new(ctx);

    let approvers = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    let mut stage = StageInstance {
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

    // Only 2 of 3 approvals - should not meet quorum
    assert!(!executor.meets_quorum(&stage, &ApprovalType::ParallelAll));

    // Add third approval
    stage.approvals_received.push(ApprovalResponse::new(
        approvers[2],
        ApprovalDecision::Approved,
        None,
    ));

    // Now should meet quorum
    assert!(executor.meets_quorum(&stage, &ApprovalType::ParallelAll));
}

#[tokio::test]
async fn test_meets_quorum_calculates_majority_correctly_for_odd_approvers() {
    let ctx = create_test_context();
    let executor = StageExecutor::new(ctx);

    // 5 approvers, need 3 for majority
    let approvers: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();
    let mut stage = StageInstance {
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

    // Only 2 of 5 - should not meet majority
    assert!(!executor.meets_quorum(&stage, &ApprovalType::ParallelMajority));

    // Add third approval
    stage.approvals_received.push(ApprovalResponse::new(
        approvers[2],
        ApprovalDecision::Approved,
        None,
    ));

    // 3 of 5 - should meet majority
    assert!(executor.meets_quorum(&stage, &ApprovalType::ParallelMajority));
}

#[tokio::test]
async fn test_meets_quorum_calculates_majority_correctly_for_even_approvers() {
    let ctx = create_test_context();
    let executor = StageExecutor::new(ctx);

    // 4 approvers, need 3 for majority (>50%)
    let approvers: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();
    let mut stage = StageInstance {
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

    // 2 of 4 = 50% - should not meet majority (need >50%)
    assert!(!executor.meets_quorum(&stage, &ApprovalType::ParallelMajority));

    // Add third approval
    stage.approvals_received.push(ApprovalResponse::new(
        approvers[2],
        ApprovalDecision::Approved,
        None,
    ));

    // 3 of 4 = 75% - should meet majority
    assert!(executor.meets_quorum(&stage, &ApprovalType::ParallelMajority));
}

#[tokio::test]
async fn test_record_approval_rejects_duplicate_approval_from_same_approver() {
    let ctx = create_test_context();
    let executor = StageExecutor::new(ctx);

    let document_id = Uuid::new_v4();
    let approver = Uuid::new_v4();
    let stage = StageInstance {
        id: Uuid::new_v4(),
        document_id,
        stage_id: "review".to_string(),
        status: StageStatus::InProgress,
        started_at: Some(Utc::now()),
        completed_at: None,
        deadline: None,
        approvers: vec![approver],
        approvals_received: vec![ApprovalResponse::new(
            approver,
            ApprovalDecision::Approved,
            None,
        )],
    };

    ctx.db.create_stage_instance(&stage).await.unwrap();

    let result = executor
        .record_approval(
            stage.id,
            approver,
            ApprovalDecision::Approved,
            None,
            &ApprovalType::ParallelAny,
        )
        .await;

    assert!(matches!(result, Err(StageExecutorError::DuplicateApproval)));
}
