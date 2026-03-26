# Section 13: Migration Strategy

## Overview

This section documents migration strategy from PostgreSQL to ArangoDB. Includes dual-write phase, read migration, cutover plan, and validation queries.

## Implementation Status

**Implemented: 2025-03-25**

## Dependencies

All previous sections must be complete.

## Implementation

### Files Created

1. `crates/iou-core/src/graphrag/migration.rs` - Migration validation helpers
2. `crates/iou-core/tests/graphrag/migration_tests.rs` - Validation tests
3. `implementation/MIGRATION_STRATEGY.md` - Full migration documentation

### Code Added

```rust
// Migration validation types and helpers
pub struct MigrationValidator { pub tolerance_pct: f64 }
pub struct ValidationResult { /* comparison results */ }
pub struct SampleComparison { /* sample data comparison */ }

impl MigrationValidator {
    pub fn new() -> Self;
    pub fn with_tolerance(tolerance_pct: f64) -> Self;
    pub async fn validate_entity_counts(&self, store, postgres_counts) -> Result<ValidationResult, StoreError>;
    pub async fn validate_sample_data(&self, store, sample_ids) -> Result<SampleComparison, StoreError>;
}
```

### Migration Phases

See `MIGRATION_STRATEGY.md` for full details:

1. **Phase 1: Dual-Write Period** (2-4 weeks)
   - Keep PostgreSQL operational
   - Add ArangoDB alongside
   - Write all graph data to both databases
   - Verify data consistency

2. **Phase 2: Read Migration** (1-2 weeks)
   - Switch read operations to ArangoDB
   - Keep PostgreSQL as backup
   - Monitor performance and correctness

3. **Phase 3: Cutover** (1 week)
   - Deprecate PostgreSQL graph storage
   - Remove dual-write code
   - Keep PostgreSQL for non-graph operations

### Tests Implemented

- `test_validation_result_default` ✅
- `test_validation_result_counts_match_identical` ✅
- `test_validation_result_counts_match_within_5_percent` ✅
- `test_validation_result_counts_match_exceeds_5_percent` ✅
- `test_validation_result_counts_match_zero_handling` ✅
- `test_migration_validator_default_tolerance` ✅
- `test_migration_validator_custom_tolerance` ✅
- `test_migration_validator_with_tolerance_strict` ✅
- `test_migration_validator_with_tolerance_lenient` ✅
- `test_validation_result_entity_type_counts` ✅
- `migration_entity_counts_match` (ignored - requires ArangoDB)
- `migration_relationship_counts_match` (ignored - requires ArangoDB)
- `migration_sample_data_matches` (ignored - requires ArangoDB)
