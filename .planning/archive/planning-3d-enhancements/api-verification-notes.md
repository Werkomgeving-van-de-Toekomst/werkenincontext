# 3DBAG API Verification Notes

**Date:** 2025-03-07
**API:** https://api.3dbag.nl/collections/pand/items

## Findings

### Verified Fields

| Field | API Attribute | Type | Notes |
|-------|---------------|------|-------|
| Roof height | `b3_h_dak_max` | numeric | Maximum roof height |
| Ground level | `b3_h_maaiveld` | numeric | Ground elevation |
| Floor count | `b3_bouwlagen` | numeric | Number of floors |
| **BAG ID** | `identificatie` | string | Unique building identifier |
| **Construction year** | `oorspronkelijkbouwjaar` | numeric | Original construction year |

### BAG ID Location

The BAG ID is available in TWO places:
1. **CityObjects key**: `"NL.IMBAG.Pand.{id}"` - Full BAG ID with namespace
2. **Attributes.identificatie**: Just the numeric ID part (e.g., "0308100000001716")

**Recommendation:** Use the `identificatie` attribute for cleaner display, or prefix with "NL.IMBAG.Pand." if full format needed.

### Address Fields

**Status:** NOT AVAILABLE in 3DBAG API response

The 3DBAG API does NOT include address information (street, house number, postal code, city).
This data would need to come from a separate BAG API lookup or be omitted from popups.

**Decision:** Omit address from popup requirements. Update section-08 accordingly.

### Construction Year

**Status:** AVAILABLE as `oorspronkelijkbouwjaar`

Field contains the original construction year. This can be displayed in popups.

### Coordinate System

The API accepts RD (Dutch national grid) coordinates in the `bbox` parameter:
- Format: `bbox=minX,minY,maxX,maxY`
- Tested bbox: `150000,470000,155000,475000` (Flevoland area in RD)
- No WGS84 bbox support detected

**Implication:** Section 03 (backend WGS84 endpoint) must convert coordinates.

## Data Structure

```
features[0]
├── id: string
├── type: "Feature"
├── CityObjects: object
│   └── "NL.IMBAG.Pand.{id}": object
│       └── attributes: object
│           ├── identificatie: string        ← BAG ID
│           ├── oorspronkelijkbouwjaar: int  ← Construction year
│           ├── b3_h_dak_max: float          ← Roof height
│           ├── b3_h_maaiveld: float         ← Ground level
│           └── b3_bouwlagen: int            ← Floor count
└── vertices: array
```

## Implications for Implementation

### Section 04 (Backend Properties)
- Extract `identificatie` as `bag_id`
- Extract `oorspronkelijkbouwjaar` as `construction_year`
- **DO NOT** extract address (not available)

### Section 08 (Frontend Popups)
- Display `bag_id`
- Display `height`
- Display `floors`
- Display `construction_year`
- **DO NOT** display address field

### Section 03 (WGS84 Endpoint)
- Must implement WGS84 to RD conversion
- 3DBAG API only accepts RD coordinates
