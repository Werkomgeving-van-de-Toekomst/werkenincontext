//! Local fallback dictionary for Dutch government organizations
//!
//! Provides canonical name mappings for common Dutch government
//! organizations and ministries. Used when the Rijksoverheid API
//! is unavailable or as a fast local cache.

use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Local fallback dictionary for Dutch government organizations
/// Used when Rijksoverheid API is unavailable
pub static FALLBACK_DICT: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Ministries with abbreviations
    // Ministry of Finance
    m.insert("MinFin".to_lowercase(), "Ministerie van Financiën");
    m.insert("minfin".to_string(), "Ministerie van Financiën");
    m.insert("Ministerie van Financiën".to_lowercase(), "Ministerie van Financiën");

    // Ministry of the Interior and Kingdom Relations
    m.insert("BZK".to_lowercase(), "Ministerie van Binnenlandse Zaken en Koninkrijksrelaties");
    m.insert("bzk".to_string(), "Ministerie van Binnenlandse Zaken en Koninkrijksrelaties");
    m.insert("Ministerie van Binnenlandse Zaken".to_lowercase(), "Ministerie van Binnenlandse Zaken en Koninkrijksrelaties");

    // Ministry of Health, Welfare and Sport
    m.insert("VWS".to_lowercase(), "Ministerie van Volksgezondheid, Welzijn en Sport");
    m.insert("vws".to_string(), "Ministerie van Volksgezondheid, Welzijn en Sport");

    // Ministry of Economic Affairs and Climate Policy
    m.insert("EZK".to_lowercase(), "Ministerie van Economische Zaken en Klimaat");
    m.insert("ezk".to_string(), "Ministerie van Economische Zaken en Klimaat");

    // Ministry of Education, Culture and Science
    m.insert("OCW".to_lowercase(), "Ministerie van Onderwijs, Cultuur en Wetenschap");
    m.insert("ocw".to_string(), "Ministerie van Onderwijs, Cultuur en Wetenschap");

    // Ministry of Justice and Security
    m.insert("JenV".to_lowercase(), "Ministerie van Justitie en Veiligheid");
    m.insert("jenv".to_string(), "Ministerie van Justitie en Veiligheid");

    // Ministry of Infrastructure and Water Management
    m.insert("IenW".to_lowercase(), "Ministerie van Infrastructuur en Waterstaat");
    m.insert("ienw".to_string(), "Ministerie van Infrastructuur en Waterstaat");

    // Ministry of Agriculture, Nature and Food Quality
    m.insert("LNV".to_lowercase(), "Ministerie van Landbouw, Natuur en Voedselkwaliteit");
    m.insert("lnv".to_string(), "Ministerie van Landbouw, Natuur en Voedselkwaliteit");

    // Ministry of Social Affairs and Employment
    m.insert("SZW".to_lowercase(), "Ministerie van Sociale Zaken en Werkgelegenheid");
    m.insert("szw".to_string(), "Ministerie van Sociale Zaken en Werkgelegenheid");

    // Ministry of Foreign Affairs
    m.insert("BZ".to_lowercase(), "Ministerie van Buitenlandse Zaken");
    m.insert("bz".to_string(), "Ministerie van Buitenlandse Zaken");

    // Common agencies and services
    m.insert("Rijkswaterstaat".to_lowercase(), "Rijkswaterstaat");
    m.insert("Dienst Wegverkeer".to_lowercase(), "Rijksdienst voor het Wegverkeer");
    m.insert("RDW".to_lowercase(), "Rijksdienst voor het Wegverkeer");
    m.insert("Rijksdienst voor het Wegverkeer".to_lowercase(), "Rijksdienst voor het Wegverkeer");
    m.insert("Belastingdienst".to_lowercase(), "Belastingdienst");
    m.insert("Centraal Planbureau".to_lowercase(), "Centraal Planbureau");
    m.insert("CPB".to_lowercase(), "Centraal Planbureau");
    m.insert("Nederlandse Bank".to_lowercase(), "De Nederlandsche Bank");
    m.insert("DNB".to_lowercase(), "De Nederlandsche Bank");
    m.insert("De Nederlandsche Bank".to_lowercase(), "De Nederlandsche Bank");

    // More agencies
    m.insert("RIVM".to_lowercase(), "Rijksinstituut voor Volksgezondheid en Milieu");
    m.insert("Rijksinstituut voor Volksgezondheid en Milieu".to_lowercase(), "Rijksinstituut voor Volksgezondheid en Milieu");
    m.insert("Inspectie SZW".to_lowercase(), "Inspectie Sociale Zaken en Werkgelegenheid");
    m.insert("UWV".to_lowercase(), "Uitvoeringsinstituut Werknemersverzekeringen");
    m.insert("SVB".to_lowercase(), "Sociale Verzekeringsbank");
    m.insert("Dutch Authority for Consumers and Markets".to_lowercase(), "Autoriteit Consument & Markt");
    m.insert("ACM".to_lowercase(), "Autoriteit Consument & Markt");

    // Provinces (examples)
    m.insert("Noord-Holland".to_lowercase(), "Provincie Noord-Holland");
    m.insert("Zuid-Holland".to_lowercase(), "Provincie Zuid-Holland");
    m.insert("Noord-Brabant".to_lowercase(), "Provincie Noord-Brabant");
    m.insert("Gelderland".to_lowercase(), "Provincie Gelderland");
    m.insert("Utrecht".to_lowercase(), "Provincie Utrecht");
    m.insert("Overijssel".to_lowercase(), "Provincie Overijssel");
    m.insert("Limburg".to_lowercase(), "Provincie Limburg");
    m.insert("Friesland".to_lowercase(), "Provincie Friesland");
    m.insert("Fryslân".to_lowercase(), "Provincie Friesland");
    m.insert("Drenthe".to_lowercase(), "Provincie Drenthe");
    m.insert("Groningen".to_lowercase(), "Provincie Groningen");
    m.insert("Zeeland".to_lowercase(), "Provincie Zeeland");

    // Major cities
    m.insert("Gemeente Amsterdam".to_lowercase(), "Gemeente Amsterdam");
    m.insert("Gemeente Rotterdam".to_lowercase(), "Gemeente Rotterdam");
    m.insert("Gemeente Den Haag".to_lowercase(), "Gemeente Den Haag");
    m.insert("Gemeente Utrecht".to_lowercase(), "Gemeente Utrecht");
    m.insert("Gemeente Eindhoven".to_lowercase(), "Gemeente Eindhoven");
    m.insert("Gemeente Groningen".to_lowercase(), "Gemeente Groningen");
    m.insert("Gemeente Tilburg".to_lowercase(), "Gemeente Tilburg");
    m.insert("Gemeente Almere".to_lowercase(), "Gemeente Almere");

    // Common variations
    m.insert("amsterdam".to_string(), "Gemeente Amsterdam");
    m.insert("rotterdam".to_string(), "Gemeente Rotterdam");
    m.insert("den haag".to_string(), "Gemeente Den Haag");
    m.insert("s-gravenhage".to_string(), "Gemeente Den Haag");
    m.insert(" utrecht".trim().to_string(), "Gemeente Utrecht");

    m
});

