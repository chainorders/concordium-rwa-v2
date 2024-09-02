use concordium_cis2::{IsTokenAmount, IsTokenId, Receiver};
use concordium_std::{AccountAddress, Address, ContractAddress, SchemaType, Serialize};

#[derive(Serialize, SchemaType)]
pub struct MintParam<A: IsTokenAmount> {
    pub address: AccountAddress,
    pub amount:  A,
}

#[derive(Serialize, SchemaType)]
pub struct MintParams<T: IsTokenId, A: IsTokenAmount> {
    pub token_id: T,
    pub owners:   Vec<MintParam<A>>,
}

#[derive(Serialize, SchemaType)]
pub struct PauseParam<T: IsTokenId> {
    pub token_id: T,
}

#[derive(Serialize, SchemaType)]
pub struct PauseParams<T: IsTokenId> {
    pub tokens: Vec<PauseParam<T>>,
}

#[derive(Serialize, SchemaType, PartialEq, Debug)]
pub struct IsPausedResponse {
    pub tokens: Vec<bool>,
}

#[derive(Debug, Serialize, Clone, SchemaType)]
#[concordium(transparent)]
pub struct BurnParams<T: IsTokenId, A: IsTokenAmount>(
    #[concordium(size_length = 2)] pub Vec<Burn<T, A>>,
);

#[derive(Debug, Serialize, Clone, SchemaType)]
pub struct Burn<T: IsTokenId, A: IsTokenAmount> {
    pub token_id: T,
    pub amount:   A,
    pub owner:    Address,
}

#[derive(Serialize, SchemaType)]
pub struct FreezeParam<T: IsTokenId, A: IsTokenAmount> {
    pub token_id:     T,
    pub token_amount: A,
}

#[derive(Serialize, SchemaType)]
pub struct FreezeParams<T: IsTokenId, A: IsTokenAmount> {
    pub owner:  Address,
    pub tokens: Vec<FreezeParam<T, A>>,
}

#[derive(Serialize, SchemaType)]
pub struct RecoverParam {
    pub lost_account: Address,
    pub new_account:  Address,
}

pub type Agent = Address;

#[derive(Serialize, SchemaType, Clone)]
pub struct AgentWithRoles<TAgentRole> {
    pub address: Address,
    pub roles:   Vec<TAgentRole>,
}

#[derive(Serialize, SchemaType, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TokenUId<T: IsTokenId> {
    pub contract: ContractAddress,
    pub id:       T,
}

impl<T: IsTokenId> TokenUId<T> {
    pub fn new(id: T, contract: ContractAddress) -> Self { Self { contract, id } }
}

impl<T: Clone+IsTokenId> TokenUId<T> {
    pub fn to_token_owner_uid(&self, owner: Receiver) -> TokenOwnerUId<T> {
        TokenOwnerUId::new(self.clone(), owner)
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct TokenOwnerUId<T: IsTokenId> {
    pub token_id: TokenUId<T>,
    pub owner:    Receiver,
}

impl<T: IsTokenId> TokenOwnerUId<T> {
    pub fn new(token_id: TokenUId<T>, owner: Receiver) -> Self { Self { token_id, owner } }
}

impl<T: Eq+IsTokenId> TokenOwnerUId<T> {
    pub fn matches_token(&self, token_id: &TokenUId<T>) -> bool { self.token_id.eq(token_id) }
}
