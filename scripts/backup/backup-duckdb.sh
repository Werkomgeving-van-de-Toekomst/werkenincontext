#!/bin/bash
# DuckDB Export/Backup Script for IOU-Modern
#
# This script exports DuckDB data to Parquet files for backup purposes.
# DuckDB is used for analytics and search - this ensures data preservation.
#
# Usage: ./backup-duckdb.sh [dry-run]
#
# Environment variables:
#   - DUCKDB_PATH: Path to DuckDB database file
#   - BACKUP_DIR: Local backup directory (default: /data/backups/duckdb)
#   - RETENTION_DAYS: Days to keep backups (default: 30)

set -euo pipefail

# Configuration from environment or defaults
DUCKDB_PATH="${DUCKDB_PATH:-/data/iou-modern.duckdb}"
BACKUP_DIR="${BACKUP_DIR:-/data/backups/duckdb}"
RETENTION_DAYS="${RETENTION_DAYS:-30}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_SUBDIR="iou_modern_${TIMESTAMP}"

# Dry run flag
DRY_RUN="${1:-}"

# Functions
log_info() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] INFO: $1"
}

log_error() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1" >&2
}

export_duckdb_to_parquet() {
    local db_path="$1"
    local output_dir="$2"

    log_info "Exporting DuckDB data to Parquet..."
    log_info "Database: ${db_path}"
    log_info "Output: ${output_dir}"

    if [[ "$DRY_RUN" == "dry-run" ]]; then
        log_info "[DRY RUN] Would export tables to ${output_dir}"
        return 0
    fi

    if [[ ! -f "${db_path}" ]]; then
        log_error "DuckDB database not found: ${db_path}"
        return 1
    fi

    mkdir -p "${output_dir}"

    # Create a Python script to export DuckDB tables to Parquet
    cat > /tmp/export_duckdb.py << 'EOF'
import duckdb
import sys
import os

db_path = sys.argv[1]
output_dir = sys.argv[2]

try:
    con = duckdb.connect(db_path)

    # Get list of all tables
    tables = con.execute("SHOW TABLES").fetchall()

    for table in tables:
        table_name = table[0]
        output_path = os.path.join(output_dir, f"{table_name}.parquet")

        print(f"Exporting table: {table_name}")

        con.execute(f"""
            COPY {table_name} TO '{output_path}'
            (FORMAT PARQUET, COMPRESSION 'ZSTD')
        """)

        # Get row count
        count = con.execute(f"SELECT COUNT(*) FROM {table_name}").fetchone()[0]
        print(f"  Exported {count} rows to {output_path}")

    con.close()
    print("Export completed successfully")

except Exception as e:
    print(f"Error: {e}", file=sys.stderr)
    sys.exit(1)
EOF

    # Run the export script
    if python3 /tmp/export_duckdb.py "${db_path}" "${output_dir}"; then
        log_info "DuckDB export completed successfully"

        # Create manifest
        cat > "${output_dir}/MANIFEST.txt" << EOF
DuckDB Export Manifest
======================
Database: ${db_path}
Export Date: $(date -Iseconds)
Export Timestamp: ${TIMESTAMP}

Tables exported:
$(find "${output_dir}" -name "*.parquet" -exec basename {} \; | sed 's/.parquet//')

File checksums:
EOF

        sha256sum "${output_dir}"/*.parquet >> "${output_dir}/MANIFEST.txt"

        # Cleanup temp script
        rm -f /tmp/export_duckdb.py

        return 0
    else
        log_error "DuckDB export failed"
        rm -f /tmp/export_duckdb.py
        return 1
    fi
}

create_archive() {
    local source_dir="$1"
    local archive_name="$2"

    log_info "Creating backup archive..."

    if [[ "$DRY_RUN" == "dry-run" ]]; then
        log_info "[DRY RUN] Would create archive: ${archive_name}"
        return 0
    fi

    # Create tar.gz archive
    if tar -czf "${archive_name}" -C "$(dirname "${source_dir}")" "$(basename "${source_dir}")"; then
        local file_size
        file_size=$(du -h "${archive_name}" | cut -f1)
        log_info "Archive created: ${archive_name} (${file_size})"

        # Create checksum
        sha256sum "${archive_name}" > "${archive_name}.sha256"
        log_info "Checksum created: ${archive_name}.sha256"

        return 0
    else
        log_error "Archive creation failed"
        return 1
    fi
}

cleanup_old_backups() {
    log_info "Cleaning up backups older than ${RETENTION_DAYS} days..."

    if [[ "$DRY_RUN" == "dry-run" ]]; then
        log_info "[DRY RUN] Would delete directories older than ${RETENTION_DAYS} days"
        find "${BACKUP_DIR}" -maxdepth 1 -type d -name "iou_modern_*" -mtime +${RETENTION_DAYS} -print
        find "${BACKUP_DIR}" -name "iou_modern_*.tar.gz" -mtime +${RETENTION_DAYS} -print
    else
        local deleted_count
        deleted_count=$(find "${BACKUP_DIR}" -maxdepth 1 -type d -name "iou_modern_*" -mtime +${RETENTION_DAYS} -exec rm -rf {} + -print 2>/dev/null | wc -l)
        deleted_count=$((deleted_count + $(find "${BACKUP_DIR}" -name "iou_modern_*.tar.gz" -mtime +${RETENTION_DAYS} -delete -print | wc -l)))
        log_info "Deleted ${deleted_count} old backup(s)"
    fi
}

# Main execution
main() {
    log_info "=========================================="
    log_info "DuckDB Export/Backup Script"
    log_info "=========================================="

    if [[ "$DRY_RUN" == "dry-run" ]]; then
        log_info "DRY RUN MODE - No changes will be made"
    fi

    # Create backup directory
    mkdir -p "${BACKUP_DIR}"

    local export_path="${BACKUP_DIR}/${BACKUP_SUBDIR}"
    local archive_path="${BACKUP_DIR}/${BACKUP_SUBDIR}.tar.gz"

    # Export DuckDB to Parquet
    if ! export_duckdb_to_parquet "${DUCKDB_PATH}" "${export_path}"; then
        log_error "Export failed"
        exit 1
    fi

    # Create archive
    if [[ "$DRY_RUN" != "dry-run" ]]; then
        if ! create_archive "${export_path}" "${archive_path}"; then
            log_error "Archive creation failed"
            exit 1
        fi

        # Remove export directory after archiving
        rm -rf "${export_path}"
        log_info "Export directory removed after archiving"
    fi

    # Cleanup old backups
    cleanup_old_backups

    log_info "=========================================="
    log_info "Backup process completed"
    log_info "=========================================="
}

# Run main function
main "$@"
