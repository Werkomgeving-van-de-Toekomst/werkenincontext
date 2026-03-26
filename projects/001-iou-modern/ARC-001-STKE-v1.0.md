# Stakeholder Analysis: IOU-Modern

> **Template Origin**: Official | **ArcKit Version**: 4.3.1 | **Command**: `/arckit:stakeholders`

## Document Control

| Field | Value |
|-------|-------|
| **Document ID** | ARC-001-STKE-v1.0 |
| **Document Type** | Stakeholder Analysis |
| **Project** | IOU-Modern (Project 001) |
| **Classification** | OFFICIAL |
| **Status** | DRAFT |
| **Version** | 1.0 |
| **Created Date** | 2026-03-20 |
| **Last Modified** | 2026-03-20 |
| **Review Cycle** | Semi-annual |
| **Next Review Date** | 2026-09-20 |
| **Owner** | Product Owner |
| **Reviewed By** | PENDING |
| **Approved By** | PENDING |

## Revision History

| Version | Date | Author | Changes | Approved By | Approval Date |
|---------|------|--------|---------|-------------|---------------|
| 1.0 | 2026-03-20 | ArcKit AI | Initial creation from `/arckit:stakeholders` command | PENDING | PENDING |

---

## Executive Summary

IOU-Modern serves **Dutch government organizations** (Rijk, Provincie, Gemeente, Waterschap, ZBO) and their stakeholders. The system processes personal data of employees (~50,000) and citizens (potentially millions mentioned in documents).

**Stakeholder Categories**:
- **Internal**: Government employees, management, IT, legal/compliance
- **External**: Citizens, businesses, suppliers, regulators
- **Data Subjects**: Employees, citizens named in documents, vulnerable populations

**Critical Insight**: The power imbalance between government and citizens requires strong privacy protections and transparency mechanisms.

---

## 1. Stakeholder Inventory

### 1.1 Internal Stakeholders

#### S1: Government Employees (End Users)

| Attribute | Value |
|-----------|-------|
| **Category** | Internal - Primary Users |
| **Size** | ~50,000 (Year 1) → 100,000 (Year 5) |
| **Role** | Create, manage, and access government information |
| **Influence** | Medium | **Power** | Low |
| **Interest** | High | |

**Goals**:
- Efficiently organize and find information
- Comply with legal requirements (Woo, AVG, Archiefwet)
- Reduce manual work through AI assistance
- Access information from anywhere

**Pain Points**:
- Information scattered across multiple systems
- Manual compliance checking is time-consuming
- Difficult to find relevant documents
- Uncertainty about Woo publication requirements

**Data Subject Rights**:
- SAR for their own employee data
- Rectification of personal information
- Erasure after employment ends + retention period

---

#### S2: Domain Owners (Zaak/Project/Beleid/Expertise Eigenaren)

| Attribute | Value |
|-----------|-------|
| **Category** | Internal - Decision Makers |
| **Size** | ~5,000 |
| **Role** | Own and manage information domains |
| **Influence** | High | **Power** | Medium |
| **Interest** | High | |

**Goals**:
- Maintain accurate and complete domain information
- Ensure compliance before publication
- Control access to sensitive information
- Demonstrate accountability

**Pain Points**:
- Uncertainty about what to publish vs. withhold
- Fear of publishing sensitive information accidentally
- Difficulty tracking compliance across domains

**RACI**:
- **Responsible**: Domain data quality, approval decisions
- **Accountable**: Compliance of their domains
- **Consulted**: On Woo publication decisions
- **Informed**: Of compliance issues in their domains

---

#### S3: Information Managers (Informatiebeheerders)

| Attribute | Value |
|-----------|-------|
| **Category** | Internal - Governance |
| **Size** | ~500 |
| **Role** | Oversee information management per Archiefwet |
| **Influence** | High | **Power** | High |
| **Interest** | High | |

**Goals**:
- Ensure compliance with Archiefwet retention requirements
- Maintain complete and accurate records
- Facilitate timely transfer to archives
- Support digital transformation

**Pain Points**:
- Difficulty ensuring consistent metadata across systems
- Challenge of tracking retention periods
- Fear of losing records during migration

**RACI**:
- **Responsible**: Records management policies
- **Accountable**: Archiefwet compliance
- **Consulted**: On retention and archival design

---

#### S4: CIO / IT Leadership

| Attribute | Value |
|-----------|-------|
| **Category** | Internal - Leadership |
| **Size** | ~50 (CIOs across organizations) |
| **Role** | Technology strategy and investment decisions |
| **Influence** | Very High | **Power** | High |
| **Interest** | High | |

**Goals**:
- Reduce vendor lock-in
- Improve system interoperability
- Control IT costs
- Enable digital service delivery

