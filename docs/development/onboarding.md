# Developer Onboarding Guide

**Last Updated:** 2026-03-14

---

## Quick Start

### Prerequisites

- Rust 1.80+ (Edition 2024)
- Docker & Docker Compose
- PostgreSQL client tools

### 1. Clone and Setup

```bash
# Clone repository
git clone https://github.com/terminal-woo/iou-modern.git
cd iou-modern

# Copy environment files
cp .env.example .env
cp .env.supabase.example .env.supabase

# Start Supabase
docker-compose -f docker-compose.supabase.yml up -d
```

### 2. Database Setup

**Supabase (Primary):**
```bash
# Export connection string
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/postgres"

# Run migrations
psql $DATABASE_URL -f migrations/postgres/001_create_initial_schema.sql
psql $DATABASE_URL -f migrations/postgres/002_rls_policies.sql
psql $DATABASE_URL -f migrations/postgres/003_optimization_indexes.sql
psql $DATABASE_URL -f migrations/postgres/004_rls_optimization.sql
psql $DATABASE_URL -f migrations/postgres/005_outbox_table.sql
```

**DuckDB (Analytics):**
```bash
# DuckDB file is created automatically by ETL
# No manual setup required
# File location: data/analytics.duckdb (created on first ETL run)
```

### 3. Run the Application

```bash
# Build
cargo build

# Run API server
cargo run --bin iou-api

# Run tests
cargo test

# Run with ETL enabled
ETL_ENABLED=true cargo run
```

---

## Architecture Overview

### Database Architecture

IOU-Modern uses a **hybrid database approach**:

```
┌──────────────┐
│  Application │
└───────┬──────┘
        │
        ├──────────────┬───────────────┐
        ▼              ▼               ▼
   ┌─────────┐   ┌─────────┐    ┌──────────┐
   │Supabase │   │Realtime │    │  DuckDB  │
   │(Primary)│   │(Supabase│    │(Analytics)│
   │         │   │   Auth) │    │          │
   └─────────┘   └─────────┘    └──────────┘
```

- **Supabase (PostgreSQL):** Primary transactional database
- **Supabase Realtime:** WebSocket connections for collaboration
- **DuckDB:** Analytics and full-text search

### Crate Structure

```
iou-modern/
├── crates/
│   ├── iou-api/          # Main API server
│   ├── iou-core/         # Domain models
│   ├── iou-storage/      # DuckDB metadata operations
│   ├── iou-frontend/     # Frontend components
│   ├── iou-ai/           # AI/ML features
│   ├── iou-regels/       # Rules engine
│   └── iou-orchestrator/ # Workflow orchestration
├── migrations/           # Database migrations
│   └── postgres/         # PostgreSQL migrations
└── docs/                 # Documentation
```

---

## Common Development Tasks

### Querying Analytics (DuckDB)

```bash
# Use DuckDB CLI
duckdb data/analytics.duckdb

# Example: Get compliance overview
SELECT * FROM v_compliance_overview;

# Example: Full-text search
SELECT * FROM information_objects
WHERE search_text LIKE '%keyword%';
```

### Checking RLS Policies

```bash
# Via psql
psql $DATABASE_URL

# View all policies
SELECT polname, polcmd, polrelid::regclass
FROM pg_policy
JOIN pg_class ON pg_class.oid = polrelid;

# Test a policy as a specific user
SET ROLE user_id;
SELECT * FROM documents LIMIT 1;
```

### Running ETL Manually

```bash
# Trigger immediate ETL cycle
curl -X POST http://localhost:8080/admin/etl/run \
  -H "Authorization: Bearer $ADMIN_TOKEN"

# Check ETL status
curl http://localhost:8080/admin/etl/stats \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

### Debugging Real-time Issues

```bash
# Check active subscriptions
psql $DATABASE_URL

SELECT * FROM realtime.subscription WHERE status = 'active';

# View channel topics
SELECT * FROM realtime.topic;

# Check presence
SELECT * FROM presence.tracker;
```

---

## Key Concepts

### Row-Level Security (RLS)

All tables use RLS for organization isolation:
- Users see only their organization's data
- Classification-based filtering applies
- Public documents accessible via Woo publication

**Adding RLS to a new table:**
```sql
ALTER TABLE new_table ENABLE ROW LEVEL SECURITY;

CREATE POLICY new_table_org_policy ON new_table
FOR ALL
TO authenticated
USING (organization_id = auth.organization_id());
```

### ETL Pipeline

Data flows from Supabase → DuckDB via ETL:
1. Application writes to Supabase
2. Outbox event created (in same transaction)
3. ETL processor picks up events
4. Data written to DuckDB
5. Event marked as processed

**Configuration:** `.env` file
```bash
ETL_ENABLED=true
ETL_INTERVAL_SECONDS=300
ETL_BATCH_SIZE=1000
```

### Real-time Subscriptions

Supabase Realtime handles WebSocket connections:
- Channel: `documents:{document_id}`
- Events: `INSERT`, `UPDATE`, `DELETE`
- Presence tracking for collaboration

---

## Troubleshooting

### Issue: Database connection errors

```bash
# Check Supabase is running
docker-compose ps supabase

# Restart if needed
docker-compose restart supabase
```

### Issue: ETL not running

```bash
# Check environment
echo $ETL_ENABLED

# Verify outbox table exists
psql $DATABASE_URL -c "\d change_outbox"

# Check logs
journalctl -u iou-api -f | grep etl
```

### Issue: Tests failing

```bash
# Ensure Supabase is running
docker-compose -f docker-compose.supabase.yml up -d

# Run with logs
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

---

## Resources

### Documentation

- [Database Architecture](../architecture/database.md)
- [ETL Troubleshooting](../operations/runbooks/etl_troubleshooting.md)
- [Stabilization Runbook](../operations/stabilization_runbook.md)
- [Migration Plan](../../planning-backend-eval/spec.md)

### Internal Tools

- Supabase Dashboard: `https://supabase.com/project/xxx`
- Grafana Dashboard: `http://localhost:3000` (if configured)
- API Documentation: `http://localhost:8080/docs`

### Getting Help

- #backend-development on Slack
- Create GitHub issue for bugs
- Contact backend lead for architecture questions

---

## Next Steps

1. Complete the [Training Checklist](#training-checklist)
2. Set up your local development environment
3. Make your first commit following the [Contribution Guidelines](../CONTRIBUTING.md)
4. Attend the weekly backend sync meeting

---

## Training Checklist

Complete these items to be fully onboarded:

- [ ] Development environment running locally
- [ ] Successfully run all tests
- [ ] Navigate Supabase dashboard independently
- [ ] Understand RLS policy structure
- [ ] Debug a real-time subscription issue
- [ ] Manually trigger ETL and verify results
- [ ] Write an analytics query in DuckDB
- [ ] Create and apply a database migration
- [ ] Debug a failing test
- [ ] Deploy to staging environment

**Mentor:** ___________________
**Completion Date:** ___________________
