#![cfg(test)]

mod utils;

use cis2_conversions::to_token_id_vec;
use concordium_cis2::{
    BalanceOfQuery, BalanceOfQueryResponse, OperatorUpdate, Receiver, TokenAmountU64, TokenIdU32,
    TokenIdUnit, UpdateOperator,
};
use concordium_protocols::concordium_cis2_ext::PlusSubOne;
use concordium_protocols::concordium_cis2_security::{AgentWithRoles, MintParam};
use concordium_protocols::rate::Rate;
use concordium_smart_contract_testing::*;
use security_sft_rewards::rewards::{
    AddRewardContractParam, ClaimRewardsParam, ClaimRewardsParams, TransferAddRewardParams,
};
use security_sft_rewards::types::{ContractMetadataUrl, InitParam, MintParams};
use utils::*;

const SFT_TOKEN_ID: TokenIdU32 = TokenIdU32(0);
const SFT_METADATA_URL: &str = "example.com";
const MIN_REWARD_TOKEN_ID: TokenIdU32 = TokenIdU32(1);
const MIN_REWARD_METADATA_URL: &str = "blank_reward.example.com";
const ADMIN: AccountAddress = AccountAddress([0; 32]);
const AGENT_MINT: AccountAddress = AccountAddress([1; 32]);
const HOLDER: AccountAddress = AccountAddress([2; 32]);
const COMPLIANT_NATIONALITIES: [&str; 2] = ["IN", "US"];
const DEFAULT_ACC_BALANCE: Amount = Amount {
    micro_ccd: 1_000_000_000_u64,
};

#[test]
fn normal_reward_distribution() {
    let admin = Account::new(ADMIN, DEFAULT_ACC_BALANCE);
    let mut chain = Chain::new();
    let (euroe, ir_contract, compliance_contract) =
        setup_chain(&mut chain, &admin, COMPLIANT_NATIONALITIES.to_vec());
    let token_contract =
        create_token_contract(&mut chain, &admin, compliance_contract, ir_contract);

    euroe::update_operator_single(&mut chain, &admin, euroe, UpdateOperator {
        operator: Address::Contract(token_contract),
        update:   OperatorUpdate::Add,
    });
    let holder = Account::new(HOLDER, DEFAULT_ACC_BALANCE);
    chain.create_account(holder.clone());
    identity_registry::register_nationalities(&mut chain, &admin, &ir_contract, vec![(
        Address::Account(holder.address),
        COMPLIANT_NATIONALITIES[1],
    )]);
    security_sft_rewards_client::mint(&mut chain, &admin, &token_contract, &MintParams {
        owners:   vec![MintParam {
            amount:  TokenAmountU64(10),
            address: holder.address,
        }],
        token_id: SFT_TOKEN_ID,
    });
    assert_eq!(
        security_sft_rewards_client::balance_of(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![
                    BalanceOfQuery {
                        address:  Address::Account(holder.address),
                        token_id: SFT_TOKEN_ID,
                    },
                    BalanceOfQuery {
                        address:  Address::Account(holder.address),
                        token_id: MIN_REWARD_TOKEN_ID,
                    },
                ],
            }
        )
        .expect("balance of"),
        BalanceOfQueryResponse(vec![TokenAmountU64(10), TokenAmountU64(10),])
    );

    security_sft_rewards_client::transfer_add_reward(
        &mut chain,
        &admin,
        token_contract,
        &TransferAddRewardParams {
            token_id:              MIN_REWARD_TOKEN_ID,
            reward_token_id:       to_token_id_vec(TokenIdUnit()),
            reward_token_contract: euroe,
            data:                  AddRewardContractParam {
                metadata_url: ContractMetadataUrl {
                    url:  "reward1.example.com".to_string(),
                    hash: None,
                },
                rate:         Rate::new(10, 1).unwrap(),
            },
        },
    );

    assert_eq!(
        security_sft_rewards_client::balance_of(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![
                    BalanceOfQuery {
                        address:  Address::Account(holder.address),
                        token_id: SFT_TOKEN_ID,
                    },
                    BalanceOfQuery {
                        address:  Address::Account(holder.address),
                        token_id: MIN_REWARD_TOKEN_ID,
                    },
                    BalanceOfQuery {
                        address:  Address::Account(holder.address),
                        token_id: MIN_REWARD_TOKEN_ID.plus_one(),
                    }
                ],
            }
        )
        .expect("balance of"),
        BalanceOfQueryResponse(vec![
            TokenAmountU64(10),
            TokenAmountU64(10),
            TokenAmountU64(0)
        ])
    );

    assert_eq!(
        euroe::balance_of_single(&mut chain, &admin, euroe, Address::Contract(token_contract)),
        TokenAmountU64(100)
    );

    security_sft_rewards_client::claim_rewards(
        &mut chain,
        &holder,
        token_contract,
        &ClaimRewardsParams {
            owner:  Receiver::Account(holder.address),
            claims: vec![ClaimRewardsParam {
                token_id: MIN_REWARD_TOKEN_ID,
                amount:   TokenAmountU64(10),
            }],
        },
    );
    assert_eq!(
        security_sft_rewards_client::balance_of(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![
                    BalanceOfQuery {
                        address:  Address::Account(holder.address),
                        token_id: SFT_TOKEN_ID,
                    },
                    BalanceOfQuery {
                        address:  Address::Account(holder.address),
                        token_id: MIN_REWARD_TOKEN_ID,
                    },
                    BalanceOfQuery {
                        address:  Address::Account(holder.address),
                        token_id: MIN_REWARD_TOKEN_ID.plus_one(),
                    }
                ],
            }
        )
        .expect("balance of"),
        BalanceOfQueryResponse(vec![
            TokenAmountU64(10),
            TokenAmountU64(0),
            TokenAmountU64(10)
        ])
    );
    assert_eq!(
        euroe::balance_of_single(&mut chain, &admin, euroe, Address::Contract(token_contract)),
        TokenAmountU64(0)
    );
    assert_eq!(
        euroe::balance_of_single(&mut chain, &admin, euroe, Address::Account(holder.address)),
        TokenAmountU64(100)
    );
}

