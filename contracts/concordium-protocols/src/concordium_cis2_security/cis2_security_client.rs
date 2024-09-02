use concordium_cis2::{IsTokenAmount, IsTokenId};
use concordium_std::*;

use super::{Burn, BurnParams, FreezeParam, FreezeParams, MintParam, MintParams};
use crate::concordium_cis2_ext::cis2_client::Cis2ClientError;
use crate::contract_client::invoke_contract;

pub type Cis2SecurityClientError = Cis2ClientError;

#[inline(always)]
pub fn mint<T: IsTokenId, A: IsTokenAmount, State: Serial+DeserialWithState<ExternStateApi>>(
    host: &mut Host<State>,
    contract: &ContractAddress,
    params: &MintParams<T, A>,
) -> Result<(), Cis2ClientError> {
    invoke_contract(
        host,
        contract,
        EntrypointName::new_unchecked("mint"),
        params,
    )
}

#[inline(always)]
pub fn mint_single<
    T: IsTokenId,
    A: IsTokenAmount,
    State: Serial+DeserialWithState<ExternStateApi>,
>(
    host: &mut Host<State>,
    contract: &ContractAddress,
    token_id: T,
    mint_param: MintParam<A>,
) -> Result<(), Cis2ClientError> {
    mint(host, contract, &MintParams {
        token_id,
        owners: vec![mint_param],
    })
}

#[inline(always)]
pub fn freeze<T: IsTokenId, A: IsTokenAmount, State: Serial+DeserialWithState<ExternStateApi>>(
    host: &mut Host<State>,
    contract: &ContractAddress,
    params: &FreezeParams<T, A>,
) -> Result<(), Cis2ClientError> {
    invoke_contract(
        host,
        contract,
        EntrypointName::new_unchecked("freeze"),
        params,
    )
}

#[inline(always)]
pub fn freeze_single<
    T: IsTokenId,
    A: IsTokenAmount,
    State: Serial+DeserialWithState<ExternStateApi>,
>(
    host: &mut Host<State>,
    contract: &ContractAddress,
    owner: Address,
    freeze_param: FreezeParam<T, A>,
) -> Result<(), Cis2ClientError> {
    freeze(host, contract, &FreezeParams {
        owner,
        tokens: vec![freeze_param],
    })
}

#[inline(always)]
pub fn force_burn<
    T: IsTokenId,
    A: IsTokenAmount,
    State: Serial+DeserialWithState<ExternStateApi>,
>(
    host: &mut Host<State>,
    contract: &ContractAddress,
    params: &BurnParams<T, A>,
) -> Result<(), Cis2ClientError> {
    invoke_contract(
        host,
        contract,
        EntrypointName::new_unchecked("forcedBurn"),
        params,
    )
}

#[inline(always)]
pub fn force_burn_single<
    T: IsTokenId,
    A: IsTokenAmount,
    State: Serial+DeserialWithState<ExternStateApi>,
>(
    host: &mut Host<State>,
    contract: &ContractAddress,
    params: Burn<T, A>,
) -> Result<(), Cis2ClientError> {
    force_burn(host, contract, &BurnParams(vec![params]))
}
