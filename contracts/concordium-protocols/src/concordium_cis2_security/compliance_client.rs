use concordium_std::{
    ContractAddress, DeserialWithState, EntrypointName, ExternStateApi, Host, Serial,
};

use crate::{
    concordium_cis2_ext::{IsTokenAmount, IsTokenId},
    contract_client::{invoke_contract_read_only, supports, ContractClientError},
};

use super::{
    BurnedParam, CanTransferParam, MintedParam, TransferredParam, COMPLIANCE_STANDARD_IDENTIFIER,
};

pub type ComplianceError = ContractClientError<()>;

#[inline(always)]
pub fn can_transfer<
    State: Serial + DeserialWithState<ExternStateApi>,
    T: IsTokenId,
    A: IsTokenAmount,
>(
    host: &Host<State>,
    contract: ContractAddress,
    params: &CanTransferParam<T, A>,
) -> Result<bool, ComplianceError> {
    invoke_contract_read_only(host, contract, EntrypointName::new_unchecked("canTransfer"), params)
}

#[inline(always)]
pub fn burned<State: Serial + DeserialWithState<ExternStateApi>, T: IsTokenId, A: IsTokenAmount>(
    host: &Host<State>,
    contract: ContractAddress,
    params: &BurnedParam<T, A>,
) -> Result<(), ComplianceError> {
    invoke_contract_read_only(host, contract, EntrypointName::new_unchecked("burned"), params)
}

#[inline(always)]
pub fn minted<State: Serial + DeserialWithState<ExternStateApi>, T: IsTokenId, A: IsTokenAmount>(
    host: &Host<State>,
    contract: ContractAddress,
    params: &MintedParam<T, A>,
) -> Result<(), ComplianceError> {
    invoke_contract_read_only(host, contract, EntrypointName::new_unchecked("minted"), params)
}

#[inline(always)]
pub fn transferred<
    State: Serial + DeserialWithState<ExternStateApi>,
    T: IsTokenId,
    A: IsTokenAmount,
>(
    host: &Host<State>,
    contract: ContractAddress,
    params: &TransferredParam<T, A>,
) -> Result<(), ComplianceError> {
    invoke_contract_read_only(host, contract, EntrypointName::new_unchecked("transferred"), params)
}

#[inline(always)]
pub fn supports_rwa_compliance_standard<State: Serial + DeserialWithState<ExternStateApi>>(
    host: &Host<State>,
    contract: ContractAddress,
) -> Result<bool, ComplianceError> {
    supports(host, contract, COMPLIANCE_STANDARD_IDENTIFIER)
}