#[test]
fn init() {
    let admin = Account::new(AccountAddress([0; 32]), DEFAULT_ACC_BALANCE);
    let mut chain = Chain::new();
    let (_, ir_contract, compliance_contract) =
        setup_chain(&mut chain, &admin, COMPLIANT_NATIONALITIES.to_vec());
    let token_contract =
        create_token_contract(&mut chain, &admin, compliance_contract, ir_contract);

    assert_eq!(
        security_sft_rewards_client::identity_registry(&mut chain, &admin, token_contract),
        ir_contract
    );
    assert_eq!(
        security_sft_rewards_client::compliance(&mut chain, &admin, token_contract),
        compliance_contract
    );

    let ir_contract = ContractAddress {
        index:    100,
        subindex: 0,
    };
    let compliance_contract = ContractAddress {
        index:    101,
        subindex: 0,
    };
    security_sft_rewards_client::set_identity_registry(
        &mut chain,
        &admin,
        token_contract,
        &ir_contract,
    );
    security_sft_rewards_client::set_compliance(
        &mut chain,
        &admin,
        token_contract,
        &compliance_contract,
    );

    assert_eq!(
        security_sft_rewards_client::identity_registry(&mut chain, &admin, token_contract),
        ir_contract
    );
    assert_eq!(
        security_sft_rewards_client::compliance(&mut chain, &admin, token_contract),
        compliance_contract
    );
}

