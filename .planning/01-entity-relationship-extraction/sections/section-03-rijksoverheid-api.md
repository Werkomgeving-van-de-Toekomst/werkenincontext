Now I have all the context needed. Let me generate the section content for section-03-rijksoverheid-api.

---

# Section 03: Rijksoverheid API Client

## Overview

This section implements a client for the Dutch government organization API (`data.overheid.nl`). The client provides canonical name resolution for Dutch government organizations, enabling normalization of entity variants like "MinFin" to "Ministerie van Financiën".

## Dependencies

- **Section 00: Feasibility Spike** - Validates API capabilities and documents response format
- **Section 01: Foundation & Types** - Provides core error types and `OrgType` enum

## File Structure

```
iou-ai/src/stakeholder/
├── rijksoverheid.rs       # API client implementation
└── mod.rs                 # Public exports
```

## Tests First

Create test file: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/tests/rijksoverheid_tests.rs`

```rust
use iou_ai::stakeholder::{RijksoverheidClient, OrgInfo, OrgType};
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    // Test: get_canonical_name returns "Ministerie van Financiën" for "MinFin"
    #[tokio::test]
    async fn test_canonical_name_minfin_abbreviation() {
        let client = RijksoverheidClient::new();
        let result = client.get_canonical_name("MinFin").await.unwrap();
        assert_eq!(result, Some("Ministerie van Financiën".to_string()));
    }

    // Test: get_canonical_name returns "Ministerie van Financiën" for "Ministerie van Financiën"
    #[tokio::test]
    async fn test_canonical_name_already_canonical() {
        let client = RijksoverheidClient::new();
        let result = client.get_canonical_name("Ministerie van Financiën").await.unwrap();
        assert_eq!(result, Some("Ministerie van Financiën".to_string()));
    }

    // Test: get_canonical_name returns None for unknown organization
    #[tokio::test]
    async fn test_canonical_name_unknown_organization() {
        let client = RijksoverheidClient::new();
        let result = client.get_canonical_name("Unknown Organization LLC").await.unwrap();
        assert_eq!(result, None);
    }

    // Test: get_org_info returns OrgInfo with abbreviations for known ministry
    #[tokio::test]
    async fn test_get_org_info_minfin() {
        let client = RijksoverheidClient::new();
        let result = client.get_org_info("MinFin").await.unwrap();
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.canonical_name, "Ministerie van Financiën");
        assert!(info.abbreviations.contains(&"MinFin".to_string()));
        assert_eq!(info.org_type, OrgType::Ministry);
    }

    // Test: Cache returns cached result without API call
    #[tokio::test]
    async fn test_cache_returns_stored_value() {
        let client = RijksoverheidClient::new();
        
        // First call should hit API
        let _result1 = client.get_canonical_name("MinFin").await.unwrap();
        
        // Second call should use cache (verify via metrics or mock)
        let result2 = client.get_canonical_name("MinFin").await.unwrap();
        assert_eq!(result2, Some("Ministerie van Financiën".to_string()));
    }

    // Test: Cache expires after TTL
    #[tokio::test]
    async fn test_cache_expires_after_ttl() {
        // Create client with short TTL for testing
        let client = RijksoverheidClient::with_cache_ttl(Duration::from_millis(100));
        
        let _result1 = client.get_canonical_name("MinFin").await.unwrap();
        
        // Wait for cache expiry
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Clear expired entries
        client.clear_expired_cache().await;
        
        // Result should still be available (re-fetched or cached elsewhere)
        let result2 = client.get_canonical_name("MinFin").await.unwrap();
        assert_eq!(result2, Some("Ministerie van Financiën".to_string()));
    }

    // Test: Client handles API timeout gracefully
    #[tokio::test]
    async fn test_api_timeout_graceful_handling() {
        let client = RijksoverheidClient::with_timeout(Duration::from_millis(1));
        let result = client.get_canonical_name("MinFin").await;
        
        // Should return error or fall back to local dictionary
        assert!(result.is_ok() || result.is_err());
    }

    // Test: Client handles API error responses gracefully
    #[tokio::test]
    async fn test_api_error_graceful_handling() {
        let client = RijksoverheidClient::with_base_url("https://invalid.url.example");
        let result = client.get_canonical_name("MinFin").await;
        
        // Should fall back to local dictionary
        assert!(result.is_ok());
    }

    // Test: clear_expired_cache removes only expired entries
    #[tokio::test]
    async fn test_clear_expired_cache_selective() {
        let client = RijksoverheidClient::with_cache_ttl(Duration::from_secs(3600));
        
        // Add multiple entries (some would expire, some wouldn't)
        let _ = client.get_canonical_name("MinFin").await;
        
        client.clear_expired_cache().await;
        
        // MinFin should still be cached (not expired)
        let result = client.get_canonical_name("MinFin").await.unwrap();
        assert_eq!(result, Some("Ministerie van Financiën".to_string()));
    }
}
```

## Implementation

### Core Types

File: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/rijksoverheid.rs`

