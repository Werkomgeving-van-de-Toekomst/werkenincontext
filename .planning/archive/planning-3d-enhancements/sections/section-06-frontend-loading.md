Now I have all the context I need. Let me generate the section content for `section-06-frontend-loading`.

# Section 6: Frontend Dynamic Loading

## Overview

This section implements dynamic building loading based on map viewport. Instead of loading a fixed set of buildings from a hardcoded bbox, the map will fetch buildings for the currently visible area as the user pans and zooms.

**Dependencies:**
- `section-03-backend-wgs84` - WGS84 bbox endpoint must be available
- `section-04-backend-properties` - Property structure must be established

## File to Modify

- `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs`

## Tests

### Frontend: Fetch Function Generation

Add tests to verify the JavaScript fetch function is generated correctly:

```rust
#[cfg(test)]
mod test_frontend_loading {
    use super::*;

    #[test]
    fn test_fetch_script_includes_wgs84_endpoint() {
        let script = build_buildings_fetch_script();
        assert!(script.contains("bbox-wgs84"), 
                "Fetch script should use WGS84 bbox parameter");
    }

    #[test]
    fn test_fetch_script_includes_debounce() {
        let script = build_buildings_fetch_script();
        assert!(script.contains("setTimeout") && script.contains("300"),
                "Fetch script should include 300ms debounce");
    }

    #[test]
    fn test_fetch_script_includes_state_variables() {
        let script = build_buildings_fetch_script();
        assert!(script.contains("lastFetchedBbox") && script.contains("fetchTimeout"),
                "Fetch script should initialize state variables");
    }

    #[test]
    fn test_fetch_script_includes_threshold_check() {
        let script = build_buildings_fetch_script();
        assert!(script.contains("0.1") || script.contains("10%"),
                "Fetch script should include 10% threshold check");
    }

    #[test]
    fn test_fetch_script_includes_error_handling() {
        let script = build_buildings_fetch_script();
        assert!(script.contains("catch") && script.contains("error"),
                "Fetch script should include error handling");
    }
}
```

### Frontend: Manual Browser Tests

These require manual testing in a browser:

- Map bounds are retrieved using `map.getBounds()`
- Bounds are formatted as comma-separated WGS84 string
- Fetch URL is constructed correctly with `bbox-wgs84` parameter
- GeoJSON response updates map source successfully
- Last fetched bbox is stored after successful fetch

## Implementation

### 1. Add State Tracking Variables

The JavaScript environment needs to track:
- Last fetched bbox to avoid duplicate requests
- Fetch timeout for debouncing

Add these variables in the map initialization script:

```javascript
let lastFetchedBbox = null;
let fetchTimeout = null;
```

### 2. Build Fetch Script Function

Create or update a function that generates the JavaScript for fetching buildings. This function should be called during map initialization.

Function signature:

```rust
fn build_buildings_fetch_script() -> String {
    // Returns JavaScript code as a string
}
```

The generated JavaScript should:

1. **Get current map bounds** (WGS84 coordinates from MapLibre)
2. **Format bounds as bbox-wgs84 parameter** (west,south,east,north)
3. **Fetch from backend API** at `/api/buildings-3d?bbox-wgs84=...&limit=150`
4. **Update the GeoJSON source** with response data
5. **Store last fetched bbox** for threshold comparison

### 3. Implement Debounce with Threshold

Add a debounce function that:

- Waits 300ms after map movement before fetching
- Checks if bounds changed by at least 10% before fetching
- Resets timeout on subsequent movement events

```javascript
function shouldFetch(newBounds) {
    if (!lastFetchedBbox) return true;

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

### 4. Add Event Listeners

Register the debounced fetch function to map events:

```javascript
map.on('moveend', debouncedFetch);
map.on('zoomend', debouncedFetch);
```

### 5. Implement Fetch Function

Create the async fetch function with error handling:

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
```

### 6. Implement Error Handlers

Add helper functions for error states:

```javascript
function showNoBuildingsMessage() {
    if (map.getSource('buildings-3d')) {
        map.getSource('buildings-3d').setData({
            type: 'FeatureCollection',
            features: []
        });
    }
}

function showErrorMessage(message) {
    // Optionally show user-facing error notification
    console.warn(message);
}
```

### 7. Implement Memory Management

Clean up old data when loading new buildings:

```javascript
function updateBuildingsSource(geojson) {
    // Clear any existing popups
    document.querySelectorAll('.maplibregl-popup').forEach(p => p.remove());

    // Update the source
    map.getSource('buildings-3d').setData(geojson);
}
```

### 8. Initial Load

Call the fetch function once on map initialization to load buildings for the initial viewport:

```javascript
// After map is initialized
debouncedFetch();
```

## Integration with Existing Code

### Locate the Map3D Initialization

Find the existing `get_map3d_init_script()` function in `data_verkenner.rs`. This function returns the JavaScript string that initializes the MapLibre map.

### Modify the Script

Add the fetch script generation to the map initialization. The fetch function should be:

1. Defined as part of the map initialization script
2. Called after the map is fully loaded
3. Attached to map move/zoom events

## Constants

Define these constants for the fetch behavior:

| Constant | Value | Purpose |
|----------|-------|---------|
| Debounce delay | 300ms | Wait time after map movement |
| Threshold | 10% (0.1) | Minimum bounds change to trigger fetch |
| Limit | 150 | Maximum buildings per API call |

## Error Cases to Handle

| Error | Handling |
|-------|----------|
| API 404 response | Clear buildings, show empty state |
| API 500 response | Show user-friendly error message |
| Network timeout | Log error, show error message |
| Empty viewport | Show "no buildings" state, clear source |
| Invalid coordinates | Don't fetch, log warning |

## Testing Checklist

After implementation, verify:

- [ ] Buildings load when panning to new area
- [ ] Buildings load when zooming
- [ ] No duplicate buildings (debounce works)
- [ ] No fetch when panning less than 10% (threshold check)
- [ ] API errors show user-friendly message
- [ ] Empty viewport clears buildings
- [ ] Old popups removed when loading new area
- [ ] Initial page load shows buildings for starting viewport
- [ ] Map remains responsive during fetch

## Actual Implementation (2025-03-07)

**File Modified:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs`

**Changes from Original Plan:**
1. **Race Condition Fix:** `lastFetchedBbox` is now updated inside `fetchBuildings()` after successful fetch, not in `debouncedFetch()`. This prevents stale data from overwriting newer data when fetches complete out of order.

2. **AbortController Added:** Implemented request cancellation to prevent wasted network resources when user moves map rapidly. Old fetches are aborted when new ones are initiated.

3. **Position Change Detection:** Enhanced `shouldFetch()` to detect panning by comparing center positions. Previously only dimension changes (zoom) would trigger fetch; now significant pans (>10% of viewport) also trigger fetch.

4. **Constant for Limit:** Extracted `limit=150` to `BUILDINGS_FETCH_LIMIT` constant for maintainability.

**Code Review Issues Applied:**
- Critical: Race condition in state updates - Fixed
- Critical: Missing fetch abort controller - Fixed
- Important: Incomplete position change detection - Fixed
- Minor: Hardcoded limit value - Fixed

**Tests Added:** 5 unit tests verifying fetch script generation (all passing)
