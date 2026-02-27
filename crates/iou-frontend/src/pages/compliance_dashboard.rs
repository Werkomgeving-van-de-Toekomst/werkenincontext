//! Compliance Dashboard
//!
//! Real-time compliance monitoring dashboard with:
//! - Overall compliance score
//! - Domain-level breakdown
//! - Trend analysis
//! - Risk alerts
//! - Recommended actions

use dioxus::prelude::*;

use crate::state::AppState;
use crate::components::{Header, Panel};

#[derive(Clone, Copy, Debug, PartialEq)]
enum TimePeriod {
    Week,
    Month,
    Quarter,
    Year,
}

impl TimePeriod {
    fn display(&self) -> &'static str {
        match self {
            TimePeriod::Week => "Week",
            TimePeriod::Month => "Maand",
            TimePeriod::Quarter => "Kwartaal",
            TimePeriod::Year => "Jaar",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ComplianceAlert {
    id: String,
    severity: AlertSeverity,
    alert_type: String,
    title: String,
    description: String,
    domain_name: Option<String>,
    due_date: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl AlertSeverity {
    fn class(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "alert-info",
            AlertSeverity::Warning => "alert-warning",
            AlertSeverity::Error => "alert-error",
            AlertSeverity::Critical => "alert-critical",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "ℹ",
            AlertSeverity::Warning => "⚠",
            AlertSeverity::Error => "✕",
            AlertSeverity::Critical => "⛔",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct DomainCompliance {
    name: String,
    score: f32,
    woo_compliant: i32,
    woo_total: i32,
    avg_compliant: i32,
    avg_total: i32,
    issues: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct TrendPoint {
    date: String,
    woo_rate: f32,
    avg_rate: f32,
    archief_rate: f32,
}

fn mock_trends() -> Vec<TrendPoint> {
    vec![
        TrendPoint { date: "2026-01-01".to_string(), woo_rate: 0.85, avg_rate: 0.92, archief_rate: 0.78 },
        TrendPoint { date: "2026-01-08".to_string(), woo_rate: 0.87, avg_rate: 0.93, archief_rate: 0.80 },
        TrendPoint { date: "2026-01-15".to_string(), woo_rate: 0.88, avg_rate: 0.93, archief_rate: 0.82 },
        TrendPoint { date: "2026-01-22".to_string(), woo_rate: 0.90, avg_rate: 0.94, archief_rate: 0.84 },
        TrendPoint { date: "2026-01-29".to_string(), woo_rate: 0.91, avg_rate: 0.95, archief_rate: 0.85 },
    ]
}

fn mock_alerts() -> Vec<ComplianceAlert> {
    vec![
        ComplianceAlert {
            id: "1".to_string(),
            severity: AlertSeverity::Critical,
            alert_type: "Destruction Overdue".to_string(),
            title: "Vernietigingstermijn verstreken".to_string(),
            description: "5 documenten hadden vernietigd moeten worden".to_string(),
            domain_name: Some("Infrastructuur".to_string()),
            due_date: Some("2026-02-01".to_string()),
        },
        ComplianceAlert {
            id: "2".to_string(),
            severity: AlertSeverity::Warning,
            alert_type: "Missing Retention Period".to_string(),
            title: "Bewaartermijn ontbreekt".to_string(),
            description: "12 documenten hebben geen bewaartermijn ingesteld".to_string(),
            domain_name: Some("Energie".to_string()),
            due_date: Some("2026-02-15".to_string()),
        },
        ComplianceAlert {
            id: "3".to_string(),
            severity: AlertSeverity::Warning,
            alert_type: "Woo Review Due".to_string(),
            title: "Woo-beoordeling vereist".to_string(),
            description: "8 nieuwe documenten vereisten Woo-assessment".to_string(),
            domain_name: Some("Windpark Almere".to_string()),
            due_date: Some("2026-02-10".to_string()),
        },
        ComplianceAlert {
            id: "4".to_string(),
            severity: AlertSeverity::Info,
            alert_type: "Pending Transfer".to_string(),
            title: "Archiefoverdracht pending".to_string(),
            description: "3 dossieren klaar voor overbrenging naar Nationaal Archief".to_string(),
            domain_name: Some("Natuur".to_string()),
            due_date: Some("2026-03-01".to_string()),
        },
    ]
}

#[component]
fn TrendBar(point: TrendPoint) -> Element {
    let woo_height = (point.woo_rate * 100.0) as i32;
    let avg_height = (point.avg_rate * 100.0) as i32;
    let archief_height = (point.archief_rate * 100.0) as i32;
    let label_short = if point.date.len() > 8 { &point.date[8..] } else { &point.date };

    rsx! {
        div { class: "chart-bar-group",
            div {
                class: "chart-bar woo",
                style: "height: {woo_height}%;",
                title: "{point.date}: Woo {woo_height}%",
            }
            div {
                class: "chart-bar avg",
                style: "height: {avg_height}%;",
                title: "{point.date}: AVG {avg_height}%",
            }
            div {
                class: "chart-bar archief",
                style: "height: {archief_height}%;",
                title: "{point.date}: Archief {archief_height}%",
            }
            div { class: "chart-label",
                "{label_short}"
            }
        }
    }
}

#[component]
fn DomainBar(domain: DomainCompliance) -> Element {
    let woo_width = domain.woo_compliant * 100 / domain.woo_total.max(1);
    let avg_width = domain.avg_compliant * 100 / domain.avg_total.max(1);
    let woo_text = format!("{}/{}", domain.woo_compliant, domain.woo_total);
    let avg_text = format!("{}/{}", domain.avg_compliant, domain.avg_total);
    let score_text = format!("{}%", (domain.score * 100.0) as i32);

    rsx! {
        div { class: "domain-item",
            div { class: "domain-header",
                div { class: "domain-name", "{domain.name}" }
                div {
                    class: if domain.score >= 0.9 { "score-badge good" }
                        else if domain.score >= 0.7 { "score-badge warning" }
                        else { "score-badge bad" },
                    "{score_text}"
                }
            }
            div { class: "domain-bars",
                div { class: "bar-row",
                    span { "Woo" }
                    div { class: "bar-container",
                        div {
                            class: "bar woo",
                            style: "width: {woo_width}%;",
                        }
                    }
                    span { "{woo_text}" }
                }
                div { class: "bar-row",
                    span { "AVG" }
                    div { class: "bar-container",
                        div {
                            class: "bar avg",
                            style: "width: {avg_width}%;",
                        }
                    }
                    span { "{avg_text}" }
                }
            }
            if !domain.issues.is_empty() {
                div { class: "domain-issues",
                    for issue in &domain.issues {
                        div { class: "issue-item", "• {issue}" }
                    }
                }
            }
        }
    }
}

#[component]
fn RiskDomainItem(domain: DomainCompliance) -> Element {
    let score = (domain.score * 100.0) as i32;
    let issues = domain.issues.join(", ");

    rsx! {
        div { class: "risk-domain",
            div { class: "risk-domain-name", "{domain.name}" }
            div { class: "risk-score", "Score: {score}%" }
            div { class: "risk-factors",
                "Risicofactoren: {issues}"
            }
        }
    }
}

#[component]
fn AlertItem(alert: ComplianceAlert, selected_alert: Signal<Option<String>>) -> Element {
    let alert_class = alert.severity.class();
    let alert_icon = alert.severity.icon();
    let alert_title = alert.title.clone();
    let alert_desc = alert.description.clone();
    let alert_domain = alert.domain_name.clone();
    let alert_type = alert.alert_type.clone();
    let alert_due = alert.due_date.clone();
    let alert_id = alert.id.clone();
    let is_selected = *selected_alert.read() == Some(alert_id.clone());

    rsx! {
        div {
            class: format!("alert-item {}", alert_class),
            onclick: move |_| {
                let id = alert_id.clone();
                if *selected_alert.read() == Some(id.clone()) {
                    selected_alert.set(None);
                } else {
                    selected_alert.set(Some(id));
                }
            },
            div { class: "alert-icon", "{alert_icon}" }
            div { class: "alert-content",
                div { class: "alert-title", "{alert_title}" }
                div { class: "alert-desc", "{alert_desc}" }
                if let Some(domain) = &alert_domain {
                    div { class: "alert-domain", "{domain}" }
                }
            }
        }
        if is_selected {
            div { class: "alert-detail",
                div { class: "detail-row",
                    span { "Type:" }
                    span { "{alert_type}" }
                }
                if let Some(due) = &alert_due {
                    div { class: "detail-row",
                        span { "Deadline:" }
                        span { "{due}" }
                    }
                }
                div { style: "display: flex; gap: 10px; margin-top: 10px;",
                    button { class: "btn btn-primary", "Actie ondernemen" }
                    button { class: "btn btn-secondary", "Sluiten" }
                }
            }
        }
    }
}

fn mock_domains() -> Vec<DomainCompliance> {
    vec![
        DomainCompliance {
            name: "Windpark Almere".to_string(),
            score: 0.92,
            woo_compliant: 45,
            woo_total: 48,
            avg_compliant: 48,
            avg_total: 48,
            issues: vec![],
        },
        DomainCompliance {
            name: "Waterstaat".to_string(),
            score: 0.85,
            woo_compliant: 32,
            woo_total: 38,
            avg_compliant: 38,
            avg_total: 38,
            issues: vec!["3 documenten missen Woo-beoordeling".to_string()],
        },
        DomainCompliance {
            name: "Infrastructuur".to_string(),
            score: 0.74,
            woo_compliant: 25,
            woo_total: 35,
            avg_compliant: 30,
            avg_total: 35,
            issues: vec![
                "5 documenten vernietigingstermijn verstreken".to_string(),
                "2 documenten missen AVG-grondslag".to_string(),
            ],
        },
        DomainCompliance {
            name: "Natuur".to_string(),
            score: 0.96,
            woo_compliant: 28,
            woo_total: 28,
            avg_compliant: 28,
            avg_total: 28,
            issues: vec![],
        },
        DomainCompliance {
            name: "Energie".to_string(),
            score: 0.68,
            woo_compliant: 18,
            woo_total: 28,
            avg_compliant: 22,
            avg_total: 28,
            issues: vec![
                "8 documenten missen bewaartermijn".to_string(),
                "3 documenten missen Woo-beoordeling".to_string(),
            ],
        },
    ]
}

#[component]
pub fn ComplianceDashboard() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut time_period = use_signal(|| TimePeriod::Month);
    let mut show_risk_analysis = use_signal(|| false);
    let alerts = use_signal(|| mock_alerts());
    let domains = use_signal(|| mock_domains());
    let trends = use_signal(|| mock_trends());
    let mut selected_alert = use_signal(|| None::<String>);

    use_effect(move || {
        state.write().user = Some(crate::state::UserInfo::flevoland());
    });

    let overall_score = domains
        .read()
        .iter()
        .map(|d| d.score)
        .sum::<f32>() / domains.read().len().max(1) as f32;

    let total_objects: i32 = domains.read().iter().map(|d| d.woo_total).sum();
    let woo_compliant: i32 = domains.read().iter().map(|d| d.woo_compliant).sum();
    let avg_compliant: i32 = domains.read().iter().map(|d| d.avg_compliant).sum();
    let avg_total: i32 = domains.read().iter().map(|d| d.avg_total).sum();

    // Computed values for display
    let overall_score_pct = (overall_score * 100.0) as i32;
    let overall_dasharray = format!("{:.0} {:.0}", overall_score * 283.0, 283.0 * (1.0 - overall_score));
    let woo_pct = ((woo_compliant as f32 / total_objects.max(1) as f32) * 100.0) as i32;
    let avg_pct = ((avg_compliant as f32 / avg_total.max(1) as f32) * 100.0) as i32;

    rsx! {
        Header {}

        main { class: "container",
            div { class: "page-header",
                h1 { "Compliance Dashboard" }
                p { "Real-time monitoring van Woo, AVG en Archiefwet compliance" }
            }

            // Period Selector
            div { class: "period-selector",
                button {
                    class: if *time_period.read() == TimePeriod::Week { "active" } else { "" },
                    onclick: move |_| time_period.set(TimePeriod::Week),
                    "Week"
                }
                button {
                    class: if *time_period.read() == TimePeriod::Month { "active" } else { "" },
                    onclick: move |_| time_period.set(TimePeriod::Month),
                    "Maand"
                }
                button {
                    class: if *time_period.read() == TimePeriod::Quarter { "active" } else { "" },
                    onclick: move |_| time_period.set(TimePeriod::Quarter),
                    "Kwartaal"
                }
                button {
                    class: if *time_period.read() == TimePeriod::Year { "active" } else { "" },
                    onclick: move |_| time_period.set(TimePeriod::Year),
                    "Jaar"
                }
            }

            // Score Cards
            div { class: "score-cards",
                div { class: "score-card overall",
                    div { class: "score-circle",
                        svg {
                            view_box: "0 0 100 100",
                            circle {
                                cx: "50",
                                cy: "50",
                                r: "45",
                                fill: "none",
                                stroke: "#e0e0e0",
                                "stroke-width": "8",
                            }
                            circle {
                                cx: "50",
                                cy: "50",
                                r: "45",
                                fill: "none",
                                stroke: if overall_score >= 0.9 { "#4CAF50" }
                                    else if overall_score >= 0.7 { "#FFC107" }
                                    else { "#F44336" },
                                "stroke-width": "8",
                                "stroke-dasharray": "{overall_dasharray}",
                                "stroke-linecap": "round",
                                transform: "rotate(-90 50 50)",
                            }
                            text {
                                x: "50",
                                y: "55",
                                "text-anchor": "middle",
                                "font-size": "20",
                                "font-weight": "bold",
                                fill: "#333",
                                "{overall_score_pct}%"
                            }
                        }
                    }
                    div { class: "score-label", "Overall Score" }
                }

                div { class: "score-card",
                    div { class: "score-value",
                        "{woo_pct}%"
                    }
                    div { class: "score-label", "Woo Compliance" }
                    div { class: "score-detail", "{woo_compliant}/{total_objects} documenten" }
                }

                div { class: "score-card",
                    div { class: "score-value",
                        "{avg_pct}%"
                    }
                    div { class: "score-label", "AVG Compliance" }
                    div { class: "score-detail", "{avg_compliant}/{avg_total} documenten" }
                }

                div { class: "score-card",
                    div { class: "score-value alert",
                        "{alerts.read().len()}"
                    }
                    div { class: "score-label", "Actieve Alerts" }
                    button {
                        class: "btn btn-secondary",
                        style: "margin-top: 8px; font-size: 0.8rem;",
                        onclick: move |_| {
                            let _ = web_sys::window()
                                .and_then(|w| w.document())
                                .and_then(|d| d.get_element_by_id("alerts-section"))
                                .map(|el| el.scroll_into_view());
                        },
                        "Bekijk"
                    }
                }
            }

            // Main Content Grid
            div { style: "display: grid; grid-template-columns: 2fr 1fr; gap: 20px; margin-top: 20px;",
                // Left Column
                div {
                    // Domain Compliance
                    Panel { title: "Domein Compliance".to_string(),
                        div { class: "domain-list",
                            for domain in domains.read().iter() {
                                DomainBar { domain: domain.clone() }
                            }
                        }
                    }

                    // Trends Chart
                    Panel { title: format!("Trends - {}", time_period.read().display()),
                        div { class: "trend-chart",
                            div { class: "chart-legend",
                                span { class: "legend-item woo", "Woo" }
                                span { class: "legend-item avg", "AVG" }
                                span { class: "legend-item archief", "Archief" }
                            }
                            div { class: "chart-bars",
                                for point in trends.read().iter() {
                                    TrendBar { point: point.clone() }
                                }
                            }
                        }
                    }
                }

                // Right Column
                div {
                    // Alerts Panel
                    Panel { title: "Actieve Alerts".to_string(),
                        div { id: "alerts-section", class: "alerts-list",
                            for alert in alerts.read().clone().iter() {
                                AlertItem {
                                    alert: alert.clone(),
                                    selected_alert: selected_alert.clone(),
                                }
                            }
                        }
                    }

                    // Risk Analysis Toggle
                    button {
                        class: "btn btn-secondary",
                        style: "width: 100%; margin-top: 10px;",
                        onclick: move |_| {
                            let current = *show_risk_analysis.read();
                            show_risk_analysis.set(!current);
                        },
                        "Toon risicoanalyse"
                    }

                    // Recommendations
                    Panel { title: "Aanbevelingen".to_string(),
                        div { class: "recommendations",
                            div { class: "recommendation urgent",
                                div { class: "rec-priority", "Urgent" }
                                div { class: "rec-content",
                                    div { class: "rec-title", "Vernietigingstermijnen verstreken" }
                                    div { class: "rec-desc", "5 documenten moeten onmiddellijk worden vernietigd" }
                                }
                            }
                            div { class: "recommendation high",
                                div { class: "rec-priority", "Hoog" }
                                div { class: "rec-content",
                                    div { class: "rec-title", "Bewaartermijnen instellen" }
                                    div { class: "rec-desc", "12 documenten missen bewaartermijn" }
                                }
                            }
                            div { class: "recommendation medium",
                                div { class: "rec-priority", "Middel" }
                                div { class: "rec-content",
                                    div { class: "rec-title", "Woo-beoordeling uitvoeren" }
                                    div { class: "rec-desc", "8 documenten wachten op assessment" }
                                }
                            }
                        }
                    }
                }
            }

            // Risk Analysis Panel (expandable)
            if *show_risk_analysis.read() {
                Panel { title: "Risicoanalyse".to_string(),
                    div { class: "risk-analysis",
                        div { class: "risk-summary",
                            div { class: "risk-level high",
                                "Overall risico: HOOG"
                            }
                            p { "Er zijn 2 domeinen met verhoogd risico op compliantieproblemen." }
                        }
                        div { style: "margin-top: 20px;",
                            h3 { "Domeinen met verhoogd risico" }
                            for domain in domains.read().iter().filter(|d| d.score < 0.8) {
                                RiskDomainItem { domain: domain.clone() }
                            }
                        }
                        div { style: "margin-top: 20px;",
                            h3 { "Voorspelde problemen" }
                            div { class: "predicted-issue",
                                div { class: "issue-probability", "70% waarschijnlijkheid" }
                                div { class: "issue-desc",
                                    "Volgende maand bereiken 15 documenten hun vernietigingsdatum"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
