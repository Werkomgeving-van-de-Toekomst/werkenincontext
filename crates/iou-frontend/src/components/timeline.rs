//! Timeline component - visual timeline display for recent documents

use dioxus::prelude::*;

/// Represents a single timeline event/document
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineEvent {
    pub id: String,
    pub title: String,
    pub date: String,
    pub date_display: String,
    pub description: String,
    pub event_type: TimelineEventType,
    pub url: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TimelineEventType {
    Document,
    Email,
    Chat,
    Besluit,
    ProjectMilestone,
    Note,
}

impl TimelineEventType {
    pub fn icon(&self) -> &str {
        match self {
            TimelineEventType::Document => "\u{1F4C4}",
            TimelineEventType::Email => "\u{2709}",
            TimelineEventType::Chat => "\u{1F4AC}",
            TimelineEventType::Besluit => "\u{1F4DD}",
            TimelineEventType::ProjectMilestone => "\u{1F3AF}",
            TimelineEventType::Note => "\u{1F4CB}",
        }
    }

    pub fn color_class(&self) -> &str {
        match self {
            TimelineEventType::Document => "timeline-doc",
            TimelineEventType::Email => "timeline-email",
            TimelineEventType::Chat => "timeline-chat",
            TimelineEventType::Besluit => "timeline-besluit",
            TimelineEventType::ProjectMilestone => "timeline-milestone",
            TimelineEventType::Note => "timeline-note",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            TimelineEventType::Document => "Document",
            TimelineEventType::Email => "E-mail",
            TimelineEventType::Chat => "Chat",
            TimelineEventType::Besluit => "Besluit",
            TimelineEventType::ProjectMilestone => "Mijlpaal",
            TimelineEventType::Note => "Notitie",
        }
    }
}

/// Timeline item component - individual event with collapsible details
#[component]
fn TimelineItem(event: TimelineEvent) -> Element {
    let mut expanded = use_signal(|| false);

    rsx! {
        div {
            class: format_args!("timeline-item {} {}", event.event_type.color_class(), if *expanded.read() { "expanded" } else { "" }),
            key: "{event.id}",

            div { class: "timeline-marker",
                div { class: "timeline-dot",
                    "{event.event_type.icon()}"
                }
                div { class: "timeline-line" }
            }

            div { class: "timeline-content",
                div { class: "timeline-date", "{event.date_display}" }
                div { class: "timeline-event-type", "{event.event_type.label()}" }

                if let Some(url) = &event.url {
                    a {
                        href: "{url}",
                        target: "_blank",
                        class: "timeline-link-title",
                        h4 { class: "timeline-title-text-inline",
                            "{event.title}"
                        }
                    }
                } else {
                    h4 { class: "timeline-title-text-inline",
                        onclick: move |_| expanded.toggle(),
                        style: "cursor: pointer;",
                        "{event.title}"
                    }
                }

                button {
                    class: format_args!("timeline-toggle-btn {}", if *expanded.read() { "expanded" } else { "collapsed" }),
                    onclick: move |_| expanded.toggle(),
                    if *expanded.read() { "Verberg details" } else { "Toon details" }
                }

                if *expanded.read() {
                    div { class: "timeline-description",
                        p {
                            "{event.description}"
                        }
                    }
                }
            }
        }
    }
}

/// Timeline component - displays events in chronological order
/// Content is collapsible - click to expand details
#[component]
pub fn Timeline(
    title: String,
    events: Vec<TimelineEvent>,
    #[props(default = 5)] max_items: usize,
    #[props(default = false)] show_context: bool,
    #[props(default = None)] context_label: Option<String>,
) -> Element {
    let display_events = events.into_iter().take(max_items).collect::<Vec<_>>();

    rsx! {
        div { class: "timeline-container",
            if !title.is_empty() {
                div { class: "timeline-header",
                    h3 { class: "timeline-title", "{title}" }
                    if let Some(context) = &context_label {
                        span { class: "timeline-context", "{context}" }
                    }
                }
            }

            div { class: "timeline",
                for event in display_events {
                    TimelineItem { event: event.clone() }
                }
            }
        }
    }
}

/// Timeline panel wrapper - integrates Timeline with Panel component
#[component]
pub fn TimelinePanel(
    title: String,
    events: Vec<TimelineEvent>,
    #[props(default = 5)] max_items: usize,
    #[props(default = None)] context_label: Option<String>,
) -> Element {
    rsx! {
        div { class: "panel",
            div { class: "panel-header",
                h2 { "{title}" }
            }
            div { class: "panel-content",
                Timeline {
                    title: String::new(),
                    events,
                    max_items,
                    show_context: true,
                    context_label,
                }
            }
        }
    }
}
