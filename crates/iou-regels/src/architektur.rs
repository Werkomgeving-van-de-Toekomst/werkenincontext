//! IOU Architectuur - Informatie Architectuur Framework
//!
//! Dit module beschrijft de IOU architectuur zoals gedefinieerd op
//! https://iou-architectuur.open-regels.nl/
//!
//! De IOU architectuur integreert semantische web technologieën, decision models
//! en Nederlandse overheidsstandaarden in een统一 systeem voor regelbeheer
//! en ruimtelijke planning.
//!
//! # Componenten
//!
//! - [`IouComponent`]: Alle componenten in het IOU ecosysteem
//! - [`Technology`]: Technology stack per component
//! - [`Standard`]: Standaarden waar het ecosysteem aan voldoet
//!
//! # Ecosysteem
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      IOU Architecture Ecosystem                   │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                  │
//! │  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐  │
//! │  │   Municipality   │     │  Keycloak IAM    │     │  Business API │  │
//! │  │     Portal       │────▶│      (OIDC)       │────▶│    (Node.js)  │  │
//! │  │     (React)      │     │                   │     │               │  │
//! │  └──────────────┘     └───────────────────┘     └───────┬───────┘  │
//! │                                                                  │
//! │  ┌──────────────┐                                     │           │
//! │  │  CPSV Editor  │                                     │           │
//! │  │    (React)    │────▶┌──────────────┐                │           │
//! │  └──────────────┘     │ TriplyDB KG │◀───────────────┘           │
//! │                        │  (SPARQL)   │                            │
//! │  ┌──────────────┐     └──────────────┘     ┌──────────────┐  │
//! │  │Linked Data   │                           │  Operaton    │  │
//! │  │  Explorer    │──────────────────────────▶│  DMN Engine  │  │
//! │  │    (React)   │                           │  (BPMN/DMN)  │  │
//! │  └──────────────┘                           └──────────────┘  │
//! │                                                                  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use std::fmt;

/// Alle componenten in het IOU architectuur ecosysteem
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IouComponent {
    /// Municipality Portal - React applicatie voor burgers en ambtenaren
    MunicipalityPortal,

    /// Keycloak IAM - Identity and Access Management met OIDC/JWT
    KeycloakIam,

    /// RONL Business API - Core business API layer (Node.js + Express)
    RonlBusinessApi,

    /// CPSV Editor - React applicatie voor CPSV-AP 3.2.0 compliant bestanden
    CpsvEditor,

    /// Linked Data Explorer - Web applicatie voor SPARQL queries
    LinkedDataExplorer,

    /// Operaton - BPMN/DMN decision engine
    OperatonEngine,

    /// Orchestration Service - Node.js service voor BPMN/DMN orchestration
    OrchestrationService,

    /// TriplyDB - Knowledge graph database met SPARQL endpoint
    TriplyDb,
}

impl IouComponent {
    /// URL van de live applicatie (indien beschikbaar)
    pub fn live_url(&self) -> Option<&'static str> {
        match self {
            IouComponent::MunicipalityPortal => Some("https://mijn.open-regels.nl"),
            IouComponent::CpsvEditor => Some("https://cpsv-editor.open-regels.nl"),
            IouComponent::LinkedDataExplorer => Some("https://linkeddata.open-regels.nl"),
            IouComponent::RonlBusinessApi => Some("https://backend.linkeddata.open-regels.nl"),
            IouComponent::OperatonEngine => Some("https://operaton.open-regels.nl"),
            IouComponent::KeycloakIam => Some("https://keycloak.open-regels.nl"),
            _ => None,
        }
    }

    /// Documentatie URL
    pub fn docs_url(&self) -> &'static str {
        match self {
            IouComponent::RonlBusinessApi => "https://iou-architectuur.open-regels.nl/ronl-business-api/",
            IouComponent::CpsvEditor => "https://iou-architectuur.open-regels.nl/cpsv-editor/",
            IouComponent::LinkedDataExplorer => "https://iou-architectuur.open-regels.nl/linked-data-explorer/",
            _ => "https://iou-architectuur.open-regels.nl/",
        }
    }

    /// Korte beschrijving
    pub fn description(&self) -> &'static str {
        match self {
            IouComponent::MunicipalityPortal => "React applicatie voor burgers en ambtenaren",
            IouComponent::KeycloakIam => "Identity en Access Management met OIDC/JWT",
            IouComponent::RonlBusinessApi => "Core business API layer met beveiligde authenticatie",
            IouComponent::CpsvEditor => "React CPSV-AP 3.2.0 editor voor overheidsdiensten",
            IouComponent::LinkedDataExplorer => "Web applicatie voor SPARQL queries",
            IouComponent::OperatonEngine => "BPMN/DMN decision engine voor regeluitvoering",
            IouComponent::OrchestrationService => "Orchestration service voor BPMN/DMN deployment",
            IouComponent::TriplyDb => "Knowledge graph database met SPARQL endpoint",
        }
    }
}

impl fmt::Display for IouComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IouComponent::MunicipalityPortal => write!(f, "Municipality Portal"),
            IouComponent::KeycloakIam => write!(f, "Keycloak IAM"),
            IouComponent::RonlBusinessApi => write!(f, "RONL Business API"),
            IouComponent::CpsvEditor => write!(f, "CPSV Editor"),
            IouComponent::LinkedDataExplorer => write!(f, "Linked Data Explorer"),
            IouComponent::OperatonEngine => write!(f, "Operaton DMN Engine"),
            IouComponent::OrchestrationService => write!(f, "Orchestration Service"),
            IouComponent::TriplyDb => write!(f, "TriplyDB"),
        }
    }
}

