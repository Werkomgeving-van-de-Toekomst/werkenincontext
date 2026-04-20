//! Settings voor systeemconfiguratie
//!
//! Settings bieden configuratiemogelijkheden per tenant, organisatie,
//! domein of gebruiker. Ondersteunt hiërarchische overrides en type-veilige
//! waarden.

// Re-export repository when server feature is enabled
#[cfg(feature = "server")]
pub mod repository;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

/// Setting voor systeemconfiguratie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub id: Uuid,
    pub key: SettingKey,
    pub value: serde_json::Value,
    pub value_type: SettingValueType,
    pub scope: SettingScope,
    pub scope_id: Option<Uuid>,
    pub description: Option<String>,
    pub default_value: Option<serde_json::Value>,
    pub validation_regex: Option<String>,
    pub is_encrypted: bool,
    pub is_public: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Setting {
    pub fn new(key: SettingKey, value: serde_json::Value, scope: SettingScope) -> Self {
        let value_type = SettingValueType::from_json(&value);
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            key,
            value,
            value_type,
            scope,
            scope_id: None,
            description: None,
            default_value: None,
            validation_regex: None,
            is_encrypted: false,
            is_public: false,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: now,
            updated_at: now,
        }
    }

    /// Controleer of waarde geldig is volgens regex
    pub fn is_valid(&self) -> Result<(), String> {
        if let Some(regex) = &self.validation_regex {
            let re = regex::Regex::new(regex)
                .map_err(|e| format!("Invalid regex: {}", e))?;

            let value_str = self.value.as_str()
                .ok_or_else(|| "Value is not a string".to_string())?;

            if !re.is_match(value_str) {
                return Err(format!("Value '{}' does not match pattern '{}'", value_str, regex));
            }
        }
        Ok(())
    }

    /// Reset naar default waarde
    pub fn reset_to_default(&mut self) -> Result<(), String> {
        if let Some(default) = &self.default_value {
            self.value = default.clone();
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err("No default value set".to_string())
        }
    }
}

/// Bekende setting keys
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SettingKey {
    // === Algemeen ===
    /// Organisatie naam
    OrganizationName,
    /// Organisatie website
    OrganizationWebsite,
    /// Organisatie logo URL
    OrganizationLogo,
    /// Primaire kleur (hex)
    PrimaryColor,
    /// Secundaire kleur (hex)
    SecondaryColor,

    // === Documenten ===
    /// Standaard bewaartermijn in jaren
    DefaultRetentionPeriod,
    /// Automatische versiebeheer aanzetten
    AutoVersioningEnabled,
    /// Maximum bestandsgrootte in MB
    MaxFileSizeMb,
    /// Toegestane bestandstypes (comma-separated)
    AllowedFileTypes,

    // === Woo ===
    /// Woo publicatie automatisch aanzetten
    WooAutoPublishEnabled,
    /// Standaard Woo weigeringsgrond
    WooDefaultRefusalReason,
    /// Woo publicatie URL template
    WooPublicationUrlTemplate,

    // === AVG ===
    /// PII automatische detectie aanzetten
    PiiAutoDetectEnabled,
    /// DPIA vereist voor nieuwe domeinen
    DpiaRequiredForDomains,

    // === Zoeken ===
    /// Maximum zoekresultaten
    SearchMaxResults,
    /// Zoek timeout in seconden
    SearchTimeoutSeconds,
    /// Semantisch zoek minimum confidence
    SemanticSearchMinConfidence,

    // === Notifications ===
    /// Email notificaties aanzetten
    EmailNotificationsEnabled,
    /// SMTP server
    SmtpServer,
    /// SMTP poort
    SmtpPort,
    /// SMTP gebruik SSL/TLS
    SmtpUseTls,

    // === Security ===
    /// Sessie timeout in minuten
    SessionTimeoutMinutes,
    /// MFA verplicht voor alle gebruikers
    MfaRequired,
    /// Maximum login pogingen
    MaxLoginAttempts,

    // === AI ===
    /// AI agent trust level (low/medium/high)
    AiTrustLevel,
    /// Auto-approval threshold (0.0-1.0)
    AiAutoApprovalThreshold,
    /// AI model naam
    AiModelName,

    // === Custom ===
    Custom(String),
}

impl From<String> for SettingKey {
    fn from(s: String) -> Self {
        Self::try_from(s.clone())
            .unwrap_or(Self::Custom(s))
    }
}

impl SettingKey {
    /// Controleer of dit een system setting is (niet door gebruikers aanpasbaar)
    pub fn is_system_setting(&self) -> bool {
        matches!(
            self,
            Self::SessionTimeoutMinutes
                | Self::MfaRequired
                | Self::MaxFileSizeMb
                | Self::PiiAutoDetectEnabled
        )
    }

