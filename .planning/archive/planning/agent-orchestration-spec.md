# Agent-Orchestrated Document Creation with Human-in-the-Loop

## Project Overview

Volledige herontwerp van de document creatie flow in IOU-Modern, georkestreerd door AI agents met human-in-the-loop goedkeuringsstappen.

## Current State

De huidige implementatie in `crates/iou-ai/src/agents/` bevat:
- Research Agent - Haalt context op uit kennisgraaf
- Content Agent - Genereert document content
- Compliance Agent - Valideert Woo/GDPR compliance
- Review Agent - Kwaliteitscontrole

De pipeline draait sequentieel via `iou-ai/src/agents/pipeline.rs`.

## Requirements

### Functional Requirements

1. **Agent Orchestration**
   - Document creatie moet uitvoerbaar zijn via verschillende agent-architecturen
   - Ondersteuning voor zowel LangGraph (Python) als Rust-native implementatie
   - Agents moeten autonoom beslissingen kunnen nemen met menselijke tussenkomst op kritieke momenten

2. **Human-in-the-Loop Checkpoints**
   - Goedkeuring vereist voordat document wordt gepubliceerd
   - Mogelijkheid voor menselijke tussenkomst en correctie tijdens generatie
   - Audit trail van alle menselijke interventies

3. **State Management**
   - Persistente state voor langlopende document creatie processen
   - Ability to resume interrupted workflows
   - Version tracking van document iteraties

4. **Integration with Existing System**
   - Must integrate with existing IOU-Modern Rust codebase
   - API endpoints voor document aanvraag, status check, en goedkeuring
   - Database opslag in DuckDB

### Architecture Options

#### Option A: LangGraph Integration
- LangGraph (Python) voor agent orchestration
- Rust API communiceert met Python service via gRPC/HTTP
- Best voor complexe agent workflows met graph-based state machines

#### Option B: Rust-Native Implementation
- Volledig Rust implementatie met eigen orchestration engine
- Gebruik makend van bestaande `iou-ai` agents
- Betere integratie met bestaande codebase
- Minder externe dependencies

### Technical Considerations

- **State Machine**: Document flow als state machine met transitions
- **Event Sourcing**: Alle events loggen voor audit trail
- **Concurrency**: Agents moeten parallel kunnen werken waar mogelijk
- **Error Handling**: Graceful degradation als agent faalt
- **Testing**: TDD benadering voor alle componenten

## Open Questions

1. Moet de orchestration engine generiek zijn (herbruikbaar voor andere workflows)?
2. Hoe gaan we om met langlopende agent taken (timeout handling)?
3. Welke UI componenten zijn nodig voor human-in-the-loop interactie?
4. Moeten we支持 zowel synchrone als asynchrone workflows?

## Success Criteria

- Document creatie is volledig georkestreerd door agents
- Menselijke goedkeuring is verplicht voor publicatie
- Alle stappen zijn traceerbaar via audit trail
- Implementatie past binnen bestaande Rust architecture
