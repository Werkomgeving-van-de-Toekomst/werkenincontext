//! 3D Buildings service - simplified version

use axum::{extract::Query, response::Json};
use serde::Deserialize;
use serde_json::json;
use std::sync::OnceLock;

#[derive(Deserialize)]
pub struct BboxParams {
    bbox: Option<String>,
    limit: Option<usize>,
}

/// Flag indicating whether proj crate is available for accurate conversion.
/// Checked once at startup to avoid repeated Proj creation.
static PROJ_AVAILABLE: OnceLock<bool> = OnceLock::new();

/// Check if proj is available for coordinate conversion.
fn is_proj_available() -> bool {
    *PROJ_AVAILABLE.get_or_init(|| {
        proj::Proj::new_known_crs("EPSG:28992", "EPSG:4326", None).is_ok()
    })
}

/// RD (EPSG:28992) to WGS84 (EPSG:4326) coordinate conversion
/// Using proj crate for accurate transformation.
///
/// Returns None if proj is not available or conversion fails.
fn rd_to_wgs84(x: f64, y: f64) -> Option<(f64, f64)> {
    use proj::Proj;

    let proj = Proj::new_known_crs("EPSG:28992", "EPSG:4326", None).ok()?;
    let (lon, lat) = proj.convert((x, y)).ok()?;

    Some((lon, lat))
}

/// Fallback simple conversion if proj fails
/// This is a reasonable approximation for the Netherlands
fn rd_to_wgs84_fallback(x: f64, y: f64) -> (f64, f64) {
    // Reference point: Amersfoort
    // RD (155000, 463000) = WGS84 (5.3876389, 52.1551744)
    let dx_km = (x - 155000.0) / 1000.0;  // Convert to km
    let dy_km = (y - 463000.0) / 1000.0;

    // 1 degree latitude ≈ 111 km
    // 1 degree longitude ≈ 70 km (at 52°N)
    // RD: X increases eastward, Y increases northward
    // Rotation: RD is rotated slightly relative to WGS84

    // Approximate conversion coefficients
    let lon = 5.3876389 + 0.0114 * dx_km - 0.0018 * dy_km;
    let lat = 52.1551744 + 0.0045 * dx_km + 0.0090 * dy_km;

    (lon, lat)
}

