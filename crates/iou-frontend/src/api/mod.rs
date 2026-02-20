//! API client for communicating with the backend
//!
//! Uses browser's fetch API for WASM builds, reqwest for native.

use serde::de::DeserializeOwned;

use iou_core::api_types::{ContextResponse, SearchResponse};
use iou_core::domain::InformationDomain;

const API_BASE: &str = "http://localhost:8000";

/// Fetch context for a domain
pub async fn fetch_context(domain_id: &str) -> Result<ContextResponse, String> {
    let url = format!("{API_BASE}/context/{domain_id}");
    fetch_json(&url).await
}

/// Fetch all domains
pub async fn fetch_domains() -> Result<Vec<InformationDomain>, String> {
    let url = format!("{API_BASE}/domains");
    fetch_json(&url).await
}

/// Search for objects
pub async fn search(query: &str) -> Result<SearchResponse, String> {
    let encoded = urlencoding::encode(query);
    let url = format!("{API_BASE}/search?q={encoded}");
    fetch_json(&url).await
}

/// Fetch recommended apps
pub async fn fetch_recommended_apps() -> Result<Vec<AppInfo>, String> {
    let url = format!("{API_BASE}/apps/recommended");
    fetch_json(&url).await
}

/// App info from API
#[derive(Clone, Debug, serde::Deserialize)]
pub struct AppInfo {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub app_type: String,
    pub icon_url: Option<String>,
    pub endpoint_url: String,
    pub relevance_score: f32,
    pub reason: String,
}

/// Generic JSON fetch helper using browser fetch API
async fn fetch_json<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    fetch_json_impl(url).await
}

// Platform-specific implementation for WASM
#[cfg(target_arch = "wasm32")]
async fn fetch_json_impl<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(web_sys::RequestMode::Cors);

    let request = web_sys::Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {e:?}"))?;

    let window = web_sys::window().expect("no global window");
    let response_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {e:?}"))?;

    let response: web_sys::Response = response_value
        .dyn_into()
        .map_err(|e| format!("Invalid response type: {e:?}"))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let text_value = JsFuture::from(response.text())
        .await
        .map_err(|e| format!("Failed to get response text: {e:?}"))?;

    let text: String = text_value
        .dyn_into()
        .map_err(|e| format!("Invalid text type: {e:?}"))?;

    serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {e}"))
}

// Platform-specific implementation for native (desktop)
// Note: This is only used when the "desktop" feature is enabled
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
async fn fetch_json_impl<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    // Re-export reqwest for this module
    extern crate reqwest;

    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response
        .json::<T>()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))
}

// Stub for native builds without desktop feature (should not be used)
#[cfg(all(not(target_arch = "wasm32"), not(feature = "desktop")))]
async fn fetch_json_impl<T: DeserializeOwned>(_url: &str) -> Result<T, String> {
    Err("HTTP client not available. Enable the 'desktop' feature.".to_string())
}

// URL encoding helper
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                    result.push(c);
                }
                ' ' => result.push_str("%20"),
                _ => {
                    for byte in c.to_string().bytes() {
                        result.push_str(&format!("%{byte:02X}"));
                    }
                }
            }
        }
        result
    }
}
