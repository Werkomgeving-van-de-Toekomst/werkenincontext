//! Context detail page

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::{Header, Panel};

#[component]
pub fn ContextDetail() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();

    view! {
        <Header/>
        <main class="container">
            <Panel title="Context Details">
                <p>"Context ID: " {id}</p>
                <p>"Implementatie in ontwikkeling..."</p>
            </Panel>
        </main>
    }
}
