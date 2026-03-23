# Project Plan: IOU-Modern - Entity Relationship Extraction

> **Template Origin**: Official | **ArcKit Version**: 4.3.1 | **Command**: `/arckit:plan` |
> **Methodology**: Agile with 2-week sprints |

## Document Control

| Field | Value |
|-------|-------|
| **Document ID** | ARC-001-PLAN-v1.0 |
| **Document Type** | Project Plan |
| **Project** | IOU-Modern - Entity Relationship Extraction (Project 001.1) |
| **Classification** | OFFICIAL |
| **Status** | DRAFT |
| **Version** | 1.0 |
| **Created Date** | 2026-03-20 |
| **Last Modified** | 2026-03-20 |
| **Review Cycle** | Per sprint |
| **Next Review Date** | End of Sprint 1 |
| **Owner** | Project Manager |
| **Reviewed By** | PENDING |
| **Approved By** | PENDING |

## Revision History

| Version | Date | Author | Changes | Approved By | Approval Date |
|---------|------|--------|---------|-------------|---------------|
| 1.0 | 2026-03-20 | ArcKit AI | Initial project plan from `/arckit:plan` command | PENDING | PENDING |

---

## Executive Summary

This plan covers the implementation of **Entity Relationship Extraction** functionality for IOU-Modern. The feature enables extraction of entities (persons, organizations, locations) from government documents, integration with the Rijksoverheid Open Data API, and knowledge graph extensions.

**Timeline**: 10 weeks across 5 sprints (2 weeks each)
**Team Size**: 4 FTE (1 Backend, 1 Frontend, 1 Data Science, 0.5 Testing)
**Budget**: €80,000 (estimated)

**Key Deliverables**:
- Baseline NER extraction (regex-based)
- LLM-powered extraction with Claude API
- Rijksoverheid API integration
- Knowledge graph extensions
- API endpoints for extraction

---

## 1. Project Overview

### 1.1 Purpose

Enable IOU-Modern to automatically extract entities from documents, integrate with external government data sources, and build a knowledge graph for semantic search and cross-domain insights.

### 1.2 Scope

**In Scope**:
- Named Entity Recognition (Person, Organization, Location)
- Entity normalization and deduplication
- Rijksoverheid Open Data API integration
- KnowledgeGraph entity and relationship storage
- API endpoints for triggering and querying extraction

**Out of Scope**:
- Document upload/import (separate feature)
- Graph visualization UI (separate feature)
- Advanced graph analytics (future phase)

### 1.3 Success Criteria

| Criterion | Measure | Target |
|-----------|----------|--------|
| Entity extraction accuracy | Precision/Recall for NER | >90% for Person, >85% for Organization |
| API integration | Rijksoverheid API calls successful | >99% uptime |
| Performance | Extraction throughput | >100 documents/minute |
| Knowledge graph | Entities stored and queryable | <100ms response for entity queries |
| User acceptance | Stakeholder approval | Approved by domain owners |

---

## 2. Work Breakdown Structure

### 2.1 Phases and Sprints

```
Phase 1: Foundation (Sprints 1-2, 4 weeks)
├── Sprint 1: Foundation Types & Baseline NER
└── Sprint 2: Rijksoverheid API Integration

Phase 2: AI Enhancement (Sprints 3-4, 4 weeks)
├── Sprint 3: LLM Extractor
└── Sprint 4: Normalization & Deduplication

Phase 3: Integration (Sprints 5-6, 4 weeks)
├── Sprint 5: Main Extractor Pipeline
└── Sprint 6: KnowledgeGraph Extensions

Phase 4: API & Testing (Sprints 7-8, 4 weeks)
├── Sprint 7: API Endpoints
└── Sprint 8: Testing & Validation

Phase 5: Deployment (Sprints 9-10, 4 weeks)
├── Sprint 9: Hardening & Documentation
└── Sprint 10: Deployment & Handoff
```

### 2.2 Sprint Details

