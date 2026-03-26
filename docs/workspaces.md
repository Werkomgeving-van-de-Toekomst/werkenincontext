# Workspace Structure

IOU-Modern uses **separate workspaces** for shared code and WASM builds to ensure compatibility.

## Directory Structure

```
iou-modern/
├── Cargo.toml                    # Root workspace (shared crates)
├── crates/
│   ├── iou-core/                 # Platform-agnostic domain models
│   └── iou-regels/               # PROVISA/BPMN/DMN rules engine
└── frontend/
    ├── Cargo.toml                # Frontend workspace (WASM-compatible)
    └── crates/
        └── iou-frontend/         # Dioxus WASM frontend
```

## Workspaces

| Workspace | Location | Target | Purpose |
|-----------|----------|--------|---------|
| Root | `Cargo.toml` | native | Shared crates (iou-core, iou-regels) |
| Frontend | `frontend/Cargo.toml` | wasm32 | WASM-compatible frontend |

## Crate Classification

### Shared Crates (root workspace)

| Crate | Frontend | Notes |
|-------|----------|-------|
| `iou-core` | ✓ | Domain models with feature flags |
| `iou-regels` | ✓ | Open Regels / PROVISA integration |

### Frontend-Only

| Crate | Server | WASM | Notes |
|-------|--------|------|-------|
| `iou-frontend` | ✗ | ✓ | Dioxus web framework |

## Building

### Core Libraries (Native)
```bash
# Build all shared crates
cargo build --release

# Build specific crate
cargo build -p iou-core --release
cargo build -p iou-regels --release
```

### Frontend (WASM)
```bash
# Development with hot reload
cd frontend
dx serve --port 8080

# Production build
dx build --release
```

## Feature Flags in iou-core

```toml
[features]
default = ["server"]  # For native builds
wasm = []             # No server dependencies
server = [
    "tokio",
    "reqwest",
    "sqlx",
    "arangors",
    # ... other server-only deps
]
```

## Module Availability

### Always Available (WASM-compatible)
- `domain` - Informatiedomeinen types
- `objects` - Informatieobjecten types
- `compliance` - Woo, AVG, Archiefwet types
- `graphrag/types` - Knowledge graph data types
- `delegation/types` - Delegatie types
- `config` - Configuratie types
- `tenancy` - Multi-tenant context types
- `escalation/types` - Escalatie types

### Server-Only (require "server" feature)
- `audit` - Audit logging
- `storage` - Storage abstraction
- `graphrag/store` - ArangoDB persistence
- `delegation/service` - Delegatie CRUD
- `escalation/service` - Escalatie services

## Troubleshooting

### "tokio doesn't work on WASM"
This is expected when building server crates for WASM. Use the appropriate workspace:
- Native builds: Use root `Cargo.toml`
- WASM builds: Use `frontend/Cargo.toml`

### Path dependencies
When crates reference shared crates:
- From frontend/crates: use `iou-core = { path = "../../crates/iou-core" }`

## Architecture Benefits

1. **Clean separation**: Frontend has isolated dependency tree
2. **No tokio conflicts**: WASM builds never see tokio dependencies
3. **Shared code**: iou-core and iou-regels used by both
4. **Simplified structure**: Only two workspaces to maintain
