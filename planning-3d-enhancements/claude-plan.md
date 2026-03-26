# Implementation Plan: 3D Buildings Enhancements

## Overview

This plan describes enhancements to the existing 3D buildings visualization in the IOU Modern Data Verkenner. The current implementation loads a fixed set of 100 buildings from a hardcoded bbox with a single gray color. The enhancements will:

1. Dynamically load buildings based on map viewport
2. Color buildings by height using a blue-to-purple gradient
3. Show building information in popups on click

**Why these changes:** The current implementation only shows buildings in one fixed area (Flevoland). Users cannot explore other regions, and buildings all look the same regardless of their actual height. Interactive popups will help users understand building properties.

## Current Architecture

```
Frontend (Dioxus/Rust) → MapLibre GL JS → Backend API (Axum/Rust) → 3DBAG External API
                                                    ↓
                                            buildings_3d.rs
```

**Key files:**
- `crates/iou-frontend/src/pages/data_verkenner.rs` - Map initialization, building fetch
- `crates/iou-api/src/routes/buildings_3d.rs` - 3DBAG proxy, coordinate conversion

## Prerequisites

Before implementing, verify the 3DBAG API response format. The plan assumes certain fields exist but they must be verified:

**Action:** Make a test API call to confirm actual field names:
```bash
curl "https://api.3dbag.nl/collections/pand/items?bbox=150000,470000,155000,475000&limit=1"
```

**Known fields (from current code):**
- `b3_h_dak_max` - Roof height
- `b3_h_maaiveld` - Ground level
- `b3_bouwlagen` - Floor count

**Fields to verify:**
- BAG ID identifier (may be in different field)
- Address information (may not be directly available)
- Construction year (may not be directly available)

If fields are not available, adjust the plan to:
1. Use only available fields
2. Or derive missing data from available attributes

## Section 1: Dynamic Building Loading

### 1.1 Current State

The map initialization uses a hardcoded RD bbox and fixed limit:

```javascript
// Current implementation (simplified)
fetch('/api/buildings-3d?bbox=150000,470000,170000,490000&limit=100')
```

### 1.2 Coordinate Conversion

**IMPORTANT:** Use the backend's `proj` crate for accurate coordinate conversion, not a simplified JavaScript formula.

The backend file `crates/iou-api/src/routes/buildings_3d.rs` already has `proj` crate integration, but it's currently disabled (`use_proj = false` on line 83). To fix coordinate accuracy:

**Backend change first:**
```rust
// In buildings_3d.rs, line 83
const USE_PROJ: bool = true;  // Change from false to true
```

**API modification:** Accept WGS84 bbox parameters and convert to RD on the backend:

```rust
// Add alternative endpoint that accepts WGS84
// /api/buildings-3d?bbox-wgs84=4.5,52.0,5.0,52.5&limit=150
```

This approach:
- Provides accurate coordinate conversion (hundreds of meters precision vs kilometers with approximation)
- Keeps conversion logic on the backend where it's maintained
- Simplifies frontend code (no JS conversion needed)

### 1.3 Frontend Changes

**Frontend (`data_verkenner.rs`):**

Replace the static fetch with a dynamic function that:

1. Gets current map bounds using `map.getBounds()` (returns WGS84)
2. Passes WGS84 bounds directly to backend API
3. Fetches buildings for the visible area
4. Updates the GeoJSON source
5. Tracks last-fetched bbox to avoid duplicate requests

**Function signature to add:**

```rust
fn build_buildings_fetch_script() -> String {
    // Returns JavaScript that:
    // 1. Gets map bounds (WGS84)
    // 2. Formats as bbox-wgs84 parameter
    // 3. Fetches from /api/buildings-3d?bbox-wgs84=...
    // 4. Updates source with new data
    // 5. Stores last fetched bbox
}
```

**State to track:**
```javascript
let lastFetchedBbox = null;
let fetchTimeout = null;
```

**Debounce requirement:** Add 300ms debounce AND minimum 10% bounds change threshold:

```javascript
function shouldFetch(newBounds) {
    if (!lastFetchedBbox) return true;

    // Check if bounds changed by at least 10%
    const width = newBounds[2] - newBounds[0];
    const lastWidth = lastFetchedBbox[2] - lastFetchedBbox[0];
    const height = newBounds[3] - newBounds[1];
    const lastHeight = lastFetchedBbox[3] - lastFetchedBbox[1];

    const widthChange = Math.abs(width - lastWidth) / lastWidth;
    const heightChange = Math.abs(height - lastHeight) / lastHeight;

    return widthChange > 0.1 || heightChange > 0.1;
}

function debouncedFetch() {
    clearTimeout(fetchTimeout);
    fetchTimeout = setTimeout(() => {
        const bounds = map.getBounds();
        const bbox = [bounds.getWest(), bounds.getSouth(),
                      bounds.getEast(), bounds.getNorth()];
        if (shouldFetch(bbox)) {
            fetchBuildings(bbox);
            lastFetchedBbox = bbox;
        }
    }, 300);
}
```

