# Detailed Design: Metadata Registry Service

> **Template Origin**: Official | **ArcKit Version**: 4.3.1 | **Status**: DRAFT

## Document Control

| Field | Value |
|-------|-------|
| **Document ID** | ARC-002-DLD-v1.0 |
| **Document Type** | Detailed Design |
| **Project** | Metadata Registry Service (Project 002) |
| **Classification** | OFFICIAL |
| **Status** | DRAFT |
| **Version** | 1.0 |
| **Created Date** | 2026-04-19 |
| **Last Modified** | 2026-04-19 |
| **Owner** | Enterprise Architect |
| **Reviewed By** | PENDING |
| **Approved By** | PENDING |

## Revision History

| Version | Date | Author | Changes | Approved By |
|---------|------|--------|---------|-------------|
| 1.0 | 2026-04-19 | ArcKit AI | Initial detailed design | PENDING |

---

## 1. Introduction

### 1.1 Purpose

This document provides the detailed design for the Metadata Registry Service, including component specifications, data structures, interfaces, and implementation guidance for code generation.

### 1.2 Scope

**In Scope**:
- Component-level design for all Rust crates
- ArangoDB schema and collection specifications
- REST/GraphQL API specifications
- Security implementation details
- Integration patterns

**Out of Scope**:
- Physical infrastructure layout
- Operational procedures
- Training materials

### 1.3 Reference Architecture

- **HLD**: ARC-002-HLDR-v1.0
- **Requirements**: ARC-002-REQ-v1.1
- **ADR-001**: Rust language selection
- **ADR-002**: ArangoDB selection
- **ADR-004**: BSW alignment

---

## 2. Component Design

### 2.1 metadata-core Crate

**Purpose**: Shared types, traits, and common functionality

**Dependencies**: None (foundational crate)

**Modules**:
```
metadata-core/
├── src/
│   ├── lib.rs              # Crate root
│   ├── entities.rs         # GGHH entity definitions
│   ├── error.rs            # Error types
│   ├── validation.rs       # Validation traits
│   ├── tijdsdimensie.rs    # Time validity types
│   ├── bewaartermijn.rs    # Retention period types
│   ├── graph.rs            # Graph types
│   ├── status.rs           # Status enums
│   └── models.rs           # Common models
└── Cargo.toml
```

#### Entity Definitions (entities.rs)

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Core GGHH V2 entity trait
pub trait GghhEntity {
    fn key(&self) -> &str;
    fn geldig_vanaf(&self) -> DateTime<Utc>;
    fn geldig_tot(&self) -> DateTime<Utc>;
    fn organisatie_id(&self) -> &str;
    fn is_valid_at(&self, timestamp: DateTime<Utc>) -> bool {
        timestamp >= self.geldig_vanaf() && timestamp < self.geldig_tot()
    }
}

/// Gebeurtenis (Event) - Primary GGHH V2 entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gebeurtenis {
    #[serde(rename = "_key")]
    pub key: String,
    pub naam: String,
    pub omschrijving: Option<String>,
    pub gebeurtenistype: GebeurtenisType,
    pub geldig_vanaf: DateTime<Utc>,
    pub geldig_tot: DateTime<Utc>,
    pub organisatie_id: String,
    pub aangemaakt_door: String,
    pub aangemaakt_op: DateTime<Utc>,
    pub gewijzigd_door: Option<String>,
    pub gewijzigd_op: Option<DateTime<Utc>>,
}

impl GghhEntity for Gebeurtenis {
    fn key(&self) -> &str { &self.key }
    fn geldig_vanaf(&self) -> DateTime<Utc> { self.geldig_vanaf }
    fn geldig_tot(&self) -> DateTime<Utc> { self.geldig_tot }
    fn organisatie_id(&self) -> &str { &self.organisatie_id }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GebeurtenisType {
    Aanvraag,
    Beschikking,
    Melding,
    Taakuitvoering,
    Andere,
}

/// Informatieobject - BSW core abstraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Informatieobject {
    #[serde(rename = "_key")]
    pub key: String,
    pub dataobject_id: String,
    pub naam: String,
    pub omschrijving: Option<String>,
    pub objecttype: String,
    
    // Woo required fields
    pub informatiecategorie: Informatiecategorie,
    pub documenttype: String,
    
    // Security classification
    pub beveiligingsniveau: Beveiligingsniveau,
    pub privacy_level: PrivacyLevel,
    
    // BSW status
    pub status: InformatieobjectStatus,
    
    // Optional references
    pub zaak_id: Option<String>,
    pub samenvatting: Option<String>,
    
    // Time validity
    pub geldig_vanaf: DateTime<Utc>,
    pub geldig_tot: DateTime<Utc>,
    
