//! App card component for context-aware apps

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn AppCard(
    name: String,
    description: String,
    endpoint: String,
    #[prop(optional)] badge: Option<String>,
) -> impl IntoView {
    let navigate = use_navigate();
    let endpoint_clone = endpoint.clone();

    let on_click = move |_| {
        let nav = navigate.clone();
        let ep = endpoint_clone.clone();
        nav(&ep, Default::default());
    };

    view! {
        <div class="app-card" on:click=on_click>
            <h3>{name}</h3>
            <p>{description}</p>
            {badge.map(|b| view! { <span class="badge">{b}</span> })}
        </div>
    }
}
