Now I have all the context I need. Let me generate the content for section-05-frontend-css. Based on the index.md, this section is about creating CSS file for popup styling and has no dependencies on other sections.

# Section 5: Frontend CSS - Popup Styling

## Overview

This section creates the CSS styling for building popup windows that will appear when users click on 3D buildings. The CSS ensures popups are visually distinct, readable, and properly formatted. This is an independent task that can be done in parallel with backend work.

## Dependencies

None. This section can be implemented independently of other sections.

## File to Create

**`/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/styles/building_popup.css`**

Create a new CSS file for building popup styling. The file will later be imported into the map component.

## CSS Specification

Create the following CSS rules for the building popup:

```css
/* Main popup container */
.building-popup {
    padding: 12px;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    font-size: 14px;
    color: #333;
    min-width: 200px;
}

/* Popup title */
.building-popup h3 {
    margin: 0 0 8px 0;
    font-size: 16px;
    color: #1a1a1a;
    border-bottom: 1px solid #ddd;
    padding-bottom: 6px;
}

/* Property rows */
.building-popup p {
    margin: 4px 0;
    line-height: 1.4;
}

/* Property labels (strong text) */
.building-popup strong {
    color: #555;
}
```

## Design Considerations

1. **System Font Stack**: Uses `-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif` for native appearance on each platform
2. **Spacing**: 12px padding provides comfortable whitespace without being too large
3. **Typography Hierarchy**: h3 title at 16px, body text at 14px for clear visual distinction
4. **Color Contrast**: #333 for body text and #555 for labels meets accessibility standards
5. **Border**: Subtle 1px #ddd border separates title from content
6. **Minimum Width**: 200px ensures popup doesn't become too narrow for property labels

## Testing Checklist

### Visual Tests (Manual Browser Testing)
- [ ] Popup appears with correct font (system default)
- [ ] Title has bottom border separator
- [ ] Property values have proper spacing (4px margin)
- [ ] Popup is at least 200px wide
- [ ] Text colors are readable (#333, #555 on white background)

### Responsive Tests
- [ ] Popup content doesn't overflow on small screens
- [ ] Long addresses wrap properly
- [ ] Popup remains readable on mobile viewports

### Integration Tests
- [ ] CSS is loaded when map initializes (verify in section-08)
- [ ] `.building-popup` class is applied to popup container
- [ ] No conflicts with existing MapLibre popup styles

## Implementation Notes

1. This CSS file will be referenced by the popup creation code in section-08 (frontend-popups)
2. MapLibre GL JS's `Popup.setDOMContent()` method uses the `.building-popup` class
3. The CSS can be bundled with the frontend assets or included in the page stylesheet
4. No JavaScript changes are required in this section - pure CSS only

## Actual Implementation (2025-03-07)

**File Created:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/styles/building_popup.css`

**Changes from Original Plan:**
- Used CSS variables (`var(--spacing-md)`, `var(--spacing-sm)`, `var(--spacing-xs)`) instead of hardcoded pixel values for consistency with project design system
- Used semantic color variables (`var(--text-primary)`, `var(--text-secondary)`, `var(--border)`) instead of hardcoded colors
- Changed font-family to `inherit` to align with project's typography
- Added `box-sizing: border-box` for robustness in MapLibre's isolated popup container

**Rationale:** The code review identified inconsistencies with the project's existing CSS design system. Using CSS variables ensures future theming consistency and easier maintenance.