    // Ownership
    pub organisatie_id: String,
    pub aangemaakt_door: String,
    pub aangemaakt_op: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InformatieobjectStatus {
    Dynamisch,      // In bewerking - mutable
    Gepersistent,   // Read-only
    Gearchiveerd,   // In CDD+
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Informatiecategorie {
    Besluit,
    Rapport,
    Nota,
    Correspondentie,
    Agenda,
    Verslag,
    Andere,
}
```

#### Error Types (error.rs)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("Entity not found: {entity_type}/{key}")]
    EntityNotFound {
        entity_type: String,
        key: String,
    },
    
    #[error("Validation failed: {message}")]
    ValidationFailed { message: String },
    
    #[error("Database error: {0}")]
    Database(#[from] arangors::error::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Time validity conflict: from {from} to {to}")]
    TimeValidityConflict {
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    },
    
    #[error("Unauthorized: {reason}")]
    Unauthorized { reason: String },
    
    #[error("Forbidden: insufficient privileges")]
    Forbidden,
}

pub type Result<T> = std::result::Result<T, MetadataError>;
```

---

### 2.2 metadata-store Crate

**Purpose**: ArangoDB repositories and data access

**Dependencies**: metadata-core, arangors, mobc

**Modules**:
```
metadata-store/
├── src/
│   ├── lib.rs
│   ├── connection.rs        # Connection pool management
│   ├── repositories.rs      # Repository trait definitions
│   ├── gebeurtenis_repo.rs  # Event repository
│   ├── gegevensproduct_repo.rs
│   ├── elementaire_set_repo.rs
│   ├── enkelvoudig_gegeven_repo.rs
│   ├── waarde_repo.rs
│   ├── context_repo.rs
│   ├── grondslag_repo.rs
│   ├── zaak_repo.rs
│   ├── informatieobject_repo.rs
│   ├── audit_repo.rs
│   └── audit_middleware.rs  # Audit logging middleware
└── Cargo.toml
```

#### Repository Pattern (repositories.rs)

```rust
use async_trait::async_trait;
use crate::error::{MetadataError, Result};

#[async_trait]
pub trait Repository<T>: Send + Sync {
    async fn get(&self, key: &str) -> Result<T>;
    async fn list(&self, filter: &RepositoryFilter) -> Result<Vec<T>>;
    async fn create(&self, entity: &T) -> Result<String>;
    async fn update(&self, key: &str, entity: &T) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
}

#[derive(Debug, Clone, Default)]
pub struct RepositoryFilter {
    pub organisatie_id: Option<String>,
    pub geldig_op: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    Asc,
    Desc,
}
```

#### Connection Pool (connection.rs)

```rust
use arangors::Client;
use mobc::{Connection, Manager};
use mobc_arangors::ArangoConnectionManager;
use std::time::Duration;

pub struct DatabasePool {
    pool: mobc::Pool<ArangoConnectionManager>,
}

impl DatabasePool {
    pub async fn new(url: &str, db_name: &str, username: &str, password: &str) 
        -> Result<Self>
    {
        let manager = ArangoConnectionManager::new(
            Client::new_with_auth(url, username, password)?
                .with_db(db_name)
        );
        
        let pool = mobc::Pool::builder()
            .max_open(100)
            .max_idle(10)
            .get_timeout(Duration::from_secs(30))
            .build(manager);
        
        Ok(Self { pool })
    }
    
    pub async fn connection(&self) 
        -> Result<Connection<ArangoConnectionManager>>
    {
        self.pool.get().await
            .map_err(|e| MetadataError::Database(e.into()))
    }
}
```

#### Gebeurtenis Repository (gebeurtenis_repo.rs)

```rust
use crate::entities::Gebeurtenis;
use crate::repositories::{Repository, RepositoryFilter};

pub struct GebeurtenisRepository {
    pool: Arc<DatabasePool>,
}

#[async_trait]
impl Repository<Gebeurtenis> for GebeurtenisRepository {
    async fn get(&self, key: &str) -> Result<Gebeurtenis> {
        let conn = self.pool.connection().await?;
        let db = conn.db();
        
        let doc: Gebeurtenis = db
            .collection("gebeurtenis")
            .document(key)
            .await?;
        
        Ok(doc)
    }
    
    async fn create(&self, entity: &Gebeurtenis) -> Result<String> {
        let conn = self.pool.connection().await?;
        let db = conn.db();
        
        // Validate time validity
        if entity.geldig_tot <= entity.geldig_vanaf {
            return Err(MetadataError::TimeValidityConflict {
                from: entity.geldig_vanaf,
                to: entity.geldig_tot,
            });
        }
        
        let key = db
            .collection("gebeurtenis")
            .create_entity(entity)
            .await?;
        
        Ok(key)
    }
    
    // ... other methods
}
```

---

### 2.3 metadata-api Crate

**Purpose**: REST and GraphQL API servers

**Dependencies**: metadata-core, metadata-store, metadata-validation, actix-web, juniper

**Modules**:
```
metadata-api/
├── src/
│   ├── bin/
│   │   └── main.rs           # API server entry point
│   ├── lib.rs
│   ├── routes.rs             # REST v1/v2 routes
│   ├── routes_v2.rs          # GGHH V2 endpoints
│   ├── graphql.rs            # GraphQL schema
│   ├── graphql_v2.rs         # GGHH V2 GraphQL
│   ├── middleware.rs         # Auth, CORS, etc.
│   ├── handlers.rs           # Request handlers
│   └── tests.rs              # Integration tests
└── Cargo.toml
```

#### REST v2 Routes (routes_v2.rs)

```rust
use actix_web::{web, HttpResponse, Scope};
use crate::handlers::GebeurtenisHandler;

pub fn v2_routes() -> Scope {
    web::scope("/api/v2")
        .service(gebeurtenis_routes())
        .service(gegevensproduct_routes())
        .service(informatieobject_routes())
}

fn gebeurtenis_routes() -> Scope {
    web::scope("/gebeurtenissen")
        .route("", web::get().to(GebeurtenisHandler::list))
        .route("", web::post().to(GebeurtenisHandler::create))
        .route("/{key}", web::get().to(GebeurtenisHandler::get))
        .route("/{key}", web::put().to(GebeurtenisHandler::update))
        .route("/{key}", web::delete().to(GebeurtenisHandler::delete))
}
```

#### Handler Implementation (handlers.rs)

```rust
use actix_web::{web, HttpResponse};
use crate::entities::Gebeurtenis;
use crate::error::MetadataError;

pub struct GebeurtenisHandler {
    repo: Arc<GebeurtenisRepository>,
    validator: Arc<ValidationEngine>,
}

impl GebeurtenisHandler {
    pub async fn create(
        &self,
        entity: web::Json<Gebeurtenis>,
    ) -> Result<HttpResponse, MetadataError> {
        // Validate
        self.validator.validate_entity(&entity).await?;
        
        // Create
        let key = self.repo.create(&entity).await?;
        
        Ok(HttpResponse::Created()
            .json(json!({ "key": key, "message": "Created" })))
    }
    
    pub async fn get(
        &self,
        path: web::Path<String>,
    ) -> Result<HttpResponse, MetadataError> {
        let key = path.into_inner();
        let entity = self.repo.get(&key).await?;
        
        Ok(HttpResponse::Ok().json(entity))
    }
}
```

---

### 2.4 metadata-validation Crate

**Purpose**: Validation engine for GGHH compliance

**Dependencies**: metadata-core, regex

**Modules**:
```
metadata-validation/
├── src/
│   ├── lib.rs
│   ├── validators.rs        # Core validation logic
│   ├── constraints.rs       # Business rule validation
│   ├── schema_validator.rs  # GGHH V2 schema validation
│   ├── status_validator.rs  # Status transitions
│   └── tooi_validator.rs    # TOOI standard validation
└── Cargo.toml
```

#### Validation Engine (validators.rs)

```rust
use crate::entities::GghhEntity;
use crate::error::{MetadataError, Result};

pub struct ValidationEngine {
    schema_validator: SchemaValidator,
    constraint_checker: ConstraintChecker,
    pii_detector: PIIDetector,
}

impl ValidationEngine {
    pub async fn validate_entity<T: GghhEntity + Serialize>(
        &self,
        entity: &T,
    ) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();
        
        // Schema validation
        self.schema_validator.validate(entity)?;
        
        // Constraint validation
        self.constraint_checker.check(entity)?;
        
        // PII detection
        let pii_result = self.pii_detector.detect(entity);
        if pii_result.detected {
            report.add_warning(ValidationWarning::PIIDetected(pii_result));
        }
        
        Ok(report)
    }
}

pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}
```

---

## 3. Data Model

### 3.1 ArangoDB Collections

| Collection | Type | Description | Estimated Size |
|------------|------|-------------|----------------|
| gebeurtenis | Document | Events | 10K-500K |
| gegevensproduct | Document | Data products | 5K-250K |
| elementaire_set | Document | Elementary data sets | 20K-1M |
| enkelvoudig_gegeven | Document | Simple data elements | 100K-5M |
| waarde_met_tijd | Document | Values with time | 1M-50M |
| context | Document | Context metadata | 5K-50K |
| grondslag | Document | Legal basis | 1K-10K |
| zaak | Document | Cases | 10K-1M |
| informatieobject | Document | Information objects | 100K-10M |
| informatieobject_catalogus | Document | Catalog entries | 100K-10M |
| informatieobject_recht | Edge | Object rights | 200K-20M |
| audit | Document | Audit log | 1M-50M |
| user | Document | Users | 1K-10K |
| organisatie | Document | Organizations | 100-500 |
| waardelijst | Document | Value lists | 50-100 |
| waarde | Document | Value list values | 1K-10K |

### 3.2 Edge Collections (29 total)

See ARC-002-DIAG-004 for complete edge collection specifications.

---

## 4. API Specification

### 4.1 REST API v2

Base URL: `https://api.metadata-registry.nl/api/v2`

Authentication: Bearer token (OAuth 2.0)

#### Gebeurtenis Endpoints

```
GET    /api/v2/gebeurtenissen
POST   /api/v2/gebeurtenissen
GET    /api/v2/gebeurtenissen/{key}
PUT    /api/v2/gebeurtenissen/{key}
PATCH  /api/v2/gebeurtenissen/{key}
DELETE /api/v2/gebeurtenissen/{key}
```

#### Query Parameters

```
?organisatie_id={org_id}
?geldig_op={timestamp}
?limit={int}
?offset={int}
?sort_by={field}
?sort_order=asc|desc
```

#### Response Format

```json
{
  "data": [...],
  "pagination": {
    "total": 100,
    "limit": 20,
    "offset": 0
  }
}
```

### 4.2 GraphQL API

Endpoint: `https://api.metadata-registry.nl/graphql`

See ARC-002-API-v1.0 for complete GraphQL schema.

---

## 5. Security Implementation

### 5.1 Authentication Middleware

```rust
use actix_web::{dev::Payload, FromRequest, HttpRequest};
use futures::future::{ready, Ready};

pub struct AuthenticatedUser {
    pub user_id: String,
    pub organisatie_id: String,
    pub roles: Vec<String>,
}

impl FromRequest for AuthenticatedUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization")
            .and_then(|h| h.to_str().ok());
        
        match auth_header {
            Some(token) if token.starts_with("Bearer ") => {
                // Validate JWT token
                match validate_token(&token[7..]) {
                    Ok(claims) => ready(Ok(AuthenticatedUser {
                        user_id: claims.sub,
                        organisatie_id: claims.org_id,
                        roles: claims.roles,
                    })),
                    Err(_) => ready(Err(MetadataError::Unauthorized.into())),
                }
            }
            _ => ready(Err(MetadataError::Unauthorized.into())),
        }
    }
}
```

### 5.2 Row-Level Security

```rust
pub struct RlsFilter {
    pub organisatie_id: String,
    pub user_roles: Vec<String>,
}

impl RlsFilter {
    pub fn apply_aql(&self, aql: &str) -> String {
        format!(
            "{} FILTER doc.organisatie_id == @org_id",
            aql
        )
    }
    
    pub fn can_access(&self, entity: &dyn GghhEntity) -> bool {
        entity.organisatie_id() == self.organisatie_id
    }
}
```

---

## 6. Implementation Guidance

### 6.1 Code Generation Templates

#### Entity Template

```rust
use crate::entities::GghhEntity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {{EntityName}} {
    #[serde(rename = "_key")]
    pub key: String,
    pub naam: String,
    pub omschrijving: Option<String>,
    pub geldig_vanaf: DateTime<Utc>,
    pub geldig_tot: DateTime<Utc>,
    pub organisatie_id: String,
}

impl GghhEntity for {{EntityName}} {
    fn key(&self) -> &str { &self.key }
    fn geldig_vanaf(&self) -> DateTime<Utc> { self.geldig_vanaf }
    fn geldig_tot(&self) -> DateTime<Utc> { self.geldig_tot }
    fn organisatie_id(&self) -> &str { &self.organisatie_id }
}
```

#### Repository Template

```rust
pub struct {{EntityName}}Repository {
    pool: Arc<DatabasePool>,
}

#[async_trait]
impl Repository<{{EntityName}}> for {{EntityName}}Repository {
    async fn get(&self, key: &str) -> Result<{{EntityName}}> {
        // Implementation
    }
    
    async fn create(&self, entity: &{{EntityName}}) -> Result<String> {
        // Implementation
    }
    
    // ... other methods
}
```

### 6.2 Build Configuration

```toml
[workspace.dependencies]
# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Types
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Async
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"

# Database
arangors = { version = "0.6", features = ["rocksdb"] }
mobc = "0.9"
mobc-arangors = "0.2"

# API
actix-web = "4.0"
actix-cors = "0.6"
juniper = "0.15"

# Validation
regex = "1.0"
validator = "0.16"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.22"

# Metrics
prometheus = "0.13"

[profile.release]
lto = true
opt-level = "z"
codegen-units = 1
strip = true
```

---

## 7. Related Documents

- ARC-002-REQ-v1.1: Requirements
- ARC-002-HLDR-v1.0: High-Level Design Review
- ARC-002-API-v1.0: API Design
- ARC-002-DB-v1.0: Database Design
- ARC-002-SEC-v1.0: Security Design
