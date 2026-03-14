//! Self-Sovereign Identity (SSI) and Verifiable Credentials support
//!
//! This module provides VC validation, DID resolution, and claims extraction
//! for EBSI, nl-wallet, and other SSI wallet providers.

pub mod verifiable_credential;
pub mod did;
pub mod presentation;
pub mod resolver;

pub use verifiable_credential::{
    VerifiableCredential, VerifiablePresentation, VCValidationError,
    Claims, ClaimValue, DIDResolver,
};
pub use did::{DidMethod, DidDocument, DidKey, parse_did};
pub use presentation::PresentationValidator;
pub use resolver::UniversalDidResolver;
