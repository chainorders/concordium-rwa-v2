use concordium_rwa_utils::conversions::exchange_rate::ExchangeError;
use concordium_rwa_utils::state_implementations::token_deposits_state::DepositedStateError;
use concordium_std::*;

#[derive(Serial, Reject, SchemaType, Debug)]
pub enum Error {
    ParseError,
    LogError,
    /// The sender is not authorized to call.
    Unauthorized,
    InvalidExchange,
    OnlyAccount,
    InsufficientSupply,
    InvalidRate,
    NotListed,
    InsufficientDeposits,
    InsufficientPayment,
    PaymentNotRequired,
    InvalidDepositData,
    InvalidListToken,
    InvalidPaymentToken,
    InvalidCommission,
    InvalidSupply,
    InvalidExchangeRates,
    Cis2WithdrawError,
    Cis2SettlementError,
    Cis2PaymentError,
    Cis2CommissionPaymentError,
    CCDPaymentError,
    CCDCommissionPaymentError,
    NotDeposited,
}

impl From<ParseError> for Error {
    fn from(_: ParseError) -> Self { Error::ParseError }
}

impl From<LogError> for Error {
    fn from(_: LogError) -> Self { Error::LogError }
}

impl From<ExchangeError> for Error {
    fn from(value: ExchangeError) -> Self {
        match value {
            ExchangeError::InvalidRate => Error::InvalidRate,
        }
    }
}

impl From<DepositedStateError> for Error {
    fn from(value: DepositedStateError) -> Self {
        match value {
            DepositedStateError::TokenNotFound => Error::NotDeposited,
            DepositedStateError::InsufficientDeposits => Error::InsufficientDeposits,
            DepositedStateError::InsufficientLocked => Error::InsufficientSupply,
        }
    }
}
