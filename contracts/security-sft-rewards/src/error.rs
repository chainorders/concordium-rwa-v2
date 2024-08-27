use concordium_protocols::concordium_cis2_security::identity_registry_client::IdentityRegistryClientError;
use concordium_rwa_utils::conversions::exchange_rate::ExchangeError;
use concordium_rwa_utils::state_implementations::cis2_security_state::Cis2SecurityStateError;
use concordium_rwa_utils::state_implementations::cis2_state::Cis2StateError;
use concordium_rwa_utils::state_implementations::holders_security_state::HolderSecurityStateError;
use concordium_rwa_utils::state_implementations::holders_state::HolderStateError;
use concordium_rwa_utils::state_implementations::rewards_state::RewardsStateError;
use concordium_rwa_utils::state_implementations::tokens_security_state::TokenSecurityError;
use concordium_rwa_utils::state_implementations::tokens_state::TokenStateError;
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
    /// Sender is unauthorized to call this function (Error code: -42000003).
    Unauthorized,
    UnVerifiedIdentity,
    /// Triggered when the transfer is non-compliant.
    InCompliantTransfer,
    /// Triggered when there is an error calling the Compliance Contract.
    ComplianceError,
    /// Triggered when there is an error invoking a contract.
    CallContractError,
    /// Triggered when the token is paused.
    PausedToken,
    /// Triggered when the amount for NFT is not 1.
    InvalidAmount,
    /// Triggered when the provided address is invalid.
    InvalidAddress,
    /// Triggered when an agent already exists.
    AgentAlreadyExists,
    /// Triggered when an agent could not be found.
    AgentNotFound,
    OnlyAccount,
    InvalidDepositData,
    Cis2WithdrawError,
    InsufficientDeposits,
    NotDeposited,
    InvalidRewardRate,
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
            Error::ComplianceError => -5,
            Error::CallContractError => -6,
            Error::PausedToken => -7,
            Error::InvalidAmount => -8,
            Error::InvalidAddress => -9,
            Error::AgentAlreadyExists => -10,
            Error::AgentNotFound => -11,
            Error::OnlyAccount => -12,
            Error::InvalidDepositData => -13,
            Error::Cis2WithdrawError => -14,
            Error::InsufficientDeposits => -15,
            Error::NotDeposited => -16,
            Error::InvalidRewardRate => -17,
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
impl From<TokenStateError> for Error {
    fn from(_: TokenStateError) -> Self { Error::InvalidTokenId }
}
impl From<HolderStateError> for Error {
    fn from(_: HolderStateError) -> Self { Error::InsufficientFunds }
}
impl From<TokenSecurityError> for Error {
    fn from(value: TokenSecurityError) -> Self {
        match value {
            TokenSecurityError::PausedToken => Error::PausedToken,
            TokenSecurityError::InvalidTokenId => Error::InvalidTokenId,
        }
    }
}
impl From<HolderSecurityStateError> for Error {
    fn from(e: HolderSecurityStateError) -> Self {
        match e {
            HolderSecurityStateError::AmountTooLarge => Error::InsufficientFunds,
            HolderSecurityStateError::AddressAlreadyRecovered => Error::InvalidAddress,
            HolderSecurityStateError::InvalidRecoveryAddress => Error::InvalidAddress,
        }
    }
}
impl From<IdentityRegistryClientError> for Error {
    fn from(_: IdentityRegistryClientError) -> Self { Error::CallContractError }
}
impl<T> From<CallContractError<T>> for Error {
    fn from(_: CallContractError<T>) -> Self { Error::CallContractError }
}
impl From<Cis2StateError> for Error {
    fn from(e: Cis2StateError) -> Self {
        match e {
            Cis2StateError::InvalidTokenId => Error::InvalidTokenId,
            Cis2StateError::InsufficientFunds => Error::InsufficientFunds,
            Cis2StateError::InvalidAmount => Error::InvalidAmount,
        }
    }
}

impl From<Cis2SecurityStateError> for Error {
    fn from(value: Cis2SecurityStateError) -> Self {
        match value {
            Cis2SecurityStateError::InvalidTokenId => Error::InvalidTokenId,
            Cis2SecurityStateError::InsufficientFunds => Error::InsufficientFunds,
            Cis2SecurityStateError::InvalidAmount => Error::InvalidAmount,
            Cis2SecurityStateError::InvalidAddress => Error::InvalidAddress,
            Cis2SecurityStateError::PausedToken => Error::PausedToken,
        }
    }
}

impl From<RewardsStateError> for Error {
    fn from(value: RewardsStateError) -> Self {
        match value {
            RewardsStateError::InsufficientFunds => Error::InsufficientFunds,
            RewardsStateError::InvalidAmount => Error::InvalidAmount,
            RewardsStateError::InvalidTokenId => Error::InvalidTokenId,
        }
    }
}
impl From<ExchangeError> for Error {
    fn from(value: ExchangeError) -> Self {
        match value {
            ExchangeError::InvalidRate => Error::InvalidRewardRate,
        }
    }
}
