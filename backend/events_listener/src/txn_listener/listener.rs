use std::collections::BTreeMap;
use std::ops::AddAssign;
use std::sync::Arc;

use async_trait::async_trait;
use concordium_rust_sdk::types::smart_contracts::{
    ContractEvent, ModuleReference, OwnedContractName,
};
use concordium_rust_sdk::types::{
    AbsoluteBlockHeight, AccountTransactionDetails, AccountTransactionEffects, BlockItemSummary,
    BlockItemSummaryDetails, ContractAddress, ContractInitializedEvent, ContractTraceElement,
    InstanceUpdatedEvent,
};
use concordium_rust_sdk::v2::{self, FinalizedBlockInfo};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use futures::StreamExt;
use log::{debug, info, warn};
use tokio::sync::RwLock;

use super::db;

#[derive(Debug, thiserror::Error)]
pub enum ProcessorError {
    #[error("R2D2 pool Database error: {0}")]
    DatabasePoolError(#[from] r2d2::Error),
    #[error("Events Parse Error: {0}")]
    EventsParseError(#[from] concordium_rust_sdk::base::contracts_common::ParseError),
    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("token id vector parse error: {0}")]
    TokenIdParseError(#[from] concordium_rust_sdk::cis2::ParseTokenIdVecError),
}

/// `EventsProcessor` is a trait that defines the necessary methods for
/// processing events of a specific contract. It is designed to be implemented
/// by structs that handle the logic for processing events of a specific
/// contract.
#[async_trait]
pub trait EventsProcessor: Send+Sync {
    /// Returns the name of the contract this processor is responsible for.
    ///
    /// # Returns
    ///
    /// * A reference to the `OwnedContractName` of the contract.
    fn contract_name(&self) -> &OwnedContractName;

    /// Returns the module reference of the contract this processor is
    /// responsible for.
    ///
    /// # Returns
    ///
    /// * A reference to the `ModuleReference` of the contract.
    fn module_ref(&self) -> &ModuleReference;

    /// Checks if this processor is responsible for the given contract.
    ///
    /// # Arguments
    ///
    /// * `module_ref` - A reference to the `ModuleReference` of the contract to
    ///   check.
    /// * `contract_name` - A reference to the `OwnedContractName` of the
    ///   contract to check.
    ///
    /// # Returns
    ///
    /// * `true` if this processor is responsible for the given contract,
    ///   `false` otherwise.
    fn matches(&self, module_ref: &ModuleReference, contract_name: &OwnedContractName) -> bool {
        module_ref.eq(self.module_ref()) && contract_name.eq(self.contract_name())
    }

    /// Processes the events of a contract.
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
    ) -> Result<u64, ProcessorError>;
}

pub enum ProcessedBlockItem {
    Init(
        (
            ContractAddress,
            ModuleReference,
            OwnedContractName,
            Vec<ContractEvent>,
        ),
    ),
    Update(BTreeMap<ContractAddress, Vec<ContractEvent>>),
    WithNoEvents,
}

#[derive(Debug, thiserror::Error)]
pub enum ListenerError {
    #[error("Finalized Block Stream ended")]
    FinalizedBlockStreamEnded,
    #[error("R2D2 pool Database error: {0}")]
    DatabasePoolError(#[from] r2d2::Error),
    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("Concordium node query error: {0}")]
    QueryError(#[from] concordium_rust_sdk::endpoints::QueryError),
    #[error("Processor error: {0}")]
    ProcessorError(#[from] ProcessorError),
}

/// `TransactionsListener` is a struct that listens to transactions from a
/// Concordium node and processes them. It maintains a connection to the node
/// and a MongoDB database, and uses a set of processors to process the
/// transactions.
pub struct TransactionsListener {
    database:             Pool<ConnectionManager<PgConnection>>, // popstgres pool
    processors:           Vec<Arc<RwLock<dyn EventsProcessor>>>, /* Processors to process
                                                                  * transactions */
    default_block_height: AbsoluteBlockHeight, // Default block height to start from
    client:               v2::Client,
}

impl TransactionsListener {
    /// Constructs a new `TransactionsListener`.
    /// # Returns
    ///
    /// * A new `TransactionsListener`.
    pub fn new(
        client: v2::Client,
        pool: Pool<ConnectionManager<PgConnection>>,
        processors: Vec<Arc<RwLock<dyn EventsProcessor>>>,
        default_block_height: AbsoluteBlockHeight,
    ) -> Self {
        Self {
            client,
            database: pool,
            processors,
            default_block_height,
        }
    }

    /// Starts listening to transactions from the Concordium node.
    ///
    /// # Returns
    ///
    /// * A Result indicating the success or failure of the operation.
    pub async fn listen(mut self) -> Result<(), ListenerError> {
        let block_height = self.get_block_height().await?;
        let mut finalized_block_stream =
            self.client.get_finalized_blocks_from(block_height).await?;

        while let Some(block) = finalized_block_stream.next().await {
            self.process_block(&block).await?;
            log::debug!("Processed block {}", block.height.height);
        }

        Err(ListenerError::FinalizedBlockStreamEnded)
    }

    async fn get_block_height(&self) -> Result<AbsoluteBlockHeight, ListenerError> {
        let mut conn = self.database.get()?;
        let block_height =
            db::get_last_processed_block(&mut conn)?.unwrap_or(self.default_block_height);

        Ok(block_height)
    }

    /// Processes a block of transactions.
    ///
    /// # Arguments
    ///
    /// * `block` - The block of transactions to be processed.
    ///
    /// # Returns
    ///
    /// * A Result indicating the success or failure of the operation.
    async fn process_block(&mut self, block: &FinalizedBlockInfo) -> Result<u64, ListenerError> {
        let mut b_events_count = 0u64;
        let mut summaries = self
            .client
            .get_block_transaction_events(block.block_hash)
            .await?
            .response;

        while let Some(summary) = summaries
            .next()
            .await
            .transpose()
            .expect("error getting block item summary")
        {
            let events_count = self.process_block_item_summary(block, &summary).await?;
            b_events_count += events_count;
        }

        info!(
            "Processed block {}, events: {}",
            block.height.height, b_events_count
        );
        let mut conn = self.database.get()?;
        if b_events_count > 0 {
            db::update_last_processed_block(&mut conn, block)?;
        }
        Ok(b_events_count)
    }

    async fn process_block_item_summary(
        &mut self,
        block: &FinalizedBlockInfo,
        summary: &BlockItemSummary,
    ) -> Result<u64, ListenerError> {
        let BlockItemSummary {
            index,
            hash,
            details,
            ..
        } = summary;

        let summary = match details {
            BlockItemSummaryDetails::AccountTransaction(details) => {
                let AccountTransactionDetails { effects, .. } = details;
                match effects {
                    AccountTransactionEffects::ContractInitialized {
                        data:
                            ContractInitializedEvent {
                                address,
                                events,
                                init_name,
                                origin_ref,
                                ..
                            },
                    } => ProcessedBlockItem::Init((
                        *address,
                        *origin_ref,
                        init_name.clone(),
                        events.to_vec(),
                    )),
                    AccountTransactionEffects::ContractUpdateIssued { effects } => {
                        let mut updates = BTreeMap::<ContractAddress, Vec<ContractEvent>>::new();
                        for trace_event in effects {
                            let (address, events) = match trace_event {
                                ContractTraceElement::Updated {
                                    data:
                                        InstanceUpdatedEvent {
                                            address, events, ..
                                        },
                                } => (*address, events.clone()),
                                ContractTraceElement::Transferred { from, .. } => (*from, vec![]),
                                ContractTraceElement::Interrupted { address, events } => {
                                    (*address, events.clone())
                                }
                                ContractTraceElement::Resumed { address, .. } => (*address, vec![]),
                                ContractTraceElement::Upgraded { address, from, to } => {
                                    warn!(
                                        "NOT SUPPORTED: Contract: {} Upgrated from module: {} to \
                                         module: {}",
                                        address, from, to
                                    );
                                    (*address, vec![])
                                }
                            };

                            match updates.get_mut(&address) {
                                Some(existing_events) => existing_events.extend(events),
                                None => {
                                    updates.insert(address, events);
                                }
                            };
                        }

                        ProcessedBlockItem::Update(updates)
                    }
                    _ => ProcessedBlockItem::WithNoEvents,
                }
            }
            _ => ProcessedBlockItem::WithNoEvents,
        };

        let mut conn = self.database.get()?;
        let events_count = match summary {
            ProcessedBlockItem::WithNoEvents => 0u64,
            ProcessedBlockItem::Init((contract_address, module_ref, contract_name, events)) => {
                let processor = self.find_processor(&module_ref, &contract_name).await;
                match processor {
                    Some(processor) => {
                        db::add_contract(
                            &mut conn,
                            &contract_address,
                            &module_ref,
                            &contract_name,
                        )?;
                        info!(
                            "[{}/{}], hash:{hash} contract: {contract_address} added",
                            block.height.height, index.index,
                        );

                        let mut processor = processor.write().await;
                        debug!(
                            "[{}/{}], hash:{} processing init events, count: {}, processor: {}",
                            block.height.height,
                            index.index,
                            hash,
                            events.len(),
                            processor.contract_name(),
                        );
                        let events_count =
                            processor.process_events(&contract_address, &events).await?;
                        debug!(
                            "[{}/{}], hash:{} processed init events, count: {}, processor: {}",
                            block.height.height,
                            index.index,
                            hash,
                            events.len(),
                            processor.contract_name(),
                        );

                        events_count
                    }
                    None => 0u64,
                }
            }
            ProcessedBlockItem::Update(updates) => {
                let mut event_counts = 0;
                for (contract_address, events) in updates.into_iter() {
                    let contract = db::find_contract(&mut conn, &contract_address)?;
                    let processor = match contract {
                        Some((module_ref, contract_name)) => {
                            self.find_processor(&module_ref, &contract_name).await
                        }
                        None => None,
                    };
                    let contract_events_counts = match processor {
                        Some(processor) => {
                            let mut processor = processor.write().await;
                            debug!(
                                "[{}/{}], hash:{} processing update events, count: {}, processor: \
                                 {}",
                                block.height.height,
                                index.index,
                                hash,
                                events.len(),
                                processor.contract_name(),
                            );
                            // Processing Update events
                            let processed_count =
                                processor.process_events(&contract_address, &events).await?;
                            debug!(
                                "[{}/{}], hash:{} processed update events, count: {}, processor: \
                                 {}",
                                block.height.height,
                                index.index,
                                hash,
                                events.len(),
                                processor.contract_name(),
                            );
                            processed_count
                        }
                        None => 0u64,
                    };
                    event_counts.add_assign(contract_events_counts);
                }

                event_counts
            }
        };

        info!(
            "[{}/{}], hash:{} events count: {}",
            block.height.height, index.index, hash, events_count
        );
        Ok(events_count)
    }

    /// Finds a processor that matches the given origin reference and contract
    /// name.
    ///
    /// # Arguments
    ///
    /// * `origin_ref` - The origin reference to match.
    /// * `init_name` - The contract name to match.
    ///
    /// # Returns
    ///
    /// * An Option containing a reference to the matching processor, or None if
    ///   no match was found.
    async fn find_processor(
        &self,
        origin_ref: &ModuleReference,
        init_name: &OwnedContractName,
    ) -> Option<Arc<RwLock<dyn EventsProcessor>>> {
        for processor in self.processors.iter() {
            let matches = processor.read().await.matches(origin_ref, init_name);
            match matches {
                true => return Some(processor.clone()),
                false => continue,
            }
        }

        None
    }
}
