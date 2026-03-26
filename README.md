# IOU-Modern

> **Informatie Ondersteunde Werkomgeving** - Modern Rust/WebAssembly implementatie

Een context-driven informatiemanagement platform voor Nederlandse overheidsorganisaties, gebouwd met Rust, WebAssembly en ArangoDB.

## 🚀 Technologie Stack

| Component | Technologie |
|-----------|-------------|
| **Core Library** | Rust with feature flags (WASM-compatible) |
| **Rules Engine** | PROVISA/BPMN/DMN via Open Regels |
| **GraphRAG** | petgraph + ArangoDB |
| **Frontend** | [Dioxus](https://dioxuslabs.com/) (WebAssembly) |
| **Maps** | Leaflet.js + Cesium (3D) |
| **NLP** | Rust regex NER |

## 📦 Project Structuur

```
iou-modern/
├── crates/
│   ├── iou-core/       # Gedeelde domain modellen (WASM-compatible)
│   └── iou-regels/     # PROVISA/BPMN/DMN rules engine
├── frontend/
│   └── crates/
│       └── iou-frontend/   # Dioxus WASM app (separate workspace)
├── docs/                # Architectuur documentatie
├── migrations/          # Database schema's
├── templates/           # Document templates (Markdown)
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
