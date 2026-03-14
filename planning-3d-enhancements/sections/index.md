<!-- PROJECT_CONFIG
runtime: rust-cargo
test_command: cargo test
END_PROJECT_CONFIG -->

<!-- SECTION_MANIFEST
section-01-prerequisites
section-02-backend-coordinate
section-03-backend-wgs84
section-04-backend-properties
section-05-frontend-css
section-06-frontend-loading
section-07-frontend-coloring
section-08-frontend-popups
END_MANIFEST -->

# Implementation Sections Index

## Dependency Graph

| Section | Depends On | Blocks | Parallelizable |
|---------|------------|--------|----------------|
| section-01-prerequisites | - | 04 | Yes |
| section-02-backend-coordinate | - | 03, 04 | Yes |
| section-03-backend-wgs84 | 02 | 04, 06 | No |
| section-04-backend-properties | 01, 02, 03 | 06 | No |
| section-05-frontend-css | - | 08 | Yes |
| section-06-frontend-loading | 03, 04 | 07, 08 | No |
| section-07-frontend-coloring | 06 | 08 | No |
| section-08-frontend-popups | 05, 06 | - | No |

## Execution Order

**Batch 1 (Parallel):**
- section-01-prerequisites (API verification - manual, no dependencies)
- section-02-backend-coordinate (Enable proj crate - no dependencies)
- section-05-frontend-css (Popup styling - no dependencies)

**Batch 2 (After 02):**
- section-03-backend-wgs84 (WGS84 bbox endpoint - requires coordinate conversion)

**Batch 3 (After 01, 02, 03):**
- section-04-backend-properties (Property extraction - requires verified fields and WGS84 support)

**Batch 4 (After 03, 04):**
- section-06-frontend-loading (Dynamic loading - requires WGS84 endpoint and property structure)

**Batch 5 (After 06):**
- section-07-frontend-coloring (Height-based colors - requires building source)

**Batch 6 (After 05, 06):**
- section-08-frontend-popups (Click popups - requires CSS and loading setup)

## Section Summaries

### section-01-prerequisites
Verify 3DBAG API response format. Make test API calls to confirm actual field names for building ID, address, and construction year. Document the correct CityJSON attribute names before implementing backend changes.

### section-02-backend-coordinate
Enable the `proj` crate in `buildings_3d.rs` for accurate WGS84 to RD coordinate conversion. Change `use_proj = false` to `true` and verify coordinate accuracy is improved.

### section-03-backend-wgs84
Add alternative endpoint accepting WGS84 bbox parameters. Implement backend conversion from WGS84 to RD before calling 3DBAG API. This keeps accurate conversion on the server and simplifies frontend code.

### section-04-backend-properties
Extract additional properties from CityJSON response. Add bag_id, address, and construction_year to GeoJSON output based on verified field names from prerequisites. Handle missing optional fields gracefully.

### section-05-frontend-css
Create CSS file for popup styling. Define styles for `.building-popup` class including title, spacing, and text formatting. Ensure popup is visually distinct and readable.

### section-06-frontend-loading
Implement dynamic building loading based on map viewport. Add debounced fetch function (300ms + 10% threshold), state tracking for last bbox, error handling for API failures, and cleanup of old building data.

### section-07-frontend-coloring
Update MapLibre paint properties to color buildings by height. Use step expression with light blue (0-5m), medium purple (5-15m), and dark purple (15m+) colors. Optionally add legend overlay.

### section-08-frontend-popups
Add click handler for buildings using XSS-safe DOM methods (no innerHTML). Create popup using `setDOMContent()` with properly escaped text content. Handle missing properties gracefully and ensure popup closes on map interaction.

## Testing Notes

- **Backend tests**: Use embedded `#[cfg(test)]` modules with `test_` naming convention
- **Frontend tests**: Manual browser testing required for MapLibre integration
- **Integration tests**: Test complete flow from map pan to popup display
- **Security tests**: Verify XSS prevention in popup content