#### Sprint 1: Foundation Types & Baseline NER (Weeks 1-2)

**Goal**: Establish core data types and baseline NER extraction

| Task ID | Task | Owner | Story Points | Dependencies |
|---------|------|-------|-------------|--------------|
| 1.1 | Define Entity types in data model | Backend | 3 | None |
| 1.2 | Implement E-011 Entity with Person type | Backend | 5 | 1.1 |
| 1.3 | Implement E-012 Relationship entity | Backend | 3 | 1.1 |
| 1.4 | Create E-013 Community entity | Backend | 2 | 1.1 |
| 1.5 | Implement regex-based NER for Dutch names | Backend | 8 | 1.1 |
| 1.6 | Add NER unit tests | Backend | 3 | 1.5 |
| 1.7 | Document baseline NER approach | Backend | 2 | 1.5 |

**Deliverables**: Foundation types, baseline NER, unit tests

#### Sprint 2: Rijksoverheid API Integration (Weeks 3-4)

**Goal**: Integrate with Rijksoverheid Open Data API for entity enrichment

| Task ID | Task | Owner | Story Points | Dependencies |
|---------|------|-------|-------------|--------------|
| 2.1 | Research Rijksoverheid API documentation | Backend | 2 | None |
| 2.2 | Design API client architecture | Backend | 3 | 2.1 |
| 2.3 | Implement HTTP client with retry logic | Backend | 5 | 2.2 |
| 2.4 | Implement Organization lookup endpoint | Backend | 3 | 2.3 |
| 2.5 | Add entity normalization from API data | Backend | 5 | 2.4 |
| 2.6 | Add API integration tests | Backend | 3 | 2.5 |
| 2.7 | Document rate limiting and caching | Backend | 2 | 2.3 |

**Deliverables**: Rijksoverheid API client, organization lookup, integration tests

#### Sprint 3: LLM Extractor (Weeks 5-6)

**Goal**: Implement Claude API-based extraction for complex entities

| Task ID | Task | Owner | Story Points | Dependencies |
|---------|------|-------|-------------|--------------|
| 3.1 | Design LLM extraction architecture | Data Science | 3 | None |
| 3.2 | Implement Claude API client | Backend | 5 | 3.1 |
| 3.3 | Create extraction prompts for each entity type | Data Science | 8 | 3.1 |
| 3.4 | Implement streaming response handling | Backend | 5 | 3.2 |
| 3.5 | Add confidence score calculation | Backend | 3 | 3.4 |
| 3.6 | Implement fallback to baseline NER on error | Backend | 3 | 3.5 |
| 3.7 | Add LLM extraction tests | Data Science | 3 | 3.6 |

**Deliverables**: LLM extractor, prompts, confidence scoring, tests

#### Sprint 4: Normalization & Deduplication (Weeks 7-8)

**Goal**: Clean and standardize extracted entities

| Task ID | Task | Owner | Story Points | Dependencies |
|---------|------|-------|-------------|--------------|
| 4.1 | Design canonical name algorithm | Backend | 5 | None |
| 4.2 | Implement person name normalization | Backend | 8 | 4.1 |
| 4.3 | Implement organization name normalization | Backend | 5 | 4.2 |
| 4.4 | Create entity resolution service | Backend | 8 | 4.3 |
| 4.5 | Implement deduplication logic | Backend | 5 | 4.4 |
| 4.6 | Add normalization tests | Backend | 3 | 4.5 |

**Deliverables**: Normalization service, deduplication logic, tests

#### Sprint 5: Main Extractor Pipeline (Weeks 9-10)

**Goal**: Orchestrate end-to-end extraction pipeline

