//! Entity deduplication for stakeholder extraction
//!
//! This module handles detection and merging of duplicate entities using
//! string similarity (Jaro-Winkler) and clustering algorithms.

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::stakeholder::{
    error::DeduplicationError,
    types::{OrganizationStakeholder, PersonStakeholder},
};

/// Similarity threshold for considering entities as duplicates
const DEFAULT_SIMILARITY_THRESHOLD: f32 = 0.85;

/// Helper function to insert into metadata
fn insert_metadata(metadata: &mut serde_json::Value, key: &str, value: serde_json::Value) {
    if let Some(obj) = metadata.as_object_mut() {
        obj.insert(key.to_string(), value);
    }
}

/// Helper function to check if metadata contains a key
fn contains_key(metadata: &serde_json::Value, key: &str) -> bool {
    metadata.as_object().map(|o| o.contains_key(key)).unwrap_or(false)
}

/// Deduplicates entities using string similarity and clustering
pub struct EntityDeduplicator {
    similarity_threshold: f32,
}

impl EntityDeduplicator {
    /// Create a new deduplicator with default threshold
    pub fn new() -> Self {
        Self {
            similarity_threshold: DEFAULT_SIMILARITY_THRESHOLD,
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
    ) -> Result<Vec<PersonStakeholder>, DeduplicationError> {
        if persons.is_empty() {
            return Ok(Vec::new());
        }

        let normalized: Vec<_> = persons
            .iter()
            .map(|p| self.normalize_for_comparison(p))
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
    ) -> Result<Vec<OrganizationStakeholder>, DeduplicationError> {
        if orgs.is_empty() {
            return Ok(Vec::new());
        }

        let normalized: Vec<_> = orgs
            .iter()
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

    /// Find clusters of duplicate entities using connected components
    fn find_clusters(&self, normalized: &[String]) -> Result<Vec<Vec<usize>>, DeduplicationError> {
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
            for j in (i + 1)..n {
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

    /// Find clusters by organization names with special handling for abbreviations
    fn find_clusters_by_names(
        &self,
        normalized: &[String],
        orgs: &[OrganizationStakeholder],
    ) -> Result<Vec<Vec<usize>>, DeduplicationError> {
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
            for j in (i + 1)..n {
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

        for (abbr, _full) in abbrev_map {
            if (name1.contains(abbr) && name2.contains("Ministerie"))
                || (name2.contains(abbr) && name1.contains("Ministerie"))
            {
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
    ) -> Result<PersonStakeholder, DeduplicationError> {
        // Select the entity with highest confidence as base
        let best_idx = *indices
            .iter()
            .max_by(|&&i, &&j| {
                persons[i]
                    .entity
                    .confidence
                    .partial_cmp(&persons[j].entity.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| DeduplicationError::MergeConflict("Empty cluster".to_string()))?;

        let mut result = persons[best_idx].clone();
        let canonical_id = Uuid::new_v4();

        // Store mapping of old IDs to new canonical ID
        let aliases: Vec<Uuid> = indices.iter().map(|&i| persons[i].entity.id).collect();

        // Update entity
        result.entity.id = canonical_id;
        insert_metadata(
            &mut result.entity.metadata,
            "merged_from",
            serde_json::json!(aliases),
        );
        insert_metadata(
            &mut result.entity.metadata,
            "merge_count",
            serde_json::json!(indices.len()),
        );
        insert_metadata(
            &mut result.entity.metadata,
            "merged_at",
            serde_json::json!(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
        );

        // Combine all metadata (non-empty values take precedence)
        for &idx in indices {
            if idx != best_idx {
                if let Some(obj) = persons[idx].entity.metadata.as_object() {
                    for (key, value) in obj {
                        if !contains_key(&result.entity.metadata, key) {
                            insert_metadata(&mut result.entity.metadata, key, value.clone());
                        }
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
    ) -> Result<OrganizationStakeholder, DeduplicationError> {
        let best_idx = *indices
            .iter()
            .max_by(|&&i, &&j| {
                orgs[i]
                    .entity
                    .confidence
                    .partial_cmp(&orgs[j].entity.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| DeduplicationError::MergeConflict("Empty cluster".to_string()))?;

        let mut result = orgs[best_idx].clone();
        let canonical_id = Uuid::new_v4();

        let aliases: Vec<Uuid> = indices.iter().map(|&i| orgs[i].entity.id).collect();

        result.entity.id = canonical_id;
        insert_metadata(
            &mut result.entity.metadata,
            "merged_from",
            serde_json::json!(aliases),
        );
        insert_metadata(
            &mut result.entity.metadata,
            "merge_count",
            serde_json::json!(indices.len()),
        );
        insert_metadata(
            &mut result.entity.metadata,
            "merged_at",
            serde_json::json!(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
        );

        // Combine names as aliases
        let name_aliases: Vec<String> = indices
            .iter()
            .filter(|&&i| i != best_idx)
            .map(|&i| orgs[i].entity.name.clone())
            .collect();

        if !name_aliases.is_empty() {
            insert_metadata(
                &mut result.entity.metadata,
                "name_aliases",
                serde_json::json!(name_aliases),
            );
        }

        Ok(result)
    }

    /// Get normalized name for comparison
    fn normalize_for_comparison(&self, person: &PersonStakeholder) -> String {
        person
            .entity
            .metadata
            .get("normalized_name")
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
///
/// Returns a value between 0.0 (no similarity) and 1.0 (identical).
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

    let jaro = (matches as f32 / s1_len as f32
        + matches as f32 / s2_len as f32
        + (matches - transpositions / 2) as f32 / matches as f32)
        / 3.0;

    // Jaro-Winkler adjustment
    let prefix_len = s1_chars
        .iter()
        .zip(s2_chars.iter())
        .take_while(|(a, b)| a == b)
        .take(4)
        .count();

    let jaro_winkler = jaro + 0.1 * prefix_len as f32 * (1.0 - jaro);

    jaro_winkler.min(1.0)
}

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
        let result = jaro_winkler("MinFin", "Ministerie van Financiën");
        assert!(
            result > 0.7,
            "Expected >0.7 for abbreviations, got {}",
            result
        );
    }

    #[test]
    fn test_jaro_winkler_dissimilar_strings() {
        let result = jaro_winkler("Ministerie van Financiën", "Ministerie van Defensie");
        // These share "Ministerie van" so similarity is higher than 0.7
        // But they should still be below 0.95 (not identical)
        assert!(
            result < 0.95 && result > 0.7,
            "Expected 0.7 < similarity < 0.95 for different ministries, got {}",
            result
        );
    }

    #[test]
    fn test_jaro_winkler_symmetry() {
        let a = "Jan de Vries";
        let b = "J. de Vries";
        assert!(
            (jaro_winkler(a, b) - jaro_winkler(b, a)).abs() < f32::EPSILON,
            "Not symmetric: {} vs {}",
            jaro_winkler(a, b),
            jaro_winkler(b, a)
        );
    }

    #[test]
    fn test_jaro_winkler_range() {
        // Test various strings
        let pairs = vec![
            ("", ""),
            ("a", ""),
            ("test", "test"),
            ("test", "testing"),
            ("completely different", "strings here"),
        ];

        for (s1, s2) in pairs {
            let sim = jaro_winkler(s1, s2);
            assert!(
                sim >= 0.0 && sim <= 1.0,
                "Similarity out of range: {} for ({}, {})",
                sim,
                s1,
                s2
            );
        }
    }

    #[test]
    fn test_deduplicator_default_threshold() {
        let dedup = EntityDeduplicator::new();
        assert_eq!(dedup.similarity_threshold, DEFAULT_SIMILARITY_THRESHOLD);
    }

    #[test]
    fn test_deduplicator_custom_threshold() {
        let dedup = EntityDeduplicator::with_threshold(0.9);
        assert_eq!(dedup.similarity_threshold, 0.9);
    }

    #[tokio::test]
    async fn test_deduplicate_empty_list() {
        let dedup = EntityDeduplicator::new();
        let result = dedup.deduplicate_persons(vec![]).await.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_deduplicate_single_person() {
        let dedup = EntityDeduplicator::new();
        let persons = vec![PersonStakeholder::new("Jan de Vries".to_string(), 0.9)];
        let result = dedup.deduplicate_persons(persons).await.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_deduplicate_very_similar_names() {
        let dedup = EntityDeduplicator::with_threshold(0.85);
        let persons = vec![
            PersonStakeholder::new("Jan de Vries".to_string(), 0.9),
            PersonStakeholder::new("Jan de Vries".to_string(), 0.8),
        ];
        let result = dedup.deduplicate_persons(persons).await.unwrap();
        // These are identical, should be merged
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_deduplicate_different_names() {
        let dedup = EntityDeduplicator::new();
        let persons = vec![
            PersonStakeholder::new("Jan de Vries".to_string(), 0.9),
            PersonStakeholder::new("Piet Jansen".to_string(), 0.9),
        ];
        let result = dedup.deduplicate_persons(persons).await.unwrap();
        // These are very different, should not be merged
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_deduplicate_organizations_with_abbreviations() {
        let dedup = EntityDeduplicator::new();
        let orgs = vec![
            OrganizationStakeholder::new("MinFin".to_string(), 0.9),
            OrganizationStakeholder::new("Ministerie van Financiën".to_string(), 0.9),
        ];
        let result = dedup.deduplicate_organizations(orgs).await.unwrap();
        // Abbreviation match should trigger merge
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_merged_entity_has_new_uuid() {
        let dedup = EntityDeduplicator::with_threshold(0.85);
        let persons = vec![
            PersonStakeholder::new("Jan de Vries".to_string(), 0.9),
            PersonStakeholder::new("Jan de Vries".to_string(), 0.8),
        ];

        let id1 = persons[0].entity.id;
        let id2 = persons[1].entity.id;

        let result = dedup.deduplicate_persons(persons).await.unwrap();
        assert_ne!(result[0].entity.id, id1);
        assert_ne!(result[0].entity.id, id2);
    }

    #[tokio::test]
    async fn test_merged_entity_stores_aliases() {
        let dedup = EntityDeduplicator::with_threshold(0.85);
        let persons = vec![
            PersonStakeholder::new("Jan de Vries".to_string(), 0.9),
            PersonStakeholder::new("Jan de Vries".to_string(), 0.8),
        ];

        let result = dedup.deduplicate_persons(persons).await.unwrap();
        let merged_from = result[0]
            .entity
            .metadata
            .get("merged_from")
            .and_then(|v| v.as_array());

        assert!(merged_from.is_some());
        assert_eq!(merged_from.unwrap().len(), 2);
    }
}
