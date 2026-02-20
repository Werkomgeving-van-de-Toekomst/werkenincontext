//! PROVISA: Provinciale Selectielijst Archieven
//!
//! Dit module bevat de selectielijsten voor alle provincies conform de Archiefwet.
//! De lijsten zijn gebaseerd op de Provinciale Enterprise Referentie Architectuur (PETRA).
//!
//! # Selectielijsten
//!
//! - **Provisa 2020+**: Procesgerichte lijst voor provinciale organen (sinds 1-1-2020)
//! - **Provisa CdK 2020+**: Specifieke lijst voor Commissaris van de Koning als rijksorgaan
//! - **Provisa 2014-2019**: Procesgerichte lijst (geldig t/m 2019)
//! - **Provisa 2005**: Documentgerichte lijst (vervallen)
//!
//! # Bewaartermijnen
//!
//! Elke procescategorie heeft een bewaartermijn:
//! - **Permanent**: Blijvend te bewaren (overbrenging naar Nationaal Archief)
//! - **Tijdelijk (X jaar)**: Vernietigen na X jaar
//!
//! # Hotspots
//!
//! Bij maatschappelijk relevante gebeurtenissen kan de waardering wijzigen
//! van tijdelijk naar permanent.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::{Display, EnumString, IntoStaticStr};

/// Versie van de Provisa selectielijst
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, IntoStaticStr)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ProvisaVersion {
    /// Provisa 2020+ - Huidige lijst (vanaf 1-1-2020)
    #[strum(serialize = "2020")]
    V2020,
    /// Provisa 2014-2019 (vervallen)
    #[strum(serialize = "2014")]
    V2014,
    /// Provisa 2005 (vervallen)
    #[strum(serialize = "2005")]
    V2005,
}

impl ProvisaVersion {
    /// Geef het eindjaar van deze versie
    pub fn end_year(&self) -> Option<u32> {
        match self {
            Self::V2020 => None, // Nog geldig
            Self::V2014 => Some(2019),
            Self::V2005 => Some(2013),
        }
    }

    /// Geef het beginjaar van deze versie
    pub fn start_year(&self) -> u32 {
        match self {
            Self::V2020 => 2020,
            Self::V2014 => 2014,
            Self::V2005 => 2005,
        }
    }

    /// Bepaal welke versie geldt voor een document uit een bepaald jaar
    pub fn for_document_year(year: u32) -> Self {
        if year >= 2020 {
            Self::V2020
        } else if year >= 2014 {
            Self::V2014
        } else {
            Self::V2005
        }
    }
}

/// Provincie-orgaan type voor selectielijst determinatie
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
pub enum ProvincieOrgaan {
    /// Provinciale Staten, Gedeputeerde Staten, CdK als provinciaal orgaan
    ProvincialeOrganen,
    /// Commissaris van de Koning als rijksorgaan (ambtsinstructie)
    CommissarisVanDeKoning,
}

/// PETRA procescategorie (Provinciale Enterprise Referentie Architectuur)
///
/// De hoogste indelingslaag in de PETRA processenbibliotheek.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "SCREAMING-KEBAB-CASE")]
pub enum PetraCategorie {
    // === Bestuurlijk en Ondersteunend ===
    /// Bestuurlijke besluitvorming en advisering
    Bestuur,
    /// Strategische ontwikkeling en beleid
    Strategie,
    /// Concernsturing en control
    Concernsturing,
    /// Organisatieontwikkeling en HRM
    HumanResourceManagement,
    /// Financieel beheer
    Financien,
    /// Informatievoorziening en ICT
    Informatievoorziening,
    /// Facilitaire bedrijfsvoering
    Faciliteiten,
    /// Communicatie
    Communicatie,

    // === Ruimtelijke Ontwikkeling ===
    /// Ruimtelijke planning en advisering
    RuimtelijkePlanning,
    /// Omgevingswet vergunningverlening
    Omgevingsvergunningen,
    /// Grondzaken en vastgoed
    Grondzaken,

    // === Milieu en Duurzaamheid ===
    /// Milieubeleid en vergunningverlening
    Milieu,
    /// Energie en klimaat
    EnergieKlimaat,
    /// Natuur en landschap
    NatuurLandschap,
    /// Water en riolering
    Water,

