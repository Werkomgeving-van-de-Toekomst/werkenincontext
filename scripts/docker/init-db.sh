#!/bin/sh
# Init script for PostgreSQL Docker container
# This script runs automatically when the container starts

set -e

echo "[IOU] Initializing IOU-Modern database..."

# Wait for PostgreSQL to be ready
until psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c '\q' 2>/dev/null; do
  echo "[IOU] Waiting for PostgreSQL to be ready..."
  sleep 1
done

echo "[IOU] PostgreSQL is ready. Database initialized."

# Create extensions if they don't exist
psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" <<-'EOSQL'
  -- Enable required extensions
  CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
  CREATE EXTENSION IF NOT EXISTS "pgcrypto";
  CREATE EXTENSION IF NOT EXISTS "pg_trgm";  -- For text search
  CREATE EXTENSION IF NOT EXISTS "btree_gin"; -- For index optimization

  -- Create custom types
  DO $$ BEGIN
    CREATE TYPE category_type AS ENUM (
      'document_type', 'domain', 'theme', 'project',
      'department', 'data_classification', 'retention_period'
    );
  EXCEPTION
    WHEN duplicate_object THEN null;
  END $$;

  DO $$ BEGIN
    CREATE TYPE tag_type AS ENUM (
      'user', 'system', 'auto', 'domain', 'project'
    );
  EXCEPTION
    WHEN duplicate_object THEN null;
  END $$;

  DO $$ BEGIN
    CREATE TYPE lawful_basis AS ENUM (
      'wettelijke_verplichting',
      'openbaarmaking',
      'toestemming',
      'contractuele_verplichting',
      'vitale_belangen',
      'openbaar_taakbelang',
      'gerechtvaardigd_belang'
    );
  EXCEPTION
    WHEN duplicate_object THEN null;
  END $$;

  -- Create indexes for common queries
  CREATE INDEX IF NOT EXISTS idx_information_objects_created_at
    ON information_objects(created_at DESC);
  CREATE INDEX IF NOT EXISTS idx_information_objects_status
    ON information_objects(status);
  CREATE INDEX IF NOT EXISTS idx_tags_name_trgm
    ON tags USING gin(name gin_trgm_ops);
  CREATE INDEX IF NOT EXISTS idx_categories_code
    ON categories(code);
  CREATE INDEX IF NOT EXISTS idx_categories_type
    ON categories(category_type);

  -- Set up row-level security helpers
  CREATE OR REPLACE FUNCTION is_admin(user_id uuid)
  RETURNS boolean AS $$
    SELECT EXISTS (
      SELECT 1 FROM user_roles
      WHERE user_id = is_admin.user_id AND role = 'admin'
    );
  $$ LANGUAGE sql SECURITY DEFINER;

  GRANT EXECUTE ON FUNCTION is_admin TO iou_user;
EOSQL

echo "[IOU] Database initialization complete!"