#[test]
fn mint() {
    let admin = Account::new(AccountAddress([0; 32]), DEFAULT_ACC_BALANCE);
    let mut chain = Chain::new();
    let (_, ir_contract, compliance_contract) =
        setup_chain(&mut chain, &admin, COMPLIANT_NATIONALITIES.to_vec());
    let agent_mint = Account::new(AGENT_MINT, DEFAULT_ACC_BALANCE);
    let non_agent_mint = Account::new(AccountAddress([99; 32]), DEFAULT_ACC_BALANCE);
    let non_agent = Account::new(AccountAddress([98; 32]), DEFAULT_ACC_BALANCE);
    chain.create_account(agent_mint.clone());
    chain.create_account(non_agent_mint.clone());
    chain.create_account(non_agent.clone());

    let token_contract =
        create_token_contract(&mut chain, &admin, compliance_contract, ir_contract);
    security_sft_rewards_client::add_agent(&mut chain, &admin, token_contract, &AgentWithRoles {
        address: Address::Account(agent_mint.address),
        roles:   vec![security_sft_rewards::types::AgentRole::Mint],
    });
    security_sft_rewards_client::add_agent(&mut chain, &admin, token_contract, &AgentWithRoles {
        address: Address::Account(non_agent_mint.address),
        roles:   vec![security_sft_rewards::types::AgentRole::Pause],
    });

    let holder = Account::new(HOLDER, DEFAULT_ACC_BALANCE);
    chain.create_account(holder.clone());
    identity_registry::register_nationalities(&mut chain, &admin, &ir_contract, vec![(
        Address::Account(holder.address),
        COMPLIANT_NATIONALITIES[1],
    )]);

    security_sft_rewards_client::mint_raw(
        &mut chain,
        &non_agent_mint,
        &token_contract,
        &MintParams {
            owners:   vec![MintParam {
                amount:  TokenAmountU64(10),
                address: holder.address,
            }],
            token_id: SFT_TOKEN_ID,
        },
    )
    .expect_err("non-agent-mint minted");
    security_sft_rewards_client::mint_raw(&mut chain, &non_agent, &token_contract, &MintParams {
        owners:   vec![MintParam {
            amount:  TokenAmountU64(10),
            address: holder.address,
        }],
        token_id: SFT_TOKEN_ID,
    })
    .expect_err("non-agent minted");
    security_sft_rewards_client::mint_raw(&mut chain, &agent_mint, &token_contract, &MintParams {
        owners:   vec![MintParam {
            amount:  TokenAmountU64(10),
            address: holder.address,
        }],
        token_id: MIN_REWARD_TOKEN_ID,
    })
    .expect_err("minted reward token id");
    security_sft_rewards_client::mint_raw(&mut chain, &agent_mint, &token_contract, &MintParams {
        owners:   vec![MintParam {
            amount:  TokenAmountU64(10),
            address: holder.address,
        }],
        token_id: TokenIdU32(100),
    })
    .expect_err("minted non-tracked token id");
    security_sft_rewards_client::mint(&mut chain, &agent_mint, &token_contract, &MintParams {
        owners:   vec![MintParam {
            amount:  TokenAmountU64(10),
            address: holder.address,
        }],
        token_id: SFT_TOKEN_ID,
    });
    assert_eq!(
        security_sft_rewards_client::balance_of(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![
                    BalanceOfQuery {
                        address:  Address::Account(holder.address),
                        token_id: SFT_TOKEN_ID,
                    },
                    BalanceOfQuery {
                        address:  Address::Account(holder.address),
                        token_id: MIN_REWARD_TOKEN_ID,
                    },
                ],
            }
        )
        .expect("balance of"),
        BalanceOfQueryResponse(vec![TokenAmountU64(10), TokenAmountU64(10),])
    );
}

fn create_token_contract(
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
            url:  SFT_METADATA_URL.to_string(),
        },
        sponsors:                  None,
        blank_reward_metadata_url: ContractMetadataUrl {
            hash: None,
            url:  MIN_REWARD_METADATA_URL.to_string(),
        },
        tracked_token_id:          SFT_TOKEN_ID,
        min_reward_token_id:       MIN_REWARD_TOKEN_ID,
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
    security_sft_rewards_client::deploy_module(chain, admin);

    let euroe = euroe::init(chain, admin).contract_address;
    euroe::grant_role(chain, admin, euroe, &euroe::RoleTypes {
        mintrole:  Address::Account(admin.address),
        burnrole:  Address::Account(admin.address),
        blockrole: Address::Account(admin.address),
        pauserole: Address::Account(admin.address),
        adminrole: Address::Account(admin.address),
    });
    euroe::mint(chain, admin, euroe, &euroe::MintParams {
        owner:  Address::Account(admin.address),
        amount: TokenAmountU64(400_000_000),
    });
    let ir_contract = identity_registry::init(chain, admin).contract_address;
    let compliance_contract =
        compliance::init_all(chain, admin, ir_contract, compliant_nationalities).contract_address;
    (euroe, ir_contract, compliance_contract)
}
