use std::ops::{Add, Sub};

use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use concordium_rust_sdk::cis2;
use concordium_rust_sdk::common::types::Timestamp;
use concordium_rust_sdk::types::{Address, ContractAddress};
use concordium_rwa_backend_shared::db::{to_naive_utc, token_amount_to_sql, DbConn, DbResult};
use diesel::prelude::*;
use num_traits::Zero;

use crate::schema::{
    cis2_agents, cis2_compliances, cis2_deposits, cis2_identity_registries, cis2_operators,
    cis2_recovery_records, cis2_token_holders, cis2_tokens,
};

#[derive(Selectable, Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = cis2_agents)]
#[diesel(primary_key(cis2_address, agent_address))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Agent {
    pub cis2_address:  String,
    pub agent_address: String,
}

impl Agent {
    pub fn new(agent_address: Address, _time: Timestamp, cis2_address: &ContractAddress) -> Self {
        Self {
            agent_address: agent_address.to_string(),
            cis2_address:  cis2_address.to_string(),
        }
    }
}

#[allow(dead_code)]
pub fn list_agents(
    conn: &mut DbConn,
    cis2_address: &ContractAddress,
    page_size: i64,
    page: i64,
) -> DbResult<(Vec<Agent>, i64)> {
    let select_filter = cis2_agents::cis2_address.eq(cis2_address.to_string());
    let res: Vec<Agent> = cis2_agents::table
        .filter(&select_filter)
        .select(Agent::as_select())
        .limit(page_size)
        .offset(page_size * page)
        .get_results(conn)?;
    let count: i64 = cis2_agents::table
        .filter(select_filter)
        .count()
        .get_result(conn)?;
    let page_count = (count + page_size - 1) / page_size;

    Ok((res, page_count))
}

pub fn insert_agent(conn: &mut DbConn, agent: Agent) -> DbResult<()> {
    let updated_rows = diesel::insert_into(cis2_agents::table)
        .values(agent)
        .execute(conn)?;
    assert_eq!(updated_rows, 1, "error {} rows were updated", updated_rows);
    Ok(())
}

pub fn remove_agent(
    conn: &mut DbConn,
    cis2_address: &ContractAddress,
    agent_address: &Address,
) -> DbResult<()> {
    let delete_filter = cis2_agents::cis2_address
        .eq(cis2_address.to_string())
        .and(cis2_agents::agent_address.eq(agent_address.to_string()));
    let updated_rows = diesel::delete(cis2_agents::table)
        .filter(delete_filter)
        .execute(conn)?;
    assert_eq!(updated_rows, 1, "error {} rows were updated", updated_rows);
    Ok(())
}

#[derive(Selectable, Queryable, Identifiable, Insertable, AsChangeset, Debug, PartialEq)]
#[diesel(table_name = cis2_compliances)]
#[diesel(primary_key(cis2_address))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Compliance {
    pub cis2_address:       String,
    pub compliance_address: String,
}

impl Compliance {
    pub fn new(cis2_address: &ContractAddress, compliance: &ContractAddress) -> Self {
        Compliance {
            cis2_address:       cis2_address.to_string(),
            compliance_address: compliance.to_string(),
        }
    }
}

pub fn upsert_compliance(conn: &mut DbConn, record: &Compliance) -> DbResult<()> {
    let row_count = diesel::insert_into(cis2_compliances::table)
        .values(record)
        .on_conflict((cis2_compliances::cis2_address,))
        .do_update()
        .set(record)
        .execute(conn)?;

    assert_eq!(row_count, 1, "More than one row updated");

    Ok(())
}

#[derive(Selectable, Queryable, Identifiable, Insertable, AsChangeset, Debug, PartialEq)]
#[diesel(table_name =
cis2_identity_registries)]
#[diesel(primary_key(cis2_address))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IdentityRegistry {
    pub cis2_address:              String,
    pub identity_registry_address: String,
}

impl IdentityRegistry {
    pub fn new(
        cis2_address: &ContractAddress,
        identity_registry_address: &ContractAddress,
    ) -> Self {
        IdentityRegistry {
            cis2_address:              cis2_address.to_string(),
            identity_registry_address: identity_registry_address.to_string(),
        }
    }
}

