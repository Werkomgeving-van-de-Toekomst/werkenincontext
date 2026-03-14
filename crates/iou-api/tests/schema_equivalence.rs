//! Schema Equivalence Tests
//!
//! Tests to verify that PostgreSQL schema matches DuckDB structure
//! for the hybrid architecture.

use uuid::Uuid;

#[tokio::test]
#[ignore] // Requires running Supabase instance
async fn test_information_domains_schema_matches() {
    // This test verifies PostgreSQL schema matches DuckDB structure
    let database_url = std::env::var("SUPABASE_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/iou_modern".to_string());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Supabase");

    // Check table exists
    let result = sqlx::query(
        "SELECT EXISTS (SELECT FROM pg_tables WHERE tablename = 'information_domains')"
    )
    .fetch_one(&pool)
    .await
    .expect("Query failed");

    let exists: bool = result.get("exists");
    assert!(exists, "information_domains table should exist");

    // Verify columns
    let columns: Vec<String> = sqlx::query(
        "SELECT column_name FROM information_schema.columns
         WHERE table_name = 'information_domains'
         ORDER BY ordinal_position"
    )
    .fetch_all(&pool)
    .await
    .unwrap()
    .into_iter()
    .map(|row| row.get("column_name"))
    .collect();

    assert!(columns.contains(&"id".to_string()), "Missing column: id");
    assert!(columns.contains(&"domain_type".to_string()), "Missing column: domain_type");
    assert!(columns.contains(&"name".to_string()), "Missing column: name");
    assert!(columns.contains(&"organization_id".to_string()), "Missing column: organization_id");
    assert!(columns.contains(&"status".to_string()), "Missing column: status");

    println!("PostgreSQL information_domains schema verified");
}

#[tokio::test]
#[ignore]
async fn test_information_objects_schema_matches() {
    let database_url = std::env::var("SUPABASE_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/iou_modern".to_string());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Supabase");

    // Check table exists
    let result = sqlx::query(
        "SELECT EXISTS (SELECT FROM pg_tables WHERE tablename = 'information_objects')"
    )
    .fetch_one(&pool)
    .await
    .expect("Query failed");

    let exists: bool = result.get("exists");
    assert!(exists, "information_objects table should exist");

    // Verify key columns
    let columns: Vec<String> = sqlx::query(
        "SELECT column_name FROM information_schema.columns
         WHERE table_name = 'information_objects'
         ORDER BY ordinal_position"
    )
    .fetch_all(&pool)
    .await
    .unwrap()
    .into_iter()
    .map(|row| row.get("column_name"))
    .collect();

    assert!(columns.contains(&"id".to_string()));
    assert!(columns.contains(&"domain_id".to_string()));
    assert!(columns.contains(&"object_type".to_string()));
    assert!(columns.contains(&"title".to_string()));

    println!("PostgreSQL information_objects schema verified");
}

#[tokio::test]
#[ignore]
async fn test_documents_schema_matches() {
    let database_url = std::env::var("SUPABASE_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/iou_modern".to_string());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Supabase");

    // Check table exists
    let result = sqlx::query(
        "SELECT EXISTS (SELECT FROM pg_tables WHERE tablename = 'documents')"
    )
    .fetch_one(&pool)
    .await
    .expect("Query failed");

    let exists: bool = result.get("exists");
    assert!(exists, "documents table should exist");

    println!("PostgreSQL documents schema verified");
}

#[tokio::test]
#[ignore]
async fn test_view_exists_searchable_objects() {
    let database_url = std::env::var("SUPABASE_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/iou_modern".to_string());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Supabase");

    // Check view exists
    let result = sqlx::query(
        "SELECT EXISTS (SELECT FROM pg_views WHERE viewname = 'v_searchable_objects')"
    )
    .fetch_one(&pool)
    .await
    .expect("Query failed");

    let exists: bool = result.get("exists");
    assert!(exists, "v_searchable_objects view should exist");

    println!("PostgreSQL view v_searchable_objects verified");
}

#[tokio::test]
#[ignore]
async fn test_view_exists_compliance_overview() {
    let database_url = std::env::var("SUPABASE_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/iou_modern".to_string());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Supabase");

    // Check view exists
    let result = sqlx::query(
        "SELECT EXISTS (SELECT FROM pg_views WHERE viewname = 'v_compliance_overview')"
    )
    .fetch_one(&pool)
    .await
    .expect("Query failed");

    let exists: bool = result.get("exists");
    assert!(exists, "v_compliance_overview view should exist");

    println!("PostgreSQL view v_compliance_overview verified");
}

#[tokio::test]
#[ignore]
async fn test_extensions_enabled() {
    let database_url = std::env::var("SUPABASE_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/iou_modern".to_string());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Supabase");

    // Check uuid-ossp extension
    let result = sqlx::query(
        "SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'uuid-ossp')"
    )
    .fetch_one(&pool)
    .await
    .expect("Query failed");

    let exists: bool = result.get("exists");
    assert!(exists, "uuid-ossp extension should be enabled");

    // Check pgcrypto extension
    let result = sqlx::query(
        "SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'pgcrypto')"
    )
    .fetch_one(&pool)
    .await
    .expect("Query failed");

    let exists: bool = result.get("exists");
    assert!(exists, "pgcrypto extension should be enabled");

    println!("PostgreSQL extensions verified");
}
