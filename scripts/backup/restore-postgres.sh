#!/bin/bash
# PostgreSQL Restore Script for IOU-Modern
#
# This script restores a PostgreSQL database from a backup file.
# WARNING: This will OVERWRITE the existing database!
#
# Usage: ./restore-postgres.sh BACKUP_FILE [--confirm] [--to-db DATABASE_NAME]
#
# Environment variables:
#   - POSTGRES_HOST: PostgreSQL host (default: localhost)
#   - POSTGRES_PORT: PostgreSQL port (default: 5432)
#   - POSTGRES_USER: Database user (default: postgres)
#   - POSTGRES_PASSWORD: Database password

set -euo pipefail

# Configuration
POSTGRES_HOST="${POSTGRES_HOST:-localhost}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
POSTGRES_USER="${POSTGRES_USER:-postgres}"
POSTGRES_DB="${POSTGRES_DB:-postgres}"
TARGET_DB="${POSTGRES_DB}"
CONFIRMED=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
log_info() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] INFO:${NC} $1"
}

log_error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1" >&2
}

log_warning() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1"
}

show_usage() {
    cat << EOF
Usage: $0 BACKUP_FILE [--confirm] [--to-db DATABASE_NAME]

Arguments:
  BACKUP_FILE        Path to the backup file (.sql.gz)

Options:
  --confirm          Skip confirmation prompt
  --to-db NAME       Restore to a different database (creates new DB)
  --help             Show this help message

Examples:
  $0 /data/backups/postgres/iou_modern_20240101_020000.sql.gz
  $0 /data/backups/postgres/iou_modern_20240101_020000.sql.gz --confirm
  $0 /data/backups/postgres/iou_modern_20240101_020000.sql.gz --to-db iou_restore_test

Environment variables:
  POSTGRES_HOST      PostgreSQL host (default: localhost)
  POSTGRES_PORT      PostgreSQL port (default: 5432)
  POSTGRES_USER      Database user (default: postgres)
  POSTGRES_PASSWORD  Database password
  POSTGRES_DB        Target database (default: postgres)

EOF
}

verify_backup_file() {
    local backup_file="$1"

    if [[ ! -f "${backup_file}" ]]; then
        log_error "Backup file not found: ${backup_file}"
        return 1
    fi

    # Check file extension
    if [[ ! "${backup_file}" =~ \.sql\.gz$ ]]; then
        log_warning "Backup file doesn't have .sql.gz extension"
    fi

    # Test gzip integrity
    log_info "Verifying backup integrity..."
    if ! gzip -t "${backup_file}" 2>/dev/null; then
        log_error "Backup file is corrupted or not a valid gzip file"
        return 1
    fi

    # Get file info
    local file_size
    file_size=$(du -h "${backup_file}" | cut -f1)
    log_info "Backup file: $(basename "${backup_file}") (${file_size})"

    return 0
}

confirm_restore() {
    local backup_file="$1"
    local target_db="$2"

    echo ""
    echo "=========================================="
    echo "       RESTORE CONFIRMATION"
    echo "=========================================="
    echo ""
    echo "This will RESTORE the database from backup!"
    echo ""
    echo "  Backup file:  $(basename "${backup_file}")"
    echo "  Target DB:    ${target_db}"
    echo "  Host:         ${POSTGRES_HOST}:${POSTGRES_PORT}"
    echo ""
    if [[ "${target_db}" == "${POSTGRES_DB}" ]]; then
        echo -e "${RED}WARNING: This will OVERWRITE the existing database!${NC}"
    else
        echo -e "${YELLOW}A new database will be created: ${target_db}${NC}"
    fi
    echo ""
    echo "=========================================="
    echo ""

    read -p "Type 'YES' to confirm restore: " confirmation

    if [[ "${confirmation}" != "YES" ]]; then
        log_info "Restore cancelled by user"
        exit 0
    fi
}

create_database() {
    local db_name="$1"

    log_info "Creating database: ${db_name}"

    export PGPASSWORD="${POSTGRES_PASSWORD:-}"

    if psql -h "${POSTGRES_HOST}" \
           -p "${POSTGRES_PORT}" \
           -U "${POSTGRES_USER}" \
           -d postgres \
           -c "CREATE DATABASE ${db_name};" 2>/dev/null; then
        log_info "Database created successfully"
    else
        log_error "Failed to create database (may already exist)"
    fi

    unset PGPASSWORD
}

