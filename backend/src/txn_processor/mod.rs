//! This module contains the transaction processor for the Concordium RWA
//! backend. It includes the definition of the database module, as well as the
//! modules for the RWA identity registry, RWA market, RWA security NFT, and RWA
//! security SFT. It also defines the listener and API configuration struct, as
//! well as the contracts API configuration struct. The module provides
//! functions to run the contracts API server and listener, as well as to
//! generate the API client. It also includes helper functions to create the
//! listener, server routes, and service for the contracts API.

pub mod rwa_identity_registry;
pub mod rwa_market;
pub mod rwa_security_cis2;

use crate::{
    shared::db::DbPool,
    txn_listener::{EventsProcessor, TransactionsListener},
};
use clap::Parser;
use concordium_rust_sdk::{
    types::smart_contracts::OwnedContractName,
    v2::{self, Endpoint},
};
use concordium_rwa_security_sft::types::NftTokenId;
use concordium_rwa_utils::cis2_types::{NftTokenAmount, SftTokenAmount, SftTokenId};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use futures::TryFutureExt;
use log::{debug, info};
use poem::{
    listener::TcpListener,
    middleware::{AddData, Cors},
    EndpointExt, Route, Server,
};
use poem_openapi::OpenApiService;
use rwa_identity_registry::processor::*;
use rwa_market::{api::*, processor::*};
use rwa_security_cis2::{api::Cis2Api, processor::RwaSecurityCIS2Processor};
use std::{io::Write, str::FromStr, sync::Arc, time::Duration};
use tokio::{spawn, sync::RwLock};

#[derive(Parser, Debug, Clone)]
pub struct ListenerConfig {
    /// Postgres Database Url
    #[clap(env, long)]
    pub database_url: String,
    #[clap(env, long)]
    pub db_pool_max_size: u32,
    /// The Concordium node URI.
    #[clap(env, long)]
    pub concordium_node_uri: String,
    /// The reference to the RWA identity registry module.
    #[clap(env, long)]
    pub rwa_identity_registry_module_ref: String,
    /// The reference to the RWA security NFT module.
    #[clap(env, long)]
    pub rwa_security_nft_module_ref: String,
    /// The reference to the RWA security SFT module.
    #[clap(env, long)]
    pub rwa_security_sft_module_ref: String,
    /// The reference to the RWA market module.
    #[clap(env, long)]
    pub rwa_market_module_ref: String,
    /// The starting block hash.
    #[clap(env, long)]
    pub default_block_height: u64,
    #[clap(env, long)]
    pub node_rate_limit: u64,
    #[clap(env, long)]
    pub node_rate_limit_duration_secs: u64,
    /// The name of the RWA security NFT contract.
    #[clap(env, long)]
    pub rwa_security_nft_contract_name: String,
    /// The name of the RWA security SFT contract.
    #[clap(env, long)]
    pub rwa_security_sft_contract_name: String,
    /// The name of the RWA identity registry contract.
    #[clap(env, long)]
    pub rwa_identity_registry_contract_name: String,
    /// The name of the RWA market contract.
    #[clap(env, long)]
    pub rwa_market_contract_name: String,
}

/// Configuration struct for the contracts API.
/// Configuration options for the Contracts API.
#[derive(Parser, Debug, Clone)]
pub struct ContractsApiConfig {
    /// Postgres Database Url
    #[clap(env, long)]
    pub database_url:                   String,
    #[clap(env, long)]
    pub db_pool_max_size:               u32,
    #[clap(env, long)]
    pub web_server_addr:                String,
    #[clap(env, long)]
    pub rwa_market_contract_name:       String,
    /// The name of the RWA security NFT contract.
    #[clap(env, long)]
    pub rwa_security_nft_contract_name: String,
    /// The name of the RWA security SFT contract.
    #[clap(env, long)]
    pub rwa_security_sft_contract_name: String,
}

pub async fn run_listener(config: ListenerConfig) -> anyhow::Result<()> {
    info!("Listener: Starting");
    debug!("{:#?}", config);

    let listener_handle = spawn(create_listener(config).and_then(TransactionsListener::listen));
    info!("Listener: Listening");

    let ret = listener_handle.await?;
    info!("Listener: Shutting down");

    ret
}

