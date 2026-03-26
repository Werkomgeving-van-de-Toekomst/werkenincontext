//! Testpagina: NL Wallet-sessie via IOU API (`POST /api/id/nl-wallet/sessions`).
//!
//! Vereist: `iou-api` met `NL_WALLET_VERIFICATION_SERVER_URL` en draaiende MinBZK verification_server.

use dioxus::prelude::*;

use crate::api::API_BASE;

#[component]
pub fn NlWalletE2e() -> Element {
    let mut usecase = use_signal(|| "iou_modern_demo".to_string());
    let mut result = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);

    rsx! {
        div { class: "nl-wallet-e2e",
            style: "max-width: 42rem; margin: 2rem auto; padding: 0 1rem; font-family: system-ui, sans-serif;",
            h1 { "NL Wallet × IOU (test)" }
            p {
                "Roept "
                code { "{API_BASE}/api/id/nl-wallet/sessions" }
                " aan. Zet op de API "
                code { "NL_WALLET_VERIFICATION_SERVER_URL" }
                " (zie docs/nl-wallet-e2e.md)."
            }
            label {
                "Usecase "
                input {
                    r#type: "text",
                    value: usecase(),
                    oninput: move |e| usecase.set(e.value()),
                    style: "width: 16rem; margin-left: 0.5rem;",
                }
            }
            p {
                button {
                    disabled: busy(),
                    onclick: move |_| {
                        let u = usecase().trim().to_string();
                        let u = if u.is_empty() {
                            "iou_modern_demo".to_string()
                        } else {
                            u
                        };
                        spawn(async move {
                            busy.set(true);
                            result.set(Some(start_session(&u).await));
                            busy.set(false);
                        });
                    },
                    if busy() { "Bezig…" } else { "Start sessie" }
                }
            }
            if let Some(text) = result() {
                pre { style: "background:#f4f4f5; padding:1rem; overflow:auto; white-space: pre-wrap;",
                    "{text}"
                }
            }
            p { style: "margin-top: 2rem; font-size: 0.9rem; color: #52525b;",
                "Statische demo met officiële knop: open "
                code { "http://127.0.0.1:8000/nl-wallet-e2e.html" }
                " in de browser (zelfde host als de API)."
            }
        }
    }
}

async fn start_session(usecase: &str) -> String {
    let url = format!("{API_BASE}/api/id/nl-wallet/sessions");
    let body = serde_json::json!({ "usecase": usecase });
    post_text(&url, &body.to_string()).await
}

#[cfg(target_arch = "wasm32")]
async fn post_text(url: &str, json_body: &str) -> String {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(web_sys::RequestMode::Cors);
    opts.set_body(&wasm_bindgen::JsValue::from_str(json_body));

    let headers = web_sys::Headers::new().expect("headers");
    let _ = headers.set("Content-Type", "application/json");
    let _ = headers.set("Accept", "application/json");
    opts.set_headers(&headers);

    let request = match web_sys::Request::new_with_str_and_init(url, &opts) {
        Ok(r) => r,
        Err(e) => return format!("request error: {e:?}"),
    };

    let window = match web_sys::window() {
        Some(w) => w,
        None => return "no window".to_string(),
    };

    let response_value = match JsFuture::from(window.fetch_with_request(&request)).await {
        Ok(v) => v,
        Err(e) => return format!("fetch failed: {e:?}"),
    };

    let response: web_sys::Response = match response_value.dyn_into() {
        Ok(r) => r,
        Err(_) => return "invalid response".to_string(),
    };

    let status = response.status();
    let ok = response.ok();
    let text_promise = match response.text() {
        Ok(p) => p,
        Err(e) => return format!("text(): {e:?}"),
    };
    let text_val = match JsFuture::from(text_promise).await {
        Ok(t) => t,
        Err(e) => return format!("read body: {e:?}"),
    };
    let body = text_val.as_string().unwrap_or_default();
    let pretty = serde_json::from_str::<serde_json::Value>(&body)
        .map(|v| serde_json::to_string_pretty(&v).unwrap_or(body.clone()))
        .unwrap_or(body);
    format!("HTTP {status} ok={ok}\n\n{pretty}")
}

#[cfg(not(target_arch = "wasm32"))]
async fn post_text(_url: &str, _json_body: &str) -> String {
    "Alleen beschikbaar in WASM-build".to_string()
}
