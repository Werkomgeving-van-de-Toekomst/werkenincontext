//! Workflow Generator component
//!
//! Natural language to workflow configuration interface:
//! - Describe workflow in plain text
//! - AI generates structured configuration
//! - Preview and edit generated stages
//! - Export to YAML

use dioxus::prelude::*;

/// Workflow generator component
#[component]
pub fn WorkflowGenerator(
    #[props(default)] domain_id: String,
    #[props(default)] on_config_generated: Option<EventHandler<WorkflowConfig>>,
) -> Element {
    let description = use_signal(|| String::new());
    let document_type = use_signal(|| "woo_besluit".to_string());
    let generated_config = use_signal(|| None::<WorkflowConfig>);
    let is_generating = use_signal(|| false);
    let show_preview = use_signal(|| false);

    let example_descriptions = vec![
        ("Woo besluit workflow", "Maak een workflow voor Woo besluiten met 3 goedkeuringslagen: juridische check, management goedkeuring, en finale controle. SLA van 48 uur per fase."),
        "Simple document approval",
        "Project charter approval with parallel legal and finance review",
    ];

    let generate_config = {
        let description = description.clone();
        let document_type = document_type.clone();
        let generated_config = generated_config.clone();
        let is_generating = is_generating.clone();
        let show_preview = show_preview.clone();
        move |_| {
            let desc = description.read().clone();
            if desc.trim().is_empty() || *is_generating.read() {
                return;
            }

            is_generating.set(true);

            // Simulate API call (would call real API in production)
            let config = generate_mock_config(&desc, &document_type.read());
            generated_config.set(Some(config));
            is_generating.set(false);
            show_preview.set(true);

            if let Some(ref handler) = on_config_generated {
                handler.call(generated_config.read().clone().unwrap());
            }
        }
    };

    let apply_example = {
        let description = description.clone();
        move |example: String| {
            description.set(example);
        }
    };

    let export_yaml = {
        move |_| {
            if let Some(ref config) = *generated_config.read() {
                let yaml = config_to_yaml(config);
                // Trigger download (would use file API in production)
                tracing::info!("Generated YAML:\n{}", yaml);
            }
        }
    };

    rsx! {
        div { class: "workflow-generator",
            h2 { class: "generator-title", "Workflow Generator" }

            div { class: "generator-content",

                // Input section
                div { class: "input-section",
                    label { "Beschrijf je workflow" }
                    textarea {
                        class: "description-input",
                        placeholder: "Bijvoorbeeld: Maak een workflow voor Woo besluiten met juridische check, management goedkeuring en finale controle...",
                        value: "{description}",
                        oninput: move |evt| description.set(evt.value()),
                        rows: 6,
                    }

                    div { class: "input-options",
                        div { class: "option-group",
                            label { "Document Type" }
                            select {
                                value: "{document_type}",
                                oninput: move |evt| document_type.set(evt.value()),
                                option { value: "woo_besluit", "Woo Besluit" }
                                option { value: "woo_informatie", "Woo Informatie" }
                                option { value: "interne_notitie", "Interne Notitie" }
                                option { value: "project_charter", "Project Charter" }
                                option { value: "contract", "Contract" }
                            }
                        }
                    }

                    // Example descriptions
                    div { class: "examples-section",
                        h4 { "Voorbeelden" }
                        div { class: "example-chips",
                            {example_descriptions.iter().map(|(label, example)| rsx! {
                                button {
                                    class: "example-chip",
                                    onclick: move |_| apply_example(example.to_string()),
                                    "{label}"
                                }
                            })}
                        }
                    }

                    button {
                        class: "generate-btn",
                        disabled: description.read().trim().is_empty() || *is_generating.read(),
                        onclick: generate_config,
                        if *is_generating.read() {
                            svg {
                                class: "spin",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                "stroke-width": "2",
                                path { d: "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" }
                            }
                            "Genereren..."
                        } else {
                            svg {
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                "stroke-width": "2",
                                path { d: "M13 10V3L4 14h7v7l9-11h-7z" }
                            }
                            "Genereer Workflow"
                        }
                    }
                }

                // Preview section
                {if *show_preview.read() {
                    generated_config.read().as_ref().map(|config| rsx! {
                        div { class: "preview-section",
                            div { class: "preview-header",
                                h3 { "Generated Workflow" }
                                div { class: "preview-actions",
                                    button {
                                        class: "action-btn secondary",
                                        onclick: export_yaml,
                                        "Export YAML"
                                    }
                                    button {
                                        class: "action-btn",
                                        onclick: move |_| show_preview.set(false),
                                        "Sluiten"
                                    }
                                }
                            }

                            // Workflow info
                            div { class: "workflow-info",
                                div { class: "info-item",
                                    span { class: "info-label", "Naam:" }
                                    span { class: "info-value", "{config.workflow_name}" }
                                }
                                div { class: "info-item",
                                    span { class: "info-label", "Fases:" }
                                    span { class: "info-value", "{}", config.stages.len() }
                                }
                            }

                            // Stages list
                            div { class: "stages-list",
                                {config.stages.iter().enumerate().map(|(i, stage)| rsx! {
                                    div { class: "stage-card",
                                        div { class: "stage-header",
                                            span { class: "stage-order", "{i + 1}" }
                                            h4 { "{stage.stage_name}" }
                                        }
                                        div { class: "stage-details",
                                            div { class: "detail-row",
                                                span { "Type:" }
                                                span { "{format_approval_type(&stage.approval_type)}" }
                                            }
                                            div { class: "detail-row",
                                                span { "SLA:" }
                                                span { "{} uur", stage.sla_hours }
                                            }
                                            div { class: "detail-row",
                                                span { "Goedkeurders:" }
                                                span { "{stage.approvers.len()} rol(len)" }
                                            }
                                        }
                                        {if !stage.approvers.is_empty() {
                                            rsx! {
                                                div { class: "approvers-list",
                                                    {stage.approvers.iter().map(|approver| rsx! {
                                                        span { class: "approver-tag", "{approver.role}" }
                                                    })}
                                                }
                                            }
                                        }}
                                    }
                                })}
                            }

                            // YAML preview
                            details { class: "yaml-preview",
                                summary { "YAML Preview" }
                                pre { class: "yaml-code",
                                    "{config_to_yaml(config)}"
                                }
                            }
                        }
                    })
                } else {
                    rsx! {
                        div { class: "placeholder-section",
                            svg {
                                class: "placeholder-icon",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                "stroke-width": "1.5",
                                path { d: "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" }
                            }
                            p { "Beschrijf je workflow en klik op Genereer om een configuratie te maken" }
                        }
                    }
                }}
            }
        }
    }
}

