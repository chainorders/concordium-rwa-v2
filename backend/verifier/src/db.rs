use chrono::{DateTime, NaiveDateTime, Utc};
use concordium_rust_sdk::base::hashes::TransactionHash;
use concordium_rust_sdk::id::types::AccountAddress;
use concordium_rust_sdk::types::ContractAddress;
use concordium_rwa_backend_shared::db::{DbConn, DbResult};
use diesel::dsl::*;
use diesel::prelude::*;

use crate::schema::verifier_challenges::dsl::*;
use crate::schema::{self};

#[derive(Selectable, Queryable, Identifiable)]
#[diesel(primary_key(id))]
#[diesel(table_name = schema::verifier_challenges)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChallengeSelect {
    pub id:          i32,
    pub challenge:   Vec<u8>,
    pub create_time: NaiveDateTime,
    pub update_time: NaiveDateTime,
}

#[derive(Debug)]
pub struct DbChallenge {
    pub id:        i32,
    pub challenge: [u8; 32],
}

impl From<ChallengeSelect> for DbChallenge {
    fn from(value: ChallengeSelect) -> Self {
        DbChallenge {
            id:        value.id,
            challenge: value
                .challenge
                .try_into()
                .expect("could not de serialize challenge stored in db"),
        }
    }
}

/// Finds a challenge without null txn hash. Denoting an unconsumed
/// challenge
pub async fn find_challenge_wo_txn(
    conn: &mut DbConn,
    for_account: &AccountAddress,
    verifier: &AccountAddress,
    identity_registry: &ContractAddress,
) -> DbResult<Option<DbChallenge>> {
    let db_challenge = verifier_challenges
        .filter(
            account_address
                .eq(for_account.to_string())
                .and(verifier_address.eq(verifier.to_string()))
                .and(identity_registry_address.eq(identity_registry.to_string()))
                .and(txn_hash.is_null()),
        )
        .select(ChallengeSelect::as_select())
        .get_result(conn)
        .optional()?;

    let ret: Option<DbChallenge> = db_challenge.map(|c| c.into());

    Ok(ret)
}

#[derive(Insertable)]
#[diesel(table_name = schema::verifier_challenges)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChallengeInsert {
    pub account_address:           String,
    pub verifier_address:          String,
    pub identity_registry_address: String,
    pub challenge:                 Vec<u8>,
    pub create_time:               NaiveDateTime,
    pub update_time:               NaiveDateTime,
}

impl ChallengeInsert {
    pub fn new(
        accnt: &AccountAddress,
        verifier_accnt: &AccountAddress,
        identity_registry: &ContractAddress,
        db_challenge: [u8; 32],
        now_time: DateTime<Utc>,
    ) -> Self {
        ChallengeInsert {
            account_address:           accnt.to_string(),
            verifier_address:          verifier_accnt.to_string(),
            identity_registry_address: identity_registry.to_string(),
            challenge:                 db_challenge.to_vec(),
            create_time:               now_time.naive_utc(),
            update_time:               now_time.naive_utc(),
        }
    }
}

pub async fn insert_challenge(conn: &mut DbConn, value: ChallengeInsert) -> DbResult<DbChallenge> {
    let res = insert_into(verifier_challenges)
        .values(value)
        .returning(ChallengeSelect::as_returning())
        .get_result(conn)?;
    Ok(res.into())
}

pub async fn update_challenge_add_txn_hash(
    conn: &mut DbConn,
    challenge_db_id: i32,
    challenge_txn_hash: TransactionHash,
) -> DbResult<usize> {
    update(verifier_challenges)
        .filter(id.eq(challenge_db_id).and(txn_hash.is_null()))
        .set((
            txn_hash.eq(challenge_txn_hash.bytes.to_vec()),
            update_time.eq(Utc::now().naive_utc()),
        ))
        .execute(conn)
}
