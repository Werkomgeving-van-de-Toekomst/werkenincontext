# Code Review: Section 06 - Frontend Dynamic Loading

## Review Scope

This review analyzes the changes made in `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs` for section-06-frontend-loading, which implements dynamic building loading based on map viewport.

**Files Modified:** 1 file (197 insertions, 84 deletions)
- `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs`

---

## Critical Issues (90-100)

### 1. Race Condition in State Updates (Confidence: 95)

**Location:** `build_buildings_fetch_script()`, line 210-211

```javascript
if (shouldFetch(bbox)) {
    fetchBuildings(bbox);
    lastFetchedBbox = bbox;  // <-- Updated BEFORE fetch completes
}
```

**Issue:** The `lastFetchedBbox` is updated immediately after initiating the fetch, not after it completes. This creates a race condition where:

1. User pans map quickly, triggering fetch A
2. `lastFetchedBbox` is updated to bbox A
3. User pans again before fetch A completes, triggering fetch B
4. `shouldFetch()` compares against bbox A (not the actual loaded data)
5. If fetch A completes after fetch B, the old data overwrites newer data

**Impact:** The user may see stale building data or incorrect buildings displayed.

**Fix:** Update `lastFetchedBbox` only after the fetch completes successfully:

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
        lastFetchedBbox = bbox;  // Move here - after successful load
        console.log('Loaded ' + data.features.length + ' buildings from 3DBAG');

    } catch (error) {
        console.error('Failed to fetch buildings:', error);
        showErrorMessage('Kon gebouwen niet laden. Probeer het opnieuw.');
    }
}

// In debouncedFetch, remove the lastFetchedBbox update:
function debouncedFetch() {
    clearTimeout(fetchTimeout);
    fetchTimeout = setTimeout(() => {
        const bounds = map.getBounds();
        const bbox = [bounds.getWest(), bounds.getSouth(),
                      bounds.getEast(), bounds.getNorth()];

        if (shouldFetch(bbox)) {
            fetchBuildings(bbox);  // State updated inside fetchBuildings now
        }
    }, 300);
}
```

---

### 2. Missing Fetch Abort Controller (Confidence: 90)

**Location:** `build_buildings_fetch_script()`, line 171-198

**Issue:** When the user moves the map rapidly, multiple fetch requests are initiated without cancelling previous ones. This causes:

1. Network bandwidth waste
2. Potential for out-of-order responses (race condition)
3. UI showing data from an area the user has already moved away from

**Impact:** Poor UX, wasted resources, and potential data inconsistencies.

**Fix:** Implement an AbortController to cancel pending fetches:

```javascript
// Add to state tracking
let abortController = null;

