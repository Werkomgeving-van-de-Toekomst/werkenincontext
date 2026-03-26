# Stabilization Runbook

**Section 05: Stabilization Phase**
**Version:** 1.0
**Last Updated:** 2026-03-14

This runbook documents operational procedures for monitoring, troubleshooting, and maintaining the Supabase + DuckDB hybrid architecture during the stabilization period.

---

## Table of Contents

1. [Monitoring Dashboard](#monitoring-dashboard)
2. [Alert Thresholds](#alert-thresholds)
3. [Common Issues and Solutions](#common-issues-and-solutions)
4. [Rollback Procedures](#rollback-procedures)
5. [Performance Tuning](#performance-tuning)
6. [ETL Operations](#etl-operations)

---

## Monitoring Dashboard

### Key Metrics to Monitor

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| Query p95 latency | <200ms | >500ms |
| Query p99 latency | <500ms | >1000ms |
| RLS policy check p95 | <250ms | >500ms |
| Replication lag | <10s | >30s |
| WAL size | <500MB | >1GB |
| Disk free | >30% | <20% |
| ETL cycle duration | <4min | >5min |
| ETL error rate | 0% | >5% |
| Auth failure rate | <0.1% | >1% |

### Dashboard Access

Configure Grafana dashboard panels for:

1. **Query Performance Panel**
   - Source: `pg_stat_statements`
   - Query: p50, p95, p99 latencies
   - Group by: Query pattern

2. **Database Health Panel**
   - Active/idle connections
   - Replication lag
   - WAL size
   - Disk usage

3. **Real-time Metrics Panel**
   - Active subscription count
   - Message throughput
   - Connection errors

4. **ETL Status Panel**
   - Last successful run
   - Records per cycle
   - Unprocessed outbox count
   - Error rate

---

## Alert Thresholds

### Configuration

Alert thresholds are configured in `src/monitoring/alerting.rs`:

```rust
pub struct AlertThresholds {
    pub query_p95_ms: 500.0,
    pub replication_lag_secs: 30,
    pub wal_size_bytes: 1024 * 1024 * 1024, // 1GB
    pub disk_free_percent: 20.0,
    pub auth_failure_rate: 0.01,
}
```

### Alert Levels

- **Warning**: Non-critical but requires attention
- **Critical**: Immediate action required

### Alert Response

1. **Query Latency Alert**
   - Check `pg_stat_statements` for slow queries
   - Run `EXPLAIN ANALYZE` on problematic queries
   - Consider adding indexes or optimizing RLS policies

2. **Replication Lag Alert**
   - Check ETL pipeline status
   - Verify outbox processing
   - Check for long-running transactions

3. **Disk Space Alert**
   - Check WAL retention policy
   - Consider VACUUM FULL if needed
   - Plan capacity expansion

---

## Common Issues and Solutions

### 1. Replication Slot Overflow

**Symptoms:**
- WAL accumulation
- Disk space pressure
- Increasing replication lag

**Diagnosis:**
```sql
SELECT slot_name, pg_size_pretty(wal_size),
       pg_size_pretty(wal_size - pg_wal_lsn_diff(lsn, replay_lsn)) as retained
FROM pg_replication_slots;
```

**Solution:**
1. Increase ETL consumer capacity
2. Switch to batch ETL if using CDC
3. As last resort, drop and recreate replication slot

### 2. RLS Policy Regression

**Symptoms:**
- Sudden query performance degradation
- High CPU usage on database

**Diagnosis:**
```sql
SELECT polname, polcmd, pg_get_expr(polqual, polrelid)
FROM pg_policy
JOIN pg_class ON pg_class.oid = polrelid
WHERE relname = 'information_objects';
```

**Solution:**
1. Review recent policy changes
2. Use `EXPLAIN ANALYZE` to identify bottlenecks
3. Consider SECURITY INVOKER functions
4. Add partial indexes

### 3. Real-time Connection Spikes

**Symptoms:**
- Abnormal subscription count
- Connection pool exhaustion

**Diagnosis:**
```sql
SELECT count(*) as active_subscriptions
FROM realtime.subscription
WHERE status = 'active';
```

**Solution:**
1. Implement connection limits
2. Investigate client reconnection logic
3. Check for connection leaks in frontend

### 4. ETL Failures

**Symptoms:**
- DuckDB data stale or missing
- Increasing outbox backlog

**Diagnosis:**
```sql
SELECT processed, COUNT(*), MIN(created_at), MAX(created_at)
FROM change_outbox
GROUP BY processed;
```

**Solution:**
1. Check ETL error logs
2. Verify outbox processing logic
3. Re-run failed cycles manually
4. Consider increasing batch size

---

## Rollback Procedures

### Rollback Triggers

Consider rollback if:
- Data inconsistency detected between databases
- Real-time latency exceeds 500ms for >5 minutes
- RLS policy performance degrades (p95 > 1s)
- Authentication failures > 1% of requests

### Rollback Process (Phase 4 Stabilization)

**Preconditions:**
- Read toggle still exists from Phase 1-2
- DuckDB database is intact

**Steps:**

1. **Stop ETL Pipeline**
   ```bash
   # Set ETL_ENABLED=false
   export ETL_ENABLED=false
   # Restart API service
   ```

2. **Switch Read Toggle**
   ```sql
   UPDATE system_settings SET setting_value = 'duckdb'
   WHERE setting_name = 'primary_database';
   ```

3. **Reconcile Data**
   - Identify records written to Supabase only
   - Apply missing records to DuckDB
   - Verify consistency

4. **Investigate Root Cause**
   - Review logs
   - Analyze metrics
   - Document findings

5. **Re-migrate After Fix**
   - Validate fix in staging
   - Re-run migration from Phase 2
   - Verify data consistency

### Emergency Rollback

If immediate rollback is required:

1. Set feature flag: `DATABASE_SOURCE=duckdb`
2. Restart all API instances
3. Verify traffic flowing to DuckDB
4. Address issues offline

---

## Performance Tuning

### Query Optimization Checklist

1. **Review `pg_stat_statements`**
   ```sql
   SELECT query, calls, total_exec_time, mean_exec_time
   FROM pg_stat_statements
   WHERE calls > 100
   ORDER BY mean_exec_time DESC
   LIMIT 20;
   ```

2. **Check Index Usage**
   ```sql
   SELECT schemaname, tablename, indexname, idx_scan
   FROM pg_stat_user_indexes
   WHERE idx_scan = 0
   AND indexname NOT LIKE '%_pkey';
   ```

3. **Analyze Table Bloat**
   ```sql
   SELECT schemaname, tablename, pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename))
   FROM pg_tables
   ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
   ```

### RLS Optimization

- Use SECURITY INVOKER functions for complex checks
- Create partial indexes for common RLS patterns
- Materialize user permissions where safe
- Consider denormalization for high-traffic queries

---

## ETL Operations

### ETL Configuration

Located in `.env`:

```bash
ETL_ENABLED=true
ETL_INTERVAL_SECONDS=300
ETL_BATCH_SIZE=1000
ETL_INCREMENTAL=true
```

### Manual ETL Run

To trigger an ETL cycle manually:

```bash
curl -X POST http://localhost:8080/admin/etl/run \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

### Check ETL Status

```bash
curl http://localhost:8080/admin/etl/status \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

### Outbox Monitoring

Check unprocessed events:

```sql
SELECT COUNT(*) as unprocessed,
       MIN(created_at) as oldest_event,
       MAX(created_at) as newest_event
FROM change_outbox
WHERE processed = false;
```

---

## Escalation

### Level 1: On-Call Engineer
- Monitor dashboard
- Respond to alerts
- Document incidents

### Level 2: Backend Lead
- Complex incidents
- Performance issues
- Data consistency concerns

### Level 3: Architecture Team
- Rollback decisions
- Major incidents
- Cross-team coordination

---

## Appendix: Useful Queries

### Find Long-Running Transactions

```sql
SELECT pid, now() - pg_stat_activity.query_start AS duration, query
FROM pg_stat_activity
WHERE (now() - pg_stat_activity.query_start) > interval '5 minutes';
```

### Check Table Sizes

```sql
SELECT
  schemaname,
  tablename,
  pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

### Find Missing Indexes

```sql
SELECT schemaname, tablename, attname, n_distinct, correlation
FROM pg_stats
WHERE schemaname = 'public'
AND n_distinct > 100
ORDER BY n_distinct DESC;
```

---

**Document History:**

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-14 | Initial version for Section 05 stabilization |
