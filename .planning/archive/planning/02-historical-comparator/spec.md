# 02-historical-comparator

## Description

Time-based map comparison component with side-by-side viewer and timeline slider.

## Background

EduGIS biedt de mogelijkheid om historische kaarten te vergelijken met de huidige situatie (Topotijdreis). Deze functionaliteit moet worden toegevoegd aan de Flevoland Data Verkenner.

## Requirements

### Viewer Modes
- [ ] Side-by-side mode (twee kaarten naast elkaar)
- [ ] Swipe mode (interactieve overlaping)
- [ ] Fade mode (transparantie slider)
- [ ] Toggle tussen modi

### Timeline Control
- [ ] Jaar slider (beschikbaar jaren)
- [ ] Play/pause animatie
- [ ] Speed control (1x, 2x, 4x)
- [ ] Jaar markers (belangrijke momenten)

### Historical Data Sources
- [ ] Topotijdreis tile integration
- [ ] Eigen historische GeoJSON (indien beschikbaar)
- [ ] PDOK historische luchtfoto's
- [ ] Configurabele tile sources

### Transitions
- [ ] Smooth crossfade tussen jaren
- [ ] Optional transition animations
- [ ] Loading states tijdens tile fetch

### User Experience
- [ ] Synchroniseer map views (zoom, pan)
- [ ] Highlight veranderingen tussen jaren
- [ ] Minimap voor locatie context
- [ ] Shareable URL parameters (jaar, modus)

## Technical Notes

**Topotijdreis API:**
- URL: `https://www.topotijdreis.nl/kml/tiles/`
- Format: XYZ tiles
- Jaren: ~1815 tot heden

**Swipe Mode Implementation:**
- CSS clip-path of mask
- JavaScript event handlers voor drag
- Sync met map pan/zoom

**State Management:**
```rust
struct HistoricalState {
    mode: CompareMode,  // SideBySide, Swipe, Fade
    year_left: u32,
    year_right: u32,
    is_playing: bool,
    speed: u32,  // ms per frame
}
```

## Files to Create

- `crates/iou-frontend/src/components/historical_comparator.rs`
- `crates/iou-frontend/src/components/timeline_slider.rs`
- `crates/iou-frontend/src/pages/historical_viewer.rs`

## Dependencies

- `01-map-engine-3d` - Vereist de nieuwe map engine

## Acceptance Criteria

- [ ] Gebruiker kan twee jaren selecteren en vergelijken
- [ ] Swipe mode werkt soepel met muis en touch
- [ ] Animatie speelt alle jaren af
- [ ] Kaarten blijven gesynchroniseerd tijdens pan/zoom
- [ ] Loading states zijn duidelijk

## Definition of Done

- Code review voltooid
- Component tests voor timeline slider
- Integration test met echte Topotijdreis tiles
- Touch device testing
- Performance test met grote tile sets
