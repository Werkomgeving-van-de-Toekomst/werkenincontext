# RLS Policy Debugging Runbook

**Purpose:** Guide for diagnosing and resolving Row-Level Security (RLS) policy issues

**Last Updated:** 2026-03-14

---

## Table of Contents

1. [Understanding RLS in IOU-Modern](#understanding-rls)
2. [Common RLS Issues](#common-rls-issues)
3. [Diagnostic Queries](#diagnostic-queries)
4. [Resolution Procedures](#resolution-procedures)
5. [Performance Tuning](#performance-tuning)
6. [Testing RLS Policies](#testing-rls-policies)

---

## Understanding RLS in IOU-Modern

### RLS Architecture

IOU-Modern uses PostgreSQL Row-Level Security for multi-tenant isolation:

- **Organization Isolation:** Users see only their organization's data
- **Classification Filtering:** Access based on security clearance
- **Owner-Based Access:** Users can modify their own records
- **Woo Publication:** Public access for published Woo documents

### Key RLS Functions

```sql
-- Extract organization_id from JWT claim
auth.organization_id() RETURNS UUID

-- Extract user role from JWT
auth.user_role() RETURNS VARCHAR

-- Check if user has required clearance
auth.has_clearance(required_level VARCHAR) RETURNS BOOLEAN
```

---

## Common RLS Issues

### Issue 1: "Permission Denied" on Own Data

**Symptoms:**
- User cannot access records they created
- 403 Forbidden errors from API
- Logs show RLS policy rejection

**Diagnosis:**
```sql
-- Check if user's organization_id matches record's organization
SELECT
    id,
    organization_id,
    created_by,
    title
FROM information_objects
WHERE id = '<record_id>';

-- Check current JWT claims (requires API context)
-- Look for organization_id in auth token
```

**Common Causes:**

| Cause | Solution |
|-------|----------|
| Missing `organization_id` claim | Ensure JWT includes custom claim |
| Organization mismatch | Verify user is in correct organization |
| Policy not applied | Run `ALTER TABLE ... ENABLE ROW LEVEL SECURITY` |

**Resolution:**
```sql
-- Verify RLS is enabled on the table
SELECT relname, relrowsecurity
FROM pg_class
WHERE relname = 'information_objects';

-- Check if user's organization_id is present
-- (In API logs or middleware)
```

---

### Issue 2: Performance Degradation with RLS

**Symptoms:**
- Query latency increases significantly after enabling RLS
- Slow queries on previously fast endpoints
- High CPU usage on database

**Diagnosis:**
```sql
-- Check for RLS policy performance issues
EXPLAIN ANALYZE
SELECT * FROM information_objects
WHERE domain_id = '<domain_id>';

-- Look for "Subquery" or "InitPlan" in output
-- These indicate RLS policy checks
```

**Common Causes:**
- Missing indexes on RLS filter columns
- Complex policy expressions (not optimized)
- Too many policies on single table

**Resolution:**
```sql
-- Add partial indexes for common RLS patterns
CREATE INDEX IF NOT EXISTS idx_objects_org_active
  ON information_objects(domain_id, created_at)
  WHERE classification != 'geheim';

-- Use SECURITY INVOKER for complex policy functions
CREATE OR REPLACE FUNCTION auth.check_complex_policy()
RETURNS BOOLEAN
SECURITY INVOKER -- Prevents inlining
AS '...'
```

---

### Issue 3: Public Woo Access Not Working

**Symptoms:**
- Anonymous users cannot access Woo documents
- 401 Unauthorized on public endpoints
- Woo publication appears broken

**Diagnosis:**
```sql
-- Check Woo public access policy
SELECT
    polname,
    polcmd,
    pg_get_expr(polqual, polrelid),
    pg_get_expr(polwithcheck, polrelid)
FROM pg_policy p
JOIN pg_class c ON c.oid = p.polrelid
WHERE c.relname = 'information_objects'
AND p.polname = 'woo_public_read';

-- Verify document meets Woo criteria
SELECT
    id,
    is_woo_relevant,
    woo_publication_date,
    classification
FROM information_objects
WHERE id = '<document_id>';
```

**Resolution:**
```sql
-- Ensure policy exists and is active
DROP POLICY IF EXISTS woo_public_read ON information_objects;

CREATE POLICY woo_public_read ON information_objects
  FOR SELECT
  TO public
  USING (
    is_woo_relevant = true
    AND woo_publication_date IS NOT NULL
    AND woo_publication_date <= CURRENT_TIMESTAMP
    AND classification = 'openbaar'
  );

-- Verify document meets all criteria
UPDATE information_objects
SET
    is_woo_relevant = true,
    woo_publication_date = CURRENT_TIMESTAMP,
    classification = 'openbaar'
WHERE id = '<document_id>';
```

---

## Diagnostic Queries

### View All RLS Policies

```sql
-- List all RLS policies on a table
SELECT
    schemaname,
    tablename,
    policyname,
    permissive,
    roles,
    cmd,
    qual,
    with_check
FROM pg_policies
WHERE schemaname = 'public'
ORDER BY tablename, policyname;
```

### Check RLS Enabled Status

```sql
-- Check which tables have RLS enabled
SELECT
    schemaname,
    tablename,
    rowsecurity as rls_enabled,
    forcerowsecurity as rls_forced
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY rowsecurity DESC, tablename;
```

### Test Policy as Specific User

```sql
-- Set role to test as specific user
SET ROLE authenticated;
SET request.jwt.claim.organization_id = '<org_uuid>';
SET request.jwt.claim.role = 'authenticated';
SET request.jwt.claim.clearance = 'intern';

-- Run query to see what user can see
SELECT * FROM information_objects LIMIT 10;

-- Reset
RESET ALL;
```

### Find Unused Indexes for RLS

```sql
-- Find indexes that might help RLS but aren't being used
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
AND idx_scan = 0
AND indexname NOT LIKE '%_pkey'
ORDER BY tablename, indexname;
```

---

## Resolution Procedures

### Procedure 1: Fix Missing JWT Claims

**Problem:** RLS policies reference custom JWT claims that aren't being set.

**Solution:**

1. **Verify JWT includes custom claims:**
```bash
# Decode JWT and check for organization_id
echo "YOUR_JWT_TOKEN" | jq .
```

2. **Add custom claims in Supabase:**
```sql
-- In Supabase SQL Editor
-- Create or update JWT hook function
CREATE OR REPLACE FUNCTION auth.jwt_custom_claims()
RETURNS jsonb AS $$
DECLARE
    claims jsonb;
    user_org_id uuid;
BEGIN
    -- Get user's organization from user_metadata
    SELECT raw_user_meta_data->>'organization_id'::uuid
    INTO user_org_id
    FROM auth.users
    WHERE id = auth.uid();

    claims := jsonb_build_object(
        'organization_id', user_org_id,
        'clearance', 'intern',
        'app_roles', '["domain_viewer"]'
    );

    RETURN claims;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
```

3. **Verify claims are added:**
```sql
-- Test the function
SELECT auth.jwt_custom_claims();
```

### Procedure 2: Fix Cross-Organization Data Access

**Problem:** User sees data from other organizations.

**Solution:**

1. **Identify the problematic policy:**
```sql
SELECT polname, pg_get_expr(polqual, polrelid) as qual
FROM pg_policy p
JOIN pg_class c ON c.oid = p.polrelid
WHERE c.relname = 'information_objects'
AND p.polname LIKE '%org%';
```

2. **Verify policy uses organization_id correctly:**
```sql
-- Should use auth.organization_id() not just organization_id
-- WRONG: USING (organization_id = organization_id)
-- RIGHT: USING (organization_id = auth.organization_id())
```

3. **Fix the policy:**
```sql
DROP POLICY IF EXISTS org_isolation_select ON information_objects;

CREATE POLICY org_isolation_select ON information_objects
  FOR SELECT
  TO authenticated
  USING (
    EXISTS (
      SELECT 1 FROM information_domains
      WHERE information_domains.id = information_objects.domain_id
      AND information_domains.organization_id = auth.organization_id()
    )
  );
```

### Procedure 3: Fix OR vs AND Policy Logic

**Problem:** Policy logic incorrect (allowing too much or too little access).

**Example:**
```sql
-- WRONG: Uses OR - allows access if user created ANY object
CREATE POLICY bad_policy ON information_objects
  FOR UPDATE
  TO authenticated
  USING (created_by = auth.uid() OR organization_id = auth.organization_id());

-- CORRECT: Uses separate policies for different access patterns
CREATE POLICY owner_update ON information_objects
  FOR UPDATE
  TO authenticated
  USING (created_by = auth.uid());

CREATE POLICY org_update ON information_objects
  FOR UPDATE
  TO authenticated
  USING (
    created_by != auth.uid()
    AND EXISTS (
      SELECT 1 FROM information_domains d
      WHERE d.id = information_objects.domain_id
      AND d.organization_id = auth.organization_id()
      AND auth.has_clearance('vertrouwelijk')
    )
  );
```

---

## Performance Tuning

### Use Partial Indexes

```sql
-- Index only records that are commonly accessed via RLS
CREATE INDEX idx_objects_not_geheim
  ON information_objects(domain_id, created_at)
  WHERE classification != 'geheim';

-- Index for Woo public documents
CREATE INDEX idx_objects_woo_public
  ON information_objects(woo_publication_date)
  WHERE is_woo_relevant = true
  AND woo_publication_date IS NOT NULL;
```

### Optimize Policy Functions

```sql
-- Use IMMUTABLE where possible
CREATE OR REPLACE FUNCTION auth.is_public_document(doc_id uuid)
RETURNS BOOLEAN
IMMUTABLE -- Can be optimized better
AS $$
  SELECT EXISTS (
    SELECT 1 FROM information_objects
    WHERE id = doc_id
    AND classification = 'openbaar'
  );
$$ LANGUAGE sql;

-- Or use STABLE to prevent repeated calculations
CREATE OR REPLACE FUNCTION auth.user_organization()
RETURNS UUID
STABLE -- Result doesn't change during transaction
AS $$
  SELECT nullif(current_setting('request.jwt.claim.organization_id', true), '')::uuid
$$ LANGUAGE sql;
```

### Materialize Common Permission Checks

```sql
-- Add computed column for common checks
ALTER TABLE information_objects
ADD COLUMN is_editable_by_org BOOLEAN
GENERATED ALWAYS AS (
  classification IN ('openbaar', 'intern')
) STORED;

-- Create index on computed column
CREATE INDEX idx_objects_editable
  ON information_objects(domain_id)
  WHERE is_editable_by_org = true;
```

---

## Testing RLS Policies

### Unit Test Template

```sql
-- Test 1: User can see own organization's data
BEGIN;
  SET ROLE authenticated;
  SET request.jwt.claim.organization_id = '<org_a_uuid>';

  SELECT COUNT(*) FROM information_domains;
  -- Should return count for org_a only
COMMIT;

-- Test 2: User cannot see other organization's data
BEGIN;
  SET ROLE authenticated;
  SET request.jwt.claim.organization_id = '<org_a_uuid>';

  SELECT COUNT(*) FROM information_domains
  WHERE organization_id = '<org_b_uuid>';
  -- Should return 0
COMMIT;

-- Test 3: Public access to Woo documents
BEGIN;
  SET ROLE public;
  -- No JWT claims for anonymous

  SELECT COUNT(*) FROM information_objects
  WHERE is_woo_relevant = true
  AND woo_publication_date <= CURRENT_TIMESTAMP;
  -- Should return only published public documents
COMMIT;
```

### Integration Test via API

```bash
# Test as authenticated user
curl -X GET http://localhost:8080/api/domains \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "X-Organization-Id: $ORG_ID"

# Test as anonymous user (Woo endpoint)
curl -X GET http://localhost:8080/api/woo/documents \
  -- No auth header

# Test with insufficient clearance
curl -X GET http://localhost:8080/api/documents/confidential \
  -H "Authorization: Bearer $JWT_TOKEN" \
  # Should return 403
```

---

## Escalation

### Level 1: On-Call Engineer
- Verify user's JWT claims
- Check RLS policy status
- Run diagnostic queries

### Level 2: Backend Lead
- Complex permission issues
- Cross-organization access problems
- Performance optimization

### Level 3: Architecture Team
- RLS architecture changes
- Security policy modifications
- Data breach investigation

---

## Related Documentation

- [Database Architecture](../../architecture/database.md)
- [ETL Troubleshooting](etl_troubleshooting.md)
- [Stabilization Runbook](../stabilization_runbook.md)
- [RLS Policies Schema](../../../migrations/postgres/002_rls_policies.sql)

---

**Contact:** #backend-ops
**Runbook Owner:** Backend Team Lead
**Version:** 1.0
