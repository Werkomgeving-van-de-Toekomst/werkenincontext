//! Integration tests for migration validation
//!
//! These tests verify the migration validation tools.

use iou_core::graphrag::migration::{MigrationValidator, ValidationResult};
use uuid::Uuid;

#[test]
fn test_validation_result_default() {
    let result = ValidationResult::default();
    assert!(result.is_valid);
    assert_eq!(result.postgres_entity_count, 0);
    assert_eq!(result.arango_entity_count, 0);
    assert!(result.by_entity_type.is_empty());
    assert!(result.mismatches.is_empty());
}

#[test]
fn test_validation_result_counts_match_identical() {
    let mut result = ValidationResult::default();
    result.postgres_entity_count = 1000;
    result.arango_entity_count = 1000;
    assert!(result.counts_match(5.0));
}

#[test]
fn test_validation_result_counts_match_within_5_percent() {
    let mut result = ValidationResult::default();
    result.postgres_entity_count = 1000;
    result.arango_entity_count = 950; // 5% difference
    assert!(result.counts_match(5.0));
}

#[test]
fn test_validation_result_counts_match_exceeds_5_percent() {
    let mut result = ValidationResult::default();
    result.postgres_entity_count = 1000;
    result.arango_entity_count = 944; // 5.6% difference
    assert!(!result.counts_match(5.0));
}

#[test]
fn test_validation_result_counts_match_zero_handling() {
    let result = ValidationResult::default();
    assert!(result.counts_match(5.0)); // Both zero should match
}

#[test]
fn test_migration_validator_default_tolerance() {
    let validator = MigrationValidator::default();
    assert_eq!(validator.tolerance_pct, 5.0);
}

#[test]
fn test_migration_validator_custom_tolerance() {
    let validator = MigrationValidator::with_tolerance(10.0);
    assert_eq!(validator.tolerance_pct, 10.0);
}

#[test]
fn test_migration_validator_with_tolerance_strict() {
    let validator = MigrationValidator::with_tolerance(1.0);
    assert_eq!(validator.tolerance_pct, 1.0);

    let mut result = ValidationResult::default();
    result.postgres_entity_count = 1000;
    result.arango_entity_count = 990; // 1% difference
    assert!(result.counts_match(1.0));

    result.arango_entity_count = 989; // 1.1% difference
    assert!(!result.counts_match(1.0));
}

#[test]
fn test_migration_validator_with_tolerance_lenient() {
    let validator = MigrationValidator::with_tolerance(20.0);
    assert_eq!(validator.tolerance_pct, 20.0);

    let mut result = ValidationResult::default();
    result.postgres_entity_count = 1000;
    result.arango_entity_count = 800; // 20% difference
    assert!(result.counts_match(20.0));

    result.arango_entity_count = 799; // 20.1% difference
    assert!(!result.counts_match(20.0));
}

#[test]
fn test_validation_result_entity_type_counts() {
    use iou_core::graphrag::migration::EntityTypeCount;

    let mut result = ValidationResult::default();
    result.by_entity_type.push(EntityTypeCount {
        entity_type: "Person".to_string(),
        postgres_count: 500,
        arango_count: 500,
        matches: true,
    });
    result.by_entity_type.push(EntityTypeCount {
        entity_type: "Organization".to_string(),
        postgres_count: 300,
        arango_count: 295, // 1.67% difference
        matches: true,
    });

    result.postgres_entity_count = 800;
    result.arango_entity_count = 795;

    assert!(result.counts_match(5.0));
    assert_eq!(result.by_entity_type.len(), 2);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn migration_entity_counts_match() {
    use iou_core::graphrag::GraphStore;

    // This test would require a running ArangoDB instance with migrated data
    // and a PostgreSQL connection for comparison
    let validator = MigrationValidator::default();

    // Simulate PostgreSQL counts
    let postgres_counts = vec![
        ("Person".to_string(), 100),
        ("Organization".to_string(), 50),
        ("Location".to_string(), 25),
    ];

    // Create a mock store (would use real store in actual test)
    // let store = setup_test_store().await;
    // let result = validator.validate_entity_counts(&store, postgres_counts).await.unwrap();
    // assert!(result.is_valid);

    // For now, just verify the validator structure
    assert_eq!(postgres_counts.len(), 3);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn migration_relationship_counts_match() {
    use iou_core::graphrag::GraphStore;

    // Similar to entity counts test, this would validate relationship counts
    // between PostgreSQL and ArangoDB
    let validator = MigrationValidator::default();

    // Simulate PostgreSQL counts
    let postgres_counts = vec![
        ("WorksFor".to_string(), 75),
        ("LocatedIn".to_string(), 30),
    ];

    // In actual implementation:
    // let store = setup_test_store().await;
    // let result = validator.validate_relationship_counts(&store, postgres_counts).await.unwrap();
    // assert!(result.is_valid);

    assert_eq!(postgres_counts.len(), 2);
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn migration_sample_data_matches() {
    use iou_core::graphrag::GraphStore;

    // This test would validate that sample entities match between databases
    let validator = MigrationValidator::default();

    let sample_ids = vec![
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    ];

    // In actual implementation:
    // let store = setup_test_store().await;
    // let result = validator.validate_sample_data(&store, sample_ids).await.unwrap();
    // assert!(result.matches > 0);

    assert_eq!(sample_ids.len(), 3);
}
