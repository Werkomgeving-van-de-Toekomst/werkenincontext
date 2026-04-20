//! Workflow Analytics Dashboard component
//!
//! Displays workflow performance metrics and insights:
//! - SLA compliance rates
//! - Average completion times
//! - Bottleneck identification
//! - Stage-level breakdowns

use dioxus::prelude::*;
use uuid::Uuid;
use std::collections::HashMap;

/// Workflow analytics dashboard
#[component]
pub fn WorkflowAnalyticsDashboard(
    workflow_id: Uuid,
    #[props(default)] domain_id: String,
) -> Element {
    let analytics = use_resource(move || {
        let wf_id = workflow_id;
        async move {
            // Fetch analytics from API
            fetch_workflow_analytics(wf_id).await
        }
    });

    rsx! {
        div { class: "workflow-analytics-dashboard",
            h2 { class: "dashboard-title", "Workflow Analytics" }

            {analytics.read().as_ref().map(|data| rsx! {
                div { class: "dashboard-content",

                    // Summary cards
                    div { class: "summary-cards",
                        SummaryCard {
                            title: "SLA Naleving",
                            value: format!("{:.1}%", data.sla_compliance_pct),
                            trend: Some(data.sla_trend),
                            icon: "✓",
                            color: if data.sla_compliance_pct >= 90.0 { "green" }
                                else if data.sla_compliance_pct >= 75.0 { "yellow" }
                                else { "red" },
                        }
                        SummaryCard {
                            title: "Gem. Doorlooptijd",
                            value: format!("{:.1}u", data.avg_completion_hours),
                            trend: Some(data.time_trend),
                            icon: "⏱",
                            color: if data.avg_completion_hours <= 48.0 { "green" }
                                else if data.avg_completion_hours <= 72.0 { "yellow" }
                                else { "red" },
                        }
                        SummaryCard {
                            title: "Totale Uitvoeringen",
                            value: format!("{}", data.total_executions),
                            trend: None,
                            icon: "📊",
                            color: "blue",
                        }
                        SummaryCard {
                            title: "Knelpunt Fase",
                            value: data.bottleneck_stage.clone().unwrap_or_else(|| "Geen".to_string()),
                            trend: None,
                            icon: "⚠️",
                            color: "orange",
                        }
                    }

                    // Charts section
                    div { class: "charts-section",
                        // SLA Compliance Chart
                        div { class: "chart-card",
                            h3 { "SLA Naleving per Fase" }
                            SLAChart { stage_data: data.stage_insights.clone() }
                        }

                        // Completion Time Chart
                        div { class: "chart-card",
                            h3 { "Doorlooptijd per Fase" }
                            DurationChart { stage_data: data.stage_insights.clone() }
                        }
                    }

                    // Stage breakdown table
                    div { class: "stage-breakdown",
                        h3 { "Fase Details" }
                        table { class: "analytics-table",
                            thead {
                                tr {
                                    th { "Fase" }
                                    th { "Status" }
                                    th { "Gem. Tijd" }
                                    th { "SLA %" }
                                    th { "Risico" }
                                }
                            }
                            tbody {
                                {data.stage_insights.iter().map(|stage| rsx! {
                                    tr {
                                        td { "{stage.stage_name}" }
                                        td {
                                            span {
                                                class: format!("status-badge {}", stage.status.to_lowercase()),
                                                "{stage.status}"
                                            }
                                        }
                                        td {
                                            "{stage.avg_duration_hours.map_or('-'.to_string(), |h| format!("{:.1}u", h))}"
                                        }
                                        td {
                                            "{stage.sla_compliance_pct.map_or('-'.to_string(), |p| format!("{:.1}%", p))}"
                                        }
                                        td {
                                            RiskIndicator { risk_level: stage.risk_level }
                                        }
                                    }
                                })}
                            }
                        }
                    }

                    // Recommendations
                    if !data.recommendations.is_empty() {
                        div { class: "recommendations",
                            h3 { "Aanbevelingen" }
                            {data.recommendations.iter().map(|rec| rsx! {
                                div { class: "recommendation-card",
                                    div { class: "rec-icon", "{rec.icon}" }
                                    div { class: "rec-content",
                                        h4 { "{rec.title}" }
                                        p { "{rec.description}" }
                                        div { class: "rec-impact",
                                            span { "Verwachte impact: " }
                                            strong { "{rec.impact}" }
                                        }
                                    }
                                }
                            })}
                        }
                    }
                }
            }).unwrap_or_else(|| rsx! {
                div { class: "loading-state",
                    div { class: "spinner" }
                    p { "Analytics laden..." }
                }
            })}
        }
    }
}

/// Summary card component
#[component]
fn SummaryCard(
    title: String,
    value: String,
    #[props(default)] trend: Option<f32>,
    icon: String,
    color: String,
) -> Element {
    rsx! {
        div { class: format!("summary-card card-{}", color),
            div { class: "card-icon", "{icon}" }
            div { class: "card-content",
                div { class: "card-title", "{title}" }
                div { class: "card-value", "{value}" }
                {trend.map(|t| rsx! {
                    div {
                        class: if t >= 0.0 { "trend positive" } else { "trend negative" },
                        "{if t >= 0.0 { '↑' } else { '↓' }} {t.abs()}%"
                    }
                })}
            }
        }
    }
}

