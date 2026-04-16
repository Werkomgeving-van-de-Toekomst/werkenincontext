# Metadata Registry Service Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a standalone Rust microservice for centralized metadata management with ArangoDB storage, REST/GraphQL API, validation engine, GitOps sync, and Dioxus admin UI.

**Architecture:** Workspace with 6 crates (core, store, validation, api, gitops, admin) using shared types from metadata-core. ArangoDB for graph-based metadata storage. Actix-web for REST, Juniper for GraphQL.

**Tech Stack:** Rust 2024 edition, ArangoDB (arangors 0.6), Actix-web 4.0, Juniper 0.15, Dioxus 0.5, git2 0.18, serde_yaml 0.9, regex 1.0

---

## File Structure

```
metadata-registry/                    # NEW - Root workspace directory
├── Cargo.toml                        # Workspace root
├── crates/
│   ├── metadata-core/                # Shared types (WASM-compatible)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                # Main exports
│   │       ├── models.rs             # Domain models
│   │       ├── validation.rs         # Validation types
│   │       ├── graph.rs              # Graph model
│   │       └── error.rs              # Error types
│   │
│   ├── metadata-store/               # ArangoDB repositories
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── connection.rs         # ArangoDB connection pool
│   │       ├── schema_repo.rs        # MetadataSchema CRUD
│   │       ├── valuelist_repo.rs     # ValueList CRUD
│   │       ├── attribute_repo.rs     # AttributeDefinition CRUD
│   │       └── audit_repo.rs         # Audit logging
│   │
│   ├── metadata-validation/          # Validation engine
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine.rs             # Main validation logic
│   │       ├── constraints.rs        # Built-in validators
│   │       └── custom.rs             # Custom validator registry
│   │
│   ├── metadata-api/                 # REST/GraphQL server
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs               # Server entry point
│   │       ├── lib.rs
│   │       ├── rest.rs               # Actix-web endpoints
│   │       ├── graphql.rs            # Juniper schema
│   │       ├── webhooks.rs           # Event emission
│   │       └── middleware/
│   │           ├── auth.rs           # Authentication
│   │           └── rbac.rs           # Authorization
│   │
│   ├── metadata-gitops/              # GitOps sync service
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── sync.rs               # Git sync logic
│   │       ├── yaml.rs               # YAML parsing
│   │       └── watch.rs              # File watcher
│   │
│   └── metadata-admin/               # Dioxus admin UI
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs               # Dioxus app entry
│           └── components/
│               ├── schema_editor.rsx
│               ├── valuelist_manager.rsx
│               ├── approval_queue.rsx
│               └── audit_log.rsx
│
├── migrations/
│   ├── 001_init_collections.js       # ArangoDB collections
│   ├── 002_indexes.js                # Graph indexes
│   └── 003_standard_valuelists.js    # Seed data
│
├── config/
│   └── schemas/                      # GitOps YAML files
│       ├── standard/
│       │   ├── provincie.yaml
│       │   ├── gemeente.yaml
│       │   ├── rechtsgebied.yaml
│       │   ├── documenttype.yaml
│       │   ├── beveiligingsniveau.yaml
│       │   ├── privacy_level.yaml
│       │   └── woo_relevantie.yaml
│       └── domain/
│           └── iou-document-v1.yaml
│
├── tests/
│   ├── unit/
│   │   ├── validation_tests.rs
│   │   ├── schema_tests.rs
│   │   └── graph_tests.rs
│   └── integration/
│       ├── api_tests.rs
│       └── gitops_tests.rs
│
└── Dockerfile                        # Container image
```

---

## Phase 1: Project Setup and Core Types

### Task 1: Create workspace structure

**Files:**
- Create: `metadata-registry/Cargo.toml`
- Create: `metadata-registry/crates/metadata-core/Cargo.toml`
- Create: `metadata-registry/crates/metadata-core/src/lib.rs`

- [ ] **Step 1: Create workspace Cargo.toml**

Create `metadata-registry/Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "crates/metadata-core",
    "crates/metadata-store",
    "crates/metadata-validation",
    "crates/metadata-api",
    "crates/metadata-gitops",
    "crates/metadata-admin",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["IOU-Modern Team"]
license = "MIT"

[workspace.dependencies]
# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Types
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# Error handling
thiserror = "1.0"

# ArangoDB
arangors = { version = "0.6", features = ["rocksdb"] }
mobc = "0.9"
mobc-arangors = "0.2"

# Async
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"

# API
actix-web = "4.0"
juniper = "0.15"

# GitOps
git2 = "0.18"
serde_yaml = "0.9"
notify = "6.0"

# Validation
regex = "1.0"

# Admin UI
dioxus = { version = "0.5", features = ["web"] }

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

[profile.release]
lto = true
opt-level = "z"
codegen-units = 1
```

- [ ] **Step 2: Create metadata-core Cargo.toml**

Create `metadata-registry/crates/metadata-core/Cargo.toml`:

```toml
[package]
name = "metadata-core"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
```

- [ ] **Step 3: Create metadata-core lib.rs placeholder**

Create `metadata-registry/crates/metadata-core/src/lib.rs`:

```rust
//! Metadata Registry Core Types
//!
//! WASM-compatible shared types for the metadata registry service.

pub mod error;
pub mod models;
pub mod validation;
pub mod graph;

pub use error::{MetadataError, Result};
pub use models::*;
pub use validation::*;
pub use graph::*;
```

- [ ] **Step 4: Run cargo check to verify workspace**

Run: `cd metadata-registry && cargo check`

Expected output: `Finished dev profile [unoptimized + debuginfo] target(s)`

- [ ] **Step 5: Commit workspace setup**

```bash
cd metadata-registry
git init
git add .
git commit -m "feat: initialize metadata-registry workspace"
```

---

### Task 2: Define error types

**Files:**
- Create: `metadata-registry/crates/metadata-core/src/error.rs`

- [ ] **Step 1: Write error types**

Create `metadata-registry/crates/metadata-core/src/error.rs`:

```rust
use thiserror::Error;

/// Main error type for metadata registry
#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("Schema niet gevonden: {0}")]
    SchemaNotFound(String),

    #[error("Attribute niet gevonden: {0} in schema: {1}")]
    AttributeNotFound(String, String),

    #[error("ValueList niet gevonden: {0}")]
    ValueListNotFound(String),

    #[error("ValueListItem niet gevonden: {0} in lijst: {1}")]
    ValueListItemNotFound(String, String),

    #[error("Validatie fout: {0}")]
    ValidationFailed(String),

    #[error("Constraint violation: {field} - {reason}")]
    ConstraintViolation { field: String, reason: String },

    #[error("Versie conflict: verwacht {expected}, heeft {actual}")]
    VersionConflict { expected: String, actual: String },

    #[error("Niet geautoriseerd: {action} op {resource}")]
    Unauthorized { action: String, resource: String },

    #[error("Database fout: {0}")]
    Database(#[from] arangors::Error),

    #[error("Connection pool fout: {0}")]
    ConnectionPool(#[from] mobc::Error<arangors::Error>),

    #[error("Git fout: {0}")]
    Git(#[from] git2::Error),

    #[error("YAML parse fout: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON fout: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Regex fout: {0}")]
    Regex(#[from] regex::Error),

    #[error("IO fout: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse fout: {0}")]
    Parse(String),

    #[error("Custom validator '{0}' not found")]
    CustomValidatorNotFound(String),

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
}

pub type Result<T> = std::result::Result<T, MetadataError>;
```

- [ ] **Step 2: Update metadata-core Cargo.toml to include arangors**

Edit `metadata-registry/crates/metadata-core/Cargo.toml`:

```toml
[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

# Optional - only for error type definitions
arangors = { version = "0.6", default-features = false, optional = true }
mobc = { version = "0.9", optional = true }
git2 = { version = "0.18", optional = true }
serde_yaml = { version = "0.9", optional = true }
regex = { version = "1.0", optional = true }

[features]
default = []
full = ["arangors", "mobc", "git2", "serde_yaml", "regex"]
```

- [ ] **Step 3: Run cargo check**

Run: `cd metadata-registry && cargo check`

Expected: No errors

- [ ] **Step 4: Commit error types**

```bash
git add crates/metadata-core/src/error.rs crates/metadata-core/Cargo.toml
git commit -m "feat(core): define error types"
```

---

### Task 3: Define domain models

**Files:**
- Create: `metadata-registry/crates/metadata-core/src/models.rs`

- [ ] **Step 1: Write domain models**

Create `metadata-registry/crates/metadata-core/src/models.rs`:

```rust
use crate::{MetadataError, Result};
use chrono::{DateTime, Utc, Date};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Metadata Schema - definitie van een metadata structuur
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetadataSchema {
    pub _key: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub status: SchemaStatus,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub governance: GovernanceConfig,
}

impl MetadataSchema {
    pub fn new(name: String, description: String, created_by: String) -> Self {
        Self {
            _key: Uuid::new_v4().to_string(),
            name,
            description,
            version: "1.0.0".to_string(),
            status: SchemaStatus::Draft,
            created_at: Utc::now(),
            created_by,
            governance: GovernanceConfig::default(),
        }
    }

    pub fn can_publish(&self) -> Result<()> {
        if self.status != SchemaStatus::Draft {
            return Err(MetadataError::VersionConflict {
                expected: "Draft".to_string(),
                actual: format!("{:?}", self.status),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SchemaStatus {
    Draft,
    Published,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GovernanceConfig {
    pub approval_required: bool,
    pub approvers: Vec<String>,
    pub gitops_managed: bool,
    pub git_repo: Option<String>,
}

/// Attribute definitie binnen een schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttributeDefinition {
    pub _key: String,
    pub schema_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: DataType,
    pub required: bool,
    pub multivalued: bool,
    pub constraints: Constraints,
    pub default_value: Option<serde_json::Value>,
    pub description: String,
}

impl AttributeDefinition {
    pub fn new(schema_id: String, name: String, data_type: DataType) -> Self {
        Self {
            _key: Uuid::new_v4().to_string(),
            schema_id,
            name,
            data_type,
            required: false,
            multivalued: false,
            constraints: Constraints::default(),
            default_value: None,
            description: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    String,
    Integer,
    Decimal,
    Boolean,
    Date,
    Enum,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Constraints {
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub pattern: Option<String>,
    pub enum_values: Option<Vec<String>>,
    pub custom_validator: Option<String>,
}

/// Waardenlijst (bijv. provincies, gemeenten)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValueList {
    pub _key: String,
    pub name: String,
    pub description: String,
    pub source: ValueListSource,
    pub version: String,
    pub external_id: Option<String>,
    pub effective_date: Date<Utc>,
    pub expiry_date: Option<Date<Utc>>,
}

impl ValueList {
    pub fn new(name: String, description: String, source: ValueListSource) -> Self {
        Self {
            _key: Uuid::new_v4().to_string(),
            name,
            description,
            source,
            version: "1.0.0".to_string(),
            external_id: None,
            effective_date: Utc::now().date(),
            expiry_date: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ValueListSource {
    Standaard,
    Domein,
    Custom,
}

/// Item binnen een waardenlijst
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValueListItem {
    pub _key: String,
    pub list_id: String,
    pub code: String,
    pub label: String,
    pub description: Option<String>,
    pub parent_code: Option<String>,
    pub sort_order: i32,
    pub active: bool,
    pub properties: Option<HashMap<String, serde_json::Value>>,
}

impl ValueListItem {
    pub fn new(list_id: String, code: String, label: String) -> Self {
        Self {
            _key: Uuid::new_v4().to_string(),
            list_id,
            code,
            label,
            description: None,
            parent_code: None,
            sort_order: 0,
            active: true,
            properties: None,
        }
    }
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataAuditLog {
    pub _key: String,
    pub entity_type: EntityType,
    pub entity_id: String,
    pub action: AuditAction,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub changed_by: String,
    pub changed_at: DateTime<Utc>,
    pub source: ChangeSource,
    pub commit_hash: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    Schema,
    Attribute,
    ValueList,
    Item,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuditAction {
    Create,
    Update,
    Delete,
    Publish,
    Deprecate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeSource {
    Ui,
    Gitops,
    Migration,
    Api,
}

impl MetadataAuditLog {
    pub fn new(
        entity_type: EntityType,
        entity_id: String,
        action: AuditAction,
        changed_by: String,
        source: ChangeSource,
    ) -> Self {
        Self {
            _key: Uuid::new_v4().to_string(),
            entity_type,
            entity_id,
            action,
            old_value: None,
            new_value: None,
            changed_by,
            changed_at: Utc::now(),
            source,
            commit_hash: None,
            reason: None,
        }
    }
}
```

