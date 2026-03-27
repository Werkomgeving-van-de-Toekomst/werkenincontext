<!-- PROJECT_CONFIG
runtime: rust-cargo
test_command: cargo test
END_PROJECT_CONFIG -->

<!-- SECTION_MANIFEST
section-01-project-setup
section-02-config-structures
section-03-terrain-encoding
section-04-map3d-component
section-05-terrain-integration
section-06-geojson-support
section-07-layer-control
section-08-page-integration
section-09-testing
END_MANIFEST -->

# Implementation Sections Index

## Dependency Graph

| Section | Depends On | Blocks | Parallelizable |
|---------|------------|--------|----------------|
| section-01-project-setup | - | 02, 03, 04 | Yes |
| section-02-config-structures | 01 | 04 | No |
| section-03-terrain-encoding | - | 05 | Yes |
| section-04-map3d-component | 01, 02 | 05, 06 | No |
| section-05-terrain-integration | 03, 04 | 06 | No |
| section-06-geojson-support | 04, 05 | 07 | No |
| section-07-layer-control | 06 | 08 | No |
| section-08-page-integration | 04, 07 | 09 | No |
| section-09-testing | 01-08 | - | No |

## Execution Order

**Batch 1 (Parallel):**
- section-01-project-setup (no dependencies)
- section-03-terrain-encoding (independent, pure Rust functions)

**Batch 2 (After 01):**
- section-02-config-structures

**Batch 3 (After 01, 02):**
- section-04-map3d-component

**Batch 4 (After 03, 04):**
- section-05-terrain-integration

**Batch 5 (After 04, 05):**
- section-06-geojson-support

**Batch 6 (After 06):**
- section-07-layer-control

**Batch 7 (After 04, 07):**
- section-08-page-integration

**Batch 8 (Final, after all):**
- section-09-testing

## Section Summaries

### section-01-project-setup
Create new component files (map_3d.rs, layer_control_3d.rs), update components/mod.rs exports, add MapLibre CSS/JS resources to Dioxus.toml.

### section-02-config-structures
Define Map3DConfig, GeoJsonLayer, LayerType structs with validation logic and environment variable support.

### section-03-terrain-encoding
Implement elevation_to_terrain_rgb() and terrain_rgb_to_elevation() functions with full test coverage.

### section-04-map3d-component
Build the core Map3D Dioxus component with MapLibre initialization via document::eval(), event handling, and state management.

### section-05-terrain-integration
Configure MapTiler terrain source, implement terrain rendering with exaggeration support, and add fallback error handling.

### section-06-geojson-support
Implement addGeoJsonLayer(), toggleLayer() JavaScript bridge functions, layer type detection, and migrate provinciegrens and cultuurhistorie layers.

### section-07-layer-control
Build LayerControl3D UI component with checkboxes for each layer, visibility state management, and Dutch localization.

### section-08-page-integration
Modify data_verkenner.rs to conditionally render Map3D or Leaflet based on MAP_3D_ENABLED feature flag, preserving existing layout.

### section-09-testing
Write unit tests, integration tests, browser compatibility checklist, and verify all acceptance criteria are met.
