# IOU-Modern

> **Informatie Ondersteunde Werkomgeving** - Modern Rust/WebAssembly implementatie

Een context-driven informatiemanagement platform voor Nederlandse overheidsorganisaties, gebouwd met Rust, WebAssembly en DuckDB.

## 🚀 Technologie Stack

| Component | Technologie |
|-----------|-------------|
| **Backend API** | [Axum](https://github.com/tokio-rs/axum) |
| **Database** | [DuckDB](https://duckdb.org/) (embedded analytical) |
| **Frontend** | [Dioxus](https://dioxuslabs.com/) 0.7 (WebAssembly) |
| **AI Agents** | Rust multi-agent pipeline |
| **Charts** | TBD |
| **Maps** | Leaflet.js via wasm-bindgen |
| **NLP** | Rust regex NER + petgraph |
| **GraphRAG** | petgraph |

## 📦 Project Structuur

```
iou-modern/
├── crates/
│   ├── iou-core/       # Gedeelde domain modellen
│   ├── iou-api/        # REST API (Axum + DuckDB)
│   ├── iou-ai/         # AI agents (Research, Content, Compliance, Review)
│   ├── iou-storage/    # S3/MinIO storage client
│   └── iou-frontend/   # Dioxus WASM app
├── migrations/         # DuckDB schema
├── templates/          # Document templates (Markdown)
├── scripts/            # Utility scripts
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
# Development server met hot reload (vanuit project root)
dx serve --package iou-frontend --port 8080

# Of met expliciet poortnummer
dx serve --package iou-frontend

# Production build
dx build --release --package iou-frontend
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

### Core API

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

### Document Creation API

| Endpoint | Methode | Beschrijving |
|----------|---------|--------------|
| `/api/documents/create` | POST | Start nieuwe document generatie |
| `/api/documents/{id}/status` | GET | Haal document status op |
| `/api/documents/{id}/approve` | POST | Keur document goed/af |
| `/api/documents/{id}/audit` | GET | Haal audit trail op |
| `/api/documents/{id}/download` | GET | Download gegenereerd document |
| `/api/templates` | GET | Lijst alle templates |
| `/api/templates` | POST | Maak nieuwe template |
| `/api/templates/{id}` | GET | Haal template op |
| `/api/templates/{id}` | PUT | Update template |
| `/api/templates/{id}` | DELETE | Verwijder template |

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

## 🤖 Document Creation System

### Multi-Agent Pipeline

Het document creation systeem gebruikt een multi-agent pipeline om automatisch Woo-compliant documenten te genereren:

```
User Input → Research Agent → Content Agent → Compliance Agent → Review Agent → Document
                                              ↓
                                         Approval Workflow → Publicatie
```

**Agents:**
1. **Research Agent** - Haalt relevante context op uit de kennisgraaf
2. **Content Agent** - Genereert document content op basis van templates
3. **Compliance Agent** - Valideert Woo/GDPR compliance en roept PII automatisch
4. **Review Agent** - Kwaliteitscontrole en goedkeuring logic

### Frontend Pagina's

| Pagina | URL | Beschrijving |
|-------|-----|------------|
| Document Maken | `/documenten/maken` | Start document generatie |
| Goedkeuring Wachtrij | `/documenten/wachtrij` | Beheer goed te keuren documenten |
| Template Beheer | `/templates` | CRUD voor document templates |

### Templates

Document templates worden opgeslagen als Markdown met Tera variable syntax:

```markdown
# {{ document_type }}

**Referentie:** {{ reference_number }}
**Datum:** {{ date }}

## Aanvraag

Op {{ request_date }} heeft {{ requester }} een verzoek ingediend.

{% if approval_granted %}
## Besluit
Het verzoek wordt toegekend.
{% endif %}
```

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
