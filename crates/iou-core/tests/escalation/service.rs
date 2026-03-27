use iou_core::escalation::{
    EscalationService, EscalationType, NotificationChannel,
    EscalationThresholds, ExpiryAction, StageDeadlineInfo,
};
use iou_core::sla::SlaCalculator;
use iou_core::realtime::{RealtimeClient, RealtimeConfig};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

fn create_test_service() -> EscalationService {
    let sla_calculator = SlaCalculator::new();

    let config = RealtimeConfig {
        websocket_url: "ws://localhost:4000/socket/websocket".to_string(),
        ..Default::default()
    };
    let rt_client = Arc::new(RealtimeClient::new(config));

    EscalationService::new(sla_calculator, rt_client)
}

fn create_test_stage_deadline(deadline: DateTime<Utc>) -> StageDeadlineInfo {
    StageDeadlineInfo {
        document_id: Uuid::new_v4(),
        stage_instance_id: Uuid::new_v4(),
        stage_id: "test_stage".to_string(),
        stage_name: "Test Review".to_string(),
        deadline,
        approvers: vec![Uuid::new_v4()],
    }
}

fn create_test_stage_business_hours(hours_until_deadline: i32) -> StageDeadlineInfo {
    // Use SLA calculator to create deadline with business hours
    let calculator = SlaCalculator::new();
    let deadline = calculator.calculate_deadline(Utc::now(), hours_until_deadline);

    StageDeadlineInfo {
        document_id: Uuid::new_v4(),
        stage_instance_id: Uuid::new_v4(),
        stage_id: "test_stage".to_string(),
        stage_name: "Test Review".to_string(),
        deadline,
        approvers: vec![Uuid::new_v4()],
    }
}

#[tokio::test]
async fn test_check_overdue_stages_with_24h_remaining() {
    let service = create_test_service();

    let stage = create_test_stage_business_hours(24);
    let stages = vec![stage.clone()];

    let escalations = service.check_overdue_stages(stages).await;

    // Should trigger approaching deadline escalation at 24h
    assert!(!escalations.is_empty());
    assert_eq!(escalations[0].escalation_type, EscalationType::ApproachingDeadline);
    assert_eq!(escalations[0].stage_info.stage_name, "Test Review");
}

#[tokio::test]
async fn test_check_overdue_stages_with_8h_remaining() {
    let service = create_test_service();

    let stage = create_test_stage_business_hours(8);
    let stages = vec![stage];

    let escalations = service.check_overdue_stages(stages).await;

    // Should trigger approaching deadline escalation at 8h
    assert!(!escalations.is_empty());
    assert_eq!(escalations[0].escalation_type, EscalationType::ApproachingDeadline);
}

#[tokio::test]
async fn test_check_overdue_stages_expired() {
    let service = create_test_service();

    // Create a deadline 1 hour in the past
    let past_deadline = Utc::now() - chrono::Duration::hours(1);
    let stage = create_test_stage_deadline(past_deadline);
    let stages = vec![stage];

    let escalations = service.check_overdue_stages(stages).await;

    // Should trigger deadline exceeded
    assert!(!escalations.is_empty());
    assert_eq!(escalations[0].escalation_type, EscalationType::DeadlineExceeded);
}

#[tokio::test]
async fn test_check_overdue_stages_no_escalation_needed() {
    let service = create_test_service();

    let stage = create_test_stage_business_hours(72); // Well outside warning window
    let stages = vec![stage];

    let escalations = service.check_overdue_stages(stages).await;

    // Should not trigger escalation
    assert!(escalations.is_empty());
}

#[tokio::test]
async fn test_escalation_type_serialization() {
    let warning = EscalationType::ApproachingDeadline;
    let json = serde_json::to_string(&warning).unwrap();
    assert!(json.contains("approaching_deadline"));

    let exceeded = EscalationType::DeadlineExceeded;
    let json = serde_json::to_string(&exceeded).unwrap();
    assert!(json.contains("deadline_exceeded"));
}

#[tokio::test]
async fn test_notification_channel_websocket() {
    let ws = NotificationChannel::WebSocket;
    let json = serde_json::to_string(&ws).unwrap();
    // snake_case of WebSocket is "web_socket", not "websocket"
    assert!(json.contains("web_socket"));
}

#[tokio::test]
async fn test_notification_channel_webhook() {
    let webhook = NotificationChannel::Webhook {
        url: "https://example.com/hook".to_string(),
    };
    let json = serde_json::to_string(&webhook).unwrap();
    assert!(json.contains("webhook"));
    assert!(json.contains("https://example.com/hook"));
}

#[tokio::test]
async fn test_expiry_action_notify_only() {
    let action = ExpiryAction::NotifyOnly;
    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("notify_only"));
}

#[tokio::test]
async fn test_expiry_action_return_to_draft() {
    let action = ExpiryAction::ReturnToDraft;
    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("return_to_draft"));
}

#[tokio::test]
async fn test_expiry_action_auto_approve() {
    let action = ExpiryAction::AutoApprove;
    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("auto_approve"));
}

#[tokio::test]
async fn test_expiry_action_escalate_to() {
    let action = ExpiryAction::EscalateTo {
        target: "manager".to_string(),
    };
    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("escalate_to"));
    assert!(json.contains("manager"));
}

#[tokio::test]
async fn test_escalation_thresholds_default() {
    let thresholds = EscalationThresholds::default();

    assert_eq!(thresholds.warning_hours, vec![24, 8, 1]);
    assert!(matches!(thresholds.expiry_action, ExpiryAction::NotifyOnly));
}
