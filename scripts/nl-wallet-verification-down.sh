#!/usr/bin/env bash
# Stop postgres + verification_server van de MinBZK dev-omgeving.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NL_WALLET_ROOT="${NL_WALLET_ROOT:-$REPO_ROOT/../nl-wallet}"

if [[ ! -f "$NL_WALLET_ROOT/scripts/start-devenv.sh" ]]; then
  echo "NL_WALLET_ROOT ongeldig: $NL_WALLET_ROOT"
  exit 1
fi

cd "$NL_WALLET_ROOT"
exec ./scripts/start-devenv.sh --stop postgres vs