    // === Verkeer en Vervoer ===
    /// Regionale verkeer en vervoer
    VerkeerVervoer,
    /// Openbaar vervoer
    OpenbaarVervoer,

    // === Economie en Samenleving ===
    /// Economische ontwikkeling
    Economie,
    /// Toerisme en recreatie
    ToerismeRecreatie,
    /// Cultuur en sport
    CultuurSport,
    /// Wonen en leefomgeving
    Wonen,
    /// Volksgezondheid
    Gezondheid,

    // === Landelijk Gebied ===
    /// Landelijk gebied en landbouw
    LandelijkGebied,
    /// Recreatie en landbouw
    Landbouw,

    // === Veiligheid ===
    /// Openbare orde en veiligheid
    Veiligheid,
    /// Brandweer en crisisbeheersing
    BrandweerCrisis,

    // === Internationaal ===
    /// Internationale samenwerking
    Internationaal,
}

impl PetraCategorie {
    /// Alle categoriën
    pub fn all() -> &'static [PetraCategorie] {
        use PetraCategorie::*;
        &[
            Bestuur,
            Strategie,
            Concernsturing,
            HumanResourceManagement,
            Financien,
            Informatievoorziening,
            Faciliteiten,
            Communicatie,
            RuimtelijkePlanning,
            Omgevingsvergunningen,
            Grondzaken,
            Milieu,
            EnergieKlimaat,
            NatuurLandschap,
            Water,
            VerkeerVervoer,
            OpenbaarVervoer,
            Economie,
            ToerismeRecreatie,
            CultuurSport,
            Wonen,
            Gezondheid,
            LandelijkGebied,
            Landbouw,
            Veiligheid,
            BrandweerCrisis,
            Internationaal,
        ]
    }

    /// Mensleesbare beschrijving
    pub fn description(&self) -> &'static str {
        match self {
            Self::Bestuur => "Bestuurlijke besluitvorming PS/GS en advisering",
            Self::Strategie => "Strategische ontwikkeling van provinciaal beleid",
            Self::Concernsturing => "Concernsturing, control en verantwoording",
            Self::HumanResourceManagement => "HRM, organisatieontwikkeling en personeelsbeheer",
            Self::Financien => "Financieel beheer, begroting en verantwoording",
            Self::Informatievoorziening => "Informatievoorziening, informatiebeveiliging en ICT",
            Self::Faciliteiten => "Huisvesting, inkoop en facilitaire diensten",
            Self::Communicatie => "Communicatie, voorlichting en mediabeheer",
            Self::RuimtelijkePlanning => "Provinciale ruimtelijke planning en structuurvisies",
            Self::Omgevingsvergunningen => "Vergunningverlening Omgevingswet",
            Self::Grondzaken => "Grondzaken, onteigening en vastgoedbeheer",
            Self::Milieu => "Milieubeleid en vergunningverlening",
            Self::EnergieKlimaat => "Energietransitie en klimaatbeleid",
            Self::NatuurLandschap => "Natuurbeheer en landschap",
            Self::Water => "Waterbeheer en riolering",
            Self::VerkeerVervoer => "Regionale verkeers- en vervoersplannen",
            Self::OpenbaarVervoer => "Openbaar vervoer en concessieverlening",
            Self::Economie => "Economische ontwikkeling en bedrijfsgericht beleid",
            Self::ToerismeRecreatie => "Toerisme- en recreatiebeleid",
            Self::CultuurSport => "Cultuur, sport en participatiebeleid",
            Self::Wonen => "Woonbeleid en volkshuisvesting",
            Self::Gezondheid => "Volksgezondheid en jeugdbeleid",
            Self::LandelijkGebied => "Landelijk gebied en agrarisch beleid",
            Self::Landbouw => "Landbouwstructuur en plattelandsontwikkeling",
            Self::Veiligheid => "Openbare orde en veiligheid",
            Self::BrandweerCrisis => "Brandweerzorg en crisisbeheersing",
            Self::Internationaal => "Internationale samenwerking en Euregios",
        }
    }
}