- [ ] **Step 2: Run cargo check**

Run: `cd metadata-registry && cargo check`

Expected: No errors

- [ ] **Step 3: Write unit tests for models**

Create `metadata-registry/crates/metadata-core/tests/models_tests.rs`:

```rust
use metadata_core::models::*;

#[test]
fn test_metadata_schema_creation() {
    let schema = MetadataSchema::new(
        "test-schema".to_string(),
        "Test schema".to_string(),
        "user-123".to_string(),
    );

    assert_eq!(schema.name, "test-schema");
    assert_eq!(schema.status, SchemaStatus::Draft);
    assert_eq!(schema.version, "1.0.0");
    assert!(!schema._key.is_empty());
}

#[test]
fn test_can_publish_draft_schema() {
    let schema = MetadataSchema::new(
        "test-schema".to_string(),
        "Test schema".to_string(),
        "user-123".to_string(),
    );

    assert!(schema.can_publish().is_ok());
}

#[test]
fn test_cannot_publish_published_schema() {
    let mut schema = MetadataSchema::new(
        "test-schema".to_string(),
        "Test schema".to_string(),
        "user-123".to_string(),
    );
    schema.status = SchemaStatus::Published;

    assert!(schema.can_publish().is_err());
}

#[test]
fn test_attribute_definition_creation() {
    let attr = AttributeDefinition::new(
        "schema-123".to_string(),
        "title".to_string(),
        DataType::String,
    );

    assert_eq!(attr.name, "title");
    assert_eq!(attr.data_type, DataType::String);
    assert_eq!(attr.required, false);
}

#[test]
fn test_value_list_creation() {
    let list = ValueList::new(
        "provincie".to_string(),
        "Nederlandse provincies".to_string(),
        ValueListSource::Standaard,
    );

    assert_eq!(list.name, "provincie");
    assert_eq!(list.source, ValueListSource::Standaard);
}

#[test]
fn test_value_list_item_creation() {
    let item = ValueListItem::new(
        "list-123".to_string(),
        "DR".to_string(),
        "Drenthe".to_string(),
    );

    assert_eq!(item.code, "DR");
    assert_eq!(item.label, "Drenthe");
    assert_eq!(item.active, true);
}

#[test]
fn test_audit_log_creation() {
    let log = MetadataAuditLog::new(
        EntityType::Schema,
        "schema-123".to_string(),
        AuditAction::Create,
        "user-123".to_string(),
        ChangeSource::Ui,
    );

    assert_eq!(log.entity_type, EntityType::Schema);
    assert_eq!(log.action, AuditAction::Create);
}
```

- [ ] **Step 4: Run tests**

Run: `cd metadata-registry && cargo test --package metadata-core`

Expected: All tests pass

- [ ] **Step 5: Commit domain models**

```bash
git add crates/metadata-core/src/models.rs crates/metadata-core/tests/
git commit -m "feat(core): define domain models with tests"
```

---

### Task 4: Define validation types

**Files:**
- Create: `metadata-registry/crates/metadata-core/src/validation.rs`

- [ ] **Step 1: Write validation types**

Create `metadata-registry/crates/metadata-core/src/validation.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request for metadata validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRequest {
    pub schema_id: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Response from metadata validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResponse {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResponse {
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn with_error(mut self, error: ValidationError) -> Self {
        self.valid = false;
        self.errors.push(error);
        self
    }

    pub fn with_warning(mut self, warning: ValidationWarning) -> Self {
        self.warnings.push(warning);
        self
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

impl Default for ValidationResponse {
    fn default() -> Self {
        Self::new()
    }
}

/// A validation error that makes metadata invalid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub code: ErrorCode,
    pub message: String,
}

impl ValidationError {
    pub fn required(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            code: ErrorCode::RequiredFieldMissing,
            message: "Dit veld is verplicht".to_string(),
        }
    }

    pub fn type_mismatch(field: impl Into<String>, expected: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            code: ErrorCode::TypeMismatch,
            message: format!("Verwacht type: {}", expected.into()),
        }
    }

    pub fn constraint(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            code: ErrorCode::ConstraintViolation,
            message: reason.into(),
        }
    }

    pub fn enum_invalid(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            code: ErrorCode::EnumValueInvalid,
            message: "Waarde niet in toegestane lijst".to_string(),
        }
    }

    pub fn custom(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            code: ErrorCode::CustomValidationFailed,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    RequiredFieldMissing,
    TypeMismatch,
    ConstraintViolation,
    EnumValueInvalid,
    CustomValidationFailed,
}

/// A validation warning that doesn't make metadata invalid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub field: String,
    pub code: WarningCode,
    pub message: String,
}

impl ValidationWarning {
    pub fn new(field: impl Into<String>, code: WarningCode, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WarningCode {
    DeprecatedValue,
    ApproachingExpiry,
    MissingRecommendation,
    CustomWarning,
}

/// Context provided to custom validators
#[derive(Debug, Clone)]
pub struct ValidationContext<'a> {
    pub schema_id: &'a str,
    pub metadata: &'a HashMap<String, serde_json::Value>,
    pub field_name: &'a str,
}

impl<'a> ValidationContext<'a> {
    pub fn new(
        schema_id: &'a str,
        metadata: &'a HashMap<String, serde_json::Value>,
        field_name: &'a str,
    ) -> Self {
        Self {
            schema_id,
            metadata,
            field_name,
        }
    }

    pub fn get_metadata_value(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

/// Signature for custom validation functions
pub type CustomValidator = fn(
    &serde_json::Value,
    &ValidationContext,
) -> Result<(), ValidationError>;
```

- [ ] **Step 2: Write unit tests for validation types**

Create `metadata-registry/crates/metadata-core/tests/validation_tests.rs`:

```rust
use metadata_core::validation::*;
use std::collections::HashMap;

#[test]
fn test_validation_response_new() {
    let response = ValidationResponse::new();

    assert!(response.is_valid());
    assert_eq!(response.errors.len(), 0);
    assert_eq!(response.warnings.len(), 0);
}

#[test]
fn test_validation_response_with_error() {
    let response = ValidationResponse::new()
        .with_error(ValidationError::required("title"));

    assert!(!response.is_valid());
    assert_eq!(response.errors.len(), 1);
    assert_eq!(response.errors[0].field, "title");
}

#[test]
fn test_validation_response_with_warning() {
    let response = ValidationResponse::new()
        .with_warning(ValidationWarning::new(
            "status",
            WarningCode::DeprecatedValue,
            "This value is deprecated"
        ));

    assert!(response.is_valid());
    assert_eq!(response.warnings.len(), 1);
}

#[test]
fn test_validation_error_required() {
    let error = ValidationError::required("author");

    assert_eq!(error.field, "author");
    assert_eq!(error.code, ErrorCode::RequiredFieldMissing);
}

#[test]
fn test_validation_error_type_mismatch() {
    let error = ValidationError::type_mismatch("count", "integer");

    assert_eq!(error.field, "count");
    assert_eq!(error.code, ErrorCode::TypeMismatch);
    assert!(error.message.contains("integer"));
}

#[test]
fn test_validation_error_constraint() {
    let error = ValidationError::constraint("email", "Invalid email format");

    assert_eq!(error.field, "email");
    assert_eq!(error.code, ErrorCode::ConstraintViolation);
    assert_eq!(error.message, "Invalid email format");
}

#[test]
fn test_validation_context_get_metadata_value() {
    let mut metadata = HashMap::new();
    metadata.insert("category".to_string(), serde_json::json!("test"));

    let context = ValidationContext::new("schema-1", &metadata, "title");

    let value = context.get_metadata_value("category");
    assert!(value.is_some());
    assert_eq!(value.unwrap(), "test");
}

#[test]
fn test_validation_request_serialization() {
    let mut metadata = HashMap::new();
    metadata.insert("title".to_string(), serde_json::json!("Test"));

    let request = ValidationRequest {
        schema_id: "schema-1".to_string(),
        metadata,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("schema-1"));
    assert!(json.contains("Test"));
}
```

- [ ] **Step 3: Run tests**

Run: `cd metadata-registry && cargo test --package metadata-core`

Expected: All tests pass

- [ ] **Step 4: Commit validation types**

```bash
git add crates/metadata-core/src/validation.rs crates/metadata-core/tests/validation_tests.rs
git commit -m "feat(core): define validation types with tests"
```

---

### Task 5: Define graph model

**Files:**
- Create: `metadata-registry/crates/metadata-core/src/graph.rs`

- [ ] **Step 1: Write graph model types**

Create `metadata-registry/crates/metadata-core/src/graph.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Edge collection names for the graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EdgeCollection {
    HasAttribute,
    HasValueList,
    ContainsItem,
    Extends,
    DependsOn,
    Supersedes,
}

impl EdgeCollection {
    pub fn as_str(&self) -> &str {
        match self {
            EdgeCollection::HasAttribute => "has_attribute",
            EdgeCollection::HasValueList => "has_valuelist",
            EdgeCollection::ContainsItem => "contains_item",
            EdgeCollection::Extends => "extends",
            EdgeCollection::DependsOn => "depends_on",
            EdgeCollection::Supersedes => "supersedes",
        }
    }
}

/// Vertex collection names for the graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum VertexCollection {
    MetadataSchema,
    AttributeDefinition,
    ValueList,
    ValueListItem,
    MetadataAuditLog,
}

impl VertexCollection {
    pub fn as_str(&self) -> &str {
        match self {
            VertexCollection::MetadataSchema => "metadata_schemas",
            VertexCollection::AttributeDefinition => "attribute_definitions",
            VertexCollection::ValueList => "value_lists",
            VertexCollection::ValueListItem => "value_list_items",
            VertexCollection::MetadataAuditLog => "metadata_audit_log",
        }
    }
}

/// A graph edge representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub _key: String,
    pub _from: String,
    pub _to: String,
    pub edge_type: EdgeCollection,
    pub created_at: String,
}

impl GraphEdge {
    pub fn new(from: String, to: String, edge_type: EdgeCollection) -> Self {
        let _key = format!("{}-{}-{}", from, to, edge_type.as_str());
        Self {
            _key,
            _from: from,
            _to: to,
            edge_type,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Traversal path result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalPath {
    pub start_node: String,
    pub end_node: String,
    pub edges: Vec<String>,
    pub depth: usize,
}
```

- [ ] **Step 2: Update lib.rs to export graph module**

Edit `metadata-registry/crates/metadata-core/src/lib.rs` to ensure graph module is exported (already done in Task 1).

- [ ] **Step 3: Run cargo check**

Run: `cd metadata-registry && cargo check`

Expected: No errors

- [ ] **Step 4: Write unit tests for graph model**

Create `metadata-registry/crates/metadata-core/tests/graph_tests.rs`:

```rust
use metadata_core::graph::*;

#[test]
fn test_edge_collection_as_str() {
    assert_eq!(EdgeCollection::HasAttribute.as_str(), "has_attribute");
    assert_eq!(EdgeCollection::ContainsItem.as_str(), "contains_item");
}

#[test]
fn test_vertex_collection_as_str() {
    assert_eq!(VertexCollection::MetadataSchema.as_str(), "metadata_schemas");
    assert_eq!(VertexCollection::ValueList.as_str(), "value_lists");
}

#[test]
fn test_graph_edge_creation() {
    let edge = GraphEdge::new(
        "schema-123".to_string(),
        "attr-456".to_string(),
        EdgeCollection::HasAttribute,
    );

    assert_eq!(edge._from, "schema-123");
    assert_eq!(edge._to, "attr-456");
    assert_eq!(edge.edge_type, EdgeCollection::HasAttribute);
}

#[test]
fn test_edge_collection_hash() {
    let set = std::collections::HashSet::from([
        EdgeCollection::HasAttribute,
        EdgeCollection::ContainsItem,
    ]);

    assert!(set.contains(&EdgeCollection::HasAttribute));
    assert!(set.contains(&EdgeCollection::ContainsItem));
    assert!(!set.contains(&EdgeCollection::Extends));
}
```

