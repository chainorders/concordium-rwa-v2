use std::path::Path;

use clap::Parser;
use concordium_rwa_backend_shared::db::DbPool;
use concordium_rwa_events_listener::txn_processor;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{Connection, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use log::{debug, info};
use poem::listener::TcpListener;
use poem::middleware::{AddData, Cors};
use poem::{EndpointExt, Route, Server};

/// Configuration struct for the contracts API.
/// Configuration options for the Contracts API.
#[derive(Parser, Debug, Clone)]
pub struct Config {
    /// Postgres Database Url
    #[clap(env, long)]
    pub database_url:     String,
    #[clap(env, long)]
    pub db_pool_max_size: u32,
    #[clap(env, long)]
    pub web_server_addr:  String,
}

#[tokio::main]
async fn main() {
    dotenvy::from_filename(Path::new(env!("CARGO_MANIFEST_DIR")).join(".env")).ok();

    env_logger::init();
    let config = Config::parse();
    info!("Contracts API: Starting Server");
    debug!("{:#?}", config);

    run_migrations(&config.database_url);
    let api_service = txn_processor::create_service();
    let ui = api_service.swagger_ui();
    let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
    let pool: DbPool = Pool::builder()
        .max_size(config.db_pool_max_size)
        .build(manager)
        .unwrap();
    let routes = Route::new()
        .nest("/", api_service)
        .nest("/ui", ui)
        .with(AddData::new(pool))
        .with(Cors::new());
    info!("Starting Server at {}", config.web_server_addr);
    Server::new(TcpListener::bind(config.web_server_addr))
        .run(routes)
        .await
        .expect("Server runtime error");
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