**Pain Points**:
- Fragmented IT landscape across government
- Pressure to adopt cloud services (sovereignty concerns)
- Legacy system integration challenges
- Budget constraints

**RACI**:
- **Accountable**: Technical architecture decisions
- **Responsible**: Infrastructure and hosting

---

#### S5: Data Protection Officer (Functionaris Gegevensbescherming)

| Attribute | Value |
|-----------|-------|
| **Category** | Internal - Compliance |
| **Size** | ~1 per organization (DPO pool) |
| **Role** | Ensure AVG/GDPR compliance |
| **Influence** | High | **Power** | High |
| **Interest** | Very High | |

**Goals**:
- Prevent data breaches
- Ensure lawful basis for all processing
- Facilitate data subject rights
- Maintain DPIA documentation

**Pain Points**:
- AI processing creates new risks (NER, profiling)
- Cross-domain relationship discovery may violate expectations
- Difficulty tracking all PII across systems
- Limited resources for oversight

**RACI**:
- **Responsible**: DPIA, privacy policies, SAR processes
- **Accountable**: AVG compliance
- **Consulted**: On all feature designs involving personal data
- **Approver**: For high-risk processing

---

#### S6: Woo Officers (Wet open overheid)

| Attribute | Value |
|-----------|-------|
| **Category** | Internal - Compliance |
| **Size** | ~200 |
| **Role** | Ensure Woo compliance and publication |
| **Influence** | High | **Power** | Medium |
| **Interest** | Very High | |

**Goals**:
- Publish government decisions on time
- Withhold only justified information
- Respond to Woo requests efficiently
- Avoid Woo violation fines

**Pain Points**:
- Manual assessment is time-consuming
- Uncertainty about refusal grounds
- Fear of publishing sensitive information
- Tight publication timelines

**RACI**:
- **Responsible**: Woo publication process
- **Accountable**: Woo compliance
- **Consulted**: On Woo workflow design

---

#### S7: Legal Counsel

| Attribute | Value |
|-----------|-------|
| **Category** | Internal - Advisory |
| **Size** | ~100 |
| **Role** | Legal advice on government decisions |
| **Influence** | High | **Power** | Medium |
| **Interest** | High | |

**Goals**:
- Ensure decisions are legally sound
- Minimize legal liability
- Protect government interests

**Pain Points**:
- Difficulty finding relevant precedents
- Uncertainty about new legal requirements
- Fear of decisions being overturned

**RACI**:
- **Consulted**: On Woo classification, refusal grounds
- **Responsible**: Legal review of templates

---

#### S8: Security Officer

| Attribute | Value |
|-----------|-------|
| **Category** | Internal - Security |
| **Size** | ~100 |
| **Role** | Information security |
| **Influence** | High | **Power** | Medium |
| **Interest** | Very High | |

**Goals**:
- Prevent data breaches
- Ensure access controls work
- Maintain security certifications

**Pain Points**:
- Government systems are attractive targets
- Insider threat from employees
- Complexity of cross-organization access

**RACI**:
- **Responsible**: Security controls, incident response
- **Consulted**: On access control design

---

### 1.2 External Stakeholders

#### S9: Citizens (Burgers)

| Attribute | Value |
|-----------|-------|
| **Category** | External - Data Subjects |
| **Size** | ~17 million Dutch population |
| **Role** | Subjects of government decisions, requesters of information |
| **Influence** | Medium (collectively) | **Power** | Low |
| **Interest** | High | |

**Goals**:
- Access government decisions affecting them
- Understand how decisions are made
- Exercise data subject rights (SAR, rectification, erasure)
- Receive government services efficiently

**Pain Points**:
- Government information difficult to find
- Unclear how to request information
- Fear of personal data misuse
- Language barriers (for non-Dutch speakers)

**Vulnerability Factors**:
- **Imbalanced power relationship**: Government has authority over citizens
- **Information asymmetry**: Citizens don't know what data government holds
- **Dependency**: Citizens need government services

**Data Subject Rights**:
- Right to access (SAR)
- Right to rectification
- Right to erasure (after retention)
- Right to portability
- Right to object to processing
- Right to explanation for automated decisions

---

#### S10: Businesses (Bedrijven)

| Attribute | Value |
|-----------|-------|
| **Category** | External - Data Subjects |
| **Size** | ~2 million registered businesses |
| **Role** | Subjects of government decisions, permit applicants |
| **Influence** | Medium (via lobbying) | **Power** | Low |
| **Interest** | High | |

**Goals**:
- Obtain permits and licenses quickly
- Understand regulatory requirements
- Access government contracts
- Protect business confidentiality

**Pain Points**:
- Permit processes are slow
- Unclear requirements
- Fear of competitive information disclosure

