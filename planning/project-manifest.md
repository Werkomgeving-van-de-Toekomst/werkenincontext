# Project Manifest - Kaartfuncties Integratie

<!-- SPLIT_MANIFEST
{
  "splits": [
    {
      "id": "01-map-engine-3d",
      "name": "Map Engine 3D",
      "description": "Core 3D map infrastructure upgrade from Leaflet to Mapbox GL JS with AHN terrain integration",
      "dependencies": [],
      "estimated_effort": "large"
    },
    {
      "id": "02-historical-comparator",
      "name": "Historical Comparator",
      "description": "Time-based map comparison with side-by-side viewer and timeline slider",
      "dependencies": ["01-map-engine-3d"],
      "estimated_effort": "medium"
    },
    {
      "id": "03-pdok-integration",
      "name": "PDOK Integration",
      "description": "National data services integration (WMS/WMTS, BRT, BAG, TOP10NL)",
      "dependencies": ["01-map-engine-3d"],
      "estimated_effort": "medium"
    }
  ],
  "execution_order": ["01-map-engine-3d", "02-historical-comparator", "03-pdok-integration"]
}
SPLIT_MANIFEST -->

## Overview

Dit project integreert geavanceerde kaartfuncties in de Flevoland Data Verkenner, geïnspireerd op EduGIS.nl. Het project is opgesplitst in drie gefaseerde componenten.

## Splits

### 01-map-engine-3d

**Beschrijving:** Core 3D kaart infrastructuur upgrade

Omschakeling van Leaflet naar Mapbox GL JS met ondersteuning voor:
- 3D terreinweergave via AHN (Actueel Hoogtebestand Nederland)
- WebGL rendering
- 3D navigatie controls (tilt, rotate, zoom)
- Externe tile providers

**Technologie:**
- Mapbox GL JS (of open source alternatief deck.gl)
- PDOK AHN3 terrain tiles
- WASM bindings voor Dioxus

**Dependencies:** Geen

**Output:**
- Functionele 3D kaart component
- Terrein data zichtbaar in Flevoland regio
- Smooth navigatie in 3D modus

### 02-historical-comparator

**Beschrijving:** Tijdsgebaseerde kaartvergelijking

Vergelijk historische kaarten met huidige situatie via:
- Side-by-side viewer (split screen)
- Swipe mode (interactieve overlaping)
- Tijdslider voor verschillende jaren
- Topotijdreis tile integratie
- Transitie animaties tussen tijdstippen

**Technologie:**
- Custom comparator UI component
- Timeline slider widget
- Historische tile services

**Dependencies:** 01-map-engine-3d

**Output:**
- Functionele dan/nu vergelijking
- Gebruiker kan historische kaarten overlaid met moderne kaart
- Smeed transities tussen tijdstippen

### 03-pdok-integration

**Beschrijving:** Nationale data services integratie

Integratie met PDOK (Publieke Dienstverlening Op de Kaart):
- WMS/WMTS service manager
- BRT Achtergrond (basis kaart)
- BAG 3D (gebouwen)
- TOP10NL (topografie)
- Perceelsgrenzen
- Layer catalog UI
- Feature info parsing

**Technologie:**
- WMS/WMTS client library
- PDOK API endpoints
- Layer control UI

**Dependencies:** 01-map-engine-3d

**Output:**
- PDOK lagen beschikbaar in kaart
- Gebruiker kan PDOK lagen toevoegen/verwijderen
- Feature info werkt voor PDOK layers

## Execution Order

1. **01-map-engine-3d** - Eerst (foundation voor andere components)
2. **02-historical-comparator** - Parallel met 03 (na 01)
3. **03-pdok-integration** - Parallel met 02 (na 01)

## Estimated Timeline

- **01-map-engine-3d:** 1-2 weken
- **02-historical-comparator:** 1 week
- **03-pdok-integration:** 1 week

Totaal: ~3-4 weken
