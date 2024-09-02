#![allow(unused)]

use concordium_cis2::{TransferParams, UpdateOperator, UpdateOperatorParams};
use concordium_smart_contract_testing::*;
use security_sft_single::types::*;

use super::{cis2, cis2_security, cis2_security_rewards, MAX_ENERGY};

pub const MODULE_PATH: &str = "../security-sft-single/contract.wasm.v1";
pub const CONTRACT_NAME: ContractName = ContractName::new_unchecked("init_security_sft_single");
pub fn deploy_module(chain: &mut Chain, sender: &Account) -> ModuleDeploySuccess {
    chain
        .module_deploy_v1(
            Signer::with_one_key(),
            sender.address,
            module_load_v1(MODULE_PATH).unwrap(),
        )
        .expect("deploying module")
}

pub fn init(chain: &mut Chain, sender: &Account, param: &InitParam) -> ContractInitSuccess {
    chain
        .contract_init(
            Signer::with_one_key(),
            sender.address,
            MAX_ENERGY,
            InitContractPayload {
                amount:    Amount::zero(),
                init_name: CONTRACT_NAME.to_owned(),
                mod_ref:   module_load_v1(MODULE_PATH).unwrap().get_module_ref(),
                param:     OwnedParameter::from_serial(param).unwrap(),
            },
        )
        .expect("init")
}

pub fn identity_registry(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
) -> ContractAddress {
    cis2_security::identity_registry(chain, sender, contract, CONTRACT_NAME)
}

pub fn set_identity_registry(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: &ContractAddress,
) -> ContractInvokeSuccess {
    cis2_security::set_identity_registry(chain, sender, contract, CONTRACT_NAME, payload)
}

pub fn compliance(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
) -> ContractAddress {
    cis2_security::compliance(chain, sender, contract, CONTRACT_NAME)
}

pub fn set_compliance(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: &ContractAddress,
) -> ContractInvokeSuccess {
    cis2_security::set_compliance(chain, sender, contract, CONTRACT_NAME, payload)
}

pub fn add_agent(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: &Agent,
) -> ContractInvokeSuccess {
    cis2_security::add_agent(chain, sender, contract, CONTRACT_NAME, payload)
}

pub fn is_agent(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: &Agent,
) -> bool {
    cis2_security::is_agent(chain, sender, contract, CONTRACT_NAME, payload)
}

pub fn agents(chain: &mut Chain, sender: &Account, contract: ContractAddress) -> Vec<Agent> {
    cis2_security::agents(chain, sender, contract, CONTRACT_NAME)
}

pub fn remove_agent(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: &Address,
) -> ContractInvokeSuccess {
    cis2_security::remove_agent(chain, sender, contract, CONTRACT_NAME, payload)
}

pub fn mint(
    chain: &mut Chain,
    sender: &Account,
    contract: &ContractAddress,
    payload: &MintParams,
) -> Result<ContractInvokeSuccess, ContractInvokeError> {
    chain.contract_update(
        Signer::with_one_key(),
        sender.address,
        sender.address.into(),
        MAX_ENERGY,
        UpdateContractPayload {
            address:      *contract,
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::construct_unchecked(
                CONTRACT_NAME,
                EntrypointName::new_unchecked("mint"),
            ),
            message:      OwnedParameter::from_serial(payload).unwrap(),
        },
    )
}

pub fn transfer_single(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: concordium_cis2::Transfer<TokenId, TokenAmount>,
) -> Result<ContractInvokeSuccess, ContractInvokeError> {
    cis2::transfer_single(chain, sender, contract, CONTRACT_NAME, payload)
}

pub fn forced_transfer_single(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: concordium_cis2::Transfer<TokenId, TokenAmount>,
) -> Result<ContractInvokeSuccess, ContractInvokeError> {
    cis2_security::forced_transfer(
        chain,
        sender,
        contract,
        CONTRACT_NAME,
        &TransferParams(vec![payload]),
    )
}

pub fn balance_of(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: &concordium_cis2::BalanceOfQueryParams<TokenId>,
) -> Result<concordium_cis2::BalanceOfQueryResponse<TokenAmount>, ContractInvokeError> {
    cis2::balance_of(chain, sender, contract, CONTRACT_NAME, payload)
}

pub fn balance_of_single(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    token_id: TokenId,
    address: Address,
) -> Result<TokenAmount, ContractInvokeError> {
    cis2::balance_of_single(chain, sender, contract, token_id, address, CONTRACT_NAME)
}

pub fn burn(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: &BurnParams,
) -> ContractInvokeSuccess {
    cis2_security::burn(chain, sender, contract, CONTRACT_NAME, payload)
}

pub fn burn_raw(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: &BurnParams,
) -> Result<ContractInvokeSuccess, ContractInvokeError> {
    cis2_security::burn_raw(chain, sender, contract, CONTRACT_NAME, payload)
}

pub fn forced_burn(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: &BurnParams,
) -> Result<ContractInvokeSuccess, ContractInvokeError> {
    cis2_security::forced_burn(chain, sender, contract, CONTRACT_NAME, payload)
}

pub fn freeze(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: &FreezeParams,
) -> Result<ContractInvokeSuccess, ContractInvokeError> {
    cis2_security::freeze(chain, sender, contract, CONTRACT_NAME, payload)
}

pub fn un_freeze(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: &FreezeParams,
) -> Result<ContractInvokeSuccess, ContractInvokeError> {
    cis2_security::un_freeze(chain, sender, contract, CONTRACT_NAME, payload)
}

pub fn balance_of_frozen(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: &BalanceOfQueryParams,
) -> Result<BalanceOfQueryResponse, ContractInvokeError> {
    cis2_security::balance_of_frozen(chain, sender, contract, CONTRACT_NAME, payload)
}

pub fn balance_of_un_frozen(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: &BalanceOfQueryParams,
) -> Result<BalanceOfQueryResponse, ContractInvokeError> {
    cis2_security::balance_of_un_frozen(chain, sender, contract, CONTRACT_NAME, payload)
}

pub fn pause(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: &PauseParams,
) -> Result<ContractInvokeSuccess, ContractInvokeError> {
    cis2_security::pause(chain, sender, contract, CONTRACT_NAME, payload)
}

pub fn update_operator(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: &UpdateOperatorParams,
) -> Result<ContractInvokeSuccess, ContractInvokeError> {
    cis2::update_operator(chain, sender, contract, CONTRACT_NAME, payload)
}

pub fn update_operator_single(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    payload: UpdateOperator,
) -> Result<ContractInvokeSuccess, ContractInvokeError> {
    update_operator(
        chain,
        sender,
        contract,
        &UpdateOperatorParams(vec![payload]),
    )
}
