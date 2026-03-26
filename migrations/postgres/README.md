# PostgreSQL Migration Scripts for Supabase

This directory contains the PostgreSQL migration scripts for the hybrid DuckDB + Supabase architecture.

## Setup

### 1. Start Supabase (persistent volume onder projectdir)

Postgres-data staat in **`data/supabase/postgres`** (bind mount in `docker-compose.supabase.yml`).

```bash
# Copy the example environment file
cp .env.supabase.example .env.supabase

# Update with secure values
# Edit .env.supabase and change POSTGRES_PASSWORD and JWT_SECRET

# Start + alle migraties in één keer (aanbevolen)
./scripts/supabase-up-and-migrate.sh

# Of handmatig:
docker compose -f docker-compose.supabase.yml --env-file .env.supabase up -d
docker compose -f docker-compose.supabase.yml --env-file .env.supabase ps
```

### 2. Run Migrations

Als je `./scripts/supabase-up-and-migrate.sh` hebt gedraaid, zijn migraties al uitgevoerd.

#### Handmatig (psql op de host):

```bash
export SUPABASE_DATABASE_URL="postgresql://postgres:YOUR_PASSWORD@localhost:5432/iou_modern"
# 003 vóór 002 (kolommen voor RLS)
for m in 001_create_initial_schema.sql 003_optimization_indexes.sql 002_rls_policies.sql \
         004_rls_optimization.sql 005_outbox_table.sql 006_graphrag_entities.sql; do
  psql "$SUPABASE_DATABASE_URL" -v ON_ERROR_STOP=1 -f "migrations/postgres/$m"
done
```

#### Using sqlx-cli (recommended for production):

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features rustls,postgres

# Run migrations
sqlx database create --database-url $SUPABASE_DATABASE_URL
# Note: You'll need to convert .sql files to sqlx format or use psql for now
```

### 3. Verify Setup

```bash
# Connect to PostgreSQL
psql postgresql://postgres:YOUR_PASSWORD@localhost:5432/iou_modern

# Check tables
\dt

# Check views
\dv

# Verify extensions
SELECT extname FROM pg_extension;
```

## Accessing Supabase Services

- **Studio**: http://localhost:3000
- **PostgREST API**: http://localhost:3001
- **Auth**: http://localhost:9999
- **Realtime**: http://localhost:4000

## Stopping Supabase

```bash
docker-compose -f docker-compose.supabase.yml down

# To remove volumes (WARNING: deletes all data)
docker-compose -f docker-compose.supabase.yml down -v
```

## Running Tests

```bash
# Set environment variables
export SUPABASE_DATABASE_URL="postgresql://postgres:YOUR_PASSWORD@localhost:5432/iou_modern"

# Run schema equivalence tests
cargo test --package iou-api --test schema_equivalence

# Run dual-write tests
cargo test --package iou-api --test dual_write
```