| Task ID | Task | Owner | Story Points | Dependencies |
|---------|------|-------|-------------|--------------|
| 5.1 | Design pipeline orchestration | Backend | 3 | None |
| 5.2 | Implement checkpoint store for state | Backend | 5 | 5.1 |
| 5.3 | Create extraction workflow orchestrator | Backend | 8 | 5.2 |
| 5.4 | Implement batch extraction job | Backend | 5 | 5.3 |
| 5.5 | Add progress tracking and status API | Backend | 3 | 5.4 |
| 5.6 | Add pipeline integration tests | Backend | 5 | 5.5 |

**Deliverables**: Extractor pipeline, checkpoint store, batch job, API endpoints

#### Sprint 6: KnowledgeGraph Extensions (Weeks 11-12)

**Goal**: Extend knowledge graph for extracted entities

| Task ID | Task | Owner | Story Points | Dependencies |
|---------|------|-------|-------------|--------------|
| 6.1 | Design KnowledgeGraph entity storage | Backend | 3 | None |
| 6.2 | Add entity storage CRUD operations | Backend | 5 | 6.1 |
| 6.3 | Implement relationship detection | Backend | 8 | 6.2 |
| 6.4 | Add community detection algorithm | Backend | 5 | 6.3 |
| 6.5 | Create knowledge graph query API | Backend | 5 | 6.4 |
| 6.6 | Add graph integration tests | Backend | 3 | 6.5 |

**Deliverables**: KnowledgeGraph extensions, relationship detection, query API

#### Sprint 7: API Endpoints (Weeks 13-14)

**Goal**: Expose extraction functionality via REST API

| Task ID | Task | Owner | Story Points | Dependencies |
|---------|------|-------|-------------|--------------|
| 7.1 | Design extraction API specification | Backend | 2 | None |
| 7.2 | Implement POST /domains/{id}/extract | Backend | 5 | 7.1 |
| 7.3 | Implement GET /domains/{id}/entities | Backend | 3 | 7.2 |
| 7.4 | Implement GET /domains/{id}/relationships | Backend | 3 | 7.3 |
| 7.5 | Add pagination and filtering | Backend | 3 | 7.4 |
| 7.6 | Add API authentication and authorization | Backend | 2 | 7.5 |
| 7.7 | Add API integration tests | Backend | 3 | 7.6 |

**Deliverables**: Extraction API endpoints, authentication, tests

#### Sprint 8: Testing & Validation (Weeks 15-16)

**Goal**: Comprehensive testing and validation

| Task ID | Task | Owner | Story Points | Dependencies |
|---------|------|-------|-------------|--------------|
| 8.1 | Create test plan | Testing | 2 | None |
| 8.2 | Implement E2E extraction tests | Testing | 8 | 8.1 |
| 8.3 | Performance testing (load testing) | Testing | 5 | 8.2 |
| 8.4 | Security testing (PII data handling) | Testing | 5 | 8.2 |
| 8.5 | Fix bugs and issues | Team | Variable | 8.4 |
| 8.6 | Conduct stakeholder demo | PM | 2 | 8.5 |

**Deliverables**: Test plan, test reports, bug fixes, stakeholder approval

#### Sprint 9: Hardening & Documentation (Weeks 17-18)

**Goal**: Production hardening and documentation

| Task ID | Task | Owner | Story Points | Dependencies |
|---------|------|-------|-------------|--------------|
| 9.1 | Add error handling and logging | Backend | 3 | None |
| 9.2 | Implement metrics and monitoring | Backend | 5 | 9.1 |
| 9.3 | Write API documentation | Backend | 3 | None |
| 9.4 | Create user guide | PM | 2 | 9.3 |
| 9.5 | Run security review | Security | 2 | 9.1 |
| 9.6 | Fix identified issues | Team | Variable | 9.5 |

**Deliverables**: Monitoring, documentation, security review

#### Sprint 10: Deployment & Handoff (Weeks 19-20)

**Goal**: Deploy to production and handoff to operations

