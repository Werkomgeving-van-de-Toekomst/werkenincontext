//! Test helpers for workflow integration tests
//!
//! ## TODO: API Integration Testing Infrastructure
//!
//! The current helpers work with domain types directly. Full API integration
//! tests would require:
//!
//! 1. **Test Server Setup**
//!    ```rust
//!    use axum::test::TestServer;
//!    use iou_api::routes::create_router;
//!
//!    let app = create_router(/* state */).await;
//!    let server = TestServer::new(app).unwrap();
//!    ```
//!
//! 2. **Test HTTP Client**
//!    ```rust
//!    use reqwest::Client;
//!    let client = Client::new();
//!
//!    let response = client
//!        .post(server.url("/api/documents"))
//!        .json(&document_request)
//!        .send()
//!        .await;
//!    ```
//!
//! 3. **Authentication Mocking**
//!    ```rust
//!    // Add test auth header
//!    let response = client
//!        .post(server.url("/api/approve"))
//!        .header("Authorization", format!("Bearer {}", test_token))
//!        .json(&approval_request)
//!        .send()
//!        .await;
//!    ```
//!
//! 4. **Database Test Container**
//!    ```rust
//!    use testcontainers::clients::cmd::CmdClient;
//!    let postgres = CmdClient::default().run(Postgres::default());
//!    ```

use chrono::{DateTime, Utc};
use uuid::Uuid;
use iou_core::{
    workflows::{
        ApprovalStage, Approver, ApprovalType, ExpiryAction,
        StageInstance, StageStatus, ApprovalResponse, ApprovalDecision,
    },
    delegation::{Delegation, DelegationType, ResolvedApprover},
};

/// Test user representation
#[derive(Debug, Clone)]
pub struct TestUser {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub auth_key: String,
}

impl TestUser {
    pub fn new(id: Uuid, name: &str, role: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            role: role.to_string(),
            auth_key: format!("test_auth_{}", id),
        }
    }

    pub fn manager() -> Self {
        Self::new(Uuid::new_v4(), "Test Manager", "manager")
    }

    pub fn director() -> Self {
        Self::new(Uuid::new_v4(), "Test Director", "director")
    }

    pub fn reviewer() -> Self {
        Self::new(Uuid::new_v4(), "Test Reviewer", "reviewer")
    }

    pub fn admin() -> Self {
        Self::new(Uuid::new_v4(), "Test Admin", "admin")
    }
}

/// Creates a test approval stage
pub fn create_test_stage(
    stage_id: &str,
    stage_name: &str,
    stage_order: i32,
    approval_type: ApprovalType,
    approvers: Vec<Uuid>,
    sla_hours: i32,
) -> ApprovalStage {
    let approver_list = approvers
        .into_iter()
        .map(|user_id| Approver {
            user_id: Some(user_id),
            role: None,
        })
        .collect();

    ApprovalStage {
        stage_id: stage_id.to_string(),
        stage_name: stage_name.to_string(),
        stage_order,
        approval_type,
        approvers: approver_list,
        sla_hours,
        expiry_action: ExpiryAction::NotifyOnly,
        is_optional: false,
        condition: None,
    }
}

/// Creates a test stage instance for a document
pub fn create_test_stage_instance(
    document_id: Uuid,
    stage_id: &str,
    approvers: Vec<Uuid>,
) -> StageInstance {
    StageInstance::new(document_id, stage_id.to_string(), approvers)
}

/// Creates a test approval response
pub fn create_test_approval(
    approver_id: Uuid,
    decision: ApprovalDecision,
    comment: Option<String>,
) -> ApprovalResponse {
    ApprovalResponse::new(approver_id, decision, comment)
}

/// Creates a test delegation
pub fn create_test_delegation(
    from_user_id: Uuid,
    to_user_id: Uuid,
    document_types: Vec<String>,
    created_by: Uuid,
) -> Delegation {
    let now = Utc::now();
    Delegation {
        id: Uuid::new_v4(),
        from_user_id,
        to_user_id,
        delegation_type: DelegationType::Temporary,
        document_types,
        document_id: None,
        starts_at: now,
        ends_at: Some(now + chrono::Duration::hours(24)),
        is_active: true,
        created_at: now,
        created_by,
    }
}

/// Creates a test time offset from now
pub fn hours_from_now(hours: i64) -> DateTime<Utc> {
    Utc::now() + chrono::Duration::hours(hours)
}

/// Creates a test time offset from now in days
pub fn days_from_now(days: i64) -> DateTime<Utc> {
    Utc::now() + chrono::Duration::days(days)
}

/// Helper to verify stage status
pub fn assert_stage_status(instance: &StageInstance, expected: StageStatus) {
    assert_eq!(
        instance.status,
        expected,
        "Expected stage status {:?}, got {:?}",
        expected,
        instance.status
    );
}

/// Helper to verify approval count
pub fn assert_approval_count(instance: &StageInstance, expected: usize) {
    assert_eq!(
        instance.approvals_received.len(),
        expected,
        "Expected {} approvals, got {}",
        expected,
        instance.approvals_received.len()
    );
}

/// Helper to verify stage completion
pub fn assert_stage_complete(instance: &StageInstance, approval_type: ApprovalType) {
    assert!(
        instance.is_complete(approval_type),
        "Expected stage to be complete with approval type {:?}",
        approval_type
    );
}

/// Helper to verify stage not complete
pub fn assert_stage_not_complete(instance: &StageInstance, approval_type: ApprovalType) {
    assert!(
        !instance.is_complete(approval_type),
        "Expected stage to not be complete with approval type {:?}",
        approval_type
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_user_creation() {
        let user = TestUser::manager();
        assert_eq!(user.role, "manager");
        assert!(!user.auth_key.is_empty());
    }

    #[test]
    fn test_create_test_stage() {
        let approvers = vec![Uuid::new_v4(), Uuid::new_v4()];
        let stage = create_test_stage(
            "test_stage",
            "Test Stage",
            1,
            ApprovalType::ParallelAll,
            approvers.clone(),
            24,
        );

        assert_eq!(stage.stage_id, "test_stage");
        assert_eq!(stage.approvers.len(), 2);
    }

    #[test]
    fn test_create_test_stage_instance() {
        let document_id = Uuid::new_v4();
        let approvers = vec![Uuid::new_v4(), Uuid::new_v4()];
        let instance = create_test_stage_instance(document_id, "test_stage", approvers);

        assert_eq!(instance.document_id, document_id);
        assert_eq!(instance.status, StageStatus::Pending);
    }

    #[test]
    fn test_hours_from_now() {
        let time = hours_from_now(2);
        let now = Utc::now();
        let diff = time.signed_duration_since(now).num_hours();
        assert!(diff >= 1 && diff <= 3); // Allow some tolerance
    }
}
