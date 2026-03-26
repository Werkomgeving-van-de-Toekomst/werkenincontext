Now I have all the context I need. Let me generate the content for section-02-foundation. This section covers Phase 1: Foundation (Weeks 3-4) which involves deploying Supabase, creating the PostgreSQL schema, establishing database connections, implementing dual-write pattern, and creating the read toggle.

# Section 02: Foundation - Supabase Deployment and Dual-Write Pattern

## Overview

This section implements Phase 1 of the migration: deploying Supabase alongside DuckDB with zero downtime. The goal is to establish the infrastructure for the hybrid architecture while maintaining full backward compatibility.

**Timeline:** Weeks 3-4

**Dependencies:** Section 01 (Assessment) must be complete, with performance baselines documented and hosting location decided.

## Objectives

1. Deploy self-hosted Supabase via Docker Compose
2. Create PostgreSQL schema matching DuckDB tables
3. Establish database connections from Rust backend
4. Implement dual-write pattern (write to both databases)
5. Create read toggle for gradual cutover

**Success Criteria:**
- Can write to both databases simultaneously
- Data consistency verified between databases
- No performance degradation compared to baseline

---

## Background: DuckDB Schema

The current DuckDB implementation uses the following schema. This must be replicated in PostgreSQL with appropriate type conversions.

### Core Tables

