"""
Geïsoleerde Deep Agents / LLM-runtime (Python).

- Draait in Docker; praat met `iou-api` alleen via HTTP (tools).
- Optioneel: installeer `deepagents` + LangGraph in de image en vervang `run_agent_stub`.
"""

from __future__ import annotations

import os
from typing import Any

import httpx
from fastapi import FastAPI, Header, HTTPException
from pydantic import BaseModel, ConfigDict, Field

from app.deepagents_runner import run_deep_agent_if_configured

app = FastAPI(title="IOU Deep Agents Worker", version="0.1.0")

EXPECTED_TOKEN = os.environ.get("CAMUNDA_WORKER_TOKEN", "")
IOU_API_BASE = os.environ.get("IOU_API_TOOL_BASE_URL", "http://127.0.0.1:8000").rstrip("/")


class RunRequest(BaseModel):
    model_config = ConfigDict(populate_by_name=True, extra="ignore")
    document_id: str = Field(alias="documentId")
    prompt_hint: str | None = Field(None, alias="promptHint")


def _check_token(x_camunda_worker_token: str | None) -> None:
    if not EXPECTED_TOKEN:
        raise HTTPException(503, "CAMUNDA_WORKER_TOKEN niet geconfigureerd in container")
    if x_camunda_worker_token != EXPECTED_TOKEN:
        raise HTTPException(401, "Ongeldige worker token")


async def _tool_fetch_graphrag_entities(client: httpx.AsyncClient) -> dict[str, Any]:
    """Voorbeeld-tool: read-only GraphRAG-lijst via publieke API-route."""
    r = await client.get(f"{IOU_API_BASE}/api/graphrag/entities", timeout=30.0)
    r.raise_for_status()
    return r.json()


async def run_agent_stub(body: RunRequest) -> dict[str, Any]:
    """
    Placeholder zonder `deepagents`-dependency.
    Voer één gecontroleerde HTTP-call uit als 'tool' en bouw een korte samenvatting.
    """
    async with httpx.AsyncClient() as client:
        try:
            entities = await _tool_fetch_graphrag_entities(client)
            n = len(entities) if isinstance(entities, list) else 1
        except Exception as e:
            return {
                "deepAgentSummary": f"Deep agent (stub) voor document {body.document_id}: tool-call mislukt ({e!s}).",
                "toolCalls": [{"tool": "graphrag_entities", "ok": False}],
            }

    hint = body.prompt_hint or "(geen aanvullende prompt)"
    return {
        "deepAgentSummary": (
            f"Stub-run document={body.document_id}; hint={hint!r}; "
            f"graphrag entities count≈{n}. Vervang door LangChain Deep Agents in productie."
        ),
        "toolCalls": [{"tool": "graphrag_entities", "ok": True, "resultSize": n}],
    }


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}


@app.post("/internal/run")
async def internal_run(
    body: RunRequest,
    x_camunda_worker_token: str | None = Header(default=None, alias="X-Camunda-Worker-Token"),
) -> dict[str, Any]:
    _check_token(x_camunda_worker_token)
    try:
        deep = await run_deep_agent_if_configured(
            body.document_id, body.prompt_hint
        )
    except NotImplementedError:
        deep = None
    if deep is not None:
        return deep
    return await run_agent_stub(body)
