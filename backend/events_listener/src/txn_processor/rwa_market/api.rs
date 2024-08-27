//! This module contains the API implementation for the RWA market.
//!
//! The `RwaMarketApi` struct provides methods to retrieve paged lists of tokens
//! that are listed, unlisted, or deposited in the RWA market. It interacts with
//! the `IRwaMarketDb` trait to fetch data from the database.
//!
//! The API endpoints are defined using the `poem_openapi` and `poem` crates,
//! and the responses are serialized as JSON using the `Json` type.
//!
//! The `MarketToken` struct represents a token listed in the RWA market,
//! containing information such as the token contract address, token ID, owner,
//! deposited amount, listed amount, and unlisted amount.
//!
//! The `RwaMarketApi` struct has the following methods:
//! - `listed`: Retrieves a paged list of tokens that are listed in the RWA
//!   market.
//! - `unlisted`: Retrieves a paged list of tokens that are unlisted in the RWA
//!   market for a specific owner.
//! - `deposited`: Retrieves a paged list of tokens that are deposited in the
//!   RWA market for a specific owner.
//!
//! The `to_paged_response` method is a helper method used by the above methods
//! to convert the query result into a paged response.
use std::ops::Add;

use concordium_rust_sdk::id::types::AccountAddress;
use concordium_rust_sdk::types::ContractAddress;
use concordium_rwa_backend_shared::api::{ApiContractAddress, Error, PagedResponse, PAGE_SIZE};
use concordium_rwa_backend_shared::db::DbPool;
use poem::web::Data;
use poem::Result;
use poem_openapi::param::Path;
use poem_openapi::payload::Json;
use poem_openapi::{Object, OpenApi};

use super::db;

#[derive(Object)]
pub struct MarketToken {
    pub token_contract:   ApiContractAddress,
    pub token_id:         String,
    pub owner:            String,
    pub deposited_amount: String,
    pub listed_amount:    String,
    pub unlisted_amount:  String,
}

impl From<db::MarketToken> for MarketToken {
    fn from(value: db::MarketToken) -> Self {
        let deposited_amount = value
            .token_listed_amount
            .clone()
            .add(&value.token_unlisted_amount);
        let token_contract: ContractAddress = value
            .token_contract_address
            .parse()
            .expect("Error parsing token contract address");

        Self {
            token_contract:   ApiContractAddress::from_contract_address(token_contract),
            deposited_amount: deposited_amount.to_string(),
            listed_amount:    value.token_listed_amount.to_string(),
            unlisted_amount:  value.token_unlisted_amount.to_string(),
            owner:            value.token_owner_address,
            token_id:         value.token_id.to_string(),
        }
    }
}

/// Represents the RWA Market API.
pub struct RwaMarketApi;

/// API implementation for the RWA market.
#[OpenApi]
impl RwaMarketApi {
    /// Retrieves a paged list of tokens that are listed in the RWA market.
    ///
    /// # Parameters
    ///
    /// - `index`: The index of the contract.
    /// - `subindex`: The subindex of the contract.
    /// - `page`: The page number of the results.
    ///
    /// # Returns
    ///
    /// A `PagedResponse` containing a list of `MarketToken` objects.
    #[oai(path = "/rwa-market/:index/:subindex/listed/:page", method = "get")]
    pub async fn listed(
        &self,
        Data(pool): Data<&DbPool>,
        Path(index): Path<u64>,
        Path(subindex): Path<u64>,
        Path(page): Path<i64>,
    ) -> Result<Json<PagedResponse<MarketToken>>, Error> {
        let market_contract = &ContractAddress { index, subindex };
        let mut conn = pool.get()?;
        let (tokens, page_count) = db::list_tokens(&mut conn, market_contract, PAGE_SIZE, page)?;
        let tokens = tokens.into_iter().map(|t| t.into()).collect();
        let res = PagedResponse {
            data: tokens,
            page,
            page_count,
        };
        Ok(Json(res))
    }

    /// Retrieves a paged list of tokens that are unlisted in the RWA market for
    /// a specific owner.
    ///
    /// # Parameters
    ///
    /// - `index`: The index of the contract.
    /// - `subindex`: The subindex of the contract.
    /// - `owner`: The owner of the tokens.
    /// - `page`: The page number of the results.
    ///
    /// # Returns
    ///
    /// A `PagedResponse` containing a list of `MarketToken` objects.
    #[oai(
        path = "/rwa-market/:index/:subindex/unlisted/:owner/:page",
        method = "get"
    )]
    pub async fn unlisted(
        &self,
        Data(pool): Data<&DbPool>,
        Path(index): Path<u64>,
        Path(subindex): Path<u64>,
        Path(owner): Path<String>,
        Path(page): Path<i64>,
    ) -> Result<Json<PagedResponse<MarketToken>>, Error> {
        let market_contract = &ContractAddress { index, subindex };
        let owner: AccountAddress = owner.parse()?;
        let mut conn = pool.get()?;
        let (tokens, page_count) =
            db::list_tokens_by_owner(&mut conn, market_contract, owner, PAGE_SIZE, page)?;
        let tokens = tokens.into_iter().map(|t| t.into()).collect();
        let res = PagedResponse {
            data: tokens,
            page,
            page_count,
        };
        Ok(Json(res))
    }

    /// Retrieves a paged list of tokens that are deposited in the RWA market
    /// for a specific owner.
    ///
    /// # Parameters
    ///
    /// - `index`: The index of the contract.
    /// - `subindex`: The subindex of the contract.
    /// - `owner`: The owner of the tokens.
    /// - `page`: The page number of the results.
    ///
    /// # Returns
    ///
    /// A `PagedResponse` containing a list of `MarketToken` objects.
    #[oai(
        path = "/rwa-market/:index/:subindex/deposited/:owner/:page",
        method = "get"
    )]
    pub async fn deposited(
        &self,
        Data(pool): Data<&DbPool>,
        Path(index): Path<u64>,
        Path(subindex): Path<u64>,
        Path(owner): Path<String>,
        Path(page): Path<i64>,
    ) -> Result<Json<PagedResponse<MarketToken>>, Error> {
        let market_contract = &ContractAddress { index, subindex };
        let owner: AccountAddress = owner.parse()?;
        let mut conn = pool.get()?;
        let (tokens, page_count) =
            db::list_tokens_by_owner(&mut conn, market_contract, owner, PAGE_SIZE, page)?;
        let tokens = tokens.into_iter().map(|t| t.into()).collect();
        let res = PagedResponse {
            data: tokens,
            page,
            page_count,
        };
        Ok(Json(res))
    }
}