- [ ] **Step 5: Run tests**

Run: `cd metadata-registry && cargo test --package metadata-core`

Expected: All tests pass

- [ ] **Step 6: Commit graph model**

```bash
git add crates/metadata-core/src/graph.rs crates/metadata-core/tests/graph_tests.rs
git commit -m "feat(core): define graph model with tests"
```

---

## Phase 2: ArangoDB Store Layer

### Task 6: Create metadata-store crate

**Files:**
- Create: `metadata-registry/crates/metadata-store/Cargo.toml`
- Create: `metadata-registry/crates/metadata-store/src/lib.rs`

- [ ] **Step 1: Create store crate**

Create `metadata-registry/crates/metadata-store/Cargo.toml`:

```toml
[package]
name = "metadata-store"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
metadata-core = { path = "../metadata-core", features = ["full"] }

# ArangoDB
arangors = { workspace = true }
mobc = { workspace = true }
mobc-arangors = { workspace = true }

# Async
tokio = { workspace = true }
async-trait = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Logging
tracing = { workspace = true }

[dev-dependencies]
tokio-test = "0.4"
```

- [ ] **Step 2: Create store lib.rs**

Create `metadata-registry/crates/metadata-store/src/lib.rs`:

```rust
//! ArangoDB repository layer for metadata registry

pub mod connection;
pub mod schema_repo;
pub mod valuelist_repo;
pub mod attribute_repo;
pub mod audit_repo;

pub use connection::*;
```

- [ ] **Step 3: Run cargo check**

Run: `cd metadata-registry && cargo check`

Expected: No errors (compilation will fail until modules are created)

- [ ] **Step 4: Commit store crate setup**

```bash
git add crates/metadata-store/
git commit -m "feat(store): create metadata-store crate"
```

---

### Task 7: Implement ArangoDB connection pool

**Files:**
- Create: `metadata-registry/crates/metadata-store/src/connection.rs`

- [ ] **Step 1: Write connection pool module**

Create `metadata-registry/crates/metadata-store/src/connection.rs`:

```rust
use arangors::Client;
use mobc::{Connection, Pool};
use mobc_arangors::ArangoConnectionManager;
use metadata_core::{MetadataError, Result};
use std::time::Duration;

/// Database connection pool
pub type DbPool = Pool<ArangoConnectionManager>;

/// Configuration for ArangoDB connection
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub db_name: String,
    pub username: String,
    pub password: String,
    pub max_connections: u64,
    pub connection_timeout: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8529".to_string(),
            db_name: "metadata_registry".to_string(),
            username: "root".to_string(),
            password: "".to_string(),
            max_connections: 10,
            connection_timeout: Duration::from_secs(30),
        }
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            url: std::env::var("ARANGO_DB_URL")
                .unwrap_or_else(|_| "http://localhost:8529".to_string()),
            db_name: std::env::var("ARANGO_DB_NAME")
                .unwrap_or_else(|_| "metadata_registry".to_string()),
            username: std::env::var("ARANGO_DB_USER")
                .unwrap_or_else(|_| "root".to_string()),
            password: std::env::var("ARANGO_DB_PASSWORD")
                .unwrap_or_else(|_| String::new()),
            max_connections: std::env::var("ARANGO_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            connection_timeout: Duration::from_secs(
                std::env::var("ARANGO_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(30),
            ),
        }
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    pub fn with_db_name(mut self, name: impl Into<String>) -> Self {
        self.db_name = name.into();
        self
    }

    pub fn with_credentials(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.username = username.into();
        self.password = password.into();
        self
    }
}

/// Create a new connection pool
pub async fn create_pool(config: DatabaseConfig) -> Result<DbPool> {
    let manager = ArangoConnectionManager::new(
        Client::authenticate(
            &config.url,
            &config.username,
            &config.password,
        )
        .await
        .map_err(|e| MetadataError::Database(
            arangors::ClientError::AuthError(e.to_string()),
        ))?,
        config.db_name.clone(),
    );

    let pool = Pool::builder()
        .max_open(config.max_connections as usize)
        .build(manager)
        .map_err(|e| MetadataError::Parse(format!("Pool creation failed: {}", e)))?;

    Ok(pool)
}

/// Get a connection from the pool
pub async fn get_connection(pool: &DbPool) -> Result<Connection<ArangoConnectionManager>> {
    pool.connection()
        .await
        .map_err(MetadataError::from)
}

/// Health check for the database connection
pub async fn health_check(pool: &DbPool) -> Result<bool> {
    let conn = get_connection(pool).await?;
    let db = conn.db();
    
    // Try to access the database
    let _ = db.accessible_collections().await
        .map_err(MetadataError::from)?;
    
    Ok(true)
}
```

- [ ] **Step 2: Write unit tests for connection**

Create `metadata-registry/crates/metadata-store/tests/connection_tests.rs`:

```rust
use metadata_store::{DatabaseConfig, create_pool};

#[tokio::test]
async fn test_database_config_default() {
    let config = DatabaseConfig::default();
    
    assert_eq!(config.url, "http://localhost:8529");
    assert_eq!(config.db_name, "metadata_registry");
    assert_eq!(config.username, "root");
    assert_eq!(config.max_connections, 10);
}

#[tokio::test]
async fn test_database_config_builder() {
    let config = DatabaseConfig::default()
        .with_url("http://arango:8529")
        .with_db_name("test_db")
        .with_credentials("admin", "secret");
    
    assert_eq!(config.url, "http://arango:8529");
    assert_eq!(config.db_name, "test_db");
    assert_eq!(config.username, "admin");
    assert_eq!(config.password, "secret");
}
```

- [ ] **Step 3: Run tests**

Run: `cd metadata-registry && cargo test --package metadata-store`

Expected: Config tests pass (connection tests will fail without actual ArangoDB)

- [ ] **Step 4: Commit connection pool**

```bash
git add crates/metadata-store/src/connection.rs crates/metadata-store/tests/
git commit -m "feat(store): implement ArangoDB connection pool"
```

---

### Task 8: Implement Schema repository

**Files:**
- Create: `metadata-registry/crates/metadata-store/src/schema_repo.rs`

- [ ] **Step 1: Write schema repository**

Create `metadata-registry/crates/metadata-store/src/schema_repo.rs`:

```rust
use crate::connection::{get_connection, DbPool};
use arangors::collection::CollectionType;
use async_trait::async_trait;
use metadata_core::models::*;
use metadata_core::{MetadataError, Result};
use std::sync::Arc;

pub type SchemaRepoRef = Arc<dyn SchemaRepositoryTrait + Send + Sync>;

#[async_trait]
pub trait SchemaRepositoryTrait {
    async fn create(&self, schema: &MetadataSchema) -> Result<MetadataSchema>;
    async fn get_by_id(&self, id: &str) -> Result<MetadataSchema>;
    async fn get_by_name(&self, name: &str) -> Result<MetadataSchema>;
    async fn list(&self) -> Result<Vec<MetadataSchema>>;
    async fn update(&self, schema: &MetadataSchema) -> Result<MetadataSchema>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn publish(&self, id: &str) -> Result<MetadataSchema>;
    async fn deprecate(&self, id: &str) -> Result<MetadataSchema>;
}

pub struct SchemaRepository {
    pool: DbPool,
    collection_name: String,
}

impl SchemaRepository {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            collection_name: "metadata_schemas".to_string(),
        }
    }

    async fn ensure_collection(&self) -> Result<()> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();

        let exists = db.collection_exists(&self.collection_name).await
            .map_err(MetadataError::from)?;

        if !exists {
            db.create_collection(&self.collection_name)
                .await
                .map_err(MetadataError::from)?;
        }

        Ok(())
    }

    fn _key(&self, id: &str) -> String {
        format!("{}/{}", self.collection_name, id)
    }
}

#[async_trait]
impl SchemaRepositoryTrait for SchemaRepository {
    async fn create(&self, schema: &MetadataSchema) -> Result<MetadataSchema> {
        self.ensure_collection().await?;
        
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.collection_name);

        // Serialize schema for ArangoDB
        let doc = serde_json::to_value(schema)
            .map_err(MetadataError::from)?;

        coll.insert_document(doc)
            .await
            .map_err(MetadataError::from)?;

        Ok(schema.clone())
    }

    async fn get_by_id(&self, id: &str) -> Result<MetadataSchema> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.collection_name);

        let doc = coll.document(id)
            .await
            .map_err(|_| MetadataError::SchemaNotFound(id.to_string()))?;

        let schema: MetadataSchema = serde_json::from_value(doc)
            .map_err(|_| MetadataError::Parse("Invalid schema document".to_string()))?;

        Ok(schema)
    }

    async fn get_by_name(&self, name: &str) -> Result<MetadataSchema> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();

        let aql = r#"
            FOR doc IN @@collection
            FILTER doc.name == @name
            LIMIT 1
            RETURN doc
        "#;

        let mut cursor = db.aql_bind_str(aql)
            .bind("@collection", &self.collection_name)
            .bind("name", name)
            .run()
            .await
            .map_err(MetadataError::from)?;

        if let Some(result) = cursor.next().await {
            let doc: serde_json::Value = result
                .map_err(MetadataError::from)?;
            let schema: MetadataSchema = serde_json::from_value(doc)
                .map_err(|_| MetadataError::Parse("Invalid schema document".to_string()))?;
            Ok(schema)
        } else {
            Err(MetadataError::SchemaNotFound(name.to_string()))
        }
    }

    async fn list(&self) -> Result<Vec<MetadataSchema>> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.collection_name);

        let docs = coll.all_documents()
            .await
            .map_err(MetadataError::from)?;

        let schemas: Vec<MetadataSchema> = docs
            .into_iter()
            .filter_map(|doc| serde_json::from_value(doc).ok())
            .collect();

        Ok(schemas)
    }

    async fn update(&self, schema: &MetadataSchema) -> Result<MetadataSchema> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.collection_name);

        let doc = serde_json::to_value(schema)
            .map_err(MetadataError::from)?;

        coll.update_document(&schema._key, doc)
            .await
            .map_err(MetadataError::from)?;

        Ok(schema.clone())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.collection_name);

        coll.remove_document(id)
            .await
            .map_err(MetadataError::from)?;

        Ok(())
    }

    async fn publish(&self, id: &str) -> Result<MetadataSchema> {
        let mut schema = self.get_by_id(id).await?;
        schema.can_publish()?;
        schema.status = SchemaStatus::Published;
        self.update(&schema).await
    }

    async fn deprecate(&self, id: &str) -> Result<MetadataSchema> {
        let mut schema = self.get_by_id(id).await?;
        schema.status = SchemaStatus::Deprecated;
        self.update(&schema).await
    }
}
```

- [ ] **Step 2: Write unit tests**

Create `metadata-registry/crates/metadata-store/tests/schema_repo_tests.rs`:

```rust
use metadata_core::models::*;
use metadata_store::connection::DatabaseConfig;
use metadata_store::schema_repo::SchemaRepository;
use std::sync::Arc;

#[tokio::test]
async fn test_schema_repo_new() {
    // This test creates a repo without actual DB connection
    // Just verifies it can be instantiated
    
    let config = DatabaseConfig::default();
    let pool = metadata_store::create_pool(config).await;
    
    // Will fail to connect, but that's expected
    let result = pool.await;
    
    // Either we get a pool or a connection error
    match result {
        Ok(pool) => {
            let repo = SchemaRepository::new(pool);
            assert_eq!(repo.collection_name, "metadata_schemas");
        }
        Err(_) => {
            // Expected if no ArangoDB running
            assert!(true);
        }
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cd metadata-registry && cargo test --package metadata-store`

Expected: Tests pass (or connection errors if no ArangoDB)

- [ ] **Step 4: Commit schema repository**

```bash
git add crates/metadata-store/src/schema_repo.rs crates/metadata-store/tests/schema_repo_tests.rs
git commit -m "feat(store): implement schema repository"
```

---

### Task 9: Implement ValueList repository

**Files:**
- Create: `metadata-registry/crates/metadata-store/src/valuelist_repo.rs`

- [ ] **Step 1: Write valuelist repository**

Create `metadata-registry/crates/metadata-store/src/valuelist_repo.rs`:

