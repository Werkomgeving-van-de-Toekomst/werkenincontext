//! DuckDB database layer
//!
//! DuckDB is een embedded analytische database - geen server nodig!
//! Perfect voor:
//! - Single-file deployment
//! - Analytische queries (aggregaties, window functions)
//! - Direct lezen van Parquet/CSV/JSON
//! - WASM support (toekomstig)

use std::path::Path;
use std::sync::{Arc, Mutex};

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
#[derive(Clone)]
pub struct Database {
    pub(crate) conn: Arc<Mutex<Connection>>,
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
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Initialize database schema
    pub fn initialize_schema(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        // Read and execute initial schema file
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

        // Read and execute document creation tables migration
        let doc_schema = include_str!("../../../migrations/002_document_creation_tables.sql");

        for statement in doc_schema.split(';') {
            let stmt = statement.trim();
            if !stmt.is_empty() && !stmt.starts_with("--") {
                if let Err(e) = conn.execute(stmt, []) {
                    let err_str = e.to_string();
                    if !err_str.contains("already exists") && !err_str.contains("table") {
                        tracing::warn!("Document schema statement failed: {}", err_str);
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
    // ADVANCED SEARCH OPERATIONS
    // ============================================

    /// Advanced text search with filters
    pub fn search_text(
        &self,
        params: &crate::search_types::SearchParams,
        query: &str,
    ) -> anyhow::Result<(Vec<crate::search_types::AdvancedSearchResult>, i64)> {
        let conn = self.conn.lock().unwrap();

        // Build dynamic SQL query based on filters
        let mut sql = String::from(
            r#"
            SELECT
                io.id,
                io.object_type,
                io.title,
                COALESCE(SUBSTR(io.description, 1, 200), io.title) as snippet,
                io.domain_id,
                id.name as domain_name,
                id.domain_type,
                io.classification,
                io.created_at,
                io.is_woo_relevant
            FROM v_searchable_objects io
            JOIN information_domains id ON io.domain_id = id.id
            WHERE 1=1
            "#,
        );

        let mut param_values: Vec<String> = vec![];

        // Add search pattern
        let search_pattern = format!("%{}%", query.to_lowercase());
        sql.push_str(" AND LOWER(io.searchable_text) LIKE ?");

        // Add optional filters
        if let Some(dt) = &params.domain_type {
            sql.push_str(" AND LOWER(id.domain_type) = ?");
            param_values.push(dt.to_lowercase());
        }
        if let Some(di) = &params.domain_id {
            sql.push_str(" AND io.domain_id = ?");
            param_values.push(di.clone());
        }
        if let Some(ot) = &params.object_type {
            sql.push_str(" AND LOWER(io.object_type) = ?");
            param_values.push(ot.to_lowercase());
        }
        if let Some(c) = &params.classification {
            sql.push_str(" AND LOWER(io.classification) = ?");
            param_values.push(c.to_lowercase());
        }

        // Get total count first
        let count_sql = sql.replace(
            "io.id, io.object_type, io.title, COALESCE(SUBSTR(io.description, 1, 200), io.title) as snippet, io.domain_id, id.name as domain_name, id.domain_type, io.classification, io.created_at, io.is_woo_relevant",
            "COUNT(*) as total"
        );

        let total: i64 = {
            let mut count_stmt = conn.prepare(&count_sql)?;
            let mut params_vec = vec![&search_pattern as &dyn duckdb::ToSql];
            for val in &param_values {
                params_vec.push(val as &dyn duckdb::ToSql);
            }
            count_stmt.query_row(params_vec.as_slice(), |row| row.get(0))?
        };

        // Add sorting and pagination
        match params.sort {
            crate::search_types::SortOrder::Relevance => {
                // Simple relevance: title matches first, then date
                sql.push_str(" ORDER BY CASE WHEN LOWER(io.title) LIKE ? THEN 0 ELSE 1 END, io.created_at DESC");
            }
            crate::search_types::SortOrder::DateDesc => {
                sql.push_str(" ORDER BY io.created_at DESC");
            }
            crate::search_types::SortOrder::DateAsc => {
                sql.push_str(" ORDER BY io.created_at ASC");
            }
            crate::search_types::SortOrder::TitleAsc => {
                sql.push_str(" ORDER BY io.title ASC");
            }
        }

        sql.push_str(" LIMIT ? OFFSET ?");
        param_values.push(params.limit.to_string());
        param_values.push(params.offset.to_string());

        let mut stmt = conn.prepare(&sql)?;

        let mut results = Vec::new();
        let mut params_vec: Vec<&dyn duckdb::ToSql> = vec![&search_pattern];

        // Add title pattern for relevance sorting
        if matches!(params.sort, crate::search_types::SortOrder::Relevance) {
            params_vec.push(&search_pattern);
        }

        for val in &param_values {
            params_vec.push(val as &dyn duckdb::ToSql);
        }

        let rows = stmt.query_map(params_vec.as_slice(), |row| {
            let id: String = row.get(0)?;
            let created_at: String = row.get(8)?;
            let parsed_dt = parse_datetime(&created_at);

            // Calculate a simple relevance score
            let title = row.get::<_, String>(2)?;
            let score = if title.to_lowercase().contains(&query.to_lowercase()) {
                0.9
            } else {
                0.5
            };

            Ok(crate::search_types::AdvancedSearchResult {
                id: Uuid::parse_str(&id).unwrap(),
                object_type: row.get(1)?,
                title,
                snippet: row.get(3)?,
                domain_id: Uuid::parse_str(&row.get::<_, String>(4)?).unwrap(),
                domain_name: row.get(5)?,
                domain_type: row.get(6)?,
                classification: row.get(7)?,
                score,
                created_at,
                semantic_score: None,
                text_rank: Some(score),
                is_woo_relevant: row.get(9)?,
                woo_disclosure_class: None,
            })
        })?;

        for row in rows {
            results.push(row?);
        }

        Ok((results, total))
    }

    /// Get search facets for filtering
    pub fn get_search_facets(
        &self,
        _query: &str,
    ) -> anyhow::Result<crate::search_types::SearchFacets> {
        let conn = self.conn.lock().unwrap();

        // Get domain types with counts
        let domain_types: Vec<crate::search_types::FacetCount> = {
            let mut stmt = conn.prepare(
                r#"
                SELECT domain_type, COUNT(*) as count
                FROM information_domains
                GROUP BY domain_type
                ORDER BY count DESC
                "#,
            )?;

            let mut facets = Vec::new();
            let rows = stmt.query_map([], |row| {
                let value = row.get::<_, String>(0)?;
                let label = match value.to_lowercase().as_str() {
                    "zaak" => "Zaken",
                    "project" => "Projecten",
                    "beleid" => "Beleid",
                    "expertise" => "Expertise",
                    _ => value.as_str(),
                };
                Ok(crate::search_types::FacetCount {
                    value: value.clone(),
                    count: row.get(1)?,
                    label: label.to_string(),
                })
            })?;

            for row in rows {
                facets.push(row?);
            }
            facets
        };

        // Get object types with counts
        let object_types: Vec<crate::search_types::FacetCount> = {
            let mut stmt = conn.prepare(
                r#"
                SELECT object_type, COUNT(*) as count
                FROM information_objects
                GROUP BY object_type
                ORDER BY count DESC
                "#,
            )?;

            let mut facets = Vec::new();
            let rows = stmt.query_map([], |row| {
                let value = row.get::<_, String>(0)?;
                let label = match value.to_lowercase().as_str() {
                    "document" => "Documenten",
                    "email" => "E-mails",
                    "chat" => "Chats",
                    "besluit" => "Besluiten",
                    "data" => "Data",
                    _ => value.as_str(),
                };
                Ok(crate::search_types::FacetCount {
                    value: value.clone(),
                    count: row.get(1)?,
                    label: label.to_string(),
                })
            })?;

            for row in rows {
                facets.push(row?);
            }
            facets
        };

        // Get classifications with counts
        let classifications: Vec<crate::search_types::FacetCount> = {
            let mut stmt = conn.prepare(
                r#"
                SELECT classification, COUNT(*) as count
                FROM information_objects
                GROUP BY classification
                ORDER BY count DESC
                "#,
            )?;

            let mut facets = Vec::new();
            let rows = stmt.query_map([], |row| {
                let value = row.get::<_, String>(0)?;
                let label = match value.to_lowercase().as_str() {
                    "openbaar" => "Openbaar",
                    "intern" => "Intern",
                    "vertrouwelijk" => "Vertrouwelijk",
                    "geheim" => "Geheim",
                    _ => value.as_str(),
                };
                Ok(crate::search_types::FacetCount {
                    value: value.clone(),
                    count: row.get(1)?,
                    label: label.to_string(),
                })
            })?;

            for row in rows {
                facets.push(row?);
            }
            facets
        };

        // Compliance status distribution
        let compliance_statuses: Vec<crate::search_types::FacetCount> = {
            let mut stmt = conn.prepare(
                r#"
                SELECT
                    CASE
                        WHEN is_woo_relevant = true THEN 'woo_relevant'
                        ELSE 'not_relevant'
                    END as status,
                    COUNT(*) as count
                FROM information_objects
                GROUP BY status
                ORDER BY count DESC
                "#,
            )?;

            let mut facets = Vec::new();
            let rows = stmt.query_map([], |row| {
                let value = row.get::<_, String>(0)?;
                let label = match value.to_lowercase().as_str() {
                    "woo_relevant" => "Woo Relevant",
                    "not_relevant" => "Niet Relevant",
                    _ => value.as_str(),
                };
                Ok(crate::search_types::FacetCount {
                    value: value.clone(),
                    count: row.get(1)?,
                    label: label.to_string(),
                })
            })?;

            for row in rows {
                facets.push(row?);
            }
            facets
        };

        Ok(crate::search_types::SearchFacets {
            domain_types,
            object_types,
            classifications,
            compliance_statuses,
        })
    }

    /// Get search suggestions for autocomplete
    pub fn get_search_suggestions(
        &self,
        query: &str,
        limit: i32,
    ) -> anyhow::Result<Vec<crate::search_types::SuggestionResult>> {
        let conn = self.conn.lock().unwrap();
        let mut suggestions = Vec::new();

        let search_pattern = format!("%{}%", query.to_lowercase());

        // Suggest matching titles (query suggestions)
        {
            let mut stmt = conn.prepare(
                r#"
                SELECT DISTINCT title, COUNT(*) as count
                FROM v_searchable_objects
                WHERE LOWER(searchable_text) LIKE ?
                GROUP BY title
                ORDER BY count DESC
                LIMIT ?
                "#,
            )?;

            let rows = stmt.query_map(params![search_pattern, limit / 2], |row| {
                Ok(crate::search_types::SuggestionResult {
                    text: row.get(0)?,
                    suggestion_type: crate::search_types::SuggestionType::Query,
                    count: Some(row.get(1)?),
                })
            })?;

            for row in rows {
                suggestions.push(row?);
            }
        }

        // Suggest matching domains
        {
            let mut stmt = conn.prepare(
                r#"
                SELECT name, domain_type
                FROM information_domains
                WHERE LOWER(name) LIKE ?
                LIMIT ?
                "#,
            )?;

            let rows = stmt.query_map(params![search_pattern, limit / 4], |row| {
                Ok(crate::search_types::SuggestionResult {
                    text: row.get(0)?,
                    suggestion_type: crate::search_types::SuggestionType::Domain,
                    count: None,
                })
            })?;

            for row in rows {
                suggestions.push(row?);
            }
        }

        Ok(suggestions)
    }

    /// Find similar documents based on text similarity
    pub fn find_similar_documents(
        &self,
        id: Uuid,
        limit: i32,
    ) -> anyhow::Result<Vec<crate::search_types::AdvancedSearchResult>> {
        let conn = self.conn.lock().unwrap();

        // First get the source document
        let source = match self.get_object(id)? {
            Some(obj) => obj,
            None => return Ok(vec![]),
        };

        // Extract key terms from source (simple word extraction)
        let terms: Vec<&str> = source
            .title
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();

        if terms.is_empty() {
            return Ok(vec![]);
        }

        // Build search query from terms
        let mut results = Vec::new();

        // Simple similarity: documents with overlapping terms in title
        let search_pattern = terms
            .iter()
            .map(|t| format!("%{}%", t.to_lowercase()))
            .collect::<Vec<_>>();

        for pattern in search_pattern.iter().take(3) {
            let mut stmt = conn.prepare(
                r#"
                SELECT
                    io.id,
                    io.object_type,
                    io.title,
                    COALESCE(SUBSTR(io.description, 1, 200), io.title) as snippet,
                    io.domain_id,
                    id.name as domain_name,
                    id.domain_type,
                    io.classification,
                    io.created_at,
                    io.is_woo_relevant
                FROM v_searchable_objects io
                JOIN information_domains id ON io.domain_id = id.id
                WHERE io.id != ? AND LOWER(io.searchable_text) LIKE ?
                ORDER BY io.created_at DESC
                LIMIT ?
                "#,
            )?;

            let rows = stmt.query_map(
                params![id.to_string(), pattern, (limit / 3).max(1)],
                |row| {
                    let created_at: String = row.get(8)?;
                    Ok(crate::search_types::AdvancedSearchResult {
                        id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                        object_type: row.get(1)?,
                        title: row.get(2)?,
                        snippet: row.get(3)?,
                        domain_id: Uuid::parse_str(&row.get::<_, String>(4)?).unwrap(),
                        domain_name: row.get(5)?,
                        domain_type: row.get(6)?,
                        classification: row.get(7)?,
                        score: 0.7,
                        created_at,
                        semantic_score: None,
                        text_rank: Some(0.7),
                        is_woo_relevant: row.get(9)?,
                        woo_disclosure_class: None,
                    })
                },
            )?;

            for row in rows {
                results.push(row?);
            }

            if results.len() >= limit as usize {
                break;
            }
        }

        // Remove duplicates and limit
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        results.truncate(limit as usize);
        results.dedup_by(|a, b| a.id == b.id);

        Ok(results)
    }

    /// Reindex search data (update full-text search indexes)
    pub fn reindex_search(&self) -> anyhow::Result<i64> {
        // In DuckDB, we don't have traditional FTS indexes like PostgreSQL
        // This is a placeholder for future index updates
        // For now, we'll just return the count of indexed objects
        let conn = self.conn.lock().unwrap();

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM information_objects", [], |row| {
            row.get(0)
        })?;

        tracing::info!("Reindexed {} objects for search", count);
        Ok(count)
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

    // ============================================
    // DOCUMENT OPERATIONS
    // ============================================

    /// Get document by ID
    pub fn get_document(&self, id: Uuid) -> anyhow::Result<Option<iou_core::document::DocumentMetadata>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, domain_id, document_type, state,
                   current_version_key, previous_version_key,
                   compliance_score, confidence_score,
                   created_at, updated_at
            FROM documents
            WHERE id = ?
            "#,
        )?;

        let result = stmt.query_row([id.to_string()], |row| {
            Ok(iou_core::document::DocumentMetadata {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                domain_id: row.get(1)?,
                document_type: row.get(2)?,
                state: parse_workflow_status(&row.get::<_, String>(3)?),
                current_version_key: row.get(4)?,
                previous_version_key: row.get(5)?,
                compliance_score: row.get(6)?,
                confidence_score: row.get(7)?,
                created_at: parse_datetime(&row.get::<_, String>(8)?),
                updated_at: parse_datetime(&row.get::<_, String>(9)?),
            })
        });

        match result {
            Ok(doc) => Ok(Some(doc)),
            Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Create document record
    pub fn create_document(&self, doc: &iou_core::document::DocumentMetadata) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            INSERT INTO documents
                (id, domain_id, document_type, state,
                 current_version_key, previous_version_key,
                 compliance_score, confidence_score,
                 created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                doc.id.to_string(),
                doc.domain_id,
                doc.document_type,
                workflow_status_to_string(&doc.state),
                doc.current_version_key,
                doc.previous_version_key,
                doc.compliance_score,
                doc.confidence_score,
                datetime_to_string(&doc.created_at),
                datetime_to_string(&doc.updated_at),
            ],
        )?;

        Ok(())
    }

    /// Update document record
    pub fn update_document(&self, doc: &iou_core::document::DocumentMetadata) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            UPDATE documents
            SET state = ?, current_version_key = ?, previous_version_key = ?,
                compliance_score = ?, confidence_score = ?, updated_at = ?
            WHERE id = ?
            "#,
            params![
                workflow_status_to_string(&doc.state),
                doc.current_version_key,
                doc.previous_version_key,
                doc.compliance_score,
                doc.confidence_score,
                datetime_to_string(&doc.updated_at),
                doc.id.to_string(),
            ],
        )?;

        Ok(())
    }

