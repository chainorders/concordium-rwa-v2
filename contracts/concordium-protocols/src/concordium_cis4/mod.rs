pub mod cis4_client;

use concordium_cis2::StandardIdentifier;
use concordium_std::*;

/// The [credential holder id](https://proposals.concordium.software/CIS/cis-4.html#credentialholderid).
pub type CredentialHolderId = PublicKeyEd25519;

/// The [status of a credential](https://proposals.concordium.software/CIS/cis-4.html#credentialstatus).
#[derive(Serial, SchemaType, Deserial, PartialEq, Eq)]
pub enum CredentialStatus {
    Active,
    Revoked,
    Expired,
    NotActivated,
}

pub const CIS4_STANDARD_IDENTIFIER: StandardIdentifier = StandardIdentifier::new_unchecked("CIS-4");
