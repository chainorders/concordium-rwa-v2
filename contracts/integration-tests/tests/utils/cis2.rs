#![allow(unused)]

use concordium_cis2::{
    BalanceOfQuery, BalanceOfQueryParams, BalanceOfQueryResponse, IsTokenAmount, IsTokenId,
    OperatorOfQuery, OperatorOfQueryParams, OperatorOfQueryResponse, TransferParams,
};
use concordium_smart_contract_testing::*;

use super::MAX_ENERGY;

pub fn balance_of<T, A>(
    chain: &mut Chain,
    invoker: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &BalanceOfQueryParams<T>,
) -> Result<BalanceOfQueryResponse<A>, ContractInvokeError>
where
    T: IsTokenId,
    A: IsTokenAmount,
{
    chain
        .contract_invoke(
            invoker.address,
            concordium_smart_contract_testing::Address::Account(invoker.address),
            MAX_ENERGY,
            UpdateContractPayload {
                address:      contract,
                amount:       Amount::zero(),
                receive_name: OwnedReceiveName::construct_unchecked(
                    contract_name,
                    EntrypointName::new_unchecked("balanceOf"),
                ),
                message:      OwnedParameter::from_serial(payload).unwrap(),
            },
        )
        .map(|r| r.parse_return_value().unwrap())
}

pub fn balance_of_single<T, A>(
    chain: &mut Chain,
    invoker: &Account,
    contract: ContractAddress,
    token_id: T,
    address: Address,
    contract_name: ContractName,
) -> Result<A, ContractInvokeError>
where
    T: IsTokenId,
    A: IsTokenAmount+Copy,
{
    let BalanceOfQueryResponse(amounts) = balance_of::<T, A>(
        chain,
        invoker,
        contract,
        contract_name,
        &BalanceOfQueryParams {
            queries: vec![BalanceOfQuery { address, token_id }],
        },
    )?;
    Ok(amounts[0])
}

pub fn transfer<T, A>(
    chain: &mut Chain,
    sender: &Account,
    contract: &ContractAddress,
    contract_name: ContractName,
    payload: &TransferParams<T, A>,
) -> Result<ContractInvokeSuccess, ContractInvokeError>
where
    T: IsTokenId,
    A: IsTokenAmount,
{
    chain.contract_update(
        Signer::with_one_key(),
        sender.address,
        sender.address.into(),
        MAX_ENERGY,
        UpdateContractPayload {
            address:      *contract,
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::construct_unchecked(
                contract_name,
                EntrypointName::new_unchecked("transfer"),
            ),
            message:      OwnedParameter::from_serial(payload).unwrap(),
        },
    )
}

pub fn transfer_single<T, A>(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: concordium_cis2::Transfer<T, A>,
) -> Result<ContractInvokeSuccess, ContractInvokeError>
where
    T: IsTokenId,
    A: IsTokenAmount,
{
    transfer(
        chain,
        sender,
        &contract,
        contract_name,
        &TransferParams(vec![payload]),
    )
}

pub fn update_operator(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &concordium_cis2::UpdateOperatorParams,
) -> Result<ContractInvokeSuccess, ContractInvokeError> {
    chain.contract_update(
        Signer::with_one_key(),
        sender.address,
        sender.address.into(),
        MAX_ENERGY,
        UpdateContractPayload {
            address:      contract,
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::construct_unchecked(
                contract_name,
                EntrypointName::new_unchecked("updateOperator"),
            ),
            message:      OwnedParameter::from_serial(payload).unwrap(),
        },
    )
}

pub fn operator_of(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &concordium_cis2::OperatorOfQueryParams,
) -> ContractInvokeSuccess {
    chain
        .contract_invoke(
            sender.address,
            Address::Account(sender.address),
            MAX_ENERGY,
            UpdateContractPayload {
                address:      contract,
                amount:       Amount::zero(),
                receive_name: OwnedReceiveName::construct_unchecked(
                    contract_name,
                    EntrypointName::new_unchecked("operatorOf"),
                ),
                message:      OwnedParameter::from_serial(payload).unwrap(),
            },
        )
        .expect("operator_of")
}

pub fn operator_of_single(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    owner: Address,
    address: Address,
    contract_name: ContractName,
) -> bool {
    let OperatorOfQueryResponse(operators) = operator_of(
        chain,
        sender,
        contract,
        contract_name,
        &OperatorOfQueryParams {
            queries: vec![OperatorOfQuery { owner, address }],
        },
    )
    .parse_return_value()
    .unwrap();

    operators[0]
}
