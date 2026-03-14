# Database Architecture

## Hybrid Approach

IOU-Modern uses a hybrid database architecture optimized for both transactional integrity and analytical performance:

### Supabase (PostgreSQL) - Primary Database

**Purpose:** Transactional operations, user authentication, real-time collaboration

- User authentication and authorization via Supabase Auth
- Document metadata and workflow states
- Real-time subscriptions for collaboration features
- Row-Level Security (RLS) for multi-tenant isolation
- Primary storage for all application data

**Connection:** `postgresql://postgres:postgres@localhost:5432/postgres`

**Key Features:**
- ACID transactions for data integrity
- RLS policies for organization-level isolation
- Full-text search with Dutch language support
- JSONB for flexible metadata storage

### DuckDB - Analytics Database

**Purpose:** Read-only analytics, full-text search, vector similarity

- Materialized views for dashboard queries
- Full-text search with Dutch language configuration
- Vector similarity search for GraphRAG entity relationships
- Data science and BI export capabilities
- Parquet file generation for external analysis

**Location:** Local file database (populated via ETL)

**Key Features:**
- Columnar storage for fast aggregations
- In-memory processing for sub-second analytics
- Parallel query execution
- Import from Parquet/CSV for external datasets

## Data Flow

```
┌─────────────┐
│   API/GUI   │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│   Supabase      │ ◄─── Primary (Read/Write)
│   (PostgreSQL)  │
└────────┬────────┘
         │
         │ ETL (Batch, every 5 min)
         ▼
┌─────────────────┐
│   DuckDB        │ ◄─── Analytics (Read-Only)
│   Analytics     │
└─────────────────┘
```

## ETL Pipeline

The ETL pipeline transfers data from Supabase to DuckDB:

- **Frequency:** Every 5 minutes (configurable)
- **Method:** Batch with transactional outbox
- **Tables Synced:**
  - `information_domains`
  - `information_objects`
  - `documents`
  - `templates`
  - `audit_trail`

**Configuration:** See `.env` file for ETL settings:
```bash
ETL_ENABLED=true
ETL_INTERVAL_SECONDS=300
ETL_BATCH_SIZE=1000
ETL_INCREMENTAL=true
```

## Schema Equivalence

PostgreSQL and DuckDB maintain equivalent schemas for analytics tables:

| PostgreSQL | DuckDB | Purpose |
|------------|--------|---------|
| `information_domains` | `information_domains` | Domain metadata |
| `information_objects` | `information_objects` | Object catalog |
| `documents` | `documents` | Document workflow |
| `templates` | `templates` | Template definitions |

## Migration History

The migration from DuckDB-primary to Supabase-primary occurred in 6 phases:

1. **Assessment** (Phase 0) - Performance baseline measurement
2. **Foundation** (Phase 1) - Supabase deployment and dual-write
3. **Auth & Real-time** (Phase 2) - Supabase Auth and RLS policies
4. **Cutover** (Phase 3) - Primary traffic migration to Supabase
5. **Stabilization** (Phase 4) - Monitoring and optimization
6. **Cleanup** (Phase 5) - Legacy code removal

**Migration Date:** March 2026
**Migration Documentation:** `planning-backend-eval/`

## Operations

### Backup Procedures

**Supabase:**
- Daily automated backups (retention: 30 days)
- Point-in-time recovery available
- Export via `pg_dump` for archival

**DuckDB:**
- Rebuildable from Supabase via ETL
- No separate backup required
- Optional snapshot before major changes

### Monitoring

Key metrics monitored:
- Query latency (p50/p95/p99)
- ETL cycle duration and error rate
- Replication lag (WAL size for CDC)
- Real-time subscription count
- Database connection pool utilization

See `docs/operations/stabilization_runbook.md` for detailed procedures.

### Troubleshooting

**Issue:** Analytics show stale data
- Check ETL service status
- Verify last successful checkpoint
- Manual trigger: `POST /admin/etl/trigger`

**Issue:** Real-time updates not working
- Verify Supabase Realtime connection
- Check RLS policy for subscription channel
- Review client-side subscription code

## Development

### Local Development Setup

```bash
# Start Supabase
docker-compose -f docker-compose.supabase.yml up -d

# Set environment
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/postgres"

# Run API
cargo run
```

### Running Migrations

```bash
# PostgreSQL migrations
psql $DATABASE_URL -f migrations/postgres/001_create_initial_schema.sql
psql $DATABASE_URL -f migrations/postgres/002_rls_policies.sql
psql $DATABASE_URL -f migrations/postgres/003_optimization_indexes.sql
psql $DATABASE_URL -f migrations/postgres/004_rls_optimization.sql
psql $DATABASE_URL -f migrations/postgres/005_outbox_table.sql
```

### Testing

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration

# Run cleanup verification tests
cargo test -p iou-api cleanup
```

---

**Last Updated:** 2026-03-14
**Version:** 1.0
