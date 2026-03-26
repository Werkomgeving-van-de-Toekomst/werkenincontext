# Deep Project Interview - Kaartfuncties Integratie

**Datum:** 2026-03-03
**Project:** Kaartfuncties integreren in Flevoland Data Verkenner (EduGIS.nl functionaliteit)

## Vragen en Antwoorden

### V1: 3D Ondersteuning
**Vraag:** Moet de kaart 3D terreinweergave ondersteunen (zoals EduGIS), of is 2D voldoende met eventueel 3D gebouwen?

**Antwoord:** Ja, 3D terrein

**Implicaties:**
- Mapbox GL JS of deck.gl nodig (Leaflet heeft geen native 3D)
- AHN (Actueel Hoogtebestand Nederland) data van PDOK voor hoogte
- WebGL renderer in browser
- Meer complexe data structuren voor terrain meshes

### V2: Gegevensbronnen
**Vraag:** Welke gegevensbronnen moeten beschikbaar zijn naast de bestaande GeoJSON bestanden?

**Antwoord:** PDOK services, Lokale bestanden, Historische kaarten

**Implicaties:**
- **PDOK:** WMS/WMTS services voor BRT, BAG, TOP10NL, AHN, etc.
- **Lokaal:** Bestaande GeoJSON workflow behouden/uitbreiden
- **Historisch:** Topotijdreis integratie of eigen historische kaart viewer

### V3: Prioriteit
**Vraag:** Welke EduGIS functies hebben de hoogste prioriteit voor de eerste implementatie?

**Antwoord:** Historische vergelijking

**Implicaties:**
- Side-by-side of swipe kaartweergave (toen/nu)
- Tijdslider voor historische data
- Transitie animaties
- Historische kaart tiles laden

## Samenvatting Eisen

### Must Have (Eerste Implementatie)
1. **3D Terreinweergave** - Volledige 3D terrain met hoogtekaarten
2. **Historische Vergelijking** - Toen/nu functionaliteit met swipe
3. **PDOK Integratie** - WMS/WMTS lagen
4. **Lokale GeoJSON** - Bestaande functionaliteit behouden

### Nice to Have (Toekomstige Iteraties)
- Tekengereedschappen (draw tools)
- Meten & bufferzones
- Print/Export functionaliteit
- 3D gebouwen (BAG 3D)
- Heatmaps & clustering

## Technische Architectuur

### Kaart Engine
- **Optie 1:** Mapbox GL JS - Native 3D, maar vereist API key
- **Optie 2:** deck.gl met Mapbox tiles - Open source alternatief
- **Optie 3:** CesiumJS - Volledige 3D globe, maar zwaar
- **Aanbeveling:** Mapbox GL JS (beste balans performance/features)

### Data Lagen
| Type | Bron | Formaat |
|------|------|---------|
| Terrein 3D | PDOK AHN3 | Terrain tiles |
| Basis kaart | PDOK BRT Achtergrond | WMTS |
| Gebouwen 3D | PDOK BAG 3D | Vector tiles |
| Historisch | Topotijdreis | XYZ tiles |
| Features | Lokaal | GeoJSON |

## Project Scope

De scope is gefocust op:
1. **3D Terrain Engine** - Implementatie van hoogtekaarten
2. **Historische Comparator** - Side-by-side vergelijking
3. **PDOK Layer Manager** - Integratie nationale kaartdiensten
4. **UI Components** - Controls voor 3D navigatie, tijdslider