pub async fn run_api_server(config: ContractsApiConfig) -> anyhow::Result<()> {
    info!("Contract Api: Starting Server");
    debug!("{:#?}", config);
    let routes = create_server_routes(config.to_owned()).await?;
    let web_server_addr = config.web_server_addr.clone();
    let server_handle = spawn(Server::new(TcpListener::bind(web_server_addr)).run(routes));
    info!("Contract Api: Listening for web requests at {}...", config.web_server_addr);
    let ret = server_handle.await?.map_err(|e| e.into());
    info!("Contracts API: Shutting down");

    ret
}

/// Creates the listener for processing transactions.
async fn create_listener(config: ListenerConfig) -> anyhow::Result<TransactionsListener> {
    let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
    let pool: Pool<ConnectionManager<PgConnection>> =
        Pool::builder().max_size(config.db_pool_max_size).build(manager).unwrap();

    let processors: Vec<Arc<RwLock<dyn EventsProcessor>>> = vec![
        Arc::new(RwLock::new(RwaIdentityRegistryProcessor {
            module_ref:    config.rwa_identity_registry_module_ref.parse()?,
            contract_name: OwnedContractName::new(config.rwa_identity_registry_contract_name)?,
            pool:          pool.clone(),
        })),
        Arc::new(RwLock::new(RwaSecurityCIS2Processor::<NftTokenId, NftTokenAmount>::new(
            pool.clone(),
            config.rwa_security_nft_module_ref.parse()?,
            OwnedContractName::new(config.rwa_security_nft_contract_name)?,
        ))),
        Arc::new(RwLock::new(RwaSecurityCIS2Processor::<SftTokenId, SftTokenAmount>::new(
            pool.clone(),
            config.rwa_security_sft_module_ref.parse()?,
            OwnedContractName::new(config.rwa_security_sft_contract_name)?,
        ))),
        Arc::new(RwLock::new(RwaMarketProcessor {
            module_ref:    config.rwa_market_module_ref.parse()?,
            contract_name: OwnedContractName::new(config.rwa_market_contract_name)?,
            pool:          pool.clone(),
        })),
    ];

    let endpoint = Endpoint::from_str(&config.concordium_node_uri)?;
    let endpoint = endpoint.rate_limit(
        config.node_rate_limit,
        Duration::from_secs(config.node_rate_limit_duration_secs),
    );

    let listener = TransactionsListener::new(
        v2::Client::new(endpoint).await?,
        pool.clone(),
        processors,
        config.default_block_height.into(),
    )
    .await?;
    Ok(listener)
}

/// Creates the server routes for the contracts API.
async fn create_server_routes(config: ContractsApiConfig) -> anyhow::Result<impl poem::Endpoint> {
    let api_service = create_service();
    let ui = api_service.swagger_ui();
    let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
    let pool: DbPool = Pool::builder().max_size(config.db_pool_max_size).build(manager).unwrap();
    let routes = Route::new()
        .nest("/", api_service)
        .nest("/ui", ui)
        .with(AddData::new(pool))
        .with(Cors::new());

    Ok(routes)
}

/// Configuration struct for generating the API client.
#[derive(Parser, Debug, Clone)]
/// Configuration struct for OpenAPI.
pub struct OpenApiConfig {
    /// The output file path for the OpenAPI specification.
    #[clap(env, default_value = "processor-openapi-spec.json")]
    pub output: String,
}

/// Generates the API client based on the OpenAPI configuration.
pub async fn generate_open_api_specs(config: OpenApiConfig) -> anyhow::Result<()> {
    let api_service = create_service();
    let spec_json = api_service.spec();
    let mut file = std::fs::File::create(config.output)?;
    file.write_all(spec_json.as_bytes())?;
    Ok(())
}

/// Creates the service for the contracts API.
fn create_service() -> OpenApiService<(RwaMarketApi, Cis2Api), ()> {
    OpenApiService::new((RwaMarketApi, Cis2Api), "RWA Contracts API", "1.0.0")
}
