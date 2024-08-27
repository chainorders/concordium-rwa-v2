#![cfg(test)]

mod utils;

use concordium_cis2::{AdditionalData, BalanceOfQuery, TokenAmountU64, TokenIdUnit, Transfer};
use concordium_protocols::concordium_cis2_security::{
    AgentWithRoles, BurnParams, FreezeParam, FreezeParams, PauseParams,
};
use concordium_smart_contract_testing::*;
use security_sft_single::types::*;
use utils::{compliance, euroe, identity_registry, security_sft_single_client};

const TOKEN_ID: TokenIdUnit = TokenIdUnit();
const METADATA_URL: &str = "example.com";
const ADMIN: AccountAddress = AccountAddress([0; 32]);
const AGENT_MINT: AccountAddress = AccountAddress([1; 32]);
const HOLDER: AccountAddress = AccountAddress([2; 32]);
const HOLDER_2: AccountAddress = AccountAddress([3; 32]);
const HOLDER_3: AccountAddress = AccountAddress([4; 32]);
const AGENT_FORCED_TRANSFER: AccountAddress = AccountAddress([5; 32]);
const AGENT_FORCED_BURN: AccountAddress = AccountAddress([6; 32]);
const COMPLIANT_NATIONALITIES: [&str; 2] = ["IN", "US"];
const DEFAULT_ACC_BALANCE: Amount = Amount {
    micro_ccd: 1_000_000_000_u64,
};

#[test]
fn mint() {
    let admin = Account::new(ADMIN, DEFAULT_ACC_BALANCE);
    let mut chain = Chain::new();
    let (ir_contract, compliance_contract) =
        setup_chain(&mut chain, &admin, COMPLIANT_NATIONALITIES.to_vec());
    let agent_mint = Account::new(AGENT_MINT, DEFAULT_ACC_BALANCE);
    chain.create_account(agent_mint.clone());
    let non_agent_mint = Account::new(AccountAddress([99; 32]), DEFAULT_ACC_BALANCE);
    chain.create_account(non_agent_mint.clone());
    let non_agent = Account::new(AccountAddress([98; 32]), DEFAULT_ACC_BALANCE);
    chain.create_account(non_agent.clone());

    let token_contract =
        create_token_contract(&mut chain, &admin, compliance_contract, ir_contract);
    security_sft_single_client::add_agent(&mut chain, &admin, token_contract, &AgentWithRoles {
        address: Address::Account(agent_mint.address),
        roles:   vec![AgentRole::Mint],
    });
    security_sft_single_client::add_agent(&mut chain, &admin, token_contract, &AgentWithRoles {
        address: Address::Account(non_agent_mint.address),
        roles:   vec![AgentRole::Pause],
    });

    let holder = Account::new(HOLDER, DEFAULT_ACC_BALANCE);
    chain.create_account(holder.clone());
    identity_registry::register_nationalities(&mut chain, &admin, &ir_contract, vec![(
        Address::Account(holder.address),
        COMPLIANT_NATIONALITIES[1],
    )]);
    let holder_2 = Account::new(HOLDER_2, DEFAULT_ACC_BALANCE);
    chain.create_account(holder_2.clone());
    // Holder 2 is not registered with the Identity Registry

    security_sft_single_client::mint(&mut chain, &non_agent_mint, &token_contract, &MintParams {
        owners:   vec![MintParam {
            amount:  TokenAmountU64(10),
            address: holder.address,
        }],
        token_id: TOKEN_ID,
    })
    .expect_err("non-agent-mint minted");
    security_sft_single_client::mint(&mut chain, &non_agent, &token_contract, &MintParams {
        owners:   vec![MintParam {
            amount:  TokenAmountU64(10),
            address: holder.address,
        }],
        token_id: TOKEN_ID,
    })
    .expect_err("non-agent minted");
    security_sft_single_client::mint(&mut chain, &agent_mint, &token_contract, &MintParams {
        owners:   vec![MintParam {
            amount:  TokenAmountU64(10),
            address: holder_2.address,
        }],
        token_id: TOKEN_ID,
    })
    .expect_err("non-compliant holder minted");
    security_sft_single_client::mint(&mut chain, &agent_mint, &token_contract, &MintParams {
        owners:   vec![MintParam {
            amount:  TokenAmountU64(10),
            address: holder.address,
        }],
        token_id: TOKEN_ID,
    })
    .expect("should mint");
    assert_eq!(
        security_sft_single_client::balance_of(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  Address::Account(holder.address),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![TokenAmountU64(10)])
    );
}