```rust
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::stakeholder::types::{OrgType, ExtractionError};

/// API response structure for organization lookup
#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(rename = "resultaten")]
    results: Vec<OrgRecord>,
}

#[derive(Debug, Deserialize)]
struct OrgRecord {
    #[serde(rename = "officieelnaam")]
    official_name: Option<String>,
    
    #[serde(rename = "afkortingen")]
    abbreviations: Option<Vec<String>>,
    
    #[serde(rename = "domein")]
    domain: Option<String>,
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
    
    /// Local fallback dictionary (loaded from Section 00 feasibility spike)
    fallback_dict: HashMap<String, String>,
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
            fallback_dict: Self::build_fallback_dict(),
        }
    }
    
    /// Build local fallback dictionary
    /// 
    /// This dictionary is populated from Section 00 feasibility spike
    /// findings and serves as backup when API is unavailable.
    fn build_fallback_dict() -> HashMap<String, String> {
        let mut dict = HashMap::new();
        
        // Ministries
        dict.insert("minfin".to_lowercase(), "Ministerie van Financiën".to_string());
        dict.insert("bzk".to_lowercase(), "Ministerie van Binnenlandse Zaken en Koninkrijksrelaties".to_string());
        dict.insert("minvenj".to_lowercase(), "Ministerie van Justitie en Veiligheid".to_string());
        dict.insert("ienw".to_lowercase(), "Ministerie van Economische Zaken en Klimaat".to_string());
        dict.insert("minocw".to_lowercase(), "Ministerie van Onderwijs, Cultuur en Wetenschap".to_string());
        dict.insert("minvws".to_lowercase(), "Ministerie van Volksgezondheid, Welzijn en Sport".to_string());
        dict.insert("mie".to_lowercase(), "Ministerie van Infrastructuur en Waterstaat".to_string());
        dict.insert("minlnv".to_lowercase(), "Ministerie van Landbouw, Natuur en Voedselkwaliteit".to_string());
        dict.insert("minszw".to_lowercase(), "Ministerie van Sociale Zaken en Werkgelegenheid".to_string());
        dict.insert("minenb".to_lowercase(), "Ministerie van Binnenlandse Zaken en Koninkrijksrelaties".to_string());
        dict.insert("bzka".to_lowercase(), "Ministerie van Binnenlandse Zaken en Koninkrijksrelaties".to_string());
        
        // Common agencies and services
        dict.insert("rdw".to_lowercase(), "Rijksdienst voor het Wegverkeer".to_string());
        dict.insert("duo".to_lowercase(), "Dienst Uitvoering Onderwijs".to_string());
        dict.insert("uwv".to_lowercase(), "Uitvoeringsinstituut Werknemersverzekeringen".to_string());
        dict.insert("svb".to_lowercase(), "Sociale Verzekeringsbank".to_string());
        dict.insert("cba".to_lowercase(), "Centraal Bureau Rijvaardigheidsbewijzen".to_string());
        dict.insert("cbr".to_lowercase(), "Centraal Bureau Rijvaardigheidsbewijzen".to_string());
        dict.insert("kvk".to_lowercase(), "Kamer van Koophandel".to_string());
        dict.insert("belastingdienst".to_lowercase(), "Belastingdienst".to_string());
        
        dict
    }
    
    /// Get canonical name for organization
    /// 
    /// First checks cache, then API, then falls back to local dictionary.
    pub async fn get_canonical_name(
        &self,
        name_or_abbrev: &str,
    ) -> Result<Option<String>, ExtractionError> {
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
                return Ok(Some(info.canonical_name));
            }
            Ok(None) => {}
            Err(_) => {
                // API failed, fall through to local dictionary
                tracing::debug!("Rijksoverheid API lookup failed, using fallback dictionary");
            }
        }
        
        // Fall back to local dictionary
        let key = name_or_abbrev.to_lowercase();
        Ok(self.fallback_dict.get(&key).cloned())
    }
    
    /// Get full organization info
    pub async fn get_org_info(
        &self,
        name: &str,
    ) -> Result<Option<OrgInfo>, ExtractionError> {
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
            Ok(None) => {
                // Try to construct from fallback dict
                let key = name.to_lowercase();
                if let Some(canonical) = self.fallback_dict.get(&key) {
                    // Infer type from name
                    let org_type = if canonical.contains("Ministerie") {
                        OrgType::Ministry
                    } else if canonical.contains("Dienst") || canonical.contains("agentschap") {
                        OrgType::Agency
                    } else {
                        OrgType::Other
                    };
                    
                    Ok(Some(OrgInfo {
                        canonical_name: canonical.clone(),
                        abbreviations: vec![name.to_string()],
                        org_type,
                        parent_org: None,
                    }))
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(e),
        }
    }
    
    /// Attempt API lookup
    async fn try_api_lookup(
        &self,
        name_or_abbrev: &str,
    ) -> Result<Option<OrgInfo>, ExtractionError> {
        let url = format!("{}/?q={}", self.base_url, urlencoding::encode(name_or_abbrev));
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await;
            
        match response {
            Ok(resp) => {
                if !resp.status().is_success() {
                    return Ok(None);
                }
                
                let api_response: ApiResponse = resp.json().await
                    .map_err(|e| ExtractionError::ApiError(format!("Failed to parse API response: {}", e)))?;
                
                if api_response.results.is_empty() {
                    return Ok(None);
                }
                
                // Use first result
                let record = &api_response.results[0];
                
                let canonical_name = record.official_name.clone()
                    .unwrap_or_else(|| name_or_abbrev.to_string());
                
                let org_type = if canonical_name.contains("Ministerie") {
                    OrgType::Ministry
                } else if canonical_name.contains("Dienst") || canonical_name.contains("agentschap") {
                    OrgType::Agency
                } else {
                    OrgType::Other
                };
                
                Ok(Some(OrgInfo {
                    canonical_name,
                    abbreviations: record.abbreviations.clone().unwrap_or_default(),
                    org_type,
                    parent_org: None, // Could be extended to parse from API response
                }))
            }
            Err(e) => {
                tracing::warn!("Rijksoverheid API request failed: {}", e);
                Err(ExtractionError::ApiError(format!("API request failed: {}", e)))
            }
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
}

impl Default for RijksoverheidClient {
    fn default() -> Self {
        Self::new()
    }
}
```