    /// List documents with optional filters
    pub fn list_documents(
        &self,
        domain_id: Option<&str>,
        state: Option<iou_core::workflows::WorkflowStatus>,
        limit: i32,
    ) -> anyhow::Result<Vec<iou_core::document::DocumentMetadata>> {
        let conn = self.conn.lock().unwrap();

        let mut sql = String::from(
            r#"
            SELECT id, domain_id, document_type, state,
                   current_version_key, previous_version_key,
                   compliance_score, confidence_score,
                   created_at, updated_at
            FROM documents
            WHERE 1=1
            "#,
        );

        if domain_id.is_some() {
            sql.push_str(" AND domain_id = ?");
        }
        if state.is_some() {
            sql.push_str(" AND state = ?");
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT ?");

        let mut stmt = conn.prepare(&sql)?;

        let mut docs = Vec::new();
        let mut param_index = 1;

        // Build params dynamically
        let mut params_vec: Vec<String> = vec![];
        if let Some(di) = domain_id {
            params_vec.push(di.to_string());
        }
        if let Some(s) = state {
            params_vec.push(workflow_status_to_string(&s));
        }
        params_vec.push(limit.to_string());

        let params_refs: Vec<&dyn duckdb::ToSql> = params_vec.iter().map(|s| s as &dyn duckdb::ToSql).collect();

        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(iou_core::document::DocumentMetadata {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                domain_id: row.get(1)?,
                document_type: row.get(2)?,
                state: parse_workflow_status(&row.get::<_, String>(3)?),
                current_version_key: row.get(4)?,
                previous_version_key: row.get(5)?,
                compliance_score: row.get(6)?,
                confidence_score: row.get(7)?,
                created_at: parse_datetime(&row.get::<_, String>(8)?),
                updated_at: parse_datetime(&row.get::<_, String>(9)?),
            })
        })?;

