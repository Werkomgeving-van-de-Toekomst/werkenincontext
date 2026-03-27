# API Extensions Interview

**Date:** 2025-03-10

## Q1: Pipeline Timeout

**Question:** Hoe lang mag de pipeline draaien voordat deze timeout?

**Answer:** 2 minuten per agent (totaal ~8 min voor 4 agents)

**Implication:** Gebruik `tokio::time::timeout(Duration::from_secs(120))` voor elke agent execution.

## Q2: WebSocket Idle Timeout

**Question:** Hoe lang moeten WebSocket connecties open blijven bij geen activiteit?

**Answer:** 5 minuten

**Implication:** Implementeer heartbeat mechanism dat connectie sluit na 5 min zonder berichten.

## Q3: S3 Access Pattern

**Question:** Hoe moeten documenten aangeboden worden voor download?

**Answer:** API Proxy (niet presigned URLs)

**Implication:**
- Server haalt document uit S3
- Stuurt door naar client
- Extra validatie mogelijk op moment van download
- Hogere server load (double data transfer)
- Betere audit trail

## Q4: Maximum Document Size

**Question:** Wat is de maximum grootte voor een document?

**Answer:** 10 MB

**Implication:** Implementeer size check bij upload en bij API proxy download.

## Q5: Synchronous Testing

**Question:** Moet de pipeline synchroon draaien voor testing?

**Answer:** Ja, synchroon

**Implication:**
- Feature flag voor sync/async execution
- In test mode: direct await van pipeline resultaat
- In production: tokio::spawn met broadcast updates

## Summary of Decisions

| Configuratie | Waarde |
|--------------|--------|
| Agent timeout | 2 minuten per agent |
| WebSocket idle | 5 minuten |
| S3 download | API Proxy (niet presigned) |
| Max document size | 10 MB |
| Test mode | Synchroon, prod async |
