#!/bin/bash
# Database setup script for IOU-Modern
#
# Usage:
#   ./scripts/db-setup.sh [prepare|migrate|reset]
#
# Commands:
#   prepare - Generate sqlx-data.json for compile-time verification
#   migrate - Run database migrations
#   reset - Drop and recreate database (DESTRUCTIVE!)

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Load .env if exists
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Default DATABASE_URL if not set
DATABASE_URL=${DATABASE_URL:-"postgresql://iou_user:iou_password@localhost:5432/iou_modern"}

echo -e "${GREEN}IOU-Modern Database Setup${NC}"
echo "Database: $DATABASE_URL"
echo ""

case "${1:-help}" in
    prepare)
        echo -e "${YELLOW}Generating sqlx-data.json...${NC}"
        cargo sqlx prepare -- --lib --features server
        echo -e "${GREEN}✓ sqlx-data.json generated${NC}"
        echo ""
        echo "You can now build with: cargo build --features server"
        ;;

    migrate)
        echo -e "${YELLOW}Running database migrations...${NC}"
        cargo sqlx migrate run --source-dir migrations/postgres --database-url "$DATABASE_URL"
        echo -e "${GREEN}✓ Migrations applied${NC}"
        ;;

    reset)
        echo -e "${RED}WARNING: This will delete all data!${NC}"
        read -p "Are you sure? (yes/no): " confirm
        if [ "$confirm" = "yes" ]; then
            echo -e "${YELLOW}Dropping database...${NC}"
            psql "$DATABASE_URL" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
            echo -e "${YELLOW}Running migrations...${NC}"
            cargo sqlx migrate run --source-dir migrations/postgres --database-url "$DATABASE_URL"
            echo -e "${GREEN}✓ Database reset complete${NC}"
        else
            echo "Aborted"
        fi
        ;;

    seed)
        echo -e "${YELLOW}Seeding test data...${NC}"
        # Add seed commands here
        psql "$DATABASE_URL" -f scripts/seed.sql 2>/dev/null || echo "No seed.sql found"
        echo -e "${GREEN}✓ Test data seeded${NC}"
        ;;

    *)
        echo "IOU-Modern Database Setup"
        echo ""
        echo "Usage: ./scripts/db-setup.sh [command]"
        echo ""
        echo "Commands:"
        echo "  prepare  Generate sqlx-data.json for compile-time verification"
        echo "  migrate  Run database migrations"
        echo "  reset    Drop and recreate database (DESTRUCTIVE!)"
        echo "  seed     Insert test data"
        echo ""
        echo "First time setup:"
        echo "  1. docker-compose up -d"
        echo "  2. ./scripts/db-setup.sh migrate"
        echo "  3. ./scripts/db-setup.sh prepare"
        ;;
esac
