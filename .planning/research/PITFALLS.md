# Pitfalls Research

**Domain:** 3D Building Visualization Enhancements
**Researched:** 2026-03-08
**Confidence:** MEDIUM

## Critical Pitfalls

### Pitfall 1: Filter Re-render Cascades

**What goes wrong:**
Every filter change triggers a full re-render of all 3D buildings, causing frame drops and UI freezing. Users experience 1-3 second delays when adjusting year/height/floor sliders, making the interface feel sluggish and unresponsive.

**Why it happens:**
Developers implement filters by clearing and re-adding the entire MapLibre layer instead of using filter expressions. The API makes this deceptively easy: `map.removeLayer()` followed by `map.addLayer()` works but forces complete geometry reprocessing.

**How to avoid:**
Use MapLibre's `setFilter()` method with filter expressions instead of layer recreation:
```javascript
// BAD - causes full re-render
map.removeLayer('buildings');
map.addLayer({
  id: 'buildings',
  filter: ['>=', ['get', 'construction_year'], minYear]
});

// GOOD - GPU-accelerated filter update
map.setFilter('buildings', ['>=', ['get', 'construction_year'], minYear]);
```

**Warning signs:**
- Frame profiler shows >100ms frames during filter changes
- Chrome DevTools shows geometry processing spikes on filter change
- Visual "flash" or blank map during filter updates

**Phase to address:**
Phase 1 (Filter Implementation) — Design filter architecture to use `setFilter()` from the start.

---

### Pitfall 2: Texture Memory Bloat

**What goes wrong:**
Applying satellite/aerial imagery textures to all buildings causes browser memory exhaustion. After panning across a few city blocks, memory usage grows to 2GB+ and tabs crash. The issue compounds with each texture-loaded building remaining in memory even when off-screen.

**Why it happens:**
Textures are created but never disposed. MapLibre's `addLayer()` with textures creates WebGL texture objects that persist until explicitly removed. Developers assume MapLibre handles texture lifecycle automatically, but it doesn't for custom textures applied via `fill-extrusion-pattern`.

**How to avoid:**
Implement a texture manager with explicit disposal:
```javascript
class TextureManager {
  constructor(gl) {
    this.gl = gl;
    this.textures = new Map();
    this.maxAge = 300000; // 5 minutes
  }

  addTexture(id, url) {
    if (this.textures.has(id)) {
      this.textures.get(id).lastUsed = Date.now();
      return this.textures.get(id).texture;
    }
    // Load and track texture...
  }

  disposeStale() {
    const now = Date.now();
    for (const [id, data] of this.textures) {
      if (now - data.lastUsed > this.maxAge) {
        this.gl.deleteTexture(data.texture);
        this.textures.delete(id);
      }
    }
  }
}
```

**Warning signs:**
- Chrome Task Manager shows tab memory growing monotonically during map panning
- Texture garbage collection doesn't happen even after viewing different areas
- Browser becomes sluggish after 10-15 minutes of use

**Phase to address:**
Phase 3 (Texture Mapping) — Build texture lifecycle management alongside texture loading.

---

### Pitfall 3: Density Heatmap Tile Artifacts

**What goes wrong:**
Density visualization shows visible seams at tile boundaries, with buildings suddenly disappearing or changing intensity when crossing tile edges. The heatmap appears "patchy" and users lose trust in the data accuracy.

**Why it happens:**
Density calculations are performed per-tile without edge consideration. A building near a tile boundary contributes to that tile's density but not the adjacent tile's, creating discontinuities. The issue is exacerbated by viewport-based dynamic loading where tiles load asynchronously.

**How to avoid:**
Use a sliding window density calculation with overlap:
```javascript
// Calculate density over a buffered region
function calculateBuildingDensity(buildings, bbox, bufferMeters = 100) {
  const bufferedBbox = expandBbox(bbox, bufferMeters);
  const buildingsInRegion = queryBuildings(bufferedBbox);

  // Group by grid cells (e.g., 100x100m)
  const grid = createDensityGrid(buildingsInRegion, 100);

  // Only return cells within original bbox
  return grid.filter(cell => bboxContains(bbox, cell.center));
}
```

