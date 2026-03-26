//! Escalation service module
//!
//! Monitors approval deadlines and sends notifications through
//! multiple channels. Server-only (requires tokio/async).

#[cfg(feature = "server")]
mod service;

#[cfg(feature = "server")]
pub use service::{
    EscalationService,
    EscalationType,
    NotificationChannel,
    EscalationMessage,
    EscalationRecord,
    EscalationStatus,
    EscalationThresholds,
    ExpiryAction,
    PendingEscalation,
    StageDeadlineInfo,
    EscalationError,
};
