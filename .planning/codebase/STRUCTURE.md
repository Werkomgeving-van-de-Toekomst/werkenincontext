# Codebase Structure

**Analysis Date:** 2026-03-08

## Directory Layout

```
[iou-modern]/
├── crates/                # Rust workspace crates
│   ├── iou-api/           # API server (Axum)
│   ├── iou-core/          # Domain models
│   ├── iou-frontend/      # WebAssembly frontend (Dioxus)
│   ├── iou-ai/            # AI services
│   ├── iou-storage/        # Storage services
│   └── iou-regels/         # Rules engine
├── migrations/           # Database migrations
├── scripts/              # Utility scripts
├── static/               # Static assets
├── data/                 # Data files
└── target/                # Build artifacts
```

## Directory Purposes

**crates/:**
- Purpose: Contains all Rust workspace members
- Contains: Individual crate directories
- Key files: `Cargo.toml` (workspace definition)

**crates/iou-api/:**
- Purpose: API server implementation
- Contains: Route handlers, middleware, database access
- Key files: `src/main.rs`, `src/routes/`, `src/db.rs`

**crates/iou-core/:**
- Purpose: Domain models and business logic
- Contains: Domain entities, value objects, domain services
- Key files: `src/lib.rs`, domain modules

**crates/iou-frontend/:**
- Purpose: WebAssembly frontend
- Contains: Pages, components, state management, API clients
- Key files: `src/main.rs`, `src/pages/`, `src/components/`

**migrations/:**
- Purpose: Database schema migrations
- Contains: SQL migration scripts
- Key files: Versioned migration files

## Key File Locations

**Entry Points:**
- `crates/iou-api/src/main.rs`: API server entry point
- `crates/iou-frontend/src/main.rs`: Frontend entry point

**Configuration:**
- `Cargo.toml`: Workspace configuration
- `crates/*/Cargo.toml`: Individual crate configurations

**Core Logic:**
- `crates/iou-core/src/`: Domain models and business logic
- `crates/iou-api/src/routes/`: API endpoints
- `crates/iou-api/src/workflows/`: Workflow management

**Testing:**
- Tests are co-located with source files in `src/` directories

## Naming Conventions

**Files:**
- `mod.rs`: Module definition files
- Snake_case for Rust files: `document.rs`, `compliance.rs`

**Directories:**
- Snake_case for directories: `crates/`, `migrations/`
- Domain-specific grouping: `routes/`, `components/`, `pages/`

## Where to Add New Code

**New Feature:**
- Primary code: Add to appropriate crate based on domain
- Tests: Co-locate with implementation files

**New Component/Module:**
- Implementation: Add to relevant crate directory
- Domain logic: `crates/iou-core/src/`
- API endpoints: `crates/iou-api/src/routes/`
- UI components: `crates/iou-frontend/src/components/`

**Utilities:**
- Shared helpers: Add to domain crate or create new utility crate

## Special Directories

**migrations/:**
- Purpose: Database schema versioning
- Generated: Yes (by migration tools)
- Committed: Yes

**target/:**
- Purpose: Build artifacts
- Generated: Yes (by Rust compiler)
- Committed: No (in .gitignore)

---

*Structure analysis: 2026-03-08*