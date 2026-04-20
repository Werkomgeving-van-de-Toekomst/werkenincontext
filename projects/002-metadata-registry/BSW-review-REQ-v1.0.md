# Review: Metadata Registry Requirements vs BSW Zaak/Dossierbeheer

**Document**: ARC-002-REQ-v1.1.md
**Input**: 20260324 Zaak dossierbeheer conform BSW.docx
**Date**: 2026-04-19
**Reviewer**: Generated review against BSW requirements

**Status**: ✅ Applied - All recommendations incorporated in v1.1

---

## Executive Summary

The Metadata Registry Service requirements (ARC-002-REQ) align well with the BSW (Beter Samenwerken) architecture principles for zaak/dossierbeheer. However, several gaps and refinements are identified to ensure full compliance with the BSW paradigm.

**Key Alignment**: The requirements correctly center on **informatieobjecten** (dataobject + metadata) as the core concept, matching BSW's paradigm shift from document-centric to information-centric management.

**Critical Gaps Identified**: 8 areas requiring attention
**Recommendations**: 15 specific requirements to add or modify

---

## Detailed Analysis by BSW Concept

### 1. Informatieobject Centricity ✅ PARTIALLY ALIGNED

**BSW Requirement**: "In de BSW architectuur staat het informatieobject centraal. Binnen BSW wordt hier een dataobject bedoeld inclusief de metadata waarmee de betekenis van het dataobject zo volledig mogelijk geduid kan worden."

**Current REQ Coverage**:
- ✅ Entity 2: Informatieobject with beveiligingsniveau, privacy_level
- ✅ Metadata fields defined (naam, objecttype, etc.)
- ⚠️ Missing: Explicit statement that information objects are the PRIMARY abstraction

**Gap**: The requirements don't explicitly state the BSW paradigm principle that everything is an informatieobject, not just documents.

**Recommendation**: Add to Executive Summary:
> "This service implements the BSW architecture principle where the **informatieobject** (dataobject + metadata) is the core abstraction, not the document or dataobject alone."

---

### 2. Dynamische vs Persistente Opslag ❌ MISSING

**BSW Requirement**: "Minimaal de opsplitsing tussen 'dynamische' opslag (dataobject is nog in bewerking en heeft het nog geen definitieve status) en 'persistente' opslag (dataobject mag niet meer gewijzigd worden en is in principe voor iedere belanghebbende vindbaar en beschikbaar)."

**Current REQ Coverage**:
- ❌ No explicit concept of dynamic vs persistent storage
- ⚠️ Time-based validity (geldig_vanaf/geldig_tot) partially addresses this
- ❌ No status field for "in bewerking" vs "gepersisteerd"

**Gap**: Critical BSW concept missing. The document explicitly states that concepts may need to be persisted even when not "final" from a workflow perspective.

**Recommendation**: Add new requirement:
```markdown
#### FR-MREG-16: Dynamic vs Persistent Status

**Description**: System shall distinguish between dynamic (in bewerking) and persistent (gepersisteerd) information objects

**Relates To**: BSW architectural principle

**Acceptance Criteria**:
- Given entity created, when status=dynamisch, then entity is mutable
- Given entity persisted, when status=persistent, then entity is read-only
- Given workflow transition, when entity finalized, then status changes to persistent
- Given persistent entity, when modification attempted, then error returned

**Data Requirements**:
- **Fields**: status (enum: dynamisch, gepersistent, gearchiveerd)
- **Transitions**: dynamisch → gepersistent → gearchiveerd

**Priority**: MUST
```

---

### 3. Informatieobject Catalogus ⚠️ PARTIALLY ALIGNED

**BSW Requirement**: "Beheer informatieobject catalogus - doel: het uitgebreid en op basis van context kunnen zoeken naar informatie die aanwezig is in de binnen de organisatie aanwezige informatieobjecten"

