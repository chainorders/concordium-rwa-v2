use concordium_protocols::concordium_cis2_security::compliance_client::ComplianceError;
use concordium_std::*;

#[derive(Serial, Reject, SchemaType)]
pub enum Error {
    ParseError,
    LogError,
    InvalidModule,
    CallContractError,
    Unauthorized,
    AgentAlreadyExists,
    AgentNotFound,
}

impl From<ParseError> for Error {
    fn from(_: ParseError) -> Self { Error::ParseError }
}

impl From<ComplianceError> for Error {
    fn from(e: ComplianceError) -> Self {
        match e {
            ComplianceError::NoResponse => Error::InvalidModule,
            ComplianceError::InvalidResponse => Error::InvalidModule,
            ComplianceError::CallContractError(_) => Error::CallContractError,
            // these should not happen
            ComplianceError::ParseResult => Error::ParseError,
            ComplianceError::ParseResultError => Error::ParseError,
        }
    }
}

impl From<LogError> for Error {
    fn from(_: LogError) -> Self { Error::LogError }
}
