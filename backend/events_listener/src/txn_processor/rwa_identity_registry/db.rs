use chrono::{DateTime, NaiveDateTime, Utc};
use concordium_rust_sdk::types::{Address, ContractAddress};
use concordium_rwa_backend_shared::db::{DbConn, DbResult};
use diesel::prelude::*;

use crate::schema::{
    identity_registry_agents, identity_registry_identities, identity_registry_issuers,
};

#[derive(Selectable, Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = identity_registry_identities)]
#[diesel(primary_key(identity_registry_address, identity_address))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Identity {
    identity_registry_address: String,
    identity_address:          String,
    create_time:               NaiveDateTime,
}

impl Identity {
    pub fn new(
        identity_address: &Address,
        time: DateTime<Utc>,
        identity_registry_address: &ContractAddress,
    ) -> Self {
        Self {
            identity_address:          identity_address.to_string(),
            create_time:               time.naive_utc(),
            identity_registry_address: identity_registry_address.to_string(),
        }
    }
}

#[allow(dead_code)]
pub fn list_identities(
    conn: &mut DbConn,
    identity_registry_address: &ContractAddress,
    page_size: i64,
    page: i64,
) -> DbResult<(Vec<Identity>, i64)> {
    let select_filter = identity_registry_identities::identity_registry_address
        .eq(identity_registry_address.to_string());
    let res: Vec<Identity> = identity_registry_identities::table
        .filter(select_filter.clone())
        .select(Identity::as_select())
        .limit(page_size)
        .offset(page_size * page)
        .get_results(conn)?;
    let count: i64 = identity_registry_identities::table
        .filter(select_filter)
        .count()
        .get_result(conn)?;
    let page_count = (count + page_size - 1) / page_size;

    Ok((res, page_count))
}

pub fn insert_identity(conn: &mut DbConn, identity: Identity) -> DbResult<usize> {
    diesel::insert_into(identity_registry_identities::table)
        .values(identity)
        .execute(conn)
}

pub fn remove_identity(conn: &mut DbConn, address: &Address) -> DbResult<usize> {
    diesel::delete(QueryDsl::filter(
        identity_registry_identities::table,
        identity_registry_identities::identity_address.eq(address.to_string()),
    ))
    .execute(conn)
}

#[derive(Selectable, Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = identity_registry_issuers)]
#[diesel(primary_key(identity_registry_address, issuer_address))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Issuer {
    identity_registry_address: String,
    issuer_address:            String,
    create_time:               NaiveDateTime,
}

impl Issuer {
    pub fn new(
        issuer_address: &ContractAddress,
        time: DateTime<Utc>,
        identity_registry_address: &ContractAddress,
    ) -> Self {
        Issuer {
            issuer_address:            issuer_address.to_string(),
            create_time:               time.naive_utc(),
            identity_registry_address: identity_registry_address.to_string(),
        }
    }
}

#[allow(dead_code)]
pub fn list_issuers(
    conn: &mut DbConn,
    identity_registry_address: &ContractAddress,
    page_size: i64,
    page: i64,
) -> DbResult<(Vec<Issuer>, i64)> {
    let select_filter = identity_registry_issuers::identity_registry_address
        .eq(identity_registry_address.to_string());
    let res: Vec<Issuer> = identity_registry_issuers::table
        .filter(select_filter.clone())
        .select(Issuer::as_select())
        .limit(page_size)
        .offset(page_size * page)
        .get_results(conn)?;
    let count: i64 = identity_registry_issuers::table
        .filter(select_filter)
        .count()
        .get_result(conn)?;
    let page_count = (count + page_size - 1) / page_size;

    Ok((res, page_count))
}

pub fn insert_issuer(conn: &mut DbConn, issuer: Issuer) -> DbResult<usize> {
    diesel::insert_into(identity_registry_issuers::table)
        .values(issuer)
        .execute(conn)
}

pub fn remove_issuer(
    conn: &mut DbConn,
    identity_registry_address: &ContractAddress,
    issuer_address: &ContractAddress,
) -> DbResult<usize> {
    let delete_filter = identity_registry_issuers::issuer_address
        .eq(issuer_address.to_string())
        .and(
            identity_registry_issuers::identity_registry_address
                .eq(identity_registry_address.to_string()),
        );

    diesel::delete(identity_registry_issuers::table)
        .filter(delete_filter)
        .execute(conn)
}

#[derive(Selectable, Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = identity_registry_agents)]
#[diesel(primary_key(identity_registry_address, agent_address))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Agent {
    pub identity_registry_address: String,
    pub agent_address:             String,
    pub create_time:               NaiveDateTime,
}

impl Agent {
    pub fn new(
        address: Address,
        time: DateTime<Utc>,
        identity_registry_address: &ContractAddress,
    ) -> Self {
        Self {
            agent_address:             address.to_string(),
            create_time:               time.naive_utc(),
            identity_registry_address: identity_registry_address.to_string(),
        }
    }
}

#[allow(dead_code)]
pub fn list_agents(
    conn: &mut DbConn,
    identity_registry_address: &ContractAddress,
    page_size: i64,
    page: i64,
) -> DbResult<(Vec<Agent>, i64)> {
    let select_filter = identity_registry_agents::identity_registry_address
        .eq(identity_registry_address.to_string());
    let res: Vec<Agent> = identity_registry_agents::table
        .filter(select_filter.clone())
        .select(Agent::as_select())
        .limit(page_size)
        .offset(page_size * page)
        .get_results(conn)?;
    let count: i64 = identity_registry_agents::table
        .filter(select_filter)
        .count()
        .get_result(conn)?;
    let page_count = (count + page_size - 1) / page_size;

    Ok((res, page_count))
}

pub fn insert_agent(conn: &mut DbConn, agent: Agent) -> DbResult<usize> {
    diesel::insert_into(identity_registry_agents::table)
        .values(agent)
        .execute(conn)
}

pub fn remove_agent(
    conn: &mut DbConn,
    identity_registry_address: &ContractAddress,
    agent_address: &Address,
) -> DbResult<usize> {
    let delete_filter = identity_registry_agents::agent_address
        .eq(agent_address.to_string())
        .and(
            identity_registry_agents::identity_registry_address
                .eq(identity_registry_address.to_string()),
        );

    diesel::delete(identity_registry_agents::table)
        .filter(delete_filter)
        .execute(conn)
}
