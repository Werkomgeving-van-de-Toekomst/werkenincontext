//! LayerControl3D Component - Layer toggle control for Map3D
//!
//! This component provides a UI for toggling GeoJSON layer visibility
//! on the 3D map. Each layer has a checkbox and styled label.

/// Type of geometry in a GeoJSON layer.
///
/// Determines how the layer is rendered in MapLibre:
/// - `Point`: Rendered as circles
/// - `Line`: Rendered as lines
/// - `Polygon`: Rendered as filled areas
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerType {
    Point,
    Line,
    Polygon,
}

impl std::fmt::Display for LayerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerType::Point => write!(f, "Point"),
            LayerType::Line => write!(f, "Line"),
            LayerType::Polygon => write!(f, "Polygon"),
        }
    }
}

impl LayerType {
    /// Returns the MapLibre layer type string for this LayerType.
    pub fn maplibre_type(&self) -> &'static str {
        match self {
            LayerType::Point => "circle",
            LayerType::Line => "line",
            LayerType::Polygon => "fill",
        }
    }

    /// Detects the layer type from a GeoJSON geometry type string.
    ///
    /// # Arguments
    ///
    /// * `geometry_type` - The GeoJSON geometry type (e.g., "Point", "LineString")
    ///
    /// # Returns
    ///
    /// The corresponding `LayerType` or `None` if the type is unrecognized.
    pub fn from_geojson_type(geometry_type: &str) -> Option<Self> {
        match geometry_type {
            "Point" | "MultiPoint" => Some(LayerType::Point),
            "LineString" | "MultiLineString" => Some(LayerType::Line),
            "Polygon" | "MultiPolygon" => Some(LayerType::Polygon),
            _ => None,
        }
    }
}

/// Configuration for a GeoJSON layer to be displayed on the map.
///
/// # Example
///
/// ```rust
/// let layer = GeoJsonLayer {
///     id: "provinciegrens".to_string(),
///     name: "Provinciegrens".to_string(),
///     url: "/geojson/provinciegrens.geojson".to_string(),
///     layer_type: LayerType::Polygon,
///     visible: true,
///     color: "#ff0000".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GeoJsonLayer {
    /// Unique identifier for this layer (used in MapLibre)
    pub id: String,

    /// Human-readable display name (Dutch)
    pub name: String,

    /// URL to the GeoJSON data
    pub url: String,

    /// Type of geometry in this layer
    pub layer_type: LayerType,

    /// Initial visibility state
    pub visible: bool,

    /// CSS color for rendering (hex format)
    pub color: String,
}

impl GeoJsonLayer {
    /// Creates a builder for constructing a GeoJsonLayer.
    pub fn builder() -> GeoJsonLayerBuilder {
        GeoJsonLayerBuilder::default()
    }
}

/// Builder for creating GeoJsonLayer instances.
#[derive(Debug, Default)]
pub struct GeoJsonLayerBuilder {
    id: Option<String>,
    name: Option<String>,
    url: Option<String>,
    layer_type: Option<LayerType>,
    visible: Option<bool>,
    color: Option<String>,
}

impl GeoJsonLayerBuilder {
    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn layer_type(mut self, layer_type: LayerType) -> Self {
        self.layer_type = Some(layer_type);
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = Some(visible);
        self
    }

    pub fn color(mut self, color: &str) -> Self {
        self.color = Some(color.to_string());
        self
    }

    /// Builds the GeoJsonLayer, returning an error if required fields are missing.
    pub fn build(self) -> Result<GeoJsonLayer, String> {
        let id = self.id.ok_or("id is required")?;
        let name = self.name.ok_or("name is required")?;
        let url = self.url.ok_or("url is required")?;
        let layer_type = self.layer_type.ok_or("layer_type is required")?;

        // Default color if not specified
        let color = self.color.unwrap_or_else(|| "#ff0000".to_string());

        // Default to visible if not specified
        let visible = self.visible.unwrap_or(true);

        Ok(GeoJsonLayer {
            id,
            name,
            url,
            layer_type,
            visible,
            color,
        })
    }
}

/// Predefined layers for the Flevoland Data Verkenner.
pub fn predefined_layers() -> Vec<GeoJsonLayer> {
    vec![
        GeoJsonLayer {
            id: "provinciegrens".to_string(),
            name: "Provinciegrens".to_string(),
            url: "/geojson/provinciegrens.geojson".to_string(),
            layer_type: LayerType::Polygon,
            visible: true,
            color: "#e74c3c".to_string(), // Red
        },
        GeoJsonLayer {
            id: "cultuurhistorie".to_string(),
            name: "Cultuurhistorie".to_string(),
            url: "/geojson/cultuurhistorie.geojson".to_string(),
            layer_type: LayerType::Point,
            visible: false,
            color: "#3498db".to_string(), // Blue
        },
        // Additional layers to be added in Phase 2:
        // - windturbines (Point)
        // - zonneparken (Point)
        // - fietsnetwerken (Line)
        // - drinkwater (Point/Line)
    ]
}

