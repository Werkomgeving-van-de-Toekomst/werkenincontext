Now I have all the context needed to generate the section content for `section-05-normalization-deduplication`. Let me create a comprehensive, self-contained implementation section.

# Section 05: Normalization & Deduplication

## Overview

This section implements entity normalization and deduplication logic for the stakeholder extraction pipeline. Normalization ensures organization entities use canonical names from the Rijksoverheid API, while deduplication identifies and merges duplicate entities using string similarity and clustering algorithms.

**Dependencies:**
- Section 01 (Foundation & Types) - Core Entity types and error handling
- Section 03 (Rijksoverheid API) - Canonical name resolution

**Blocks:** Section 06 (Main Extractor Implementation)

## File Structure

```
iou-ai/src/stakeholder/
├── normalizer.rs       # Entity normalization with Rijksoverheid API
├── deduplicator.rs     # Duplicate detection and merging
└── mod.rs              # Public exports (updated)
```

## Tests First

### Unit Tests for Jaro-Winkler Similarity

Location: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/deduplicator.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaro_winkler_identical_strings() {
        let result = jaro_winkler("Ministerie van Financiën", "Ministerie van Financiën");
        assert!((result - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_jaro_winkler_similar_strings() {
        // Common variations in Dutch documents
        let result = jaro_winkler("MinFin", "Ministerie van Financiën");
        assert!(result > 0.9, "Expected >0.9 for abbreviations, got {}", result);
    }

    #[test]
    fn test_jaro_winkler_dissimilar_strings() {
        let result = jaro_winkler("Ministerie van Financiën", "Ministerie van Defensie");
        assert!(result < 0.5, "Expected <0.5 for different ministries, got {}", result);
    }

    #[test]
    fn test_jaro_winkler_symmetry() {
        let a = "Jan de Vries";
        let b = "J. de Vries";
        assert!((jaro_winkler(a, b) - jaro_winkler(b, a)).abs() < f32::EPSILON);
    }
}
```

### Unit Tests for Normalization

Location: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/normalizer.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::stakeholder::OrganizationStakeholder;

    #[tokio::test]
    async fn test_normalization_applies_canonical_name() {
        // Mock the API client
        let mut mock_client = MockRijksoverheidClient::new();
        mock_client.expect_get_canonical_name()
            .with(mock::eq("MinFin"))
            .returning(|_| Ok(Some("Ministerie van Financiën".to_string())));

        let normalizer = EntityNormalizer::new_with_client(mock_client);
        let entity = OrganizationStakeholder::new("MinFin".to_string(), 0.9);
        
        let normalized = normalizer.normalize_organization(entity).await.unwrap();
        assert_eq!(normalized.entity.name, "Ministerie van Financiën");
    }

    #[tokio::test]
    async fn test_normalization_uses_fallback() {
        let mut mock_client = MockRijksoverheidClient::new();
        mock_client.expect_get_canonical_name()
            .returning(|_| Err(ApiError::Unavailable));
            
        let normalizer = EntityNormalizer::new_with_client(mock_client);
        let entity = OrganizationStakeholder::new("MinFin".to_string(), 0.9);
        
        let normalized = normalizer.normalize_organization(entity).await.unwrap();
        // Should use local fallback dictionary
        assert!(normalized.entity.name.contains("Financiën"));
    }
}
```

### Unit Tests for Deduplication

```rust
#[cfg(test)]
mod dedup_tests {
    use super::*;

    #[tokio::test]
    async fn test_connected_components_groups_duplicates() {
        let entities = vec![
            PersonStakeholder::new("Jan de Vries".to_string(), 0.9),
            PersonStakeholder::new("J. de Vries".to_string(), 0.8),
            PersonStakeholder::new("Piet Jansen".to_string(), 0.9),
        ];
        
        let deduplicator = EntityDeduplicator::new();
        let deduplicated = deduplicator.deduplicate_persons(entities).await.unwrap();
        
        // Jan and J. should be merged
        assert_eq!(deduplicated.len(), 2);
    }

    #[tokio::test]
    async fn test_merge_creates_new_uuid() {
        let entity1 = PersonStakeholder::new("Jan de Vries".to_string(), 0.9);
        let entity2 = PersonStakeholder::new("J. de Vries".to_string(), 0.8);
        
        let old_id1 = entity1.entity.id;
        let old_id2 = entity2.entity.id;
        
        let merged = EntityDeduplicator::merge_persons(vec![entity1, entity2]).await;
        
        assert_ne!(merged.entity.id, old_id1);
        assert_ne!(merged.entity.id, old_id2);
        
        // Check aliases stored in metadata
        let aliases = merged.entity.metadata.get("aliases")
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(aliases.len(), 2);
    }
}
```

## Implementation

### 1. Entity Normalizer

Location: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/normalizer.rs`

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::stakeholder::{
    Entity, EntityType, ExtractionError,
    PersonStakeholder, OrganizationStakeholder,
};
use crate::stakeholder::rijksoverheid::{RijksoverheidClient, OrgInfo, ApiError};

/// Cache entry for normalized entities
struct CacheEntry {
    canonical_name: String,
    org_info: Option<OrgInfo>,
    cached_at: Instant,
}

/// Normalizes entity names to canonical forms
pub struct EntityNormalizer {
    api_client: Arc<RijksoverheidClient>,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    cache_ttl: Duration,
    /// Local fallback for known government organizations
    fallback_dict: HashMap<String, String>,
}

impl EntityNormalizer {
    /// Create a new normalizer with default cache TTL (24 hours)
    pub fn new(api_client: Arc<RijksoverheidClient>) -> Self {
        Self {
            api_client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(86_400), // 24 hours
            fallback_dict: Self::build_fallback_dict(),
        }
    }

    /// Create with custom cache TTL
    pub fn with_cache_ttl(api_client: Arc<RijksoverheidClient>, ttl: Duration) -> Self {
        Self {
            api_client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: ttl,
            fallback_dict: Self::build_fallback_dict(),
        }
    }

    /// Build local fallback dictionary for known organizations
    fn build_fallback_dict() -> HashMap<String, String> {
        let mut dict = HashMap::new();
        
        // Ministries and common abbreviations
        dict.insert("MinFin".to_lowercase(), "Ministerie van Financiën".to_string());
        dict.insert("BZK".to_lowercase(), "Ministerie van Binnenlandse Zaken en Koninkrijksrelaties".to_string());
        dict.insert("JenV".to_lowercase(), "Ministerie van Justitie en Veiligheid".to_string());
        dict.insert("VWS".to_lowercase(), "Ministerie van Volksgezondheid, Welzijn en Sport".to_string());
        dict.insert("OCW".to_lowercase(), "Ministerie van Onderwijs, Cultuur en Wetenschap".to_string());
        dict.insert("EZK".to_lowercase(), "Ministerie van Economische Zaken en Klimaat".to_string());
        dict.insert("LNV".to_lowercase(), "Ministerie van Landbouw, Natuur en Voedselkwaliteit".to_string());
        dict.insert("IenW".to_lowercase(), "Ministerie van Infrastructuur en Waterstaat".to_string());
        dict.insert("SZW".to_lowercase(), "Ministerie van Sociale Zaken en Werkgelegenheid".to_string());
        dict.insert("BZ".to_lowercase(), "Ministerie van Buitenlandse Zaken".to_string());
        dict.insert("Defensie".to_lowercase(), "Ministerie van Defensie".to_string());
        
        dict
    }

    /// Normalize an organization entity to its canonical name
    pub async fn normalize_organization(
        &self,
        org: OrganizationStakeholder,
    ) -> Result<OrganizationStakeholder, ExtractionError> {
        let name = org.entity.name.as_str();
        
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(name) {
                if entry.cached_at.elapsed() < self.cache_ttl {
                    let mut result = org;
                    result.entity.name = entry.canonical_name.clone();
                    if let Some(info) = &entry.org_info {
                        result.entity.metadata.insert("org_type".to_string(), 
                            serde_json::json!(info.org_type));
                    }
                    return Ok(result);
                }
            }
        }

        // Try API lookup
        let canonical_name = match self.api_client.get_canonical_name(name).await {
            Ok(Some(name)) => name,
            Ok(None) => {
                // No canonical found, try fallback
                self.try_fallback(name).unwrap_or_else(|| name.to_string())
            }
            Err(ApiError::Unavailable) => {
                // API down, use fallback
                self.try_fallback(name).unwrap_or_else(|| name.to_string())
            }
            Err(e) => return Err(ExtractionError::NormalizationFailed(e.to_string())),
        };

        // Get full org info if we found a canonical name
        let org_info = if canonical_name != name {
            self.api_client.get_org_info(&canonical_name).await.ok().flatten()
        } else {
            None
        };

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(name.to_string(), CacheEntry {
                canonical_name: canonical_name.clone(),
                org_info: org_info.clone(),
                cached_at: Instant::now(),
            });
        }

        // Build result
        let mut result = org;
        result.entity.name = canonical_name;
        
        if let Some(info) = org_info {
            result.entity.metadata.insert("org_type".to_string(), 
                serde_json::json!(info.org_type));
            result.entity.metadata.insert("abbreviations".to_string(), 
                serde_json::json!(info.abbreviations));
            if let Some(parent) = info.parent_org {
                result.entity.metadata.insert("parent_org".to_string(), 
                    serde_json::json!(parent));
            }
        }

        Ok(result)
    }

    /// Normalize a person entity (applies Dutch name rules)
    pub fn normalize_person(&self, person: PersonStakeholder) -> PersonStakeholder {
        // Apply Dutch name normalization for comparison
        // This doesn't change the stored name, just adds normalized form
        let normalized = Self::normalize_dutch_name(&person.entity.name);
        
        let mut result = person;
        result.entity.metadata.insert("normalized_name".to_string(), 
            serde_json::json!(normalized));
        result
    }

    /// Normalize Dutch names for comparison (lowercase prefixes, etc.)
    fn normalize_dutch_name(name: &str) -> String {
        let prefixes = ["van", "van der", "de", "ten", "ter", "in", "den", "der", "bij", "op"];
        
        let mut parts: Vec<&str> = name.split_whitespace().collect();
        if parts.is_empty() {
            return name.to_lowercase();
        }

        // Process prefix handling
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < parts.len() {
            // Check multi-word prefixes first
            if i + 1 < parts.len() {
                let two_word = format!("{} {}", parts[i].to_lowercase(), parts[i+1].to_lowercase());
                if prefixes.contains(&two_word.as_str()) {
                    result.push(two_word);
                    i += 2;
                    continue;
                }
            }
            
            // Check single-word prefixes
            let lower = parts[i].to_lowercase();
            if prefixes.contains(&lower.as_str()) && i < parts.len() - 1 {
                result.push(lower);
            } else {
                // Not a prefix, capitalize original
                result.push(parts[i].to_string());
            }
            i += 1;
        }
        
        result.join(" ").to_lowercase()
    }

    /// Try local fallback dictionary
    fn try_fallback(&self, name: &str) -> Option<String> {
        let key = name.to_lowercase();
        self.fallback_dict.get(&key).cloned()
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
        let expired = cache.values()
            .filter(|e| e.cached_at.elapsed() >= self.cache_ttl)
            .count();
        
        CacheStats {
            total_entries: total,
            expired_entries: expired,
            active_entries: total - expired,
        }
    }
}

pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
}
```

### 2. Entity Deduplicator

Location: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/deduplicator.rs`

```rust
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::stakeholder::{
    Entity, EntityType, ExtractionError,
    PersonStakeholder, OrganizationStakeholder,
    MentionRelationship,
};

/// Similarity threshold for considering entities as duplicates
const SIMILARITY_THRESHOLD: f32 = 0.85;

/// Deduplicates entities using string similarity and clustering
pub struct EntityDeduplicator {
    similarity_threshold: f32,
}

impl EntityDeduplicator {
    pub fn new() -> Self {
        Self {
            similarity_threshold: SIMILARITY_THRESHOLD,
        }
    }

    /// Create with custom similarity threshold
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            similarity_threshold: threshold,
        }
    }

    /// Deduplicate person entities
    pub async fn deduplicate_persons(
        &self,
        persons: Vec<PersonStakeholder>,
    ) -> Result<Vec<PersonStakeholder>, ExtractionError> {
        if persons.is_empty() {
            return Ok(Vec::new());
        }

        let normalized: Vec<_> = persons.into_iter()
            .map(|p| self.normalize_for_comparison(&p))
            .collect();

        let clusters = self.find_clusters(&normalized)?;
        
        let mut results = Vec::new();
        for cluster in clusters {
            if cluster.len() == 1 {
                results.push(persons[cluster[0]].clone());
            } else {
                let merged = self.merge_person_cluster(&cluster, &persons).await?;
                results.push(merged);
            }
        }

        Ok(results)
    }

    /// Deduplicate organization entities
    pub async fn deduplicate_organizations(
        &self,
        orgs: Vec<OrganizationStakeholder>,
    ) -> Result<Vec<OrganizationStakeholder>, ExtractionError> {
        if orgs.is_empty() {
            return Ok(Vec::new());
        }

        let normalized: Vec<_> = orgs.iter()
            .map(|o| o.entity.name.to_lowercase())
            .collect();

        let clusters = self.find_clusters_by_names(&normalized, &orgs)?;
        
        let mut results = Vec::new();
        for cluster in clusters {
            if cluster.len() == 1 {
                results.push(orgs[cluster[0]].clone());
            } else {
                let merged = self.merge_org_cluster(&cluster, &orgs).await?;
                results.push(merged);
            }
        }

        Ok(results)
    }

    /// Update relationships after deduplication
    pub fn update_relationships(
        &self,
        relationships: Vec<MentionRelationship>,
        merged_entities: &HashMap<Uuid, Uuid>,
    ) -> Vec<MentionRelationship> {
        relationships.into_iter()
            .map(|mut rel| {
                if let Some(&canonical_id) = merged_entities.get(&rel.entity_id) {
                    rel.entity_id = canonical_id;
                }
                rel
            })
            .collect()
    }

    /// Find clusters of duplicate entities using connected components
    fn find_clusters(&self, normalized: &[String]) -> Result<Vec<Vec<usize>>, ExtractionError> {
        let n = normalized.len();
        let mut parent: Vec<usize> = (0..n).collect();
        
        // Union-Find for connected components
        fn find(parent: &mut [usize], i: usize) -> usize {
            if parent[i] != i {
                parent[i] = find(parent, parent[i]);
            }
            parent[i]
        }

        fn union(parent: &mut [usize], i: usize, j: usize) {
            let pi = find(parent, i);
            let pj = find(parent, j);
            if pi != pj {
                parent[pi] = pj;
            }
        }

        for i in 0..n {
            for j in (i+1)..n {
                if jaro_winkler(&normalized[i], &normalized[j]) >= self.similarity_threshold {
                    union(&mut parent, i, j);
                }
            }
        }

        // Group by parent
        let mut clusters: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..n {
            let p = find(&mut parent.clone(), i);
            clusters.entry(p).or_default().push(i);
        }

        Ok(clusters.into_values().collect())
    }

    /// Find clusters by organization names with special handling
    fn find_clusters_by_names(
        &self,
        normalized: &[String],
        orgs: &[OrganizationStakeholder],
    ) -> Result<Vec<Vec<usize>>, ExtractionError> {
        let n = normalized.len();
        let mut parent: Vec<usize> = (0..n).collect();
        
        fn find(parent: &mut [usize], i: usize) -> usize {
            if parent[i] != i {
                parent[i] = find(parent, parent[i]);
            }
            parent[i]
        }

        fn union(parent: &mut [usize], i: usize, j: usize) {
            let pi = find(parent, i);
            let pj = find(parent, j);
            if pi != pj {
                parent[pi] = pj;
            }
        }

        for i in 0..n {
            for j in (i+1)..n {
                // Check direct similarity
                if jaro_winkler(&normalized[i], &normalized[j]) >= self.similarity_threshold {
                    union(&mut parent, i, j);
                    continue;
                }

                // Check abbreviation matches
                let name_i = &orgs[i].entity.name;
                let name_j = &orgs[j].entity.name;
                if self.is_abbreviation_match(name_i, name_j) {
                    union(&mut parent, i, j);
                }
            }
        }

        let mut clusters: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..n {
            let p = find(&mut parent.clone(), i);
            clusters.entry(p).or_default().push(i);
        }

        Ok(clusters.into_values().collect())
    }

    /// Check if two names are abbreviation/long-form matches
    fn is_abbreviation_match(&self, name1: &str, name2: &str) -> bool {
        // Common Dutch ministry abbreviations
        let abbrev_map: &[(&str, &str)] = &[
            ("MinFin", "Ministerie van Financiën"),
            ("BZK", "Ministerie van Binnenlandse Zaken"),
            ("JenV", "Ministerie van Justitie en Veiligheid"),
            ("VWS", "Ministerie van Volksgezondheid"),
            ("OCW", "Ministerie van Onderwijs"),
            ("EZK", "Ministerie van Economische Zaken"),
            ("IenW", "Ministerie van Infrastructuur"),
            ("SZW", "Ministerie van Sociale Zaken"),
        ];

        for (abbr, full) in abbrev_map {
            if (name1.contains(abbr) && name2.contains("Ministerie")) ||
               (name2.contains(abbr) && name1.contains("Ministerie")) {
                return true;
            }
        }

        false
    }

    /// Merge a cluster of person entities
    async fn merge_person_cluster(
        &self,
        indices: &[usize],
        persons: &[PersonStakeholder],
    ) -> Result<PersonStakeholder, ExtractionError> {
        // Select the entity with highest confidence as base
        let best_idx = *indices.iter()
            .max_by_key(|&&i| {
                persons[i].entity.metadata.get("confidence")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32
            })
            .ok_or_else(|| ExtractionError::DeduplicationFailed("Empty cluster".to_string()))?;

        let mut result = persons[best_idx].clone();
        let canonical_id = Uuid::new_v4();
        
        // Store mapping of old IDs to new canonical ID
        let aliases: Vec<Uuid> = indices.iter()
            .map(|&i| persons[i].entity.id)
            .collect();

        // Update entity
        result.entity.id = canonical_id;
        result.entity.metadata.insert("merged_from".to_string(), 
            serde_json::json!(aliases));
        result.entity.metadata.insert("merge_count".to_string(), 
            serde_json::json!(indices.len()));
        result.entity.metadata.insert("merged_at".to_string(), 
            serde_json::json!(SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()));

        // Combine all metadata (non-empty values take precedence)
        for &idx in indices {
            if idx != best_idx {
                for (key, value) in &persons[idx].entity.metadata {
                    if !result.entity.metadata.contains_key(key) {
                        result.entity.metadata.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        Ok(result)
    }

    /// Merge a cluster of organization entities
    async fn merge_org_cluster(
        &self,
        indices: &[usize],
        orgs: &[OrganizationStakeholder],
    ) -> Result<OrganizationStakeholder, ExtractionError> {
        let best_idx = *indices.iter()
            .max_by_key(|&&i| {
                orgs[i].entity.metadata.get("confidence")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32
            })
            .ok_or_else(|| ExtractionError::DeduplicationFailed("Empty cluster".to_string()))?;

        let mut result = orgs[best_idx].clone();
        let canonical_id = Uuid::new_v4();
        
        let aliases: Vec<Uuid> = indices.iter()
            .map(|&i| orgs[i].entity.id)
            .collect();

        result.entity.id = canonical_id;
        result.entity.metadata.insert("merged_from".to_string(), 
            serde_json::json!(aliases));
        result.entity.metadata.insert("merge_count".to_string(), 
            serde_json::json!(indices.len()));
        result.entity.metadata.insert("merged_at".to_string(), 
            serde_json::json!(SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()));

        // Combine names as aliases
        let name_aliases: Vec<String> = indices.iter()
            .filter(|&&i| i != best_idx)
            .map(|&i| orgs[i].entity.name.clone())
            .collect();
        
        if !name_aliases.is_empty() {
            result.entity.metadata.insert("name_aliases".to_string(), 
                serde_json::json!(name_aliases));
        }

        Ok(result)
    }

    /// Get normalized name for comparison
    fn normalize_for_comparison(&self, person: &PersonStakeholder) -> String {
        person.entity.metadata.get("normalized_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&person.entity.name)
            .to_lowercase()
    }
}

impl Default for EntityDeduplicator {
    fn default() -> Self {
        Self::new()
    }
}

/// Jaro-Winkler similarity metric for string comparison
pub fn jaro_winkler(s1: &str, s2: &str) -> f32 {
    if s1 == s2 {
        return 1.0;
    }

    let s1_len = s1.chars().count();
    let s2_len = s2.chars().count();

    if s1_len == 0 || s2_len == 0 {
        return 0.0;
    }

    let match_distance = s1_len.max(s2_len) / 2 - 1;
    if match_distance < 0 {
        return 0.0;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let mut s1_matches = vec![false; s1_len];
    let mut s2_matches = vec![false; s2_len];

    let mut matches = 0;
    for i in 0..s1_len {
        let start = i.saturating_sub(match_distance);
        let end = (i + match_distance + 1).min(s2_len);

        for j in start..end {
            if !s2_matches[j] && s1_chars[i] == s2_chars[j] {
                s1_matches[i] = true;
                s2_matches[j] = true;
                matches += 1;
                break;
            }
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let mut transpositions = 0;
    let mut k = 0;
    for i in 0..s1_len {
        if s1_matches[i] {
            while !s2_matches[k] {
                k += 1;
            }
            if s1_chars[i] != s2_chars[k] {
                transpositions += 1;
            }
            k += 1;
        }
    }

    let jaro = (
        matches as f32 / s1_len as f32 +
        matches as f32 / s2_len as f32 +
        (matches - transpositions / 2) as f32 / matches as f32
    ) / 3.0;

    // Jaro-Winkler adjustment
    let prefix_len = s1_chars.iter()
        .zip(s2_chars.iter())
        .take_while(|(a, b)| a == b)
        .take(4)
        .count();

    let jaro_winkler = jaro + (0.1 * prefix_len as f32 * (1.0 - jaro));

    jaro_winkler.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaro_winkler_identical() {
        assert!((jaro_winkler("test", "test") - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_jaro_winkler_similar() {
        let sim = jaro_winkler("MinFin", "Ministerie van Financiën");
        assert!(sim > 0.8);
    }

    #[test]
    fn test_jaro_winkler_dissimilar() {
        let sim = jaro_winkler("Financiën", "Defensie");
        assert!(sim < 0.5);
    }

    #[test]
    fn test_jaro_winkler_symmetric() {
        let a = "Jan de Vries";
        let b = "J. de Vries";
        assert!((jaro_winkler(a, b) - jaro_winkler(b, a)).abs() < f32::EPSILON);
    }
}
```

### 3. Update Module Exports

Location: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/mod.rs`

```rust
pub mod normalizer;
pub mod deduplicator;

pub use normalizer::{EntityNormalizer, CacheStats};
pub use deduplicator::{EntityDeduplicator, jaro_winkler};
```

## Error Types

Add to `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/errors.rs` (or existing error module):

```rust
#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    #[error("Normalization failed: {0}")]
    NormalizationFailed(String),
    
    #[error("Deduplication failed: {0}")]
    DeduplicationFailed(String),
    
    // ... existing variants
}
```

## Integration Points

### Using in Main Extractor

The main extractor (Section 06) will use these components:

```rust
use crate::stakeholder::{EntityNormalizer, EntityDeduplicator};

// In StakeholderExtractor implementation:
let normalizer = EntityNormalizer::new(api_client);
let deduplicator = EntityDeduplicator::new();

// Normalize organizations
let normalized_orgs: Vec<_> = organizations.into_iter()
    .map(|org| normalizer.normalize_organization(org))
    .collect::<FuturesUnordered<_>>()
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;

// Deduplicate
let deduped_orgs = deduplicator.deduplicate_organizations(normalized_orgs).await?;
let deduped_persons = deduplicator.deduplicate_persons(persons).await?;

// Update relationships with new canonical IDs
let merged_ids = build_merge_map(&deduped_orgs, &deduped_persons);
let relationships = deduplicator.update_relationships(relationships, &merged_ids);
```

## Configuration

Environment variables (add to `.env` or config):

```bash
# Normalization cache TTL (seconds)
RIJKSOVERHEID_CACHE_TTL=86400

# Deduplication similarity threshold (0.0-1.0)
DEDUPLICATION_SIMILARITY_THRESHOLD=0.85
```

## Success Criteria

- [ ] Jaro-Winkler similarity returns 1.0 for identical strings
- [ ] Jaro-Winkler similarity >0.9 for similar Dutch name variations
- [ ] Jaro-Winkler similarity <0.5 for dissimilar strings
- [ ] Normalization applies canonical name from Rijksoverheid API
- [ ] Normalization uses local fallback when API unavailable
- [ ] Connected components clustering groups duplicate entities
- [ ] Merge creates new canonical UUID
- [ ] Merge stores old UUIDs as aliases in metadata
- [ ] Merge updates all MentionRelationships to canonical entity
- [ ] Merge log contains audit trail of entity changes
- [ ] Cache reduces API calls for repeated lookups

## Property-Based Tests

Location: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/deduplicator.rs`

```rust
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_jaro_winkler_reflexive(s in "[a-zA-Z]{1,20}") {
            assert!((jaro_winkler(&s, &s) - 1.0).abs() < f32::EPSILON);
        }

        #[test]
        fn test_jaro_winkler_symmetric(s1 in "[a-zA-Z]{1,20}", s2 in "[a-zA-Z]{1,20}") {
            let sim1 = jaro_winkler(&s1, &s2);
            let sim2 = jaro_winkler(&s2, &s1);
            assert!((sim1 - sim2).abs() < f32::EPSILON);
        }

        #[test]
        fn test_jaro_winkler_range(s1 in "[a-zA-Z]{1,20}", s2 in "[a-zA-Z]{1,20}") {
            let sim = jaro_winkler(&s1, &s2);
            assert!(sim >= 0.0 && sim <= 1.0);
        }
    }
}
```