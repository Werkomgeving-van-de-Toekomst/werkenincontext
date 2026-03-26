# ArangoDB Graph Persistence Implementation

## Project Context

This is a Rust-based knowledge graph system (iou-modern) that needs to persist and query graph data using ArangoDB instead of PostgreSQL. The system extracts entities and relationships from documents and needs efficient graph traversal capabilities.

## Requirements

### Primary Goal

Implement a graph persistence layer using ArangoDB that supports:
1. **Entity storage** - Store graph entities (Person, Organization, Location, Law, Date, Money, Policy, Miscellaneous)
2. **Relationship storage** - Store typed relationships between entities (WORKS_FOR, LOCATED_IN, SUBJECT_TO, etc.)
3. **Community detection** - Store and query communities (clusters) of related entities
4. **Graph traversal** - Efficient multi-hop queries for relationship discovery
5. **Snapshots** - Track graph state over time

### Technical Stack

- **Language**: Rust
- **Database**: ArangoDB 3.12+
- **Client Library**: `arangors` crate
- **Existing Types**:
  - `Entity` struct with id, name, entity_type, canonical_name, description, confidence, source_domain_id, metadata, created_at
  - `EntityType` enum: Person, Organization, Location, Law, Date, Money, Policy, Miscellaneous
  - `Relationship` struct with id, source_entity_id, target_entity_id, relationship_type, weight, confidence, context, source_domain_id, created_at
  - `RelationshipType` enum: WorksFor, LocatedIn, SubjectTo, RefersTo, RelatesTo, OwnerOf, ReportsTo, CollaboratesWith, Follows, PartOf, Unknown
  - `Community` struct with id, name, description, level, parent_community_id, member_entity_ids, summary, keywords, created_at

### Key Differences from PostgreSQL Approach

1. **Native graph queries** - Use AQL graph traversal instead of recursive CTEs
2. **Document + Graph model** - Entities as documents, relationships as edge collections
3. **Schema flexibility** - No migrations needed for schema changes
4. **Built-in shortest path** - Use native traversal algorithms

### Non-Functional Requirements

- **Performance**: Sub-100ms response for single-hop queries, sub-500ms for 3-hop traversals
- **Scalability**: Support 100K+ entities, 500K+ relationships
- **Reliability**: ACID transactions for multi-document operations
- **Testability**: Unit tests with in-memory or containerized ArangoDB

## Existing Codebase Context

- **Module location**: `crates/iou-core/src/graphrag/`
- **Current implementation**: Single file `graphrag.rs` with type definitions
- **Need to convert**: Module directory structure with `mod.rs`, `store.rs`, etc.
- **Test location**: `crates/iou-core/tests/graphrag/`

## Questions to Resolve

1. Should entities be stored as vertex documents or use ArangoDB's search capability?
2. How to handle entity deduplication (canonical_name matching)?
3. Should we use ArangoDB's built-in graph functions or custom AQL traversals?
4. How to model community membership - edge collection or embedded array?
5. Should we use ArangoDB's Foxx microservices for any operations?

## Success Criteria

- [ ] ArangoDB connection pool configured
- [ ] Entity CRUD operations working
- [ ] Relationship CRUD operations working
- [ ] Graph traversal queries implemented (1-5 hops)
- [ ] Community storage and retrieval working
- [ ] Snapshot functionality implemented
- [ ] Integration tests passing
- [ ] Performance benchmarks meet requirements