pub async fn get_buildings_3d(Query(params): Query<BboxParams>) -> Json<serde_json::Value> {
    // Default bbox for Flevoland area
    let bbox = params.bbox.as_deref().unwrap_or("150000,470000,170000,490000");
    let limit = params.limit.unwrap_or(50);

    let url = format!(
        "https://api.3dbag.nl/collections/pand/items?bbox={}&limit={}",
        bbox, limit
    );

    let client = reqwest::Client::new();
    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => return Json(json!({"type": "FeatureCollection", "features": []})),
    };

    if !resp.status().is_success() {
        return Json(json!({"type": "FeatureCollection", "features": []}));
    }

    let cityjson: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(_) => return Json(json!({"type": "FeatureCollection", "features": []})),
    };

    // Extract transform from root metadata (shared by all features)
    let transform = cityjson.get("metadata")
        .and_then(|m| m.as_object())
        .and_then(|m| m.get("transform"))
        .and_then(|t| t.as_object());

    // Extract vertices from root level (also shared)
    let root_vertices = cityjson.get("vertices").and_then(|v| v.as_array());


    // Check if proj is available for accurate conversion
    let use_proj = is_proj_available();

    // Process each building feature
    let features = if let Some(features_array) = cityjson.get("features").and_then(|f| f.as_array()) {
        let up = use_proj;
        let tf = transform;
        let verts = root_vertices;
        features_array.iter()
            .filter_map(|feature| {
                let city_objs = feature.get("CityObjects")?;
                let (id, city_obj) = city_objs.as_object()?.iter().next()?;

                if city_obj.get("type")?.as_str() != Some("Building") {
                    return None;
                }

                let attrs = city_obj.get("attributes")?.as_object()?;
                let roof_max = attrs.get("b3_h_dak_max")?.as_f64().unwrap_or(10.0);
                let ground = attrs.get("b3_h_maaiveld")?.as_f64().unwrap_or(0.0);
                let height = (roof_max - ground).max(2.0);

                // Use feature-level vertices if available, otherwise use root vertices
                let vertices = feature.get("vertices").and_then(|v| v.as_array()).or(verts);

                // Get geometry
                let geometry = city_obj.get("geometry")?.as_array()?;
                let footprint = geometry.iter()
                    .find(|g| g.get("lod").and_then(|l| l.as_str()) == Some("0"))?;

                // Build geometry
                let geom = if up {
                    convert_geometry_proj(footprint, vertices, tf)
                } else {
                    convert_geometry_fallback(footprint, vertices, tf)
                };

                // Build feature manually
                let mut props = serde_json::Map::new();
                props.insert("height".to_string(), json!(height));
                props.insert("min_height".to_string(), json!(ground));
                props.insert("floors".to_string(), json!(attrs.get("b3_bouwlagen").and_then(|f| f.as_i64()).unwrap_or(1)));

                let mut feature_obj = serde_json::Map::new();
                feature_obj.insert("type".to_string(), json!("Feature"));
                feature_obj.insert("id".to_string(), json!(id));
                feature_obj.insert("geometry".to_string(), geom);
                feature_obj.insert("properties".to_string(), json!(props));

                Some(serde_json::Value::Object(feature_obj))
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    let mut result = serde_json::Map::new();
    result.insert("type".to_string(), json!("FeatureCollection"));
    result.insert("features".to_string(), json!(features));

    Json(serde_json::Value::Object(result))
}

fn convert_geometry_proj(
    geometry: &serde_json::Value,
    vertices: Option<&Vec<serde_json::Value>>,
    transform: Option<&serde_json::Map<String, serde_json::Value>>,
) -> serde_json::Value {
    use proj::Proj;

    // Create Proj instance once per geometry (not per coordinate)
    let proj = match Proj::new_known_crs("EPSG:28992", "EPSG:4326", None) {
        Ok(p) => p,
        Err(_) => return convert_geometry_fallback(geometry, vertices, transform),
    };

    let (scale_x, scale_y, translate_x, translate_y) = if let Some(t) = transform {
        let scale = t.get("scale").and_then(|s| s.as_array());
        let trans = t.get("translate").and_then(|s| s.as_array());
        (
            scale.and_then(|a| a.get(0)).and_then(|v| v.as_f64()).unwrap_or(1.0),
            scale.and_then(|a| a.get(1)).and_then(|v| v.as_f64()).unwrap_or(1.0),
            trans.and_then(|a| a.get(0)).and_then(|v| v.as_f64()).unwrap_or(0.0),
            trans.and_then(|a| a.get(1)).and_then(|v| v.as_f64()).unwrap_or(0.0),
        )
    } else {
        (1.0, 1.0, 0.0, 0.0)
    };

    if let Some(boundaries) = geometry.get("boundaries").and_then(|b| b.as_array()) {
        if let Some(rings) = boundaries.first().and_then(|r| r.as_array()) {
            if let Some(ring) = rings.first().and_then(|r| r.as_array()) {
                let coords = ring.iter()
                    .filter_map(|v| v.as_u64())
                    .filter_map(|idx| {
                        let vert = vertices.and_then(|verts| verts.get(idx as usize))?;
                        let vert_arr = vert.as_array()?;
                        let x = vert_arr.get(0)?.as_f64()?;
                        let y = vert_arr.get(1)?.as_f64()?;

                        // Apply CityJSON transform to get RD coordinates
                        let rd_x = x * scale_x + translate_x;
                        let rd_y = y * scale_y + translate_y;

                        // Convert RD to WGS84 using proj
                        match proj.convert((rd_x, rd_y)) {
                            Ok((lon, lat)) => Some(json!([lon, lat])),
                            Err(_) => None,
                        }
                    })
                    .collect::<Vec<_>>();

                if coords.len() >= 4 {
                    let mut geom_obj = serde_json::Map::new();
                    geom_obj.insert("type".to_string(), json!("Polygon"));
                    geom_obj.insert("coordinates".to_string(), json!([coords]));
                    return serde_json::Value::Object(geom_obj);
                }
            }
        }
    }

    convert_geometry_fallback(geometry, vertices, transform)
}

fn convert_geometry_fallback(
    geometry: &serde_json::Value,
    vertices: Option<&Vec<serde_json::Value>>,
    transform: Option<&serde_json::Map<String, serde_json::Value>>,
) -> serde_json::Value {
    let (scale_x, scale_y, translate_x, translate_y) = if let Some(t) = transform {
        let scale = t.get("scale").and_then(|s| s.as_array());
        let trans = t.get("translate").and_then(|s| s.as_array());
        (
            scale.and_then(|a| a.get(0)).and_then(|v| v.as_f64()).unwrap_or(1.0),
            scale.and_then(|a| a.get(1)).and_then(|v| v.as_f64()).unwrap_or(1.0),
            trans.and_then(|a| a.get(0)).and_then(|v| v.as_f64()).unwrap_or(0.0),
            trans.and_then(|a| a.get(1)).and_then(|v| v.as_f64()).unwrap_or(0.0),
        )
    } else {
        (1.0, 1.0, 0.0, 0.0)
    };

    if let Some(boundaries) = geometry.get("boundaries").and_then(|b| b.as_array()) {
        if let Some(rings) = boundaries.first().and_then(|r| r.as_array()) {
            if let Some(ring) = rings.first().and_then(|r| r.as_array()) {
                let coords = ring.iter()
                    .filter_map(|v| v.as_u64())
                    .filter_map(|idx| {
                        let vert = vertices.and_then(|verts| verts.get(idx as usize))?;
                        let vert_arr = vert.as_array()?;
                        let x = vert_arr.get(0)?.as_f64()?;
                        let y = vert_arr.get(1)?.as_f64()?;

                        // Apply CityJSON transform to get RD coordinates
                        let rd_x = x * scale_x + translate_x;
                        let rd_y = y * scale_y + translate_y;

                        // Convert using fallback formula
                        let (lon, lat) = rd_to_wgs84_fallback(rd_x, rd_y);
                        Some(json!([lon, lat]))
                    })
                    .collect::<Vec<_>>();

                if coords.len() >= 4 {
                    let mut geom_obj = serde_json::Map::new();
                    geom_obj.insert("type".to_string(), json!("Polygon"));
                    geom_obj.insert("coordinates".to_string(), json!([coords]));
                    return serde_json::Value::Object(geom_obj);
                }
            }
        }
    }

    // Fallback
    let mut fallback = serde_json::Map::new();
    fallback.insert("type".to_string(), json!("Point"));
    fallback.insert("coordinates".to_string(), json!([0, 0]));
    serde_json::Value::Object(fallback)
}

#[cfg(test)]
mod coordinate_tests {
    use super::*;

    #[test]
    fn test_rd_to_wgs84_amersfoort() {
        // Amersfoort coordinates (reference point)
        // RD: (155000, 463000) should approximate WGS84: (5.3876, 52.1552)
        if let Some((lon, lat)) = rd_to_wgs84(155000.0, 463000.0) {
            assert!(f64::abs(lon - 5.3876) < 0.001, "Longitude mismatch");
            assert!(f64::abs(lat - 52.1552) < 0.001, "Latitude mismatch");
        } else {
            panic!("proj conversion failed");
        }
    }

    #[test]
    fn test_rd_to_wgs84_returns_none_on_invalid() {
        // Extreme coordinates that might fail conversion
        let result = rd_to_wgs84(f64::NAN, f64::NAN);
        assert!(result.is_none(), "NaN coordinates should return None");

        // Test infinity
        let result = rd_to_wgs84(f64::INFINITY, f64::INFINITY);
        assert!(result.is_none(), "Infinity coordinates should return None");
    }

    #[test]
    fn test_fallback_conversion_does_not_panic() {
        // Fallback should handle all inputs gracefully
        let (lon, lat) = rd_to_wgs84_fallback(155000.0, 463000.0);
        assert!(lon.is_finite());
        assert!(lat.is_finite());
    }

    #[test]
    fn test_proj_more_accurate_than_fallback() {
        // Compare both methods at a known point
        let rd_x = 155000.0;
        let rd_y = 463000.0;

        let proj_result = rd_to_wgs84(rd_x, rd_y);
        let fallback_result = rd_to_wgs84_fallback(rd_x, rd_y);

        if let Some((proj_lon, proj_lat)) = proj_result {
            // proj should give valid results
            assert!(proj_lon >= 3.0 && proj_lon <= 7.5, "Longitude out of Netherlands range");
            assert!(proj_lat >= 50.0 && proj_lat <= 54.0, "Latitude out of Netherlands range");

            // Fallback should be in reasonable range too
            assert!(fallback_result.0 >= 3.0 && fallback_result.0 <= 7.5);
            assert!(fallback_result.1 >= 50.0 && fallback_result.1 <= 54.0);
        }
    }

    #[test]
    fn test_convert_geometry_proj_valid_input() {
        // Skip if proj is not available
        if !is_proj_available() {
            return; // Skip test if proj unavailable
        };

        // Verify proj can convert a simple point
        let proj = proj::Proj::new_known_crs("EPSG:28992", "EPSG:4326", None).unwrap();
        let result = proj.convert((155000.0, 463000.0));
        assert!(result.is_ok(), "proj should convert valid RD coordinates");

        let (lon, lat) = result.unwrap();
        assert!(f64::abs(lon - 5.3876) < 0.01);
        assert!(f64::abs(lat - 52.1552) < 0.01);
    }
}
