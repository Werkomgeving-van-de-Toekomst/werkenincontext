//! Approval workflow queue page
//!
//! Interface for approvers to review and approve/reject pending documents.

use dioxus::prelude::*;
use crate::api::documents::{get_document_status, DocumentStatus};
use crate::components::{DocumentCard, ApprovalActions};

#[component]
pub fn ApprovalQueue() -> Element {
    let mut documents = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let error = use_signal(|| None::<String>);
    let mut selected_document = use_signal(|| None::<DocumentStatus>);

    // For demo purposes, we'll use hardcoded document IDs
    // In production, this would come from an API endpoint listing pending documents
    let pending_document_ids: Vec<uuid::Uuid> = vec![
        // TODO: Fetch from API
    ];

    // Fetch pending documents on mount
    use_effect(move || {
        let doc_ids = pending_document_ids.clone();
        spawn(async move {
            let mut docs = Vec::new();
            for doc_id in doc_ids {
                match get_document_status(doc_id).await {
                    Ok(status) => {
                        docs.push(status);
                    }
                    Err(_e) => {
                        // Silently fail
                    }
                }
            }
            documents.set(docs);
            loading.set(false);
        });
    });

    rsx! {
        div { class: "approval-queue-container",
            div { class: "queue-sidebar",
                h1 { "Goedkeuring Wachtrij" }

                if *loading.read() {
                    div { class: "loading", "Laden..." }
                } else if let Some(err) = error.read().as_ref() {
                    div { class: "alert alert-error", "{err}" }
                } else if documents.read().is_empty() {
                    div { class: "empty-state",
                        p { "Geen documenten wachten op goedkeuring" }
                        p { class: "empty-note",
                            "Documenten die goedkeuring vereisten worden hier getoond."
                        }
                    }
                } else {
                    div { class: "queue-list",
                        for doc in documents.read().iter().cloned() {
                            div {
                                class: if selected_document.read().as_ref().map(|d| d.document_id) == Some(doc.document_id) {
                                    "queue-item selected"
                                } else {
                                    "queue-item"
                                },
                                onclick: move |_| selected_document.set(Some(doc.clone())),

                                DocumentCard {
                                    document: doc.clone(),
                                    selected: selected_document.read().as_ref().map(|d| d.document_id) == Some(doc.document_id),
                                    onclick: |_| {},
                                }
                            }
                        }
                    }
                }
            }

            if let Some(selected) = selected_document.read().as_ref() {
                div { class: "approval-detail",
                    div { class: "detail-header",
                        h2 { "Document Details" }
                        button {
                            class: "btn-close",
                            onclick: move |_| selected_document.set(None),
                            "×"
                        }
                    }

                    div { class: "detail-section",
                        h3 { "Informatie" }
                        div { class: "detail-row",
                            span { class: "label", "Type:" }
                            span { "{selected.document_type}" }
                        }
                        div { class: "detail-row",
                            span { class: "label", "Status:" }
                            span { class: "state-badge", "{selected.state}" }
                        }
                    }

                    div { class: "detail-section",
                        h3 { "Scores" }
                        div { class: "score-bar",
                            span { class: "score-label", "Compliance: {(selected.compliance_score * 100.0) as i32}%" }
                            div {
                                class: "score-fill compliance",
                                style: "width: {(selected.compliance_score * 100.0) as i32}%",
                            }
                        }
                        div { class: "score-bar",
                            span { class: "score-label", "Vertrouwen: {(selected.confidence_score * 100.0) as i32}%" }
                            div {
                                class: "score-fill confidence",
                                style: "width: {(selected.confidence_score * 100.0) as i32}%",
                            }
                        }
                    }

                    if selected.requires_approval {
                        div { class: "detail-section approval-required",
                            div { class: "approval-notice",
                                "Dit document vereist goedkeuring voordat het gepubliceerd kan worden."
                            }
                        }
                    }

                    div { class: "detail-section",
                        h3 { "Actie" }
                        ApprovalActions {
                            document_id: selected.document_id,
                            on_approved: move || {
                                let doc_id = selected_document.read().as_ref().map(|d| d.document_id);
                                if let Some(id) = doc_id {
                                    let current = documents.read().clone();
                                    documents.set(
                                        current
                                            .iter()
                                            .filter(|d| d.document_id != id)
                                            .cloned()
                                            .collect()
                                    );
                                    selected_document.set(None);
                                }
                            },
                            on_rejected: move || {
                                let doc_id = selected_document.read().as_ref().map(|d| d.document_id);
                                if let Some(id) = doc_id {
                                    let current = documents.read().clone();
                                    documents.set(
                                        current
                                            .iter()
                                            .filter(|d| d.document_id != id)
                                            .cloned()
                                            .collect()
                                    );
                                    selected_document.set(None);
                                }
                            }
                        }
                    }

                    // Audit trail preview
                    details { class: "detail-section",
                        summary { "Audit Trail" }
                        crate::components::AuditTrailViewer {
                            document_id: Some(selected.document_id),
                            compact: true
                        }
                    }
                }
            }
        }
    }
}
