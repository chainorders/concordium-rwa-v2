use std::sync::Arc;

use chrono::{DateTime, Utc};
use concordium_rust_sdk::common::to_bytes;
use concordium_rust_sdk::id::types::IpIdentity;
use concordium_rust_sdk::smart_contracts::common::{AccountAddress, AccountAddressParseError};
use concordium_rust_sdk::types::smart_contracts::InstanceInfo;
use concordium_rust_sdk::types::{Address, ContractAddress, Energy, WalletAccount};
use concordium_rust_sdk::v2::{BlockIdentifier, QueryError};
use concordium_rust_sdk::web3id::did::Network;
use concordium_rwa_backend_shared::api::ApiContractAddress;
use concordium_rwa_backend_shared::db::DbPool;
use hex::encode;
use log::debug;
use poem::web::Data;
use poem_openapi::payload::Json;
use poem_openapi::{ApiResponse, Object, OpenApi, OpenApiService};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::db;
use crate::identity_registry_client::{self, IdentityRegistryClient};
use crate::web3_id_utils::{
    verify_presentation, CredStatement, GlobalContext, IdStatement, Presentation,
    VerifyPresentationError,
};

#[derive(Debug, ApiResponse)]
pub enum VerifierApiError {
    #[oai(status = 400)]
    BadRequest,
    #[oai(status = 500)]
    InternalServer,
    #[oai(status = 404)]
    NotFound,
}
impl From<concordium_rust_sdk::base::contracts_common::ParseError> for VerifierApiError {
    fn from(_: concordium_rust_sdk::base::contracts_common::ParseError) -> Self { Self::BadRequest }
}
impl From<diesel::result::Error> for VerifierApiError {
    fn from(_: diesel::result::Error) -> Self { Self::InternalServer }
}
impl From<r2d2::Error> for VerifierApiError {
    fn from(_: r2d2::Error) -> Self { Self::InternalServer }
}
impl From<serde_json::Error> for VerifierApiError {
    fn from(_: serde_json::Error) -> Self { Self::InternalServer }
}
impl From<anyhow::Error> for VerifierApiError {
    fn from(_: anyhow::Error) -> Self { Self::InternalServer }
}
impl From<AccountAddressParseError> for VerifierApiError {
    fn from(_: AccountAddressParseError) -> Self { Self::BadRequest }
}
impl From<QueryError> for VerifierApiError {
    fn from(_: QueryError) -> Self { Self::InternalServer }
}
impl From<VerifyPresentationError> for VerifierApiError {
    fn from(_: VerifyPresentationError) -> Self { VerifierApiError::BadRequest }
}
impl From<identity_registry_client::Error> for VerifierApiError {
    fn from(_: identity_registry_client::Error) -> Self { VerifierApiError::InternalServer }
}
#[derive(Object)]
pub struct GenerateChallengeRequest {
    account: String,
}

#[derive(Object)]
pub struct GenerateChallengeResponse {
    challenge:          String,
    id_statement:       Value,
    cred_statement:     Value,
    issuers:            Vec<ApiContractAddress>,
    identity_providers: Vec<u32>,
}

/// The API for the verifier.
pub struct Api;

#[derive(Object)]
pub struct RegisterIdentityRequest {
    pub proof:    Value,
    pub account:  String,
    pub contract: Option<ApiContractAddress>,
}

#[derive(Object)]
pub struct RegisterIdentityResponse {
    txn_hash: String,
}

#[OpenApi]
impl Api {
    /// Generate a challenge for the user to generate Identity and Credential
    /// proofs.
    ///
    /// # Arguments
    ///
    /// * `request` - The account for which the challenge is to be generated.
    ///
    /// # Returns
    ///
    /// * A challenge for the user to generate Identity and Credential proofs.
    #[allow(clippy::too_many_arguments)]
    #[oai(path = "/verifier/generateChallenge", method = "post")]
    pub async fn generate_challenge(
        &self,
        Data(db): Data<&DbPool>,
        Data(verifier_account): Data<&AccountAddress>,
        Data(identity_providers): Data<&Vec<IpIdentity>>,
        Data(issuers): Data<&Vec<ContractAddress>>,
        Data(id_statement): Data<&IdStatement>,
        Data(cred_statement): Data<&CredStatement>,
        Data(identity_registry): Data<&ContractAddress>,
        Json(request): Json<GenerateChallengeRequest>,
    ) -> Result<Json<GenerateChallengeResponse>, VerifierApiError> {
        let mut conn = db.get()?;
        let account: AccountAddress = request.account.parse()?;
        let challenge =
            db::find_challenge_wo_txn(&mut conn, &account, verifier_account, identity_registry)
                .await?;
        let challenge = match challenge {
            Some(challenge) => challenge,
            None => {
                let challenge =
                    create_new_challenge(account, Utc::now(), id_statement, identity_registry);
                db::insert_challenge(
                    &mut conn,
                    db::ChallengeInsert::new(
                        &account,
                        verifier_account,
                        identity_registry,
                        challenge,
                        Utc::now(),
                    ),
                )
                .await?
            }
        };

        Ok(Json(GenerateChallengeResponse {
            challenge:          encode(challenge.challenge),
            cred_statement:     serde_json::to_value(cred_statement)?,
            id_statement:       serde_json::to_value(id_statement)?,
            identity_providers: identity_providers.iter().map(|ip| ip.0).collect(),
            issuers:            issuers
                .iter()
                .map(|c| ApiContractAddress::from_contract_address(*c))
                .collect(),
        }))
    }

