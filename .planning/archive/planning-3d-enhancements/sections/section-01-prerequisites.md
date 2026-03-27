Now I have all the context I need. Let me generate the content for section-01-prerequisites based on the information from the plan files.

---

# Section 1: Prerequisites - 3DBAG API Verification

## Overview

This section contains manual verification tasks that must be completed before implementing any other sections. The 3DBAG API response format determines which fields are available for building properties (ID, address, construction year). Implementing without verification risks using non-existent fields or incorrect attribute names.

**Why this matters:** The current implementation plan assumes certain CityJSON fields exist. If these assumptions are wrong, subsequent sections will fail or produce incorrect results.

**Dependencies:** None - this should be completed first.

## Context

The backend proxy at `/api/buildings-3d` forwards requests to the 3DBAG external API and converts CityJSON responses to GeoJSON. The current code extracts these fields:

- `b3_h_dak_max` - Roof height
- `b3_h_maaiveld` - Ground level
- `b3_bouwlagen` - Floor count

For popup functionality (section-08), we need to verify additional fields:

1. **BAG ID** - Unique building identifier
2. **Address information** - Street, house number, postal code, city
3. **Construction year** - When building was constructed

These fields may or may not be available in the standard 3DBAG API response.

## Tests

### API Verification Tests (Manual)

These are manual tests to be run before implementation. Record results for reference in subsequent sections.

**Test 1: Basic API Response Structure**

```bash
curl "https://api.3dbag.nl/collections/pand/items?bbox=150000,470000,155000,475000&limit=1"
```

Expected to verify:
- [ ] Response returns valid GeoJSON/JSON structure
- [ ] Response contains `features` array
- [ ] Each feature has `properties` object

**Test 2: Verify Known Fields**

```bash
curl "https://api.3dbag.nl/collections/pand/items?bbox=150000,470000,155000,475000&limit=1" | jq '.features[0].properties | keys'
```

Expected to verify:
- [ ] `b3_h_dak_max` exists and is numeric
- [ ] `b3_h_maaiveld` exists and is numeric
- [ ] `b3_bouwlagen` exists and is numeric

**Test 3: Find BAG ID Field**

```bash
curl "https://api.3dbag.nl/collections/pand/items?bbox=150000,470000,155000,475000&limit=1" | jq '.features[0].properties'
```

Expected to verify:
- [ ] Identify the field containing BAG/pand identifier
- [ ] Document the exact field name (e.g., `pand_id`, `bag_id`, `identificatie`)

**Test 4: Check Address Availability**

```bash
curl "https://api.3dbag.nl/collections/pand/items?bbox=150000,470000,155000,475000&limit=1" | jq '.features[0].properties'
```

Expected to verify:
- [ ] Document if address fields exist
- [ ] If yes, record exact field names (e.g., `straatnaam`, `huisnummer`, `postcode`, `woonplaats`)
- [ ] If no, plan to derive from other sources or omit from popup

**Test 5: Check Construction Year**

```bash
curl "https://api.3dbag.nl/collections/pand/items?bbox=150000,470000,155000,475000&limit=1" | jq '.features[0].properties'
```

Expected to verify:
- [ ] Document if construction year field exists
- [ ] Record exact field name if available
- [ ] If not available, plan to omit from popup

**Test 6: Bbox Parameter Format**

```bash
curl "https://api.3dbag.nl/collections/pand/items?bbox=150000,470000,155000,475000&limit=1"
```

Expected to verify:
- [ ] Confirm bbox accepts RD coordinates (minX,minY,maxX,maxY)
- [ ] Note any alternative bbox formats (e.g., `bbox-crs` parameter)

## Implementation Tasks

### Task 1: Execute API Verification

Run each of the tests above and document findings.

Create a verification record file (for reference, not committed to repo):

```
File: /Users/marc/Projecten/iou-modern/planning-3d-enhancements/api-verification-notes.md
```

Document:
- Actual field names found in 3DBAG response
- Fields that are NOT available
- Any unexpected response structure
- Coordinate system used for bbox parameter

### Task 2: Update Implementation Plan Based on Findings

Based on API verification results, adjust subsequent sections:

**If BAG ID field found:**
- Note exact field name for use in section-04-backend-properties

**If BAG ID field NOT found:**
- Use feature ID from CityJSON as fallback
- Document this decision

**If address fields found:**
- Record exact field names for section-04-backend-properties
- Plan to combine into single address string

**If address fields NOT found:**
- Remove address from popup requirements
- Update section-08-frontend-popups accordingly

**If construction year found:**
- Record exact field name for section-04-backend-properties

**If construction year NOT found:**
- Remove from popup requirements
- Update section-08-frontend-popups accordingly

### Task 3: Verify Coordinate System

Confirm the bbox format expected by 3DBAG API:

**Current assumption:** 3DBAG accepts RD (Dutch national grid) coordinates in `bbox` parameter

**To verify:**
- The test curl command uses RD coordinates (150000,470000 area is Flevoland in RD)
- If API also supports WGS84, document alternative parameter
- This informs section-03-backend-wgs84 implementation

## Decision Points

Based on API verification, make these decisions before proceeding:

1. **BAG ID source:** Which field provides unique building identifier?
2. **Address display:** Show full address, partial address, or omit?
3. **Construction year:** Include in popup or omit?
4. **Coordinate conversion:** Does 3DBAG accept WGS84 bbox directly, or must backend convert?

## File References

No code files are modified in this section. This is a verification-only section.

**Files to reference for understanding current implementation:**

- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/buildings_3d.rs` - Current 3DBAG proxy implementation

## Next Sections

After completing this section:

- **section-02-backend-coordinate** - Can proceed in parallel (does not depend on API field verification)
- **section-04-backend-properties** - Depends on verified field names from this section

## Success Criteria

This section is complete when:

- [x] All API verification tests have been run
- [x] Field names are documented (BAG ID, address, construction year)
- [x] Missing fields are identified and alternative approaches planned
- [x] Coordinate system requirements are confirmed
- [x] Implementation plan adjustments are noted for relevant sections

## Actual Findings (2025-03-07)

**BAG ID:** Found as `identificatie` attribute in CityObjects attributes
**Construction Year:** Found as `oorspronkelijkbouwjaar` attribute
**Address:** NOT AVAILABLE in 3DBAG API - omit from popups
**Coordinate System:** 3DBAG accepts RD coordinates only - backend conversion required

**Verification notes saved to:** `/Users/marc/Projecten/iou-modern/planning-3d-enhancements/api-verification-notes.md`