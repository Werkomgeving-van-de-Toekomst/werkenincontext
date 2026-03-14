# PostgreSQL Migration Scripts for Supabase

This directory contains the PostgreSQL migration scripts for the hybrid DuckDB + Supabase architecture.

## Setup

### 1. Start Supabase

```bash
# Copy the example environment file
cp .env.supabase.example .env.supabase

# Update with secure values
# Edit .env.supabase and change POSTGRES_PASSWORD and JWT_SECRET

# Start Supabase
docker-compose -f docker-compose.supabase.yml up -d

# Check status
docker-compose -f docker-compose.supabase.yml ps
```

### 2. Run Migrations

#### Using psql directly:

```bash
# Set database URL
export SUPABASE_DATABASE_URL="postgresql://postgres:YOUR_PASSWORD@localhost:5432/iou_modern"

# Run migration
psql $SUPABASE_DATABASE_URL -f migrations/postgres/001_create_initial_schema.sql
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
