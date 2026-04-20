# IOU-Modern Codebase Structure

## Workspace Architecture

The IOU-Modern platform uses Cargo workspaces to organize code by project and responsibility.

```
iou-modern/
├── Cargo.toml                    # Root workspace (shared crates)
│
├── crates/                       # Platform-agnostic shared libraries
│   ├── iou-core/                  # Core types (Id, Timestamp, etc.)
│   └── iou-regels/                # Open Regels integration
│
├── metadata-registry/            # Project 002: Metadata Registry Service
│   ├── Cargo.toml                # Service workspace
│   └── crates/
│       ├── metadata-core/        # Core types and GGHH V2 models
│       ├── metadata-store/       # ArangoDB storage layer
│       ├── metadata-api/         # REST/GraphQL APIs
│       ├── metadata-gitops/      # GitOps synchronization
│       ├── metadata-validation/  # TOOI/MDTO validation
│       ├── metadata-admin/       # Dioxus admin UI
│       └── metadata-migration/   # Database migrations
│
├── context-aware/                # Project 003: Context-Aware Data Services
│   ├── Cargo.toml                # Service workspace
│   └── crates/
│       ├── context-core/         # Core context model (GGHH V2 aligned)
│       ├── context-api/          # Context Capture API
│       ├── context-domain/       # Domain context implementation
│       ├── context-semantic/     # Semantic context & entity extraction
│       ├── context-provenance/   # Provenance & lineage tracking
│       ├── context-inference/    # AI-powered context enrichment
│       ├── context-quality/      # Quality scoring & validation
│       └── context-store/        # ArangoDB graph storage
│
├── server/                       # Project 001: Backend Services
│   ├── Cargo.toml                # Server workspace
│   └── crates/
│       ├── iou-api/              # Main REST API (Axum)
│       ├── iou-storage/          # S3 + local storage
│       ├── iou-orchestrator/     # Workflow orchestration
│       ├── iou-ai/               # AI service integration
│       ├── iou-ai-service/       # AI processing workers
│       └── iou-camunda-worker/   # Camunda workflow worker
│
├── frontend/                     # Project 001: User Interface
│   ├── Cargo.toml                # Frontend workspace
│   └── crates/
│       └── iou-frontend/         # Leptos/Dioxus UI
│
└── projects/                     # ArcKit documentation
    ├── 001-iou-modern/           # Main platform requirements & design
    ├── 002-metadata-registry/    # Metadata Registry artifacts
    └── 003-context-aware-data/   # Context-Aware Data artifacts
```

## Project Relationships

| Project | Directory | Purpose | Dependencies |
|---------|-----------|---------|--------------|
| **001** IOU-Modern | `server/`, `frontend/` | Main platform | Uses 002, 003 services |
| **002** Metadata Registry | `metadata-registry/` | GGHH V2 metamodel | Independent service |
| **003** Context-Aware | `context-aware/` | Context services | Uses 002 for metadata |

## Key Integration Points

### 1. Metadata Registry (002) → IOU-Modern (001)
```rust
// In server/crates/iou-api/src/metadata.rs
use metadata_api::Client;

pub struct MetadataService {
    client: Client,
}

impl MetadataService {
    pub async fn get_context(&self, info_id: Uuid) -> Result<Context> {
        self.client.get_context(info_id).await
    }
}
```

### 2. Context-Aware (003) → Metadata Registry (002)
```rust
// In context-aware/crates/context-store/src/arango.rs
use metadata_core::{Informatieobject, Context as MetadataContext};

pub struct ContextStore {
    arango: arangors::Client,
    metadata_client: metadata_api::Client,
}

impl ContextStore {
    pub async fn create(&self, context: &Context) -> Result<()> {
        // Store context with link to metadata
        self.metadata_client.link_context(
            context.informatieobject_id,
            context.id
        ).await?;
        Ok(())
    }
}
```

### 3. Context-Aware (003) → IOU-Modern (001)
```rust
// In server/crates/iou-api/src/context.rs
use context_api::ContextClient;

pub struct DocumentService {
    context_client: ContextClient,
}

impl DocumentService {
    pub async fn enrich_document(&self, doc_id: Uuid) -> Result<Document> {
        let context = self.context_client.capture_context(doc_id).await?;
        // Enrich document with context
        Ok(Document { context, .. })
    }
}
```

## Build Commands

```bash
# Build all workspaces
cargo build --workspace

# Build specific workspace
cargo build --manifest-path metadata-registry/Cargo.toml
cargo build --manifest-path context-aware/Cargo.toml
cargo build --manifest-path server/Cargo.toml
cargo build --manifest-path frontend/Cargo.toml

# Run services
cargo run --manifest-path metadata-registry/Cargo.toml --bin metadata-api
cargo run --manifest-path context-aware/Cargo.toml --bin context-api
cargo run --manifest-path server/Cargo.toml --bin iou-api
```

## Development Workflow

1. **Implement in 002**: Add new metadata types to `metadata-core`
2. **Integrate in 003**: Build context services using 002's metadata
3. **Expose via 001**: Orchestrate through main API for frontend

## Shared Type System

All workspaces share types from `crates/iou-core`:

```rust
// In crates/iou-core/src/lib.rs
pub type Id<T> = Uuid;  // Typed UUID wrapper
pub type Timestamp = i64;  // Unix milliseconds
pub type OrganisationId = Id<Organisation>;
```

This ensures type safety across workspace boundaries.
