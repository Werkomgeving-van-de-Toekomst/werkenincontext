//! Migration strategy and validation helpers for PostgreSQL to ArangoDB migration
//!
//! This module provides tools for:
//! - Validating data consistency between PostgreSQL and ArangoDB
//! - Running comparison queries
//! - Monitoring migration progress

use crate::graphrag::{Entity, EntityType, GraphStore, StoreError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Validation result comparing PostgreSQL and ArangoDB data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Total entities in PostgreSQL
    pub postgres_entity_count: u64,
    /// Total entities in ArangoDB
    pub arango_entity_count: u64,
    /// Count comparison by entity type
    pub by_entity_type: Vec<EntityTypeCount>,
    /// Any mismatches found
    pub mismatches: Vec<String>,
    /// Overall validation status
    pub is_valid: bool,
}

/// Entity count by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTypeCount {
    pub entity_type: String,
    pub postgres_count: u64,
    pub arango_count: u64,
    pub matches: bool,
}

/// Sample data comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleComparison {
    pub postgres_sample: Vec<EntitySample>,
    pub arango_sample: Vec<EntitySample>,
    pub matches: usize,
    pub total: usize,
}

/// Sample entity data for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySample {
    pub id: Uuid,
    pub name: String,
    pub entity_type: String,
    pub canonical_name: Option<String>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            postgres_entity_count: 0,
            arango_entity_count: 0,
            by_entity_type: Vec::new(),
            mismatches: Vec::new(),
            is_valid: true,
        }
    }

    /// Check if counts match within tolerance
    pub fn counts_match(&self, tolerance_pct: f64) -> bool {
        if self.postgres_entity_count == 0 && self.arango_entity_count == 0 {
            return true;
        }

        let diff = (self.postgres_entity_count as i64 - self.arango_entity_count as i64).abs() as f64;
        let max = self.postgres_entity_count.max(self.arango_entity_count) as f64;

        if max == 0.0 {
            return true;
        }

        (diff / max) * 100.0 <= tolerance_pct
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Migration validator for comparing PostgreSQL and ArangoDB data
pub struct MigrationValidator {
    /// Tolerance percentage for count matching (default: 5%)
    pub tolerance_pct: f64,
}

impl MigrationValidator {
    /// Create a new validator with default tolerance
    pub fn new() -> Self {
        Self { tolerance_pct: 5.0 }
    }

    /// Create a new validator with custom tolerance
    pub fn with_tolerance(tolerance_pct: f64) -> Self {
        Self { tolerance_pct }
    }

    /// Validate entity counts between PostgreSQL and ArangoDB
    ///
    /// This method compares total entity counts and counts by type.
    /// In a real migration, this would query PostgreSQL for the actual counts.
    ///
    /// # Arguments
    /// * `arango_store` - The ArangoDB graph store
    /// * `postgres_counts` - Entity counts from PostgreSQL (type, count)
    ///
    /// # Returns
    /// Validation result with comparison details
    pub async fn validate_entity_counts(
        &self,
        arango_store: &GraphStore,
        postgres_counts: Vec<(String, u64)>,
    ) -> Result<ValidationResult, StoreError> {
        let mut result = ValidationResult::new();

        // Get ArangoDB counts by querying each collection
        // Note: This is a simplified approach - actual implementation
        // would use AQL queries with COLLECT

        for (entity_type_str, pg_count) in postgres_counts {
            // In a real implementation, we would query the specific collection
            // and count documents by entity_type
            //
            // For now, we'll create placeholder entries
            result.by_entity_type.push(EntityTypeCount {
                entity_type: entity_type_str.clone(),
                postgres_count: pg_count,
                arango_count: 0, // Would be filled by actual query
                matches: false,
            });

            result.postgres_entity_count += pg_count;
        }

        // Check if counts match within tolerance
        result.is_valid = result.counts_match(self.tolerance_pct);

        if !result.is_valid {
            result.mismatches.push(format!(
                "Entity counts differ: PG={}, Arango={}",
                result.postgres_entity_count, result.arango_entity_count
            ));
        }

        Ok(result)
    }

    /// Validate a sample of entities match between databases
    ///
    /// # Arguments
    /// * `arango_store` - The ArangoDB graph store
    /// * `sample_ids` - List of entity IDs to compare
    ///
    /// # Returns
    /// Sample comparison result
    pub async fn validate_sample_data(
        &self,
        arango_store: &GraphStore,
        sample_ids: Vec<Uuid>,
    ) -> Result<SampleComparison, StoreError> {
        let mut arango_sample = Vec::new();
        let total = sample_ids.len();

        for id in &sample_ids {
            match arango_store.get_entity(*id).await {
                Ok(Some(entity)) => {
                    arango_sample.push(EntitySample {
                        id: entity.id,
                        name: entity.name.clone(),
                        entity_type: format!("{:?}", entity.entity_type),
                        canonical_name: entity.canonical_name.clone(),
                    });
                }
                Ok(None) => {
                    // Entity not found in ArangoDB
                    continue;
                }
                Err(_) => {
                    // Query error - log and continue
                    continue;
                }
            }
        }

        Ok(SampleComparison {
            postgres_sample: Vec::new(), // Would be filled by PostgreSQL query
            arango_sample,
            matches: 0,
            total,
        })
    }
}

impl Default for MigrationValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_new() {
        let result = ValidationResult::new();
        assert!(result.is_valid);
        assert_eq!(result.postgres_entity_count, 0);
        assert_eq!(result.arango_entity_count, 0);
    }

    #[test]
    fn test_validation_result_counts_match_empty() {
        let result = ValidationResult::new();
        assert!(result.counts_match(5.0));
    }

    #[test]
    fn test_validation_result_counts_match_exact() {
        let mut result = ValidationResult::new();
        result.postgres_entity_count = 100;
        result.arango_entity_count = 100;
        assert!(result.counts_match(5.0));
    }

    #[test]
    fn test_validation_result_counts_match_within_tolerance() {
        let mut result = ValidationResult::new();
        result.postgres_entity_count = 100;
        result.arango_entity_count = 95; // 5% difference
        assert!(result.counts_match(5.0));
    }

    #[test]
    fn test_validation_result_counts_match_outside_tolerance() {
        let mut result = ValidationResult::new();
        result.postgres_entity_count = 100;
        result.arango_entity_count = 94; // 6% difference
        assert!(!result.counts_match(5.0));
    }

    #[test]
    fn test_migration_validator_new() {
        let validator = MigrationValidator::new();
        assert_eq!(validator.tolerance_pct, 5.0);
    }

    #[test]
    fn test_migration_validator_with_tolerance() {
        let validator = MigrationValidator::with_tolerance(10.0);
        assert_eq!(validator.tolerance_pct, 10.0);
    }
}
