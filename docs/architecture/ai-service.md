# AI als aparte service

## Doel

- **iou-api** blijft de hoofd-API (data, Camunda-bridge, GraphRAG); zware of gevoelige LLM-calls kunnen in een **eigen proces** draaien met eigen schaal en secrets.
- **Deep agents** (`services/deep-agents-worker`, Python) blijven een **tweede service**: orchestratie + tools; zij kunnen voor tekstgeneratie naar deze gateway praten i.p.v. direct naar een cloud- of SLM-endpoint.
- **SLM** (klein lokaal model, o.a. Ollama) heeft een **eigen configuratieprefix** (`SLM_*`), los van de primaire cloud-LLM (`LLM_*`).

**Welk Ollama-model?** Standaard **`qwen2.5:3b`** (licht, meertalig). Mistral lokaal: **`ministral-3:3b`** (3B) of **`mistral`** (7B). Volledige matrix + embeddings: [**ollama-models.md**](./ollama-models.md).

**Apple Silicon (lokaal, zonder Ollama):** MLX + Metal — zie [**mlx-apple-silicon.md**](./mlx-apple-silicon.md) en het voorbeeld **`examples/apple-mlx-document-digest/`** (documentdigest).

## Component: `iou-ai-service`

Rust-binary in het workspace: `crates/iou-ai-service`.

| Route | Doel |
|--------|------|
| `GET /health` | Liveness |
| `POST /v1/chat` | Chat via `LlmConfig::from_env()` → `LLM_*` (o.a. Mistral) |
| `POST /v1/slm/chat` | Chat via `LlmConfig::from_slm_env()` → `SLM_*` (OpenAI-compatibel, o.a. Ollama) |

Body (beide chat-routes):

```json
{
  "messages": [
    { "role": "user", "content": "..." }
  ]
}
```

Antwoord:

```json
{ "content": "..." }
```

### Optionele beveiliging

Als `IOU_AI_SERVICE_TOKEN` is gezet, moet elke chat-request header `X-IOU-AI-Token` meesturen met dezelfde waarde.

### Poort / bind

- `IOU_AI_SERVICE_HOST` (default `0.0.0.0`)
- `IOU_AI_SERVICE_PORT` (default `8090`)

### Lokaal draaien

```bash
export LLM_API_KEY=...   # voor /v1/chat
export SLM_BASE_URL=http://127.0.0.1:11434   # voor /v1/slm/chat (Ollama)
cargo run -p iou-ai-service
```

Docker: `docker build -f crates/iou-ai-service/Dockerfile .`

## Integratie met `iou-api` (volgende stap)

Nu gebruikt **iou-api** nog **in-process** `iou-ai` (pipeline, GraphRAG, templates). Dat is bewust: gedeelde state en lage latency.

Wil je alleen LLM **remote** hebben, dan kun je stap voor stap:

1. Clients of workers laten praten met `http://iou-ai-service:8090/v1/chat`.
2. Later optioneel: dunne HTTP-client in `iou-ai` (`RemoteLlmBackend`) en een feature-flag in de pipeline.

Zet daarvoor in clients/workers bijvoorbeeld:

- `IOU_AI_SERVICE_URL=http://iou-ai-service:8090`

(geen verplichte wijziging in iou-api tot je die client wired.)

## Deep agents

- **Vandaag:** `iou-api` roept `DEEP_AGENT_SERVICE_URL` aan (Python).
- **Aanbevolen:** in de Python-worker LLM-calls naar `iou-ai-service` laten gaan (één plek voor rate limits, logging, modelkeuze). Geen wijziging in het Camunda-contract nodig.

## Environment-overzicht

| Variabele | Gebruikt door |
|-----------|----------------|
| `LLM_API_KEY`, `LLM_MODEL`, `LLM_BASE_URL` | Primaire LLM (`/v1/chat`) |
| `SLM_BASE_URL`, `SLM_MODEL`, `SLM_API_KEY` | SLM (`/v1/slm/chat`) |
| `IOU_AI_SERVICE_TOKEN` | Optioneel: gateway-auth |
| `DEEP_AGENT_SERVICE_URL` | iou-api → Python deep agents |

Zie ook root **`.env.example`**.
