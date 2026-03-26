# Code Review: Section 12 - Bulk Operations

## Summary

The implementation introduces bulk operations for entities and relationships to enable efficient large-scale data import. The code demonstrates good patterns for grouping by collection and handling empty inputs. However, there are **2 critical bugs** that will cause production failures and several important issues that need to be addressed.

**Overall Assessment:** The implementation requires critical bug fixes before it can be used in production.

---

## Critical Issues

### 1. Hardcoded Collection Names Break All Non-Standard Relationships

**File:** `crates/iou-core/src/graphrag/store.rs`
**Lines:** 1713-1715

The `bulk_create_relationships` method uses hardcoded collection names ("persons" and "organizations") for ALL relationships. This will cause ArangoDB errors for ANY relationship that doesn't involve persons → organizations.

**Fix:** Use the generic "entities" collection approach like in `create_relationship`.

### 2. Invalid AQL Syntax in bulk_delete_entities

**File:** `crates/iou-core/src/graphrag/store.rs`
**Line:** 1783

The AQL query uses `REMOVE entity IN entity` which is invalid ArangoDB syntax. The second parameter must be a collection name, not a variable.

**Fix:** Use PARSE_IDENTIFIER to extract collection name or query each collection separately.

---

## Important Issues

### 3. Missing Cascade Delete for Relationships

The `bulk_delete_entities` method deletes entities but does not clean up connected relationships, leaving orphaned edges.

### 4. Missing Performance Test

The spec requires `bulk_create_entities_performance_1000_per_sec` but it was not implemented.

### 5. No Transaction Support

Bulk operations execute without transaction support - partial failures could leave inconsistent state.

### 6. Inefficient Cross-Collection Query

The delete query uses Cartesian product approach which is inefficient.

---

## Things That Look Good

1. Early return for empty inputs
2. UUID generation for nil IDs
3. Grouping by collection pattern
4. Proper error handling
5. AQL injection prevention via bind parameters
6. Good test coverage for basic scenarios

---

## Priority Actions

1. **CRITICAL:** Fix hardcoded collection names in `bulk_create_relationships`
2. **CRITICAL:** Fix invalid AQL syntax in `bulk_delete_entities`
3. **HIGH:** Implement cascade delete for edges
4. **MEDIUM:** Add missing performance test
