use concordium_protocols::concordium_cis2_security::identity_registry_client::IdentityRegistryClientError;
use concordium_rwa_utils::conversions::exchange_rate::ExchangeError;
use concordium_std::num::NonZeroI32;
use concordium_std::{CallContractError, LogError, ParseError, Reject, SchemaType};

#[derive(SchemaType)]
pub enum Error {
    /// Triggered when there is an error parsing a value.
    ParseError,
    /// Triggered when there is an error logging a value.
    LogError,
    /// Triggered when the receiver of the token is not verified.
    InvalidTokenId,
    /// The balance of the token owner is insufficient for the transfer (Error code: -42000002).
    InsufficientFunds,
    InsufficientRewardFunds,
    /// Sender is unauthorized to call this function (Error code: -42000003).
    Unauthorized,
    UnVerifiedIdentity,
    /// Triggered when the transfer is non-compliant.
    InCompliantTransfer,
    /// Triggered when there is an error calling the Compliance Contract.
    CallContractError,
    /// Triggered when the token is paused.
    PausedToken,
    /// Triggered when the amount for NFT is not 1.
    InvalidAmount,
    /// Triggered when the provided address is invalid.
    InvalidAddress,
    /// Triggered when an agent could not be found.
    InvalidRewardRate,
    RecoveredAddress,
}

impl Error {
    fn error_code(&self) -> NonZeroI32 {
        NonZeroI32::new(match self {
            // CIS2 Errors codes
            Error::InvalidTokenId => -42000001,
            Error::InsufficientFunds => -42000002,
            Error::Unauthorized => -42000003,
            // General Errors codes
            Error::ParseError => -1,
            Error::LogError => -2,
            Error::UnVerifiedIdentity => -3,
            Error::InCompliantTransfer => -4,
            Error::CallContractError => -5,
            Error::PausedToken => -6,
            Error::InvalidAmount => -7,
            Error::InvalidAddress => -8,
            Error::InvalidRewardRate => -14,
            Error::RecoveredAddress => -15,
            Error::InsufficientRewardFunds => -16,
        })
        .unwrap()
    }
}
impl From<Error> for Reject {
    fn from(err: Error) -> Self {
        Reject {
            error_code: err.error_code(),
            ..Default::default()
        }
    }
}
impl From<ParseError> for Error {
    fn from(_: ParseError) -> Self { Error::ParseError }
}
impl From<LogError> for Error {
    fn from(_: LogError) -> Self { Error::LogError }
}
impl From<IdentityRegistryClientError> for Error {
    fn from(_: IdentityRegistryClientError) -> Self { Error::CallContractError }
}
impl<T> From<CallContractError<T>> for Error {
    fn from(_: CallContractError<T>) -> Self { Error::CallContractError }
}
impl From<ExchangeError> for Error {
    fn from(value: ExchangeError) -> Self {
        match value {
            ExchangeError::InvalidRate => Error::InvalidRewardRate,
        }
    }
}