**Vulnerability Factors**:
- **Economic dependency**: Need permits to operate
- **Competitive harm**: Business secrets in government documents

---

#### S11: Journalists and Media

| Attribute | Value |
|-----------|-------|
| **Category** | External - Information Requesters |
| **Size** | ~10,000 |
| **Role** | Investigate government activities, inform public |
| **Influence** | High (public agenda) | **Power** | Low |
| **Interest** | Very High | |

**Goals**:
- Access government information quickly
- Find stories and scandals
- Hold government accountable

**Pain Points**:
- Woo request delays
- Redacted documents
- Unclear refusal grounds

---

#### S12: Suppliers and Vendors

| Attribute | Value |
|-----------|-------|
| **Category** | External - Partners |
| **Size** | ~50 (AI, cloud, integration) |
| **Role** | Provide technology and services |
| **Influence** | Medium | **Power** | Low |
| **Interest** | High | |

**Goals**:
- Win government contracts
- Maintain long-term relationships
- Provide reliable services

**Pain Points**:
- Complex procurement process
- Strict security requirements
- Pressure to lower prices

---

#### S13: Regulators and Oversight Bodies

| Attribute | Value |
|-----------|-------|
| **Category** | External - Oversight |
| **Size** | ~10 |
| **Role** | Enforce compliance (Woo, AVG, Archiefwet) |
| **Influence** | Very High | **Power** | High |
| **Interest** | Very High | |

**Types**:
- Autoriteit Persoonsgegevens (AP) - GDPR enforcement
- Inspectie Overheid en Openbare Besturen (OOB) - Woo enforcement
- Nationaal Archief - Archiefwet enforcement

**Goals**:
- Ensure government compliance
- Investigate violations
- Issue fines and sanctions

---

### 1.3 Vulnerable Data Subjects

#### S14: Children and Minors

| Attribute | Value |
|-----------|-------|
| **Category** | External - Vulnerable |
| **Size** | ~3 million (under 18) |
| **Vulnerability** | Cannot consent, may not understand rights |
| **Protection Required** | Enhanced |

**Additional Safeguards**:
- No direct interaction with system (no accounts)
- Data appears in documents (education, youth services)
- Parental consent for processing

---

#### S15: Elderly and Disabled

| Attribute | Value |
|-----------|-------|
| **Category** | External - Vulnerable |
| **Size** | ~3 million (65+) |
| **Vulnerability** | May need assistance exercising rights |
| **Protection Required** | Accessibility support |

**Additional Safeguards**:
- WCAG 2.1 AA compliance
- Plain language explanations
- Assistance options for SAR requests

---

#### S16: Asylum Seekers and Undocumented

| Attribute | Value |
|-----------|-------|
| **Category** | External - Vulnerable |
| **Size** | ~200,000 |
| **Vulnerability** | Language barriers, fear of authorities |
| **Protection Required** | Enhanced |

**Additional Safeguards**:
- Translation services
- Confidential handling
- No disclosure without consent (except legal requirements)

---

## 2. Stakeholder Impact Assessment

### 2.1 Impact of IOU-Modern by Stakeholder

| Stakeholder | Positive Impact | Negative Impact/Risk | Mitigation |
|-------------|-----------------|---------------------|------------|
| **S1: Employees** | Faster document retrieval, AI assistance | Learning curve, job changes | Training, phased rollout |
| **S2: Domain Owners** | Better control, compliance visibility | Responsibility burden | Clear guidelines, support |
| **S3: Info Managers** | Automated retention tracking | Migration effort | Migration support |
| **S4: CIO/IT** | Reduced vendor lock-in | Integration complexity | API-first design |
| **S5: DPO** | Better PII tracking | New AI risks (profiling) | DPIA, privacy controls |
| **S6: Woo Officers** | Automated Woo assessment | AI errors | Human approval required |
| **S7: Legal** | Better precedent search | AI liability concerns | Audit trail, oversight |
| **S8: Security** | Centralized access control | Target attractiveness | Security reviews |
| **S9: Citizens** | Easier Woo access | Profiling concerns | Transparency, opt-out |
| **S10: Businesses** | Faster permits | Confidentiality risk | Careful classification |
| **S11: Journalists** | Better search | None identified | — |
| **S12: Suppliers** | New opportunities | Complex requirements | Clear contracts |
| **S13: Regulators** | Easier auditing | None identified | — |
| **S14-16: Vulnerable** | Better service accessibility | Potential profiling | Enhanced protections |

### 2.2 Power vs. Interest Matrix

```
                    | Low Influence        | High Influence
                    | (Keep Satisfied)     | (Manage Closely)
                    +---------------------+---------------------+
    High Interest   | S9: Citizens         | S5: DPO
                    | S10: Businesses      | S13: Regulators
                    | S14-16: Vulnerable   | S4: CIO/IT
                    +---------------------+---------------------+
                    |                     |
    Low Interest    | S12: Suppliers       | S7: Legal
                    |                     | S8: Security
                    +---------------------+---------------------+
```

