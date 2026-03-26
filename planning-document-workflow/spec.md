# Enhanced Document Workflow - Specification

## Overview

Extend the existing document workflow system to support multi-stage approvals, delegation, expiry handling, and version history with diff visualization. This will enable organizations to implement complex approval processes while maintaining compliance and auditability.

## Current State

The system already has:
- Basic workflow states (Draft → Submitted → InReview → Approved → Published)
- Approval queue UI (`/documenten/wachtrij`)
- Document metadata with version tracking
- Trust level system for auto-approval decisions
- Audit trail entries
- Workflow engine with transition validation

## Desired Features

### 1. Multi-Stage Approvals

Documents should be able to go through multiple approval stages before final publication. Each stage can have different approvers and requirements.

**Use Cases:**
- Woo documents require legal review → department head → communication officer approval
- Budget documents require financial control → management board approval
- Project documents require project lead → steering committee approval

**Requirements:**
- Define approval stages per document type or domain
- Each stage has one or more required approvers
- Parallel approval support (multiple approvers at same stage)
- Sequential approval support (stages must complete in order)
- Stage can be optional based on document properties

### 2. Delegation

Approvers can delegate their approval authority to another user for a specific period or permanently.

**Use Cases:**
- Manager on vacation delegates approval to deputy
- Department head delegates specific document types to team lead
- Temporary delegation during absences

**Requirements:**
- Define delegation rules (from user, to user, document types, time period)
- Delegation can be temporary (date range) or indefinite
- Delegated approvals show original approver in audit trail
- View active delegations for a user
- Bulk delegation (all pending approvals during absence)

### 3. Approval Expiry & Escalation

Approvals should have deadlines to prevent documents from stalling in the workflow.

**Use Cases:**
- Legal review must be completed within 5 business days
- Escalation to manager if approval not received within 48 hours
- Automatic return to draft if no action within 30 days

**Requirements:**
- Define SLA per approval stage (e.g., 3 business days)
- Countdown timer visible in approval queue
- Escalation actions when deadline approaches (notify manager)
- Auto-action on expiry (return to draft, escalate, or continue waiting)
- Configurable per document type/domain

### 4. Version History with Diff View

Track document versions and show what changed between them.

**Use Cases:**
- Reviewer sees what changed since previous version before approving
- Audit trail shows exact changes made at each revision
- Compare any two versions of a document

**Requirements:**
- Store full document content for each version
- Version numbering (v1, v2, v3...)
- Diff visualization (added/deleted/changed text highlighted)
- Compare any two versions
- Restore previous version if needed
- Version metadata (who changed, when, why)

## Technical Constraints

- Database: DuckDB (embedded) + PostgreSQL (Supabase for realtime)
- Backend: Rust with Axum
- Frontend: Dioxus WASM
- Storage: S3/MinIO for document content
- Existing workflow engine must be extended, not replaced
- Must maintain backward compatibility with existing simple workflows

## Success Criteria

1. Multi-stage workflow can be defined and documents flow through all stages correctly
2. User can delegate their approval authority and see delegated approvals
3. Approval queue shows deadline countdown and escalations occur automatically
4. Version comparison shows visual diff of changes between document versions
5. All actions are logged in audit trail for compliance
6. Existing single-stage workflows continue to work without modification

## Open Questions

- Should approval stages be defined in code or configurable in UI?
- How should business days be calculated for SLAs (ignore weekends/holidays)?
- Should diff be calculated on backend (Rust) or frontend (JavaScript/WASM)?
- What diff format to use? (unified diff, side-by-side, inline?)
