# Quick Start: Document Creation System

## 1. Start de API Server

```bash
# Terminal 1: Backend
cd /Users/marc/Projecten/iou-modern
cargo run --bin iou-api
```

De API draait nu op `http://localhost:8000`

## 2. Seed de Database

Open een nieuwe terminal:

```bash
# Terminal 2: Seed data
cd /Users/marc/Projecten/iou-modern
./scripts/seed-database.sh
```

Dit voegt:
- 2 templates (WOO Besluit, WOO Info)
- 1 test document
- Domain configuraties
- Audit trail entries

## 3. Test de API

```bash
# Lijst met templates
curl http://localhost:8000/api/templates

# Maak een nieuw document
curl -X POST http://localhost:8000/api/documents/create \
  -H "Content-Type: application/json" \
  -d '{
    "domain_id": "woo_minfin",
    "document_type": "woo_besluit",
    "context": {
      "reference_number": "REF-2024-001",
      "date": "2024-03-02",
      "municipality": "Amsterdam",
      "requester": "Jan Jansen",
      "request_subject": "Besluit openbaarmaking WOO verzoek"
    }
  }'

# Check document status
curl http://localhost:8000/api/documents/{document_id}/status

# Haal audit trail op
curl http://localhost:8000/api/documents/{document_id}/audit
```

## 4. Start de Frontend (Optioneel)

```bash
# Terminal 3: Frontend
cargo run --bin iou-frontend
```

Ga naar:
- `http://localhost:8000/documenten/maken` - Document maken
- `http://localhost:8000/documenten/wachtrij` - Goedkeuring wachtrij
- `http://localhost:8000/templates` - Templates beheren

## API Endpoints

| Endpoint | Methode | Beschrijving |
|----------|---------|--------------|
| `/api/documents/create` | POST | Maak nieuw document |
| `/api/documents/:id/status` | GET | Haal document status op |
| `/api/documents/:id/approve` | POST | Keur document goed/af |
| `/api/documents/:id/audit` | GET | Haal audit trail op |
| `/api/documents/:id/download` | GET | Download document |
| `/api/templates` | GET | Lijst alle templates |
| `/api/templates` | POST | Maak nieuwe template |
| `/api/templates/:id` | GET | Haal template op |
| `/api/templates/:id` | PUT | Update template |
| `/api/templates/:id` | DELETE | Verwijder template |

## Troubleshooting

**Database niet gevonden?**
- De API maakt de database automatisch bij de eerste start
- Controleer of de `data/` map bestaat

**CORS fouten?**
- De API heeft CORS aanstaan voor alle origins (development mode)

**Frontend laadt niet?**
- Zorg dat de API draait op poort 8000
- Controleer de browser console voor foutmeldingen
