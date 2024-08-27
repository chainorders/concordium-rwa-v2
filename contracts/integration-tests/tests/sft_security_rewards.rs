pub mod utils;

use cis2_conversions::to_token_id_vec;
use concordium_cis2::{
    BalanceOfQuery, BalanceOfQueryResponse, OperatorUpdate, Receiver, TokenAmountU64, TokenIdU32,
    TokenIdUnit, UpdateOperator,
};
use concordium_protocols::concordium_cis2_ext::PlusSubOne;
use concordium_rwa_market::types::Rate;
use concordium_smart_contract_testing::*;
use security_sft_rewards::rewards::{
    AddRewardContractParam, ClaimRewardsParam, ClaimRewardsParams, TransferAddRewardParams,
};
use security_sft_rewards::types::{ContractMetadataUrl, InitParam, MintParam};
use utils::*;

pub const DEFAULT_ACC_BALANCE: Amount = Amount {
    micro_ccd: 1_000_000_000_u64,
};

#[test]
fn normal_reward_distribution() {
    const SFT_TOKEN_ID: TokenIdU32 = TokenIdU32(0);
    const MIN_REWARD_TOKEN_ID: TokenIdU32 = TokenIdU32(1);
    let compliant_nationalities = ["IN".to_owned(), "US".to_owned()];
    let admin = Account::new(AccountAddress([0; 32]), DEFAULT_ACC_BALANCE);
    let seller = Account::new(AccountAddress([2; 32]), DEFAULT_ACC_BALANCE);

    let mut chain = Chain::new();
    chain.create_account(admin.clone());
    chain.create_account(seller.clone());

    euroe::deploy_module(&mut chain, &admin);
    identity_registry::deploy_module(&mut chain, &admin);
    compliance::deploy_module(&mut chain, &admin);
    sft_security::deploy_module(&mut chain, &admin);

    let euroe = euroe::init(&mut chain, &admin).contract_address;
    euroe::grant_role(&mut chain, &admin, euroe, &euroe::RoleTypes {
        mintrole:  Address::Account(admin.address),
        burnrole:  Address::Account(admin.address),
        blockrole: Address::Account(admin.address),
        pauserole: Address::Account(admin.address),
        adminrole: Address::Account(admin.address),
    });
    euroe::mint(&mut chain, &admin, euroe, &euroe::MintParams {
        owner:  Address::Account(admin.address),
        amount: TokenAmountU64(400_000_000),
    });
    let ir_contract = identity_registry::init(&mut chain, &admin).contract_address;
    let compliance_contract = compliance::init_all(
        &mut chain,
        &admin,
        ir_contract,
        compliant_nationalities.to_vec(),
    )
    .contract_address;

    let token_contract = sft_security::init(&mut chain, &admin, &InitParam {
        compliance:                compliance_contract,
        identity_registry:         ir_contract,
        metadata_url:              ContractMetadataUrl {
            hash: None,
            url:  "example.com".to_string(),
        },
        sponsors:                  vec![],
        blank_reward_metadata_url: ContractMetadataUrl {
            hash: None,
            url:  "blank_reward.example.com".to_string(),
        },
        tracked_token_id:          SFT_TOKEN_ID,
        min_reward_token_id:       MIN_REWARD_TOKEN_ID,
    })
    .contract_address;
    euroe::update_operator_single(&mut chain, &admin, euroe, UpdateOperator {
        operator: Address::Contract(token_contract),
        update:   OperatorUpdate::Add,
    });

    identity_registry::register_nationalities(&mut chain, &admin, &ir_contract, vec![(
        Address::Account(seller.address),
        compliant_nationalities[1].clone(),
    )]);
    sft_security::mint(&mut chain, &admin, &token_contract, &MintParam {
        amount:   TokenAmountU64(10),
        owner:    Receiver::Account(seller.address),
        token_id: SFT_TOKEN_ID,
    });
    assert_eq!(
        sft_security::balance_of(
            &mut chain,
            &seller,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![
                    BalanceOfQuery {
                        address:  Address::Account(seller.address),
                        token_id: SFT_TOKEN_ID,
                    },
                    BalanceOfQuery {
                        address:  Address::Account(seller.address),
                        token_id: MIN_REWARD_TOKEN_ID,
                    },
                ],
            }
        ),
        BalanceOfQueryResponse(vec![TokenAmountU64(10), TokenAmountU64(10),])
    );

    sft_security::transfer_add_reward(
        &mut chain,
        &admin,
        token_contract,
        &TransferAddRewardParams {
            token_id:       to_token_id_vec(TokenIdUnit()),
            token_contract: euroe,
            data:           AddRewardContractParam {
                metadata_url: ContractMetadataUrl {
                    url:  "reward1.example.com".to_string(),
                    hash: None,
                },
                rate:         Rate::new(10, 1).unwrap(),
            },
        },
    );

    assert_eq!(
        sft_security::balance_of(
            &mut chain,
            &seller,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![
                    BalanceOfQuery {
                        address:  Address::Account(seller.address),
                        token_id: SFT_TOKEN_ID,
                    },
                    BalanceOfQuery {
                        address:  Address::Account(seller.address),
                        token_id: MIN_REWARD_TOKEN_ID,
                    },
                    BalanceOfQuery {
                        address:  Address::Account(seller.address),
                        token_id: MIN_REWARD_TOKEN_ID.plus_one(),
                    }
                ],
            }
        ),
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

    sft_security::claim_rewards(&mut chain, &seller, token_contract, &ClaimRewardsParams {
        owner:  Receiver::Account(seller.address),
        claims: vec![ClaimRewardsParam {
            token_id: MIN_REWARD_TOKEN_ID,
            amount:   TokenAmountU64(10),
        }],
    });
    assert_eq!(
        sft_security::balance_of(
            &mut chain,
            &seller,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![
                    BalanceOfQuery {
                        address:  Address::Account(seller.address),
                        token_id: SFT_TOKEN_ID,
                    },
                    BalanceOfQuery {
                        address:  Address::Account(seller.address),
                        token_id: MIN_REWARD_TOKEN_ID,
                    },
                    BalanceOfQuery {
                        address:  Address::Account(seller.address),
                        token_id: MIN_REWARD_TOKEN_ID.plus_one(),
                    }
                ],
            }
        ),
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
        euroe::balance_of_single(&mut chain, &admin, euroe, Address::Account(seller.address)),
        TokenAmountU64(100)
    );
}