pub fn upsert_identity_registry(conn: &mut DbConn, record: &IdentityRegistry) -> DbResult<()> {
    let row_count = diesel::insert_into(cis2_identity_registries::table)
        .values(record)
        .on_conflict((cis2_identity_registries::cis2_address,))
        .do_update()
        .set(record)
        .execute(conn)?;

    assert_eq!(row_count, 1, "More than one row updated");

    Ok(())
}

pub fn update_token_paused(
    conn: &mut DbConn,
    cis2_address: &ContractAddress,
    token_id: &cis2::TokenId,
    paused: bool,
) -> DbResult<()> {
    let update_filter = cis2_tokens::cis2_address
        .eq(cis2_address.to_string())
        .and(cis2_tokens::token_id.eq(token_id.to_string()));
    let update = cis2_tokens::is_paused.eq(paused);

    let row_count = diesel::update(cis2_tokens::table)
        .filter(update_filter)
        .set(update)
        .execute(conn)?;
    assert_eq!(row_count, 1, "More than one row updated");

    Ok(())
}

#[derive(Selectable, Queryable, Identifiable, Insertable, AsChangeset, Debug, PartialEq)]
#[diesel(table_name = cis2_token_holders)]
#[diesel(primary_key(cis2_address, token_id, holder_address))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TokenHolder {
    pub cis2_address:   String,
    pub token_id:       String,
    pub holder_address: String,
    pub balance:        BigDecimal,
    pub frozen_balance: BigDecimal,
    pub create_time:    NaiveDateTime,
}

impl TokenHolder {
    pub fn new(
        cis2_address: &ContractAddress,
        token_id: &cis2::TokenId,
        holder_address: &Address,
        balance: &cis2::TokenAmount,
        frozen_balance: &cis2::TokenAmount,
        create_time: Timestamp,
    ) -> Self {
        Self {
            cis2_address:   cis2_address.to_string(),
            token_id:       token_id.to_string(),
            holder_address: holder_address.to_string(),
            balance:        token_amount_to_sql(balance),
            frozen_balance: token_amount_to_sql(frozen_balance),
            create_time:    to_naive_utc(create_time),
        }
    }
}

pub fn list_tokens_by_holder(
    conn: &mut DbConn,
    cis2_address: &ContractAddress,
    holder_address: &Address,
    page_size: i64,
    page: i64,
) -> DbResult<(Vec<TokenHolder>, i64)> {
    let query = cis2_token_holders::table.filter(
        cis2_token_holders::cis2_address
            .eq(cis2_address.to_string())
            .and(cis2_token_holders::holder_address.eq(holder_address.to_string())),
    );
    let tokens = query
        .clone()
        .select(TokenHolder::as_select())
        .order(cis2_token_holders::create_time)
        .offset(page * page_size)
        .limit(page_size)
        .get_results(conn)?;
    let count_total: i64 = query.count().get_result(conn)?;

    let page_count = (count_total + page_size - 1) / page_size;
    Ok((tokens, page_count))
}

pub fn list_holders_by_token(
    conn: &mut DbConn,
    cis2_address: &ContractAddress,
    token_id: &cis2::TokenId,
    page_size: i64,
    page: i64,
) -> DbResult<(Vec<TokenHolder>, i64)> {
    let query = cis2_token_holders::table.filter(
        cis2_token_holders::cis2_address
            .eq(cis2_address.to_string())
            .and(cis2_token_holders::token_id.eq(token_id.to_string())),
    );
    let tokens = query
        .clone()
        .select(TokenHolder::as_select())
        .order(cis2_token_holders::create_time)
        .offset(page * page_size)
        .limit(page_size)
        .get_results(conn)?;
    let count_total: i64 = query.count().get_result(conn)?;

    let page_count = (count_total + page_size - 1) / page_size;
    Ok((tokens, page_count))
}

