use concordium_rust_sdk::constants::SHA256;
use concordium_rust_sdk::contract_client::CredentialStatus;
use concordium_rust_sdk::id::constants::ArCurve;
use concordium_rust_sdk::id::id_proof_types::{AtomicProof, AtomicStatement};
use concordium_rust_sdk::id::types::AttributeTag;
use concordium_rust_sdk::types::ContractAddress;
use concordium_rust_sdk::v2::{self, BlockIdentifier};
use concordium_rust_sdk::web3id::did::Network;
use concordium_rust_sdk::web3id::{
    self, CredentialHolderId, CredentialLookupError, CredentialProof,
    PresentationVerificationError, Web3IdAttribute,
};

pub type Presentation = concordium_rust_sdk::web3id::Presentation<ArCurve, Web3IdAttribute>;
pub type GlobalContext = concordium_rust_sdk::id::types::GlobalContext<ArCurve>;
pub type IdStatement = Vec<AtomicStatement<ArCurve, AttributeTag, Web3IdAttribute>>;
pub type CredStatement = Vec<AtomicStatement<ArCurve, String, Web3IdAttribute>>;
pub enum VerifyPresentationError {
    CredentialLookup(CredentialLookupError),
    InActiveCredential,
    PresentationVerification(PresentationVerificationError),
    InvalidChallenge,
}
impl From<CredentialLookupError> for VerifyPresentationError {
    fn from(e: CredentialLookupError) -> Self { VerifyPresentationError::CredentialLookup(e) }
}
impl From<PresentationVerificationError> for VerifyPresentationError {
    fn from(e: PresentationVerificationError) -> Self {
        VerifyPresentationError::PresentationVerification(e)
    }
}

pub struct VerifyPresentationResponse {
    pub revealed_attributes: Vec<(AttributeTag, Web3IdAttribute)>,
    pub credentials:         Vec<(ContractAddress, CredentialHolderId)>,
}

/// # Verifies that
/// 1. All the Web3 Id Credentials are active
/// 2. The challenge in the presentation matches the one sent
/// 3. The presentation is valid
/// # Returns
/// A list of the revealed id attributes
pub async fn verify_presentation(
    network: Network,
    concordium_client: &mut v2::Client,
    global_context: &GlobalContext,
    presentation: &Presentation,
    sent_challenge: [u8; SHA256],
) -> Result<VerifyPresentationResponse, VerifyPresentationError> {
    let public_data = web3id::get_public_data(
        concordium_client,
        network,
        presentation,
        BlockIdentifier::LastFinal,
    )
    .await?;
    if !public_data
        .iter()
        .all(|cm| matches!(cm.status, CredentialStatus::Active))
    {
        return Err(VerifyPresentationError::InActiveCredential);
    }
    let ver_res = presentation.verify(global_context, public_data.iter().map(|cm| &cm.inputs))?;
    if ver_res.challenge != sent_challenge.into() {
        return Err(VerifyPresentationError::InvalidChallenge);
    }
    let revealed_id_attributes = get_revealed_id_attributes(presentation);
    let credentials = get_credentials(presentation);
    Ok(VerifyPresentationResponse {
        revealed_attributes: revealed_id_attributes,
        credentials,
    })
}

/// Retrieves the revealed identity attributes from a given
/// `VerifyPresentationResponse`.
///
/// # Arguments
///
/// * `response` - A reference to the `VerifyPresentationResponse` from which to
///   retrieve the attributes.
///
/// # Returns
///
/// * A Result containing a `HashMap` where the keys are the attribute names and
///   the values are the revealed attributes, or an `Error`.
///
/// # Errors
///
/// This function will return an error if the `VerifyPresentationResponse` does
/// not contain the expected data or if there is a problem decoding the
/// attributes.
fn get_revealed_id_attributes(presentation: &Presentation) -> Vec<(AttributeTag, Web3IdAttribute)> {
    presentation
        .verifiable_credential
        .iter()
        .filter_map(|vc| match vc {
            CredentialProof::Account { proofs, .. } => Some(
                proofs
                    .iter()
                    .filter_map(|proof| match proof {
                        (
                            AtomicStatement::RevealAttribute { statement },
                            AtomicProof::RevealAttribute { attribute, .. },
                        ) => Some((statement.attribute_tag, attribute.clone())),
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
            ),
            CredentialProof::Web3Id { .. } => None,
        })
        .flatten()
        .collect::<Vec<_>>()
}

fn get_credentials(presentation: &Presentation) -> Vec<(ContractAddress, CredentialHolderId)> {
    presentation
        .verifiable_credential
        .iter()
        .filter_map(|vc| match vc {
            CredentialProof::Account { .. } => None,
            CredentialProof::Web3Id {
                contract, holder, ..
            } => Some((contract.to_owned(), holder.to_owned())),
        })
        .collect()
}
