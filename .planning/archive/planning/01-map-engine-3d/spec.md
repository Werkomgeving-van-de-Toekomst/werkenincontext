# 01-map-engine-3d

## Description

Core 3D map infrastructure upgrade from Leaflet to Mapbox GL JS with AHN terrain integration.

## Background

De huidige Data Verkenner gebruikt Leaflet.js met 2D kaarten. Om 3D terreinweergave en moderne kaartfuncties te ondersteunen, moet de kaartengine worden geüpgraded.

## Requirements

### Core Map Engine
- [ ] Vervang Leaflet met Mapbox GL JS (of MapLibre GL JS)
- [ ] Behoud bestaande GeoJSON lagen functionaliteit
- [ ] Implementeer 3D navigatie controls (tilt, rotate, zoom)
- [ ] Ondersteuning voor externe tile providers

### 3D Terrain
- [ ] Integreer AHN3 (Actueel Hoogtebestand) terrain tiles
- [ ] Configureer terrain exaggeration factor
- [ ] Toon hoogtelijnen optioneel
- [ ] Focus op Flevoland regio (begrenzingsbox)

### Performance
- [ ] Smooth 60fps rendering
- [ ] Lazy loading van terrain tiles
- [ ] Level of Detail (LOD) management

### WASM Integration
- [ ] Mapbox GL JS integratie met Dioxus
- [ ] JS bridge voor map events
- [ ] State management voor map view

### Existing Features
- [ ] Migratie huidige GeoJSON lagen:
  - provinciegrens.geojson
  - cultuurhistorie.geojson
  - windturbines.geojson
  - zonneparken.geojson
  - fietsnetwerken.geojson
  - drinkwater.geojson
- [ ] Behoud layer control functionaliteit
- [ ] Behoud pop-up styling

## Technical Notes

**Mapbox GL JS vs Alternatief:**
- Mapbox GL JS is proprietary (gratis tot 50k MAU)
- Alternatieven: deck.gl (MIT), MapLibre GL JS (Apache 2.0)
- Aanbeveling: Start met MapLibre GL JS (open source fork)

**AHN3 Tiles:**
- URL: `https://service.pdok.nl/cds/wms/ahn3_5m/wms/v1_0?request=GetTile&service=WMS`
- Formaat: RGB encoded terrain
- Resolutie: 5m voor Nederland

**Dioxus Integration:**
- Gebruik `document::eval()` voor JS bridge
- Events via window callbacks
- State management met Dioxus signals

## Files to Modify

- `crates/iou-frontend/src/pages/data_verkenner.rs` - Main map component
- `crates/iou-frontend/src/components/` - New map controls
- `crates/iou-frontend/Cargo.toml` - Add map library dependency

## Acceptance Criteria

- [ ] 3D kaart laadt met Flevoland terrein zichtbaar
- [ ] Gebruiker kan kantelen, roteren, zoomen in 3D modus
- [ ] Alle bestaande GeoJSON lagen werken nog steeds
- [ ] Performance acceptabel op moderne hardware
- [ ] Kaart werkt in Chrome, Firefox, Safari, Edge

## Definition of Done

- Code review voltooid
- Unit tests voor JS bridge functions
- Integration test met echte GeoJSON data
- Browser compatibiliteitstest
- Documentatie van API