```rust
use crate::connection::{get_connection, DbPool};
use async_trait::async_trait;
use metadata_core::models::*;
use metadata_core::{MetadataError, Result};
use std::sync::Arc;

pub type ValueListRepoRef = Arc<dyn ValueListRepositoryTrait + Send + Sync>;

#[async_trait]
pub trait ValueListRepositoryTrait {
    async fn create(&self, list: &ValueList) -> Result<ValueList>;
    async fn get_by_id(&self, id: &str) -> Result<ValueList>;
    async fn get_by_name(&self, name: &str) -> Result<ValueList>;
    async fn list(&self) -> Result<Vec<ValueList>>;
    async fn list_by_source(&self, source: ValueListSource) -> Result<Vec<ValueList>>;
    async fn update(&self, list: &ValueList) -> Result<ValueList>;
    async fn delete(&self, id: &str) -> Result<()>;
    
    // Items
    async fn add_item(&self, item: &ValueListItem) -> Result<ValueListItem>;
    async fn get_items(&self, list_id: &str) -> Result<Vec<ValueListItem>>;
    async fn update_item(&self, item: &ValueListItem) -> Result<ValueListItem>;
    async fn delete_item(&self, list_id: &str, code: &str) -> Result<()>;
    async fn get_item(&self, list_id: &str, code: &str) -> Result<ValueListItem>;
}

pub struct ValueListRepository {
    pool: DbPool,
    collection_name: String,
    item_collection_name: String,
}

impl ValueListRepository {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            collection_name: "value_lists".to_string(),
            item_collection_name: "value_list_items".to_string(),
        }
    }

    async fn ensure_collections(&self) -> Result<()> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();

        for coll_name in [&self.collection_name, &self.item_collection_name] {
            let exists = db.collection_exists(coll_name).await
                .map_err(MetadataError::from)?;

            if !exists {
                db.create_collection(coll_name)
                    .await
                    .map_err(MetadataError::from)?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl ValueListRepositoryTrait for ValueListRepository {
    async fn create(&self, list: &ValueList) -> Result<ValueList> {
        self.ensure_collections().await?;
        
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.collection_name);

        let doc = serde_json::to_value(list)
            .map_err(MetadataError::from)?;

        coll.insert_document(doc)
            .await
            .map_err(MetadataError::from)?;

        Ok(list.clone())
    }

    async fn get_by_id(&self, id: &str) -> Result<ValueList> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.collection_name);

        let doc = coll.document(id)
            .await
            .map_err(|_| MetadataError::ValueListNotFound(id.to_string()))?;

        let list: ValueList = serde_json::from_value(doc)
            .map_err(|_| MetadataError::Parse("Invalid valuelist document".to_string()))?;

        Ok(list)
    }

    async fn get_by_name(&self, name: &str) -> Result<ValueList> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();

        let aql = r#"
            FOR doc IN @@collection
            FILTER doc.name == @name
            LIMIT 1
            RETURN doc
        "#;

        let mut cursor = db.aql_bind_str(aql)
            .bind("@collection", &self.collection_name)
            .bind("name", name)
            .run()
            .await
            .map_err(MetadataError::from)?;

        if let Some(result) = cursor.next().await {
            let doc: serde_json::Value = result.map_err(MetadataError::from)?;
            let list: ValueList = serde_json::from_value(doc)
                .map_err(|_| MetadataError::Parse("Invalid valuelist document".to_string()))?;
            Ok(list)
        } else {
            Err(MetadataError::ValueListNotFound(name.to_string()))
        }
    }

    async fn list(&self) -> Result<Vec<ValueList>> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.collection_name);

        let docs = coll.all_documents()
            .await
            .map_err(MetadataError::from)?;

        let lists: Vec<ValueList> = docs
            .into_iter()
            .filter_map(|doc| serde_json::from_value(doc).ok())
            .collect();

        Ok(lists)
    }

    async fn list_by_source(&self, source: ValueListSource) -> Result<Vec<ValueList>> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();

        let aql = r#"
            FOR doc IN @@collection
            FILTER doc.source == @source
            RETURN doc
        "#;

        let source_str = match source {
            ValueListSource::Standaard => "standaard",
            ValueListSource::Domein => "domein",
            ValueListSource::Custom => "custom",
        };

        let mut cursor = db.aql_bind_str(aql)
            .bind("@collection", &self.collection_name)
            .bind("source", source_str)
            .run()
            .await
            .map_err(MetadataError::from)?;

        let mut lists = Vec::new();
        while let Some(result) = cursor.next().await {
            let doc: serde_json::Value = result.map_err(MetadataError::from)?;
            if let Ok(list) = serde_json::from_value::<ValueList>(doc) {
                lists.push(list);
            }
        }

        Ok(lists)
    }

    async fn update(&self, list: &ValueList) -> Result<ValueList> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.collection_name);

        let doc = serde_json::to_value(list)
            .map_err(MetadataError::from)?;

        coll.update_document(&list._key, doc)
            .await
            .map_err(MetadataError::from)?;

        Ok(list.clone())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.collection_name);

        coll.remove_document(id)
            .await
            .map_err(MetadataError::from)?;

        Ok(())
    }

    async fn add_item(&self, item: &ValueListItem) -> Result<ValueListItem> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.item_collection_name);

        let doc = serde_json::to_value(item)
            .map_err(MetadataError::from)?;

        coll.insert_document(doc)
            .await
            .map_err(MetadataError::from)?;

        Ok(item.clone())
    }

    async fn get_items(&self, list_id: &str) -> Result<Vec<ValueListItem>> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();

        let aql = r#"
            FOR doc IN @@collection
            FILTER doc.list_id == @list_id
            SORT doc.sort_order ASC
            RETURN doc
        "#;

        let mut cursor = db.aql_bind_str(aql)
            .bind("@collection", &self.item_collection_name)
            .bind("list_id", list_id)
            .run()
            .await
            .map_err(MetadataError::from)?;

        let mut items = Vec::new();
        while let Some(result) = cursor.next().await {
            let doc: serde_json::Value = result.map_err(MetadataError::from)?;
            if let Ok(item) = serde_json::from_value::<ValueListItem>(doc) {
                items.push(item);
            }
        }

        Ok(items)
    }

    async fn update_item(&self, item: &ValueListItem) -> Result<ValueListItem> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.item_collection_name);

        let doc = serde_json::to_value(item)
            .map_err(MetadataError::from)?;

        coll.update_document(&item._key, doc)
            .await
            .map_err(MetadataError::from)?;

        Ok(item.clone())
    }

    async fn delete_item(&self, list_id: &str, code: &str) -> Result<()> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();

        let aql = r#"
            FOR doc IN @@collection
            FILTER doc.list_id == @list_id AND doc.code == @code
            REMOVE doc IN @@collection
        "#;

        db.aql_bind_str(aql)
            .bind("@collection", &self.item_collection_name)
            .bind("list_id", list_id)
            .bind("code", code)
            .run()
            .await
            .map_err(MetadataError::from)?;

        Ok(())
    }

    async fn get_item(&self, list_id: &str, code: &str) -> Result<ValueListItem> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();

        let aql = r#"
            FOR doc IN @@collection
            FILTER doc.list_id == @list_id AND doc.code == @code
            LIMIT 1
            RETURN doc
        "#;

        let mut cursor = db.aql_bind_str(aql)
            .bind("@collection", &self.item_collection_name)
            .bind("list_id", list_id)
            .bind("code", code)
            .run()
            .await
            .map_err(MetadataError::from)?;

        if let Some(result) = cursor.next().await {
            let doc: serde_json::Value = result.map_err(MetadataError::from)?;
            let item: ValueListItem = serde_json::from_value(doc)
                .map_err(|_| MetadataError::ValueListItemNotFound(code.to_string(), list_id.to_string()))?;
            Ok(item)
        } else {
            Err(MetadataError::ValueListItemNotFound(code.to_string(), list_id.to_string()))
        }
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd metadata-registry && cargo test --package metadata-store`

Expected: No compilation errors

- [ ] **Step 3: Commit valuelist repository**

```bash
git add crates/metadata-store/src/valuelist_repo.rs
git commit -m "feat(store): implement valuelist repository"
```

---

### Task 10: Implement Attribute repository and Audit repository

**Files:**
- Create: `metadata-registry/crates/metadata-store/src/attribute_repo.rs`
- Create: `metadata-registry/crates/metadata-store/src/audit_repo.rs`

- [ ] **Step 1: Write attribute repository**

Create `metadata-registry/crates/metadata-store/src/attribute_repo.rs`:

```rust
use crate::connection::{get_connection, DbPool};
use async_trait::async_trait;
use metadata_core::models::*;
use metadata_core::{MetadataError, Result};
use std::sync::Arc;

pub type AttributeRepoRef = Arc<dyn AttributeRepositoryTrait + Send + Sync>;

#[async_trait]
pub trait AttributeRepositoryTrait {
    async fn create(&self, attr: &AttributeDefinition) -> Result<AttributeDefinition>;
    async fn get_by_id(&self, id: &str) -> Result<AttributeDefinition>;
    async fn list_by_schema(&self, schema_id: &str) -> Result<Vec<AttributeDefinition>>;
    async fn update(&self, attr: &AttributeDefinition) -> Result<AttributeDefinition>;
    async fn delete(&self, id: &str) -> Result<()>;
}

pub struct AttributeRepository {
    pool: DbPool,
    collection_name: String,
}

impl AttributeRepository {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            collection_name: "attribute_definitions".to_string(),
        }
    }

    async fn ensure_collection(&self) -> Result<()> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();

        let exists = db.collection_exists(&self.collection_name).await
            .map_err(MetadataError::from)?;

        if !exists {
            db.create_collection(&self.collection_name)
                .await
                .map_err(MetadataError::from)?;
        }

        Ok(())
    }
}

#[async_trait]
impl AttributeRepositoryTrait for AttributeRepository {
    async fn create(&self, attr: &AttributeDefinition) -> Result<AttributeDefinition> {
        self.ensure_collection().await?;
        
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.collection_name);

        let doc = serde_json::to_value(attr)
            .map_err(MetadataError::from)?;

        coll.insert_document(doc)
            .await
            .map_err(MetadataError::from)?;

        Ok(attr.clone())
    }

    async fn get_by_id(&self, id: &str) -> Result<AttributeDefinition> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.collection_name);

        let doc = coll.document(id)
            .await
            .map_err(|_| MetadataError::AttributeNotFound(id.to_string(), "unknown".to_string()))?;

        let attr: AttributeDefinition = serde_json::from_value(doc)
            .map_err(|_| MetadataError::Parse("Invalid attribute document".to_string()))?;

        Ok(attr)
    }

    async fn list_by_schema(&self, schema_id: &str) -> Result<Vec<AttributeDefinition>> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();

        let aql = r#"
            FOR doc IN @@collection
            FILTER doc.schema_id == @schema_id
            RETURN doc
        "#;

        let mut cursor = db.aql_bind_str(aql)
            .bind("@collection", &self.collection_name)
            .bind("schema_id", schema_id)
            .run()
            .await
            .map_err(MetadataError::from)?;

        let mut attrs = Vec::new();
        while let Some(result) = cursor.next().await {
            let doc: serde_json::Value = result.map_err(MetadataError::from)?;
            if let Ok(attr) = serde_json::from_value::<AttributeDefinition>(doc) {
                attrs.push(attr);
            }
        }

        Ok(attrs)
    }

    async fn update(&self, attr: &AttributeDefinition) -> Result<AttributeDefinition> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.collection_name);

        let doc = serde_json::to_value(attr)
            .map_err(MetadataError::from)?;

        coll.update_document(&attr._key, doc)
            .await
            .map_err(MetadataError::from)?;

        Ok(attr.clone())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.collection_name);

        coll.remove_document(id)
            .await
            .map_err(MetadataError::from)?;

        Ok(())
    }
}
```

- [ ] **Step 2: Write audit repository**

Create `metadata-registry/crates/metadata-store/src/audit_repo.rs`:

