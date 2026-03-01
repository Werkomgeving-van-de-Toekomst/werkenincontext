# Research Findings - Document Creation Agents

## Overview

This document combines codebase analysis with external research on AI agent orchestration, document template systems, and Open Overheid guidelines.

---

## 1. Codebase Analysis: IOU-Modern

### 1.1 Project Architecture

IOU-Modern is a Rust/WebAssembly **Informatie Ondersteunde Werkomgeving** for Dutch government organizations.

**Technology Stack:**
- **Backend**: Axum REST API with async/await
- **Database**: DuckDB (embedded analytical database)
- **Frontend**: Dioxus 0.7 (WebAssembly)
- **AI/ML**: Custom Rust implementations for NER, semantic search, and compliance

**Project Structure:**
```
iou-modern/
├── crates/
│   ├── iou-core/       # Shared domain models
│   ├── iou-api/        # REST API (Axum + DuckDB)
│   ├── iou-ai/         # AI services (NER, GraphRAG)
│   ├── iou-regels/     # Open Regels integration
│   └── iou-frontend/   # Dioxus WASM app
├── migrations/         # DuckDB schema
└── data/              # DuckDB database file
```

### 1.2 PROVISA Management Features

**Location**: `/crates/iou-frontend/src/pages/provisa_manager.rs`

**Current Implementation:**
- Supports three PROVISA versions: 2020, 2014, and 2005
- Faceted search with filters (domain, version, status)
- Hotspot management capabilities
- Version comparison features (shows differences between versions)
- AI-powered classification suggestions
- Retention period calculator

**Key Patterns Used:**
- Mock data for development (consistent across project)
- Selectable record detail view
- Expandable version comparison panel
- Responsive grid layout

### 1.3 Woo (Wet open overheid) Document Integration

**Locations:**
- Core models: `/crates/iou-core/src/compliance.rs`
- AI compliance: `/crates/iou-ai/src/compliance.rs`
- API routes: `/crates/iou-api/src/routes/compliance.rs`
- Frontend: `/crates/iou-frontend/src/pages/compliance_dashboard.rs`

**Current Implementation:**
- **Rule-based compliance checking** using regex patterns
- **Automatic Woo relevance assessment** (document type, classification, content keywords)
- **Disclosure class determination**:
  - Openbaar (fully public)
  - GedeeltelijkOpenbaar (partially public with redactions)
  - NietOpenbaar (not public with refusal grounds)
  - NogNietBeoordeeld (not yet assessed)
- **Confidence scoring** (0.0-1.0)
- **Refusal grounds enumeration** (Woo articles 5.1 and 5.2)

### 1.4 Compliance Dashboard

**Location**: `/crates/iou-frontend/src/pages/compliance_dashboard.rs`

**Features:**
- Real-time compliance monitoring
- Overall compliance score (circular progress indicator)
- Domain-level breakdown with bar charts
- Trend analysis (week/month/quarter/year)
- Risk alerts with severity levels
- Recommended actions
- Predictive risk analysis

### 1.5 Key Backend Endpoints

```
/health - Health check
/context/{id} - Full context with related domains
/domains - CRUD operations
/objects - Information object management
/search - Full-text and semantic search
/compliance/* - Compliance monitoring and alerts
/apps/recommended - Context-aware app recommendations
/graphrag/* - Knowledge graph operations
/workflows/* - Workflow management
```

### 1.6 State Management Pattern

```rust
pub struct AppState {
    pub user: Option<UserInfo>,
    pub current_domain: Option<InformationDomain>,
    pub search_query: String,
    pub is_loading: bool,
}
```

### 1.7 Testing Setup

- **Unit Tests**: Standard Rust `#[test]` attributes
- **No formal testing framework**: Only standard Rust testing
- **Mock data**: Extensive use for development/demo
- **Limited integration tests**

---

## 2. AI Agent Orchestration Patterns (2025)

### 2.1 Multi-Agent Coordination Patterns

Based on Microsoft Azure's official AI Agent design patterns (2026):

**Sequential Orchestration (Pipeline Pattern)**
- Agents process tasks in a predefined linear sequence
- Best for: Multi-stage processes with clear dependencies
- **Document creation example**: Template Selection → Content Customization → Compliance Review → Risk Assessment
- When to avoid: Early stage failures that propagate

**Concurrent Orchestration (Fan-out/Fan-in)**
- Multiple agents work simultaneously on the same input
- Best for: Independent analysis from multiple perspectives
- Aggregation strategies: Classification voting, weighted merging, LLM synthesis

**Group Chat Coordination**
- Agents participate in a shared dialogue managed by a chat manager
- Best for: Consensus building, collaborative brainstorming

**Maker-Checker Pattern**
- One agent creates, another validates
- Iterative refinement until approval

**Handoff Coordination**
- Dynamic task delegation based on capability assessment
- Best for: Multi-domain problems

### 2.2 Framework Comparison

| Framework | Best For | Production Ready |
|-----------|----------|------------------|
| **LangGraph** | Complex workflows, production environments | Yes (industry standard) |
| **AutoGen** | Quick prototypes, code execution | Warning: weak observability |
| **CrewAI** | Multi-agent collaboration, role-based teams | Growing adoption |

