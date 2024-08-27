use concordium_protocols::concordium_cis4::cis4_client::Cis4ClientError;
use concordium_std::*;

/// Represents the different types of errors that can occur in the contract.
#[derive(Serial, Reject, SchemaType)]
pub enum Error {
    /// Triggered when there is an error parsing a value.
    ParseError,
    /// Triggered when there is an error logging a value.
    LogError,
    /// Triggered when an unauthorized action is attempted.
    Unauthorized,
    /// Triggered when an identity could not be found.
    IdentityNotFound,
    /// Triggered when an issuer could not be found.
    IssuerNotFound,
    /// Triggered when an issuer already exists.
    IssuerAlreadyExists,
    /// Triggered when an agent already exists.
    AgentAlreadyExists,
    /// Triggered when an agent could not be found.
    AgentNotFound,
    /// Triggered when an issuer is invalid.
    InvalidIssuer,
    /// Triggered when there is an error calling a contract.
    CallContractError,
}

impl From<ParseError> for Error {
    fn from(_: ParseError) -> Self { Error::ParseError }
}

impl From<LogError> for Error {
    fn from(_: LogError) -> Self { Error::LogError }
}

impl From<Cis4ClientError> for Error {
    fn from(e: Cis4ClientError) -> Self {
        match e {
            Cis4ClientError::NoResponse => Error::InvalidIssuer,
            Cis4ClientError::InvalidResponse => Error::InvalidIssuer,
            Cis4ClientError::CallContractError(_) => Error::CallContractError,
            // these should not happen
            Cis4ClientError::ParseResult => Error::ParseError,
            Cis4ClientError::ParseResultError => Error::ParseError,
        }
    }
}
