# Code Review Interview: Section 04 - Content Agent

## Date
2026-03-01

## Triage Summary

### Auto-Fixes Applied

#### 1. SectionMetadata Length Calculation (Medium)
**Issue:** The `length` field is hardcoded to 0.

**Decision:** Implement basic character counting for sections by tracking section boundaries during parsing.

#### 2. Default Section Name Constant (Low)
**Issue:** Magic string "content" for default section.

**Decision:** Use a constant `DEFAULT_SECTION_NAME`.

#### 3. AI Section Comments (Low)
**Issue:** Hardcoded Dutch section names without flexibility.

**Decision:** Add comment explaining Dutch is the default and note where to add i18n support.

### User Interview Items

### 1. ResearchContext Missing suggested_sections Field (High Priority)

**Issue:** The implementation uses `research.optional_sections` to determine which optional sections to include, but this is semantically wrong. The plan calls for `suggested_sections` to indicate WHICH optional sections should be included for a specific document.

**Current code (content.rs:263-266):**
```rust
let should_include = research.optional_sections.contains(section_name);
```

**Options:**
1. Add `suggested_sections: Vec<String>` field to ResearchContext in research.rs
2. Keep current approach and treat `optional_sections` as both "available" and "suggested"
3. Add a separate mechanism to track section inclusion decisions

**Recommendation:** Add `suggested_sections` to ResearchContext for semantic clarity.

### 2. VariableSource Type Duplication (High Priority)

**Issue:** Two different `VariableSource` enums exist:
- `iou_core::document::VariableSource` (used by TemplateVariable)
- `iou_ai::agents::research::VariableSource` (used by TemplateVariableSuggestion)

**Options:**
1. Consolidate to use only `iou_core::document::VariableSource`
2. Keep both and add conversion between them
3. Rename one to avoid confusion (e.g., `SuggestionSource`)

**Recommendation:** Consolidate to use `iou_core::document::VariableSource` everywhere.

## Decisions Made

### Decision 1: Add suggested_sections to ResearchContext
**Rationale:** The current approach of using `optional_sections` for both availability AND inclusion decisions is semantically unclear. Adding `suggested_sections` allows the Research Agent to explicitly recommend which optional sections should be included.

**Action Required:**
1. Add `suggested_sections: Vec<String>` to ResearchContext
2. Update conditional section logic to use `suggested_sections`

### Decision 2: Consolidate VariableSource Types
**Rationale:** Having two separate `VariableSource` enums causes type confusion and potential bugs.

**Action Required:**
1. Remove `VariableSource` from agents::research module
2. Use `iou_core::document::VariableSource` everywhere
3. Update TemplateVariableSuggestion to use core type

### Auto-Fixes to Apply

1. Add section length calculation
2. Add DEFAULT_SECTION_NAME constant
3. Update conditional section logic to use suggested_sections
