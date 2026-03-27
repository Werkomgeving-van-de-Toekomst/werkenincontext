# ArangoDB GraphRAG Module - Usage Guide

## Overview

This module provides a knowledge graph persistence layer using ArangoDB for the IOU application. It supports entities, relationships, communities, and graph traversals.

## Quick Start

### Configuration

```rust
use iou_core::graphrag::ArangoConfig;

let config = ArangoConfig::new(
    "http://localhost:8529",  // ArangoDB URL
    "root",                     // Username
    "",                         // Password
    "_system"                   // Database name
);

// Or load from environment variables:
let config = ArangoConfig::from_env()?;
```

### Creating a Graph Store

```rust
use iou_core::graphrag::GraphStore;

let store = GraphStore::new(&config).await?;

// Ensure collections exist
store.ensure_collections().await?;
store.ensure_indexes().await?;
```

### Working with Entities

```rust
use iou_core::graphrag::{Entity, EntityType};

// Create an entity
let person = Entity {
    id: uuid::Uuid::nil(), // Will be generated
    name: "John Doe".to_string(),
    entity_type: EntityType::Person,
    canonical_name: Some("john-doe".to_string()),
    description: Some("A person".to_string()),
    confidence: 1.0,
    source_domain_id: None,
    metadata: serde_json::json!({}),
    created_at: chrono::Utc::now(),
};

let created = store.create_entity(&person).await?;

// Get an entity
let found = store.get_entity(created.id).await?;
assert!(found.is_some());

// Update an entity
use iou_core::graphrag::EntityUpdate;
let updated = store.update_entity(
    created.id,
    EntityUpdate {
        name: Some("Jane Doe".to_string()),
        ..Default::default()
    }
).await?;

// Delete an entity
store.delete_entity(created.id, false).await?;
```

### Working with Relationships

```rust
use iou_core::graphrag::{Relationship, RelationshipType};

let relationship = Relationship {
    id: uuid::Uuid::nil(),
    source_entity_id: person_id,
    target_entity_id: org_id,
    relationship_type: RelationshipType::WorksFor,
    weight: 1.0,
    confidence: 1.0,
    context: None,
    source_domain_id: None,
    created_at: chrono::Utc::now(),
};

let created_rel = store.create_relationship(&relationship).await?;
```

### Graph Traversals

```rust
use iou_core::graphrag::{TraversalRequest, TraversalDirection};

let request = TraversalRequest {
    start_id: entity_id,
    min_depth: 1,
    max_depth: 3,
    direction: TraversalDirection::Outgoing,
    limit: 100,
};

let result = store.traverse(&request).await?;
println!("Found {} vertices and {} edges",
    result.vertices.len(),
    result.edges.len());
```

### Bulk Operations

```rust
// Create multiple entities efficiently
let entities: Vec<Entity> = vec![/* ... */];
let created = store.bulk_create_entities(entities).await?;

// Delete multiple entities
let ids: Vec<uuid::Uuid> = vec![/* ... */];
let deleted_count = store.bulk_delete_entities(ids).await?;
```

### Communities

```rust
use iou_core::graphrag::Community;

let community = Community {
    id: uuid::Uuid::new_v4(),
    name: "Engineering Team".to_string(),
    description: Some("Software engineers".to_string()),
    level: 0,
    parent_community_id: None,
    member_entity_ids: vec![entity_id1, entity_id2],
    summary: None,
    keywords: vec!["engineering".to_string()],
    created_at: chrono::Utc::now(),
};

store.create_community(&community).await?;

// Add entity to community
store.add_community_member(community.id, entity_id).await?;
```

### Migration Validation

```rust
use iou_core::graphrag::MigrationValidator;

let validator = MigrationValidator::with_tolerance(5.0);

// Compare counts between databases
let pg_counts = vec![
    ("Person".to_string(), 100),
    ("Organization".to_string(), 50),
];
let result = validator.validate_entity_counts(&store, pg_counts).await?;

if result.is_valid {
    println!("Migration successful!");
} else {
    eprintln!("Migration issues detected: {:?}", result.mismatches);
}
```

## Module Structure

```
iou-core/src/graphrag/
├── connection.rs   # ArangoDB connection pool and configuration
├── error.rs        # StoreError types
├── migration.rs    # Migration validation helpers
├── mod.rs          # Module exports
├── queries.rs      # AQL query builders
├── store.rs        # Main GraphStore implementation
└── types.rs        # Entity, Relationship, Community types
```

## Testing

Run tests (requires ArangoDB for integration tests):

```bash
# Run all tests
cargo test --package iou-core

# Run only graphrag tests
cargo test --package iou-core tests::graphrag

# Run ignored tests (requires ArangoDB)
cargo test --package iou-core -- --ignored
```

## API Reference

See [docs.rs](https://docs.rs/iou-core) for full API documentation.

## Implementation Notes

- **Transactions**: Deferred to future iteration - use `get_or_create_entity` for race condition handling
- **Performance**: Bulk operations target ~1000 entities/sec throughput
- **Collection Strategy**: Type-specific collections (persons, organizations, locations, etc.)
- **Edge Strategy**: Type-specific edge collections with generic "entities/" references

## Recent Changes

### Section 12: Bulk Operations
- `bulk_create_entities` - Efficient batch entity creation
- `bulk_create_relationships` - Efficient batch relationship creation
- `bulk_delete_entities` - Efficient batch entity deletion

### Section 13: Migration Strategy
- `MigrationValidator` - Compare PostgreSQL and ArangoDB data
- `MIGRATION_STRATEGY.md` - 3-phase migration guide

### Section 11: Transactions (Deferred)
- Individual operations with immediate commit
- `get_or_create_entity` handles race conditions via UniqueViolation handling
- `upsert_entity` provides atomic upsert semantics