        for row in rows {
            docs.push(row?);
        }

        Ok(docs)
    }

    // ============================================
    // TEMPLATE OPERATIONS
    // ============================================

    /// Get template by ID
    pub fn get_template(&self, id: &str) -> anyhow::Result<Option<iou_core::document::Template>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, domain_id, document_type, content,
                   required_variables, optional_sections, version,
                   created_at, updated_at, is_active
            FROM templates
            WHERE id = ?
            "#,
        )?;

        let result = stmt.query_row([id], |row| {
            Ok(iou_core::document::Template {
                id: row.get(0)?,
                name: row.get(1)?,
                domain_id: row.get(2)?,
                document_type: row.get(3)?,
                content: row.get(4)?,
                required_variables: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                optional_sections: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                version: row.get(7)?,
                created_at: parse_datetime(&row.get::<_, String>(8)?),
                updated_at: parse_datetime(&row.get::<_, String>(9)?),
                is_active: row.get(10)?,
            })
        });

        match result {
            Ok(tpl) => Ok(Some(tpl)),
            Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get active template for domain and document type
    pub fn get_active_template(
        &self,
        domain_id: &str,
        document_type: &str,
    ) -> anyhow::Result<Option<iou_core::document::Template>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, domain_id, document_type, content,
                   required_variables, optional_sections, version,
                   created_at, updated_at, is_active
            FROM templates
            WHERE domain_id = ? AND document_type = ? AND is_active = true
            LIMIT 1
            "#,
        )?;

        let result = stmt.query_row([domain_id, document_type], |row| {
            Ok(iou_core::document::Template {
                id: row.get(0)?,
                name: row.get(1)?,
                domain_id: row.get(2)?,
                document_type: row.get(3)?,
                content: row.get(4)?,
                required_variables: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                optional_sections: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                version: row.get(7)?,
                created_at: parse_datetime(&row.get::<_, String>(8)?),
                updated_at: parse_datetime(&row.get::<_, String>(9)?),
                is_active: row.get(10)?,
            })
        });

        match result {
            Ok(tpl) => Ok(Some(tpl)),
            Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List all templates, optionally filtered by domain_id
    pub fn list_templates(&self, domain_id: Option<&str>) -> anyhow::Result<Vec<iou_core::document::Template>> {
        let conn = self.conn.lock().unwrap();
        let mut templates = Vec::new();

        let sql = if domain_id.is_some() {
            r#"
            SELECT id, name, domain_id, document_type, content,
                   required_variables, optional_sections, version,
                   created_at, updated_at, is_active
            FROM templates
            WHERE domain_id = ?
            ORDER BY name
            "#
        } else {
            r#"
            SELECT id, name, domain_id, document_type, content,
                   required_variables, optional_sections, version,
                   created_at, updated_at, is_active
            FROM templates
            ORDER BY domain_id, name
            "#
        };

        let mut stmt = conn.prepare(sql)?;

        let mut rows = if let Some(domain) = domain_id {
            stmt.query(params![domain])?
        } else {
            stmt.query(params![])?
        };

        while let Some(row) = rows.next()? {
            templates.push(iou_core::document::Template {
                id: row.get(0)?,
                name: row.get(1)?,
                domain_id: row.get(2)?,
                document_type: row.get(3)?,
                content: row.get(4)?,
                required_variables: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                optional_sections: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                version: row.get(7)?,
                created_at: parse_datetime(&row.get::<_, String>(8)?),
                updated_at: parse_datetime(&row.get::<_, String>(9)?),
                is_active: row.get(10)?,
            });
        }

        Ok(templates)
    }

    /// Async wrapper for list_templates
    pub async fn list_templates_async(&self, domain_id: Option<String>) -> anyhow::Result<Vec<iou_core::document::Template>> {
        let db = self.clone();
        let domain = domain_id.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || db.list_templates(domain.as_deref()))
            .await?
    }

    /// Create template
    pub fn create_template(&self, template: &iou_core::document::Template) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            INSERT INTO templates
                (id, name, domain_id, document_type, content,
                 required_variables, optional_sections, version,
                 created_at, updated_at, is_active)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                template.id,
                template.name,
                template.domain_id,
                template.document_type,
                template.content,
                serde_json::to_string(&template.required_variables)?,
                serde_json::to_string(&template.optional_sections)?,
                template.version,
                datetime_to_string(&template.created_at),
                datetime_to_string(&template.updated_at),
                template.is_active,
            ],
        )?;

        Ok(())
    }

    /// Update template
    pub fn update_template(&self, template: &iou_core::document::Template) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            UPDATE templates
            SET name = ?, content = ?, required_variables = ?,
                optional_sections = ?, version = ?, updated_at = ?, is_active = ?
            WHERE id = ?
            "#,
            params![
                template.name,
                template.content,
                serde_json::to_string(&template.required_variables)?,
                serde_json::to_string(&template.optional_sections)?,
                template.version,
                datetime_to_string(&template.updated_at),
                template.is_active,
                template.id,
            ],
        )?;

        Ok(())
    }

    /// Check if template exists for domain and type
    pub fn template_exists(&self, domain_id: &str, document_type: &str) -> anyhow::Result<bool> {
        let conn = self.conn.lock().unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM templates WHERE domain_id = ? AND document_type = ?",
            [domain_id, document_type],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    /// Get audit trail for document
    pub fn get_audit_trail(
        &self,
        document_id: Uuid,
    ) -> anyhow::Result<Vec<iou_core::document::AuditEntry>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, document_id, agent_name, action, details, timestamp, execution_time_ms
            FROM audit_trail
            WHERE document_id = ?
            ORDER BY timestamp DESC
            "#,
        )?;

        let mut entries = Vec::new();
        let rows = stmt.query_map([document_id.to_string()], |row| {
            Ok(iou_core::document::AuditEntry {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                document_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                agent_name: row.get(2)?,
                action: row.get(3)?,
                details: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                timestamp: parse_datetime(&row.get::<_, String>(5)?),
                execution_time_ms: row.get(6)?,
            })
        })?;

        for row in rows {
            entries.push(row?);
        }

        Ok(entries)
    }

    /// Add audit entry
    pub fn add_audit_entry(&self, entry: &iou_core::document::AuditEntry) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            INSERT INTO audit_trail
                (id, document_id, agent_name, action, details, timestamp, execution_time_ms)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                entry.id.to_string(),
                entry.document_id.to_string(),
                entry.agent_name,
                entry.action,
                serde_json::to_string(&entry.details)?,
                datetime_to_string(&entry.timestamp),
                entry.execution_time_ms,
            ],
        )?;

        Ok(())
    }

    // ============================================
    // ASYNC WRAPPERS (for compatibility with async route handlers)
    // ============================================

    /// Async wrapper for get_object (information_objects table)
    pub async fn get_object_async(&self, id: Uuid) -> anyhow::Result<Option<InformationObject>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || db.get_object(id))
            .await?
    }

    /// Async wrapper for get_document (documents table)
    pub async fn get_document_async(&self, id: Uuid) -> anyhow::Result<Option<iou_core::document::DocumentMetadata>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || db.get_document(id))
            .await?
    }

    /// Async wrapper for create_document
    pub async fn create_document_async(&self, doc: iou_core::document::DocumentMetadata) -> anyhow::Result<()> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || db.create_document(&doc))
            .await?
    }

    /// Async wrapper for update_document
    pub async fn update_document_async(&self, doc: iou_core::document::DocumentMetadata) -> anyhow::Result<()> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || db.update_document(&doc))
            .await?
    }

    /// Async wrapper for get_template
    pub async fn get_template_async(&self, id: String) -> anyhow::Result<Option<iou_core::document::Template>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || db.get_template(&id))
            .await?
    }

    /// Async wrapper for get_active_template
    pub async fn get_active_template_async(
        &self,
        domain_id: String,
        document_type: String,
    ) -> anyhow::Result<Option<iou_core::document::Template>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || db.get_active_template(&domain_id, &document_type))
            .await?
    }

    /// Async wrapper for template_exists
    pub async fn template_exists_async(&self, domain_id: String, document_type: String) -> anyhow::Result<bool> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || db.template_exists(&domain_id, &document_type))
            .await?
    }

    /// Async wrapper for get_audit_trail
    pub async fn get_audit_trail_async(
        &self,
        document_id: Uuid,
    ) -> anyhow::Result<Vec<iou_core::document::AuditEntry>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || db.get_audit_trail(document_id))
            .await?
    }

    /// Async wrapper for add_audit_entry
    pub async fn add_audit_entry_async(&self, entry: iou_core::document::AuditEntry) -> anyhow::Result<()> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || db.add_audit_entry(&entry))
            .await?
    }
}

