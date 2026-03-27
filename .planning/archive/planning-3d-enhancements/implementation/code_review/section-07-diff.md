# Section 07 Diff: Frontend Height-Based Coloring

## Files Modified

- `crates/iou-frontend/src/pages/data_verkenner.rs` (9 insertions, 1 deletion)

## Summary of Changes

This section implements height-based coloring for 3D buildings. The static gray color (`#8899aa`) is replaced with a step expression that colors buildings based on their height:

- 0-5m: Light blue (`#64B5F6`)
- 5-15m: Medium purple (`#9B59B6`)
- 15m+: Dark purple (`#8E44AD`)

## Full Diff

diff --git a/crates/iou-frontend/src/pages/data_verkenner.rs b/crates/iou-frontend/src/pages/data_verkenner.rs
index b2cb26b..1f95ca3 100644
--- a/crates/iou-frontend/src/pages/data_verkenner.rs
+++ b/crates/iou-frontend/src/pages/data_verkenner.rs
@@ -157,7 +157,15 @@ fn build_buildings_fetch_script() -> String {
                     type: 'fill-extrusion',
                     source: 'buildings',
                     paint: {
-                        'fill-extrusion-color': '#8899aa',
+                        'fill-extrusion-color': [
+                            'step',
+                            ['get', 'height'],
+                            '#64B5F6',  // 0-5m: Light blue
+                            5,
+                            '#9B59B6',  // 5-15m: Medium purple
+                            15,
+                            '#8E44AD'   // 15m+: Dark purple
+                        ],
                         'fill-extrusion-height': ['coalesce', ['get', 'height'], 10],
                         'fill-extrusion-base': 0,
                         'fill-extrusion-opacity': 0.8
