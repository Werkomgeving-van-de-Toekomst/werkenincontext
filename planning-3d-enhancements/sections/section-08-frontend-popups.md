Now I have all the context I need. Let me generate the section content for `section-08-frontend-popups`. Based on the plan, this section covers:

1. Adding click handlers for buildings
2. Creating XSS-safe popup content using DOM methods
3. CSS styling (dependency on section-05)
4. Property display from the backend API

The section depends on:
- section-05-frontend-css (popup styling)
- section-06-frontend-loading (dynamic building loading, property structure)</think># Section 8: Frontend Click Popups

## Overview

This section implements interactive popups that display building information when users click on 3D buildings. The popups show properties extracted by the backend (BAG ID, height, floors, construction year) and use XSS-safe DOM methods to prevent script injection from untrusted API data.

**Note:** Address is NOT available in the 3DBAG API response and has been removed from popup requirements.

## Dependencies

This section requires:
- **section-05-frontend-css**: Popup CSS styles must be defined first
- **section-06-frontend-loading**: Building source and property structure must be available
- **section-04-backend-properties**: Property extraction must be complete (bag_id, height, floors, construction_year)

## Tests

Write these tests before implementing the popup functionality.

### Manual Browser Tests

The MapLibre integration requires manual browser testing. Test the following:

**Click Handler Tests:**
- Click on a building triggers a popup
- Click outside building does not trigger a popup
- Only the top building is selected when buildings overlap
- Popup appears at the clicked location

**Popup Content Tests (XSS-Safe):**
- Popup title is set using `textContent`, not `innerHTML`
- All property values use `textContent` (no template literals with untrusted data)
- Special characters in addresses (`<`, `>`, `&`, quotes) display as text, not HTML
- Null/missing properties show "N/A" or are omitted gracefully
- Height is formatted to 1 decimal place with "m" suffix

**Popup Behavior Tests:**
- Popup has `building-popup` CSS class applied
- Popup closes when clicking elsewhere on the map
- Popup closes when the map moves
- Only one popup visible at a time

**Security Tests:**
- HTML tags in API data are not rendered (display as literal text)
- Script tags in any field do not execute
- Unicode and special characters display correctly

### Frontend Component Test

Add a test to verify the JavaScript generation in `data_verkenner.rs`:

```rust
#[cfg(test)]
mod test_popup_handler {
    use super::*;

    #[test]
    fn test_popup_script_uses_dom_methods() {
        let script = build_popup_handler_script();
        
        // Verify XSS-safe methods are used
        assert!(script.contains("textContent"), 
                "Popup must use textContent for XSS safety");
        assert!(!script.contains("innerHTML") || script.contains("innerHTML") == false, 
                "Popup must NOT use innerHTML");
    }

    #[test]
    fn test_popup_includes_all_properties() {
        let script = build_popup_handler_script();
        
        // Verify all expected properties are referenced
        assert!(script.contains("bag_id"), "Popup should show BAG ID");
        assert!(script.contains("height"), "Popup should show height");
        assert!(script.contains("floors"), "Popup should show floors");
        assert!(script.contains("construction_year"), "Popup should show construction year");
    }
}
```

## Implementation

### File: `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs`

Add the popup handler generation function. This function returns JavaScript code that:

1. Registers a click event listener on the building layer
2. Queries features at the click point
3. Creates popup content using XSS-safe DOM methods
4. Displays the popup at the click location

**Function signature:**

```rust
fn build_popup_handler_script() -> String {
    // Returns JavaScript for building click popups
}
```

**Key implementation points:**

1. **Event Listener Registration:**
   - Target the `building-3d` layer
   - Use `map.queryRenderedFeatures()` to get clicked features
   - Handle case where no features are clicked

2. **XSS-Safe Content Creation:**
   - Create container `div` with `building-popup` class
   - Use `document.createElement()` for all elements
   - Set text content using `textContent`, never `innerHTML`
   - Build content programmatically, not via template literals

3. **Property Display:**
   - Display `bag_id` (building identifier)
   - Display `height` formatted to 1 decimal place with "m" suffix
   - Display `floors` as integer
   - Display `construction_year` if available
   - Omit or show "N/A" for null/undefined values

4. **Popup Lifecycle:**
   - Close existing popup before opening new one
   - Popup auto-closes on map click (MapLibre default behavior)
   - Popup auto-closes on map move

### JavaScript Structure

The generated JavaScript should follow this pattern:

