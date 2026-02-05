//! DuckDB database layer
//!
//! DuckDB is een embedded analytische database - geen server nodig!
//! Perfect voor:
//! - Single-file deployment
//! - Analytische queries (aggregaties, window functions)
//! - Direct lezen van Parquet/CSV/JSON
//! - WASM support (toekomstig)

use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, NaiveDate, Utc};
use duckdb::{params, Connection, Result as DuckResult};
use uuid::Uuid;

use iou_core::domain::{DomainStatus, DomainType, InformationDomain};
use iou_core::objects::{InformationObject, ObjectType};

/// Convert DateTime to string for DuckDB storage
fn datetime_to_string(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
}

/// Parse DateTime from DuckDB string
fn parse_datetime(s: &str) -> DateTime<Utc> {
    // Try various formats
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.6f")
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.6f"))
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S"))
        .map(|ndt| ndt.and_utc())
        .unwrap_or_else(|_| Utc::now())
}

/// Parse optional date from DuckDB (as NaiveDate)
#[allow(dead_code)]
fn parse_optional_date(s: Option<String>) -> Option<NaiveDate> {
    s.and_then(|ds| chrono::NaiveDate::parse_from_str(&ds, "%Y-%m-%d").ok())
}

/// Parse optional datetime from DuckDB
fn parse_optional_datetime(s: Option<String>) -> Option<DateTime<Utc>> {
    s.map(|ds| parse_datetime(&ds))
}

