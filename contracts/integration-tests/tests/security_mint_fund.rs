#![cfg(test)]

mod utils;

use concordium_cis2::{
    BalanceOfQuery, BalanceOfQueryParams, BalanceOfQueryResponse, OperatorUpdate, Receiver,
    TokenIdU32, TokenIdUnit, UpdateOperator,
};
use concordium_protocols::concordium_cis2_security::{AgentWithRoles, TokenUId};
use concordium_protocols::rate::Rate;
use concordium_smart_contract_testing::*;
use security_mint_fund::{
    CancelInvestParams, CancelInvestmentParam, ClaimInvestParams, ClaimInvestmentParam, FundState,
    State, TransferInvestParams,
};
use security_sft_rewards::types::InitParam;
use security_sft_single::types::ContractMetadataUrl;
use utils::cis2_conversions::to_token_id_vec;
use utils::{
    compliance, euroe, identity_registry, security_mint_fund_client, security_p2p_trading_client,
    security_sft_rewards_client, security_sft_single_client,
};

const INVESTMENT_TOKEN_ID: TokenIdU32 = TokenIdU32(0);
const INVESTMENT_TOKEN_METADATA_URL: &str = "example.com";
const WRAPPED_TOKEN_METADATA_URL: &str = "wrapped.example.com";
const WRAPPED_TOKEN_ID: TokenIdUnit = TokenIdUnit();
const MIN_REWARD_TOKEN_ID: TokenIdU32 = TokenIdU32(1);
const MIN_REWARD_METADATA_URL: &str = "blank_reward.example.com";
const ADMIN: AccountAddress = AccountAddress([0; 32]);
const INVESTOR_1: AccountAddress = AccountAddress([2; 32]);
const INVESTOR_2: AccountAddress = AccountAddress([3; 32]);
const TREASURY: AccountAddress = AccountAddress([4; 32]);
const COMPLIANT_NATIONALITIES: [&str; 2] = ["IN", "US"];
const DEFAULT_ACC_BALANCE: Amount = Amount {
    micro_ccd: 1_000_000_000_u64,
};
#[test]
fn normal_flow() {
    let admin = Account::new(ADMIN, DEFAULT_ACC_BALANCE);
    let mut chain = Chain::new();
    let (euroe_contract, ir_contract, compliance_contract) =
        setup_chain(&mut chain, &admin, COMPLIANT_NATIONALITIES.to_vec());
    let treasury = Account::new(TREASURY, DEFAULT_ACC_BALANCE);
    chain.create_account(treasury.clone());
    let investor_1 = Account::new(INVESTOR_1, DEFAULT_ACC_BALANCE);
    chain.create_account(investor_1.clone());
    let investor_2 = Account::new(INVESTOR_2, DEFAULT_ACC_BALANCE);
    chain.create_account(investor_2.clone());
    let investment_token_contract =
        create_rewards_token_contract(&mut chain, &admin, compliance_contract, ir_contract);
    let wrapped_token_contract =
        create_wrapped_token_contract(&mut chain, &admin, compliance_contract, ir_contract);

    let fund_contract = security_mint_fund_client::init(&mut chain, &admin, &State {
        token:            TokenUId {
            id:       to_token_id_vec(WRAPPED_TOKEN_ID),
            contract: wrapped_token_contract,
        },
        currency_token:   TokenUId {
            id:       to_token_id_vec(TokenIdUnit()),
            contract: euroe_contract,
        },
        investment_token: TokenUId {
            contract: investment_token_contract,
            id:       to_token_id_vec(INVESTMENT_TOKEN_ID),
        },
        fund_state:       FundState::Open,
        rate:             Rate {
            numerator:   1,
            denominator: 1000,
        },
    })
    .contract_address;
    identity_registry::register_nationalities(&mut chain, &admin, &ir_contract, vec![
        (
            Address::Account(investor_1.address),
            COMPLIANT_NATIONALITIES[1],
        ),
        (
            Address::Account(investor_2.address),
            COMPLIANT_NATIONALITIES[1],
        ),
    ]);

    security_sft_single_client::add_agent(
        &mut chain,
        &admin,
        wrapped_token_contract,
        &AgentWithRoles {
            address: fund_contract.into(),
            roles:   vec![
                security_sft_single::types::AgentRole::Mint,
                security_sft_single::types::AgentRole::Freeze,
                security_sft_single::types::AgentRole::ForcedTransfer,
                security_sft_single::types::AgentRole::ForcedBurn,
            ],
        },
    );
    security_sft_rewards_client::add_agent(
        &mut chain,
        &admin,
        investment_token_contract,
        &AgentWithRoles {
            address: fund_contract.into(),
            roles:   vec![security_sft_rewards::types::AgentRole::Mint],
        },
    );

    euroe::mint(&mut chain, &admin, euroe_contract, &euroe::MintParams {
        owner:  investor_1.address.into(),
        amount: 1000.into(),
    });
    euroe::mint(&mut chain, &admin, euroe_contract, &euroe::MintParams {
        owner:  investor_2.address.into(),
        amount: 2000.into(),
    });

    euroe::update_operator_single(&mut chain, &investor_1, euroe_contract, UpdateOperator {
        update:   OperatorUpdate::Add,
        operator: fund_contract.into(),
    });

    // First Investment
    security_mint_fund_client::transfer_invest(
        &mut chain,
        &investor_1,
        fund_contract,
        &TransferInvestParams {
            amount: 1000.into(),
        },
    )
    .expect("transfer_invest");

    euroe::update_operator_single(&mut chain, &investor_2, euroe_contract, UpdateOperator {
        update:   OperatorUpdate::Add,
        operator: fund_contract.into(),
    });

    // Second Investment
    security_mint_fund_client::transfer_invest(
        &mut chain,
        &investor_2,
        fund_contract,
        &TransferInvestParams {
            amount: 2000.into(),
        },
    )
    .expect("transfer_invest");

    assert_eq!(
        euroe::balance_of(&mut chain, &admin, euroe_contract, &BalanceOfQueryParams {
            queries: vec![
                BalanceOfQuery {
                    address:  investor_1.address.into(),
                    token_id: TokenIdUnit(),
                },
                BalanceOfQuery {
                    address:  investor_2.address.into(),
                    token_id: TokenIdUnit(),
                },
                BalanceOfQuery {
                    address:  fund_contract.into(),
                    token_id: TokenIdUnit(),
                },
                BalanceOfQuery {
                    address:  treasury.address.into(),
                    token_id: TokenIdUnit(),
                }
            ],
        })
        .expect("balance of euroe"),
        BalanceOfQueryResponse(vec![0.into(), 0.into(), 3000.into(), 0.into()])
    );
    assert_eq!(
        security_sft_single_client::balance_of(
            &mut chain,
            &admin,
            wrapped_token_contract,
            &BalanceOfQueryParams {
                queries: vec![
                    BalanceOfQuery {
                        address:  investor_1.address.into(),
                        token_id: TokenIdUnit(),
                    },
                    BalanceOfQuery {
                        address:  investor_2.address.into(),
                        token_id: TokenIdUnit(),
                    },
                ],
            }
        )
        .expect("balance of wrapped"),
        BalanceOfQueryResponse(vec![1.into(), 2.into()])
    );
    assert_eq!(
        security_sft_rewards_client::balance_of(
            &mut chain,
            &admin,
            investment_token_contract,
            &BalanceOfQueryParams {
                queries: vec![
                    BalanceOfQuery {
                        address:  investor_1.address.into(),
                        token_id: INVESTMENT_TOKEN_ID,
                    },
                    BalanceOfQuery {
                        address:  investor_2.address.into(),
                        token_id: INVESTMENT_TOKEN_ID,
                    },
                ],
            }
        )
        .expect("balance of security"),
        BalanceOfQueryResponse(vec![0.into(), 0.into()])
    );

    // Updating fund state to success
    security_mint_fund_client::update_fund_state(
        &mut chain,
        &admin,
        fund_contract,
        &FundState::Success(Receiver::Account(treasury.address)),
    )
    .expect("update_fund_state");
    // Euro e investments has been transferred to the treasury
    assert_eq!(
        euroe::balance_of(&mut chain, &admin, euroe_contract, &BalanceOfQueryParams {
            queries: vec![
                BalanceOfQuery {
                    address:  investor_1.address.into(),
                    token_id: TokenIdUnit(),
                },
                BalanceOfQuery {
                    address:  investor_2.address.into(),
                    token_id: TokenIdUnit(),
                },
                BalanceOfQuery {
                    address:  fund_contract.into(),
                    token_id: TokenIdUnit(),
                },
                BalanceOfQuery {
                    address:  treasury.address.into(),
                    token_id: TokenIdUnit(),
                }
            ],
        })
        .expect("balance of euroe"),
        BalanceOfQueryResponse(vec![0.into(), 0.into(), 0.into(), 3000.into()])
    );

    // Should not be able to cancel investment after completion
    security_mint_fund_client::cancel_investment(
        &mut chain,
        &investor_1,
        fund_contract,
        &CancelInvestParams {
            investments: vec![CancelInvestmentParam {
                investor: investor_1.address,
                amount:   1000.into(),
            }],
        },
    )
    .expect_err("cancel_investment after completion");

    security_mint_fund_client::claim_investment(
        &mut chain,
        &investor_1,
        fund_contract,
        &ClaimInvestParams {
            investments: vec![ClaimInvestmentParam {
                investor: investor_1.address,
            }],
        },
    )
    .expect("claim_investment investor 1");
    security_mint_fund_client::claim_investment(
        &mut chain,
        &investor_2,
        fund_contract,
        &ClaimInvestParams {
            investments: vec![ClaimInvestmentParam {
                investor: investor_2.address,
            }],
        },
    )
    .expect("claim_investment investor 2");
    assert_eq!(
        security_sft_single_client::balance_of(
            &mut chain,
            &admin,
            wrapped_token_contract,
            &BalanceOfQueryParams {
                queries: vec![
                    BalanceOfQuery {
                        address:  investor_1.address.into(),
                        token_id: TokenIdUnit(),
                    },
                    BalanceOfQuery {
                        address:  investor_2.address.into(),
                        token_id: TokenIdUnit(),
                    },
                ],
            }
        )
        .expect("balance of wrapped"),
        BalanceOfQueryResponse(vec![0.into(), 0.into()])
    );
    assert_eq!(
        security_sft_rewards_client::balance_of(
            &mut chain,
            &admin,
            investment_token_contract,
            &BalanceOfQueryParams {
                queries: vec![
                    BalanceOfQuery {
                        address:  investor_1.address.into(),
                        token_id: INVESTMENT_TOKEN_ID,
                    },
                    BalanceOfQuery {
                        address:  investor_2.address.into(),
                        token_id: INVESTMENT_TOKEN_ID,
                    },
                ],
            }
        )
        .expect("balance of security"),
        BalanceOfQueryResponse(vec![1.into(), 2.into()])
    );
}

