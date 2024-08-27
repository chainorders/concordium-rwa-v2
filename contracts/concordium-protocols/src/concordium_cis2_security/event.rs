use concordium_cis2::{Cis2Event, IsTokenAmount, IsTokenId, TokenAmountU64, TokenIdVec};
use concordium_std::{
    schema::SchemaType, AccountAddress, Address, ContractAddress, Cursor, SchemaType, Serialize,
};

use super::TokenUId;

/// Represents an event that is triggered when an agent is updated (Added /
/// Removed).
#[derive(Serialize, SchemaType, Debug)]
pub struct AgentUpdatedEvent<R> {
    pub agent: Address,
    pub roles: Vec<R>,
}

/// Represents the event when tokens are frozen / un frozen.
#[derive(Serialize, SchemaType, Debug)]
pub struct TokenFrozen<T: IsTokenId + SchemaType, A: IsTokenAmount + SchemaType> {
    pub token_id: T,
    pub amount:   A,
    pub address:  Address,
}

/// Represents the event when a token is paused / unpaused.
#[derive(Serialize, SchemaType, Debug)]
pub struct Paused<T: IsTokenId + SchemaType> {
    pub token_id: T,
}

/// Represents the event when an identity registry is added.
#[derive(Serialize, SchemaType, Debug)]
#[concordium(transparent)]
pub struct IdentityRegistryAdded(pub ContractAddress);

/// Represents the event when compliance is added.
#[derive(Serialize, SchemaType, Debug)]
#[concordium(transparent)]
pub struct ComplianceAdded(pub ContractAddress);

/// Represents an event for recovering a lost account.
#[derive(Serialize, SchemaType, Debug)]
pub struct RecoverEvent {
    /// The address of the lost account.
    pub lost_account: Address,
    /// The address of the new account.
    pub new_account:  Address,
}

#[derive(Serialize, SchemaType, Debug)]
pub struct TokenDeposited {
    pub token_id: TokenUId<TokenIdVec>,
    pub owner:    AccountAddress,
    pub amount:   TokenAmountU64,
}

#[derive(Serialize, SchemaType, Debug)]
#[concordium(repr(u8))]
pub enum Cis2SecurityEvent<T, A, R>
where
    T: IsTokenId,
    A: IsTokenAmount, {
    /// Event triggered when an account is recovered.
    #[concordium(tag = 242)]
    Recovered(RecoverEvent),

    /// Event triggered when an identity registry is added.
    #[concordium(tag = 243)]
    IdentityRegistryAdded(IdentityRegistryAdded),

    /// Event triggered when compliance is added.
    #[concordium(tag = 244)]
    ComplianceAdded(ComplianceAdded),

    /// Event triggered when a token is unpaused.
    #[concordium(tag = 245)]
    UnPaused(Paused<T>),

    /// Event triggered when a token is paused.
    #[concordium(tag = 246)]
    Paused(Paused<T>),

    /// Event triggered when tokens are frozen.
    #[concordium(tag = 247)]
    TokenFrozen(TokenFrozen<T, A>),

    /// Event triggered when tokens are unfrozen.
    #[concordium(tag = 248)]
    TokenUnFrozen(TokenFrozen<T, A>),

    /// Event triggered when an agent is removed.
    #[concordium(tag = 249)]
    AgentRemoved(AgentUpdatedEvent<R>),

    /// Event triggered when an agent is added.
    #[concordium(tag = 250)]
    AgentAdded(AgentUpdatedEvent<R>),

    /// Event forwarded from the CIS2 contract.
    #[concordium(forward = cis2_events)]
    Cis2(Cis2Event<T, A>),
}
