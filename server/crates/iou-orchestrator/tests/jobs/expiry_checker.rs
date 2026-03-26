use iou_orchestrator::jobs::{ExpiryChecker, ExpiryCheckerConfig};
use iou_core::escalation::{EscalationService, NotificationChannel};
use iou_core::sla::SlaCalculator;
use iou_core::realtime::{RealtimeClient, RealtimeConfig};
use std::sync::Arc;

#[tokio::test]
async fn test_expiry_checker_config_default() {
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
    let rt_client = Arc::new(RealtimeClient::new(rt_config));

    let escalation_service = EscalationService::new(sla_calculator, rt_client);
    let checker = ExpiryChecker::with_defaults(escalation_service);

    assert_eq!(checker.config().check_interval_secs, 300);
}

#[tokio::test]
async fn test_run_once_with_no_stages() {
    let sla_calculator = SlaCalculator::new();

    let rt_config = RealtimeConfig {
        websocket_url: "ws://localhost:4000/socket/websocket".to_string(),
        ..Default::default()
    };
    let rt_client = Arc::new(RealtimeClient::new(rt_config));

    let escalation_service = EscalationService::new(sla_calculator, rt_client);
    let checker = ExpiryChecker::with_defaults(escalation_service);

    // Should not error with no stages
    let result = checker.run_once().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_expiry_checker_custom_config() {
    let sla_calculator = SlaCalculator::new();

    let rt_config = RealtimeConfig {
        websocket_url: "ws://localhost:4000/socket/websocket".to_string(),
        ..Default::default()
    };
    let rt_client = Arc::new(RealtimeClient::new(rt_config));

    let escalation_service = EscalationService::new(sla_calculator, rt_client);

    let config = ExpiryCheckerConfig {
        check_interval_secs: 60,
        notification_channels: vec![NotificationChannel::WebSocket],
        execute_actions: false,
    };

    let checker = ExpiryChecker::new(config, escalation_service);

    assert_eq!(checker.config().check_interval_secs, 60);
    assert_eq!(checker.config().notification_channels.len(), 1);
    assert!(!checker.config().execute_actions);
}

#[tokio::test]
async fn test_expiry_checker_with_webhook_channel() {
    let sla_calculator = SlaCalculator::new();

    let rt_config = RealtimeConfig {
        websocket_url: "ws://localhost:4000/socket/websocket".to_string(),
        ..Default::default()
    };
    let rt_client = Arc::new(RealtimeClient::new(rt_config));

    let escalation_service = EscalationService::new(sla_calculator, rt_client);

    let config = ExpiryCheckerConfig {
        check_interval_secs: 300,
        notification_channels: vec![
            NotificationChannel::Webhook {
                url: "https://example.com/escalations".to_string(),
            },
        ],
        execute_actions: true,
    };

    let checker = ExpiryChecker::new(config, escalation_service);

    assert_eq!(checker.config().notification_channels.len(), 1);
}