    /// Controleer of dit een sensitive setting is (versleuteld opslaan)
    pub fn is_sensitive(&self) -> bool {
        matches!(
            self,
            Self::SmtpServer | Self::SmtpPort | Self::AiModelName | Self::Custom(_)
        )
    }
}

/// Type van de setting waarde
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingValueType {
    String,
    Integer,
    Float,
    Boolean,
    Json,
    StringArray,
}

impl std::str::FromStr for SettingValueType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "string" => Ok(Self::String),
            "integer" => Ok(Self::Integer),
            "float" => Ok(Self::Float),
            "boolean" => Ok(Self::Boolean),
            "json" => Ok(Self::Json),
            "string_array" => Ok(Self::StringArray),
            _ => Err(format!("unknown SettingValueType: {}", s)),
        }
    }
}

impl SettingValueType {
    fn from_json(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::String(_) => Self::String,
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    Self::Integer
                } else {
                    Self::Float
                }
            }
            serde_json::Value::Bool(_) => Self::Boolean,
            serde_json::Value::Array(_) => Self::StringArray,
            serde_json::Value::Object(_) => Self::Json,
            serde_json::Value::Null => Self::String,
        }
    }
}

/// Scope van de setting (hiërarchie: System > Tenant > Domain > User)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SettingScope {
    /// Systeem-breed (alle tenants)
    System,
    /// Tenant specifiek (gemeente/provincie)
    Tenant,
    /// Organisatie specifiek
    Organization,
    /// Informatiedomein specifiek
    Domain,
    /// Gebruiker specifiek
    User,
}

impl std::str::FromStr for SettingScope {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "system" => Ok(Self::System),
            "tenant" => Ok(Self::Tenant),
            "organization" => Ok(Self::Organization),
            "domain" => Ok(Self::Domain),
            "user" => Ok(Self::User),
            _ => Err(format!("unknown SettingScope: {}", s)),
        }
    }
}

impl SettingScope {
    /// Controleer of deze scope override mag geven aan andere scope
    pub fn can_override(&self, other: SettingScope) -> bool {
        // Specifieker (lager in hiërarchie) kan algemener (hoger) override
        match (self, other) {
            (SettingScope::System, _) => false,
            (SettingScope::Tenant, SettingScope::System) => true,
            (SettingScope::Organization, SettingScope::System | SettingScope::Tenant) => true,
            (SettingScope::Domain, SettingScope::System | SettingScope::Tenant | SettingScope::Organization) => true,
            (SettingScope::User, _) => true,
            _ => false,
        }
    }

    /// Niveau in hiërarchie (0 = systeem, 4 = user)
    pub fn level(&self) -> i32 {
        match self {
            SettingScope::System => 0,
            SettingScope::Tenant => 1,
            SettingScope::Organization => 2,
            SettingScope::Domain => 3,
            SettingScope::User => 4,
        }
    }
}

/// Setting historie voor audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingHistory {
    pub id: Uuid,
    pub setting_id: Uuid,
    pub key: SettingKey,
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
    pub change_reason: Option<String>,
}

impl SettingHistory {
    pub fn new(
        setting_id: Uuid,
        key: SettingKey,
        old_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
        changed_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            setting_id,
            key,
            old_value,
            new_value,
            changed_by,
            changed_at: Utc::now(),
            change_reason: None,
        }
    }
}

/// Setting bulk update operatie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingBulkUpdate {
    pub updates: Vec<SettingUpdateItem>,
    pub scope: SettingScope,
    pub scope_id: Option<Uuid>,
    pub updated_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingUpdateItem {
    pub key: SettingKey,
    pub value: serde_json::Value,
}

/// Setting query voor ophalen van settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingQuery {
    pub keys: Option<Vec<SettingKey>>,
    pub scope: Option<SettingScope>,
    pub scope_id: Option<Uuid>,
    pub include_defaults: bool,
    pub resolve_hierarchy: bool,
}

impl Default for SettingQuery {
    fn default() -> Self {
        Self {
            keys: None,
            scope: None,
            scope_id: None,
            include_defaults: true,
            resolve_hierarchy: true,
        }
    }
}

/// Setting groep voor UI weergave
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingGroup {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub settings: Vec<SettingKey>,
    pub order: i32,
}

