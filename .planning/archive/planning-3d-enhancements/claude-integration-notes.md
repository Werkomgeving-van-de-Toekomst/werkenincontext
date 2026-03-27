# Integration Notes: Opus Review Feedback

## Decisions: What to Integrate

### ✅ Integrating (Critical Issues)

**P0-1: Coordinate Conversion Accuracy**
- **Issue:** Simplified WGS84 to RD formula causes significant positioning errors
- **Action:** Enable `proj` crate on backend (currently hardcoded to `false` on line 83)
- **Reasoning:** Accurate positioning is essential for any mapping application. The backend already has proper conversion via `proj` crate - we should use it.

**P0-2: 3DBAG Field Name Verification**
- **Issue:** Plan assumes field names (bag_id, address, construction_year) without verification
- **Action:** Add prerequisite step to verify 3DBAG API response format before implementation
- **Reasoning:** Implementing against unverified API contracts causes rework.

**P1-1: Error Handling**
- **Issue:** No handling for API failures, timeouts, empty responses
- **Action:** Add error handling section to plan
- **Reasoning:** Production code must handle failure gracefully.

**P1-2: State Management for Dynamic Loading**
- **Issue:** No tracking of last-fetched bbox, cleanup of old data
- **Action:** Add state management requirements
- **Reasoning:** Prevents memory leaks and duplicate requests.

**P2-1: XSS Prevention in Popups**
- **Issue:** Raw data insertion creates XSS vulnerability
- **Action:** Specify safe HTML construction approach
- **Reasoning:** Security is non-negotiable.

**P2-2: Popup CSS**
- **Issue:** No plan for popup styling
- **Action:** Add CSS requirement
- **Reasoning:** Usable UI needs styling.

### ❌ Not Integrating (With Reasons)

**Performance: Progressive Loading**
- **Feedback:** Implement progressive rendering as buildings arrive
- **Reason:** Out of scope for this enhancement. Current scope is synchronous batch loading. Can be future enhancement.

**Browser Compatibility Testing**
- **Feedback:** Test Safari, mobile, older browsers
- **Reason:** IOU Modern likely targets modern browsers. This is general QA, not specific to this feature.

**Accessibility: ARIA Labels**
- **Feedback:** Add ARIA labels, keyboard navigation
- **Reason:** Valid concern but broader than this feature. Should be addressed in a dedicated accessibility pass.

**Internationalization**
- **Feedback:** Make Dutch strings translatable
- **Reason:** The application already uses Dutch strings. This is a broader architectural decision.

**Density-Based Loading**
- **Feedback:** Use density-based limit instead of fixed 150
- **Reason:** Adds significant complexity. Fixed limit is reasonable for initial enhancement.

## Plan Modifications

### New Section Added: "Prerequisites"
- Verify 3DBAG API response format

### Section 1 Modified: Dynamic Loading
- Replace simplified coordinate conversion with backend `proj` crate approach
- Add state management requirements (track last bbox, cleanup old data)
- Add error handling subsection

### Section 3 Modified: Click Popups
- Specify safe HTML construction to prevent XSS
- Add CSS requirement
- Verify actual 3DBAG field names

### Section 5 Expanded: Testing
- Add error scenario tests
- Add memory management tests
- Add coordinate accuracy tests

## Files to Modify

**Added:**
- `crates/iou-api/src/routes/buildings_3d.rs` - Enable `proj` crate

**Modified:**
- Multiple sections updated as described above
