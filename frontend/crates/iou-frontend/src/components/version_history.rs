//! Version history component
//!
//! Displays document version history with metadata.
//! Allows selecting two versions for comparison and restoring previous versions.

use dioxus::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::api::documents::{
    list_versions, restore_version,
    VersionViewApi,
};

/// Main version history component
#[component]
pub fn VersionHistory(
    document_id: Uuid,
    #[props(default)] on_compare: Option<EventHandler<(String, String)>>,
    #[props(default)] on_restored: Option<EventHandler<()>>,
) -> Element {
    let versions = use_resource(move || {
        let doc_id = document_id;
        async move {
            list_versions(doc_id).await
                .unwrap_or(Vec::new())
                .into_iter()
                .map(VersionView::from)
                .collect::<Vec<VersionView>>()
        }
    });

    let mut selected_for_comparison = use_signal(|| (None::<String>, None::<String>));
    let mut show_restore_confirm = use_signal(|| None::<(VersionView, String)>);
    let mut restoring = use_signal(|| None::<String>);

    // Extract versions data for rendering
    let versions_data = versions.read().clone().unwrap_or_default();

    rsx! {
        div { class: "version-history",
            h3 { "Versiegeschiedenis" }

            if versions_data.is_empty() && versions.read().is_some() {
                div { class: "empty-versions",
                    p { "Geen versies gevonden" }
                }
            } else if !versions_data.is_empty() {
                // Version list with selection checkboxes
                VersionList {
                    versions: versions_data.clone(),
                    document_id: document_id,
                    selected_for_comparison: selected_for_comparison.clone(),
                    show_restore_confirm: show_restore_confirm.clone(),
                    restoring: restoring.clone(),
                    on_compare: on_compare.clone(),
                    on_restored: on_restored.clone(),
                }

                // Compare button (enabled when 2 versions selected)
                {
                    let comparison_data = selected_for_comparison.read().clone();
                    match (comparison_data, on_compare) {
                        ((Some(v1), Some(v2)), Some(handler)) => {
                            let v1_clone = v1.clone();
                            let v2_clone = v2.clone();
                            let compare_text = format!("Vergelijk {} met {}", v1, v2);
                            rsx!(
                                div { class: "version-actions",
                                    button {
                                        class: "btn btn-secondary",
                                        onclick: move |_| handler.call((v1_clone.clone(), v2_clone.clone())),
                                        svg {
                                            class: "btn-icon",
                                            view_box: "0 0 24 24",
                                            fill: "none",
                                            stroke: "currentColor",
                                            "stroke-width": "2",
                                            path { d: "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" }
                                        }
                                        "{compare_text}"
                                    }
                                }
                            )
                        },
                        _ => rsx!(),
                    }
                }
            } else {
                div { class: "loading-versions",
                    div { class: "spinner" }
                    p { "Versies laden..." }
                }
            }
        }
    }
}

/// Version list component
#[component]
fn VersionList(
    versions: Vec<VersionView>,
    document_id: Uuid,
    selected_for_comparison: Signal<(Option<String>, Option<String>)>,
    show_restore_confirm: Signal<Option<(VersionView, String)>>,
    restoring: Signal<Option<String>>,
    on_compare: Option<EventHandler<(String, String)>>,
    on_restored: Option<EventHandler<()>>,
) -> Element {
    // Create separate signals for tracking to avoid lifetime issues
    let mut selection_1 = use_signal(|| None::<String>);
    let mut selection_2 = use_signal(|| None::<String>);

    rsx! {
        div { class: "version-list",
            div { class: "version-list-header",
                div { class: "header-cell version-number", "Versie" }
                div { class: "header-cell version-date", "Datum" }
                div { class: "header-cell version-author", "Auteur" }
                div { class: "header-cell version-summary", "Samenvatting" }
                div { class: "header-cell version-actions", "Acties" }
            }

            {versions.iter().map(|version| {
                // Clone the version to own it for the closures
                let version_owned = version.clone();
                let version_id = version_owned.version_id.clone();
                let version_id_check = version_id.clone();
                let version_id_for_checkbox = version_id.clone();
                let version_id_for_restore = version_id.clone();
                let version_number = version_owned.version_number;
                let is_current = version_owned.is_current;
                let created_by = version_owned.created_by_name.clone().unwrap_or_else(|| "Onbekend".to_string());
                let summary = version_owned.change_summary.clone().unwrap_or_else(|| "Geen samenvatting".to_string());
                let date = format_date(&version_owned.created_at);
                let is_restoring = restoring.read().as_ref() == Some(&version_id_check);

                rsx!(
                    div {
                        class: "version-row",
                        key: "{version_id}",

                        div { class: "cell version-number",
                            input {
                                r#type: "checkbox",
                                checked: selection_1.read().as_ref() == Some(&version_id_check)
                                    || selection_2.read().as_ref() == Some(&version_id_check),
                                disabled: is_current,
                                onchange: move |_| {
                                    let current1 = selection_1.read().clone();
                                    let current2 = selection_2.read().clone();
                                    let vid = version_id_for_checkbox.clone();
                                    let new_selection = if current1.as_ref() == Some(&vid) {
                                        (None, current2)
                                    } else if current2.as_ref() == Some(&vid) {
                                        (current1, None)
                                    } else if current1.is_none() {
                                        (Some(vid.clone()), current2)
                                    } else {
                                        (current1, Some(vid.clone()))
                                    };
                                    let new_1 = new_selection.0.clone();
                                    let new_2 = new_selection.1.clone();
                                    selection_1.set(new_1);
                                    selection_2.set(new_2);

                                    // Update the parent signal
                                    selected_for_comparison.set((new_selection.0, new_selection.1));
                                },
                            }
                            span { class: "version-badge",
                                "v{version_number}"
                            }
                            if is_current {
                                span { class: "current-badge", "Huidig" }
                            }
                        }

                        div { class: "cell version-date",
                            "{date}"
                        }

                        div { class: "cell version-author",
                            "{created_by}"
                        }

                        div { class: "cell version-summary",
                            "{summary}"
                        }

                        div { class: "cell version-actions",
                            if !is_current {
                                button {
                                    class: "btn btn-sm btn-outline",
                                    disabled: is_restoring,
                                    onclick: move |_| {
                                        show_restore_confirm.set(Some((version_owned.clone(), version_id_for_restore.clone())));
                                    },
                                    {if is_restoring { "Herstellen..." } else { "Herstel" }}
                                }
                            } else {
                                span { class: "current-indicator", "—" }
                            }
                        }
                    }
                )
            })}
        }

        // Restore confirmation modal
        if let Some((version, _version_id)) = &*show_restore_confirm.read() {
            RestoreConfirmation {
                version: version.clone(),
                is_restoring: restoring.read().as_ref() == Some(&version.version_id),
                document_id: document_id,
                on_restored: on_restored.clone(),
                restoring: restoring.clone(),
                show_restore_confirm: show_restore_confirm.clone(),
            }
        }
    }
}

