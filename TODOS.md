# TODO

## Validation Prototype (from design doc)

## Validation Prototype (from design doc)

### Build NL-to-BPMN validation prototype
**What:** 48-hour Python/FastAPI prototype to validate natural language to BPMN translation quality.

**Why:** Before committing to any workflow builder implementation, validate that AI-generated workflows match what government workers mean. Test with 5+ actual users.

**Spec:** See design doc at `~/.gstack/projects/Werkomgeving-van-de-Toekomst-werkenincontext/marc-main-workflow-builder-validation-design-20260420-181643.md`

**Pros:**
- Fastest path to learning (48 hours vs. weeks)
- Real user validation before heavy investment
- Minimal commitment — single HTML file

**Cons:**
- Not production-ready (would need rebuild in Rust if successful)

**Depends on / blocked by:** None (can start immediately)

**Context:** This was the APPROVED approach in the design doc. The Rust implementation that was started diverged from this plan. If NL-to-BPMN validation succeeds, THEN consider whether to extend existing Camunda workflows or build new AI integration.

## Pre-existing Test Failures

### Priority: P0
**Title:** Fix Rust test suite compilation errors

**Description:**
The test suite has pre-existing compilation errors that prevent cargo test from running:
- Unresolved imports: `iou_core::tag::TagRepository`, `iou_core::category::CategoryRepository`
- Missing enum variants: `TagType::User`, `TagType::Domain`
- Type annotation errors in test files

**Branch noticed on:** feature/metadata-registry-context-aware
**Also affects:** main branch (9 errors on main, 112 cascading errors on feature)

**Files affected:**
- `crates/iou-core/tests/` (test files)
- Related modules: tag, category, graphrag

**Next steps:**
1. Fix module exports to include TagRepository, CategoryRepository
2. Add missing TagType enum variants
3. Fix type annotation errors
