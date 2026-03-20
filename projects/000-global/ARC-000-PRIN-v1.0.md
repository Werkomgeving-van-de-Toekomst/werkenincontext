# Architecture Principles: IOU-Modern

> **Template Origin**: Official | **ArcKit Version**: 4.3.1 | **Command**: `/arckit:principles`

## Document Control

| Field | Value |
|-------|-------|
| **Document ID** | ARC-000-PRIN-v1.0 |
| **Document Type** | Architecture Principles |
| **Project** | 000-Global (Enterprise) |
| **Classification** | OFFICIAL |
| **Status** | DRAFT |
| **Version** | 1.0 |
| **Created Date** | 2026-03-20 |
| **Last Modified** | 2026-03-20 |
| **Review Cycle** | Annual |
| **Next Review Date** | 2027-03-20 |
| **Owner** | Enterprise Architect |
| **Reviewed By** | PENDING |
| **Approved By** | PENDING |

## Revision History

| Version | Date | Author | Changes | Approved By | Approval Date |
|---------|------|--------|---------|-------------|---------------|
| 1.0 | 2026-03-20 | ArcKit AI | Initial creation from `/arckit:principles` command | PENDING | PENDING |

---

## Executive Summary

IOU-Modern is built on **Privacy by Design**, **Open Government**, and **Sovereign Technology** principles. These principles guide architectural decisions to ensure compliance with Dutch government regulations (Woo, AVG, Archiefwet) while maintaining technical independence and sustainability.

**Core Philosophy**: Build transparent, privacy-respecting government information systems using open-source technologies that remain under Dutch control.

---

## Principle Catalog

### P1: Privacy by Design (AVG First)

**Statement**: Privacy is embedded into the system architecture from the ground up, not added as an afterthought.

**Rationale**:
- AVG (GDPR) requires privacy by design and by default (Article 25)
- Government systems process sensitive citizen data
- Trust is essential for digital government services

**Implications**:
- Privacy impact assessment (DPIA) before processing personal data
- Data minimization: collect only what's necessary
- PII tracked at entity level with privacy classification
- Automated deletion after retention periods
- Row-Level Security (RLS) for organization-level isolation

**Trade-offs Considered**:
- Functionality vs. Privacy: Privacy wins when conflict exists
- Performance vs. Encryption: Encryption required despite overhead

---

### P2: Open Government (Woo Compliance)

**Statement**: Government information is public by default, with limited and justified exceptions.

**Rationale**:
- Wet open overheid (Woo) mandates transparency
- Democratic accountability requires accessible government decisions
- Public trust increases with transparency

**Implications**:
- All documents assessed for Woo relevance automatically
- `is_woo_relevant` flag tracked for every InformationObject
- Publication workflow: draft → review → publish
- Human approval required for ALL Woo-relevant documents
- Classification defaults to Intern (conservative), Openbaar requires justification

**Trade-offs Considered**:
- Transparency vs. Privacy: Privacy protections apply to personal data
- Automation vs. Oversight: Human review required for publication

---

### P3: Archival Integrity (Archiefwet)

**Statement**: Government records are preserved according to Archiefwet requirements with complete provenance.

**Rationale**:
- Archiefwet 1995 mandates record retention (5-20 years depending on type)
- Historical accountability requires complete records
- Legal certainty depends on preserved decisions

**Implications**:
- Retention periods: Besluit (20y), Document (10y), Email (5y), Chat (1y)
- Version history maintained for all InformationObjects
- Audit trail logs all agent actions with timestamps
- Automated deletion only after retention expires
- Records transferred to Nationaal Archief after retention period

**Trade-offs Considered**:
- Storage cost vs. Retention: Retention wins (legal requirement)
- Performance vs. Audit logging: Complete audit trail required

---

### P4: Sovereign Technology (Open Source First)

**Statement**: Technology choices prioritize open-source software that can be maintained independently.

