//! Approval action buttons component
//!
//! Provides approve/reject buttons for the approval workflow.

use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn ApprovalActions(
    document_id: Uuid,
    on_approved: EventHandler,
    on_rejected: EventHandler,
) -> Element {
    let mut loading = use_signal(|| false);
    let mut show_reject_dialog = use_signal(|| false);
    let mut reject_comments = use_signal(|| String::new());
    let mut error = use_signal(|| None::<String>);

    let handle_approve = move |_| {
        loading.set(true);
        error.set(None);

        let doc_id = document_id;
        spawn(async move {
            match crate::api::documents::approve_document(
                doc_id,
                &crate::api::documents::ApprovalRequest {
                    approved: true,
                    comments: None,
                },
            ).await {
                Ok(_) => {
                    loading.set(false);
                    on_approved.call(());
                }
                Err(e) => {
                    error.set(Some(e));
                    loading.set(false);
                }
            }
        });
    };

    let handle_reject = move |_| {
        show_reject_dialog.set(true);
    };

    let confirm_reject = move |_| {
        let comments = reject_comments.read().clone();
        let doc_id = document_id;
        loading.set(true);
        show_reject_dialog.set(false);

        spawn(async move {
            match crate::api::documents::approve_document(
                doc_id,
                &crate::api::documents::ApprovalRequest {
                    approved: false,
                    comments: if comments.trim().is_empty() { None } else { Some(comments) },
                },
            ).await {
                Ok(_) => {
                    loading.set(false);
                    on_rejected.call(());
                }
                Err(e) => {
                    error.set(Some(e));
                    loading.set(false);
                }
            }
        });
    };

    let cancel_reject = move |_| {
        show_reject_dialog.set(false);
        reject_comments.set(String::new());
    };

    rsx! {
        div { class: "approval-actions",
            if *loading.read() {
                div { class: "loading-spinner", "Bezig..." }
            } else {
                button {
                    class: "btn btn-success",
                    onclick: handle_approve,
                    "Goedkeuren"
                }
                button {
                    class: "btn btn-danger",
                    onclick: handle_reject,
                    "Afwijzen"
                }
            }

            if let Some(err) = error.read().as_ref() {
                div { class: "error-message", "{err}" }
            }
        }

        if *show_reject_dialog.read() {
            div { class: "reject-dialog-overlay",
                div { class: "reject-dialog",
                    h3 { "Document Afwijzen" }
                    p { "Voeg optioneel een reden op voor afwijzing:" }

                    textarea {
                        class: "reject-comments",
                        rows: "4",
                        placeholder: "Reden voor afwijzing...",
                        value: "{reject_comments}",
                        oninput: move |evt: Event<FormData>| {
                            let value = evt.value();
                            reject_comments.set(value);
                        }
                    }

                    div { class: "dialog-actions",
                        button {
                            class: "btn btn-danger",
                            onclick: confirm_reject,
                            "Bevestig Afwijzing"
                        }
                        button {
                            class: "btn btn-secondary",
                            onclick: cancel_reject,
                            "Annuleren"
                        }
                    }
                }
            }
        }
    }
}