async function fetchBuildings(bbox) {
    // Cancel any pending fetch
    if (abortController) {
        abortController.abort();
    }

    // Create new abort controller for this fetch
    abortController = new AbortController();

    try {
        const response = await fetch(
            `/api/buildings-3d?bbox-wgs84=${bbox.join(',')}&limit=150`,
            { signal: abortController.signal }
        );

        // ... rest of fetch logic

    } catch (error) {
        if (error.name === 'AbortError') {
            console.log('Fetch aborted due to new request');
            return;
        }
        console.error('Failed to fetch buildings:', error);
        showErrorMessage('Kon gebouwen niet laden. Probeer het opnieuw.');
    } finally {
        abortController = null;
    }
}
```

---

## Important Issues (80-89)

### 3. Incomplete Position Change Detection (Confidence: 85)

**Location:** `build_buildings_fetch_script()`, line 108-123

```javascript
function shouldFetch(newBounds) {
    if (!lastFetchedBbox) return true;

    const width = newBounds[2] - newBounds[0];
    const lastWidth = lastFetchedBbox[2] - lastFetchedBbox[0];
    const height = newBounds[3] - newBounds[1];
    const lastHeight = lastFetchedBbox[3] - lastFetchedBbox[1];

    // Handle zero dimensions
    if (lastWidth === 0 || lastHeight === 0) return true;

    const widthChange = Math.abs(width - lastWidth) / lastWidth;
    const heightChange = Math.abs(height - lastHeight) / lastHeight;

    return widthChange > 0.1 || heightChange > 0.1;
}
```

**Issue:** The `shouldFetch()` function only checks if the viewport dimensions changed by >10%, but doesn't check if the viewport position changed significantly. A user can pan the map a long distance without zooming, and `shouldFetch()` will return `false` because the width/height remain the same.

**Impact:** Users can pan outside the originally loaded area and see no buildings, with no automatic reload.

**Fix:** Add center position comparison:

```javascript
function shouldFetch(newBounds) {
    if (!lastFetchedBbox) return true;

    const width = newBounds[2] - newBounds[0];
    const lastWidth = lastFetchedBbox[2] - lastFetchedBbox[0];
    const height = newBounds[3] - newBounds[1];
    const lastHeight = lastFetchedBbox[3] - lastFetchedBbox[1];

    // Handle zero dimensions
    if (lastWidth === 0 || lastHeight === 0) return true;

    // Calculate centers
    const newCenterX = (newBounds[0] + newBounds[2]) / 2;
    const newCenterY = (newBounds[1] + newBounds[3]) / 2;
    const lastCenterX = (lastFetchedBbox[0] + lastFetchedBbox[2]) / 2;
    const lastCenterY = (lastFetchedBbox[1] + lastFetchedBbox[3]) / 2;

    // Check if center moved more than 10% of viewport size
    const centerMoveX = Math.abs(newCenterX - lastCenterX) / lastWidth;
    const centerMoveY = Math.abs(newCenterY - lastCenterY) / lastHeight;

    const widthChange = Math.abs(width - lastWidth) / lastWidth;
    const heightChange = Math.abs(height - lastHeight) / lastHeight;

    return widthChange > 0.1 || heightChange > 0.1 || centerMoveX > 0.1 || centerMoveY > 0.1;
}
```

---

### 4. Unused Error Message (Confidence: 82)

**Location:** `build_buildings_fetch_script()`, line 166-168

```javascript
function showErrorMessage(message) {
    console.warn(message);
}
```

**Issue:** The `showErrorMessage()` function receives a user-facing Dutch error message but only logs it to the console. Users won't see any error feedback in the UI.

**Impact:** Users don't know when a fetch fails unless they open the browser console.

**Fix:** Either implement UI error notification or remove the parameter:

```javascript
function showErrorMessage(message) {
    console.warn(message);
    // TODO: Show toast/notification to user
    // For now, at least ensure the building layer is cleared
    showNoBuildingsMessage();
}
```

---

### 5. Popup Clearing May Be Incomplete (Confidence: 80)

**Location:** `build_buildings_fetch_script()`, line 126-128

```javascript
function updateBuildingsSource(geojson) {
    // Clear any existing popups
    document.querySelectorAll('.maplibregl-popup').forEach(p => p.remove());
```

**Issue:** Using `querySelectorAll` to remove popups by class name may miss popups that:
1. Have different class names due to MapLibre GL JS version changes
2. Are stored in the MapLibre instance but not yet attached to DOM
3. Use custom popup containers

**Impact:** Stale popups may remain when the map moves to a new area.

**Fix:** Track popups properly using MapLibre's API:

```javascript
// Store reference to open popup
let openPopup = null;

function updateBuildingsSource(geojson) {
    // Close any tracked popup
    if (openPopup) {
        openPopup.remove();
        openPopup = null;
    }

    // Also clear any remaining popups as fallback
    document.querySelectorAll('.maplibregl-popup').forEach(p => p.remove());

    // ... rest of function
}
```

Then update popup creation code (likely in a click handler elsewhere) to set `openPopup`.

---

## Minor Issues (70-79)

### 6. Test Coverage Could Be More Specific (Confidence: 75)

**Location:** Test module, lines 482-514

**Issue:** The tests only check for string presence, not actual behavior. They verify keywords exist but don't validate:
- Correct function signatures
- Proper error handling flow
- Threshold logic correctness

**Example:**
```rust
#[test]
fn test_fetch_script_includes_error_handling() {
    let script = build_buildings_fetch_script();
    assert!(script.contains("catch") && script.contains("error"),
            "Fetch script should include error handling");
}
```

This test passes even if the error handling is malformed.

**Recommendation:** Consider more structured testing or integration tests. However, for embedded JavaScript in Rust, string-based testing may be the only practical option without significant refactoring.

---

### 7. Hardcoded Limit Value (Confidence: 72)

**Location:** `build_buildings_fetch_script()`, line 174

```javascript
`/api/buildings-3d?bbox-wgs84=${bbox.join(',')}&limit=150`
```

**Issue:** The `limit=150` is hardcoded. While not necessarily wrong, it would be better as a constant at the top of the script for easier tuning.

**Fix:**
```javascript
// At top of script
const BUILDINGS_FETCH_LIMIT = 150;

// In fetch URL
`/api/buildings-3d?bbox-wgs84=${bbox.join(',')}&limit=${BUILDINGS_FETCH_LIMIT}`
```

---

### 8. Missing Loading State Indication (Confidence: 78)

**Location:** `build_buildings_fetch_script()`, line 171-198

**Issue:** There's no visual loading indicator while buildings are being fetched. Users may not realize data is loading, especially on slow connections.

**Fix:** Add a loading state:

```javascript
let isLoading = false;

async function fetchBuildings(bbox) {
    if (isLoading) return;  // Prevent concurrent fetches
    isLoading = true;

    // Show loading indicator
    document.body.classList.add('buildings-loading');

    try {
        // ... fetch logic
    } finally {
        isLoading = false;
        document.body.classList.remove('buildings-loading');
    }
}
```

---

## Positive Observations

1. **Clean separation of concerns:** The fetch script is isolated in its own function (`build_buildings_fetch_script()`)

2. **Proper debouncing:** 300ms debounce is appropriate for map interactions

3. **Good error handling basics:** Try-catch blocks are present, 404 is handled specially

4. **WGS84 API integration:** Correctly uses `bbox-wgs84` parameter with proper array formatting

5. **State initialization:** Variables are properly initialized at the top of the script scope

---

## Summary

The implementation successfully replaces hardcoded RD bbox fetching with dynamic WGS84-based viewport loading. However, there are **two critical issues** that should be addressed before merging:

1. **Race condition in state updates** - `lastFetchedBbox` is updated before fetch completes
2. **Missing request cancellation** - No AbortController for rapid map movements

Additionally, **three important issues** significantly impact UX:
3. Incomplete position change detection (pan without zoom doesn't trigger fetch)
4. Error messages not shown to users
5. Popup clearing may be incomplete

**Recommendation:** Address the race condition and missing abort controller before merging. The other issues can be tackled in follow-up improvements.

---

## Files Reviewed

- `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs` (lines 100-216, 482-514)

**Review Date:** 2026-03-07
**Reviewer:** Code Review Agent
**Confidence Threshold:** >= 80 reported only