fn create_rewards_token_contract(
    chain: &mut Chain,
    admin: &Account,
    compliance_contract: ContractAddress,
    ir_contract: ContractAddress,
) -> ContractAddress {
    security_sft_rewards_client::init(chain, admin, &InitParam {
        compliance:                compliance_contract,
        identity_registry:         ir_contract,
        metadata_url:              ContractMetadataUrl {
            hash: None,
            url:  INVESTMENT_TOKEN_METADATA_URL.to_string(),
        },
        sponsors:                  None,
        blank_reward_metadata_url: ContractMetadataUrl {
            hash: None,
            url:  MIN_REWARD_METADATA_URL.to_string(),
        },
        tracked_token_id:          INVESTMENT_TOKEN_ID,
        min_reward_token_id:       MIN_REWARD_TOKEN_ID,
    })
    .contract_address
}

fn create_wrapped_token_contract(
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
            url:  WRAPPED_TOKEN_METADATA_URL.to_string(),
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
    security_sft_rewards_client::deploy_module(chain, admin);
    security_p2p_trading_client::deploy_module(chain, admin);
    security_mint_fund_client::deploy_module(chain, admin);

    let euroe_contract = euroe::init(chain, admin).contract_address;
    euroe::grant_role(chain, admin, euroe_contract, &euroe::RoleTypes {
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
