# Migration Plan: Unified Architecture Stack

**Document ID**: ARC-MIG-001-v1.0
**Status**: DRAFT
**Created**: 2026-04-20
**Option**: A - Unified Stack (Rust/ArangoDB/Dioxus)

## Executive Summary

This document details the migration of Projects 002 (Metadata Registry) and 003 (Context-Aware Data) to a unified technology stack based on Rust, ArangoDB, and Dioxus. The migration will be executed over 8 months in 5 phases.

---

## Target Architecture

### Unified Stack Definition

```
┌─────────────────────────────────────────────────────────────────────┐
│                    IOU-MODERN UNIFIED PLATFORM                      │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Rust Workspace (Cargo)                                     │   │
│  │                                                              │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │   │
│  │  │ metadata-api │  │ context-api  │  │ unified-ui   │      │   │
│  │  │ (actix-web)  │  │ (actix-web)  │  │ (dioxus)     │      │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘      │   │
│  │         │                 │                  │              │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │   │
│  │  │metadata-store│  │context-store │  │ inference    │      │   │
│  │  │ (arangors)   │  │ (arangors)   │  │ (claude-sdk) │      │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘      │   │
│  │         │                 │                                 │   │
│  │  ┌────────────────────────────────────────────────────┐    │   │
│  │  │         shared (common types, traits)             │    │   │
│  │  └────────────────────────────────────────────────────┘    │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  ArangoDB Cluster (Single Database)                         │   │
│  │                                                              │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │   │
│  │  │   GGHH V2   │  │   Context   │  │   Vector    │         │   │
│  │  │ Collections │  │ Collections │  │  (ahocorasick)│        │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘         │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### Technology Specifications

| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| **Language** | Rust | 2021 edition | Primary implementation language |
| **Web Framework** | actix-web | 4.0 | REST/GraphQL API server |
| **GraphQL** | juniper | 0.15 | GraphQL API |
| **Database** | ArangoDB | 3.11+ | Graph + document storage |
| **Client** | arangors | 0.6 | ArangoDB Rust driver |
| **Frontend** | Dioxus | 0.7 | WebAssembly UI |
| **Async Runtime** | tokio | 1.0 | Async runtime |
| **Serialization** | serde | 1.0 | JSON/binary serialization |
| **AI/ML** | anthropic-rust | 0.1 | Claude API integration |
| **Vector Search** | ahocorasick | 1.0 | Fast text similarity |

---

## Phase 1: Foundation (Months 1-2)

### Objectives

1. Establish unified Cargo workspace structure
2. Define shared data models and traits
3. Create ArangoDB schema for both projects
4. Set up CI/CD pipeline

### Deliverables

| Deliverable | Description | Owner |
|-------------|-------------|-------|
| Unified workspace | Cargo.toml with workspace members | Backend Lead |
| Shared crate | Common types, traits, errors | Backend Lead |
| Database schema | Combined GGHH V2 + Context schema | DBA |
| CI/CD pipeline | GitHub Actions for Rust | DevOps |

### Workspace Structure

```
iou-modern/
├── Cargo.toml                 # Workspace root
├── crates/
│   ├── shared/                # Shared types and traits
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── entities.rs    # Common entity traits
│   │   │   ├── error.rs       # Shared error types
│   │   │   ├── validation.rs  # Validation traits
│   │   │   ├── auth.rs        # Authentication types
│   │   │   └── api.rs         # API common types
│   │   └── Cargo.toml
│   │
│   ├── metadata/              # Project 002 implementation
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── gghh.rs        # GGHH V2 entities
│   │   │   ├── repositories.rs
│   │   │   ├── api.rs         # REST/GraphQL handlers
│   │   │   └── validation.rs
│   │   └── Cargo.toml
│   │
│   ├── context/               # Project 003 implementation
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── context.rs     # Context entities
│   │   │   ├── layers.rs      # Context layer model
│   │   │   ├── inference.rs   # AI inference
│   │   │   ├── quality.rs     # Quality monitoring
│   │   │   └── api.rs
│   │   └── Cargo.toml
│   │
│   ├── database/              # Shared database layer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── connection.rs  # Connection pool
│   │   │   ├── migrations.rs  # Schema migrations
│   │   │   └── repositories.rs # Generic repository trait
│   │   └── Cargo.toml
│   │
│   ├── api/                   # Unified API server
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── metadata.rs    # Metadata routes
│   │   │   ├── context.rs     # Context routes
│   │   │   ├── graphql.rs     # GraphQL schema
│   │   │   └── middleware.rs  # Auth, CORS, logging
│   │   └── Cargo.toml
│   │
│   ├── inference/             # AI/ML inference service
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── claude.rs      # Claude API integration
│   │   │   ├── ner.rs         # Named entity recognition
│   │   │   └── classification.rs
│   │   └── Cargo.toml
│   │
│   └── ui/                    # Dioxus admin interface
│       ├── src/
│       │   ├── main.rs
│       │   ├── metadata.rs    # Metadata management UI
│       │   ├── context.rs     # Context management UI
│       │   └── components.rs  # Shared UI components
│       └── Cargo.toml
│
├── migrations/                # ArangoDB migrations
│   ├── 001_initial_schema.sql
│   ├── 002_gghv_v2.sql
│   ├── 003_context.sql
│   └── 004_indexes.sql
│
├── tests/                     # Integration tests
│   ├── metadata_tests.rs
│   └── context_tests.rs
│
└── docker/
    ├── Dockerfile.api
    ├── Dockerfile.ui
    └── docker-compose.yml
