# Code Review Interview: Section 06 - Frontend Dynamic Loading

## Date
2025-03-07

## Review Triage

### Critical Issues - Auto-Fix

**1. Race Condition in State Updates (Confidence: 95)**
- **Decision:** Auto-fix
- **Reason:** This is a real race condition where `lastFetchedBbox` is updated before fetch completes, causing stale data issues.
- **Fix:** Move `lastFetchedBbox = bbox` inside `fetchBuildings()` after successful data update.

**2. Missing Fetch Abort Controller (Confidence: 90)**
- **Decision:** Auto-fix
- **Reason:** Without abort controller, rapid map movements cause wasted network requests and potential data overwrites.
- **Fix:** Add `abortController` state variable and use it to cancel pending fetches.

### Important Issues

**3. Incomplete Position Change Detection (Confidence: 85)**
- **Decision:** Auto-fix
- **Reason:** Users can pan far outside loaded area without triggering new fetch because only dimension changes are checked.
- **Fix:** Add center position comparison to `shouldFetch()`.

**4. Unused Error Message (Confidence: 82)**
- **Decision:** Let go
- **Reason:** Error logging to console is sufficient for this scope. UI notification system is out of scope.
- **Note:** Can be added in future if needed.

**5. Popup Clearing May Be Incomplete (Confidence: 80)**
- **Decision:** Let go
- **Reason:** Popup creation code is in section-08. Will address popup tracking there.
- **Note:** Revisit in section-08 when implementing click handlers.

### Minor Issues

**6. Test Coverage Could Be More Specific (Confidence: 75)**
- **Decision:** Let go
- **Reason:** String-based testing is appropriate for embedded JavaScript in Rust. More complex testing would require significant refactoring.

**7. Hardcoded Limit Value (Confidence: 72)**
- **Decision:** Auto-fix
- **Reason:** Using a constant is good practice and makes the code more maintainable.

**8. Missing Loading State Indication (Confidence: 78)**
- **Decision:** Let go
- **Reason:** Loading indicators are UX improvements that can be added later. Not critical for functionality.

## No User Questions Required

All issues identified were clear technical improvements with well-defined fixes. No tradeoffs or architectural decisions needed user input.

## Actions to Apply

1. Move `lastFetchedBbox` update to after successful fetch (fix race condition)
2. Add AbortController for request cancellation
3. Add center position comparison to `shouldFetch()`
4. Extract `limit=150` to a constant
