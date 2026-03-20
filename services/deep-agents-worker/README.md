# Deep Agents worker (Python, Docker)

Geïsoleerde runtime voor de Camunda-service task `iou-deep-agent`. De Rust-API (`iou-api`) proxy’t hiernaartoe via `POST /api/internal/camunda/deep-agent`.

## Lokale run

```bash
export CAMUNDA_WORKER_TOKEN=dev-token
export IOU_API_TOOL_BASE_URL=http://127.0.0.1:8000
pip install -r requirements.txt
uvicorn app.main:app --reload --port 8091
```

## Productie

- Bouw en draai via [`docker-compose.document-workflow.yml`](../../docker-compose.document-workflow.yml) (service `deep-agents`).
- **Stub (standaard):** `POST /internal/run` gebruikt `run_agent_stub` (geen extra pakketten).
- **Echte Deep Agents:** uncomment de pakketten in [`requirements-deepagents.txt`](requirements-deepagents.txt), installeer ze in de image (`pip install -r requirements.txt -r requirements-deepagents.txt`), zet `USE_DEEPAGENTS=1` en implementeer [`app/deepagents_runner.py`](app/deepagents_runner.py). Houd tools beperkt tot HTTP-calls naar `iou-api` (geen secrets in procesvariabelen).