```

### Shared Crate Definition

```rust
// crates/shared/src/entities.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Base trait for all IOU entities
pub trait IouEntity {
    fn id(&self) -> &str;
    fn created_at(&self) -> DateTime<Utc>;
    fn modified_at(&self) -> Option<DateTime<Utc>>;
    fn organisation_id(&self) -> &str;
}

/// Time validity trait for temporal entities
pub trait Temporal {
    fn geldig_vanaf(&self) -> DateTime<Utc>;
    fn geldig_tot(&self) -> DateTime<Utc>;
    fn is_valid_at(&self, timestamp: DateTime<Utc>) -> bool {
        timestamp >= self.geldig_vanaf() && timestamp < self.geldig_tot()
    }
}

/// Ownership trait for governance
pub trait Owned {
    fn owner(&self) -> &str;
    fn steward(&self) -> Option<&str>;
}

/// Security classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SecurityLevel {
    Open,
    Intern,
    Confidential,
    Secret,
}

/// Information object reference (links metadata to context)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationObjectRef {
    pub object_id: String,
    pub object_type: String,
    pub source_system: String,
}
```

### ArangoDB Unified Schema

```javascript
// migrations/002_combined_schema.js

// GGHH V2 Collections
db._createDatabase("iou_modern");

// Core GGHH V2 collections
db._createCollection("gebeurtenis");
db._createCollection("gegevensproduct");
db._createCollection("elementaire_set");
db._createCollection("enkelvoudig_gegeven");
db._createCollection("waarde_met_tijd");

// Context collections
db._createCollection("information_object");
db._createCollection("context");
db._createCollection("context_type");
db._createCollection("context_layer");
db._createCollection("context_inference");
db._createCollection("context_quality");

// Shared collections
db._createCollection("audit_log");
db._createCollection("organisation");
db._createCollection("user");

// Edge collections for relationships
db._createEdgeCollection("has_context");
db._createEdgeCollection("has_type");
db._createEdgeCollection("has_layer");
db._createEdgeCollection("inferred_by");

// Indexes
db.information_object.ensureIndex({
    type: "persistent",
    fields: ["object_id", "organisation_id"],
    unique: true
});

db.context.ensureIndex({
    type: "persistent",
    fields: ["object_id", "context_type_id"],
    unique: false
});

// Full-text search
db.context.ensureIndex({
    type: "fulltext",
    fields: ["context_value"],
    minLength: 3
});
```

### Success Criteria

- [ ] Workspace compiles with `cargo build --workspace`
- [ ] All tests pass: `cargo test --workspace`
- [ ] ArangoDB schema deployed and validated
- [ ] CI/CD pipeline produces Docker images
- [ ] Documentation updated

---

## Phase 2: Context Backend Migration (Months 3-4)

### Objectives

1. Implement Context entities in Rust
2. Create Context repositories for ArangoDB
3. Implement Context Capture API
4. Port business logic from Java design

### Deliverables

| Deliverable | Description | Lines of Code |
|-------------|-------------|---------------|
| Context entities | Rust structs for 8 context entities | ~800 |
| Context repositories | ArangoDB repositories | ~1,200 |
| Context API | REST/GraphQL handlers | ~600 |
| Validation engine | Context validation rules | ~400 |
| **Total** | | **~3,000** |

### Context Entity Implementation

```rust
// crates/context/src/context.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use shared::entities::{IouEntity, Temporal, Owned};

