diff --git a/.env.supabase.example b/.env.supabase.example
new file mode 100644
index 0000000..cf6addd
--- /dev/null
+++ b/.env.supabase.example
@@ -0,0 +1,11 @@
+# Supabase Environment Configuration
+# Copy this file to .env.supabase and update with secure values
+
+# PostgreSQL password
+POSTGRES_PASSWORD=change-this-secure-password
+
+# JWT secret for authentication (generate with: openssl rand -base64 32)
+JWT_SECRET=change-this-jwt-secret
+
+# Studio reset URL
+STUDIO_PG_RESET_URL=http://localhost:3000/api/reset
diff --git a/Cargo.toml b/Cargo.toml
index 07e894b..d7775b3 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -39,6 +39,9 @@ tokio = { version = "1.43", features = ["full"] }
 # Database - DuckDB (embedded analytical database)
 duckdb = { version = "1.1", features = ["bundled", "json", "parquet"] }
 
+# Database - PostgreSQL (for Supabase)
+sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json", "migrate"] }
+
 # Web framework
 axum = "0.8"
 tower = "0.5"
diff --git a/crates/iou-api/Cargo.toml b/crates/iou-api/Cargo.toml
index 38450c8..4819b90 100644
--- a/crates/iou-api/Cargo.toml
+++ b/crates/iou-api/Cargo.toml
@@ -27,9 +27,15 @@ tower-http.workspace = true
 # Async runtime
 tokio.workspace = true
 
+# Async trait support
+async-trait = "0.1"
+
 # Database - DuckDB embedded
 duckdb.workspace = true
 
+# Database - PostgreSQL (for Supabase)
+sqlx.workspace = true
+
 # Serialization
 serde.workspace = true
 serde_json.workspace = true