| Task ID | Task | Owner | Story Points | Dependencies |
|---------|------|-------|-------------|--------------|
| 10.1 | Create deployment plan | DevOps | 2 | None |
| 10.2 | Configure production infrastructure | DevOps | 3 | 10.1 |
| 10.3 | Deploy to staging environment | DevOps | 2 | 10.2 |
| 10.4 | Run staging acceptance tests | Testing | 3 | 10.3 |
| 10.5 | Deploy to production | DevOps | 2 | 10.4 |
| 10.6 | Conduct operations handover | PM | 2 | 10.5 |
| 10.7 | Create runbook | Operations | 3 | 10.6 |
| 10.8 | Sprint review and retrospective | Team | 1 | 10.7 |

**Deliverables**: Production deployment, runbook, handoff complete

---

## 3. Resource Allocation

### 3.1 Team

| Role | Person | Allocation | Cost |
|------|-------|------------|------|
| Project Manager | PM | 50% | €10,000 |
| Backend Developer | BE | 100% | €30,000 |
| Frontend Developer | FE | 50% (Phase 4 only) | €10,000 |
| Data Scientist | DS | 100% (Phases 2-3) | €25,000 |
| QA Engineer | QA | 50% (Phase 4 only) | €5,000 |

### 3.2 Budget

| Category | Cost | Notes |
|----------|------|-------|
| Personnel | €80,000 | 10 weeks at blended rate |
| Infrastructure | €5,000 | Cloud resources, API costs |
| Tools | €2,000 | Development tools, licenses |
| Contingency | €10,000 | 12.5% |
| **Total** | **€97,000** | Rounded to €100,000 |

---

## 4. Timeline

### 4.1 Gantt Chart Overview

```
Sprint  1     2     3     4     5     6     7     8     9     10
Week     1-2   3-4   5-6   7-8   9-10  11-12 13-14 15-16 17-18 19-20
         │     │     │     │     │     │     │     │     │
Phase 1  ████  ████
Phase 2                    ████  ████
Phase 3                          ████  ████
Phase 4                                    ████  ████
Phase 5                                                ████  ████
```

### 4.2 Key Milestones

| Milestone | Date | Deliverable |
|-----------|------|------------|
| M1: Complete | Week 2 | Foundation types, baseline NER working |
| M2: Complete | Week 4 | Rijksoverheid API integrated |
| M3: Complete | Week 6 | LLM extractor functional |
| M4: Complete | Week 8 | Normalization pipeline working |
| M5: Complete | Week 10 | Extraction pipeline operational |
| M6: Complete | Week 12 | KnowledgeGraph extensions complete |
| M7: Complete | Week 14 | API endpoints ready |
| M8: Complete | Week 16 | Testing complete, stakeholder approval |
| M9: Complete | Week 18 | Documentation complete |
| M10: Complete | Week 20 | Production deployed, handoff complete |

---

## 5. Dependencies

### 5.1 Technical Dependencies

| Dependency | Type | Owner | Status | Risk |
|-----------|------|-------|-------|------|
| PostgreSQL 15+ | Infrastructure | DevOps | Available | Low |
| Rust toolchain | Development | Backend | Available | Low |
| Claude API key | External | PM | Pending | Medium |
| Rijksoverheid API access | External | PM | Pending | Low |
| Test data | Data | Domain Owner | Pending | Medium |

### 5.2 External Dependencies

| Dependency | Owner | Due Date | Status |
|-----------|-------|----------|--------|
| Claude API key procurement | PM | Week 2 | Pending |
| Rijksoverheid API account | PM | Week 2 | Pending |
| Test document approval | Domain Owner | Week 4 | Pending |
| Production access | Security | Week 18 | Pending |

---

## 6. Risk Management

### 6.1 Project Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Claude API rate limits | Medium | High | Implement caching, fallback to baseline NER |
| Rijksoverheid API changes | Low | Medium | Version API client, monitor changes |
| Entity extraction accuracy | Medium | High | Continuous testing, fallback mechanisms |
| Team availability | Low | High | Cross-training, buffer in timeline |
| Scope creep | Medium | Medium | Clear acceptance criteria, change control |