pub fn update_balance_frozen(
    conn: &mut DbConn,
    cis2_address: &ContractAddress,
    token_id: &cis2::TokenId,
    holder_address: &Address,
    balance_delta: &cis2::TokenAmount,
    increase: bool,
) -> DbResult<()> {
    let update_filter = cis2_token_holders::cis2_address
        .eq(cis2_address.to_string())
        .and(cis2_token_holders::token_id.eq(token_id.to_string()))
        .and(cis2_token_holders::holder_address.eq(holder_address.to_string()));

    let updated_rows = match increase {
        true => {
            let update = cis2_token_holders::frozen_balance
                .eq(cis2_token_holders::frozen_balance.add(token_amount_to_sql(balance_delta)));
            diesel::update(cis2_token_holders::table)
                .filter(update_filter)
                .set(update)
                .execute(conn)?
        }
        false => {
            let update = cis2_token_holders::frozen_balance
                .eq(cis2_token_holders::frozen_balance.sub(token_amount_to_sql(balance_delta)));
            diesel::update(cis2_token_holders::table)
                .filter(update_filter)
                .set(update)
                .execute(conn)?
        }
    };
    assert_eq!(updated_rows, 1, "error: {} rows(s) updated", updated_rows);

    Ok(())
}

pub fn insert_holder_or_add_balance(conn: &mut DbConn, holder: &TokenHolder) -> DbResult<()> {
    let updated_rows = diesel::insert_into(cis2_token_holders::table)
        .values(holder)
        .on_conflict((
            cis2_token_holders::cis2_address,
            cis2_token_holders::token_id,
            cis2_token_holders::holder_address,
        ))
        .do_update()
        .set(cis2_token_holders::balance.eq(cis2_token_holders::balance.add(&holder.balance)))
        .execute(conn)?;
    assert_eq!(updated_rows, 1, "error: {} rows(s) updated", updated_rows);

    Ok(())
}

pub fn update_sub_balance(
    conn: &mut DbConn,
    cis2_address: &ContractAddress,
    token_id: &cis2::TokenId,
    holder_address: &Address,
    balance_delta: &cis2::TokenAmount,
) -> DbResult<()> {
    conn.transaction(|conn| {
        let update_filter = cis2_token_holders::cis2_address
            .eq(cis2_address.to_string())
            .and(cis2_token_holders::token_id.eq(token_id.to_string()))
            .and(cis2_token_holders::holder_address.eq(holder_address.to_string()));
        let update = cis2_token_holders::balance
            .eq(cis2_token_holders::balance.sub(token_amount_to_sql(balance_delta)));
        let updated_rows = diesel::update(cis2_token_holders::table)
            .filter(&update_filter)
            .set(update)
            .execute(conn)?;
        assert_eq!(updated_rows, 1, "error: {} rows(s) updated", updated_rows);

        let delete_filter = update_filter.and(cis2_token_holders::balance.eq(BigDecimal::zero()));
        diesel::delete(cis2_token_holders::table)
            .filter(&delete_filter)
            .execute(conn)?;

        Ok(())
    })
}

pub fn update_replace_holder(
    conn: &mut DbConn,
    cis2_address: &ContractAddress,
    holder_address: &Address,
    recovery_address: &Address,
) -> DbResult<usize> {
    let updated_rows = diesel::update(cis2_token_holders::table)
        .filter(
            cis2_token_holders::cis2_address
                .eq(cis2_address.to_string())
                .and(cis2_token_holders::holder_address.eq(holder_address.to_string())),
        )
        .set(cis2_token_holders::holder_address.eq(recovery_address.to_string()))
        .execute(conn)?;

    Ok(updated_rows)
}

#[derive(Selectable, Queryable, Identifiable, Insertable, AsChangeset, Debug, PartialEq)]
#[diesel(table_name = cis2_tokens)]
#[diesel(primary_key(cis2_address, token_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Token {
    pub cis2_address:  String,
    pub token_id:      String,
    pub is_paused:     bool,
    pub metadata_url:  String,
    pub metadata_hash: Option<Vec<u8>>,
    pub supply:        BigDecimal,
    pub create_time:   NaiveDateTime,
}
impl Token {
    pub fn new(
        cis2_address: &ContractAddress,
        token_id: &cis2::TokenId,
        is_paused: bool,
        metadata_url: String,
        metadata_hash: Option<[u8; 32]>,
        supply: &cis2::TokenAmount,
        create_time: Timestamp,
    ) -> Self {
        Self {
            cis2_address: cis2_address.to_string(),
            token_id: token_id.to_string(),
            is_paused,
            metadata_url,
            metadata_hash: metadata_hash.map(|value: [u8; 32]| value.to_vec()),
            supply: token_amount_to_sql(supply),
            create_time: to_naive_utc(create_time),
        }
    }
}

