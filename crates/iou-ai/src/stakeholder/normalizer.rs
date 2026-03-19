//! Entity normalization for stakeholder extraction
//!
//! This module handles normalization of entity names to canonical forms,
//! particularly for Dutch government organizations using the Rijksoverheid API.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;

use crate::stakeholder::{
    error::NormalizationError,
    rijksoverheid::{RijksoverheidClient, OrgInfo},
    types::{OrganizationStakeholder, PersonStakeholder},
};

/// Cache entry for normalized entities
#[derive(Clone)]
struct CacheEntry {
    canonical_name: String,
    org_info: Option<OrgInfo>,
    cached_at: Instant,
}

/// Statistics about the normalization cache
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
}

/// Helper function to insert into metadata
fn insert_metadata(metadata: &mut serde_json::Value, key: &str, value: serde_json::Value) {
    if let Some(obj) = metadata.as_object_mut() {
        obj.insert(key.to_string(), value);
    }
}

/// Normalizes entity names to canonical forms using the Rijksoverheid API
pub struct EntityNormalizer {
    api_client: Arc<RijksoverheidClient>,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    cache_ttl: Duration,
}

impl EntityNormalizer {
    /// Create a new normalizer with default cache TTL (24 hours)
    pub fn new(api_client: Arc<RijksoverheidClient>) -> Self {
        Self {
            api_client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(86_400), // 24 hours
        }
    }

    /// Create with custom cache TTL
    pub fn with_cache_ttl(api_client: Arc<RijksoverheidClient>, ttl: Duration) -> Self {
        Self {
            api_client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: ttl,
        }
    }

    /// Normalize an organization entity to its canonical name
    pub async fn normalize_organization(
        &self,
        org: OrganizationStakeholder,
    ) -> Result<OrganizationStakeholder, NormalizationError> {
        let name = org.entity.name.as_str();

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(name) {
                if entry.cached_at.elapsed() < self.cache_ttl {
                    let mut result = org;
                    result.entity.name = entry.canonical_name.clone();
                    result.entity.canonical_name = Some(entry.canonical_name.clone());
                    if let Some(info) = &entry.org_info {
                        insert_metadata(
                            &mut result.entity.metadata,
                            "org_type",
                            serde_json::json!(info.org_type.to_string()),
                        );
                    }
                    return Ok(result);
                }
            }
        }

        // Try API lookup
        let canonical_name = match self.api_client.get_canonical_name(name).await {
            Ok(Some(n)) => n,
            Ok(None) => name.to_string(),
            Err(_) => name.to_string(),
        };