### 6.2 Issue Tracking

Current issues and blocking items:

| ID | Issue | Severity | Owner | Status |
|----|-------|---------|-------|--------|
| PLAN-001 | Claude API key not obtained | High | PM | Open |
| PLAN-002 | Test data not approved | Medium | PM | Open |

---

## 7. Communication Plan

### 7.1 Stakeholder Communication

| Stakeholder | Frequency | Channel | Owner |
|-------------|-----------|--------|-------|
| Domain Owners | Bi-weekly | Email updates | PM |
| Technical Team | Daily | Standup | PM |
| Security Officer | Monthly | Security review | PM |
| DPO | Per sprint | Privacy impact review | PM |

### 7.2 Reporting

| Report | Frequency | Audience | Format |
|-------|-----------|----------|-------|
| Sprint status | Weekly | Project team | Email |
| Sprint review | Bi-weekly | Stakeholders | Presentation |
| Metrics report | Monthly | Leadership | Dashboard |

---

## 8. Quality Gates

### 8.1 Definition of Done

**Sprint Done** means:
- All planned tasks completed
| - Code reviewed and merged
| - Tests passing with >80% coverage
| - Documentation updated
| - Stakeholder demo conducted

**Release Done** means:
- All sprints completed
| - Acceptance criteria met
| - Security review passed
| - Operations handoff complete
| - Production deployed

### 8.2 Acceptance Criteria

| Criterion | Test Method | Success |
|-----------|-------------|--------|
| Entity extraction accuracy | Test suite | >90% precision for Person entities |
| API performance | Load test | <100ms P95 response time |
| Knowledge graph query | Integration test | <100ms for entity lookup |
| Rijksoverheid integration | Integration test | >99% successful lookups |

---

## 9. Change Control

### 9.1 Change Request Process

1. Submit change request with rationale
2. Assess impact on timeline and budget
3. Get approval from Project Manager
4. Update project plan and timeline
5. Communicate changes to team

### 9.2 Change History

| Version | Date | Change | Impact |
|---------|------|--------|--------|
| 1.0 | 2026-03-20 | Initial plan | Baseline |

---

## 10. Success Metrics

### 10.1 KPIs

| Metric | Target | Measurement |
|--------|--------|-------------|
| Velocity | 30-40 story points per sprint | Sprint burndown chart |
| On-time delivery | 90% of sprints on time | Milestone tracking |
| Defect rate | <5 bugs per sprint | Bug tracker |
| Stakeholder satisfaction | 4+ out of 5 | Survey |

### 10.2 ROI Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Time saved on entity entry | 50% reduction | User survey after 3 months |
| Extraction accuracy | >90% precision | Automated testing |
| API uptime | >99.5% | Monitoring |

---

## 11. Glossary

| Term | Definition |
|------|------------|
| **NER** | Named Entity Recognition |
| **LLM** | Large Language Model |
| **API** | Application Programming Interface |
| **Rijksoverheid** | Dutch national government organization |
| **KnowledgeGraph** | Graph-based knowledge representation |
| **Sprint** | 2-week development iteration |
| **Story Points** | Estimate of effort (complexity) |

---

## 12. Appendix

### 12.1 Detailed Task List

See Sprint sections in `.planning/01-entity-relationship-extraction/sections/` for detailed implementation specifications.

### 12.2 Related Documents

| Document | Link |
|----------|------|
| Data Model | projects/001-iou-modern/ARC-001-DATA-v1.0.md |
| Architecture Decisions | projects/001-iou-modern/ARC-001-ADR-v1.0.md |
| API Documentation | `/api/v1/docs` (when deployed) |

---

**END OF PROJECT PLAN**

## Generation Metadata

**Generated by**: ArcKit `/arckit:plan` command
**Generated on**: 2026-03-20
**ArcKit Version**: 4.3.1
**Project**: IOU-Modern (Project 001.1)
**AI Model**: Claude Opus 4.6
