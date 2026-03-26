//! Header component

use dioxus::prelude::*;

use crate::state::AppState;

#[component]
pub fn Header() -> Element {
    let state = use_context::<Signal<AppState>>();
    let user = state.read().user.clone();

    rsx! {
        header { class: "header",
            div { style: "display: flex; justify-content: space-between; align-items: center;",
                div {
                    h1 { "IOU-Modern" }
                    p { class: "subtitle", "Informatie Ondersteunde Werkomgeving" }
                }

                if let Some(user) = user {
                    div { class: "user-info",
                        div {
                            div { style: "font-weight: 600;", "{user.name}" }
                            div { style: "font-size: 0.875rem; opacity: 0.9;", "{user.organization}" }
                        }
                        div { class: "user-avatar", "{user.initials}" }
                    }
                }
            }
        }
    }
}
