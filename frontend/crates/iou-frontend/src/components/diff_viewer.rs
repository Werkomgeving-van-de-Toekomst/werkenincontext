//! Diff viewer component
//!
//! Displays document version differences in multiple formats.
//! Provides toggle between unified, side-by-side, and inline diff views.

use dioxus::prelude::*;
use uuid::Uuid;
use crate::api::documents::{get_diff, DiffResponse, DiffChangeView};

/// Main diff viewer component
#[component]
pub fn DiffViewer(
    document_id: Uuid,
    #[props(default)] from_version: Option<String>,
    #[props(default)] to_version: Option<String>,
    #[props(default)] initial_format: DiffFormat,
) -> Element {
    let mut format = use_signal(|| initial_format);
    let from = from_version.clone();
    let to = to_version.clone();

    // Use use_resource with reactivity by including format in the closure
    let diff_data = use_resource(move || {
        let doc_id = document_id;
        let from_ver = from.clone();
        let to_ver = to.clone();
        let current_format = *format.read();
        async move {
            if from_ver.is_none() || to_ver.is_none() {
                Ok(DiffData::default())
            } else {
                get_diff(doc_id, from_ver, to_ver, Some(current_format.to_string())).await
                    .map(|response| DiffData::from_response(response))
            }
        }
    });

    rsx! {
        div { class: "diff-viewer",
            // Format toggle buttons
            div { class: "diff-controls",
                button {
                    class: if *format.read() == DiffFormat::Unified { "active" },
                    onclick: move |_| {
                        format.set(DiffFormat::Unified);
                    },
                    "Unified"
                }
                button {
                    class: if *format.read() == DiffFormat::SideBySide { "active" },
                    onclick: move |_| {
                        format.set(DiffFormat::SideBySide);
                    },
                    "Side by Side"
                }
                button {
                    class: if *format.read() == DiffFormat::Inline { "active" },
                    onclick: move |_| {
                        format.set(DiffFormat::Inline);
                    },
                    "Inline"
                }
            }

            // Diff content
            match &*diff_data.read() {
                Some(Ok(data)) => rsx! {
                    DiffContent {
                        data: data.clone(),
                        format: *format.read(),
                    }
                },
                Some(Err(e)) => rsx! {
                    div { class: "diff-error",
                        svg {
                            class: "error-icon",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            path { d: "M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" }
                        }
                        p { "Fout bij laden diff: {e}" }
                    }
                },
                None => rsx! {
                    div { class: "diff-loading",
                        div { class: "spinner" }
                        p { "Diff laden..." }
                    }
                },
            }
        }
    }
}

