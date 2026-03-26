//! LayerControl3D Component - Layer toggle control for Map3D
//!
//! This component provides a UI for toggling GeoJSON layer visibility
//! on the 3D map. Each layer has a checkbox and styled label.

use serde::{Deserialize, Serialize};

/// Type of geometry in a GeoJSON layer.
///
/// Determines how the layer is rendered in MapLibre:
/// - `Point`: Rendered as circles
/// - `Line`: Rendered as lines
/// - `Polygon`: Rendered as filled areas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Map view settings (EduGIS-style catalog, 2D: pitch 0).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlevolandMapSettings {
    pub center: [f64; 2],
    pub zoom: f64,
    pub pitch: f64,
    pub bearing: f64,
}

/// Eén item in de kaartcatalogus: GeoJSON (eigen data) of raster (EduGIS/PDOK-achtige tegels).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum CatalogLayer {
    Geojson {
        id: String,
        name: String,
        url: String,
        layer_type: LayerType,
        visible: bool,
        color: String,
        #[serde(default)]
        group: Option<String>,
    },
    Raster {
        id: String,
        name: String,
        visible: bool,
        tiles: Vec<String>,
        attribution: String,
        #[serde(default)]
        minzoom: Option<u8>,
        #[serde(default)]
        maxzoom: Option<u8>,
        #[serde(default)]
        bounds: Option<[f64; 4]>,
        #[serde(default)]
        group: Option<String>,
        /// Alleen één laag tegelijk zichtbaar binnen deze groep (zoals EduGIS-achtergronden).
        #[serde(default)]
        exclusive_group: Option<String>,
    },
}

impl CatalogLayer {
    pub fn id(&self) -> &str {
        match self {
            CatalogLayer::Geojson { id, .. } | CatalogLayer::Raster { id, .. } => id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            CatalogLayer::Geojson { name, .. } | CatalogLayer::Raster { name, .. } => name,
        }
    }

    pub fn visible(&self) -> bool {
        match self {
            CatalogLayer::Geojson { visible, .. } | CatalogLayer::Raster { visible, .. } => *visible,
        }
    }

    pub fn group_label(&self) -> &str {
        match self {
            CatalogLayer::Geojson { group, .. } => group.as_deref().unwrap_or("Kaartdata"),
            CatalogLayer::Raster { group, .. } => group.as_deref().unwrap_or("Achtergrond"),
        }
    }

    pub fn exclusive_group(&self) -> Option<&str> {
        match self {
            CatalogLayer::Geojson { .. } => None,
            CatalogLayer::Raster {
                exclusive_group, ..
            } => exclusive_group.as_deref(),
        }
    }
}

/// Full layer catalog for the Data Verkenner (`assets/geodata/flevoland_layers.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlevolandLayersConfig {
    pub map: FlevolandMapSettings,
    pub style_url: String,
    pub layers: Vec<CatalogLayer>,
}

/// Loads the Flevoland layer catalog bundled at build time.
pub fn load_flevoland_layers_config() -> FlevolandLayersConfig {
    const JSON: &str = include_str!("../../assets/geodata/flevoland_layers.json");
    serde_json::from_str(JSON).expect("assets/geodata/flevoland_layers.json must be valid JSON")
}

/// Volledige catalogus (raster + GeoJSON) voor het laagpaneel.
pub fn catalog_layers() -> Vec<CatalogLayer> {
    load_flevoland_layers_config().layers
}