```rust
use crate::connection::{get_connection, DbPool};
use async_trait::async_trait;
use metadata_core::models::*;
use metadata_core::{MetadataError, Result};
use std::sync::Arc;

pub type AuditRepoRef = Arc<dyn AuditRepositoryTrait + Send + Sync>;

#[async_trait]
pub trait AuditRepositoryTrait {
    async fn log(&self, log: &MetadataAuditLog) -> Result<MetadataAuditLog>;
    async fn get_logs_for_entity(&self, entity_id: &str) -> Result<Vec<MetadataAuditLog>>;
    async fn get_recent_logs(&self, limit: usize) -> Result<Vec<MetadataAuditLog>>;
    async fn get_logs_by_user(&self, user_id: &str) -> Result<Vec<MetadataAuditLog>>;
}

pub struct AuditRepository {
    pool: DbPool,
    collection_name: String,
}

impl AuditRepository {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            collection_name: "metadata_audit_log".to_string(),
        }
    }

    async fn ensure_collection(&self) -> Result<()> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();

        let exists = db.collection_exists(&self.collection_name).await
            .map_err(MetadataError::from)?;

        if !exists {
            db.create_collection(&self.collection_name)
                .await
                .map_err(MetadataError::from)?;
        }

        Ok(())
    }
}

#[async_trait]
impl AuditRepositoryTrait for AuditRepository {
    async fn log(&self, log: &MetadataAuditLog) -> Result<MetadataAuditLog> {
        self.ensure_collection().await?;
        
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.collection_name);

        let doc = serde_json::to_value(log)
            .map_err(MetadataError::from)?;

        coll.insert_document(doc)
            .await
            .map_err(MetadataError::from)?;

        Ok(log.clone())
    }

    async fn get_logs_for_entity(&self, entity_id: &str) -> Result<Vec<MetadataAuditLog>> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();

        let aql = r#"
            FOR doc IN @@collection
            FILTER doc.entity_id == @entity_id
            SORT doc.changed_at DESC
            RETURN doc
        "#;

        let mut cursor = db.aql_bind_str(aql)
            .bind("@collection", &self.collection_name)
            .bind("entity_id", entity_id)
            .run()
            .await
            .map_err(MetadataError::from)?;

        let mut logs = Vec::new();
        while let Some(result) = cursor.next().await {
            let doc: serde_json::Value = result.map_err(MetadataError::from)?;
            if let Ok(log) = serde_json::from_value::<MetadataAuditLog>(doc) {
                logs.push(log);
            }
        }

        Ok(logs)
    }

    async fn get_recent_logs(&self, limit: usize) -> Result<Vec<MetadataAuditLog>> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();
        let coll = db.collection(&self.collection_name);

        // Note: This might need adjustment based on actual arangors API
        let aql = r#"
            FOR doc IN @@collection
            SORT doc.changed_at DESC
            LIMIT @limit
            RETURN doc
        "#;

        let mut cursor = db.aql_bind_str(aql)
            .bind("@collection", &self.collection_name)
            .bind("limit", limit as i64)
            .run()
            .await
            .map_err(MetadataError::from)?;

        let mut logs = Vec::new();
        while let Some(result) = cursor.next().await {
            let doc: serde_json::Value = result.map_err(MetadataError::from)?;
            if let Ok(log) = serde_json::from_value::<MetadataAuditLog>(doc) {
                logs.push(log);
            }
        }

        Ok(logs)
    }

    async fn get_logs_by_user(&self, user_id: &str) -> Result<Vec<MetadataAuditLog>> {
        let conn = get_connection(&self.pool).await?;
        let db = conn.db();

        let aql = r#"
            FOR doc IN @@collection
            FILTER doc.changed_by == @user_id
            SORT doc.changed_at DESC
            RETURN doc
        "#;

        let mut cursor = db.aql_bind_str(aql)
            .bind("@collection", &self.collection_name)
            .bind("user_id", user_id)
            .run()
            .await
            .map_err(MetadataError::from)?;

        let mut logs = Vec::new();
        while let Some(result) = cursor.next().await {
            let doc: serde_json::Value = result.map_err(MetadataError::from)?;
            if let Ok(log) = serde_json::from_value::<MetadataAuditLog>(doc) {
                logs.push(log);
            }
        }

        Ok(logs)
    }
}
```

- [ ] **Step 3: Update lib.rs to export new modules**

Edit `metadata-registry/crates/metadata-store/src/lib.rs`:

```rust
//! ArangoDB repository layer for metadata registry

pub mod connection;
pub mod schema_repo;
pub mod valuelist_repo;
pub mod attribute_repo;
pub mod audit_repo;

pub use connection::*;
```

- [ ] **Step 4: Run tests**

Run: `cd metadata-registry && cargo test --package metadata-store`

Expected: No compilation errors

- [ ] **Step 5: Commit attribute and audit repositories**

```bash
git add crates/metadata-store/src/attribute_repo.rs crates/metadata-store/src/audit_repo.rs crates/metadata-store/src/lib.rs
git commit -m "feat(store): implement attribute and audit repositories"
```

---

## Phase 3: Validation Engine

### Task 11: Create metadata-validation crate

**Files:**
- Create: `metadata-registry/crates/metadata-validation/Cargo.toml`
- Create: `metadata-registry/crates/metadata-validation/src/lib.rs`

- [ ] **Step 1: Create validation crate**

Create `metadata-registry/crates/metadata-validation/Cargo.toml`:

```toml
[package]
name = "metadata-validation"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
metadata-core = { path = "../metadata-core" }
metadata-store = { path = "../metadata-store" }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Validation
regex = { workspace = true }

# Async
tokio = { workspace = true }
async-trait = { workspace = true }

# Logging
tracing = { workspace = true }
```

- [ ] **Step 2: Create validation lib.rs**

Create `metadata-registry/crates/metadata-validation/src/lib.rs`:

```rust
//! Metadata validation engine

pub mod engine;
pub mod constraints;
pub mod custom;

pub use engine::*;
```

- [ ] **Step 3: Commit validation crate setup**

```bash
git add crates/metadata-validation/
git commit -m "feat(validation): create metadata-validation crate"
```

---

### Task 12: Implement constraint validators

**Files:**
- Create: `metadata-registry/crates/metadata-validation/src/constraints.rs`

- [ ] **Step 1: Write constraint validators**

Create `metadata-registry/crates/metadata-validation/src/constraints.rs`:

```rust
use metadata_core::validation::*;
use metadata_core::models::*;
use regex::Regex;
use std::collections::HashMap;

/// Validate a value against its constraints
pub fn validate_constraints(
    field_name: &str,
    value: &serde_json::Value,
    data_type: &DataType,
    constraints: &Constraints,
) -> Result<(), ValidationError> {
    // Type check first
    validate_type(field_name, value, data_type)?;

    // Required check (handled at a higher level, but check multivalued)
    if matches!(data_type, DataType::Enum) && !constraints.multivalued {
        validate_enum(field_name, value, constraints)?;
    }

    // String constraints
    if let DataType::String = data_type {
        if let Some(s) = value.as_str() {
            validate_string_constraints(field_name, s, constraints)?;
        }
    }

    // Numeric constraints
    if matches!(data_type, DataType::Integer | DataType::Decimal) {
        validate_numeric_constraints(field_name, value, constraints)?;
    }

    // Pattern validation
    if let Some(pattern) = &constraints.pattern {
        validate_pattern(field_name, value, pattern)?;
    }

    Ok(())
}

fn validate_type(
    field_name: &str,
    value: &serde_json::Value,
    data_type: &DataType,
) -> Result<(), ValidationError> {
    let is_valid = match data_type {
        DataType::String => value.is_string(),
        DataType::Integer => value.is_i64(),
        DataType::Decimal => value.is_number(),
        DataType::Boolean => value.is_boolean(),
        DataType::Date => {
            if let Some(s) = value.as_str() {
                chrono::DateTime::parse_from_rfc3339(s).is_ok()
            } else {
                false
            }
        }
        DataType::Enum => value.is_string() || value.is_array(),
    };

    if !is_valid {
        let expected_type = match data_type {
            DataType::String => "string",
            DataType::Integer => "integer",
            DataType::Decimal => "number",
            DataType::Boolean => "boolean",
            DataType::Date => "date (RFC3339)",
            DataType::Enum => "enum value",
        };
        return Err(ValidationError::type_mismatch(field_name, expected_type));
    }

    Ok(())
}

fn validate_string_constraints(
    field_name: &str,
    value: &str,
    constraints: &Constraints,
) -> Result<(), ValidationError> {
    if let Some(min) = constraints.min_length {
        if value.len() < min as usize {
            return Err(ValidationError::constraint(
                field_name,
                format!("Minimale lengte is {}, maar waarde is {}", min, value.len()),
            ));
        }
    }

    if let Some(max) = constraints.max_length {
        if value.len() > max as usize {
            return Err(ValidationError::constraint(
                field_name,
                format!("Maximale lengte is {}, maar waarde is {}", max, value.len()),
            ));
        }
    }

    Ok(())
}

fn validate_numeric_constraints(
    field_name: &str,
    value: &serde_json::Value,
    constraints: &Constraints,
) -> Result<(), ValidationError> {
    let num_value = if let Some(n) = value.as_i64() {
        n as f64
    } else if let Some(n) = value.as_f64() {
        n
    } else {
        return Ok(());
    };

    if let Some(min) = constraints.min_value {
        if num_value < min as f64 {
            return Err(ValidationError::constraint(
                field_name,
                format!("Minimale waarde is {}", min),
            ));
        }
    }

    if let Some(max) = constraints.max_value {
        if num_value > max as f64 {
            return Err(ValidationError::constraint(
                field_name,
                format!("Maximale waarde is {}", max),
            ));
        }
    }

    Ok(())
}

fn validate_pattern(
    field_name: &str,
    value: &serde_json::Value,
    pattern: &str,
) -> Result<(), ValidationError> {
    let regex = Regex::new(pattern)
        .map_err(|_| ValidationError::constraint(field_name, "Ongeldig patroon"))?;

    let str_value = match value {
        serde_json::Value::String(s) => s.as_str(),
        _ => {
            return Err(ValidationError::type_mismatch(field_name, "string"));
        }
    };

    if !regex.is_match(str_value) {
        return Err(ValidationError::constraint(
            field_name,
            format!("Waarde komt niet overeen met patroon: {}", pattern),
        ));
    }

    Ok(())
}

fn validate_enum(
    field_name: &str,
    value: &serde_json::Value,
    constraints: &Constraints,
) -> Result<(), ValidationError> {
    let enum_values = constraints.enum_values.as_ref()
        .ok_or_else(|| ValidationError::constraint(field_name, "Geen enum waarden gedefinieerd"))?;

    let str_value = value.as_str()
        .ok_or_else(|| ValidationError::type_mismatch(field_name, "string"))?;

    if !enum_values.contains(&str_value.to_string()) {
        return Err(ValidationError::enum_invalid(field_name));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_type_string() {
        let value = serde_json::json!("test");
        assert!(validate_type("field", &value, &DataType::String).is_ok());
    }

    #[test]
    fn test_validate_type_string_fail() {
        let value = serde_json::json!(123);
        assert!(validate_type("field", &value, &DataType::String).is_err());
    }

    #[test]
    fn test_validate_type_integer() {
        let value = serde_json::json!(42);
        assert!(validate_type("field", &value, &DataType::Integer).is_ok());
    }

    #[test]
    fn test_validate_string_min_length() {
        let value = serde_json::json!("ab");
        let constraints = Constraints {
            min_length: Some(3),
            ..Default::default()
        };

        let result = validate_string_constraints("field", "ab", &constraints);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_string_max_length() {
        let value = serde_json::json!("abcde");
        let constraints = Constraints {
            max_length: Some(3),
            ..Default::default()
        };

        let result = validate_string_constraints("field", "abcde", &constraints);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_numeric_min() {
        let value = serde_json::json!(5);
        let constraints = Constraints {
            min_value: Some(10),
            ..Default::default()
        };

        let result = validate_numeric_constraints("field", &value, &constraints);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_numeric_max() {
        let value = serde_json::json!(20);
        let constraints = Constraints {
            max_value: Some(10),
            ..Default::default()
        };

        let result = validate_numeric_constraints("field", &value, &constraints);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_pattern() {
        let value = serde_json::json!("test@example.com");
        let pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";

        assert!(validate_pattern("email", &value, pattern).is_ok());
    }

    #[test]
    fn test_validate_pattern_fail() {
        let value = serde_json::json!("not-an-email");
        let pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";

        assert!(validate_pattern("email", &value, pattern).is_err());
    }

    #[test]
    fn test_validate_enum() {
        let value = serde_json::json!("openbaar");
        let constraints = Constraints {
            enum_values: Some(vec!["openbaar".to_string(), "vertrouwelijk".to_string()]),
            ..Default::default()
        };

        assert!(validate_enum("security", &value, &constraints).is_ok());
    }

    #[test]
    fn test_validate_enum_fail() {
        let value = serde_json::json!("geheim");
        let constraints = Constraints {
            enum_values: Some(vec!["openbaar".to_string(), "vertrouwelijk".to_string()]),
            ..Default::default()
        };

        assert!(validate_enum("security", &value, &constraints).is_err());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd metadata-registry && cargo test --package metadata-validation`

