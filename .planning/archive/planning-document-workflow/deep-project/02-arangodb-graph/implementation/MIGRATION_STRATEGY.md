# Migration Strategy: PostgreSQL to ArangoDB

## Overview

This document describes the strategy for migrating graph data from PostgreSQL to ArangoDB.

## Migration Phases

### Phase 1: Dual-Write Period

**Duration:** 2-4 weeks

**Activities:**
- Keep PostgreSQL operational as primary graph storage
- Deploy ArangoDB alongside PostgreSQL
- Implement dual-write layer that writes all graph data to both databases
- Add consistency checks between databases
- Monitor write latency and data integrity

**Validation:**
- Compare entity counts between databases daily
- Verify sample data matches weekly
- Monitor for any consistency issues

**Exit Criteria:**
- Less than 1% write latency increase
- Zero data inconsistencies detected
- All validation queries pass

### Phase 2: Read Migration

**Duration:** 1-2 weeks

**Activities:**
- Switch read operations to ArangoDB
- Keep PostgreSQL as backup
- Monitor query performance and correctness
- Collect metrics on query latency

**Validation:**
- Compare query results between databases
- Validate performance metrics meet SLAs
- Check for any missing or incorrect data

**Exit Criteria:**
- All read queries perform within acceptable thresholds
- No data quality issues reported
- PostgreSQL shows no read activity for graph operations

### Phase 3: Cutover

**Duration:** 1 week

**Activities:**
- Deprecate PostgreSQL graph storage
- Remove dual-write code
- Keep PostgreSQL for non-graph operations
- Archive old PostgreSQL graph data

**Validation:**
- Final data comparison
- Backup ArangoDB data
- Remove PostgreSQL graph tables (optional)

**Exit Criteria:**
- ArangoDB is sole source of truth for graph data
- All tests pass
- Documentation updated

## Validation Queries

### Compare Entity Counts

```sql
-- PostgreSQL
SELECT entity_type, COUNT(*) FROM graph_entities GROUP BY entity_type;
```

```aql
-- ArangoDB
FOR e IN entities
  COLLECT type = e.entity_type WITH COUNT INTO count
  RETURN {type, count}
```

### Compare Relationship Counts

```sql
-- PostgreSQL
SELECT relationship_type, COUNT(*) FROM graph_relationships GROUP BY relationship_type;
```

```aql
-- ArangoDB (for each edge collection)
FOR edge IN edge_works_for
  COLLECT type = "WorksFor" WITH COUNT INTO count
  RETURN {type, count}
```

### Sample Data Validation

```sql
-- PostgreSQL - Get 10 random entities
SELECT id, name, entity_type, canonical_name
FROM graph_entities
ORDER BY RANDOM()
LIMIT 10;
```

```aql
-- ArangoDB - Get same entities by ID
FOR id IN @ids
  FOR e IN entities
    FILTER e.id == id
    RETURN e
```

## Rollback Plan

If critical issues are discovered during migration:

1. **Phase 1 Rollback:** Disable ArangoDB writes, continue with PostgreSQL
2. **Phase 2 Rollback:** Switch reads back to PostgreSQL
3. **Phase 3 Rollback:** Restore PostgreSQL from backup, reinstate dual-write

## Monitoring

Track these metrics during migration:

- Write latency (ms)
- Query latency (ms)
- Data consistency checks (% match)
- Error rates (% failed operations)
- Database size (GB)

## Implementation Notes

- Use `MigrationValidator` in `crate::graphrag::migration` for validation
- Set appropriate tolerance percentages for count matching
- Run validation queries regularly during dual-write phase
- Archive PostgreSQL data before removal