    /// Register an identity.
    ///
    /// # Arguments
    ///
    /// * `request` - The request to register an identity.
    ///
    /// # Returns
    ///
    /// * A transaction hash of the transaction that registered the identity.
    #[allow(clippy::too_many_arguments)]
    #[oai(path = "/verifier/registerIdentity", method = "post")]
    pub async fn register_identity(
        &self,
        Data(db): Data<&DbPool>,
        Data(verifier_account): Data<&AccountAddress>,
        Data(global_context): Data<&GlobalContext>,
        Data(concordium_client): Data<&concordium_rust_sdk::v2::Client>,
        Data(identity_registry): Data<&ContractAddress>,
        Data(network): Data<&Network>,
        Data(max_energy): Data<&Energy>,
        Data(agent_wallet): Data<&Arc<WalletAccount>>,
        Json(request): Json<RegisterIdentityRequest>,
    ) -> Result<Json<RegisterIdentityResponse>, VerifierApiError> {
        // Parse the account from the request
        let account: AccountAddress = request.account.parse()?;
        debug!(
            "Registering identity for contract: {:?} from account: {}",
            request.contract, request.account
        );
        debug!("Register Identity Proofs: {:?}", request.proof);

        // Convert the proof to Presentation type
        let proof: Presentation = request.proof.clone().try_into()?;
        let mut conn = db.get()?;
        let challenge =
            db::find_challenge_wo_txn(&mut conn, &account, verifier_account, identity_registry)
                .await?
                .ok_or(VerifierApiError::NotFound)?;
        debug!("Challenge: {:?}", challenge);

        // Verify the presentation and get the verification response
        let mut concordium_client = concordium_client.clone();
        let verification_response = verify_presentation(
            *network,
            &mut concordium_client,
            global_context,
            &proof,
            challenge.challenge,
        )
        .await?;
        debug!(
            "Revealed Id Attributes: {:?}",
            verification_response.revealed_attributes
        );
        debug!("Credentials: {:?}", verification_response.credentials);

        // Determine the identity address based on the contract and account
        let identity_address: Address = {
            if let Some(contract) = request.contract {
                let contract: ContractAddress = contract.into();
                let contract_info = concordium_client
                    .get_instance_info(contract, BlockIdentifier::LastFinal)
                    .await?;
                let contract_owner = match contract_info.response {
                    InstanceInfo::V0 { owner, .. } => owner,
                    InstanceInfo::V1 { owner, .. } => owner,
                };

                if contract_owner != account {
                    debug!(
                        "Contract owner: {} does not match",
                        contract_owner.to_string()
                    );
                    // Enhancement: Return a more specific error. Include debug message as error
                    // message
                    return Err(VerifierApiError::BadRequest);
                }

                Address::Contract(contract)
            } else {
                Address::Account(account)
            }
        };

        // Register the identity using the IdentityRegistryClient
        let txn = IdentityRegistryClient::new(concordium_client, *identity_registry)
            .register_identity(
                agent_wallet,
                identity_address,
                verification_response,
                *max_energy,
            )
            .await?;
        debug!("Register Identity Transaction Hash: {}", txn.to_string());
        db::update_challenge_add_txn_hash(&mut conn, challenge.id, txn).await?;

        Ok(Json(RegisterIdentityResponse {
            txn_hash: txn.to_string(),
        }))
    }
}

fn create_new_challenge(
    account: AccountAddress,
    now: DateTime<Utc>,
    id_statement: &IdStatement,
    identity_registry: &ContractAddress,
) -> [u8; 32] {
    // Create a new challenge based on various inputs
    let mut hasher = Sha256::new();
    hasher.update(to_bytes(id_statement));
    hasher.update(AsRef::<[u8; 32]>::as_ref(&account)); // Add type annotation to specify the implementation of AsRef to use
    hasher.update(identity_registry.index.to_be_bytes());
    hasher.update(identity_registry.subindex.to_be_bytes());
    hasher.update(now.to_rfc3339().as_bytes());

    let result = hasher.finalize();
    let mut challenge = [0; 32];
    challenge.copy_from_slice(&result);
    challenge
}

pub fn create_service() -> OpenApiService<Api, ()> {
    OpenApiService::new(Api, "RWA Verifier API", "1.0.0")
}
