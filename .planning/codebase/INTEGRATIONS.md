# External Integrations

**Analysis Date:** 2026-03-08

## APIs & External Services

**LLM Provider:**
- Mistral AI - Content generation and AI assistance
  - SDK: Custom HTTP client implementation
  - Auth: `LLM_API_KEY` environment variable
  - Location: `crates/iou-ai/src/llm.rs`
  - Default model: `mistral-small-latest`

**Government Data:**
- Open Regels (regels.overheid.nl) - Dutch government rules and regulations
  - SDK: Custom SPARQL client
  - Endpoints:
    - Acceptance: `https://api.open-regels.triply.cc/datasets/stevengort/DMN-discovery/services/DMN-discovery/sparql`
    - Production: `https://regels.overheid.nl/lab/sparql`
  - Location: `crates/iou-regels/src/client.rs`
  - Format: SPARQL queries, JSON-LD responses

## Data Storage

**Databases:**
- DuckDB - Embedded analytical database
  - Connection: Local file at `data/iou-modern.duckdb`
  - Client: Native Rust integration
  - Features: SQL, Parquet/CSV/JSON support

**File Storage:**
- AWS S3/MinIO - Document storage
  - Client: AWS SDK S3 (`crates/iou-storage`)
  - Compatible with: AWS S3, MinIO, Garage
  - Environment variables: `STORAGE_*`
  - Location: `crates/iou-storage/src/s3.rs`

**Caching:**
- In-memory only - No external caching service

## Authentication & Identity

**Auth Provider:**
- JWT-based authentication
  - Implementation: `jsonwebtoken` crate
  - Location: `crates/iou-api/src/auth.rs`
  - Secret: `JWT_SECRET` environment variable

**No external identity providers** - All authentication handled internally

## Monitoring & Observability

**Error Tracking:**
- Not implemented - No external error tracking service

**Logs:**
- tracing 0.1 - Structured logging
- Level configurable via `RUST_LOG` environment variable
- Output: Console (structured JSON format)

## CI/CD & Deployment

**Hosting:**
- Not specified - Flexible deployment options

**CI Pipeline:**
- GitHub Actions present (`.github/` directory)
  - Location: `.github/`
  - Purpose: Automated testing and deployment

## Environment Configuration

**Required env vars:**
- `DATABASE_PATH` - DuckDB file location
- `JWT_SECRET` - JWT signing secret
- `LLM_API_KEY` - Mistral AI API key (optional)
- `STORAGE_ACCESS_KEY_ID` - S3 access key
- `STORAGE_SECRET_ACCESS_KEY` - S3 secret key
- `STORAGE_ENDPOINT` - S3 endpoint URL
- `STORAGE_BUCKET` - S3 bucket name

**Secrets location:**
- Environment variables
- `.env` file (not committed)
- Secure storage recommended for production

## Webhooks & Callbacks

**Incoming:**
- None detected - No webhook endpoints implemented

**Outgoing:**
- AWS S3 API calls for document storage
- Mistral AI API calls for content generation
- Open Regels SPARQL queries for rule retrieval

---

*Integration audit: 2026-03-08*