**Event listeners to add:**
```javascript
map.on('moveend', debouncedFetch);
map.on('zoomend', debouncedFetch);
```

### 1.4 Error Handling

Add robust error handling for API calls:

```javascript
async function fetchBuildings(bbox) {
    try {
        const response = await fetch(
            `/api/buildings-3d?bbox-wgs84=${bbox.join(',')}&limit=150`
        );

        if (!response.ok) {
            if (response.status === 404) {
                showNoBuildingsMessage();
                return;
            }
            throw new Error(`API error: ${response.status}`);
        }

        const data = await response.json();

        if (data.features.length === 0) {
            showNoBuildingsMessage();
            return;
        }

        updateBuildingsSource(data);

    } catch (error) {
        console.error('Failed to fetch buildings:', error);
        showErrorMessage('Kon gebouwen niet laden. Probeer het opnieuw.');
    }
}

function showNoBuildingsMessage() {
    // Remove existing building layer
    if (map.getSource('buildings-3d')) {
        map.getSource('buildings-3d').setData({
            type: 'FeatureCollection',
            features: []
        });
    }
}
```

### 1.5 Limit Setting

Use limit of 150 buildings per viewport. This balances detail vs performance. In dense urban areas (Amsterdam), this may not cover the full viewport, but prevents API overload.

### 1.6 Memory Management

Add cleanup when loading new buildings:

```javascript
function updateBuildingsSource(geojson) {
    // Clear any existing popups
    document.querySelectorAll('.maplibregl-popup').forEach(p => p.remove());

    // Update the source
    map.getSource('buildings-3d').setData(geojson);
}
```

## Section 2: Height-Based Coloring

### 2.1 Current State

All buildings use a single gray color:

```javascript
paint: {
    'fill-extrusion-color': '#8899aa',
    'fill-extrusion-height': ['coalesce', ['get', 'height'], 10],
    'fill-extrusion-base': 0,
    'fill-extrusion-opacity': 0.8
}
```

### 2.2 Required Changes

Replace `'fill-extrusion-color'` with a step expression:

```javascript
paint: {
    'fill-extrusion-color': [
        'step',
        ['get', 'height'],
        '#64B5F6',  // 0-5m: Light blue
        5,
        '#9B59B6',  // 5-15m: Medium purple
        15,
        '#8E44AD'   // 15m+: Dark purple
    ],
    'fill-extrusion-height': ['coalesce', ['get', 'height'], 10],
    'fill-extrusion-base': 0,
    'fill-extrusion-opacity': 0.8
}
```

### 2.3 Legend (Optional Enhancement)

Consider adding a small legend overlay showing the color scale. This can be a simple HTML overlay positioned in the corner of the map.

## Section 3: Click Popups

### 3.1 Current State

No click handlers exist. Building properties are available in the GeoJSON but not displayed.

### 3.2 Required Changes

**Add click event listener:**

```javascript
map.on('click', 'building-3d', function(e) {
    var features = map.queryRenderedFeatures(e.point, { layers: ['building-3d'] });
    if (features.length > 0) {
        var props = features[0].properties;
        var popup = createBuildingPopup(props);
        popup.setLngLat(e.lngLat).addTo(map);
    }
});
```

**Popup content structure (XSS-safe):**

**SECURITY:** Use DOM methods instead of template literals to prevent XSS attacks from untrusted API data.

```javascript
function createBuildingPopup(props) {
    var container = document.createElement('div');
    container.className = 'building-popup';

    var title = document.createElement('h3');
    title.textContent = 'Gebouw Info';
    container.appendChild(title);

    function addRow(label, value) {
        if (value === null || value === undefined) return;
        var row = document.createElement('p');
        var labelEl = document.createElement('strong');
        labelEl.textContent = label + ': ';
        row.appendChild(labelEl);
        row.appendChild(document.createTextNode(String(value)));
        container.appendChild(row);
    }

    addRow('ID', props.bag_id);
    addRow('Hoogte', props.height ? props.height.toFixed(1) + 'm' : null);
    addRow('Verdiepingen', props.floors);
    addRow('Adres', props.address);
    addRow('Bouwjaar', props.construction_year);

    return new maplibregl.Popup().setDOMContent(container);
}
```

### 3.3 CSS Requirements

Add popup styles to the map CSS:

