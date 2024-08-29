#![allow(dead_code)]

use concordium_cis2::{IsTokenAmount, IsTokenId, TokenAmountU64, TokenIdVec};
use concordium_std::{to_bytes, Cursor, Deserial, Serial};

pub fn to_token_id_vec<T: IsTokenId+Serial>(token_id: T) -> TokenIdVec {
    TokenIdVec(to_bytes(&token_id)[1..].to_vec())
}

pub fn to_token_amount_u64<A: IsTokenAmount+Serial>(token_amount: A) -> TokenAmountU64 {
    let mut token_amount_bytes = to_bytes(&token_amount);
    let mut cursor = Cursor::new(&mut token_amount_bytes);
    TokenAmountU64::deserial(&mut cursor).unwrap()
}