/// Database wrapper met thread-safe connection
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Create new database connection
    pub fn new(path: &str) -> anyhow::Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Initialize database schema
    pub fn initialize_schema(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        // Read and execute schema file
        let schema = include_str!("../../../migrations/001_initial_schema.sql");

        // Split on semicolons and execute each statement
        // (DuckDB doesn't support multiple statements in one execute)
        for statement in schema.split(';') {
            let stmt = statement.trim();
            if !stmt.is_empty() && !stmt.starts_with("--") {
                // Skip comments and empty statements
                if let Err(e) = conn.execute(stmt, []) {
                    // Ignore "already exists" errors for idempotent schema
                    let err_str = e.to_string();
                    if !err_str.contains("already exists") {
                        tracing::warn!("Schema statement failed: {}", err_str);
                    }
                }
            }
        }

        tracing::info!("Database schema initialized");
        Ok(())
    }

    // ============================================
    // DOMAIN OPERATIONS
    // ============================================

    /// Get domain by ID
    pub fn get_domain(&self, id: Uuid) -> anyhow::Result<Option<InformationDomain>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, domain_type, name, description, status,
                   organization_id, owner_user_id, parent_domain_id,
                   metadata, created_at, updated_at
            FROM information_domains
            WHERE id = ?
            "#,
        )?;

        let result = stmt.query_row([id.to_string()], |row| {
            Ok(InformationDomain {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                domain_type: parse_domain_type(&row.get::<_, String>(1)?),
                name: row.get(2)?,
                description: row.get(3)?,
                status: parse_domain_status(&row.get::<_, String>(4)?),
                organization_id: Uuid::parse_str(&row.get::<_, String>(5)?).unwrap(),
                owner_user_id: row
                    .get::<_, Option<String>>(6)?
                    .map(|s| Uuid::parse_str(&s).unwrap()),
                parent_domain_id: row
                    .get::<_, Option<String>>(7)?
                    .map(|s| Uuid::parse_str(&s).unwrap()),
                metadata: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default(),
                created_at: parse_datetime(&row.get::<_, String>(9)?),
                updated_at: parse_datetime(&row.get::<_, String>(10)?),
            })
        });

        match result {
            Ok(domain) => Ok(Some(domain)),
            Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List all domains with optional filter
    pub fn list_domains(
        &self,
        domain_type: Option<DomainType>,
        status: Option<DomainStatus>,
        limit: i32,
        offset: i32,
    ) -> anyhow::Result<Vec<InformationDomain>> {
        let conn = self.conn.lock().unwrap();

        let mut sql = String::from(
            r#"
            SELECT id, domain_type, name, description, status,
                   organization_id, owner_user_id, parent_domain_id,
                   metadata, created_at, updated_at
            FROM information_domains
            WHERE 1=1
            "#,
        );

        if domain_type.is_some() {
            sql.push_str(" AND domain_type = ?");
        }
        if status.is_some() {
            sql.push_str(" AND status = ?");
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

        let mut stmt = conn.prepare(&sql)?;

        // Note: This is a simplified version that ignores the filters.
        // In production, use proper parameter binding with the domain_type and status filters.
        let _ = (domain_type, status); // Acknowledge filters are not yet used
        let mut domains = Vec::new();

        let rows = stmt.query_map(
            params![limit, offset],
            |row| -> DuckResult<InformationDomain> {
                Ok(InformationDomain {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    domain_type: parse_domain_type(&row.get::<_, String>(1)?),
                    name: row.get(2)?,
                    description: row.get(3)?,
                    status: parse_domain_status(&row.get::<_, String>(4)?),
                    organization_id: Uuid::parse_str(&row.get::<_, String>(5)?).unwrap(),
                    owner_user_id: row
                        .get::<_, Option<String>>(6)?
                        .map(|s| Uuid::parse_str(&s).unwrap()),
                    parent_domain_id: row
                        .get::<_, Option<String>>(7)?
                        .map(|s| Uuid::parse_str(&s).unwrap()),
                    metadata: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default(),
                    created_at: parse_datetime(&row.get::<_, String>(9)?),
                    updated_at: parse_datetime(&row.get::<_, String>(10)?),
                })
            },
        )?;

        for row in rows {
            domains.push(row?);
        }

        Ok(domains)
    }

    /// Create a new domain
    pub fn create_domain(&self, domain: &InformationDomain) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            INSERT INTO information_domains
                (id, domain_type, name, description, status, organization_id,
                 owner_user_id, parent_domain_id, metadata, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                domain.id.to_string(),
                domain.domain_type.to_string().to_lowercase(),
                domain.name,
                domain.description,
                domain.status.to_string().to_lowercase(),
                domain.organization_id.to_string(),
                domain.owner_user_id.map(|u| u.to_string()),
                domain.parent_domain_id.map(|u| u.to_string()),
                serde_json::to_string(&domain.metadata)?,
                datetime_to_string(&domain.created_at),
                datetime_to_string(&domain.updated_at),
            ],
        )?;

        Ok(())
    }

    // ============================================
    // OBJECT OPERATIONS
    // ============================================

    /// Get object by ID
    pub fn get_object(&self, id: Uuid) -> anyhow::Result<Option<InformationObject>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, domain_id, object_type, title, description, content_location,
                   content_text, mime_type, file_size, classification, retention_period,
                   is_woo_relevant, woo_publication_date, privacy_level, tags, metadata,
                   version, previous_version_id, created_by, created_at, updated_at
            FROM information_objects
            WHERE id = ?
            "#,
        )?;

        let result = stmt.query_row([id.to_string()], |row| {
            Ok(InformationObject {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                domain_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                object_type: parse_object_type(&row.get::<_, String>(2)?),
                title: row.get(3)?,
                description: row.get(4)?,
                content_location: row.get(5)?,
                content_text: row.get(6)?,
                mime_type: row.get(7)?,
                file_size: row.get(8)?,
                classification: iou_core::compliance::Classification::Intern, // TODO: parse
                retention_period: row.get(10)?,
                is_woo_relevant: row.get(11)?,
                woo_publication_date: parse_optional_datetime(row.get(12)?),
                privacy_level: iou_core::compliance::PrivacyLevel::default(),
                tags: vec![], // TODO: parse array
                metadata: serde_json::Value::Object(serde_json::Map::new()),
                version: row.get(16)?,
                previous_version_id: row
                    .get::<_, Option<String>>(17)?
                    .map(|s| Uuid::parse_str(&s).unwrap()),
                created_by: Uuid::parse_str(&row.get::<_, String>(18)?).unwrap(),
                created_at: parse_datetime(&row.get::<_, String>(19)?),
                updated_at: parse_datetime(&row.get::<_, String>(20)?),
            })
        });

        match result {
            Ok(obj) => Ok(Some(obj)),
            Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Create a new information object
    pub fn create_object(&self, object: &InformationObject) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            INSERT INTO information_objects
                (id, domain_id, object_type, title, description, content_location,
                 content_text, mime_type, file_size, classification, retention_period,
                 is_woo_relevant, privacy_level, tags, metadata, version, created_by,
                 created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                object.id.to_string(),
                object.domain_id.to_string(),
                object.object_type.to_string().to_lowercase(),
                object.title,
                object.description,
                object.content_location,
                object.content_text,
                object.mime_type,
                object.file_size,
                object.classification.to_string().to_lowercase(),
                object.retention_period,
                object.is_woo_relevant,
                object.privacy_level.to_string().to_lowercase(),
                serde_json::to_string(&object.tags)?,
                serde_json::to_string(&object.metadata)?,
                object.version,
                object.created_by.to_string(),
                datetime_to_string(&object.created_at),
                datetime_to_string(&object.updated_at),
            ],
        )?;

        Ok(())
    }

    // ============================================
    // SEARCH OPERATIONS
    // ============================================

    /// Full-text search using DuckDB's string functions
    pub fn search(&self, query: &str, limit: i32) -> anyhow::Result<Vec<SearchResult>> {
        let conn = self.conn.lock().unwrap();

        // DuckDB doesn't have built-in FTS like PostgreSQL,
        // so we use ILIKE for simple search. For production,
        // consider using the FTS extension or external search.
        let search_pattern = format!("%{}%", query.to_lowercase());

        let mut stmt = conn.prepare(
            r#"
            SELECT
                io.id,
                io.object_type,
                io.title,
                COALESCE(io.description, '') as snippet,
                io.domain_id,
                id.name as domain_name,
                io.classification,
                io.created_at
            FROM v_searchable_objects io
            JOIN information_domains id ON io.domain_id = id.id
            WHERE LOWER(io.searchable_text) LIKE ?
            ORDER BY io.created_at DESC
            LIMIT ?
            "#,
        )?;

        let mut results = Vec::new();
        let rows = stmt.query_map(params![search_pattern, limit], |row| {
            Ok(SearchResult {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                object_type: row.get(1)?,
                title: row.get(2)?,
                snippet: row.get(3)?,
                domain_id: Uuid::parse_str(&row.get::<_, String>(4)?).unwrap(),
                domain_name: row.get(5)?,
                classification: row.get(6)?,
                created_at: parse_datetime(&row.get::<_, String>(7)?),
            })
        })?;

        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    // ============================================
    // ANALYTICS (DuckDB's strength!)
    // ============================================

    /// Get compliance overview using DuckDB's analytical capabilities
    #[allow(dead_code)]
    pub fn get_compliance_overview(&self) -> anyhow::Result<Vec<ComplianceOverview>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT * FROM v_compliance_overview
            ORDER BY total_objects DESC
            "#,
        )?;

        let mut results = Vec::new();
        let rows = stmt.query_map([], |row| {
            Ok(ComplianceOverview {
                domain_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                domain_name: row.get(1)?,
                domain_type: row.get(2)?,
                total_objects: row.get(3)?,
                woo_relevant_count: row.get(4)?,
                public_count: row.get(5)?,
                missing_retention: row.get(6)?,
                avg_retention_years: row.get(7)?,
            })
        })?;

        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }
}