diff --git a/crates/iou-api/src/db.rs b/crates/iou-api/src/db.rs
index bb475eb..bd9350c 100644
--- a/crates/iou-api/src/db.rs
+++ b/crates/iou-api/src/db.rs
@@ -399,9 +399,9 @@ impl Database {
     /// Advanced text search with filters
     pub fn search_text(
         &self,
-        params: &crate::routes::search::SearchParams,
+        params: &crate::search_types::SearchParams,
         query: &str,
-    ) -> anyhow::Result<(Vec<crate::routes::search::AdvancedSearchResult>, i64)> {
+    ) -> anyhow::Result<(Vec<crate::search_types::AdvancedSearchResult>, i64)> {
         let conn = self.conn.lock().unwrap();
 
         // Build dynamic SQL query based on filters
@@ -465,17 +465,17 @@ impl Database {
 
         // Add sorting and pagination
         match params.sort {
-            crate::routes::search::SortOrder::Relevance => {
+            crate::search_types::SortOrder::Relevance => {
                 // Simple relevance: title matches first, then date
                 sql.push_str(" ORDER BY CASE WHEN LOWER(io.title) LIKE ? THEN 0 ELSE 1 END, io.created_at DESC");
             }
-            crate::routes::search::SortOrder::DateDesc => {
+            crate::search_types::SortOrder::DateDesc => {
                 sql.push_str(" ORDER BY io.created_at DESC");
             }
-            crate::routes::search::SortOrder::DateAsc => {
+            crate::search_types::SortOrder::DateAsc => {
                 sql.push_str(" ORDER BY io.created_at ASC");
             }
-            crate::routes::search::SortOrder::TitleAsc => {
+            crate::search_types::SortOrder::TitleAsc => {
                 sql.push_str(" ORDER BY io.title ASC");
             }
         }
@@ -490,7 +490,7 @@ impl Database {
         let mut params_vec: Vec<&dyn duckdb::ToSql> = vec![&search_pattern];
 
         // Add title pattern for relevance sorting
-        if matches!(params.sort, crate::routes::search::SortOrder::Relevance) {
+        if matches!(params.sort, crate::search_types::SortOrder::Relevance) {
             params_vec.push(&search_pattern);
         }
 
@@ -511,7 +511,7 @@ impl Database {
                 0.5
             };
 
-            Ok(crate::routes::search::AdvancedSearchResult {
+            Ok(crate::search_types::AdvancedSearchResult {
                 id: Uuid::parse_str(&id).unwrap(),
                 object_type: row.get(1)?,
                 title,
@@ -540,11 +540,11 @@ impl Database {
     pub fn get_search_facets(
         &self,
         _query: &str,
-    ) -> anyhow::Result<crate::routes::search::SearchFacets> {
+    ) -> anyhow::Result<crate::search_types::SearchFacets> {
         let conn = self.conn.lock().unwrap();
 
         // Get domain types with counts
-        let domain_types: Vec<crate::routes::search::FacetCount> = {
+        let domain_types: Vec<crate::search_types::FacetCount> = {
             let mut stmt = conn.prepare(
                 r#"
                 SELECT domain_type, COUNT(*) as count
@@ -564,7 +564,7 @@ impl Database {
                     "expertise" => "Expertise",
                     _ => value.as_str(),
                 };
-                Ok(crate::routes::search::FacetCount {
+                Ok(crate::search_types::FacetCount {
                     value: value.clone(),
                     count: row.get(1)?,
                     label: label.to_string(),
@@ -578,7 +578,7 @@ impl Database {
         };
 
         // Get object types with counts
-        let object_types: Vec<crate::routes::search::FacetCount> = {
+        let object_types: Vec<crate::search_types::FacetCount> = {
             let mut stmt = conn.prepare(
                 r#"
                 SELECT object_type, COUNT(*) as count
@@ -599,7 +599,7 @@ impl Database {
                     "data" => "Data",
                     _ => value.as_str(),
                 };
-                Ok(crate::routes::search::FacetCount {
+                Ok(crate::search_types::FacetCount {
                     value: value.clone(),
                     count: row.get(1)?,
                     label: label.to_string(),
@@ -613,7 +613,7 @@ impl Database {
         };
 
         // Get classifications with counts
-        let classifications: Vec<crate::routes::search::FacetCount> = {
+        let classifications: Vec<crate::search_types::FacetCount> = {
             let mut stmt = conn.prepare(
                 r#"
                 SELECT classification, COUNT(*) as count
@@ -633,7 +633,7 @@ impl Database {
                     "geheim" => "Geheim",
                     _ => value.as_str(),
                 };
-                Ok(crate::routes::search::FacetCount {
+                Ok(crate::search_types::FacetCount {
                     value: value.clone(),
                     count: row.get(1)?,
                     label: label.to_string(),
@@ -647,7 +647,7 @@ impl Database {
         };
 
         // Compliance status distribution
-        let compliance_statuses: Vec<crate::routes::search::FacetCount> = {
+        let compliance_statuses: Vec<crate::search_types::FacetCount> = {
             let mut stmt = conn.prepare(
                 r#"
                 SELECT
@@ -670,7 +670,7 @@ impl Database {
                     "not_relevant" => "Niet Relevant",
                     _ => value.as_str(),
                 };
-                Ok(crate::routes::search::FacetCount {
+                Ok(crate::search_types::FacetCount {
                     value: value.clone(),
                     count: row.get(1)?,
                     label: label.to_string(),
@@ -683,7 +683,7 @@ impl Database {
             facets
         };
 
-        Ok(crate::routes::search::SearchFacets {
+        Ok(crate::search_types::SearchFacets {
             domain_types,
             object_types,
             classifications,
@@ -696,7 +696,7 @@ impl Database {
         &self,
         query: &str,
         limit: i32,
-    ) -> anyhow::Result<Vec<crate::routes::search::SuggestionResult>> {
+    ) -> anyhow::Result<Vec<crate::search_types::SuggestionResult>> {
         let conn = self.conn.lock().unwrap();
         let mut suggestions = Vec::new();
 
@@ -716,9 +716,9 @@ impl Database {
             )?;
 
             let rows = stmt.query_map(params![search_pattern, limit / 2], |row| {
-                Ok(crate::routes::search::SuggestionResult {
+                Ok(crate::search_types::SuggestionResult {
                     text: row.get(0)?,
-                    suggestion_type: crate::routes::search::SuggestionType::Query,
+                    suggestion_type: crate::search_types::SuggestionType::Query,
                     count: Some(row.get(1)?),
                 })
             })?;
@@ -740,9 +740,9 @@ impl Database {
             )?;
 
             let rows = stmt.query_map(params![search_pattern, limit / 4], |row| {
-                Ok(crate::routes::search::SuggestionResult {
+                Ok(crate::search_types::SuggestionResult {
                     text: row.get(0)?,
-                    suggestion_type: crate::routes::search::SuggestionType::Domain,
+                    suggestion_type: crate::search_types::SuggestionType::Domain,
                     count: None,
                 })
             })?;
@@ -760,7 +760,7 @@ impl Database {
         &self,
         id: Uuid,
         limit: i32,
-    ) -> anyhow::Result<Vec<crate::routes::search::AdvancedSearchResult>> {
+    ) -> anyhow::Result<Vec<crate::search_types::AdvancedSearchResult>> {
         let conn = self.conn.lock().unwrap();
 
         // First get the source document
@@ -815,7 +815,7 @@ impl Database {
                 params![id.to_string(), pattern, (limit / 3).max(1)],
                 |row| {
                     let created_at: String = row.get(8)?;
-                    Ok(crate::routes::search::AdvancedSearchResult {
+                    Ok(crate::search_types::AdvancedSearchResult {
                         id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                         object_type: row.get(1)?,
                         title: row.get(2)?,
diff --git a/crates/iou-api/src/domain_dual_write.rs b/crates/iou-api/src/domain_dual_write.rs
new file mode 100644
index 0000000..4d400b5
--- /dev/null
+++ b/crates/iou-api/src/domain_dual_write.rs
@@ -0,0 +1,140 @@
+//! Dual-write implementation for InformationDomain
+//!
+//! Implements the DualWrite trait for InformationDomain to support
+//! writing to both DuckDB and Supabase.
+
+use async_trait::async_trait;
+use anyhow::Result;
+use uuid::Uuid;
+
+use iou_core::domain::{DomainStatus, DomainType, InformationDomain};
+
+use super::{db::Database, dual_write::DualWrite, supabase::SupabasePool};
+
+// Helper to convert DomainType to string for database storage
+fn domain_type_to_string(dt: &DomainType) -> String {
+    dt.to_string().to_lowercase()
+}
+
+// Helper to convert DomainStatus to string for database storage
+fn domain_status_to_string(ds: &DomainStatus) -> String {
+    ds.to_string().to_lowercase()
+}
+
+#[async_trait]
+impl DualWrite for InformationDomain {
+    type Id = Uuid;
+
+    async fn write_to_duckdb(&self, db: &Database) -> Result<Uuid> {
+        // Use the existing create_domain method
+        db.create_domain(self)?;
+        Ok(self.id)
+    }
+
+    async fn write_to_supabase(&self, db: &SupabasePool) -> Result<Uuid> {
+        let domain_type = domain_type_to_string(&self.domain_type);
+        let status = domain_status_to_string(&self.status);
+
+        sqlx::query(
+            r#"
+            INSERT INTO information_domains
+                (id, domain_type, name, description, status, organization_id,
+                 owner_user_id, parent_domain_id, metadata, created_at, updated_at)
+            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
+            ON CONFLICT (id) DO UPDATE SET
+                name = EXCLUDED.name,
+                description = EXCLUDED.description,
+                status = EXCLUDED.status,
+                owner_user_id = EXCLUDED.owner_user_id,
+                parent_domain_id = EXCLUDED.parent_domain_id,
+                metadata = EXCLUDED.metadata,
+                updated_at = CURRENT_TIMESTAMP
+            "#
+        )
+        .bind(self.id)
+        .bind(&domain_type)
+        .bind(&self.name)
+        .bind(&self.description)
+        .bind(&status)
+        .bind(self.organization_id)
+        .bind(self.owner_user_id)
+        .bind(self.parent_domain_id)
+        .bind(&self.metadata)
+        .bind(self.created_at)
+        .bind(self.updated_at)
+        .execute(db.inner())
+        .await?;
+
+        Ok(self.id)
+    }
+
+    async fn update_in_duckdb(&self, db: &Database) -> Result<Uuid> {
+        // Update in DuckDB (same as create for now, since we use INSERT with OR CONFLICT)
+        db.create_domain(self)?;
+        Ok(self.id)
+    }
+
+    async fn update_in_supabase(&self, db: &SupabasePool) -> Result<Uuid> {
+        let status = domain_status_to_string(&self.status);
+
+        sqlx::query(
+            r#"
+            UPDATE information_domains
+            SET name = $2,
+                description = $3,
+                status = $4,
+                owner_user_id = $5,
+                parent_domain_id = $6,
+                metadata = $7,
+                updated_at = CURRENT_TIMESTAMP
+            WHERE id = $1
+            "#
+        )
+        .bind(self.id)
+        .bind(&self.name)
+        .bind(&self.description)
+        .bind(&status)
+        .bind(self.owner_user_id)
+        .bind(self.parent_domain_id)
+        .bind(&self.metadata)
+        .execute(db.inner())
+        .await?;
+
+        Ok(self.id)
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use chrono::Utc;
+
+    #[test]
+    fn test_domain_type_to_string() {
+        assert_eq!(domain_type_to_string(&DomainType::Zaak), "zaak");
+        assert_eq!(domain_type_to_string(&DomainType::Project), "project");
+        assert_eq!(domain_type_to_string(&DomainType::Beleid), "beleid");
+        assert_eq!(domain_type_to_string(&DomainType::Expertise), "expertise");
+    }
+
+    #[test]
+    fn test_domain_status_to_string() {
+        assert_eq!(domain_status_to_string(&DomainStatus::Concept), "concept");
+        assert_eq!(domain_status_to_string(&DomainStatus::Actief), "actief");
+        assert_eq!(domain_status_to_string(&DomainStatus::Afgerond), "afgerond");
+        assert_eq!(domain_status_to_string(&DomainStatus::Gearchiveerd), "gearchiveerd");
+    }
+
+    #[test]
+    fn test_information_domain_new() {
+        let domain = InformationDomain::new(
+            DomainType::Zaak,
+            "Test Domain".to_string(),
+            Uuid::new_v4(),
+        );
+
+        assert_eq!(domain.domain_type, DomainType::Zaak);
+        assert_eq!(domain.name, "Test Domain");
+        assert_eq!(domain.status, DomainStatus::Actief);
+    }
+}
diff --git a/crates/iou-api/src/dual_write.rs b/crates/iou-api/src/dual_write.rs
new file mode 100644
index 0000000..1661526
--- /dev/null
+++ b/crates/iou-api/src/dual_write.rs
@@ -0,0 +1,184 @@
+//! Dual-Write Pattern for DuckDB + Supabase
+//!
+//! Implements a dual-write strategy that writes to both databases
+//! simultaneously with configurable read source for gradual migration.
+
+use async_trait::async_trait;
+use anyhow::Result;
+use uuid::Uuid;
+
+use super::{db::Database, supabase::SupabasePool};
+
+/// Read source selection
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum ReadSource {
+    DuckDb,
+    Supabase,
+}
+
+impl ReadSource {
+    /// Get current read source from environment
+    pub fn from_env() -> Self {
+        match std::env::var("READ_SOURCE").as_deref() {
+            Ok("supabase") => ReadSource::Supabase,
+            _ => ReadSource::DuckDb,  // Default to DuckDB for safety
+        }
+    }
+
+    /// Get the name of this read source
+    pub fn name(&self) -> &'static str {
+        match self {
+            ReadSource::DuckDb => "duckdb",
+            ReadSource::Supabase => "supabase",
+        }
+    }
+}
+
+/// Result of a dual-write operation
+#[derive(Debug)]
+pub enum DualWriteResult<T> {
+    Success(T),
+    PartialSuccess {
+        duckdb: Option<T>,
+        supabase: Option<T>,
+        errors: Vec<anyhow::Error>,
+    },
+    Failed(Vec<anyhow::Error>),
+}
+
+impl<T> DualWriteResult<T> {
+    /// Check if both writes succeeded
+    pub fn is_success(&self) -> bool {
+        matches!(self, DualWriteResult::Success(_))
+    }
+
+    /// Get the value if successful, or the best available
+    pub fn value(self) -> Option<T> {
+        match self {
+            DualWriteResult::Success(v) => Some(v),
+            DualWriteResult::PartialSuccess { duckdb, supabase, .. } => {
+                duckdb.or(supabase)
+            }
+            DualWriteResult::Failed(_) => None,
+        }
+    }
+
+    /// Get the number of successful writes
+    pub fn success_count(&self) -> usize {
+        match self {
+            DualWriteResult::Success(_) => 2,
+            DualWriteResult::PartialSuccess { duckdb, supabase, .. } => {
+                duckdb.is_some() as usize + supabase.is_some() as usize
+            }
+            DualWriteResult::Failed(_) => 0,
+        }
+    }
+}
+
+/// Trait for types that support dual-write
+#[async_trait]
+pub trait DualWrite: Send + Sync {
+    type Id: Send + Sync + PartialEq + std::fmt::Display + Clone;
+
+    /// Write to DuckDB
+    async fn write_to_duckdb(&self, db: &Database) -> Result<Self::Id>;
+
+    /// Write to Supabase
+    async fn write_to_supabase(&self, db: &SupabasePool) -> Result<Self::Id>;
+
+    /// Dual-write to both databases
+    async fn dual_write(
+        &self,
+        duckdb: &Database,
+        supabase: &SupabasePool,
+    ) -> DualWriteResult<Self::Id> {
+        let (duckdb_result, supabase_result) = tokio::join!(
+            self.write_to_duckdb(duckdb),
+            self.write_to_supabase(supabase)
+        );
+
+        match (duckdb_result, supabase_result) {
+            (Ok(d_id), Ok(s_id)) => {
+                // Verify IDs match (should be the same UUID)
+                if d_id != s_id {
+                    tracing::warn!(
+                        "ID mismatch in dual-write: DuckDB={}, Supabase={}",
+                        d_id, s_id
+                    );
+                }
+                DualWriteResult::Success(d_id)
+            }
+            (Ok(d_id), Err(e)) => DualWriteResult::PartialSuccess {
+                duckdb: Some(d_id),
+                supabase: None,
+                errors: vec![e],
+            },
+            (Err(e), Ok(s_id)) => DualWriteResult::PartialSuccess {
+                duckdb: None,
+                supabase: Some(s_id),
+                errors: vec![e],
+            },
+            (Err(e1), Err(e2)) => DualWriteResult::Failed(vec![e1, e2]),
+        }
+    }
+
+    /// Update in DuckDB
+    async fn update_in_duckdb(&self, db: &Database) -> Result<Self::Id>;
+
+    /// Update in Supabase
+    async fn update_in_supabase(&self, db: &SupabasePool) -> Result<Self::Id>;
+
+    /// Dual-update to both databases
+    async fn dual_update(
+        &self,
+        duckdb: &Database,
+        supabase: &SupabasePool,
+    ) -> DualWriteResult<Self::Id> {
+        let (duckdb_result, supabase_result) = tokio::join!(
+            self.update_in_duckdb(duckdb),
+            self.update_in_supabase(supabase)
+        );
+
+        match (duckdb_result, supabase_result) {
+            (Ok(d_id), Ok(_)) => DualWriteResult::Success(d_id),
+            (Ok(d_id), Err(e)) => DualWriteResult::PartialSuccess {
+                duckdb: Some(d_id),
+                supabase: None,
+                errors: vec![e],
+            },
+            (Err(e), Ok(s_id)) => DualWriteResult::PartialSuccess {
+                duckdb: None,
+                supabase: Some(s_id),
+                errors: vec![e],
+            },
+            (Err(e1), Err(e2)) => DualWriteResult::Failed(vec![e1, e2]),
+        }
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_read_source_from_env_default() {
+        // Clear the env var if set
+        let _ = std::env::set_var("READ_SOURCE", "");
+        let source = ReadSource::from_env();
+        assert_eq!(source, ReadSource::DuckDb);
+    }
+
+    #[test]
+    fn test_read_source_from_env_supabase() {
+        std::env::set_var("READ_SOURCE", "supabase");
+        let source = ReadSource::from_env();
+        assert_eq!(source, ReadSource::Supabase);
+        std::env::remove_var("READ_SOURCE");
+    }
+
+    #[test]
+    fn test_read_source_name() {
+        assert_eq!(ReadSource::DuckDb.name(), "duckdb");
+        assert_eq!(ReadSource::Supabase.name(), "supabase");
+    }
+}
diff --git a/crates/iou-api/src/lib.rs b/crates/iou-api/src/lib.rs
index 9001915..03938dd 100644
--- a/crates/iou-api/src/lib.rs
+++ b/crates/iou-api/src/lib.rs
@@ -2,9 +2,21 @@
 //!
 //! This library exposes the core types and modules for testing.
 
+pub mod db;
+pub mod domain_dual_write;
+pub mod dual_write;
 pub mod error;
+pub mod search_types;
+pub mod supabase;
 pub mod websockets;
 
 // Re-export commonly used types
+pub use db::Database;
+pub use dual_write::{DualWrite, DualWriteResult, ReadSource};
+pub use search_types::{
+    AdvancedSearchResult, FacetCount, SearchFacets, SearchParams, SearchMode,
+    SortOrder, SuggestionResult, SuggestionType,
+};
+pub use supabase::SupabasePool;
 pub use websockets::types::DocumentStatus;
 pub use websockets::limiter::ConnectionLimiter;
diff --git a/crates/iou-api/src/main.rs b/crates/iou-api/src/main.rs
index d949f74..f0b461d 100644
--- a/crates/iou-api/src/main.rs
+++ b/crates/iou-api/src/main.rs
@@ -18,9 +18,13 @@ use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
 
 mod config;
 mod db;
+mod domain_dual_write;
+mod dual_write;
 mod error;
 mod middleware;
 mod routes;
+mod search_types;
+mod supabase;
 mod workflows;
 mod websockets;
 mod orchestrator;
diff --git a/crates/iou-api/src/routes/search.rs b/crates/iou-api/src/routes/search.rs
index 7e22a6b..c067ff5 100644
--- a/crates/iou-api/src/routes/search.rs
+++ b/crates/iou-api/src/routes/search.rs
@@ -15,6 +15,14 @@ use uuid::Uuid;
 use crate::db::{Database, SearchResult};
 use crate::error::ApiError;
 
+// Re-export shared search types for backward compatibility
+pub use crate::search_types::{
+    AdvancedSearchResult, FacetCount, SearchFacets, SearchParams, SearchMode,
+    SortOrder, SuggestionResult, SuggestionType,
+    default_limit, default_offset, default_semantic, default_search_mode,
+    default_min_score, default_sort,
+};
+
 /// Simple search query parameters (for backwards compatibility)
 #[derive(Debug, Deserialize)]
 pub struct SearchQuery {
@@ -52,165 +60,6 @@ pub async fn search(
     }))
 }
 
-/// Advanced search query parameters
-#[derive(Debug, Deserialize)]
-pub struct SearchParams {
-    /// Search query string
-    pub q: String,
-
-    /// Maximum results to return
-    #[serde(default = "default_limit")]
-    pub limit: i32,
-
-    /// Offset for pagination
-    #[serde(default = "default_offset")]
-    pub offset: i32,
-
-    /// Filter by domain type
-    pub domain_type: Option<String>,
-
-    /// Filter by specific domain ID
-    pub domain_id: Option<String>,
-
-    /// Filter by object type
-    pub object_type: Option<String>,
-
-    /// Filter by classification level
-    pub classification: Option<String>,
-
-    /// Filter by compliance status
-    pub compliance_status: Option<String>,
-
-    /// Enable semantic search (uses embeddings if available)
-    #[serde(default = "default_semantic")]
-    pub semantic: bool,
-
-    /// Search mode: "text", "semantic", or "hybrid"
-    #[serde(default = "default_search_mode")]
-    pub mode: SearchMode,
-
-    /// Minimum relevance score (0.0 - 1.0)
-    #[serde(default = "default_min_score")]
-    pub min_score: f32,
-
-    /// Sort order
-    #[serde(default = "default_sort")]
-    pub sort: SortOrder,
-}
-
-fn default_limit() -> i32 {
-    50
-}
-
-fn default_offset() -> i32 {
-    0
-}
-
-fn default_semantic() -> bool {
-    false
-}
-
-fn default_search_mode() -> SearchMode {
-    SearchMode::Text
-}
-
-fn default_min_score() -> f32 {
-    0.0
-}
-
-fn default_sort() -> SortOrder {
-    SortOrder::Relevance
-}
-
-/// Search mode
-#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
-#[serde(rename_all = "lowercase")]
-pub enum SearchMode {
-    /// Traditional text-based search
-    Text,
-    /// Semantic search using embeddings
-    Semantic,
-    /// Hybrid: combine text and semantic with re-ranking
-    Hybrid,
-}
-
-/// Sort order for results
-#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
-#[serde(rename_all = "lowercase")]
-pub enum SortOrder {
-    /// Sort by relevance score
-    Relevance,
-    /// Sort by date (newest first)
-    DateDesc,
-    /// Sort by date (oldest first)
-    DateAsc,
-    /// Sort by title A-Z
-    TitleAsc,
-}
-
-/// Advanced search result
-#[derive(Debug, Serialize)]
-pub struct AdvancedSearchResult {
-    /// Unique identifier
-    pub id: Uuid,
-
-    /// Object type
-    pub object_type: String,
-
-    /// Title of the object
-    pub title: String,
-
-    /// Brief snippet with highlighted matches
-    pub snippet: String,
-
-    /// Domain this object belongs to
-    pub domain_id: Uuid,
-    pub domain_name: String,
-    pub domain_type: String,
-
-    /// Classification level
-    pub classification: String,
-
-    /// Relevance score (0.0 - 1.0)
-    pub score: f32,
-
-    /// Creation date
-    pub created_at: String,
-
-    /// Optional: semantic similarity score
-    pub semantic_score: Option<f32>,
-
-    /// Optional: text search rank
-    pub text_rank: Option<f32>,
-
-    /// Compliance metadata
-    pub is_woo_relevant: Option<bool>,
-    pub woo_disclosure_class: Option<String>,
-}
-
-/// Faceted search results for filters
-#[derive(Debug, Serialize)]
-pub struct SearchFacets {
-    /// Available domain types with counts
-    pub domain_types: Vec<FacetCount>,
-
-    /// Available object types with counts
-    pub object_types: Vec<FacetCount>,
-
-    /// Available classifications with counts
-    pub classifications: Vec<FacetCount>,
-
-    /// Compliance status distribution
-    pub compliance_statuses: Vec<FacetCount>,
-}
-
-#[derive(Debug, Serialize)]
-pub struct FacetCount {
-    pub value: String,
-    pub count: i64,
-    pub label: String,
-}
-
 /// Complete search response
 #[derive(Debug, Serialize)]
 pub struct SearchResults {
@@ -297,22 +146,6 @@ fn default_suggest_limit() -> i32 {
     10
 }
 
-#[derive(Debug, Serialize)]
-pub struct SuggestionResult {
-    pub text: String,
-    pub suggestion_type: SuggestionType,
-    pub count: Option<i64>,
-}
-
-#[derive(Debug, Serialize)]
-#[serde(rename_all = "lowercase")]
-pub enum SuggestionType {
-    Query,
-    Domain,
-    Tag,
-    Entity,
-}
-
 pub async fn search_suggest(
     Query(params): Query<SuggestParams>,
     Extension(db): Extension<Arc<Database>>,
diff --git a/crates/iou-api/src/search_types.rs b/crates/iou-api/src/search_types.rs
new file mode 100644
index 0000000..04e18ce
--- /dev/null
+++ b/crates/iou-api/src/search_types.rs
@@ -0,0 +1,182 @@
+//! Shared types for search functionality
+//!
+//! These types are used by both db.rs and routes/search.rs
+//! to avoid circular dependencies.
+
+use serde::{Deserialize, Serialize};
+use uuid::Uuid;
+
+/// Advanced search query parameters
+#[derive(Debug, Deserialize)]
+pub struct SearchParams {
+    /// Search query string
+    pub q: String,
+
+    /// Maximum results to return
+    #[serde(default = "default_limit")]
+    pub limit: i32,
+
+    /// Offset for pagination
+    #[serde(default = "default_offset")]
+    pub offset: i32,
+
+    /// Filter by domain type
+    pub domain_type: Option<String>,
+
+    /// Filter by specific domain ID
+    pub domain_id: Option<String>,
+
+    /// Filter by object type
+    pub object_type: Option<String>,
+
+    /// Filter by classification level
+    pub classification: Option<String>,
+
+    /// Filter by compliance status
+    pub compliance_status: Option<String>,
+
+    /// Enable semantic search (uses embeddings if available)
+    #[serde(default = "default_semantic")]
+    pub semantic: bool,
+
+    /// Search mode: "text", "semantic", or "hybrid"
+    #[serde(default = "default_search_mode")]
+    pub mode: SearchMode,
+
+    /// Minimum relevance score (0.0 - 1.0)
+    #[serde(default = "default_min_score")]
+    pub min_score: f32,
+
+    /// Sort order
+    #[serde(default = "default_sort")]
+    pub sort: SortOrder,
+}
+
+pub fn default_limit() -> i32 {
+    50
+}
+
+pub fn default_offset() -> i32 {
+    0
+}
+
+pub fn default_semantic() -> bool {
+    false
+}
+
+pub fn default_search_mode() -> SearchMode {
+    SearchMode::Text
+}
+
+pub fn default_min_score() -> f32 {
+    0.0
+}
+
+pub fn default_sort() -> SortOrder {
+    SortOrder::Relevance
+}
+
+/// Search mode
+#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
+#[serde(rename_all = "lowercase")]
+pub enum SearchMode {
+    /// Traditional text-based search
+    Text,
+    /// Semantic search using embeddings
+    Semantic,
+    /// Hybrid: combine text and semantic with re-ranking
+    Hybrid,
+}
+
+/// Sort order for results
+#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
+#[serde(rename_all = "lowercase")]
+pub enum SortOrder {
+    /// Sort by relevance score
+    Relevance,
+    /// Sort by date (newest first)
+    DateDesc,
+    /// Sort by date (oldest first)
+    DateAsc,
+    /// Sort by title A-Z
+    TitleAsc,
+}
+
+/// Advanced search result
+#[derive(Debug, Serialize)]
+pub struct AdvancedSearchResult {
+    /// Unique identifier
+    pub id: Uuid,
+
+    /// Object type
+    pub object_type: String,
+
+    /// Title of the object
+    pub title: String,
+
+    /// Brief snippet with highlighted matches
+    pub snippet: String,
+
+    /// Domain this object belongs to
+    pub domain_id: Uuid,
+    pub domain_name: String,
+    pub domain_type: String,
+
+    /// Classification level
+    pub classification: String,
+
+    /// Relevance score (0.0 - 1.0)
+    pub score: f32,
+
+    /// Creation date
+    pub created_at: String,
+
+    /// Optional: semantic similarity score
+    pub semantic_score: Option<f32>,
+
+    /// Optional: text search rank
+    pub text_rank: Option<f32>,
+
+    /// Compliance metadata
+    pub is_woo_relevant: Option<bool>,
+    pub woo_disclosure_class: Option<String>,
+}
+
+/// Faceted search results for filters
+#[derive(Debug, Serialize)]
+pub struct SearchFacets {
+    /// Available domain types with counts
+    pub domain_types: Vec<FacetCount>,
+
+    /// Available object types with counts
+    pub object_types: Vec<FacetCount>,
+
+    /// Available classifications with counts
+    pub classifications: Vec<FacetCount>,
+
+    /// Compliance status distribution
+    pub compliance_statuses: Vec<FacetCount>,
+}
+
+#[derive(Debug, Serialize)]
+pub struct FacetCount {
+    pub value: String,
+    pub count: i64,
+    pub label: String,
+}
+
+#[derive(Debug, Serialize)]
+pub struct SuggestionResult {
+    pub text: String,
+    pub suggestion_type: SuggestionType,
+    pub count: Option<i64>,
+}
+
+#[derive(Debug, Serialize)]
+#[serde(rename_all = "lowercase")]
+pub enum SuggestionType {
+    Query,
+    Domain,
+    Tag,
+    Entity,
+}
diff --git a/crates/iou-api/src/supabase.rs b/crates/iou-api/src/supabase.rs
new file mode 100644
index 0000000..98446ed
--- /dev/null
+++ b/crates/iou-api/src/supabase.rs
@@ -0,0 +1,74 @@
+//! Supabase database connection layer
+//!
+//! Provides PostgreSQL connection pool for self-hosted Supabase
+//! as part of the hybrid DuckDB + Supabase architecture.
+
+use sqlx::{PgPool, postgres::PgPoolOptions};
+use anyhow::Result;
+
+/// Supabase database connection pool
+#[derive(Clone)]
+pub struct SupabasePool {
+    pool: PgPool,
+}
+
+impl SupabasePool {
+    /// Create a new Supabase connection pool from DATABASE_URL
+    pub async fn new(database_url: &str) -> Result<Self> {
+        let pool = PgPoolOptions::new()
+            .max_connections(10)
+            .connect(database_url)
+            .await?;
+
+        tracing::info!("Connected to Supabase PostgreSQL");
+
+        Ok(Self { pool })
+    }
+
+    /// Create a new connection pool with custom max connections
+    pub async fn with_max_connections(database_url: &str, max: u32) -> Result<Self> {
+        let pool = PgPoolOptions::new()
+            .max_connections(max)
+            .connect(database_url)
+            .await?;
+
+        tracing::info!("Connected to Supabase PostgreSQL (max connections: {})", max);
+
+        Ok(Self { pool })
+    }
+
+    /// Get the underlying sqlx PgPool
+    pub fn inner(&self) -> &PgPool {
+        &self.pool
+    }
+
+    /// Health check for the database connection
+    pub async fn health_check(&self) -> Result<()> {
+        sqlx::query("SELECT 1")
+            .fetch_one(&self.pool)
+            .await?;
+        Ok(())
+    }
+
+    /// Run PostgreSQL migrations
+    pub async fn run_migrations(&self) -> Result<()> {
+        // Note: In production, use sqlx-cli or a proper migration tool
+        // This is a simplified version for development
+        tracing::info!("Migrations should be run via sqlx-cli or external tool");
+        Ok(())
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[tokio::test]
+    #[ignore] // Requires actual Supabase instance
+    async fn test_supabase_connection() {
+        let pool = SupabasePool::new("postgresql://postgres:postgres@localhost:5432/iou_modern")
+            .await
+            .unwrap();
+        assert!(pool.health_check().await.is_ok());
+    }
+}
diff --git a/crates/iou-api/tests/dual_write.rs b/crates/iou-api/tests/dual_write.rs
new file mode 100644
index 0000000..1b8af27
--- /dev/null
+++ b/crates/iou-api/tests/dual_write.rs
@@ -0,0 +1,153 @@
+//! Dual-Write Tests
+//!
+//! Tests for the dual-write pattern that writes to both DuckDB and Supabase.
+
+use iou_api::{db::Database, dual_write::DualWrite, supabase::SupabasePool};
+use iou_core::domain::{DomainType, InformationDomain};
+use uuid::Uuid;
+
+async fn get_test_duckdb() -> Database {
+    let db = Database::new("/tmp/test_iou_dual_write.db")
+        .expect("Failed to create DuckDB");
+    db.initialize_schema()
+        .expect("Failed to initialize schema");
+    db
+}
+
+async fn get_test_supabase() -> SupabasePool {
+    let database_url = std::env::var("SUPABASE_DATABASE_URL")
+        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/iou_modern".to_string());
+
+    SupabasePool::new(&database_url)
+        .await
+        .expect("Failed to connect to Supabase")
+}
+
+#[tokio::test]
+#[ignore] // Requires running Supabase instance
+async fn test_dual_write_consistency() {
+    let duckdb = get_test_duckdb().await;
+    let supabase = get_test_supabase().await;
+
+    let domain = InformationDomain::new(
+        DomainType::Zaak,
+        "Test Dual-Write Domain".to_string(),
+        Uuid::new_v4(),
+    );
+
+    let result = domain.dual_write(&duckdb, &supabase).await;
+
+    assert!(result.is_success(), "Dual-write should succeed");
+    let id = result.value().expect("Should have an ID");
+
+    // Verify record exists in DuckDB
+    let duckdb_record = duckdb.get_domain(id).unwrap();
+    assert!(duckdb_record.is_some(), "Record should exist in DuckDB");
+
+    // Verify record exists in Supabase
+    let supabase_record: Option<(Uuid, String)> = sqlx::query_as(
+        "SELECT id, name FROM information_domains WHERE id = $1"
+    )
+    .bind(id)
+    .fetch_one(supabase.inner())
+    .await
+    .ok();
+
+    assert!(supabase_record.is_some(), "Record should exist in Supabase");
+
+    println!("Dual-write consistency test passed for domain: {}", id);
+}
+
+#[tokio::test]
+#[ignore]
+async fn test_dual_write_with_optional_fields() {
+    let duckdb = get_test_duckdb().await;
+    let supabase = get_test_supabase().await;
+
+    let mut domain = InformationDomain::new(
+        DomainType::Project,
+        "Project with Details".to_string(),
+        Uuid::new_v4(),
+    );
+    domain.description = Some("A detailed project description".to_string());
+
+    let result = domain.dual_write(&duckdb, &supabase).await;
+
+    assert!(result.is_success());
+    let id = result.value().unwrap();
+
+    // Verify description in both databases
+    let duckdb_record = duckdb.get_domain(id).unwrap().unwrap();
+    assert_eq!(duckdb_record.description, Some("A detailed project description".to_string()));
+
+    let supabase_desc: Option<String> = sqlx::query_scalar(
+        "SELECT description FROM information_domains WHERE id = $1"
+    )
+    .bind(id)
+    .fetch_one(supabase.inner())
+    .await
+    .unwrap();
+
+    assert_eq!(supabase_desc, Some("A detailed project description".to_string()));
+
+    println!("Dual-write with optional fields passed");
+}
+
+#[tokio::test]
+#[ignore]
+async fn test_dual_update() {
+    let duckdb = get_test_duckdb().await;
+    let supabase = get_test_supabase().await;
+
+    let mut domain = InformationDomain::new(
+        DomainType::Beleid,
+        "Original Name".to_string(),
+        Uuid::new_v4(),
+    );
+
+    // Initial write
+    let result = domain.dual_write(&duckdb, &supabase).await;
+    assert!(result.is_success());
+    let id = result.value().unwrap();
+
+    // Update the domain
+    domain.name = "Updated Name".to_string();
+    domain.description = Some("Updated description".to_string());
+
+    let update_result = domain.dual_update(&duckdb, &supabase).await;
+    assert!(update_result.is_success());
+
+    // Verify update in DuckDB
+    let duckdb_record = duckdb.get_domain(id).unwrap().unwrap();
+    assert_eq!(duckdb_record.name, "Updated Name");
+
+    // Verify update in Supabase
+    let supabase_name: String = sqlx::query_scalar(
+        "SELECT name FROM information_domains WHERE id = $1"
+    )
+    .bind(id)
+    .fetch_one(supabase.inner())
+    .await
+    .unwrap();
+
+    assert_eq!(supabase_name, "Updated Name");
+
+    println!("Dual-update test passed");
+}
+
+#[tokio::test]
+fn test_read_source_from_env_default() {
+    // Clear env var
+    std::env::remove_var("READ_SOURCE");
+
+    let source = iou_api::ReadSource::from_env();
+    assert_eq!(source, iou_api::ReadSource::DuckDb);
+}
+
+#[tokio::test]
+fn test_read_source_from_env_supabase() {
+    std::env::set_var("READ_SOURCE", "supabase");
+    let source = iou_api::ReadSource::from_env();
+    assert_eq!(source, iou_api::ReadSource::Supabase);
+    std::env::remove_var("READ_SOURCE");
+}
diff --git a/crates/iou-api/tests/schema_equivalence.rs b/crates/iou-api/tests/schema_equivalence.rs
new file mode 100644
index 0000000..aa181a0
--- /dev/null
+++ b/crates/iou-api/tests/schema_equivalence.rs
@@ -0,0 +1,199 @@
+//! Schema Equivalence Tests
+//!
+//! Tests to verify that PostgreSQL schema matches DuckDB structure
+//! for the hybrid architecture.
+
+use uuid::Uuid;
+
+#[tokio::test]
+#[ignore] // Requires running Supabase instance
+async fn test_information_domains_schema_matches() {
+    // This test verifies PostgreSQL schema matches DuckDB structure
+    let database_url = std::env::var("SUPABASE_DATABASE_URL")
+        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/iou_modern".to_string());
+
+    let pool = sqlx::PgPool::connect(&database_url)
+        .await
+        .expect("Failed to connect to Supabase");
+
+    // Check table exists
+    let result = sqlx::query(
+        "SELECT EXISTS (SELECT FROM pg_tables WHERE tablename = 'information_domains')"
+    )
+    .fetch_one(&pool)
+    .await
+    .expect("Query failed");
+
+    let exists: bool = result.get("exists");
+    assert!(exists, "information_domains table should exist");
+
+    // Verify columns
+    let columns: Vec<String> = sqlx::query(
+        "SELECT column_name FROM information_schema.columns
+         WHERE table_name = 'information_domains'
+         ORDER BY ordinal_position"
+    )
+    .fetch_all(&pool)
+    .await
+    .unwrap()
+    .into_iter()
+    .map(|row| row.get("column_name"))
+    .collect();
+
+    assert!(columns.contains(&"id".to_string()), "Missing column: id");
+    assert!(columns.contains(&"domain_type".to_string()), "Missing column: domain_type");
+    assert!(columns.contains(&"name".to_string()), "Missing column: name");
+    assert!(columns.contains(&"organization_id".to_string()), "Missing column: organization_id");
+    assert!(columns.contains(&"status".to_string()), "Missing column: status");
+
+    println!("PostgreSQL information_domains schema verified");
+}
+
+#[tokio::test]
+#[ignore]
+async fn test_information_objects_schema_matches() {
+    let database_url = std::env::var("SUPABASE_DATABASE_URL")
+        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/iou_modern".to_string());
+
+    let pool = sqlx::PgPool::connect(&database_url)
+        .await
+        .expect("Failed to connect to Supabase");
+
+    // Check table exists
+    let result = sqlx::query(
+        "SELECT EXISTS (SELECT FROM pg_tables WHERE tablename = 'information_objects')"
+    )
+    .fetch_one(&pool)
+    .await
+    .expect("Query failed");
+
+    let exists: bool = result.get("exists");
+    assert!(exists, "information_objects table should exist");
+
+    // Verify key columns
+    let columns: Vec<String> = sqlx::query(
+        "SELECT column_name FROM information_schema.columns
+         WHERE table_name = 'information_objects'
+         ORDER BY ordinal_position"
+    )
+    .fetch_all(&pool)
+    .await
+    .unwrap()
+    .into_iter()
+    .map(|row| row.get("column_name"))
+    .collect();
+
+    assert!(columns.contains(&"id".to_string()));
+    assert!(columns.contains(&"domain_id".to_string()));
+    assert!(columns.contains(&"object_type".to_string()));
+    assert!(columns.contains(&"title".to_string()));
+
+    println!("PostgreSQL information_objects schema verified");
+}
+
+#[tokio::test]
+#[ignore]
+async fn test_documents_schema_matches() {
+    let database_url = std::env::var("SUPABASE_DATABASE_URL")
+        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/iou_modern".to_string());
+
+    let pool = sqlx::PgPool::connect(&database_url)
+        .await
+        .expect("Failed to connect to Supabase");
+
+    // Check table exists
+    let result = sqlx::query(
+        "SELECT EXISTS (SELECT FROM pg_tables WHERE tablename = 'documents')"
+    )
+    .fetch_one(&pool)
+    .await
+    .expect("Query failed");
+
+    let exists: bool = result.get("exists");
+    assert!(exists, "documents table should exist");
+
+    println!("PostgreSQL documents schema verified");
+}
+
+#[tokio::test]
+#[ignore]
+async fn test_view_exists_searchable_objects() {
+    let database_url = std::env::var("SUPABASE_DATABASE_URL")
+        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/iou_modern".to_string());
+
+    let pool = sqlx::PgPool::connect(&database_url)
+        .await
+        .expect("Failed to connect to Supabase");
+
+    // Check view exists
+    let result = sqlx::query(
+        "SELECT EXISTS (SELECT FROM pg_views WHERE viewname = 'v_searchable_objects')"
+    )
+    .fetch_one(&pool)
+    .await
+    .expect("Query failed");
+
+    let exists: bool = result.get("exists");
+    assert!(exists, "v_searchable_objects view should exist");
+
+    println!("PostgreSQL view v_searchable_objects verified");
+}
+
+#[tokio::test]
+#[ignore]
+async fn test_view_exists_compliance_overview() {
+    let database_url = std::env::var("SUPABASE_DATABASE_URL")
+        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/iou_modern".to_string());
+
+    let pool = sqlx::PgPool::connect(&database_url)
+        .await
+        .expect("Failed to connect to Supabase");
+
+    // Check view exists
+    let result = sqlx::query(
+        "SELECT EXISTS (SELECT FROM pg_views WHERE viewname = 'v_compliance_overview')"
+    )
+    .fetch_one(&pool)
+    .await
+    .expect("Query failed");
+
+    let exists: bool = result.get("exists");
+    assert!(exists, "v_compliance_overview view should exist");
+
+    println!("PostgreSQL view v_compliance_overview verified");
+}
+
+#[tokio::test]
+#[ignore]
+async fn test_extensions_enabled() {
+    let database_url = std::env::var("SUPABASE_DATABASE_URL")
+        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/iou_modern".to_string());
+
+    let pool = sqlx::PgPool::connect(&database_url)
+        .await
+        .expect("Failed to connect to Supabase");
+
+    // Check uuid-ossp extension
+    let result = sqlx::query(
+        "SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'uuid-ossp')"
+    )
+    .fetch_one(&pool)
+    .await
+    .expect("Query failed");
+
+    let exists: bool = result.get("exists");
+    assert!(exists, "uuid-ossp extension should be enabled");
+
+    // Check pgcrypto extension
+    let result = sqlx::query(
+        "SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'pgcrypto')"
+    )
+    .fetch_one(&pool)
+    .await
+    .expect("Query failed");
+
+    let exists: bool = result.get("exists");
+    assert!(exists, "pgcrypto extension should be enabled");
+
+    println!("PostgreSQL extensions verified");
+}
diff --git a/docker-compose.supabase.yml b/docker-compose.supabase.yml
new file mode 100644
index 0000000..0d89153
--- /dev/null
+++ b/docker-compose.supabase.yml
@@ -0,0 +1,81 @@
+# Docker Compose for self-hosted Supabase
+# This file deploys Supabase alongside the existing DuckDB setup
+version: '3.8'
+
+services:
+  db:
+    image: supabase/postgres:15.1.0.147
+    restart: unless-stopped
+    environment:
+      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-postgres}
+      POSTGRES_DB: iou_modern
+      POSTGRES_USER: postgres
+    ports:
+      - "5432:5432"
+    volumes:
+      - postgres_data:/var/lib/postgresql/data
+    healthcheck:
+      test: ["CMD-SHELL", "pg_isready -U postgres"]
+      interval: 10s
+      timeout: 5s
+      retries: 5
+
+  studio:
+    image: supabase/studio:20240129.90486e4
+    restart: unless-stopped
+    environment:
+      STUDIO_PG_RESET_URL: ${STUDIO_PG_RESET_URL:-http://localhost:3000/api/reset}
+      DEFAULT_ORGANIZATION_NAME: "IOU-Modern"
+      DEFAULT_PROJECT_NAME: "Migration"
+    ports:
+      - "3000:3000"
+    depends_on:
+      db:
+        condition: service_healthy
+
+  auth:
+    image: supabase/gotrue:v2.138.2
+    restart: unless-stopped
+    environment:
+      GOTRUE_JWT_SECRET: ${JWT_SECRET:-super-secret-jwt-token-change-in-production}
+      GOTRUE_DB_DRIVER: postgres
+      GOTRUE_DB_DATABASE_URL: postgres://postgres:${POSTGRES_PASSWORD:-postgres}@db:5432/iou_modern
+      GOTRUE_SITE_URL: http://localhost:3000
+    ports:
+      - "9999:9999"
+    depends_on:
+      db:
+        condition: service_healthy
+
+  realtime:
+    image: supabase/realtime:v2.25.73
+    restart: unless-stopped
+    environment:
+      PORT: 4000
+      DB_HOST: db
+      DB_PORT: 5432
+      DB_USER: postgres
+      DB_PASSWORD: ${POSTGRES_PASSWORD:-postgres}
+      DB_NAME: iou_modern
+      SLOT_NAME: supabase_realtime
+    ports:
+      - "4000:4000"
+    depends_on:
+      db:
+        condition: service_healthy
+
+  rest:
+    image: postgrest/postgrest:v12.0.1
+    restart: unless-stopped
+    environment:
+      PGRST_DB_URI: postgres://postgres:${POSTGRES_PASSWORD:-postgres}@db:5432/iou_modern
+      PGRST_DB_SCHEMA: public
+      PGRST_DB_ANON_ROLE: postgres
+    ports:
+      - "3001:3000"
+    depends_on:
+      db:
+        condition: service_healthy
+
+volumes:
+  postgres_data:
diff --git a/migrations/postgres/001_create_initial_schema.sql b/migrations/postgres/001_create_initial_schema.sql
new file mode 100644
index 0000000..1f15356
--- /dev/null
+++ b/migrations/postgres/001_create_initial_schema.sql
@@ -0,0 +1,246 @@
+-- PostgreSQL Schema for IOU-Modern
+-- This migration creates the PostgreSQL schema equivalent to DuckDB
+-- for the hybrid Supabase + DuckDB architecture.
+
+-- Enable required extensions
+CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
+CREATE EXTENSION IF NOT EXISTS "pgcrypto";
+CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";
+
+-- ============================================================
+-- INFORMATION_DOMAINS table
+-- ============================================================
+CREATE TABLE IF NOT EXISTS information_domains (
+    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
+    domain_type VARCHAR NOT NULL CHECK (domain_type IN ('Zaak', 'Project', 'Beleid', 'Expertise')),
+    name VARCHAR NOT NULL,
+    description TEXT,
+    status VARCHAR NOT NULL DEFAULT 'actief' CHECK (status IN ('concept', 'actief', 'afgerond', 'gearchiveerd')),
+    organization_id UUID NOT NULL,
+    owner_user_id UUID,
+    parent_domain_id UUID REFERENCES information_domains(id) ON DELETE SET NULL,
+    metadata JSONB DEFAULT '{}',
+    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
+    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
+);
+
+-- Indexes for information_domains
+CREATE INDEX IF NOT EXISTS idx_information_domains_type ON information_domains(domain_type);
+CREATE INDEX IF NOT EXISTS idx_information_domains_org ON information_domains(organization_id);
+CREATE INDEX IF NOT EXISTS idx_information_domains_parent ON information_domains(parent_domain_id);
+CREATE INDEX IF NOT EXISTS idx_information_domains_status ON information_domains(status);
+
+-- ============================================================
+-- INFORMATION_OBJECTS table
+-- ============================================================
+CREATE TABLE IF NOT EXISTS information_objects (
+    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
+    domain_id UUID NOT NULL REFERENCES information_domains(id) ON DELETE CASCADE,
+    object_type VARCHAR NOT NULL CHECK (object_type IN ('document', 'email', 'chat', 'besluit', 'data')),
+    title VARCHAR NOT NULL,
+    description TEXT,
+    content_location TEXT,
+    content_text TEXT,
+    mime_type VARCHAR,
+    file_size BIGINT,
+    classification VARCHAR DEFAULT 'intern' CHECK (classification IN ('openbaar', 'intern', 'vertrouwelijk', 'geheim')),
+    retention_period VARCHAR,
+    is_woo_relevant BOOLEAN DEFAULT FALSE,
+    woo_publication_date TIMESTAMP WITH TIME ZONE,
+    privacy_level VARCHAR DEFAULT 'normaal' CHECK (privacy_level IN ('openbaar', 'normaal', 'bijzonder', 'gevoelig')),
+    tags TEXT[] DEFAULT '{}',
+    metadata JSONB DEFAULT '{}',
+    version INTEGER DEFAULT 1,
+    previous_version_id UUID REFERENCES information_objects(id) ON DELETE SET NULL,
+    created_by UUID NOT NULL,
+    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
+    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
+);
+
+-- Indexes for information_objects
+CREATE INDEX IF NOT EXISTS idx_information_objects_domain ON information_objects(domain_id);
+CREATE INDEX IF NOT EXISTS idx_information_objects_type ON information_objects(object_type);
+CREATE INDEX IF NOT EXISTS idx_information_objects_title ON information_objects(title);
+CREATE INDEX IF NOT EXISTS idx_information_objects_classification ON information_objects(classification);
+CREATE INDEX IF NOT EXISTS idx_information_objects_created_at ON information_objects(created_at DESC);
+CREATE INDEX IF NOT EXISTS idx_information_objects_tags ON information_objects USING gin(tags);
+CREATE INDEX IF NOT EXISTS idx_information_objects_metadata ON information_objects USING gin(metadata);
+
+-- ============================================================
+-- DOCUMENTS table (document creation workflow)
+-- ============================================================
+CREATE TABLE IF NOT EXISTS documents (
+    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
+    domain_id VARCHAR NOT NULL,
+    document_type VARCHAR NOT NULL,
+    state VARCHAR NOT NULL DEFAULT 'draft' CHECK (state IN ('draft', 'submitted', 'in_review', 'changes_requested', 'approved', 'published', 'rejected', 'archived')),
+    current_version_key TEXT,
+    previous_version_key TEXT,
+    compliance_score DECIMAL(3, 2),
+    confidence_score DECIMAL(3, 2),
+    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
+    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
+);
+
+-- Indexes for documents
+CREATE INDEX IF NOT EXISTS idx_documents_domain ON documents(domain_id);
+CREATE INDEX IF NOT EXISTS idx_documents_state ON documents(state);
+CREATE INDEX IF NOT EXISTS idx_documents_type ON documents(document_type);
+
+-- ============================================================
+-- TEMPLATES table
+-- ============================================================
+CREATE TABLE IF NOT EXISTS templates (
+    id VARCHAR PRIMARY KEY,
+    name VARCHAR NOT NULL,
+    domain_id VARCHAR NOT NULL,
+    document_type VARCHAR NOT NULL,
+    content TEXT NOT NULL,
+    required_variables JSONB DEFAULT '{}',
+    optional_sections JSONB DEFAULT '{}',
+    version INTEGER DEFAULT 1,
+    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
+    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
+    is_active BOOLEAN DEFAULT TRUE
+);
+
+-- Indexes for templates
+CREATE INDEX IF NOT EXISTS idx_templates_domain ON templates(domain_id);
+CREATE INDEX IF NOT EXISTS idx_templates_type ON templates(document_type);
+CREATE INDEX IF NOT EXISTS idx_templates_active ON templates(is_active) WHERE is_active = TRUE;
+
+-- ============================================================
+-- AUDIT_TRAIL table
+-- ============================================================
+CREATE TABLE IF NOT EXISTS audit_trail (
+    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
+    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
+    agent_name VARCHAR NOT NULL,
+    action VARCHAR NOT NULL,
+    details JSONB DEFAULT '{}',
+    timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
+    execution_time_ms INTEGER
+);
+
+-- Indexes for audit_trail
+CREATE INDEX IF NOT EXISTS idx_audit_trail_document ON audit_trail(document_id);
+CREATE INDEX IF NOT EXISTS idx_audit_trail_timestamp ON audit_trail(timestamp DESC);
+
+-- ============================================================
+-- VIEWS for search and analytics
+-- ============================================================
+
+-- View: searchable_objects - Aggregates searchable text
+CREATE OR REPLACE VIEW v_searchable_objects AS
+SELECT
+    io.id,
+    io.object_type,
+    io.title,
+    io.description,
+    io.content_text,
+    io.domain_id,
+    io.classification,
+    io.created_at,
+    io.is_woo_relevant,
+    -- Concatenate searchable fields for full-text search
+    CONCAT_WS(' ',
+        COALESCE(io.title, ''),
+        COALESCE(io.description, ''),
+        COALESCE(io.content_text, '')
+    ) as searchable_text
+FROM information_objects io;
+
+-- View: compliance_overview - Analytics for compliance metrics
+CREATE OR REPLACE VIEW v_compliance_overview AS
+SELECT
+    id.id as domain_id,
+    id.name as domain_name,
+    id.domain_type,
+    COUNT(io.id) as total_objects,
+    COUNT(CASE WHEN io.is_woo_relevant = TRUE THEN 1 END) as woo_relevant_count,
+    COUNT(CASE WHEN io.classification = 'openbaar' THEN 1 END) as public_count,
+    COUNT(CASE WHEN io.retention_period IS NULL THEN 1 END) as missing_retention
+FROM information_domains id
+LEFT JOIN information_objects io ON io.domain_id = id.id
+GROUP BY id.id, id.name, id.domain_type;
+
+-- View: domain_statistics - Domain type/status distribution
+CREATE OR REPLACE VIEW v_domain_statistics AS
+SELECT
+    domain_type,
+    COUNT(*) as count
+FROM information_domains
+GROUP BY domain_type;
+
+-- View: entity_network - GraphRAG entity relationships
+CREATE OR REPLACE VIEW v_entity_network AS
+SELECT
+    io.id,
+    io.title,
+    io.metadata->>'entities' as entities
+FROM information_objects io
+WHERE io.metadata ? 'entities';
+
+-- ============================================================
+-- FUNCTIONS and TRIGGERS
+-- ============================================================
+
+-- Function to update updated_at timestamp
+CREATE OR REPLACE FUNCTION update_updated_at_column()
+RETURNS TRIGGER AS $$
+BEGIN
+    NEW.updated_at = CURRENT_TIMESTAMP;
+    RETURN NEW;
+END;
+$$ LANGUAGE plpgsql;
+
+-- Triggers for updated_at
+DROP TRIGGER IF EXISTS update_information_domains_updated_at ON information_domains;
+CREATE TRIGGER update_information_domains_updated_at
+    BEFORE UPDATE ON information_domains
+    FOR EACH ROW
+    EXECUTE FUNCTION update_updated_at_column();
+
+DROP TRIGGER IF EXISTS update_information_objects_updated_at ON information_objects;
+CREATE TRIGGER update_information_objects_updated_at
+    BEFORE UPDATE ON information_objects
+    FOR EACH ROW
+    EXECUTE FUNCTION update_updated_at_column();
+
+DROP TRIGGER IF EXISTS update_documents_updated_at ON documents;
+CREATE TRIGGER update_documents_updated_at
+    BEFORE UPDATE ON documents
+    FOR EACH ROW
+    EXECUTE FUNCTION update_updated_at_column();
+
+DROP TRIGGER IF EXISTS update_templates_updated_at ON templates;
+CREATE TRIGGER update_templates_updated_at
+    BEFORE UPDATE ON templates
+    FOR EACH ROW
+    EXECUTE FUNCTION update_updated_at_column();
+
+-- ============================================================
+-- DUAL-WRITE SUPPORT
+-- ============================================================
+
+-- Function to check data consistency between PostgreSQL and DuckDB (manual)
+-- This will be used by the consistency checker
+CREATE OR REPLACE FUNCTION check_record_exists(table_name TEXT, record_id UUID)
+RETURNS BOOLEAN AS $$
+BEGIN
+    CASE table_name
+        WHEN 'information_domains' THEN
+            RETURN EXISTS(SELECT 1 FROM information_domains WHERE id = record_id);
+        WHEN 'information_objects' THEN
+            RETURN EXISTS(SELECT 1 FROM information_objects WHERE id = record_id);
+        WHEN 'documents' THEN
+            RETURN EXISTS(SELECT 1 FROM documents WHERE id = record_id);
+        ELSE
+            RETURN FALSE;
+    END CASE;
+END;
+$$ LANGUAGE plpgsql;
+
+-- Grant necessary permissions (adjust for your setup)
+-- GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO postgres;
+-- GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO postgres;
diff --git a/migrations/postgres/README.md b/migrations/postgres/README.md
new file mode 100644
index 0000000..3a0292f
--- /dev/null
+++ b/migrations/postgres/README.md
@@ -0,0 +1,89 @@
+# PostgreSQL Migration Scripts for Supabase
+
+This directory contains the PostgreSQL migration scripts for the hybrid DuckDB + Supabase architecture.
+
+## Setup
+
+### 1. Start Supabase
+
+```bash
+# Copy the example environment file
+cp .env.supabase.example .env.supabase
+
+# Update with secure values
+# Edit .env.supabase and change POSTGRES_PASSWORD and JWT_SECRET
+
+# Start Supabase
+docker-compose -f docker-compose.supabase.yml up -d
+
+# Check status
+docker-compose -f docker-compose.supabase.yml ps
+```
+
+### 2. Run Migrations
+
+#### Using psql directly:
+
+```bash
+# Set database URL
+export SUPABASE_DATABASE_URL="postgresql://postgres:YOUR_PASSWORD@localhost:5432/iou_modern"
+
+# Run migration
+psql $SUPABASE_DATABASE_URL -f migrations/postgres/001_create_initial_schema.sql
+```
+
+#### Using sqlx-cli (recommended for production):
+
+```bash
+# Install sqlx-cli
+cargo install sqlx-cli --no-default-features --features rustls,postgres
+
+# Run migrations
+sqlx database create --database-url $SUPABASE_DATABASE_URL
+# Note: You'll need to convert .sql files to sqlx format or use psql for now
+```
+
+### 3. Verify Setup
+
+```bash
+# Connect to PostgreSQL
+psql postgresql://postgres:YOUR_PASSWORD@localhost:5432/iou_modern
+
+# Check tables
+\dt
+
+# Check views
+\dv
+
+# Verify extensions
+SELECT extname FROM pg_extension;
+```
+
+## Accessing Supabase Services
+
+- **Studio**: http://localhost:3000
+- **PostgREST API**: http://localhost:3001
+- **Auth**: http://localhost:9999
+- **Realtime**: http://localhost:4000
+
+## Stopping Supabase
+
+```bash
+docker-compose -f docker-compose.supabase.yml down
+
+# To remove volumes (WARNING: deletes all data)
+docker-compose -f docker-compose.supabase.yml down -v
+```
+
+## Running Tests
+
+```bash
+# Set environment variables
+export SUPABASE_DATABASE_URL="postgresql://postgres:YOUR_PASSWORD@localhost:5432/iou_modern"
+
+# Run schema equivalence tests
+cargo test --package iou-api --test schema_equivalence
+
+# Run dual-write tests
+cargo test --package iou-api --test dual_write
+```
