# Code Review: Section-01 Project Setup

## Critical Issues (Confidence: 95)

### 1. Invalid Dioxus.toml resource section name

- **File**: `/Users/marc/Projecten/iou-modern/crates/iou-frontend/Dioxus.toml` (lines 19-27)
- **Issue**: The section `[web.resource.map_3d]` uses a custom extension name `map_3d` which is not a standard Dioxus configuration. Dioxus only recognizes `[web.resource]` and `[web.resource.dev]` sections. Custom section names like `map_3d` will be ignored and the MapLibre resources will not be loaded.
- **Impact**: The MapLibre GL JS CSS and JavaScript files will not be loaded at all, causing runtime failures when the 3D map components are used.
- **Fix**: Add MapLibre directly to `[web.resource]`:
  ```toml
  [web.resource]
  style = [
      "https://unpkg.com/leaflet@1.9.4/dist/leaflet.css",
      "https://unpkg.com/maplibre-gl@5.0.0/dist/maplibre-gl.css",
  ]
  script = [
      "https://unpkg.com/leaflet@1.9.4/dist/leaflet.js",
      "https://unpkg.com/vis-network@9.1.9/standalone/umd/vis-network.min.js",
      "https://unpkg.com/maplibre-gl@5.0.0/dist/maplibre-gl.js",
  ]
  ```

### 2. Misleading comment about feature flag

- **File**: `/Users/marc/Projecten/iou-modern/crates/iou-frontend/Dioxus.toml` (line 21)
- **Issue**: The comment states "These resources are conditionally loaded based on MAP_3D_ENABLED feature flag" but no such feature flag exists in `Cargo.toml`.
- **Impact**: Creates false expectation about how conditional loading works.
- **Fix**: Remove the misleading comment.

## Minor Issues

### 3. Comment typo in mod.rs

- **File**: `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/mod.rs` (line 27)
- **Issue**: The line `/ pub use layer_control_3d::LayerControl3D;` is missing the second slash.
- **Fix**: Change to `// pub use layer_control_3d::LayerControl3D;`

## Positive Observations

1. **Code Organization**: The module structure is well-organized with clear separation of concerns.
2. **Documentation**: Good use of module-level doc comments (`//!`) explaining component purposes.
3. **Placeholder Strategy**: Clear indication that these are placeholders with references to future implementation sections.
