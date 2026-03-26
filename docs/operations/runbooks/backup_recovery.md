# Backup and Recovery Procedures

**Purpose:** Guide for database backup and disaster recovery operations

**Last Updated:** 2026-03-14

---

## Table of Contents

1. [Backup Strategy Overview](#backup-strategy)
2. [Backup Procedures](#backup-procedures)
3. [Recovery Procedures](#recovery-procedures)
4. [Disaster Recovery Scenarios](#disaster-scenarios)
5. [Testing Backups](#testing-backups)
6. [Maintenance Schedules](#maintenance)

---

## Backup Strategy Overview {#backup-strategy}

### Backup Types

| Type | Frequency | Retention | Location | RTO | RPO |
|------|-----------|-----------|----------|-----|-----|
| PostgreSQL Full | Daily (2 AM UTC) | 30 days | Hetzner Storage Box | 1 hour | 5 min |
| PostgreSQL WAL | Continuous | 7 days | Local disk | 15 min | 0 |
| DuckDB Export | Weekly (Sun 3 AM) | 4 weeks | Hetzner Storage Box | 5 min | 1 week |
| Config/Scripts | On change | Indefinite | Git + Storage Box | 5 min | 0 |

### Backup Architecture

```
┌─────────────────┐
│   PostgreSQL    │───── WAL (continuous) ───► Local Storage
│   (Primary)     │
└─────────────────┘
         │
         │ pg_dump (daily)
         ▼
┌─────────────────┐     rsync (daily)     ┌──────────────────┐
│   Local Backup  │──────────────────────►│  Hetzner Storage │
│   /backups/pg/  │                       │      Box         │
└─────────────────┘                       └──────────────────┘
         │
         │ export (weekly)
         ▼
┌─────────────────┐
│   DuckDB        │──────────────────────►│  Hetzner Storage │
│   /data/*.duckdb │                       │      Box         │
└─────────────────┘                       └──────────────────┘
```

---

## Backup Procedures {#backup-procedures}

### PostgreSQL Full Backup

**Automated (via cron):**

```bash
#!/bin/bash
# /usr/local/bin/backup-postgres.sh
# Runs daily at 02:00 UTC

set -e

BACKUP_DIR="/backups/pg"
RETENTION_DAYS=30
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/ioumodern_$DATE.sql.gz"

# Ensure backup directory exists
mkdir -p "$BACKUP_DIR"

# Create backup
pg_dump -h localhost -U postgres -d ioumodern \
  --no-owner --no-acl \
  --format=plain \
  --exclude-table-data='realtime.*' \
  2>&1 | gzip > "$BACKUP_FILE"

# Verify backup was created
if [ ! -s "$BACKUP_FILE" ]; then
  echo "Backup failed: $BACKUP_FILE is empty"
  exit 1
fi

# Upload to Hetzner Storage Box
rsync -avz --delete "$BACKUP_DIR/" /mnt/storage-box/backups/pg/

# Clean up old backups
find "$BACKUP_DIR" -name "ioumodern_*.sql.gz" -mtime +$RETENTION_DAYS -delete

echo "Backup completed: $BACKUP_FILE"
```

**Manual backup:**

```bash
# Quick backup
pg_dump -h localhost -U postgres -d ioumodern \
  --format=custom \
  --file=/backups/pg/manual_$(date +%Y%m%d).dump

# Compressed SQL format
pg_dump -h localhost -U postgres -d ioumodern \
  --no-owner --no-acl \
  | gzip > backup_$(date +%Y%m%d).sql.gz
```

### WAL Archiving

**Configure in `postgresql.conf`:**

```ini
# Enable WAL archiving
wal_level = replica
archive_mode = on
archive_command = 'rsync -a %p /mnt/wal_archive/%f'
archive_timeout = 300  # 5 minutes
```

**Manual WAL checkpoint:**

```sql
-- Force WAL switch and archive
SELECT pg_switch_wal();
```

### DuckDB Backup

**Automated (weekly):**

```bash
#!/bin/bash
# /usr/local/bin/backup-duckdb.sh
# Runs weekly on Sunday at 03:00 UTC

set -e

BACKUP_DIR="/backups/duckdb"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"

# Export database to SQL (can be reimported)
duckdb data/analytics.duckdb "EXPORT DATABASE '$BACKUP_DIR/iou_analytics_$DATE.sql'"

# Also copy the database file (faster restore)
cp data/analytics.duckdb "$BACKUP_DIR/analytics_$DATE.duckdb"

# Upload to storage
rsync -avz "$BACKUP_DIR/" /mnt/storage-box/backups/duckdb/

# Keep 4 weeks
find "$BACKUP_DIR" -name "*.duckdb" -mtime +28 -delete
find "$BACKUP_DIR" -name "*.sql" -mtime +28 -delete

echo "DuckDB backup completed"
```

**Manual export:**

```bash
# Export to SQL
duckdb data/analytics.duckdb "EXPORT DATABASE 'backup.sql'"

# Export specific tables
duckdb data/analytics.duckdb "COPY information_objects TO 'objects.csv' (HEADER, DELIMITER ',')"
```

### Configuration Backup

```bash
# Backup all configuration files
tar -czf /backups/config_$(date +%Y%m%d).tar.gz \
  /etc/postgresql/ \
  /etc/supabase/ \
  docker-compose.supabase.yml \
  .env.supabase

# Also keep configs in git (excluding secrets)
git add docker-compose.supabase.yml
git commit -m "chore: backup configuration $(date +%Y-%m-%d)"
```

---

## Recovery Procedures {#recovery-procedures}

### Restore PostgreSQL from Backup

**Scenario:** Recover from accidental data deletion or corruption

**Step 1: Stop application**

```bash
# Stop API to prevent writes during recovery
systemctl stop iou-api
# Or
docker-compose stop iou-api
```

**Step 2: Identify backup to restore**

```bash
# List available backups
ls -lht /backups/pg/*.sql.gz | head -10

# Or from storage box
ls -lht /mnt/storage-box/backups/pg/*.sql.gz | head -10
```

**Step 3: Restore backup**

```bash
# Download from storage if needed
cp /mnt/storage-box/backups/pg/ioumodern_20260314_020000.sql.gz /tmp/

# Decompress
gunzip /tmp/ioumodern_20260314_020000.sql.gz

# Restore (drops and recreates database)
psql -h localhost -U postgres -d postgres <<EOF
DROP DATABASE IF EXISTS ioumodern;
CREATE DATABASE ioumodern;
GRANT ALL PRIVILEGES ON DATABASE ioumodern TO postgres;
EOF

# Restore data
psql -h localhost -U postgres -d ioumodern < /tmp/ioumodern_20260314_020000.sql
```

**Step 4: Run migrations for any changes since backup**

```bash
# Apply any migrations that happened after the backup
psql -h localhost -U postgres -d ioumodern -f migrations/postgres/005_outbox_table.sql
```

**Step 5: Restart application**

```bash
systemctl start iou-api
# Or
docker-compose start iou-api
```

**Step 6: Verify**

```bash
# Check row counts
psql -h localhost -U postgres -d ioumodern -c "
SELECT
  'information_domains' as table_name, COUNT(*) as row_count
  FROM information_domains
UNION ALL
SELECT 'information_objects', COUNT(*) FROM information_objects
UNION ALL
SELECT 'documents', COUNT(*) FROM documents;
"

# Run health check
curl http://localhost:8080/health
```

### Point-in-Time Recovery (PITR)

**Scenario:** Recover to specific time before error

**Prerequisites:** WAL archives available

```bash
# 1. Create recovery.conf
cat > /var/lib/postgresql/data/recovery.conf <<EOF
restore_command = 'cp /mnt/wal_archive/%f %p'
recovery_target_time = '2026-03-14 14:30:00 UTC'
EOF

# 2. Start PostgreSQL (will enter recovery mode)
systemctl restart postgresql

# 3. Monitor recovery
tail -f /var/log/postgresql/postgresql-*.log
# Look for "recovery target reached"

# 4. Once recovered, promote to primary
pg_ctl promote -D /var/lib/postgresql/data

# 5. Remove recovery.conf
rm /var/lib/postgresql/data/recovery.conf
```

### Restore DuckDB from Backup

**Scenario:** DuckDB file corrupted or lost

**Quick restore (from file copy):**

```bash
# Stop API (DuckDB must not be in use)
systemctl stop iou-api

# Restore from backup
cp /backups/duckdb/analytics_20260310_030000.duckdb data/analytics.duckdb

# Restart
systemctl start iou-api
```

**Full restore (from SQL export):**

```bash
# Create new database
rm data/analytics.duckdb

# Import from SQL export
duckdb data/analytics.duckdb < /backups/duckdb/iou_analytics_20260310_030000.sql

# Or rebuild from Supabase via ETL
curl -X POST http://localhost:8080/admin/etl/full-refresh \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

### Recover Single Table

**Scenario:** Accidental DROP TABLE or TRUNCATE

**From PostgreSQL backup:**

```bash
# Extract single table from full backup
zcat /backups/pg/ioumodern_20260314.sql.gz | \
  grep -A 1000 "COPY public.information_objects" | \
  psql -h localhost -U postgres -d ioumodern
```

**Using pg_dump for single table:**

```bash
# Dump just the table structure and data
pg_dump -h localhost -U postgres -d ioumodern \
  -t information_objects \
  --format=plain \
  > information_objects_backup.sql

# Restore it
psql -h localhost -U postgres -d ioumodern < information_objects_backup.sql
```

---

## Disaster Recovery Scenarios {#disaster-scenarios}

### Scenario 1: Complete Server Failure

**Severity:** Critical

**Recovery Steps:**

1. **Provision new server** (Hetzner rescue or new CX41)
2. **Install dependencies:**
   ```bash
   apt update && apt install -y postgresql-client docker-compose
   ```
3. **Deploy Supabase:**
   ```bash
   docker-compose -f docker-compose.supabase.yml up -d
   ```
4. **Restore latest PostgreSQL backup** (see procedure above)
5. **Rebuild DuckDB from Supabase:**
   ```bash
   curl -X POST http://localhost:8080/admin/etl/full-refresh \
     -H "Authorization: Bearer $ADMIN_TOKEN"
   ```
6. **Update DNS** to point to new server IP

**Estimated RTO:** 1-2 hours

### Scenario 2: Data Corruption Detected

**Severity:** High

**Recovery Steps:**

1. **Identify corruption time window**
   ```sql
   SELECT MIN(timestamp), MAX(timestamp)
   FROM audit_trail
   WHERE details LIKE '%corruption%' OR action = 'data_error';
   ```
2. **Stop all writes**
   ```bash
   systemctl stop iou-api
   ```
3. **Restore from backup before corruption**
4. **Replay WAL to point just before corruption** (PITR)
5. **Verify data integrity**
6. **Resume operations**

**Estimated RTO:** 30-60 minutes

### Scenario 3: Accidental Data Deletion

**Severity:** Medium

**Recovery Steps:**

1. **Identify deleted records from audit trail**
   ```sql
   SELECT * FROM audit_trail
   WHERE action = 'DELETE'
   AND timestamp > NOW() - INTERVAL '1 hour'
   ORDER BY timestamp DESC;
   ```
2. **Restore specific table** from backup
3. **Merge with current data** (keep changes after deletion)
4. **Verify referential integrity**

**Estimated RTO:** 15-30 minutes

### Scenario 4: Ransomware Attack

**Severity:** Critical

**Immediate Actions:**

1. **Disconnect network** - Stop all services
2. **Preserve forensic evidence** - Don't modify affected systems
3. **Assess scope** - Check which systems are compromised
4. **Contact security team** - Initiate incident response

**Recovery Steps:**

1. **Wipe affected servers**
2. **Rebuild from known-good backups**
3. **Change all credentials**
4. **Scan for persistence mechanisms**
5. **Monitor for suspicious activity**

**Estimated RTO:** 4-8 hours (plus incident response)

---

## Testing Backups {#testing-backups}

### Automated Backup Verification

**Weekly automated test:**

```bash
#!/bin/bash
# /usr/local/bin/test-backups.sh

set -e

echo "Testing backup restoration..."

# Create test database
psql -h localhost -U postgres -d postgres <<EOF
DROP DATABASE IF EXISTS ioumodern_restore_test;
CREATE DATABASE ioumodern_restore_test;
EOF

# Restore latest backup to test database
LATEST_BACKUP=$(ls -t /backups/pg/*.sql.gz | head -1)
gunzip -c "$LATEST_BACKUP" | \
  psql -h localhost -U postgres -d ioumodern_restore_test > /dev/null

# Verify data integrity
RESULT=$(psql -h localhost -U postgres -d ioumodern_restore_test -tAc "
  SELECT COUNT(*) FROM information_domains;
")

if [ "$RESULT" -gt 0 ]; then
  echo "✓ Backup test passed: $RESULT domains restored"
else
  echo "✗ Backup test failed: No data restored"
  exit 1
fi

# Cleanup
psql -h localhost -U postgres -d postgres -c "DROP DATABASE ioumodern_restore_test;"

echo "Backup verification completed successfully"
```

### Monthly Full Restore Test

**Procedure:**

1. **Deploy staging environment**
2. **Restore backup to staging**
3. **Run full test suite**
4. **Verify data integrity**
5. **Document any issues**

**Checklist:**

- [ ] Backup file is readable
- [ ] Database schema matches expected
- [ ] Row counts are reasonable
- [ ] RLS policies work correctly
- [ ] API connects to restored database
- [ ] Integration tests pass

---

## Maintenance Schedules {#maintenance}

### Daily Tasks (Automated)

- [ ] PostgreSQL full backup (2 AM UTC)
- [ ] WAL archival (continuous)
- [ ] Backup upload to Storage Box (3 AM UTC)
- [ ] Cleanup of old backups (per retention policy)

### Weekly Tasks (Automated)

- [ ] DuckDB export (Sunday 3 AM UTC)
- [ ] Backup verification script (Sunday 4 AM UTC)

### Monthly Tasks (Manual)

- [ ] Review backup sizes and growth trends
- [ ] Test restore procedure (non-production)
- [ ] Verify Storage Box capacity
- [ ] Update documentation if procedures changed

### Quarterly Tasks (Manual)

- [ ] Full disaster recovery drill
- [ ] Review and update backup retention policy
- [ ] Assess need for additional storage
- [ ] Update contact information for escalation

---

## Backup Verification Commands

### Check Backup Status

```bash
# Latest PostgreSQL backup
ls -lht /backups/pg/*.sql.gz | head -1

# Latest DuckDB backup
ls -lht /backups/duckdb/*.duckdb | head -1

# Storage Box sync status
rsync --dryrun -avz /backups/pg/ /mnt/storage-box/backups/pg/
```

### Monitor Disk Usage

```bash
# Backup directory size
du -sh /backups/pg /backups/duckdb

# Available space
df -h /mnt/storage-box

# Database size
psql -h localhost -U postgres -d ioumodern -c "
  SELECT pg_database.datname,
         pg_size_pretty(pg_database_size(pg_database.datname))
  FROM pg_database
  WHERE datname = 'ioumodern';
"
```

---

## Escalation

### Level 1: On-Call Engineer
- Monitor backup jobs
- Respond to backup failures
- Perform test restores

### Level 2: Backend Lead
- Complex recovery scenarios
- Data corruption issues
- PITR procedures

### Level 3: Architecture Team
- Disaster recovery declaration
- Ransomware response
- Major data loss events

---

## Emergency Contacts

| Role | Name | Contact |
|------|------|---------|
| On-Call Engineer | | pagerduty@example.com |
| Backend Lead | | backend-lead@example.com |
| Hetzner Support | | https://www.hetzner.com/support |

---

## Related Documentation

- [Database Architecture](../../architecture/database.md)
- [ETL Troubleshooting](etl_troubleshooting.md)
- [Stabilization Runbook](../stabilization_runbook.md)
- [Hosting Decision](../../../planning-backend-eval/implementation/docs/hosting_decision.md)

---

**Contact:** #backend-ops
**Runbook Owner:** Backend Team Lead
**Version:** 1.0
