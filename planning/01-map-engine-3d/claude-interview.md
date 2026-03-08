# Interview Transcript - Map Engine 3D Upgrade

**Datum:** 2026-03-03

## Q1: Terrain Data Strategy

**Vraag:** PDOK AHN3 doesn't provide XYZ tiles in Terrain-RGB format directly. Which approach should we use for terrain data?

**Antwoord:** PDOK tile proxy

**Besluit:**
- Implement a tile proxy service that converts PDOK AHN3 WMS service to Terrain-RGB XYZ tiles
- This allows using AHN3 (5m resolution) data with MapLibre GL JS
- Proxy will handle:
  - WMS → XYZ tile conversion
  - RGB encoding for elevation data
  - Tile caching
  - CORS headers

**Implicaties:**
- Extra backend service needed
- Or implement as a serverless function
- Tile caching strategy important for performance

## Q2: Feature Migration Scope

**Vraag:** How many existing Leaflet features must be preserved in the 3D upgrade?

**Antwoord:** 3D terrain first, Core only

**Besluit:**
- Focus on getting 3D terrain working first
- Core Leaflet features to preserve:
  - Basic map display
  - GeoJSON layer loading
  - Layer control (toggle layers on/off)
- Can defer for now:
  - Advanced pop-ups
  - Custom point styling
  - Layer reordering

**MVP Definition:**
1. MapLibre GL JS initialized with 3D terrain
2. Flevoland region visible with terrain elevation
3. 3D navigation (pitch, rotate, zoom) functional
4. At least 2 existing GeoJSON layers working

## Q3: Browser Compatibility

**Vraag:** What browser compatibility is required for the 3D map?

**Antwoord:** Chromium+Firefox

**Besluit:**
- Primary support: Chrome, Chromium-based browsers (Edge, Brave, etc.)
- Secondary: Firefox
- Safari support can come later if needed (WebGL2 support exists)
- Test on:
  - Chrome/Edge (Chromium)
  - Firefox

**Technical Note:**
- MapLibre GL JS requires WebGL2
- Both Chromium and Firefox have good WebGL2 support
- Safari has WebGL2 but may have different performance characteristics

## Summary of Decisions

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| Terrain Source | PDOK tile proxy | Use actual AHN3 data for Flevoland |
| Feature Scope | 3D terrain first, core features | Phased approach, reduce complexity |
| Browser Support | Chromium+Firefox | Cover majority of users, Safari later |
| Map Library | MapLibre GL JS v5.x | Open source, no API key required |
| Initial View | Flevoland [5.5, 52.4], zoom 10 | Centered on province |
| Pitch | 60 degrees | Good 3D perspective |

## Implementation Priority

1. **Phase 1 (MVP):**
   - MapLibre GL JS integration
   - 3D terrain with AHN3 (via proxy)
   - 3D navigation controls
   - 2 GeoJSON layers migrated
   - Chrome/Firefox testing

2. **Phase 2 (Future):**
   - Full GeoJSON layer migration
   - Advanced pop-ups
   - Safari support
   - Performance optimization