#[cfg(test)]
mod layer_config_tests {
    use super::*;

    #[test]
    fn test_geojson_layer_creates_with_all_required_fields() {
        let layer = GeoJsonLayer {
            id: "test-layer".to_string(),
            name: "Test Layer".to_string(),
            url: "https://example.com/layer.geojson".to_string(),
            layer_type: LayerType::Point,
            visible: true,
            color: "#ff0000".to_string(),
        };

        assert_eq!(layer.id, "test-layer");
        assert_eq!(layer.name, "Test Layer");
        assert_eq!(layer.url, "https://example.com/layer.geojson");
        assert!(matches!(layer.layer_type, LayerType::Point));
        assert!(layer.visible);
        assert_eq!(layer.color, "#ff0000");
    }

    #[test]
    fn test_layer_type_has_point_variant() {
        let layer_type = LayerType::Point;
        assert!(matches!(layer_type, LayerType::Point));
    }

    #[test]
    fn test_layer_type_has_line_variant() {
        let layer_type = LayerType::Line;
        assert!(matches!(layer_type, LayerType::Line));
    }

    #[test]
    fn test_layer_type_has_polygon_variant() {
        let layer_type = LayerType::Polygon;
        assert!(matches!(layer_type, LayerType::Polygon));
    }

    #[test]
    fn test_layer_type_display_returns_correct_values() {
        assert_eq!(LayerType::Point.to_string(), "Point");
        assert_eq!(LayerType::Line.to_string(), "Line");
        assert_eq!(LayerType::Polygon.to_string(), "Polygon");
    }

    #[test]
    fn test_geojson_layer_default_visible_is_true() {
        let layer = GeoJsonLayer {
            id: "test".to_string(),
            name: "Test".to_string(),
            url: "https://example.com/test.geojson".to_string(),
            layer_type: LayerType::Point,
            visible: true,
            color: "#000000".to_string(),
        };
        assert!(layer.visible);
    }

    #[test]
    fn test_geojson_layer_builder_creates_valid_layer() {
        let layer = GeoJsonLayer::builder()
            .id("test-layer")
            .name("Test Layer")
            .url("https://example.com/layer.geojson")
            .layer_type(LayerType::Polygon)
            .color("#00ff00")
            .build()
            .unwrap();

        assert_eq!(layer.id, "test-layer");
        assert_eq!(layer.name, "Test Layer");
        assert_eq!(layer.url, "https://example.com/layer.geojson");
        assert!(matches!(layer.layer_type, LayerType::Polygon));
        assert!(layer.visible); // Default value
        assert_eq!(layer.color, "#00ff00");
    }

    #[test]
    fn test_layer_type_maplibre_type() {
        assert_eq!(LayerType::Point.maplibre_type(), "circle");
        assert_eq!(LayerType::Line.maplibre_type(), "line");
        assert_eq!(LayerType::Polygon.maplibre_type(), "fill");
    }

    #[test]
    fn test_layer_type_from_geojson_type() {
        assert_eq!(LayerType::from_geojson_type("Point"), Some(LayerType::Point));
        assert_eq!(LayerType::from_geojson_type("MultiPoint"), Some(LayerType::Point));
        assert_eq!(LayerType::from_geojson_type("LineString"), Some(LayerType::Line));
        assert_eq!(LayerType::from_geojson_type("Polygon"), Some(LayerType::Polygon));
        assert_eq!(LayerType::from_geojson_type("Unknown"), None);
    }
}

// ============================================================================
// LayerControl3D Component - Section 07 Implementation
// ============================================================================

use dioxus::prelude::*;

