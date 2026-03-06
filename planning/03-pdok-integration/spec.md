# 03-pdok-integration

## Description

National data services integration with PDOK (Publieke Dienstverlening Op de Kaart).

## Background

PDOK is de centrale voorziening voor geografische data van de Nederlandse overheid. Integratie met PDOK geeft toegang tot actuele, officiële kaartlagen.

## Requirements

### WMS/WMTS Client
- [ ] WMS 1.3.0 support
- [ ] WMTS 1.0.0 support
- [ ] XYZ tile support
- [ ] Async tile loading
- [ ] Error handling en retry

### PDOK Services
- [ ] BRT Achtergrond (basis kaart)
- [ ] BRT Achtergrond - Grijs
- [ ] BAG 3D (gebouwen)
- [ ] TOP10NL (topografie)
- [ ] TOP100NL (overzicht)
- [ ] Perceelsgrenzen (kadastrale grenzen)
- [ ] AHN3 (hoogtekaart)

### Layer Catalog
- [ ] Beschikbare lagen tonen
- [ ] Toggle lagen aan/uit
- [ ] Layer opacity control
- [ ] Layer reorder (drag & drop)
- [ ] Groepeeren van lagen

### Feature Info
- [ ] Click op feature toont informatie
- [ ] WMS GetFeatureInfo parsing
- [ ] Custom pop-up styling
- [ ] Multi-feature selectie

### Configuration
- [ ] PDOK endpoint configuratie
- [ ] Custom layer definitions
- [ ] Cache settings
- [ ] Tile size config

## Technical Notes

**PDOK Service URLs:**
```
WMTS BRT: https://service.pdok.nl/cbs/wmts/v1_0
WMS BAG: https://service.pdok.nl/lv/bag/wms/v1_0
WMS TOP10NL: https://service.pdok.nl/tms/1.0.0/top10nl/
WMTS AHN: https://service.pdok.nl/ahn/wmts/v1_0
```

**Layer Configuration:**
```rust
struct PdokLayer {
    id: String,
    name: String,
    service_type: ServiceType,  // WMS, WMTS, XYZ
    url: String,
    layers: Vec<String>,  // WMS layer names
    format: String,
    min_zoom: u8,
    max_zoom: u8,
    attribution: String,
}
```

**WMTS TileMatrix:**
- Nederlandse raster indeling
- Matrix levels 0-15
- Tile format: image/png

## Files to Create

- `crates/iou-frontend/src/services/pdok_client.rs`
- `crates/iou-frontend/src/components/layer_catalog.rs`
- `crates/iou-frontend/src/components/layer_control.rs`

## Dependencies

- `01-map-engine-3d` - Vereist de nieuwe map engine

## Acceptance Criteria

- [ ] PDOK BRT laag laadt als basis kaart
- [ ] Gebruiker kan PDOK lagen toevoegen via catalog
- [ ] Layer toggle werkt direct
- [ ] Feature info toont relevante data
- [ ] Error handling werkt bij service outage

## Definition of Done

- Code review voltooid
- Unit tests voor WMS/WMTS client
- Integration test met PDOK services
- Performance test met meerdere lagen
- Documentatie van beschikbare PDOK lagen
