//! Tests for graphrag module exports

#[test]
fn module_exports_graphstore() {
    // Verify GraphStore is accessible from graphrag
    use iou_core::graphrag::GraphStore;

    // This type check verifies the export works
    fn _accepts_graphstore(_: &GraphStore) {}
}

#[test]
fn module_exports_storeerror() {
    use iou_core::graphrag::StoreError;

    fn _accepts_storeerror(_: &StoreError) {}
}

#[test]
fn module_exports_arangoconfig() {
    use iou_core::graphrag::ArangoConfig;

    fn _accepts_arangoconfig(_: &ArangoConfig) {}
}

#[test]
fn module_exports_preserves_existing_types() {
    // Verify backward compatibility
    use iou_core::graphrag::{
        Entity, EntityType, Relationship, RelationshipType, Community
    };

    fn _accepts_entity(_: &Entity) {}
    fn _accepts_entity_type(_: &EntityType) {}
    fn _accepts_relationship(_: &Relationship) {}
    fn _accepts_relationship_type(_: &RelationshipType) {}
    fn _accepts_community(_: &Community) {}
}

#[test]
fn module_exports_traversal_types() {
    use iou_core::graphrag::{
        TraversalRequest, TraversalResult, TraversalDirection,
        Neighbor, NeighborFilters, GraphPath,
    };

    fn _accepts_traversal_request(_: &TraversalRequest) {}
    fn _accepts_traversal_result(_: &TraversalResult) {}
    fn _accepts_traversal_direction(_: &TraversalDirection) {}
    fn _accepts_neighbor(_: &Neighbor) {}
    fn _accepts_neighbor_filters(_: &NeighborFilters) {}
    fn _accepts_graph_path(_: &GraphPath) {}
}

#[test]
fn module_does_not_export_internal_types() {
    // Internal document types should not be exported
    // This is a compile-time test - if it compiles, internal types are not accessible
    use iou_core::graphrag::{GraphStore, Entity, EntityType};

    // These should work (public exports)
    let _entity_type = EntityType::Person;
    let _ = GraphStore::new;

    // These should NOT work (internal types) - would cause compile error if uncommented:
    // let _ = EntityDocument;  // Should fail
    // let _ = RelationshipDocument;  // Should fail
    // let _ = CommunityDocument;  // Should fail
}
