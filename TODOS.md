# TODO

## Workflow Builder Cleanup

### Delete unused workflow-builder implementation
**What:** Remove migration 050 and empty workflow-builder crates that duplicate existing Camunda + iou-core workflow infrastructure.

**Why:** The new workflow-builder schema and Rust crates overlap with existing workflow capabilities. Camunda Zeebe handles BPMN orchestration, and iou-core provides approval stage types. Building a parallel system creates duplication without clear benefit.

**Files to delete:**
- `migrations/050_workflow_builder.sql`
- `server/crates/workflow-builder/` (entire directory)

**Pros:**
- Eliminates duplicate workflow infrastructure
- Reduces maintenance burden
- Clarifies architectural direction (Camunda-first)

**Cons:**
- None — the crates are empty (no implementation lost)

**Depends on / blocked by:** None

**Context:** Eng review found that migration 050's `workflows`, `workflow_versions`, `workflow_executions`, etc. duplicate concepts already handled by Camunda Zeebe process instances and iou-core's `WorkflowExecution`, `StageInstance`. The design doc called for a Python validation prototype, but Rust implementation was started instead. Decision: scrap new implementation, use existing infrastructure.

---

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
