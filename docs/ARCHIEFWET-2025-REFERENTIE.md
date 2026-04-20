# Archiefwet 2025 - Referentie voor IOU-Modern

## Overzicht

Op **20 februari 2025** heeft de Tweede Kamer de nieuwe Archiefwet aangenomen. Deze wet brengt belangrijke wijzigingen aan voor de overheidsinformatiehuishouding, met directe impact op IOU-Modern.

## Belangrijkste Wijzigingen

| Aspect | Archiefwet 1995 | Archiefwet 2025 |
|--------|----------------|-----------------|
| **Overdracht termijn** | 20 jaar | **10 jaar** |
| **Focus** | Opslag | **Digitale toegankelijkheid** |
| **Reikwijd** | Beperkt | **Alle overheden** |
| **Koppeling Woo** | Onvoldoende | **Duidelijk onderscheid** |

## Kernprincipes

### 1. 10-Jaar Regel

Permanente overheidsinformatie wordt al na **10 jaar** (was 20 jaar) overgebracht naar archiefinstellingen:

- Nationaal Archief (Rijk)
- Regionale archiefdiensten (provincies)
- Gemeentelijke archiefdiensten (gemeenten)

**Voordelen**:
- Kwetsbare digitale informatie komt sneller bij professionals
- Minder risico op verouderde bestandsformaten
- Betere waarborging voor langetermijntoegankelijkheid

### 2. Digitale Toegankelijkheid

De wet benadrukt dat informatie niet alleen moet worden opgeslagen, maar ook **daadwerkelijk toegankelijk** moet blijven:

- Metadata vanaf het begin vastleggen
- Bestandsformaten voor langetermijngebruik
- Actief beheer van verouderde software
- Vernietiging op tijd als niet meer bewaard hoeft te worden

### 3. Cultureel Erfgoed

Overheidsinformatie wordt beschermd als onderdeel van het Nederlandse cultureel erfgoed:

- Parlementaire democratie vereist toegankelijke besluitvorming
- Overheid moet zich kunnen verantwoorden voor keuzes en uitvoering
- Historisch besef wordt versterkt door openbare archieven

### 4. Wet open overheid Koppeling

Duidelijk onderscheid tussen twee wetten:

| Wet | Reikwijdte |
|-----|-----------|
| **Woo** | Informatie bij overheidsorganisaties (actief) |
| **Archiefwet** | Informatie bij archiefdiensten (gearchiveerd) |

## Impact op IOU-Modern

### Nieuwe Requirements

**Archiefwet 2025** vereist:

1. **10-jaar overdracht**: Systeem moet overdracht naar archiefdiensten ondersteunen na 10 jaar
2. **Metadatavoorziening**: Compleet metagegevenspakket vanaf aanvang
3. **Formatbeheer**: Bestandsformaten die eeuwig leesbaar blijven
4. **Vernietigingsworkflows**: Geautomatiseerde vernietiging na bewaartermijn
5. **Toegankelijkheidsmetrieken**: Monitoren of informatie daadwerkelijk toegankelijk is

### Wijzigingen in IOU-Modern

#### 1. Architectuurprincipes (ARC-000-PRIN)

**P3: Archival Integrity** bijgewerkt met:
- Referentie naar Archiefwet 2025
- 10-jaar overdrachtsregel
- Focus op digitale toegankelijkheid
- Koppeling met Metamodel GGHH V2

#### 2. Requirements (ARC-001-REQ-v1.1)

**NFR-COMP-003** bijgewerkt:
- Van "Archiefwet 1995" naar "Archiefwet 2025 (10-year transfer rule)"

**Glossary** uitgebreid met:
- Referentie naar nieuwe wet en 10-jaar regel

#### 3. Metadata Registry (ARC-002-REQ-v1.1)

**Nieuwe requirements** voor:
- Time-based validity met 10-jaar overdracht
- CDD+ integratie voor archiefoverdracht
- Metadatavolledigheid voor archiefwaardigheid

## Implementatie Roadmap

### Fase 1: Metadata (Kwartaal 1-2, 2025)
- [ ] Metamodel GGHH V2 entiteiten uitbreiden met archiefmetadata
- [ ] 10-jaar overdracht workflow ontwerpen
- [ ] CDD+ integratie architectuur vastleggen

### Fase 2: Services (Kwartaal 3-4, 2025)
- [ ] Context-aware services implementeren
- [ ] Automatische metadata-extractie uitbouwen
- [ ] Vernietigingsworkflows implementeren

### Fase 3: Integratie (2026)
- [ ] Archiefdienst API's integreren
- [ ] Format validatie en migratie tools
- [ ] Toegankelijkheidsmonitoring

## Referenties

- [Officieel nieuwsbericht](https://www.open-overheid.nl/actueel/nieuws/2025/02/20/nieuwe-archiefwet-aangenomen-door-tweede-kamer)
- [Archiefwet 2025 wetsvoorstel](https://www.rijksoverheid.nl/documenten/publicaties/2024/02/02/wetsvoorstel-wijziging-van-de-archiefwet-1995)
- [Metamodel GGHH Overheid](https://regels.overheid.nl/standaarden/gghh)
- [IOU-Modern Architecture Principles](../projects/000-global/ARC-000-PRIN-v1.0.md)

## Versiebeheer

| Versie | Datum | Wijziging |
|--------|-------|-----------|
| 1.0 | 2025-02-20 | Initiële versie naar aanleiding van aanneming wet |
