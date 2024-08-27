#![allow(unused)]

use concordium_cis2::{
    BalanceOfQueryParams, BalanceOfQueryResponse, IsTokenAmount, IsTokenId, TransferParams,
};
use concordium_protocols::concordium_cis2_security::{
    AgentWithRoles, BurnParams, FreezeParams, PauseParams,
};
use concordium_smart_contract_testing::*;
use concordium_std::{Deserial, Serial};

use super::MAX_ENERGY;

pub fn add_agent<R>(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &AgentWithRoles<R>,
) -> ContractInvokeSuccess
where
    R: Serial,
{
    chain
        .contract_update(
            Signer::with_one_key(),
            sender.address,
            sender.address.into(),
            MAX_ENERGY,
            UpdateContractPayload {
                address:      contract,
                amount:       Amount::zero(),
                receive_name: OwnedReceiveName::construct_unchecked(
                    contract_name,
                    EntrypointName::new_unchecked("addAgent"),
                ),
                message:      OwnedParameter::from_serial(payload).unwrap(),
            },
        )
        .expect("Add Agent")
}

pub fn is_agent<R>(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &AgentWithRoles<R>,
) -> bool
where
    R: Serial,
{
    chain
        .contract_invoke(
            sender.address,
            Address::Account(sender.address),
            MAX_ENERGY,
            UpdateContractPayload {
                amount:       Amount::zero(),
                address:      contract,
                receive_name: OwnedReceiveName::construct_unchecked(
                    contract_name,
                    EntrypointName::new_unchecked("isAgent"),
                ),
                message:      OwnedParameter::from_serial(payload).unwrap(),
            },
        )
        .expect("Is Agent")
        .parse_return_value()
        .unwrap()
}

pub fn agents<R>(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
) -> Vec<AgentWithRoles<R>>
where
    R: Deserial,
{
    chain
        .contract_invoke(
            sender.address,
            Address::Account(sender.address),
            MAX_ENERGY,
            UpdateContractPayload {
                amount:       Amount::zero(),
                address:      contract,
                receive_name: OwnedReceiveName::construct_unchecked(
                    contract_name,
                    EntrypointName::new_unchecked("agents"),
                ),
                message:      OwnedParameter::from_serial(&()).unwrap(),
            },
        )
        .expect("Agents")
        .parse_return_value()
        .unwrap()
}

pub fn remove_agent(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &Address,
) -> ContractInvokeSuccess
where {
    chain
        .contract_update(
            Signer::with_one_key(),
            sender.address,
            sender.address.into(),
            MAX_ENERGY,
            UpdateContractPayload {
                address:      contract,
                amount:       Amount::zero(),
                receive_name: OwnedReceiveName::construct_unchecked(
                    contract_name,
                    EntrypointName::new_unchecked("removeAgent"),
                ),
                message:      OwnedParameter::from_serial(payload).unwrap(),
            },
        )
        .expect("Remove Agent")
}

pub fn set_identity_registry(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &ContractAddress,
) -> ContractInvokeSuccess
where {
    chain
        .contract_update(
            Signer::with_one_key(),
            sender.address,
            sender.address.into(),
            MAX_ENERGY,
            UpdateContractPayload {
                address:      contract,
                amount:       Amount::zero(),
                receive_name: OwnedReceiveName::construct_unchecked(
                    contract_name,
                    EntrypointName::new_unchecked("setIdentityRegistry"),
                ),
                message:      OwnedParameter::from_serial(payload).unwrap(),
            },
        )
        .expect("Set Identity Registry")
}

pub fn set_compliance(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &ContractAddress,
) -> ContractInvokeSuccess
where {
    chain
        .contract_update(
            Signer::with_one_key(),
            sender.address,
            sender.address.into(),
            MAX_ENERGY,
            UpdateContractPayload {
                address:      contract,
                amount:       Amount::zero(),
                receive_name: OwnedReceiveName::construct_unchecked(
                    contract_name,
                    EntrypointName::new_unchecked("setCompliance"),
                ),
                message:      OwnedParameter::from_serial(payload).unwrap(),
            },
        )
        .expect("Set Compliance")
}