// Helper functions for parsing enums
fn parse_domain_type(s: &str) -> DomainType {
    match s.to_lowercase().as_str() {
        "zaak" => DomainType::Zaak,
        "project" => DomainType::Project,
        "beleid" => DomainType::Beleid,
        "expertise" => DomainType::Expertise,
        _ => DomainType::Zaak,
    }
}

fn parse_domain_status(s: &str) -> DomainStatus {
    match s.to_lowercase().as_str() {
        "concept" => DomainStatus::Concept,
        "actief" => DomainStatus::Actief,
        "afgerond" => DomainStatus::Afgerond,
        "gearchiveerd" => DomainStatus::Gearchiveerd,
        _ => DomainStatus::Actief,
    }
}

fn parse_object_type(s: &str) -> ObjectType {
    match s.to_lowercase().as_str() {
        "document" => ObjectType::Document,
        "email" => ObjectType::Email,
        "chat" => ObjectType::Chat,
        "besluit" => ObjectType::Besluit,
        "data" => ObjectType::Data,
        _ => ObjectType::Document,
    }
}

// Search result struct
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub object_type: String,
    pub title: String,
    pub snippet: String,
    pub domain_id: Uuid,
    pub domain_name: String,
    pub classification: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// Compliance overview struct
#[derive(Debug, Clone, serde::Serialize)]
#[allow(dead_code)]
pub struct ComplianceOverview {
    pub domain_id: Uuid,
    pub domain_name: String,
    pub domain_type: String,
    pub total_objects: i64,
    pub woo_relevant_count: i64,
    pub public_count: i64,
    pub missing_retention: i64,
    pub avg_retention_years: Option<f64>,
}
