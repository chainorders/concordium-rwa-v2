use concordium_cis2::{TokenAmountU64, TokenIdVec};
use concordium_protocols::concordium_cis2_security;
use concordium_std::*;

use super::error::Error;

pub type TokenId = TokenIdVec;
/// Represents the amount of a token. This should be large enough to accommodate
/// for any token amount which can be received Or exchanged by the contract.
pub type Cis2TokenAmount = TokenAmountU64;
pub type TokenUId = concordium_cis2_security::TokenUId<TokenId>;
pub type TokenOwnerUId = concordium_cis2_security::TokenOwnerUId<TokenId>;
pub type ContractResult<T> = Result<T, Error>;
pub use concordium_rwa_utils::conversions::exchange_rate::{ExchangeError, Rate};

#[derive(Serialize, SchemaType, Clone)]
pub enum ExchangeRate {
    Ccd(Rate),
    Cis2((TokenUId, Rate)),
}

impl ExchangeRate {
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Ccd(rate) => rate.is_valid(),
            Self::Cis2((_, rate)) => rate.is_valid(),
        }
    }
}
