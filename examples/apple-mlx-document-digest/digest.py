#!/usr/bin/env python3
"""
IOU-Modern voorbeeld: documentdigest met MLX-LM op Apple Silicon (Metal).

Use-case: korte, gestructureerde analyse van een .md/.txt in de geest van
overheid / Woo — lokaal, zonder Ollama of iou-ai-service.

Zie README.md en docs/architecture/mlx-apple-silicon.md
"""

from __future__ import annotations

import argparse
import platform
import sys
from pathlib import Path

# Ruim genoeg voor typische secties; voorkomt enorme context bij per ongeluk grote files.
MAX_INPUT_CHARS = 24_000

SYSTEM_NL = (
    "Je bent een assistent voor Nederlandse overheids- en Woo-gerelateerde documenten. "
    "Geef een beknopte analyse in het Nederlands met deze structuur:\n"
    "1) Kern — maximaal drie zinnen.\n"
    "2) Bulletpunten — feiten, besluiten, data of zaaknummers die expliciet in de tekst staan.\n"
    "3) Indicatie informatiecategorie — alleen als je dat voorzichtig uit de tekst kunt afleiden "
    "(bijv. openbaarheid, persoonsgegevens, beleid).\n"
    "Wees strikt feitelijk: verzin geen data, wetartikelen of zaaknummers die niet in de brontekst voorkomen."
)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Documentdigest via MLX (Apple Silicon). Geen netwerk na model-download."
    )
    parser.add_argument(
        "input",
        type=Path,
        help="Pad naar .md of .txt",
    )
    parser.add_argument(
        "--model",
        default="mlx-community/Llama-3.2-3B-Instruct-4bit",
        help="mlx-community model op Hugging Face (4-bit quant aanbevolen)",
    )
    parser.add_argument(
        "--max-tokens",
        type=int,
        default=512,
        help="Maximum aantal gegenereerde tokens",
    )
    args = parser.parse_args()

    if platform.system() != "Darwin":
        print(
            "Waarschuwing: MLX is primair voor macOS. Op Linux/Windows gebruik je Ollama of cloud-LLM.",
            file=sys.stderr,
        )
    elif platform.machine() != "arm64":
        print(
            "Waarschuwing: MLX profiteert het meest van Apple Silicon (arm64).",
            file=sys.stderr,
        )

    if not args.input.is_file():
        print(f"Bestand niet gevonden: {args.input}", file=sys.stderr)
        return 1

    try:
        from mlx_lm import generate, load
    except ImportError:
        print(
            "mlx_lm niet geïnstalleerd. Op Apple Silicon:\n"
            "  python3 -m venv .venv && source .venv/bin/activate\n"
            "  pip install -r requirements.txt",
            file=sys.stderr,
        )
        return 1

    raw = args.input.read_text(encoding="utf-8", errors="replace")
    if len(raw) > MAX_INPUT_CHARS:
        print(
            f"Waarschuwing: invoer ingekort tot {MAX_INPUT_CHARS} tekens.",
            file=sys.stderr,
        )
        raw = raw[:MAX_INPUT_CHARS]

    user_msg = f"Analyseer het volgende document:\n\n---\n{raw}\n---"
    messages = [
        {"role": "system", "content": SYSTEM_NL},
        {"role": "user", "content": user_msg},
    ]

    print(f"Model laden: {args.model} (eerste run downloadt weights)…", file=sys.stderr)
    model, tokenizer = load(args.model)

    if getattr(tokenizer, "chat_template", None) is None:
        print(
            "Tokenizer heeft geen chat_template; kies een instructiemodel uit mlx-community.",
            file=sys.stderr,
        )
        return 1

    prompt = tokenizer.apply_chat_template(
        messages,
        add_generation_prompt=True,
        tokenize=False,
    )

    text = generate(
        model,
        tokenizer,
        prompt=prompt,
        max_tokens=args.max_tokens,
        verbose=False,
    )
    print(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
