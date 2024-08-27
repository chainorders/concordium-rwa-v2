use concordium_cis2::{
    IsTokenAmount, IsTokenId, TokenMetadataQueryParams, TokenMetadataQueryResponse, Transfer,
    TransferParams,
};
use concordium_std::{
    ContractAddress, DeserialWithState, EntrypointName, ExternStateApi, Host, MetadataUrl, Serial,
};

use crate::contract_client::{invoke_contract, invoke_contract_read_only, ContractClientError};

pub fn token_metadata<T: IsTokenId, State: Serial+DeserialWithState<ExternStateApi>>(
    host: &Host<State>,
    contract: ContractAddress,
    params: &TokenMetadataQueryParams<T>,
) -> Result<TokenMetadataQueryResponse, ContractClientError<()>> {
    invoke_contract_read_only(
        host,
        contract,
        EntrypointName::new_unchecked("tokenMetadata"),
        params,
    )
}

pub fn token_metadata_single<T: IsTokenId, State: Serial+DeserialWithState<ExternStateApi>>(
    host: &Host<State>,
    contract: ContractAddress,
    token_id: T,
) -> Result<MetadataUrl, ContractClientError<()>> {
    let params = TokenMetadataQueryParams {
        queries: vec![token_id],
    };
    let res = token_metadata(host, contract, &params)?;
    let res = res.0.first().ok_or(ContractClientError::InvalidResponse)?;
    Ok(res.clone())
}

pub fn transfer<T: IsTokenId, A: IsTokenAmount, State: Serial+DeserialWithState<ExternStateApi>>(
    host: &mut Host<State>,
    contract: ContractAddress,
    params: &TransferParams<T, A>,
) -> Result<(), ContractClientError<()>> {
    invoke_contract(
        host,
        contract,
        EntrypointName::new_unchecked("transfer"),
        params,
    )
}

pub fn transfer_single<
    T: IsTokenId,
    A: IsTokenAmount,
    State: Serial+DeserialWithState<ExternStateApi>,
>(
    host: &mut Host<State>,
    contract: ContractAddress,
    param: Transfer<T, A>,
) -> Result<(), ContractClientError<()>> {
    let params = TransferParams(vec![param]);
    transfer(host, contract, &params)
}
