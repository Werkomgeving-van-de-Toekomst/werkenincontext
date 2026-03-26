//! Workflow stage tracker component
//!
//! Visualizes multi-stage approval workflow progress as a horizontal stepper.
//! Shows completed stages, current stage in progress, and pending stages.

use dioxus::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use crate::api::documents::{get_document_stages, StageView as ApiStageView, ApproverView as ApiApproverView};

/// Main workflow stage tracker component
#[component]
pub fn WorkflowStageTracker(
    document_id: Uuid,
    #[props(default)] current_stage_id: Option<String>,
    #[props(default)] on_stage_click: Option<EventHandler<String>>,
) -> Element {
    let stages = use_resource(move || {
        let doc_id = document_id;
        async move {
            get_document_stages(doc_id).await
                .map(|stages| stages.into_iter().map(StageView::from).collect())
                .unwrap_or(Vec::new())
        }
    });

    let stages_data = stages.read().clone().unwrap_or_default();

    // Pre-compute stage items for rendering
    let stage_items: Vec<StageItem> = stages_data.iter().map(|stage| {
        let stage_id = stage.stage_id.clone();
        let is_current = current_stage_id.as_ref() == Some(&stage_id);
        let has_delegation = stage.delegated_approvers();
        let has_escalation = !stage.escalations.is_empty();
        let approver_initials: Vec<String> = stage.approvers.iter()
            .take(3)
            .map(|a| a.name.chars().next().unwrap_or('?').to_string())
            .collect();
        let remaining_approvers = stage.approvers.len().saturating_sub(3);
        let deadline = stage.deadline;
        let status = stage.status;
        let stage_name = stage.stage_name.clone();

        StageItem {
            stage_id,
            stage_name,
            is_current,
            status,
            deadline,
            has_delegation,
            has_escalation,
            escalation_count: stage.escalations.len(),
            approver_initials,
            remaining_approvers,
        }
    }).collect();

    rsx! {
        div { class: "stage-tracker",
            {stage_items.iter().map(|item| {
                let current_class = if item.is_current { " current" } else { "" };
                let status_class = match item.status {
                    StageStatus::Completed => " completed",
                    StageStatus::InProgress => " in-progress",
                    _ => " pending",
                };

                rsx!(
                    div {
                        class: "stage-step{current_class}{status_class}",
                        key: "{item.stage_id}",

                        // Stage icon
                        div { class: "stage-icon",
                            {get_stage_icon(item.status)}
                        }

                        // Stage name
                        div { class: "stage-name",
                            "{item.stage_name}"
                        }

                        // Countdown timer for in-progress stages
                        if item.is_current || matches!(item.status, StageStatus::InProgress) {
                            if let Some(deadline) = item.deadline {
                                CountdownTimer {
                                    deadline: deadline,
                                    status: item.status,
                                }
                            }
                        }

                        // Delegation badge
                        if item.has_delegation {
                            div { class: "delegation-badge-mini",
                                "Delegatie"
                            }
                        }

                        // Escalation icon
                        if item.has_escalation {
                            div { class: "escalation-badge-mini",
                                svg {
                                    class: "escalation-icon",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    "stroke-width": "2",
                                    path { d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" }
                                }
                                span { "{item.escalation_count}" }
                            }
                        }

                        // Approver avatars
                        if let Some(_) = on_stage_click {
                            div {
                                class: "approvers-avatars",
                                {item.approver_initials.iter().map(|initial| rsx! {
                                    div {
                                        class: "avatar",
                                        title: "{initial}",
                                        "{initial}"
                                    }
                                })}
                                {if item.remaining_approvers > 0 {
                                    rsx! { div { class: "avatar more", "+{item.remaining_approvers}" } }
                                } else {
                                    rsx! {}
                                }}
                            }
                        }
                    }
                )
            })}
        }
    }
}

/// Countdown timer component for stage deadlines
#[component]
fn CountdownTimer(
    deadline: DateTime<Utc>,
    status: StageStatus,
) -> Element {
    // Calculate remaining time directly without continuous updates
    // The timer will update when the parent component re-renders
    let now = Utc::now();
    let remaining_seconds = if deadline > now {
        deadline.signed_duration_since(now).num_seconds()
    } else {
        0
    };

    let (time_text, urgency_class) = if remaining_seconds <= 0 {
        let overdue = -remaining_seconds;
        let text = if overdue >= 86400 {
            format!("{}d overdue", overdue / 86400)
        } else if overdue >= 3600 {
            format!("{}h overdue", overdue / 3600)
        } else {
            format!("{}m overdue", overdue / 60)
        };
        (text, "overdue")
    } else {
        let text = if remaining_seconds >= 86400 {
            format!("{}d {}h left", remaining_seconds / 86400, (remaining_seconds % 86400) / 3600)
        } else if remaining_seconds >= 3600 {
            format!("{}h {}m left", remaining_seconds / 3600, (remaining_seconds % 3600) / 60)
        } else {
            format!("{}m left", remaining_seconds / 60)
        };
        let urgency = if remaining_seconds >= 86400 {
            "healthy"
        } else if remaining_seconds >= 28800 {
            "warning"
        } else {
            "critical"
        };
        (text, urgency)
    };

    rsx! {
        div { class: "countdown-timer {urgency_class}",
            svg {
                class: "timer-icon",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                "stroke-width": "2",
                circle { cx: "12", cy: "12", r: "10" }
                path { d: "M12 6v6l4 2" }
            }
            span { class: "timer-text", "{time_text}" }
        }
    }
}

/// Get stage icon based on status
fn get_stage_icon(status: StageStatus) -> Element {
    match status {
        StageStatus::Completed => rsx! {
            svg {
                class: "icon-check",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                "stroke-width": "2",
                path { d: "M20 6L9 17l-5-5" }
            }
        },
        StageStatus::InProgress => rsx! {
            svg {
                class: "icon-spinner spin",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                "stroke-width": "2",
                circle { cx: "12", cy: "12", r: "10", "stroke-opacity": "0.25" }
                path { d: "M12 2a10 10 0 0 1 10 10" }
            }
        },
        _ => rsx! {
            svg {
                class: "icon-circle",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                "stroke-width": "2",
                circle { cx: "12", cy: "12", r: "10" }
            }
        },
    }
}

// ==============================================================================
// Types
// ==============================================================================

/// Stage item for rendering
#[derive(Clone, Debug)]
struct StageItem {
    stage_id: String,
    stage_name: String,
    is_current: bool,
    status: StageStatus,
    deadline: Option<DateTime<Utc>>,
    has_delegation: bool,
    has_escalation: bool,
    escalation_count: usize,
    approver_initials: Vec<String>,
    remaining_approvers: usize,
}

/// Stage view for frontend display
#[derive(Clone, PartialEq, Debug)]
pub struct StageView {
    pub stage_id: String,
    pub stage_name: String,
    pub stage_order: i32,
    pub status: StageStatus,
    pub deadline: Option<DateTime<Utc>>,
    pub approvers: Vec<ApproverView>,
    pub escalations: Vec<EscalationView>,
}

impl StageView {
    /// Get list of delegated approvers
    fn delegated_approvers(&self) -> bool {
        self.approvers.iter().any(|a| a.delegated_from.is_some())
    }
}

impl From<ApiStageView> for StageView {
    fn from(api: ApiStageView) -> Self {
        let status = match api.status.as_str() {
            "completed" | "approved" => StageStatus::Completed,
            "in_progress" | "active" => StageStatus::InProgress,
            "pending" => StageStatus::Pending,
            "skipped" => StageStatus::Skipped,
            "expired" | "rejected" => StageStatus::Expired,
            _ => StageStatus::Pending,
        };

        let approvers = api.approvers.into_iter()
            .map(ApproverView::from)
            .collect();

        // Note: Escalations are not included in the StageView list API response.
        // They would need to be fetched from get_stage_detail() which returns
        // StageDetailView with escalation_count and current_escalation_level.
        // For list display, we initialize with an empty vector.
        StageView {
            stage_id: api.stage_id,
            stage_name: api.stage_name,
            stage_order: api.stage_order,
            status,
            deadline: api.deadline.and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            approvers,
            escalations: Vec::new(), // Populate from detail view if needed
        }
    }
}

/// Approver information
#[derive(Clone, PartialEq, Debug)]
pub struct ApproverView {
    pub user_id: Uuid,
    pub name: String,
    pub status: String,
    pub responded_at: Option<DateTime<Utc>>,
    pub delegated_from: Option<Uuid>,
    pub delegated_from_name: Option<String>,
}

impl From<ApiApproverView> for ApproverView {
    fn from(api: ApiApproverView) -> Self {
        ApproverView {
            user_id: api.user_id,
            name: api.user_name.unwrap_or_else(|| "Onbekend".to_string()),
            status: api.status,
            responded_at: api.responded_at.and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            delegated_from: api.delegated_from,
            delegated_from_name: None,
        }
    }
}

/// Escalation view
#[derive(Clone, PartialEq, Debug)]
pub struct EscalationView {
    pub escalation_level: i32,
    pub escalated_at: DateTime<Utc>,
    pub escalated_to: Option<String>,
}

/// Stage status
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StageStatus {
    Completed,
    InProgress,
    Pending,
    Skipped,
    Expired,
}