/// Type besluit binnen een PETRA categorie
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "SCREAMING-KEBAB-CASE")]
pub enum BesluitType {
    /// Provinciaal verordening
    Verordening,
    /// Beleidsregel
    Beleidsregel,
    /// Besluit van Gedeputeerde Staten
    Besluit,
    /// Aanstelling/benoeming
    Aanstelling,
    /// Vergunning
    Vergunning,
    /// Subsidiebeschikking
    Subsidie,
    /// Contract/Overeenkomst
    Contract,
    /// Behandeling van aanvraag/verzoek
    Aanvraag,
    /// Advies
    Advies,
    /// Rapport/Notitie
    Rapport,
    /// Brief/Uitgaand stuk
    Brief,
    /// Email
    Email,
    /// Notitie van ambtelijke overleg
    Notitie,
    ///agenda/Notulen
    AgendaNotulen,
}

/// Bewaartermijn specificatie
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bewaartermijn {
    /// Aantal jaar bewaren (None = permanent)
    pub jaren: Option<u32>,
    /// Archiefwaarde
    pub waarde: Archiefwaarde,
    /// Selectielijst referentie
    pub selectielijst_ref: String,
}

impl Bewaartermijn {
    /// Maak een tijdelijke bewaartermijn
    pub fn tijdelijk(jaren: u32, ref_code: impl Into<String>) -> Self {
        Self {
            jaren: Some(jaren),
            waarde: Archiefwaarde::Tijdelijk,
            selectielijst_ref: ref_code.into(),
        }
    }

    /// Maak een permanente bewaartermijn
    pub fn permanent(ref_code: impl Into<String>) -> Self {
        Self {
            jaren: None,
            waarde: Archiefwaarde::Permanent,
            selectielijst_ref: ref_code.into(),
        }
    }

    /// Bereken de vernietigingsdatum op basis van creatiedatum
    pub fn vernietigingsdatum(&self, creatie_datum: NaiveDate) -> Option<NaiveDate> {
        self.jaren.map(|j| creatie_datum + chrono::Months::new((j * 12) as u32))
    }

    /// Is dit document al vernietigbaar per de gegeven datum?
    pub fn is_vernietigbaar_per(&self, creatie_datum: NaiveDate, per: NaiveDate) -> bool {
        match self.vernietigingsdatum(creatie_datum) {
            Some(datum) => per >= datum,
            None => false, // Permanent bewaren → nooit vernietigbaar
        }
    }
}

/// Archiefwaarde
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum Archiefwaarde {
    /// Permanent te bewaren (overbrenging naar Nationaal Archief na 20 jaar)
    Permanent,
    /// Tijdelijk te bewaren (vernietigen na termijn)
    Tijdelijk,
}

/// PROVISA bewaartermijn voor een specifiek combinatie van categorie en besluittype
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisaBepaling {
    /// PETRA categorie
    pub categorie: PetraCategorie,
    /// Type besluit/document
    pub besluit_type: BesluitType,
    /// Orgaan type
    pub orgaan: ProvincieOrgaan,
    /// Bewaartermijn
    pub bewaartermijn: Bewaartermijn,
    /// Provisa versie
    pub versie: ProvisaVersion,
    /// Toelichting
    pub toelichting: Option<String>,
}

impl ProvisaBepaling {
    /// Nieuwe bepaling
    pub fn new(
        categorie: PetraCategorie,
        besluit_type: BesluitType,
        orgaan: ProvincieOrgaan,
        bewaartermijn: Bewaartermijn,
    ) -> Self {
        Self {
            categorie,
            besluit_type,
            orgaan,
            bewaartermijn,
            versie: ProvisaVersion::V2020,
            toelichting: None,
        }
    }

    /// Met specifieke Provisa versie
    pub fn met_versie(mut self, versie: ProvisaVersion) -> Self {
        self.versie = versie;
        self
    }

    /// Met toelichting
    pub fn met_toelichting(mut self, toelichting: impl Into<String>) -> Self {
        self.toelichting = Some(toelichting.into());
        self
    }

    /// Unieke sleutel voor deze bepaling
    pub fn key(&self) -> (ProvincieOrgaan, PetraCategorie, BesluitType) {
        (self.orgaan, self.categorie, self.besluit_type)
    }
}

