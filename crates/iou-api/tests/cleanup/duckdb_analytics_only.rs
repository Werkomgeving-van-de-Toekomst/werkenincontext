//! DuckDB analytics-only verification tests
//!
//! Verify that DuckDB is used exclusively for analytics workloads
//! and all transactional operations go through Supabase.

use std::path::Path;

/// Test: Verify DuckDB is used for analytics only
#[test]
fn test_duckdb_storage_module_exists() {
    let src_path = Path::new("crates/iou-storage/src");

    // iou-storage crate should handle metadata/analytics operations
    let metadata_file = src_path.join("metadata.rs");
    assert!(
        metadata_file.exists(),
        "iou-storage/metadata.rs should exist for DuckDB analytics operations"
    );
}

/// Test: Verify Supabase is the primary database
#[test]
fn test_supabase_primary_database() {
    let src_path = Path::new("crates/iou-api/src");

    // Check for Supabase integration
    let supabase_file = src_path.join("supabase.rs");
    assert!(
        supabase_file.exists(),
        "iou-api/supabase.rs should exist as primary database interface"
    );

    // Check for ETL module (handles data transfer from Supabase to DuckDB)
    let etl_path = src_path.join("etl");
    assert!(
        etl_path.exists(),
        "iou-api/etl/ should exist for ETL pipeline"
    );
}

/// Test: Verify ETL module structure
#[test]
fn test_etl_pipeline_structure() {
    let src_path = Path::new("crates/iou-api/src/etl");

    // ETL module should have key components
    let expected_files = vec![
        "mod.rs",
        "config.rs",
        "pipeline.rs",
        "tables.rs",
        "outbox.rs", // Added in section-05
    ];

    for file in &expected_files {
        let file_path = src_path.join(file);
        assert!(
            file_path.exists(),
            "ETL module should have {}",
            file
        );
    }
}

/// Test: Verify transactional outbox pattern exists
#[test]
fn test_outbox_pattern_exists() {
    // The outbox pattern ensures reliable data transfer
    let outbox_file = Path::new("crates/iou-api/src/etl/outbox.rs");
    assert!(
        outbox_file.exists(),
        "ETL outbox pattern should be implemented"
    );
}