```css
.building-popup {
    padding: 12px;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    font-size: 14px;
    color: #333;
    min-width: 200px;
}

.building-popup h3 {
    margin: 0 0 8px 0;
    font-size: 16px;
    color: #1a1a1a;
    border-bottom: 1px solid #ddd;
    padding-bottom: 6px;
}

.building-popup p {
    margin: 4px 0;
    line-height: 1.4;
}

.building-popup strong {
    color: #555;
}
```

### 3.4 Backend Changes for Additional Properties

**IMPORTANT:** Verify actual 3DBAG API response format before implementing. See Prerequisites section.

**Modify `buildings_3d.rs`:**

After verifying field names, add properties to the GeoJSON output:

```rust
// In the feature construction, add verified fields
// Field names below are examples - adjust based on actual API response

// Use the CityJSON ID as bag_id
props.insert("bag_id".to_string(), json!(feature_id));

// Extract height data (already available)
let roof_height = attributes.get("b3_h_dak_max").and_then(|v| v.as_f64());
let ground_level = attributes.get("b3_h_maaiveld").and_then(|v| v.as_f64());
let floors = attributes.get("b3_bouwlagen").and_then(|v| v.as_i64());

props.insert("height".to_string(), json!(roof_height.unwrap_or(0.0)));
props.insert("ground_level".to_string(), json!(ground_level.unwrap_or(0.0)));
props.insert("floors".to_string(), json!(floors.unwrap_or(1)));

// Add additional fields if available in API response
if let Some(address) = extract_address(&attributes) {
    props.insert("address".to_string(), json!(address));
}
if let Some(year) = attributes.get("construction_year").and_then(|v| v.as_i64()) {
    props.insert("construction_year".to_string(), json!(year));
}
```

**Helper function example:**
```rust
fn extract_address(attrs: &Map<String, Value>) -> Option<String> {
    // Implementation depends on actual API structure
    // May need to combine street, house_number, postal_code, city
    None // Placeholder until API structure is verified
}
```

## Section 4: Implementation Order

1. **Prerequisites:** Verify 3DBAG API response format (test API call)
2. **Backend:**
   - Enable `proj` crate for accurate coordinate conversion
   - Add WGS84 bbox parameter support
   - Extract additional properties (based on verified API response)
3. **Frontend - CSS:** Add popup styling
4. **Frontend - dynamic loading:** Implement bounds-based fetching with error handling
5. **Frontend - coloring:** Update paint properties
6. **Frontend - popups:** Add XSS-safe click handler

## Section 5: Testing Checklist

### Functional Tests

- [ ] Buildings load when panning to new area
- [ ] Buildings load when zooming
- [ ] No duplicate buildings (debounce works)
- [ ] No fetch when panning less than 10% (threshold check)
- [ ] Buildings show correct colors based on height
- [ ] Click on building shows popup
- [ ] Popup contains ID, height, floors
- [ ] Popup closes when clicking elsewhere
- [ ] API limit respected (150 buildings max)

### Error Handling Tests

- [ ] Graceful handling when 3DBAG API is down (error message shown)
- [ ] Graceful handling when viewport has no buildings
- [ ] Network timeout handling (no hang)
- [ ] Invalid coordinate bounds handling

### Memory Management Tests

- [ ] Old buildings removed when loading new area
- [ ] Popups closed when map moves
- [ ] No memory leak after 50+ pan operations

### Coordinate Accuracy Tests

- [ ] Buildings appear at correct map positions
- [ ] No visible offset between building locations and map tiles
- [ ] Test in multiple Netherlands locations (north, south, east, west)

### Security Tests

- [ ] Popup text content is escaped (no XSS from API data)
- [ ] Special characters in addresses display correctly

### Performance Tests

- [ ] Initial page load with buildings completes in < 3 seconds
- [ ] Pan operation triggers fetch after 300ms debounce
- [ ] Map remains responsive during building fetch

## Section 6: File Modifications

**Files to modify:**

1. `crates/iou-api/src/routes/buildings_3d.rs`
   - Enable `proj` crate (line 83: change `use_proj = false` to `true`)
   - Add alternative endpoint accepting WGS84 bbox: `/api/buildings-3d?bbox-wgs84=...`
   - Extract and add verified CityJSON attributes to output
   - Add bag_id, address, construction_year (if available)

2. `crates/iou-frontend/src/pages/data_verkenner.rs`
   - Update `get_map3d_init_script()` function
   - Add state variables (lastFetchedBbox, fetchTimeout)
   - Add debounced fetch function with threshold check
   - Add error handling for API failures
   - Add cleanup for old data
   - Add click handler using DOM methods (XSS-safe)
   - Update paint properties for height-based coloring

3. **New file:** `crates/iou-frontend/src/styles/building_popup.css`
   - Add popup styling as defined in Section 3.3
   - Import in data_verkenner.rs or include in map styles