/// PROVISA selectielijst
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisaSelectielijst {
    /// Versie van de selectielijst
    pub versie: ProvisaVersion,
    /// Orgaan type
    pub orgaan: ProvincieOrgaan,
    /// Bepalingen per categorie en besluittype
    pub bepalingen: HashMap<(PetraCategorie, BesluitType), Bewaartermijn>,
    /// Naam van de selectielijst
    pub naam: String,
    /// URL naar officiële publicatie
    pub url: Option<String>,
}

impl ProvisaSelectielijst {
    /// Maak een nieuwe selectielijst
    pub fn new(versie: ProvisaVersion, orgaan: ProvincieOrgaan) -> Self {
        let naam = match (versie, orgaan) {
            (ProvisaVersion::V2020, ProvincieOrgaan::ProvincialeOrganen) => {
                "Selectielijst voor documenten van de provinciale organen (2020)".to_string()
            }
            (ProvisaVersion::V2020, ProvincieOrgaan::CommissarisVanDeKoning) => {
                "Selectielijst voor documenten van de CdK als rijksorgaan (2020)".to_string()
            }
            (v, o) => format!("Selectielijst {v:?} - {o:?}"),
        };

        Self {
            versie,
            orgaan,
            bepalingen: HashMap::new(),
            naam,
            url: None,
        }
    }

    /// Voeg een bepaling toe
    pub fn voeg_bepaling_toe(&mut self, categorie: PetraCategorie, besluit_type: BesluitType, termijn: Bewaartermijn) {
        self.bepalingen.insert((categorie, besluit_type), termijn);
    }

    /// Zoek bewaartermijn voor een combinatie
    pub fn zoek_bewaartermijn(&self, categorie: &PetraCategorie, besluit_type: &BesluitType) -> Option<&Bewaartermijn> {
        self.bepalingen.get(&(*categorie, *besluit_type))
    }

    /// Standaard PROVISA 2020 voor provinciale organen
    pub fn provinciaal_2020() -> Self {
        let mut lijst = Self::new(ProvisaVersion::V2020, ProvincieOrgaan::ProvincialeOrganen);

        // Bestuurlijke besluitvorming - meestal permanent
        use BesluitType::*;
        use PetraCategorie::*;

        // Provinciale verordeningen en beleidsregels zijn altijd permanent
        for &cat in PetraCategorie::all() {
            lijst.voeg_bepaling_toe(cat, Verordening, Bewaartermijn::permanent("provisa-2020-v"));
            lijst.voeg_bepaling_toe(cat, Beleidsregel, Bewaartermijn::permanent("provisa-2020-br"));
        }

        // Besluiten - afhankelijk van impact
        let permanente_besluittypen = [
            (Bestuur, Besluit, "provisa-2020-b1"),
            (Strategie, Besluit, "provisa-2020-b2"),
            (RuimtelijkePlanning, Besluit, "provisa-2020-b3"),
            (Milieu, Vergunning, "provisa-2020-m1"),
        ];
        for (cat, btype, ref_code) in permanente_besluittypen {
            lijst.voeg_bepaling_toe(cat, btype, Bewaartermijn::permanent(ref_code));
        }

        // Tijdelijke bewaartermijnen voor uitvoerende stukken
        lijst.voeg_bepaling_toe(HumanResourceManagement, Aanstelling, Bewaartermijn::tijdelijk(90, "provisa-2020-hr1"));
        lijst.voeg_bepaling_toe(Financien, Subsidie, Bewaartermijn::tijdelijk(20, "provisa-2020-fin1"));
        lijst.voeg_bepaling_toe(Financien, Contract, Bewaartermijn::tijdelijk(10, "provisa-2020-fin2"));
        lijst.voeg_bepaling_toe(Informatievoorziening, Email, Bewaartermijn::tijdelijk(5, "provisa-2020-ict1"));
        lijst.voeg_bepaling_toe(Communicatie, Brief, Bewaartermijn::tijdelijk(10, "provisa-2020-com1"));
        lijst.voeg_bepaling_toe(Veiligheid, Aanvraag, Bewaartermijn::tijdelijk(15, "provisa-2020-v1"));

        // Agenda en notulen - permanent voor PS/GS
        for cat in [Bestuur, Strategie, Concernsturing] {
            lijst.voeg_bepaling_toe(cat, AgendaNotulen, Bewaartermijn::permanent("provisa-2020-an"));
        }

        lijst
    }

