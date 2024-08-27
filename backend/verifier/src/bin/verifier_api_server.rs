use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Datelike;
use clap::Parser;
use concordium_rust_sdk::id::constants::AttributeKind;
use concordium_rust_sdk::id::id_proof_types::{
    AtomicStatement, AttributeInRangeStatement, RevealAttributeStatement,
};
use concordium_rust_sdk::id::types::AttributeTag;
use concordium_rust_sdk::types::{Address, ContractAddress, Energy, WalletAccount};
use concordium_rust_sdk::v2::{self, BlockIdentifier};
use concordium_rust_sdk::web3id::did::Network;
use concordium_rust_sdk::web3id::Web3IdAttribute;
use concordium_rwa_backend_shared::db::DbPool;
use concordium_rwa_verifier_api::api;
use concordium_rwa_verifier_api::identity_registry_client::IdentityRegistryClient;
use concordium_rwa_verifier_api::web3_id_utils::{CredStatement, GlobalContext, IdStatement};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{Connection, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use futures::{StreamExt, TryStreamExt};
use log::{debug, info};
use poem::listener::TcpListener;
use poem::middleware::{AddData, Cors};
use poem::{EndpointExt, Route, Server};

#[derive(Parser, Debug, Clone)]
pub struct Config {
    #[clap(env, long)]
    pub concordium_node_uri: String,
    #[clap(env, long)]
    pub web_server_addr:     String,
    #[clap(env, long)]
    pub database_url:        String,
    #[clap(env, long)]
    pub db_pool_max_size:    u32,
    #[clap(env, long)]
    pub identity_registry:   String,
    #[clap(env, long)]
    pub wallet_path:         PathBuf,
    #[clap(env, long)]
    pub max_energy:          String,
    #[clap(env, long)]
    pub network:             String,
}

#[tokio::main]
async fn main() {
    dotenvy::from_filename(Path::new(env!("CARGO_MANIFEST_DIR")).join(".env")).ok();
    env_logger::init();
    let config = Config::parse();
    info!("Verifier API: Starting Server");
    debug!("{:#?}", config);

    run_migrations(&config.database_url);
    let routes = create_server_routes(config.to_owned()).await;
    info!("Starting Server at {}", config.web_server_addr);
    Server::new(TcpListener::bind(config.web_server_addr))
        .run(routes)
        .await
        .expect("Server runtime error");
}

async fn create_server_routes(config: Config) -> impl poem::Endpoint {
    let wallet = WalletAccount::from_json_file(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(config.wallet_path),
    )
    .expect("Failed to load wallet");
    let agent_address = wallet.address;
    let identity_registry: ContractAddress = config
        .identity_registry
        .parse()
        .expect("Failed to parse identity registry address");
    let network: Network = config.network.parse().expect("Failed to parse network");
    let max_energy: Energy = config
        .max_energy
        .parse()
        .expect("Failed to parse max energy");

    let endpoint: v2::Endpoint = config
        .concordium_node_uri
        .parse()
        .expect("Failed to parse Concordium Node URI");
    let mut concordium_client = v2::Client::new(endpoint)
        .await
        .expect("Failed to create Concordium Client");

    let (global_context, identity_providers) =
        get_concordium_identity_providers(&mut concordium_client)
            .await
            .expect("Failed to get identity providers");

    let mut identity_registry_client =
        IdentityRegistryClient::new(concordium_client.clone(), identity_registry);
    let issuers = identity_registry_client
        .issuers()
        .await
        .expect("Failed to get issuers");
    info!("Issuers: {:?}", issuers);

    identity_registry_client
        .is_agent(&Address::Account(wallet.address))
        .await
        .expect("Failed to check if agent is an agent")
        .then_some(())
        .expect("provided agent wallet is not an agent in identity registry");

    let (id_statement, cred_statement) = create_statements();
    let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
    let pool: DbPool = Pool::builder()
        .max_size(config.db_pool_max_size)
        .build(manager)
        .unwrap();
    let api_service = api::create_service();
    let ui = api_service.swagger_ui();

    Route::new()
        .nest("/", api_service)
        .nest("/ui", ui)
        .with(AddData::new(pool))
        .with(AddData::new(global_context))
        .with(AddData::new(identity_providers))
        .with(AddData::new(issuers))
        .with(AddData::new(concordium_client))
        .with(AddData::new(id_statement))
        .with(AddData::new(cred_statement))
        .with(AddData::new(identity_registry))
        .with(AddData::new(network))
        .with(AddData::new(max_energy))
        .with(AddData::new(agent_address))
        .with(AddData::new(Arc::new(wallet)))
        .with(Cors::new())
}

fn create_statements() -> (IdStatement, CredStatement) {
    let now = chrono::Utc::now();
    let year = u64::try_from(now.year()).ok().unwrap();
    let years_ago = year.checked_sub(18).unwrap();
    let date_years_ago = format!("{:04}{:02}{:02}", years_ago, now.month(), now.day());
    let upper = Web3IdAttribute::String(AttributeKind(date_years_ago));
    let lower = Web3IdAttribute::String(AttributeKind(String::from("18000101")));
    let id_statement: IdStatement = vec![
        AtomicStatement::AttributeInRange {
            statement: AttributeInRangeStatement {
                // date of birth
                attribute_tag: AttributeTag(3),
                lower,
                upper,
                _phantom: std::marker::PhantomData,
            },
        },
        AtomicStatement::RevealAttribute {
            statement: RevealAttributeStatement {
                // nationality
                attribute_tag: AttributeTag(5),
            },
        },
    ];

    let cred_statement: CredStatement = vec![AtomicStatement::RevealAttribute {
        statement: RevealAttributeStatement {
            // `degreeType` is being used to enable testing of the project using the [web3 id test tools](https://github.com/Concordium/concordium-web3id/blob/main/test-tools/issuer-front-end/README.md)
            attribute_tag: "degreeType".to_string(),
        },
    }];
    (id_statement, cred_statement)
}

async fn get_concordium_identity_providers(
    concordium_client: &mut v2::Client,
) -> Result<
    (
        GlobalContext,
        Vec<concordium_rust_sdk::id::types::IpIdentity>,
    ),
    anyhow::Error,
> {
    let global_context = concordium_client
        .get_cryptographic_parameters(BlockIdentifier::LastFinal)
        .await?
        .response;
    let identity_providers = concordium_client
        .get_identity_providers(BlockIdentifier::LastFinal)
        .await?
        .response
        .map_ok(|ip_info| ip_info.ip_identity)
        .filter_map(|r| async move {
            match r {
                Ok(r) => Some(r),
                Err(_) => None,
            }
        })
        .collect::<Vec<_>>()
        .await;
    info!("Identity Providers: {:?}", identity_providers);
    Ok((global_context, identity_providers))
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
fn run_migrations(database_url: &str) {
    info!("Running migrations on database: {}", database_url);
    let mut conn = PgConnection::establish(database_url).expect("Error connecting to Postgres");
    let applied_migrations = conn
        .run_pending_migrations(MIGRATIONS)
        .expect("Error running migrations");
    applied_migrations
        .iter()
        .for_each(|m| info!("Applied migration: {}", m));
}
