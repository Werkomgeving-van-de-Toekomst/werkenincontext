//! Document creation page
//!
//! Form-based page for initiating document generation through the AI agent pipeline.

use dioxus::prelude::*;
use crate::api::documents::{create_document, list_templates};

#[component]
pub fn DocumentCreator() -> Element {
    let mut domain_id = use_signal(|| String::new());
    let mut document_type = use_signal(|| String::new());
    let mut context_reference = use_signal(|| String::new());
    let mut context_title = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut success = use_signal(|| None::<String>);
    let mut available_templates = use_signal(Vec::new);

    // Fetch available templates on mount
    use_effect(move || {
        spawn(async move {
            match list_templates(None).await {
                Ok(templates) => {
                    available_templates.set(templates);
                }
                Err(_e) => {
                    // Silently fail for now
                }
            }
        });
    });

    // Get available document types for selected domain
    let mut available_types = use_signal(Vec::new);
    use_effect(move || {
        let domain = domain_id.read().clone();
        if domain.is_empty() {
            available_types.set(Vec::new());
        } else {
            let types: Vec<String> = available_templates.read()
                .iter()
                .filter(|t| t.domain_id == domain)
                .map(|t| t.document_type.clone())
                .collect();
            available_types.set(types);
        }
    });

    let submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        let domain = domain_id.read().clone();
        let doc_type = document_type.read().clone();
        let reference = context_reference.read().clone();
        let title = context_title.read().clone();

        if domain.is_empty() || doc_type.is_empty() {
            error.set(Some("Selecteer een domein en document type".to_string()));
            return;
        }

        let mut context = serde_json::Map::new();
        if !reference.is_empty() {
            context.insert("reference".to_string(), serde_json::Value::String(reference));
        }
        if !title.is_empty() {
            context.insert("title".to_string(), serde_json::Value::String(title));
        }

        let request = crate::api::documents::CreateDocumentRequest {
            domain_id: domain,
            document_type: doc_type,
            context,
        };

        loading.set(true);
        error.set(None);

        spawn(async move {
            match create_document(&request).await {
                Ok(response) => {
                    success.set(Some(format!(
                        "Document created: {}. Status: {}",
                        response.document_id, response.state
                    )));
                    loading.set(false);
                    // Clear form
                    domain_id.set(String::new());
                    document_type.set(String::new());
                    context_reference.set(String::new());
                    context_title.set(String::new());
                }
                Err(e) => {
                    error.set(Some(e));
                    loading.set(false);
                }
            }
        });
    };

    let domains: Vec<String> = available_templates.read()
        .iter()
        .map(|t| t.domain_id.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    rsx! {
        div { class: "document-creator-container",
            h1 { "Document Maken" }

            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-error", "{err}" }
            }

            if let Some(msg) = success.read().as_ref() {
                div { class: "alert alert-success", "{msg}" }
            }

            form { class: "document-form", onsubmit: submit,
                // Domain selection
                div { class: "form-group",
                    label { "Domein" }
                    select {
                        name: "domain",
                        value: "{domain_id}",
                        oninput: move |evt: Event<FormData>| {
                            let value = evt.value();
                            domain_id.set(value);
                            document_type.set(String::new());
                        },
                        option { value: "", "Selecteer domein..." }
                        for domain in domains.iter() {
                            option { value: "{domain}", "{domain}" }
                        }
                    }
                }

                // Document type selection
                div { class: "form-group",
                    label { "Document Type" }
                    select {
                        name: "document_type",
                        value: "{document_type}",
                        oninput: move |evt: Event<FormData>| {
                            let value = evt.value();
                            document_type.set(value);
                        },
                        option { value: "", "Selecteer type..." }
                        for doc_type in available_types.read().iter() {
                            option { value: "{doc_type}", "{doc_type}" }
                        }
                    }
                }

                // Context fields
                div { class: "form-group",
                    label { "Referentie" }
                    input {
                        r#type: "text",
                        name: "reference",
                        value: "{context_reference}",
                        placeholder: "bijv. REF-2024-001",
                        oninput: move |evt: Event<FormData>| {
                            let value = evt.value();
                            context_reference.set(value);
                        }
                    }
                }

                div { class: "form-group",
                    label { "Titel" }
                    input {
                        r#type: "text",
                        name: "title",
                        value: "{context_title}",
                        placeholder: "Document titel...",
                        oninput: move |evt: Event<FormData>| {
                            let value = evt.value();
                            context_title.set(value);
                        }
                    }
                }

                div { class: "form-actions",
                    button {
                        class: "btn btn-primary",
                        r#type: "submit",
                        disabled: *loading.read(),
                        if *loading.read() { "Bezig..." } else { "Maken" }
                    }
                }
            }
        }
    }
}
