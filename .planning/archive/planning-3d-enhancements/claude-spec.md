# 3D Buildings Enhancements - Specification

## Overview
Enhance the existing 3D buildings layer in the IOU Modern Data Verkenner with dynamic loading, height-based coloring, and interactive popups.

## Current State
- **Frontend**: `crates/iou-frontend/src/pages/data_verkenner.rs`
  - Hardcoded bbox: `150000,470000,170000,490000` (Flevoland area)
  - Fixed limit: 100 buildings
  - Single gray color (#8899aa) for all buildings
  - No click interactions
  - Map auto-fits to building bounds on load

- **Backend**: `crates/iou-api/src/routes/buildings_3d.rs`
  - Endpoint: `GET /api/buildings-3d?bbox={rd_bbox}&limit={n}`
  - Returns GeoJSON with: height, min_height, floors
  - RD to WGS84 coordinate conversion

## Requirements

### 1. Dynamic Building Loading
**Goal**: Load buildings based on current map viewport instead of fixed bbox.

**Implementation**:
- Use MapLibre's `map.getBounds()` to get current viewport
- Convert to RD coordinates (EPSG:28992) for 3DBAG API
- Listen to `moveend` and `zoomend` events
- Debounce requests (300ms) to avoid API overload
- Limit: 150 buildings per viewport (conservative for performance)

**API Changes**:
- Current bbox is in RD (Rijksdriehoek)
- Need WGS84 → RD conversion for bbox calculation

### 2. Height-Based Coloring
**Goal**: Color buildings based on height using a cool gradient.

**Color Scheme**:
| Height Range | Color | Hex |
|--------------|-------|-----|
| 0-5m | Light blue | #64B5F6 |
| 5-15m | Medium purple | #9B59B6 |
| 15m+ | Dark purple | #8E44AD |

**Implementation**:
- Use MapLibre `step` expression in `paint['fill-extrusion-color']`
- Expression: `['step', ['get', 'height'], '#64B5F6', 5, '#9B59B6', 15, '#8E44AD']`

### 3. Click Popups
**Goal**: Show building information when clicking on a building.

**Popup Content**:
- Building ID: `NL.IMBAG.Pand.*`
- Height: X meters
- Floors: X levels
- Additional info (if available from 3DBAG): address, year

**Implementation**:
- Add click event listener to building layer
- Use `map.queryRenderedFeatures()` to get clicked feature
- Create `new maplibregl.Popup()` with feature properties
- Position popup at click coordinates

### 4. Backend Enhancements
**Goal**: Return additional building attributes for popups.

**Add to GeoJSON properties**:
- `bag_id`: Building identifier
- `address`: Street address (if available)
- `construction_year`: Build year (if available)

## Technical Approach

### Frontend Changes
1. **Extract building layer management** to separate function
2. **Add event handlers** for map move/zoom
3. **Implement debouncing** for API calls
4. **Add click handler** for popups
5. **Update layer paint properties** for height-based colors

### Backend Changes
1. **Parse additional attributes** from 3DBAG CityJSON
2. **Include more properties** in GeoJSON output
3. **Add error handling** for API failures

## Dependencies
- `dioxus` - Frontend framework
- `maplibre-gl-js` - Map rendering (already loaded)
- `axum` - Backend framework
- `reqwest` - HTTP client
- `serde_json` - JSON handling

## Success Criteria
- [ ] Buildings load when map is panned/zoomed
- [ ] Buildings show correct color based on height
- [ ] Clicking a building shows popup with ID, height, floors
- [ ] Performance acceptable with 150 buildings
- [ ] No duplicate buildings loaded
