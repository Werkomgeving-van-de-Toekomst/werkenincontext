//! Dual-Write Tests
//!
//! Tests for the dual-write pattern that writes to both DuckDB and Supabase.

use iou_api::{db::Database, dual_write::DualWrite, supabase::SupabasePool};
use iou_core::domain::{DomainType, InformationDomain};
use uuid::Uuid;

async fn get_test_duckdb() -> Database {
    let temp_dir = std::env::var("CARGO_TARGET_TMPDIR")
        .unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().to_string());
    let db_path = std::path::PathBuf::from(temp_dir)
        .join(format!("test_iou_dual_write_{}.db", Uuid::new_v4()));

    let db = Database::new(db_path.to_str().unwrap())
        .expect("Failed to create DuckDB");
    db.initialize_schema()
        .expect("Failed to initialize schema");
    db
}

async fn get_test_supabase() -> SupabasePool {
    let database_url = std::env::var("SUPABASE_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/iou_modern".to_string());

    SupabasePool::new(&database_url)
        .await
        .expect("Failed to connect to Supabase")
}

#[tokio::test]
#[ignore] // Requires running Supabase instance
async fn test_dual_write_consistency() {
    let duckdb = get_test_duckdb().await;
    let supabase = get_test_supabase().await;

    let domain = InformationDomain::new(
        DomainType::Zaak,
        "Test Dual-Write Domain".to_string(),
        Uuid::new_v4(),
    );

    let result = domain.dual_write(&duckdb, &supabase).await;

    assert!(result.is_success(), "Dual-write should succeed");
    let id = result.value().expect("Should have an ID");

    // Verify record exists in DuckDB
    let duckdb_record = duckdb.get_domain(id).unwrap();
    assert!(duckdb_record.is_some(), "Record should exist in DuckDB");

    // Verify record exists in Supabase
    let supabase_record: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT id, name FROM information_domains WHERE id = $1"
    )
    .bind(id)
    .fetch_one(supabase.inner())
    .await
    .ok();

    assert!(supabase_record.is_some(), "Record should exist in Supabase");

    println!("Dual-write consistency test passed for domain: {}", id);
}

#[tokio::test]
#[ignore]
async fn test_dual_write_with_optional_fields() {
    let duckdb = get_test_duckdb().await;
    let supabase = get_test_supabase().await;

    let mut domain = InformationDomain::new(
        DomainType::Project,
        "Project with Details".to_string(),
        Uuid::new_v4(),
    );
    domain.description = Some("A detailed project description".to_string());

    let result = domain.dual_write(&duckdb, &supabase).await;

    assert!(result.is_success());
    let id = result.value().unwrap();

    // Verify description in both databases
    let duckdb_record = duckdb.get_domain(id).unwrap().unwrap();
    assert_eq!(duckdb_record.description, Some("A detailed project description".to_string()));

    let supabase_desc: Option<String> = sqlx::query_scalar(
        "SELECT description FROM information_domains WHERE id = $1"
    )
    .bind(id)
    .fetch_one(supabase.inner())
    .await
    .unwrap();

    assert_eq!(supabase_desc, Some("A detailed project description".to_string()));

    println!("Dual-write with optional fields passed");
}

#[tokio::test]
#[ignore]
async fn test_dual_update() {
    let duckdb = get_test_duckdb().await;
    let supabase = get_test_supabase().await;

    let mut domain = InformationDomain::new(
        DomainType::Beleid,
        "Original Name".to_string(),
        Uuid::new_v4(),
    );

    // Initial write
    let result = domain.dual_write(&duckdb, &supabase).await;
    assert!(result.is_success());
    let id = result.value().unwrap();

    // Update the domain
    domain.name = "Updated Name".to_string();
    domain.description = Some("Updated description".to_string());

    let update_result = domain.dual_update(&duckdb, &supabase).await;
    assert!(update_result.is_success());

    // Verify update in DuckDB
    let duckdb_record = duckdb.get_domain(id).unwrap().unwrap();
    assert_eq!(duckdb_record.name, "Updated Name");

    // Verify update in Supabase
    let supabase_name: String = sqlx::query_scalar(
        "SELECT name FROM information_domains WHERE id = $1"
    )
    .bind(id)
    .fetch_one(supabase.inner())
    .await
    .unwrap();

    assert_eq!(supabase_name, "Updated Name");

    println!("Dual-update test passed");
}

#[tokio::test]
fn test_read_source_from_env_default() {
    // Clear env var
    std::env::remove_var("READ_SOURCE");

    let source = iou_api::ReadSource::from_env();
    assert_eq!(source, iou_api::ReadSource::DuckDb);
}

#[tokio::test]
fn test_read_source_from_env_supabase() {
    std::env::set_var("READ_SOURCE", "supabase");
    let source = iou_api::ReadSource::from_env();
    assert_eq!(source, iou_api::ReadSource::Supabase);
    std::env::remove_var("READ_SOURCE");
}