/// Context metadata record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    #[serde(rename = "_key")]
    pub key: String,

    /// Reference to parent information object
    pub object_id: String,

    /// Context type identifier
    pub context_type_id: String,

    /// Context layer (CORE, DOMAIN, SEMANTIC, PROVENANCE)
    pub layer: ContextLayer,

    /// Context value (flexible JSON)
    pub context_value: serde_json::Value,

    /// Whether this was inferred
    pub is_inferred: bool,

    /// Inference confidence (if applicable)
    pub confidence: Option<f64>,

    /// Time validity
    pub geldig_vanaf: DateTime<Utc>,
    pub geldig_tot: DateTime<Utc>,

    /// Ownership
    pub organisatie_id: String,
    pub aangemaakt_door: String,
    pub aangemaakt_op: DateTime<Utc>,
    pub gewijzigd_door: Option<String>,
    pub gewijzigd_op: Option<DateTime<Utc>>,
}

impl IouEntity for Context {
    fn id(&self) -> &str { &self.key }
    fn created_at(&self) -> DateTime<Utc> { self.aangemaakt_op }
    fn modified_at(&self) -> Option<DateTime<Utc>> { self.gewijzigd_op }
    fn organisation_id(&self) -> &str { &self.organisatie_id }
}

impl Temporal for Context {
    fn geldig_vanaf(&self) -> DateTime<Utc> { self.geldig_vanaf }
    fn geldig_tot(&self) -> DateTime<Utc> { self.geldig_tot }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContextLayer {
    Core,       // Layer 1: Mandatory
    Domain,     // Layer 2: Case/project context
    Semantic,   // Layer 3: Legal basis, meaning
    Provenance, // Layer 4: Audit trail
}

/// Context type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextType {
    #[serde(rename = "_key")]
    pub key: String,

    pub name: String,
    pub description: String,
    pub layer: ContextLayer,

    /// Validation rules
    pub validation_rule: Option<serde_json::Value>,

    /// Allowed values (for enum types)
    pub allowed_values: Option<Vec<String>>,

    /// Steward
    pub steward: String,

    pub geldig_vanaf: DateTime<Utc>,
    pub geldig_tot: DateTime<Utc>,
}

/// Information object (links metadata and context)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationObject {
    #[serde(rename = "_key")]
    pub key: String,

    pub object_id: String,
    pub object_type: String,
    pub source_system: String,

    /// Core metadata (from GGHH V2)
    pub title: String,
    pub description: Option<String>,
    pub creator: String,
    pub created_at: DateTime<Utc>,

    pub organisatie_id: String,
    pub geldig_vanaf: DateTime<Utc>,
    pub geldig_tot: DateTime<Utc>,
}

impl IouEntity for InformationObject { /* ... */ }
impl Temporal for InformationObject { /* ... */ }
```

### Context Repository

```rust
// crates/context/src/repository.rs

use arangors::Client;
use anyhow::Result;

use crate::context::{Context, ContextType};

pub struct ContextRepository {
    client: Client,
    db_name: String,
}

impl ContextRepository {
    pub async fn create(&self, context: &Context) -> Result<String> {
        let db = self.client.db(&self.db_name).await?;

        // Validate
        self.validate_context(context).await?;

        // Check for duplicates
        let existing = self
            .find_by_object_and_type(&context.object_id, &context.context_type_id)
            .await?;

        if existing.is_some() {
            // Update existing
            return self.update(context).await;
        }

        // Create new
        let collection = db.collection("context").await?;
        let result: Context = collection.create_document(context).await?;
        Ok(result.key)
    }

    pub async fn find_by_object(&self, object_id: &str) -> Result<Vec<Context>> {
        let db = self.client.db(&self.db_name).await?;
        let collection = db.collection("context").await?;

        let aql = r#"
            FOR ctx IN context
            FILTER ctx.object_id == @object_id
                AND ctx.geldig_vanaf <= @now
                AND ctx.geldig_tot > @now
            RETURN ctx
        "#;

        let mut cursor = db.query_batch(aql.clone())
            .bind("object_id", object_id)
            .bind("now", chrono::Utc::now())
            .await?;

        let results: Vec<Context> = cursor.next().await?.unwrap_or_default();
        Ok(results)
    }

