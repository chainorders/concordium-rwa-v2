use concordium_rust_sdk::cis2;
use concordium_rust_sdk::smart_contracts::common::{AccountAddressParseError, AddressParseError};
use concordium_rust_sdk::types::{Address, ContractAddress};
use poem_openapi::payload::Json;
use poem_openapi::types::{ParseFromJSON, ToJSON, Type};
use poem_openapi::{ApiResponse, Object};

pub const PAGE_SIZE: i64 = 20;

/// The error type for the API.
#[derive(Debug, ApiResponse)]
pub enum Error {
    #[oai(status = 400)]
    ParseError,
    #[oai(status = 500)]
    InternalServerError,
}
impl From<diesel::result::Error> for Error {
    fn from(_: diesel::result::Error) -> Self { Self::InternalServerError }
}
impl From<r2d2::Error> for Error {
    fn from(_: r2d2::Error) -> Self { Self::InternalServerError }
}
impl From<cis2::ParseTokenIdVecError> for Error {
    fn from(_: cis2::ParseTokenIdVecError) -> Self { Self::ParseError }
}
impl From<AccountAddressParseError> for Error {
    fn from(_: AccountAddressParseError) -> Self { Self::ParseError }
}
impl From<AddressParseError> for Error {
    fn from(_: AddressParseError) -> Self { Self::ParseError }
}

/// A wrapper around the `ContractAddress` type that can be used in the API.
#[derive(Object, Debug, Clone, Copy, PartialEq)]
pub struct ApiContractAddress {
    pub index:    u64,
    pub subindex: u64,
}

impl ApiContractAddress {
    pub fn from_contract_address(contract_address: ContractAddress) -> Self {
        Self {
            index:    contract_address.index,
            subindex: contract_address.subindex,
        }
    }
}

impl From<ContractAddress> for ApiContractAddress {
    fn from(value: ContractAddress) -> Self { Self::from_contract_address(value) }
}

impl From<ApiContractAddress> for ContractAddress {
    fn from(val: ApiContractAddress) -> Self {
        ContractAddress {
            index:    val.index,
            subindex: val.subindex,
        }
    }
}

/// A wrapper around the `Address` type that can be used in the API.
#[derive(Object, Debug)]
pub struct ApiAddress {
    pub account_address:  Option<String>,
    pub contract_address: Option<ApiContractAddress>,
}

impl From<Address> for ApiAddress {
    fn from(value: Address) -> Self {
        match value {
            Address::Account(value) => ApiAddress {
                account_address:  Some(value.to_string()),
                contract_address: None,
            },
            Address::Contract(value) => ApiAddress {
                account_address:  None,
                contract_address: Some(value.into()),
            },
        }
    }
}

/// Pages Response. This is a generic response that can be used to return a list
/// of items with pagination.
#[derive(Object)]
pub struct PagedResponse<T: Sync+Send+Type+ToJSON+ParseFromJSON> {
    pub page_count: i64,
    pub page:       i64,
    pub data:       Vec<T>,
}

/// A wrapper around the `AccountAddress` type that can be used in the API.
pub type ApiAccountAddress = String;

pub type ApiResult<T> = Result<Json<T>, Error>;