/// Restore confirmation dialog
#[component]
fn RestoreConfirmation(
    version: VersionView,
    is_restoring: bool,
    document_id: Uuid,
    on_restored: Option<EventHandler<()>>,
    restoring: Signal<Option<String>>,
    show_restore_confirm: Signal<Option<(VersionView, String)>>,
) -> Element {
    rsx! {
        div { class: "modal-overlay",
            div { class: "modal-content",
                div { class: "modal-header",
                    h3 { "Versie Herstellen Bevestiging" }
                    button {
                        class: "modal-close",
                        onclick: move |_| show_restore_confirm.set(None),
                        "×"
                    }
                }

                div { class: "modal-body",
                    div { class: "alert alert-warning",
                        svg {
                            class: "alert-icon",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            path { d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" }
                        }
                        p { "Weet je zeker dat je versie {version.version_number} wilt herstellen?" }
                    }

                    p { "Dit zal een nieuwe versie aanmaken met de inhoud van de geselecteerde versie. De huidige versie blijft behouden in de versiegeschiedenis." }

                    div { class: "restore-details",
                        div { class: "detail-row",
                            span { class: "detail-label", "Versie:" }
                            span { class: "detail-value", "v{version.version_number}" }
                        }
                        div { class: "detail-row",
                            span { class: "detail-label", "Auteur:" }
                            span { class: "detail-value",
                                {version.created_by_name.as_deref().unwrap_or("Onbekend")}
                            }
                        }
                        div { class: "detail-row",
                            span { class: "detail-label", "Datum:" }
                            span { class: "detail-value",
                                {format_date(&version.created_at)}
                            }
                        }
                        if let Some(summary) = &version.change_summary {
                            div { class: "detail-row",
                                span { class: "detail-label", "Samenvatting:" }
                                span { class: "detail-value", "{summary}" }
                            }
                        }
                    }
                }

                div { class: "modal-footer",
                    button {
                        class: "btn btn-danger",
                        disabled: is_restoring,
                        onclick: move |_| {
                            let doc_id = document_id;
                            let ver_id = version.version_id.clone();
                            let mut restoring_clone = restoring.clone();
                            let mut show_restore_confirm_clone = show_restore_confirm.clone();
                            let on_restored = on_restored.clone();

                            spawn(async move {
                                restoring_clone.set(Some(ver_id.clone()));
                                match restore_version(doc_id, ver_id, None).await {
                                    Ok(_) => {
                                        restoring_clone.set(None);
                                        show_restore_confirm_clone.set(None);
                                        if let Some(ref handler) = on_restored {
                                            handler.call(());
                                        }
                                    }
                                    Err(e) => {
                                        restoring_clone.set(None);
                                        eprintln!("Error restoring version: {}", e);
                                    }
                                }
                            });
                        },
                        {if is_restoring { "Bezig..." } else { "Ja, Herstel Versie" }}
                    }
                    button {
                        class: "btn btn-secondary",
                        disabled: is_restoring,
                        onclick: move |_| show_restore_confirm.set(None),
                        "Annuleren"
                    }
                }
            }
        }
    }
}

/// Format a DateTime for display
fn format_date(dt: &DateTime<Utc>) -> String {
    dt.format("%d-%m-%Y %H:%M").to_string()
}

// ==============================================================================
// Types
// ==============================================================================

/// Version view for frontend display
#[derive(Clone, PartialEq, Debug)]
pub struct VersionView {
    pub version_id: String,
    pub document_id: Uuid,
    pub version_number: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub created_by_name: Option<String>,
    pub change_summary: Option<String>,
    pub is_compressed: bool,
    pub parent_version_id: Option<String>,
    pub is_current: bool,
    pub is_active: bool,
}

impl From<VersionViewApi> for VersionView {
    fn from(api: VersionViewApi) -> Self {
        let created_at = api.created_at.parse::<DateTime<Utc>>()
            .unwrap_or_else(|_| Utc::now());

        VersionView {
            version_id: api.id.to_string(),
            document_id: api.document_id,
            version_number: api.version_number,
            created_at,
            created_by: api.created_by,
            created_by_name: api.created_by_name,
            change_summary: if api.change_summary.is_empty() { None } else { Some(api.change_summary) },
            is_compressed: api.is_compressed,
            parent_version_id: api.parent_version_id.map(|id| id.to_string()),
            is_current: api.current,
            is_active: true,
        }
    }
}
