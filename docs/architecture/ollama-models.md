# Aanbevolen lichte Ollama-modellen (documentverwerking & deep agents)

IOU-Modern gebruikt voor **lokale** inferentie het prefix **`SLM_*`** (zie [`ai-service.md`](./ai-service.md)). Standaard staat `SLM_MODEL` op **`qwen2.5:3b`** — een goede balans tussen kwaliteit, meertaligheid (inclusief Nederlands in praktijkgebruik) en geheugen/CPU.

**Mistral lokaal vs. cloud:** zelfde OpenAI-compatibele API-vorm (`/v1/chat/completions`). **Cloud:** `LLM_BASE_URL=https://api.mistral.ai` + `LLM_API_KEY` (primaire LLM in `iou-ai` / `iou-ai-service` `/v1/chat`). **Lokaal (Ollama):** zet `SLM_MODEL` op een Mistral-model hieronder en `SLM_BASE_URL` op je Ollama-host.

## Chat / generatie (document pipeline & agents)

| Use case | Aanbevolen (licht) | `ollama pull` | Opmerking |
|----------|-------------------|---------------|-----------|
| **Standaard** (samenvatting, secties, eenvoudige structuur) | **qwen2.5:3b** | `ollama pull qwen2.5:3b` | Default in code; sterk instructievolgend |
| **Mistral, licht (3B)** | **ministral-3:3b** | `ollama pull ministral-3:3b` | Officieel Mistral; meertalig, function calling / JSON; recente Ollama-versie nodig |
| **Mistral, klassiek (7B)** | **mistral** of **mistral:7b** | `ollama pull mistral` | Zwaarder dan 3B; vertrouwde instructiestijl, EU-aanbieder |
| **Zeer beperkte hardware** | **gemma2:2b** | `ollama pull gemma2:2b` | Sneller/kleiner, iets minder nuance |
| **Alternatief 3B-klasse** | **llama3.2:3b** | `ollama pull llama3.2:3b` | Goed op Apple Silicon; EN-zwaarder |
| **Deep agents** (meerdere tool-stappen, langere redenering) | **qwen2.5:7b** of **llama3.1:8b** | `ollama pull qwen2.5:7b` | Nog “licht” voor een server; vaak betrouwbaarder bij tools |
| **Deep agents (Mistral-pad, zwaar)** | **mistral-small3.2** of **mistral-small:24b** | `ollama pull mistral-small3.2` | Niet “licht” (~tientallen GB VRAM/RAM afhankelijk van quant); sterk voor tools en lange context |

**Richtlijn:** start met **qwen2.5:3b** voor zowel documentverwerking als stub/agent-runs via `iou-ai-service` (`POST /v1/slm/chat`). Wil je **Mistral** houden (leverancier / compliance), gebruik **`ministral-3:3b`** als lichte SLM of **`mistral`** (7B) als je wat meer kwaliteit wilt. Als agent-runs vaak hallucineren of tools verkeerd kiezen, schaal naar **7B–8B** (of **mistral-small3.2** als je Mistral wilt en de hardware het trekt); houd de document-pipeline op 3B als dat voldoende is.

## Temperature (optioneel)

- **Struktuur / feitelijke extractie:** lagere temperature (bijv. `0.2`–`0.4`) — vandaag nog globaal via `LlmConfig` in code; later kan `SLM_TEMPERATURE` env worden toegevoegd.
- **Creatieve secties / concepttekst:** `0.6`–`0.8`.

## Embeddings (RAG / semantisch zoeken, los van chat)

Als je embeddings via Ollama wilt (bijv. naast `SemanticSearchService`), zijn dit gangbare **lichte** modellen:

| Model | Commando | Gebruik |
|-------|----------|---------|
| **nomic-embed-text** | `ollama pull nomic-embed-text` | Klein, snel, veel gebruikt |
| **mxbai-embed-large** | `ollama pull mxbai-embed-large` | Iets zwaarder, vaak beter voor retrieval |

Chat-SLM en embedding-model zijn **gescheiden** in Ollama; kies per taak.

## Apple MLX (Mac, geen Ollama)

Op **Apple Silicon** kun je dezelfde soort document-use-case lokaal draaien met **MLX** (Metal, unified memory), bijvoorbeeld met `mlx-lm` en modellen van **mlx-community**. Dat is een ander pad dan `SLM_BASE_URL` → Ollama; handig voor ontwikkelen op je Mac. Zie [**mlx-apple-silicon.md**](./mlx-apple-silicon.md) en **`examples/apple-mlx-document-digest/`**.

## Deep agents worker (Python)

Zet in de container of `.env` dezelfde logica als de gateway:

- `SLM_MODEL=qwen2.5:3b` (of de 7B-variant alleen voor de worker)
- `SLM_BASE_URL` wijzend naar Ollama of naar **`iou-ai-service`**, zodat alle LLM-calls dezelfde policy en logging delen.

Optioneel later: **`DEEP_AGENT_SLM_MODEL`** alleen in de Python-service als je bewust een zwaarder model voor agents wilt dan voor batch-documentjobs (vereist dan aparte route of env in je agent-code).

## Verificatie

```bash
ollama pull qwen2.5:3b
# of Mistral (licht): ollama pull ministral-3:3b
# of Mistral (7B):    ollama pull mistral
curl -s http://127.0.0.1:11434/api/tags | head
```

Daarna `iou-ai-service` met `SLM_BASE_URL=http://127.0.0.1:11434` en default model, of expliciet `SLM_MODEL=qwen2.5:3b` / `SLM_MODEL=ministral-3:3b` / `SLM_MODEL=mistral`.
