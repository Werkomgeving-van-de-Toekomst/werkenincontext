//! Delegation manager component
//!
//! Provides UI for creating, viewing, and revoking delegations.
//! Lists both delegations created by the current user and delegations received from others.

use dioxus::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::api::documents::{
    list_delegations, revoke_delegation,
    DelegationsListResponse, DelegationView as ApiDelegationView,
};

/// Main delegation manager component
#[component]
pub fn DelegationManager(
    user_id: Uuid,
) -> Element {
    let mut show_create_form = use_signal(|| false);
    let mut refresh_trigger = use_signal(|| 0u32);

    let delegations = use_resource(move || {
        let _trigger = *refresh_trigger.read();
        async move {
            list_delegations().await.unwrap_or_else(|_| DelegationsListResponse {
                created: Vec::new(),
                received: Vec::new(),
            })
        }
    });

    let mut refresh = move || {
        let current = *refresh_trigger.read();
        refresh_trigger.set(current + 1);
    };

    rsx! {
        div { class: "delegation-manager",
            h2 { "Mijn Delegaties" }

            // Create button
            button {
                class: "btn btn-primary",
                onclick: move |_| {
                    let current = *show_create_form.read();
                    show_create_form.set(!current);
                },
                "+ Nieuwe Delegatie"
            }

            // Create form (collapsible)
            if *show_create_form.read() {
                CreateDelegationForm {
                    user_id: user_id,
                    on_created: move || {
                        show_create_form.set(false);
                        refresh();
                    },
                    on_cancel: move || {
                        show_create_form.set(false);
                    },
                }
            }

            // Delegations list
            match &*delegations.read() {
                Some(delegations_list) => rsx! {
                    if !delegations_list.created.is_empty() || !delegations_list.received.is_empty() {
                        DelegationList {
                            delegations_api: delegations_list.clone(),
                            on_revoked: move || {
                                refresh();
                            },
                        }
                    } else {
                        div { class: "empty-delegations",
                            p { "Geen actieve delegaties" }
                            p { class: "empty-note",
                                "Maak een delegatie om je goedkeuringsrechten over te dragen aan een andere gebruiker."
                            }
                        }
                    }
                },
                None => rsx! {
                    div { class: "loading-delegations",
                        div { class: "spinner" }
                        p { "Delegaties laden..." }
                    }
                },
            }
        }
    }
}

/// Delegations list component
#[component]
fn DelegationList(
    delegations_api: DelegationsListResponse,
    on_revoked: EventHandler<()>,
) -> Element {
    let mut show_revoke_confirm = use_signal(|| None::<Uuid>);
    let mut revoking = use_signal(|| None::<Uuid>);

    let revoke = move |delegation_id: Uuid| {
        let id = delegation_id;
        let mut revoking_clone = revoking.clone();
        let on_revoked = on_revoked.clone();

        spawn(async move {
            revoking_clone.set(Some(id));
            match revoke_delegation(id).await {
                Ok(_) => {
                    revoking_clone.set(None);
                    on_revoked.call(());
                }
                Err(e) => {
                    revoking_clone.set(None);
                    eprintln!("Error revoking delegation: {}", e);
                }
            }
        });
    };

    // Convert API delegations to frontend view
    let created: Vec<DelegationView> = delegations_api.created.iter()
        .map(|d| DelegationView::from_api(d.clone()))
        .collect();
    let received: Vec<DelegationView> = delegations_api.received.iter()
        .map(|d| DelegationView::from_api(d.clone()))
        .collect();

    rsx! {
        div { class: "delegation-lists",
            // Created delegations
            if !created.is_empty() {
                div { class: "delegation-section",
                    h3 { "Aangemaakt" }
                    p { class: "section-description", "Delegaties die je hebt gemaakt aan andere gebruikers" }

                    {created.iter().map(|delegation| {
                        let delegation_id = delegation.id;
                        let is_revoking_val = revoking.read().as_ref() == Some(&delegation_id);
                        let show_confirm_val = show_revoke_confirm.read().as_ref() == Some(&delegation_id);
                        rsx!(
                            DelegationCard {
                                delegation: delegation.clone(),
                                is_owner: true,
                                is_revoking: is_revoking_val,
                                show_confirm: show_confirm_val,
                                on_revoke: move || revoke(delegation_id),
                                on_confirm: move || revoke(delegation_id),
                                on_cancel: move || show_revoke_confirm.set(None),
                            }
                        )
                    })}
                }
            }

            // Received delegations
            if !received.is_empty() {
                div { class: "delegation-section",
                    h3 { "Ontvangen" }
                    p { class: "section-description", "Delegaties die je van andere gebruikers hebt ontvangen" }

                    {received.iter().map(|delegation| rsx! {
                        DelegationCard {
                            delegation: delegation.clone(),
                            is_owner: false,
                            is_revoking: false,
                            show_confirm: false,
                            on_revoke: |_| {},
                            on_confirm: |_| {},
                            on_cancel: |_| {},
                        }
                    })}
                }
            }
        }
    }
}

