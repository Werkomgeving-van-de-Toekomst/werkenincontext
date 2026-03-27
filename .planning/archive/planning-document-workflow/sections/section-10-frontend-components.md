Now I have all the context needed. Let me generate the section content for `section-10-frontend-components`.

---

# Section 10: Frontend Components

## Overview

This section implements the Dioxus WASM frontend components that expose the enhanced document workflow features to users. The components integrate with the API endpoints from section 9 to provide visual workflow tracking, diff visualization, delegation management, and version history.

## Dependencies

This section depends on:
- **section-09-api-endpoints**: All API routes must be implemented before frontend components can consume them
- **section-08-version-storage**: Version history requires storage service
- **section-06-delegation-system**: Delegation manager requires delegation API
- **section-07-sla-escalation**: Countdown timers require SLA calculation from backend

## Files to Create/Modify

- `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/approval_queue.rs` (modify)
- `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/workflow_stage_tracker.rs` (new)
- `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/diff_viewer.rs` (new)
- `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/delegation_manager.rs` (new)
- `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/version_history.rs` (new)
- `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/api/mod.rs` (modify - add API client methods)

## Frontend Architecture

The frontend uses Dioxus (Rust WASM framework). All components follow the pattern:

```rust
// Standard component pattern
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct ComponentProps {
    pub document_id: Uuid,
    // ... other props
}

pub fn Component(cx: Scope<ComponentProps>) -> Element {
    // State management with use_state_hook
    // API calls with use_future
    // Render UI with rsx!
}
```

