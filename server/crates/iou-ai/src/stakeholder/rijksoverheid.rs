//! Rijksoverheid API client for Dutch government organization lookup
//!
//! This module provides a client for the Dutch government organization API
//! (data.overheid.nl). It enables canonical name resolution for Dutch
//! government organizations, normalizing variants like "MinFin" to
//! "Ministerie van Financiën".

use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::stakeholder::types::OrgType;
use crate::stakeholder::error::NormalizationError;

/// API response structure for organization lookup
#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(rename = "resultaten")]
    results: Vec<OrgRecord>,

    #[serde(default)]
    #[serde(rename = "aantal")]
    count: usize,
}

#[derive(Debug, Deserialize)]
struct OrgRecord {
    #[serde(rename = "officieelnaam")]
    official_name: Option<String>,

    #[serde(rename = "afkortingen")]
    abbreviations: Option<Vec<String>>,

    #[serde(rename = "domein")]
    domain: Option<String>,

    #[serde(rename = "type")]
    org_type_field: Option<String>,

    #[serde(rename = "hoofdorganisatie")]
    parent_org: Option<String>,
}

/// Organization information from Rijksoverheid API
#[derive(Debug, Clone, Serialize)]
pub struct OrgInfo {
    /// Canonical official name
    pub canonical_name: String,

    /// Known abbreviations
    pub abbreviations: Vec<String>,

    /// Organization type
    pub org_type: OrgType,

    /// Parent organization (if applicable)
    pub parent_org: Option<String>,

    /// Domain/website (if available)
    pub domain: Option<String>,
}

/// Cache entry with timestamp
#[derive(Clone)]
struct CacheEntry {
    info: OrgInfo,
    expires_at: Instant,
}

/// Client for Dutch government organization API
pub struct RijksoverheidClient {
    base_url: String,
    client: Client,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    cache_ttl: Duration,
    request_timeout: Duration,
}

impl RijksoverheidClient {
    /// Create new client with default settings
    pub fn new() -> Self {
        Self::with_config(
            "https://api.data.overheid.nl/io/organisaties".to_string(),
            Duration::from_secs(86400), // 24 hour TTL
            Duration::from_secs(5),      // 5 second timeout
        )
    }

    /// Create client with custom base URL
    pub fn with_base_url(url: &str) -> Self {
        Self::with_config(
            url.to_string(),
            Duration::from_secs(86400),
            Duration::from_secs(5),
        )
    }

    /// Create client with custom cache TTL
    pub fn with_cache_ttl(ttl: Duration) -> Self {
        Self::with_config(
            "https://api.data.overheid.nl/io/organisaties".to_string(),
            ttl,
            Duration::from_secs(5),
        )
    }