**Current REQ Coverage**:
- ✅ FR-MREG-8: Full-Text Search
- ✅ Entity relationships via edge collections
- ⚠️ Missing: Explicit catalog entity that stores location references
- ❌ Missing: Search on context (werkproces, zaak context)

**Gap**: BSW specifies a catalog that stores metadata + LOCATION (not the content itself). The current requirements conflate search with storage.

**Recommendation**: Add new entity:
```markdown
#### Entity: InformatieobjectCatalogus

**Description**: Stores metadata and location references to information objects in archives

**Attributes**:
| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| _key | String | Yes | Unique identifier |
| informatieobject_id | String | Yes | Reference to Informatieobject |
| locatie_uri | String | Yes | Storage location (CDD+, file system) |
| zoek_index | Text | Yes | Full-text searchable content |
| context_metadata | JSON | Yes | Context for search (zaak, werkproces) |

**Relationships**:
- Many-to-one with Informatieobject
- Many-to-one with Zaak/Dossier (via catalogus_zaak edge)
```

---

### 4. Metadata Requirements ⚠️ GAPS IDENTIFIED

**BSW Required Metadata**:
1. **Informatiecategorie** (Woo verplicht)
2. **Documentomschrijving** (logische omschrijving)
3. **Documenttype** (voor zoekfunctionaliteit)
4. **Samenvatting** (kan automatisch via AI)
5. **Zaak/dossier ID** (uniek op organisatieniveau)
6. **Type zaak/dossier** (Woo verplicht)

**Current REQ Coverage**:
- ✅ naam, omschrijving
- ❌ Missing: informatiecategorie (Woo required)
- ❌ Missing: documenttype (search requirement)
- ❌ Missing: zaak_id (for grouping)
- ⚠️ samenvatting mentioned in AI enrichment (FR-MREG-13) but not as core field

**Recommendation**: Add to Informatieobject entity:
```markdown
| informatiecategorie | String | Yes | Woo category | Enum: Woo categories |
| documenttype | String | Yes | Document type classification | Enum: standard types |
| zaak_id | String | No | Related case/dossier | Foreign key |
| samenvatting | Text | No | AI-generated summary | Indexable |
```

---

### 5. Context-Aware Search ❌ MISSING

**BSW Requirement**: "Zoek en vind functionaliteit - doel: Het kunnen zoeken en vinden van relevante informatie. Dit wil zeggen op basis van een combinatie van de eigen context en de context waarin de informatie is ontstaan."

**Current REQ Coverage**:
- ✅ FR-MREG-8: Full-Text Search
- ❌ Missing: Context-based search (user context + information origin context)
- ❌ Missing: Search on dossier level
- ❌ Missing: Gegevensspecificatie search (gegevensboekhouding)

**Gap**: BSW emphasizes that search should work from USER context and INFORMATION origin context. The current requirements only implement basic full-text search.

**Recommendation**: Enhance FR-MREG-8:
```markdown
#### FR-MREG-8: Context-Aware Search

**Description**: System shall provide search combining user context with information origin context

**Acceptance Criteria**:
- Given search from zaak context, when query executed, then results prioritized from same zaak
- Given search with werkproces context, when query executed, then results filtered by process
- Given user context (role, department), when query executed, then results scoped to user context
- Given search without context, when query executed, then all accessible results returned

**Context Dimensions**:
- User context: role, organization, active zaak/dossier
- Information origin: zaak_id, werkproces, creatie_datum
- Combined scoring: context_match × relevance_score
```

---

### 6. Workflow Integration ⚠️ PARTIALLY ALIGNED

**BSW Requirement**: "Workflow is een voorgedefineerde reeks van activiteiten die moeten worden uitgevoerd... Een workflow is vaak onderdeel van een taakapplicaties. Het hebben van een 'externe' workflow zou... leiden tot het dupliceren van logica"

**Current REQ Coverage**:
- ✅ UC-MREG-3: Woo Publication Workflow (status transitions)
- ✅ FR-MREG-9: Woo Publication Workflow
- ❌ Missing: General workflow integration pattern
- ❌ Missing: Status field for information object lifecycle