/// Format approval type for display
fn format_approval_type(approval_type: &ApprovalType) -> &'static str {
    match approval_type {
        ApprovalType::Sequential => "Sequentieel",
        ApprovalType::ParallelAny => "Parallel (één genoeg)",
        ApprovalType::ParallelAll => "Parallel (allen required)",
        ApprovalType::ParallelMajority => "Parallel (meerderheid)",
    }
}

/// Convert config to YAML string
fn config_to_yaml(config: &WorkflowConfig) -> String {
    let mut yaml = format!("workflow_name: {}\n", config.workflow_name);
    yaml.push_str(&format!("description: {}\n", config.description));
    yaml.push_str("stages:\n");

    for stage in &config.stages {
        yaml.push_str(&format!("  - stage_id: {}\n", stage.stage_id));
        yaml.push_str(&format!("    stage_name: {}\n", stage.stage_name));
        yaml.push_str(&format!("    stage_order: {}\n", stage.stage_order));
        yaml.push_str(&format!("    approval_type: {:?}\n", stage.approval_type));
        yaml.push_str(&format!("    sla_hours: {}\n", stage.sla_hours));
        yaml.push_str("    approvers:\n");
        for approver in &stage.approvers {
            yaml.push_str(&format!("      - role: {}\n", approver.role));
        }
    }

    yaml
}

/// Mock config generation (would call real API in production)
fn generate_mock_config(description: &str, document_type: &str) -> WorkflowConfig {
    let has_parallel = description.to_lowercase().contains("parallel");
    let has_legal = description.to_lowercase().contains("juridisch") || description.to_lowercase().contains("legal");
    let has_management = description.to_lowercase().contains("management");

    WorkflowConfig {
        workflow_name: format!("{} Workflow", document_type),
        description: description.to_string(),
        stages: vec![
            StageConfig {
                stage_id: "stage_1".to_string(),
                stage_name: if has_legal { "Juridische Check" } else { "Initiële Review" }.to_string(),
                stage_order: 1,
                approval_type: ApprovalType::ParallelAny,
                approvers: vec![ApproverConfig { role: "Jurist".to_string() }],
                sla_hours: 24,
                is_optional: false,
            },
            StageConfig {
                stage_id: "stage_2".to_string(),
                stage_name: if has_management { "Management Goedkeuring" } else { "Review" }.to_string(),
                stage_order: 2,
                approval_type: if has_parallel { ApprovalType::ParallelAll } else { ApprovalType::Sequential },
                approvers: vec![
                    ApproverConfig { role: "Manager".to_string() },
                    ApproverConfig { role: "Team Lead".to_string() },
                ],
                sla_hours: 48,
                is_optional: false,
            },
            StageConfig {
                stage_id: "stage_3".to_string(),
                stage_name: "Finale Controle".to_string(),
                stage_order: 3,
                approval_type: ApprovalType::Sequential,
                approvers: vec![ApproverConfig { role: "Directeur".to_string() }],
                sla_hours: 48,
                is_optional: false,
            },
        ],
    }
}

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct WorkflowConfig {
    pub workflow_name: String,
    pub description: String,
    pub stages: Vec<StageConfig>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StageConfig {
    pub stage_id: String,
    pub stage_name: String,
    pub stage_order: i32,
    pub approval_type: ApprovalType,
    pub approvers: Vec<ApproverConfig>,
    pub sla_hours: i32,
    pub is_optional: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ApproverConfig {
    pub role: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ApprovalType {
    Sequential,
    ParallelAny,
    ParallelAll,
    ParallelMajority,
}
