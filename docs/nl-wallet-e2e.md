# NL Wallet end-to-end met IOU-Modern

Deze flow koppelt de MinBZK **[verification_server](https://github.com/MinBZK/nl-wallet)** aan IOU (`POST /api/id/nl-wallet/sessions`) en optioneel de officiële **`wallet_web`**-knop.

## 1. MinBZK-backend (aanbevolen)

1. Clone [nl-wallet](https://github.com/MinBZK/nl-wallet) (bijvoorbeeld naast dit project).
2. **Eenmalig** in die repo: volg hun README voor `./scripts/setup-devenv.sh` (Rust, Flutter, certificaten, enz.).
3. Start alleen Postgres + verification server:

```bash
export NL_WALLET_ROOT=/pad/naar/nl-wallet   # optioneel; default is ../nl-wallet t.o.v. iou-modern root
chmod +x scripts/nl-wallet-verification-up.sh scripts/nl-wallet-verification-down.sh
./scripts/nl-wallet-verification-up.sh
```

Stoppen:

```bash
./scripts/nl-wallet-verification-down.sh
```

De standaard **public URL-poort** van de verification server in hun dev-stack staat in `scripts/configuration.sh` als **`VERIFICATION_SERVER_PORT=3011`**. Controleer in jouw `verification_server.toml` / logs als dat afwijkt.

## 2. IOU API

```bash
export NL_WALLET_VERIFICATION_SERVER_URL=http://127.0.0.1:3011
cargo run -p iou-api
```

- Zonder deze variabele antwoordt `POST /api/id/nl-wallet/sessions` met **503**.
- CORS staat in de API al ruim open voor lokale tests.

## 3. Frontend / browser

### A. Statische demo (zelfde oorsprong als de API)

Als `iou-api` draait, open:

`http://127.0.0.1:8000/nl-wallet-e2e.html`

(Statisch bestand uit `static/nl-wallet-e2e.html` — geen CORS-probleem voor `POST /api/...`.)

### B. Officiële `nl-wallet-button` (wallet_web)

1. In de **nl-wallet**-repo: `cd wallet_web && npm install && npm run build`.
2. Kopieer `wallet_web/dist/nl-wallet-web.iife.js` naar **`static/nl-wallet-web.iife.js`** in dit project (naast `nl-wallet-e2e.html`).
3. Herstart `iou-api` en herlaad `nl-wallet-e2e.html` — de pagina laadt het script en toont de echte custom element-knop.

`start-url` op de knop wijst naar **`/api/id/nl-wallet/sessions`** (relatief ten opzichte van de API-host).

### C. Dioxus-app (poort 8080)

Route **`/apps/nl-wallet`** start een sessie via `fetch` naar `http://localhost:8000/api/id/nl-wallet/sessions` (zelfde basis-URL als `src/api/mod.rs`). Zorg dat de API draait en dat `NL_WALLET_VERIFICATION_SERVER_URL` gezet is.

## 4. Optioneel: alleen Postgres (geavanceerd)

Als je **zelf** `verification_server` vanuit `wallet_core` wilt draaien met een eigen `verification_server.toml`:

```bash
docker compose -f docker-compose.nl-wallet-verification.yml up -d
```

Zet in `[storage].url` o.a.:

`postgres://postgres:postgres@127.0.0.1:5433/verification_server`

Daarna migraties en `cargo run` zoals in de nl-wallet-documentatie. Dit vervangt **niet** `setup-devenv.sh` (keys/usecases blijven nodig).

## 5. Usecase-naam

`POST /api/id/nl-wallet/sessions` stuurt `{ "usecase": "<jouw_keuze>" }` door naar de verification server. Die naam moet overeenkomen met een geconfigureerde usecase in **hun** server (zie hun `verification_server.toml` / demo-config).

Voor tests kun je beginnen met de usecase die hun **demo relying party** gebruikt, of een eigen entry die aan jouw nl-wallet-config is toegevoegd.
