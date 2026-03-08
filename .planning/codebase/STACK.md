# Technology Stack

**Analysis Date:** 2026-03-08

## Languages

**Primary:**
- Rust 2024 edition - Core backend, API server, and AI services
  - Location: `crates/iou-api`, `crates/iou-ai`, `crates/iou-regels`, `crates/iou-storage`
  - Workspace-managed dependencies

**Secondary:**
- JavaScript (via WebAssembly) - Frontend rendering
  - Location: `crates/iou-frontend`
  - Compiled to WASM using Dioxus framework

## Runtime

**Environment:**
- Rust 1.43+ with Tokio async runtime
- WebAssembly (for frontend)

**Package Manager:**
- Cargo (Rust package manager)
- Lockfile: `Cargo.lock` (workspace-level)

## Frameworks

**Core:**
- Axum 0.8 - Web framework for REST API
  - Location: `crates/iou-api/src/main.rs`
  - Features: CORS, tracing, static file serving

**Frontend:**
- Dioxus 0.7 - WebAssembly UI framework
  - Location: `crates/iou-frontend`
  - Features: Router, desktop/web builds

**Testing:**
- tokio-test 0.4 - Async testing utilities
- serial_test 3 - Serial test execution
- approx 0.5 - Floating-point comparison

**Build/Dev:**
- Dioxus CLI - Frontend development and builds
- cargo-watch - Optional auto-reload during development

## Key Dependencies

**Critical:**
- DuckDB 1.1 - Embedded analytical database
  - Features: bundled, JSON support, Parquet support
  - Location: Workspace dependency
  - File: `data/iou-modern.duckdb`

**AI/ML:**
- petgraph 0.6 - Graph algorithms for GraphRAG
  - Location: `crates/iou-ai`
  - Usage: Knowledge graph construction and traversal

- regex 1.10 - Pattern matching for NER
  - Location: `crates/iou-ai`
  - Usage: Dutch government entity recognition

**Authentication:**
- jsonwebtoken 9.3 - JWT token handling
  - Location: `crates/iou-api`
  - Usage: API authentication and authorization

**Storage:**
- AWS SDK S3 1.65 - S3-compatible storage client
  - Location: `crates/iou-storage`
  - Features: MinIO, AWS S3, Garage compatibility
  - Environment variables: `STORAGE_*`

**HTTP/Networking:**
- reqwest 0.12 - HTTP client
  - Features: JSON support, async/await
  - Location: All crates

## Configuration

**Environment:**
- dotenvy 0.15 - Environment variable loading
- Environment variables for configuration

**Build:**
- Workspace configuration in `Cargo.toml`
- Feature flags for frontend (web, desktop)
- Release profile with LTO and optimization

## Platform Requirements

**Development:**
- Rust toolchain 1.43+
- WASM target: `wasm32-unknown-unknown`
- Dioxus CLI (optional)

**Production:**
- Any platform supporting Rust binaries
- Web server capable of serving static files
- S3-compatible storage (AWS, MinIO, etc.)

---

*Stack analysis: 2026-03-08*