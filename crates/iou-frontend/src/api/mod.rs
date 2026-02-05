//! API client for communicating with the backend

use gloo_net::http::Request;
use serde::de::DeserializeOwned;

use iou_core::api_types::{ContextResponse, SearchResponse};
use iou_core::domain::InformationDomain;

const API_BASE: &str = "http://localhost:8000";

/// Fetch context for a domain
pub async fn fetch_context(domain_id: &str) -> Result<ContextResponse, String> {
    fetch_json(&format!("{}/context/{}", API_BASE, domain_id)).await
}

/// Fetch all domains
pub async fn fetch_domains() -> Result<Vec<InformationDomain>, String> {
    fetch_json(&format!("{}/domains", API_BASE)).await
}

/// Search for objects
pub async fn search(query: &str) -> Result<SearchResponse, String> {
    fetch_json(&format!("{}/search?q={}", API_BASE, urlencoding::encode(query))).await
}

/// Fetch recommended apps
pub async fn fetch_recommended_apps() -> Result<Vec<AppInfo>, String> {
    fetch_json(&format!("{}/apps/recommended", API_BASE)).await
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

/// Generic JSON fetch helper
async fn fetch_json<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    let response = Request::get(url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))
}

// URL encoding helper (since we can't use percent-encoding crate easily in WASM)
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
                        result.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
        }
        result
    }
}
