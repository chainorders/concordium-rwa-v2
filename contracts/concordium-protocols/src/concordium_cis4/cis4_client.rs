use concordium_std::*;

use crate::contract_client::{invoke_contract_read_only, supports, ContractClientError};

use super::{CredentialHolderId, CredentialStatus, CIS4_STANDARD_IDENTIFIER};

pub type Cis4ClientError = ContractClientError<()>;

#[inline(always)]
pub fn credential_status<State: Serial + DeserialWithState<ExternStateApi>>(
    host: &Host<State>,
    contract: &ContractAddress,
    credential_holder_id: CredentialHolderId,
) -> Result<CredentialStatus, Cis4ClientError> {
    invoke_contract_read_only(
        host,
        contract,
        EntrypointName::new_unchecked("credentialStatus"),
        &Parameter::new_unchecked(&credential_holder_id.0),
    )
}

#[inline(always)]
pub fn issuer<State: Serial + DeserialWithState<ExternStateApi>>(
    host: &Host<State>,
    contract: &ContractAddress,
) -> Result<CredentialHolderId, Cis4ClientError> {
    invoke_contract_read_only(
        host,
        contract,
        EntrypointName::new_unchecked("issuer"),
        &Parameter::empty(),
    )
}

#[inline(always)]
pub fn supports_cis4<State: Serial + DeserialWithState<ExternStateApi>>(
    host: &Host<State>,
    contract: &ContractAddress,
) -> Result<bool, Cis4ClientError> {
    supports(host, contract, CIS4_STANDARD_IDENTIFIER)
}