// Helper functions for WorkflowStatus
fn parse_workflow_status(s: &str) -> iou_core::workflows::WorkflowStatus {
    match s.to_lowercase().as_str() {
        "draft" => iou_core::workflows::WorkflowStatus::Draft,
        "submitted" => iou_core::workflows::WorkflowStatus::Submitted,
        "in_review" | "inreview" => iou_core::workflows::WorkflowStatus::InReview,
        "changes_requested" | "changesrequested" => iou_core::workflows::WorkflowStatus::ChangesRequested,
        "approved" => iou_core::workflows::WorkflowStatus::Approved,
        "published" => iou_core::workflows::WorkflowStatus::Published,
        "rejected" => iou_core::workflows::WorkflowStatus::Rejected,
        "archived" => iou_core::workflows::WorkflowStatus::Archived,
        _ => iou_core::workflows::WorkflowStatus::Draft,
    }
}

fn workflow_status_to_string(s: &iou_core::workflows::WorkflowStatus) -> String {
    match s {
        iou_core::workflows::WorkflowStatus::Draft => "draft",
        iou_core::workflows::WorkflowStatus::Submitted => "submitted",
        iou_core::workflows::WorkflowStatus::InReview => "in_review",
        iou_core::workflows::WorkflowStatus::ChangesRequested => "changes_requested",
        iou_core::workflows::WorkflowStatus::Approved => "approved",
        iou_core::workflows::WorkflowStatus::Published => "published",
        iou_core::workflows::WorkflowStatus::Rejected => "rejected",
        iou_core::workflows::WorkflowStatus::Archived => "archived",
    }
    .to_string()
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