drop_database() {
    local db_name="$1"

    log_info "Dropping database: ${db_name}"

    export PGPASSWORD="${POSTGRES_PASSWORD:-}"

    # Close all connections first
    psql -h "${POSTGRES_HOST}" \
         -p "${POSTGRES_PORT}" \
         -U "${POSTGRES_USER}" \
         -d postgres \
         -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '${db_name}';" >/dev/null 2>&1 || true

    # Drop database
    if psql -h "${POSTGRES_HOST}" \
           -p "${POSTGRES_PORT}" \
           -U "${POSTGRES_USER}" \
           -d postgres \
           -c "DROP DATABASE IF EXISTS ${db_name};" 2>/dev/null; then
        log_info "Database dropped successfully"
    else
        log_warning "Failed to drop database"
    fi

    unset PGPASSWORD
}

restore_backup() {
    local backup_file="$1"
    local target_db="$2"

    log_info "Starting restore..."
    log_info "This may take several minutes for large databases..."

    export PGPASSWORD="${POSTGRES_PASSWORD:-}"

    local start_time
    start_time=$(date +%s)

    # Restore the backup
    if gunzip -c "${backup_file}" | \
       psql -h "${POSTGRES_HOST}" \
            -p "${POSTGRES_PORT}" \
            -U "${POSTGRES_USER}" \
            -d "${target_db}"; then
        local end_time
        end_time=$(date +%s)
        local duration=$((end_time - start_time))

        log_info "Restore completed in ${duration} seconds"

        # Get table count
        local table_count
        table_count=$(psql -h "${POSTGRES_HOST}" \
                        -p "${POSTGRES_PORT}" \
                        -U "${POSTGRES_USER}" \
                        -d "${target_db}" \
                        -tAc \
                        "SELECT COUNT(*) FROM information_tables WHERE table_schema = 'public';" 2>/dev/null || echo "?")

        log_info "Restored ${table_count} tables"

    else
        log_error "Restore failed!"
        unset PGPASSWORD
        return 1
    fi

    unset PGPASSWORD
    return 0
}

verify_restore() {
    local target_db="$1"

    log_info "Verifying restore..."

    export PGPASSWORD="${POSTGRES_PASSWORD:-}"

    # Check some key tables
    local tables=(
        "information_objects"
        "information_domains"
    )

    local all_ok=true
    for table in "${tables[@]}"; do
        local count
        count=$(psql -h "${POSTGRES_HOST}" \
                    -p "${POSTGRES_PORT}" \
                    -U "${POSTGRES_USER}" \
                    -d "${target_db}" \
                    -tAc \
                    "SELECT COUNT(*) FROM ${table};" 2>/dev/null || echo "ERROR")

        if [[ "${count}" == "ERROR" ]]; then
            log_warning "Table ${table} not found"
        else
            log_info "Table ${table}: ${count} rows"
        fi
    done

    unset PGPASSWORD
}

# Parse arguments
BACKUP_FILE=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --confirm)
            CONFIRMED=true
            shift
            ;;
        --to-db)
            TARGET_DB="$2"
            shift 2
            ;;
        --help|-h)
            show_usage
            exit 0
            ;;
        -*)
            log_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
        *)
            BACKUP_FILE="$1"
            shift
            ;;
    esac
done

# Validate arguments
if [[ -z "${BACKUP_FILE}" ]]; then
    log_error "No backup file specified"
    show_usage
    exit 1
fi

# Verify backup file
if ! verify_backup_file "${BACKUP_FILE}"; then
    exit 1
fi

# Confirm restore
if [[ "${CONFIRMED}" != true ]]; then
    confirm_restore "${BACKUP_FILE}" "${TARGET_DB}"
fi

# Main execution
log_info "=========================================="
log_info "PostgreSQL Restore Script"
log_info "=========================================="

# If restoring to a different database, create it first
if [[ "${TARGET_DB}" != "${POSTGRES_DB}" ]]; then
    create_database "${TARGET_DB}"
else
    # Restoring to existing database - drop and recreate
    log_warning "Target database will be dropped and recreated"
    if [[ "${CONFIRMED}" != true ]]; then
        read -p "Continue? (yes/no): " confirm_drop
        if [[ "${confirm_drop}" != "yes" ]]; then
            log_info "Restore cancelled"
            exit 0
        fi
    fi
    drop_database "${TARGET_DB}"
    create_database "${TARGET_DB}"
fi

# Perform restore
if restore_backup "${BACKUP_FILE}" "${TARGET_DB}"; then
    verify_restore "${TARGET_DB}"
    log_info "=========================================="
    log_success "Restore completed successfully!"
    log_info "=========================================="
else
    log_error "=========================================="
    log_error "Restore failed!"
    log_error "=========================================="
    exit 1
fi