## 10.1 Workflow Stage Tracker

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/workflow_stage_tracker.rs`

### Purpose

Visualizes multi-stage approval workflow progress as a horizontal stepper. Shows completed stages, current stage in progress, and pending stages. Includes countdown timers, delegation badges, and escalation indicators.

### Props

```rust
#[derive(Props, PartialEq, Clone)]
pub struct StageTrackerProps {
    pub document_id: Uuid,
    pub stages: Vec<StageView>,
    pub current_stage_id: Option<String>,
    pub on_stage_click: Option<EventHandler<String>>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct StageView {
    pub stage_id: String,
    pub stage_name: String,
    pub stage_order: i32,
    pub status: StageStatus,
    pub deadline: Option<DateTime<Utc>>,
    pub approvers: Vec<ApproverView>,
    pub escalations: Vec<EscalationView>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ApproverView {
    pub user_id: Uuid,
    pub name: String,
    pub decision: Option<ApprovalDecision>,
    pub delegated_from: Option<Uuid>,
    pub delegated_from_name: Option<String>,
    pub responded_at: Option<DateTime<Utc>>,
}
```

### Component Structure

```rust
pub fn WorkflowStageTracker(cx: Scope<StageTrackerProps>) -> Element {
    // Render horizontal stepper
    // Each stage shows:
    //   - Status icon (checkmark for completed, spinner for in-progress, circle for pending)
    //   - Stage name
    //   - Countdown timer (if deadline set and stage in progress)
    //   - Approver avatars with status
    //   - Delegation badge (if any approver is delegated)
    //   - Escalation icon (if escalated)
    
    cx.render(rsx! {
        div { class: "stage-tracker",
            {cx.props.stages.iter().map(|stage| rsx! {
                StageStep { 
                    key: "{stage.stage_id}",
                    stage: stage.clone(),
                    is_current: cx.props.current_stage_id.as_ref() == Some(&stage.stage_id),
                    on_click: cx.props.on_stage_click,
                }
            })}
        }
    })
}

fn StageStep(cx: Scope<StageStepProps>) -> Element {
    // Individual stage rendering with:
    // - Visual connector line to next stage
    // - Status-based styling
    // - Countdown timer component
    // - Expandable approver list
}
```

### Countdown Timer

```rust
#[derive(Props, PartialEq, Clone)]
pub struct CountdownTimerProps {
    pub deadline: DateTime<Utc>,
    pub status: StageStatus,
}

pub fn CountdownTimer(cx: Scope<CountdownTimerProps>) -> Element {
    // Updates every minute via use_coroutine or use_future
    // Color coding:
    //   - Green: > 24 hours remaining
    //   - Yellow: 8-24 hours remaining
    //   - Red: < 8 hours remaining OR overdue
    // Display format: "2d 4h" or "5h 30m" or "30m overdue"
}
```

### Tests

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/tests/components/approval_queue.rs`

Write tests (if Dioxus testing framework is available):

- Test: WorkflowStageTracker renders all stages in order
- Test: WorkflowStageTracker marks completed stages with checkmark icon
- Test: WorkflowStageTracker highlights current stage with active styling
- Test: WorkflowStageTracker shows pending stages as dimmed
- Test: Countdown timer displays hours remaining for stage deadline
- Test: Countdown timer color codes correctly (green > 24h, yellow 8-24h, red < 8h)
- Test: Delegation badge appears next to delegated approver names
- Test: Escalation icon appears when stage has been escalated
- Test: Clicking stage expands to show approvers and their status
- Test: Stage tracker handles empty stages list gracefully

## 10.2 Diff Viewer Component

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/diff_viewer.rs`

### Purpose

Displays document version differences in multiple formats. Provides toggle between unified, side-by-side, and inline diff views. Uses color coding to highlight additions (green) and deletions (red).

### Props

```rust
#[derive(Props, PartialEq, Clone)]
pub struct DiffViewerProps {
    pub document_id: Uuid,
    pub from_version: String,
    pub to_version: String,
    pub initial_format: DiffFormat,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DiffFormat {
    Unified,
    SideBySide,
    Inline,
}
```

### Component Structure

```rust
pub fn DiffViewer(cx: Scope<DiffViewerProps>) -> Element {
    let format = use_state(cx, || cx.props.initial_format);
    let diff_data = use_future(cx, (), |_| {
        // Fetch diff from /api/documents/{id}/versions/diff
        // Query params: from, to, format
    });

    cx.render(rsx! {
        div { class: "diff-viewer",
            // Format toggle buttons
            div { class: "diff-controls",
                button {
                    class: if **format == DiffFormat::Unified { "active" },
                    onclick: move |_| format.set(DiffFormat::Unified),
                    "Unified"
                }
                button {
                    class: if **format == DiffFormat::SideBySide { "active" },
                    onclick: move |_| format.set(DiffFormat::SideBySide),
                    "Side by Side"
                }
                button {
                    class: if **format == DiffFormat::Inline { "active" },
                    onclick: move |_| format.set(DiffFormat::Inline),
                    "Inline"
                }
            }
            
            // Diff content
            match diff_data.value() {
                Some(Ok(diff)) => rsx! {
                    DiffContent { 
                        diff: diff.clone(),
                        format: **format,
                    }
                },
                Some(Err(e)) => rsx! { div { class: "error", "Error loading diff: {e}" } },
                None => rsx! { div { class: "loading", "Loading..." } },
            }
        }
    })
}

fn DiffContent(cx: Scope<DiffContentProps>) -> Element {
    match cx.props.format {
        DiffFormat::Unified => cx.render(rsx! { UnifiedDiff { diff: cx.props.diff.clone() } }),
        DiffFormat::SideBySide => cx.render(rsx! { SideBySideDiff { diff: cx.props.diff.clone() } }),
        DiffFormat::Inline => cx.render(rsx! { InlineDiff { diff: cx.props.diff.clone() } }),
    }
}
```

### Diff Rendering

```rust
fn UnifiedDiff(cx: Scope<UnifiedDiffProps>) -> Element {
    // Git-style unified diff:
    // - Lines starting with '-' are red (deleted)
    // - Lines starting with '+' are green (inserted)
    // - Other lines are default color
    // Line numbers for context
}

fn SideBySideDiff(cx: Scope<SideBySideDiffProps>) -> Element {
    // Two-column layout:
    // - Left column: old version
    // - Right column: new version
    // - Aligned by line number
    // - Highlighted cells for changes
}

fn InlineDiff(cx: Scope<InlineDiffProps>) -> Element {
    // Single document with inline highlights:
    // - Deleted text: red strikethrough
    // - Inserted text: green underline or background
}
```

### Tests

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/tests/components/diff_viewer.rs`

Write tests:

- Test: DiffViewer renders diff with appropriate styling
- Test: DiffViewer shows additions in green
- Test: DiffViewer shows deletions in red
- Test: DiffViewer shows unchanged text in default color
- Test: Format toggle switches between unified, side-by-side, inline
- Test: DiffViewer handles large diffs without performance issues (use sample large diff)
- Test: DiffViewer shows "no changes" message for identical versions
- Test: DiffViewer loading state displays while fetching
- Test: DiffViewer error state displays on API failure

## 10.3 Delegation Manager

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/delegation_manager.rs`

### Purpose

Provides UI for creating, viewing, and revoking delegations. Lists both delegations created by the current user and delegations received from others.

### Props

```rust
#[derive(Props, PartialEq, Clone)]
pub struct DelegationManagerProps {
    pub user_id: Uuid,
}
```

### Component Structure

```rust
pub fn DelegationManager(cx: Scope<DelegationManagerProps>) -> Element {
    let show_create_form = use_state(cx, || false);
    let delegations = use_future(cx, (), |_| {
        // Fetch from /api/delegations
    });

    cx.render(rsx! {
        div { class: "delegation-manager",
            h2 { "Mijn Delegaties" }
            
            // Create button
            button {
                class: "btn-primary",
                onclick: move |_| show_create_form.set(!**show_create_form),
                "Nieuwe Delegatie"
            }
            
            // Create form (collapsible)
            if **show_create_form {
                CreateDelegationForm {
                    user_id: cx.props.user_id,
                    on_created: move |_| {
                        show_create_form.set(false);
                        // Refresh delegations list
                    },
                }
            }
            
            // Delegations list
            match delegations.value() {
                Some(Ok(delegations_list)) => rsx! {
                    DelegationList { 
                        delegations: delegations_list.clone(),
                        on_revoked: move |_| {
                            // Refresh list
                        },
                    }
                },
                Some(Err(e)) => rsx! { div { class: "error", "{e}" } },
                None => rsx! { div { class: "loading", "Laden..." } },
            }
        }
    })
}
```

### Create Delegation Form

```rust
#[derive(Props, PartialEq, Clone)]
pub struct CreateDelegationFormProps {
    pub user_id: Uuid,
    pub on_created: EventHandler<()>,
}

pub fn CreateDelegationForm(cx: Scope<CreateDelegationFormProps>) -> Element {
    let to_user = use_state(cx, || None::<Uuid>);
    let delegation_type = use_state(cx, || DelegationType::Temporary);
    let document_types = use_state(cx, || Vec::<String>::new());
    let starts_at = use_state(cx, || Utc::now());
    let ends_at = use_state(cx, || None::<DateTime<Utc>>);
    let errors = use_state(cx, || Vec::<String>::new());
    let is_submitting = use_state(cx, || false);

    // Form validation:
    // - to_user is required
    // - ends_at > starts_at (if Temporary)
    // - At least one document_type for Bulk delegation
    
    // Submit to POST /api/delegations
}
```

### Delegation List

```rust
#[derive(Props, PartialEq, Clone)]
pub struct DelegationListProps {
    pub delegations: Vec<DelegationView>,
    pub on_revoked: EventHandler<Uuid>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct DelegationView {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub from_user_name: String,
    pub to_user_id: Uuid,
    pub to_user_name: String,
    pub delegation_type: DelegationType,
    pub document_types: Vec<String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

fn DelegationList(cx: Scope<DelegationListProps>) -> Element {
    // Group by "Aangemaakt" (created by me) vs "Ontvangen" (received from others)
    // Show delegation type badge
    // Show date range for temporary delegations
    // Show document types for bulk delegations
    // Revoke button (only for created by me, requires confirmation)
}
```

### Tests

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/tests/components/delegation_manager.rs`

Write tests:

- Test: DelegationManager lists active delegations
- Test: DelegationManager shows delegation type (temporary, permanent, bulk)
- Test: DelegationManager shows date range for temporary delegations
- Test: CreateDelegationForm validates to_user is required
- Test: CreateDelegationForm validates ends_at > starts_at
- Test: CreateDelegationForm allows document_types selection
- Test: Revoke button removes delegation from list after confirmation
- Test: Revoke button requires confirmation before action
- Test: Delegations are grouped into "Aangemaakt" and "Ontvangen" sections
- Test: Inactive/expired delegations are shown differently (dimmed)

## 10.4 Version History Component

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/version_history.rs`

### Purpose

Displays document version history with metadata. Allows selecting two versions for comparison and restoring previous versions.

### Props

```rust
#[derive(Props, PartialEq, Clone)]
pub struct VersionHistoryProps {
    pub document_id: Uuid,
    pub on_compare: EventHandler<(String, String)>,
    pub on_restored: EventHandler<()>,
}
```

### Component Structure

```rust
pub fn VersionHistory(cx: Scope<VersionHistoryProps>) -> Element {
    let versions = use_future(cx, (), |_| {
        // Fetch from /api/documents/{id}/versions
    });
    let selected_for_comparison = use_state(cx, || (None::<String>, None::<String>));

    cx.render(rsx! {
        div { class: "version-history",
            h3 { "Versiegeschiedenis" }
            
            match versions.value() {
                Some(Ok(versions_list)) => rsx! {
                    // Version list with selection checkboxes
                    VersionList {
                        versions: versions_list.clone(),
                        selected: selected_for_comparison.clone(),
                        on_selection_change: move |(v1, v2)| {
                            selected_for_comparison.set((v1, v2));
                        },
                    }
                    
                    // Compare button (enabled when 2 versions selected)
                    if let (Some(v1), Some(v2)) = &*selected_for_comparison {
                        button {
                            class: "btn-secondary",
                            onclick: move |_| {
                                cx.props.on_compare.call((v1.clone(), v2.clone()));
                            },
                            "Vergelijk {v1} met {v2}"
                        }
                    }
                },
                Some(Err(e)) => rsx! { div { class: "error", "{e}" } },
                None => rsx! { div { class: "loading", "Laden..." } },
            }
        }
    })
}
```

### Version List

```rust
#[derive(Props, PartialEq, Clone)]
pub struct VersionListProps {
    pub versions: Vec<VersionView>,
    pub selected: (Option<String>, Option<String>),
    pub on_selection_change: EventHandler<(Option<String>, Option<String>)>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct VersionView {
    pub version_id: String,
    pub version_number: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub created_by_name: String,
    pub change_summary: Option<String>,
    pub is_current: bool,
}

fn VersionList(cx: Scope<VersionListProps>) -> Element {
    // Table or list view of versions
    // Columns: Version #, Datum, Auteur, Samenvatting, Acties
    // Checkbox for comparison selection
    // Current version highlighted
    // Restore button (non-current versions only)
}
```

### Restore Confirmation

```rust
#[derive(Props, PartialEq, Clone)]
pub struct RestoreConfirmationProps {
    pub version: VersionView,
    pub on_confirm: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
}

pub fn RestoreConfirmation(cx: Scope<RestoreConfirmationProps>) -> Element {
    // Modal dialog
    // Warning: "Weet je zeker dat je versie X wilt herstellen?"
    // Note: "Dit zal een nieuwe versie aanmaken, de huidige versie blijft behouden."
    // Confirm and Cancel buttons
}
```

### Tests

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/tests/components/version_history.rs`

Write tests:

- Test: VersionHistory lists all versions with metadata
- Test: VersionHistory shows version number, created_by, created_at
- Test: VersionHistory allows selecting two versions for comparison
- Test: Compare button is disabled when fewer than 2 versions selected
- Test: Compare button triggers on_compare callback with selected versions
- Test: Restore button shows confirmation dialog
- Test: Restore confirmation shows warning about overwriting current version
- Test: Confirming restore triggers on_restored callback
- Test: Current version is highlighted differently in the list
- Test: Restore button is not shown for current version

## 10.5 Approval Queue Integration

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/approval_queue.rs` (modify)

### Changes to Existing Approval Queue

The existing approval queue page at `/documenten/wachtrij` needs to be enhanced with:

1. **Stage Tracker Integration**
   - Add `WorkflowStageTracker` above document details
   - Fetch stages from `/api/documents/{id}/stages`
   - Update in real-time via WebSocket for status changes

2. **Deadline Timers**
   - Add `CountdownTimer` to each document card in queue
   - Sort queue by urgency (overdue first, then nearest deadline)
   - Visual urgency indicator on card

3. **Delegation Indicators**
   - Show badge when document is assigned via delegation
   - Display "In plaats van [Original Approver]" on approver name
   - Show delegation chain for multi-hop delegations

4. **Escalation Badges**
   - Show escalation icon when stage has been escalated
   - Display escalation count (1, 2, 3...)
   - Color code by escalation severity

### Integration Example

```rust
// In approval_queue.rs, modify the document card rendering
fn DocumentCard(cx: Scope<DocumentCardProps>) -> Element {
    let document = &cx.props.document;
    
    cx.render(rsx! {
        div { class: "document-card",
            // Existing fields...
            
            // NEW: Stage tracker for multi-stage documents
            if document.stage_count > 1 {
                WorkflowStageTracker {
                    document_id: document.id,
                    stages: document.stages.clone(),
                    current_stage_id: document.current_stage_id.clone(),
                    on_stage_click: None,
                }
            }
            
            // NEW: Countdown timer
            if let Some(deadline) = &document.current_deadline {
                CountdownTimer {
                    deadline: deadline.clone(),
                    status: document.status.clone(),
                }
            }
            
            // NEW: Delegation badge
            if document.is_delegated {
                DelegationBadge {
                    original_approver: document.original_approver.clone(),
                }
            }
            
            // NEW: Escalation indicator
            if document.escalation_count > 0 {
                EscalationBadge {
                    count: document.escalation_count,
                }
            }
        }
    })
}
```

## 10.6 API Client Integration

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/api/mod.rs` (modify)

Add API client methods for the new endpoints:

```rust
// Add to existing API client
impl ApiClient {
    // Workflow stages
    pub async fn get_document_stages(&self, document_id: Uuid) -> Result<Vec<StageView>> {
        // GET /api/documents/{id}/stages
    }
    
    pub async fn get_stage_detail(&self, document_id: Uuid, stage_id: String) -> Result<StageDetailView> {
        // GET /api/documents/{id}/stages/{stage_id}
    }
    
    pub async fn approve_stage(&self, document_id: Uuid, stage_id: String, comment: Option<String>) -> Result<()> {
        // POST /api/documents/{id}/stages/{stage_id}/approve
    }
    
    pub async fn reject_stage(&self, document_id: Uuid, stage_id: String, comment: Option<String>) -> Result<()> {
        // POST /api/documents/{id}/stages/{stage_id}/reject
    }
    
    // Delegations
    pub async fn list_delegations(&self) -> Result<Vec<DelegationView>> {
        // GET /api/delegations
    }
    
    pub async fn create_delegation(&self, request: CreateDelegationRequest) -> Result<DelegationView> {
        // POST /api/delegations
    }
    
    pub async fn revoke_delegation(&self, delegation_id: Uuid) -> Result<()> {
        // DELETE /api/delegations/{id}
    }
    
    // Versions
    pub async fn list_versions(&self, document_id: Uuid) -> Result<Vec<VersionView>> {
        // GET /api/documents/{id}/versions
    }
    
    pub async fn get_diff(&self, document_id: Uuid, from: String, to: String, format: DiffFormat) -> Result<DocumentDiff> {
        // GET /api/documents/{id}/versions/diff
    }
    
    pub async fn restore_version(&self, document_id: Uuid, version_id: String) -> Result<VersionView> {
        // POST /api/documents/{id}/versions/{version_id}/restore
    }
}
```

## Styling Considerations

All components should follow the existing IOU-Modern design system:

- Use Dutch language for all user-facing text
- Consistent color scheme with existing UI
- Responsive design for mobile and desktop
- Accessible keyboard navigation
- Loading states and error handling
- Toast notifications for successful actions

### CSS Classes (add to existing stylesheet)

```css
/* Stage Tracker */
.stage-tracker {
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    padding: 1rem 0;
}

.stage-step {
    display: flex;
    flex-direction: column;
    align-items: center;
    position: relative;
}

.stage-step.completed .stage-icon {
    background-color: #10b981;
}

.stage-step.in-progress .stage-icon {
    background-color: #3b82f6;
}

.stage-step.pending .stage-icon {
    background-color: #e5e7eb;
}

/* Countdown Timer */
.countdown-timer.healthy { color: #10b981; }
.countdown-timer.warning { color: #f59e0b; }
.countdown-timer.critical { color: #ef4444; }
.countdown-timer.overdue { color: #dc2626; font-weight: bold; }

/* Diff Viewer */
.diff-viewer .diff-add { background-color: #d1fae5; color: #065f46; }
.diff-viewer .diff-remove { background-color: #fee2e2; color: #991b1b; }
.diff-viewer .diff-unchanged { background-color: transparent; }

/* Delegation Badge */
.delegation-badge {
    display: inline-flex;
    align-items: center;
    padding: 0.25rem 0.5rem;
    background-color: #fef3c7;
    color: #92400e;
    border-radius: 9999px;
    font-size: 0.75rem;
}

/* Escalation Badge */
.escalation-badge {
    display: inline-flex;
    align-items: center;
    padding: 0.25rem 0.5rem;
    background-color: #fee2e2;
    color: #991b1b;
    border-radius: 9999px;
    font-size: 0.75rem;
}
```

## WebSocket Integration

All components should subscribe to real-time updates via Supabase Realtime:

```rust
// In component initialization
use_coroutine(cx, |mut rx: UnboundedReceiver<RealtimeEvent>| async move {
    while let Some(event) = rx.next().await {
        match event {
            RealtimeEvent::StageUpdated { document_id, stage } => {
                // Update stage tracker
            }
            RealtimeEvent::ApprovalReceived { document_id, stage_id, approver } => {
                // Update approval status
            }
            RealtimeEvent::DelegationCreated { delegation } => {
                // Refresh delegation list
            }
            RealtimeEvent::VersionCreated { document_id, version } => {
                // Refresh version history
            }
            _ => {}
        }
    }
});
```

## TODO Checklist

Implement in order:

1. Create `workflow_stage_tracker.rs`
   - [ ] Stage stepper component
   - [ ] Countdown timer with color coding
   - [ ] Approver avatars with status
   - [ ] Delegation badge component
   - [ ] Escalation icon component

2. Create `diff_viewer.rs`
   - [ ] Unified diff renderer
   - [ ] Side-by-side diff renderer
   - [ ] Inline diff renderer
   - [ ] Format toggle controls

3. Create `delegation_manager.rs`
   - [ ] Delegation list component
   - [ ] Create delegation form
   - [ ] Form validation
   - [ ] Revoke confirmation dialog

4. Create `version_history.rs`
   - [ ] Version list component
   - [ ] Version comparison selection
   - [ ] Restore confirmation dialog

5. Modify `approval_queue.rs`
   - [ ] Integrate stage tracker
   - [ ] Add countdown timers to cards
   - [ ] Add delegation badges
   - [ ] Add escalation indicators

6. Update `api/mod.rs`
   - [ ] Add workflow stages API methods
   - [ ] Add delegation API methods
   - [ ] Add version API methods

7. Add CSS styles
   - [ ] Stage tracker styles
   - [ ] Diff viewer styles
   - [ ] Delegation and escalation badge styles

8. Write tests (if Dioxus testing available)
   - [ ] Approval queue component tests
   - [ ] Diff viewer component tests
   - [ ] Delegation manager component tests
   - [ ] Version history component tests

---

## Implementation Notes

### Files Created
- `crates/iou-frontend/src/components/workflow_stage_tracker.rs` (379 lines)
- `crates/iou-frontend/src/components/diff_viewer.rs` (348 lines)
- `crates/iou-frontend/src/components/delegation_manager.rs` (448 lines)
- `crates/iou-frontend/src/components/version_history.rs` (371 lines)

### Files Modified
- `crates/iou-frontend/src/components/mod.rs` - Added module declarations and exports

### Code Review Fixes Applied
1. **CountdownTimer** - Simplified to calculate on render instead of infinite loop
2. **DiffViewer format toggle** - Made reactive by capturing `*format.read()` in closure
3. **Delegated approvers logic** - Fixed inverted boolean
4. **Escalations vector** - Added comment explaining API limitation
5. **Borrow/ownership issues** - Fixed with proper cloning patterns for Dioxus closures

### Known Limitations
1. **CreateDelegationForm** - Stub implementation, shows "not implemented" error
2. **Escalations data** - Not available in list API, would need detail API call
3. **Error notifications** - Currently logged to stderr only
4. **Tests** - No component tests added yet
5. **CSS styles** - Styles mentioned in spec need to be added to global stylesheet

### API Integration
All components use the API methods from `crates/iou-frontend/src/api/documents.rs`:
- `get_document_stages()` - Workflow stages
- `get_diff()` - Document diffs
- `list_delegations()` - Delegation list
- `revoke_delegation()` - Revoke delegation
- `list_versions()` - Version history
- `restore_version()` - Restore previous version