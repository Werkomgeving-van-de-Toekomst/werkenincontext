# {{ document_type }}

**Referentie:** {{ reference_number }}
**Datum:** {{ date|dutch_date }}
**Gemeente:** {{ municipality }}

## 1. Aanvraag

Op {{ request_date|dutch_date }} heeft {{ requester }} een verzoek ingediend op grond van de Wet open overheid.

### 1.1 Onderwerp van het verzoek

{{ request_subject }}

{% if additional_details %}
### 1.2 Aanvullende details

{{ additional_details }}
{% endif %}

## 2. Beoordeling

### 2.1 Reikwijdte van het verzoek

Het verzoek betreft de volgende informatie:

{{ request_scope }}

### 2.2 Openbaarmaking

Na afweging van alle belangen wordt besloten tot:

{% if approval_granted %}
**Openbaarmaking** van de gevraagde informatie.

{% else %}
**Gedeeltelijke weigering** op grond van {{ refusal_ground }}.
{% endif %}

## 3. Besluit

Inhoudende:

{% if approval_granted %}
1. Het verzoek toe te kennen
2. De gevraagde informatie openbaar te maken

{% else %}
1. Het verzoek gedeeltelijk af te wijzen
2. De volgende informatie openbaar te maken: {{ disclosed_info }}

{% endif %}

**Auteur:** {{ author_name }}
**Handtekening:** ___________________

---
*Dit besluit is automatisch gegenereerd door IOU-Modern*
