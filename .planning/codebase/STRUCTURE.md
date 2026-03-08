# Codebase Structure

**Analysis Date:** 2026-03-08

## Directory Layout

```
iou-modern/
├── crates/                      # Rust workspace crates
│   ├── iou-core/               # Shared domain models
│   ├── iou-api/                # REST API server
│   ├── iou-ai/                 # AI agents and services
│   ├── iou-regels/             # Business rules engine
│   ├── iou-storage/            # S3/MinIO abstraction
│   └── iou-frontend/           # Dioxus WASM app
├── migrations/                 # DuckDB schema migrations
├── templates/                  # Document templates (Markdown)
├── scripts/                    # Build and utility scripts
├── data/                       # DuckDB database file location
├── static/                     # Static web assets
└── .planning/                  # Generated analysis documents
```

## Directory Purposes

**[crates/]:**
- Purpose: Rust workspace with modular architecture
- Contains: Loosely coupled crates with clear boundaries
- Key files: `Cargo.toml` workspace manifest

**[crates/iou-core/]:**
- Purpose: Domain models and business logic
- Contains: Domain entities, value objects, workflows
- Key files: `domain.rs`, `objects.rs`, `compliance.rs`, `workflows.rs`

**[crates/iou-api/]:**
- Purpose: HTTP API server and business logic
- Contains: Axum routes, middleware, database access, workflow engine
- Key files: `main.rs`, `routes/`, `workflows/`, `db.rs`

**[crates/iou-ai/]:**
- Purpose: AI/ML services and multi-agent pipeline
- Contains: NER, GraphRAG, compliance, document generation agents
- Key files: `agents/`, `graphrag.rs`, `ner.rs`, `llm.rs`

**[crates/iou-frontend/]:**
- Purpose: WebAssembly UI application
- Contains: Dioxus components, pages, API client
- Key files: `main.rs`, `components/`, `pages/`, `state/`

**[migrations/]:**
- Purpose: Database schema migrations for DuckDB
- Contains: SQL migration scripts
- Key files: SQL files for versioned schema changes

**[templates/]:**
- Purpose: Document generation templates
- Contains: Markdown files with Tera syntax
- Key files: Template documents for various domain types

**[scripts/]:**
- Purpose: Build and utility scripts
- Contains: Dev helpers, build scripts, deployment tools
- Key files: `generate-ahn-terrain.sh`

**[data/]:**
- Purpose: DuckDB database storage
- Contains: Embedded database files
- Key files: `iou-modern.duckdb`

## Key File Locations

**Entry Points:**
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/main.rs`: API server
- `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/main.rs`: Frontend app
- `/Users/marc/Projecten/iou-modern/Cargo.toml`: Workspace manifest

**Configuration:**
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/config.rs`: API configuration
- `/Users/marc/Projecten/iou-modern/crates/iou-frontend/assets/style.css`: Frontend styles

**Core Logic:**
- `/Users/marc/Projecten/iou-modern/crates/iou-core/src/lib.rs`: Domain models
- `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/pipeline.rs`: Document pipeline
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/workflows/mod.rs`: Workflow engine

**Testing:**
- No dedicated test structure detected in codebase

## Naming Conventions

**Files:**
- Rust files: `snake_case.rs` (e.g., `domain.rs`, `user_auth.rs`)
- Config files: `kebab-case.toml` (e.g., `config.toml`)
- Templates: `kebab-case.md` (e.g., `zaak-template.md`)
- JavaScript: `camelCase.js` (frontend WASM bindings)

**Directories:**
- Modules: `snake_case/` (e.g., `src/components/`, `src/pages/`)
- Feature-specific: `feature_name/` (e.g., `src/pages/flevoland/`)

**Types:**
- Structs: PascalCase (e.g., `InformationDomain`, `UserInfo`)
- Enums: PascalCase (e.g., `DomainType`, `WorkflowStatus`)
- Functions: snake_case (e.g., `create_domain`, `get_context`)
- Constants: SCREAMING_SNAKE_CASE (e.g., `MAX_ITERATIONS`)

**Variables:**
- Parameters: snake_case (e.g., `user_id`, `request_body`)
- Local: snake_case (e.g., `domain_list`, `new_context`)
- Module-level: snake_case (e.g., `db_connection`)

## Where to Add New Code

**New API Endpoint:**
- Primary code: `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/`
- Handler: Create new file in `routes/` or add to existing file
- Tests: Add to integration test (if any)

**New Component/Module:**
- Implementation: `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/`
- Page: `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/`
- State update: `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/state/`

**New Domain Entity:**
- Model: `/Users/marc/Projecten/iou-modern/crates/iou-core/src/`
- API types: Update `api_types.rs`
- Serialization: Ensure derive `serde::Serialize`/`Deserialize`

**New AI Agent:**
- Agent code: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/`
- Pipeline integration: Update `pipeline.rs`
- Configuration: Add to `config.rs`

**New Document Template:**
- Location: `/Users/marc/Projecten/iou-modern/templates/`
- Format: Markdown with Tera syntax
- Registration: Add to template registry

**New Business Rule:**
- Implementation: `/Users/marc/Projecten/iou-modern/crates/iou-regels/src/`
- Integration: Use from AI or API layer

## Special Directories

**[crates/iou-frontend/src/pages/]:**
- Purpose: Page-specific components and logic
- Contains: Feature-based page modules (flevoland, minfin, zuidholland)
- Generated: No
- Committed: Yes

**[crates/iou-frontend/assets/]:**
- Purpose: Static assets for frontend
- Contains: CSS, geodata, terrain tiles
- Generated: Partially (terrain tiles)
- Committed: Yes

**[static/]:**
- Purpose: Web root for static files
- Contains: WASM builds, static HTML
- Generated: Yes (from frontend build)
- Committed: Yes (for deployment)

**[data/]:**
- Purpose: Database storage directory
- Contains: DuckDB database files
- Generated: Yes
- Committed: No (excluded in .gitignore)

**[templates/]:**
- Purpose: Document template storage
- Contains: Markdown templates for document generation
- Generated: No
- Committed: Yes

---

*Structure analysis: 2026-03-08*