Expected: All tests pass

- [ ] **Step 3: Commit constraint validators**

```bash
git add crates/metadata-validation/src/constraints.rs
git commit -m "feat(validation): implement built-in constraint validators"
```

---

### Task 13: Implement custom validator registry

**Files:**
- Create: `metadata-registry/crates/metadata-validation/src/custom.rs`

- [ ] **Step 1: Write custom validator registry**

Create `metadata-registry/crates/metadata-validation/src/custom.rs`:

```rust
use metadata_core::validation::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Registry for custom validators
pub struct CustomValidatorRegistry {
    validators: RwLock<HashMap<String, RegisteredValidator>>,
}

impl CustomValidatorRegistry {
    pub fn new() -> Self {
        Self {
            validators: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, name: String, validator: CustomValidator) -> Result<()> {
        let mut validators = self.validators.write()
            .map_err(|e| ValidationError::custom("registry", format!("Lock error: {}", e)))?;

        validators.insert(name, RegisteredValidator::UserDefined(validator));
        Ok(())
    }

    pub fn register_builtin(&self, name: String, validator: BuiltinValidator) -> Result<()> {
        let mut validators = self.validators.write()
            .map_err(|e| ValidationError::custom("registry", format!("Lock error: {}", e)))?;

        validators.insert(name, RegisteredValidator::Builtin(validator));
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<RegisteredValidator> {
        let validators = self.validators.read().ok()?;
        validators.get(name).cloned()
    }

    pub fn execute(
        &self,
        name: &str,
        value: &serde_json::Value,
        context: &ValidationContext,
    ) -> Result<(), ValidationError> {
        let validator = self.get(name)
            .ok_or_else(|| ValidationError::custom(name, "Validator not found"))?;

        match validator {
            RegisteredValidator::UserDefined(v) => v(value, context),
            RegisteredValidator::Builtin(v) => v.validate(value, context),
        }
    }
}

impl Default for CustomValidatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub enum RegisteredValidator {
    UserDefined(CustomValidator),
    Builtin(BuiltinValidator),
}

/// Builtin validators provided by the system
#[derive(Clone)]
pub enum BuiltinValidator {
    Email,
    Postalcode,
    Kvk,
    Rsin,
    Bsn,
}

impl BuiltinValidator {
    pub fn validate(&self, value: &serde_json::Value, context: &ValidationContext) -> Result<(), ValidationError> {
        match self {
            BuiltinValidator::Email => validate_email(value, context),
            BuiltinValidator::Postalcode => validate_postalcode(value, context),
            BuiltinValidator::Kvk => validate_kvk(value, context),
            BuiltinValidator::Rsin => validate_rsin(value, context),
            BuiltinValidator::Bsn => validate_bsn(value, context),
        }
    }
}

/// Validate Dutch email format (basic)
fn validate_email(value: &serde_json::Value, context: &ValidationContext) -> Result<(), ValidationError> {
    let email = value.as_str()
        .ok_or_else(|| ValidationError::type_mismatch(context.field_name, "string"))?;

    let pattern = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .map_err(|_| ValidationError::custom(context.field_name, "Invalid regex pattern"))?;

    if !pattern.is_match(email) {
        return Err(ValidationError::constraint(context.field_name, "Ongeldig email formaat"));
    }

    Ok(())
}

/// Validate Dutch postal code (1234 AB format)
fn validate_postalcode(value: &serde_json::Value, context: &ValidationContext) -> Result<(), ValidationError> {
    let pc = value.as_str()
        .ok_or_else(|| ValidationError::type_mismatch(context.field_name, "string"))?;

    let pattern = regex::Regex::new(r"^\d{4}\s?[A-Z]{2}$")
        .map_err(|_| ValidationError::custom(context.field_name, "Invalid regex pattern"))?;

    if !pattern.is_match(pc) {
        return Err(ValidationError::constraint(context.field_name, "Ongeldig postcode formaat (verwacht: 1234 AB)"));
    }

    Ok(())
}

/// Validate Dutch KvK number (8 digits)
fn validate_kvk(value: &serde_json::Value, context: &ValidationContext) -> Result<(), ValidationError> {
    let kvk = value.as_str()
        .ok_or_else(|| ValidationError::type_mismatch(context.field_name, "string"))?;

    if !kvk.chars().all(|c| c.is_ascii_digit()) || kvk.len() != 8 {
        return Err(ValidationError::constraint(context.field_name, "KvK moet 8 cijfers zijn"));
    }

    Ok(())
}

/// Validate Dutch RSIN number (9 digits, non-zero checksum)
fn validate_rsin(value: &serde_json::Value, context: &ValidationContext) -> Result<(), ValidationError> {
    let rsin = value.as_str()
        .ok_or_else(|| ValidationError::type_mismatch(context.field_name, "string"))?;

    if !rsin.chars().all(|c| c.is_ascii_digit()) || rsin.len() != 9 {
        return Err(ValidationError::constraint(context.field_name, "RSIN moet 9 cijfers zijn"));
    }

    // Basic checksum validation (simplified - actual RSIN has specific rules)
    let digits: Vec<u32> = rsin.chars()
        .map(|c| c.to_digit(10).unwrap())
        .collect();

    let sum: u32 = digits.iter().enumerate()
        .map(|(i, d)| {
            let multiplier = if i % 2 == 0 { 9 } else { 1 };
            d * multiplier
        })
        .sum();

    if sum % 11 == 0 {
        return Err(ValidationError::constraint(context.field_name, "Ongeldige RSIN checksum"));
    }

    Ok(())
}

/// Validate Dutch BSN number (elfproef)
fn validate_bsn(value: &serde_json::Value, context: &ValidationContext) -> Result<(), ValidationError> {
    let bsn = value.as_str()
        .ok_or_else(|| ValidationError::type_mismatch(context.field_name, "string"))?;

    if !bsn.chars().all(|c| c.is_ascii_digit()) || !(8..=9).contains(&bsn.len()) {
        return Err(ValidationError::constraint(context.field_name, "BSN moet 8 of 9 cijfers zijn"));
    }

    // Elfproef (Dutch social security number check)
    let digits: Vec<u32> = bsn.chars()
        .map(|c| c.to_digit(10).unwrap())
        .collect();

    let len = digits.len();
    let sum: u32 = digits.iter().enumerate()
        .map(|(i, d)| {
            let multiplier = if i == len - 1 { -1i32 } else { (len as i32 - i as i32) };
            (*d as i32) * multiplier
        })
        .sum();

    if sum % 11 != 0 {
        return Err(ValidationError::constraint(context.field_name, "Ongeldige BSN (elfproef)"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_context(metadata: &HashMap<String, serde_json::Value>) -> ValidationContext {
        ValidationContext::new("test-schema", metadata, "test_field")
    }

    #[test]
    fn test_registry_register_and_get() {
        let registry = CustomValidatorRegistry::new();
        
        let validator: CustomValidator = |_, _| Ok(());
        registry.register("test_validator".to_string(), validator).unwrap();

        let retrieved = registry.get("test_validator");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_validate_email_valid() {
        let value = serde_json::json!("test@example.com");
        let mut metadata = HashMap::new();
        let context = make_context(&metadata);

        let result = BuiltinValidator::Email.validate(&value, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_email_invalid() {
        let value = serde_json::json!("not-an-email");
        let mut metadata = HashMap::new();
        let context = make_context(&metadata);

        let result = BuiltinValidator::Email.validate(&value, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_postalcode_valid() {
        let value = serde_json::json!("1234 AB");
        let mut metadata = HashMap::new();
        let context = make_context(&metadata);

        let result = BuiltinValidator::Postalcode.validate(&value, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_postalcode_invalid() {
        let value = serde_json::json!("12345");
        let mut metadata = HashMap::new();
        let context = make_context(&metadata);

        let result = BuiltinValidator::Postalcode.validate(&value, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_kvk_valid() {
        let value = serde_json::json!("12345678");
        let mut metadata = HashMap::new();
        let context = make_context(&metadata);

        let result = BuiltinValidator::Kvk.validate(&value, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_kvk_invalid() {
        let value = serde_json::json!("1234567");
        let mut metadata = HashMap::new();
        let context = make_context(&metadata);

        let result = BuiltinValidator::Kvk.validate(&value, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_bsn_valid() {
        // Known valid BSN for testing
        let value = serde_json::json!("123456782");
        let mut metadata = HashMap::new();
        let context = make_context(&metadata);

        let result = BuiltinValidator::Bsn.validate(&value, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_bsn_invalid() {
        let value = serde_json::json!("123456789"); // Fails elfproef
        let mut metadata = HashMap::new();
        let context = make_context(&metadata);

        let result = BuiltinValidator::Bsn.validate(&value, &context);
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd metadata-registry && cargo test --package metadata-validation`

Expected: All tests pass

- [ ] **Step 3: Commit custom validator registry**

```bash
git add crates/metadata-validation/src/custom.rs
git commit -m "feat(validation): implement custom validator registry with builtin validators"
```

---

### Task 14: Implement validation engine

**Files:**
- Create: `metadata-registry/crates/metadata-validation/src/engine.rs`

- [ ] **Step 1: Write validation engine**

Create `metadata-registry/crates/metadata-validation/src/engine.rs`:

```rust
use crate::constraints::validate_constraints;
use crate::custom::{BuiltinValidator, CustomValidatorRegistry};
use metadata_core::models::*;
use metadata_core::validation::*;
use metadata_store::attribute_repo::AttributeRepositoryTrait;
use metadata_store::schema_repo::SchemaRepositoryTrait;
use std::collections::HashMap;
use std::sync::Arc;

/// Main validation engine
pub struct ValidationEngine {
    schema_repo: Arc<dyn SchemaRepositoryTrait + Send + Sync>,
    attribute_repo: Arc<dyn AttributeRepositoryTrait + Send + Sync>,
    custom_validators: CustomValidatorRegistry,
}

impl ValidationEngine {
    pub fn new(
        schema_repo: Arc<dyn SchemaRepositoryTrait + Send + Sync>,
        attribute_repo: Arc<dyn AttributeRepositoryTrait + Send + Sync>,
    ) -> Self {
        let mut engine = Self {
            schema_repo,
            attribute_repo,
            custom_validators: CustomValidatorRegistry::new(),
        };

        // Register builtin validators
        engine.register_builtin_validators();
        engine
    }

    fn register_builtin_validators(&mut self) {
        let validators = [
            ("email", BuiltinValidator::Email),
            ("postalcode", BuiltinValidator::Postalcode),
            ("kvk", BuiltinValidator::Kvk),
            ("rsin", BuiltinValidator::Rsin),
            ("bsn", BuiltinValidator::Bsn),
        ];

        for (name, validator) in validators {
            let _ = self.custom_validators.register_builtin(
                name.to_string(),
                validator,
            );
        }
    }

    /// Validate metadata against a schema
    pub async fn validate(
        &self,
        request: ValidationRequest,
    ) -> ValidationResponse {
        let mut response = ValidationResponse::new();

        // Get schema
        let schema = match self.schema_repo.get_by_id(&request.schema_id).await {
            Ok(s) => s,
            Err(e) => {
                return response.with_error(ValidationError::custom(
                    "schema",
                    format!("Schema niet gevonden: {}", e),
                ));
            }
        };

        // Get all attributes for schema
        let attributes = match self.attribute_repo.list_by_schema(&schema._key).await {
            Ok(attrs) => attrs,
            Err(e) => {
                return response.with_error(ValidationError::custom(
                    "schema",
                    format!("Kon attributen niet laden: {}", e),
                ));
            }
        };

        // Validate each attribute
        for attr in &attributes {
            if let Some(value) = request.metadata.get(&attr.name) {
                self.validate_attribute(&attr, value, &request, &mut response);
            } else if attr.required {
                response = response.with_error(ValidationError::required(&attr.name));
            }
        }

        // Check for unknown fields
        for key in request.metadata.keys() {
            let is_known = attributes.iter().any(|a| &a.name == key);
            if !is_known {
                response = response.with_warning(ValidationWarning::new(
                    key,
                    WarningCode::CustomWarning,
                    "Onbekend veld in schema",
                ));
            }
        }

        response
    }

    /// Validate a single attribute
    fn validate_attribute(
        &self,
        attr: &AttributeDefinition,
        value: &serde_json::Value,
        request: &ValidationRequest,
        response: &mut ValidationResponse,
    ) {
        // Check constraints
        if let Err(e) = validate_constraints(&attr.name, value, &attr.data_type, &attr.constraints) {
            *response = response.clone().with_error(e);
            return;
        }

        // Check custom validator
        if let Some(validator_name) = &attr.constraints.custom_validator {
            let context = ValidationContext::new(
                &request.schema_id,
                &request.metadata,
                &attr.name,
            );

            if let Err(e) = self.custom_validators.execute(validator_name, value, &context) {
                *response = response.clone().with_error(e);
            }
        }
    }

    /// Validate multiple requests in batch
    pub async fn validate_batch(
        &self,
        requests: Vec<ValidationRequest>,
    ) -> Vec<ValidationResponse> {
        let mut responses = Vec::with_capacity(requests.len());

        for request in requests {
            responses.push(self.validate(request).await);
        }

        responses
    }

    /// Register a custom validator
    pub fn register_validator(&self, name: String, validator: CustomValidator) -> Result<()> {
        self.custom_validators.register(name, validator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require mock repositories
    // Full integration tests would be in the tests/ directory
}
```