**Rationale**:
- Digital sovereignty: Government must control its technology stack
- Vendor lock-in risks: Proprietary dependencies create leverage
- Security: Open-source code can be audited
- Cost: No licensing fees for open-source alternatives

**Implications**:
- Backend: Rust (open-source, memory-safe)
- Database: PostgreSQL (open-source) + DuckDB (open-source)
- Frontend: Dioxus (open-source WebAssembly framework)
- Storage: MinIO (open-source S3-compatible storage)
- Avoid: Proprietary cloud services, vendor-specific APIs

**Allowed Exceptions**:
- AI models: May use commercial APIs (OpenAI, Anthropic) with fallback plans
- Maps: Leaflet.js (open-source) preferred over Google Maps

**Trade-offs Considered**:
- Open-source vs. Commercial support: Open-source preferred; budget for internal expertise
- Features vs. Independence: Independence prioritized

---

### P5: Domain-Driven Organization (Zaak/Project/Beleid/Expertise)

**Statement**: Information is organized by context, not by document type or storage location.

**Rationale**:
- Government work is organized by domains (Zaak, Project, Beleid, Expertise)
- Cross-domain insights enable better governance
- Citizens interact with government services, not IT systems

**Implications**:
- InformationDomain as first-class entity
- Documents belong to domains, not folders
- GraphRAG discovers cross-domain relationships
- Semantic search across domains, not keyword search within folders

**Trade-offs Considered**:
- Flexibility vs. Structure: Domain structure enables flexibility within governance

---

### P6: Human-in-the-Loop AI (Augmentation, Not Replacement)

**Statement**: AI systems assist government professionals, never replace human judgment.

**Rationale**:
- Government decisions have legal consequences
- Citizens have right to human review of decisions affecting them
- AI errors can cause harm at scale

**Implications**:
- ALL Woo-relevant documents require human approval before publication
- AI provides confidence scores; humans decide
- Audit trail captures AI recommendations vs. human decisions
- Humans can override AI decisions at any point

**AI Applications**:
- Named Entity Recognition (NER): Extract entities, human validates
- Compliance scoring: Suggest compliance, human reviews
- GraphRAG: Discover relationships, human interprets

**Trade-offs Considered**:
- Speed vs. Accuracy: Accuracy prioritized for government decisions
- Automation cost vs. Human review cost: Human review mandatory

---

### P7: Data Sovereignty (EU-Only Processing)

**Statement**: All data processing occurs within the European Union.

**Rationale**:
- GDPR requires adequate protection for cross-border data transfers
- Dutch government data must remain under Dutch jurisdiction
- Cloud Act (US) could compel disclosure of data stored in US

**Implications**:
- Primary database: Netherlands/EU region only
- Backup storage: Netherlands/EU region only
- S3/MinIO storage: On-premises or Netherlands region
- AI APIs: Use EU-based providers or ensure EU data processing

**Prohibited**:
- US cloud storage for government data
- AI providers that process data outside EU without safeguards

**Trade-offs Considered**:
- Cost vs. Sovereignty: Sovereignty wins
- Feature availability vs. Jurisdiction: Jurisdiction prioritized

---

### P8: Interoperability (Open Standards)

**Statement**: Systems use open standards to enable interoperability across government organizations.

**Rationale**:
- Government organizations must collaborate
- Citizens expect seamless services
- Vendor lock-in reduces innovation

**Implications**:
- REST API with OpenAPI specification
- Standard data formats: JSON, CSV, PDF/A
- Semantic search compatible with existing government systems
- Integration with existing case management systems (Sqills, Centric)

**Trade-offs Considered**:
- Custom formats vs. Standards: Standards required unless legal reason

---

### P9: Accessibility (WCAG 2.1 AA)

**Statement**: All user interfaces are accessible to citizens with disabilities.

**Rationale**:
- Legal requirement: European Accessibility Act
- Moral obligation: Inclusive government serves all citizens
- Better UX for everyone

**Implications**:
- WCAG 2.1 AA compliance for web interfaces
- Keyboard navigation
- Screen reader compatibility
- Color contrast ratios met
- DigiD integration for authentication