/// Alleen GeoJSON-lagen (bv. voor legacy-tests).
pub fn predefined_layers() -> Vec<GeoJsonLayer> {
    catalog_layers()
        .into_iter()
        .filter_map(|l| match l {
            CatalogLayer::Geojson {
                id,
                name,
                url,
                layer_type,
                visible,
                color,
                ..
            } => Some(GeoJsonLayer {
                id,
                name,
                url,
                layer_type,
                visible,
                color,
            }),
            CatalogLayer::Raster { .. } => None,
        })
        .collect()
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

/// Groepeert cataloguslagen op het veld `group` (volgorde: eerste voorkomen).
fn group_catalog_layers(layers: &[CatalogLayer]) -> Vec<(String, Vec<CatalogLayer>)> {
    let mut out: Vec<(String, Vec<CatalogLayer>)> = Vec::new();
    for layer in layers {
        let key = layer.group_label().to_string();
        if let Some(i) = out.iter().position(|(k, _)| k == &key) {
            out[i].1.push(layer.clone());
        } else {
            out.push((key, vec![layer.clone()]));
        }
    }
    out
}

/// Layer control panel for the 3D map.
///
/// Displays checkboxes for raster- en GeoJSON-lagen, gegroepeerd.
///
/// # Props
///
/// * `layers` - Catalogus (zie `catalog_layers()`)
/// * `map_id` - ID of the map instance to control
///
/// # Example
///
/// ```rust
/// let layers = catalog_layers();
/// rsx! {
///     LayerControl3D {
///         layers: layers,
///         map_id: "map".to_string(),
///     }
/// }
/// ```
#[component]
pub fn LayerControl3D(
    layers: Vec<CatalogLayer>,
    map_id: String,
) -> Element {
    let mut layer_visibility = use_signal(|| {
        layers
            .iter()
            .map(|l| (l.id().to_string(), l.visible()))
            .collect::<std::collections::HashMap<String, bool>>()
    });

    let grouped = group_catalog_layers(&layers);
    let layers_for_exclusive = layers.clone();

    rsx! {
        div {
            class: "layer-control",
            style: "background: white; padding: 1rem; border-radius: 4px;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1); min-width: 220px; max-height: 70vh; overflow-y: auto;",

            h3 {
                style: "margin: 0 0 1rem 0; font-size: 1rem; font-weight: 600; color: #333;",
                "Kaartlagen"
            }

            for (group_name, items) in grouped {
                div {
                    key: "{group_name}",
                    h4 {
                        style: "margin: 0.75rem 0 0.4rem 0; font-size: 0.85rem; font-weight: 600; color: #555;",
                        "{group_name}"
                    }
                    {items.into_iter().map(|layer| {
                        let layer_id = layer.id().to_string();
                        let layer_name = layer.name().to_string();
                        let layer_id_for_input = layer_id.clone();
                        let exclusive = match &layer {
                            CatalogLayer::Raster {
                                exclusive_group,
                                ..
                            } => exclusive_group.clone(),
                            CatalogLayer::Geojson { .. } => None,
                        };
                        let map_id_clone = map_id.clone();
                        let layers_ex = layers_for_exclusive.clone();
                        let is_visible = layer_visibility.read().get(&layer_id).copied().unwrap_or(layer.visible());
                        rsx! {
                            div {
                                class: "layer-item",
                                key: "{layer_id}",
                                style: "display: flex; align-items: center; margin: 0.35rem 0;",

                                input {
                                    r#type: "checkbox",
                                    id: "layer-{layer_id_for_input}",
                                    checked: is_visible,
                                    style: "margin-right: 0.5rem; cursor: pointer;",
                                    onchange: move |evt| {
                                        let checked = evt.checked();
                                        if checked {
                                            if let Some(ref g) = exclusive {
                                                for other in layers_ex.iter() {
                                                    if other.id() == layer_id.as_str() {
                                                        continue;
                                                    }
                                                    if other.exclusive_group() == Some(g.as_str()) {
                                                        layer_visibility.write().insert(other.id().to_string(), false);
                                                        let off = crate::components::map_3d::build_toggle_layer_visibility_script(
                                                            &map_id_clone,
                                                            other.id(),
                                                            false,
                                                        );
                                                        let _ = document::eval(&off);
                                                    }
                                                }
                                            }
                                        }
                                        layer_visibility.write().insert(layer_id.clone(), checked);
                                        let js = crate::components::map_3d::build_toggle_layer_visibility_script(
                                            &map_id_clone,
                                            &layer_id,
                                            checked,
                                        );
                                        let _ = document::eval(&js);
                                    }
                                }

                                label {
                                    r#for: "layer-{layer_id_for_input}",
                                    style: "cursor: pointer; user-select: none; color: #333; font-size: 0.9rem;",
                                    "{layer_name}"
                                }
                            }
                        }
                    })}
                }
            }
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
    fn test_predefined_layers_matches_catalog() {
        let layers = predefined_layers();
        assert_eq!(layers.len(), 6);
        assert_eq!(layers[0].id, "provinciegrens");
        assert!(matches!(layers[0].layer_type, LayerType::Polygon));
        let fiets = layers.iter().find(|l| l.id == "fietsnetwerken").unwrap();
        assert!(matches!(fiets.layer_type, LayerType::Line));
    }

    #[test]
    fn test_catalog_layers_includes_raster_and_geojson() {
        let cat = catalog_layers();
        assert_eq!(cat.len(), 13);
        let rasters = cat.iter().filter(|l| matches!(l, CatalogLayer::Raster { .. })).count();
        let gj = cat.iter().filter(|l| matches!(l, CatalogLayer::Geojson { .. })).count();
        assert_eq!(rasters, 7);
        assert_eq!(gj, 6);
        assert!(cat.iter().any(|l| l.id() == "pdok_brt_standaard"));
        assert!(cat.iter().any(|l| l.id() == "edugis_osm"));
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