**Management Strategy**:
- **Manage Closely**: DPO, Regulators, CIO — Weekly engagement
- **Keep Satisfied**: Citizens, Businesses — Regular communication
- **Monitor**: Legal, Security, Suppliers — As-needed engagement

---

## 3. Communication Plan

### 3.1 Stakeholder Communication Matrix

| Stakeholder | Frequency | Channel | Content | Owner |
|-------------|-----------|---------|---------|-------|
| S1: Employees | Monthly | Intranet, email | Feature updates, training | Product Team |
| S2: Domain Owners | Bi-weekly | Newsletter, webinars | Compliance updates, best practices | Product Owner |
| S5: DPO | Weekly | Meetings, reports | Risk assessments, DPIA updates | Privacy Officer |
| S6: Woo Officers | Monthly | Workshops | Workflow changes, compliance tips | Woo Team |
| S9: Citizens | Quarterly | Public portal | Transparency reports, Woo stats | Communications |
| S13: Regulators | Annually | Formal reports | Compliance certification, audit results | Compliance Officer |

### 3.2 Consultation Requirements

| Stakeholder | Method | Timing | Purpose |
|-------------|---------|---------|---------|
| S5: DPO | Interviews | Pre-design | DPIA guidance |
| S2: Domain Owners | Workshops | Beta testing | Workflow feedback |
| S9: Citizens | Surveys | Pre-launch | Privacy notice feedback |
| S6: Woo Officers | Focus groups | Pre-launch | Workflow validation |

---

## 4. Data Subject Rights Mapping

### 4.1 Rights by Stakeholder Category

| GDPR Right | S1: Employees | S9: Citizens | S10: Businesses | S14-16: Vulnerable |
|------------|---------------|--------------|-----------------|-------------------|
| **Access (Art 15)** | SAR API | SAR API | SAR API | SAR API + assistance |
| **Rectification (Art 16)** | Profile update | DPO request | DPO request | DPO request + assistance |
| **Erasure (Art 17)** | After retention | After retention | After retention | After retention + legal review |
| **Portability (Art 20)** | Export API | Export API | Export API | Export API |
| **Object (Art 21)** | Profiling opt-out | Profiling opt-out | Profiling opt-out | Profiling opt-out |
| **Restrict (Art 18)** | Future | Future | Future | Future |
| **Decisions (Art 22)** | Human review available | Human review available | Human review available | Human review + explanation |

---

## 5. Stakeholder RACI Matrix

### 5.1 Key Project Activities

| Activity | S1: Employees | S2: Domain Owners | S5: DPO | S6: Woo Officers | S13: Regulators |
|----------|--------------|-------------------|---------|-----------------|-----------------|
| Requirements gathering | C | R | C | C | I |
| Data model design | I | C | A | I | I |
| Privacy design | I | I | A | I | C |
| Woo workflow design | I | R | C | A | I |
| UAT testing | R | R | C | C | I |
| Go-live decision | I | C | A | C | I |
| Annual DPIA review | I | I | A | I | R |

**Legend**: R=Responsible, A=Accountable, C=Consulted, I=Informed

---

## 6. Success Criteria by Stakeholder

| Stakeholder | Success Metric | Target |
|-------------|---------------|--------|
| S1: Employees | Search finds relevant documents | >80% satisfaction |
| S2: Domain Owners | Compliance confidence | >90% feel confident |
| S5: DPO | DPIA compliance | 100% high-risk processing assessed |
| S6: Woo Officers | On-time publication | >95% published within deadline |
| S9: Citizens | Woo request satisfaction | >80% satisfied with response |
| S13: Regulators | Audit findings | Zero critical findings |

---

## 7. Glossary

| Term | Definition |
|------|------------|
| **DPO** | Data Protection Officer (Functionaris Gegevensbescherming) |
| **SAR** | Subject Access Request |
| **Woo** | Wet open overheid (Government Information Act) |
| **AVG** | Algemene verordening gegevensbescherming (GDPR) |
| **Archiefwet** | Dutch Archives Act |
| **Domain Owner** | User responsible for an InformationDomain |
| **Information Manager** | Informatiebeheerder (records manager) |
| **NER** | Named Entity Recognition |

---

**END OF STAKEHOLDER ANALYSIS**

## Generation Metadata

**Generated by**: ArcKit `/arckit:stakeholders` command
**Generated on**: 2026-03-20
**ArcKit Version**: 4.3.1
**Project**: IOU-Modern (Project 001)
**AI Model**: Claude Opus 4.6
