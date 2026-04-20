# IOU-Modern

> **Informatie Ondersteunde Werkomgeving** - Modern Rust/WebAssembly implementatie

Een context-driven informatiemanagement platform voor Nederlandse overheidsorganisaties, gebouwd met Rust, WebAssembly en ArangoDB.

## 🚀 Technologie Stack

| Component | Technologie |
|-----------|-------------|
| **Core Library** | Rust with feature flags (WASM-compatible) |
| **Rules Engine** | PROVISA/BPMN/DMN via Open Regels |
| **GraphRAG** | petgraph + DuckDB (analytical) + PostgreSQL (transactional) |
| **Frontend** | [Dioxus](https://dioxuslabs.com/) (WebAssembly) |
| **Maps** | Leaflet.js + Cesium (3D) |
| **NLP** | Rust regex NER + AI agents |

## 📦 Project Structuur

```
iou-modern/
├── crates/
│   ├── iou-core/       # Gedeelde domain modellen (WASM-compatible)
│   └── iou-regels/     # PROVISA/BPMN/DMN rules engine
├── server/
│   ├── crates/
│   │   ├── iou-api/         # Axum HTTP server + WebSocket
│   │   ├── iou-storage/     # PostgreSQL + S3 storage layer
│   │   ├── iou-ai/          # AI agents + NER + compliance
│   │   ├── iou-ai-service/  # AI service binary
│   │   ├── iou-camunda-worker/ # Camunda 8 workflow worker
│   │   └── iou-orchestrator/  # Workflow orchestration
│   └── Cargo.toml       # Server workspace
├── frontend/
│   └── crates/
│       └── iou-frontend/   # Dioxus WASM app (separate workspace)
├── docs/                # Architectuur documentatie (ArcKit)
├── migrations/          # PostgreSQL schema's (SQLx)
├── projects/            # ArcKit project artifacts
│   ├── 000-global/      # Global architecture principles
│   ├── 001-iou-modern/  # Main project documentation
│   ├── 002-metadata-registry/
│   └── 003-context-aware-data/
└── static/              # Statische HTML pagina's
```

## 🛠️ Development Setup

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# WASM target
rustup target add wasm32-unknown-unknown

# Dioxus CLI
cargo install dioxus-cli

# Optional: cargo-watch for auto-reload
cargo install cargo-watch
```

### Build & Run

```bash
# Clone repository
git clone https://github.com/Werkomgeving-van-de-Toekomst/werkenincontext.git
cd werkenincontext

# Build all crates
cargo build

# Build core library only
cargo build -p iou-core

# Build rules engine
cargo build -p iou-regels
```

### Frontend Development

```bash
# Development server met hot reload
cd frontend
dx serve --port 8080

# Production build
dx build --release
```

### Database Setup

```bash
# PostgreSQL (development)
docker run -d -p 5432:5432 \
  -e POSTGRES_DB=iou_modern \
  -e POSTGRES_USER=iou \
  -e POSTGRES_PASSWORD=dev \
  postgres:16

# Run migrations
cargo run --bin iou-api -- migrate

# DuckDB (embedded, no setup required)
# Analytical queries and full-text search
```

### Server Development

```bash
# Run API server
cd server
cargo run --bin iou-api

# Run AI service
cargo run --bin iou-ai-service

# Run Camunda worker
cargo run --bin iou-camunda-worker
```

## 🎯 Kernconcepten

### Informatiedomein
Centrale organiserende eenheid. Vier typen:
- **Zaak**: Uitvoerend werk (vergunningen, subsidies, bezwaren)
- **Project**: Tijdelijke samenwerkingsinitiatieven
- **Beleid**: Beleidsontwikkeling en -evaluatie
- **Expertise**: Kennisdeling en samenwerking

### PROVISA (Provinciale Selectielijst)
Automatische naleving van provinciale archiefwetgeving:
- **Bewaartermijnen** per documenttype
- **Hotspot** detectie voor upgrade naar permanente bewaring
- **PETRA** categorie classificatie

### GraphRAG
Automatische relatiedetectie via:
- Named Entity Recognition (provincies, gemeentes, wetten)
- Community detection algoritmes
- Semantic similarity

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p iou-core
cargo test -p iou-regels

# With coverage (requires cargo-tarpaulin)
cargo tarpaulin
```

## 📝 Workspace Structure

Zie [docs/workspaces.md](docs/workspaces.md) voor details over de multi-workspace setup.

| Workspace | Location | Target | Purpose |
|-----------|----------|--------|---------|
| Root | `Cargo.toml` | - | Shared crates (iou-core, iou-regels) |
| Frontend | `frontend/Cargo.toml` | wasm32 | WASM-compatible frontend |

## 📚 Gebaseerd op

- [IOU-concept](https://github.com/terminal-woo/iou-concept) - Originele Python/FastAPI implementatie
- [PROVISA 2020](https://www.provincie.nl/onderwerpen/archief/selectielijsten) - Provinciale selectielijst

## 📄 Licentie

MIT