    /// Create client with custom timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self::with_config(
            "https://api.data.overheid.nl/io/organisaties".to_string(),
            Duration::from_secs(86400),
            timeout,
        )
    }

    /// Create client with full configuration
    fn with_config(base_url: String, cache_ttl: Duration, request_timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(request_timeout)
            .build()
            .unwrap_or_default();

        Self {
            base_url,
            client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
            request_timeout,
        }
    }

    /// Get canonical name for organization
    ///
    /// First checks cache, then API, then falls back to local dictionary.
    pub async fn get_canonical_name(
        &self,
        name_or_abbrev: &str,
    ) -> Result<Option<String>, NormalizationError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(name_or_abbrev) {
                if entry.expires_at > Instant::now() {
                    return Ok(Some(entry.info.canonical_name.clone()));
                }
            }
        }

        // Try API lookup
        match self.try_api_lookup(name_or_abbrev).await {
            Ok(Some(info)) => {
                self.cache_entry(name_or_abbrev, info.clone()).await;
                Ok(Some(info.canonical_name))
            }
            Ok(None) | Err(_) => {
                // Fall back to local dictionary (both for not found and API errors)
                let key = name_or_abbrev.trim().to_lowercase();
                if let Some(canonical) = crate::stakeholder::fallback_dict::get_fallback_canonical_name(&key) {
                    // Infer type from name for caching
                    let org_type = if canonical.contains("Ministerie") {
                        OrgType::Ministry
                    } else if canonical.contains("Dienst") || canonical.contains("agentschap") {
                        OrgType::Agency
                    } else if canonical.contains("Gemeente") {
                        OrgType::Municipal
                    } else {
                        OrgType::Other
                    };

                    let info = OrgInfo {
                        canonical_name: canonical.to_string(),
                        abbreviations: vec![name_or_abbrev.to_string()],
                        org_type,
                        parent_org: None,
                        domain: None,
                    };

                    // Cache the fallback result
                    self.cache_entry(name_or_abbrev, info.clone()).await;

                    Ok(Some(info.canonical_name))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Get full organization info
    pub async fn get_org_info(
        &self,
        name: &str,
    ) -> Result<Option<OrgInfo>, NormalizationError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(name) {
                if entry.expires_at > Instant::now() {
                    return Ok(Some(entry.info.clone()));
                }
            }
        }

        // Try API
        match self.try_api_lookup(name).await {
            Ok(Some(info)) => {
                self.cache_entry(name, info.clone()).await;
                Ok(Some(info))
            }
            Ok(None) | Err(_) => {
                // Try to construct from fallback dict (both for not found and API errors)
                let key = name.trim().to_lowercase();
                if let Some(canonical) = crate::stakeholder::fallback_dict::get_fallback_canonical_name(&key) {
                    // Infer type from name
                    let org_type = if canonical.contains("Ministerie") {
                        OrgType::Ministry
                    } else if canonical.contains("Dienst") || canonical.contains("agentschap") {
                        OrgType::Agency
                    } else if canonical.contains("Gemeente") {
                        OrgType::Municipal
                    } else {
                        OrgType::Other
                    };

                    let info = OrgInfo {
                        canonical_name: canonical.to_string(),
                        abbreviations: vec![name.to_string()],
                        org_type,
                        parent_org: None,
                        domain: None,
                    };

                    // Cache the fallback result
                    self.cache_entry(name, info.clone()).await;

                    Ok(Some(info))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Attempt API lookup
    async fn try_api_lookup(
        &self,
        name_or_abbrev: &str,
    ) -> Result<Option<OrgInfo>, NormalizationError> {
        let url = format!(
            "{}/?q={}&rows=1",
            self.base_url,
            urlencoding::encode(name_or_abbrev)
        );

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await;

        match response {
            Ok(resp) => {
                if !resp.status().is_success() {
                    if resp.status().is_client_error() || resp.status().is_server_error() {
                        // API returned error, return None (not found)
                        return Ok(None);
                    }
                    return Err(NormalizationError::ApiRequestFailed(
                        format!("Unexpected status: {}", resp.status())
                    ));
                }

                let api_response: ApiResponse = resp.json().await
                    .map_err(|e| NormalizationError::InvalidResponse(format!("Failed to parse API response: {}", e)))?;

                if api_response.results.is_empty() {
                    return Ok(None);
                }

                // Use first result
                let record = &api_response.results[0];

                let canonical_name = record.official_name.clone()
                    .unwrap_or_else(|| name_or_abbrev.to_string());

                let org_type = Self::infer_org_type(&canonical_name, record.org_type_field.as_deref());

                Ok(Some(OrgInfo {
                    canonical_name,
                    abbreviations: record.abbreviations.clone().unwrap_or_default(),
                    org_type,
                    parent_org: record.parent_org.clone(),
                    domain: record.domain.clone(),
                }))
            }
            Err(e) => {
                if e.is_timeout() {
                    tracing::warn!("Rijksoverheid API timeout for '{}': {}", name_or_abbrev, e);
                    Err(NormalizationError::ApiTimeout(self.request_timeout.as_secs()))
                } else if e.is_connect() {
                    // Connection error - likely network issue or invalid URL
                    tracing::debug!("Rijksoverheid API connection failed for '{}': {}", name_or_abbrev, e);
                    Err(NormalizationError::NetworkError(e.to_string()))
                } else {
                    tracing::warn!("Rijksoverheid API request failed for '{}': {}", name_or_abbrev, e);
                    Err(NormalizationError::ApiRequestFailed(e.to_string()))
                }
            }
        }
    }

    /// Infer organization type from canonical name and optional API type field
    fn infer_org_type(canonical_name: &str, api_type: Option<&str>) -> OrgType {
        // First check the API type field if present
        if let Some(t) = api_type {
            match t.to_lowercase().as_str() {
                "ministerie" => return OrgType::Ministry,
                "gemeente" => return OrgType::Municipal,
                "dienst" | "agentschap" => return OrgType::Agency,
                _ => {}
            }
        }

        // Fall back to pattern matching on the name
        if canonical_name.contains("Ministerie") {
            OrgType::Ministry
        } else if canonical_name.contains("Gemeente") {
            OrgType::Municipal
        } else if canonical_name.contains("Dienst") || canonical_name.contains("agentschap") {
            OrgType::Agency
        } else {
            OrgType::Other
        }
    }

    /// Cache an organization info entry
    async fn cache_entry(&self, key: &str, info: OrgInfo) {
        let mut cache = self.cache.write().await;
        cache.insert(
            key.to_string(),
            CacheEntry {
                info,
                expires_at: Instant::now() + self.cache_ttl,
            }
        );
    }

    /// Clear expired cache entries
    pub async fn clear_expired_cache(&self) {
        let mut cache = self.cache.write().await;
        let now = Instant::now();
        cache.retain(|_, entry| entry.expires_at > now);
    }

    /// Get cache statistics (for monitoring)
    pub async fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read().await;
        let now = Instant::now();
        let total = cache.len();
        let expired = cache.values().filter(|e| e.expires_at <= now).count();
        (total, expired)
    }

    /// Prefetch multiple organizations into cache
    ///
    /// Useful for warming up the cache with known organizations.
    pub async fn prefetch(&self, names: &[&str]) -> Result<(), NormalizationError> {
        for name in names {
            // This will cache the result if found
            let _ = self.get_canonical_name(name).await;
        }
        Ok(())
    }
}

impl Default for RijksoverheidClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_document(content: &str) -> crate::agents::content::GeneratedDocument {
        crate::agents::content::GeneratedDocument {
            document_id: uuid::Uuid::new_v4(),
            content: content.to_string(),
            variables: vec![],
            entity_links: vec![],
            sections: vec![],
            generated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_canonical_name_minfin_abbreviation() {
        let client = RijksoverheidClient::new();
        let result = client.get_canonical_name("MinFin").await.unwrap();
        assert_eq!(result, Some("Ministerie van Financiën".to_string()));
    }

    #[tokio::test]
    async fn test_canonical_name_already_canonical() {
        let client = RijksoverheidClient::new();
        let result = client.get_canonical_name("Ministerie van Financiën").await.unwrap();
        assert_eq!(result, Some("Ministerie van Financiën".to_string()));
    }

    #[tokio::test]
    async fn test_canonical_name_unknown_organization() {
        let client = RijksoverheidClient::new();
        let result = client.get_canonical_name("Unknown Organization LLC").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_canonical_name_case_insensitive() {
        let client = RijksoverheidClient::new();
        let result = client.get_canonical_name("MINFIN").await.unwrap();
        assert_eq!(result, Some("Ministerie van Financiën".to_string()));
    }

    #[tokio::test]
    async fn test_canonical_name_lowercase() {
        let client = RijksoverheidClient::new();
        let result = client.get_canonical_name("minfin").await.unwrap();
        assert_eq!(result, Some("Ministerie van Financiën".to_string()));
    }

    #[tokio::test]
    async fn test_get_org_info_minfin() {
        let client = RijksoverheidClient::new();
        let result = client.get_org_info("MinFin").await.unwrap();
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.canonical_name, "Ministerie van Financiën");
        assert!(info.abbreviations.contains(&"MinFin".to_string()) || info.abbreviations.is_empty());
        assert_eq!(info.org_type, OrgType::Ministry);
    }

    #[tokio::test]
    async fn test_get_org_info_bzk() {
        let client = RijksoverheidClient::new();
        let result = client.get_org_info("BZK").await.unwrap();
        assert!(result.is_some());
        let info = result.unwrap();
        assert!(info.canonical_name.contains("Binnenlandse Zaken"));
        assert_eq!(info.org_type, OrgType::Ministry);
    }

    #[tokio::test]
    async fn test_cache_returns_stored_value() {
        let client = RijksoverheidClient::new();

        // First call should hit fallback dict
        let result1 = client.get_canonical_name("MinFin").await.unwrap();
        assert_eq!(result1, Some("Ministerie van Financiën".to_string()));

        // Second call should use cache
        let result2 = client.get_canonical_name("MinFin").await.unwrap();
        assert_eq!(result2, Some("Ministerie van Financiën".to_string()));
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let client = RijksoverheidClient::new();

        // Add some entries to cache
        let _ = client.get_canonical_name("MinFin").await.unwrap();
        let _ = client.get_canonical_name("BZK").await.unwrap();

        let (total, expired) = client.cache_stats().await;
        assert!(total >= 2);
        assert_eq!(expired, 0); // All entries should be fresh
    }

    #[tokio::test]
    async fn test_clear_expired_cache_selective() {
        let client = RijksoverheidClient::with_cache_ttl(Duration::from_millis(100));

        // Add entry
        let _ = client.get_canonical_name("MinFin").await.unwrap();

        // Wait for cache expiry
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Clear expired entries
        client.clear_expired_cache().await;

        // MinFin should still be available from fallback dict
        let result = client.get_canonical_name("MinFin").await.unwrap();
        assert_eq!(result, Some("Ministerie van Financiën".to_string()));
    }

    #[tokio::test]
    async fn test_api_timeout_graceful_handling() {
        let client = RijksoverheidClient::with_timeout(Duration::from_millis(1));
        let result = client.get_canonical_name("MinFin").await;

        // Should fall back to local dictionary and still return Ok
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, Some("Ministerie van Financiën".to_string()));
    }

    #[tokio::test]
    async fn test_invalid_base_url_falls_back_to_dict() {
        let client = RijksoverheidClient::with_base_url("https://invalid.url.example/nonexistent");
        let result = client.get_canonical_name("MinFin").await;

        // Should fall back to local dictionary
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, Some("Ministerie van Financiën".to_string()));
    }

    #[tokio::test]
    async fn test_org_type_inference_ministry() {
        let client = RijksoverheidClient::new();
        let result = client.get_org_info("MinFin").await.unwrap();
        let info = result.unwrap();
        assert_eq!(info.org_type, OrgType::Ministry);
    }

    #[tokio::test]
    async fn test_org_type_inference_municipal() {
        let client = RijksoverheidClient::new();
        let result = client.get_org_info("Amsterdam").await.unwrap();
        let info = result.unwrap();
        assert_eq!(info.org_type, OrgType::Municipal);
        assert!(info.canonical_name.contains("Amsterdam"));
    }

    #[tokio::test]
    async fn test_multiple_abbreviations_cached() {
        let client = RijksoverheidClient::new();

        // Lookup multiple organizations
        let minfin = client.get_canonical_name("MinFin").await.unwrap();
        let bzk = client.get_canonical_name("BZK").await.unwrap();
        let vws = client.get_canonical_name("VWS").await.unwrap();

        assert_eq!(minfin, Some("Ministerie van Financiën".to_string()));
        assert!(bzk.unwrap().contains("Binnenlandse Zaken"));
        assert!(vws.unwrap().contains("Volksgezondheid"));
    }

    #[tokio::test]
    async fn test_trimming_whitespace() {
        let client = RijksoverheidClient::new();
        let result = client.get_canonical_name("  MinFin  ").await.unwrap();
        assert_eq!(result, Some("Ministerie van Financiën".to_string()));
    }

    #[tokio::test]
    async fn test_empty_string_returns_none() {
        let client = RijksoverheidClient::new();
        let result = client.get_canonical_name("").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_prefetch_multiple_names() {
        let client = RijksoverheidClient::new();
        let names = vec!["MinFin", "BZK", "VWS", "EZK"];

        let result = client.prefetch(&names).await;
        assert!(result.is_ok());

        // Verify they're now cached
        let (total, _) = client.cache_stats().await;
        assert!(total >= 4);
    }
}