    async fn validate_context(&self, context: &Context) -> Result<()> {
        // Core layer validation
        if matches!(context.layer, ContextLayer::Core) {
            if context.context_value.is_null() {
                anyhow::bail!("Core context cannot have null value");
            }
        }

        // Type-specific validation
        let context_type = self.get_type(&context.context_type_id).await?;
        if let Some(validation) = context_type.and_then(|t| t.validation_rule) {
            self.apply_validation(&context.context_value, &validation)?;
        }

        Ok(())
    }
}
```

### Context API Handler

```rust
// crates/api/src/context.rs

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use context::ContextRepository;
use shared::error::ApiError;

#[derive(Deserialize)]
pub struct CreateContextRequest {
    pub object_id: String,
    pub object_type: String,
    pub context: serde_json::Value,
}

#[derive(Serialize)]
pub struct CreateContextResponse {
    pub context_id: String,
    pub completeness: CompletenessScore,
    pub quality_score: f64,
}

#[derive(Serialize)]
pub struct CompletenessScore {
    pub core: f64,
    pub domain: f64,
    pub semantic: f64,
    pub provenance: f64,
    pub overall: f64,
}

pub async fn create_context(
    repo: web::Data<ContextRepository>,
    req: web::Json<CreateContextRequest>,
) -> Result<HttpResponse, ApiError> {
    // Parse context by layer
    let contexts = parse_context_layers(&req.object_id, &req.context)?;

    // Create each context record
    let mut created = vec![];
    for ctx in contexts {
        let key = repo.create(&ctx).await?;
        created.push(key);
    }

    // Calculate completeness
    let completeness = calculate_completeness(&repo, &req.object_id).await?;

    Ok(HttpResponse::Created().json(CreateContextResponse {
        context_id: created.first().cloned().unwrap_or_default(),
        completeness,
        quality_score: 1.0,
    }))
}

fn parse_context_layers(object_id: &str, data: &serde_json::Value) -> Result<Vec<Context>, ApiError> {
    let mut contexts = vec![];

    // Core context (required)
    if let Some(core) = data.get("core") {
        for (key, value) in core.as_object().ok_or(ApiError::InvalidInput)? {
            contexts.push(Context {
                key: format!("{}-{}-{}", object_id, "core", key),
                object_id: object_id.to_string(),
                context_type_id: format!("core.{}", key),
                layer: ContextLayer::Core,
                context_value: value.clone(),
                is_inferred: false,
                confidence: None,
                geldig_vanaf: chrono::Utc::now(),
                geldig_tot: chrono::Utc::now() + chrono::Duration::days(365 * 20),
                organisatie_id: "".to_string(), // From auth
                aangemaakt_door: "".to_string(),
                aangemaakt_op: chrono::Utc::now(),
                gewijzigd_door: None,
                gewijzigd_op: None,
            });
        }
    }

    // Additional layers...
    Ok(contexts)
}
```

### Success Criteria

- [ ] All context entities implemented
- [ ] Repository tests passing
- [ ] API endpoints functional
- [ ] Performance < 100ms (p95)

---

## Phase 3: Inference Service Migration (Months 4-5)

### Objectives

1. Implement Claude API integration in Rust
2. Port NER and classification logic
3. Implement human review workflow

### Claude SDK Integration

```rust
// crates/inference/src/claude.rs

use anthropic::client::Client;
use anthropic::types::{
    ContentBlock, Message, MessageCreateParams, MessageCreateParamsInternal,
    Role, Tool, ToolChoice, ToolUseBlock,
};
use anyhow::Result;

const MODEL: &str = "claude-3-5-sonnet-20241022";

pub struct ContextInferenceService {
    client: Client,
}

impl ContextInferenceService {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .with_api_key(api_key)
            .build()
            .expect("Failed to create Claude client");

