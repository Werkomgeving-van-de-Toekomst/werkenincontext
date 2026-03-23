#!/bin/bash
# PostgreSQL Backup Script for IOU-Modern
#
# This script creates automated backups of the PostgreSQL/Supabase database
# and uploads them to S3-compatible storage (MinIO).
#
# Usage: ./backup-postgres.sh [dry-run]
#
# Environment variables required:
#   - POSTGRES_HOST: PostgreSQL host (default: localhost)
#   - POSTGRES_PORT: PostgreSQL port (default: 5432)
#   - POSTGRES_DB: Database name (default: postgres)
#   - POSTGRES_USER: Database user (default: postgres)
#   - POSTGRES_PASSWORD: Database password
#   - BACKUP_DIR: Local backup directory (default: /data/backups/postgres)
#   - RETENTION_DAYS: Days to keep backups (default: 30)
#   - S3_BUCKET: S3 bucket name
#   - MINIO_ENDPOINT: MinIO endpoint URL

set -euo pipefail

# Configuration from environment or defaults
POSTGRES_HOST="${POSTGRES_HOST:-localhost}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
POSTGRES_DB="${POSTGRES_DB:-postgres}"
POSTGRES_USER="${POSTGRES_USER:-postgres}"
BACKUP_DIR="${BACKUP_DIR:-/data/backups/postgres}"
RETENTION_DAYS="${RETENTION_DAYS:-30}"
S3_BUCKET="${S3_BUCKET:-iou-modern-backups}"
MINIO_ENDPOINT="${MINIO_ENDPOINT:-http://localhost:9000}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="iou_modern_${TIMESTAMP}.sql.gz"

# Dry run flag
DRY_RUN="${1:-}"

# Functions
log_info() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] INFO: $1"
}

log_error() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1" >&2
}

cleanup_old_backups() {
    log_info "Cleaning up backups older than ${RETENTION_DAYS} days..."

    if [[ "$DRY_RUN" == "dry-run" ]]; then
        log_info "[DRY RUN] Would delete files older than ${RETENTION_DAYS} days"
        find "${BACKUP_DIR}" -name "iou_modern_*.sql.gz" -mtime +${RETENTION_DAYS} -print
    else
        local deleted_count
        deleted_count=$(find "${BACKUP_DIR}" -name "iou_modern_*.sql.gz" -mtime +${RETENTION_DAYS} -delete -print | wc -l)
        log_info "Deleted ${deleted_count} old backup(s)"
    fi
}

upload_to_s3() {
    local file="$1"

    # Check for S3 configuration via env var prefix (avoid false positives)
    if [[ -z "${AWS_ACCESS_KEY_ID:-}" ]]; then
        log_info "S3 credentials not configured, skipping upload"
        return 0
    fi

    log_info "Uploading ${file} to S3..."

    if [[ "$DRY_RUN" == "dry-run" ]]; then
        log_info "[DRY RUN] Would upload ${file} to s3://${S3_BUCKET}/postgres/"
    else
        # Use AWS CLI if available
        if command -v aws &> /dev/null; then
            aws s3 cp "${file}" "s3://${S3_BUCKET}/postgres/" \
                --endpoint-url="${MINIO_ENDPOINT}"
        elif command -v mc &> /dev/null; then
            # Use MinIO client
            mc alias set minio "${MINIO_ENDPOINT}" \
                "${AWS_ACCESS_KEY_ID}" "${AWS_SECRET_ACCESS_KEY}" >/dev/null 2>&1 || true
            mc cp "${file}" "minio/${S3_BUCKET}/postgres/"
        else
            log_info "Neither aws nor mc CLI found. Skipping S3 upload."
        fi
    fi
}

create_backup() {
    log_info "Starting PostgreSQL backup..."
    log_info "Host: ${POSTGRES_HOST}:${POSTGRES_PORT}, Database: ${POSTGRES_DB}"

    # Create backup directory
    mkdir -p "${BACKUP_DIR}"

    local backup_path="${BACKUP_DIR}/${BACKUP_FILE}"

    if [[ "$DRY_RUN" == "dry-run" ]]; then
        log_info "[DRY RUN] Would create backup: ${backup_path}"
        echo "${backup_path}"
        return 0
    fi

    # Get database password from environment
    export PGPASSWORD="${POSTGRES_PASSWORD:-}"

    # Create the backup
    if pg_dump -h "${POSTGRES_HOST}" \
               -p "${POSTGRES_PORT}" \
               -U "${POSTGRES_USER}" \
               -d "${POSTGRES_DB}" \
               --verbose \
               --no-owner \
               --no-acl \
               --format=plain \
               2>&1 | gzip > "${backup_path}"; then

        local file_size
        file_size=$(du -h "${backup_path}" | cut -f1)
        log_info "Backup completed successfully: ${BACKUP_FILE} (${file_size})"

        # Create checksum
        sha256sum "${backup_path}" > "${backup_path}.sha256"
        log_info "Checksum created: ${BACKUP_FILE}.sha256"

    else
        log_error "Backup failed!"
        return 1
    fi

    unset PGPASSWORD
    echo "${backup_path}"
}

verify_backup() {
    local file="$1"

    log_info "Verifying backup integrity..."

    if [[ "$DRY_RUN" == "dry-run" ]]; then
        log_info "[DRY RUN] Would verify ${file}"
        return 0
    fi

    if [[ ! -f "${file}" ]]; then
        log_error "Backup file not found: ${file}"
        return 1
    fi

    # Test gzip integrity
    if gzip -t "${file}" 2>/dev/null; then
        log_info "Gzip integrity check passed"

        # Verify checksum if exists
        if [[ -f "${file}.sha256" ]]; then
            if sha256sum -c "${file}.sha256" >/dev/null 2>&1; then
                log_info "Checksum verification passed"
            else
                log_error "Checksum verification failed!"
                return 1
            fi
        fi

        return 0
    else
        log_error "Gzip integrity check failed!"
        return 1
    fi
}

# Main execution
main() {
    log_info "=========================================="
    log_info "PostgreSQL Backup Script"
    log_info "=========================================="

    if [[ "$DRY_RUN" == "dry-run" ]]; then
        log_info "DRY RUN MODE - No changes will be made"
    fi

    # Create backup
    local backup_path
    backup_path=$(create_backup)

    if [[ ${PIPESTATUS[0]} -ne 0 ]]; then
        log_error "Backup creation failed"
        exit 1
    fi

    # Verify backup
    if [[ "$DRY_RUN" != "dry-run" ]]; then
        if ! verify_backup "${backup_path}"; then
            log_error "Backup verification failed"
            # Don't delete the failed backup for investigation
            exit 1
        fi
    fi

    # Upload to S3
    upload_to_s3 "${backup_path}"

    # Cleanup old backups
    cleanup_old_backups

    log_info "=========================================="
    log_info "Backup process completed"
    log_info "=========================================="
}

# Run main function
main "$@"