**LangGraph** (Recommended for Production):
- State graph-driven architecture
- 50+ step task completion with <15% degradation
- Native checkpoint support for fault recovery
- Best for: Financial automation, compliance systems

### 2.3 Document Workflow Patterns

1. **Sequential Pipeline** for document creation:
   - Research → Structure → Content → Validation → Review

2. **Maker-Checker** for quality assurance:
   - Generator creates, Reviewer validates, iterate until approval

3. **Concurrent Analysis** for document review:
   - Parallel agents check: legal compliance, style, technical accuracy, accessibility

4. **Context Engineering** (2025 best practice):
   - Select Context: Load only relevant knowledge/tools
   - Compress Context: Summarize tool results, filter irrelevant info

### 2.4 Sources

- [Microsoft Azure AI Agent Design Patterns](https://learn.microsoft.com/zh-cn/azure/architecture/ai-ml/guide/ai-agent-design-patterns)
- [IBM watsonx Orchestrate](https://www.ibm.com/docs/zh/watsonx/watson-orchestrate/base?topic=agents-designing-high-performance)

---

## 3. Document Template Systems

### 3.1 Template Engine Options

| Engine | Language | Best For |
|--------|----------|----------|
| **Jinja2** | Python | Complex Python projects (Flask) |
| **Handlebars** | JavaScript | Frontend rendering, Node.js |
| **Tera** | Rust | Rust web applications |

**For Rust integration**: Tera is the natural choice, already used in many Rust web frameworks.

### 3.2 Validation Approaches

**JSON Schema Validation** (Industry Standard):
- Schema-based structural validation
- Type checking at template boundaries
- Business rule validation with custom validators

**Multi-stage Validation**:
1. Structural validation (schema)
2. Business logic validation (rules)
3. Content quality validation (AI/ML)

### 3.3 Dynamic Document Generation

**Core Components**:
1. Document Template with placeholder tags
2. Template Tags defining dynamic content regions
3. Input Data (JSON) driving content generation
4. Generation API replacing tags with actual values

### 3.4 Sources

- [Adobe Document Generation API](https://developer.adobe.com/document-services/docs/overview/document-generation-api/)
- [JSON Schema Validation Best Practices](https://www.mongodb.com/zh-cn/docs/v7.0/reference/operator/query/jsonSchema/)

---

## 4. Open Overheid Guidelines

### 4.1 Wet open overheid (Woo) - Key Principles

- **Right to access**: Every citizen has right to access government information
- **Transparency**: Ensures openness of government, including universities
- **Proactive publication**: Information should be proactively published

**Recent Case** (Utrecht Province, September 2025):
- Personal data accidentally disclosed in Woo response
- Demonstrates critical need for automated PII detection and redaction

### 4.2 Wet digitale overheid Requirements

**Security Standards**:
- **HTTPS mandatory** (since July 1, 2023)
- **HSTS required** for HTTPS-only connections
- TLS and Web Application guidelines from NCSC

**Accessibility**:
- Digital accessibility requirements legally binding
- European directive on accessibility implemented

### 4.3 Open Document Standards

**ODF (OpenDocument Format) - ISO/IEC 26300**:
- International standard since 2006
- Netherlands mandate: Open standards required for public sector data exchange
- Requirements: Long-term accessibility, interoperability, vendor independence

### 4.4 Sources

- [Wet digitale overheid - Digitale Overheid](https://www.digitaleoverheid.nl/overzicht-van-alle-onderwerpen/wetgeving/wet-digitale-overheid/)
- [ODF Adoption Analysis](https://blog.documentfoundation.org/blog/2025/06/14/odf-analysis-of-adoption/)
- [Utrecht Woo Data Breach](https://new.qq.com/rain/a/20251001A006MQ00)

---

## 5. Key Insights for Implementation

### 5.1 Integration Points

The new agent system should integrate with:
- **Compliance AI** (`/crates/iou-ai/src/compliance.rs`) - For Woo validation
- **Workflow API** (`/workflows/*`) - For state management
- **PROVISA Manager** - For document versioning and comparison
- **Knowledge Graph** - For context-aware document generation

### 5.2 Technical Recommendations

1. **Use LangGraph patterns** adapted for Rust async/await
2. **Implement Tera templates** for Rust compatibility
3. **Add automated PII detection** before document publication
4. **Use ODF formats** for long-term preservation
5. **Follow WCAG guidelines** for accessibility

### 5.3 Architecture Pattern

Recommended: **Sequential Pipeline with Maker-Checker loops**

```
Research Agent → Structure Agent → Content Agent
                                        ↓
                              Validation Agent ←─────┐
                                        ↓            │
                              Review Agent ─────────┘
                                        ↓
                              (Human Approval Check)
                                        ↓
                                  Final Document
```

---

## 6. Testing Strategy

Given the existing codebase patterns:
- Use standard Rust `#[test]` attributes
- Create mock data for development
- Implement JSON Schema validation tests
- Add compliance rule tests (following existing pattern in `compliance.rs`)