```javascript
// Register click handler for buildings
map.on('click', 'building-3d', function(e) {
    // Query features at click point
    var features = map.queryRenderedFeatures(e.point, {
        layers: ['building-3d']
    });
    
    if (features.length > 0) {
        var props = features[0].properties;
        var popup = createBuildingPopup(props);
        popup.setLngLat(e.lngLat).addTo(map);
    }
});

function createBuildingPopup(props) {
    var container = document.createElement('div');
    container.className = 'building-popup';
    
    // Title
    var title = document.createElement('h3');
    title.textContent = 'Gebouw Info';
    container.appendChild(title);
    
    // Helper function for adding rows
    function addRow(label, value) {
        if (value === null || value === undefined) return;
        var row = document.createElement('p');
        var labelEl = document.createElement('strong');
        labelEl.textContent = label + ': ';
        row.appendChild(labelEl);
        row.appendChild(document.createTextNode(String(value)));
        container.appendChild(row);
    }
    
    // Add property rows
    addRow('ID', props.bag_id);
    addRow('Hoogte', props.height ? props.height.toFixed(1) + 'm' : null);
    addRow('Verdiepingen', props.floors);
    addRow('Bouwjaar', props.construction_year);
    
    return new maplibregl.Popup().setDOMContent(container);
}
```

### Integration with Map Initialization

In the `get_map3d_init_script()` function, add the popup handler script after the building layer is defined. Ensure the script is added after the source and layer are created.

```rust
pub fn get_map3d_init_script() -> String {
    format!(r#"
    // ... existing map initialization ...
    
    // Add building layer
    map.addLayer({{ ... }});
    
    // Add popup handler
    {popup_handler_script}
    "#, popup_handler_script = build_popup_handler_script())
}
```

### CSS Dependency

Ensure the popup CSS from `section-05-frontend-css` is loaded. The CSS should be defined in:

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/styles/building_popup.css`

And referenced in the component. Expected CSS classes:

```css
.building-popup { ... }
.building-popup h3 { ... }
.building-popup p { ... }
.building-popup strong { ... }
```

## Security Considerations

**XSS Prevention:**

The popup implementation must use DOM methods exclusively. Do NOT use template literals or `innerHTML` with user-controlled data.

**Correct:**
```javascript
title.textContent = 'Gebouw Info';  // Safe
row.appendChild(document.createTextNode(String(value)));  // Safe
```

**Incorrect:**
```javascript
container.innerHTML = `<h3>${props.address}</h3>`;  // UNSAFE - XSS vulnerability
```

**Property Sanitization:**

- All values from the API are treated as untrusted
- The `textContent` property automatically escapes HTML entities
- Do not implement custom HTML escaping - use browser built-ins

## Error Handling

Handle missing or null properties gracefully:

```javascript
// Helper to safely get and format properties
function formatProperty(props, key, suffix = '') {
    var value = props[key];
    if (value === null || value === undefined) {
        return null;
    }
    return String(value) + suffix;
}

// Usage
addRow('Hoogte', formatProperty(props, 'height', 'm'));
addRow('Adres', formatProperty(props, 'address'));
```

## Testing Checklist

After implementation, verify:

- [ ] Click on building shows popup
- [ ] Popup displays all expected properties
- [ ] Popup styles are applied correctly
- [ ] Popup closes on map click
- [ ] Popup closes on map move
- [ ] Special characters display correctly (not as HTML)
- [ ] Missing properties are handled gracefully
- [ ] Height displays with 1 decimal place
- [ ] Only one popup visible at a time
- [ ] No XSS vulnerabilities (verify DOM methods only)
- [ ] Performance: popup appears instantly on click

## Related Files

- `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs` - Main implementation
- `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/styles/building_popup.css` - Popup styles (from section-05)
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/buildings_3d.rs` - Backend property extraction (from section-04)

## Actual Implementation (2025-03-07)

**File Modified:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs`

**Changes from Original Plan:**
- Added null check for `features[0].properties` before access (defensive programming)
- Popup closes on map move using `querySelectorAll` approach

**Code Review Fixes Applied:**
- Added `&& features[0].properties` check to prevent errors from malformed data

**Features Implemented:**
- Click handler on `building-3d` layer
- XSS-safe popup creation using `textContent` and `createElement`
- Display of: bag_id, height (formatted to 1 decimal), floors, construction_year
- Popup closes on map move
- 15px offset for better positioning

**Tests Added:** 5 tests covering XSS safety and functionality (all passing)
