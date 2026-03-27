# Research Findings: DuckDB vs Convex vs Supabase for IOU-Modern

## Table of Contents
1. [Codebase Analysis](#codebase-analysis)
2. [Database Comparison 2026](#database-comparison-2026)
3. [Convex Deep Dive](#convex-deep-dive)
4. [Supabase Features](#supabase-features)
5. [Dutch Government Compliance](#dutch-government-compliance)
6. [Summary Recommendations](#summary-recommendations)

---

## Codebase Analysis

### Current DuckDB Schema & Data Organization

**Core Tables:**
- `information_domains` - Organizational context (Zaak, Project, Beleid, Expertise)
- `information_objects` - Main document/content with full-text search
- `documents` - Document metadata with workflow states
- `templates` - Document templates
- `audit_trail` - Full audit trail

**Key Views:**
- `v_searchable_objects` - Aggregates searchable text
- `v_compliance_overview` - Compliance analytics
- `v_domain_statistics` - Domain distribution
- `v_entity_network` - GraphRAG relationships

### Query Patterns

- **Thread-safe wrapper**: `Arc<Mutex<Connection>>` for concurrent access
- **Parameterized queries**: SQL injection prevention
- **Async wrappers**: `tokio::task::spawn_blocking` for compatibility

### Current Limitations

1. **Full-text Search**: Using basic `ILIKE` instead of DuckDB's FTS extension
2. **Array Handling**: Tags stored as `VARCHAR[]` but not properly parsed
3. **Vector Search**: Embeddings stored but no similarity search
4. **S3 Integration**: References only, no direct DuckDB-to-S3 queries

### Architecture Strengths

- Columnar storage optimized for analytics
- Single-file deployment
- JSON fields for flexible metadata
- Excellent for analytical queries

---

## Database Comparison 2026

| Feature | DuckDB | Convex | Supabase |
|---------|--------|--------|----------|
| **Type** | In-process OLAP | Reactive backend | PostgreSQL-based BaaS |
| **Query Language** | SQL | TypeScript functions | SQL + PostgreSQL |
| **Real-time** | No | Yes (built-in) | Yes (via PostgreSQL CDC) |
| **Self-hosting** | N/A (embedded) | Yes (since 2025) | Yes (Docker) |
| **Latency** | N/A | Sub-50ms | 100-200ms p99 |
| **License** | MIT | FSL 1.1 → Apache 2.0 | MIT |

### Performance Characteristics

**DuckDB:**
- Excels at analytical workloads
- Single-writer limitation
- Best with SSD/NVMe storage
- 15TB+ tested limits

**Convex:**
- Sub-50ms latency at 5,000 concurrent connections
- Automatic caching
- Reactive programming model

**Supabase:**
- PostgreSQL performance for CRUD
- Real-time via logical replication
- Traditional scaling considerations

---

## Convex Deep Dive

### Real-time Capabilities
- Reactive queries with automatic subscriptions
- Sub-50ms sync latency
- End-to-end TypeScript

### Authentication
- Convex Auth (beta): 80+ OAuth providers
- Clerk, Auth0 integration
- Custom OpenID Connect

### Self-Hosting (2025+)
- Docker-based deployment
- PostgreSQL, SQLite storage
- SQL migrations between versions
- Three services required

### Vendor Lock-in Risks
- **FSL 1.1-Apache 2.0 license**: Converts to Apache after 2 years
- **NOT OSI-compliant FOSS**
- TypeScript functions tightly coupled to Convex
- Migration requires code rewrite

### Pricing
- **Free**: Up to 6 team members
- **Professional**: $25/member/month + usage fees

---

## Supabase Features

### Real-time Subscriptions
- PostgreSQL logical decoding (CDC)
- Elixir-based Realtime server
- WAL-based change capture
- At least 2 nodes per region for HA

### Row-Level Security (RLS)
- PostgreSQL-native
- Simplifies application logic
- Column-level security available
- Bypass RLS for views by default

### Auth Providers
- Email/password, magic links, OTP
- Social login (OAuth)
- SSO, OAuth 2.1 server
- Phone authentication

### Self-Hosting
- Docker Compose (recommended)
- Full stack: Postgres, Realtime, Auth, Storage, Studio
- Production deployments: Azure, Hetzner, Fly.io

### Security & Compliance
- **SOC 2 Type 2 certified**
- **HIPAA compliant**
- BAA available

---

## Dutch Government Compliance

### GDPR (AVG in Netherlands)
- UAVG implements EU GDPR
- Dutch DPA (Autoriteit Persoonsgegevens) oversight
- Data flow mapping required

### NORA (Nederlandse Overheid Referentie Architectuur)
- National interoperability framework
- 'Bouwstenen' (building blocks) approach
- Base registries as authoritative sources
- Standard for Dutch government ICT

### Archiefwet (Archives Act)
- Transfer after 10 years (reduced from 20)
- Must be 'findable, readable, usable'
- Selectielijsten required for retention
- Timely destruction when period expires

### Woo (Wet open overheid)
- Replaces WOB
- Active publication obligations
- API requirements for data access

### Database Compliance Matrix

| Requirement | Supabase | Convex | DuckDB |
|-------------|----------|--------|--------|
| **Data Portability** | PostgreSQL standard | ZIP export/import | File-based |
| **GDPR Compliance** | SOC 2/HIPAA | Self-hosting | Full control |
| **Audit Trails** | PostgreSQL WAL | Available | App-level |
| **Data Retention** | PostgreSQL features | Manual | Manual |
| **Archiving** | Standard tools | Export ZIP | Direct file |
| **NORA Alignment** | Standard SQL | FSL license | Fully open |

---

## Summary Recommendations

### For Government Projects (NL)

**Primary Recommendation: Supabase**
- PostgreSQL-based with standard SQL (NORA alignment)
- SOC 2 Type 2 and HIPAA compliant
- Row-Level Security for granular access
- Self-hosting with Docker for data sovereignty
- Standard PostgreSQL tools for archiving (Archiefwet)
- Strong authentication options

**Alternative: DuckDB (for analytics)**
- Suitable for analytical reporting
- Can be embedded alongside transactional DB
- Excellent for aggregated reporting
- Direct file access supports archival

**Caution with Convex:**
- FSL license (not truly FOSS for 2 years)
- Non-standard query model
- Better for real-time collaborative apps
- Consider only if real-time is primary requirement

### Decision Matrix

| Use Case | Recommended | Rationale |
|----------|-------------|-----------|
| Analytics/BI | DuckDB | Columnar storage optimized |
| Real-time collaboration | Convex | Sub-50ms reactive |
| Government compliance | Supabase | SOC 2, SQL standard |
| Self-hosting required | Supabase | Standard PostgreSQL |
| TypeScript-first | Convex | End-to-end TS |
| SQL-heavy | Supabase | Native PostgreSQL |

---

## Testing Approach

### Existing Rust Testing Framework

**Project: IOU-Modern (Rust + Axum)**

**Test Framework:** Rust built-in (`cargo test`)

**Test Organization:**
- Unit tests: Inline in modules (`#[cfg(test)]`)
- Integration tests: `tests/` directory
- Doc tests: In documentation examples

**Testing Patterns Found:**
```rust
// Unit test pattern
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example() {
        // Test implementation
    }
}
```

**Key Testing Dependencies:**
- `tokio::test` for async tests
- `tower` for testing Axum handlers
- Mock patterns for database interactions

**Coverage:** No coverage tool currently configured

**Testing Requirements for Migration:**

Since this is a database migration project, testing will focus on:
1. **Schema equivalence tests** - Ensure PostgreSQL schema matches DuckDB
2. **Data migration tests** - Verify ETL correctness
3. **Dual-write consistency** - Compare write results between databases
4. **Performance regression tests** - Benchmark before/after
5. **RLS policy tests** - Verify security policies
6. **Real-time subscription tests** - Validate WebSocket/Realtime behavior

**Recommended Test Structure for Migration:**
```
tests/
├── migration/
│   ├── schema_tests.rs      # Schema equivalence
│   ├── etl_tests.rs         # Data migration validation
│   ├── dual_write_tests.rs  # Consistency checks
│   ├── rls_tests.rs         # Security policy tests
│   └── realtime_tests.rs    # Real-time validation
└── benchmarks/
    └── performance.rs       # Regression tests
```

---

## Sources

### DuckDB
- [Performance Guide](https://duckdb.org/docs/stable/guides/performance/overview)
- [Concurrency](https://duckdb.org/docs/stable/connect/concurrency)
- [Limits](https://duckdb.org/docs/stable/operations_manual/limits)

### Convex
- [Convex vs Supabase 2026](https://bertomill.medium.com/convex-vs-supabase-which-backend-should-you-choose-in-2026-50d228c517de)
- [Self-Hosting](https://docs.convex.dev/self-hosting)
- [Convex Auth](https://docs.convex.dev/auth/convex-auth)
- [FSL License](https://fsl.software/)
- [Pricing](https://www.convex.dev/pricing)

### Supabase
- [Self-Hosting Docker](https://supabase.com/docs/guides/self-hosting/docker)
- [Row Level Security](https://supabase.com/docs/guides/database/postgres/row-level-security)
- [Realtime Architecture](https://supabase.com/docs/guides/realtime/architecture)
- [SOC 2 Compliance](https://supabase.com/docs/guides/security/soc-2-compliance)

### Dutch Government
- [NORA](https://www.digitaleoverheid.nl/overzicht-van-alle-onderwerpen/nora/)
- [Archiefwet](https://www.rijksoverheid.nl/onderwerpen/archieven/archieven-van-de-overheid)
- [Bewaartermijnen](https://www.inspectie-oe.nl/onderwerpen/selectie-en-vernietiging/vraag-en-antwoord/bewaartermijnen-volgens-de-archiefwet)
- [GDPR Netherlands](https://business.gov.nl/running-your-business/legal-matters/how-to-make-your-business-gdpr-compliant/)
