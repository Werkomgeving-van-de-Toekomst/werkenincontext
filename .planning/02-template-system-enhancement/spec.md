# Spec: Template System Enhancement

**Split:** 02-template-system-enhancement
**Created:** 2026-03-14
**Estimated Duration:** 1 sprint

## Overview

Build a visual template editor that allows non-technical users to create, modify, and preview document templates. Currently templates are managed only through code/API, requiring developer involvement for changes.

## Context

### Current State

Templates are stored in PostgreSQL with this schema:
- `id`, `name`, `domain_id`, `document_type`
- `content` (Tera template syntax)
- `required_variables`, `optional_sections`
- `version`, `is_active`, `created_at`, `updated_at`

The template engine uses Tera (Rust template engine) in `crates/iou-ai/src/templates.rs`.

### User Pain Points

- Non-technical users cannot create templates without developer help
- No way to preview templates with sample data before deployment
- Template changes require code changes or API calls
- No visual indication of required vs optional variables

## Requirements

### Core Requirements

| ID | Requirement | Description |
|----|-------------|-------------|
| TMPL-01 | Visual Editor | WYSIWYG editor for creating/editing Tera templates |
| TMPL-02 | Variable Management | UI for defining required and optional template variables |
| TMPL-03 | Template Preview | Preview templates with sample data before saving |
| TMPL-04 | Version Control | Track template versions with comparison and rollback |
| TMPL-05 | Validation | Validate template syntax and variable completeness before save |

### User Stories

1. **As a policy officer**, I want to create document templates visually so I don't need developer help.
2. **As a policy officer**, I want to preview my template with sample data so I know it will render correctly.
3. **As a policy officer**, I want to define variables (like {{reference}}) so users know what data to provide.
4. **As a system administrator**, I want to see template version history so I can revert if needed.
5. **As a content creator**, I want to see available templates with descriptions so I choose the right one.

### Acceptance Criteria

- [ ] Visual editor creates valid Tera template syntax
- [ ] Variables are extracted/defined with name, type, and description
- [ ] Preview renders with user-provided sample data
- [ ] Template versions are tracked with diff view
- [ ] Invalid templates are rejected with helpful error messages
- [ ] Templates can be activated/deactivated without deletion

## Technical Details

### Template Syntax (Tera)

```tera
# {{ title }}

Reference: {{ reference }}
Date: {{ date | date(format="%Y-%m-%d") }}

{% for section in sections %}
## {{ section.name }}

{{ section.content }}
{% endfor %}

{% if optional_note %}
**Note:** {{ optional_note }}
{% endif %}
```

### Variable Schema

| Field | Type | Description |
|-------|------|-------------|
| name | string | Variable name (matches template) |
| type | enum | string, number, date, boolean, array |
| required | boolean | Whether variable must be provided |
| default | any | Default value if optional |
| description | string | Help text for users |

### Version Comparison

Display side-by-side or unified diff of template content between versions.

## UI Components

### Main Editor Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Template Editor                    [Save] [Preview] [Cancel]│
├─────────────────────┬───────────────────────────────────────┤
│                     │                                       │
│  Template Info      │         Variable Manager               │
│  ─────────────      │         ────────────────               │
│  Name: [________]   │                                       │
│  Type: [dropdown]   │  Required Variables:                   │
│  Domain: [dropdown] │  + reference (string)                 │
│                     │  + date (date)                         │
│  Editor             │                                       │
│  ──────────         │  Optional Variables:                   │
│  [Tera template     │  + optional_note (string)              │
│   editor area]      │                                       │
│                     │  [+ Add Variable]                     │
│                     │                                       │
│  Preview            │                                       │
│  ────────           │                                       │
│  [Rendered output   │                                       │
│   with sample data] │                                       │
└─────────────────────┴───────────────────────────────────────┘
```

### Template List View

```
┌─────────────────────────────────────────────────────────────┐
│  Templates                                    [+ New Template]│
├─────────────────────────────────────────────────────────────┤
│  Name              Type        Domain    Version    Status   │
│  ──────────────────────────────────────────────────────────│
│  Woo Besluit       woo_besluit  minfin    v3         Active  │
│  Woo Info          woo_info    minfin    v2         Active  │
│  Project Memo      internal    general   v1         Active  │
│  Old Template      woo_besluit  minfin    v1         Inactive│
└─────────────────────────────────────────────────────────────┘
```

## Dependencies

### From Codebase

- `iou-ai` template engine (Tera)
- `iou-core` template types
- `iou-api` template CRUD endpoints

### External Dependencies

- Tera template syntax parser
- Dioxus for frontend UI

## API Endpoints Needed

| Method | Path | Description |
|--------|------|-------------|
| GET | /api/templates | List all templates |
| POST | /api/templates | Create new template |
| GET | /api/templates/:id | Get template by ID |
| PUT | /api/templates/:id | Update template |
| DELETE | /api/templates/:id | Delete (deactivate) template |
| POST | /api/templates/:id/validate | Validate template syntax |
| POST | /api/templates/:id/preview | Preview with sample data |
| GET | /api/templates/:id/versions | Get version history |
| POST | /api/templates/:id/rollback/:version | Rollback to version |

## Out of Scope

- Rich text document editing (separate from templates)
- Advanced template features like loops/conditioning in UI (edit raw Tera for those)
- Template marketplace or sharing between organizations

## Constraints

- Must output valid Tera template syntax
- Must validate variables are present in template
- Frontend uses Dioxus (WASM)
- All template changes must be versioned for compliance

## Security Considerations

- Sanitize all template inputs to prevent XSS
- Escape variables by default in rendering
- Validate template syntax before saving
- Restrict template access by domain RBAC

## Success Metrics

| Metric | Target |
|--------|--------|
| Template creation time | <5 minutes by non-technical user |
| Template syntax errors caught | 100% before save |
| Templates created without developer help | >80% |

---

## References

- Original requirements: `.planning/future-features-requirements.md`
- Interview: `.planning/deep_project_interview.md`
- Template engine: `crates/iou-ai/src/templates.rs`
- Tera docs: https://tera.netlify.app/
