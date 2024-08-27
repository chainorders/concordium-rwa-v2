use chrono::Utc;
use concordium_rust_sdk::base::contracts_common::{
    NewReceiveNameError, ParseError, PublicKeyEd25519,
};
use concordium_rust_sdk::base::hashes::TransactionHash;
use concordium_rust_sdk::base::smart_contracts::{
    ExceedsParameterSize, OwnedContractName, OwnedParameter,
};
use concordium_rust_sdk::base::transactions::send::GivenEnergy;
use concordium_rust_sdk::common::types::{Amount, TransactionTime};
use concordium_rust_sdk::contract_client::{ContractClient, ContractTransactionMetadata};
use concordium_rust_sdk::types::{
    AccountInfo, Address, ContractAddress, Energy, RejectReason, WalletAccount,
};
use concordium_rust_sdk::v2::{self, BlockIdentifier};
use concordium_rwa_identity_registry::identities::RegisterIdentityParams;
use concordium_rwa_identity_registry::types::{Identity, IdentityAttribute, IdentityCredential};

use super::web3_id_utils::VerifyPresentationResponse;

/// Errors that can occur when interacting with the Identity Registry contract.
#[derive(Debug)]
pub enum Error {
    ParamsSerialization,
    AccountInfoQuery(v2::QueryError),
    TransactionSend(v2::RPCError),
    InstanceInvokeQuery(v2::QueryError),
    ReceiveName(NewReceiveNameError),
    ClientQuery(v2::QueryError),
    InvokeInstance(RejectReason),
    Parse(ParseError),
    Rpc(v2::RPCError),
    ExceedsParameterSize(ExceedsParameterSize),
}

impl From<v2::QueryError> for Error {
    fn from(e: v2::QueryError) -> Self { Error::ClientQuery(e) }
}

impl From<NewReceiveNameError> for Error {
    fn from(e: NewReceiveNameError) -> Self { Error::ReceiveName(e) }
}

impl From<RejectReason> for Error {
    fn from(e: RejectReason) -> Self { Error::InvokeInstance(e) }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self { Error::Parse(e) }
}

impl From<v2::RPCError> for Error {
    fn from(e: v2::RPCError) -> Self { Error::Rpc(e) }
}

impl From<ExceedsParameterSize> for Error {
    fn from(e: ExceedsParameterSize) -> Self { Error::ExceedsParameterSize(e) }
}

/// A client for the Identity Registry contract.
pub struct IdentityRegistryClient {
    pub client: ContractClient<Self>,
}

impl IdentityRegistryClient {
    /// Create a new client for the Identity Registry contract.
    ///
    /// # Arguments
    ///
    /// * `concordium_client` - Client to interact with the Concordium node.
    /// * `identity_registry` - Address of the Identity Registry contract.
    ///
    /// # Returns
    ///
    /// * A new `IdentityRegistryClient`.
    pub fn new(concordium_client: v2::Client, identity_registry: ContractAddress) -> Self {
        let client = ContractClient::new(
            concordium_client,
            identity_registry,
            OwnedContractName::new_unchecked("init_rwa_identity_registry".to_owned()),
        );
        Self { client }
    }

    pub async fn is_agent(&mut self, agent: &Address) -> Result<bool, Error> {
        let res = self
            .client
            .view_raw::<bool, Error>(
                "isAgent",
                OwnedParameter::from_serial(agent).unwrap(),
                BlockIdentifier::LastFinal,
            )
            .await?;

        Ok(res)
    }

    /// Get the list of issuers.
    ///
    /// This is a view function and does not require a transaction.
    ///
    /// # Returns
    ///
    /// * A Result containing a vector of `ContractAddress`es representing the
    ///   issuers, or an `Error`.
    pub async fn issuers(&mut self) -> Result<Vec<ContractAddress>, Error> {
        let res = self
            .client
            .view_raw::<Vec<ContractAddress>, Error>(
                "issuers",
                OwnedParameter::empty(),
                BlockIdentifier::LastFinal,
            )
            .await?;

        Ok(res)
    }

    /// Register a new identity.
    ///
    /// # Arguments
    ///
    /// * `agent` - The account that will be registering the identity.
    /// * `address` - The address of the identity to be registered.
    /// * `verification_response` - Verification Presentation from Users wallet.
    /// * `energy` - The amount of energy to use for the transaction.
    ///
    /// # Returns
    ///
    /// * A Result containing the `TransactionHash` of the transaction, or an
    ///   `Error`.
    pub async fn register_identity(
        &mut self,
        agent: &WalletAccount,
        address: Address,
        verification_response: VerifyPresentationResponse,
        energy: Energy,
    ) -> Result<TransactionHash, Error> {
        let AccountInfo { account_nonce, .. } = self
            .client
            .client
            .get_account_info(&agent.address.into(), BlockIdentifier::LastFinal)
            .await
            .map_err(Error::AccountInfoQuery)?
            .response;
        let register_identity_payload = RegisterIdentityParams {
            address,
            identity: Identity {
                attributes:  verification_response
                    .revealed_attributes
                    .iter()
                    .map::<Result<IdentityAttribute, _>, _>(|(tag, value)| {
                        Ok(IdentityAttribute {
                            tag:   tag.0,
                            value: value.to_string(),
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?,
                credentials: verification_response
                    .credentials
                    .iter()
                    .map(|(contract_address, key)| IdentityCredential {
                        issuer: contract_address.to_owned(),
                        key:    PublicKeyEd25519(key.public_key.to_bytes()),
                    })
                    .collect(),
            },
        };
        let txn = self
            .client
            .update::<_, Error>(
                &agent.keys,
                &ContractTransactionMetadata {
                    nonce:          account_nonce,
                    expiry:         default_expiry_time(),
                    sender_address: agent.address,
                    amount:         Amount::zero(),
                    energy:         GivenEnergy::Absolute(energy),
                },
                "registerIdentity",
                &register_identity_payload,
            )
            .await?;
        Ok(txn)
    }
}

fn default_expiry_time() -> TransactionTime {
    TransactionTime::from_seconds((Utc::now().timestamp() + 300) as u64)
}
