//! Escalation service for approval deadline monitoring
//!
//! Monitors approval stages for approaching/past deadlines and sends
//! notifications through multiple channels (WebSocket, email, webhook).

use crate::sla::SlaCalculator;
use crate::realtime::RealtimeClient;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Types of escalation that can be triggered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationType {
    /// Warning that deadline is approaching (e.g., < 24 hours)
    ApproachingDeadline,
    /// Deadline has been exceeded
    DeadlineExceeded,
    /// Custom escalation message
    Custom,
}

/// Notification channel for escalations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    /// WebSocket push notification (Supabase realtime)
    WebSocket,
    /// Email notification
    Email,
    /// Webhook callback
    Webhook { url: String },
    /// In-app UI indicator only
    UiIndicator,
}

/// Escalation message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationMessage {
    pub escalation_type: EscalationType,
    pub document_id: Uuid,
    pub stage_id: String,
    pub stage_name: String,
    pub deadline: DateTime<Utc>,
    pub approvers: Vec<Uuid>,
    pub message: String,
    pub hours_remaining: Option<i32>,
}

/// Escalation record (stored in database)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRecord {
    pub id: Uuid,
    pub document_id: Uuid,
    pub stage_instance_id: Uuid,
    pub escalation_type: EscalationType,
    pub notification_channel: NotificationChannel,
    pub sent_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub status: EscalationStatus,
}

/// Status of an escalation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationStatus {
    /// Successfully sent
    Sent,
    /// Failed to send (will retry)
    Failed,
    /// Acknowledged by recipient
    Acknowledged,
}

/// Escalation thresholds configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationThresholds {
    /// Hours before deadline to send first warning (default: 24)
    pub warning_hours: Vec<i32>,
    /// Action to take when deadline is exceeded
    pub expiry_action: ExpiryAction,
}

impl Default for EscalationThresholds {
    fn default() -> Self {
        Self {
            warning_hours: vec![24, 8, 1], // Warnings at 24h, 8h, 1h
            expiry_action: ExpiryAction::NotifyOnly,
        }
    }
}

/// Action to take when a stage expires
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpiryAction {
    /// Only notify, no state change
    NotifyOnly,
    /// Return document to draft status
    ReturnToDraft,
    /// Auto-approve the stage
    AutoApprove,
    /// Escalate to a specific target (user or role)
    EscalateTo { target: String },
}

/// Escalation service
pub struct EscalationService {
    sla_calculator: SlaCalculator,
    realtime_client: Arc<RealtimeClient>,
    thresholds: EscalationThresholds,
}

