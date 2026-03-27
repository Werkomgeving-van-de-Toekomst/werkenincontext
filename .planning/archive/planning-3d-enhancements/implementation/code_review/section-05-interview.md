# Code Review Interview: Section 05 - Frontend CSS

## Date
2025-03-07

## Review Triage

### Items to Auto-Fix

**1. Inconsistent Spacing Units (Confidence: 85)**
- **Decision:** Auto-fix
- **Reason:** Project has established CSS variables for spacing. Using them ensures consistency and makes future theming easier.
- **Fix:** Replace literal pixel values with `var(--spacing-*)` variables

**2. Inconsistent Color Values (Confidence: 85)**
- **Decision:** Auto-fix
- **Reason:** Project uses semantic color variables. This ensures consistency across the application.
- **Fix:** Replace hardcoded colors with `var(--text-primary)`, `var(--text-secondary)`, `var(--border)`

**3. Missing Box-Sizing Reset (Confidence: 80)**
- **Decision:** Auto-fix
- **Reason:** MapLibre popups may be in isolated containers. Adding explicit box-sizing prevents layout issues.
- **Fix:** Add `box-sizing: border-box` to popup and its children

**4. Font Stack Inconsistency (Confidence: 82)**
- **Decision:** Auto-fix
- **Reason:** Using `inherit` is cleaner and aligns with project's typography.
- **Fix:** Change font-family to `inherit`

### Items to Let Go

**5. Z-index consideration (Medium Priority)**
- **Decision:** Let go
- **Reason:** MapLibre manages popup z-index internally. Adding explicit z-index could cause conflicts.

**6. Focus styles (Medium Priority)**
- **Decision:** Let go
- **Reason:** Popup currently has no interactive elements (links/buttons). Can be added in section-08 if needed.

**7. Max-width constraint (Low Priority)**
- **Decision:** Let go
- **Reason:** Current `min-width: 200px` is sufficient. Max-width can be added later if needed.

## No User Questions Required

All issues identified were clear consistency improvements with the existing codebase. No tradeoffs or architectural decisions needed user input.

## Actions to Apply

1. Update `building_popup.css` to use CSS variables for spacing and colors
2. Add box-sizing reset
3. Change font-family to inherit
