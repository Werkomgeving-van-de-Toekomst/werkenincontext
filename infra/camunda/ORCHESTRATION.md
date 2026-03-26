# Orchestratie: Rust vs Camunda

## Camunda vs Deep Agents (rolverdeling)

In lijn met een **hybride** procesarchitectuur (zie [`README.md`](README.md) — strategische context):

- **Camunda / Zeebe** — vast BPMN-pad: wachtpunten, goedkeuring, correlatie op `documentId`, job-retries en incidenten. Geschikt voor **kernproces** en audittrail op procesniveau.
- **Deep Agents (Python, geïsoleerd in Docker)** — één of meerdere **service tasks** met tools die **uitsluitend** `iou-api` aanroepen (geen willekeurige shell op productie). Geschikt voor dynamischer werk: dieper zoeken, samenvatten, meerdere tool-stappen — mits output weer **terug** in variabelen of domeinopslag landt.
- **Rust-pipeline** (`iou-run-pipeline`) — vaste keten Research → Content → Compliance → Review; blijft de bron voor deterministische bedrijfslogica en finalize naar DuckDB + S3.

Zo combineer je **structuur** (Camunda + Rust) met **flexibiliteit** (Deep Agents) zonder dubbele “waarheid” voor één documentinstantie: zie modustabel hieronder.

## Beslissing (plan §4)

| Modus | `IOU_DOCUMENT_WORKFLOW` | Bron van waarheid | Documentstatus / UI |
|-------|-------------------------|-------------------|---------------------|
| **Hybride (legacy)** | unset of `rust` | `WorkflowOrchestrator` + DuckDB `documents.state` | WebSocket via agent-simulatie in orchestrator |
| **Camunda** | `camunda` (of `c8` / `zeebe`) | Zeebe proces + DuckDB voor domeindata | WebSocket bij o.a. Camunda-start en na `run-pipeline` (zie tabel) |

Er is **geen dubbele orchestratie** voor één document: in Camunda-modus start de API **geen** `WorkflowOrchestrator` voor `POST /documents/create`.

## WebSocket / status-mapping (Camunda-modus)

| Gebeurtenis | `StatusMessage` | Opmerking |
|-------------|-------------------|-----------|
| Proces gestart (API) | `Started` — agent `camunda` | Direct na `create_process_instance` |
| Rust-pipeline job klaar (worker → API) | `Started` + `Progress` 100% — agent `camunda_pipeline` | Zie `run_pipeline_job` |
| Goedkeuring (API) | (geen extra WS tenzij je later uitbreidt) | Zeebe gaat verder via `document_approved` |

Uitbreiding mogelijk: Zeebe exporter / Hazelcast → webhook → API broadcast (niet in deze repo).

## Procesvariabelen (start)

Zie [`VARIABLES.md`](VARIABLES.md). Geen secrets in variabelen — alleen IDs en vlaggen; worker-auth via `CAMUNDA_WORKER_TOKEN` header.
