use core::fmt;

use chrono::{DateTime, Utc};
use concordium_cis2::{
    BurnEvent, Cis2Event, IsTokenAmount, IsTokenId, MintEvent, OperatorUpdate, TokenMetadataEvent,
    TransferEvent, UpdateOperatorEvent,
};
use concordium_protocols::concordium_cis2_security::*;
use concordium_rust_sdk::base::contracts_common::{Cursor, Deserial, Serial};
use concordium_rust_sdk::base::smart_contracts::ContractEvent;
use concordium_rust_sdk::cis2;
use concordium_rust_sdk::types::ContractAddress;
use concordium_rwa_backend_shared::db::*;
use diesel::Connection;
use num_bigint::BigUint;
use num_traits::Zero;
use security_sft_rewards::types::Event;
use tracing::{debug, instrument};

use super::db;
use crate::txn_listener::listener::ProcessorError;

fn cis2<T, A>(
    conn: &mut DbConn,
    now: DateTime<Utc>,
    cis2_address: &ContractAddress,
    event: Cis2Event<T, A>,
) -> DbResult<()>
where
    T: IsTokenId,
    A: IsTokenAmount+Serial,
{
    match event {
        Cis2Event::Mint(MintEvent {
            token_id,
            owner,
            amount,
        }) => {
            let token_id = to_cis2_token_id(&token_id);
            let token_amount = to_cis2_token_amount(&amount);
            conn.transaction(|conn| {
                db::insert_holder_or_add_balance(
                    conn,
                    &db::TokenHolder::new(
                        cis2_address,
                        &token_id,
                        &owner,
                        &token_amount,
                        &cis2::TokenAmount(BigUint::zero()),
                        now,
                    ),
                )?;
                db::update_supply(conn, cis2_address, &token_id, &token_amount, true)?;
                DbResult::Ok(())
            })
        }
        Cis2Event::TokenMetadata(TokenMetadataEvent {
            token_id,
            metadata_url,
        }) => {
            let token_id = to_cis2_token_id(&token_id);
            db::insert_token_or_update_metadata(
                conn,
                &db::Token::new(
                    cis2_address,
                    &token_id,
                    false,
                    metadata_url.url,
                    metadata_url.hash,
                    &cis2::TokenAmount(BigUint::zero()),
                    now,
                ),
            )
        }
        Cis2Event::Burn(BurnEvent {
            token_id,
            owner,
            amount,
        }) => {
            let token_id = to_cis2_token_id(&token_id);
            let token_amount = to_cis2_token_amount(&amount);
            conn.transaction(|conn| {
                db::update_sub_balance(conn, cis2_address, &token_id, &owner, &token_amount)?;
                db::update_supply(conn, cis2_address, &token_id, &token_amount, false)?;
                DbResult::Ok(())
            })
        }
        Cis2Event::Transfer(TransferEvent {
            token_id,
            from,
            to,
            amount,
        }) => {
            let token_id = to_cis2_token_id(&token_id);
            let token_amount = to_cis2_token_amount(&amount);
            conn.transaction(|conn| {
                db::update_sub_balance(conn, cis2_address, &token_id, &from, &token_amount)?;
                db::insert_holder_or_add_balance(
                    conn,
                    &db::TokenHolder::new(
                        cis2_address,
                        &token_id,
                        &to,
                        &token_amount,
                        &cis2::TokenAmount(BigUint::zero()),
                        now,
                    ),
                )
            })
        }
        Cis2Event::UpdateOperator(UpdateOperatorEvent {
            owner,
            operator,
            update,
        }) => {
            let record = db::Operator::new(cis2_address, &owner, &operator);
            match update {
                OperatorUpdate::Add => db::insert_operator(conn, &record),
                OperatorUpdate::Remove => db::delete_operator(conn, &record),
            }
        }
    }
}

fn to_cis2_token_amount<A>(amount: &A) -> cis2::TokenAmount
where A: IsTokenAmount+Serial {
    let mut bytes = vec![];
    amount.serial(&mut bytes).unwrap();
    let mut cursor: Cursor<_> = Cursor::new(bytes);

    cis2::TokenAmount::deserial(&mut cursor).unwrap()
}

