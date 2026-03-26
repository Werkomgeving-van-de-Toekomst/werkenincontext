# Section 05 Diff: Frontend CSS - Popup Styling

## Files Created

- `crates/iou-frontend/src/styles/building_popup.css` (new file)

## Diff Content

```diff
diff --git a/crates/iou-frontend/src/styles/building_popup.css b/crates/iou-frontend/src/styles/building_popup.css
new file mode 100644
index 0000000..30449cb
--- /dev/null
+++ b/crates/iou-frontend/src/styles/building_popup.css
@@ -0,0 +1,30 @@
+/* Building Popup Styles for 3D Buildings Layer */
+
+/* Main popup container */
+.building-popup {
+    padding: 12px;
+    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
+    font-size: 14px;
+    color: #333;
+    min-width: 200px;
+}
+
+/* Popup title */
+.building-popup h3 {
+    margin: 0 0 8px 0;
+    font-size: 16px;
+    color: #1a1a1a;
+    border-bottom: 1px solid #ddd;
+    padding-bottom: 6px;
+}
+
+/* Property rows */
+.building-popup p {
+    margin: 4px 0;
+    line-height: 1.4;
+}
+
+/* Property labels (strong text) */
+.building-popup strong {
+    color: #555;
+}
```

## Context

This CSS file provides styling for the building popup windows that will appear when users click on 3D buildings. The styles follow the design considerations from the section plan:

- System font stack for native appearance
- 12px padding for comfortable whitespace
- Typography hierarchy (h3 at 16px, body at 14px)
- Color contrast meeting accessibility standards (#333, #555)
- Subtle 1px border separating title from content
- 200px minimum width

The CSS will be referenced by the popup creation code in section-08 (frontend-popups) using MapLibre GL JS's `Popup.setDOMContent()` method.
