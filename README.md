# IOU-Modern

> **Informatie Ondersteunde Werkomgeving** - Modern Rust/WebAssembly implementatie

Een context-driven informatiemanagement platform voor Nederlandse overheidsorganisaties, gebouwd met Rust, WebAssembly en DuckDB.

## 🚀 Technologie Stack

| Component | Technologie |
|-----------|-------------|
| **Backend API** | [Axum](https://github.com/tokio-rs/axum) |
| **Database** | [DuckDB](https://duckdb.org/) (embedded analytical) |
| **Frontend** | [Dioxus](https://dioxuslabs.com/) 0.7 (WebAssembly) |
| **Charts** | TBD |
| **Maps** | Leaflet.js via wasm-bindgen |
| **AI/NLP** | Rust regex NER + petgraph |
| **GraphRAG** | petgraph |

## 📦 Project Structuur

```
iou-modern/
├── crates/
│   ├── iou-core/       # Gedeelde domain modellen
│   ├── iou-api/        # REST API (Axum + DuckDB)
│   ├── iou-ai/         # AI services (NER, GraphRAG)
│   └── iou-frontend/   # Dioxus WASM app
├── migrations/         # DuckDB schema
└── data/              # DuckDB database file
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
git clone https://github.com/terminal-woo/iou-modern.git
cd iou-modern

# Build all crates
cargo build

# Run API server (met DuckDB)
cargo run -p iou-api

# API beschikbaar op http://localhost:8000
# Health check: http://localhost:8000/health
```

### Frontend Development

```bash
cd crates/iou-frontend

# Development server met hot reload
dx serve

# Production build
dx build --release
```

## 🗄️ Database (DuckDB)

DuckDB is een embedded analytical database - geen server nodig!

**Voordelen:**
- ✅ Single-file deployment
- ✅ Analytisch geoptimaliseerd (columnar storage)
- ✅ Full SQL support
- ✅ Direct Parquet/CSV/JSON lezen
- ✅ Zero configuratie

**Schema initialiseren:**
De API initialiseert automatisch het schema bij eerste start.

```bash
# Database locatie (configureerbaar via DATABASE_PATH)
data/iou-modern.duckdb
```

## 🔌 API Endpoints

| Endpoint | Methode | Beschrijving |
|----------|---------|--------------|
| `/health` | GET | Health check |
| `/context/{id}` | GET | Volledige context met related domains |
| `/domains` | GET | Lijst alle domeinen |
| `/domains` | POST | Nieuw informatiedomein aanmaken |
| `/objects` | POST | Nieuw informatieobject met compliance |
| `/objects/{id}` | GET | Informatieobject ophalen |
| `/search?q=` | GET | Full-text zoeken |
| `/apps/recommended` | GET | Context-aware app aanbevelingen |
| `/graphrag/relations/{id}` | GET | Gerelateerde domeinen via kennisgraaf |
| `/graphrag/entities` | GET | Alle entiteiten |
| `/graphrag/communities` | GET | Community clusters |

## 🎯 Kernconcepten

### Informatiedomein
Centrale organiserende eenheid. Vier typen:
- **Zaak**: Uitvoerend werk (vergunningen, subsidies, bezwaren)
- **Project**: Tijdelijke samenwerkingsinitiatieven
- **Beleid**: Beleidsontwikkeling en -evaluatie
- **Expertise**: Kennisdeling en samenwerking

### Compliance by Design
Automatische naleving van:
- **Woo** (Wet open overheid): Openbaarmaking
- **AVG** (GDPR): Privacy
- **Archiefwet**: Bewaring en vernietiging

### GraphRAG
Automatische relatiedetectie via:
- Named Entity Recognition (provincies, gemeentes, wetten)
- Graph algorithms (community detection)
- Semantic similarity

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p iou-core
cargo test -p iou-ai

# With coverage (requires cargo-tarpaulin)
cargo tarpaulin
```

## 📝 Environment Variables

```bash
# .env
HOST=0.0.0.0
PORT=8000
DATABASE_PATH=data/iou-modern.duckdb
JWT_SECRET=your-secret-here
RUST_LOG=info
```

## 🚢 Deployment

### GitHub Pages (Frontend only)

```bash
cd crates/iou-frontend
dx build --release
# Deploy target/dx/iou-frontend/release/web/public/ to GitHub Pages
```

### Docker

```dockerfile
# Dockerfile
FROM rust:1.83 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p iou-api

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/iou-api /usr/local/bin/
CMD ["iou-api"]
```

## 📚 Gebaseerd op

- [IOU-concept](https://github.com/terminal-woo/iou-concept) - Originele Python/FastAPI implementatie
- [Dashboard Studio RS](https://blog.gopenai.com/dashboard-studio-rs-how-webassembly-and-rust-are-reshaping-data-visualization-in-bi-ef950243f700) - Architectuur inspiratie

## 📄 Licentie

MIT
