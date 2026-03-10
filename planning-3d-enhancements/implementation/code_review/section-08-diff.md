# Section 08 Diff: Frontend Click Popups

## Files Modified

- `crates/iou-frontend/src/pages/data_verkenner.rs` (111 insertions)

## Summary of Changes

This section implements interactive popups that display building information when users click on 3D buildings.

**Key additions:**
1. New function `build_popup_handler_script()` - generates JavaScript for popup handling
2. Click event listener on `building-3d` layer
3. XSS-safe DOM methods for creating popup content (no innerHTML)
4. Display of building properties: bag_id, height, floors, construction_year
5. Popup closes on map move
6. 5 new tests verifying XSS safety and functionality

**Security:**
- Uses `textContent` for all user-controlled data
- Uses `document.createElement()` for DOM construction
- No use of `innerHTML` or template literals with untrusted data

## Full Diff

diff --git a/crates/iou-frontend/src/pages/data_verkenner.rs b/crates/iou-frontend/src/pages/data_verkenner.rs
index f6726e7..d517b8f 100644
--- a/crates/iou-frontend/src/pages/data_verkenner.rs
+++ b/crates/iou-frontend/src/pages/data_verkenner.rs
@@ -256,9 +256,64 @@ fn build_buildings_fetch_script() -> String {
     "#.to_string()
 }
 
+/// Builds the JavaScript popup handler script for building click interactions.
+fn build_popup_handler_script() -> String {
+    r#"
+        // Create popup content using XSS-safe DOM methods
+        function createBuildingPopup(props) {
+            var container = document.createElement('div');
+            container.className = 'building-popup';
+
+            // Title
+            var title = document.createElement('h3');
+            title.textContent = 'Gebouw Info';
+            container.appendChild(title);
+
+            // Helper function for adding rows (XSS-safe)
+            function addRow(label, value) {
+                if (value === null || value === undefined) return;
+                var row = document.createElement('p');
+                var labelEl = document.createElement('strong');
+                labelEl.textContent = label + ': ';
+                row.appendChild(labelEl);
+                row.appendChild(document.createTextNode(String(value)));
+                container.appendChild(row);
+            }
+
+            // Add property rows
+            addRow('ID', props.bag_id);
+            addRow('Hoogte', props.height !== undefined ? props.height.toFixed(1) + 'm' : null);
+            addRow('Verdiepingen', props.floors);
+            addRow('Bouwjaar', props.construction_year);
+
+            return new maplibregl.Popup({ offset: 15 }).setDOMContent(container);
+        }
+
+        // Register click handler for buildings
+        map.on('click', 'building-3d', function(e) {
+            var features = map.queryRenderedFeatures(e.point, {
+                layers: ['building-3d']
+            });
+
+            if (features.length > 0) {
+                var props = features[0].properties;
+                var popup = createBuildingPopup(props);
+                popup.setLngLat(e.lngLat).addTo(map);
+            }
+        });
+
+        // Close popup when map moves
+        map.on('move', function() {
+            var popups = document.querySelectorAll('.maplibregl-popup');
+            popups.forEach(function(p) { p.remove(); });
+        });
+    "#.to_string()
+}
+
 /// Returns the Map3D initialization script.
 fn get_map3d_init_script() -> String {
     let fetch_script = build_buildings_fetch_script();
+    let popup_handler_script = build_popup_handler_script();
     format!(
         r#"
     (function() {{
@@ -312,6 +367,9 @@ fn get_map3d_init_script() -> String {
 
                     // Initial fetch for starting viewport
                     debouncedFetch();
+
+                    // Setup popup handler after buildings are loaded
+                    {popup_handler_script}
                 }});
 
                 map.on('error', function(e) {{
@@ -554,3 +612,56 @@ mod test_frontend_loading {
                 "Fetch script should include error handling");
     }
 }
+
+#[cfg(test)]
+mod test_popup_handler {
+    use super::*;
+
+    #[test]
+    fn test_popup_script_uses_dom_methods() {
+        let script = build_popup_handler_script();
+
+        // Verify XSS-safe methods are used
+        assert!(script.contains("textContent"),
+                "Popup must use textContent for XSS safety");
+    }
+
+    #[test]
+    fn test_popup_does_not_use_innerhtml() {
+        let script = build_popup_handler_script();
+
+        // Verify innerHTML is NOT used
+        assert!(!script.contains("innerHTML"),
+                "Popup must NOT use innerHTML for XSS safety");
+    }
+
+    #[test]
+    fn test_popup_includes_all_properties() {
+        let script = build_popup_handler_script();
+
+        // Verify all expected properties are referenced
+        assert!(script.contains("bag_id"), "Popup should show BAG ID");
+        assert!(script.contains("height"), "Popup should show height");
+        assert!(script.contains("floors"), "Popup should show floors");
+        assert!(script.contains("construction_year"), "Popup should show construction year");
+    }
+
+    #[test]
+    fn test_popup_has_correct_css_class() {
+        let script = build_popup_handler_script();
+
+        // Verify the CSS class from section-05 is used
+        assert!(script.contains("building-popup"),
+                "Popup should use the building-popup CSS class");
+    }
+
+    #[test]
+    fn test_popup_has_click_handler() {
+        let script = build_popup_handler_script();
+
+        // Verify click event listener is registered
+        assert!(script.contains("'click', 'building-3d'") ||
+                script.contains("\"click\", \"building-3d\""),
+                "Popup should register click handler on building-3d layer");
+    }
+}