**Gap**: The requirements implement a specific Woo workflow but don't provide a general pattern for workflow integration as specified in BSW.

**Recommendation**: Add workflow integration requirement:
```markdown
#### FR-MREG-17: Workflow Status Integration

**Description**: System shall support external workflow status management for information objects

**Acceptance Criteria**:
- Given external workflow, when status changes, then entity updated with workflow_status
- Given workflow transition to persistent, when triggered, then entity becomes read-only
- Given workflow event, when received, then audit trail updated with workflow context

**Integration Pattern**: Event-driven status updates via webhook/message queue

**Priority**: SHOULD
```

---

### 7. Autorisatie op Informatieobject Niveau ❌ MISSING

**BSW Requirement**: "Toegang tot informatieobjecten moet ook mogelijk zijn op basis van het informatieobject en NIET op het niveau van een eventuele applicatie... bij samenwerking in bijvoorbeeld een keten kan het zijn dat het document/dossier/gegevensset onder meerdere zorgdragers valt"

**Current REQ Coverage**:
- ✅ FR-MREG-5: Row-Level Security (organization level)
- ❌ Missing: Authorization at individual information object level
- ❌ Missing: Share capabilities for collaboration
- ❌ Missing: Multi-caretaker scenarios

**Gap**: Critical BSW requirement for collaboration. Current RLS only filters by organization, not by specific information object grants.

**Recommendation**: Add new requirement:
```markdown
#### FR-MREG-18: Information Object Level Authorization

**Description**: System shall support authorization grants at individual information object level

**Relates To**: BSW samenwerking requirement

**Acceptance Criteria**:
- Given information object, when shared with user, then specific access granted
- Given shared object, when owner revokes access, then access removed immediately
- Given multi-caretaker scenario, when object archived, then preserved for each caretaker
- Given user with object grant, when query executed, then only accessible objects returned

**Data Requirements**:
- **Entity**: InformatieobjectRecht (object_id, user_id, recht_type: lezen/bewerken)
- **Inheritance**: Zaak-level rights inherit to contained objects

**Priority**: MUST (for samenwerking use case)
```

---

### 8. AI Enrichment Requirements ⚠️ PARTIALLY ALIGNED

**BSW Requirement**: "Informatieobject verrijking - samenvatten tekst, vertalen tekst, analyseren document, verrijken metadata... Let op! De resultaten van deze AI bewerkingen zullen in veel gevallen wel getoetst moeten worden door de medewerker"

**Current REQ Coverage**:
- ✅ Phase 6 Entities: AIEnrichment, AISummary, AITranslation
- ✅ BR-MREG-013: PII detection via AI
- ❌ Missing: Human validation requirement for AI results
- ❌ Missing: AI result status (getoetst vs ongecontroleerd)

**Gap**: BSW explicitly requires human validation of AI results. This is not captured in the requirements.

**Recommendation**: Add validation requirement:
```markdown
#### FR-MREG-19: AI Result Validation

**Description**: AI enrichment results must support human validation before trusted use

**Acceptance Criteria**:
- Given AI enrichment completed, when result created, then status=ongecontroleerd
- Given human validation, when approved, then status=getoetst and validator_id stored
- Given unvalidated result, when used in critical process, then warning displayed
- Given AI result, when rejected, then alternatieve_waarde stored

**Fields**:
- ai_status: enum (ongecontroleerd, getoetst, afgewezen)
- getoetst_door: user_id
- getoetst_op: timestamp
- vertrouwensscore: 0-1 (from AI model)

**Priority**: MUST (for Woo publication automation)
```

---

### 9. CDD+ Integration ✅ ALIGNED

**BSW Requirement**: "CDD+ is binnen JenV de aangewezen applicatie voor archivering (basisvoorziening). Aansluiten op deze dienst t.b.v. archivering is dan ook op basis van 'comply or explain'."