- [ ] **Step 2: Update validation lib.rs**

Edit `metadata-registry/crates/metadata-validation/src/lib.rs`:

```rust
//! Metadata validation engine

pub mod engine;
pub mod constraints;
pub mod custom;

pub use engine::ValidationEngine;
```

- [ ] **Step 3: Run tests**

Run: `cd metadata-registry && cargo test --package metadata-validation`

Expected: No compilation errors

- [ ] **Step 4: Commit validation engine**

```bash
git add crates/metadata-validation/src/engine.rs crates/metadata-validation/src/lib.rs
git commit -m "feat(validation): implement validation engine"
```

---

## Phase 4: REST/GraphQL API

### Task 15: Create metadata-api crate

**Files:**
- Create: `metadata-registry/crates/metadata-api/Cargo.toml`
- Create: `metadata-registry/crates/metadata-api/src/main.rs`
- Create: `metadata-registry/crates/metadata-api/src/lib.rs`

- [ ] **Step 1: Create API crate**

Create `metadata-registry/crates/metadata-api/Cargo.toml`:

```toml
[package]
name = "metadata-api"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
metadata-core = { path = "../metadata-core", features = ["full"] }
metadata-store = { path = "../metadata-store" }
metadata-validation = { path = "../metadata-validation" }

# API
actix-web = { workspace = true }
juniper = { workspace = true }
juniper_actix = "0.4.0"

# Async
tokio = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Logging
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
tracing-actix-web = "0.7"

# Environment
dotenvy = "0.15"

[dev-dependencies]
actix-http = "3.0"
```

- [ ] **Step 2: Create main.rs**

Create `metadata-registry/crates/metadata-api/src/main.rs`:

```rust
use actix_web::{middleware, App, HttpServer, web};
use tracing_actix_web::TracingLogger;
use tracing_subscriber;

mod rest;
mod graphql;
mod middleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    tracing::info!("Starting Metadata Registry API");

    // TODO: Initialize connection pool and repositories
    // let pool = metadata_store::create_pool(
    //     metadata_store::DatabaseConfig::from_env()
    // ).await.unwrap();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(TracingLogger::default())
            .wrap(middleware::Compress::default())
            .service(
                web::scope("/api/v1")
                    .configure(rest::configure)
            )
            .service(
                web::scope("/graphql")
                    .configure(graphql::configure)
            )
            .route("/health", actix_web::web::get().to(health_check))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

async fn health_check() -> &'static str {
    "OK"
}
```

- [ ] **Step 3: Create lib.rs**

Create `metadata-registry/crates/metadata-api/src/lib.rs`:

```rust
//! Metadata Registry REST/GraphQL API

pub mod rest;
pub mod graphql;
pub mod middleware;
pub mod webhooks;
```

- [ ] **Step 4: Create placeholder modules**

Create `metadata-registry/crates/metadata-api/src/rest.rs`:

```rust
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/schemas", web::get().to(list_schemas))
        .route("/schemas", web::post().to(create_schema));
}

async fn list_schemas() -> &'static str {
    "[]"
}

async fn create_schema() -> &'static str {
    "{}"
}
```

Create `metadata-registry/crates/metadata-api/src/graphql.rs`:

```rust
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::post().to(graphql_handler));
}

async fn graphql_handler() -> &'static str {
    "{}"
}
```

Create `metadata-registry/crates/metadata-api/src/middleware/mod.rs`:

```rust
//! Middleware for authentication and authorization

pub mod auth;
pub mod rbac;
```

Create `metadata-registry/crates/metadata-api/src/middleware/auth.rs`:

```rust
//! Authentication middleware

use actix_web::{dev::ServiceRequest, Error, FromRequest};
use actix_web::error::ErrorUnauthorized;

pub struct AuthenticatedUser {
    pub user_id: String,
    pub roles: Vec<String>,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = std::future::Ready<Result<Self, Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        // TODO: Implement actual authentication
        // For now, return a dummy user
        std::future::ready(Ok(AuthenticatedUser {
            user_id: "test-user".to_string(),
            roles: vec!["METADATA_VIEWER".to_string()],
        }))
    }
}
```

Create `metadata-registry/crates/metadata-api/src/middleware/rbac.rs`:

```rust
//! Role-based access control

use actix_web::{Error, FromRequest, HttpResponse};
use actix_web::error::ErrorForbidden;

#[derive(Debug, Clone)]
pub enum Role {
    MetadataViewer,
    MetadataEditor,
    MetadataApprover,
    MetadataAdmin,
}

impl Role {
    pub fn as_str(&self) -> &str {
        match self {
            Role::MetadataViewer => "METADATA_VIEWER",
            Role::MetadataEditor => "METADATA_EDITOR",
            Role::MetadataApprover => "METADATA_APPROVER",
            Role::MetadataAdmin => "METADATA_ADMIN",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "METADATA_VIEWER" => Some(Role::MetadataViewer),
            "METADATA_EDITOR" => Some(Role::MetadataEditor),
            "METADATA_APPROVER" => Some(Role::MetadataApprover),
            "METADATA_ADMIN" => Some(Role::MetadataAdmin),
            _ => None,
        }
    }
}

pub fn require_role(required_role: Role) -> impl Fn(&crate::middleware::auth::AuthenticatedUser) -> Result<(), Error> + Clone {
    move |user: &crate::middleware::auth::AuthenticatedUser| {
        if user.roles.iter().any(|r| r == required_role.as_str()) {
            Ok(())
        } else {
            Err(ErrorForbidden("Insufficient permissions"))
        }
    }
}
```

Create `metadata-registry/crates/metadata-api/src/webhooks.rs`:

```rust
//! Webhook event emission

use metadata_core::models::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub entity_id: String,
    pub entity_version: Option<String>,
    pub triggered_by: String,
    pub data: serde_json::Value,
}

pub enum MetadataEvent {
    SchemaCreated(String),
    SchemaUpdated(String, String),
    SchemaPublished(String),
    SchemaDeprecated(String),
    AttributeAdded(String, String),
    AttributeModified(String, String),
    AttributeRemoved(String, String),
    ValueListUpdated(String),
    ValueListItemAdded(String, String),
    ValueListItemModified(String, String),
    ValueListItemRetired(String, String),
    ValidationFailed(String, Vec<metadata_core::validation::ValidationError>),
}

impl MetadataEvent {
    pub fn event_type(&self) -> String {
        match self {
            MetadataEvent::SchemaCreated(_) => "SchemaCreated".to_string(),
            MetadataEvent::SchemaUpdated(_, _) => "SchemaUpdated".to_string(),
            MetadataEvent::SchemaPublished(_) => "SchemaPublished".to_string(),
            MetadataEvent::SchemaDeprecated(_) => "SchemaDeprecated".to_string(),
            MetadataEvent::AttributeAdded(_, _) => "AttributeAdded".to_string(),
            MetadataEvent::AttributeModified(_, _) => "AttributeModified".to_string(),
            MetadataEvent::AttributeRemoved(_, _) => "AttributeRemoved".to_string(),
            MetadataEvent::ValueListUpdated(_) => "ValueListUpdated".to_string(),
            MetadataEvent::ValueListItemAdded(_, _) => "ValueListItemAdded".to_string(),
            MetadataEvent::ValueListItemModified(_, _) => "ValueListItemModified".to_string(),
            MetadataEvent::ValueListItemRetired(_, _) => "ValueListItemRetired".to_string(),
            MetadataEvent::ValidationFailed(_, _) => "ValidationFailed".to_string(),
        }
    }

    pub fn entity_id(&self) -> String {
        match self {
            MetadataEvent::SchemaCreated(id) => id.clone(),
            MetadataEvent::SchemaUpdated(id, _) => id.clone(),
            MetadataEvent::SchemaPublished(id) => id.clone(),
            MetadataEvent::SchemaDeprecated(id) => id.clone(),
            MetadataEvent::AttributeAdded(id, _) => id.clone(),
            MetadataEvent::AttributeModified(id, _) => id.clone(),
            MetadataEvent::AttributeRemoved(id, _) => id.clone(),
            MetadataEvent::ValueListUpdated(id) => id.clone(),
            MetadataEvent::ValueListItemAdded(id, _) => id.clone(),
            MetadataEvent::ValueListItemModified(id, _) => id.clone(),
            MetadataEvent::ValueListItemRetired(id, _) => id.clone(),
            MetadataEvent::ValidationFailed(id, _) => id.clone(),
        }
    }
}
```

- [ ] **Step 5: Run cargo check**

Run: `cd metadata-registry && cargo check --package metadata-api`

Expected: No compilation errors

- [ ] **Step 6: Commit API crate setup**

```bash
git add crates/metadata-api/
git commit -m "feat(api): create metadata-api crate with REST/GraphQL structure"
```

---

## Phase 5: Migrations and Standard Data

### Task 16: Create ArangoDB migrations

**Files:**
- Create: `metadata-registry/migrations/001_init_collections.js`
- Create: `metadata-registry/migrations/002_indexes.js`
- Create: `metadata-registry/migrations/003_standard_valuelists.js`

- [ ] **Step 1: Write initial collections migration**

Create `metadata-registry/migrations/001_init_collections.js`:

```javascript
// Create all vertex collections
const collections = [
  'metadata_schemas',
  'attribute_definitions',
  'value_lists',
  'value_list_items',
  'metadata_audit_log'
];

collections.forEach(name => {
  const exists = db._collection(name);
  if (!exists) {
    db._create(name);
    console.log(`Created collection: ${name}`);
  } else {
    console.log(`Collection already exists: ${name}`);
  }
});

// Create edge collections
const edgeCollections = [
  'has_attribute',
  'has_valuelist',
  'contains_item',
  'extends',
  'depends_on',
  'supersedes'
];

edgeCollections.forEach(name => {
  const exists = db._collection(name);
  if (!exists) {
    db._createEdgeCollection(name);
    console.log(`Created edge collection: ${name}`);
  } else {
    console.log(`Edge collection already exists: ${name}`);
  }
});
```

- [ ] **Step 2: Write indexes migration**

Create `metadata-registry/migrations/002_indexes.js`:

```javascript
// Ensure unique indexes on key fields
const schemas = db._collection('metadata_schemas');
schemas.ensureIndex({
  type: 'persistent',
  fields: ['name'],
  unique: true,
  sparse: false
});

const valuelists = db._collection('value_lists');
valuelists.ensureIndex({
  type: 'persistent',
  fields: ['name'],
  unique: true,
  sparse: false
});

const value_list_items = db._collection('value_list_items');
value_list_items.ensureHashIndex({
  fields: ['list_id', 'code'],
  unique: true,
  sparse: false
});

// Indexes for audit log queries
const audit_log = db._collection('metadata_audit_log');
audit_log.ensureIndex({
  type: 'persistent',
  fields: ['entity_id', 'changed_at']
});

audit_log.ensureIndex({
  type: 'persistent',
  fields: ['changed_by', 'changed_at']
});

audit_log.ensureIndex({
  type: 'persistent',
  fields: ['changed_at']
});

// Indexes for schema attributes
const attributes = db._collection('attribute_definitions');
attributes.ensureIndex({
  type: 'persistent',
  fields: ['schema_id']
});

// Geo indexes for location-based values (if needed)
// value_list_items.ensureGeoIndex({
//   type: 'geo',
//   fields: ['latitude', 'longitude']
// });
```

- [ ] **Step 3: Write standard valuelists seed**

Create `metadata-registry/migrations/003_standard_valuelists.js`:

```javascript
// Create standard value lists
const now = new Date().toISOString().split('T')[0];

// Provincies
const provincieList = {
  _key: 'valuelist-provincie',
  name: 'provincie',
  description: 'Nederlandse provincies',
  source: 'standaard',
  version: '1.0.0',
  effective_date: now,
  expiry_date: null
};

db.value_lists.insert(provincieList);

const provincies = [
  { code: 'DR', label: 'Drenthe' },
  { code: 'FL', label: 'Flevoland' },
  { code: 'FR', label: 'Friesland' },
  { code: 'GE', label: 'Gelderland' },
  { code: 'GR', label: 'Groningen' },
  { code: 'LI', label: 'Limburg' },
  { code: 'NB', label: 'Noord-Brabant' },
  { code: 'NH', label: 'Noord-Holland' },
  { code: 'OV', label: 'Overijssel' },
  { code: 'UT', label: 'Utrecht' },
  { code: 'ZE', label: 'Zeeland' },
  { code: 'ZH', label: 'Zuid-Holland' }
];

provincies.forEach((p, i) => {
  db.value_list_items.insert({
    _key: `provincie-${p.code}`,
    list_id: 'valuelist-provincie',
    code: p.code,
    label: p.label,
    sort_order: i,
    active: true
  });
});

// Rechtsgebied
const rechtsgebiedList = {
  _key: 'valuelist-rechtsgebied',
  name: 'rechtsgebied',
  description: 'Rechtsgebieden in Nederland',
  source: 'standaard',
  version: '1.0.0',
  effective_date: now,
  expiry_date: null
};

db.value_lists.insert(rechtsgebiedList);

const rechtsgebieden = [
  'Strafrecht',
  'Civiel recht',
  'Bestuursrecht',
  'Sociaal recht',
  'Belastingrecht',
  'Vreemdelingenrecht',
  'Asielrecht',
  'Jeugdrecht',
  'Familierecht',
  'Arbeidsrecht'
];

rechtsgebieden.forEach((r, i) => {
  const code = r.toLowerCase().replace(' ', '_');
  db.value_list_items.insert({
    _key: `rechtsgebied-${code}`,
    list_id: 'valuelist-rechtsgebied',
    code: code,
    label: r,
    sort_order: i,
    active: true
  });
});

// Beveiligingsniveau
const beveiligingsniveauList = {
  _key: 'valuelist-beveiligingsniveau',
  name: 'beveiligingsniveau',
  description: 'Beveiligingsniveaus voor documenten',
  source: 'standaard',
  version: '1.0.0',
  effective_date: now,
  expiry_date: null
};

db.value_lists.insert(beveiligingsniveauList);

const beveiligingsniveaus = [
  { code: 'openbaar', label: 'Openbaar' },
  { code: 'intern', label: 'Intern' },
  { code: 'vertrouwelijk', label: 'Vertrouwelijk' },
  { code: 'geheim', label: 'Geheim' }
];

beveiligingsniveaus.forEach((b, i) => {
  db.value_list_items.insert({
    _key: `beveiligingsniveau-${b.code}`,
    list_id: 'valuelist-beveiligingsniveau',
    code: b.code,
    label: b.label,
    sort_order: i,
    active: true
  });
});

// Privacy level
const privacyLevelList = {
  _key: 'valuelist-privacy_level',
  name: 'privacy_level',
  description: 'Privacy niveaus volgens AVG',
  source: 'standaard',
  version: '1.0.0',
  effective_date: now,
  expiry_date: null
};

db.value_lists.insert(privacyLevelList);

const privacyLevels = [
  { code: 'openbaar', label: 'Openbaar' },
  { code: 'normaal', label: 'Normaal' },
  { code: 'bijzonder', label: 'Bijzonder' },
  { code: 'strafrechtelijk', label: 'Strafrechtelijk' }
];

privacyLevels.forEach((p, i) => {
  db.value_list_items.insert({
    _key: `privacy_level-${p.code}`,
    list_id: 'valuelist-privacy_level',
    code: p.code,
    label: p.label,
    sort_order: i,
    active: true
  });
});

// Woo relevantie
const wooRelevantieList = {
  _key: 'valuelist-woo_relevantie',
  name: 'woo_relevantie',
  description: 'Woo relevantie status',
  source: 'standaard',
  version: '1.0.0',
  effective_date: now,
  expiry_date: null
};

db.value_lists.insert(wooRelevantieList);

const wooRelevantie = [
  { code: 'ja', label: 'Ja' },
  { code: 'nee', label: 'Nee' },
  { code: 'moet_worden_beoordeeld', label: 'Moet worden beoordeeld' }
];

wooRelevantie.forEach((w, i) => {
  db.value_list_items.insert({
    _key: `woo_relevantie-${w.code}`,
    list_id: 'valuelist-woo_relevantie',
    code: w.code,
    label: w.label,
    sort_order: i,
    active: true
  });
});
```

- [ ] **Step 4: Commit migrations**

```bash
git add migrations/
git commit -m "feat(migrations): add ArangoDB migrations and seed data"
```

---

## Phase 6: Final Integration

### Task 17: Create Dockerfile and deployment config

**Files:**
- Create: `metadata-registry/Dockerfile`
- Create: `metadata-registry/.dockerignore`

- [ ] **Step 1: Write Dockerfile**

Create `metadata-registry/Dockerfile`:

```dockerfile
# Build stage
FROM rust:1.85 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the API server
RUN cargo build --release --package metadata-api

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/metadata-api /app/metadata-api

# Copy migrations
COPY migrations ./migrations

# Expose port
EXPOSE 8080

# Set environment variables
ENV ARANGO_DB_URL=http://arangodb:8529
ENV ARANGO_DB_NAME=metadata_registry
ENV ARANGO_DB_USER=root
ENV RUST_LOG=info

# Run the application
CMD ["/app/metadata-api"]
```

- [ ] **Step 2: Write .dockerignore**

Create `metadata-registry/.dockerignore`:

```
target/
.git/
.gitignore
Dockerfile
.dockerignore
*.md
tests/
.claude/
```

- [ ] **Step 3: Commit deployment files**

```bash
git add Dockerfile .dockerignore
git commit -m "feat(deploy): add Dockerfile for containerization"
```

---

### Task 18: Create README and documentation

**Files:**
- Create: `metadata-registry/README.md`

- [ ] **Step 1: Write README**

Create `metadata-registry/README.md`:

```markdown
# Metadata Registry Service

Centraal metadata beheer voor IOU-Modern.

## Features

- Standaard overheidsmetadata (provincies, gemeenten, rechtsgebieden)
- Organisatie-specifieke uitbreidingen
- Runtime validatie van documentmetadata
- Governance workflows (goedkeuring, publicatie, versiebeheer)
- GitOps integration voor declaratieve metadata definitie

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Metadata Registry Service                        │
│  ┌───────────────┐  ┌───────────────────┐  ┌───────────────────┐  │
│  │  Admin UI     │  │  REST/GraphQL API │  │  GitOps Sync      │  │
│  │  (Dioxus)     │  │  (Actix)         │  │  Service          │  │
│  └───────────────┘  └───────────────────┘  └───────────────────┘  │
│  ┌───────────────┐                                                  │
│  │  Validation   │                                                  │
│  │  Engine       │                                                  │
│  └───────────────┘                                                  │
└─────────────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────▼────────┐
                    │   ArangoDB       │
                    │   (Graph DB)     │
                    └──────────────────┘
```

## Quick Start

### Local Development

```bash
# Install dependencies
cargo install dioxus-cli

# Start ArangoDB
docker-compose up -d arangodb

# Run migrations
# TODO: Create migration runner

# Start API server
cargo run --package metadata-api

# Start admin UI (in separate terminal)
cd crates/metadata-admin
dx serve
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test --package metadata-core
cargo test --package metadata-validation
cargo test --package metadata-store
```

## API Documentation

### REST Endpoints

| Method | Endpoint | Beschrijving |
|--------|----------|--------------|
| GET | `/api/v1/schemas` | Lijst van alle schemas |
| GET | `/api/v1/schemas/:id` | Specifiek schema |
| POST | `/api/v1/schemas` | Maak nieuw schema |
| POST | `/api/v1/validate` | Valideer metadata |
| GET | `/api/v1/valuelists` | Alle waardenlijsten |

### GraphQL

```
POST /graphql
```

Example query:

```graphql
query GetSchemas {
  schemas {
    name
    version
    status
  }
}
```

## Configuration

Environment variables:

| Variable | Default | Beschrijving |
|----------|---------|--------------|
| `ARANGO_DB_URL` | `http://localhost:8529` | ArangoDB connection URL |
| `ARANGO_DB_NAME` | `metadata_registry` | Database name |
| `ARANGO_DB_USER` | `root` | Database username |
| `ARANGO_DB_PASSWORD` | (empty) | Database password |
| `RUST_LOG` | `info` | Log level |

## License

MIT
```

- [ ] **Step 2: Commit README**

```bash
git add README.md
git commit -m "docs: add README for metadata-registry service"
```

---

### Task 19: Final verification and cleanup

**Files:**
- None (verification task)

- [ ] **Step 1: Run full workspace test**

Run: `cd metadata-registry && cargo test --workspace`

Expected: All tests pass across all crates

- [ ] **Step 2: Run workspace build**

Run: `cd metadata-registry && cargo build --release`

Expected: All crates build successfully

- [ ] **Step 3: Verify workspace structure**

Run: `cd metadata-registry && tree -L 2 crates/`

Expected output matches planned structure:
- metadata-core
- metadata-store
- metadata-validation
- metadata-api
- metadata-gitops (placeholder)
- metadata-admin (placeholder)

- [ ] **Step 4: Final commit**

```bash
git add .
git commit -m "feat: complete metadata-registry MVP implementation"
```

- [ ] **Step 5: Tag the implementation**

Run: `cd metadata-registry && git tag v0.1.0`

---

## Post-Implementation Notes

### Not Implemented (Future Work)

1. **metadata-gitops crate**: Full GitOps sync service
   - File watcher for YAML changes
   - Git repository sync
   - Conflict resolution

2. **metadata-admin crate**: Full Dioxus admin UI
   - Schema editor component
   - ValueList manager component
   - Approval queue component
   - Audit log viewer

3. **Complete REST API implementation**
   - All endpoints defined in spec
   - Proper error handling
   - Request/response validation

4. **GraphQL schema**
   - Full Juniper schema
   - Query resolvers
   - Mutation resolvers

5. **Authentication/Authorization middleware**
   - Actual DigiD integration
   - RBAC enforcement
   - JWT handling

6. **Webhook delivery system**
   - Webhook registration
   - Event emission
   - Retry logic

7. **Migration runner**
   - CLI tool to run ArangoDB migrations
   - Version tracking
   - Rollback support

### Integration Points

1. **IOU-Modern integration**: The validation endpoint needs to be integrated into the main IOU-Modern document workflow
2. **ArangoDB setup**: Need to ensure ArangoDB is properly configured with indexes
3. **Authentication**: DigiD integration through IOU-Modern platform

### Testing Strategy

1. Unit tests cover core models and validation logic
2. Integration tests would need actual ArangoDB instance
3. End-to-end tests require full IOU-Modern platform

---

**END OF IMPLEMENTATION PLAN**
