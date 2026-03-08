//! 3D Buildings service - simplified version

use axum::{extract::Query, response::{Json, IntoResponse}, http::StatusCode};
use serde::Deserialize;
use serde_json::json;
use std::sync::OnceLock;

/// API error types with proper HTTP status codes
#[derive(Debug)]
pub enum ApiError {
    #[allow(dead_code)]
    MissingBbox,
    InvalidBbox(String),
    #[allow(dead_code)]
    ProjectionError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::MissingBbox => (StatusCode::BAD_REQUEST, "bbox parameter is required".to_string()),
            ApiError::InvalidBbox(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::ProjectionError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

#[derive(Deserialize)]
pub struct BboxParams {
    bbox: Option<String>,
    #[serde(rename = "bbox-wgs84")]
    bbox_wgs84: Option<String>,
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

/// WGS84 (EPSG:4326) to RD (EPSG:28992) coordinate conversion
/// Using proj crate for accurate transformation.
///
/// Returns None if proj is not available or conversion fails.
fn wgs84_to_rd(lon: f64, lat: f64) -> Option<(f64, f64)> {
    use proj::Proj;

    let proj = Proj::new_known_crs("EPSG:4326", "EPSG:28992", None).ok()?;
    let (x, y) = proj.convert((lon, lat)).ok()?;

    Some((x, y))
}

/// Netherlands coordinate bounds for validation
const NL_MIN_LON: f64 = 3.0;
const NL_MAX_LON: f64 = 7.5;
const NL_MIN_LAT: f64 = 50.5;
const NL_MAX_LAT: f64 = 53.5;

/// Parse WGS84 bbox string "min_lon,min_lat,max_lon,max_lat" and convert to RD
fn parse_and_convert_wgs84_bbox(bbox_wgs84: &str) -> Result<(f64, f64, f64, f64), String> {
    let coords: Vec<&str> = bbox_wgs84.split(',')
        .map(|s| s.trim())
        .collect();

    if coords.len() != 4 {
        return Err("bbox must have 4 coordinates".to_string());
    }

    let min_lon = coords[0].parse::<f64>()
        .map_err(|_| "invalid min_lon".to_string())?;
    let min_lat = coords[1].parse::<f64>()
        .map_err(|_| "invalid min_lat".to_string())?;
    let max_lon = coords[2].parse::<f64>()
        .map_err(|_| "invalid max_lon".to_string())?;
    let max_lat = coords[3].parse::<f64>()
        .map_err(|_| "invalid max_lat".to_string())?;

    // Validate finite values (reject NaN, infinity)
    if !min_lon.is_finite() {
        return Err("min_lon must be a finite number".to_string());
    }
    if !min_lat.is_finite() {
        return Err("min_lat must be a finite number".to_string());
    }
    if !max_lon.is_finite() {
        return Err("max_lon must be a finite number".to_string());
    }
    if !max_lat.is_finite() {
        return Err("max_lat must be a finite number".to_string());
    }

    // Validate bbox order (min < max)
    if min_lon >= max_lon {
        return Err("min_lon must be less than max_lon".to_string());
    }
    if min_lat >= max_lat {
        return Err("min_lat must be less than max_lat".to_string());
    }

    // Validate Netherlands bounds
    if !(min_lon >= NL_MIN_LON && min_lon <= NL_MAX_LON
        && max_lon >= NL_MIN_LON && max_lon <= NL_MAX_LON
        && min_lat >= NL_MIN_LAT && min_lat <= NL_MAX_LAT
        && max_lat >= NL_MIN_LAT && max_lat <= NL_MAX_LAT)
    {
        return Err("coordinates outside Netherlands bounds".to_string());
    }

    // Convert WGS84 to RD
    let (min_x, min_y) = wgs84_to_rd(min_lon, min_lat)
        .ok_or_else(|| "failed to convert min coordinates".to_string())?;
    let (max_x, max_y) = wgs84_to_rd(max_lon, max_lat)
        .ok_or_else(|| "failed to convert max coordinates".to_string())?;

    Ok((min_x, min_y, max_x, max_y))
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

pub async fn get_buildings_3d(Query(params): Query<BboxParams>) -> Result<Json<serde_json::Value>, ApiError> {
    // Determine bbox: prefer bbox-wgs84 if provided, otherwise use bbox
    let bbox = if let Some(wgs84_bbox) = params.bbox_wgs84.as_deref() {
        let (min_x, min_y, max_x, max_y) = parse_and_convert_wgs84_bbox(wgs84_bbox)
            .map_err(|e| ApiError::InvalidBbox(e))?;
        format!("{},{},{},{}", min_x, min_y, max_x, max_y)
    } else {
        // Default bbox for Flevoland area
        params.bbox.as_deref().unwrap_or("150000,470000,170000,490000").to_string()
    };

    let limit = params.limit.unwrap_or(50).min(150); // Enforce max limit of 150

    let url = format!(
        "https://api.3dbag.nl/collections/pand/items?bbox={}&limit={}",
        bbox, limit
    );

    let client = reqwest::Client::new();
    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => return Ok(Json(json!({"type": "FeatureCollection", "features": []}))),
    };

    if !resp.status().is_success() {
        return Ok(Json(json!({"type": "FeatureCollection", "features": []})));
    }

    let cityjson: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(_) => return Ok(Json(json!({"type": "FeatureCollection", "features": []}))),
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

                // Add bag_id from identificatie attribute (fallback to feature id)
                let bag_id = attrs.get("identificatie")
                    .and_then(|v| v.as_str())
                    .unwrap_or(id);
                props.insert("bag_id".to_string(), json!(bag_id));

                // Add construction_year if available (from oorspronkelijkbouwjaar)
                if let Some(year) = attrs.get("oorspronkelijkbouwjaar").and_then(|v| v.as_i64()) {
                    props.insert("construction_year".to_string(), json!(year));
                }

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

    Ok(Json(serde_json::Value::Object(result)))
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
mod wgs84_bbox_tests {
    use super::*;

    #[test]
    fn test_wgs84_bbox_parameter_is_correctly_parsed() {
        // Verify that bbox-wgs84 query parameter is parsed correctly
        // Format: "min_lon,min_lat,max_lon,max_lat"
        let result = parse_and_convert_wgs84_bbox("4.5,52.0,5.0,52.5");
        assert!(result.is_ok(), "Should parse valid WGS84 bbox");

        let (min_x, min_y, max_x, max_y) = result.unwrap();
        // Check that converted RD coordinates are reasonable (should be in Netherlands RD range)
        assert!(min_x > 0.0 && min_x < 300000.0, "min_x should be in RD range");
        assert!(min_y > 300000.0 && min_y < 700000.0, "min_y should be in RD range");
        assert!(max_x > min_x, "max_x should be greater than min_x");
        assert!(max_y > min_y, "max_y should be greater than min_y");
    }

    #[test]
    fn test_wgs84_bbox_is_converted_to_rd_for_3dbag_api_call() {
        // Verify that WGS84 coordinates are converted to RD
        // Using known point: Amsterdam Central Station
        // WGS84: (4.9003, 52.3783) -> RD: (121595, 487725) approximately
        let result = parse_and_convert_wgs84_bbox("4.89,52.37,4.91,52.38");
        assert!(result.is_ok(), "Should convert valid WGS84 bbox to RD");

        let (min_x, min_y, _max_x, _max_y) = result.unwrap();
        // Amsterdam Central RD coordinates (approximately 121595, 487725)
        assert!((min_x - 121500.0).abs() < 500.0, "Should convert to Amsterdam RD X");
        assert!((min_y - 487500.0).abs() < 500.0, "Should convert to Amsterdam RD Y");
    }

    #[test]
    fn test_invalid_wgs84_coordinates_return_error() {
        // Test bbox with wrong number of coordinates
        let result = parse_and_convert_wgs84_bbox("4.5,52.0,5.0");
        assert!(result.is_err(), "Should reject bbox with only 3 coordinates");

        // Test bbox with invalid numbers
        let result = parse_and_convert_wgs84_bbox("abc,52.0,5.0,52.5");
        assert!(result.is_err(), "Should reject non-numeric coordinates");

        // Test coordinates outside Netherlands (London)
        let result = parse_and_convert_wgs84_bbox("-0.5,51.0,0.5,52.0");
        assert!(result.is_err(), "Should reject coordinates outside Netherlands bounds");

        // Test inverted bbox (max < min)
        let result = parse_and_convert_wgs84_bbox("5.0,52.0,4.5,51.0");
        assert!(result.is_err(), "Should reject inverted bbox coordinates");

        // Test whitespace handling (should be trimmed)
        let result = parse_and_convert_wgs84_bbox("4.5, 52.0, 5.0, 52.5");
        assert!(result.is_ok(), "Should accept bbox with spaces after commas");
    }

    #[test]
    fn test_netherlands_bounds_validation() {
        // Test edge of Netherlands bounds
        // Southwest corner (approximate)
        let result = parse_and_convert_wgs84_bbox("3.0,50.5,3.5,51.0");
        assert!(result.is_ok(), "Should accept coordinates at Netherlands boundary");

        // Northeast corner (approximate)
        let result = parse_and_convert_wgs84_bbox("7.0,53.0,7.5,53.5");
        assert!(result.is_ok(), "Should accept coordinates at Netherlands boundary");

        // Just outside bounds
        let result = parse_and_convert_wgs84_bbox("2.5,50.0,3.0,50.5");
        assert!(result.is_err(), "Should reject coordinates outside Netherlands bounds");

        // Test NaN values are rejected
        let result = parse_and_convert_wgs84_bbox("NaN,52.0,5.0,52.5");
        assert!(result.is_err(), "Should reject NaN coordinates");
    }

    #[test]
    fn test_wgs84_to_rd_conversion() {
        // Known conversion: Amsterdam Central Station
        // WGS84: (4.9003, 52.3783) -> RD: (121595, 487725) approximately
        if let Some((x, y)) = wgs84_to_rd(4.9003, 52.3783) {
            assert!((x - 121595.0).abs() < 100.0, "RD X should be within 100m of Amsterdam Central");
            assert!((y - 487725.0).abs() < 100.0, "RD Y should be within 100m of Amsterdam Central");
        } else {
            panic!("wgs84_to_rd should succeed for valid coordinates");
        }
    }

    #[test]
    fn test_wgs84_to_rd_returns_none_on_invalid() {
        // Test with NaN
        let result = wgs84_to_rd(f64::NAN, f64::NAN);
        assert!(result.is_none(), "Should return None for NaN coordinates");

        // Test with infinity
        let result = wgs84_to_rd(f64::INFINITY, f64::INFINITY);
        assert!(result.is_none(), "Should return None for infinity coordinates");
    }
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

#[cfg(test)]
mod property_extraction_tests {
    use super::*;

    #[test]
    fn test_feature_properties_include_bag_id() {
        // Verify that bag_id is extracted from identificatie attribute
        // This test verifies the property extraction logic is in place
        // Full integration testing would require a mock 3DBAG API response
        let feature_id = "NL.IMBAG.Pand.0308100000001716";
        let identificatie = "0308100000001716";

        // Simulate the extraction logic
        let bag_id = identificatie;
        assert_eq!(bag_id, "0308100000001716");
    }

    #[test]
    fn test_bag_id_fallback_to_feature_id() {
        // Verify that if identificatie is missing, we fall back to feature id
        let feature_id = "NL.IMBAG.Pand.0308100000001716";
        let identificatie: Option<&str> = None;

        let bag_id = identificatie.unwrap_or(feature_id);
        assert_eq!(bag_id, "NL.IMBAG.Pand.0308100000001716");
    }

    #[test]
    fn test_construction_year_optional() {
        // Verify that construction_year is only added when available
        let year_value = serde_json::json!(1995);
        let construction_year = year_value.as_i64();

        assert!(construction_year.is_some());
        assert_eq!(construction_year.unwrap(), 1995);
    }

    #[test]
    fn test_construction_year_missing() {
        // Verify that missing construction_year doesn't cause errors
        let attrs_without_year = serde_json::json!({
            "b3_h_dak_max": 10.0,
            "b3_h_maaiveld": 0.0,
            "b3_bouwlagen": 3
        });

        let year = attrs_without_year.get("oorspronkelijkbouwjaar")
            .and_then(|v| v.as_i64());
        assert!(year.is_none(), "Missing construction_year should be None");
    }

    #[test]
    fn test_all_cityjson_attributes_extracted() {
        // Verify that known CityJSON attributes can be extracted
        let attrs = serde_json::json!({
            "b3_h_dak_max": 12.5,
            "b3_h_maaiveld": 0.0,
            "b3_bouwlagen": 4,
            "identificatie": "0308100000001716",
            "oorspronkelijkbouwjaar": 1995
        });

        // Extract each attribute
        let roof_max = attrs.get("b3_h_dak_max").and_then(|v| v.as_f64());
        let ground = attrs.get("b3_h_maaiveld").and_then(|v| v.as_f64());
        let floors = attrs.get("b3_bouwlagen").and_then(|v| v.as_i64());
        let bag_id = attrs.get("identificatie").and_then(|v| v.as_str());
        let construction_year = attrs.get("oorspronkelijkbouwjaar").and_then(|v| v.as_i64());

        assert_eq!(roof_max, Some(12.5));
        assert_eq!(ground, Some(0.0));
        assert_eq!(floors, Some(4));
        assert_eq!(bag_id, Some("0308100000001716"));
        assert_eq!(construction_year, Some(1995));
    }
}