        // Get full org info if we found a different canonical name
        let org_info = if canonical_name != name {
            self.api_client.get_org_info(&canonical_name).await.ok().flatten()
        } else {
            None
        };

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                name.to_string(),
                CacheEntry {
                    canonical_name: canonical_name.clone(),
                    org_info: org_info.clone(),
                    cached_at: Instant::now(),
                },
            );
        }

        // Build result
        let mut result = org;
        result.entity.name = canonical_name.clone();
        result.entity.canonical_name = Some(canonical_name);

        if let Some(info) = org_info {
            insert_metadata(
                &mut result.entity.metadata,
                "org_type",
                serde_json::json!(info.org_type.to_string()),
            );
            insert_metadata(
                &mut result.entity.metadata,
                "abbreviations",
                serde_json::json!(info.abbreviations),
            );
            if let Some(parent) = info.parent_org {
                insert_metadata(
                    &mut result.entity.metadata,
                    "parent_org",
                    serde_json::json!(parent),
                );
            }
        }

        Ok(result)
    }

    /// Normalize a person entity (adds normalized name for comparison)
    pub fn normalize_person(&self, person: PersonStakeholder) -> PersonStakeholder {
        let normalized = Self::normalize_dutch_name(&person.entity.name);

        let mut result = person;
        insert_metadata(
            &mut result.entity.metadata,
            "normalized_name",
            serde_json::json!(normalized),
        );
        result
    }

    /// Normalize Dutch names for comparison (lowercase)
    fn normalize_dutch_name(name: &str) -> String {
        name.to_lowercase()
    }

    /// Clear expired cache entries
    pub async fn clear_expired_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.retain(|_, entry| entry.cached_at.elapsed() < self.cache_ttl);
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let total = cache.len();
        let expired = cache
            .values()
            .filter(|e| e.cached_at.elapsed() >= self.cache_ttl)
            .count();

        CacheStats {
            total_entries: total,
            expired_entries: expired,
            active_entries: total - expired,
        }
    }

    /// Batch normalize multiple organizations
    pub async fn normalize_organizations(
        &self,
        orgs: Vec<OrganizationStakeholder>,
    ) -> Result<Vec<OrganizationStakeholder>, NormalizationError> {
        let mut results = Vec::with_capacity(orgs.len());

        for org in orgs {
            results.push(self.normalize_organization(org).await?);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_normalizer_creates_entity() {
        let client = Arc::new(RijksoverheidClient::new());
        let normalizer = EntityNormalizer::new(client);

        let org = OrganizationStakeholder::new("Test Organization".to_string(), 0.9);

        // Should not error on unknown organization
        let result = normalizer.normalize_organization(org).await.unwrap();
        assert_eq!(result.entity.name, "Test Organization");
    }

    #[tokio::test]
    async fn test_normalize_minfin_to_canonical() {
        let client = Arc::new(RijksoverheidClient::new());
        let normalizer = EntityNormalizer::new(client);

        let org = OrganizationStakeholder::new("MinFin".to_string(), 0.9);

        let result = normalizer.normalize_organization(org).await.unwrap();
        assert_eq!(result.entity.name, "Ministerie van Financiën");
        assert_eq!(
            result.entity.canonical_name,
            Some("Ministerie van Financiën".to_string())
        );
    }

    #[tokio::test]
    async fn test_normalize_person_adds_metadata() {
        let client = Arc::new(RijksoverheidClient::new());
        let normalizer = EntityNormalizer::new(client);

        let person = PersonStakeholder::new("Jan de Vries".to_string(), 0.9);

        let result = normalizer.normalize_person(person);
        assert_eq!(
            result
                .entity
                .metadata
                .get("normalized_name")
                .and_then(|v| v.as_str()),
            Some("jan de vries")
        );
    }

    #[tokio::test]
    async fn test_cache_stats_initially_empty() {
        let client = Arc::new(RijksoverheidClient::new());
        let normalizer = EntityNormalizer::new(client);

        let stats = normalizer.cache_stats().await;
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.active_entries, 0);
        assert_eq!(stats.expired_entries, 0);
    }

    #[tokio::test]
    async fn test_cache_stats_after_lookup() {
        let client = Arc::new(RijksoverheidClient::new());
        let normalizer = EntityNormalizer::new(client);

        let org = OrganizationStakeholder::new("MinFin".to_string(), 0.9);
        let _ = normalizer.normalize_organization(org).await.unwrap();

        let stats = normalizer.cache_stats().await;
        assert!(stats.total_entries > 0);
    }

    #[tokio::test]
    async fn test_clear_expired_cache() {
        let client = Arc::new(RijksoverheidClient::new());
        let normalizer = EntityNormalizer::with_cache_ttl(client, Duration::from_millis(100));

        let org = OrganizationStakeholder::new("MinFin".to_string(), 0.9);
        let _ = normalizer.normalize_organization(org).await.unwrap();

        // Wait for cache to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        normalizer.clear_expired_cache().await;

        let stats = normalizer.cache_stats().await;
        assert_eq!(stats.active_entries, 0);
    }

    #[tokio::test]
    async fn test_batch_normalize() {
        let client = Arc::new(RijksoverheidClient::new());
        let normalizer = EntityNormalizer::new(client);

        let orgs = vec![
            OrganizationStakeholder::new("MinFin".to_string(), 0.9),
            OrganizationStakeholder::new("BZK".to_string(), 0.9),
            OrganizationStakeholder::new("Unknown Org".to_string(), 0.5),
        ];

        let results = normalizer.normalize_organizations(orgs).await.unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].entity.name, "Ministerie van Financiën");
        assert!(results[1].entity.name.contains("Binnenlandse Zaken"));
        assert_eq!(results[2].entity.name, "Unknown Org");
    }
}