/// Diff content renderer
#[component]
fn DiffContent(
    data: DiffData,
    format: DiffFormat,
) -> Element {
    let has_changes = data.summary.additions > 0 || data.summary.deletions > 0;

    if !has_changes && data.summary.unchanged > 0 {
        return rsx! {
            div { class: "diff-no-changes",
                svg {
                    class: "no-changes-icon",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    path { d: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" }
                }
                h3 { "Geen wijzigingen" }
                p { "De geselecteerde versies zijn identiek." }
            }
        };
    }

    rsx! {
        div { class: "diff-summary",
            div { class: "summary-item additions",
                span { class: "summary-label", "Toegevoegd:" }
                span { class: "summary-count", "{data.summary.additions}" }
            }
            div { class: "summary-item deletions",
                span { class: "summary-label", "Verwijderd:" }
                span { class: "summary-count", "{data.summary.deletions}" }
            }
            div { class: "summary-item unchanged",
                span { class: "summary-label", "Ongewijzigd:" }
                span { class: "summary-count", "{data.summary.unchanged}" }
            }
        }

        {match format {
            DiffFormat::Unified => rsx! {
                UnifiedDiff { data: data.clone() }
            },
            DiffFormat::SideBySide => rsx! {
                SideBySideDiff { data: data.clone() }
            },
            DiffFormat::Inline => rsx! {
                InlineDiff { data: data.clone() }
            },
        }}
    }
}

/// Unified diff view
#[component]
fn UnifiedDiff(data: DiffData) -> Element {
    let changes: Vec<(String, String, String)> = data.changes.iter()
        .map(|change| {
            let (class, prefix) = match change.change_type.as_str() {
                "add" | "addition" => ("diff-add".to_string(), "+".to_string()),
                "remove" | "deletion" => ("diff-remove".to_string(), "-".to_string()),
                _ => ("diff-unchanged".to_string(), " ".to_string()),
            };
            let text = change.new_text.as_ref().or(change.old_text.as_ref())
                .unwrap_or(&String::new()).clone();
            let key = format!("change-{}", change.line_number.unwrap_or(0));
            (class, prefix, text)
        })
        .collect();

    rsx! {
        div { class: "unified-diff",
            pre { class: "diff-content",
                code { class: "language-diff",
                    {changes.iter().map(|(class, prefix, text)| rsx! {
                        div {
                            class: "{class}",
                            span { class: "diff-prefix", "{prefix}" }
                            span { class: "diff-content", "{text}" }
                        }
                    })}
                }
            }
        }
    }
}

/// Side-by-side diff view
#[component]
fn SideBySideDiff(data: DiffData) -> Element {
    let mut old_lines: Vec<Option<String>> = Vec::new();
    let mut new_lines: Vec<Option<String>> = Vec::new();

    for change in &data.changes {
        match change.change_type.as_str() {
            "add" | "addition" => {
                old_lines.push(None);
                new_lines.push(change.new_text.clone());
            },
            "remove" | "deletion" => {
                old_lines.push(change.old_text.clone());
                new_lines.push(None);
            },
            _ => {
                let text = change.new_text.as_ref().or(change.old_text.as_ref()).cloned();
                old_lines.push(text.clone());
                new_lines.push(text);
            },
        }
    }

    let old_display: Vec<(usize, String)> = old_lines.iter().enumerate()
        .map(|(i, line)| (i, line.as_ref().unwrap_or(&String::new()).clone()))
        .collect();
    let new_display: Vec<(usize, String)> = new_lines.iter().enumerate()
        .map(|(i, line)| (i, line.as_ref().unwrap_or(&String::new()).clone()))
        .collect();

    rsx! {
        div { class: "side-by-side-diff",
            div { class: "diff-column old-version",
                div { class: "column-header", "Oude Versie ({data.from_version})" }
                pre { class: "diff-content",
                    {old_display.iter().map(|(i, text)| rsx! {
                        div {
                            class: "diff-line",
                            span { class: "line-number", "{i + 1}" }
                            span { class: "line-content", "{text}" }
                        }
                    })}
                }
            }
            div { class: "diff-column new-version",
                div { class: "column-header", "Nieuwe Versie ({data.to_version})" }
                pre { class: "diff-content",
                    {new_display.iter().map(|(i, text)| rsx! {
                        div {
                            class: "diff-line",
                            span { class: "line-number", "{i + 1}" }
                            span { class: "line-content", "{text}" }
                        }
                    })}
                }
            }
        }
    }
}

/// Inline diff view
#[component]
fn InlineDiff(data: DiffData) -> Element {
    let changes: Vec<(String, String, String)> = data.changes.iter()
        .map(|change| {
            let (class, icon) = match change.change_type.as_str() {
                "add" | "addition" => ("diff-add-inline".to_string(), "+".to_string()),
                "remove" | "deletion" => ("diff-remove-inline".to_string(), "-".to_string()),
                _ => ("diff-unchanged-inline".to_string(), "·".to_string()),
            };
            let text = change.new_text.as_ref().or(change.old_text.as_ref())
                .unwrap_or(&String::new()).clone();
            let key = format!("inline-change-{}", change.line_number.unwrap_or(0));
            (class, icon, text)
        })
        .collect();

    rsx! {
        div { class: "inline-diff",
            {changes.iter().map(|(class, icon, text)| rsx! {
                div {
                    class: "{class}",
                    span { class: "change-icon", "{icon}" }
                    span { class: "change-text", "{text}" }
                }
            })}
        }
    }
}

// ==============================================================================
// Types
// ==============================================================================

/// Diff display format
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DiffFormat {
    #[default]
    Unified,
    SideBySide,
    Inline,
}

impl DiffFormat {
    pub fn to_string(&self) -> String {
        match self {
            DiffFormat::Unified => "unified".to_string(),
            DiffFormat::SideBySide => "side_by_side".to_string(),
            DiffFormat::Inline => "inline".to_string(),
        }
    }
}

/// Internal diff data structure
#[derive(Clone, PartialEq, Debug, Default)]
struct DiffData {
    from_version: String,
    to_version: String,
    changes: Vec<DiffChange>,
    summary: DiffSummaryData,
}

impl DiffData {
    fn from_response(response: DiffResponse) -> Self {
        DiffData {
            from_version: response.from_version,
            to_version: response.to_version,
            changes: response.changes.into_iter().map(Into::into).collect(),
            summary: DiffSummaryData {
                additions: response.summary.additions,
                deletions: response.summary.deletions,
                unchanged: response.summary.unchanged,
            },
        }
    }
}

/// Diff change for frontend
#[derive(Clone, PartialEq, Debug)]
struct DiffChange {
    pub change_type: String,
    pub old_text: Option<String>,
    pub new_text: Option<String>,
    pub line_number: Option<usize>,
}

impl From<DiffChangeView> for DiffChange {
    fn from(api: DiffChangeView) -> Self {
        DiffChange {
            change_type: api.change_type,
            old_text: api.old_text,
            new_text: api.new_text,
            line_number: api.line_number,
        }
    }
}

/// Diff summary for frontend
#[derive(Clone, PartialEq, Debug, Default)]
struct DiffSummaryData {
    pub additions: i32,
    pub deletions: i32,
    pub unchanged: i32,
}
