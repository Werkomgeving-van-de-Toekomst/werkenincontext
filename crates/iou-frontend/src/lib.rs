//! IOU-Modern Frontend - Leptos WebAssembly Application
//!
//! Context-driven information management dashboard voor Nederlandse overheid.

pub mod api;
pub mod components;
pub mod pages;
pub mod state;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use pages::*;

/// Root application component
#[component]
pub fn App() -> impl IntoView {
    // Provide global state
    provide_context(state::AppState::new());

    view! {
        <Title text="IOU-Modern - Informatie Ondersteunde Werkomgeving"/>
        <Meta name="description" content="Context-driven informatiemanagement voor Nederlandse overheid"/>

        <Router>
            <Routes fallback=|| "Pagina niet gevonden">
                <Route path=path!("/") view=Dashboard/>
                <Route path=path!("/context/:id") view=ContextDetail/>
                <Route path=path!("/documents/:id") view=DocumentDetail/>
                <Route path=path!("/search") view=SearchResults/>
                <Route path=path!("/graphrag") view=GraphRAGExplorer/>

                // Context-aware apps
                <Route path=path!("/apps/data-verkenner") view=DataVerkenner/>
                <Route path=path!("/apps/document-generator") view=DocumentGenerator/>
                <Route path=path!("/apps/nalevingscontrole") view=Nalevingscontrole/>
            </Routes>
        </Router>
    }
}

/// Initialize and mount the application
pub fn hydrate() {
    // Better panic messages in browser console
    console_error_panic_hook::set_once();

    // Initialize tracing for WASM
    tracing_wasm::set_as_global_default();

    // Mount the app
    mount_to_body(App);
}
