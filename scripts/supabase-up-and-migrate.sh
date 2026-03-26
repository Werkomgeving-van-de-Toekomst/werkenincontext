#!/usr/bin/env bash
# Start Supabase (Docker) met persistent volume onder ./data/supabase/postgres
# en voer alle migrations/postgres/*.sql migrations uit (volgorde vast).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

COMPOSE_FILE="docker-compose.supabase.yml"
ENV_FILE=".env.supabase"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Kopieer eerst .env.supabase.example naar .env.supabase en pas wachtwoorden aan."
  exit 1
fi

mkdir -p data/supabase/postgres

echo "==> Docker compose up ($COMPOSE_FILE)"
docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" up -d

echo "==> Wachten op Postgres..."
for i in $(seq 1 60); do
  if docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" exec -T db pg_isready -U postgres -d iou_modern >/dev/null 2>&1; then
    echo "Postgres is bereikbaar."
    break
  fi
  if [[ "$i" -eq 60 ]]; then
    echo "Timeout: Postgres werd niet healthy."
    exit 1
  fi
  sleep 2
done

# 003 vóór 002: RLS verwijst naar documents.organization_id e.d. (wordt in 003 toegevoegd)
MIGRATIONS=(
  "001_create_initial_schema.sql"
  "003_optimization_indexes.sql"
  "002_rls_policies.sql"
  "004_rls_optimization.sql"
  "005_outbox_table.sql"
  "006_graphrag_entities.sql"
)

for m in "${MIGRATIONS[@]}"; do
  f="migrations/postgres/$m"
  echo "==> Migratie: $f"
  docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" exec -T db \
    psql -U postgres -d iou_modern -v ON_ERROR_STOP=1 < "$f"
done

echo "==> Klaar. Postgres: localhost:5432 db=iou_modern user=postgres"
echo "    Studio: http://localhost:3000  PostgREST: http://localhost:3001"
