# Workspace Structure

IOU-Modern uses **separate workspaces** for server and WASM builds to resolve tokio compatibility issues with `wasm32-unknown-unknown`.

## Directory Structure

```
iou-modern/
├── Cargo.toml                    # Root workspace (shared crates)
├── crates/
│   ├── iou-core/                 # Platform-agnostic domain models
│   └── iou-regels/               # Open Regels integration (shared)
├── server/
│   ├── Cargo.toml                # Server workspace
│   └── crates/
│       ├── iou-api/              # REST API server
│       ├── iou-storage/          # S3/MinIO storage
│       ├── iou-orchestrator/     # Workflow orchestration
│       ├── iou-ai/                # AI/ML services
│       ├── iou-ai-service/       # AI HTTP gateway
│       └── iou-camunda-worker/   # Zeebe job worker
└── frontend/
    ├── Cargo.toml                # Frontend workspace (WASM-compatible)
    └── crates/
        └── iou-frontend/         # Dioxus WASM frontend
```

## Workspaces

| Workspace | Location | Target | Purpose |
|-----------|----------|--------|---------|
| Root | `Cargo.toml` | - | Shared crates (iou-core, iou-regels) |
| Server | `server/Cargo.toml` | native | Server-only crates with tokio |
| Frontend | `frontend/Cargo.toml` | wasm32 | WASM-compatible frontend |

## Crate Classification

### Shared Crates (root workspace)
| Crate | Server | WASM | Notes |
|-------|--------|------|-------|
| `iou-core` | ✓ | ✓ | Domain models with feature flags |
| `iou-regels` | ✓ | ✓ | Open Regels integration |

### Server-Only Crates
| Crate | WASM | Dependencies |
|-------|------|-------------|
| `iou-api` | ✗ | Axum + tokio + sqlx |
| `iou-storage` | ✗ | AWS SDK + tokio |
| `iou-orchestrator` | ✗ | tokio sync primitives |
| `iou-ai` | ✗ | tokio + HTTP client |
| `iou-ai-service` | ✗ | tokio server |
| `iou-camunda-worker` | ✗ | tokio worker |

### Frontend-Only
| Crate | Server | WASM | Notes |
|-------|--------|------|-------|
| `iou-frontend` | ✗ | ✓ | Dioxus web framework |

## Building

### Server (Native)
```bash
# Build all server crates
cargo build --manifest-path server/Cargo.toml --release

# Run server
cargo run --manifest-path server/Cargo.toml --bin iou-api
```

### Frontend (WASM)
```bash
# Option 1: Use the build script
./scripts/build-wasm.sh

# Option 2: Manual build
# First build iou-core for WASM
cargo build --manifest-path frontend/Cargo.toml --package iou-core --target wasm32-unknown-unknown --release

# Then bundle with Dioxus
dx build --manifest-path frontend/Cargo.toml --release
```

### Development

```bash
# Server development (hot reload)
cargo build --manifest-path server/Cargo.toml

# Frontend development (Dioxus hot reload)
dx serve --manifest-path frontend/Cargo.toml
```

## Feature Flags in iou-core

```toml
[features]
default = ["server"]  # For native builds
wasm = []             # No server dependencies
server = [            # All the extras
    "tokio",
    "reqwest",
    "http",
    "async-trait",
    "sqlx",
    "arangors",
    "mobc",
    "mobc-arangors",
    "notify",
]
```

## Module Availability

### Always Available (WASM-compatible)
- `domain` - Informatiedomeinen types
- `objects` - Informatieobjecten types
- `compliance` - Woo, AVG, Archiefwet types
- `organization` - Organisatie types
- `graphrag/types` - Knowledge graph data types
- `api_types` - API request/response types
- `workflows` - Workflow status en definities
- `delegation/types` - Delegatie types
- `config` - Configuratie types
- `diff` - Document diff generatie
- `document` - Document types
- `sla` - SLA calculatie types
- `tenancy` - Multi-tenant context types

### Server-Only (require "server" feature)
- `audit` - Audit logging met PostgreSQL
- `storage` - S3/MinIO storage
- `versions` - Document versioning
- `ssi` - SSI/VC met DID resolution
- `realtime` - WebSocket communicatie
- `escalation` - Escalatie services
- `delegation/service` - Delegatie CRUD
- `delegation/resolver` - Delegatie resolution
- `graphrag/store` - ArangoDB persistence

## Troubleshooting

### "tokio doesn't work on WASM"
This is expected when building server crates for WASM. Use the appropriate workspace:
- Server builds: `--manifest-path server/Cargo.toml`
- WASM builds: `--manifest-path frontend/Cargo.toml`

### Workspace dependency errors
When adding new dependencies:
- For shared crates (iou-core, iou-regels): Add to root `Cargo.toml`
- For server crates: Add to `server/Cargo.toml`
- For frontend crates: Add to `frontend/Cargo.toml`

### Path dependencies
When crates reference shared crates:
- From server/crates: use `iou-core = { path = "../crates/iou-core" }` (relative to server/)
- From frontend/crates: use `iou-core = { path = "../crates/iou-core" }` (relative to frontend/)

## Architecture Benefits

1. **Clean separation**: Server and frontend have isolated dependency trees
2. **No tokio conflicts**: WASM builds never see tokio dependencies
3. **Shared code**: iou-core and iou-regels are used by both workspaces
4. **Independent development**: Server and frontend can build independently
