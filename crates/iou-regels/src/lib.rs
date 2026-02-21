//! IOU-Regels: Open Regels integratie voor IOU-Modern
//!
//! Dit crate biedt toegang tot het Nederlandse overheidsregelregister via
//! de Linked Data / SPARQL interface van regels.overheid.nl.
//!
//! # Functionaliteit
//!
//! - [`architektur`]: IOU Architectuur componenten en standaarden
//! - [`client`]: SPARQL client voor de Open Regels Linked Data endpoint
//! - [`model`]: Domeinmodellen voor regelspecificaties (FLINT, DMN, ReSpec)
//! - [`tools`]: Agentic tools — plug-in klaar voor LLM tool_use loops
//! - [`compliance`]: Koppeling van regelspecificaties aan iou-core compliance types
//! - [`provisa`]: Provinciale selectielijsten en archiefwetgeving
//!
//! # Gebruik
//!
//! ```rust,no_run
//! use iou_regels::{IouArchitecture, IouComponent, Standard};
//!
//! fn main() {
//!     println!("IOU Architectuur v{}", IouArchitecture::default().version);
//!
//!     for component in IouArchitecture::components() {
//!         println!(" - {} ({})", component, component.description());
//!         if let Some(url) = component.live_url() {
//!             println!("   Live: {}", url);
//!         }
//!     }
//!
//!     for standard in IouArchitecture::standards() {
//!         println!("Standaard: {}", standard);
//!     }
//! }
//! ```
//!
//! # Omgevingen
//!
//! - **Productie**: `https://regels.overheid.nl/lab/sparql`
//! - **Acceptatie**: `https://acc.linkeddata.open-regels.nl/sparql`
//!   (gebruik dit voor ontwikkeling en experimenteren)
//!
//! # IOU Architectuur
//!
//! IOU integreert semantische web technologieën, decision models en Nederlandse
//! overheidsstandaarden. Zie [`IouArchitecture`] voor alle componenten en standaarden.
//!
//! Documentatie: https://iou-architectuur.open-regels.nl/

pub mod architektur;
pub mod compliance;
pub mod provisa;

// HTTP client and tools are only available for native builds
#[cfg(not(target_arch = "wasm32"))]
pub mod client;
#[cfg(not(target_arch = "wasm32"))]
pub mod model;
#[cfg(not(target_arch = "wasm32"))]
pub mod tools;

#[cfg(not(target_arch = "wasm32"))]
pub use client::OpenRegelsClient;
#[cfg(not(target_arch = "wasm32"))]
pub use model::{Regel, RegelDetail, RegelType, JuriconnectRef};
#[cfg(not(target_arch = "wasm32"))]
pub use tools::OpenRegelsTools;

pub use architektur::{
    IouArchitecture, IouComponent, Technology, Standard,
};
pub use provisa::{
    ProvisaVersion, ProvincieOrgaan, PetraCategorie, BesluitType,
    Bewaartermijn, Archiefwaarde, ProvisaBepaling, ProvisaSelectielijst,
    Hotspot, HotspotRegister, ProvisaBeoordeling,
};
