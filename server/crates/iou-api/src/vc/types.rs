//! Re-export of VC types from parent module
//!
//! This module re-exports the common types for convenience.

pub use crate::vc::{
    VcConfig, VcError, WalletAuthRequest, WalletAuthResponse,
    VcUserContext, VerifiablePresentation, VerifiableCredential,
    CustomMdtAttributes, OneOrMany, Issuer, CredentialSubject, Proof,
    PresentationSubmission, DescriptorMap,
};