**Trade-offs Considered**:
- Aesthetics vs. Accessibility: Accessibility wins

---

### P10: Observability (Audit Everything)

**Statement**: All system actions are logged for compliance, debugging, and accountability.

**Rationale**:
- AVG requires accountability (Article 30)
- Debugging distributed systems requires tracing
- Security incident response needs logs

**Implications**:
- AuditTrail entity logs all agent actions
- PII access logged separately
- Logs retained for 7 years (compliance standard)
- Structured logging (JSON) for analysis
- Security monitoring for unauthorized access

**Trade-offs Considered**:
- Performance vs. Logging: Logging wins (compliance requirement)
- Storage cost vs. Retention: Retention wins (legal requirement)

---

## Principle Conflicts and Resolution

| Conflict | Principles | Resolution | Rationale |
|----------|------------|------------|-----------|
| Transparency vs. Privacy | P2 (Open Government) vs. P1 (Privacy) | Privacy first; publish only after PII review | Personal data protected before publication |
| AI Speed vs. Human Oversight | P6 (Human-in-the-Loop) vs. Performance | Human oversight required for Woo documents | Legal consequences require human judgment |
| Cost vs. Sovereignty | P4 (Open Source) vs. Cost | Sovereignty prioritized; budget allocated for expertise | Digital sovereignty is strategic necessity |
| Performance vs. Observability | P10 (Observability) vs. Performance | Observability required; optimize later | Compliance requirement cannot be compromised |

---

## Compliance Mapping

| Principle | Woo | AVG | Archiefwet | WCAG 2.1 |
|-----------|-----|-----|------------|----------|
| P1: Privacy by Design | ✓ | ✓ | | |
| P2: Open Government | ✓ | | | |
| P3: Archival Integrity | | | ✓ | |
| P4: Sovereign Technology | ✓ | ✓ | | |
| P5: Domain-Driven | | | | |
| P6: Human-in-the-Loop AI | ✓ | ✓ | | |
| P7: Data Sovereignty | | ✓ | | |
| P8: Interoperability | ✓ | | | |
| P9: Accessibility | | | | ✓ |
| P10: Observability | | ✓ | ✓ | |

---

## Decision Framework

When making architecture decisions, apply principles in priority order:

1. **Legal Compliance**: Principles driven by law (P1, P2, P3, P7, P9) — Non-negotiable
2. **Strategic Sovereignty**: Principles ensuring independence (P4, P8) — High priority
3. **Ethical AI**: Principles ensuring human oversight (P6) — High priority
4. **Operational Excellence**: Principles enabling operations (P5, P10) — Medium priority

**Conflict Resolution**: If principles conflict, legal compliance > strategic > ethical > operational.

---

## Review Process

| Frequency | Action | Owner |
|-----------|--------|-------|
| Annually | Full principle review | Enterprise Architect |
| Per project | Verify principles applied | Solution Architect |
| On change | Assess impact on principles | Architecture Board |

**Review Triggers**:
- New regulation (Woo, AVG updates)
- New technology adoption
- Security incident
- External audit findings

---

## External References

| Regulation | Link |
|------------|------|
| Wet open overheid (Woo) | https://wetten.overheid.nl/BWBR0036881/2022-05-01 |
| Algemene verordening gegevensbescherming (AVG) | https://gdpr-info.eu/ |
| Archiefwet 1995 | https://wetten.overheid.nl/BWBR0006646/2021-01-01 |
| WCAG 2.1 | https://www.w3.org/WAI/WCAG21/quickref/ |
| European Accessibility Act | https://eur-lex.europa.eu/eli/dir/2016/2102 |

---

**END OF PRINCIPLES**

## Generation Metadata

**Generated by**: ArcKit `/arckit:principles` command
**Generated on**: 2026-03-20
**ArcKit Version**: 4.3.1
**Project**: IOU-Modern (Global)
**AI Model**: Claude Opus 4.6
