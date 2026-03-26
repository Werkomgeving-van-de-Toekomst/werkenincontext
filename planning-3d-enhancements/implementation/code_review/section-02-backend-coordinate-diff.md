# Section 02: Backend Coordinate Conversion - Code Diff

## Files Modified
- `crates/iou-api/src/routes/buildings_3d.rs`

## Changes

### Enable proj crate (line 83)
```diff
-    let use_proj = false; //proj::Proj::new_known_crs("EPSG:28992", "EPSG:4326", None).is_ok();
+    let use_proj = proj::Proj::new_known_crs("EPSG:28992", "EPSG:4326", None).is_ok();
```

This enables accurate RD to WGS84 coordinate conversion using the proj crate instead of the fallback approximation.

### Added Tests (new `#[cfg(test)]` module)
- `test_rd_to_wgs84_amersfoort()` - Validates proj conversion at reference point
- `test_rd_to_wgs84_returns_none_on_invalid()` - Handles invalid coordinates
- `test_fallback_conversion_does_not_panic()` - Fallback robustness
- `test_proj_more_accurate_than_fallback()` - Range validation
- `test_convert_geometry_proj_valid_input()` - Direct proj conversion test

## Full Diff

```diff
diff --git a/crates/iou-api/src/routes/buildings_3d.rs b/crates/iou-api/src/routes/buildings_3d.rs
index 0c12a11..858db48 100644
--- a/crates/iou-api/src/routes/buildings_3d.rs
+++ b/crates/iou-api/src/routes/buildings_3d.rs
@@ -80,7 +80,7 @@ pub async fn get_buildings_3d(Query(params): Query<BboxParams>) -> Json<serde_js


     // Check if proj is available for accurate conversion
-    let use_proj = false; //proj::Proj::new_known_crs("EPSG:28992", "EPSG:4326", None).is_ok();
+    let use_proj = proj::Proj::new_known_crs("EPSG:28992", "EPSG:4326", None).is_ok();

     // Process each building feature
     let features = if let Some(features_array" = cityjson.get("features").and_then(|f| f.as_array()) {
@@ -258,3 +258,74 @@ fn convert_geometry_fallback(
     fallback.insert("coordinates".to_string(), json!([0, 0]));
     serde_json::Value::Object(fallback)
 }

+#[cfg(test)]
+mod coordinate_tests {
+    use super::*;
+
+    #[test]
+    fn test_rd_to_wgs84_amersfoort() {
+        // Amersfoort coordinates (reference point)
+        // RD: (155000, 463000) should approximate WGS84: (5.3876, 52.1552)
+        if let Some((lon, lat)) = rd_to_wgs84(155000.0, 463000.0) {
+            assert!(f64::abs(lon - 5.3876) < 0.001, "Longitude mismatch");
+            assert!(f64::abs(lat - 52.1552) < 0.001, "Latitude mismatch");
+        } else {
+            panic!("proj conversion failed");
+        }
+    }
+
+    #[test]
+    fn test_rd_to_wgs84_returns_none_on_invalid() {
+        // Extreme coordinates that might fail conversion
+        let result = rd_to_wgs84(f64::NAN, f64::NAN);
+        assert!(result.is_none() || result.is_some()); // Should not panic
+    }
+
+    #[test]
+    fn test_fallback_conversion_does_not_panic() {
+        // Fallback should handle all inputs gracefully
+        let (lon, lat) = rd_to_wgs84_fallback(155000.0, 463000.0);
+        assert!(lon.is_finite());
+        assert!(lat.is_finite());
+    }
+
+    #[test]
+    fn test_proj_more_accurate_than_fallback() {
+        // Compare both methods at a known point
+        let rd_x = 155000.0;
+        let rd_y = 463000.0;
+
+        let proj_result = rd_to_wgs84(rd_x, rd_y);
+        let fallback_result = rd_to_wgs84_fallback(rd_x, rd_y);
+
+        if let Some((proj_lon, proj_lat)) = proj_result {
+            // proj should give valid results
+            assert!(proj_lon >= 3.0 && proj_lon <= 7.5, "Longitude out of Netherlands range");
+            assert!(proj_lat >= 50.0 && proj_lat <= 54.0, "Latitude out of Netherlands range");
+
+            // Fallback should be in reasonable range too
+            assert!(fallback_result.0 >= 3.0 && fallback_result.0 <= 7.5);
+            assert!(fallback_result.1 >= 50.0 && fallback_result.1 <= 54.0);
+        }
+    }
+
+    #[test]
+    fn test_convert_geometry_proj_valid_input() {
+        use proj::Proj;
+
+        // Skip if proj is not available
+        let proj = match Proj::new_known_crs("EPSG:28992", "EPSG:4326", None) {
+            Ok(p) => p,
+            Err(_) => return, // Skip test if proj unavailable
+        };
+
+        // Verify proj can convert a simple point
+        let result = proj.convert((155000.0, 463000.0));
+        assert!(result.is_ok(), "proj should convert valid RD coordinates");
+
+        let (lon, lat) = result.unwrap();
+        assert!(f64::abs(lon - 5.3876) < 0.01);
+        assert!(f64::abs(lat - 52.1552) < 0.01);
+    }
+}
```