**information_domains** - Organizational context
```sql
CREATE TABLE information_domains (
    id UUID PRIMARY KEY DEFAULT uuid(),
    domain_type VARCHAR NOT NULL,  -- 'Zaak', 'Project', 'Beleid', 'Expertise'
    title VARCHAR NOT NULL,
    description TEXT,
    parent_id UUID REFERENCES information_domains(id),
    organization_id UUID NOT NULL,
    metadata JSON DEFAULT '{}',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**information_objects** - Main document/content with full-text search
```sql
CREATE TABLE information_objects (
    id UUID PRIMARY KEY DEFAULT uuid(),
    domain_id UUID NOT NULL REFERENCES information_domains(id),
    title VARCHAR NOT NULL,
    content TEXT,
    content_vectors FLOAT[],  -- Vector embeddings
    metadata JSON DEFAULT '{}',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**documents** - Document metadata with workflow states
```sql
CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT uuid(),
    information_object_id UUID NOT NULL REFERENCES information_objects(id),
    title VARCHAR NOT NULL,
    document_type VARCHAR NOT NULL,
    status VARCHAR NOT NULL DEFAULT 'draft',  -- draft, review, approved, published, archived
    classification VARCHAR DEFAULT 'internal',  -- internal, confidential, public
    woo_published BOOLEAN DEFAULT FALSE,
    woo_publication_date TIMESTAMP,
    retention_date DATE,
    destruction_date DATE,
    metadata JSON DEFAULT '{}',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**templates** - Document templates
```sql
CREATE TABLE templates (
    id UUID PRIMARY KEY DEFAULT uuid(),
    name VARCHAR NOT NULL,
    description TEXT,
    template_content TEXT NOT NULL,
    variables JSON DEFAULT '{}',
    organization_id UUID NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**audit_trail** - Full audit trail
```sql
CREATE TABLE audit_trail (
    id UUID PRIMARY KEY DEFAULT uuid(),
    table_name VARCHAR NOT NULL,
    record_id UUID NOT NULL,
    action VARCHAR NOT NULL,  -- INSERT, UPDATE, DELETE
    old_values JSON,
    new_values JSON,
    user_id UUID,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Key Views

```sql
-- v_searchable_objects - Aggregates searchable text
CREATE VIEW v_searchable_objects AS
SELECT io.id, io.title, io.content, d.title as document_title, 
       d.status, d.classification, io.domain_id
FROM information_objects io
LEFT JOIN documents d ON io.id = d.information_object_id;

-- v_compliance_overview - Analytics for compliance metrics
CREATE VIEW v_compliance_overview AS
SELECT d.status, d.classification, d.woo_published,
       COUNT(*) as count, MIN(d.retention_date) as earliest_retention
FROM documents d
GROUP BY d.status, d.classification, d.woo_published;

-- v_domain_statistics - Domain type/status distribution
CREATE VIEW v_domain_statistics AS
SELECT domain_type, COUNT(*) as count
FROM information_domains
GROUP BY domain_type;

-- v_entity_network - GraphRAG entity relationships
CREATE VIEW v_entity_network AS
SELECT io.id, io.title, io.metadata->>'entities' as entities
FROM information_objects io
WHERE io.metadata ? 'entities';
```

---

## SQL Compatibility Notes for Migration

When migrating from DuckDB to PostgreSQL, these differences must be addressed:

| Feature | DuckDB | PostgreSQL | Action Required |
|---------|--------|------------|-----------------|
| UUID generation | `uuid()` | `gen_random_uuid()` | Schema migration required |
| Array types | `VARCHAR[]` | `TEXT[]` or `VARCHAR[]` | Minor syntax differences |
| JSON existence | `metadata ? 'key'` | `metadata ? 'key'` | Same operator, but enable JSONB |
| JSON access | `metadata->>'key'` | `metadata->>'key'` | Same operator |
| Default timestamp | `CURRENT_TIMESTAMP` | `CURRENT_TIMESTAMP` | Same behavior |
| Connection model | `Arc<Mutex<Connection>>` | `PgPool` (connection pool) | Architectural change required |

---

## Implementation

### Step 1: Deploy Supabase via Docker Compose

**File:** `docker-compose.supabase.yml`

Create a Docker Compose configuration for self-hosted Supabase:

```yaml
# docker-compose.supabase.yml
version: '3.8'

services:
  db:
    image: supabase/postgres:15.1.0.147
    restart: unless-stopped
    environment:
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
      POSTGRES_DB: iou_modern
      POSTGRES_USER: postgres
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5

  studio:
    image: supabase/studio:20240129.90486e4
    restart: unless-stopped
    environment:
      STUDIO_PG_RESET_URL: ${STUDIO_PG_RESET_URL}
      DEFAULT_ORGANIZATION_NAME: "IOU-Modern"
      DEFAULT_PROJECT_NAME: "Migration"
    ports:
      - "3000:3000"
    depends_on:
      db:
        condition: service_healthy

  auth:
    image: supabase/gotrue:v2.138.2
    restart: unless-stopped
    environment:
      GOTRUE_JWT_SECRET: ${JWT_SECRET}
      GOTRUE_DB_DRIVER: postgres
      GOTRUE_DB_DATABASE_URL: postgres://postgres:${POSTGRES_PASSWORD}@db:5432/iou_modern
      GOTRUE_SITE_URL: http://localhost:3000
    ports:
      - "9999:9999"
    depends_on:
      db:
        condition: service_healthy

  realtime:
    image: supabase/realtime:v2.25.73
    restart: unless-stopped
    environment:
      PORT: 4000
      DB_HOST: db
      DB_PORT: 5432
      DB_USER: postgres
      DB_PASSWORD: ${POSTGRES_PASSWORD}
      DB_NAME: iou_modern
      SLOT_NAME: supabase_realtime
    ports:
      - "4000:4000"
    depends_on:
      db:
        condition: service_healthy

  rest:
    image: postgrest/postgrest:v12.0.1
    restart: unless-stopped
    environment:
      PGRST_DB_URI: postgres://postgres:${POSTGRES_PASSWORD}@db:5432/iou_modern
      PGRST_DB_SCHEMA: public
      PGRST_DB_ANON_ROLE: postgres
    ports:
      - "3001:3000"
    depends_on:
      db:
        condition: service_healthy

volumes:
  postgres_data:
```

**Environment variables:** Create `.env.supabase` file (not in version control):

```bash
POSTGRES_PASSWORD=<secure_password>
JWT_SECRET=<generated_jwt_secret>
STUDIO_PG_RESET_URL=http://localhost:3000/api/reset
```

**Deployment:**
```bash
docker-compose -f docker-compose.supabase.yml up -d
```

---

### Step 2: Create PostgreSQL Schema

**File:** `migrations/001_create_initial_schema.sql`

This migration script creates the PostgreSQL schema equivalent to DuckDB:

```sql
-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- information_domains table
CREATE TABLE information_domains (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_type VARCHAR NOT NULL CHECK (domain_type IN ('Zaak', 'Project', 'Beleid', 'Expertise')),
    title VARCHAR NOT NULL,
    description TEXT,
    parent_id UUID REFERENCES information_domains(id) ON DELETE SET NULL,
    organization_id UUID NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for information_domains
CREATE INDEX idx_information_domains_type ON information_domains(domain_type);
CREATE INDEX idx_information_domains_org ON information_domains(organization_id);
CREATE INDEX idx_information_domains_parent ON information_domains(parent_id);

-- information_objects table
CREATE TABLE information_objects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_id UUID NOT NULL REFERENCES information_domains(id) ON DELETE CASCADE,
    title VARCHAR NOT NULL,
    content TEXT,
    content_vectors REAL[],  -- Vector embeddings ( FLOAT[] in DuckDB -> REAL[] in PostgreSQL)
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for information_objects
CREATE INDEX idx_information_objects_domain ON information_objects(domain_id);
CREATE INDEX idx_information_objects_title ON information_objects(title);
CREATE GIN INDEX idx_information_objects_metadata ON information_objects USING gin(metadata);

-- documents table
CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    information_object_id UUID NOT NULL REFERENCES information_objects(id) ON DELETE CASCADE,
    title VARCHAR NOT NULL,
    document_type VARCHAR NOT NULL,
    status VARCHAR NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'review', 'approved', 'published', 'archived')),
    classification VARCHAR DEFAULT 'internal' CHECK (classification IN ('internal', 'confidential', 'public')),
    woo_published BOOLEAN DEFAULT FALSE,
    woo_publication_date TIMESTAMP WITH TIME ZONE,
    retention_date DATE,
    destruction_date DATE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for documents
CREATE INDEX idx_documents_object ON documents(information_object_id);
CREATE INDEX idx_documents_status ON documents(status);
CREATE INDEX idx_documents_classification ON documents(classification);
CREATE INDEX idx_documents_retention ON documents(retention_date) WHERE retention_date IS NOT NULL;
CREATE INDEX idx_documents_woo ON documents(woo_published) WHERE woo_published = TRUE;
CREATE GIN INDEX idx_documents_metadata ON documents USING gin(metadata);

-- templates table
CREATE TABLE templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description TEXT,
    template_content TEXT NOT NULL,
    variables JSONB DEFAULT '{}',
    organization_id UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for templates
CREATE INDEX idx_templates_org ON templates(organization_id);
CREATE INDEX idx_templates_name ON templates(name);

-- audit_trail table
CREATE TABLE audit_trail (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name VARCHAR NOT NULL CHECK (table_name IN ('information_domains', 'information_objects', 'documents', 'templates')),
    record_id UUID NOT NULL,
    action VARCHAR NOT NULL CHECK (action IN ('INSERT', 'UPDATE', 'DELETE')),
    old_values JSONB,
    new_values JSONB,
    user_id UUID,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for audit_trail
CREATE INDEX idx_audit_trail_table ON audit_trail(table_name);
CREATE INDEX idx_audit_trail_record ON audit_trail(table_name, record_id);
CREATE INDEX idx_audit_trail_timestamp ON audit_trail(timestamp DESC);

-- Views
CREATE VIEW v_searchable_objects AS
SELECT io.id, io.title, io.content, d.title as document_title, 
       d.status, d.classification, io.domain_id
FROM information_objects io
LEFT JOIN documents d ON io.id = d.information_object_id;

CREATE VIEW v_compliance_overview AS
SELECT d.status, d.classification, d.woo_published,
       COUNT(*) as count, MIN(d.retention_date) as earliest_retention
FROM documents d
GROUP BY d.status, d.classification, d.woo_published;

CREATE VIEW v_domain_statistics AS
SELECT domain_type, COUNT(*) as count
FROM information_domains
GROUP BY domain_type;

CREATE VIEW v_entity_network AS
SELECT io.id, io.title, io.metadata->>'entities' as entities
FROM information_objects io
WHERE io.metadata ? 'entities';

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for updated_at
CREATE TRIGGER update_information_domains_updated_at
    BEFORE UPDATE ON information_domains
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_information_objects_updated_at
    BEFORE UPDATE ON information_objects
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_documents_updated_at
    BEFORE UPDATE ON documents
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_templates_updated_at
    BEFORE UPDATE ON templates
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

---

### Step 3: Database Connection from Rust Backend

**File:** `crates/database/src/supabase.rs`

Add Supabase connection support alongside existing DuckDB connection:

```rust
use sqlx::{PgPool, postgres::PgPoolOptions};
use anyhow::Result;

/// Supabase database connection pool
pub struct SupabasePool {
    pool: PgPool,
}

impl SupabasePool {
    /// Create a new Supabase connection pool from DATABASE_URL
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        
        Ok(Self { pool })
    }

    /// Get the underlying sqlx PgPool
    pub fn inner(&self) -> &PgPool {
        &self.pool
    }

    /// Health check for the database connection
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
}
```

**File:** `crates/database/src/lib.rs`

Update the database module to export both connections:

```rust
pub mod duckdb;
pub mod supabase;

pub use duckdb::DuckDbConnection;
pub use supabase::SupabasePool;
```

**Configuration update:** Add to `config/default.toml`:

```toml
[database.supabase]
url = "postgresql://postgres:password@localhost:5432/iou_modern"
enabled = true
```

---

### Step 4: Dual-Write Pattern

**File:** `crates/database/src/dual_write.rs`

Implement a trait-based dual-write system that writes to both databases:

```rust
use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;

use super::{DuckDbConnection, SupabasePool};

/// Read source selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadSource {
    DuckDb,
    Supabase,
}

impl ReadSource {
    /// Get current read source from environment
    pub fn from_env() -> Self {
        match std::env::var("READ_SOURCE").as_deref() {
            Ok("supabase") => ReadSource::Supabase,
            _ => ReadSource::DuckDb,  // Default to DuckDB for safety
        }
    }
}

/// Result of a dual-write operation
#[derive(Debug)]
pub enum DualWriteResult<T> {
    Success(T),
    PartialSuccess {
        duckdb: Option<T>,
        supabase: Option<T>,
        errors: Vec<anyhow::Error>,
    },
    Failed(Vec<anyhow::Error>),
}

impl<T> DualWriteResult<T> {
    /// Check if both writes succeeded
    pub fn is_success(&self) -> bool {
        matches!(self, DualWriteResult::Success(_))
    }

    /// Get the value if successful, or the best available
    pub fn value(self) -> Option<T> {
        match self {
            DualWriteResult::Success(v) => Some(v),
            DualWriteResult::PartialSuccess { duckdb, supabase, .. } => {
                duckdb.or(supabase)
            }
            DualWriteResult::Failed(_) => None,
        }
    }
}

/// Trait for types that support dual-write
#[async_trait]
pub trait DualWrite {
    type Id;
    
    /// Write to DuckDB
    async fn write_to_duckdb(&self, db: &DuckDbConnection) -> Result<Self::Id>;
    
    /// Write to Supabase
    async fn write_to_supabase(&self, db: &SupabasePool) -> Result<Self::Id>;
    
    /// Dual-write to both databases
    async fn dual_write(
        &self,
        duckdb: &DuckDbConnection,
        supabase: &SupabasePool,
    ) -> DualWriteResult<Self::Id> {
        let (duckdb_result, supabase_result) = tokio::join!(
            self.write_to_duckdb(duckdb),
            self.write_to_supabase(supabase)
        );
        
        match (duckdb_result, supabase_result) {
            (Ok(d_id), Ok(s_id)) => {
                // Verify IDs match
                if d_id != s_id {
                    tracing::error!(
                        "ID mismatch: DuckDB={}, Supabase={}",
                        d_id, s_id
                    );
                }
                DualWriteResult::Success(d_id)
            }
            (Ok(d_id), Err(e)) => DualWriteResult::PartialSuccess {
                duckdb: Some(d_id),
                supabase: None,
                errors: vec![e],
            },
            (Err(e), Ok(s_id)) => DualWriteResult::PartialSuccess {
                duckdb: None,
                supabase: Some(s_id),
                errors: vec![e],
            },
            (Err(e1), Err(e2)) => DualWriteResult::Failed(vec![e1, e2]),
        }
    }
}
```

---

### Step 5: Implement Dual-Write for Information Domains

**File:** `crates/database/src/information_domains.rs`

```rust
use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{DuckDbConnection, SupabasePool, DualWrite, DualWriteResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationDomain {
    pub id: Option<Uuid>,
    pub domain_type: DomainType,
    pub title: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub organization_id: Uuid,
    pub metadata: serde_json::Value,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum DomainType {
    Zaak,
    Project,
    Beleid,
    Expertise,
}

#[async_trait]
impl DualWrite for InformationDomain {
    type Id = Uuid;
    
    async fn write_to_duckdb(&self, db: &DuckDbConnection) -> Result<Uuid> {
        let id = self.id.unwrap_or_else(Uuid::new_v4);
        
        sqlx::query!(
            r#"
            INSERT INTO information_domains (id, domain_type, title, description, parent_id, organization_id, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT (id) DO UPDATE SET
                title = excluded.title,
                description = excluded.description,
                parent_id = excluded.parent_id,
                metadata = excluded.metadata,
                updated_at = CURRENT_TIMESTAMP
            "#,
            id, self.domain_type as DomainType, self.title, self.description, self.parent_id, self.organization_id, self.metadata
        )
        .execute(db.inner())
        .await?;
        
        Ok(id)
    }
    
    async fn write_to_supabase(&self, db: &SupabasePool) -> Result<Uuid> {
        let id = self.id.unwrap_or_else(Uuid::new_v4);
        
        sqlx::query!(
            r#"
            INSERT INTO information_domains (id, domain_type, title, description, parent_id, organization_id, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                title = EXCLUDED.title,
                description = EXCLUDED.description,
                parent_id = EXCLUDED.parent_id,
                metadata = EXCLUDED.metadata,
                updated_at = CURRENT_TIMESTAMP
            "#,
            id, self.domain_type as DomainType, &self.title, &self.description, &self.parent_id, &self.organization_id, &self.metadata
        )
        .execute(db.inner())
        .await?;
        
        Ok(id)
    }
}
```

---

### Step 6: Read Toggle Implementation

**File:** `crates/database/src/repository.rs`

Create a repository that reads from either database based on configuration:

```rust
use anyhow::Result;
use uuid::Uuid;
use crate::{ReadSource, DualWriteResult};

/// Repository for reading data based on ReadSource configuration
pub struct InformationDomainRepository {
    read_source: ReadSource,
    duckdb: super::DuckDbConnection,
    supabase: super::SupabasePool,
}

impl InformationDomainRepository {
    pub fn new(
        read_source: ReadSource,
        duckdb: super::DuckDbConnection,
        supabase: super::SupabasePool,
    ) -> Self {
        Self {
            read_source,
            duckdb,
            supabase,
        }
    }
    
    /// Get by ID from configured source
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<super::information_domains::InformationDomain>> {
        match self.read_source {
            ReadSource::DuckDb => self.get_from_duckdb(id).await,
            ReadSource::Supabase => self.get_from_supabase(id).await,
        }
    }
    
    /// Get from DuckDB specifically
    pub async fn get_from_duckdb(&self, id: Uuid) -> Result<Option<super::information_domains::InformationDomain>> {
        // Implementation using DuckDB query
        todo!("Implement DuckDB query")
    }
    
    /// Get from Supabase specifically  
    pub async fn get_from_supabase(&self, id: Uuid) -> Result<Option<super::information_domains::InformationDomain>> {
        // Implementation using Supabase query
        todo!("Implement Supabase query")
    }
    
    /// Compare data between sources for consistency checking
    pub async fn compare_consistency(&self, id: Uuid) -> Result<bool> {
        let duckdb_value = self.get_from_duckdb(id).await?;
        let supabase_value = self.get_from_supabase(id).await?;
        
        // Compare the two values
        match (duckdb_value, supabase_value) {
            (Some(d), Some(s)) => Ok(d == s),
            (None, None) => Ok(true),
            _ => Ok(false),  // One exists, one doesn't - inconsistent
        }
    }
}
```

---

### Step 7: API Integration

**File:** `crates/api/src/handlers/information_domains.rs`

Update API handlers to use dual-write:

```rust
use axum::{Json, Extension};
use uuid::Uuid;

use crate::state::AppState;
use crate::database::{DualWrite, InformationDomain};

pub async fn create_domain(
    Extension(state): Extension<AppState>,
    Json(payload): Json<CreateDomainRequest>,
) -> Result<Json<DomainResponse>, ApiError> {
    let domain = InformationDomain {
        id: None,
        domain_type: payload.domain_type,
        title: payload.title,
        description: payload.description,
        parent_id: payload.parent_id,
        organization_id: payload.organization_id,
        metadata: payload.metadata.unwrap_or_default(),
        created_at: None,
        updated_at: None,
    };
    
    // Dual-write to both databases
    let result = domain.dual_write(&state.duckdb, &state.supabase).await;
    
    match result {
        crate::database::DualWriteResult::Success(id) => {
            Ok(Json(DomainResponse { id }))
        }
        crate::database::DualWriteResult::PartialSuccess { value: Some(id), .. } => {
            // Log warning but continue - one write succeeded
            tracing::warn!("Partial success in dual-write, using ID: {}", id);
            Ok(Json(DomainResponse { id }))
        }
        crate::database::DualWriteResult::Failed(errors) => {
            tracing::error!("Dual-write failed: {:?}", errors);
            Err(ApiError::Internal("Database write failed".into()))
        }
        _ => Err(ApiError::Internal("Unexpected write result".into()))
    }
}
```

---

## Tests

### Schema Equivalence Tests

**File:** `crates/database/tests/schema_equivalence.rs`

```rust
use sqlx::{PgPool, Row};

#[tokio::test]
async fn test_information_domains_schema_matches() {
    // Verify PostgreSQL schema matches DuckDB structure
    let supabase = get_supabase_pool().await;
    
    // Check table exists
    let result = sqlx::query(
        "SELECT EXISTS (SELECT FROM pg_tables WHERE tablename = 'information_domains')"
    )
    .fetch_one(&supabase)
    .await;
    
    assert!(result.is_ok(), "information_domains table should exist");
    
    // Verify columns
    let columns: Vec<String> = sqlx::query(
        "SELECT column_name FROM information_schema.columns 
         WHERE table_name = 'information_domains' 
         ORDER BY ordinal_position"
    )
    .fetch_all(&supabase)
    .await
    .unwrap()
    .into_iter()
    .map(|row| row.get("column_name"))
    .collect();
    
    assert!(columns.contains(&"id".to_string()));
    assert!(columns.contains(&"domain_type".to_string()));
    assert!(columns.contains(&"title".to_string()));
    assert!(columns.contains(&"organization_id".to_string()));
}

#[tokio::test]
async fn test_documents_schema_matches() {
    // Similar test for documents table
}

#[tokio::test]
async fn test_view_equivalence_searchable_objects() {
    // Verify v_searchable_objects produces same results
}
```

### Dual-Write Tests

**File:** `crates/database/tests/dual_write.rs`

```rust
use uuid::Uuid;
use crate::database::{DualWrite, InformationDomain, DomainType};

#[tokio::test]
async fn test_dual_write_consistency() {
    let duckdb = get_duckdb_connection().await;
    let supabase = get_supabase_pool().await;
    
    let domain = InformationDomain {
        id: None,
        domain_type: DomainType::Zaak,
        title: "Test Domain".to_string(),
        description: Some("Test description".to_string()),
        parent_id: None,
        organization_id: Uuid::new_v4(),
        metadata: serde_json::json!({}),
        created_at: None,
        updated_at: None,
    };
    
    let result = domain.dual_write(&duckdb, &supabase).await;
    
    assert!(result.is_success());
    let id = result.value().unwrap();
    
    // Verify record exists in both databases
    let duckdb_record = get_from_duckdb(&duckdb, id).await;
    let supabase_record = get_from_supabase(&supabase, id).await;
    
    assert!(duckdb_record.is_some());
    assert!(supabase_record.is_some());
}

#[tokio::test]
async fn test_partial_failure_handling() {
    // Test behavior when one database fails
    // This requires mocking a database failure
}

#[tokio::test]
async fn test_read_toggle_switching() {
    // Verify READ_SOURCE environment variable correctly switches source
}
```

### Connection Tests

**File:** `crates/database/tests/connection.rs`

```rust
#[tokio::test]
async fn test_supabase_connection() {
    let pool = get_supabase_pool().await;
    let result = pool.health_check().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_duckdb_still_operational() {
    // Verify DuckDB still works after Supabase setup
    let conn = get_duckdb_connection().await;
    // Test query
}
```

### Performance Tests

**File:** `crates/database/benches/dual_write.rs`

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_dual_write(c: &mut Criterion) {
    c.bench_function("dual_write", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                // Perform dual-write and measure latency
            });
    });
}

