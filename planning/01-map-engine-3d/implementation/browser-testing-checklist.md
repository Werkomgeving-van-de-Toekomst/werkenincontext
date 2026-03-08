# Browser Testing Checklist - 3D Map Engine

**Date:** 2026-03-03
**Status:** Ready for Manual Testing
**Test Environment:**
- URL: http://localhost:8080/data_verkenner
- Feature Flag: `MAP_3D_ENABLED=true`

## Test Setup

### Enable 3D Map

```bash
export MAP_3D_ENABLED=true
cargo run
```

### Disable 3D Map (Leaflet Mode)

```bash
unset MAP_3D_ENABLED
# or
export MAP_3D_ENABLED=false
cargo run
```

---

## Leaflet Mode Tests (MAP_3D_ENABLED=false)

### Map Loading
- [ ] Map loads centered on Flevoland (52.45, 5.50)
- [ ] OpenStreetMap tiles render correctly
- [ ] Map is interactive (pan, zoom)

### Layer Loading
- [ ] Provinciegrens layer loads (blue polygon outline)
- [ ] Cultuurhistorische waarden layer loads (brown/orange point markers)
- [ ] Windturbines layer loads (blue point markers)
- [ ] Zonneparken layer loads (yellow filled polygons)
- [ ] Fietsnetwerken layer loads (green lines)
- [ ] Drinkwatergebieden layer loads (blue filled polygons)

### Layer Control
- [ ] Layer control panel is visible
- [ ] All six layers are listed
- [ ] Toggling layer checkbox shows/hides layer
- [ ] Layer state persists during pan/zoom

### Page Integration
- [ ] Dataset selector works
- [ ] Visualization panels update correctly
- [ ] No console errors on page load

---

## 3D Mode Tests (MAP_3D_ENABLED=true)

### Map Loading
- [ ] MapLibre GL JS initializes successfully
- [ ] Map loads centered on Flevoland (5.5, 52.4)
- [ ] WebGL2 context is available
- [ ] Map container has correct dimensions (550px height)

### Navigation
- [ ] Mouse drag rotates the map (bearing)
- [ ] Right-click drag (or Shift+drag) tilts the map (pitch)
- [ ] Scroll wheel zooms in/out
- [ ] Double-click zooms in
- [ ] Pinch zoom works (touch devices)

### Terrain Rendering
- [ ] Terrain tiles load from MapTiler
- [ ] Elevation is visible when map is tilted
- [ ] Terrain exaggeration applies (if configured)
- [ ] No "mesh-like" artifacts or missing tiles

### Layer Rendering
- [ ] Provinciegrens layer displays as red polygon outline
- [ ] Cultuurhistorie layer displays as blue point markers
- [ ] Layer colors match configuration

### Layer Control
- [ ] "Kaartlagen" panel is visible
- [ ] Provinciegrens checkbox shows initial checked state
- [ ] Cultuurhistorie checkbox shows initial unchecked state
- [ ] Toggling checkbox shows/hides layer immediately
- [ ] Layer state persists during navigation

### Error Handling
- [ ] Missing MAPTILER_API_KEY shows appropriate error
- [ ] WebGL2 not supported shows error message
- [ ] Invalid GeoJSON files handled gracefully

### Page Integration
- [ ] Dataset selector works (same as Leaflet mode)
- [ ] Visualization panels update correctly
- [ ] No console errors on page load
- [ ] Feature flag toggle doesn't require page reload

---

## Browser Compatibility

### Chrome/Edge (Chromium)
- [ ] WebGL2 available
- [ ] MapLibre renders correctly
- [ ] Performance is acceptable (60fps target)

### Firefox
- [ ] WebGL2 available
- [ ] MapLibre renders correctly
- [ ] No visual artifacts

### Safari
- [ ] WebGL2 available (may require Safari 15+)
- [ ] MapLibre renders correctly
- [ ] Known Safari issues documented

---

## Performance Tests

### Load Time
- [ ] Initial page load < 3 seconds
- [ ] First map render < 2 seconds
- [ ] Terrain tiles load progressively

### Runtime Performance
- [ ] Smooth panning (no stuttering)
- [ ] Smooth zooming
- [ ] Smooth rotation/pitch changes
- [ ] No memory leaks on layer toggle

---

## Accessibility

### Keyboard Navigation
- [ ] Map can be controlled via keyboard
- [ ] Layer checkboxes accessible via Tab key
- [ ] Focus indicators visible

### Screen Reader
- [ ] Layer control labels are announced
- [ ] Map status changes announced

---

## Known Issues

| Issue | Browser | Workaround | Status |
|-------|---------|------------|--------|
| | | | |

---

## Test Results Log

| Date | Tester | Browser | Mode | Result | Notes |
|------|--------|---------|------|--------|-------|
| 2026-03-03 | - | - | - | Pending | Ready for testing |

---

## Sign-off

**Tester:** _________________ **Date:** _________

**All critical tests passed:** [ ] Yes [ ] No

**Ready for production deployment:** [ ] Yes [ ] No

**Notes:**