    /// Standaard PROVISA 2020 voor CdK als rijksorgaan
    pub fn cdk_2020() -> Self {
        let mut lijst = Self::new(ProvisaVersion::V2020, ProvincieOrgaan::CommissarisVanDeKoning);

        use BesluitType::*;
        use PetraCategorie::*;

        // CdK taken zijn voornamelijk tijdelijk, behalve burgemeestersbenoemingen
        lijst.voeg_bepaling_toe(Bestuur, Aanstelling, Bewaartermijn::permanent("provisa-cdk-2020-b1"));
        lijst.voeg_bepaling_toe(Bestuur, Advies, Bewaartermijn::tijdelijk(20, "provisa-cdk-2020-a1"));
        lijst.voeg_bepaling_toe(Veiligheid, Besluit, Bewaartermijn::tijdelijk(15, "provisa-cdk-2020-v1"));

        lijst
    }
}

/// Hotspot - maatschappelijk relevante gebeurtenis die archiefwaarde beïnvloedt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotspot {
    /// Unieke identificatie
    pub id: String,
    /// Naam van de hotspot
    pub naam: String,
    /// Beschrijving
    pub beschrijving: String,
    /// Startdatum
    pub start_datum: NaiveDate,
    /// Einddatum (indien van toepassing)
    pub eind_datum: Option<NaiveDate>,
    /// Gerelateerde PETRA categorieën
    pub categorieen: Vec<PetraCategorie>,
    /// Publicatiedatum in Staatscourant
    pub publicatie_datum: Option<NaiveDate>,
    /// URL naar officiële bekendmaking
    pub url: Option<String>,
}

impl Hotspot {
    /// Nieuwe hotspot
    pub fn new(
        id: impl Into<String>,
        naam: impl Into<String>,
        beschrijving: impl Into<String>,
        start_datum: NaiveDate,
    ) -> Self {
        Self {
            id: id.into(),
            naam: naam.into(),
            beschrijving: beschrijving.into(),
            start_datum,
            eind_datum: None,
            categorieen: Vec::new(),
            publicatie_datum: None,
            url: None,
        }
    }

    /// Categorieën toevoegen
    pub fn met_categorieen(mut self, cats: Vec<PetraCategorie>) -> Self {
        self.categorieen = cats;
        self
    }

    /// Einddatum
    pub fn met_einddatum(mut self, datum: NaiveDate) -> Self {
        self.eind_datum = Some(datum);
        self
    }

    /// Publicatie
    pub fn met_publicatie(mut self, datum: NaiveDate, url: Option<String>) -> Self {
        self.publicatie_datum = Some(datum);
        self.url = url;
        self
    }

    /// Is deze hotspot actief op een bepaalde datum?
    pub fn is_actief_op(&self, datum: NaiveDate) -> bool {
        if datum < self.start_datum {
            return false;
        }
        match self.eind_datum {
            Some(eind) => datum <= eind,
            None => true,
        }
    }

    /// Beïnvloedt deze hotspot een bepaalde categorie?
    pub fn betreft_categorie(&self, categorie: &PetraCategorie) -> bool {
        self.categorieen.contains(categorie)
    }
}

/// Geregistreerde hotspots voor een provincie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotRegister {
    /// Provincie
    pub provincie: String,
    /// Geregistreerde hotspots
    pub hotspots: Vec<Hotspot>,
}

impl HotspotRegister {
    /// Nieuw register
    pub fn new(provincie: impl Into<String>) -> Self {
        Self {
            provincie: provincie.into(),
            hotspots: Vec::new(),
        }
    }

    /// Hotspot toevoegen
    pub fn voeg_toe(&mut self, hotspot: Hotspot) {
        self.hotspots.push(hotspot);
    }

    /// Alle actieve hotspots op een datum
    pub fn actief_op(&self, datum: NaiveDate) -> Vec<&Hotspot> {
        self.hotspots
            .iter()
            .filter(|h| h.is_actief_op(datum))
            .collect()
    }

