#!/bin/bash
# Backup Testing Script for IOU-Modern
#
# This script tests backup integrity by:
# 1. Verifying backup file integrity (gzip checksum)
# 2. Restoring to a test database
# 3. Running basic queries to verify data
# 4. Cleaning up the test database
#
# Usage: ./test-backups.sh [--keep-test-db] [--backup-file PATH]
#
# Environment variables:
#   - POSTGRES_HOST: PostgreSQL host (default: localhost)
#   - POSTGRES_PORT: PostgreSQL port (default: 5432)
#   - POSTGRES_USER: Database user (default: postgres)
#   - POSTGRES_PASSWORD: Database password
#   - BACKUP_DIR: Local backup directory (default: /data/backups/postgres)

set -euo pipefail

# Configuration
POSTGRES_HOST="${POSTGRES_HOST:-localhost}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
POSTGRES_USER="${POSTGRES_USER:-postgres}"
BACKUP_DIR="${BACKUP_DIR:-/data/backups/postgres}"
TEST_DB_NAME="test_restore_$(date +%s)"
KEEP_TEST_DB=false
SPECIFIC_BACKUP=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --keep-test-db)
            KEEP_TEST_DB=true
            shift
            ;;
        --backup-file)
            SPECIFIC_BACKUP="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--keep-test-db] [--backup-file PATH]"
            exit 1
            ;;
    esac
done

# Functions
log_info() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] INFO: $1"
}

log_error() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1" >&2
}

log_success() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] SUCCESS: $1"
}

find_latest_backup() {
    log_info "Finding latest backup..."

    if [[ -n "${SPECIFIC_BACKUP}" ]]; then
        if [[ -f "${SPECIFIC_BACKUP}" ]]; then
            echo "${SPECIFIC_BACKUP}"
            return 0
        else
            log_error "Specified backup file not found: ${SPECIFIC_BACKUP}"
            return 1
        fi
    fi

    local latest
    latest=$(find "${BACKUP_DIR}" -name "iou_modern_*.sql.gz" -type f -printf '%T@ %p\n' | sort -n | tail -1 | cut -d' ' -f2-)

    if [[ -z "${latest}" ]]; then
        log_error "No backup files found in ${BACKUP_DIR}"
        return 1
    fi

    log_info "Latest backup: $(basename "${latest}")"
    echo "${latest}"
}

verify_backup_integrity() {
    local backup_file="$1"

    log_info "Verifying backup integrity: $(basename "${backup_file}")"

    # Check if file exists
    if [[ ! -f "${backup_file}" ]]; then
        log_error "Backup file not found: ${backup_file}"
        return 1
    fi

    # Test gzip integrity
    if ! gzip -t "${backup_file}" 2>/dev/null; then
        log_error "Gzip integrity check failed"
        return 1
    fi
    log_success "Gzip integrity check passed"

    # Verify checksum if exists
    if [[ -f "${backup_file}.sha256" ]]; then
        if sha256sum -c "${backup_file}.sha256" >/dev/null 2>&1; then
            log_success "Checksum verification passed"
        else
            log_error "Checksum verification failed"
            return 1
        fi
    else
        log_info "No checksum file found, skipping checksum verification"
    fi

    # Get file size
    local file_size
    file_size=$(du -h "${backup_file}" | cut -f1)
    log_info "Backup size: ${file_size}"

    return 0
}

create_test_database() {
    log_info "Creating test database: ${TEST_DB_NAME}"

    export PGPASSWORD="${POSTGRES_PASSWORD:-}"

    if psql -h "${POSTGRES_HOST}" \
           -p "${POSTGRES_PORT}" \
           -U "${POSTGRES_USER}" \
           -d postgres \
           -c "CREATE DATABASE ${TEST_DB_NAME};" 2>/dev/null; then
        log_success "Test database created"
    else
        log_error "Failed to create test database"
        return 1
    fi

    unset PGPASSWORD
}

restore_backup() {
    local backup_file="$1"

    log_info "Restoring backup to test database..."

    export PGPASSWORD="${POSTGRES_PASSWORD:-}"

    # Restore the backup
    if gunzip -c "${backup_file}" | \
       psql -h "${POSTGRES_HOST}" \
            -p "${POSTGRES_PORT}" \
            -U "${POSTGRES_USER}" \
            -d "${TEST_DB_NAME}" \
            -q >/dev/null 2>&1; then
        log_success "Backup restored successfully"
    else
        log_error "Backup restoration failed"
        unset PGPASSWORD
        return 1
    fi

    unset PGPASSWORD
}

