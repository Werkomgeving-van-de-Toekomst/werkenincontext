//! Header component

use leptos::prelude::*;

use crate::state::AppState;

#[component]
pub fn Header() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState must be provided");

    view! {
        <header class="header">
            <div style="display: flex; justify-content: space-between; align-items: center;">
                <div>
                    <h1>"IOU-Modern"</h1>
                    <p class="subtitle">"Informatie Ondersteunde Werkomgeving"</p>
                </div>

                {move || {
                    state.user.get().map(|user| {
                        view! {
                            <div class="user-info">
                                <div>
                                    <div style="font-weight: 600;">{user.name}</div>
                                    <div style="font-size: 0.875rem; opacity: 0.9;">{user.organization}</div>
                                </div>
                                <div class="user-avatar">{user.initials}</div>
                            </div>
                        }
                    })
                }}
            </div>
        </header>
    }
}