/// Get canonical name from fallback dictionary
///
/// Performs a case-insensitive lookup in the fallback dictionary.
///
/// # Arguments
///
/// * `name` - The organization name to look up
///
/// # Returns
///
/// * `Some(&str)` - The canonical name if found
/// * `None` - If the organization is not in the dictionary
pub fn get_fallback_canonical_name(name: &str) -> Option<&'static str> {
    FALLBACK_DICT.get(&name.trim().to_lowercase()).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minfin_mapping() {
        assert_eq!(
            get_fallback_canonical_name("MinFin"),
            Some("Ministerie van Financiën")
        );
        assert_eq!(
            get_fallback_canonical_name("minfin"),
            Some("Ministerie van Financiën")
        );
        assert_eq!(
            get_fallback_canonical_name("MINFIN"),
            Some("Ministerie van Financiën")
        );
    }

    #[test]
    fn test_unknown_org() {
        assert_eq!(get_fallback_canonical_name("Unknown Org Ltd"), None);
    }

    #[test]
    fn test_trimming() {
        assert_eq!(
            get_fallback_canonical_name("  MinFin  "),
            Some("Ministerie van Financiën")
        );
    }

    #[test]
    fn test_city_mapping() {
        assert_eq!(
            get_fallback_canonical_name("Amsterdam"),
            Some("Gemeente Amsterdam")
        );
        assert_eq!(
            get_fallback_canonical_name("amsterdam"),
            Some("Gemeente Amsterdam")
        );
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(get_fallback_canonical_name(""), None);
    }

    #[test]
    fn test_whitespace_only() {
        assert_eq!(get_fallback_canonical_name("   "), None);
    }
}
