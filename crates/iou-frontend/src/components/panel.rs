//! Panel component - reusable card with header

use leptos::prelude::*;

#[component]
pub fn Panel(
    title: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="panel">
            <div class="panel-header">
                <h2>{title}</h2>
            </div>
            <div class="panel-content">
                {children()}
            </div>
        </div>
    }
}