**Warning signs:**
- Visible lines or color shifts at tile boundaries
- Density values jump when map moves slightly
- Console shows tiles loading at different times

**Phase to address:**
Phase 4 (Density Analysis) — Implement buffered density calculations with edge smoothing.

---

### Pitfall 4: 2D/3D Toggle State Desynchronization

**What goes wrong:**
After toggling between 2D and 3D views multiple times, building properties become desynchronized. Clicking a building shows popup data from a different building, or filters stop working correctly. Users report "ghost buildings" that don't match what's displayed.

**Why it happens:**
The 2D/3D toggle is implemented by swapping between two separate layers with different data sources. Event handlers and filter state become attached to the wrong layer during swaps. MapLibre doesn't automatically update listeners when layers are removed/re-added.

**How to avoid:**
Use a single layer with dynamic fill-extrusion properties:
```javascript
// BAD - separate layers
map.addLayer({ id: 'buildings-2d', type: 'fill', ... });
map.addLayer({ id: 'buildings-3d', type: 'fill-extrusion', ... });
// Toggle by showing/hiding

// GOOD - single layer, toggle properties
map.addLayer({
  id: 'buildings',
  type: 'fill-extrusion',
  paint: {
    'fill-extrusion-height': ['case', is3D, ['get', 'height'], 0]
  }
});
// Toggle by updating is3D variable
```

**Warning signs:**
- Popups show wrong building ID after toggle
- Filter state resets when switching views
- Click events stop responding after 2-3 toggles

**Phase to address:**
Phase 2 (2D/3D Toggle) — Design state management to keep layer identity stable.

---

### Pitfall 5: Viewport Debouncing Race Conditions

**What goes wrong:**
Rapid map panning causes multiple simultaneous API requests for the same bbox. Buildings flicker or appear multiple times, and the console shows "canceled request" errors. Under slow network conditions, old data arrives after new data, displaying stale buildings.

**Why it happens:**
The debounce implementation waits for the user to stop moving, but doesn't cancel pending requests. When a new viewport change occurs, a new request starts while the previous one is still in-flight. The faster the network, the worse the problem (more requests overlap).

**How to avoid:**
Use an abort controller chain:
```javascript
let currentAbortController = null;

function loadBuildingsForViewport(bbox) {
  // Cancel previous request
  if (currentAbortController) {
    currentAbortController.abort();
  }

  currentAbortController = new AbortController();

  fetch(`/api/buildings?bbox=${bbox}`, {
    signal: currentAbortController.signal
  })
    .then(res => res.json())
    .then(data => {
      // Only update if this is still the latest request
      if (!currentAbortController.signal.aborted) {
        updateBuildings(data);
      }
    })
    .catch(err => {
      if (err.name !== 'AbortError') {
        console.error('Load failed:', err);
      }
    });
}
```

**Warning signs:**
- Network tab shows multiple simultaneous requests for overlapping bboxes
- Buildings visually "pop" or flicker during panning
- Console logs show AbortError frequently

**Phase to address:**
Phase 1 (Filter Implementation) — Request cancellation is critical for viewport-based loading performance.

---

### Pitfall 6: Filter Expression Complexity Explosion

**What goes wrong:**
Combining multiple filters (year range + height range + floor count) creates exponentially complex filter expressions. With 1000+ buildings, the frame rate drops from 60fps to 15fps when all filters are active. The filtering logic works but makes the map unusably slow.

**Why it happens:**
MapLibre evaluates filter expressions on the CPU for each feature. Complex nested expressions like `['all', ['>=', ...], ['<=', ...], ['all', ...]]` cause O(n×m) evaluation where n is building count and m is expression complexity. Developers assume GPU acceleration applies, but it doesn't for property-based filters.