impl SettingGroup {
    pub fn setting_groups() -> Vec<Self> {
        vec![
            Self {
                id: "general".to_string(),
                name: "Algemeen".to_string(),
                description: "Algemene organisatie instellingen".to_string(),
                icon: Some("cog".to_string()),
                settings: vec![
                    SettingKey::OrganizationName,
                    SettingKey::OrganizationWebsite,
                    SettingKey::OrganizationLogo,
                    SettingKey::PrimaryColor,
                    SettingKey::SecondaryColor,
                ],
                order: 1,
            },
            Self {
                id: "documents".to_string(),
                name: "Documenten".to_string(),
                description: "Instellingen voor documentbeheer".to_string(),
                icon: Some("file".to_string()),
                settings: vec![
                    SettingKey::DefaultRetentionPeriod,
                    SettingKey::AutoVersioningEnabled,
                    SettingKey::MaxFileSizeMb,
                    SettingKey::AllowedFileTypes,
                ],
                order: 2,
            },
            Self {
                id: "woo".to_string(),
                name: "Woo".to_string(),
                description: "Woo publicatie instellingen".to_string(),
                icon: Some("gavel".to_string()),
                settings: vec![
                    SettingKey::WooAutoPublishEnabled,
                    SettingKey::WooDefaultRefusalReason,
                    SettingKey::WooPublicationUrlTemplate,
                ],
                order: 3,
            },
            Self {
                id: "avg".to_string(),
                name: "AVG".to_string(),
                description: "AVG compliantie instellingen".to_string(),
                icon: Some("shield".to_string()),
                settings: vec![
                    SettingKey::PiiAutoDetectEnabled,
                    SettingKey::DpiaRequiredForDomains,
                ],
                order: 4,
            },
            Self {
                id: "search".to_string(),
                name: "Zoeken".to_string(),
                description: "Zoek functionaliteit instellingen".to_string(),
                icon: Some("search".to_string()),
                settings: vec![
                    SettingKey::SearchMaxResults,
                    SettingKey::SearchTimeoutSeconds,
                    SettingKey::SemanticSearchMinConfidence,
                ],
                order: 5,
            },
            Self {
                id: "security".to_string(),
                name: "Beveiliging".to_string(),
                description: "Security en authenticatie instellingen".to_string(),
                icon: Some("lock".to_string()),
                settings: vec![
                    SettingKey::SessionTimeoutMinutes,
                    SettingKey::MfaRequired,
                    SettingKey::MaxLoginAttempts,
                ],
                order: 6,
            },
            Self {
                id: "ai".to_string(),
                name: "AI".to_string(),
                description: "AI agent instellingen".to_string(),
                icon: Some("robot".to_string()),
                settings: vec![
                    SettingKey::AiTrustLevel,
                    SettingKey::AiAutoApprovalThreshold,
                    SettingKey::AiModelName,
                ],
                order: 7,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_creation() {
        let setting = Setting::new(
            SettingKey::OrganizationName,
            serde_json::json!("Gemeente Utrecht"),
            SettingScope::Organization,
        );

        assert_eq!(setting.value_type, SettingValueType::String);
        assert!(!setting.is_encrypted);
    }

    #[test]
    fn test_setting_type_detection() {
        assert_eq!(
            SettingValueType::from_json(&serde_json::json!("text")),
            SettingValueType::String
        );
        assert_eq!(
            SettingValueType::from_json(&serde_json::json!(42)),
            SettingValueType::Integer
        );
        assert_eq!(
            SettingValueType::from_json(&serde_json::json!(3.14)),
            SettingValueType::Float
        );
        assert_eq!(
            SettingValueType::from_json(&serde_json::json!(true)),
            SettingValueType::Boolean
        );
        assert_eq!(
            SettingValueType::from_json(&serde_json::json!(["a", "b"])),
            SettingValueType::StringArray
        );
    }

    #[test]
    fn test_setting_scope_hierarchy() {
        assert!(SettingScope::User.can_override(SettingScope::Domain));
        assert!(SettingScope::Domain.can_override(SettingScope::Organization));
        assert!(SettingScope::Organization.can_override(SettingScope::Tenant));
        assert!(SettingScope::Tenant.can_override(SettingScope::System));

        assert!(!SettingScope::System.can_override(SettingScope::User));
    }

    #[test]
    fn test_setting_key_validation() {
        assert!(SettingKey::SmtpServer.is_sensitive());
        assert!(SettingKey::MfaRequired.is_system_setting());
        assert!(!SettingKey::OrganizationName.is_sensitive());
    }

    #[test]
    fn test_setting_reset_to_default() {
        let mut setting = Setting::new(
            SettingKey::OrganizationName,
            serde_json::json!("New Name"),
            SettingScope::Organization,
        );
        setting.default_value = Some(serde_json::json!("Default Name"));

        assert!(setting.reset_to_default().is_ok());
        assert_eq!(setting.value, serde_json::json!("Default Name"));
    }

    #[test]
    fn test_custom_setting_key() {
        let key = SettingKey::from("custom.setting".to_string());
        assert!(matches!(key, SettingKey::Custom(_)));
    }

    #[test]
    fn test_setting_groups() {
        let groups = SettingGroup::setting_groups();
        assert!(!groups.is_empty());

        let general = groups.iter().find(|g| g.id == "general").unwrap();
        assert!(general.settings.contains(&SettingKey::OrganizationName));
    }
}