pub fn identity_registry(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
) -> ContractAddress {
    chain
        .contract_invoke(
            sender.address,
            Address::Account(sender.address),
            MAX_ENERGY,
            UpdateContractPayload {
                amount:       Amount::zero(),
                address:      contract,
                receive_name: OwnedReceiveName::construct_unchecked(
                    contract_name,
                    EntrypointName::new_unchecked("identityRegistry"),
                ),
                message:      OwnedParameter::empty(),
            },
        )
        .expect("Identity Registry")
        .parse_return_value()
        .unwrap()
}

pub fn compliance(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
) -> ContractAddress {
    chain
        .contract_invoke(
            sender.address,
            Address::Account(sender.address),
            MAX_ENERGY,
            UpdateContractPayload {
                amount:       Amount::zero(),
                address:      contract,
                receive_name: OwnedReceiveName::construct_unchecked(
                    contract_name,
                    EntrypointName::new_unchecked("compliance"),
                ),
                message:      OwnedParameter::empty(),
            },
        )
        .expect("Compliance")
        .parse_return_value()
        .unwrap()
}

pub fn burn<T, A>(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &BurnParams<T, A>,
) -> ContractInvokeSuccess
where
    T: IsTokenId,
    A: IsTokenAmount,
{
    burn_raw(chain, sender, contract, contract_name, payload).expect("burn")
}

pub fn burn_raw<T, A>(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &BurnParams<T, A>,
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
            address:      contract,
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::construct_unchecked(
                contract_name,
                EntrypointName::new_unchecked("burn"),
            ),
            message:      OwnedParameter::from_serial(payload).unwrap(),
        },
    )
}

pub fn forced_burn<T, A>(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &BurnParams<T, A>,
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
            address:      contract,
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::construct_unchecked(
                contract_name,
                EntrypointName::new_unchecked("forcedBurn"),
            ),
            message:      OwnedParameter::from_serial(payload).unwrap(),
        },
    )
}

pub fn freeze<T, A>(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &FreezeParams<T, A>,
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
            address:      contract,
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::construct_unchecked(
                contract_name,
                EntrypointName::new_unchecked("freeze"),
            ),
            message:      OwnedParameter::from_serial(payload).unwrap(),
        },
    )
}

pub fn un_freeze<T, A>(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &FreezeParams<T, A>,
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
            address:      contract,
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::construct_unchecked(
                contract_name,
                EntrypointName::new_unchecked("unFreeze"),
            ),
            message:      OwnedParameter::from_serial(payload).unwrap(),
        },
    )
}

pub fn balance_of_frozen<T, A>(
    chain: &mut Chain,
    sender: &Account,
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
            sender.address,
            Address::Account(sender.address),
            MAX_ENERGY,
            UpdateContractPayload {
                amount:       Amount::zero(),
                address:      contract,
                receive_name: OwnedReceiveName::construct_unchecked(
                    contract_name,
                    EntrypointName::new_unchecked("balanceOfFrozen"),
                ),
                message:      OwnedParameter::from_serial(payload).unwrap(),
            },
        )
        .map(|r| r.parse_return_value().unwrap())
}

pub fn balance_of_un_frozen<T, A>(
    chain: &mut Chain,
    sender: &Account,
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
            sender.address,
            Address::Account(sender.address),
            MAX_ENERGY,
            UpdateContractPayload {
                amount:       Amount::zero(),
                address:      contract,
                receive_name: OwnedReceiveName::construct_unchecked(
                    contract_name,
                    EntrypointName::new_unchecked("balanceOfUnFrozen"),
                ),
                message:      OwnedParameter::from_serial(payload).unwrap(),
            },
        )
        .map(|r| r.parse_return_value().unwrap())
}

pub fn forced_transfer<T: IsTokenId, A: IsTokenAmount>(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &TransferParams<T, A>,
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
                EntrypointName::new_unchecked("forcedTransfer"),
            ),
            message:      OwnedParameter::from_serial(payload).unwrap(),
        },
    )
}

pub fn pause<T: IsTokenId>(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &PauseParams<T>,
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
                EntrypointName::new_unchecked("pause"),
            ),
            message:      OwnedParameter::from_serial(payload).unwrap(),
        },
    )
}
