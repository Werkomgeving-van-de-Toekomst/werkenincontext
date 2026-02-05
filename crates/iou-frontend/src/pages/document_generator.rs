//! Document Generator app

use leptos::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn DocumentGenerator() -> impl IntoView {
    let (step, set_step) = signal(1);

    view! {
        <Header/>
        <main class="container">
            <Panel title="Document Generator">
                // Wizard steps
                <div style="display: flex; gap: 10px; margin-bottom: 20px;">
                    <div class={move || if step.get() >= 1 { "tag woo" } else { "tag" }}>"1. Type"</div>
                    <div class={move || if step.get() >= 2 { "tag woo" } else { "tag" }}>"2. Content"</div>
                    <div class={move || if step.get() >= 3 { "tag woo" } else { "tag" }}>"3. Metadata"</div>
                    <div class={move || if step.get() >= 4 { "tag woo" } else { "tag" }}>"4. Compliance"</div>
                    <div class={move || if step.get() >= 5 { "tag woo" } else { "tag" }}>"5. Preview"</div>
                </div>

                {move || match step.get() {
                    1 => view! {
                        <div>
                            <h3 style="margin-bottom: 15px;">"Selecteer documenttype"</h3>
                            <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 15px;">
                                <div class="app-card" on:click=move |_| set_step.set(2)>
                                    <h3>"Adviesbrief"</h3>
                                    <p>"Formeel advies aan college of raad"</p>
                                </div>
                                <div class="app-card" on:click=move |_| set_step.set(2)>
                                    <h3>"Besluit"</h3>
                                    <p>"Officieel besluit op aanvraag"</p>
                                </div>
                                <div class="app-card" on:click=move |_| set_step.set(2)>
                                    <h3>"Raadsvoorstel"</h3>
                                    <p>"Voorstel voor provinciale staten"</p>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    2 => view! {
                        <div>
                            <h3 style="margin-bottom: 15px;">"Document inhoud"</h3>
                            <textarea
                                style="width: 100%; height: 200px; padding: 10px; border: 1px solid #ddd; border-radius: 8px;"
                                placeholder="Voer de documentinhoud in..."
                            ></textarea>
                            <div style="display: flex; justify-content: space-between; margin-top: 15px;">
                                <button class="btn btn-secondary" on:click=move |_| set_step.set(1)>"Vorige"</button>
                                <button class="btn btn-primary" on:click=move |_| set_step.set(3)>"Volgende"</button>
                            </div>
                        </div>
                    }.into_any(),
                    3 => view! {
                        <div>
                            <h3 style="margin-bottom: 15px;">"Metadata (AI suggesties)"</h3>
                            <div class="compliance-indicator ok">
                                <div class="icon">"🤖"</div>
                                <div class="label">"Onderwerp: Windenergie"</div>
                                <div class="value">"95%"</div>
                            </div>
                            <div class="compliance-indicator ok">
                                <div class="icon">"🤖"</div>
                                <div class="label">"Tags: subsidie, duurzaam, energie"</div>
                                <div class="value">"88%"</div>
                            </div>
                            <div style="display: flex; justify-content: space-between; margin-top: 15px;">
                                <button class="btn btn-secondary" on:click=move |_| set_step.set(2)>"Vorige"</button>
                                <button class="btn btn-primary" on:click=move |_| set_step.set(4)>"Volgende"</button>
                            </div>
                        </div>
                    }.into_any(),
                    4 => view! {
                        <div>
                            <h3 style="margin-bottom: 15px;">"Compliance check"</h3>
                            <div class="compliance-indicator ok">
                                <div class="icon">"✓"</div>
                                <div class="label">"Woo classificatie"</div>
                                <div class="value">"Openbaar"</div>
                            </div>
                            <div class="compliance-indicator ok">
                                <div class="icon">"✓"</div>
                                <div class="label">"Bewaartermijn"</div>
                                <div class="value">"20 jaar"</div>
                            </div>
                            <div class="compliance-indicator ok">
                                <div class="icon">"✓"</div>
                                <div class="label">"AVG check"</div>
                                <div class="value">"Geen PII"</div>
                            </div>
                            <div style="display: flex; justify-content: space-between; margin-top: 15px;">
                                <button class="btn btn-secondary" on:click=move |_| set_step.set(3)>"Vorige"</button>
                                <button class="btn btn-primary" on:click=move |_| set_step.set(5)>"Volgende"</button>
                            </div>
                        </div>
                    }.into_any(),
                    _ => view! {
                        <div>
                            <h3 style="margin-bottom: 15px;">"Preview & Opslaan"</h3>
                            <div style="background: #f5f7fa; padding: 20px; border-radius: 8px; margin-bottom: 15px;">
                                <p>"Document preview wordt hier getoond..."</p>
                            </div>
                            <div style="display: flex; justify-content: space-between;">
                                <button class="btn btn-secondary" on:click=move |_| set_step.set(4)>"Vorige"</button>
                                <button class="btn btn-primary">"Opslaan"</button>
                            </div>
                        </div>
                    }.into_any(),
                }}
            </Panel>
        </main>
    }
}
