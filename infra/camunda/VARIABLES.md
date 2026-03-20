# Zeebe-procesvariabelen — `DocumentPipeline`

Gezet bij [`start_document_pipeline`](../../crates/iou-api/src/camunda/gateway.rs) (camelCase in JSON):

| Variabele | Type (FEEL) | Betekenis |
|-----------|-------------|-----------|
| `documentId` | string | UUID van het document |
| `tenant` | string | Gelijk aan `domain_id` (informatiedomein) |
| `domainId` | string | Domein-id |
| `documentType` | string | Template/documenttype |
| `templateId` | string | Actieve template `id` uit DuckDB |
| `initiatorUserId` | string | UUID aanvrager (nu placeholder tot auth) |
| `workflowVersion` | string | B.v. `1.0.0` — mapt op `WorkflowContext.workflow_version` |
| `correlationKey` | string | Gelijk aan `documentId` voor externe callbacks |
| `runDeepAgent` | bool | Of de Deep Agent-service task moet draaien |

Na job `iou-run-pipeline` (output):

| `requiresHumanApproval` | bool | Gateway naar message catch `document_approved` |
| `complianceScore` | number | |
| `confidenceScore` | number | |
| `finalStatus` | string | draft / in_review / … |

Na job `iou-deep-agent` (Python → stub of Deep Agents):

| `deepAgentSummary` | string | |
| `toolCalls` | array | Auditbare tool-aanroepen |

## BPMN ↔ `WorkflowState` (conceptueel)

| Zeebe / BPMN | Ruwe analogie `WorkflowState` |
|--------------|-------------------------------|
| Proces loopt, jobs actief | `Running` |
| Wacht op `document_approved` | `AwaitingApproval` |
| Proces beeindigd | `Completed` (domeinstatus in DuckDB via pipeline-finalize) |
| Incident / fail job | `Failed` (na retries) |
