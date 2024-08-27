use async_trait::async_trait;
use chrono::{DateTime, Utc};
use concordium_rust_sdk::types::smart_contracts::{
    ContractEvent, ModuleReference, OwnedContractName,
};
use concordium_rust_sdk::types::ContractAddress;
use concordium_rwa_backend_shared::db::{DbConn, DbPool};
use concordium_rwa_identity_registry::event::Event;
use log::debug;

use super::db::{self};
use crate::txn_listener::{EventsProcessor, ProcessorError};

/// `RwaIdentityRegistryProcessor` is a struct that processes events for the
/// rwa-identity-registry contract. It maintains a connection to a MongoDB
/// database and contains the module reference and name of the contract.
pub struct RwaIdentityRegistryProcessor {
    /// Module reference of the contract.
    pub module_ref:    ModuleReference,
    pub contract_name: OwnedContractName,
    pub pool:          DbPool,
}

#[async_trait]
impl EventsProcessor for RwaIdentityRegistryProcessor {
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
        process_events(&mut conn, Utc::now(), contract, events)?;
        Ok(events.len() as u64)
    }
}

/// Processes the events of the rwa-identity-registry contract.
///
/// # Arguments
///
/// * `conn` - A mutable reference to the `DbConn` connection.
/// * `now` - The current time as a `DateTime<Utc>`.
/// * `contract` - A reference to the `ContractAddress` of the contract whose
///   events are to be processed.
/// * `events` - A slice of `ContractEvent`s to be processed.
///
/// # Returns
///
/// * A `Result` indicating the success or failure of the operation.
pub fn process_events(
    conn: &mut DbConn,
    now: DateTime<Utc>,
    contract: &ContractAddress,
    events: &[ContractEvent],
) -> Result<(), ProcessorError> {
    for event in events {
        let parsed_event = event.parse::<Event>()?;
        debug!(
            "Processing event for contract: {}/{}",
            contract.index, contract.subindex
        );
        debug!("Event details: {:#?}", parsed_event);

        match parsed_event {
            Event::AgentAdded(e) => {
                db::insert_agent(conn, db::Agent::new(e.agent, now, contract))?;
            }
            Event::AgentRemoved(e) => {
                db::remove_agent(conn, contract, &e.agent)?;
            }
            Event::IdentityRegistered(e) => {
                db::insert_identity(conn, db::Identity::new(&e.address, now, contract))?;
            }
            Event::IdentityRemoved(e) => {
                db::remove_identity(conn, &e.address)?;
            }
            Event::IssuerAdded(e) => {
                db::insert_issuer(conn, db::Issuer::new(&e.issuer, now, contract))?;
            }
            Event::IssuerRemoved(e) => {
                db::remove_issuer(conn, contract, &e.issuer)?;
            }
        }
    }

    Ok(())
}