/// SLA compliance chart
#[component]
fn SLAChart(stage_data: Vec<StageInsightData>) -> Element {
    let max_width = 300;

    rsx! {
        div { class: "bar-chart",
            {stage_data.iter().map(|stage| rsx! {
                div { class: "chart-row",
                    div { class: "chart-label",
                        title: "{stage.stage_name}",
                        "{truncate_string(&stage.stage_name, 20)}"
                    }
                    div { class: "chart-bar-container",
                        div {
                            class: "chart-bar",
                            style: "width: {stage.sla_compliance_pct.unwrap_or(0.0) * max_width as f64 / 100.0}px; \
                                    background: {get_sla_color(stage.sla_compliance_pct.unwrap_or(0.0))};",
                            "{stage.sla_compliance_pct.map_or('-'.to_string(), |p| format!("{:.0}%", p))}"
                        }
                    }
                }
            })}
        }
    }
}

/// Duration chart
#[component]
fn DurationChart(stage_data: Vec<StageInsightData>) -> Element {
    let max_duration = stage_data.iter()
        .filter_map(|s| s.avg_duration_hours)
        .fold(0.0_f64, |a, b| a.max(b))
        .max(1.0);

    rsx! {
        div { class: "bar-chart horizontal",
            {stage_data.iter().map(|stage| rsx! {
                div { class: "chart-row",
                    div { class: "chart-label",
                        title: "{stage.stage_name}",
                        "{truncate_string(&stage.stage_name, 15)}"
                    }
                    div { class: "chart-bar-container",
                        div {
                            class: "chart-bar",
                            style: "width: {stage.avg_duration_hours.unwrap_or(0.0) * 280.0 / max_duration}px; \
                                    background: #0066CC;",
                            "{stage.avg_duration_hours.map_or('-'.to_string(), |h| format!("{:.1}u", h))}"
                        }
                    }
                }
            })}
        }
    }
}

/// Risk level indicator
#[component]
fn RiskIndicator(risk_level: RiskLevel) -> Element {
    let (color, label) = match risk_level {
        RiskLevel::Low => ("green", "Laag"),
        RiskLevel::Medium => ("yellow", "Middel"),
        RiskLevel::High => ("orange", "Hoog"),
        RiskLevel::Critical => ("red", "Kritiek"),
        RiskLevel::Unknown => ("gray", "Onbekend"),
    };

    rsx! {
        span {
            class: format!("risk-indicator risk-{}", color),
            "{label}"
        }
    }
}

/// Get color for SLA percentage
fn get_sla_color(sla_pct: f64) -> &'static str {
    if sla_pct >= 90.0 { "#22c55e" }
    else if sla_pct >= 75.0 { "#eab308" }
    else { "#ef4444" }
}

/// Truncate string to max length
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

// ============================================================================
// Types and Mock Data
// ============================================================================

#[derive(Clone, Debug)]
struct WorkflowAnalyticsData {
    sla_compliance_pct: f64,
    sla_trend: f32,
    avg_completion_hours: f64,
    time_trend: f32,
    total_executions: i32,
    bottleneck_stage: Option<String>,
    stage_insights: Vec<StageInsightData>,
    recommendations: Vec<Recommendation>,
}

#[derive(Clone, Debug)]
struct StageInsightData {
    stage_id: String,
    stage_name: String,
    status: String,
    avg_duration_hours: Option<f64>,
    sla_compliance_pct: Option<f64>,
    risk_level: RiskLevel,
}

#[derive(Clone, Debug)]
struct Recommendation {
    icon: String,
    title: String,
    description: String,
    impact: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum RiskLevel {
    Unknown,
    Low,
    Medium,
    High,
    Critical,
}

/// Mock fetch function (would call API in production)
async fn fetch_workflow_analytics(workflow_id: Uuid) -> WorkflowAnalyticsData {
    // Simulate API delay
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    WorkflowAnalyticsData {
        sla_compliance_pct: 78.5,
        sla_trend: 5.2,
        avg_completion_hours: 48.2,
        time_trend: -8.3,
        total_executions: 234,
        bottleneck_stage: Some("Finale Goedkeuring".to_string()),
        stage_insights: vec![
            StageInsightData {
                stage_id: "stage_1".to_string(),
                stage_name: "Juridische Check".to_string(),
                status: "Completed".to_string(),
                avg_duration_hours: Some(6.2),
                sla_compliance_pct: Some(94.0),
                risk_level: RiskLevel::Low,
            },
            StageInsightData {
                stage_id: "stage_2".to_string(),
                stage_name: "Management Review".to_string(),
                status: "InProgress".to_string(),
                avg_duration_hours: Some(12.4),
                sla_compliance_pct: Some(72.0),
                risk_level: RiskLevel::Medium,
            },
            StageInsightData {
                stage_id: "stage_3".to_string(),
                stage_name: "Finale Goedkeuring".to_string(),
                status: "Pending".to_string(),
                avg_duration_hours: Some(18.1),
                sla_compliance_pct: Some(58.0),
                risk_level: RiskLevel::High,
            },
        ],
        recommendations: vec![
            Recommendation {
                icon: "⚠️".to_string(),
                title: "Knelpunt aanpakken".to_string(),
                description: "De 'Finale Goedkeuring' fase heeft een lage SLA naleving van 58%. Overweeg parallelle goedkeuring.".to_string(),
                impact: "+15% SLA".to_string(),
            },
            Recommendation {
                icon: "⏱️".to_string(),
                title: "SLA verlengen".to_string(),
                description: "Huidige SLA van 48 uur wordt vaak niet gehaald. Overweeg verlenging naar 72 uur.".to_string(),
                impact: "-5 klachten/maand".to_string(),
            },
        ],
    }
}
