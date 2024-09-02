#![cfg(test)]

mod utils;

use concordium_cis2::{
    BalanceOfQuery, BalanceOfQueryParams, BalanceOfQueryResponse, OperatorUpdate, TokenAmountU64,
    TokenIdUnit, UpdateOperator,
};
use concordium_protocols::concordium_cis2_security::TokenUId;
use concordium_protocols::rate::Rate;
use concordium_smart_contract_testing::*;
use concordium_std::{AccountAddress, Amount};
use security_p2p_trading::{
    Deposit, ExchangeParams, GetDepositParams, InitParam, TransferExchangeParams,
    TransferSellParams,
};
use security_sft_single::types::{ContractMetadataUrl, MintParam, MintParams};
use utils::cis2_conversions::to_token_id_vec;
use utils::euroe::RoleTypes;
use utils::{
    compliance, euroe, identity_registry, security_p2p_trading_client, security_sft_single_client,
};

const TOKEN_ID: TokenIdUnit = TokenIdUnit();
const METADATA_URL: &str = "example.com";
const ADMIN: AccountAddress = AccountAddress([0; 32]);
const HOLDER: AccountAddress = AccountAddress([2; 32]);
const HOLDER_2: AccountAddress = AccountAddress([3; 32]);
const COMPLIANT_NATIONALITIES: [&str; 2] = ["IN", "US"];
const DEFAULT_ACC_BALANCE: Amount = Amount {
    micro_ccd: 1_000_000_000_u64,
};

#[test]
pub fn normal_flow() {
    let admin = Account::new(ADMIN, DEFAULT_ACC_BALANCE);
    let mut chain = Chain::new();
    let holder = Account::new(HOLDER, DEFAULT_ACC_BALANCE);
    chain.create_account(holder.clone());
    let holder_2 = Account::new(HOLDER_2, DEFAULT_ACC_BALANCE);
    chain.create_account(holder_2.clone());

    let (euroe_contract, ir_contract, compliance_contract) =
        setup_chain(&mut chain, &admin, COMPLIANT_NATIONALITIES.to_vec());
    let token_contract =
        create_token_contract(&mut chain, &admin, compliance_contract, ir_contract);
    let trading_contract = security_p2p_trading_client::init(&mut chain, &admin, &InitParam {
        currency: TokenUId {
            id:       to_token_id_vec(TokenIdUnit()),
            contract: euroe_contract,
        },
        token:    TokenUId {
            id:       to_token_id_vec(TOKEN_ID),
            contract: token_contract,
        },
    })
    .contract_address;
    identity_registry::register_nationalities(&mut chain, &admin, &ir_contract, vec![
        (Address::Account(holder.address), COMPLIANT_NATIONALITIES[1]),
        (
            Address::Account(holder_2.address),
            COMPLIANT_NATIONALITIES[1],
        ),
        (
            Address::Contract(trading_contract),
            COMPLIANT_NATIONALITIES[1],
        ),
    ]);

    security_sft_single_client::mint(&mut chain, &admin, &token_contract, &MintParams {
        owners:   vec![MintParam {
            amount:  TokenAmountU64(50),
            address: holder.address,
        }],
        token_id: TOKEN_ID,
    })
    .expect("should mint");
    security_sft_single_client::update_operator_single(
        &mut chain,
        &holder,
        token_contract,
        UpdateOperator {
            update:   OperatorUpdate::Add,
            operator: trading_contract.into(),
        },
    )
    .expect("should update operator");

    let rate = Rate::new(1, 1000).unwrap();
    security_p2p_trading_client::transfer_sell(
        &mut chain,
        &holder,
        trading_contract,
        &TransferSellParams {
            amount: TokenAmountU64(10),
            rate,
        },
    )
    .expect("should transfer sell");
    assert_eq!(
        security_sft_single_client::balance_of_single(
            &mut chain,
            &admin,
            token_contract,
            TOKEN_ID,
            holder.address.into()
        )
        .expect("should get balance"),
        TokenAmountU64(40)
    );
    assert_eq!(
        security_p2p_trading_client::get_deposit(
            &mut chain,
            &admin,
            trading_contract,
            &GetDepositParams {
                from: holder.address,
            }
        )
        .expect("should get deposit"),
        Deposit {
            amount: TokenAmountU64(10),
            rate
        }
    );

    euroe::update_operator_single(&mut chain, &holder_2, euroe_contract, UpdateOperator {
        update:   OperatorUpdate::Add,
        operator: trading_contract.into(),
    });
    euroe::mint(&mut chain, &admin, euroe_contract, &euroe::MintParams {
        owner:  holder_2.address.into(),
        amount: TokenAmountU64(1000),
    });
    security_p2p_trading_client::transfer_exchange(
        &mut chain,
        &holder_2,
        trading_contract,
        &TransferExchangeParams {
            pay: TokenAmountU64(1000),
            get: ExchangeParams {
                from: holder.address,
                rate,
            },
        },
    )
    .expect("should transfer buy");
    assert_eq!(
        euroe::balance_of_single(
            &mut chain,
            &holder_2,
            euroe_contract,
            holder_2.address.into()
        ),
        TokenAmountU64(0)
    );
    assert_eq!(
        security_p2p_trading_client::get_deposit(
            &mut chain,
            &admin,
            trading_contract,
            &GetDepositParams {
                from: holder.address,
            }
        )
        .expect("should get deposit"),
        Deposit {
            amount: TokenAmountU64(9),
            rate
        }
    );
    assert_eq!(
        security_sft_single_client::balance_of(
            &mut chain,
            &admin,
            token_contract,
            &BalanceOfQueryParams {
                queries: vec![
                    BalanceOfQuery {
                        token_id: TOKEN_ID,
                        address:  holder.address.into(),
                    },
                    BalanceOfQuery {
                        token_id: TOKEN_ID,
                        address:  holder_2.address.into(),
                    }
                ],
            }
        )
        .expect("should get balance"),
        BalanceOfQueryResponse(vec![TokenAmountU64(40), TokenAmountU64(1),])
    );
}

fn create_token_contract(
    chain: &mut Chain,
    admin: &Account,
    compliance_contract: ContractAddress,
    ir_contract: ContractAddress,
) -> ContractAddress {
    security_sft_single_client::init(chain, admin, &security_sft_single::types::InitParam {
        compliance:        compliance_contract,
        identity_registry: ir_contract,
        metadata_url:      ContractMetadataUrl {
            hash: None,
            url:  METADATA_URL.to_string(),
        },
        sponsors:          None,
    })
    .contract_address
}

fn setup_chain(
    chain: &mut Chain,
    admin: &Account,
    compliant_nationalities: Vec<&str>,
) -> (ContractAddress, ContractAddress, ContractAddress) {
    chain.create_account(admin.clone());

    euroe::deploy_module(chain, admin);
    identity_registry::deploy_module(chain, admin);
    compliance::deploy_module(chain, admin);
    security_sft_single_client::deploy_module(chain, admin);
    security_p2p_trading_client::deploy_module(chain, admin);

    let euroe_contract = euroe::init(chain, admin).contract_address;
    euroe::grant_role(chain, admin, euroe_contract, &RoleTypes {
        adminrole: admin.address.into(),
        blockrole: admin.address.into(),
        burnrole:  admin.address.into(),
        mintrole:  admin.address.into(),
        pauserole: admin.address.into(),
    });
    let ir_contract = identity_registry::init(chain, admin).contract_address;
    let compliance_contract =
        compliance::init_all(chain, admin, ir_contract, compliant_nationalities).contract_address;

    (euroe_contract, ir_contract, compliance_contract)
}
