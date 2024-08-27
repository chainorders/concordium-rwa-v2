use concordium_std::{
    Address, ContractAddress, DeserialWithState, EntrypointName, ExternStateApi, Host, Serial,
};

use crate::contract_client::{invoke_contract_read_only, supports, ContractClientError};

use super::{Identity, IDENTITY_REGISTRY_STANDARD_IDENTIFIER};

pub type IdentityRegistryClientError = ContractClientError<()>;

#[inline(always)]
pub fn get_identity<State: Serial + DeserialWithState<ExternStateApi>>(
    host: &Host<State>,
    contract: ContractAddress,
    address: Address,
) -> Result<Identity, IdentityRegistryClientError> {
    invoke_contract_read_only(
        host,
        contract,
        EntrypointName::new_unchecked("getIdentity"),
        &address,
    )
}

#[inline(always)]
pub fn is_same<State: Serial + DeserialWithState<ExternStateApi>>(
    host: &Host<State>,
    contract: ContractAddress,
    address1: &Address,
    address2: &Address,
) -> Result<bool, IdentityRegistryClientError> {
    invoke_contract_read_only(
        host,
        contract,
        EntrypointName::new_unchecked("isSame"),
        &(address1, address2),
    )
}

#[inline(always)]
pub fn is_verified<State: Serial + DeserialWithState<ExternStateApi>>(
    host: &Host<State>,
    contract: ContractAddress,
    address: &Address,
) -> Result<bool, IdentityRegistryClientError> {
    invoke_contract_read_only(host, contract, EntrypointName::new_unchecked("isVerified"), address)
}

#[inline(always)]
pub fn supports_rwa_identity_registry_standard<
    State: Serial + DeserialWithState<ExternStateApi>,
>(
    host: &Host<State>,
    contract: ContractAddress,
) -> Result<bool, IdentityRegistryClientError> {
    supports(host, contract, IDENTITY_REGISTRY_STANDARD_IDENTIFIER)
}