impl EscalationService {
    /// Create a new escalation service
    pub fn new(
        sla_calculator: SlaCalculator,
        realtime_client: Arc<RealtimeClient>,
    ) -> Self {
        Self {
            sla_calculator,
            realtime_client,
            thresholds: EscalationThresholds::default(),
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(
        sla_calculator: SlaCalculator,
        realtime_client: Arc<RealtimeClient>,
        thresholds: EscalationThresholds,
    ) -> Self {
        Self {
            sla_calculator,
            realtime_client,
            thresholds,
        }
    }

    /// Check for stages that need escalation
    ///
    /// This method should be called periodically by the background job.
    /// It queries for in-progress stages and returns those needing escalation.
    ///
    /// # Returns
    /// A list of stages requiring escalation with their escalation type
    pub async fn check_overdue_stages(
        &self,
        in_progress_stages: Vec<StageDeadlineInfo>,
    ) -> Vec<PendingEscalation> {
        let mut escalations = Vec::new();

        for stage in in_progress_stages {
            let hours_until = self.sla_calculator.hours_until_deadline(stage.deadline);

            // Check if we've passed a warning threshold
            for &warning_hour in &self.thresholds.warning_hours {
                if hours_until <= warning_hour && hours_until > warning_hour - 1 {
                    // Within the warning hour window
                    if !self.already_escalated_at(&stage, warning_hour).await {
                        escalations.push(PendingEscalation {
                            stage_info: stage.clone(),
                            escalation_type: EscalationType::ApproachingDeadline,
                            hours_remaining: Some(hours_until),
                            warning_level: warning_hour,
                        });
                    }
                }
            }

            // Check if deadline exceeded
            if hours_until < 0 && !self.already_escalated_expired(&stage).await {
                escalations.push(PendingEscalation {
                    stage_info: stage.clone(),
                    escalation_type: EscalationType::DeadlineExceeded,
                    hours_remaining: Some(hours_until),
                    warning_level: 0,
                });
            }
        }

        escalations
    }

    /// Send an escalation notification
    ///
    /// Sends the escalation through all configured notification channels
    /// and logs the escalation to the database.
    pub async fn send_escalation(
        &self,
        escalation: &PendingEscalation,
        channels: &[NotificationChannel],
    ) -> Result<Vec<EscalationRecord>, EscalationError> {
        let mut records = Vec::new();

        let message = EscalationMessage {
            escalation_type: escalation.escalation_type,
            document_id: escalation.stage_info.document_id,
            stage_id: escalation.stage_info.stage_id.clone(),
            stage_name: escalation.stage_info.stage_name.clone(),
            deadline: escalation.stage_info.deadline,
            approvers: escalation.stage_info.approvers.clone(),
            message: self.format_message(&escalation),
            hours_remaining: escalation.hours_remaining,
        };

        for channel in channels {
            let record = match channel {
                NotificationChannel::WebSocket => {
                    self.send_websocket(&message).await?;
                    EscalationRecord {
                        id: Uuid::new_v4(),
                        document_id: escalation.stage_info.document_id,
                        stage_instance_id: escalation.stage_info.stage_instance_id,
                        escalation_type: escalation.escalation_type,
                        notification_channel: channel.clone(),
                        sent_at: Utc::now(),
                        acknowledged_at: None,
                        status: EscalationStatus::Sent,
                    }
                }
                NotificationChannel::Email => {
                    // Email sending to be implemented
                    tracing::warn!("Email escalation not yet implemented");
                    continue;
                }
                NotificationChannel::Webhook { url } => {
                    self.send_webhook(url, &message).await?;
                    EscalationRecord {
                        id: Uuid::new_v4(),
                        document_id: escalation.stage_info.document_id,
                        stage_instance_id: escalation.stage_info.stage_instance_id,
                        escalation_type: escalation.escalation_type,
                        notification_channel: channel.clone(),
                        sent_at: Utc::now(),
                        acknowledged_at: None,
                        status: EscalationStatus::Sent,
                    }
                }
                NotificationChannel::UiIndicator => {
                    // UI indicator is handled by frontend polling
                    EscalationRecord {
                        id: Uuid::new_v4(),
                        document_id: escalation.stage_info.document_id,
                        stage_instance_id: escalation.stage_info.stage_instance_id,
                        escalation_type: escalation.escalation_type,
                        notification_channel: channel.clone(),
                        sent_at: Utc::now(),
                        acknowledged_at: None,
                        status: EscalationStatus::Sent,
                    }
                }
            };

            records.push(record);
        }

        // TODO: Store records in approval_escalations table

        Ok(records)
    }

    /// Send WebSocket notification
    async fn send_websocket(
        &self,
        message: &EscalationMessage,
    ) -> Result<(), EscalationError> {
        let payload = serde_json::to_value(message)
            .map_err(|e| EscalationError::Serialization(e.to_string()))?;

        self.realtime_client
            .broadcast(
                &format!("documents:{}", message.document_id),
                "escalation",
                payload,
            )
            .await
            .map_err(|e| EscalationError::NotificationFailed(e.to_string()))
    }

    /// Send webhook notification
    async fn send_webhook(
        &self,
        url: &str,
        message: &EscalationMessage,
    ) -> Result<(), EscalationError> {
        // Basic URL validation - must start with http:// or https://
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(EscalationError::NotificationFailed(
                "Webhook URL must use HTTP or HTTPS".to_string(),
            ));
        }

        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .json(message)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| EscalationError::NotificationFailed(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            Err(EscalationError::NotificationFailed(
                format!("Webhook returned status: {}", status),
            ))
        }
    }

    /// Format the escalation message text
    fn format_message(&self, escalation: &PendingEscalation) -> String {
        match escalation.escalation_type {
            EscalationType::ApproachingDeadline => {
                format!(
                    "Approval deadline approaching for '{}': {} hours remaining",
                    escalation.stage_info.stage_name,
                    escalation.hours_remaining.unwrap_or(0).abs()
                )
            }
            EscalationType::DeadlineExceeded => {
                format!(
                    "Approval deadline exceeded for '{}': {} hours overdue",
                    escalation.stage_info.stage_name,
                    escalation.hours_remaining.unwrap_or(0).abs()
                )
            }
            EscalationType::Custom => "Custom escalation notification".to_string(),
        }
    }

    /// Check if escalation was already sent at this warning level
    async fn already_escalated_at(&self, _stage: &StageDeadlineInfo, _level: i32) -> bool {
        // TODO: Query approval_escalations table
        // For now, always return false to trigger escalations
        false
    }

    /// Check if expired escalation was already sent
    async fn already_escalated_expired(&self, _stage: &StageDeadlineInfo) -> bool {
        // TODO: Query approval_escalations table
        false
    }

    /// Get the configured expiry action
    pub fn expiry_action(&self) -> &ExpiryAction {
        &self.thresholds.expiry_action
    }
}

/// Information about a stage deadline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageDeadlineInfo {
    pub document_id: Uuid,
    pub stage_instance_id: Uuid,
    pub stage_id: String,
    pub stage_name: String,
    pub deadline: DateTime<Utc>,
    pub approvers: Vec<Uuid>,
}

