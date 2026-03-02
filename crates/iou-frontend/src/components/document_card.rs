//! Document card component for displaying document summaries in the approval queue.

use dioxus::prelude::*;
use crate::api::documents::DocumentStatus;

#[component]
pub fn DocumentCard(
    document: DocumentStatus,
    #[props(default = false)] selected: bool,
    onclick: EventHandler,
) -> Element {
    let state_class = match document.state.as_str() {
        "draft" => "state-draft",
        "submitted" | "in_review" => "state-pending",
        "approved" => "state-approved",
        "published" => "state-published",
        "rejected" => "state-rejected",
        _ => "state-unknown",
    };

    let compliance_pct = document.compliance_score * 100.0;
    let confidence_pct = document.confidence_score * 100.0;

    rsx! {
        div {
            class: if selected { "document-card selected" } else { "document-card" },
            onclick: move |_| onclick.call(()),

            div { class: "document-card-header",
                span { class: "document-type", "{document.document_type}" }
                span { class: format!("state-badge {state_class}"), "{document.state}" }
            }

            div { class: "document-card-body",
                div { class: "document-id", "ID: {document.document_id}" }

                if document.requires_approval {
                    div { class: "approval-badge", "Goedkeuring vereist" }
                }

                div { class: "scores-container",
                    div { class: "score-item",
                        span { class: "score-label", "Compliance:" }
                        div { class: "score-bar",
                            div {
                                class: "score-fill compliance",
                                style: "width: {compliance_pct}%",
                            }
                        }
                        span { class: "score-value", "{compliance_pct as i32}%" }
                    }

                    div { class: "score-item",
                        span { class: "score-label", "Vertrouwen:" }
                        div { class: "score-bar",
                            div {
                                class: "score-fill confidence",
                                style: "width: {confidence_pct}%",
                            }
                        }
                        span { class: "score-value", "{confidence_pct as i32}%" }
                    }
                }
            }

            if !document.errors.is_empty() {
                div { class: "document-errors",
                    div { class: "error-title", "Fouten:" }
                    for error in document.errors.iter() {
                        div { class: "error-message", "{error}" }
                    }
                }
            }
        }
    }
}
