use async_trait::async_trait;
use chrono::{DateTime, Utc};
use concordium_rust_sdk::cis2::{self};
use concordium_rust_sdk::types::smart_contracts::{
    ContractEvent, ModuleReference, OwnedContractName,
};
use concordium_rust_sdk::types::ContractAddress;
use concordium_rwa_backend_shared::db::{DbConn, DbPool};
use concordium_rwa_market::event::{Event, PaymentAmount, PaymentTokenUId};
use diesel::Connection;
use log::debug;
use num_bigint::BigUint;
use num_traits::Zero;

use super::db;
use crate::txn_listener::{EventsProcessor, ProcessorError};

pub struct RwaMarketProcessor {
    pub pool:          DbPool,
    /// Module reference of the contract.
    pub module_ref:    ModuleReference,
    /// Name of the contract.
    pub contract_name: OwnedContractName,
}

#[async_trait]
impl EventsProcessor for RwaMarketProcessor {
    /// Returns the name of the contract this processor is responsible for.
    ///
    /// # Returns
    ///
    /// * A reference to the `OwnedContractName` of the contract.
    fn contract_name(&self) -> &OwnedContractName { &self.contract_name }

    /// Returns the module reference of the contract this processor is
    /// responsible for.
    ///
    /// # Returns
    ///
    /// * A reference to the `ModuleReference` of the contract.
    fn module_ref(&self) -> &ModuleReference { &self.module_ref }

    /// Processes the events of the rwa-identity-registry contract.
    ///
    /// # Arguments
    ///
    /// * `contract` - A reference to the `ContractAddress` of the contract
    ///   whose events are to be processed.
    /// * `events` - A slice of `ContractEvent`s to be processed.
    ///
    /// # Returns
    ///
    /// * A Result indicating the success or failure of the operation.
    async fn process_events(
        &mut self,
        contract: &ContractAddress,
        events: &[ContractEvent],
    ) -> Result<u64, ProcessorError> {
        let mut conn = self.pool.get()?;
        let count = process_events(&mut conn, Utc::now(), contract, events)?;
        Ok(count as u64)
    }
}

pub fn process_events(
    conn: &mut DbConn,
    _now: DateTime<Utc>,
    contract: &ContractAddress,
    events: &[ContractEvent],
) -> Result<usize, ProcessorError> {
    let mut process_events_count: usize = 0;
    for event in events {
        let parsed_event = event.parse::<Event>()?;
        debug!("Event: {}/{}", contract.index, contract.subindex);
        debug!("{:#?}", parsed_event);

        match parsed_event {
            Event::Deposited(e) => {
                let token = db::MarketToken::new(
                    *contract,
                    e.token_id.contract,
                    e.token_id.id.to_string().parse()?,
                    e.owner,
                    e.amount.0.into(),
                    cis2::TokenAmount(BigUint::zero()),
                );
                db::insert_or_inc_unlisted_supply(conn, &token)?;

                debug!("Deposited Market Token");
                debug!("{:#?}", token);
                process_events_count += 1;
            }
            Event::Withdraw(e) => {
                db::update_dec_unlisted_supply(
                    conn,
                    contract,
                    &e.token_id.contract,
                    &e.token_id.id.to_string().parse()?,
                    e.owner,
                    e.amount.0.into(),
                )?;

                debug!("Withdraw Market Token");
                process_events_count += 1;
            }
            Event::Listed(e) => {
                db::update_unlisted_to_listed_supply(
                    conn,
                    contract,
                    &e.token_id.contract,
                    &e.token_id.id.to_string().parse()?,
                    e.owner,
                    e.supply.0.into(),
                )?;

                debug!("Listed Market Token");
                process_events_count += 1;
            }
            Event::DeListed(e) => {
                db::update_listed_all_to_unlisted_supply(
                    conn,
                    contract,
                    &e.token_id.contract,
                    &e.token_id.id.to_string().parse()?,
                    e.owner,
                )?;

                debug!("DeListed Market Token");
                process_events_count += 1;
            }
            Event::Exchanged(e) => {
                conn.transaction(|conn| {
                    db::update_dec_listed_supply(
                        conn,
                        contract,
                        &e.buy_token_id.contract,
                        &e.buy_token_id.id.to_string().parse()?,
                        e.buy_token_owner,
                        e.buy_amount.0.into(),
                    )?;

                    if let (PaymentTokenUId::Cis2(token_id), PaymentAmount::Cis2(amount)) =
                        (e.pay_token_id, e.pay_amount)
                    {
                        // If the amount to buy has been paid in another cis2 token
                        db::update_dec_unlisted_supply(
                            conn,
                            contract,
                            &token_id.contract,
                            &token_id.id.to_string().parse()?,
                            e.pay_token_owner,
                            amount.0.into(),
                        )?;
                    };

                    Ok::<_, ProcessorError>(())
                })?;

                debug!("Exchanged Market Token");
                process_events_count += 1;
            }
        }
    }

    Ok(process_events_count)
}
