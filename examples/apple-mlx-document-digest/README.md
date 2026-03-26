# Apple MLX — voorbeeld: documentdigest

Use-case die aansluit op **IOU-Modern** (overheid / documenten / Woo): een **lokale** korte analyse van een markdown- of tekstbestand, draaiend op **MLX + Metal** op je Mac.

## Waarom MLX hier?

- **Unified memory:** model en activaties delen RAM met de GPU — efficiënt op M1–M4.
- **Geen Ollama nodig** voor dit experiment: één `pip install` en `python digest.py …`.
- Zelfde *soort* output als je later via `SLM_*` / `iou-ai-service` zou kunnen doen — ander **runtime**-pad.

Architectuur en plaats in de stack: [`docs/architecture/mlx-apple-silicon.md`](../../docs/architecture/mlx-apple-silicon.md).

## Vereisten

- **macOS** op **Apple Silicon** (arm64), Python **3.10+**
- Eerste run: internet voor `pip` en voor download van modelweights (Hugging Face cache)

## Setup

```bash
cd examples/apple-mlx-document-digest
python3 -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\activate
pip install -r requirements.txt
```

## Run

```bash
python digest.py sample-document.md
```

Eigen bestand:

```bash
python digest.py /pad/naar/jouw-notitie.md --max-tokens 600
```

Ander MLX-model (zwaarder, vaak beter op NL):

```bash
python digest.py sample-document.md --model mlx-community/Mistral-7B-Instruct-v0.3-4bit
```

## Output

De analyse wordt naar **stdout** geschreven; gebruik `>` om op te slaan:

```bash
python digest.py sample-document.md > digest-output.md
```

## Integratie-ideeën (later)

- Output als tijdelijke markdown in een zaak-workflow (handmatig of via script).
- Parallel aan **Ollama**: zelfde prompts testen op Mac (MLX) vs. server (Ollama) voor kwaliteitsvergelijking.
