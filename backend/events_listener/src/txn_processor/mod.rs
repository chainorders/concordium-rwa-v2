//! This module contains the transaction processor for the Concordium RWA
//! backend. It includes the definition of the database module, as well as the
//! modules for the RWA identity registry, RWA market, RWA security NFT, and RWA
//! security SFT. It also defines the listener and API configuration struct, as
//! well as the contracts API configuration struct. The module provides
//! functions to run the contracts API server and listener, as well as to
//! generate the API client. It also includes helper functions to create the
//! listener, server routes, and service for the contracts API.
pub mod cis2_security;
pub mod identity_registry;
pub mod nft_multi_rewarded;

use cis2_security::api::Cis2Api;
use poem_openapi::OpenApiService;

/// Creates the service for the contracts API.
pub fn create_service() -> OpenApiService<Cis2Api, ()> {
    OpenApiService::new(Cis2Api, "RWA Contracts API", "1.0.0")
}
