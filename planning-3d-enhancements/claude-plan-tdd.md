# TDD Plan: 3D Buildings Enhancements

This document defines the tests to write BEFORE implementing each section of the implementation plan. Tests follow the project's conventions: embedded `#[cfg(test)]` modules with `test_{feature}` naming.

---

## Overview Tests

### Backend (buildings_3d.rs)
- Test: proj crate conversion produces accurate RD coordinates
- Test: WGS84 bbox parameter is correctly parsed
- Test: WGS84 bbox is converted to RD for 3DBAG API call
- Test: CityJSON attributes are extracted correctly
- Test: Missing optional attributes don't cause errors
- Test: GeoJSON output contains expected property structure

### Frontend (data_verkenner.rs)
- Test: JavaScript fetch function is generated with correct endpoint
- Test: Debounce timeout is set to 300ms
- Test: State variables (lastFetchedBbox, fetchTimeout) are initialized

---

## Prerequisites Tests

### API Verification (Manual Test)
- Test: 3DBAG API returns expected CityJSON structure
- Test: Verify actual field names for ID, height, floors
- Test: Check availability of address and construction_year fields
- Test: Confirm bbox parameter format expected by 3DBAG

---

## Section 1: Dynamic Building Loading Tests

### Backend: Coordinate Conversion
- Test: WGS84 to RD conversion using proj crate is accurate
- Test: Conversion handles edge cases (bbox boundaries)
- Test: Disabled proj fallback doesn't cause crashes
- Test: Invalid coordinate bounds return appropriate error

### Backend: API Endpoint
- Test: `/api/buildings-3d?bbox-wgs84=...` endpoint accepts WGS84 coordinates
- Test: Endpoint converts WGS84 to RD before 3DBAG call
- Test: Limit parameter is respected (max 150)
- Test: Response format is valid GeoJSON FeatureCollection
- Test: Empty viewport returns empty FeatureCollection (not error)

### Frontend: Fetch Function
- Test: Map bounds are retrieved using getBounds()
- Test: Bounds are formatted as comma-separated WGS84 string
- Test: Fetch URL is constructed correctly
- Test: GeoJSON response updates map source
- Test: Last fetched bbox is stored after successful fetch

### Frontend: Debounce Logic
- Test: Fetch is not called immediately on map move
- Test: Fetch is called after 300ms timeout
- Test: Timeout is reset on subsequent move events
- Test: No fetch when bounds change < 10%

### Frontend: Error Handling
- Test: API errors (404, 500) show user-friendly message
- Test: Empty response shows "no buildings" message
- Test: Network timeout is handled gracefully
- Test: Invalid coordinates don't crash the map

### Frontend: Memory Management
- Test: Old popups are removed before new fetch
- Test: Building source is cleared on error
- Test: State variables are updated correctly

---

## Section 2: Height-Based Coloring Tests

### Frontend: Paint Properties
- Test: fill-extrusion-color uses step expression
- Test: Color thresholds match specification (5m, 15m)
- Test: Buildings with height 0-5m show light blue
- Test: Buildings with height 5-15m show medium purple
- Test: Buildings with height 15m+ show dark purple
- Test: Missing height defaults to reasonable color
- Test: Opacity is set to 0.8

### Frontend: Legend (Optional)
- Test: Legend HTML is generated
- Test: Legend shows correct color scale
- Test: Legend is positioned on map

---

## Section 3: Click Popups Tests

### Frontend: Click Handler
- Test: Click on building triggers popup
- Test: Click outside building doesn't trigger popup
- Test: Only top building is selected when overlapping
- Test: Popup is positioned at click location

### Frontend: Popup Content (XSS-Safe)
- Test: Popup title is set using textContent
- Test: All property values use textContent (no innerHTML)
- Test: Special characters in addresses are escaped
- Test: Null/missing properties show "N/A" or are omitted
- Test: Height is formatted with 1 decimal place

### Frontend: Popup Styling
- Test: Popup has building-popup CSS class
- Test: Popup styles are loaded
- Test: Popup closes when clicking elsewhere on map
- Test: Popup closes when map moves

### Frontend: Security
- Test: HTML in API data is not rendered (XSS prevention)
- Test: Script tags in address field don't execute
- Test: Special characters are displayed as text, not HTML

---

## Section 4: Implementation Order Tests

### Backend Tests (Write First)
1. Enable proj crate
2. Add WGS84 bbox support
3. Extract additional properties

### Frontend Tests (Write First)
1. CSS styles for popup
2. Dynamic loading with error handling
3. Height-based coloring
4. XSS-safe popup handler

---

## Section 5: Testing Checklist Tests

### Functional Tests
- Test: Pan to new area triggers building fetch
- Test: Zoom in/out triggers building fetch
- Test: No duplicate API calls after debounce
- Test: 10% threshold prevents unnecessary fetches
- Test: Building colors match height ranges
- Test: Click shows popup with correct data
- Test: Popup closes on map click
- Test: API limit is enforced (150 max)

### Error Handling Tests
- Test: 3DBAG API down shows error message
- Test: Empty viewport shows "no buildings" message
- Test: Network timeout doesn't hang UI
- Test: Invalid coordinates return error

### Memory Management Tests
- Test: Old buildings removed when loading new area
- Test: All popups closed on map move
- Test: No memory leak after 50+ pan operations

### Coordinate Accuracy Tests
- Test: Buildings align with map tiles
- Test: No visible offset at viewport edges
- Test: Multiple Netherlands locations work correctly

### Security Tests
- Test: Popup text is properly escaped
- Test: Script injection is prevented
- Test: Special characters display correctly

### Performance Tests
- Test: Initial load completes in < 3 seconds
- Test: Debounce delay is 300ms
- Test: Map remains responsive during fetch

---

## Integration Tests

### End-to-End Tests
- Test: Complete flow - load page, pan, see buildings, click for popup
- Test: Dense area (Amsterdam) loads correctly
- Test: Sparse area loads correctly
- Test: Rapid panning doesn't cause duplicate fetches

### Browser Tests
- Test: Chrome/Edge compatibility
- Test: Firefox compatibility
- Test: Safari compatibility (if available)

---

## Notes for Implementation

1. **Write tests first** for each section before implementation
2. **Use embedded test modules** following project conventions
3. **Run `cargo test` frequently** during development
4. **Focus on critical paths** - error handling, security, coordinate accuracy
5. **Manual browser testing** required for MapLibre integration
