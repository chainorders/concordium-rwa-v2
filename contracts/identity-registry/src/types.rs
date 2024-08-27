use concordium_protocols::concordium_cis2_security;

pub type ContractResult<T> = Result<T, super::error::Error>;
pub type AttributeTag = concordium_cis2_security::AttributeTag;
pub type AttributeValue = concordium_cis2_security::AttributeValue;
pub type Identity = concordium_cis2_security::Identity;
pub type Issuer = concordium_cis2_security::Issuer;
pub use concordium_cis2_security::{IdentityAttribute, IdentityCredential};
pub type CredentialId = concordium_std::PublicKeyEd25519;