/// Individual delegation card
#[component]
fn DelegationCard(
    delegation: DelegationView,
    is_owner: bool,
    is_revoking: bool,
    show_confirm: bool,
    on_revoke: EventHandler<()>,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let type_badge = match delegation.delegation_type {
        DelegationType::Temporary => ("Tijdelijk", "badge-temporary"),
        DelegationType::Permanent => ("Permanent", "badge-permanent"),
        DelegationType::Bulk => ("Bulk", "badge-bulk"),
    };

    let starts_at = delegation.starts_at.format("%d-%m-%Y").to_string();
    let ends_at = delegation.ends_at.map(|d| d.format("%d-%m-%Y").to_string());

    let doc_types_text = delegation.document_types.join(", ");
    let doc_id_text = delegation.document_id.map(|id| id.to_string());

    rsx! {
        div {
            class: if delegation.is_active { "delegation-card active" }
                   else { "delegation-card inactive" },
            div { class: "delegation-header",
                div { class: "delegation-type-badge {type_badge.1}",
                    "{type_badge.0}"
                }
                if !delegation.is_active {
                    span { class: "status-badge inactive", "Inactief" }
                } else {
                    span { class: "status-badge active", "Actief" }
                }
            }

            div { class: "delegation-body",
                if is_owner {
                    div { class: "delegation-row",
                        span { class: "row-label", "Aan:" }
                        span { class: "row-value",
                            {delegation.to_user_name.as_deref().unwrap_or("Onbekend")}
                        }
                    }
                } else {
                    div { class: "delegation-row",
                        span { class: "row-label", "Van:" }
                        span { class: "row-value",
                            {delegation.from_user_name.as_deref().unwrap_or("Onbekend")}
                        }
                    }
                }

                div { class: "delegation-row",
                    span { class: "row-label", "Van:" }
                    span { class: "row-value", "{starts_at}" }
                }

                if let Some(end) = ends_at {
                    div { class: "delegation-row",
                        span { class: "row-label", "Tot:" }
                        span { class: "row-value", "{end}" }
                    }
                }

                if !doc_types_text.is_empty() {
                    div { class: "delegation-row",
                        span { class: "row-label", "Document types:" }
                        span { class: "row-value", "{doc_types_text}" }
                    }
                }

                if let Some(doc_id) = &doc_id_text {
                    div { class: "delegation-row",
                        span { class: "row-label", "Document ID:" }
                        span { class: "row-value doc-id",
                            code { "{doc_id}" }
                        }
                    }
                }
            }

            div { class: "delegation-footer",
                if is_owner && delegation.is_active {
                    if show_confirm {
                        div { class: "revoke-confirmation",
                            span { "Weet je zeker dat je deze delegatie wilt intrekken?" }
                            div { class: "confirmation-actions",
                                button {
                                    class: "btn btn-danger btn-sm",
                                    disabled: is_revoking,
                                    onclick: move |_| on_confirm.call(()),
                                    {if is_revoking { "Intrekken..." } else { "Ja, intrekken" }}
                                }
                                button {
                                    class: "btn btn-secondary btn-sm",
                                    disabled: is_revoking,
                                    onclick: move |_| on_cancel.call(()),
                                    "Annuleren"
                                }
                            }
                        }
                    } else {
                        button {
                            class: "btn btn-outline btn-sm",
                            disabled: is_revoking,
                            onclick: move |_| on_revoke.call(()),
                            "Intrekken"
                        }
                    }
                }
            }
        }
    }
}