    /// Hotspots die een bepaalde categorie beïnvloeden
    pub fn voor_categorie(&self, categorie: &PetraCategorie) -> Vec<&Hotspot> {
        self.hotspots
            .iter()
            .filter(|h| h.betreft_categorie(categorie))
            .collect()
    }

    /// Bepaal of een document onder een hotspot valt (upgrade naar permanent)
    pub fn upgrade_naar_permanent(
        &self,
        categorie: &PetraCategorie,
        document_datum: NaiveDate,
    ) -> Option<&Hotspot> {
        self.hotspots
            .iter()
            .find(|h| h.is_actief_op(document_datum) && h.betreft_categorie(categorie))
    }
}

/// PROVISA compliance beoordeling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisaBeoordeling {
    /// Bewaartermijn volgens selectielijst
    pub bewaartermijn: Option<Bewaartermijn>,
    /// Of het document onder een hotspot valt
    pub hotspot: Option<Hotspot>,
    /// Uiteindelijke archiefwaarde (rekening houdend met hotspot)
    pub uiteindelijke_waarde: Archiefwaarde,
    /// Vernietigingsdatum (indien tijdelijk)
    pub vernietigingsdatum: Option<NaiveDate>,
    /// Overbrengingsdatum naar archief (indien permanent, meestal 20 jaar)
    pub overbrengingsdatum: Option<NaiveDate>,
    /// Provisa versie die van toepassing is
    pub versie: ProvisaVersion,
    /// Toelichting bij de beoordeling
    pub toelichting: Vec<String>,
}

impl ProvisaBeoordeling {
    /// Beoordeel een document op basis van PROVISA
    pub fn beoordeel(
        selectielijst: &ProvisaSelectielijst,
        categorie: &PetraCategorie,
        besluit_type: &BesluitType,
        creatie_datum: NaiveDate,
        hotspot_register: Option<&HotspotRegister>,
    ) -> Self {
        let bewaartermijn = selectielijst.zoek_bewaartermijn(categorie, besluit_type);
        let versie = selectielijst.versie;
        let mut toelichting = Vec::new();

        // Bepaal basis archiefwaarde
        let basis_waarde = bewaartermijn
            .as_ref()
            .map(|b| b.waarde)
            .unwrap_or(Archiefwaarde::Tijdelijk);

        // Check hotspots
        let hotspot = match hotspot_register {
            Some(register) => register.upgrade_naar_permanent(categorie, creatie_datum),
            None => None,
        };

        // Hotspot upgrade naar permanent
        let uiteindelijke_waarde = if hotspot.is_some() {
            toelichting.push(format!(
                "Document valt onder hotspot '{}': upgrade naar permanente bewaring",
                hotspot.as_ref().unwrap().naam
            ));
            Archiefwaarde::Permanent
        } else {
            basis_waarde
        };

        // Bereken datums
        let vernietigingsdatum = if uiteindelijke_waarde == Archiefwaarde::Tijdelijk {
            bewaartermijn
                .as_ref()
                .and_then(|b| b.vernietigingsdatum(creatie_datum))
        } else {
            None
        };

        let overbrengingsdatum = if uiteindelijke_waarde == Archiefwaarde::Permanent {
            // Naar nationaal archief na 20 jaar (of direct voor hotspot documenten)
            Some(creatie_datum + chrono::Months::new(240))
        } else {
            None
        };

        if let Some(ref bt) = bewaartermijn {
            toelichting.push(format!(
                "Bewaartermijn volgens {}: {}",
                selectielijst.naam,
                match bt.jaren {
                    Some(j) => format!("{} jaar (tijdelijk)", j),
                    None => "permanent".to_string(),
                }
            ));
        } else {
            toelichting.push(format!(
                "Geen specifieke bewaartermijn gevonden voor {} + {}",
                categorie, besluit_type
            ));
        }

        Self {
            bewaartermijn: bewaartermijn.cloned(),
            hotspot: hotspot.cloned(),
            uiteindelijke_waarde,
            vernietigingsdatum,
            overbrengingsdatum,
            versie,
            toelichting,
        }
    }

    /// Is dit document al vernietigbaar per de gegeven datum?
    pub fn is_vernietigbaar_per(&self, per: NaiveDate) -> bool {
        match self.vernietigingsdatum {
            Some(datum) => per >= datum,
            None => false,
        }
    }