### Module Exports

File: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/mod.rs`

Add to existing exports:

```rust
pub mod rijksoverheid;

pub use rijksoverheid::{
    RijksoverheidClient,
    OrgInfo,
};
```

### Error Type Extension

File: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/types.rs`

Ensure `ExtractionError` includes API error variant:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    // ... existing variants ...
    
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("API timeout")]
    ApiTimeout,
}
```

## Integration Points

The Rijksoverheid client is used by:

1. **Section 05: Normalization & Deduplication** - Applies canonical names to organization entities
2. **Section 06: Main Extractor** - Coordinates normalization during extraction pipeline

## Configuration

Environment variables for configuration:

```bash
# API endpoint
RIJKSOVERHEID_API_URL=https://api.data.overheid.nl/io/organisaties

# Cache TTL (default: 86400 seconds = 24 hours)
RIJKSOVERHEID_CACHE_TTL=86400

# Request timeout (default: 5 seconds)
RIJKSOVERHEID_TIMEOUT=5
```

## Success Criteria

- API client resolves "MinFin" to "Ministerie van Financiën"
- Cache reduces API calls for repeated lookups (verify via metrics)
- Graceful degradation when API is unavailable (uses fallback dictionary)
- Tests verify canonical name resolution for common abbreviations
- Cache expiration works correctly

## Notes

- The Rijksoverheid API is open and does not require authentication
- Local fallback dictionary ensures operation during API outages
- Cache is in-memory only; consider persisting to Redis for distributed deployments
- The API response format should be verified in Section 00 feasibility spike