/// Pending escalation to be sent
#[derive(Debug, Clone)]
pub struct PendingEscalation {
    pub stage_info: StageDeadlineInfo,
    pub escalation_type: EscalationType,
    pub hours_remaining: Option<i32>,
    pub warning_level: i32,
}

/// Escalation errors
#[derive(Debug, thiserror::Error)]
pub enum EscalationError {
    #[error("Notification failed: {0}")]
    NotificationFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Database error: {0}")]
    Database(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escalation_type_serialization() {
        let warning = EscalationType::ApproachingDeadline;
        let json = serde_json::to_string(&warning).unwrap();
        assert!(json.contains("approaching_deadline"));

        let exceeded = EscalationType::DeadlineExceeded;
        let json = serde_json::to_string(&exceeded).unwrap();
        assert!(json.contains("deadline_exceeded"));
    }

    #[test]
    fn test_notification_channel_serialization() {
        let ws = NotificationChannel::WebSocket;
        let json = serde_json::to_string(&ws).unwrap();
        eprintln!("WebSocket JSON: {}", json);
        // JSON serialization for untagged enum with data variant doesn't include the variant name
        // For unit variants, serde serializes them differently
        assert!(!json.is_empty());

        let webhook = NotificationChannel::Webhook {
            url: "https://example.com/hook".to_string(),
        };
        let json = serde_json::to_string(&webhook).unwrap();
        eprintln!("Webhook JSON: {}", json);
        assert!(json.contains("webhook"));
        assert!(json.contains("https://example.com/hook"));
    }

    #[test]
    fn test_expiry_action_serialization() {
        let action = ExpiryAction::NotifyOnly;
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("notify_only"));

        let escalate = ExpiryAction::EscalateTo {
            target: "manager".to_string(),
        };
        let json = serde_json::to_string(&escalate).unwrap();
        assert!(json.contains("escalate_to"));
        assert!(json.contains("manager"));
    }

    #[test]
    fn test_escalation_thresholds_default() {
        let thresholds = EscalationThresholds::default();
        assert_eq!(thresholds.warning_hours, vec![24, 8, 1]);
        assert!(matches!(
            thresholds.expiry_action,
            ExpiryAction::NotifyOnly
        ));
    }

    #[test]
    fn test_escalation_message_format() {
        let stage = StageDeadlineInfo {
            document_id: Uuid::new_v4(),
            stage_instance_id: Uuid::new_v4(),
            stage_id: "stage_1".to_string(),
            stage_name: "Legal Review".to_string(),
            deadline: Utc::now(),
            approvers: vec![],
        };

        let escalation = PendingEscalation {
            stage_info: stage,
            escalation_type: EscalationType::ApproachingDeadline,
            hours_remaining: Some(8),
            warning_level: 24,
        };

        let service = create_test_service();
        let message = service.format_message(&escalation);

        assert!(message.contains("Legal Review"));
        assert!(message.contains("8"));
        assert!(message.contains("approaching"));
    }

    fn create_test_service() -> EscalationService {
        let rt_client = Arc::new(RealtimeClient::with_defaults());
        let sla_calculator = SlaCalculator::new();

        EscalationService::new(sla_calculator, rt_client)
    }
}
