"""
Optionele integratie met LangChain Deep Agents.

1. pip install -r requirements-deepagents.txt (pakketten uncommenten).
2. Implementeer `run_deep_agent(document_id, prompt_hint) -> dict` hieronder.
3. In `main.py`: roep deze functie aan vóór de stub wanneer `USE_DEEPAGENTS=1`.
"""

from __future__ import annotations

import os
from typing import Any


async def run_deep_agent_if_configured(
    document_id: str, prompt_hint: str | None
) -> dict[str, Any] | None:
    if os.environ.get("USE_DEEPAGENTS", "").lower() not in ("1", "true", "yes"):
        return None
    # Voorbeeld na installatie van deepagents + LangGraph:
    # from deepagents import create_deep_agent
    # agent = create_deep_agent(...)
    # result = await agent.ainvoke(...)
    # return {"deepAgentSummary": ..., "toolCalls": [...]}
    raise NotImplementedError(
        "Zet USE_DEEPAGENTS=1 alleen na implementatie van run_deep_agent_if_configured "
        "met tools die alleen iou-api HTTP aanroepen."
    )
