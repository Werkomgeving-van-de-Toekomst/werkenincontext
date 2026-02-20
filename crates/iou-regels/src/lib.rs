//! IOU-Regels: Open Regels integratie voor IOU-Modern
//!
//! Dit crate biedt toegang tot het Nederlandse overheidsregelregister via
//! de Linked Data / SPARQL interface van regels.overheid.nl.
//!
//! # Functionaliteit
//!
//! - [`client`]: SPARQL client voor de Open Regels Linked Data endpoint
//! - [`model`]: Domeinmodellen voor regelspecificaties (FLINT, DMN, ReSpec)
//! - [`tools`]: Agentic tools — plug-in klaar voor LLM tool_use loops
//! - [`compliance`]: Koppeling van regelspecificaties aan iou-core compliance types
//!
//! # Gebruik
//!
//! ```rust,no_run
//! use iou_regels::tools::OpenRegelsTools;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let tools = OpenRegelsTools::acc();
//!     let regels: Vec<iou_regels::Regel> = tools.zoek_regels("beslagvrije voet").await?;
//!     println!("{} regels gevonden", regels.len());
//!     Ok(())
//! }
//! ```
//!
//! # Omgevingen
//!
//! - **Productie**: `https://regels.overheid.nl/lab/sparql`
//! - **Acceptatie**: `https://acc.linkeddata.open-regels.nl/sparql`
//!   (gebruik dit voor ontwikkeling en experimenteren)

pub mod client;
pub mod model;
pub mod tools;
pub mod compliance;
pub mod provisa;

pub use client::OpenRegelsClient;
pub use model::{Regel, RegelDetail, RegelType, JuriconnectRef};
pub use tools::OpenRegelsTools;
pub use provisa::{
    ProvisaVersion, ProvincieOrgaan, PetraCategorie, BesluitType,
    Bewaartermijn, Archiefwaarde, ProvisaBepaling, ProvisaSelectielijst,
    Hotspot, HotspotRegister, ProvisaBeoordeling,
};