/// Create delegation form
#[component]
fn CreateDelegationForm(
    user_id: Uuid,
    on_created: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut errors = use_signal(|| Vec::<String>::new());
    let is_submitting = use_signal(|| false);

    // Form state would go here - for now using placeholder handlers
    let mut submit = move || {
        // Validation placeholder
        errors.set(vec!["Formulier validatie nog niet geïmplementeerd".to_string()]);
    };

    rsx! {
        div { class: "create-delegation-form",
            h3 { "Nieuwe Delegatie Maken" }

            if !errors.read().is_empty() {
                div { class: "alert alert-error",
                    svg {
                        class: "alert-icon",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        path { d: "M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" }
                    }
                    {errors.read().iter().map(|e| rsx! {
                        p { "{e}" }
                    })}
                }
            }

            // Form fields
            div { class: "form-group",
                label { "Gebruiker" }
                input {
                    r#type: "text",
                    placeholder: "Zoek op naam...",
                    disabled: *is_submitting.read(),
                }
            }

            div { class: "form-group",
                label { "Type" }
                select {
                    disabled: *is_submitting.read(),
                    option { value: "temporary", "Tijdelijk" }
                    option { value: "permanent", "Permanent" }
                    option { value: "bulk", "Bulk" }
                }
            }

            div { class: "form-group",
                label { "Einddatum" }
                input {
                    r#type: "datetime-local",
                    disabled: *is_submitting.read(),
                }
            }

            // Actions
            div { class: "form-actions",
                button {
                    class: "btn btn-primary",
                    disabled: *is_submitting.read(),
                    onclick: move |_| submit(),
                    {if *is_submitting.read() { "Bezig..." } else { "Maak Delegatie" }}
                }
                button {
                    class: "btn btn-secondary",
                    onclick: move |_| on_cancel.call(()),
                    disabled: *is_submitting.read(),
                    "Annuleren"
                }
            }
        }
    }
}

// ==============================================================================
// Types
// ==============================================================================

/// Delegation view for frontend display
#[derive(Clone, PartialEq, Debug)]
pub struct DelegationView {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub from_user_name: Option<String>,
    pub to_user_id: Uuid,
    pub to_user_name: Option<String>,
    pub delegation_type: DelegationType,
    pub document_types: Vec<String>,
    pub document_id: Option<Uuid>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

impl DelegationView {
    fn from_api(api: ApiDelegationView) -> Self {
        let delegation_type = match api.delegation_type.as_str() {
            "permanent" => DelegationType::Permanent,
            "bulk" => DelegationType::Bulk,
            _ => DelegationType::Temporary,
        };

        let starts_at = api.starts_at.parse::<DateTime<Utc>>()
            .unwrap_or_else(|_| Utc::now());

        let ends_at = api.ends_at.and_then(|d| d.parse().ok());

        DelegationView {
            id: api.id,
            from_user_id: api.from_user_id,
            from_user_name: api.from_user_name,
            to_user_id: api.to_user_id,
            to_user_name: api.to_user_name,
            delegation_type,
            document_types: api.document_types,
            document_id: api.document_id,
            starts_at,
            ends_at,
            is_active: api.is_active,
        }
    }
}

/// Delegation type
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DelegationType {
    #[default]
    Temporary,
    Permanent,
    Bulk,
}

impl DelegationType {
    pub fn to_string(&self) -> String {
        match self {
            DelegationType::Temporary => "temporary".to_string(),
            DelegationType::Permanent => "permanent".to_string(),
            DelegationType::Bulk => "bulk".to_string(),
        }
    }
}
