#!/usr/bin/env bash
# Start de MinBZK NL Wallet verification_server + PostgreSQL via hun officiële tooling.
#
# Vereist:
#   - Een clone van https://github.com/MinBZK/nl-wallet
#   - Eenmalig in die clone: ./scripts/setup-devenv.sh (zie upstream README)
#
# Gebruik:
#   export NL_WALLET_ROOT=/pad/naar/nl-wallet   # default: ../nl-wallet t.o.v. repo-root
#   ./scripts/nl-wallet-verification-up.sh
#
# IOU API (na start):
#   export NL_WALLET_VERIFICATION_SERVER_URL=http://127.0.0.1:3011
#   cargo run -p iou-api
#
# Zie docs/nl-wallet-e2e.md voor volledige end-to-end flow.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NL_WALLET_ROOT="${NL_WALLET_ROOT:-$REPO_ROOT/../nl-wallet}"

if [[ ! -f "$NL_WALLET_ROOT/scripts/start-devenv.sh" ]]; then
  echo "MinBZK nl-wallet niet gevonden op: $NL_WALLET_ROOT"
  echo ""
  echo "  git clone https://github.com/MinBZK/nl-wallet.git \"$REPO_ROOT/../nl-wallet\""
  echo "  cd \"$REPO_ROOT/../nl-wallet\" && ./scripts/setup-devenv.sh"
  echo ""
  echo "Daarna opnieuw (optioneel):"
  echo "  export NL_WALLET_ROOT=$REPO_ROOT/../nl-wallet"
  echo "  $0"
  exit 1
fi

echo "==> NL_WALLET_ROOT=$NL_WALLET_ROOT"
echo "==> Start postgres + verification_server (MinBZK start-devenv.sh postgres vs)..."
cd "$NL_WALLET_ROOT"
exec ./scripts/start-devenv.sh postgres vs