fn to_cis2_token_id<T>(token_id: &T) -> cis2::TokenId
where T: IsTokenId+Serial {
    let mut bytes = vec![];

    token_id.serial(&mut bytes).unwrap();
    let mut cursor: Cursor<_> = Cursor::new(bytes);

    cis2::TokenId::deserial(&mut cursor).unwrap()
}

#[instrument(skip_all, fields(contract = %cis2_address, events = events.len()))]
pub fn process_events<T, A, R>(
    conn: &mut DbConn,
    now: DateTime<Utc>,
    cis2_address: &ContractAddress,
    events: &[ContractEvent],
) -> Result<(), ProcessorError>
where
    T: IsTokenId+fmt::Debug,
    A: IsTokenAmount+fmt::Debug,
    R: Deserial+fmt::Debug,
{
    for event in events {
        let parsed_event = event.parse::<Event>().expect("Failed to parse event");
        debug!("Event: {}/{}", cis2_address.index, cis2_address.subindex);
        debug!("{:#?}", parsed_event);

        match parsed_event {
            Event::AgentAdded(AgentUpdatedEvent {
                agent,
                roles: _, // todo: add roles to the database
            }) => db::insert_agent(conn, db::Agent::new(agent, now, cis2_address))?,
            Event::AgentRemoved(AgentUpdatedEvent { agent, roles: _ }) => {
                db::remove_agent(conn, cis2_address, &agent)?
            }
            Event::ComplianceAdded(ComplianceAdded(compliance_contract)) => db::upsert_compliance(
                conn,
                &db::Compliance::new(cis2_address, &compliance_contract),
            )?,
            Event::IdentityRegistryAdded(IdentityRegistryAdded(identity_registry_contract)) => {
                db::upsert_identity_registry(
                    conn,
                    &db::IdentityRegistry::new(cis2_address, &identity_registry_contract),
                )?
            }
            Event::Paused(Paused { token_id }) => {
                db::update_token_paused(conn, cis2_address, &to_cis2_token_id(&token_id), true)?
            }
            Event::UnPaused(Paused { token_id }) => {
                db::update_token_paused(conn, cis2_address, &to_cis2_token_id(&token_id), false)?
            }
            Event::Recovered(RecoverEvent {
                lost_account,
                new_account,
            }) => {
                let updated_rows = conn.transaction(|conn| {
                    db::insert_recovery_record(
                        conn,
                        &db::RecoveryRecord::new(cis2_address, &lost_account, &new_account),
                    )?;
                    db::update_replace_holder(conn, cis2_address, &lost_account, &new_account)
                })?;
                debug!("account recovery, {} token ids updated", updated_rows);
            }
            Event::TokenFrozen(TokenFrozen {
                address,
                amount,
                token_id,
            }) => db::update_balance_frozen(
                conn,
                cis2_address,
                &to_cis2_token_id(&token_id),
                &address,
                &to_cis2_token_amount(&amount),
                true,
            )?,
            Event::TokenUnFrozen(TokenFrozen {
                address,
                amount,
                token_id,
            }) => db::update_balance_frozen(
                conn,
                cis2_address,
                &to_cis2_token_id(&token_id),
                &address,
                &to_cis2_token_amount(&amount),
                false,
            )?,
            Event::Cis2(e) => cis2(conn, now, cis2_address, e)?,
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use concordium_cis2::{TokenAmountU64, TokenAmountU8};
    use concordium_rust_sdk::cis2;
    use num_bigint::BigUint;
    use num_traits::FromPrimitive;

    use super::to_cis2_token_amount;

    #[test]
    fn token_amount_conversions() {
        let amount = to_cis2_token_amount(&TokenAmountU8(0));
        assert_eq!(amount, cis2::TokenAmount(BigUint::from_u8(0).unwrap()));
        assert_eq!(amount.to_string(), "0");

        let amount: u8 = 255;
        let token_amount = to_cis2_token_amount(&TokenAmountU8(amount));
        assert_eq!(
            token_amount,
            cis2::TokenAmount(BigUint::from_u8(amount).unwrap())
        );
        assert_eq!(token_amount.to_string(), "255");

        let amount: u64 = 255;
        let token_amount = to_cis2_token_amount(&TokenAmountU64(amount));
        assert_eq!(
            token_amount,
            cis2::TokenAmount(BigUint::from_u64(amount).unwrap())
        );
        assert_eq!(token_amount.to_string(), "255");
    }
}
