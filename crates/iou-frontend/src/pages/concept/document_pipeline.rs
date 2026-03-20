//! Documentverwerking — koppeling met Werken in context en hybride orchestratie (Camunda + AI).
//!
//! Toont hoe IOU-Modern procesgestuurde documentstromen combineert met agentische AI,
//! in lijn met het concept *Werken in context* (WvdT).

use dioxus::prelude::*;

use crate::components::{Header, Panel};
use crate::Route;

const WERKEN_IN_CONTEXT_URL: &str =
    "https://werkomgeving-van-de-toekomst.github.io/werkenincontext/concept";

#[component]
pub fn ConceptDocumentPipeline() -> Element {
    rsx! {
        div { class: "concept",
            Header {}
            main { class: "container",
                div { class: "context-bar",
                    div { class: "breadcrumb",
                        Link { to: Route::Home, "IOU-Modern" }
                        span { " \u{203A} " }
                        Link { to: Route::ConceptDashboard, "Werken in Context" }
                        span { " \u{203A} " }
                        span { class: "current", "Documentverwerking" }
                    }
                    span { class: "tag woo", "Orchestratie" }
                }

                div { class: "concept-intro",
                    h2 { "Documentverwerking in context" }
                    p {
                        "IOU-Modern kan documentketens "
                        strong { "procesgestuurd" }
                        " laten verlopen (Camunda 8 / Zeebe) en tegelijk "
                        strong { "AI-ondersteunde" }
                        " stappen isoleren in aparte services. Dat sluit aan bij "
                        strong { "Werken in context" }
                        ": de juiste ondersteuning op het juiste moment, met traceerbare stappen en controle over data en tools."
                    }
                    p { style: "margin-top: 12px;",
                        a {
                            class: "btn btn-primary",
                            href: WERKEN_IN_CONTEXT_URL,
                            target: "_blank",
                            rel: "noopener noreferrer",
                            "Concept «Werken in context» (extern) \u{2197}"
                        }
                    }
                }

                Panel { title: "Hybride aanpak: structuur + flexibiliteit".to_string(),
                    p { style: "margin-bottom: 14px; color: #555; font-size: 0.9rem;",
                        "E\u{00e9}n motor bewaakt het proces; zware of onvoorspelbare AI-stappen draaien ge\u{00ef}soleerd. Zo combineer je compliance (wie deed wat, wanneer) met ruimte voor onderzoek en samenvatting."
                    }
                    ul { style: "margin: 0; padding-left: 1.25rem; line-height: 1.6;",
                        li {
                            strong { "Camunda 8 (Zeebe) " }
                            "— BPMN-workflow: o.a. pipeline-job, optionele deep-agent-stap, wachten op goedkeuring, retries en observability."
                        }
                        li {
                            strong { "Rust API & agent-pipeline " }
                            "— domeinlogica, templates, compliance-keten, audit en opslag (o.a. S3-compatibel)."
                        }
                        li {
                            strong { "Deep Agents (Python, container) " }
                            "— service task met tools die alleen via HTTP naar de API gaan; geen secrets in procesvariabelen."
                        }
                    }
                }

                div { style: "height: 16px;" }

                Panel { title: "Hoe dit aansluit op «Werken in context»".to_string(),
                    ul { style: "margin: 0; padding-left: 1.25rem; line-height: 1.65;",
                        li {
                            strong { "Contextuele ondersteuning: " }
                            "taken worden getriggerd vanuit het dossier/document (document-id, domein, template) — niet los van de zaak."
                        }
                        li {
                            strong { "Transparantie: " }
                            "processtappen en auditsporen blijven vindbaar; AI-output landt in het domeinmodel, niet alleen in een chatvenster."
                        }
                        li {
                            strong { "Modulair: " }
                            "Zeebe, workers en Python-runtime kunnen als aparte containers draaien; de kern blijft in Rust."
                        }
                    }
                }

                div { style: "height: 16px;" }

                Panel { title: "Voor beheer en ontwikkeling".to_string(),
                    p { style: "font-size: 0.88rem; color: #666; line-height: 1.55;",
                        "Technische details, omgevingsvariabelen en Docker Compose staan in de repository onder "
                        code { "infra/camunda/" }
                        " (o.a. README en ORCHESTRATION). Activeer Camunda-modus met "
                        code { "IOU_DOCUMENT_WORKFLOW=camunda" }
                        " op de API."
                    }
                    p { style: "margin-top: 10px; font-size: 0.88rem;",
                        Link { to: Route::ConceptArchitectuur,
                            "Terug naar systeemarchitectuur \u{2192}"
                        }
                    }
                }
            }
        }
    }
}
