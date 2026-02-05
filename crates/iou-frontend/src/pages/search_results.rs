//! Search results page

use leptos::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn SearchResults() -> impl IntoView {
    view! {
        <Header/>
        <main class="container">
            <Panel title="Zoekresultaten">
                <p>"Implementatie in ontwikkeling..."</p>
            </Panel>
        </main>
    }
}
