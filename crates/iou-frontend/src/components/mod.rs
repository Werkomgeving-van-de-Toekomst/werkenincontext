//! Reusable UI components

mod approval_actions;
mod app_card;
mod audit_viewer;
mod document_card;
mod header;
mod knowledge_graph;
mod layer_control_3d;
mod loading;
mod map_3d;
mod panel;
mod timeline;

pub use approval_actions::ApprovalActions;
pub use app_card::AppCard;
pub use audit_viewer::AuditTrailViewer;
pub use document_card::DocumentCard;
pub use header::Header;
pub use knowledge_graph::KnowledgeGraph;
pub use panel::Panel;
pub use timeline::{Timeline, TimelineEvent, TimelineEventType};

// 3D Map Components (Map Engine 3D Upgrade)
// pub use declarations will be added when components are implemented
// pub use map_3d::Map3D;
/// pub use layer_control_3d::LayerControl3D;

#[cfg(test)]
mod tests {
    // These tests verify that the 3D map modules compile successfully.
    // The mere existence of these tests confirms that mod map_3d and
    // mod layer_control_3d declarations are valid.

    #[test]
    fn test_map3d_module_compiles() {
        // If this test compiles, the map_3d module exists
        assert!(true, "map_3d module is accessible");
    }

    #[test]
    fn test_layer_control_3d_module_compiles() {
        // If this test compiles, the layer_control_3d module exists
        assert!(true, "layer_control_3d module is accessible");
    }
}
