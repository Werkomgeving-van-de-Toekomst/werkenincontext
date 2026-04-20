"""
Workflow Validator Prototype
NL-to-BPMN translation for Dutch government workflows
"""
import os
import json
from typing import Optional, List, Dict, Any
from dotenv import load_dotenv
from fastapi import FastAPI, HTTPException, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel, Field
from openai import OpenAI

load_dotenv()

app = FastAPI(title="Workflow Validator Prototype")

# CORS configuration
allowed_origins = os.getenv("ALLOWED_ORIGINS", "http://localhost:8000").split(",")
app.add_middleware(
    CORSMiddleware,
    allow_origins=allowed_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# OpenAI client
client = OpenAI(api_key=os.getenv("OPENAI_API_KEY"))
MODEL = os.getenv("OPENAI_MODEL", "gpt-4o-mini")


class TranslationRequest(BaseModel):
    description: str = Field(
        ...,
        description="Dutch natural language description of the workflow",
        min_length=10,
    )


class Node(BaseModel):
    id: str
    type: str
    label: str
    action: Optional[str] = None
    assignee: Optional[str] = None
    description: Optional[str] = None


class Edge(BaseModel):
    from_node: str
    to_node: str
    condition: Optional[str] = None


class Workflow(BaseModel):
    id: str
    name: str
    description: str
    nodes: List[Node]
    edges: List[Edge]


class TranslationResponse(BaseModel):
    workflow: Workflow
    clarifications: List[str] = []
    confidence: float = 0.0
    mermaid: str  # Mermaid diagram syntax


# Five hardcoded Dutch workflow templates
TEMPLATES = {
    "vergunning": {
        "name": "Aanvraag Vergunning",
        "description": "Template voor vergunningaanvraagprocessen",
        "example": "Een burger vraagt een vergunning aan. De gemeente ontvangt de aanvraag, controleert compleetheid, vraagt ontbrekende documenten op, neemt een besluit, en stuurt de vergunning of afwijzing.",
    },
    "document_beoordeling": {
        "name": "Document Beoordeling",
        "description": "Template voor documentbeoordelingsworkflows",
        "example": "Een document wordt ingediend voor beoordeling. Eerste beoordelaar bekijkt het, bij twijfel naar tweede beoordelaar. Als goedgekeurd, publiceren. Als afgekeurd, terug naar afzender met opmerkingen.",
    },
    "burger_inquiry": {
        "name": "Burger Inquiry Routing",
        "description": "Template voor routing van burgervragen",
        "example": "Een burger stuurt een vraag. De vraag wordt gecategoriseerd (algemeen, technisch, klacht). Algemene vragen worden direct beantwoord. Technische vragen gaan naar specialist. Klachten starten een onderzoekstraject.",
    },
    "besluit_goedkeuring": {
        "name": "Besluit Goedkeuring",
        "description": "Template voor goedkeuringsprocessen van besluiten",
        "example": "Wanneer een besluit binnenkomt van de provincie, eerst PROVISA checken. Als PROVISA OK, naar directie voor goedkeuring. Als goedgekeurd, archiveren. Als afgekeurd, terug naar provincie met reden.",
    },
    "compliance_controle": {
        "name": "Compliance Controle",
        "description": "Template voor compliance controles",
        "example": "Document wordt gecontroleerd op compliance. Eerste check: basisvereisten. Als fail, terug naar afzender. Als pass, tweede check: geavanceerde regels. Bij beide passes: markeren als compliant.",
    },
}


def get_system_prompt() -> str:
    """Build the system prompt for BPMN generation."""
    templates_text = "\n".join(
        f"- {k}: {v['name']} - {v['description']}\n  Voorbeeld: {v['example']}"
        for k, v in TEMPLATES.items()
    )

    return f"""Je bent een expert in BPMN workflow modellering voor de Nederlandse overheid.

Je taak is het vertalen van Nederlandse taal beschrijvingen naar gestructureerde BPMN workflows.

 Beschikbare templates:
{templates_text}

Regels:
1. Genereer ALTIJD geldige BPMN met deze elementen:
   - startEvent (precies één)
   - endEvent (minstens één)
   - serviceTask (automatische taken)
   - userTask (menselijke goedkeuring)
   - exclusiveGateway (beslissingen met 2+ uitgangen)

2. Elke node heeft een uniek ID (kebab-case), type, label, en optionele beschrijving

3. Edges verbinden nodes met "from" en "to". Gebruik "condition" voor gateway vertakkingen.

4. De output MOET geldig JSON zijn.

5. Als de input onduidelijk is, voeg een vraag toe aan "clarifications"

6. Respons in het Nederlands voor node descriptions en labels

7. Geef ook een Mermaid diagram syntax terug voor visualisatie

Output formaat (STRICT JSON):
{{
  "workflow": {{
    "id": "workflow-001",
    "name": "Naam van workflow",
    "description": "Korte beschrijving",
    "nodes": [
      {{"id": "start", "type": "startEvent", "label": "Start"}},
      ...
    ],
    "edges": [
      {{"from_node": "start", "to_node": "task1"}},
      ...
    ]
  }},
  "clarifications": [],
  "confidence": 0.95
}}
"""


@app.get("/")
async def root():
    """Serve the HTML frontend"""
    from fastapi.responses import FileResponse
    return FileResponse("static/index.html")


@app.get("/api/templates")
async def get_templates():
    """Get available workflow templates"""
    return {
        "templates": [
            {"id": k, "name": v["name"], "description": v["description"], "example": v["example"]}
            for k, v in TEMPLATES.items()
        ]
    }


@app.post("/api/translate", response_model=TranslationResponse)
async def translate_to_bpmn(request: TranslationRequest):
    """
    Translate Dutch natural language to BPMN workflow.

    Returns a workflow definition with nodes, edges, and Mermaid diagram syntax.
    """
    try:
        system_prompt = get_system_prompt()

        response = client.chat.completions.create(
            model=MODEL,
            messages=[
                {"role": "system", "content": system_prompt},
                {
                    "role": "user",
                    "content": f"Vertaal de volgende workflowbeschrijving naar BPMN:\n\n{request.description}"
                },
            ],
            response_format={"type": "json_object"},
            temperature=0.3,
        )

        result = json.loads(response.choices[0].message.content)

        # Extract workflow data
        workflow_data = result.get("workflow", {})

        # Build Mermaid diagram
        mermaid_lines = ["graph TD"]

        # Create node lookup for edge mapping
        node_labels = {}
        for node in workflow_data.get("nodes", []):
            node_id = node["id"]
            node_type = node.get("type", "process")
            label = node.get("label", node_id)
            node_labels[node_id] = label

            # Mermaid syntax for different node types
            if node_type == "startEvent":
                mermaid_lines.append(f"    {node_id}([({label})])")
            elif node_type == "endEvent":
                mermaid_lines.append(f"    {node_id}((( {label} )))")
            elif node_type == "userTask":
                mermaid_lines.append(f"    {node_id}[{label}]")
            elif node_type == "serviceTask":
                mermaid_lines.append(f"    {node_id}[/{label}/]")
            elif node_type == "exclusiveGateway":
                mermaid_lines.append(f"    {node_id}{{{label}}}")
            else:
                mermaid_lines.append(f"    {node_id}[{label}]")

        # Add edges
        for edge in workflow_data.get("edges", []):
            from_node = edge.get("from", edge.get("from_node", ""))
            to_node = edge.get("to", edge.get("to_node", ""))
            condition = edge.get("condition")

            if condition:
                mermaid_lines.append(f"    {from_node}--{condition}-->|{to_node}|")
            else:
                mermaid_lines.append(f"    {from_node}-->|to_node}|")

        mermaid_diagram = "\n".join(mermaid_lines)

        # Build response
        workflow = Workflow(
            id=workflow_data.get("id", "workflow-001"),
            name=workflow_data.get("name", "Workflow"),
            description=workflow_data.get("description", ""),
            nodes=[
                Node(
                    id=n.get("id"),
                    type=n.get("type"),
                    label=n.get("label"),
                    action=n.get("action"),
                    assignee=n.get("assignee"),
                    description=n.get("description"),
                )
                for n in workflow_data.get("nodes", [])
            ],
            edges=[
                Edge(
                    from_node=e.get("from", e.get("from_node", "")),
                    to_node=e.get("to", e.get("to_node", "")),
                    condition=e.get("condition"),
                )
                for e in workflow_data.get("edges", [])
            ],
        )

        return TranslationResponse(
            workflow=workflow,
            clarifications=result.get("clarifications", []),
            confidence=result.get("confidence", 0.0),
            mermaid=mermaid_diagram,
        )

    except Exception as e:
        raise HTTPException(
            status_code=500,
            detail=f"Kon geen diagram genereren: {str(e)}"
        )


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
