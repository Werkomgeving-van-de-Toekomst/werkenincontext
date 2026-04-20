//! AI Workflow Chat component
//!
//! Interactive chat interface for AI workflow assistance:
//! - Ask questions about workflows
//! - Request configuration suggestions
//! - Get optimization recommendations

use dioxus::prelude::*;
use uuid::Uuid;

/// AI workflow chat component
#[component]
pub fn AiWorkflowChat(
    #[props(default)] workflow_id: Option<Uuid>,
    #[props(default)] document_id: Option<Uuid>,
    #[props(default)] domain_id: String,
) -> Element {
    let messages = use_signal(|| vec![
        ChatMessage {
            role: MessageRole::Assistant,
            content: "Hallo! Ik kan je helpen met workflow analyse, configuratie en optimalisatie. Wat wil je weten?".to_string(),
        }
    ]);
    let input = use_signal(|| String::new());
    let is_loading = use_signal(|| false);
    let selected_suggestion = use_signal(|| None::<String>);

    let suggestions = vec![
        "Analyseer de workflow prestaties",
        "Suggesties voor verbetering",
        "Genereer workflow configuratie",
        "Optimaliseer deze workflow",
    ];

    let send_message = {
        let messages = messages.clone();
        let input = input.clone();
        let is_loading = is_loading.clone();
        move |_| {
            let text = input.current().trim().to_string();
            if text.is_empty() || *is_loading.current() {
                return;
            }

            messages.with_mut(|msgs| {
                msgs.push(ChatMessage {
                    role: MessageRole::User,
                    content: text.clone(),
                });
            });

            input.set(String::new());
            is_loading.set(true);

            // Simulate AI response (in real implementation, call API)
            let response = generate_ai_response(&text, workflow_id, document_id);
            messages.with_mut(|msgs| {
                msgs.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: response,
                });
            });
            is_loading.set(false);
        }
    };

    let apply_suggestion = {
        let input = input.clone();
        move |suggestion: String| {
            input.set(suggestion);
        }
    };

    rsx! {
        div { class: "ai-workflow-chat",
            // Header
            div { class: "chat-header",
                div { class: "chat-title",
                    svg {
                        class: "chat-icon",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        "stroke-width": "2",
                        path { d: "M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" }
                    }
                    "AI Workflow Assistent"
                }
                div { class: "chat-status",
                    span { class: "status-indicator" }
                    "Online"
                }
            }

            // Messages area
            div { class: "chat-messages",
                {messages.read().iter().map(|msg| rsx! {
                    div {
                        class: if matches!(msg.role, MessageRole::User) {
                            "message user-message"
                        } else {
                            "message assistant-message"
                        },
                        div { class: "message-content",
                            "{msg.content}"
                        }
                        div { class: "message-time",
                            {format_message_time()}
                        }
                    }
                })}

                // Loading indicator
                if *is_loading.current() {
                    div { class: "message assistant-message",
                        div { class: "typing-indicator",
                            span {}
                            span {}
                            span {}
                        }
                    }
                }
            }

            // Suggested prompts
            if messages.read().len() <= 1 {
                div { class: "chat-suggestions",
                    {suggestions.iter().map(|suggestion| rsx! {
                        button {
                            class: "suggestion-chip",
                            onclick: move |_| apply_suggestion(suggestion.to_string()),
                            "{suggestion}"
                        }
                    })}
                }
            }

            // Input area
            div { class: "chat-input-area",
                textarea {
                    class: "chat-input",
                    placeholder: "Stel een vraag over workflows...",
                    value: "{input}",
                    oninput: move |evt| input.set(evt.value()),
                    rows: 1,
                    onkeydown: move |evt| {
                        if evt.key == "Enter" && !evt.shift_key() {
                            evt.prevent_default();
                            send_message(());
                        }
                    }
                }
                button {
                    class: "send-button",
                    disabled: *is_loading.current() || input.read().trim().is_empty(),
                    onclick: send_message,
                    svg {
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        "stroke-width": "2",
                        path { d: "M12 19l9 2-9-18-9 18 9-2zm0 0v-8" }
                    }
                }
            }
        }
    }
}

/// Generate AI response (simulated - would call API in production)
fn generate_ai_response(query: &str, workflow_id: Option<Uuid>, document_id: Option<Uuid>) -> String {
    let query_lower = query.to_lowercase();

    if query_lower.contains("analyse") || query_lower.contains("performance") || query_lower.contains("prestaties") {
        "Op basis van de beschikbare data:\n\n\
         📊 **Workflow Analyse**\n\
         • Gemiddelde doorlooptijd: 48.2 uur\n\
         • SLA naleving: 78.5%\n\
         • Knelpunt: 'Finale Goodkeuring' fase\n\
         • 3 van 5 stagen hebben verbeterpotentieel\n\n\
         Wil je gedetailleerde optimalisatie suggesties?"
            .to_string()
    } else if query_lower.contains("verbeter") || query_lower.contains("optimaliseer") {
        "Hier zijn mijn aanbevelingen:\n\n\
         🔧 **Optimalisatie Suggesties**\n\n\
         1. **Knelpunt aanpakken**\n\
            De 'Finale Goedkeuring' fase duurt gemiddeld 18 uur.\n\
            → Overweeg parallelle goedkeuring\n\
            → Voeg herinneringen toe bij 50% SLA\n\n\
         2. **Proces verbetering**\n\
            Hoge delegatiegraad in 'Juridische Check'.\n\
            → Maak rollen en verantwoordelijkheden duidelijker\n\
            → Breid pool van goedkeurders uit\n\n\
         Verwachte impact: +15% SLA naleving, -12 uur gemiddelde doorlooptijd."
            .to_string()
    } else if query_lower.contains("configuratie") || query_lower.contains("genereer") {
        "Ik kan een workflow configuratie genereren. Beschrijf:\n\n\
         • Het type document\n\
         • Het aantal goedkeuringslagen\n\
         • Specifieke eisen (parallel, sequentieel, etc.)\n\
         • SLA vereisten\n\n\
         Voorbeeld: \"Maak een workflow voor Woo besluiten met 3 goedkeuringslagen: juridisch, management en privacy.\""
            .to_string()
    } else if query_lower.contains("sla") {
        "**SLA Analyse**\n\n\
         | Fase | Gemiddeld | SLA | Naleving |\n\
         |------|----------|-----|----------|\n\
         | Juridische Check | 6.2h | 8h | ✅ 94% |\n\
         | Management Review | 12.4h | 24h | ⚠️ 72% |\n\
         | Finale Goedkeuring | 18.1h | 48h | ❌ 58% |\n\n\
         De laatste fase heeft SLA problemen. Aanbeveling: verleng SLA naar 72h of splits de fase in kleinere stappen."
            .to_string()
    } else {
        "Ik begrijp je vraag. Kan je meer details geven? Ik kan helpen met:\n\n\
         • Workflow performance analyse\n\
         • Configuratie suggesties\n\
         • Optimalisatie aanbevelingen\n\
         • SLA en doorlooptijd advies\n\
         • Goedkeurings proces verbetering"
            .to_string()
    }
}

/// Format message time
fn format_message_time() -> String {
    use chrono::Utc;
    let now = Utc::now();
    format!("{:02}:{:02}", now.hour(), now.minute())
}

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Debug)]
struct ChatMessage {
    role: MessageRole,
    content: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MessageRole {
    User,
    Assistant,
}
