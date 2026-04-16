# Metadata Registry Service - Design Document

> **Project**: IOU-Modern
> **Date**: 2026-04-16
> **Author**: Design Session
> **Status**: DRAFT

## Executive Summary

De Metadata Registry Service is een standalone Rust microservice die centraal metadata beheer biedt voor IOU-Modern. Het systeem ondersteunt:

- **Standaard overheidsmetadata** (provincies, gemeenten, rechtsgebieden, BSW documenttypes)
- **Organisatie-specifieke uitbreidingen**
- **Runtime validatie** van documentmetadata
- **Governance workflows** (goedkeuring, publicatie, versiebeheer)
- **GitOps integration** voor declaratieve metadata definitie

---

## Table of Contents

1. [Architecture](#1-architecture)
2. [Data Model](#2-data-model)
3. [Validation System](#3-validation-system)
4. [Governance Workflow](#4-governance-workflow)
5. [API Design](#5-api-design)
6. [Security](#6-security)
7. [Implementation Structure](#7-implementation-structure)
8. [Deployment](#8-deployment)

---

## 1. Architecture

### 1.1 System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    IOU-Modern Platform                                 │
│  ┌───────────┐  ┌────────────┐  ┌──────────────┐  ┌────────────┐      │
│  │  Frontend │  │   Server   │  │ Document     │  │    Other   │      │
│  │ (Dioxus)  │──│   (Rust)   │──│  Workflow    │  │  Services  │      │
│  └───────────┘  └────────────┘  └──────────────┘  └────────────┘      │
│                                                 │                      │
│                              ┌──────────────────┼─────────────────┐    │
│                              ▼                  ▼                 │    │
│                    ┌─────────────────────────────────────────────┐│    │
│                    │         Metadata Registry Service            ││    │
│                    │  ┌───────────────┐  ┌───────────────────┐  ││    │
│                    │  │  Admin UI     │  │  GraphQL/REST API │  ││    │
│                    │  └───────────────┘  └───────────────────┘  ││    │
│                    │  ┌───────────────┐  ┌───────────────────┐  ││    │
│                    │  │  Validation   │  │  GitOps Sync      │  ││    │
│                    │  │  Engine       │  │  Service          │  ││    │
│                    │  └───────────────┘  └───────────────────┘  ││    │
│                    └─────────────────────────────────────────────┘│    │
│                                      │                               │    │
│                              ┌───────▼────────┐                       │    │
│                              │  ArangoDB      │                       │    │
│                              │  (Metadata     │                       │    │
│                              │   Graph)       │                       │    │
│                              └────────────────┘                       │    │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Key Characteristics

| Aspect | Keuze |
|--------|-------|
| **Type** | Standalone Rust microservice |
| **Database** | ArangoDB (graph model) |
| **API** | REST + GraphQL |
| **Admin UI** | Dioxus WASM |
| **Governance** | Hybrid: Admin UI + GitOps |
| **Integration** | HTTP API, webhooks |

---

## 2. Data Model

### 2.1 Entity Nodes

```rust
/// Metadata Schema - definitie van een metadata structuur
pub struct MetadataSchema {
    pub _key: String,
    pub name: String,              // uniek
    pub description: String,
    pub version: String,           // semver
    pub status: SchemaStatus,      // DRAFT, PUBLISHED, DEPRECATED
    pub created_at: DateTime<Utc>,
    pub created_by: String,        // user_id
    pub governance: GovernanceConfig,
}

pub enum SchemaStatus {
    Draft,
    Published,
    Deprecated,
}

pub struct GovernanceConfig {
    pub approval_required: bool,
    pub approvers: Vec<String>,
    pub gitops_managed: bool,
    pub git_repo: Option<String>,
}

/// Attribute definitie binnen een schema
pub struct AttributeDefinition {
    pub _key: String,
    pub schema_id: String,         // → MetadataSchema
    pub name: String,
    pub r#type: DataType,
    pub required: bool,
    pub multivalued: bool,
    pub constraints: Constraints,
    pub default_value: Option<serde_json::Value>,
    pub description: String,
}

pub enum DataType {
    String,
    Integer,
    Decimal,
    Boolean,
    Date,
    Enum,
}

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
pub struct ValueList {
    pub _key: String,
    pub name: String,              // uniek, bijv. "provincie"
    pub description: String,
    pub source: ValueListSource,
    pub version: String,
    pub external_id: Option<String>, // bijv. "RDW-Provincie"
    pub effective_date: Date<Utc>,
    pub expiry_date: Option<Date<Utc>>,
}

pub enum ValueListSource {
    Standaard,   // Landelijke standaard
    Domein,      // Organisatie-specifiek
    Custom,      // User-defined
}

/// Item binnen een waardenlijst
pub struct ValueListItem {
    pub _key: String,
    pub list_id: String,           // → ValueList
    pub code: String,              // uniek binnen list
    pub label: String,
    pub description: Option<String>,
    pub parent_code: Option<String>, // → ValueListItem (hiërarchie)
    pub sort_order: i32,
    pub active: bool,
    pub properties: Option<HashMap<String, serde_json::Value>>,
}
```

### 2.2 Edge Relations

| Edge | From | To | Beschrijving |
|------|------|-----|--------------|
| `HAS_ATTRIBUTE` | MetadataSchema | AttributeDefinition | Schema heeft attributen |
| `HAS_VALUELIST` | AttributeDefinition | ValueList | Enum attribut verwijst naar lijst |
| `CONTAINS_ITEM` | ValueList | ValueListItem | Lijst bevat items |
| `EXTENDS` | MetadataSchema | MetadataSchema | Schema inheritance |
| `DEPENDS_ON` | AttributeDefinition | AttributeDefinition | Cross-field dependency |
| `SUPERSEDES` | ValueList | ValueList | Versie opvolging |

### 2.3 Standard Value Lists

De volgende waardenlijsten worden vooraf geïmporteerd (Source: STANDAARD):

| Naam | Beschrijving |
|------|--------------|
| `provincie` | Alle 12 Nederlandse provincies |
| `gemeente` | Alle 352 gemeenten |
| `rechtsgebied` | Strafrecht, Civiel recht, Bestuursrecht, etc. |
| `documenttype` | BSW categoriën |
| `beveiligingsniveau` | Openbaar, Intern, Vertrouwelijk, Geheim |
| `privacy_level` | Openbaar, Normaal, Bijzonder, Strafrechtelijk |
| `woo_relevantie` | Ja, Nee, Moet worden beoordeeld |

---

## 3. Validation System

### 3.1 Three-Layer Architecture

```
LAAG 1: DECLARATIEF (in Metamodel)
├── type: DataType
├── required: bool
├── multivalued: bool
└── constraints: { min_length, max_length, pattern, enum_values }

              ↓

LAAG 2: VALIDATION SERVICE
├── POST /api/v1/validate
├── Input: { schema_id, metadata }
└── Output: { valid, errors, warnings }

              ↓

LAAG 3: REAL-TIME HOOKS
├── document.metadata.created
├── document.metadata.updated
└── document.metadata.deleted
```

### 3.2 Validation Request/Response

```rust
pub struct ValidationRequest {
    pub schema_id: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct ValidationResponse {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

pub struct ValidationError {
    pub field: String,
    pub code: ErrorCode,
    pub message: String,
}

pub enum ErrorCode {
    RequiredFieldMissing,
    TypeMismatch,
    ConstraintViolation,
    EnumValueInvalid,
    CustomValidationFailed,
}
```

### 3.3 Custom Validators

Uitbreidbare validatie regels:

```rust
/// Voorbeeld: woonplaats moet in geselecteerde gemeente liggen
fn validate_woonplaats_in_gemeente(
    value: &serde_json::Value,
    context: &ValidationContext,
) -> Result<(), ValidationError> {
    let gemeente = context.metadata.get("gemeente")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ValidationError::missing("gemeente"))?;

    let woonplaats = value.as_str()
        .ok_or_else(|| ValidationError::type_mismatch("woonplaats", "string"))?;

    if !GEMEENTE_WOONPLAATEN.get(gemeente)
        .map(|list| list.contains(woonplaats))
        .unwrap_or(false)
    {
        return Err(ValidationError::constraint(
            "woonplaats",
            "Woonplaats niet in geselecteerde gemeente"
        ));
    }

    Ok(())
}
```

---

## 4. Governance Workflow

### 4.1 Workflow A: Kleine changes (Admin UI)

```
1. Gebruiker opent Admin UI
2. Wijzigt ValueListItem of AttributeDefinition
3. Slaat op als "DRAFT"
4. Indien approval_required: stuur naar approvers
5. Approver keurt goed → status wordt "PUBLISHED"
6. GitOps sync schrijft wijziging terug naar git (audit)
```

### 4.2 Workflow B: Grote changes (GitOps)

```
1. Developer clone metadata registry repo
2. Wijzig YAML bestanden in schemas/
3. Open PR met description
4. CI valideert schema syntax
5. Review + merge
6. GitOps sync detected change → import naar ArangoDB
```

**GitOps polling interval**: 30 seconden (configureerbaar per omgeving)
**Conflict resolution**: Git altijd leidend bij GitOps-managed schemas

### 4.3 YAML Format

```yaml
# schemas/iou-document-v1.yaml
apiVersion: metadata.iou.modern/v1
kind: MetadataSchema
metadata:
  name: iou-document-v1
  description: IOU Document metadata
  version: 1.2.0
spec:
  attributes:
    - name: documenttype
      type: ENUM
      required: true
      valueList: documenttype-bsw
    - name: bewaartermijn
      type: INTEGER
      constraints:
        min_value: 0
        custom_validator: bewaartermijn_valid
  governance:
    approval_required: true
    approvers:
      - domain-owner
      - records-manager
```

### 4.4 Audit Trail

```rust
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

pub enum EntityType {
    Schema,
    Attribute,
    ValueList,
    Item,
}

pub enum AuditAction {
    Create,
    Update,
    Delete,
    Publish,
    Deprecate,
}

pub enum ChangeSource {
    Ui,
    GitOps,
    Migration,
    Api,
}
```

---

## 5. API Design

### 5.1 REST Endpoints

| Methode | Endpoint | Beschrijving |
|---------|----------|--------------|
| GET | `/api/v1/schemas` | Lijst van alle schemas |
| GET | `/api/v1/schemas/:id` | Specifiek schema |
| POST | `/api/v1/schemas` | Maak nieuw schema |
| PUT | `/api/v1/schemas/:id` | Update schema |
| DELETE | `/api/v1/schemas/:id` | Verwijder schema |
| POST | `/api/v1/schemas/:id/publish` | Publiceer DRAFT schema |
| GET | `/api/v1/valuelists` | Alle waardenlijsten |
| GET | `/api/v1/valuelists/:id` | Specifieke lijst |
| GET | `/api/v1/valuelists/:id/items` | Items in lijst |
| POST | `/api/v1/valuelists` | Nieuwe lijst |
| POST | `/api/v1/valuelists/:id/items` | Voeg item toe |
| POST | `/api/v1/validate` | Valideer metadata |
| POST | `/api/v1/validate/batch` | Bulk validatie |
| GET | `/api/v1/audit/log` | Audit trail |
| GET | `/api/v1/audit/changes/:entity_id` | Changes per entity |

### 5.2 GraphQL Query Examples

```graphql
query GetSchemaWithAttributes {
  schema(id: "iou-document-v1") {
    name
    version
    attributes {
      name
      type
      valueList {
        name
        items {
          code
          label
        }
      }
    }
    dependsOn {
      name
    }
  }
}

mutation PublishSchema {
  publishSchema(id: "iou-document-v1") {
    success
    schema {
      status
    }
  }
}
```

### 5.3 Webhook Events

```rust
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
    ValueItemListRetired(String, String),
    ValidationFailed(String, Vec<ValidationError>),
}

// Webhook payload
pub struct WebhookPayload {
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub entity_id: String,
    pub entity_version: Option<String>,
    pub triggered_by: String,
    pub data: serde_json::Value,
}
```

---

## 6. Security

### 6.1 Authentication

- **DigiD** via IOU-Modern platform
- **Service-to-service**: mTLS / API Keys
- **Admin UI**: DigiD + MFA verplicht

### 6.2 Authorization (RBAC)

| Rol | Permissies |
|-----|-----------|
| **METADATA_VIEWER** | ✓ View schemas/valuelists, ✓ Validate metadata, ✗ Geen wijzigingen |
| **METADATA_EDITOR** | ✓ Create/edit DRAFT schemas, ✓ Add/modify valuelist items, ✗ Geen PUBLISH |
| **METADATA_APPROVER** | ✓ Approve/reject changes, ✓ Publish DRAFT→PUBLISHED, ✓ Deprecate schemas |
| **METADATA_ADMIN** | ✓ Alle acties, ✓ Beheer rollen, ✓ GitOps configuratie |

### 6.3 Audit Logging

- Alle wijzigingen gelogd (wie, wat, wanneer)
- Git commits als audit trail (immutable)
- Export naar SIEM (bijv. Splunk)

### 6.4 Rate Limiting

| Endpoint | Limit | Periode |
|----------|-------|---------|
| POST /api/v1/validate | 1000 requests | Per tenant, per minuut |
| GraphQL queries | 1000 complexity score | Per request |
| Admin UI endpoints | 100 requests | Per user, per minuut |
| GET /api/v1/* | 10 000 requests | Per tenant, per minuut |

---

## 7. Implementation Structure

### 7.1 Crate Structure

```
metadata-registry/
├── Cargo.toml                          # Workspace root
├── crates/
│   ├── metadata-core/                  # Gedeelde types (WASM-compatible)
│   │   ├── src/
│   │   │   ├── models.rs
│   │   │   ├── validation.rs
│   │   │   ├── graph.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── metadata-store/                 # ArangoDB repository layer
│   │   ├── src/
│   │   │   ├── connection.rs
│   │   │   ├── schema_repo.rs
│   │   │   ├── valuelist_repo.rs
│   │   │   ├── audit_repo.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── metadata-validation/            # Validation engine
│   │   ├── src/
│   │   │   ├── engine.rs
│   │   │   ├── constraints.rs
│   │   │   ├── custom.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── metadata-api/                   # REST/GraphQL API server
│   │   ├── src/
│   │   │   ├── rest.rs
│   │   │   ├── graphql.rs
│   │   │   ├── webhooks.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── metadata-gitops/                # GitOps sync service
│   │   ├── src/
│   │   │   ├── sync.rs
│   │   │   ├── yaml.rs
│   │   │   ├── watch.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   └── metadata-admin/                 # Admin UI (Dioxus)
│       ├── src/
│       │   ├── components/
│       │   │   ├── schema_editor.rsx
│       │   │   ├── valuelist_manager.rsx
│       │   │   ├── approval_queue.rsx
│       │   │   └── audit_log.rsx
│       │   └── main.rs
│       └── Cargo.toml
│
├── migrations/
│   ├── 001_init_collections.js
│   ├── 002_indexes.js
│   └── 003_standard_valuelists.js
│
└── config/
    └── schemas/                        # GitOps YAML definitions
        ├── standard/
        │   ├── provincie.yaml
        │   ├── gemeente.yaml
        │   └── documenttype-bsw.yaml
        └── domain/
            └── iou-document-v1.yaml
```

### 7.2 Key Dependencies

```toml
[workspace.dependencies]
# Core
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "1.0"

# ArangoDB
arangors = "0.2"

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
```

### 7.3 Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("Schema niet gevonden: {0}")]
    SchemaNotFound(String),

    #[error("Attribute niet gevonden: {0} in schema: {1}")]
    AttributeNotFound(String, String),

    #[error("ValueList niet gevonden: {0}")]
    ValueListNotFound(String),

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

    #[error("Git fout: {0}")]
    Git(#[from] git2::Error),

    #[error("YAML parse fout: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, MetadataError>;
```

---

## 8. Deployment

### 8.1 Infrastructure

```
                    ┌──────────────────┐
                    │   Ingress / TLS  │
                    └────────┬─────────┘
                             │
               ┌─────────────┴─────────────┐
               │                           │
      ┌────────▼────────┐        ┌────────▼────────┐
      │  Admin UI       │        │  API Gateway    │
      │  (Dioxus WASM)  │        │  (REST/GQL)     │
      └─────────────────┘        └────────┬────────┘
                                         │
                           ┌─────────────┴──────────┐
                           │                        │
                  ┌────────▼────────┐      ┌────────▼────────┐
                  │ Validation      │      │ GitOps Sync      │
                  │ Service         │      │ Service          │
                  └────────┬────────┘      └────────┬────────┘
                           │                        │
                           └──────────┬─────────────┘
                                      │
                            ┌─────────▼────────┐
                            │   ArangoDB       │
                            │   - 3 replicas   │
                            └──────────────────┘
```

### 8.2 Resources

| Component | CPU | Memory |
|-----------|-----|--------|
| API Service | 250m - 500m | 256Mi - 512Mi |
| Validation Service | 250m - 500m | 256Mi - 512Mi |
| GitOps Service | 100m - 250m | 128Mi - 256Mi |
| Admin UI | 50m - 100m | 64Mi - 128Mi |
| ArangoDB | 1000m - 2000m | 2Gi - 4Gi |

### 8.3 Monitoring

- Health check: `GET /health`
- Prometheus metrics:
  - `metadata_validation_requests_total`
  - `metadata_validation_duration_seconds`
  - `metadata_api_requests_total`
  - `metadata_git_sync_last_success_timestamp`

### 8.4 Backup & Recovery

- ArangoDB snapshots: dagelijks
- Git repos: native git backup
- Audit log (PostgreSQL): continuous archiving

---

## Appendix A: Standard Value Lists

### Provincie (12 items)

- Drenthe (DR)
- Flevoland (FL)
- Friesland (FR)
- Gelderland (GE)
- Groningen (GR)
- Limburg (LI)
- Noord-Brabant (NB)
- Noord-Holland (NH)
- Overijssel (OV)
- Utrecht (UT)
- Zeeland (ZE)
- Zuid-Holland (ZH)

### Rechtsgebied

- Strafrecht
- Civiel recht
- Bestuursrecht
- Sociaal recht
- Belastingrecht
- Vreemdelingenrecht
- Asielrecht
- Jeugdrecht
- Familierecht
- Arbeidsrecht

### Beveiligingsniveau

- Openbaar
- Intern
- Vertrouwelijk
- Geheim

### Privacy Level

- Openbaar
- Normaal
- Bijzonder
- Strafrechtelijk

---

**END OF DESIGN DOCUMENT**
