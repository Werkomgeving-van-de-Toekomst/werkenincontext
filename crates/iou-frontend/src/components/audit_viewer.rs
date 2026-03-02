//! Audit trail viewer component
//!
//! Displays the audit trail for a document with expandable entries.

use dioxus::prelude::*;
use crate::api::documents::get_audit_trail;

#[component]
pub fn AuditTrailViewer(
    document_id: Option<uuid::Uuid>,
    #[props(default = false)] compact: bool,
) -> Element {
    let mut entries = use_signal(Vec::new);
    let mut loading = use_signal(|| document_id.is_some());
    let mut expanded_idx = use_signal(|| None::<usize>);

    // Load audit trail when document_id is provided
    if let Some(doc_id) = document_id {
        use_effect(move || {
            spawn(async move {
                match get_audit_trail(doc_id).await {
                    Ok(trail) => {
                        entries.set(trail.audit_trail);
                        loading.set(false);
                    }
                    Err(_e) => {
                        loading.set(false);
                    }
                }
            });
        });
    }

    let is_expanded = move |idx: usize| *expanded_idx.read() == Some(idx);

    let mut toggle_expand = move |idx: usize| {
        let current = *expanded_idx.read();
        expanded_idx.set(if current == Some(idx) { None } else { Some(idx) });
    };

    rsx! {
        div { class: if compact { "audit-trail compact" } else { "audit-trail" },
            h3 { if compact { "Recente Activiteit" } else { "Audit Trail" } }

            if *loading.read() {
                div { class: "loading", "Laden..." }
            } else if entries.read().is_empty() {
                div { class: "empty", "Geen audit entries gevonden" }
            } else {
                ul { class: "audit-list",
                    for (idx, entry) in entries.read().iter().enumerate() {
                        li {
                            class: if is_expanded(idx) { "expanded" } else { "" },

                            div {
                                class: "audit-entry-header",
                                onclick: move |_| toggle_expand(idx),

                                span { class: "agent-name", "{entry.agent}" }
                                span { class: "action", "{entry.action}" }
                                span { class: "timestamp", "{entry.timestamp}" }
                            }

                            if is_expanded(idx) {
                                div { class: "audit-entry-details",
                                    div { class: "detail-row",
                                        span { class: "label", "Details:" }
                                        pre { class: "json-details",
                                            "{serde_json::to_string_pretty(&entry.details).unwrap_or_default()}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
