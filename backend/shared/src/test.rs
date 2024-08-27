use concordium_cis2::{TokenIdU64, TokenIdU8, TokenIdVec};
use concordium_rust_sdk::base::contracts_common::Serial;
use concordium_rust_sdk::base::smart_contracts::ContractEvent;
use diesel::{Connection, PgConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ImageExt};
use testcontainers_modules::postgres::{self, Postgres};

pub struct TestDbContext {
    pub base_url: String,
    pub db_name:  String,
}

pub async fn create_new_database_container(
    migrations: EmbeddedMigrations,
) -> (String, ContainerAsync<Postgres>) {
    let pg_container = postgres::Postgres::default()
        .with_tag("14-alpine")
        .start()
        .await
        .unwrap();
    let database_url = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        pg_container.get_host_port_ipv4(5432).await.unwrap()
    );

    PgConnection::establish(&database_url)
        .expect("Error connecting to Postgres")
        .run_pending_migrations(migrations)
        .expect("Error running migrations");

    (database_url, pg_container)
}

pub fn to_contract_event<Event>(e: &Event) -> ContractEvent
where Event: Serial {
    let mut out: Vec<u8> = Vec::new();
    e.serial(&mut out).expect("Error serializing event");
    out.into()
}

pub fn to_token_id_vec_u8(token_id: TokenIdU8) -> TokenIdVec {
    let mut bytes: Vec<u8> = vec![];
    token_id
        .serial(&mut bytes)
        .expect("Error converting tokenid u8 to token id vec");
    TokenIdVec(bytes)
}

pub fn to_token_id_vec_u64(token_id: TokenIdU64) -> TokenIdVec {
    let mut bytes: Vec<u8> = vec![];
    token_id
        .serial(&mut bytes)
        .expect("Error converting tokenid u64 to token id vec");
    TokenIdVec(bytes)
}
