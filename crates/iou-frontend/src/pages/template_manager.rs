//! Template management page
//!
//! CRUD interface for managing document templates.

use dioxus::prelude::*;
use crate::api::documents::{list_templates, get_template, create_template, update_template, delete_template, Template, CreateTemplateRequest};

#[component]
pub fn TemplateManager() -> Element {
    let mut templates = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let mut editing = use_signal(|| None::<Template>);
    let mut error = use_signal(|| None::<String>);
    let mut success = use_signal(|| None::<String>);

    use_effect(move || {
        spawn(async move {
            match list_templates(None).await {
                Ok(tmpls) => {
                    templates.set(tmpls);
                    loading.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to load templates: {}", e)));
                    loading.set(false);
                }
            }
        });
    });

    // Clone editing template for use in closures
    let editing_tmpl = editing.read().clone();

    rsx! {
        div { class: "template-manager",
            div { class: "template-header",
                h1 { "Template Beheer" }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        editing.set(Some(Template::empty()));
                    },
                    "+ Nieuwe Template"
                }
            }

            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-error", "{err}" }
            }

            if let Some(msg) = success.read().as_ref() {
                div { class: "alert alert-success", "{msg}" }
            }

            if *loading.read() {
                div { class: "loading", "Laden..." }
            } else {
                div { class: "template-list",
                    if templates.read().is_empty() {
                        div { class: "empty-state",
                            p { "Geen templates gevonden." }
                            p { class: "empty-note", "Maak uw eerste template aan om te beginnen." }
                        }
                    } else {
                        for tmpl in templates.read().iter().cloned() {
                            div { class: "template-item",
                                div { class: "template-item-header",
                                    h3 { "{tmpl.name}" }
                                    span { class: "template-version", "v{tmpl.version}" }
                                }
                                div { class: "template-meta",
                                    span { "Domein: {tmpl.domain_id}" }
                                    span { "Type: {tmpl.document_type}" }
                                    if !tmpl.is_active {
                                        span { class: "inactive-badge", "Inactief" }
                                    }
                                }
                                div { class: "template-actions",
                                    button {
                                        class: "btn btn-secondary btn-sm",
                                        onclick: move |_| {
                                            let tmpl_id = tmpl.id.clone();
                                            spawn(async move {
                                                match get_template(tmpl_id).await {
                                                    Ok(full_template) => {
                                                        editing.set(Some(full_template));
                                                    }
                                                    Err(e) => {
                                                        error.set(Some(format!("Failed to load template: {}", e)));
                                                    }
                                                }
                                            });
                                        },
                                        "Bewerken"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(tmpl) = editing_tmpl.as_ref() {
                TemplateEditor {
                    template: tmpl.clone(),
                    on_save: move |saved: Template| {
                        let tmpl_for_closure = editing_tmpl.clone();
                        let saved_name = saved.name.clone();
                        spawn(async move {
                            let is_new = tmpl_for_closure.as_ref().map(|t| t.id.is_empty()).unwrap_or(true);
                            let tmpl_id = tmpl_for_closure.as_ref().map(|t| t.id.clone()).unwrap_or_default();

                            let success_val = if is_new {
                                match create_template(&CreateTemplateRequest {
                                    name: saved.name.clone(),
                                    domain_id: saved.domain_id.clone(),
                                    document_type: saved.document_type.clone(),
                                    content: saved.content.clone(),
                                    required_variables: saved.required_variables.clone(),
                                    optional_sections: saved.optional_sections.clone(),
                                }).await {
                                    Ok(_) => true,
                                    Err(e) => {
                                        error.set(Some(format!("Failed to save template: {}", e)));
                                        return;
                                    }
                                }
                            } else {
                                match update_template(tmpl_id.clone(), &saved).await {
                                    Ok(_) => true,
                                    Err(e) => {
                                        error.set(Some(format!("Failed to save template: {}", e)));
                                        return;
                                    }
                                }
                            };

                            if success_val {
                                success.set(Some(if is_new {
                                    format!("Template '{}' aangemaakt", saved_name)
                                } else {
                                    format!("Template '{}' bijgewerkt", saved_name)
                                }));
                                if let Ok(t) = list_templates(None).await {
                                    templates.set(t);
                                }
                                editing.set(None);
                            }
                        });
                    },
                    on_cancel: move |_| editing.set(None)
                }
            }
        }
    }
}

#[component]
fn TemplateEditor(
    template: Template,
    on_save: EventHandler<Template>,
    on_cancel: EventHandler,
) -> Element {
    let mut name = use_signal(|| template.name.clone());
    let mut domain_id = use_signal(|| template.domain_id.clone());
    let mut document_type = use_signal(|| template.document_type.clone());
    let mut content = use_signal(|| template.content.clone());
    let mut required_vars = use_signal(|| template.required_variables.join(", "));
    let mut optional_sections = use_signal(|| template.optional_sections.join(", "));

    rsx! {
        div { class: "template-editor-overlay",
            div { class: "template-editor",
                div { class: "template-editor-header",
                    h2 { if template.id.is_empty() { "Nieuwe Template" } else { "Template Bewerken" } }
                    button {
                        class: "btn-close",
                        onclick: move |_| on_cancel.call(()),
                        "×"
                    }
                }

                form {
                    onsubmit: move |evt: Event<FormData>| {
                        evt.prevent_default();
                        let updated = Template {
                            id: template.id.clone(),
                            name: name.read().clone(),
                            domain_id: domain_id.read().clone(),
                            document_type: document_type.read().clone(),
                            content: content.read().clone(),
                            required_variables: required_vars.read()
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect(),
                            optional_sections: optional_sections.read()
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect(),
                            version: template.version,
                            is_active: template.is_active,
                            created_at: template.created_at.clone(),
                            updated_at: template.updated_at.clone(),
                        };
                        on_save.call(updated);
                    },

                    div { class: "form-group",
                        label { "Naam" }
                        input {
                            r#type: "text",
                            value: "{name}",
                            required: true,
                            oninput: move |evt: Event<FormData>| {
                                let value = evt.value();
                                name.set(value);
                            }
                        }
                    }

                    div { class: "form-group",
                        label { "Domein ID" }
                        input {
                            r#type: "text",
                            value: "{domain_id}",
                            required: true,
                            placeholder: "bijv. woo_minfin",
                            oninput: move |evt: Event<FormData>| {
                                let value = evt.value();
                                domain_id.set(value);
                            }
                        }
                    }

                    div { class: "form-group",
                        label { "Document Type" }
                        input {
                            r#type: "text",
                            value: "{document_type}",
                            required: true,
                            placeholder: "bijv. woo_besluit",
                            oninput: move |evt: Event<FormData>| {
                                let value = evt.value();
                                document_type.set(value);
                            }
                        }
                    }

                    div { class: "form-group",
                        label { "Content (Markdown)" }
                        textarea {
                            class: "template-content",
                            rows: "20",
                            value: "{content}",
                            required: true,
                            oninput: move |evt: Event<FormData>| {
                                let value = evt.value();
                                content.set(value);
                            }
                        }
                    }

                    div { class: "form-group",
                        label { "Vereiste Variabelen (komma-gescheiden)" }
                        input {
                            r#type: "text",
                            value: "{required_vars}",
                            placeholder: "bijv. reference, title, date",
                            oninput: move |evt: Event<FormData>| {
                                let value = evt.value();
                                required_vars.set(value);
                            }
                        }
                    }

                    div { class: "form-group",
                        label { "Optionele Secties (komma-gescheiden)" }
                        input {
                            r#type: "text",
                            value: "{optional_sections}",
                            placeholder: "bijv. references, appendix",
                            oninput: move |evt: Event<FormData>| {
                                let value = evt.value();
                                optional_sections.set(value);
                            }
                        }
                    }

                    div { class: "form-actions",
                        button {
                            class: "btn btn-primary",
                            r#type: "submit",
                            "Opslaan"
                        }
                        button {
                            class: "btn btn-secondary",
                            r#type: "button",
                            onclick: move |_| on_cancel.call(()),
                            "Annuleren"
                        }
                    }
                }
            }
        }
    }
}
