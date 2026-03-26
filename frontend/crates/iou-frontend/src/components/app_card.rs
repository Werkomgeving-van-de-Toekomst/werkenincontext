//! App card component for context-aware apps

use dioxus::prelude::*;

#[component]
pub fn AppCard(name: String, description: String, badge: Option<String>) -> Element {
    rsx! {
        div { class: "app-card",
            h3 { "{name}" }
            p { "{description}" }
            if let Some(ref badge) = badge {
                span { class: "badge", "{badge}" }
            }
        }
    }
}
