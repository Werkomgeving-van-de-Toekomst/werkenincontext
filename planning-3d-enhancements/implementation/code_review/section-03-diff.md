diff --git a/crates/iou-api/src/routes/buildings_3d.rs b/crates/iou-api/src/routes/buildings_3d.rs
index 9b975b2..0057cee 100644
--- a/crates/iou-api/src/routes/buildings_3d.rs
+++ b/crates/iou-api/src/routes/buildings_3d.rs
@@ -8,6 +8,8 @@ use std::sync::OnceLock;
 #[derive(Deserialize)]
 pub struct BboxParams {
     bbox: Option<String>,
+    #[serde(rename = "bbox-wgs84")]
+    bbox_wgs84: Option<String>,
     limit: Option<usize>,
 }
 
@@ -35,6 +37,58 @@ fn rd_to_wgs84(x: f64, y: f64) -> Option<(f64, f64)> {
     Some((lon, lat))
 }
 
+/// WGS84 (EPSG:4326) to RD (EPSG:28992) coordinate conversion
+/// Using proj crate for accurate transformation.
+///
+/// Returns None if proj is not available or conversion fails.
+fn wgs84_to_rd(lon: f64, lat: f64) -> Option<(f64, f64)> {
+    use proj::Proj;
+
+    let proj = Proj::new_known_crs("EPSG:4326", "EPSG:28992", None).ok()?;
+    let (x, y) = proj.convert((lon, lat)).ok()?;
+
+    Some((x, y))
+}
+
+/// Parse WGS84 bbox string "min_lon,min_lat,max_lon,max_lat" and convert to RD
+fn parse_and_convert_wgs84_bbox(bbox_wgs84: &str) -> Result<(f64, f64, f64, f64), String> {
+    let coords: Vec<&str> = bbox_wgs84.split(',').collect();
+    if coords.len() != 4 {
+        return Err("bbox must have 4 coordinates".to_string());
+    }
+
+    let min_lon = coords[0].parse::<f64>()
+        .map_err(|_| "invalid min_lon".to_string())?;
+    let min_lat = coords[1].parse::<f64>()
+        .map_err(|_| "invalid min_lat".to_string())?;
+    let max_lon = coords[2].parse::<f64>()
+        .map_err(|_| "invalid max_lon".to_string())?;
+    let max_lat = coords[3].parse::<f64>()
+        .map_err(|_| "invalid max_lat".to_string())?;
+
+    // Validate Netherlands bounds
+    const NL_MIN_LON: f64 = 3.0;
+    const NL_MAX_LON: f64 = 7.5;
+    const NL_MIN_LAT: f64 = 50.5;
+    const NL_MAX_LAT: f64 = 53.5;
+
+    if !(min_lon >= NL_MIN_LON && min_lon <= NL_MAX_LON
+        && max_lon >= NL_MIN_LON && max_lon <= NL_MAX_LON
+        && min_lat >= NL_MIN_LAT && min_lat <= NL_MAX_LAT
+        && max_lat >= NL_MIN_LAT && max_lat <= NL_MAX_LAT)
+    {
+        return Err("coordinates outside Netherlands bounds".to_string());
+    }
+
+    // Convert WGS84 to RD
+    let (min_x, min_y) = wgs84_to_rd(min_lon, min_lat)
+        .ok_or_else(|| "failed to convert min coordinates".to_string())?;
+    let (max_x, max_y) = wgs84_to_rd(max_lon, max_lat)
+        .ok_or_else(|| "failed to convert max coordinates".to_string())?;
+
+    Ok((min_x, min_y, max_x, max_y))
+}
+
 /// Fallback simple conversion if proj fails
 /// This is a reasonable approximation for the Netherlands
 fn rd_to_wgs84_fallback(x: f64, y: f64) -> (f64, f64) {
@@ -56,9 +110,24 @@ fn rd_to_wgs84_fallback(x: f64, y: f64) -> (f64, f64) {
 }
 
 pub async fn get_buildings_3d(Query(params): Query<BboxParams>) -> Json<serde_json::Value> {
-    // Default bbox for Flevoland area
-    let bbox = params.bbox.as_deref().unwrap_or("150000,470000,170000,490000");
-    let limit = params.limit.unwrap_or(50);
+    // Determine bbox: prefer bbox-wgs84 if provided, otherwise use bbox
+    let bbox = if let Some(wgs84_bbox) = params.bbox_wgs84.as_deref() {
+        match parse_and_convert_wgs84_bbox(wgs84_bbox) {
+            Ok((min_x, min_y, max_x, max_y)) => {
+                format!("{},{},{},{}", min_x, min_y, max_x, max_y)
+            }
+            Err(e) => return Json(json!({
+                "type": "FeatureCollection",
+                "features": [],
+                "error": e
+            })),
+        }
+    } else {
+        // Default bbox for Flevoland area
+        params.bbox.as_deref().unwrap_or("150000,470000,170000,490000").to_string()
+    };
+
+    let limit = params.limit.unwrap_or(50).min(150); // Enforce max limit of 150
 
     let url = format!(
         "https://api.3dbag.nl/collections/pand/items?bbox={}&limit={}",
@@ -271,6 +340,93 @@ fn convert_geometry_fallback(
     serde_json::Value::Object(fallback)
 }
 
+#[cfg(test)]
+mod wgs84_bbox_tests {
+    use super::*;
+
+    #[test]
+    fn test_wgs84_bbox_parameter_is_correctly_parsed() {
+        // Verify that bbox-wgs84 query parameter is parsed correctly
+        // Format: "min_lon,min_lat,max_lon,max_lat"
+        let result = parse_and_convert_wgs84_bbox("4.5,52.0,5.0,52.5");
+        assert!(result.is_ok(), "Should parse valid WGS84 bbox");
+
+        let (min_x, min_y, max_x, max_y) = result.unwrap();
+        // Check that converted RD coordinates are reasonable (should be in Netherlands RD range)
+        assert!(min_x > 0.0 && min_x < 300000.0, "min_x should be in RD range");
+        assert!(min_y > 300000.0 && min_y < 700000.0, "min_y should be in RD range");
+        assert!(max_x > min_x, "max_x should be greater than min_x");
+        assert!(max_y > min_y, "max_y should be greater than min_y");
+    }
+
+    #[test]
+    fn test_wgs84_bbox_is_converted_to_rd_for_3dbag_api_call() {
+        // Verify that WGS84 coordinates are converted to RD
+        // Using known point: Amsterdam Center approximately (4.9, 52.37) in WGS84
+        let result = parse_and_convert_wgs84_bbox("4.8,52.3,5.0,52.4");
+        assert!(result.is_ok(), "Should convert valid WGS84 bbox to RD");
+
+        let (min_x, min_y, max_x, max_y) = result.unwrap();
+        // Amsterdam in RD is approximately (120000, 480000)
+        assert!(min_x > 100000.0 && min_x < 140000.0, "Should convert to Amsterdam RD X range");
+        assert!(min_y > 460000.0 && min_y < 500000.0, "Should convert to Amsterdam RD Y range");
+    }
+
+    #[test]
+    fn test_invalid_wgs84_coordinates_return_error() {
+        // Test bbox with wrong number of coordinates
+        let result = parse_and_convert_wgs84_bbox("4.5,52.0,5.0");
+        assert!(result.is_err(), "Should reject bbox with only 3 coordinates");
+
+        // Test bbox with invalid numbers
+        let result = parse_and_convert_wgs84_bbox("abc,52.0,5.0,52.5");
+        assert!(result.is_err(), "Should reject non-numeric coordinates");
+
+        // Test coordinates outside Netherlands (London)
+        let result = parse_and_convert_wgs84_bbox("-0.5,51.0,0.5,52.0");
+        assert!(result.is_err(), "Should reject coordinates outside Netherlands bounds");
+    }
+
+    #[test]
+    fn test_netherlands_bounds_validation() {
+        // Test edge of Netherlands bounds
+        // Southwest corner (approximate)
+        let result = parse_and_convert_wgs84_bbox("3.0,50.5,3.5,51.0");
+        assert!(result.is_ok(), "Should accept coordinates at Netherlands boundary");
+
+        // Northeast corner (approximate)
+        let result = parse_and_convert_wgs84_bbox("7.0,53.0,7.5,53.5");
+        assert!(result.is_ok(), "Should accept coordinates at Netherlands boundary");
+
+        // Just outside bounds
+        let result = parse_and_convert_wgs84_bbox("2.5,50.0,3.0,50.5");
+        assert!(result.is_err(), "Should reject coordinates outside Netherlands bounds");
+    }
+
+    #[test]
+    fn test_wgs84_to_rd_conversion() {
+        // Known conversion: Amsterdam Central Station
+        // WGS84: (4.9003, 52.3783) -> RD: (121595, 487725) approximately
+        if let Some((x, y)) = wgs84_to_rd(4.9003, 52.3783) {
+            assert!(x > 120000.0 && x < 125000.0, "RD X should be near Amsterdam");
+            assert!(y > 480000.0 && y < 495000.0, "RD Y should be near Amsterdam");
+        } else {
+            panic!("wgs84_to_rd should succeed for valid coordinates");
+        }
+    }
+
+    #[test]
+    fn test_wgs84_to_rd_returns_none_on_invalid() {
+        // Test with NaN
+        let result = wgs84_to_rd(f64::NAN, f64::NAN);
+        assert!(result.is_none(), "Should return None for NaN coordinates");
+
+        // Test with infinity
+        let result = wgs84_to_rd(f64::INFINITY, f64::INFINITY);
+        assert!(result.is_none(), "Should return None for infinity coordinates");
+    }
+}
+
 #[cfg(test)]
 mod coordinate_tests {
     use super::*;
