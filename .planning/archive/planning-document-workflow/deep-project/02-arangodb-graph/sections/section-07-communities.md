# Section 7: Community Operations

## Overview

This section implements community management using vertex + edges approach. Communities are vertices in `communities` collection with membership via `edge_member_of` edges.

## Dependencies

- Section 2 (Connection)
- Section 3 (Errors)
- Section 4 (Entity Operations)

## Tests

All tests implemented in `crates/iou-core/tests/graphrag/community_operations.rs`:
- `create_community_creates_community_vertex` ✅
- `create_community_creates_membership_edges` ✅
- `get_community_found_returns_with_members` ✅
- `get_community_not_found_returns_none` ✅
- `add_community_member_creates_edge` ✅
- `add_community_member_prevents_duplicates` ✅
- `remove_community_member_removes_edge` ✅
- `remove_community_member_non_member_returns_error` ✅
- `get_community_members_returns_all_members` ✅

## Implementation

File: `crates/iou-core/src/graphrag/store.rs`

### Collections Added

- `communities` - Vertex collection for communities
- `edge_member_of` - Entity to Community edges (membership)
- `edge_subcommunity` - Community to Community edges (hierarchy - defined but not yet used)

### Methods Implemented

```rust
impl GraphStore {
    pub async fn create_community(&self, community: &Community) -> Result<Community, StoreError>;
    pub async fn get_community(&self, id: Uuid) -> Result<Option<Community>, StoreError>;
    pub async fn add_community_member(&self, community_id: Uuid, entity_id: Uuid) -> Result<bool, StoreError>;
    pub async fn remove_community_member(&self, community_id: Uuid, entity_id: Uuid) -> Result<bool, StoreError>;
    pub async fn get_community_members(&self, community_id: Uuid) -> Result<Vec<Entity>, StoreError>;

    // Helper method
    async fn is_community_member(&self, community_id: Uuid, entity_id: Uuid) -> Result<bool, StoreError>;
}
```

### Types Added

- `CommunityDocument` - ArangoDB document representation for communities
- `MembershipEdge` - ArangoDB edge representation for community membership

### Implementation Notes

1. **create_community**: Creates community vertex and automatically creates membership edges for entities in `member_entity_ids`

2. **add_community_member**: Checks for existing membership before adding edge to prevent duplicates

3. **remove_community_member**: Removes edge_member_of edge, returns false if not a member

4. **get_community_members**: Queries across all vertex collections to find member entities

5. **edge_subcommunity**: Defined in EDGE_COLLECTIONS but no operations implemented yet - reserved for future hierarchy support
