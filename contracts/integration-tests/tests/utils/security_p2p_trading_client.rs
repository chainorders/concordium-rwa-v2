#![allow(unused)]

use concordium_smart_contract_testing::*;
use concordium_std::ContractName;
use security_p2p_trading::{
    Deposit, ForceCancelSellParams, GetDepositParams, InitParam, TransferExchangeParams,
    TransferSellParams,
};

use super::MAX_ENERGY;

const MODULE_PATH: &str = "../security-p2p-trading/contract.wasm.v1";
const CONTRACT_NAME: ContractName = ContractName::new_unchecked("init_security_p2p_trading");

pub fn deploy_module(chain: &mut Chain, sender: &Account) -> ModuleDeploySuccess {
    chain
        .module_deploy_v1(
            Signer::with_one_key(),
            sender.address,
            module_load_v1(MODULE_PATH).unwrap(),
        )
        .expect("deploying module")
}

pub fn init(chain: &mut Chain, sender: &Account, params: &InitParam) -> ContractInitSuccess {
    chain
        .contract_init(
            Signer::with_one_key(),
            sender.address,
            MAX_ENERGY,
            InitContractPayload {
                amount:    Amount::zero(),
                init_name: CONTRACT_NAME.to_owned(),
                mod_ref:   module_load_v1(MODULE_PATH).unwrap().get_module_ref(),
                param:     OwnedParameter::from_serial(params).unwrap(),
            },
        )
        .expect("init")
}

pub fn transfer_sell(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    params: &TransferSellParams,
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
                CONTRACT_NAME,
                EntrypointName::new_unchecked("transferSell"),
            ),
            message:      OwnedParameter::from_serial(params).unwrap(),
        },
    )
}

pub fn cancel_sell(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
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
                CONTRACT_NAME,
                EntrypointName::new_unchecked("cancelSell"),
            ),
            message:      OwnedParameter::empty(),
        },
    )
}

pub fn force_cancel_sell(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    params: &ForceCancelSellParams,
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
                CONTRACT_NAME,
                EntrypointName::new_unchecked("forceCancelSell"),
            ),
            message:      OwnedParameter::from_serial(params).unwrap(),
        },
    )
}

pub fn transfer_exchange(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    params: &TransferExchangeParams,
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
                CONTRACT_NAME,
                EntrypointName::new_unchecked("transferExchange"),
            ),
            message:      OwnedParameter::from_serial(params).unwrap(),
        },
    )
}

pub fn get_deposit(
    chain: &mut Chain,
    invoker: &Account,
    contract: ContractAddress,
    params: &GetDepositParams,
) -> Result<Deposit, ContractInvokeError> {
    chain
        .contract_invoke(
            invoker.address,
            concordium_smart_contract_testing::Address::Account(invoker.address),
            MAX_ENERGY,
            UpdateContractPayload {
                address:      contract,
                amount:       Amount::zero(),
                receive_name: OwnedReceiveName::construct_unchecked(
                    CONTRACT_NAME,
                    EntrypointName::new_unchecked("getDeposit"),
                ),
                message:      OwnedParameter::from_serial(params).unwrap(),
            },
        )
        .map(|r| r.parse_return_value().unwrap())
}