/// Technologie gebruikt in IOU componenten
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Technology {
    /// Keycloak - IAM platform
    Keycloak,
    /// Operaton - Open source BPMN/DMN platform
    Operaton,
    /// Node.js - Backend runtime
    NodeJs,
    /// Express - Node.js web framework
    Express,
    /// React - Frontend framework
    React,
    /// PostgreSQL - Relationele database
    PostgreSql,
    /// Redis - Cache en message broker
    Redis,
    /// Caddy - Reverse proxy
    Caddy,
    /// TriplyDB - Knowledge graph database
    TriplyDb,
}

impl Technology {
    /// License van de technologie
    pub fn license(&self) -> &'static str {
        match self {
            Technology::Keycloak => "Apache 2.0",
            Technology::Operaton => "Apache 2.0",
            Technology::NodeJs => "MIT",
            Technology::Express => "MIT",
            Technology::React => "MIT",
            Technology::PostgreSql => "PostgreSQL License",
            Technology::Redis => "BSD 3-Clause",
            Technology::Caddy => "Apache 2.0",
            Technology::TriplyDb => "Proprietary",
        }
    }
}

/// Standaarden waar IOU aan voldoet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standard {
    /// CPSV-AP 3.2.0 - EU Public Service Vocabulary
    CpsvAp,
    /// CPRMV - Core Public Rule Management Vocabulary
    Cprmv,
    /// RONL - Dutch Rules Vocabulary
    Ronl,
    /// BIO - Baseline Informatiebeveiliging Overheid
    Bio,
    /// NEN 7510 - Healthcare information security
    Nen7510,
    /// AVG/GDPR - Data protection
    AvgGdpr,
    /// DMN 1.4 - Decision Model and Notation
    Dmn,
    /// BPMN 2.0 - Business Process Model and Notation
    Bpmn,
    /// OIDC - OpenID Connect
    Oidc,
}

impl Standard {
    /// URL naar specificatie (indien beschikbaar)
    pub fn specification_url(&self) -> Option<&'static str> {
        match self {
            Standard::CpsvAp => Some("https://semic.org/solutions/cpsv-ap"),
            Standard::Dmn => Some("https://www.omg.org/spec/DMN/"),
            Standard::Bpmn => Some("https://www.omg.org/spec/BPMN/"),
            Standard::Oidc => Some("https://openid.net/connect/"),
            _ => None,
        }
    }
}

impl fmt::Display for Standard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Standard::CpsvAp => write!(f, "CPSV-AP 3.2.0"),
            Standard::Cprmv => write!(f, "CPRMV"),
            Standard::Ronl => write!(f, "RONL"),
            Standard::Bio => write!(f, "BIO"),
            Standard::Nen7510 => write!(f, "NEN 7510"),
            Standard::AvgGdpr => write!(f, "AVG/GDPR"),
            Standard::Dmn => write!(f, "DMN 1.4"),
            Standard::Bpmn => write!(f, "BPMN 2.0"),
            Standard::Oidc => write!(f, "OIDC"),
        }
    }
}

/// IOU Architectuur overzicht
pub struct IouArchitecture {
    /// Versie van de architectuur documentatie
    pub version: &'static str,
    /// Laatste update datum
    pub last_updated: &'static str,
    /// License van de documentatie
    pub license: &'static str,
}

impl Default for IouArchitecture {
    fn default() -> Self {
        Self {
            version: "1.0",
            last_updated: "February 2026",
            license: "EUPL v1.2",
        }
    }
}

impl IouArchitecture {
    /// Alle componenten in het ecosysteem
    pub fn components() -> &'static [IouComponent] {
        &[
            IouComponent::MunicipalityPortal,
            IouComponent::KeycloakIam,
            IouComponent::RonlBusinessApi,
            IouComponent::CpsvEditor,
            IouComponent::LinkedDataExplorer,
            IouComponent::OperatonEngine,
            IouComponent::OrchestrationService,
            IouComponent::TriplyDb,
        ]
    }

    /// Alle technologieën in de stack
    pub fn technologies() -> &'static [Technology] {
        &[
            Technology::Keycloak,
            Technology::Operaton,
            Technology::NodeJs,
            Technology::Express,
            Technology::React,
            Technology::PostgreSql,
            Technology::Redis,
            Technology::Caddy,
            Technology::TriplyDb,
        ]
    }

    /// Alle standaarden waar aan voldaan wordt
    pub fn standards() -> &'static [Standard] {
        &[
            Standard::CpsvAp,
            Standard::Cprmv,
            Standard::Ronl,
            Standard::Bio,
            Standard::Nen7510,
            Standard::AvgGdpr,
            Standard::Dmn,
            Standard::Bpmn,
            Standard::Oidc,
        ]
    }

    /// URL van de architectuur documentatie
    pub fn documentation_url() -> &'static str {
        "https://iou-architectuur.open-regels.nl/"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_urls() {
        assert_eq!(
            IouComponent::CpsvEditor.live_url(),
            Some("https://cpsv-editor.open-regels.nl")
        );
        assert_eq!(
            IouComponent::LinkedDataExplorer.live_url(),
            Some("https://linkeddata.open-regels.nl")
        );
    }

    #[test]
    fn test_all_components_have_descriptions() {
        for component in IouArchitecture::components() {
            assert!(!component.description().is_empty());
        }
    }

    #[test]
    fn test_technology_licenses() {
        assert_eq!(Technology::Keycloak.license(), "Apache 2.0");
        assert_eq!(Technology::React.license(), "MIT");
    }

    #[test]
    fn test_standards_display() {
        assert_eq!(Standard::CpsvAp.to_string(), "CPSV-AP 3.2.0");
        assert_eq!(Standard::Dmn.to_string(), "DMN 1.4");
    }
}