**How to avoid:**
Pre-filter on the backend and use simple expressions on frontend:
```javascript
// BAD - complex client-side filter
map.setFilter('buildings', [
  'all',
  ['>=', ['get', 'construction_year'], minYear],
  ['<=', ['get', 'construction_year'], maxYear],
  ['>=', ['get', 'height'], minHeight],
  ['<=', ['get', 'height'], maxHeight],
  ['>=', ['get', 'floors'], minFloors]
]);

// GOOD - backend filters, simple client filter
const filteredData = await fetch(
  `/api/buildings?bbox=${bbox}&year=${minYear}-${maxYear}&height=${minHeight}-${maxHeight}`
);
// Then just display what server returns
```

**Warning signs:**
- Frame rate correlates with filter count (more filters = slower)
- Chrome profiler shows 80%+ time in "evaluate expression"
- Performance degrades linearly with building count

**Phase to address:**
Phase 1 (Filter Implementation) — Design filter API to handle complexity server-side.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Skip texture disposal during MVP | Ships texture feature 2 days faster | Memory leaks require architecture rewrite to fix | Only for demos/proofs-of-concept, never for production |
| Use hardcoded bbox query parameters | Avoids viewport math complexity | Can't adapt to different screen sizes or map orientations | Only for initial testing, must be removed before users see it |
| Implement density as simple building count per tile | Works in 1 hour | Produces misleading visualization, breaks trust when users notice | Never — density requires area-based calculation from the start |
| Client-side filter only | No backend changes needed | Doesn't scale past 500 buildings, requires rewrite later | Acceptable for prototype only, must mark as tech debt |
| Skip 2D/3D toggle state tests | UI test is tedious, requires manual clicking | State bugs appear in production after 10+ toggles | Never — state synchronization bugs are hard to debug |

## Integration Gotchas

Common mistakes when connecting to external services.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| 3DBAG API | Assuming CityJSON parses directly to GeoJSON | Use a CityJSON → GeoJSON converter library (cityjson-js) before adding to MapLibre |
| DuckDB queries | Running separate queries for each filter condition | Build a single parameterized query with all filter conditions |
| Proj crate coordinate conversion | Converting coordinates in a loop for each building | Batch convert arrays of coordinates using projected arrays |
| MapLibre event handlers | Attaching click handlers in useEffect without cleanup | Return cleanup function that removes event listeners |
| Texture loading | Loading textures synchronously in render loop | Preload textures asynchronously, show loading indicator |

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Re-querying buildings on every filter change | Works fine with 100 buildings, 500ms delay at 1000 buildings | Cache viewport results client-side, apply filters via `setFilter()` | 500+ buildings per viewport |
| Creating new layer for each texture | 1-2 textures work fine, crashes at 20+ textures | Reuse texture slots, dispose old textures | 10+ simultaneous textures |
| Density calculation on every viewport move | Smooth at small datasets, freezes at city scale | Throttle density recalculation to once per second | 1000+ buildings in view |
| Unbounded result set from API | Fast during testing with small bbox, timeouts with large bbox | Always enforce max results limit (e.g., 2000 buildings) | Bbox > 1km² in urban areas |
| Synchronous popup rendering | Instant with few buildings, noticeable lag with many | Defer popup content rendering, show skeleton first | 500+ buildings with rich popup content |

## Security Mistakes