verify_restored_data() {
    log_info "Verifying restored data..."

    export PGPASSWORD="${POSTGRES_PASSWORD:-}"

    local errors=0

    # Check if key tables exist and have data
    local tables=(
        "information_objects"
        "information_domains"
        "documents"
        "templates"
        "subject_access_requests"
        "woo_publication_requests"
    )

    for table in "${tables[@]}"; do
        local count
        count=$(psql -h "${POSTGRES_HOST}" \
                    -p "${POSTGRES_PORT}" \
                    -U "${POSTGRES_USER}" \
                    -d "${TEST_DB_NAME}" \
                    -tAc \
                    "SELECT COUNT(*) FROM ${table};" 2>/dev/null || echo "0")

        if [[ "${count}" == "0" ]]; then
            log_info "Table ${table}: empty or doesn't exist"
        else
            log_success "Table ${table}: ${count} rows"
        fi
    done

    # Check total table count
    local table_count
    table_count=$(psql -h "${POSTGRES_HOST}" \
                    -p "${POSTGRES_PORT}" \
                    -U "${POSTGRES_USER}" \
                    -d "${TEST_DB_NAME}" \
                    -tAc \
                    "SELECT COUNT(*) FROM information_tables WHERE table_schema = 'public';" 2>/dev/null || echo "0")

    log_info "Total tables in database: ${table_count}"

    # Run a basic query
    local test_query
    test_query=$(psql -h "${POSTGRES_HOST}" \
                     -p "${POSTGRES_PORT}" \
                     -U "${POSTGRES_USER}" \
                     -d "${TEST_DB_NAME}" \
                     -tAc \
                     "SELECT COUNT(*) FROM information_objects LIMIT 1;" 2>/dev/null || echo "-1")

    if [[ "${test_query}" != "-1" ]]; then
        log_success "Basic query test passed"
    else
        log_error "Basic query test failed"
        errors=$((errors + 1))
    fi

    unset PGPASSWORD

    return ${errors}
}

cleanup_test_database() {
    if [[ "${KEEP_TEST_DB}" == true ]]; then
        log_info "Keeping test database: ${TEST_DB_NAME}"
        log_info "To drop it manually: DROP DATABASE ${TEST_DB_NAME};"
        return 0
    fi

    log_info "Cleaning up test database..."

    export PGPASSWORD="${POSTGRES_PASSWORD:-}"

    # Close all connections to the test database
    psql -h "${POSTGRES_HOST}" \
         -p "${POSTGRES_PORT}" \
         -U "${POSTGRES_USER}" \
         -d postgres \
         -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '${TEST_DB_NAME}';" >/dev/null 2>&1 || true

    # Drop the test database
    if psql -h "${POSTGRES_HOST}" \
           -p "${POSTGRES_PORT}" \
           -U "${POSTGRES_USER}" \
           -d postgres \
           -c "DROP DATABASE ${TEST_DB_NAME};" 2>/dev/null; then
        log_success "Test database dropped"
    else
        log_error "Failed to drop test database (may need manual cleanup)"
    fi

    unset PGPASSWORD
}

# Main execution
main() {
    log_info "=========================================="
    log_info "Backup Testing Script"
    log_info "=========================================="

    # Find latest backup
    local backup_file
    backup_file=$(find_latest_backup)
    if [[ $? -ne 0 ]]; then
        exit 1
    fi

    # Verify backup integrity
    if ! verify_backup_integrity "${backup_file}"; then
        log_error "Backup integrity check failed"
        exit 1
    fi

    # Create test database
    if ! create_test_database; then
        log_error "Failed to create test database"
        exit 1
    fi

    # Restore backup
    if ! restore_backup "${backup_file}"; then
        log_error "Failed to restore backup"
        cleanup_test_database
        exit 1
    fi

    # Verify restored data
    if ! verify_restored_data; then
        log_error "Data verification failed"
        cleanup_test_database
        exit 1
    fi

    # Cleanup
    cleanup_test_database

    log_info "=========================================="
    log_success "All backup tests passed!"
    log_info "=========================================="
}

# Trap to ensure cleanup on exit
trap cleanup_test_database EXIT

# Run main function
main "$@"
