//! Rijksoverheid API probe module
//!
//! Probes the Rijksoverheid API to document its capabilities and availability.
//! This is a feasibility spike to determine if the API can be used for
//! organization name normalization.

use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Probe result documenting API capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProbeResult {
    pub api_available: bool,
    pub endpoint_url: String,
    pub requires_auth: bool,
    pub rate_limit_documented: bool,
    pub sample_response: Option<serde_json::Value>,
    pub error_message: Option<String>,
}

/// Probe the Rijksoverheid API to document its capabilities
///
/// This function attempts to query the Rijksoverheid Data API
/// to check for organization name lookup capabilities.
///
/// # Returns
///
/// An `ApiProbeResult` containing information about API availability,
/// authentication requirements, and sample response (if successful).
pub async fn probe_rijksoverheid_api() -> ApiProbeResult {
    // Try multiple potential endpoints
    const ENDPOINTS: &[&str] = &[
        "https://api.data.overheid.nl/io/oa/organisaties",
        "https://api.data.overheid.nl/io/oa/organisatie",
        "https://directory.acceptatie.overheid.nl/public/organizations",
    ];

    for endpoint_url in ENDPOINTS {
        let result = try_endpoint(endpoint_url).await;
        if result.api_available || result.requires_auth {
            return result;
        }
    }

    // All endpoints failed
    ApiProbeResult {
        api_available: false,
        endpoint_url: ENDPOINTS[0].to_string(),
        requires_auth: false,
        rate_limit_documented: false,
        sample_response: None,
        error_message: Some("All Rijksoverheid API endpoints unavailable".to_string()),
    }
}

async fn try_endpoint(endpoint_url: &str) -> ApiProbeResult {
    let client = match Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return ApiProbeResult {
                api_available: false,
                endpoint_url: endpoint_url.to_string(),
                requires_auth: false,
                rate_limit_documented: false,
                sample_response: None,
                error_message: Some(format!("Failed to create client: {}", e)),
            };
        }
    };

    // Attempt to query the API with a search term
    let response = client
        .get(endpoint_url)
        .query(&[("zoekterm", "MinFin")])
        .header("Accept", "application/json")
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let requires_auth = status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN;

            // Check for rate limit headers
            let rate_limit_documented = resp.headers().get("X-RateLimit-Limit").is_some()
                || resp.headers().get("RateLimit-Limit").is_some();

            if status.is_success() {
                match resp.json().await {
                    Ok(json) => ApiProbeResult {
                        api_available: true,
                        endpoint_url: endpoint_url.to_string(),
                        requires_auth,
                        rate_limit_documented,
                        sample_response: Some(json),
                        error_message: None,
                    },
                    Err(e) => ApiProbeResult {
                        api_available: true,
                        endpoint_url: endpoint_url.to_string(),
                        requires_auth,
                        rate_limit_documented,
                        sample_response: None,
                        error_message: Some(format!("Failed to parse JSON: {}", e)),
                    }
                }
            } else if status == reqwest::StatusCode::NOT_FOUND {
                ApiProbeResult {
                    api_available: false,
                    endpoint_url: endpoint_url.to_string(),
                    requires_auth: false,
                    rate_limit_documented: false,
                    sample_response: None,
                    error_message: Some(format!("API endpoint not found: {}", status)),
                }
            } else {
                ApiProbeResult {
                    api_available: false,
                    endpoint_url: endpoint_url.to_string(),
                    requires_auth,
                    rate_limit_documented: false,
                    sample_response: None,
                    error_message: Some(format!("API returned status: {}", status)),
                }
            }
        }
        Err(e) => ApiProbeResult {
            api_available: false,
            endpoint_url: endpoint_url.to_string(),
            requires_auth: false,
            rate_limit_documented: false,
            sample_response: None,
            error_message: Some(format!("Request failed: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_probe_does_not_panic() {
        // Basic smoke test - should never panic
        let result = probe_rijksoverheid_api().await;
        // We don't assert API availability (it may be down),
        // just verify the function completes
        assert!(result.endpoint_url.len() > 0);
    }
}
