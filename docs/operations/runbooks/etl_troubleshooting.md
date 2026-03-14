# ETL Troubleshooting Runbook

**Purpose:** Guide for diagnosing and resolving ETL pipeline issues

**Last Updated:** 2026-03-14

---

## Common Issues

### 1. Lag Between Supabase and DuckDB

**Symptoms:**
- Analytics dashboard shows stale data
- Recent documents not appearing in analytics queries
- Compliance overview numbers are outdated

**Diagnosis Steps:**

1. Check ETL service status:
   ```bash
   # Check if API service is running
   systemctl status iou-api
   # Or check process
   ps aux | grep iou-api
   ```

2. Verify last successful checkpoint:
   ```sql
   -- In PostgreSQL
   SELECT * FROM change_outbox
   ORDER BY created_at DESC
   LIMIT 10;
   ```

3. Check unprocessed event count:
   ```sql
   SELECT COUNT(*) as pending_count
   FROM change_outbox
   WHERE processed = false;
   ```

**Resolution:**

- **If service is down:** Restart the API service
  ```bash
  systemctl restart iou-api
  ```

- **If events are pending:** Trigger manual ETL run
  ```bash
  curl -X POST http://localhost:8080/admin/etl/run \
    -H "Authorization: Bearer $ADMIN_TOKEN"
  ```

- **If errors found:** Check logs
  ```bash
  journalctl -u iou-api -f | grep -i etl
  ```

---

### 2. ETL Pipeline Errors

**Symptoms:**
- Error rate > 0% in monitoring
- Failed event count increasing
- ETL metrics show `failed_count > 0`

**Diagnosis Steps:**

1. Check outbox for failed events:
   ```sql
   SELECT
       id,
       aggregate_type,
       event_type,
       retry_count,
       last_error,
       created_at
   FROM change_outbox
   WHERE processed = false
     AND retry_count > 0
   ORDER BY created_at DESC;
   ```

2. Review error messages in logs:
   ```bash
   journalctl -u iou-api -n 1000 | grep -i "etl\|outbox"
   ```

**Common Causes:**

| Error | Cause | Resolution |
|-------|-------|------------|
| Connection refused | DuckDB file locked | Restart API service |
| Column does not exist | Schema mismatch | Run migrations |
| Foreign key violation | Referential integrity issue | Check source data |
| Timeout | Batch size too large | Reduce `ETL_BATCH_SIZE` |

**Resolution:**

- **Retry failed events:** The system automatically retries up to 3 times
- **Manual retry:** Update `retry_count` to 0 and wait for next cycle
- **Skip problematic events:** Mark as processed if non-critical

---

### 3. DuckDB File Issues

**Symptoms:**
- "DuckDB file is locked" errors
- "Database file not found" errors
- Analytics queries return no data

**Diagnosis Steps:**

1. Check DuckDB file exists:
   ```bash
   ls -la /path/to/analytics.duckdb
   ```

2. Check file permissions:
   ```bash
   ls -l /path/to/analytics.duckdb
   # Should be readable by API process
   ```

3. Check for file locks:
   ```bash
   lsof /path/to/analytics.duckdb
   ```

**Resolution:**

- **File missing:** Trigger full ETL refresh to rebuild
  ```bash
  curl -X POST http://localhost:8080/admin/etl/full-refresh \
    -H "Authorization: Bearer $ADMIN_TOKEN"
  ```

- **File locked:** Restart API service to release locks
  ```bash
  systemctl restart iou-api
  ```

- **Corrupted file:** Delete and let ETL rebuild
  ```bash
  # WARNING: Only do this if you have a Supabase backup
  rm /path/to/analytics.duckdb
  systemctl restart iou-api
  ```

---

### 4. Performance Degradation

**Symptoms:**
- ETL cycle time exceeds 5 minutes
- High CPU/memory usage during ETL
- Database connection pool exhaustion

**Diagnosis Steps:**

1. Check current batch size:
   ```bash
   grep ETL_BATCH_SIZE .env
   # Default: 1000
   ```

2. Check ETL metrics:
   ```bash
   curl http://localhost:8080/admin/etl/stats \
    -H "Authorization: Bearer $ADMIN_TOKEN"
   ```

**Resolution:**

- **Reduce batch size:**
  ```bash
  # In .env
  ETL_BATCH_SIZE=500
  systemctl restart iou-api
  ```

- **Increase interval:**
  ```bash
  ETL_INTERVAL_SECONDS=600  # 10 minutes
  systemctl restart iou-api
  ```

- **Enable parallel table processing:**
  ```bash
  ETL_PARALLEL_TABLES=true
  ```

---

## Maintenance Procedures

### Regular Checks (Daily)

- [ ] Verify last ETL cycle completed successfully
- [ ] Check pending outbox event count (< 100)
- [ ] Review ETL error rate (< 1%)
- [ ] Confirm DuckDB file size is reasonable

### Regular Checks (Weekly)

- [ ] Review ETL cycle duration trend
- [ ] Check for stale indexes on PostgreSQL
- [ ] Verify backup completion
- [ ] Review DuckDB file growth rate

### Regular Checks (Monthly)

- [ ] Run full ETL refresh test
- [ ] Review and optimize slow queries
- [ ] Update statistics on PostgreSQL
- [ ] Clean up old audit trail records

---

## Escalation

### Level 1: On-Call Engineer
- Monitor ETL metrics dashboard
- Respond to ETL alerts
- Document incidents
- Perform standard troubleshooting

### Level 2: Backend Lead
- Complex ETL issues
- Performance optimization
- Schema changes
- Data consistency concerns

### Level 3: Architecture Team
- ETL pipeline redesign
- Major version upgrades
- Cross-system issues
- Emergency recovery procedures

---

## Related Documentation

- [Stabilization Runbook](../stabilization_runbook.md)
- [Database Architecture](../../architecture/database.md)
- [Migration Documentation](../../../planning-backend-eval/)

---

**Contact:** #backend-ops
**Runbook Owner:** Backend Team Lead
