# Real-time Subscription Issues Runbook

**Purpose:** Guide for diagnosing and resolving Supabase Realtime subscription problems

**Last Updated:** 2026-03-14

---

## Table of Contents

1. [Real-time Architecture Overview](#architecture)
2. [Common Issues](#common-issues)
3. [Diagnostic Procedures](#diagnostics)
4. [Resolution Steps](#resolutions)
5. [Performance Tuning](#tuning)
6. [Testing Real-time](#testing)

---

## Architecture Overview {#architecture}

### Components

```
┌─────────────┐     WebSocket     ┌──────────────────┐
│   Client    │◄────────────────────►│  Supabase       │
│ (Browser)   │     wss://          │  Realtime        │
└─────────────┘                    │  (PostgreSQL CDC)│
                                    └──────────────────┘
                                          │
                                          │ WAL Replication
                                          ▼
                                    ┌──────────────────┐
                                    │   PostgreSQL     │
                                    │   (Primary DB)    │
                                    └──────────────────┘
```

### Channel Configuration

| Channel | Pattern | Description |
|---------|---------|-------------|
| `documents:*` | Wildcard | All document changes |
| `documents:{id=eq.UUID}` | Filtered | Specific document changes |
| `information_objects:domain_id=eq.UUID` | Filtered | Domain-specific changes |

### Message Types

- **INSERT** - New record created
- **UPDATE** - Record modified
- **DELETE** - Record deleted
- **ERROR** - Subscription error

---

## Common Issues {#common-issues}

### Issue 1: Client Cannot Connect

**Symptoms:**
- WebSocket connection fails immediately
- Connection timeout after 30 seconds
- 401 Unauthorized during WebSocket handshake

**Diagnosis:**

```javascript
// Browser console
new WebSocket('wss://your-project.supabase.co/realtime/v1/documents:*')

// Expected: Connected
// Error: "Authentication failed" or "Connection refused"
```

**Common Causes:**

| Cause | Solution |
|-------|----------|
| Invalid JWT token | Refresh token, include in connection URL |
| RLS policy blocks subscription | Add policy for ` realtime.subscription` |
| CORS misconfiguration | Add WebSocket origin to Supabase project |
| Network firewall | Allow outbound WebSocket (port 4000) |

**Resolution:**

```bash
# Verify JWT is valid
curl -I https://your-project.supabase.co/rest/v1/documents \
  -H "apikey: YOUR_ANON_KEY" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Check Realtime service status
curl https://your-project.supabase.co/realtime/v1/status
```

---

### Issue 2: Not Receiving Updates

**Symptoms:**
- WebSocket connected but no updates received
- Changes in database not broadcasted
- Updates delayed significantly

**Diagnosis:**

```sql
-- Check if realtime publication is enabled
SELECT * FROM pg_publication WHERE pubname = 'supabase_realtime';

-- Check which tables are published
SELECT * FROM pg_publication_tables WHERE pubname = 'supabase_realtime';

-- Verify WAL level is sufficient
SHOW wal_level;
-- Should be 'logical'
```

**Resolution:**

```sql
-- Add table to realtime publication
ALTER PUBLICATION supabase_realtime ADD TABLE documents;
ALTER PUBLICATION supabase_realtime ADD TABLE information_objects;

-- Or add all tables
ALTER PUBLICATION supabase_realtime ADD ALL TABLES;
```

---

### Issue 3: RLS Blocking Real-time Updates

**Symptoms:**
- Subscription connects but returns no data
- Some users receive updates, others don't
- Error: "permission denied" in Realtime logs

**Diagnosis:**

```sql
-- Check RLS status on the table
SELECT relname, relrowsecurity
FROM pg_class
WHERE relname = 'documents';

-- Check for realtime-specific RLS policies
SELECT polname, polcmd
FROM pg_policy p
JOIN pg_class c ON c.oid = p.polrelid
WHERE c.relname = 'documents';
```

**Resolution:**

```sql
-- Add policy to allow authenticated users to subscribe
-- (This is in addition to data access policies)

CREATE POLICY "Realtime can subscribe to documents"
ON documents
FOR SELECT
TO authenticated
USING (
  organization_id = auth.organization_id()
);

-- Enable access via realtime schema
GRANT SELECT ON documents TO authenticated;
```

---

### Issue 4: Connection Drops Intermittently

**Symptoms:**
- WebSocket connects but disconnects after a few minutes
- Reconnection loop in client logs
- "Ping timeout" errors

**Diagnosis:**

```bash
# Check Supabase logs
docker-compose logs -f realtime | grep -i "timeout\|disconnect"

# Monitor WebSocket connections
SELECT COUNT(*) as active_connections,
       MIN(connected_at) as oldest_connection
FROM realtime.subscription
WHERE status = 'active';
```

**Common Causes:**

| Cause | Solution |
|-------|----------|
| No heartbeat/ping | Increase heartbeat interval |
| Connection pool exhaustion | Increase `pool_size` |
| Network instability | Implement exponential backoff |
| Database restart | Automatic reconnection in client |

**Resolution:**

```javascript
// Client-side: Add heartbeat and reconnection
const channel = supabase
  .channel('documents:123')
  .on('postgres_changes', { event: '*', schema: 'public' }, payload => {
    console.log('Change received!', payload);
  })
  .subscribe((status) => {
    if (status === 'SUBSCRIPTION_BAD' || status === 'TIMED_OUT') {
      // Reconnect
      setTimeout(() => channel.subscribe(), 1000);
    }
  });
```

---

### Issue 5: High Memory Usage with Many Subscriptions

**Symptoms:**
- Realtime service memory increases over time
- Server slows down with many concurrent subscriptions
- OOM kills on smaller instances

**Diagnosis:**

```bash
# Check Realtime pod/container memory
kubectl top pod -l app=realtime
# Or for Docker
docker stats $(docker ps -q | xargs docker inspect --format '{{.Name}}' | grep realtime)

# Count active subscriptions
SELECT COUNT(*) FROM realtime.subscription WHERE status = 'active';
```

**Resolution:**

```sql
-- Cleanup stale subscriptions
DELETE FROM realtime.subscription
WHERE status = 'closed'
AND updated_at < NOW() - INTERVAL '1 hour';

-- Add connection limits
-- In Supabase configuration or API gateway
```

---

## Diagnostic Procedures {#diagnostics}

### Check Realtime Service Health

```bash
# Health check endpoint
curl https://your-project.supabase.co/realtime/v1/

# Expected: {"status":"ok"}

# With authentication
curl -H "Authorization: Bearer $JWT_TOKEN" \
  https://your-project.supabase.co/realtime/v1/
```

### Monitor Active Subscriptions

```sql
-- View all active subscriptions
SELECT
    s.id,
    s.topics,
    s.status,
    s.created_at,
    s.updated_at,
    EXTRACT(EPOCH FROM (NOW() - s.updated_at)) / 60 as minutes_since_update
FROM realtime.subscription s
WHERE s.status = 'active'
ORDER BY s.updated_at;

-- Find stale subscriptions (no updates in >30 minutes)
SELECT
    s.id,
    s.topics,
    s.created_at
FROM realtime.subscription s
WHERE s.status = 'active'
AND s.updated_at < NOW() - INTERVAL '30 minutes';
```

### Test Real-time Publication

```sql
-- Enable additional logging
SET log_min_messages = 'info';

-- Make a test change
INSERT INTO documents (id, domain_id, document_type, state)
VALUES (gen_random_uuid(), 'test-domain', 'memo', 'draft')
RETURNING id, created_at;

-- Check WAL activity
SELECT * FROM pg_stat_wal_receiver;
```

### Client-Side Diagnostics

```javascript
// Enable detailed logging
const { createClient } = require('@supabase/supabase-js')

const supabase = createClient(SUPABASE_URL, SUPABASE_KEY, {
  db: { schema: 'public' },
  global: {
    headers: { 'X-Client-Info': 'realtime-test' },
  },
  realtime: {
    params: {
      eventsPerSecond: 10
    }
  }
})

// Monitor subscription state
const channel = supabase.channel('test')

channel
  .on('system', {}, (payload) => {
    console.log('System event:', payload)
  })
  .subscribe((status, err) => {
    console.log('Status:', status)
    if (err) console.error('Error:', err)
  })
```

---

## Resolution Steps {#resolutions}

### Step 1: Verify Prerequisites

```bash
# 1. Check WAL level
psql $DATABASE_URL -c "SHOW wal_level"
# Expected: logical

# 2. Check publication exists
psql $DATABASE_URL -c "SELECT * FROM pg_publication"
# Expected: supabase_realtime

# 3. Check tables in publication
psql $DATABASE_URL -c "SELECT * FROM pg_publication_tables WHERE pubname = 'supabase_realtime'"
```

### Step 2: Add Missing Tables to Publication

```sql
-- Add individual table
ALTER PUBLICATION supabase_realtime ADD TABLE documents;

-- Or add all current and future tables
ALTER PUBLICATION supabase_realtime ADD ALL TABLES;

-- Verify
SELECT schemaname, tablename
FROM pg_publication_tables
WHERE pubname = 'supabase_realtime';
```

### Step 3: Configure RLS for Realtime

```sql
-- Grant necessary permissions
GRANT USAGE ON SCHEMA public TO postgres;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO postgres;

-- Add real-time specific policy if needed
CREATE POLICY "Enable realtime access"
ON documents
FOR SELECT
TO authenticated
USING (true);
```

### Step 4: Update Client Connection String

```javascript
// Include JWT token in connection
const { createClient } = require('@supabase/supabase-js')

const supabase = createClient(
  'https://your-project.supabase.co',
  'your-anon-key',
  {
    global: {
      headers: {
        Authorization: `Bearer ${userJwtToken}`
      }
    },
    realtime: {
      // Configure connection settings
      timeout: 30000,
      heartbeatIntervalMs: 10000
    }
  }
)
```

---

## Performance Tuning {#tuning}

### Optimize Subscription Queries

```sql
-- Add indexes for subscription filters
CREATE INDEX idx_documents_org_for_realtime
  ON documents(organization_id)
  WHERE state != 'archived';

-- Include columns commonly used in WHERE clauses
CREATE INDEX idx_objects_domain_created
  ON information_objects(domain_id, created_at DESC);
```

### Reduce WAL Size

```sql
-- Archive old audit trail records
DELETE FROM audit_trail
WHERE timestamp < NOW() - INTERVAL '90 days';

-- Vacuum to reclaim space
VACUUM FULL audit_trail;
```

### Configure Connection Pooling

```bash
# In Supabase or connection pooler (PgBouncer)
# pools: 4 (number of pools)
# default_pool_size: 25 (connections per pool)
# max_client_conn: 100 (total connections)
```

---

## Testing Real-time {#testing}

### Manual Test Procedure

1. **Start Realtime service:**
```bash
docker-compose up -d realtime
```

2. **Create test subscription (client):**
```javascript
const channel = supabase.channel('documents')
  .on('postgres_changes', {
    event: '*',
    schema: 'public',
    table: 'documents',
    filter: 'id=eq.some-uuid'
  }, (payload) => {
    console.log('Change:', payload)
  })
  .subscribe()
```

3. **Make database change:**
```sql
INSERT INTO documents (domain_id, document_type, state)
VALUES ('test', 'memo', 'draft')
RETURNING *;
```

4. **Verify client received update**

### Automated Test

```javascript
// tests/integration/realtime.test.js
import { createClient } from '@supabase/supabase-js'

describe('Realtime subscriptions', () => {
  test('receives INSERT events', async () => {
    const received = []
    const channel = supabase.channel('test-realtime')

    channel.on('postgres_changes', {
      event: 'INSERT',
      schema: 'public',
      table: 'documents'
    }, (payload) => {
      received.push(payload)
    })

    await channel.subscribe()

    // Make a change
    await db.documents.insert({ domain_id: 'test', document_type: 'memo' })

    // Wait for event (with timeout)
    await new Promise(resolve => setTimeout(resolve, 2000))

    expect(received).toHaveLength(1)
    expect(received[0].eventType).toBe('INSERT')
  })
})
```

---

## Escalation

### Level 1: On-Call Engineer
- Check service status
- Verify client connection strings
- Review recent deployments

### Level 2: Backend Lead
- RLS permission issues
- Performance problems
- Complex subscription debugging

### Level 3: Architecture Team
- Realtime architecture changes
- Cross-service issues
- Major outages

---

## Quick Reference

### Useful URLs

| Resource | URL |
|----------|-----|
| Realtime health | `/realtime/v1/` |
| Supabase dashboard | `https://app.supabase.com/project/xxx` |
| WebSocket URL | `wss://xxx.supabase.co/realtime/v1` |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `SUPABASE_URL` | Supabase project URL |
| `SUPABASE_ANON_KEY` | Anonymous/public key |
| `SUPABASE_JWT_SECRET` | For verifying JWT tokens |
| `SUPABASE_REALTIME_URL` | WebSocket endpoint |

---

## Related Documentation

- [Database Architecture](../../architecture/database.md)
- [RLS Debugging](rls_debugging.md)
- [Stabilization Runbook](../stabilization_runbook.md)
- [Realtime Implementation](../../../crates/iou-api/src/realtime/)

---

**Contact:** #backend-ops
**Runbook Owner:** Backend Team Lead
**Version:** 1.0