criterion_group!(benches, benchmark_dual_write);
criterion_main!(benches);
```

---

## Validation Checklist

After completing this section, verify:

- [ ] Docker Compose successfully starts all Supabase services
- [ ] All tables created in PostgreSQL with correct schema
- [ ] All indexes created for performance
- [ ] All views created and return expected results
- [ ] Rust backend can connect to both databases
- [ ] Dual-write succeeds for all entity types
- [ ] Read toggle switches between databases correctly
- [ ] Data consistency verified between databases
- [ ] Performance baseline not degraded (measure with benchmarks)
- [ ] All schema equivalence tests pass
- [ ] All dual-write tests pass

---

## Rollback Procedures

If any validation fails during this phase:

1. **Stop dual-write:** Set `READ_SOURCE=duckdb` environment variable
2. **Stop Supabase:** `docker-compose -f docker-compose.supabase.yml down`
3. **Reconcile data:** Run reconciliation script to ensure DuckDB has all data
4. **Resume normal operation:** Continue using DuckDB-only mode

**Data reconciliation script:**

```bash
# Find records in Supabase not in DuckDB
# This should be integrated into a proper migration tool
```

---

## Next Steps

After completing this section:

1. Review results with stakeholders
2. If validation passes, proceed to **section-03-auth-realtime**
3. If issues found, address before proceeding

---

## Dependencies

- **section-01-assessment** must be complete with:
  - Performance baseline documented
  - Current authentication cataloged
  - Hosting location decided
  - Stakeholder sign-off obtained