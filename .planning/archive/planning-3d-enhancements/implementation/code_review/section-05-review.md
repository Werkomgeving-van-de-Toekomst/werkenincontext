# Code Review: Section 05 - Frontend CSS (Popup Styling)

## Review Scope

Reviewing CSS changes for the building popup component at:
- `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/styles/building_popup.css`
- Compared against existing styles in `/Users/marc/Projecten/iou-modern/crates/iou-frontend/assets/style.css`

---

## Critical Issues (90-100)

**No critical issues identified.**

---

## Important Issues (80-89)

### 1. Inconsistent Spacing Units - Confidence: 85

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/styles/building_popup.css`

**Lines:** 20, 29, 38

**Issue:** The new CSS uses literal pixel values (`12px`, `8px`, `4px`, `6px`) instead of the CSS custom properties defined in the main stylesheet. The project has established spacing variables (`--spacing-xs`, `--spacing-sm`, `--spacing-md`) for consistency.

**Existing pattern from style.css:**
```css
/* Spacing */
--spacing-xs: 4px;
--spacing-sm: 8px;
--spacing-md: 16px;
--spacing-lg: 24px;
```

**Suggested fix:**
```css
.building-popup {
    padding: var(--spacing-md);  /* 16px instead of 12px */
}

.building-popup h3 {
    margin: 0 0 var(--spacing-sm) 0;  /* 8px */
    padding-bottom: var(--spacing-xs);  /* 4px instead of 6px */
}

.building-popup p {
    margin: var(--spacing-xs) 0;  /* 4px */
}
```

---

### 2. Inconsistent Color Values - Confidence: 85

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/styles/building_popup.css`

**Lines:** 23, 31, 44

**Issue:** Hardcoded colors (`#333`, `#1a1a1a`, `#555`, `#ddd`) are used instead of the semantic color tokens defined in the project's CSS variables.

**Existing pattern from style.css:**
```css
/* Neutrals */
--background: #f5f7fa;
--surface: #ffffff;
--text-primary: #333333;
--text-secondary: #666666;
--border: #e0e0e0;
```

**Suggested fix:**
```css
.building-popup {
    color: var(--text-primary);
}

.building-popup h3 {
    color: var(--text-primary);  /* or darken with custom property */
    border-bottom: 1px solid var(--border);
}

.building-popup strong {
    color: var(--text-secondary);
}
```

---

### 3. Missing Box-Sizing Reset Context - Confidence: 80

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/styles/building_popup.css`

**Issue:** The main stylesheet has a global box-sizing reset (`* { box-sizing: border-box; }`), but MapLibre GL JS popups inject content into shadow DOM or isolated containers. Depending on MapLibre's implementation, the popup might not inherit this reset, potentially causing layout issues.

**Suggested fix:**
```css
.building-popup,
.building-popup * {
    box-sizing: border-box;
}
```

---

### 4. Font Stack Inconsistency - Confidence: 82

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/styles/building_popup.css`

**Lines:** 21

**Issue:** The font stack defined (`-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif`) differs from the project's standard font stack defined in style.css which uses `'Segoe UI', Tahoma, Geneva, Verdana, sans-serif`.

**Existing pattern from style.css:**
```css
body {
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
}
```

**Suggested fix:** Either inherit from body or use the project's standard:
```css
.building-popup {
    font-family: inherit;  /* Will use body font */
}
```

---

## Positive Observations

1. **Good color contrast**: The colors (#333 on white, #555 for labels) provide adequate contrast for accessibility (WCAG AA compliant).

2. **Appropriate sizing**: The 14px base font size and 16px title size provide good readability hierarchy.

3. **Minimum width constraint**: The `min-width: 200px` prevents the popup from becoming too narrow on smaller viewports.

4. **Clean, minimal styling**: The styles are appropriately simple for a map popup which should be lightweight and unobtrusive.

---

## Recommendations

### High Priority

1. **Adopt CSS custom properties** for colors and spacing to maintain consistency with the project's design system.

2. **Add box-sizing reset** for robustness in the isolated popup context.

3. **Use inherited font family** to align with the project's typography.

### Medium Priority

4. **Consider adding a z-index** in case of layer conflicts with other map controls:
   ```css
   .building-popup {
       z-index: 1000;
   }
   ```

5. **Add focus styles** if the popup contains interactive elements:
   ```css
   .building-popup a:focus,
   .building-popup button:focus {
       outline: 2px solid var(--flevoland-blauw);
       outline-offset: 2px;
   }
   ```

### Low Priority

6. **Consider adding a max-width** to prevent excessive width on large screens:
   ```css
   .building-popup {
       max-width: 300px;
   }
   ```

---

## Summary

The CSS implementation is functional and follows basic accessibility principles. The main concerns are inconsistency with the project's existing design system (CSS custom properties for colors/spacing/fonts). These inconsistencies could make future maintenance and theming more difficult.

**Overall Assessment:** Approved with recommended improvements for consistency.