**Current REQ Coverage**:
- ✅ FR-MREG-12: CDD+ Archive Integration
- ✅ INT-MREG-4: Integration with CDD+ Archive
- ✅ Archive metadata entities (Phase 7)

**Status**: Well aligned with BSW requirements.

---

### 10. Zaak/Dossier Metadata Overerving ⚠️ PARTIALLY ALIGNED

**BSW Requirement**: "Zaak / dossier beheer... op basis vanuit het businessperspectief gewenste samenhang met andere informatieobjecten. Dit wil zeggen dat de metadata van een zaak/dossier 'overerft' kunnen worden op de informatieobjecten."

**Current REQ Coverage**:
- ✅ Entity relationships via edge collections
- ✅ Zaak entity implied (mentioned but not fully defined)
- ❌ Missing: Explicit metadata inheritance mechanism
- ❌ Missing: Zaak entity definition in Appendix C

**Gap**: Metadata inheritance is a core BSW concept but not explicitly designed.

**Recommendation**: Add inheritance requirement:
```markdown
#### FR-MREG-20: Metadata Inheritance

**Description**: Metadata from zaak/dossier shall be inheritable by contained information objects

**Acceptance Criteria**:
- Given zaak with metadata, when informatieobject created within zaak, then inherited metadata available
- Given inherited metadata, when zaak metadata updated, then objects can inherit new values
- Given information object, when displayed, then inherited and own metadata both visible
- Given inheritance conflict, when both defined, then object-level metadata takes precedence

**Implementation**: Edge collection with inheritance_type field (overerven, overschrijven)

**Priority**: SHOULD
```

---

## Summary Table: Gaps and Recommendations

| # | BSW Concept | Current Status | Gap Type | Priority | Recommendation |
|---|-------------|----------------|----------|----------|----------------|
| 1 | Informatieobject centrality | ⚠️ Partial | Clarification | Low | Add explicit BSW paradigm statement |
| 2 | Dynamische/Persistente opslag | ❌ Missing | Critical | MUST | Add status field and transitions |
| 3 | Informatieobject catalogus | ⚠️ Partial | Entity | MUST | Add catalog entity with location |
| 4 | Metadata requirements | ⚠️ Gaps | Data fields | MUST | Add informatiecategorie, documenttype, zaak_id |
| 5 | Context-aware search | ❌ Missing | Functional | MUST | Enhance search with context dimensions |
| 6 | Workflow integration | ⚠️ Partial | Pattern | SHOULD | Add general workflow integration pattern |
| 7 | Object-level authorization | ❌ Missing | Security | MUST | Add information object grants |
| 8 | AI validation | ❌ Missing | Process | MUST | Add human validation workflow |
| 9 | CDD+ integration | ✅ Complete | - | - | No action needed |
| 10 | Metadata inheritance | ⚠️ Partial | Functional | SHOULD | Add inheritance mechanism |

---

## Priority Actions

### MUST (Critical for BSW compliance):
1. Add dynamisch/gepersistent status to Informatieobject
2. Create InformatieobjectCatalogus entity
3. Add missing metadata fields (informatiecategorie, documenttype, zaak_id)
4. Implement context-aware search
5. Add information object-level authorization
6. Add AI result validation workflow

### SHOULD (Important for BSW alignment):
1. Add general workflow integration pattern
2. Implement metadata inheritance mechanism
3. Define Zaak entity explicitly

### COULD (Nice to have):
1. External workflow component consideration (per BSW note)
2. Robin 2.0 integration pattern for context matching

---

## Conclusion

The Metadata Registry requirements provide a solid foundation aligned with BSW principles. The core concept of information objects as data + metadata is correctly implemented. However, to fully comply with BSW zaak/dossierbeheer requirements, the gaps identified above should be addressed, particularly:

1. The dynamic/persistent storage distinction (foundational BSW concept)
2. Information object catalogus with location references
3. Object-level authorization for collaboration
4. Context-aware search combining user and information context

These enhancements will ensure the Metadata Registry Service fully supports the BSW architecture vision for zaak/dossierbeheer.
