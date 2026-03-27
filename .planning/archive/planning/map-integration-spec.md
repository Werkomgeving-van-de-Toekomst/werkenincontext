# Kaartfuncties Integratie - Flevoland Data Verkenner

## Projectbeschrijving

Integreer geavanceerde kaartfuncties in de bestaande Data Verkenner van Flevoland, geïnspireerd op de functionaliteit van EduGIS.nl. De huidige implementatie heeft een basis Leaflet kaart met GeoJSON lagen, maar moet worden uitgebreid met professionele GIS-tools voor educatief en onderzoek gebruik.

## Huidige Situatie

De Data Verkenner (`crates/iou-frontend/src/pages/data_verkenner.rs`) bevat:
- Basis Leaflet kaart gecentreerd op Flevoland [52.45, 5.50]
- GeoJSON lagen uit `/assets/geodata/`:
  - provinciegrens.geojson
  - cultuurhistorie.geojson
  - windturbines.geojson
  - zonneparken.geojson
  - fietsnetwerken.geojson
  - drinkwater.geojson
- Simpele layer control met togglebare lagen
- Dataset visualisatie met bar charts

## Gewenste Functionaliteit (gebaseerd op EduGIS.nl)

### 1. Kaartfuncties
- **3D Terreinweergave** - Met hoogtekaarten voor Flevoland
- **Buffer Zones** - Gebruiker kan buffers tekenen rondom features
- **Meten & Afstanden** - Lengte- en oppervlaktemetingen
- **Tekengereedschappen** - Points, lines, polygons tekenen
- **Print/Export** - Kaarten exporteren als PDF/PNG

### 2. Laag Management
- **Geavanceerde Layer Control** - Opacity, reorder, clustering
- **Tijdlijn Slider** - Historische data door de tijd heen
- **Heatmaps** - Voor data zoals verkeersintensiteit
- **Choropleth Maps** - Voor statistische data per gemeente

### 3. Datavisualisatie
- **Pop-ups met Rich Content** - Charts, images in pop-ups
- **Side Panel Info** - Klik op feature toont details in zijbalk
- **Filter & Query** - Selecteer features op basis van eigenschappen
- **Linked Views** - Kaart selectie update charts en vice versa

### 4. EduGIS Specifieke Features
- **Story Mode** - Stap-voor-stap verkenning met annotations
- **Vergelijkmodus** - Side-by-side kaartvergelijking (toen/nu)
- **Bookmarks/Views** - Opgeslagen kaartstanden
- **Embed/Focus Mode** - Fullscreen kaartweergave

## Technische Context

- **Frontend**: Dioxus 0.7 (WebAssembly)
- **Kaartbibliotheek**: Leaflet.js via wasm-bindgen
- **Data**: GeoJSON files in `/assets/geodata/`
- **Styling**: Custom CSS
- **Taal**: Nederlands

## Randvoorwaarden

1. Ondersteuning voor PDOK (Publieke Dienstverlening Op de Kaart) lagen
2. Mogelijkheid om eigen GeoJSON te uploaden
3. Responsief design voor desktop en tablet
4. Toegankelijkheid (WCAG 2.1 AA)
5. Performance met grote datasets

## Doelgroep

- Leerlingen in het voortgezet onderwijs
- Docenten aardrijkskunde
- Provinciaal onderzoekers
- Beleidsmakers Flevoland

## Succescriteria

- Studenten kunnen zelfstandig kaartanalyses uitvoeren
- Docenten kunnen kant-en-klare lessen met kaarten maken
- Performance blijft goed met >1000 features
- Alle functies werken in moderne browsers (Chrome, Firefox, Edge, Safari)
