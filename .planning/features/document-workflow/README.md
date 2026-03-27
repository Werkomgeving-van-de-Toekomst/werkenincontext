# Document Workflow Enhancements

## Overview

Uitbreiding van het IOU-Modern document workflow systeem met multi-stage goedkeuringen, delegatie, goedkeuringsverloop met escalatie, en versiegeschiedenis.

## Status

📋 **Planned** - Database schema ontworpen, awaiting implementation

## Key Features

1. **Multi-stage approvals** - Configureerbare goedkeuringsstromen per documenttype
2. **Delegation** - Tijdelijke overdracht van goedkeuringsbevoegdheid
3. **Approval expiry** - Automatische escalatie bij timeout
4. **Version history** - Volledige versiegeschiedenis met diff visualization

## Current State

Bestaande workflow states: `Draft` → `Submitted` → `InReview` → `Approved` → `Published`

Locatie: `iou-orchestrator/src/state_machine.rs` en `iou-core/src/workflows.rs`

## Database Schema

Nieuwe tabellen:

```sql
approval_stages              - Configuratie van goedkeuringsstromen
delegations                  - Actieve delegaties
document_approval_stages     - Per-document state tracking
document_approvals           - Individuele goedkeuringsreacties
approval_escalations         - Escalatie log
```

## Approval Flow

```
Submitted
    ↓
Stage 1 (bijv. Juridische Toetsing)
    ↓ [optional: delegation]
Stage 2 (bijv. Management Goedkeuring)
    ↓ [optional: escalation]
Approved
    ↓
Published
```

## Approval Types

| Type | Beschrijving |
|------|-------------|
| `any` | 1 van N goedkeurders voldoet |
| `all` | Alle N goedkeurders moeten akkoord |
| `quorum` | Meerderheid (configurable) |

## Escalation Actions

| Action | Trigger |
|--------|---------|
| `notify_only` | Email herinnering |
| `escalate_manager` | Escalatie naar manager |
| `escalate_director` | Escalatie naar directeur |
| `auto_approve` | Automatische goedkeuring |

## Technology Stack

| Component | Technology |
|-----------|-----------|
| Database | DuckDB + Supabase PostgreSQL |
| State Machine | Bestaande `WorkflowEngine` |
| Real-time | Supabase realtime subscriptions |
| Diff | `similar` crate voor text diff |

## Backward Compatibility

Alle wijzigingen preserveren bestaande single-stage workflow gedrag zonder modificatie.

## Related Documents

- Original plan: `../../planning-document-workflow/claude-plan.md`
- Agent orchestration: `../agent-orchestration/`
- API extensions: `../api-extensions/`

---

*Source: planning-document-workflow/claude-plan.md*