#[test]
fn burn() {
    let admin = Account::new(ADMIN, DEFAULT_ACC_BALANCE);
    let mut chain = Chain::new();
    let (ir_contract, compliance_contract) =
        setup_chain(&mut chain, &admin, COMPLIANT_NATIONALITIES.to_vec());
    let token_contract =
        create_token_contract(&mut chain, &admin, compliance_contract, ir_contract);
    let holder = Account::new(HOLDER, DEFAULT_ACC_BALANCE);
    chain.create_account(holder.clone());
    let holder_2 = Account::new(HOLDER_2, DEFAULT_ACC_BALANCE);
    chain.create_account(holder_2.clone());

    identity_registry::register_nationalities(&mut chain, &admin, &ir_contract, vec![
        (Address::Account(holder.address), COMPLIANT_NATIONALITIES[1]),
        (
            Address::Account(holder_2.address),
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
    security_sft_single_client::freeze(&mut chain, &admin, token_contract, &FreezeParams {
        owner:  Address::Account(holder.address),
        tokens: vec![FreezeParam {
            token_id:     TOKEN_ID,
            token_amount: TokenAmountU64(10),
        }],
    })
    .expect("should freeze");
    security_sft_single_client::burn_raw(
        &mut chain,
        &holder_2,
        token_contract,
        &concordium_protocols::concordium_cis2_security::BurnParams(vec![Burn {
            amount:   TokenAmountU64(5),
            owner:    Address::Account(holder.address),
            token_id: TOKEN_ID,
        }]),
    )
    .expect_err("non-owner burned");
    security_sft_single_client::burn_raw(
        &mut chain,
        &holder,
        token_contract,
        &concordium_protocols::concordium_cis2_security::BurnParams(vec![Burn {
            amount:   TokenAmountU64(41),
            owner:    Address::Account(holder.address),
            token_id: TOKEN_ID,
        }]),
    )
    .expect_err("burned frozen");
    security_sft_single_client::burn(
        &mut chain,
        &holder,
        token_contract,
        &concordium_protocols::concordium_cis2_security::BurnParams(vec![Burn {
            amount:   TokenAmountU64(5),
            owner:    Address::Account(holder.address),
            token_id: TOKEN_ID,
        }]),
    );
    assert_eq!(
        security_sft_single_client::balance_of(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  Address::Account(holder.address),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![TokenAmountU64(45)])
    );
    security_sft_single_client::burn_raw(
        &mut chain,
        &holder,
        token_contract,
        &concordium_protocols::concordium_cis2_security::BurnParams(vec![Burn {
            amount:   TokenAmountU64(46),
            owner:    Address::Account(holder.address),
            token_id: TOKEN_ID,
        }]),
    )
    .expect_err("burned more than minted");
    security_sft_single_client::burn_raw(
        &mut chain,
        &holder_2,
        token_contract,
        &concordium_protocols::concordium_cis2_security::BurnParams(vec![Burn {
            amount:   TokenAmountU64(5),
            owner:    Address::Account(holder_2.address),
            token_id: TOKEN_ID,
        }]),
    )
    .expect_err("non-existing holder burned");
    security_sft_single_client::un_freeze(&mut chain, &admin, token_contract, &FreezeParams {
        owner:  Address::Account(holder.address),
        tokens: vec![FreezeParam {
            token_id:     TOKEN_ID,
            token_amount: TokenAmountU64(9),
        }],
    })
    .expect("should unfreeze");
    security_sft_single_client::burn(
        &mut chain,
        &holder,
        token_contract,
        &concordium_protocols::concordium_cis2_security::BurnParams(vec![Burn {
            amount:   TokenAmountU64(41),
            owner:    Address::Account(holder.address),
            token_id: TOKEN_ID,
        }]),
    );
    assert_eq!(
        security_sft_single_client::balance_of(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  Address::Account(holder.address),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![TokenAmountU64(4)])
    );

    security_sft_single_client::pause(
        &mut chain,
        &admin,
        token_contract,
        &concordium_protocols::concordium_cis2_security::PauseParams {
            tokens: vec![TOKEN_ID],
        },
    )
    .expect("should pause");
    security_sft_single_client::burn_raw(
        &mut chain,
        &holder,
        token_contract,
        &concordium_protocols::concordium_cis2_security::BurnParams(vec![Burn {
            amount:   TokenAmountU64(1),
            owner:    Address::Account(holder.address),
            token_id: TOKEN_ID,
        }]),
    )
    .expect_err("burned paused token");
}

#[test]
fn forced_burn() {
    let admin = Account::new(ADMIN, DEFAULT_ACC_BALANCE);
    let mut chain = Chain::new();
    let (ir_contract, compliance_contract) =
        setup_chain(&mut chain, &admin, COMPLIANT_NATIONALITIES.to_vec());
    let token_contract =
        create_token_contract(&mut chain, &admin, compliance_contract, ir_contract);

    let agent_forced_burn = Account::new(AGENT_FORCED_BURN, DEFAULT_ACC_BALANCE);
    chain.create_account(agent_forced_burn.clone());

    security_sft_single_client::add_agent(&mut chain, &admin, token_contract, &AgentWithRoles {
        address: Address::Account(agent_forced_burn.address),
        roles:   vec![AgentRole::ForcedBurn],
    });
    let holder = Account::new(HOLDER, DEFAULT_ACC_BALANCE);
    chain.create_account(holder.clone());
    let holder_2 = Account::new(HOLDER_2, DEFAULT_ACC_BALANCE);
    chain.create_account(holder_2.clone());

    identity_registry::register_nationalities(&mut chain, &admin, &ir_contract, vec![
        (Address::Account(holder.address), COMPLIANT_NATIONALITIES[1]),
        (
            Address::Account(holder_2.address),
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
    security_sft_single_client::freeze(&mut chain, &admin, token_contract, &FreezeParams {
        owner:  Address::Account(holder.address),
        tokens: vec![FreezeParam {
            token_id:     TOKEN_ID,
            token_amount: 10.into(),
        }],
    })
    .expect("should freeze");
    assert_eq!(
        security_sft_single_client::balance_of_frozen(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  holder.address.into(),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![10.into()])
    );
    assert_eq!(
        security_sft_single_client::balance_of_un_frozen(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  holder.address.into(),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![40.into()])
    );
    security_sft_single_client::forced_burn(
        &mut chain,
        &holder,
        token_contract,
        &BurnParams(vec![Burn {
            token_id: TOKEN_ID,
            amount:   50.into(),
            owner:    holder.address.into(),
        }]),
    )
    .expect_err("non-agent forced burn");
    security_sft_single_client::forced_burn(
        &mut chain,
        &agent_forced_burn,
        token_contract,
        &BurnParams(vec![Burn {
            token_id: TOKEN_ID,
            amount:   10.into(),
            owner:    holder.address.into(),
        }]),
    )
    .expect("should burn");
    assert_eq!(
        security_sft_single_client::balance_of_frozen(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  holder.address.into(),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![10.into()])
    );
    assert_eq!(
        security_sft_single_client::balance_of_un_frozen(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  holder.address.into(),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![30.into()])
    );

    security_sft_single_client::forced_burn(
        &mut chain,
        &agent_forced_burn,
        token_contract,
        &BurnParams(vec![Burn {
            token_id: TOKEN_ID,
            amount:   30.into(),
            owner:    holder.address.into(),
        }]),
    )
    .expect("should burn");
    assert_eq!(
        security_sft_single_client::balance_of_frozen(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  holder.address.into(),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![10.into()])
    );
    assert_eq!(
        security_sft_single_client::balance_of_un_frozen(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  holder.address.into(),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![0.into()])
    );
    security_sft_single_client::un_freeze(&mut chain, &admin, token_contract, &FreezeParams {
        owner:  holder.address.into(),
        tokens: vec![FreezeParam {
            token_id:     TOKEN_ID,
            token_amount: 10.into(),
        }],
    })
    .expect("should unfreeze");
    assert_eq!(
        security_sft_single_client::balance_of_frozen(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  holder.address.into(),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![0.into()])
    );
    assert_eq!(
        security_sft_single_client::balance_of_un_frozen(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  holder.address.into(),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![10.into()])
    );
    security_sft_single_client::forced_burn(
        &mut chain,
        &agent_forced_burn,
        token_contract,
        &BurnParams(vec![Burn {
            token_id: TOKEN_ID,
            amount:   11.into(),
            owner:    holder.address.into(),
        }]),
    )
    .expect_err("burned more than minted");

    security_sft_single_client::pause(&mut chain, &admin, token_contract, &PauseParams {
        tokens: vec![TOKEN_ID],
    })
    .expect("should pause");
}

#[test]
fn transfer() {
    let admin = Account::new(ADMIN, DEFAULT_ACC_BALANCE);
    let mut chain = Chain::new();
    let (ir_contract, compliance_contract) =
        setup_chain(&mut chain, &admin, COMPLIANT_NATIONALITIES.to_vec());
    let token_contract =
        create_token_contract(&mut chain, &admin, compliance_contract, ir_contract);
    let holder = Account::new(HOLDER, DEFAULT_ACC_BALANCE);
    chain.create_account(holder.clone());
    let holder_2 = Account::new(HOLDER_2, DEFAULT_ACC_BALANCE);
    chain.create_account(holder_2.clone());
    let holder_3 = Account::new(HOLDER_3, DEFAULT_ACC_BALANCE);
    chain.create_account(holder_2.clone());

    identity_registry::register_nationalities(&mut chain, &admin, &ir_contract, vec![
        (Address::Account(holder.address), COMPLIANT_NATIONALITIES[1]),
        (
            Address::Account(holder_2.address),
            COMPLIANT_NATIONALITIES[1],
        ),
    ]);
    // holder 3 is not registered with the Identity Registry

    security_sft_single_client::mint(&mut chain, &admin, &token_contract, &MintParams {
        owners:   vec![MintParam {
            amount:  50.into(),
            address: holder.address,
        }],
        token_id: TOKEN_ID,
    })
    .expect("should mint");

    security_sft_single_client::transfer_single(&mut chain, &holder, token_contract, Transfer {
        token_id: TOKEN_ID,
        amount:   51.into(),
        from:     Address::Account(holder.address),
        to:       holder_2.address.into(),
        data:     AdditionalData::empty(),
    })
    .expect_err("transferred more than minted");
    security_sft_single_client::transfer_single(&mut chain, &holder, token_contract, Transfer {
        token_id: TOKEN_ID,
        amount:   0.into(),
        from:     Address::Account(holder.address),
        to:       holder_2.address.into(),
        data:     AdditionalData::empty(),
    })
    .expect_err("transferred 0");
    security_sft_single_client::freeze(&mut chain, &admin, token_contract, &FreezeParams {
        owner:  Address::Account(holder.address),
        tokens: vec![FreezeParam {
            token_id:     TOKEN_ID,
            token_amount: 10.into(),
        }],
    })
    .expect("should freeze");
    security_sft_single_client::transfer_single(&mut chain, &holder, token_contract, Transfer {
        token_id: TOKEN_ID,
        amount:   41.into(),
        from:     Address::Account(holder.address),
        to:       holder_2.address.into(),
        data:     AdditionalData::empty(),
    })
    .expect_err("transferred frozen");
    security_sft_single_client::transfer_single(&mut chain, &holder, token_contract, Transfer {
        token_id: TOKEN_ID,
        amount:   1.into(),
        from:     Address::Account(holder.address),
        to:       holder_3.address.into(),
        data:     AdditionalData::empty(),
    })
    .expect_err("transferred to non-compliant");
    security_sft_single_client::transfer_single(&mut chain, &holder, token_contract, Transfer {
        token_id: TOKEN_ID,
        amount:   30.into(),
        from:     Address::Account(holder.address),
        to:       holder_2.address.into(),
        data:     AdditionalData::empty(),
    })
    .expect("should transfer");
    assert_eq!(
        security_sft_single_client::balance_of(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![
                    BalanceOfQuery {
                        address:  holder.address.into(),
                        token_id: TOKEN_ID,
                    },
                    BalanceOfQuery {
                        address:  holder_2.address.into(),
                        token_id: TOKEN_ID,
                    }
                ],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![20.into(), 30.into()])
    );

    security_sft_single_client::pause(
        &mut chain,
        &admin,
        token_contract,
        &concordium_protocols::concordium_cis2_security::PauseParams {
            tokens: vec![TOKEN_ID],
        },
    )
    .expect("should pause");
    security_sft_single_client::transfer_single(&mut chain, &holder, token_contract, Transfer {
        token_id: TOKEN_ID,
        amount:   1.into(),
        from:     Address::Account(holder.address),
        to:       holder_2.address.into(),
        data:     AdditionalData::empty(),
    })
    .expect_err("transferred paused token");
}

#[test]
fn forced_transfer() {
    let admin = Account::new(ADMIN, DEFAULT_ACC_BALANCE);
    let mut chain = Chain::new();
    let (ir_contract, compliance_contract) =
        setup_chain(&mut chain, &admin, COMPLIANT_NATIONALITIES.to_vec());
    let token_contract =
        create_token_contract(&mut chain, &admin, compliance_contract, ir_contract);
    let agent_forced_transfer = Account::new(AGENT_FORCED_TRANSFER, DEFAULT_ACC_BALANCE);
    chain.create_account(agent_forced_transfer.clone());

    security_sft_single_client::add_agent(&mut chain, &admin, token_contract, &AgentWithRoles {
        address: Address::Account(agent_forced_transfer.address),
        roles:   vec![AgentRole::ForcedTransfer],
    });
    let holder = Account::new(HOLDER, DEFAULT_ACC_BALANCE);
    chain.create_account(holder.clone());
    let holder_2 = Account::new(HOLDER_2, DEFAULT_ACC_BALANCE);
    chain.create_account(holder_2.clone());
    let holder_3 = Account::new(HOLDER_3, DEFAULT_ACC_BALANCE);
    chain.create_account(holder_2.clone());

    identity_registry::register_nationalities(&mut chain, &admin, &ir_contract, vec![
        (Address::Account(holder.address), COMPLIANT_NATIONALITIES[1]),
        (
            Address::Account(holder_2.address),
            COMPLIANT_NATIONALITIES[1],
        ),
    ]);
    // holder 3 is not registered with the Identity Registry

    security_sft_single_client::mint(&mut chain, &admin, &token_contract, &MintParams {
        owners:   vec![MintParam {
            amount:  50.into(),
            address: holder.address,
        }],
        token_id: TOKEN_ID,
    })
    .expect("should mint");
    security_sft_single_client::freeze(&mut chain, &admin, token_contract, &FreezeParams {
        owner:  Address::Account(holder.address),
        tokens: vec![FreezeParam {
            token_id:     TOKEN_ID,
            token_amount: 10.into(),
        }],
    })
    .expect("should freeze");
    assert_eq!(
        security_sft_single_client::balance_of_frozen(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  holder.address.into(),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![10.into()])
    );
    assert_eq!(
        security_sft_single_client::balance_of_un_frozen(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  holder.address.into(),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![40.into()])
    );

    security_sft_single_client::forced_transfer_single(
        &mut chain,
        &agent_forced_transfer,
        token_contract,
        Transfer {
            token_id: TOKEN_ID,
            amount:   51.into(),
            from:     holder.address.into(),
            to:       holder_2.address.into(),
            data:     AdditionalData::empty(),
        },
    )
    .expect_err("transferred more than minted");
    security_sft_single_client::forced_transfer_single(
        &mut chain,
        &agent_forced_transfer,
        token_contract,
        Transfer {
            token_id: TOKEN_ID,
            amount:   1.into(),
            from:     holder.address.into(),
            to:       holder_3.address.into(),
            data:     AdditionalData::empty(),
        },
    )
    .expect_err("transferred to non-compliant");
    security_sft_single_client::forced_transfer_single(
        &mut chain,
        &agent_forced_transfer,
        token_contract,
        Transfer {
            token_id: TOKEN_ID,
            amount:   1.into(),
            from:     holder.address.into(),
            to:       holder_2.address.into(),
            data:     AdditionalData::empty(),
        },
    )
    .expect("should transfer");
    assert_eq!(
        security_sft_single_client::balance_of(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![
                    BalanceOfQuery {
                        address:  holder.address.into(),
                        token_id: TOKEN_ID,
                    },
                    BalanceOfQuery {
                        address:  holder_2.address.into(),
                        token_id: TOKEN_ID,
                    }
                ],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![49.into(), 1.into()])
    );
    assert_eq!(
        security_sft_single_client::balance_of_frozen(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  holder.address.into(),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![10.into()])
    );
    assert_eq!(
        security_sft_single_client::balance_of_un_frozen(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  holder.address.into(),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![39.into()])
    );

    security_sft_single_client::forced_transfer_single(
        &mut chain,
        &agent_forced_transfer,
        token_contract,
        Transfer {
            token_id: TOKEN_ID,
            amount:   49.into(),
            from:     holder.address.into(),
            to:       holder_2.address.into(),
            data:     AdditionalData::empty(),
        },
    )
    .expect("should transfer frozen");
    assert_eq!(
        security_sft_single_client::balance_of_frozen(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  holder.address.into(),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![0.into()])
    );
    assert_eq!(
        security_sft_single_client::balance_of_un_frozen(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![BalanceOfQuery {
                    address:  holder.address.into(),
                    token_id: TOKEN_ID,
                },],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![0.into()])
    );
    assert_eq!(
        security_sft_single_client::balance_of(
            &mut chain,
            &holder,
            token_contract,
            &concordium_cis2::BalanceOfQueryParams {
                queries: vec![
                    BalanceOfQuery {
                        address:  holder.address.into(),
                        token_id: TOKEN_ID,
                    },
                    BalanceOfQuery {
                        address:  holder_2.address.into(),
                        token_id: TOKEN_ID,
                    }
                ],
            }
        )
        .unwrap(),
        concordium_cis2::BalanceOfQueryResponse(vec![0.into(), 50.into()])
    );

    security_sft_single_client::pause(&mut chain, &admin, token_contract, &PauseParams {
        tokens: vec![TOKEN_ID],
    })
    .expect("should pause");
    security_sft_single_client::forced_transfer_single(
        &mut chain,
        &agent_forced_transfer,
        token_contract,
        Transfer {
            token_id: TOKEN_ID,
            amount:   1.into(),
            from:     holder_2.address.into(),
            to:       holder.address.into(),
            data:     AdditionalData::empty(),
        },
    )
    .expect_err("transferred paused token");
}

fn create_token_contract(
    chain: &mut Chain,
    admin: &Account,
    compliance_contract: ContractAddress,
    ir_contract: ContractAddress,
) -> ContractAddress {
    security_sft_single_client::init(chain, admin, &InitParam {
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
) -> (ContractAddress, ContractAddress) {
    chain.create_account(admin.clone());

    euroe::deploy_module(chain, admin);
    identity_registry::deploy_module(chain, admin);
    compliance::deploy_module(chain, admin);
    security_sft_single_client::deploy_module(chain, admin);
    let ir_contract = identity_registry::init(chain, admin).contract_address;
    let compliance_contract =
        compliance::init_all(chain, admin, ir_contract, compliant_nationalities).contract_address;

    (ir_contract, compliance_contract)
}
