# Backend Database Evaluation

## Overview

Evaluation van drie kandidaten voor het IOU-Modern backend: DuckDB (current), Convex, en Supabase.

## Status

📋 **Research** - Evaluation complete, recommendation pending

## Candidates

| Candidate | Type | License | Self-host |
|-----------|------|---------|-----------|
| **DuckDB** | Embedded OLAP | MIT | ✅ Yes |
| **Convex** | Cloud platform | Proprietary | ❌ No |
| **Supabase** | PostgreSQL BaaS | Apache 2.0 | ✅ Yes |

## Current State (DuckDB)

**Strengths:**
- Excellent analytical query performance
- Single-file deployment
- Full SQL support
- Works seamlessly with Rust

**Weaknesses:**
- No real-time sync
- Single-writer architecture
- No built-in auth
- Limited transactional performance

## Decision Criteria

| Criterion | Weight | DuckDB | Convex | Supabase |
|-----------|--------|--------|--------|----------|
| Real-time collaboration | Must | ❌ | ✅ | ✅ |
| Self-hosting | Must | ✅ | ❌ | ✅ |
| Open source | High | ✅ | ❌ | ✅ |
| Time to implement | Urgent (<3mo) | ✅ | ✅ | ✅ |
| Dutch gov compliance | High | ✅ | ❌ | ✅ |

## Migration Consideraties

### DuckDB → PostgreSQL (Supabase)

**Schema changes:**
```sql
-- UUID generation
uuid() → gen_random_uuid()

-- Full-text search
ILIKE → tsvector with Dutch config

-- Connection model
Arc<Mutex<Connection>> → PgPool (connection pool)
```

**Complexity:** Medium - syntax changes but same SQL base

## Recommendation

**Hybrid approach:**
- **DuckDB** blijft voor analytics (GraphRAG, entity extraction)
- **Supabase** voor transactionele data + real-time sync
- **S3** voor document storage

## Next Steps

1. Prototype Supabase integration
2. Test Dutch full-text search quality
3. Validate real-time sync performance
4. Migration plan voor productie data

## Related Documents

- Original evaluation: `../../planning-backend-eval/claude-plan.md`

---

*Source: planning-backend-eval/claude-plan.md*