Domain-specific security issues beyond general web security.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Exposing raw bbox coordinates in API URLs | Users can probe the entire database systematically | Add rate limiting per IP, monitor for scraping patterns |
| Client-side filter bypass via MapLibre console | Users can modify filters to see all data including hidden buildings | Apply filters server-side, never rely on client for access control |
| Unvalidated construction_year in SQL queries | SQL injection via crafted year parameter | Use parameterized queries for all user input |
| Texture URLs from user input | XSS via malicious image URLs serving JavaScript | Validate texture URLs against whitelist, use CSP |
| Unbounded bbox queries | DoS via extremely large bbox requests | Enforce maximum bbox area (e.g., 5km²), return error if exceeded |

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Filter sliders without value display | Users can't tell they've selected "1990-2005" vs "1991-2004" | Always show current filter values next to controls |
| 2D/3D toggle without camera transition | Jarring perspective shift causes disorientation | Animate camera pitch/bearing when toggling views |
| Density heatmap without legend | Users don't know what colors mean | Include interactive legend with scale |
| Clearing all buildings during filter change | Map flashes empty, users think data is gone | Keep buildings visible until new filtered data arrives |
| No feedback during texture loading | Users wonder if feature is broken | Show loading spinner or progress bar for texture loads |
| Density overlay obscures buildings | Can't see individual buildings under heatmap | Use semi-transparent overlay with opacity slider |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Filter year range:** Often missing century boundaries (e.g., 1800s, 1900s) — verify buildings with construction_year < 1900 display correctly
- [ ] **Texture mapping:** Often missing fallback for failed texture loads — verify broken image URLs don't crash the renderer
- [ ] **Density calculation:** Often missing normalization by area — verify dense urban areas vs sparse suburbs use comparable scales
- [ ] **2D/3D toggle:** Often missing camera position preservation — verify map center and zoom stay consistent across toggle
- [ ] **Viewport loading:** Often missing loading indicator — verify users see feedback when panning to new areas
- [ ] **Popup positioning:** Often missing edge detection — verify popups don't get cut off at screen edges
- [ ] **Filter reset:** Often missing "reset to defaults" button — verify users can easily clear all filters

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Filter re-render cascades | HIGH | Rewrite filter system to use `setFilter()`, requires refactoring all filter logic (1-2 days) |
| Texture memory bloat | HIGH | Implement texture manager with disposal, may need to add reference tracking to existing textures (2-3 days) |
| Density heatmap tile artifacts | MEDIUM | Add buffered calculation to existing density function, adjust tile overlap (1 day) |
| 2D/3D toggle state desync | MEDIUM | Refactor to single-layer architecture, update all event handlers (1-2 days) |
| Viewport debounce race conditions | LOW | Add AbortController to existing fetch calls (2-4 hours) |
| Filter expression complexity | HIGH | Move filtering to backend API, add filter parameters to endpoint (2-3 days) |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Filter re-render cascades | Phase 1 - Filter Implementation | Profile frame rate during filter changes, verify <16ms per frame |
| Texture memory bloat | Phase 3 - Texture Mapping | Monitor memory usage in Chrome Task Manager during 15 min pan session |
| Density heatmap tile artifacts | Phase 4 - Density Analysis | Visual test for tile seams, automated test for density continuity across boundaries |
| 2D/3D toggle state desync | Phase 2 - 2D/3D Toggle | Test rapid toggling (20+ times), verify popup data matches clicked building |
| Viewport debounce race conditions | Phase 1 - Filter Implementation | Network tab should show only 1 active request during rapid panning |
| Filter expression complexity | Phase 1 - Filter Implementation | Backend API tests with complex filter combos, verify <100ms response time |

## Sources

- **MapLibre GL JS Documentation** - Filter expression performance characteristics (HIGH confidence)
- **WebGL Best Practices** (MDN) - Texture lifecycle management (HIGH confidence)
- **Common MapLibre Performance Issues** - Community discussions on filter re-render patterns (MEDIUM confidence)
- **3D Visualization Anti-patterns** - Industry knowledge on LOD and density calculation (MEDIUM confidence)
- **MapLibre GitHub Issues** - Race condition reports for viewport-based loading (LOW confidence - inferred from patterns)

---
*Pitfalls research for: 3D Building Visualization Enhancements*
*Researched: 2026-03-08*