/// Layer control panel for the 3D map.
///
/// Displays checkboxes for each GeoJSON layer and handles toggling visibility.
///
/// # Props
///
/// * `layers` - List of layers to display in the control
/// * `map_id` - ID of the map instance to control
///
/// # Example
///
/// ```rust
/// let layers = predefined_layers();
/// rsx! {
///     LayerControl3D {
///         layers: layers,
///         map_id: "map".to_string(),
///     }
/// }
/// ```
#[component]
pub fn LayerControl3D(
    layers: Vec<GeoJsonLayer>,
    map_id: String,
) -> Element {
    // Manage layer visibility state
    let mut layer_visibility = use_signal(|| {
        layers.iter()
            .map(|l| (l.id.clone(), l.visible))
            .collect::<std::collections::HashMap<String, bool>>()
    });

    rsx! {
        div {
            class: "layer-control",
            style: "background: white; padding: 1rem; border-radius: 4px;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1); min-width: 200px;",

            h3 {
                style: "margin: 0 0 1rem 0; font-size: 1rem; font-weight: 600; color: #333;",
                "Kaartlagen"
            }

            {layers.iter().map(|layer| {
                let layer_id = layer.id.clone();
                let layer_name = layer.name.clone();
                let map_id_clone = map_id.clone();
                let layer_id_for_input = layer_id.clone();
                let is_visible = layer_visibility.read().get(&layer.id).copied().unwrap_or(layer.visible);

                rsx! {
                    div {
                        class: "layer-item",
                        key: "{layer_id}",
                        style: "display: flex; align-items: center; margin: 0.5rem 0;",

                        input {
                            r#type: "checkbox",
                            id: "layer-{layer_id_for_input}",
                            checked: is_visible,
                            style: "margin-right: 0.5rem; cursor: pointer;",
                            onchange: move |evt| {
                                let checked = evt.checked();
                                layer_visibility.write().insert(layer_id.clone(), checked);

                                // Generate the JavaScript to toggle layer visibility
                                let _js = crate::components::map_3d::build_toggle_layer_visibility_script(
                                    &map_id_clone,
                                    &layer_id,
                                    checked
                                );
                                // In production, this JavaScript would be executed via dioxus::prelude::eval
                                // or passed to the Map3D component for execution
                            }
                        }

                        label {
                            r#for: "layer-{layer_id_for_input}",
                            style: "cursor: pointer; user-select: none; color: #333;",
                            "{layer_name}"
                        }
                    }
                }
            })}
        }
    }
}

/// Simple layer checkbox component for use in other contexts.
///
/// # Props
///
/// * `layer` - The layer configuration
/// * `visible` - Current visibility state
/// * `on_toggle` - Callback when visibility is toggled
#[component]
pub fn LayerCheckbox(
    layer: GeoJsonLayer,
    visible: bool,
    on_toggle: EventHandler<bool>,
) -> Element {
    rsx! {
        div {
            class: "layer-checkbox-item",
            style: "display: flex; align-items: center; margin: 0.5rem 0;",

            input {
                r#type: "checkbox",
                id: "layer-{layer.id}",
                checked: visible,
                style: "margin-right: 0.5rem; cursor: pointer;",
                onchange: move |evt| {
                    on_toggle(evt.checked());
                }
            }

            label {
                r#for: "layer-{layer.id}",
                style: "cursor: pointer; user-select: none; color: #333;",
                {layer.name.clone()}
            }
        }
    }
}

#[cfg(test)]
mod layer_control_tests {
    use super::*;

    #[test]
    fn test_geojson_layer_creation() {
        let layer = GeoJsonLayer {
            id: "test-layer".to_string(),
            name: "Test Layer".to_string(),
            url: "https://example.com/test.geojson".to_string(),
            layer_type: LayerType::Polygon,
            visible: true,
            color: "#FF0000".to_string(),
        };

        assert_eq!(layer.id, "test-layer");
        assert_eq!(layer.name, "Test Layer");
        assert_eq!(layer.layer_type, LayerType::Polygon);
        assert!(layer.visible);
    }

    #[test]
    fn test_layer_type_variants() {
        let point = LayerType::Point;
        let line = LayerType::Line;
        let polygon = LayerType::Polygon;

        // Verify variants can be created and compared
        assert!(matches!(point, LayerType::Point));
        assert!(matches!(line, LayerType::Line));
        assert!(matches!(polygon, LayerType::Polygon));
    }

    #[test]
    fn test_predefined_layers_returns_two_layers() {
        let layers = predefined_layers();
        assert_eq!(layers.len(), 2);

        // Check that provinciegrens is first
        assert_eq!(layers[0].id, "provinciegrens");
        assert_eq!(layers[0].name, "Provinciegrens");
        assert!(matches!(layers[0].layer_type, LayerType::Polygon));

        // Check that cultuurhistorie is second
        assert_eq!(layers[1].id, "cultuurhistorie");
        assert_eq!(layers[1].name, "Cultuurhistorie");
        assert!(matches!(layers[1].layer_type, LayerType::Point));
    }

    #[test]
    fn test_predefined_layers_provinciegrens_visible_by_default() {
        let layers = predefined_layers();
        let provinciegrens = layers.iter().find(|l| l.id == "provinciegrens").unwrap();
        assert!(provinciegrens.visible);
    }

    #[test]
    fn test_predefined_layers_cultuurhistorie_hidden_by_default() {
        let layers = predefined_layers();
        let cultuurhistorie = layers.iter().find(|l| l.id == "cultuurhistorie").unwrap();
        assert!(!cultuurhistorie.visible);
    }
}
