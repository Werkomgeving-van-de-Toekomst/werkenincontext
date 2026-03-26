# Developer Onboarding Guide

**Last Updated:** 2026-03-26

---

## Quick Start

### Prerequisites

- Rust 1.80+ (Edition 2024)
- Git

### 1. Clone and Setup

```bash
# Clone repository
git clone https://github.com/Werkomgeving-van-de-Toekomst/werkenincontext.git
cd werkenincontext
```

### 2. Build Core Libraries

```bash
# Build all shared crates
cargo build --release

# Build specific crate
cargo build -p iou-core --release
cargo build -p iou-regels --release
```

### 3. Frontend Development

```bash
# Navigate to frontend workspace
cd frontend

# Development server with hot reload
dx serve --port 8080

# Production build
dx build --release
```

## Project Structure

```
iou-modern/
├── crates/
│   ├── iou-core/       # Shared domain models (WASM-compatible)
│   └── iou-regels/     # PROVISA/BPMN/DMN rules engine
├── frontend/
│   └── crates/
│       └── iou-frontend/   # Dioxus WASM app
├── docs/                # Architecture documentation
└── templates/           # Document templates
```

## Key Concepts

### iou-core
Platform-agnostic domain models that work on both native and WASM targets:
- Domain types (Informatiedomeinen)
- Compliance (Woo, AVG, Archiefwet)
- GraphRAG data types
- Delegation types
- Multi-tenancy support

### iou-regels
Dutch government rules engine:
- **PROVISA**: Provinciale selectielijst archiefwetgeving
- **BPMN**: Business Process Model and Notation
- **DMN**: Decision Model and Notation
- Open Regels integration

### Workspace Configuration
See [docs/workspaces.md](../workspaces.md) for details on the multi-workspace setup.

## Development Workflow

### Working on Core Library

```bash
# Make changes to crates/iou-core
cargo build -p iou-core
cargo test -p iou-core
```

### Working on Rules Engine

```bash
# Make changes to crates/iou-regels
cargo build -p iou-regels
cargo test -p iou-regels
```

### Working on Frontend

```bash
cd frontend
dx serve  # Auto-reload on changes
```

## Testing

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p iou-core
cargo test -p iou-regels

# With output
cargo test -- --nocapture
```

## Feature Flags

### iou-core

```toml
[features]
default = ["server"]  # Native builds include server features
wasm = []             # WASM builds exclude server dependencies
server = [
    "tokio",
    "reqwest",
    "sqlx",
    "arangors",
    # ... other server-only deps
]
```

## Common Tasks

### Adding a new dependency

1. **Shared dependency**: Add to root `Cargo.toml` `[workspace.dependencies]`
2. **Frontend-only**: Add to `frontend/Cargo.toml`

### Running the frontend locally

```bash
cd frontend
dx serve
# Open http://localhost:8080
```

## Resources

- [README.md](../../README.md) - Project overview
- [workspaces.md](../workspaces.md) - Workspace structure details
- [architecture/](../architecture/) - Architecture documentation

## Troubleshooting

### "tokio doesn't work on WASM"
Expected. Use root workspace for native builds, frontend workspace for WASM.

### Feature flag errors
Make sure to build with correct features:
- Native: `cargo build` (default features)
- WASM: `cd frontend && dx build` (automatically uses wasm feature)

### Path dependency issues
When frontend depends on shared crates:
```toml
iou-core = { path = "../../crates/iou-core" }
```
