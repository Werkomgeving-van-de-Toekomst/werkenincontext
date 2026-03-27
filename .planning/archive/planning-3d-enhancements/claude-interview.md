# Interview Transcript: 3D Buildings Enhancements

## User Requirements

**Original Spec:**
- Load more buildings (increase limit from 100)
- Dynamic bbox based on map position
- Color coding by building height
- Click popup with building info

## Research Findings Summary

**Codebase Research:**
- Current implementation in `data_verkenner.rs` uses hardcoded bbox and limit
- Backend API at `/api/buildings-3d` supports bbox and limit parameters
- GeoJSON format includes: height, min_height, floors properties
- No click handlers currently implemented
- Fixed styling with single color (#8899aa)

**Web Research Topics:**
- MapLibre dynamic loading with getBounds() and moveend events
- Data-driven styling using property expressions
- Click interactions with queryRenderedFeatures

## Interview Answers

**Q1: How many buildings should be loaded at once?**
A: Conservative (100-200)
- Current: 100 buildings
- New: 150-200 buildings per viewport
- Reason: Balance detail vs performance

**Q2: What color scheme should indicate building height?**
A: Cool gradient (Blue → Purple)
- Low buildings (0-5m): Light blue
- Medium buildings (5-15m): Medium purple
- High buildings (15m+): Dark purple
- Using MapLibre step expressions

**Q3: What information should the building popup show?**
A: Building ID, Dimensions, Additional info
- Building ID: `NL.IMBAG.Pand.*`
- Height: in meters
- Floors: count
- Additional: Address, construction year (if available from 3DBAG)

## Implementation Notes

**Files to modify:**
1. `crates/iou-frontend/src/pages/data_verkenner.rs` - Map initialization and event handlers
2. `crates/iou-api/src/routes/buildings_3d.rs` - Add more attributes to response

**Key technical decisions:**
- Use MapLibre's `map.getBounds()` for dynamic bbox
- Listen to `moveend` and `zoomend` events (debounced)
- Use `step` expression for color-based height styling
- Use `map.on('click', ...)` for popup interaction