        Self { client }
    }

    /// Analyze text and extract context
    pub async fn extract_context(
        &self,
        text: &str,
        object_type: &str,
    ) -> Result<InferenceResult> {
        let prompt = self.build_extraction_prompt(text, object_type);

        let message = Message::new()
            .with_role(Role::User)
            .with_content(ContentBlock::Text {
                text: prompt,
            });

        let params = MessageCreateParams::builder()
            .model(MODEL.to_string())
            .messages(vec![message])
            .max_tokens(1024_u32)
            .temperature(0.0)
            .build()
            .unwrap();

        let response = self.client.messages().create(params).await?;

        self.parse_inference_response(&response, object_type)
    }

    fn build_extraction_prompt(&self, text: &str, object_type: &str) -> String {
        format!(
            r#"Analyze the following government document and extract contextual metadata.

Document Type: {object_type}

Document Text:
{text}

Extract the following context fields (if present):
- Core: creator, title, classification
- Domain: case_number, case_type, project_code, policy_reference
- Semantic: legal_basis (BWBR references), subject_tags, relationships
- Provenance: source_system, document_id

Return JSON with confidence scores for each field."#
        )
    }

    fn parse_inference_response(
        &self,
        response: &anthropic::types::Message,
        object_type: &str,
    ) -> Result<InferenceResult> {
        let text = response
            .content
            .first()
            .and_then(|b| b.as_text())
            .ok_or(anyhow::anyhow!("No text in response"))?;

        let extracted: serde_json::Value = serde_json::from_str(text)?;
        let confidence = self.calculate_confidence(&extracted)?;

        Ok(InferenceResult {
            context: extracted,
            confidence,
            requires_review: confidence < 0.9,
            model: MODEL.to_string(),
            inferred_at: chrono::Utc::now(),
        })
    }

    fn calculate_confidence(&self, result: &serde_json::Value) -> Result<f64> {
        // Calculate confidence based on field completeness
        let fields = result.as_object().map(|o| o.len()).unwrap_or(0);
        let total_expected = 10; // Expected number of fields

        Ok(fields as f64 / total_expected as f64)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub context: serde_json::Value,
    pub confidence: f64,
    pub requires_review: bool,
    pub model: String,
    pub inferred_at: chrono::DateTime<chrono::Utc>,
}
```

### Success Criteria

- [ ] Claude API integration working
- [ ] Inference confidence > 0.8 for test documents
- [ ] Human review workflow functional

---

## Phase 4: Frontend Migration (Months 6-7)

### Objectives

1. Implement Context Portal in Dioxus
2. Create reusable UI components
3. Integrate with unified API

### Dioxus Components

```rust
// crates/ui/src/context.rs

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use reqwest::Client;

use shared::api::ContextClient;

#[component]
pub fn ContextPortal() -> Element {
    let api_client = use_api_client();
    let navigate = use_navigate();

    rsx! {
        div { class: "context-portal",
            nav { class: "sidebar",
                NavLink { to: "/context/types", "Context Types" }
                NavLink { to: "/context/objects", "Information Objects" }
                NavLink { to: "/context/quality", "Quality Dashboard" }
                NavLink { to: "/context/inference", "Inference Queue" }
            }

            main { class: "content",
                Outlet::<ContextRouter>()
            }
        }
    }
}

