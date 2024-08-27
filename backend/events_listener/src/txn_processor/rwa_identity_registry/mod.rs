//! This module contains the implementation of the RWA Identity Registry.
//! The `RwaIdentityRegistry` struct provides methods to interact with the RWA
//! Identity Registry contract. It interacts with the `IRwaIdentityRegistryDb`
//! trait to fetch data from the database. The API endpoints are defined using
//! the `poem_openapi` and `poem` crates, and the responses are serialized as
//! JSON using the `Json` type.

pub mod db;
pub mod processor;
// todo add api module exposing open api
// todo update integration tests using the api
#[cfg(test)]
mod integration_tests {
    use chrono::{DateTime, Utc};
    use concordium_rust_sdk::base::smart_contracts::ContractEvent;
    use concordium_rust_sdk::id::types::{AccountAddress, ACCOUNT_ADDRESS_SIZE};
    use concordium_rust_sdk::types::{Address, ContractAddress};
    use concordium_rwa_backend_shared::test::{create_new_database_container, to_contract_event};
    use concordium_rwa_identity_registry::event::{
        AgentUpdatedEvent, Event, IdentityUpdatedEvent, IssuerUpdatedEvent,
    };
    use diesel::r2d2::ConnectionManager;
    use diesel::{Connection, PgConnection};
    use diesel_migrations::{embed_migrations, EmbeddedMigrations};
    use r2d2::Pool;

    use crate::txn_processor::rwa_identity_registry::{db, processor};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

    #[tokio::test]
    async fn add_identity() {
        let (database_url, _container) = create_new_database_container(MIGRATIONS).await;
        let now = DateTime::<Utc>::from_timestamp_millis(Utc::now().timestamp_millis()).unwrap();

        let contract_address = ContractAddress::new(0, 0);
        let events = [
            Event::IdentityRegistered(IdentityUpdatedEvent {
                address: Address::Account(AccountAddress([0; ACCOUNT_ADDRESS_SIZE])),
            }),
            Event::IdentityRegistered(IdentityUpdatedEvent {
                address: Address::Account(AccountAddress([1; ACCOUNT_ADDRESS_SIZE])),
            }),
            Event::IdentityRemoved(IdentityUpdatedEvent {
                address: Address::Account(AccountAddress([0; ACCOUNT_ADDRESS_SIZE])),
            }),
            Event::AgentAdded(AgentUpdatedEvent {
                agent: Address::Account(AccountAddress([10; ACCOUNT_ADDRESS_SIZE])),
            }),
            Event::AgentAdded(AgentUpdatedEvent {
                agent: Address::Account(AccountAddress([11; ACCOUNT_ADDRESS_SIZE])),
            }),
            Event::AgentRemoved(AgentUpdatedEvent {
                agent: Address::Account(AccountAddress([11; ACCOUNT_ADDRESS_SIZE])),
            }),
            Event::IssuerAdded(IssuerUpdatedEvent {
                issuer: ContractAddress {
                    index:    1000,
                    subindex: 0,
                },
            }),
            Event::IssuerAdded(IssuerUpdatedEvent {
                issuer: ContractAddress {
                    index:    1001,
                    subindex: 0,
                },
            }),
            Event::IssuerRemoved(IssuerUpdatedEvent {
                issuer: ContractAddress {
                    index:    1000,
                    subindex: 0,
                },
            }),
        ];
        let events: Vec<ContractEvent> = events.iter().map(to_contract_event).collect();

        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(1)
            .build(manager)
            .expect("Error creating database pool");
        let mut conn = pool.get().expect("Error getting database connection");

        conn.test_transaction(|conn| {
            processor::process_events(conn, now, &contract_address, &events)
                .expect("Error inserting events for first contract");
            processor::process_events(conn, now, &ContractAddress::new(1, 0), &events)
                .expect("Error inserting events for second contract");

            let (identities, page_count) = db::list_identities(conn, &contract_address, 10, 0)
                .expect("Error Listing Identitites");
            assert_eq!(identities, vec![db::Identity::new(
                &Address::Account(AccountAddress([1; ACCOUNT_ADDRESS_SIZE])),
                now,
                &contract_address
            )]);
            assert_eq!(page_count, 1);

            let (agents, page_count) =
                db::list_agents(conn, &contract_address, 10, 0).expect("Error Listing Agents");
            assert_eq!(agents, vec![db::Agent::new(
                Address::Account(AccountAddress([10; ACCOUNT_ADDRESS_SIZE])),
                now,
                &contract_address
            )]);
            assert_eq!(page_count, 1);

            let (issuers, page_count) =
                db::list_issuers(conn, &contract_address, 10, 0).expect("Error Listing Issuers");
            assert_eq!(issuers, vec![db::Issuer::new(
                &ContractAddress {
                    index:    1001,
                    subindex: 0,
                },
                now,
                &contract_address
            )]);
            assert_eq!(page_count, 1);
            Ok::<_, diesel::result::Error>(())
        });
    }
}