pub fn list_tokens_for_contract(
    conn: &mut DbConn,
    cis2_address: &ContractAddress,
    page_size: i64,
    page: i64,
) -> DbResult<(Vec<Token>, i64)> {
    let query = cis2_tokens::table.filter(cis2_tokens::cis2_address.eq(cis2_address.to_string()));
    let tokens = query
        .clone()
        .select(Token::as_select())
        .order(cis2_tokens::create_time)
        .offset(page * page_size)
        .limit(page_size)
        .get_results(conn)?;
    let count_total: i64 = query.count().get_result(conn)?;

    let page_count = (count_total + page_size - 1) / page_size;
    Ok((tokens, page_count))
}

pub fn insert_token_or_update_metadata(conn: &mut DbConn, token: &Token) -> DbResult<()> {
    let row_count = diesel::insert_into(cis2_tokens::table)
        .values(token)
        .on_conflict((cis2_tokens::cis2_address, cis2_tokens::token_id))
        .do_update()
        .set((
            cis2_tokens::metadata_url.eq(&token.metadata_url),
            cis2_tokens::metadata_hash.eq(&token.metadata_hash),
        ))
        .execute(conn)?;

    assert_eq!(row_count, 1, "More than 1 row updated");
    Ok(())
}

pub fn update_supply(
    conn: &mut DbConn,
    cis2_address: &ContractAddress,
    token_id: &cis2::TokenId,
    supply_delta: &cis2::TokenAmount,
    increase: bool,
) -> DbResult<()> {
    let update_filter = cis2_tokens::cis2_address
        .eq(cis2_address.to_string())
        .and(cis2_tokens::token_id.eq(token_id.to_string()));
    let query = diesel::update(cis2_tokens::table).filter(&update_filter);
    let update_rows = match increase {
        true => query
            .set(cis2_tokens::supply.eq(cis2_tokens::supply.add(token_amount_to_sql(supply_delta))))
            .execute(conn)?,
        false => query
            .set(cis2_tokens::supply.eq(cis2_tokens::supply.sub(token_amount_to_sql(supply_delta))))
            .execute(conn)?,
    };
    assert_eq!(update_rows, 1, "error {} rows updated", update_rows);
    Ok(())
}

#[derive(Selectable, Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = cis2_operators)]
#[diesel(primary_key(cis2_address, holder_address, operator_address))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Operator {
    pub cis2_address:     String,
    pub holder_address:   String,
    pub operator_address: String,
}
impl Operator {
    pub fn new(
        cis2_address: &ContractAddress,
        holder_address: &Address,
        operator_address: &Address,
    ) -> Self {
        Self {
            cis2_address:     cis2_address.to_string(),
            holder_address:   holder_address.to_string(),
            operator_address: operator_address.to_string(),
        }
    }
}

pub fn insert_operator(conn: &mut DbConn, record: &Operator) -> DbResult<()> {
    diesel::insert_into(cis2_operators::table)
        .values(record)
        .on_conflict_do_nothing()
        .execute(conn)?;
    Ok(())
}

pub fn delete_operator(conn: &mut DbConn, record: &Operator) -> DbResult<()> {
    let delete_filter = cis2_operators::cis2_address.eq(&record.cis2_address).and(
        cis2_operators::holder_address
            .eq(&record.holder_address)
            .and(cis2_operators::operator_address.eq(&record.holder_address)),
    );

    diesel::delete(cis2_operators::table)
        .filter(delete_filter)
        .execute(conn)?;

    Ok(())
}

#[derive(Selectable, Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = cis2_recovery_records)]
#[diesel(primary_key(cis2_address, holder_address))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RecoveryRecord {
    pub cis2_address:      String,
    pub holder_address:    String,
    pub recovered_address: String,
}
impl RecoveryRecord {
    pub fn new(
        cis2_address: &ContractAddress,
        holder_address: &Address,
        recovered_address: &Address,
    ) -> Self {
        Self {
            cis2_address:      cis2_address.to_string(),
            holder_address:    holder_address.to_string(),
            recovered_address: recovered_address.to_string(),
        }
    }
}

