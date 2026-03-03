//! Layer Detection - Detect layer type from GeoJSON content
//!
//! This module provides functionality to analyze GeoJSON content
//! and determine the primary geometry type for proper rendering.

use crate::components::layer_control_3d::LayerType;
use serde_json::Value;

/// Detects the primary geometry type from GeoJSON content.
///
/// Analyzes the GeoJSON and returns the first geometry type found.
/// For mixed or empty content, returns Polygon as a sensible default.
///
/// # Arguments
///
/// * `geojson_content` - The GeoJSON content as a string
///
/// # Returns
///
/// The detected `LayerType` (Point, Line, or Polygon)
///
/// # Example
///
/// ```rust
/// let geojson = r#"{
///     "type": "FeatureCollection",
///     "features": [{"type": "Feature", "geometry": {"type": "Point", "coordinates": [0, 0]}}]
/// }"#;
/// assert_eq!(detect_layer_type(geojson), LayerType::Point);
/// ```
pub fn detect_layer_type(geojson_content: &str) -> LayerType {
    let json: Value = match serde_json::from_str(geojson_content) {
        Ok(v) => v,
        Err(_) => return LayerType::Polygon, // Default for invalid JSON
    };

    // Navigate to features array
    let features = json
        .get("features")
        .and_then(|f| f.as_array());

    let Some(features) = features else {
        return LayerType::Polygon;
    };

    if features.is_empty() {
        return LayerType::Polygon;
    }

    // Check first feature's geometry type
    let first_geometry = features
        .first()
        .and_then(|f| f.get("geometry"))
        .and_then(|g| g.get("type"))
        .and_then(|t| t.as_str());

    match first_geometry {
        Some("Point") | Some("MultiPoint") => LayerType::Point,
        Some("LineString") | Some("MultiLineString") => LayerType::Line,
        Some("Polygon") | Some("MultiPolygon") => LayerType::Polygon,
        _ => LayerType::Polygon,
    }
}

/// Checks if GeoJSON content contains mixed geometry types.
///
/// Returns true if the FeatureCollection contains features with
/// different geometry types.
///
/// # Arguments
///
/// * `geojson_content` - The GeoJSON content as a string
///
/// # Returns
///
/// true if mixed geometries are detected, false otherwise
pub fn has_mixed_geometries(geojson_content: &str) -> bool {
    let json: Value = match serde_json::from_str(geojson_content) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let features = json
        .get("features")
        .and_then(|f| f.as_array());

    let Some(features) = features else {
        return false;
    };

    if features.len() < 2 {
        return false;
    }

    // Get the first geometry type
    let first_type = features
        .first()
        .and_then(|f| f.get("geometry"))
        .and_then(|g| g.get("type"))
        .and_then(|t| t.as_str());

    let Some(first_type) = first_type else {
        return false;
    };

    // Check if any feature has a different geometry type
    for feature in features.iter().skip(1) {
        let geom_type = feature
            .get("geometry")
            .and_then(|g| g.get("type"))
            .and_then(|t| t.as_str());

        if let Some(gt) = geom_type {
            if gt != first_type {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_point_layer() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [{"type": "Feature", "geometry": {"type": "Point", "coordinates": [0, 0]}}]
        }"#;
        assert_eq!(detect_layer_type(geojson), LayerType::Point);
    }

    #[test]
    fn test_detect_multipoint_layer() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [{"type": "Feature", "geometry": {"type": "MultiPoint", "coordinates": [[0, 0], [1, 1]]}}]
        }"#;
        assert_eq!(detect_layer_type(geojson), LayerType::Point);
    }

    #[test]
    fn test_detect_line_layer() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [{"type": "Feature", "geometry": {"type": "LineString", "coordinates": [[0, 0], [1, 1]]}}]
        }"#;
        assert_eq!(detect_layer_type(geojson), LayerType::Line);
    }

    #[test]
    fn test_detect_multilinestring_layer() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [{"type": "Feature", "geometry": {"type": "MultiLineString", "coordinates": [[[0, 0], [1, 1]], [[2, 2], [3, 3]]]}}]
        }"#;
        assert_eq!(detect_layer_type(geojson), LayerType::Line);
    }

    #[test]
    fn test_detect_polygon_layer() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [{"type": "Feature", "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1], [0, 1], [0, 0]]]}}]
        }"#;
        assert_eq!(detect_layer_type(geojson), LayerType::Polygon);
    }

    #[test]
    fn test_detect_multipolygon_layer() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [{"type": "Feature", "geometry": {"type": "MultiPolygon", "coordinates": [[[[0, 0], [1, 0], [1, 1], [0, 1], [0, 0]]]]}}]
        }"#;
        assert_eq!(detect_layer_type(geojson), LayerType::Polygon);
    }

    #[test]
    fn test_detect_handles_empty_geojson() {
        let geojson = r#"{"type": "FeatureCollection", "features": []}"#;
        assert_eq!(detect_layer_type(geojson), LayerType::Polygon);
    }

    #[test]
    fn test_detect_handles_invalid_json() {
        let geojson = r#"not valid json"#;
        assert_eq!(detect_layer_type(geojson), LayerType::Polygon);
    }

    #[test]
    fn test_detect_mixed_geometries_returns_first_type() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [
                {"type": "Feature", "geometry": {"type": "Point", "coordinates": [0, 0]}},
                {"type": "Feature", "geometry": {"type": "LineString", "coordinates": [[0, 0], [1, 1]]}}
            ]
        }"#;
        // Returns the first type detected
        assert_eq!(detect_layer_type(geojson), LayerType::Point);
    }

    #[test]
    fn test_detect_handles_no_features_field() {
        let geojson = r#"{"type": "FeatureCollection"}"#;
        assert_eq!(detect_layer_type(geojson), LayerType::Polygon);
    }

    #[test]
    fn test_detect_handles_null_geometry() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [{"type": "Feature", "geometry": null}]
        }"#;
        assert_eq!(detect_layer_type(geojson), LayerType::Polygon);
    }

    #[test]
    fn test_has_mixed_geometries_with_same_type() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [
                {"type": "Feature", "geometry": {"type": "Point", "coordinates": [0, 0]}},
                {"type": "Feature", "geometry": {"type": "Point", "coordinates": [1, 1]}}
            ]
        }"#;
        assert!(!has_mixed_geometries(geojson));
    }

    #[test]
    fn test_has_mixed_geometries_with_different_types() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [
                {"type": "Feature", "geometry": {"type": "Point", "coordinates": [0, 0]}},
                {"type": "Feature", "geometry": {"type": "LineString", "coordinates": [[0, 0], [1, 1]]}}
            ]
        }"#;
        assert!(has_mixed_geometries(geojson));
    }

    #[test]
    fn test_has_mixed_geometries_with_single_feature() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [{"type": "Feature", "geometry": {"type": "Point", "coordinates": [0, 0]}}]
        }"#;
        assert!(!has_mixed_geometries(geojson));
    }

    #[test]
    fn test_has_mixed_geometries_with_empty_features() {
        let geojson = r#"{"type": "FeatureCollection", "features": []}"#;
        assert!(!has_mixed_geometries(geojson));
    }

    #[test]
    fn test_has_mixed_geometries_with_invalid_json() {
        let geojson = r#"not valid json"#;
        assert!(!has_mixed_geometries(geojson));
    }
}
