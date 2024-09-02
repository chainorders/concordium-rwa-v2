use concordium_cis2::{
    BalanceOfQuery, BalanceOfQueryParams, BalanceOfQueryResponse, IsTokenAmount, IsTokenId,
    TokenMetadataQueryParams, TokenMetadataQueryResponse, Transfer, TransferParams,
};
use concordium_std::{
    ContractAddress, DeserialWithState, EntrypointName, ExternStateApi, Host, MetadataUrl, Serial,
};

use crate::contract_client::{invoke_contract, invoke_contract_read_only, ContractClientError};
pub type Cis2ClientError = ContractClientError<()>;

#[inline(always)]
pub fn token_metadata<T: IsTokenId, State: Serial+DeserialWithState<ExternStateApi>>(
    host: &Host<State>,
    contract: &ContractAddress,
    params: &TokenMetadataQueryParams<T>,
) -> Result<TokenMetadataQueryResponse, Cis2ClientError> {
    invoke_contract_read_only(
        host,
        contract,
        EntrypointName::new_unchecked("tokenMetadata"),
        params,
    )
}

#[inline(always)]
pub fn token_metadata_single<T: IsTokenId, State: Serial+DeserialWithState<ExternStateApi>>(
    host: &Host<State>,
    contract: &ContractAddress,
    token_id: T,
) -> Result<MetadataUrl, Cis2ClientError> {
    let params = TokenMetadataQueryParams {
        queries: vec![token_id],
    };
    let res = token_metadata(host, contract, &params)?;
    let res = res.0.first().ok_or(ContractClientError::InvalidResponse)?;
    Ok(res.clone())
}

#[inline(always)]
pub fn transfer<T: IsTokenId, A: IsTokenAmount, State: Serial+DeserialWithState<ExternStateApi>>(
    host: &mut Host<State>,
    contract: &ContractAddress,
    params: &TransferParams<T, A>,
) -> Result<(), Cis2ClientError> {
    invoke_contract(
        host,
        contract,
        EntrypointName::new_unchecked("transfer"),
        params,
    )
}

#[inline(always)]
pub fn transfer_single<
    T: IsTokenId,
    A: IsTokenAmount,
    State: Serial+DeserialWithState<ExternStateApi>,
>(
    host: &mut Host<State>,
    contract: &ContractAddress,
    param: Transfer<T, A>,
) -> Result<(), Cis2ClientError> {
    let params = TransferParams(vec![param]);
    transfer(host, contract, &params)
}

#[inline(always)]
pub fn balance_of<
    T: IsTokenId,
    A: IsTokenAmount,
    State: Serial+DeserialWithState<ExternStateApi>,
>(
    host: &Host<State>,
    contract: &ContractAddress,
    params: &BalanceOfQueryParams<T>,
) -> Result<BalanceOfQueryResponse<A>, Cis2ClientError> {
    invoke_contract_read_only(
        host,
        contract,
        EntrypointName::new_unchecked("balanceOf"),
        &params,
    )
}

#[inline(always)]
pub fn balance_of_single<
    T: IsTokenId,
    A: IsTokenAmount+Copy,
    State: Serial+DeserialWithState<ExternStateApi>,
>(
    host: &Host<State>,
    contract: &ContractAddress,
    params: BalanceOfQuery<T>,
) -> Result<A, Cis2ClientError> {
    let res: BalanceOfQueryResponse<A> = balance_of(host, contract, &BalanceOfQueryParams {
        queries: vec![params],
    })?;
    let res = res.0.first().ok_or(ContractClientError::InvalidResponse)?;
    Ok(*res)
}