pub fn insert_recovery_record(conn: &mut DbConn, record: &RecoveryRecord) -> DbResult<()> {
    diesel::insert_into(cis2_recovery_records::table)
        .values(record)
        .execute(conn)?;

    Ok(())
}

#[derive(Selectable, Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = cis2_deposits)]
#[diesel(primary_key(
    cis2_address,
    deposited_cis2_address,
    deposited_token_id,
    deposited_holder_address
))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Cis2Deposit {
    pub cis2_address:             String,
    pub deposited_cis2_address:   String,
    pub deposited_token_id:       String,
    pub deposited_holder_address: String,
    pub deposited_amount:         BigDecimal,
}

impl Cis2Deposit {
    pub fn new(
        cis2_address: &ContractAddress,
        deposited_cis2_address: &ContractAddress,
        deposited_token_id: &cis2::TokenId,
        deposited_holder_address: &Address,
        deposited_amount: &cis2::TokenAmount,
    ) -> Self {
        Self {
            cis2_address:             cis2_address.to_string(),
            deposited_cis2_address:   deposited_cis2_address.to_string(),
            deposited_token_id:       deposited_token_id.to_string(),
            deposited_holder_address: deposited_holder_address.to_string(),
            deposited_amount:         token_amount_to_sql(deposited_amount),
        }
    }
}

pub fn list_deposits_by_holder(
    conn: &mut DbConn,
    cis2_address: &ContractAddress,
    holder_address: &Address,
    page_size: i64,
    page: i64,
) -> DbResult<(Vec<Cis2Deposit>, i64)> {
    let query = cis2_deposits::table.filter(
        cis2_deposits::cis2_address
            .eq(cis2_address.to_string())
            .and(cis2_deposits::deposited_holder_address.eq(holder_address.to_string())),
    );

    let tokens = query
        .clone()
        .select(Cis2Deposit::as_select())
        .order(cis2_deposits::deposited_cis2_address)
        .offset(page * page_size)
        .limit(page_size)
        .get_results(conn)?;
    let count_total: i64 = query.count().get_result(conn)?;

    let page_count = (count_total + page_size - 1) / page_size;
    Ok((tokens, page_count))
}

pub fn insert_or_inc_deposit_amount(conn: &mut DbConn, record: &Cis2Deposit) -> DbResult<()> {
    let row_count = diesel::insert_into(cis2_deposits::table)
        .values(record)
        .on_conflict((
            cis2_deposits::cis2_address,
            cis2_deposits::deposited_cis2_address,
            cis2_deposits::deposited_token_id,
            cis2_deposits::deposited_holder_address,
        ))
        .do_update()
        .set(
            cis2_deposits::deposited_amount
                .eq(cis2_deposits::deposited_amount.add(&record.deposited_amount)),
        )
        .execute(conn)?;
    assert_eq!(row_count, 1, "error {} rows updated", row_count);
    Ok(())
}

pub fn update_sub_deposit_amount(conn: &mut DbConn, record: &Cis2Deposit) -> DbResult<()> {
    conn.transaction(|conn| {
        let update_filter = cis2_deposits::cis2_address
            .eq(&record.cis2_address)
            .and(cis2_deposits::deposited_cis2_address.eq(&record.deposited_cis2_address))
            .and(cis2_deposits::deposited_token_id.eq(&record.deposited_token_id))
            .and(cis2_deposits::deposited_holder_address.eq(&record.deposited_holder_address));
        let update_query = cis2_deposits::deposited_amount
            .eq(cis2_deposits::deposited_amount.sub(&record.deposited_amount));
        let row_count = diesel::update(cis2_deposits::table)
            .filter(&update_filter)
            .set(update_query)
            .execute(conn)?;
        assert_eq!(row_count, 1, "error {} rows updated", row_count);

        let delete_filter =
            update_filter.and(cis2_deposits::deposited_amount.eq(BigDecimal::zero()));
        diesel::delete(cis2_deposits::table)
            .filter(delete_filter)
            .execute(conn)?;

        Ok(())
    })
}