#[component]
pub fn ContextTypeManager() -> Element {
    let api = use_context::<ContextClient>();
    let mut types = use_signal(Vec::new);

    use_effect(move || async move {
        if let Some(api) = api {
            let result = api.list_context_types().await;
            types.set(result.unwrap_or_default());
        }
    });

    rsx! {
        div { class: "context-type-manager",
            h1 { "Context Type Management" }

            button { class: "btn-primary", "Add New Type" }

            table { class: "data-table",
                thead {
                    tr {
                        th { "Name" }
                        th { "Layer" }
                        th { "Steward" }
                        th { "Actions" }
                    }
                }
                tbody {
                    for context_type in types.read().iter() {
                        tr {
                            td { "{context_type.name}" }
                            td { "{context_type.layer:?}" }
                            td { "{context_type.steward}" }
                            td {
                                button { class: "btn-sm", "Edit" }
                                button { class: "btn-sm btn-danger", "Delete" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ContextVisualization(object_id: String) -> Element {
    let api = use_context::<ContextClient>();
    let mut context = use_signal(None::<ContextData>);

    use_effect(move || async move {
        if let Some(api) = api {
            let result = api.get_object_context(&object_id).await;
            context.set(result.ok());
        }
    });

    rsx! {
        div { class: "context-visualization",
            h2 { "Context for {object_id}" }

            if let Some(ctx) = context.read().as_ref() {
                div { class: "context-layers",
                    ContextLayerView {
                        layer: "Core",
                        contexts: ctx.core.clone(),
                        color: "#28a745"
                    }
                    ContextLayerView {
                        layer: "Domain",
                        contexts: ctx.domain.clone(),
                        color: "#007bff"
                    }
                    ContextLayerView {
                        layer: "Semantic",
                        contexts: ctx.semantic.clone(),
                        color: "#ffc107"
                    }
                    ContextLayerView {
                        layer: "Provenance",
                        contexts: ctx.provenance.clone(),
                        color: "#6c757d"
                    }
                }
            } else {
                div { class: "loading", "Loading..." }
            }
        }
    }
}

#[component]
fn ContextLayerView(
    layer: String,
    contexts: Vec<ContextItem>,
    color: String,
) -> Element {
    rsx! {
        div { class: "context-layer",
            style: "border-left: 4px solid {color}",
            h3 { "{layer} Context" }
            div { class: "context-items",
                for item in contexts {
                    div { class: "context-item",
                        span { class: "label", "{item.name}:" }
                        span { class: "value", "{item.value}" }
                        if item.is_inferred {
                            span { class: "badge badge-info", "Inferred" }
                        }
                    }
                }
            }
        }
    }
}
```

### Success Criteria

- [ ] Dioxus app compiles to WASM
- [ ] All Context UI screens functional
- [ ] API integration working

---

## Phase 5: Integration & Cleanup (Month 8)

### Objectives

1. Deploy unified platform
2. Migrate production data
3. Decommission legacy systems
4. Update documentation

### Deployment Checklist

- [ ] ArangoDB cluster deployed
- [ ] Unified API server running
- [ ] Dioxus UI built and deployed
- [ ] Data migration executed
- [ ] Integration tests passing
- [ ] Performance benchmarks met
- [ ] Security scan passed
- [ ] Documentation updated

---

## Risk Management

### Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Rust skill gap | MEDIUM | HIGH | Training program; external consultants |
| Data migration issues | MEDIUM | HIGH | Comprehensive testing; rollback plan |
| Performance regression | LOW | MEDIUM | Benchmarks; optimization sprint |
| Integration defects | MEDIUM | MEDIUM | Extensive testing; parallel operation |
| Timeline overrun | HIGH | MEDIUM | Buffer in schedule; phased delivery |

### Rollback Plan

If critical issues arise:
1. Stop migration at current phase
2. Keep legacy systems operational
3. Revert any data changes
4. Regroup and reassess

---

## Resource Plan

### Team Composition

| Role | FTE | Duration | Person-Months |
|------|-----|----------|---------------|
| Rust Backend Lead | 2 | 8 months | 16 |
| Database Developer | 1 | 4 months | 4 |
| Frontend Developer (Dioxus) | 1 | 3 months | 3 |
| DevOps Engineer | 0.5 | 8 months | 4 |
| QA Engineer | 0.5 | 6 months | 3 |
| **Total** | **5** | | **30** |

### Budget Estimate

| Category | Cost |
|----------|------|
| Personnel (30 person-months @ €7k) | €210,000 |
| Training & External Support | €30,000 |
| Infrastructure (dev/staging) | €20,000 |
| Tools & Licenses (zero for OSS) | €0 |
| Contingency (20%) | €52,000 |
| **Total** | **€312,000** |

---

## Success Metrics

### Technical Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Build time | < 5 min | TBD |
| Test coverage | > 80% | TBD |
| API latency (p95) | < 100ms | TBD |
| WASM bundle size | < 5MB | TBD |
| Zero-downtime deployments | 100% | TBD |

### Business Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Development velocity | +30% | TBD |
| Operational complexity | -50% | TBD |
| Developer onboarding time | -40% | TBD |

---

## Next Steps

1. **Approval**: Present this plan to Architecture Board
2. **Resourcing**: Allocate budget and team
3. **Kickoff**: Start Phase 1
4. **Governance**: Weekly status reviews

---

**Document Owner**: Enterprise Architect
**Status**: DRAFT - Pending Approval
**Last Updated**: 2026-04-20
