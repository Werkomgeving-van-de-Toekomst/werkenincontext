# TDD Plan - Map Engine 3D Upgrade

This document defines tests to write BEFORE implementing each section of the implementation plan.

## 3. Components

### 3.1 Map3D Component

**Tests to write before implementation:**
- Test: Map3DConfig creates with valid default values
- Test: Map3DConfig validates longitude (-180 to 180) and latitude (-90 to 90)
- Test: Map3DConfig validates pitch (0 to 60) and bearing (0 to 360)
- Test: Map3DConfig validates zoom range (6 to 18)
- Test: Map3D component renders container div with correct id
- Test: Map3D component initializes only once (prevents duplicate map instances)

### 3.2 LayerControl3D Component

**Tests to write before implementation:**
- Test: GeoJsonLayer creates with all required fields
- Test: LayerType enum has Point, Line, Polygon variants
- Test: LayerControl3D renders checkboxes for each layer
- Test: Toggling checkbox updates layer visibility state

### 3.4 Dioxus Integration Pattern

**Tests to write before implementation:**
- Test: use_effect cleanup function executes on unmount
- Test: map_loaded signal prevents re-initialization
- Test: document::eval return value is parsed correctly

## 5. Terrain Data Strategy

### 5.2 Terrain-RGB Encoding

**Tests to write before implementation:**
- Test: elevation_to_terrain_rgb encodes minimum elevation (-10000m) as (0, 0, 0)
- Test: elevation_to_terrain_rgb encodes sea level (0m) correctly
- Test: elevation_to_terrain_rgb encodes maximum elevation within Flevoland range
- Test: elevation_to_terrain_rgb is reversible (decode after encode matches original)
- Test: elevation_to_terrain_rgb handles negative elevations correctly
- Test: elevation_to_terrain_rgb handles fractional elevations (rounding behavior)

## 6. GeoJSON Layer Migration

### 6.1 Layer Type Detection

**Tests to write before implementation:**
- Test: detect_layer_type returns Point for GeoJSON with Point geometries
- Test: detect_layer_type returns Line for GeoJSON with LineString geometries
- Test: detect_layer_type returns Polygon for GeoJSON with Polygon geometries
- Test: detect_layer_type handles FeatureCollection correctly
- Test: detect_layer_type handles empty GeoJSON gracefully
- Test: detect_layer_type logs warning for mixed geometry types

## 7. Configuration

### 7.1 Map Configuration

**Tests to write before implementation:**
- Test: Map3DConfig::default() creates sensible defaults
- Test: Map3DConfig validates terrain_exaggeration range (0.1 to 5.0)
- Test: Map3DConfig::from_env reads MAP_3D_ENABLED flag
- Test: Map3DConfig::from_env defaults to false when flag not set

## 8. Testing Strategy

### 8.1 Unit Tests

**Test files to create:**
- `crates/iou-frontend/src/components/terrain_encoding_test.rs` - Terrain-RGB encoding tests
- `crates/iou-frontend/src/components/layer_config_test.rs` - Layer configuration tests

### 8.2 Integration Tests

**Test files to create:**
- `crates/iou-frontend/tests/map3d_integration_test.rs` - Map3D component behavior

### 8.3 Browser Tests

**Manual test checklist:**
- [ ] Chrome: Map loads without console errors
- [ ] Chrome: Terrain is visible when pitch > 0
- [ ] Chrome: Can tilt from 0 to 60 degrees
- [ ] Chrome: Can rotate 0 to 360 degrees
- [ ] Chrome: Can zoom in and out smoothly
- [ ] Firefox: All above tests pass
- [ ] Performance: Frame rate stays above 30fps during navigation
- [ ] provinciegrens layer displays correctly
- [ ] cultuurhistorie layer displays correctly
- [ ] Layer toggle turns layers on/off
- [ ] Feature flag MAP_3D_ENABLED=false shows Leaflet map
- [ ] Feature flag MAP_3D_ENABLED=true shows 3D map

### 8.4 Error Handling

**Tests to write before implementation:**
- Test: Invalid GeoJSON URL triggers error state
- Test: Malformed GeoJSON content is handled gracefully
- Test: Missing container element is handled without crash
- Test: WebGL2 not detected shows fallback message

## 9. Implementation Steps

### Step 1: Project Setup

**Tests to write before implementation:**
- Test: mod.rs exports Map3D component
- Test: mod.rs exports LayerControl3D component

### Step 2: MapLibre Integration

**Manual verification:**
- [ ] MapLibre CSS loads without errors
- [ ] MapLibre JS loads without errors
- [ ] Map container div has correct CSS class
- [ ] Navigation controls appear on map

### Step 3: Terrain Integration

**Manual verification:**
- [ ] Terrain source loads from MapTiler
- [ ] Terrain is visible when pitch > 30 degrees
- [ ] Elevation exaggeration affects terrain appearance

### Step 4: GeoJSON Support

**Tests to write before implementation:**
- Test: add_geojson_layer generates correct JavaScript
- Test: toggle_layer generates correct JavaScript
- Test: Layer colors are applied correctly

### Step 5: Layer Control UI

**Manual verification:**
- [ ] All configured layers appear in control
- [ ] Checkboxes match initial layer visibility
- [ ] Toggling checkbox updates map immediately

### Step 6: Integration

**Tests to write before implementation:**
- Test: data_verkenner page renders without crash
- Test: MAP_3D_ENABLED=false renders Leaflet map
- Test: MAP_3D_ENABLED=true renders Map3D component

### Step 7: Testing

**Automated:**
- Test: `cargo test` passes all new tests
- Test: Code coverage > 80% for new modules

## Test Execution Commands

```bash
# Run all tests
cargo test

# Run only frontend tests
cargo test -p iou-frontend

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_elevation_encoding
```

## Test Coverage Goals

- Unit tests: 100% for pure Rust functions (encoding, config)
- Integration tests: 80% for component behavior
- Manual tests: Full browser compatibility matrix
