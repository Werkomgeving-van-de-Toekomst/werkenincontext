# Agent Orchestration

## Overview

Georkestreerde document creatie flow voor IOU-Modern, waarbij AI agents werken onder menselijk toezicht met goedkeuringspunten na elke agent.

## Status

📋 **Planned** - Architecture designed, awaiting implementation

## Key Features

1. **Rust-native orchestration** - Geen Python bridge nodig
2. **State machine driven** - smlang DSL voor compile-time validatie
3. **Human-in-the-loop** - Goedkeuringspunten na elke agent
4. **In-memory state met checkpoints** - DuckDB voor crash recovery
5. **Event sourcing** - Audit trail voor compliance

## Architecture

```
API Layer (POST /api/workflows)
    ↓
Workflow Orchestrator (State Machine + Event Bus + Checkpoint Manager)
    ↓
Agents (Research → Content → Compliance → Review)
```

## Agents

| Agent | Responsibility | Output |
|-------|---------------|--------|
| Research | Haalt relevante wetgeving, precedenten | ResearchResult |
| Content | Genereert document concept | ContentDraft |
| Compliance | Checkt tegen regels (PROVISA, Woo, etc.) | ComplianceReport |
| Review | Eindredactie en kwaliteitstoets | FinalDocument |

## State Machine

States: `Created` → `Running` → `AwaitingApproval` → `Running` → ... → `Completed`

Guards:
- `can_proceed` - Check if human approved
- `can_retry` - Check if max retries not exceeded

## Technology Stack

| Component | Technology |
|-----------|-----------|
| Async Runtime | Tokio |
| State Machine | smlang |
| Channels | tokio::sync (mpsc, oneshot) |
| Storage | DuckDB |
| Testing | loom (deterministic concurrency) |

## Related Documents

- Original plan: `../../planning/claude-plan.md`
- API integration: `../api-extensions/`
- Document workflow: `../document-workflow/`

## Implementation Phases

1. **Foundation** - State machine, event bus, checkpoint manager
2. **Agent Integration** - Connect existing agents from `iou-ai`
3. **Human-in-the-Loop** - Approval UI, notification system
4. **Production** - Monitoring, metrics, error handling

---

*Source: planning/claude-plan.md*
