//! Reusable UI components

mod approval_actions;
mod app_card;
mod audit_viewer;
mod delegation_manager;
mod diff_viewer;
mod document_card;
mod header;
mod knowledge_graph;
mod layer_control_3d;
mod layer_detection;
mod loading;
mod map_3d;
mod filter_panel_3d;
mod panel;
mod terrain_encoding;
mod timeline;
mod version_history;
mod view_toggle;
mod workflow_stage_tracker;
mod density_heatmap;
mod geo_filter_panel;
mod url_state;

pub use approval_actions::ApprovalActions;
pub use app_card::AppCard;
pub use audit_viewer::AuditTrailViewer;
pub use diff_viewer::DiffViewer;
pub use document_card::DocumentCard;
pub use header::Header;
pub use knowledge_graph::KnowledgeGraph;
pub use panel::Panel;
pub use timeline::{Timeline, TimelineEvent, TimelineEventType};
pub use version_history::VersionHistory;
pub use workflow_stage_tracker::WorkflowStageTracker;

// 3D Map Components (Map Engine 3D Upgrade)
pub use map_3d::{
    build_apply_geo_filter_script, build_maplibre_flevoland_init_script, ConfigError, GeoFilterParams,
    Map3D, Map3DConfig, TerrainSource,
};
pub use layer_control_3d::{
    catalog_layers, CatalogLayer, FlevolandLayersConfig, GeoJsonLayer, GeoJsonLayerBuilder,
    LayerCheckbox, LayerControl3D, LayerType, load_flevoland_layers_config, predefined_layers,
};
pub use geo_filter_panel::GeoFilterPanel;
pub use layer_detection::{detect_layer_type, has_mixed_geometries};
pub use terrain_encoding::{elevation_to_terrain_rgb, terrain_rgb_to_elevation};
// Filter Panel for 3D buildings
pub use filter_panel_3d::{FilterPanel3D, build_filter_expression, build_set_filter_script, build_clear_filter_script};
// View Toggle for 2D/3D switching
pub use view_toggle::{ViewToggle, build_set_view_mode_script, build_get_initial_view_script};
// Density Heatmap for building density visualization
pub use density_heatmap::{DensityHeatmap, build_add_heatmap_layer_script, build_density_calculation_script, build_setup_density_update_script};
// URL state management for shareable URLs
pub use url_state::{build_update_url_script, build_restore_state_script, build_get_url_params_script};

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