    /// Moet dit document overgebracht worden naar het archief?
    pub fn moet_overbrengen(&self) -> bool {
        self.uiteindelijke_waarde == Archiefwaarde::Permanent
    }

    /// Is dit document al overdraagbaar naar het archief?
    pub fn is_overdraagbaar_per(&self, per: NaiveDate) -> bool {
        match self.overbrengingsdatum {
            Some(datum) => per >= datum,
            None => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provisa_version_for_year() {
        assert_eq!(ProvisaVersion::for_document_year(2024), ProvisaVersion::V2020);
        assert_eq!(ProvisaVersion::for_document_year(2018), ProvisaVersion::V2014);
        assert_eq!(ProvisaVersion::for_document_year(2010), ProvisaVersion::V2005);
    }

    #[test]
    fn test_bewaartermijn_tijdelijk() {
        let termijn = Bewaartermijn::tijdelijk(10, "test-ref");
        assert_eq!(termijn.jaren, Some(10));
        assert_eq!(termijn.waarde, Archiefwaarde::Tijdelijk);

        let datum = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let vernietiging = termijn.vernietigingsdatum(datum);
        assert_eq!(vernietiging, Some(NaiveDate::from_ymd_opt(2030, 1, 1).unwrap()));
    }

    #[test]
    fn test_bewaartermijn_permanent() {
        let termijn = Bewaartermijn::permanent("test-ref");
        assert_eq!(termijn.jaren, None);
        assert_eq!(termijn.waarde, Archiefwaarde::Permanent);
    }

    #[test]
    fn test_hotspot_is_actief() {
        let hotspot = Hotspot::new(
            "test-1",
            "Test Hotspot",
            "Test beschrijving",
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        )
        .met_einddatum(NaiveDate::from_ymd_opt(2020, 12, 31).unwrap());

        assert!(hotspot.is_actief_op(NaiveDate::from_ymd_opt(2020, 6, 1).unwrap()));
        assert!(!hotspot.is_actief_op(NaiveDate::from_ymd_opt(2019, 12, 31).unwrap()));
        assert!(!hotspot.is_actief_op(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap()));
    }

    #[test]
    fn test_provinciaal_2020_lijst() {
        let lijst = ProvisaSelectielijst::provinciaal_2020();
        assert_eq!(lijst.versie, ProvisaVersion::V2020);
        assert_eq!(lijst.orgaan, ProvincieOrgaan::ProvincialeOrganen);

        // Verordening moet permanent zijn
        let termijn = lijst.zoek_bewaartermijn(&PetraCategorie::Bestuur, &BesluitType::Verordening);
        assert!(termijn.is_some());
        assert_eq!(termijn.unwrap().waarde, Archiefwaarde::Permanent);
    }

    #[test]
    fn test_provisa_beoordeling() {
        let lijst = ProvisaSelectielijst::provinciaal_2020();
        let datum = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let beoordeling = ProvisaBeoordeling::beoordeel(
            &lijst,
            &PetraCategorie::Bestuur,
            &BesluitType::Verordening,
            datum,
            None,
        );

        assert_eq!(beoordeling.uiteindelijke_waarde, Archiefwaarde::Permanent);
        assert!(beoordeling.moet_overbrengen());
        assert!(!beoordeling.is_vernietigbaar_per(datum));
    }

    #[test]
    fn test_hotspot_upgrade() {
        let mut register = HotspotRegister::new("Flevoland");
        register.voeg_toe(
            Hotspot::new(
                "hs-1",
                "Lelystad Airport",
                "Herstructurering Lelystad Airport",
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            )
            .met_categorieen(vec![PetraCategorie::VerkeerVervoer]),
        );

        let lijst = ProvisaSelectielijst::provinciaal_2020();
        let datum = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();

        let beoordeling = ProvisaBeoordeling::beoordeel(
            &lijst,
            &PetraCategorie::VerkeerVervoer,
            &BesluitType::Brief, // Normaal tijdelijk
            datum,
            Some(&register),
        );

        // Moet upgraded zijn naar permanent door hotspot
        assert_eq!(beoordeling.uiteindelijke_waarde, Archiefwaarde::Permanent);
        assert!(beoordeling.hotspot.is_some());
    }
}
